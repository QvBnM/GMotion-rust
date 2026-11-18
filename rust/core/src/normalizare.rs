//! Normalizarea codului SQL inainte de comparare, plus hash-uri si scorul de
//! similaritate.
//!
//! Normalizarea sterge zgomotul care NU trebuie sa conteze la clasificarea
//! "egal / modificat": comentarii, hint-uri de lock, prefixul `dbo.`,
//! `COLLATE database_default`, parantezele drepte si spatiile redundante.
//! Textul normalizat se foloseste doar la comparare; afisarea pastreaza mereu
//! originalul.
//!
//! **Se strica astfel:** doua obiecte identice apar drept „modificate" (sau
//! invers, o diferenta reala e inghitita), si scoruri de similaritate absurde.
//! Simptomul e in lista de obiecte, nu in editor.
//!
//! Regulile se aplica **doar pe cod**: string-urile, identificatorii care nu
//! sunt simpli si comentariile sunt delimitate de `lexer`. Pana la auditul din
//! 2026-11-17 regulile mergeau pe tot textul (ca in v3.4 si in motorul JS), si
//! `'--- x'` sau `'/*'` stergeau cod real, iar `'Activ'` = `'ACTIV'`. Vezi
//! `JURNAL.md`, „Normalizarea nu mai atinge string-urile".

use crate::lexer::Fel;
use once_cell::sync::Lazy;
use regex::Regex;
use sha2::{Digest, Sha256};

/// Hint-urile de lock care se sterg la normalizare.
pub const HINTURI: &str = concat!(
    "NOLOCK|READPAST|ROWLOCK|TABLOCK|TABLOCKX|UPDLOCK|XLOCK|HOLDLOCK|",
    "READUNCOMMITTED|READCOMMITTED|REPEATABLEREAD|SERIALIZABLE|PAGLOCK|NOWAIT"
);

static RE_WITH_HINTURI: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"(?i)\bWITH\s*\(\s*(?:{h})(?:\s*,\s*(?:{h}))*\s*\)",
        h = HINTURI
    ))
    .unwrap()
});
static RE_HINTURI: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!(r"(?i)\b(?:{h})\b", h = HINTURI)).unwrap());
// `dbo.` se sterge doar cand e prima parte a numelui: in `Sales.dbo.Orders`
// (baza.schema.tabel) nu e prefix, si stergerea il facea egal cu
// `Sales.Orders` (schema.tabel). Crate-ul `regex` nu are lookbehind, deci
// caracterul dinainte se captureaza si se pune la loc.
static RE_DBO_PARANTEZAT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(^|[^.\w\]])\[dbo\]\.").unwrap());
static RE_DBO: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(^|[^.\w\]])dbo\.").unwrap());
static RE_IDENTIFICATOR_SIMPLU: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\[[\p{L}_@#][\p{L}\p{N}_@#$]*\]$").unwrap());
static RE_COLLATE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bCOLLATE\s+database_default\b").unwrap());
static RE_PARANTEZE_DREPTE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\[\]]+)\]").unwrap());
static RE_SPATII: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
static RE_SPATIU_ORFAN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+([,;)])").unwrap());

/// Marcajele pentru bucatile care trec neatinse prin regulile de cod. Din zona
/// de uz privat Unicode; aparitiile lor in cod se inlocuiesc inainte, ca un
/// text sa nu poata fabrica un marcaj.
const MARCAJ_START: char = '\u{E000}';
const MARCAJ_SFARSIT: char = '\u{E001}';

