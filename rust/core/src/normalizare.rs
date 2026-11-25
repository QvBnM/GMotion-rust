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
// Doar in pozitie de hint: `tabela (NOLOCK)`, `(UPDLOCK, HOLDLOCK)`. Inainte
// se stergea orice cuvant din lista, oriunde: `@nolock` devenea `@`, `#nowait`
// devenea `#`, iar `SET TRANSACTION ISOLATION LEVEL SERIALIZABLE` disparea
// (F-M3 din auditul QA).
static RE_HINTURI: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"(?i)\(\s*(?:{h})(?:\s*,\s*(?:{h}))*\s*\)",
        h = HINTURI
    ))
    .unwrap()
});
// `dbo.` se sterge doar cand e prima parte a numelui: in `Sales.dbo.Orders`
// (baza.schema.tabel) nu e prefix, si stergerea il facea egal cu
// `Sales.Orders` (schema.tabel). Crate-ul `regex` nu are lookbehind, deci
// caracterul dinainte se captureaza si se pune la loc.
static RE_DBO_PARANTEZAT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(^|[^.\w\]])\[dbo\]\.").unwrap());
/// `dbo.` ca schema, nu ca parte dintr-un nume: `@dbo.x` (variabila) si
/// `#dbo.x` raman, altfel `@dbo_x` si `@x` ar fi ajuns identice (auditul
/// 2026-11-22, aceeasi familie ca hinturile din F-M3).
static RE_DBO: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(^|[^.\w\]@#$])dbo\.").unwrap());
static RE_IDENTIFICATOR_SIMPLU: Lazy<Regex> =
    // nu `[@x]`: fara paranteze ar deveni variabila `@x`, alt lucru decat
    // coloana numita asa (auditul 2026-11-23). `[#t]` e tot tabela `#t`.
    Lazy::new(|| Regex::new(r"^\[[\p{L}_#][\p{L}\p{N}_@#$]*\]$").unwrap());
static RE_COLLATE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bCOLLATE\s+database_default\b").unwrap());
static RE_PARANTEZE_DREPTE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\[\]]+)\]").unwrap());
static RE_SPATII: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
/// Un sir de spatii care trece de un capat de rand: reindentarea SQL-ului dinamic.
static RE_SPATII_CU_RAND: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*[\r\n]\s*").unwrap());
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
/// Pastrate: string-urile (`'...'`), cu majusculele lor, dar cu spatiile
/// stranse ca in v3.4; identificatorii in ghilimele, exact.
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
            // in string-uri se strang doar spatiile care trec de un capat de
            // rand: SQL-ul dinamic (`'... \n\t\tJOIN ...'`) difera des doar in
            // indentare (proba pe Windows, 2026-11-19). Pe acelasi rand, un tab
            // fata de un spatiu sau doua spatii fata de unul sunt alte date
            // (Mihai, 2026-11-22; v3.4 le strangea pe toate).
            Fel::Sir => pastreaza(&mut cod, RE_SPATII_CU_RAND.replace_all(t, " ").into_owned()),
            Fel::IdentificatorGhilimele => pastreaza(&mut cod, t.to_string()),
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

/// Hinturile de blocare din cod, in ordine, fara spatii si cu majuscule:
/// `WITH(UPDLOCK,HOLDLOCK);(NOLOCK)`. Normalizarea le sterge (sunt zgomot la
/// comparare), deci doua obiecte care difera **doar** prin ele au acelasi
/// hash; asta le deosebeste, ca sa nu iasa „identice" (F-M3: o reparatie de
/// concurenta nu se deployeaza niciodata). Ce e in siruri si comentarii nu
/// conteaza.
pub fn hinturi(s: &str) -> String {
    let mut cod = String::with_capacity(s.len());
    for seg in crate::lexer::segmente(s) {
        match seg.fel {
            Fel::Cod => cod.push_str(&s[seg.start..seg.sfarsit]),
            _ => cod.push(' '),
        }
    }
    let mut gasite: Vec<(usize, String)> = Vec::new();
    let curat = |t: &str| -> String {
        t.chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .to_uppercase()
    };
    for m in RE_WITH_HINTURI.find_iter(&cod) {
        gasite.push((m.start(), curat(m.as_str())));
    }
    let fara_with = RE_WITH_HINTURI.replace_all(&cod, |c: &regex::Captures| " ".repeat(c[0].len()));
    for m in RE_HINTURI.find_iter(&fara_with) {
        gasite.push((m.start(), curat(m.as_str())));
    }
    gasite.sort();
    gasite
        .into_iter()
        .map(|(_, t)| t)
        .collect::<Vec<_>>()
        .join(";")
}

