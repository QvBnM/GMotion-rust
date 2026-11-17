//! Erorile modulului, cu coduri `GM-MOD-*`.
//!
//! **Se strica astfel:** acelasi lucru ca la celelalte crate-uri — un raport
//! fara nimic de care sa te prinzi. Registrul e in `JURNAL.md`.

pub type Rezultat<T> = Result<T, Eroare>;

#[derive(Debug, thiserror::Error)]
pub enum Eroare {
    #[error("[GM-MOD-001] nu s-a rulat nicio comparare")]
    NicioComparare,

    #[error("[GM-MOD-002] niciun obiect de deployat. Obiectele marcate exista doar in bazele comparate, deci nu exista cod de adus din referinta")]
    NimicDeDeployat,

    #[error("[GM-MOD-003] nu pot scrie in {cale}: {detaliu}")]
    ScriereDeploy { cale: String, detaliu: String },

    #[error("[GM-MOD-004] obiectul cu id {0} nu exista in rezultate")]
    RandNecunoscut(usize),

    #[error("[GM-MOD-005] nu pot exporta in {cale}: {detaliu}")]
    Export { cale: String, detaliu: String },

    #[error("[GM-MOD-006] {0}")]
    Platforma(String),

    #[error("[GM-MOD-007] {0}")]
    Date(String),

    #[error("[GM-MOD-008] numele {nume:?} contine caractere de control (rand nou, tab etc.); intr-un script de deploy ar putea deveni cod executabil, deci nu se scrie")]
    NumeNesigur { nume: String },

    #[error("[GM-MOD-009] definitia lui {obiect} are linia `GO` (liniile {linii}); sqlcmd si SSMS taie batch-ul acolo, chiar si in interiorul unui string, deci restul s-ar executa separat. Obiectul trebuie deployat de mana")]
    GoInDefinitie { obiect: String, linii: String },

    #[error("[GM-MOD-010] directorul {cale} nu e gol; fisierele dintr-o rulare veche s-ar amesteca cu cele noi. Alege un director nou sau gol")]
    DirectorNegol { cale: String },

    #[error("[GM-MOD-011] {obiect} e criptat (WITH ENCRYPTION) intr-una din baze; codul nu se poate citi, deci nu se compara si nu se deployeaza")]
    ObiectCriptat { obiect: String },
}

impl Eroare {
    pub fn cod(&self) -> &'static str {
        match self {
            Eroare::NicioComparare => "GM-MOD-001",
            Eroare::NimicDeDeployat => "GM-MOD-002",
            Eroare::ScriereDeploy { .. } => "GM-MOD-003",
            Eroare::RandNecunoscut(_) => "GM-MOD-004",
            Eroare::Export { .. } => "GM-MOD-005",
            Eroare::Platforma(_) => "GM-MOD-006",
            Eroare::Date(_) => "GM-MOD-007",
            Eroare::NumeNesigur { .. } => "GM-MOD-008",
            Eroare::GoInDefinitie { .. } => "GM-MOD-009",
            Eroare::DirectorNegol { .. } => "GM-MOD-010",
            Eroare::ObiectCriptat { .. } => "GM-MOD-011",
        }
    }
}

impl From<gmotion_db::Eroare> for Eroare {
    fn from(e: gmotion_db::Eroare) -> Self {
        // Codul original se pastreaza in text, ca sa nu se pierda informatia
        // la traversarea granitei.
        Eroare::Date(e.to_string())
    }
}

impl From<gmotion_platforma::Eroare> for Eroare {
    fn from(e: gmotion_platforma::Eroare) -> Self {
        Eroare::Platforma(e.to_string())
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn coduri_unice_si_prezente_in_mesaj() {
        let toate = [
            Eroare::NicioComparare,
            Eroare::NimicDeDeployat,
            Eroare::ScriereDeploy {
                cale: "c".into(),
                detaliu: "d".into(),
            },
            Eroare::RandNecunoscut(1),
            Eroare::Export {
                cale: "c".into(),
                detaliu: "d".into(),
            },
            Eroare::Platforma("p".into()),
            Eroare::Date("d".into()),
            Eroare::NumeNesigur { nume: "n".into() },
            Eroare::GoInDefinitie {
                obiect: "o".into(),
                linii: "1".into(),
            },
            Eroare::DirectorNegol { cale: "c".into() },
            Eroare::ObiectCriptat { obiect: "o".into() },
        ];
        let mut c: Vec<&str> = toate.iter().map(|e| e.cod()).collect();
        c.sort_unstable();
        let n = c.len();
        c.dedup();
        assert_eq!(c.len(), n);
        for e in &toate {
            assert!(e.to_string().contains(e.cod()));
        }
    }

    #[test]
    fn codul_original_nu_se_pierde_la_traversare() {
        let db = gmotion_db::Eroare::BazaLipsa {
            server: "s".into(),
            baza: "b".into(),
        };
        let m: Eroare = db.into();
        assert_eq!(m.cod(), "GM-MOD-007");
        assert!(m.to_string().contains("GM-DB-005"), "codul original: {m}");
    }
}
