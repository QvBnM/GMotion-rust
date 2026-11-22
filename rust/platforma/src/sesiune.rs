//! Sesiuni: progresul salvat al unui modul pe o baza de referinta.
//!
//! **Se strica astfel:** progresul se pierde, sau — mai rau — un modul
//! suprascrie sesiunea altuia. Al doilea era un bug garantat in versiunea
//! Python, unde cheia era doar numele bazei (`<slug>.dat`): modulul Upgrade
//! salvand progres pe aceeasi baza ar fi sters sesiunea modulului de diff.
//! Aici cheia e `<modul>/<slug>.dat` din start.
//!
//! **O sesiune nu contine niciodata secrete.** Nu e o conventie, e o
//! proprietate a tipului: [`Sesiune`] nu are camp de parola, iar
//! `gmotion_db::Autentificare` nu implementeaza `Serialize`. Asta face
//! acceptabil ca pe non-Windows fisierul sa nu fie criptat. Daca vreodata
//! trebuie salvat un secret, calea de non-Windows din `protejare.rs` trebuie
//! sa refuze, nu sa scrie in clar.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::cai::director_sesiuni;
use crate::eroare::{valideaza_identificator, Eroare, Rezultat};
use crate::protejare::{deprotejeaza, protejeaza};

pub const VERSIUNE_SESIUNE: u32 = 1;

/// Progresul salvat. `date` e opac pentru platforma: fiecare modul pune acolo
/// ce are nevoie, fara ca platforma sa trebuiasca sa stie structura.
///
/// Platforma impune doar ce e comun: versiunea, ce server si ce baze, si cand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sesiune {
    pub versiune: u32,
    /// Numele serverului. **Niciodata** parola sau textul de conexiune.
    pub server: String,
    /// Eticheta de autentificare: `SQL` sau `Windows`. Nu acreditarile.
    pub auth: String,
    /// Utilizatorul SQL, cand e cazul. Util la reluare; nu e un secret.
    #[serde(default)]
    pub utilizator: String,
    pub referinta: String,
    pub baze: Vec<String>,
    /// ISO 8601.
    pub salvata: String,
    /// Starea proprie a modulului. Platforma nu o interpreteaza.
    #[serde(default)]
    pub date: BTreeMap<String, serde_json::Value>,
}

impl Sesiune {
    pub fn noua(
        server: impl Into<String>,
        auth: impl Into<String>,
        referinta: impl Into<String>,
        salvata: impl Into<String>,
    ) -> Self {
        Sesiune {
            versiune: VERSIUNE_SESIUNE,
            server: server.into(),
            auth: auth.into(),
            utilizator: String::new(),
            referinta: referinta.into(),
            baze: Vec::new(),
            salvata: salvata.into(),
            date: BTreeMap::new(),
        }
    }

    /// Adauga bazele comparate acum la cele de care sesiunea stie deja.
    ///
    /// **Nu le inlocuieste.** O sesiune tine munca facuta pe toate bazele
    /// comparate vreodata cu aceeasi referinta; scrisa peste, lista spunea
    /// doar ce s-a comparat ultima data, iar urmatoarea curatare stergea
    /// restul (C3/C4 din auditul QA).
    pub fn adauga_baze(&mut self, baze: &[String]) {
        for b in baze {
            if !self.baze.contains(b) {
                self.baze.push(b.clone());
            }
        }
    }
}

/// Numele de fisier pentru o baza de referinta.
///
/// Numele de baze pot contine orice; fisierele nu. Se reduce la litere, cifre
/// si cratima, pentru lizibilitate, plus 48 de biti din SHA-256 pe numele
/// **exact**, ca doua nume diferite sa nu ajunga niciodata in acelasi fisier.
/// Pana la audit discriminantul era lungimea numelui, deci `Prod.2024` si
/// `Prod_2024` (doua baze distincte) aveau acelasi fisier, iar o sesiune o
/// suprascria tacut pe cealalta.
pub fn slug(nume: &str) -> String {
    let h = gmotion_core::calc_hash(nume);
    format!("{}-{}", nume_lizibil(nume), &h[..12])
}

