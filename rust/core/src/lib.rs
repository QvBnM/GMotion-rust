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
pub mod segmentare;
pub mod seqmatch;

pub use aliniere::{aliniaza, compune, Aliniere, Linie, RandAliniat};
pub use diff::{
    diff_aliniat, rezumat_diff, rezumat_rapid, FelSchimbare, RandDiff, Segment, TipDiferenta,
};
pub use normalizare::{
    calc_hash, norm, normalizeaza, pregateste_afisare, round_py, scor_similaritate,
};
pub use segmentare::{cheie_bloc, linii_start_blocuri, split_statements};

/// Tipurile de obiecte SQL comparate, asa cum apar in `sys.objects.type`.
/// `P` procedura, `V` view, `FN`/`IF`/`TF` functii, `TR` trigger.
pub const TIPURI_OBIECTE: [&str; 6] = ["P", "V", "FN", "IF", "TF", "TR"];

/// Numele tipului, pentru utilizator. Sta aici, langa `TIPURI_OBIECTE`, ca sa
/// fie acelasi text in `cli` si in fereastra (un singur loc, nu doua liste).
/// Textele sunt cele din machete (`cercetare/machete/Main.dc.html`).
pub fn nume_tip(tip: &str) -> &'static str {
    match tip.trim().to_ascii_uppercase().as_str() {
        "P" => "procedura",
        "V" => "view",
        "FN" | "IF" | "TF" => "functie",
        "TR" => "trigger",
        _ => "obiect",
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
    }
}