/// Normalizare, pe text care are deja terminatoarele de linie unificate.
///
/// Ultimul pas (`\s+([,;)])` -> `$1`) exista pentru ca stergerea hint-urilor
/// sau a lui `COLLATE database_default` poate lasa un spatiu orfan inainte de
/// virgula sau de paranteza inchisa (ex: `char(24) ,`). Fara el, doua blocuri
/// identice semantic ieseau cu normalizari diferite si erau marcate gresit ca
/// `modificat`.
///
/// Pastrate exact: string-urile (`'...'`) si identificatorii in ghilimele.
/// Identificatorii in paranteze: `[simplu]` devine `simplu` ca inainte, iar
/// `[cu spatiu]` isi pastreaza parantezele (doar literele mici), fiindca
/// `[a b]` (o coloana) nu e `a b` (coloana cu alias).
pub fn norm(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let mut cod = String::with_capacity(s.len());
    let mut pastrate: Vec<String> = Vec::new();
    // Marcajul nu e litera, cifra sau spatiu pentru regulile de cod (`\b`,
    // `\s`), deci nu schimba nimic in jurul lui: restaurarea e exacta.
    let mut pastreaza = |cod: &mut String, text: String| {
        cod.push(MARCAJ_START);
        cod.push_str(&pastrate.len().to_string());
        cod.push(MARCAJ_SFARSIT);
        pastrate.push(text);
    };
    for seg in crate::lexer::segmente(s) {
        let t = &s[seg.start..seg.sfarsit];
        match seg.fel {
            Fel::Cod => cod.extend(t.chars().map(|c| match c {
                MARCAJ_START | MARCAJ_SFARSIT => '\u{FFFD}',
                // BOM-ul e spatiu pentru JS si pentru SQL Server; nu e cod.
                '\u{FEFF}' => ' ',
                c => c,
            })),
            Fel::ComentariuLinie | Fel::ComentariuBloc => cod.push(' '),
            Fel::IdentificatorParanteze if RE_IDENTIFICATOR_SIMPLU.is_match(t) => cod.push_str(t),
            Fel::IdentificatorParanteze => pastreaza(&mut cod, t.to_lowercase()),
            Fel::Sir | Fel::IdentificatorGhilimele => pastreaza(&mut cod, t.to_string()),
        }
    }
    let s = RE_WITH_HINTURI.replace_all(&cod, " ");
    let s = RE_HINTURI.replace_all(&s, " ");
    let s = RE_DBO_PARANTEZAT.replace_all(&s, "$1");
    let s = RE_DBO.replace_all(&s, "$1");
    let s = RE_COLLATE.replace_all(&s, "");
    let s = RE_PARANTEZE_DREPTE.replace_all(&s, "$1");
    let s = RE_SPATII.replace_all(&s, " ");
    let s = RE_SPATIU_ORFAN.replace_all(&s, "$1");
    let s = s.trim().to_lowercase();
    if pastrate.is_empty() {
        return s;
    }
    let mut rez = String::with_capacity(s.len());
    let mut rest = s.as_str();
    while let Some(i) = rest.find(MARCAJ_START) {
        let dupa = &rest[i + MARCAJ_START.len_utf8()..];
        let Some(j) = dupa.find(MARCAJ_SFARSIT) else {
            break;
        };
        rez.push_str(&rest[..i]);
        let idx: usize = dupa[..j].parse().unwrap_or(usize::MAX);
        rez.push_str(pastrate.get(idx).map(String::as_str).unwrap_or(""));
        rest = &dupa[j + MARCAJ_SFARSIT.len_utf8()..];
    }
    rez.push_str(rest);
    rez
}

/// Unifica terminatoarele de linie, apoi normalizeaza.
pub fn normalizeaza(cod: &str) -> String {
    if cod.is_empty() {
        return String::new();
    }
    norm(&unifica_linii(cod))
}

/// Terminatoare de linie unificate la `\n`.
pub fn unifica_linii(cod: &str) -> String {
    cod.replace("\r\n", "\n").replace('\r', "\n")
}