/// Partea lizibila a unui nume de fisier: litere, cifre si cratima, cel mult
/// 48 de caractere. Nu e unica singura — de-aia peste ea vine un hash.
fn nume_lizibil(nume: &str) -> String {
    let mut s: String = nume
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    let s = s.trim_matches('-').to_string();
    let baza = if s.is_empty() { "baza".to_string() } else { s };
    baza.chars().take(48).collect()
}

/// Numele de fisier al unei sesiuni: baza de referinta **si serverul**.
///
/// `PROD\\Ref` si `TEST\\Ref` sunt doua sesiuni diferite. Pana la auditul QA
/// cheia era doar referinta, deci isi suprascriau una alteia progresul si
/// `incarca` intorcea sesiunea celuilalt server (F-B4). Partea lizibila
/// ramane numele bazei; discriminantul e hash-ul perechii exacte.
pub fn slug_sesiune(server: &str, referinta: &str) -> String {
    let h = gmotion_core::calc_hash(&format!("{server}\0{referinta}"));
    format!("{}-{}", nume_lizibil(referinta), &h[..12])
}

/// Sesiunile unui modul. Un `Sesiuni` e legat de **un** modul: platforma da
/// fiecarui modul instanta lui, deci un modul nu poate ajunge la sesiunile
/// altuia nici din greseala.
pub struct Sesiuni {
    modul: String,
    director: PathBuf,
}

impl Sesiuni {
    pub fn pentru_modul(modul: &str) -> Rezultat<Self> {
        valideaza_identificator(modul)?;
        Ok(Sesiuni {
            modul: modul.to_string(),
            director: director_sesiuni(modul)?,
        })
    }

    /// Sesiunile unui modul intr-un director dat, nu in cel de date al
    /// aplicatiei. Pentru teste: fara el, un test al starii aplicatiei ar
    /// trebui sa schimbe `GMOTION_DIR_DATE`, adica mediul procesului comun.
    pub fn in_director(modul: &str, director: PathBuf) -> Rezultat<Self> {
        valideaza_identificator(modul)?;
        Ok(Sesiuni {
            modul: modul.to_string(),
            director,
        })
    }

    pub fn modul(&self) -> &str {
        &self.modul
    }

    pub fn director(&self) -> &PathBuf {
        &self.director
    }

    fn cale(&self, server: &str, referinta: &str) -> PathBuf {
        self.director
            .join(format!("{}.dat", slug_sesiune(server, referinta)))
    }

    /// Calea de dinainte de auditul QA: numai pe referinta. Se citeste o
    /// singura data, ca sa mute sesiunea pe numele nou.
    fn cale_veche(&self, referinta: &str) -> PathBuf {
        self.director.join(format!("{}.dat", slug(referinta)))
    }

    /// Muta o sesiune scrisa sub numele vechi (doar pe referinta) pe numele
    /// nou, care tine si serverul. Intoarce `true` daca a mutat ceva.
    ///
    /// Se cheama la `incarca`: altfel progresul de pana acum ar aparea
    /// pierdut la prima pornire cu versiunea noua.
    fn migreaza(&self, referinta: &str) -> bool {
        let vechi = self.cale_veche(referinta);
        let Ok(date) = std::fs::read(&vechi) else {
            return false;
        };
        let Some(s) = deprotejeaza(&date)
            .and_then(|clar| serde_json::from_slice::<Sesiune>(&clar).ok())
            .filter(|s| s.referinta == referinta)
        else {
            return false;
        };
        let nou = self.cale(&s.server, referinta);
        if nou.exists() {
            return false;
        }
        match std::fs::rename(&vechi, &nou) {
            Ok(()) => {
                tracing::info!(
                    target: "gmotion::platforma",
                    de_la = %vechi.display(), la = %nou.display(),
                    "sesiune mutata pe numele care tine si serverul"
                );
                true
            }
            Err(e) => {
                tracing::warn!(
                    target: "gmotion::platforma", eroare = %e,
                    cale = %vechi.display(), "sesiunea veche nu s-a putut muta"
                );
                false
            }
        }
    }

