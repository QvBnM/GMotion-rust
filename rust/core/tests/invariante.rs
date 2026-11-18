//! Teste de invariante pe intrari generate (fuzz dirijat).
//!
//! Se strica astfel: un caz de T-SQL pe care nimeni nu l-a scris de mana in
//! `referinta/cazuri.json` taie un bloc in mijlocul unui literal sau pierde cod
//! la segmentare. Testul de conformitate acopera 24 de cazuri alese; asta
//! acopera forme pe care nu le-a ales nimeni.
//!
//! Generatorul NU produce SQL aleator: produce exact formele care pun la
//! incercare invariantele — literale cu apostrof escapat care contin cuvinte
//! cheie, comentarii bloc pe mai multe linii, `CASE` imbricate, paranteze
//! adanci, `UNION` la final, cursor `FOR`, `INSERT ... SELECT`, `BEGIN TRAN`.
//! Istoricul proiectului arata ca segmentarea e singura zona care a divergat,
//! deci acolo se investeste.
//!
//! Deterministe: acelasi seed da acelasi text. Un esec se reproduce cu
//! `cargo test --test invariante -- --nocapture` si seed-ul din mesaj.

use gmotion_core::{diff_aliniat, norm, normalizeaza, scor_similaritate, split_statements};

/// Cate intrari se genereaza per test. Bugetul de timp al portii e de ordinul
/// secundelor, nu al minutelor — un fuzz care nu se ruleaza nu prinde nimic.
const SEED_URI: u64 = 400;

/// xorshift64*. Fara dependinta noua: `core` ramane cu amprenta minima, fiindca
/// e crate-ul care trebuie sa compileze si pe wasm.
struct Rng(u64);