/// Textul pregatit pentru afisare: terminatoare unificate, spatii de la capatul
/// liniilor sterse, linii goale de la inceput si de la sfarsit eliminate.
///
/// **Nimic nu se atinge in interiorul unui sir, al unui `[...]` sau al unui
/// `"..."`.** Textul asta nu e doar ce se vede: „Aplica toate" il trimite ca
/// text final in baza. Un `\r\n` sau un spatiu de la capat de linie din
/// interiorul unui sir face parte din datele produse de procedura (SQL
/// dinamic, iesiri cu latime fixa, comparatii cu `CHAR(13)`), deci se
/// pastreaza octet cu octet. Granitele vin din `lexer::segmente`, ca orice
/// regula pe text.
pub fn pregateste_afisare(cod: &str) -> String {
    if cod.is_empty() {
        return String::new();
    }
    // pentru fiecare octet, daca e in interiorul unui sir / identificator
    let mut protejat = vec![false; cod.len()];
    for s in crate::lexer::segmente(cod) {
        if matches!(
            s.fel,
            crate::lexer::Fel::Sir
                | crate::lexer::Fel::IdentificatorParanteze
                | crate::lexer::Fel::IdentificatorGhilimele
        ) {
            protejat[s.start..s.sfarsit]
                .iter_mut()
                .for_each(|p| *p = true);
        }
    }

    // 1. terminatoare unificate, doar in afara sirurilor. Delimitatorii si
    //    spatiile sunt ASCII, deci lucrul pe octeti e sigur: un octet de
    //    continuare UTF-8 e >= 0x80.
    let b = cod.as_bytes();
    let mut text: Vec<(u8, bool)> = Vec::with_capacity(b.len());
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b'\r' && !protejat[i] {
            if i + 1 < b.len() && b[i + 1] == b'\n' {
                i += 1; // \r\n -> \n: \n-ul se adauga la pasul urmator
                continue;
            }
            text.push((b'\n', false));
            i += 1;
            continue;
        }
        text.push((b[i], protejat[i]));
        i += 1;
    }

    // 2. liniile se taie doar la `\n`-urile din afara sirurilor: un `\n` din
    //    interiorul unui sir ramane in continutul liniei
    let mut linii: Vec<&[(u8, bool)]> = Vec::new();
    let mut inceput = 0usize;
    for (poz, (c, prot)) in text.iter().enumerate() {
        if *c == b'\n' && !prot {
            linii.push(&text[inceput..poz]);
            inceput = poz + 1;
        }
    }
    linii.push(&text[inceput..]);

    // 3. spatiile de la capatul liniei, tot doar in afara sirurilor
    for l in linii.iter_mut() {
        let mut sfarsit = l.len();
        while sfarsit > 0 {
            let (c, prot) = l[sfarsit - 1];
            if prot || !(c == b' ' || c == b'\t') {
                break;
            }
            sfarsit -= 1;
        }
        *l = &l[..sfarsit];
    }

    // 4. linii goale de la inceput si de la sfarsit; o linie care contine un
    //    sir nu e goala, deci nu se poate pierde nimic din el
    let goala = |l: &&[(u8, bool)]| l.iter().all(|(c, _)| c.is_ascii_whitespace());
    while linii.first().is_some_and(goala) {
        linii.remove(0);
    }
    while linii.last().is_some_and(goala) {
        linii.pop();
    }

    let mut octeti: Vec<u8> = Vec::with_capacity(text.len());
    for (poz, l) in linii.iter().enumerate() {
        if poz > 0 {
            octeti.push(b'\n');
        }
        octeti.extend(l.iter().map(|(c, _)| *c));
    }
    // taierea e pe granite ASCII, deci textul ramane UTF-8 valid
    String::from_utf8(octeti).unwrap_or_else(|_| cod.to_string())
}

/// SHA-256 hex al textului normalizat. Folosit pentru respingerea rapida a
/// obiectelor identice, inainte de orice comparare costisitoare.
pub fn calc_hash(normalizat: &str) -> String {
    let mut h = Sha256::new();
    h.update(normalizat.as_bytes());
    hex::encode(h.finalize())
}

/// Rotunjire identica cu `round()` din Python (half-to-even, "bancara").
///
/// `Math.round` din JS si `f64::round` din Rust rotunjesc 0.5 in sus, deci
/// 98.125 ar da 98.13, in timp ce Python da 98.12. Scorurile ajung in interfata
/// si in CSV, deci diferenta ar fi vizibila.
pub fn round_py(x: f64, zecimale: u32) -> f64 {
    let m = 10f64.powi(zecimale as i32);
    let v = x * m;
    let f = v.floor();
    let diferenta = v - f;
    let r = if (diferenta - 0.5).abs() < 1e-9 {
        // exact la jumatate -> spre numarul par
        if (f as i64) % 2 == 0 {
            f
        } else {
            f + 1.0
        }
    } else {
        v.round()
    };
    r / m
}

