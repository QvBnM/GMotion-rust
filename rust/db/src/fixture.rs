//! Sursa de obiecte dintr-un fisier: instantanee scrise de pe o baza reala.
//!
//! **Se strica astfel:** un instantaneu nu se incarca, sau se incarca si
//! produce alte rezultate decat baza din care a fost luat. Al doilea caz e cel
//! rau, fiindca duce la depanarea unui bug care nu exista.
//!
//! Rostul acestui fisier e sa faca bug-urile reproductibile. Fluxul:
//! utilizatorul loveste o problema pe o baza de productie, ruleaza
//! `gmotion instantaneu`, trimite JSON-ul; acelasi JSON se reia aici, pe orice
//! masina, fara SQL Server, si devine test de regresie. Fara asta, orice bug de
//! pe un server la care nu avem acces se depaneaza prin „incearca si spune-mi
//! ce vezi".

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::eroare::{Eroare, Rezultat};
use crate::model::Obiect;
use crate::sursa::SursaObiecte;

/// Versiunea formatului. Se incrementeaza doar la o schimbare incompatibila, si
/// atunci se noteaza in `JURNAL.md`.
pub const VERSIUNE_INSTANTANEU: u32 = 1;

/// Un instantaneu: obiectele brute ale mai multor baze, la un moment dat.
///
/// `BTreeMap` si nu `HashMap`: ordinea stabila face diff-ul a doua instantanee
/// citibil, si JSON-ul reproductibil intre rulari.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instantaneu {
    pub versiune: u32,
    /// De pe ce server a fost luat. Informativ — pentru jurnal si rapoarte.
    pub server: String,
    /// ISO 8601, ca text: `db` nu depinde de un crate de date pentru o eticheta.
    pub creat: String,
    /// Note libere: ce bug se urmarea, ce versiune de GMotion.
    #[serde(default)]
    pub nota: String,
    pub baze: BTreeMap<String, Vec<Obiect>>,
}

impl Instantaneu {
    pub fn nou(server: impl Into<String>, creat: impl Into<String>) -> Self {
        Instantaneu {
            versiune: VERSIUNE_INSTANTANEU,
            server: server.into(),
            creat: creat.into(),
            nota: String::new(),
            baze: BTreeMap::new(),
        }
    }

    /// Scrie instantaneul, formatat ca sa fie citibil si diff-abil de om.
    pub fn scrie(&self, cale: impl AsRef<Path>) -> Rezultat<()> {
        let cale = cale.as_ref();
        let text = serde_json::to_string_pretty(self).map_err(|e| Eroare::ScriereInstantaneu {
            cale: cale.display().to_string(),
            detaliu: e.to_string(),
        })?;
        // Intai intr-un fisier temporar, apoi redenumit: o cadere la jumatatea
        // scrierii nu lasa un instantaneu trunchiat in locul celui vechi.
        let temporar = cale.with_extension("tmp-gmotion");
        let eroare = |e: std::io::Error| Eroare::ScriereInstantaneu {
            cale: cale.display().to_string(),
            detaliu: e.to_string(),
        };
        std::fs::write(&temporar, text).map_err(eroare)?;
        std::fs::rename(&temporar, cale).map_err(|e| {
            let _ = std::fs::remove_file(&temporar);
            eroare(e)
        })?;
        tracing::info!(
            target: "gmotion::db",
            cale = %cale.display(),
            baze = self.baze.len(),
            obiecte = self.baze.values().map(Vec::len).sum::<usize>(),
            "instantaneu scris"
        );
        Ok(())
    }

    pub fn citeste(cale: impl AsRef<Path>) -> Rezultat<Self> {
        let cale = cale.as_ref();
        let text = std::fs::read_to_string(cale).map_err(|e| Eroare::CitireInstantaneu {
            cale: cale.display().to_string(),
            detaliu: e.to_string(),
        })?;
        let inst: Instantaneu =
            serde_json::from_str(&text).map_err(|e| Eroare::CitireInstantaneu {
                cale: cale.display().to_string(),
                detaliu: e.to_string(),
            })?;
        if inst.versiune != VERSIUNE_INSTANTANEU {
            return Err(Eroare::VersiuneInstantaneu {
                cale: cale.display().to_string(),
                gasita: inst.versiune,
                asteptata: VERSIUNE_INSTANTANEU,
            });
        }
        inst.valideaza()
            .map_err(|detaliu| Eroare::CitireInstantaneu {
                cale: cale.display().to_string(),
                detaliu,
            })?;
        Ok(inst)
    }

