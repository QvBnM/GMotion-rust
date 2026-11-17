//! Aplicatia cu interfata: leaga ecranele din `gmotion-fereastra` de motor.
//!
//! **Se strica astfel:** o comanda face altceva decat se vede pe ecran (bifa
//! pusa in editor nu ajunge in lista, obiectul deschis nu mai e cel selectat).
//! De-aia starea si rutarea stau in `rutare`, fara Slint, si se testeaza acolo;
//! binarul doar deseneaza si trimite comenzi.

pub mod rutare;

pub use rutare::{Aplicatie, Comanda, Ecran};
