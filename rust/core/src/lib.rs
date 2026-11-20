//! Motorul de comparare a obiectelor SQL.
//!
//! Logica pura: fara I/O, fara acces la baza de date, fara nimic specific unei
//! interfete. Asta il face testabil direct si refolosibil in doua tinte - binar
//! nativ (prin PyO3 sau direct, pentru backend) si WebAssembly (pentru editorul
//! live din interfata). Scopul e sa existe UN SINGUR motor, in loc de cele doua
//! implementari paralele de acum (`mostenire/motor.py` si portul JS din `mostenire/ui.py`), care au
//! divergat deja - vezi `referinta/README.md`.
//!
//! Semantica portata e cea a motorului JS, care e varianta corecta si mai noua.
//!
//! Fiecare modul are antetul lui, cu categoria de bug care ii apartine:
//! `normalizare` (fals „modificat"), `segmentare` (blocuri taiate prost),
//! `diff` (blocuri imperecheate gresit), `aliniere` (linii pierdute la
//! compunerea textului final), `seqmatch` (abateri de la `difflib`).
//! Contractul complet — invariante, erori tipice, depanare — e in `CRATE.md`.
//!
//! ```
//! use gmotion_core::{diff_aliniat, scor_similaritate};
//!
//! let referinta = "SELECT a FROM t WHERE x = 1";
//! let curent    = "SELECT a FROM t WHERE x = 2";
//!
//! assert!(scor_similaritate(referinta, curent) > 80.0);
//! assert_eq!(diff_aliniat(referinta, curent).len(), 1);
//! ```

pub mod aliniere;
pub mod clasificare;
pub mod diff;
pub mod lexer;
pub mod normalizare;
pub mod plasare;
pub mod segmentare;
pub mod seqmatch;

pub use aliniere::{aliniaza, aliniaza_cu, compune, Aliniere, Linie, RandAliniat};
pub use diff::{
    diff_aliniat, diff_aliniat_cu, rezumat_diff, rezumat_rapid, FelSchimbare, RandDiff, Segment,
    TipDiferenta,
};
pub use normalizare::{
    calc_hash, hinturi, norm, normalizeaza, pregateste_afisare, round_py, scor_similaritate,
};
pub use plasare::{sugereaza, Sugestie, Tinta};
pub use segmentare::{are_cursor, cheie_bloc, linii_start_blocuri, split_statements};

/// Tipurile de obiecte SQL comparate, asa cum apar in `sys.objects.type`.
/// `P` procedura, `V` view, `FN`/`IF`/`TF` functii, `TR` trigger.
pub const TIPURI_OBIECTE: [&str; 6] = ["P", "V", "FN", "IF", "TF", "TR"];

/// Limba textelor pentru om. Sta in `core` fiindca si textele facute aici
/// (numele tipurilor, „difera in ...") urmeaza comutatorul din fereastra; fara
/// I/O, deci merge si pe wasm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Limba {
    #[default]
    Romana,
    Engleza,
}

impl Limba {
    pub fn din_nume(nume: &str) -> Option<Limba> {
        match nume.trim().to_lowercase().as_str() {
            "ro" | "romana" | "rom\u{e2}n\u{103}" => Some(Limba::Romana),
            "en" | "engleza" | "english" => Some(Limba::Engleza),
            _ => None,
        }
    }

    /// Numele scurt, cel care se scrie in configurare.
    pub fn nume(self) -> &'static str {
        match self {
            Limba::Romana => "ro",
            Limba::Engleza => "en",
        }
    }

    /// Cum se arata in interfata.
    pub fn eticheta(self) -> &'static str {
        match self {
            Limba::Romana => "Romana",
            Limba::Engleza => "English",
        }
    }

    pub fn toate() -> [Limba; 2] {
        [Limba::Romana, Limba::Engleza]
    }

    /// Textul in limba asta, din perechea data. Pentru textele facute in Rust
    /// (cu numere sau nume in ele); cele fixe ale ferestrei stau in
    /// `fereastra::texte`.
    pub fn alege<'a>(self, ro: &'a str, en: &'a str) -> &'a str {
        match self {
            Limba::Romana => ro,
            Limba::Engleza => en,
        }
    }

    /// „1 obiect" / „4 obiecte" / „21 de obiecte" / „101 obiecte"; in
    /// engleza „1 object" / „4 objects". Formele se dau: `(unu, mai_multe)`
    /// pentru fiecare limba.
    pub fn numar(self, n: usize, ro: (&str, &str), en: (&str, &str)) -> String {
        match self {
            Limba::Romana => {
                let r = n % 100;
                if n == 1 {
                    format!("1 {}", ro.0)
                } else if n < 20 || (1..20).contains(&r) {
                    format!("{n} {}", ro.1)
                } else {
                    format!("{n} de {}", ro.1)
                }
            }
            Limba::Engleza => {
                if n == 1 {
                    format!("1 {}", en.0)
                } else {
                    format!("{n} {}", en.1)
                }
            }
        }
    }
}

/// Numele tipului, pentru utilizator, in romana (`cli`). Sta aici, langa
/// `TIPURI_OBIECTE`, ca sa fie acelasi text in `cli` si in fereastra (un
/// singur loc, nu doua liste). Textele sunt cele din machete.
pub fn nume_tip(tip: &str) -> &'static str {
    nume_tip_in(tip, Limba::Romana)
}

/// Numele tipului, in limba data.
pub fn nume_tip_in(tip: &str, l: Limba) -> &'static str {
    match tip.trim().to_ascii_uppercase().as_str() {
        "P" => l.alege("procedura", "procedure"),
        "V" => "view",
        "FN" | "IF" | "TF" => l.alege("functie", "function"),
        "TR" => "trigger",
        _ => l.alege("obiect", "object"),
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn fiecare_tip_comparat_are_un_nume_pentru_utilizator() {
        for t in TIPURI_OBIECTE {
            assert_ne!(nume_tip(t), "obiect", "tipul {t} n-are nume");
        }
        assert_eq!(nume_tip("P"), "procedura");
        assert_eq!(nume_tip("v"), "view");
        assert_eq!(nume_tip(" TF "), "functie");
        assert_eq!(nume_tip("TR"), "trigger");
        // un tip pe care nu-l comparam: nu da panica, da un nume generic
        assert_eq!(nume_tip("U"), "obiect");
        assert_eq!(nume_tip_in("P", Limba::Engleza), "procedure");
        assert_eq!(nume_tip_in("U", Limba::Engleza), "object");
    }

    #[test]
    fn limba_se_citeste_si_se_scrie_cu_acelasi_nume() {
        for l in Limba::toate() {
            assert_eq!(Limba::din_nume(l.nume()), Some(l));
        }
        assert_eq!(Limba::din_nume("English"), Some(Limba::Engleza));
        assert_eq!(Limba::din_nume("klingon"), None);
        assert_eq!(Limba::Engleza.alege("da", "yes"), "yes");
        let n = |n, l: Limba| l.numar(n, ("obiect", "obiecte"), ("object", "objects"));
        assert_eq!(n(1, Limba::Romana), "1 obiect");
        assert_eq!(n(19, Limba::Romana), "19 obiecte");
        assert_eq!(n(20, Limba::Romana), "20 de obiecte");
        assert_eq!(n(101, Limba::Romana), "101 obiecte");
        assert_eq!(n(120, Limba::Romana), "120 de obiecte");
        assert_eq!(n(0, Limba::Romana), "0 obiecte");
        assert_eq!(n(1, Limba::Engleza), "1 object");
        assert_eq!(n(21, Limba::Engleza), "21 objects");
    }
}