    /// Un instantaneu circula intre oameni, deci continutul lui nu e de
    /// incredere. Pana la audit nu se verifica nimic: o baza numita
    /// `x];DROP DATABASE Prod;--` sau doua obiecte cu acelasi nume (ultimul
    /// castiga, tacut) treceau mai departe.
    fn valideaza(&self) -> Result<(), String> {
        for (baza, obiecte) in &self.baze {
            if crate::eroare::valideaza_baza(baza).is_err() {
                return Err(format!("numele de baza {baza:?} nu e valid"));
            }
            let mut vazute = std::collections::BTreeSet::new();
            for o in obiecte {
                if o.nume.is_empty() || o.schema.is_empty() {
                    return Err(format!("obiect fara nume sau schema in baza {baza:?}"));
                }
                if !vazute.insert(o.id()) {
                    return Err(format!(
                        "obiectul {} apare de doua ori in baza {baza:?}",
                        o.id()
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Sursa de obiecte alimentata dintr-un instantaneu.
pub struct SursaFixture {
    instantaneu: Instantaneu,
    cale: Option<PathBuf>,
}

impl SursaFixture {
    pub fn din_fisier(cale: impl AsRef<Path>) -> Rezultat<Self> {
        let cale = cale.as_ref().to_path_buf();
        Ok(SursaFixture {
            instantaneu: Instantaneu::citeste(&cale)?,
            cale: Some(cale),
        })
    }

    pub fn din_instantaneu(instantaneu: Instantaneu) -> Self {
        SursaFixture {
            instantaneu,
            cale: None,
        }
    }
}

impl SursaObiecte for SursaFixture {
    fn identitate(&self) -> String {
        // serverul **declarat in instantaneu**, nu calea fisierului: acelasi
        // server exportat de doua ori e tot acelasi server, deci aceeasi
        // sesiune (F-B4). Calea e in `descriere`, pentru om.
        format!("instantaneu {}", self.instantaneu.server)
    }

    fn descriere(&self) -> String {
        match &self.cale {
            Some(c) => format!(
                "instantaneu {} (server {})",
                c.display(),
                self.instantaneu.server
            ),
            None => format!(
                "instantaneu in memorie (server {})",
                self.instantaneu.server
            ),
        }
    }

    fn baze(&mut self) -> Rezultat<Vec<String>> {
        Ok(self.instantaneu.baze.keys().cloned().collect())
    }

    fn obiecte(&mut self, baza: &str) -> Rezultat<Vec<Obiect>> {
        self.instantaneu
            .baze
            .get(baza)
            .cloned()
            .ok_or_else(|| Eroare::BazaLipsa {
                server: self.instantaneu.server.clone(),
                baza: baza.to_string(),
            })
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    fn obiect(nume: &str) -> Obiect {
        Obiect {
            schema: "dbo".into(),
            nume: nume.into(),
            tip: "P".into(),
            definitie: format!("CREATE PROCEDURE dbo.{nume} AS SELECT 1"),
            setari: None,
            modificat: None,
            creat: None,
        }
    }

    fn exemplu() -> Instantaneu {
        let mut i = Instantaneu::nou("SRV01", "2026-11-17T10:00:00Z");
        i.baze
            .insert("Baza1".into(), vec![obiect("pA"), obiect("pB")]);
        i.baze.insert("Baza2".into(), vec![obiect("pA")]);
        i
    }

    #[test]
    fn dus_intors_prin_json() {
        let i = exemplu();
        let text = serde_json::to_string(&i).unwrap();
        let revenit: Instantaneu = serde_json::from_str(&text).unwrap();
        assert_eq!(revenit.baze, i.baze);
        assert_eq!(revenit.server, "SRV01");
    }

    #[test]
    fn sursa_intoarce_bazele_sortate() {
        let mut s = SursaFixture::din_instantaneu(exemplu());
        assert_eq!(s.baze().unwrap(), vec!["Baza1", "Baza2"]);
        assert_eq!(s.obiecte("Baza1").unwrap().len(), 2);
    }

    #[test]
    fn baza_lipsa_da_eroarea_cu_cod() {
        let mut s = SursaFixture::din_instantaneu(exemplu());
        let e = s.obiecte("Inexistenta").unwrap_err();
        assert_eq!(e.cod(), "GM-DB-005");
    }

    #[test]
    fn versiune_necunoscuta_e_refuzata() {
        let dir = std::env::temp_dir().join(format!("gmotion-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cale = dir.join("vechi.json");
        let mut i = exemplu();
        i.versiune = 99;
        std::fs::write(&cale, serde_json::to_string(&i).unwrap()).unwrap();
        let e = Instantaneu::citeste(&cale).unwrap_err();
        assert_eq!(e.cod(), "GM-DB-009");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn fisier_lipsa_si_json_rupt_dau_cod_007() {
        let e = Instantaneu::citeste("/cale/care/nu/exista.json").unwrap_err();
        assert_eq!(e.cod(), "GM-DB-007");

        let dir = std::env::temp_dir().join(format!("gmotion-test-rupt-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cale = dir.join("rupt.json");
        std::fs::write(&cale, "{ nu e json").unwrap();
        assert_eq!(Instantaneu::citeste(&cale).unwrap_err().cod(), "GM-DB-007");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Regresie din audit: continutul instantaneului nu se verifica.
    #[test]
    fn instantaneu_cu_continut_ostil_e_refuzat() {
        let dir = std::env::temp_dir().join(format!("gmotion-test-ostil-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let ob =
            |n: &str| serde_json::json!({"schema":"dbo","nume":n,"tip":"P","definitie":"SELECT 1"});
        for (eticheta, baze) in [
            (
                "control",
                serde_json::json!({"x\nGO\nDROP DATABASE Prod": [ob("p")]}),
            ),
            ("duplicat", serde_json::json!({"A": [ob("p"), ob("p")]})),
            ("gol", serde_json::json!({"A": [ob("")]})),
        ] {
            let cale = dir.join(format!("{eticheta}.json"));
            let inst = serde_json::json!({"versiune":1,"server":"s","creat":"t","baze":baze});
            std::fs::write(&cale, inst.to_string()).unwrap();
            let e = Instantaneu::citeste(&cale).unwrap_err();
            assert_eq!(e.cod(), "GM-DB-007", "{eticheta}: {e}");
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn scrierea_nu_lasa_fisiere_temporare() {
        let dir = std::env::temp_dir().join(format!("gmotion-test-atomic-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cale = dir.join("i.json");
        Instantaneu::nou("S", "t").scrie(&cale).unwrap();
        Instantaneu::nou("S", "t2").scrie(&cale).unwrap();
        assert_eq!(Instantaneu::citeste(&cale).unwrap().creat, "t2");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }
}
