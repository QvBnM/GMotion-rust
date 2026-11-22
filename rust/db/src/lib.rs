//! Accesul la datele din SQL Server: descoperire instante, listare baze,
//! citire obiecte.
//!
//! Singurul crate cu I/O de retea. Deasupra lui, nimic nu stie daca obiectele
//! vin de pe un server sau dintr-un fisier — asta e rostul lui
//! [`SursaObiecte`], si motivul pentru care restul aplicatiei se poate testa
//! fara SQL Server.
//!
//! Modulele, fiecare cu categoria lui de bug:
//!
//! - [`eroare`] — coduri stabile `GM-DB-*`, ca un raport de bug sa duca direct
//!   la cauza;
//! - [`model`] — tipurile care traverseaza granita, date brute, nenormalizate;
//! - [`sursa`] — contractul [`SursaObiecte`];
//! - [`fixture`] — instantanee din fisier: bug-uri reproductibile;
//! - [`sqlserver`] — baza reala, prin TDS nativ, fara ODBC;
//! - [`descoperire`] — SSRP pe UDP 1434, cu parsarea separata de retea.
//!
//! Contractul complet e in `CRATE.md`.

pub mod descoperire;
pub mod eroare;
pub mod fixture;
pub mod model;
pub mod sqlserver;
pub mod sursa;

pub use descoperire::{cauta_instante, parseaza_raspuns};
pub use eroare::{valideaza_baza, valideaza_nume, Eroare, Rezultat};
pub use fixture::{Instantaneu, SursaFixture, VERSIUNE_INSTANTANEU};
pub use model::{
    Autentificare, Certificat, InfoServer, Instanta, MesajServer, Obiect, Setari, Tinta,
    VerificareServer,
};
pub use sqlserver::SursaSqlServer;
pub use sursa::SursaObiecte;