/// Token-urile folosite la calculul scorului: forma normalizata, taiata pe
/// spatiu.
///
/// `split(' ')` si nu `split_whitespace()`: pe text gol trebuie sa rezulte un
/// singur token gol, exact ca in motorul existent. Altfel scorul pe obiectele
/// goale ar fi diferit.
///
/// Expusa separat ca sa poata fi calculata o singura data per bloc. Alinierea
/// din `diff` compara fiecare bloc din referinta cu fiecare bloc din tinta,
/// deci renormalizarea la fiecare pereche ar face `n * m` normalizari ale
/// acelorasi texte.
pub fn tokeni_scor(text: &str) -> Vec<String> {
    normalizeaza(text)
        .split(' ')
        .map(|t| t.to_string())
        .collect()
}

/// Procentul de similaritate pe token-uri deja calculate.
pub fn scor_pe_tokeni(a: &[String], b: &[String]) -> f64 {
    let sm = crate::seqmatch::SequenceMatcher::new(a, b);
    round_py(sm.ratio() * 100.0, 2)
}

/// Procentul de similaritate intre doua texte, rotunjit la doua zecimale.
pub fn scor_similaritate(referinta: &str, curent: &str) -> f64 {
    scor_pe_tokeni(&tokeni_scor(referinta), &tokeni_scor(curent))
}

#[cfg(test)]
mod teste {
    use super::*;

    /// C5 din `cercetare/audit-qa-2026-11-18.md`: textul pregatit pentru
    /// afisare e si textul pe care „Aplica toate" trimite in baza. Daca
    /// taierea spatiilor de la capatul liniei intra intr-un sir, deploy-ul
    /// schimba datele pe care le produce procedura — SQL dinamic, iesiri cu
    /// latime fixa, comparatii cu CHAR(13).
    #[test]
    fn afisarea_nu_atinge_interiorul_unui_sir() {
        // spatii la capat de linie **in interiorul** sirului, si \r\n in sir
        let cod = "SELECT 'a   \r\nb'   \r\nFROM t";
        assert_eq!(pregateste_afisare(cod), "SELECT 'a   \r\nb'\nFROM t");
        // acelasi lucru pentru [..] si ".."
        assert_eq!(
            pregateste_afisare("SELECT [a   \r\nb]  \n1"),
            "SELECT [a   \r\nb]\n1"
        );
        assert_eq!(
            pregateste_afisare("SELECT \"a   \r\nb\"  \n1"),
            "SELECT \"a   \r\nb\"\n1"
        );
        // in cod si in comentarii se taie, ca pana acum
        assert_eq!(
            pregateste_afisare("SELECT 1   \r\n-- gata   \r\n"),
            "SELECT 1\n-- gata"
        );
    }

    #[test]
    fn sterge_hinturi_si_dbo() {
        assert_eq!(
            norm("SELECT a FROM [dbo].[t] WITH (NOLOCK) WHERE x = 1"),
            norm("SELECT a FROM t WHERE x = 1")
        );
    }

    #[test]
    fn sterge_collate_database_default() {
        assert_eq!(
            norm("CAST(a AS char(24) ) COLLATE database_default"),
            norm("CAST(a AS char(24))")
        );
    }

    #[test]
    fn spatiu_orfan_inainte_de_virgula() {
        assert_eq!(norm("char(24) , b"), "char(24), b");
    }

    #[test]
    fn comentariile_dispar() {
        assert_eq!(
            norm("SELECT a -- comentariu\nFROM t"),
            norm("SELECT a FROM t")
        );
        assert_eq!(norm("/* bloc\npe linii */SELECT a"), norm("SELECT a"));
    }

