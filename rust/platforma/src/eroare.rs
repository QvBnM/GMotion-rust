//! Erorile platformei, cu coduri stabile `GM-PLAT-*`.
//!
//! **Se strica astfel:** acelasi lucru ca la `db` — un raport de bug fara
//! nimic de care sa te prinzi. Registrul de coduri e in `JURNAL.md`; un cod
//! alocat nu se refoloseste.

pub type Rezultat<T> = Result<T, Eroare>;

#[derive(Debug, thiserror::Error)]
pub enum Eroare {
    #[error("[GM-PLAT-001] nu gasesc un director de date pentru utilizatorul curent")]
    DirectorDate,

    #[error("[GM-PLAT-002] nu pot scrie sesiunea in {cale}: {detaliu}")]
    ScriereSesiune { cale: String, detaliu: String },

    #[error("[GM-PLAT-003] nu pot citi sesiunea din {cale}: {detaliu}")]
    CitireSesiune { cale: String, detaliu: String },

    #[error("[GM-PLAT-004] sesiunea din {cale} nu poate fi decriptata. A fost salvata de alt utilizator sau pe alt calculator")]
    DecriptareSesiune { cale: String },

    #[error("[GM-PLAT-005] sesiunea din {cale} are versiunea {gasita}, dar aceasta versiune de GMotion citeste {asteptata}")]
    VersiuneSesiune {
        cale: String,
        gasita: u32,
        asteptata: u32,
    },

    #[error("[GM-PLAT-006] identificator invalid: {0:?}. Sunt permise doar litere mici, cifre si cratima")]
    IdentificatorInvalid(String),

    #[error("[GM-PLAT-007] modulul {0:?} nu e inregistrat")]
    ModulNecunoscut(String),

    #[error("[GM-PLAT-008] modulul {0:?} e deja inregistrat")]
    ModulDuplicat(String),

    #[error("[GM-PLAT-009] ruta {0:?} nu are forma <modul>/<metoda>")]
    RutaInvalida(String),

    #[error("[GM-PLAT-010] modulul {modul} cere {cerinta}, dar a primit {primit}")]
    CerinteNeindeplinite {
        modul: String,
        cerinta: String,
        primit: String,
    },

    #[error("[GM-PLAT-011] configurarea din {cale} nu poate fi citita: {detaliu}")]
    Configurare { cale: String, detaliu: String },
}

impl Eroare {
    pub fn cod(&self) -> &'static str {
        match self {
            Eroare::DirectorDate => "GM-PLAT-001",
            Eroare::ScriereSesiune { .. } => "GM-PLAT-002",
            Eroare::CitireSesiune { .. } => "GM-PLAT-003",
            Eroare::DecriptareSesiune { .. } => "GM-PLAT-004",
            Eroare::VersiuneSesiune { .. } => "GM-PLAT-005",
            Eroare::IdentificatorInvalid(_) => "GM-PLAT-006",
            Eroare::ModulNecunoscut(_) => "GM-PLAT-007",
            Eroare::ModulDuplicat(_) => "GM-PLAT-008",
            Eroare::RutaInvalida(_) => "GM-PLAT-009",
            Eroare::CerinteNeindeplinite { .. } => "GM-PLAT-010",
            Eroare::Configurare { .. } => "GM-PLAT-011",
        }
    }
}

/// Identificator de modul sau de ecran: litere mici, cifre, cratima.
///
/// Strict pe nedrept? Nu: identificatorul devine **nume de director** pentru
/// sesiuni si **prefix de ruta**. Un `..` sau un `/` acolo ar scrie in afara
/// directorului de date.
pub fn valideaza_identificator(s: &str) -> Rezultat<()> {
    if s.is_empty()
        || s.len() > 64
        || !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(Eroare::IdentificatorInvalid(s.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn identificatori_valizi() {
        for s in ["diff-obiecte", "upgrade", "stoc2", "a"] {
            assert!(valideaza_identificator(s).is_ok(), "{s}");
        }
    }

    #[test]
    fn identificatori_care_ar_scrie_in_afara_directorului() {
        for s in [
            "",
            "..",
            "../x",
            "a/b",
            "a\\b",
            "Diff",
            "cu spatiu",
            "a_b",
            "a.b",
        ] {
            assert!(
                valideaza_identificator(s).is_err(),
                "{s:?} ar trebui refuzat"
            );
        }
    }

    #[test]
    fn coduri_unice_si_prezente_in_mesaj() {
        let toate = [
            Eroare::DirectorDate,
            Eroare::ScriereSesiune {
                cale: "c".into(),
                detaliu: "d".into(),
            },
            Eroare::CitireSesiune {
                cale: "c".into(),
                detaliu: "d".into(),
            },
            Eroare::DecriptareSesiune { cale: "c".into() },
            Eroare::VersiuneSesiune {
                cale: "c".into(),
                gasita: 1,
                asteptata: 2,
            },
            Eroare::IdentificatorInvalid("i".into()),
            Eroare::ModulNecunoscut("m".into()),
            Eroare::ModulDuplicat("m".into()),
            Eroare::RutaInvalida("r".into()),
            Eroare::CerinteNeindeplinite {
                modul: "m".into(),
                cerinta: "c".into(),
                primit: "p".into(),
            },
            Eroare::Configurare {
                cale: "c".into(),
                detaliu: "d".into(),
            },
        ];
        let mut c: Vec<&str> = toate.iter().map(|e| e.cod()).collect();
        c.sort_unstable();
        let n = c.len();
        c.dedup();
        assert_eq!(c.len(), n, "coduri duplicate");
        for e in &toate {
            assert!(e.to_string().contains(e.cod()));
        }
    }
}
