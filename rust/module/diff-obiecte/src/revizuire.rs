//! Ce a revizuit utilizatorul, legat de continutul obiectului.
//!
//! **Se strica astfel:** un obiect ramane bifat „revizuit" dupa ce cineva l-a
//! schimbat pe server. Atunci bifa minte exact in momentul cel mai scump: la
//! deploy, cand te bazezi pe ea ca sa nu te mai uiti. De-aia bifa nu se tine
//! pe nume, ci pe **amprenta celor doua definitii**: daca una s-a schimbat,
//! bifa dispare singura.
//!
//! Starea sta in `date` din `gmotion_platforma::Sesiune`, campul pe care
//! platforma nu-l interpreteaza: mecanismul de salvare e al platformei,
//! continutul e al modulului.

use std::collections::BTreeMap;

use gmotion_platforma::Sesiune;
use serde::{Deserialize, Serialize};

use crate::comparare::Rand;

/// Cheia sub care sta starea in `Sesiune::date`.
pub const CHEIE_SESIUNE: &str = "revizuire";

/// Amprenta celor doua versiuni in momentul in care s-a pus bifa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amprenta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referinta: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curent: Option<String>,
}

impl Amprenta {
    pub fn a_randului(r: &Rand) -> Self {
        Amprenta {
            referinta: r.hash_referinta.clone(),
            curent: r.hash_curent.clone(),
        }
    }
}

/// Obiectele revizuite, pe baza comparata.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revizuire {
    /// `baza` -> `schema.nume` -> amprenta de atunci.
    #[serde(default)]
    baze: BTreeMap<String, BTreeMap<String, Amprenta>>,
    /// Obiectele omise („Omite" din v3.4): analizate, dar lasate deliberat
    /// in afara deploy-ului. Aceeasi regula ca bifa: cad daca obiectul se
    /// schimba. Un obiect e ori revizuit, ori omis, nu amandoua.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    omise: BTreeMap<String, BTreeMap<String, Amprenta>>,
    /// Obiectele scrise intr-un deploy, cu amprenta de atunci. Spre deosebire
    /// de bife, nu cad cand obiectul se schimba: atunci obiectul e o
    /// „versiune noua" fata de ce s-a deployat (coloana Deploy din v3.4).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    deployate: BTreeMap<String, BTreeMap<String, Amprenta>>,
}

impl Revizuire {
    pub fn noua() -> Self {
        Self::default()
    }

    /// Marcheaza randul ca omis (si scoate bifa „revizuit", daca era).
    pub fn omite(&mut self, r: &Rand) {
        self.scoate(&r.baza, &r.obiect);
        self.omise
            .entry(r.baza.clone())
            .or_default()
            .insert(r.obiect.clone(), Amprenta::a_randului(r));
    }

    /// Scoate marcajul „omis". Intoarce `true` daca exista.
    pub fn scoate_omis(&mut self, baza: &str, obiect: &str) -> bool {
        let scos = self
            .omise
            .get_mut(baza)
            .is_some_and(|b| b.remove(obiect).is_some());
        self.omise.retain(|_, b| !b.is_empty());
        scos
    }

    /// Omis, si obiectul e acelasi ca atunci.
    pub fn e_omis(&self, r: &Rand) -> bool {
        self.omise
            .get(&r.baza)
            .and_then(|b| b.get(&r.obiect))
            .is_some_and(|a| *a == Amprenta::a_randului(r))
    }

    /// Obiectul a fost scris intr-un deploy.
    pub fn marcheaza_deployat(&mut self, r: &Rand) {
        self.deployate
            .entry(r.baza.clone())
            .or_default()
            .insert(r.obiect.clone(), Amprenta::a_randului(r));
    }

    /// Deployat, si e tot obiectul de atunci.
    pub fn e_deployat(&self, r: &Rand) -> bool {
        self.deployate
            .get(&r.baza)
            .and_then(|b| b.get(&r.obiect))
            .is_some_and(|a| *a == Amprenta::a_randului(r))
    }

