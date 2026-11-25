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

use gmotion_core::{
    aliniaza, compune, diff_aliniat, norm, normalizeaza, scor_similaritate, split_statements,
};

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

/// Invariantul 4e: un bloc e **exact** liniile lui din textul original, unite
/// cu `\n`, iar liniile lui sunt crescatoare si nu se repeta intre blocuri.
/// De aici vine faptul ca pozitia unui bloc e corecta prin constructie: pana
/// la etapa 30 se cauta textul blocului inapoi in original, iar blocurile
/// unite (`UNION`, cursor) nu sunt subsir, deci nu erau gasite niciodata (B-1,
/// CRITIC: „Aplica" scria peste alte linii).
#[test]
fn blocul_e_exact_liniile_lui() {
    for seed in 0..SEED_URI {
        let cod = genereaza(seed, false);
        let linii: Vec<&str> = cod.split('\n').collect();
        for uneste_cursor in [false, true] {
            let s = gmotion_core::segmenteaza(&cod, uneste_cursor);
            let mut ultima: Option<usize> = None;
            for (i, b) in s.blocuri.iter().enumerate() {
                let din_linii: Vec<&str> = b.linii.iter().map(|&k| linii[k]).collect();
                assert_eq!(
                    b.text,
                    din_linii.join("\n"),
                    "seed {seed}, uneste_cursor={uneste_cursor}, blocul {i}: textul nu e \
                     liniile {:?}\n--- intrare ---\n{cod}",
                    b.linii
                );
                assert!(
                    b.linii.windows(2).all(|w| w[0] < w[1]),
                    "seed {seed}, blocul {i}: linii in dezordine {:?}",
                    b.linii
                );
                if let (Some(u), Some(&prima)) = (ultima, b.linii.first()) {
                    assert!(
                        u < prima,
                        "seed {seed}, blocul {i}: linia {prima} e deja intr-un bloc de dinainte"
                    );
                }
                ultima = b.linii.last().copied().or(ultima);
            }
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

/// Compunerea textului final (Aplica bloc / toate): fara nimic aplicat,
/// textul e exact cel din dreapta, octet cu octet; cu toate diferentele
/// aplicate, obiectul iese **identic** cu stanga, cum il clasifica lista (forma
/// normalizata egala). Daca prima cade, deploy-ul ar schimba cod pe care
/// nimeni nu l-a atins; daca a doua cade, aplicarea pierde sau inventeaza cod.
///
/// Nu se cere „zero blocuri diferite" la recalculare: liniile dintre blocuri
/// (comentarii, linii goale) raman din dreapta, iar un comentariu lipit de cod
/// muta granitele segmentarii. Gasit de fuzz la seed 14.
#[test]
fn compunerea_pastreaza_dreapta_si_aplica_tot() {
    for seed in 0..SEED_URI {
        let st = genereaza(seed, false);
        let dr = genereaza(seed + 10_000, false);
        let (st, dr) = (
            gmotion_core::pregateste_afisare(&st),
            gmotion_core::pregateste_afisare(&dr),
        );
        let a = aliniaza(&st, &dr);
        assert_eq!(
            compune(&a, &st, &dr, |_| false),
            dr,
            "seed {seed}: nimic aplicat"
        );
        let tot = compune(&a, &st, &dr, |_| true);
        assert_eq!(
            normalizeaza(&tot),
            normalizeaza(&st),
            "seed {seed}: dupa Aplica toate obiectul nu e identic\n--- stanga\n{st}\n--- rezultat\n{tot}"
        );
    }
}

/// Aplicarea prin plasare (`aplica_blocuri`, ce folosesc editorul, tragerea
/// si modul ghidat): fara nimic aplicat da exact dreapta; cu un singur bloc
/// aplicat, oricare, liniile lui din stanga ajung toate in rezultat, iar
/// rezultatul nu pierde linii din dreapta in afara celor inlocuite (nu e mai
/// scurt decat dreapta minus blocul cel mai mare din ea).
#[test]
fn aplicarea_prin_plasare_aduce_blocul_si_nu_strica_restul() {
    for seed in 0..SEED_URI {
        let st = gmotion_core::pregateste_afisare(&genereaza(seed, false));
        let dr = gmotion_core::pregateste_afisare(&genereaza(seed + 10_000, false));
        assert_eq!(
            gmotion_core::aplica_blocuri(&st, &dr, false, &[]),
            dr,
            "seed {seed}: nimic aplicat"
        );
        let a = aliniaza(&st, &dr);
        let cel_mai_mare = a
            .diff
            .iter()
            .filter_map(|d| d.dreapta.as_ref().map(|t| t.split('\n').count()))
            .max()
            .unwrap_or(0);
        for b in a.diferente() {
            let rez = gmotion_core::aplica_blocuri(&st, &dr, false, &[b]);
            // `split`, nu `lines`: un ultim rand gol e tot o linie
            let linii_rez: Vec<&str> = rez.split('\n').collect();
            for l in a.randuri.iter().filter(|r| r.bloc == Some(b)) {
                if let Some(l) = &l.stanga {
                    assert!(
                        linii_rez.contains(&l.text.as_str()),
                        "seed {seed}, bloc {b}: linia {:?} nu a ajuns\n{rez}",
                        l.text
                    );
                }
            }
            assert!(
                linii_rez.len() + cel_mai_mare >= dr.split('\n').count(),
                "seed {seed}, bloc {b}: s-au pierdut linii\n--- dreapta\n{dr}\n--- rezultat\n{rez}"
            );
        }
    }
}
/// Proceduri tipice; `@@` marcheaza un loc unde o instructiune noua e valida
/// fara sa schimbe structura (nu intre `IF` si corpul lui, nu in mijlocul unui
/// `INSERT ... SELECT`). In original linia `@@` dispare.
const PROCEDURI_CU_PUNCTE: &[&str] = &[
    "CREATE PROCEDURE dbo.p1\n(\n@a int\n)\nAS\n@@\nBEGIN\n@@\nSET NOCOUNT ON\n@@\ndeclare @x int = null\n@@\nIF @x = 1\nBEGIN\n@@\nSELECT 1\n@@\nUPDATE t SET c = 1\n@@\nEND\n@@\nELSE\nBEGIN\n@@\nSELECT 2\n@@\nEND\n@@\nWHILE @x < 10\nBEGIN\n@@\nSET @x = @x + 1\n@@\nEND\n@@\nRETURN 0\n@@\nEND",
    "CREATE PROCEDURE dbo.p2\nAS\n@@\nSET NOCOUNT ON\n@@\nBEGIN TRY\n@@\nBEGIN TRAN\n@@\nUPDATE t SET c = 1\n@@\nCOMMIT TRAN\n@@\nEND TRY\n@@\nBEGIN CATCH\n@@\nROLLBACK TRAN\n@@\nTHROW\n@@\nEND CATCH\n@@",
    "CREATE PROCEDURE dbo.p3\nAS\nBEGIN\n@@\n;WITH c AS (\nSELECT 1 AS x\n)\nSELECT * FROM c\n@@\nWAITFOR DELAY '00:00:01'\n@@\neticheta:\n@@\nGOTO eticheta\n@@\nEND",
    "CREATE TRIGGER dbo.tr1 ON dbo.t\nAFTER INSERT\nAS\n@@\nBEGIN\n@@\nIF UPDATE(c)\nBEGIN\n@@\nINSERT INTO log (id)\nSELECT id FROM inserted\n@@\nEND\n@@\nEND",
    "CREATE PROCEDURE dbo.p4\nAS\nBEGIN\n@@\nMERGE dbo.t AS d\nUSING dbo.s AS s ON d.id = s.id\nWHEN MATCHED THEN\nUPDATE SET d.c = s.c\nWHEN NOT MATCHED THEN\nINSERT (id, c) VALUES (s.id, s.c);\n@@\nINSERT INTO #r\nEXEC dbo.altul\n@@\nUPDATE t\nSET @v = c = c + 1\n@@\nEND",
    "CREATE PROCEDURE dbo.p5 @a int\nAS\nBEGIN\n@@\nIF EXISTS (\nSELECT 1 FROM t WHERE id = @a\n)\nBEGIN\n@@\nDELETE FROM t WHERE id = @a\n@@\nEND\n@@\nIF @a = 1\nAND @a > 0\nSELECT 'unu'\n@@\nELSE IF @a = 2\nSELECT 'doi'\n@@\nELSE\nBEGIN\n@@\nSELECT CASE\nWHEN @a > 3 THEN 'mare'\nELSE 'mic'\nEND AS m\n@@\nEND ELSE\nSELECT 'x'\n@@\nIF @a IS NULL RETURN\n@@\nSELECT 3\n@@\nEND",
    "CREATE PROCEDURE dbo.p6\nAS\nBEGIN\n@@\nDECLARE c CURSOR FOR\nSELECT id FROM t\n@@\nOPEN c\n@@\nWHILE @@FETCH_STATUS = 0\nBEGIN\n@@\nPRINT @id\n@@\nEND\n@@\nCLOSE c\n@@\nDEALLOCATE c\n@@\nEND",
];

/// `BEGIN`-uri + `CASE`-uri minus `END`-uri, si paranteze deschise minus
/// inchise, pe textul normalizat.
fn echilibru(t: &str) -> (i64, i64) {
    let n = norm(t);
    let cuvinte: Vec<&str> = n
        .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '@'))
        .collect();
    let numar = |kw: &[&str]| cuvinte.iter().filter(|w| kw.contains(w)).count() as i64;
    let paranteze = n.matches('(').count() as i64 - n.matches(')').count() as i64;
    (numar(&["begin", "case"]) - numar(&["end"]), paranteze)
}

/// Doua versiuni care difera printr-o singura instructiune adaugata: orice
/// diferenta din editor (grup de blocuri), aplicata **singura**, lasa `BEGIN`/`END` si parantezele
/// echilibrate. Pana pe 2026-11-21 `BEGIN` se lipea de antet si `END` de
/// ultima instructiune, iar „Aplica" pe un bloc „modificat" sterge sau
/// dubla un `END`: 452 de aplicari stricate pe acest corpus (vezi
/// `JURNAL.md`, 2026-11-21).
#[test]
fn aplicarea_unui_bloc_nu_strica_structura() {
    let noi = ["SELECT 99 AS nou", "SET @z = 1", "EXEC dbo.altceva @a = 1"];
    let mut stricate: Vec<String> = Vec::new();
    for proc in PROCEDURI_CU_PUNCTE {
        let linii: Vec<&str> = proc.lines().collect();
        let original: String = linii
            .iter()
            .filter(|l| **l != "@@")
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        let puncte: Vec<usize> = (0..linii.len()).filter(|&i| linii[i] == "@@").collect();
        for &p in &puncte {
            for nou in noi {
                let mutat: String = linii
                    .iter()
                    .enumerate()
                    .filter(|(i, l)| *i == p || **l != "@@")
                    .map(|(i, l)| if i == p { nou } else { *l })
                    .collect::<Vec<_>>()
                    .join("\n");
                for (st, dr) in [(&original, &mutat), (&mutat, &original)] {
                    // ca editorul: o diferenta e un grup de blocuri, scris
                    // prin `aplica_blocuri`, unde spune plasarea
                    let a = gmotion_core::aliniaza_cu(st, dr, false);
                    let plasari = gmotion_core::plasari_aliniere(&a, dr, false);
                    for g in gmotion_core::grupuri_diferente(&a.diff, &plasari) {
                        let rez = gmotion_core::aplica_blocuri(st, dr, false, &g);
                        if echilibru(&rez) != echilibru(dr) {
                            let d = g[0];
                            stricate.push(format!(
                                "{nou:?} la linia {p}: grup {g:?}, primul {:?}\n    stanga: {:?}\n    dreapta: {:?}",
                                a.diff[d].tip, a.diff[d].stanga, a.diff[d].dreapta
                            ));
                        }
                    }
                }
            }
        }
    }
    assert!(
        stricate.is_empty(),
        "{} aplicari stricate, primele:\n{}",
        stricate.len(),
        stricate[..stricate.len().min(5)].join("\n")
    );
}

/// A1 din `cercetare/audit-qa-2026-11-22.md`: un bloc fara pereche care
/// seamana cu unul deja imperecheat in Modificata nu se scrie peste el.
#[test]
fn aplicarea_nu_scrie_peste_un_bloc_egal() {
    let cazuri = [
        // doua instructiuni asemanatoare: a doua lipseste
        (
            "CREATE PROC p AS\nSELECT a, b, c FROM T WHERE x = 1\nSELECT a, b, c FROM T WHERE x = 2",
            "CREATE PROC p AS\nSELECT a, b, c FROM T WHERE x = 1",
        ),
        // un IF ... BEGIN ... END adaugat nu ia END-ul procedurii
        (
            "CREATE PROC p AS\nBEGIN\nIF @x = 1\nBEGIN\nSELECT 1\nEND\nSELECT 2\nEND",
            "CREATE PROC p AS\nBEGIN\nSELECT 2\nEND",
        ),
        // o instructiune duplicata
        (
            "CREATE PROC p AS\nSELECT a FROM t WHERE x = 1\nSELECT a FROM t WHERE x = 1",
            "CREATE PROC p AS\nSELECT a FROM t WHERE x = 1",
        ),
        // doua instructiuni inversate
        (
            "CREATE PROC p AS\nSET @y = 2\nUPDATE C SET s = 1 WHERE id = @id2",
            "CREATE PROC p AS\nUPDATE C SET s = 1 WHERE id = @id2\nSET @y = 2",
        ),
    ];
    for (st, dr) in cazuri {
        let a = gmotion_core::aliniaza_cu(st, dr, false);
        let plasari = gmotion_core::plasari_aliniere(&a, dr, false);
        for g in gmotion_core::grupuri_diferente(&a.diff, &plasari) {
            if g.iter()
                .any(|&b| a.diff[b].tip != gmotion_core::TipDiferenta::Sters)
            {
                continue;
            }
            let rez = gmotion_core::aplica_blocuri(st, dr, false, &g);
            for linie in dr.lines() {
                let inainte = dr.lines().filter(|l| *l == linie).count();
                let dupa = rez.lines().filter(|l| *l == linie).count();
                assert!(
                    dupa >= inainte,
                    "grupul {g:?} a sters {linie:?}\n--- dreapta\n{dr}\n--- rezultat\n{rez}"
                );
            }
            assert_ne!(rez, dr, "grupul {g:?} nu schimba nimic\n{dr}");
        }
        assert_eq!(converge(st, dr), Ok(()), "{st:?} / {dr:?}");
    }
}

/// Ca omul: aplica prima diferenta, recompara, iar si iar. Trebuie sa ajunga
/// la Martor (normalizat), fara sa oscileze.
fn converge(st: &str, dr: &str) -> Result<(), String> {
    let mut cur = dr.to_string();
    for _ in 0..60 {
        let a = gmotion_core::aliniaza_cu(st, &cur, false);
        let plasari = gmotion_core::plasari_aliniere(&a, &cur, false);
        let grupuri = gmotion_core::grupuri_diferente(&a.diff, &plasari);
        let Some(g) = grupuri.first() else {
            return if gmotion_core::normalizeaza(&cur) == gmotion_core::normalizeaza(st) {
                Ok(())
            } else {
                Err(format!("fara diferente, dar altul decat martorul:\n{cur}"))
            };
        };
        cur = gmotion_core::aplica_blocuri(st, &cur, false, g);
    }
    Err(format!("nu converge in 60 de pasi:\n{cur}"))
}

/// Aplicarea grup cu grup ajunge la Martor pe perechi generate: inserari,
/// stergeri, modificari si mutari de instructiuni (inainte de reparatia A1,
/// 31% din perechi nu ajungeau).
#[test]
fn aplicarea_grup_cu_grup_ajunge_la_martor() {
    const INSTR: &[&str] = &[
        "SELECT a, b FROM Clienti WHERE id = @id",
        "UPDATE Comenzi SET stare = 'nou' WHERE id = @id",
        "UPDATE Comenzi SET stare = 'vechi' WHERE id = @id2",
        "INSERT INTO #tmp (a, b)\nSELECT a, b FROM Facturi",
        "DELETE FROM #tmp WHERE a = 1",
        "SET @x = @x + 1",
        "EXEC dbo.Altceva @p = 1",
        "EXEC dbo.Altceva @p = 2",
        "IF @x = 1\nBEGIN\n    SELECT 1\nEND",
        "PRINT 'gata'",
        "SELECT CASE\nWHEN a = 1 THEN 'x'\nELSE 'y'\nEND AS m\nFROM t",
        "-- comentariu\nSET @y = 2",
        "RETURN 0",
    ];
    let mut s: u64 = 12345;
    let mut n = |k: usize| {
        s ^= s >> 12;
        s ^= s << 25;
        s ^= s >> 27;
        (s.wrapping_mul(2685821657736338717) % k as u64) as usize
    };
    let impacheteaza = |v: &[&str]| {
        format!(
            "CREATE PROCEDURE dbo.p @id INT\nAS\nBEGIN\n{}\nEND",
            v.join("\n")
        )
    };
    let mut esecuri = Vec::new();
    for _ in 0..600 {
        let lung = 2 + n(8);
        let baza: Vec<&str> = (0..lung).map(|_| INSTR[n(INSTR.len())]).collect();
        let mut alt = baza.clone();
        for _ in 0..1 + n(3) {
            match n(3) {
                0 => {
                    let p = n(alt.len() + 1);
                    alt.insert(p, INSTR[n(INSTR.len())]);
                }
                1 if alt.len() > 1 => {
                    let p = n(alt.len());
                    alt.remove(p);
                }
                _ if alt.len() > 1 => {
                    let p = n(alt.len());
                    let x = alt.remove(p);
                    let q = n(alt.len() + 1);
                    alt.insert(q, x);
                }
                _ => {}
            }
        }
        let (st, dr) = (impacheteaza(&baza), impacheteaza(&alt));
        if let Err(e) = converge(&st, &dr) {
            esecuri.push(format!("--- martor\n{st}\n--- modificata\n{dr}\n--- {e}"));
        }
    }
    assert!(
        esecuri.is_empty(),
        "{} perechi nu ajung la martor, prima:\n{}",
        esecuri.len(),
        esecuri.first().cloned().unwrap_or_default()
    );
}

/// N1 si N3 din `cercetare/audit-qa-2026-11-23.md`: linii goale in mijlocul
/// instructiunilor (inainte de WHERE, dupa UNION ALL, in ciclul unui cursor)
/// nu fac „Aplica" sa piarda sau sa dubleze linii, cu si fara cursorul unit.
#[test]
fn liniile_goale_din_instructiuni_nu_strica_aplicarea() {
    let cazuri = [
        (
            "CREATE PROC p AS\nDELETE FROM Comenzi\n\nWHERE id = @id\nSELECT 9",
            "CREATE PROC p AS\nDELETE FROM Comenzi\nWHERE id = @id2\nSELECT 9",
        ),
        (
            "CREATE PROC p AS\nSELECT a FROM t1\nUNION ALL\n\nSELECT a FROM t2\nSELECT 9",
            "CREATE PROC p AS\nSELECT a FROM t1\nUNION ALL\nSELECT b FROM t2\nSELECT 9",
        ),
        (
            "CREATE PROC p AS\nSELECT a FROM t1 UNION ALL\n\nSELECT a FROM t2\nSELECT 9",
            "CREATE PROC p AS\nSELECT a FROM t1 UNION ALL\nSELECT b FROM t2\nSELECT 9",
        ),
        (
            "CREATE PROC p AS\nDECLARE c CURSOR FOR SELECT id FROM t\n\nOPEN c\nFETCH NEXT FROM c INTO @id\nCLOSE c\nDEALLOCATE c\nSELECT 9",
            "CREATE PROC p AS\nDECLARE c CURSOR FOR SELECT id FROM t2\nOPEN c\nFETCH NEXT FROM c INTO @id\nCLOSE c\nDEALLOCATE c\nSELECT 9",
        ),
    ];
    for uneste in [false, true] {
        for (st, dr) in cazuri {
            let mut cur = dr.to_string();
            for _ in 0..20 {
                let a = gmotion_core::aliniaza_cu(st, &cur, uneste);
                let pl = gmotion_core::plasari_aliniere(&a, &cur, uneste);
                let g = gmotion_core::grupuri_diferente(&a.diff, &pl);
                let Some(g) = g.first() else { break };
                cur = gmotion_core::aplica_blocuri(st, &cur, uneste, g);
            }
            assert_eq!(
                gmotion_core::normalizeaza(&cur),
                gmotion_core::normalizeaza(st),
                "uneste={uneste}\n--- martor\n{st}\n--- rezultat\n{cur}"
            );
            // nicio instructiune nu ramane fara WHERE pe drum
            assert!(!cur.contains("Comenzi\nSELECT"), "{cur}");
        }
    }
}