    /// Perechi care **difera** si trebuie sa ramana diferite. Toate erau
    /// raportate „identic" inainte de auditul din 2026-11-17 (si in v3.4).
    #[test]
    fn diferentele_din_string_uri_si_identificatori_nu_dispar() {
        let perechi = [
            // comentariu in string: tot codul de dupa disparea
            (
                "PRINT '--- start'; DELETE FROM Clienti",
                "PRINT '--- start'; SELECT 1",
            ),
            ("SET @s = '--' + @a", "SET @s = '--' + @b"),
            (
                "SET @p = '/*'\nDELETE FROM t\nSET @q = '*/'",
                "SET @p = '/*'\nSELECT 1\nSET @q = '*/'",
            ),
            // comentariu de linie care contine /*
            (
                "-- vezi /*\nDELETE FROM Clienti WHERE 1=1\n-- */",
                "-- vezi /*\nDELETE FROM Clienti WHERE id=@id\n-- */",
            ),
            ("SELECT \"a--b\", x FROM t", "SELECT \"a--b\", y FROM u"),
            // continutul literalelor
            ("WHERE s = 'Activ'", "WHERE s = 'ACTIV'"),
            ("SELECT N'Ăla'", "SELECT N'ăla'"), // diacritice-permise: date de test
            ("SELECT 'a  b'", "SELECT 'a b'"),
            ("SELECT 'a ,b'", "SELECT 'a,b'"),
            ("WHERE c LIKE '[0-9]%'", "WHERE c LIKE '0-9%'"),
            ("EXEC('SELECT * FROM dbo.t')", "EXEC('SELECT * FROM t')"),
            ("PRINT 'NOLOCK'", "PRINT ' '"),
            // identificatori
            (
                "SELECT * FROM Sales.dbo.Orders",
                "SELECT * FROM Sales.Orders",
            ),
            ("SELECT [a b] FROM t", "SELECT a b FROM t"),
            ("SELECT [a]]b] FROM t", "SELECT a]b] FROM t"),
        ];
        for (a, b) in perechi {
            assert_ne!(norm(a), norm(b), "ar trebui sa difere:\n  {a:?}\n  {b:?}");
        }
    }

    /// Cosmeticele raman cosmetice: reparatia nu are voie sa faca „modificat"
    /// ce era corect „identic".
    #[test]
    fn cosmeticele_raman_identice() {
        let perechi = [
            ("SELECT  a\n  FROM t", "select a from t"),
            ("SELECT a -- x\nFROM t", "SELECT a /* y */ FROM t"),
            ("/* a /* b */ c */ SELECT 2", "SELECT 2"),
            ("SELECT [a] FROM [dbo].[t]", "SELECT a FROM t"),
            ("SELECT a FROM dbo.t WITH (NOLOCK)", "SELECT a FROM t"),
            ("SELECT x COLLATE database_default , y", "SELECT x, y"),
            ("\u{FEFF}SELECT 1", "SELECT 1"),
            ("WHERE s = 'Activ'", "where S = 'Activ'"),
            ("SELECT [Coloana_1] FROM t", "SELECT coloana_1 FROM t"),
        ];
        for (a, b) in perechi {
            assert_eq!(
                norm(a),
                norm(b),
                "ar trebui sa fie egale:\n  {a:?}\n  {b:?}"
            );
        }
    }

    #[test]
    fn norm_e_idempotenta_si_pe_string_uri_si_paranteze() {
        for t in [
            "[[a]]",
            "[a]]b]",
            "SELECT 'x -- y', [a b] FROM \"q\"",
            "'neinchis -- x",
            "a \u{E000}0\u{E001} b",
        ] {
            let o = norm(t);
            assert_eq!(norm(&o), o, "{t:?}");
        }
    }

    #[test]
    fn marcajele_din_text_nu_pot_fabrica_un_string() {
        // Textul contine chiar caracterele folosite ca marcaj.
        assert_ne!(
            norm("SELECT \u{E000}0\u{E001}, 'x'"),
            norm("SELECT 'x', 'x'")
        );
    }

    #[test]
    fn afisarea_taie_doar_la_capete() {
        assert_eq!(pregateste_afisare("\n\n  SELECT 1  \n\n\n"), "  SELECT 1");
        assert_eq!(pregateste_afisare("a\n\nb"), "a\n\nb");
    }

    #[test]
    fn rotunjire_bancara() {
        assert_eq!(round_py(98.125, 2), 98.12);
        assert_eq!(round_py(98.135, 2), 98.14);
        assert_eq!(round_py(2.5, 0), 2.0);
        assert_eq!(round_py(3.5, 0), 4.0);
    }

    #[test]
    fn scor_pe_texte_goale() {
        assert_eq!(scor_similaritate("", ""), 100.0);
    }

    #[test]
    fn scor_identice() {
        assert_eq!(
            scor_similaritate("SELECT a FROM t", "SELECT a FROM t"),
            100.0
        );
    }
}