    /// A fost deployat vreodata (poate intr-o versiune mai veche).
    pub fn a_fost_deployat(&self, r: &Rand) -> bool {
        self.deployate
            .get(&r.baza)
            .is_some_and(|b| b.contains_key(&r.obiect))
    }

    pub fn numar_omise(&self) -> usize {
        self.omise.values().map(BTreeMap::len).sum()
    }

    /// Marcheaza randul ca revizuit, cu amprenta lui de acum.
    pub fn marcheaza(&mut self, r: &Rand) {
        self.scoate_omis(&r.baza, &r.obiect);
        self.baze
            .entry(r.baza.clone())
            .or_default()
            .insert(r.obiect.clone(), Amprenta::a_randului(r));
    }

    /// Scoate bifa. Intoarce `true` daca exista.
    pub fn scoate(&mut self, baza: &str, obiect: &str) -> bool {
        self.baze
            .get_mut(baza)
            .is_some_and(|b| b.remove(obiect).is_some())
    }

    /// Bifa, dar **numai daca** obiectul e acelasi ca atunci cand s-a pus.
    /// Daca definitia s-a schimbat in oricare din cele doua baze, raspunsul e
    /// `false`: bifa veche nu mai spune nimic despre ce e acum pe server.
    pub fn e_revizuit(&self, r: &Rand) -> bool {
        self.baze
            .get(&r.baza)
            .and_then(|b| b.get(&r.obiect))
            .is_some_and(|a| *a == Amprenta::a_randului(r))
    }

    /// Cate bife sunt inca valabile pentru randurile date, si cate au cazut
    /// fiindca obiectul s-a schimbat.
    pub fn valabile(&self, randuri: &[Rand]) -> (usize, usize) {
        let valabile = randuri.iter().filter(|r| self.e_revizuit(r)).count();
        let cunoscute = randuri
            .iter()
            .filter(|r| {
                self.baze
                    .get(&r.baza)
                    .is_some_and(|b| b.contains_key(&r.obiect))
            })
            .count();
        (valabile, cunoscute - valabile)
    }

    /// Scoate bifele care nu mai corespund niciunui rand de acum (obiectul a
    /// disparut, sau s-a schimbat). Intoarce cate a scos.
    ///
    /// Se cheama explicit, la salvare: altfel fisierul creste la nesfarsit cu
    /// bife despre obiecte care nu mai exista.
    pub fn curata(&mut self, randuri: &[Rand]) -> usize {
        let mut scoase = 0;
        for (baza, obiecte) in self.baze.iter_mut() {
            obiecte.retain(|obiect, amprenta| {
                let pastreaza = randuri.iter().any(|r| {
                    r.baza == *baza && r.obiect == *obiect && Amprenta::a_randului(r) == *amprenta
                });
                if !pastreaza {
                    scoase += 1;
                }
                pastreaza
            });
        }
        self.baze.retain(|_, obiecte| !obiecte.is_empty());
        for (baza, obiecte) in self.omise.iter_mut() {
            obiecte.retain(|obiect, amprenta| {
                let pastreaza = randuri.iter().any(|r| {
                    r.baza == *baza && r.obiect == *obiect && Amprenta::a_randului(r) == *amprenta
                });
                scoase += usize::from(!pastreaza);
                pastreaza
            });
        }
        self.omise.retain(|_, obiecte| !obiecte.is_empty());
        scoase
    }

    pub fn numar(&self) -> usize {
        self.baze.values().map(BTreeMap::len).sum()
    }

    pub fn e_goala(&self) -> bool {
        self.numar() == 0 && self.numar_omise() == 0 && self.deployate.is_empty()
    }

