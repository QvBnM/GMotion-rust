//! Motorul de comparare a obiectelor SQL.
//!
//! Logica pura: fara I/O, fara acces la baza de date, fara nimic specific unei
//! interfete. Asta il face testabil direct si refolosibil in doua tinte - binar
//! nativ (prin PyO3 sau direct, pentru backend) si WebAssembly (pentru editorul
//! live din interfata). Scopul e sa existe UN SINGUR motor, in loc de cele doua
//! implementari paralele de acum (`mostenire/motor.py` si portul JS din `mostenire/ui.py`), care au
//! divergat deja - vezi `referinta/README.md`.
//!
//! Semantica portata e cea a motorului JS, care e varianta corecta si mai nouă.
//!
//! Fiecare modul are antetul lui, cu categoria de bug care ii aparține:
//! `normalizare` (fals „modificat"), `segmentare` (blocuri taiate prost),
//! `diff` (blocuri imperecheate greșit), `seqmatch` (abateri de la `difflib`).
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

pub mod diff;
pub mod lexer;
pub mod normalizare;
pub mod segmentare;
pub mod seqmatch;

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
