//! Configurarea: ce se poate schimba fara recompilare.
//!
//! **Se strica astfel:** o valoare implicita greșita face aplicatia sa pară
//! stricata pe o masina si buna pe alta. De-aia implicitele sunt in cod, nu in
//! fisier: un fisier lipsa trebuie sa dea comportamentul corect, nu o eroare.
//!
//! Un fisier de configurare **stricat** nu e ignorat in silenta: ar insemna sa
//! rulezi cu alte valori decat crezi.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::eroare::{Eroare, Rezultat};

// Fara `Eq`: `prag_similaritate` e f64. `PartialEq` ajunge pentru teste.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Configurare {
    /// Cat se asteapta raspunsurile la descoperirea instantelor, in milisecunde.
    pub timeout_descoperire_ms: u64,
    /// Cat se asteapta deschiderea unei conexiuni, in secunde.
    pub timeout_conectare_s: u64,
    /// Sub ce procent de similaritate un obiect se considera „modificat
    /// substantial". Doar pentru prezentare; nu schimba clasificarea.
    pub prag_similaritate: f64,
    /// Cate obiecte intr-un script de deploy, inainte de a-l imparti.
    pub obiecte_per_script: usize,
    /// Uneste ciclul de cursor la segmentare (comutatorul din interfata).
    pub uneste_cursor: bool,
}

impl Default for Configurare {
    fn default() -> Self {
        Configurare {
            // Versiunea Python folosea 2 s pentru descoperire si 30 s pentru
            // conectare. Se pastreaza, ca sa nu se schimbe comportamentul
            // perceput odata cu rescrierea.
            timeout_descoperire_ms: 2000,
            timeout_conectare_s: 30,
            prag_similaritate: 95.0,
            obiecte_per_script: 10,
            uneste_cursor: false,
        }
    }
}

impl Configurare {
    /// Citeste configurarea. Fisier lipsa -> implicitele, fara eroare.
    /// Fisier stricat -> eroare, ca sa nu ruleze cu alte valori decat crezi.
    pub fn citeste(cale: impl AsRef<Path>) -> Rezultat<Self> {
        let cale = cale.as_ref();
        match std::fs::read_to_string(cale) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!(
                    target: "gmotion::platforma",
                    cale = %cale.display(),
                    "configurare inexistenta, folosesc implicitele"
                );
                Ok(Configurare::default())
            }
            Err(e) => Err(Eroare::Configurare {
                cale: cale.display().to_string(),
                detaliu: e.to_string(),
            }),
            Ok(text) => {
                let c: Configurare =
                    serde_json::from_str(&text).map_err(|e| Eroare::Configurare {
                        cale: cale.display().to_string(),
                        detaliu: e.to_string(),
                    })?;
                c.valideaza(cale)?;
                Ok(c)
            }
        }
    }

    fn valideaza(&self, cale: &Path) -> Rezultat<()> {
        let greseala = if self.timeout_descoperire_ms == 0 || self.timeout_descoperire_ms > 60_000 {
            Some("timeout_descoperire_ms trebuie intre 1 si 60000")
        } else if self.timeout_conectare_s == 0 || self.timeout_conectare_s > 600 {
            Some("timeout_conectare_s trebuie intre 1 si 600")
        } else if !(0.0..=100.0).contains(&self.prag_similaritate) {
            Some("prag_similaritate trebuie intre 0 si 100")
        } else if self.obiecte_per_script == 0 || self.obiecte_per_script > 10_000 {
            Some("obiecte_per_script trebuie intre 1 si 10000")
        } else {
            None
        };
        match greseala {
            None => Ok(()),
            Some(d) => Err(Eroare::Configurare {
                cale: cale.display().to_string(),
                detaliu: d.to_string(),
            }),
        }
    }

    pub fn scrie(&self, cale: impl AsRef<Path>) -> Rezultat<()> {
        let cale = cale.as_ref();
        if let Some(p) = cale.parent() {
            std::fs::create_dir_all(p).map_err(|e| Eroare::Configurare {
                cale: p.display().to_string(),
                detaliu: e.to_string(),
            })?;
        }
        let text = serde_json::to_string_pretty(self).map_err(|e| Eroare::Configurare {
            cale: cale.display().to_string(),
            detaliu: e.to_string(),
        })?;
        std::fs::write(cale, text).map_err(|e| Eroare::Configurare {
            cale: cale.display().to_string(),
            detaliu: e.to_string(),
        })
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    fn dir(et: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("gmotion-cfg-{et}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn fisier_lipsa_da_implicitele() {
        let c = Configurare::citeste("/cale/inexistenta/configurare.json").unwrap();
        assert_eq!(c, Configurare::default());
        assert_eq!(c.timeout_conectare_s, 30);
    }

    #[test]
    fn fisier_stricat_nu_e_ignorat_in_silenta() {
        let d = dir("stricat");
        let cale = d.join("c.json");
        std::fs::write(&cale, "{ nu e json").unwrap();
        assert_eq!(
            Configurare::citeste(&cale).unwrap_err().cod(),
            "GM-PLAT-011"
        );
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn camp_necunoscut_e_eroare_nu_e_ignorat() {
        let d = dir("necunoscut");
        let cale = d.join("c.json");
        // Un camp scris greșit trebuie observat: altfel utilizatorul crede ca a
        // schimbat o setare si nu s-a schimbat nimic.
        std::fs::write(&cale, r#"{"timeout_conectare_sec": 5}"#).unwrap();
        assert!(Configurare::citeste(&cale).is_err());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn valori_absurde_sunt_refuzate() {
        let d = dir("absurd");
        let cale = d.join("c.json");
        for text in [
            r#"{"timeout_conectare_s": 0}"#,
            r#"{"timeout_descoperire_ms": 0}"#,
            r#"{"prag_similaritate": 150.0}"#,
            r#"{"prag_similaritate": -1.0}"#,
            r#"{"obiecte_per_script": 0}"#,
        ] {
            std::fs::write(&cale, text).unwrap();
            assert_eq!(
                Configurare::citeste(&cale).unwrap_err().cod(),
                "GM-PLAT-011",
                "{text} ar trebui refuzat"
            );
        }
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn dus_intors() {
        let d = dir("dus-intors");
        let cale = d.join("c.json");
        let c = Configurare {
            uneste_cursor: true,
            obiecte_per_script: 25,
            ..Configurare::default()
        };
        c.scrie(&cale).unwrap();
        assert_eq!(Configurare::citeste(&cale).unwrap(), c);
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn fisier_partial_completeaza_cu_implicitele() {
        let d = dir("partial");
        let cale = d.join("c.json");
        std::fs::write(&cale, r#"{"obiecte_per_script": 50}"#).unwrap();
        let c = Configurare::citeste(&cale).unwrap();
        assert_eq!(c.obiecte_per_script, 50);
        assert_eq!(c.timeout_conectare_s, 30, "restul rămâne implicit");
        std::fs::remove_dir_all(&d).ok();
    }
}