    /// Obiectele bifate pe o baza, in ordine.
    pub fn obiecte(&self, baza: &str) -> Vec<&str> {
        self.baze
            .get(baza)
            .map(|b| b.keys().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Citeste starea dintr-o sesiune. O sesiune fara starea asta, sau cu una
    /// pe care nu o mai intelegem, da o stare goala: progresul se pierde, dar
    /// aplicatia porneste. (Invers ar insemna sa nu se mai poata deschide o
    /// sesiune veche.)
    pub fn din_sesiune(s: &Sesiune) -> Self {
        s.date
            .get(CHEIE_SESIUNE)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// Pune starea in sesiune, peste ce era.
    pub fn in_sesiune(&self, s: &mut Sesiune) {
        match serde_json::to_value(self) {
            Ok(v) => {
                s.date.insert(CHEIE_SESIUNE.to_string(), v);
            }
            Err(_) => {
                s.date.remove(CHEIE_SESIUNE);
            }
        }
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use crate::comparare::Categorie;

    fn rand(obiect: &str, h_ref: &str, h_curent: &str) -> Rand {
        Rand {
            id: 1,
            obiect: obiect.into(),
            baza: "Client1".into(),
            scor: Some(90.0),
            rezumat: "1 mod".into(),
            tip: "P".into(),
            categorie: Categorie::Diferit,
            hash_referinta: Some(h_ref.into()),
            hash_curent: Some(h_curent.into()),
        }
    }

    #[test]
    fn bifa_tine_cat_timp_obiectul_e_acelasi() {
        let r = rand("dbo.p", "aaa", "bbb");
        let mut rev = Revizuire::noua();
        assert!(!rev.e_revizuit(&r));
        rev.marcheaza(&r);
        assert!(rev.e_revizuit(&r));
        assert_eq!(rev.numar(), 1);
        assert_eq!(rev.obiecte("Client1"), ["dbo.p"]);
    }

    #[test]
    fn bifa_cade_singura_cand_obiectul_s_a_schimbat_pe_server() {
        let mut rev = Revizuire::noua();
        rev.marcheaza(&rand("dbo.p", "aaa", "bbb"));
        // s-a schimbat in baza comparata
        assert!(!rev.e_revizuit(&rand("dbo.p", "aaa", "ALTCEVA")));
        // s-a schimbat in referinta
        assert!(!rev.e_revizuit(&rand("dbo.p", "ALTCEVA", "bbb")));
        // acelasi obiect, alta baza: bifa nu se intinde
        let mut alta = rand("dbo.p", "aaa", "bbb");
        alta.baza = "Client2".into();
        assert!(!rev.e_revizuit(&alta));
    }

    #[test]
    fn se_vede_cate_bife_au_cazut() {
        let mut rev = Revizuire::noua();
        rev.marcheaza(&rand("dbo.p", "aaa", "bbb"));
        rev.marcheaza(&rand("dbo.q", "ccc", "ddd"));
        let randuri = vec![rand("dbo.p", "aaa", "bbb"), rand("dbo.q", "ccc", "ALTCEVA")];
        assert_eq!(rev.valabile(&randuri), (1, 1));
    }

    #[test]
    fn curatarea_scoate_bifele_care_nu_mai_corespund() {
        let mut rev = Revizuire::noua();
        rev.marcheaza(&rand("dbo.p", "aaa", "bbb"));
        rev.marcheaza(&rand("dbo.q", "ccc", "ddd"));
        rev.marcheaza(&rand("dbo.sters", "eee", "fff"));
        let randuri = vec![rand("dbo.p", "aaa", "bbb"), rand("dbo.q", "ccc", "ALTCEVA")];
        assert_eq!(
            rev.curata(&randuri),
            2,
            "q s-a schimbat, sters nu mai exista"
        );
        assert_eq!(rev.numar(), 1);
        assert!(rev.e_revizuit(&randuri[0]));
    }

    #[test]
    fn scoaterea_bifei_spune_daca_exista() {
        let mut rev = Revizuire::noua();
        rev.marcheaza(&rand("dbo.p", "aaa", "bbb"));
        assert!(rev.scoate("Client1", "dbo.p"));
        assert!(!rev.scoate("Client1", "dbo.p"));
        assert!(!rev.scoate("Client1", "dbo.inexistent"));
        assert!(rev.e_goala());
    }

    #[test]
    fn starea_face_drumul_dus_intors_prin_sesiune() {
        let mut rev = Revizuire::noua();
        rev.marcheaza(&rand("dbo.p", "aaa", "bbb"));
        let mut s = Sesiune::noua("SRV", "SQL", "Referinta", "2026-11-17T10:00:00Z");
        rev.in_sesiune(&mut s);
        // sesiunea ramane serializabila, si nu contine secrete
        let text = serde_json::to_string(&s).expect("sesiunea se serializeaza");
        assert!(text.contains("revizuire") && text.contains("dbo.p"));
        let s2: Sesiune = serde_json::from_str(&text).unwrap();
        assert_eq!(Revizuire::din_sesiune(&s2), rev);
    }

    #[test]
    fn o_sesiune_veche_sau_stricata_da_stare_goala_nu_eroare() {
        let mut s = Sesiune::noua("SRV", "SQL", "Referinta", "2026-11-17T10:00:00Z");
        assert!(
            Revizuire::din_sesiune(&s).e_goala(),
            "sesiune fara starea asta"
        );
        s.date.insert(
            CHEIE_SESIUNE.to_string(),
            serde_json::json!("altceva decat ne asteptam"),
        );
        assert!(Revizuire::din_sesiune(&s).e_goala());
    }

    #[test]
    fn omis_si_revizuit_se_exclud_si_omis_cade_la_schimbare() {
        let r = rand_test("a", "b");
        let mut v = Revizuire::noua();
        v.omite(&r);
        assert!(v.e_omis(&r) && !v.e_revizuit(&r));
        v.marcheaza(&r);
        assert!(v.e_revizuit(&r) && !v.e_omis(&r), "revizuit scoate omis");
        v.omite(&r);
        assert!(v.e_omis(&r) && !v.e_revizuit(&r), "omis scoate revizuit");
        let schimbat = rand_test("a", "b2");
        assert!(!v.e_omis(&schimbat));
        assert!(!v.e_goala());
        assert_eq!(v.curata(&[schimbat]), 1);
        assert!(v.e_goala());
    }

    #[test]
    fn omisele_supravietuiesc_sesiunii() {
        let r = rand_test("a", "b");
        let mut v = Revizuire::noua();
        v.omite(&r);
        let mut s = Sesiune::noua("srv", "Windows", "Ref", "t");
        v.in_sesiune(&mut s);
        assert!(Revizuire::din_sesiune(&s).e_omis(&r));
    }

    #[test]
    fn deployat_ramane_dar_devine_versiune_noua_la_schimbare() {
        let r = rand_test("a", "b");
        let mut v = Revizuire::noua();
        assert!(!v.a_fost_deployat(&r));
        v.marcheaza_deployat(&r);
        assert!(v.e_deployat(&r) && v.a_fost_deployat(&r));
        let schimbat = rand_test("a", "b2");
        assert!(!v.e_deployat(&schimbat), "versiune noua");
        assert!(v.a_fost_deployat(&schimbat));
        // curatarea nu atinge istoricul de deploy
        v.curata(std::slice::from_ref(&schimbat));
        assert!(v.a_fost_deployat(&schimbat));
        let mut s = Sesiune::noua("srv", "Windows", "Ref", "t");
        v.in_sesiune(&mut s);
        assert!(Revizuire::din_sesiune(&s).e_deployat(&r));
    }

    fn rand_test(hr: &str, hc: &str) -> Rand {
        Rand {
            id: 0,
            obiect: "dbo.p".into(),
            baza: "Client1".into(),
            scor: Some(90.0),
            rezumat: String::new(),
            tip: "P".into(),
            categorie: Categorie::Diferit,
            hash_referinta: Some(hr.into()),
            hash_curent: Some(hc.into()),
        }
    }
}
