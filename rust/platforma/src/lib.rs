//! Ce e comun tuturor uneltelor din GMotion.
//!
//! Regula de impartire: ce foloseste mai mult de un modul sta aici; ce e
//! specific unei unelte sta in crate-ul uneltei. Niciun modul nu depinde de alt
//! modul — daca are nevoie de ceva din altul, acel ceva apartine platformei.
//!
//! Modulele, fiecare cu categoria lui de bug:
//!
//! - [`eroare`] — coduri stabile `GM-PLAT-*`;
//! - [`cai`] — unde stau datele pe disc;
//! - [`protejare`] — DPAPI pe Windows, marcaj clar in dezvoltare;
//! - [`sesiune`] — progres salvat, cu spatiu de nume per modul;
//! - [`configurare`] — ce se schimba fara recompilare;
//! - [`jurnal`] — tinte per crate, plus tampon in memorie;
//! - [`modul`] — contractul de modul, registrul si rutarea.
//!
//! Contractul complet e in `CRATE.md`.

pub mod cai;
pub mod configurare;
pub mod eroare;
pub mod jurnal;
pub mod modul;
pub mod protejare;
pub mod sesiune;
pub mod timp;

pub use configurare::Configurare;
pub use eroare::{valideaza_identificator, Eroare, Rezultat};
pub use modul::{CerinteBaze, Ecran, Modul, Registru, Ruta, METODE_PLATFORMA};
pub use sesiune::{slug, Sesiune, Sesiuni, VERSIUNE_SESIUNE};
