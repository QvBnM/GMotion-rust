//! Modulul 1: compara obiectele SQL ale mai multor baze fata de o referinta si
//! genereaza scripturi de deploy.
//!
//! Acelasi lucru pe care-l face GMotion v3.4, dar ca **biblioteca**: toata
//! orchestrarea e aici, fara nicio interfata. Interfata (si linia de comanda)
//! doar cheama.
//!
//! Modulele, fiecare cu categoria lui de bug:
//!
//! - [`comparare`] — clasificarea obiectelor. Se strica punand un obiect in
//!   categoria greșita;
//! - [`diferente`] — diff-ul unui obiect, la cerere;
//! - [`deploy`] — generatorul de scripturi. Cea mai scumpa greseala posibila,
//!   fiindca se vede dupa ce a rulat pe o baza;
//! - [`plan`] — ce intra in deploy, cu amprentele de la comparare, si garda
//!   „baza n-a mai fost atinsa". Se strica spunand „neatins" despre un obiect
//!   schimbat intre timp;
//! - [`export`] — CSV pentru Excel;
//! - [`eroare`] — coduri `GM-MOD-*`.
//!
//! Contractul complet e in `CRATE.md`.

pub mod comparare;
pub mod deploy;
pub mod diferente;
pub mod eroare;
pub mod export;
mod modul;
pub mod plan;
pub mod revizuire;

pub use comparare::{
    compara, pregateste, setari_diferite, Baza, Categorie, Comparare, Numaratoare, Progres, Rand,
};
pub use deploy::{aplica_create_or_alter, DeDeployat, ModImpartire, PasRevenire, Raport};
pub use diferente::{detalii_obiect, diferente_obiect, Bucata, Detalii, Diferenta, Fel, Marcaj};
pub use eroare::{Eroare, Rezultat};
pub use export::{scrie_csv, Stare};
pub use modul::ModulDiffObiecte;
pub use plan::{amprenta_exacta, FelSchimbare, Parte, Pas, Plan, Schimbare};
pub use revizuire::{Amprenta, Revizuire};

/// Identificatorul modulului. Devine prefix de ruta si nume de director pentru
/// sesiuni, deci e un singur loc de adevar.
pub const NUME: &str = "diff-obiecte";