    pub fn salveaza(&self, sesiune: &Sesiune) -> Rezultat<PathBuf> {
        let cale = self.cale(&sesiune.server, &sesiune.referinta);
        std::fs::create_dir_all(&self.director).map_err(|e| Eroare::ScriereSesiune {
            cale: self.director.display().to_string(),
            detaliu: e.to_string(),
        })?;
        let text = serde_json::to_vec(sesiune).map_err(|e| Eroare::ScriereSesiune {
            cale: cale.display().to_string(),
            detaliu: e.to_string(),
        })?;
        let protejat = protejeaza(&text).ok_or_else(|| Eroare::ScriereSesiune {
            cale: cale.display().to_string(),
            detaliu: "sistemul a refuzat protejarea datelor".into(),
        })?;
        crate::cai::scrie_atomic(&cale, &protejat).map_err(|e| Eroare::ScriereSesiune {
            cale: cale.display().to_string(),
            detaliu: e.to_string(),
        })?;
        tracing::info!(
            target: "gmotion::platforma",
            modul = %self.modul, referinta = %sesiune.referinta,
            cale = %cale.display(), "sesiune salvata"
        );
        Ok(cale)
    }

    /// `Ok(None)` cand nu exista sesiune — nu e o eroare.
    ///
    /// `server` face parte din cheie: sesiunea lui `PROD` nu se intoarce cand
    /// se cere a lui `TEST` (F-B4). O sesiune scrisa sub numele vechi (doar pe
    /// referinta) se muta pe numele nou la prima citire.
    pub fn incarca(&self, server: &str, referinta: &str) -> Rezultat<Option<Sesiune>> {
        let mut cale = self.cale(server, referinta);
        if !cale.exists() && self.migreaza(referinta) {
            cale = self.cale(server, referinta);
        }
        let protejat = match std::fs::read(&cale) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(Eroare::CitireSesiune {
                    cale: cale.display().to_string(),
                    detaliu: e.to_string(),
                })
            }
        };
        let clar = deprotejeaza(&protejat).ok_or_else(|| Eroare::DecriptareSesiune {
            cale: cale.display().to_string(),
        })?;
        let s: Sesiune = serde_json::from_slice(&clar).map_err(|e| Eroare::CitireSesiune {
            cale: cale.display().to_string(),
            detaliu: e.to_string(),
        })?;
        if s.versiune != VERSIUNE_SESIUNE {
            return Err(Eroare::VersiuneSesiune {
                cale: cale.display().to_string(),
                gasita: s.versiune,
                asteptata: VERSIUNE_SESIUNE,
            });
        }
        // Un fisier mutat sau redenumit de mana nu are voie sa devina sesiunea
        // altei baze.
        if s.referinta != referinta {
            return Err(Eroare::CitireSesiune {
                cale: cale.display().to_string(),
                detaliu: format!(
                    "fisierul e sesiunea bazei {:?}, nu a lui {:?}",
                    s.referinta, referinta
                ),
            });
        }
        if s.server != server {
            return Err(Eroare::CitireSesiune {
                cale: cale.display().to_string(),
                detaliu: format!(
                    "fisierul e sesiunea serverului {:?}, nu a lui {:?}",
                    s.server, server
                ),
            });
        }
        Ok(Some(s))
    }

    pub fn sterge(&self, server: &str, referinta: &str) -> Rezultat<bool> {
        self.migreaza(referinta);
        let cale = self.cale(server, referinta);
        match std::fs::remove_file(&cale) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(Eroare::ScriereSesiune {
                cale: cale.display().to_string(),
                detaliu: e.to_string(),
            }),
        }
    }

    /// Sesiunile existente ale acestui modul, sortate dupa referinta.
    ///
    /// O sesiune care nu se poate decripta sau parsa e **sarita cu
    /// avertisment**, nu face listarea sa cada: altfel un singur fisier stricat
    /// ar ascunde toate celelalte.
    pub fn lista(&self) -> Rezultat<Vec<Sesiune>> {
        let citire = match std::fs::read_dir(&self.director) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(Eroare::CitireSesiune {
                    cale: self.director.display().to_string(),
                    detaliu: e.to_string(),
                })
            }
        };
        let mut out = Vec::new();
        for intrare in citire.flatten() {
            let cale = intrare.path();
            if cale.extension().and_then(|e| e.to_str()) != Some("dat") {
                continue;
            }
            match std::fs::read(&cale)
                .ok()
                .and_then(|d| deprotejeaza(&d))
                .and_then(|c| serde_json::from_slice::<Sesiune>(&c).ok())
            {
                Some(s) if s.versiune == VERSIUNE_SESIUNE => out.push(s),
                _ => tracing::warn!(
                    target: "gmotion::platforma",
                    cale = %cale.display(),
                    "sesiune ilizibila, sarita la listare"
                ),
            }
        }
        out.sort_by(|a, b| a.referinta.cmp(&b.referinta));
        Ok(out)
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn sesiunile_dintr_un_director_dat_se_salveaza_acolo() {
        let d = std::env::temp_dir().join(format!("gmotion-in-director-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        let s = Sesiuni::in_director("diff-obiecte", d.clone()).unwrap();
        assert_eq!(s.director(), &d);
        let cale = s.salveaza(&Sesiune::noua("SRV", "SQL", "Ref", "")).unwrap();
        assert!(cale.starts_with(&d), "{}", cale.display());
        assert!(s.incarca("SRV", "Ref").unwrap().is_some());
        assert!(Sesiuni::in_director("../afara", d.clone()).is_err());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn slug_curata_si_e_stabil() {
        assert!(slug("Baza Mea").starts_with("baza-mea-"));
        assert!(slug("a/b\\c").starts_with("a-b-c-"));
        assert!(slug("---").starts_with("baza-"));
        assert!(slug("ÎmiPlace").starts_with("miplace-")); // diacritice-permise: date de test
                                                           // acelasi nume da mereu acelasi slug
        assert_eq!(slug("Baza1"), slug("Baza1"));
    }

    /// Regresie din audit: discriminantul era lungimea, deci perechile astea
    /// (nume distincte, aceeasi lungime) dadeau acelasi fisier.
    #[test]
    fn nume_diferite_nu_ajung_in_acelasi_fisier() {
        for (a, b) in [
            ("a/b", "a//b"),
            ("Prod.2024", "Prod_2024"),
            ("Baza_A", "baza-a"),
            ("Ăla", "Îla"), // diacritice-permise: date de test
        ] {
            assert_ne!(slug(a), slug(b), "{a:?} / {b:?}");
        }
    }

    #[test]
    fn slugul_nu_poate_iesi_din_director() {
        for n in ["../../etc/passwd", "..", "./..", "C:\\Windows"] {
            let s = slug(n);
            assert!(
                !s.contains('/') && !s.contains('\\') && !s.contains(".."),
                "{s}"
            );
        }
    }

    #[test]
    fn sesiunea_nu_are_loc_pentru_parola() {
        // Documentatie executabila: daca cineva adauga un camp de parola,
        // testul asta trebuie sa-l faca sa se opreasca si sa se intrebe de ce.
        let s = Sesiune::noua("SRV", "SQL", "Ref", "2026-11-17T00:00:00Z");
        let text = serde_json::to_string(&s).unwrap();
        for cuvant in ["parola", "password", "pwd", "secret"] {
            assert!(!text.contains(cuvant), "{cuvant} apare in sesiune: {text}");
        }
    }
}