impl Rng {
    fn nou(seed: u64) -> Self {
        Rng(seed.wrapping_mul(2685821657736338717).max(1))
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(2685821657736338717)
    }
    fn pana_la(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn sansa(&mut self, din: usize) -> bool {
        self.pana_la(din) == 0
    }
}

/// Fragmente bine formate: literale inchise, comentarii inchise, paranteze si
/// `BEGIN`/`END` echilibrate. Pentru astea se cer toate invariantele.
const FRAGMENTE_CORECTE: &[&str] = &[
    "SELECT a, b FROM Clienti WITH (NOLOCK) WHERE id = @id",
    "SELECT 'INSERT INTO fals SELECT 1' AS capcana",
    "SELECT 'it''s a SELECT inside' AS apostrof_escapat",
    "SELECT '/* nu e comentariu */' AS comentariu_in_literal",
    "SELECT '--nici asta' AS linie_in_literal",
    // Decizia de split se ia per linie, si doar cand linia INCEPE cu un cuvant
    // cheie. Deci fragmentele care conteaza sunt cele in care o linie din
    // interiorul unui literal sau comentariu multilinie incepe cu un cuvant
    // cheie — fara ele, invariantul 4 nu e pus la incercare deloc.
    "/* comentariu bloc\nSELECT 1 din comentariu\nINSERT INTO fals VALUES (1)\n*/",
    "SELECT 'prima linie\nSELECT fals in literal\nINSERT fals in literal' AS literal_multilinie",
    "SELECT a\nFROM (\nSELECT b FROM t\nUNION ALL\nSELECT c FROM u\n) AS sub",
    "SELECT CASE\nWHEN a = 1 THEN 'x'\nELSE 'y'\nEND AS case_multilinie\nFROM t",
    "UPDATE t SET a = (\nSELECT MAX(b)\nFROM u\n)",
    "/* SELECT\nDECLARE @x INT\nEXEC ceva\n*/\nSELECT 1",
    "SELECT CASE WHEN a = 1 THEN CASE WHEN b = 2 THEN 'x' ELSE 'y' END ELSE 'z' END AS imbricat",
    "SELECT (((a + b) * (c - d)) / ((e)))\nFROM t",
    "INSERT INTO #tmp (a, b)\nSELECT a, b FROM Facturi",
    "SELECT a FROM t1\nUNION ALL\nSELECT a FROM t2",
    "DECLARE cur CURSOR FOR\nSELECT id FROM Produse",
    "OPEN cur",
    "FETCH NEXT FROM cur INTO @id",
    "CLOSE cur",
    "DEALLOCATE cur",
    "BEGIN TRAN\nUPDATE Stoc SET cant = 0\nCOMMIT",
    "IF @x = 1\nBEGIN\n    SELECT 1\nEND",
    "WHILE @@FETCH_STATUS = 0\nBEGIN\n    SET @n = @n + 1\nEND",
    "-- comentariu de linie cu SELECT si DECLARE\nSET @y = 2",
    "EXEC dbo.Altceva @p = 1",
    "UPDATE Comenzi SET stare = 'nou' WHERE id IN (SELECT id FROM #tmp)",
    "DELETE FROM #tmp",
    "RETURN",
    "PRINT 'gata'",
    "DECLARE @t TABLE (a INT, b NVARCHAR(50))",
];

/// Fragmente rupte intentionat. Pentru astea se cere doar sa nu dea panica si
/// sa nu se piarda cod: pe un literal neinchis nu are sens sa ceri ca blocul sa
/// se termine in afara literalului.
const FRAGMENTE_RUPTE: &[&str] = &[
    "SELECT 'literal neinchis",
    "/* comentariu bloc neinchis\nSELECT 1",
    "SELECT ((((a",
    "END END END",
    "CASE WHEN",
    "SELECT a FROM t\nUNION ALL",
    "DECLARE cur2 CURSOR FOR",
    "INSERT INTO t (a)",
];

fn genereaza(seed: u64, rupte: bool) -> String {
    let mut r = Rng::nou(seed);
    let n = 1 + r.pana_la(8);
    let mut linii: Vec<String> = Vec::new();
    for _ in 0..n {
        let f = if rupte && r.sansa(4) {
            FRAGMENTE_RUPTE[r.pana_la(FRAGMENTE_RUPTE.len())]
        } else {
            FRAGMENTE_CORECTE[r.pana_la(FRAGMENTE_CORECTE.len())]
        };
        linii.push(f.to_string());
        // linii goale ca separator: exact ce arunca segmentarea
        if r.sansa(3) {
            linii.push(String::new());
        }
        if r.sansa(9) {
            linii.push("   ".to_string());
        }
    }
    linii.join("\n")
}

/// Stare de scanare reimplementata **independent** de `segmentare.rs`. Rostul
/// unui test de invariante e sa nu repete codul testat: daca ar chema aceeasi
/// functie, ar confirma doar ca e consecventa cu sine.
#[derive(Default, Debug, PartialEq, Eq)]
struct Stare {
    in_literal: bool,
    in_comentariu: bool,
    paranteze: i32,
}

fn scaneaza(text: &str) -> Stare {
    let ch: Vec<char> = text.chars().collect();
    let mut s = Stare::default();
    let mut i = 0usize;
    while i < ch.len() {
        if s.in_comentariu {
            if ch[i] == '*' && ch.get(i + 1) == Some(&'/') {
                s.in_comentariu = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if s.in_literal {
            if ch[i] == '\'' {
                if ch.get(i + 1) == Some(&'\'') {
                    i += 2;
                } else {
                    s.in_literal = false;
                    i += 1;
                }
            } else {
                i += 1;
            }
            continue;
        }
        match ch[i] {
            '-' if ch.get(i + 1) == Some(&'-') => {
                while i < ch.len() && ch[i] != '\n' {
                    i += 1;
                }
            }
            '/' if ch.get(i + 1) == Some(&'*') => {
                s.in_comentariu = true;
                i += 2;
            }
            '\'' => {
                s.in_literal = true;
                i += 1;
            }
            '(' => {
                s.paranteze += 1;
                i += 1;
            }
            ')' => {
                s.paranteze -= 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    s
}

/// Invariantul 5 din `rust/core/CRATE.md`: segmentarea nu pierde si nu dubleaza
/// **cod**. Nu se poate cere egalitate pe text brut: segmentarea arunca
/// intentionat liniile goale separatoare, antetul si blocurile care sunt doar
/// comentariu, iar blocurile unite (`UNION`, cursor) se lipesc cu un singur
/// `\n`. Tot ce se pierde normalizeaza la gol, deci egalitatea se cere pe forma
/// normalizata.
#[test]
fn segmentarea_nu_pierde_cod() {
    for seed in 0..SEED_URI {
        let cod = genereaza(seed, false);
        for uneste_cursor in [false, true] {
            let blocuri = split_statements(&cod, uneste_cursor);
            let dupa = norm(&blocuri.join("\n"));
            let inainte = norm(&cod);
            assert_eq!(
                dupa, inainte,
                "seed {seed}, uneste_cursor={uneste_cursor}: cod pierdut sau dublat\n\
                 --- intrare ---\n{cod}\n--- blocuri ---\n{blocuri:#?}"
            );
        }
    }
}

/// Invariantul 4: un bloc nu se termina niciodata in interiorul unui literal,
/// al unui comentariu bloc sau al unei paranteze deschise. Daca s-ar intampla,
/// blocul urmator ar porni cu starea gresita si clasificarea ar fi aiurea —
/// exact clasa de bug care a produs divergentele din v3.4.
#[test]
fn niciun_bloc_nu_se_taie_in_mijlocul_unui_construct() {
    for seed in 0..SEED_URI {
        let cod = genereaza(seed, false);
        for uneste_cursor in [false, true] {
            for (i, bloc) in split_statements(&cod, uneste_cursor).iter().enumerate() {
                let s = scaneaza(bloc);
                assert_eq!(
                    s,
                    Stare::default(),
                    "seed {seed}, uneste_cursor={uneste_cursor}, blocul {i} se termina \
                     cu starea {s:?}\n--- bloc ---\n{bloc}\n--- intrare ---\n{cod}"
                );
            }
        }
    }
}

/// Invariantul 3: `norm` e idempotenta. Daca nu e, doua obiecte identice pot
/// aparea drept „modificate" in functie de cate ori s-a normalizat textul pe
/// drum.
#[test]
fn norm_e_idempotenta() {
    for seed in 0..SEED_URI {
        for rupte in [false, true] {
            let cod = genereaza(seed, rupte);
            let o = norm(&cod);
            assert_eq!(norm(&o), o, "seed {seed}, rupte={rupte}:\n{cod}");
            let p = normalizeaza(&cod);
            assert_eq!(normalizeaza(&p), p, "seed {seed}, rupte={rupte}:\n{cod}");
        }
    }
}

/// Invariantul 2: un obiect comparat cu el insusi da 100. Pare trivial, dar
/// trece prin token-izare, `SequenceMatcher` si rotunjire bancara.
#[test]
fn obiectul_comparat_cu_sine_da_suta() {
    for seed in 0..SEED_URI {
        for rupte in [false, true] {
            let cod = genereaza(seed, rupte);
            if norm(&cod).is_empty() {
                continue;
            }
            let s = scor_similaritate(&cod, &cod);
            assert_eq!(s, 100.0, "seed {seed}, rupte={rupte}: scor {s}\n{cod}");
        }
    }
}

/// Invariantul 6: functiile sunt totale pe intrari UTF-8 valide. Nu intorc
/// `Result`, deci singurul mod in care pot cadea e panica — inclusiv pe SQL
/// rupt, care in practica ajunge in motor (obiecte generate, cod comentat pe
/// jumatate).
#[test]
fn nu_da_panica_pe_intrari_rupte() {
    for seed in 0..SEED_URI {
        let a = genereaza(seed, true);
        let b = genereaza(seed + SEED_URI, true);
        for uneste_cursor in [false, true] {
            let _ = split_statements(&a, uneste_cursor);
        }
        let _ = normalizeaza(&a);
        let _ = scor_similaritate(&a, &b);
        let _ = diff_aliniat(&a, &b);
        let _ = gmotion_core::rezumat_rapid(&a, &b);
        let _ = gmotion_core::rezumat_diff(&diff_aliniat(&a, &b));
    }
}

/// Intrari degenerate, scrise de mana: tot ce nu produce generatorul.
#[test]
fn intrari_degenerate() {
    for cod in [
        "",
        "\n",
        "\n\n\n",
        "   ",
        "\t\t",
        "'",
        "/*",
        "--",
        "\r\n\r\n",
        "SELECT 1\r\nSELECT 2",
        "ă î â ș ț",                  // diacritice-permise: date de test
        "SELECT N'diacritice ăîâșț'", // diacritice-permise: date de test
    ] {
        for uneste_cursor in [false, true] {
            let blocuri = split_statements(cod, uneste_cursor);
            assert_eq!(
                norm(&blocuri.join("\n")),
                norm(cod),
                "cod pierdut pe {cod:?}, uneste_cursor={uneste_cursor}"
            );
        }
        let n = norm(cod);
        assert_eq!(norm(&n), n, "norm neidempotenta pe {cod:?}");
        let _ = diff_aliniat(cod, cod);
    }
}