/// Amprenta comentariilor din cod: o forma scurta si stabila a tot ce e
/// comentariu, in ordine.
///
/// Normalizarea sterge comentariile (sunt zgomot la comparare, ca in v3.4),
/// deci un obiect care difera **doar** intr-un comentariu are acelasi hash si
/// nu ajunge niciodata in lista. Mecanismul e acelasi ca la [`hinturi`]: o
/// amprenta separata pentru ce arunca normalizarea, ca sa ramana vizibil fara sa
/// se schimbe verdictul „identic" pe cod (decizia e in `JURNAL.md`, etapa 30).
///
/// Spatiile din interiorul comentariului se strang (reindentarea nu e o
/// schimbare de comentariu), dar majusculele ramane: comentariul e text pentru
/// om. Gol cand nu exista niciun comentariu cu text.
pub fn amprenta_comentarii(cod: &str) -> String {
    use crate::lexer::Fel;
    let mut bucati: Vec<String> = Vec::new();
    for seg in crate::lexer::segmente(cod) {
        if !matches!(seg.fel, Fel::ComentariuLinie | Fel::ComentariuBloc) {
            continue;
        }
        let t = RE_SPATII.replace_all(&cod[seg.start..seg.sfarsit], " ");
        let t = t.trim();
        // `--` sau `/* */` fara nimic in ele nu sunt un comentariu schimbat
        let fara_marci = t
            .trim_start_matches(['-', '/', '*'])
            .trim_end_matches(['*', '/'])
            .trim();
        if fara_marci.is_empty() {
            continue;
        }
        bucati.push(t.to_string());
    }
    if bucati.is_empty() {
        return String::new();
    }
    let mut h = Sha256::new();
    h.update(bucati.join("\n").as_bytes());
    hex::encode(h.finalize())[..32].to_string()
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

/// Rotunjire identica cu `round()` din Python: pe valoarea zecimala exacta,
/// la egalitate spre par.
///
/// `Math.round` din JS si `f64::round` din Rust rotunjesc 0.5 in sus, deci
/// 98.125 ar da 98.13, in timp ce Python da 98.12. Scorurile ajung in interfata
/// si in CSV, deci diferenta ar fi vizibila.
pub fn round_py(x: f64, zecimale: u32) -> f64 {
    // `format!` rotunjeste valoarea zecimala **exacta** a lui `x`, la
    // egalitate spre par: exact ce face `round` din Python. Varianta pe
    // `x * 10^n` gresea cand inmultirea iese din reprezentare (0.005 -> 0,
    // Python 0.01; 2.675 -> 2.68, Python 2.67; auditul 2026-11-22).
    if !x.is_finite() {
        return x;
    }
    format!("{:.*}", zecimale as usize, x).parse().unwrap_or(x)
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

/// Marginea de sus a scorului, calculata **numai** din lungimi.
///
/// `ratio = 2 * potriviri / (n + m)`, iar `potriviri <= min(n, m)`, deci
/// `scor <= 200 * min(n, m) / (n + m)`. Marginea e exacta, nu euristica: o
/// pereche a carei margine e sub prag **nu poate** trece pragul, deci nu merita
/// nicio comparatie. Pe matricile `n*m` de scoruri din `diff` si `aliniere`
/// asta sare peste cele mai multe perechi fara sa schimbe niciun rezultat.
pub fn scor_maxim_posibil(n: usize, m: usize) -> f64 {
    if n == 0 && m == 0 {
        return 100.0;
    }
    200.0 * n.min(m) as f64 / (n + m) as f64
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
mod teste_b11 {
    use super::*;

    /// `unifica_linii` e API public si nu avea test: orice fel de terminator
    /// devine `\n`, si nimic altceva nu se atinge.
    #[test]
    fn unifica_linii_face_doar_terminatoarele() {
        assert_eq!(unifica_linii("a\r\nb\rc\nd"), "a\nb\nc\nd");
        assert_eq!(unifica_linii(""), "");
        assert_eq!(unifica_linii("fara terminator"), "fara terminator");
        // un `\r` din interiorul unui sir e tot terminator aici: filtrarea pe
        // siruri e treaba lui `pregateste_afisare`, nu a acestei functii
        assert_eq!(unifica_linii("SELECT 'a\r\nb'"), "SELECT 'a\nb'");
        assert_eq!(unifica_linii("\r\n\r\n"), "\n\n");
    }

    /// `calc_hash` e API public (il folosesc `app`, `modul` si `cli`) si nu
    /// avea test propriu: acelasi text da acelasi hash, texte diferite dau
    /// hash-uri diferite, si rezultatul e stabil intre rulari.
    #[test]
    fn calc_hash_e_stabil_si_deosebeste() {
        let a = calc_hash("create procedure p as select 1");
        assert_eq!(a, calc_hash("create procedure p as select 1"));
        assert_ne!(a, calc_hash("create procedure p as select 2"));
        assert_ne!(calc_hash(""), calc_hash(" "));
        assert!(!a.is_empty());
        // nu depinde de lungime: doua texte lungi care difera la capat
        let lung = "x".repeat(10_000);
        assert_ne!(calc_hash(&lung), calc_hash(&(lung.clone() + "y")));
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn round_py_rotunjeste_ca_python() {
        for (x, n, asteptat) in [
            (0.005, 2, 0.01),
            (2.675, 2, 2.67),
            (0.125, 2, 0.12),
            (0.375, 2, 0.38),
            (33.335, 2, 33.34),
            (0.5, 0, 0.0),
            (1.5, 0, 2.0),
            (2.5, 0, 2.0),
            (87.5, 2, 87.5),
        ] {
            assert_eq!(round_py(x, n), asteptat, "round({x}, {n})");
        }
    }

    #[test]
    fn coloana_cu_arond_nu_e_variabila() {
        assert_ne!(
            normalizeaza("SELECT [@x] FROM t"),
            normalizeaza("SELECT @x FROM t")
        );
        assert_eq!(
            normalizeaza("SELECT * FROM [#t]"),
            normalizeaza("SELECT * FROM #t")
        );
        assert_eq!(
            normalizeaza("SELECT [a] FROM t"),
            normalizeaza("SELECT a FROM t")
        );
    }

    #[test]
    fn dbo_dupa_arond_nu_e_schema() {
        assert_ne!(normalizeaza("SELECT @dbo.x"), normalizeaza("SELECT @x"));
        assert_eq!(
            normalizeaza("SELECT x FROM dbo.t"),
            normalizeaza("SELECT x FROM t")
        );
    }

    /// C2 din `cercetare/audit-qa-2026-11-22.md`: in interiorul unui sir, un
    /// tab fata de un spatiu pe acelasi rand e alta valoare (`REPLACE(x, TAB,
    /// ';')`); reindentarea pe mai multe randuri a SQL-ului dinamic nu e.
    #[test]
    fn in_sir_se_strange_doar_spatiul_care_trece_de_rand() {
        assert_ne!(normalizeaza("SELECT 'a\tb'"), normalizeaza("SELECT 'a b'"));
        assert_ne!(normalizeaza("SELECT 'a  b'"), normalizeaza("SELECT 'a b'"));
        assert_eq!(
            normalizeaza("EXEC('SELECT 1\n\t\tFROM t')"),
            normalizeaza("EXEC('SELECT 1\r\n    FROM t')")
        );
        assert_eq!(
            normalizeaza("SELECT 'a \n b'"),
            normalizeaza("SELECT 'a b'")
        );
    }

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
            // SQL dinamic reindentat: spatiile care trec de rand se strang
            // (pe acelasi rand nu: `in_sir_se_strange_doar_spatiul_care_trece_de_rand`)
            (
                "SET @s = ') W\n\t\tJOIN t X ON W.a=X.a '",
                "SET @s = ') W\n            JOIN t X ON W.a=X.a '",
            ),
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

    /// F-M3: hinturile se scot doar din pozitia de hint. Inainte, `@nolock`
    /// devenea `@`, iar o schimbare de nivel de izolare iesea „identica".
    #[test]
    fn hinturile_se_scot_doar_din_pozitia_de_hint() {
        assert_eq!(
            normalizeaza("SELECT a FROM t WITH (NOLOCK) JOIN u (NOLOCK) ON 1=1"),
            normalizeaza("SELECT a FROM t JOIN u ON 1=1")
        );
        assert!(normalizeaza("DECLARE @nolock INT").contains("@nolock"));
        assert!(normalizeaza("SELECT * INTO #nowait FROM t").contains("#nowait"));
        assert_ne!(
            normalizeaza("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE"),
            normalizeaza("SET TRANSACTION ISOLATION LEVEL")
        );
    }

    #[test]
    fn semnatura_hinturilor_deosebeste_ce_normalizarea_uneste() {
        let a = "SELECT a FROM t WITH (NOLOCK)";
        let b = "SELECT a FROM t WITH (UPDLOCK, HOLDLOCK)";
        assert_eq!(
            normalizeaza(a),
            normalizeaza(b),
            "la comparare raman zgomot"
        );
        assert_eq!(hinturi(a), "WITH(NOLOCK)");
        assert_eq!(hinturi(b), "WITH(UPDLOCK,HOLDLOCK)");
        assert_eq!(
            hinturi("SELECT 1 FROM t (nolock) -- WITH (UPDLOCK)"),
            "(NOLOCK)"
        );
        assert_eq!(
            hinturi("SELECT 'WITH (NOLOCK)'"),
            "",
            "sirurile nu conteaza"
        );
        assert_eq!(hinturi("SELECT a FROM t"), "");
    }

    #[test]
    fn amprenta_comentariilor_prinde_ce_sterge_normalizarea() {
        let a = "SELECT 1 -- vechi\n";
        let b = "SELECT 1 -- nou\n";
        assert_eq!(normalizeaza(a), normalizeaza(b));
        assert_ne!(amprenta_comentarii(a), amprenta_comentarii(b));
        // reindentarea in comentariu nu e o schimbare
        assert_eq!(
            amprenta_comentarii("/* a\n   b */"),
            amprenta_comentarii("/* a\nb */")
        );
        // majuscula in comentariu e o schimbare: e text pentru om
        assert_ne!(amprenta_comentarii("-- Nu"), amprenta_comentarii("-- nu"));
        // fara comentariu, fara amprenta
        assert_eq!(amprenta_comentarii("SELECT 1"), "");
        assert_eq!(amprenta_comentarii("SELECT 1 --\n/**/"), "");
        // un `--` dintr-un sir nu e comentariu
        assert_eq!(amprenta_comentarii("SELECT '-- nu'"), "");
    }

    #[test]
    fn marginea_scorului_din_lungimi() {
        assert_eq!(scor_maxim_posibil(0, 0), 100.0);
        assert_eq!(scor_maxim_posibil(10, 10), 100.0);
        assert_eq!(scor_maxim_posibil(1, 9), 20.0);
        assert_eq!(scor_maxim_posibil(5, 0), 0.0);
        // marginea nu minte niciodata: scorul real nu o depaseste
        for (a, b) in [
            (
                "SELECT 1",
                "SELECT 1 FROM t WHERE x = 1 AND y = 2 AND z = 3",
            ),
            ("a b c d e f", "a"),
            ("", "SELECT 1"),
        ] {
            let (ta, tb) = (tokeni_scor(a), tokeni_scor(b));
            assert!(
                scor_pe_tokeni(&ta, &tb) <= scor_maxim_posibil(ta.len(), tb.len()) + 1e-9,
                "{a:?} / {b:?}"
            );
        }
    }
}
