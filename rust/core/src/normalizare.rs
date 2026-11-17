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

use once_cell::sync::Lazy;
use regex::Regex;
use sha2::{Digest, Sha256};

/// Hint-urile de lock care se sterg la normalizare.
pub const HINTURI: &str = concat!(
    "NOLOCK|READPAST|ROWLOCK|TABLOCK|TABLOCKX|UPDLOCK|XLOCK|HOLDLOCK|",
    "READUNCOMMITTED|READCOMMITTED|REPEATABLEREAD|SERIALIZABLE|PAGLOCK|NOWAIT"
);

static RE_COMENTARIU_BLOC: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)/\*.*?\*/").unwrap());
static RE_COMENTARIU_LINIE: Lazy<Regex> = Lazy::new(|| Regex::new(r"--[^\n]*").unwrap());
static RE_WITH_HINTURI: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"(?i)\bWITH\s*\(\s*(?:{h})(?:\s*,\s*(?:{h}))*\s*\)",
        h = HINTURI
    ))
    .unwrap()
});
static RE_HINTURI: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!(r"(?i)\b(?:{h})\b", h = HINTURI)).unwrap());
static RE_DBO_PARANTEZAT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\[dbo\]\.").unwrap());
static RE_DBO: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bdbo\.").unwrap());
static RE_COLLATE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bCOLLATE\s+database_default\b").unwrap());
static RE_PARANTEZE_DREPTE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\[\]]+)\]").unwrap());
static RE_SPATII: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
static RE_SPATIU_ORFAN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+([,;)])").unwrap());

/// Normalizare, pe text care are deja terminatoarele de linie unificate.
///
/// Ultimul pas (`\s+([,;)])` -> `$1`) exista pentru ca stergerea hint-urilor
/// sau a lui `COLLATE database_default` poate lasa un spatiu orfan inainte de
/// virgula sau de paranteza inchisa (ex: `char(24) ,`). Fara el, doua blocuri
/// identice semantic ieseau cu normalizari diferite si erau marcate greșit ca
/// `modificat`.
pub fn norm(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let s = RE_COMENTARIU_BLOC.replace_all(s, " ");
    let s = RE_COMENTARIU_LINIE.replace_all(&s, " ");
    let s = RE_WITH_HINTURI.replace_all(&s, " ");
    let s = RE_HINTURI.replace_all(&s, " ");
    let s = RE_DBO_PARANTEZAT.replace_all(&s, "");
    let s = RE_DBO.replace_all(&s, "");
    let s = RE_COLLATE.replace_all(&s, "");
    let s = RE_PARANTEZE_DREPTE.replace_all(&s, "$1");
    let s = RE_SPATII.replace_all(&s, " ");
    let s = RE_SPATIU_ORFAN.replace_all(&s, "$1");
    s.trim().to_lowercase()
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
/// Nu modifica nimic in interior - ce se afiseaza ramane codul utilizatorului.
pub fn pregateste_afisare(cod: &str) -> String {
    let cod = unifica_linii(cod);
    let mut linii: Vec<&str> = cod.split('\n').map(|l| l.trim_end()).collect();
    while linii.first().is_some_and(|l| l.trim().is_empty()) {
        linii.remove(0);
    }
    while linii.last().is_some_and(|l| l.trim().is_empty()) {
        linii.pop();
    }
    linii.join("\n")
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
