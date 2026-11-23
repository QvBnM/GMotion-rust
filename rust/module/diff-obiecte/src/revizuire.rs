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
    /// Notele scrise de utilizator („a ramas aici"), pe `baza` ->
    /// `schema.nume`. Spre deosebire de bife, se tin pe **nume**: o nota e
    /// despre obiect, nu despre o versiune a lui, si nu cade cand se schimba.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    note: BTreeMap<String, BTreeMap<String, String>>,
}

impl Revizuire {
    pub fn noua() -> Self {
        Self::default()
    }

    /// Sterge bifele „revizuit" si „omis". Notele si marcajele de deploy
    /// raman: nu sunt progres de revizuire, ci istoric (`revizuire
    /// --goleste`, C10). Intoarce cate bife s-au sters.
    pub fn goleste_bifele(&mut self) -> usize {
        let n = self.numar() + self.numar_omise();
        self.baze.clear();
        self.omise.clear();
        n
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

    /// Pune nota obiectului; un text gol (sau doar spatii) o sterge.
    /// Intoarce `true` daca s-a schimbat ceva.
    pub fn pune_nota(&mut self, baza: &str, obiect: &str, text: &str) -> bool {
        let text = text.trim_end();
        if text.trim().is_empty() {
            let scos = self
                .note
                .get_mut(baza)
                .is_some_and(|b| b.remove(obiect).is_some());
            self.note.retain(|_, b| !b.is_empty());
            return scos;
        }
        let b = self.note.entry(baza.to_string()).or_default();
        if b.get(obiect).map(String::as_str) == Some(text) {
            return false;
        }
        b.insert(obiect.to_string(), text.to_string());
        true
    }

    /// Nota obiectului, daca are.
    pub fn nota(&self, baza: &str, obiect: &str) -> Option<&str> {
        self.note
            .get(baza)
            .and_then(|b| b.get(obiect))
            .map(String::as_str)
    }

    /// Toate notele, pe `(baza, schema.nume)`, in ordine.
    pub fn note(&self) -> Vec<(&str, &str, &str)> {
        self.note
            .iter()
            .flat_map(|(b, o)| {
                o.iter()
                    .map(move |(n, t)| (b.as_str(), n.as_str(), t.as_str()))
            })
            .collect()
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
    pub fn curata(&mut self, randuri: &[Rand], baze_comparate: &[String]) -> usize {
        let mut scoase = 0;
        // un index, nu `any` pe toate randurile pentru fiecare bifa: 5 s la
        // 10.000 de obiecte (P6 din auditul QA)
        let index: std::collections::HashMap<(&str, &str), Amprenta> = randuri
            .iter()
            .map(|r| {
                (
                    (r.baza.as_str(), r.obiect.as_str()),
                    Amprenta::a_randului(r),
                )
            })
            .collect();
        // bifele „revizuit" cad cand obiectul se schimba; omiterile raman cat
        // obiectul e in comparare, ca deploy-ul sa poata spune ca au cazut
        // (`omise_cazute`), in loc sa le stearga tacut la prima salvare (N4)
        let mut curata_harta = |harta: &mut BTreeMap<String, BTreeMap<String, Amprenta>>,
                                pe_amprenta: bool| {
            for (baza, obiecte) in harta.iter_mut() {
                // o baza care nu e in compararea de acum nu se atinge: n-are
                // randuri, deci totul ar parea „nu mai corespunde" si s-ar
                // pierde munca facuta pe ea alta data (C3/C4 din auditul QA)
                if !baze_comparate.iter().any(|b| b == baza) {
                    continue;
                }
                obiecte.retain(|obiect, amprenta| {
                    let pastreaza = index
                        .get(&(baza.as_str(), obiect.as_str()))
                        .is_some_and(|a| !pe_amprenta || a == amprenta);
                    scoase += usize::from(!pastreaza);
                    pastreaza
                });
            }
            harta.retain(|_, obiecte| !obiecte.is_empty());
        };
        curata_harta(&mut self.baze, true);
        curata_harta(&mut self.omise, false);
        scoase
    }

    pub fn numar(&self) -> usize {
        self.baze.values().map(BTreeMap::len).sum()
    }

    pub fn e_goala(&self) -> bool {
        self.numar() == 0
            && self.numar_omise() == 0
            && self.deployate.is_empty()
            && self.note.is_empty()
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
    /// Ca [`Self::din_sesiune`], dar o stare care exista si nu se poate citi
    /// (alta versiune, fisier editat) e eroare, nu stare goala: goala, omisele
    /// ar fi intrat in deploy si apoi ar fi fost sterse de pe disc (auditul
    /// 2026-11-23).
    pub fn din_sesiune_verificat(s: &Sesiune) -> crate::Rezultat<Self> {
        match s.date.get(CHEIE_SESIUNE) {
            None => Ok(Self::default()),
            Some(v) => serde_json::from_value(v.clone()).map_err(|e| {
                crate::Eroare::Date(format!(
                    "starea de revizuire din sesiune nu se poate citi: {e}"
                ))
            }),
        }
    }

    /// Obiectele omise care s-au schimbat de atunci: omiterea nu mai e
    /// valabila, dar nici nu trebuie sa intre tacut in deploy (N4).
    pub fn omise_cazute(&self, randuri: &[Rand]) -> Vec<(String, String)> {
        randuri
            .iter()
            .filter(|r| {
                self.omise
                    .get(&r.baza)
                    .and_then(|b| b.get(&r.obiect))
                    .is_some_and(|a| *a != Amprenta::a_randului(r))
            })
            .map(|r| (r.baza.clone(), r.obiect.clone()))
            .collect()
    }

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
            exacta_curent: None,
        }
    }

    /// C10: golirea lasa notele si marcajele de deploy.
    /// N4: omiterea unui obiect schimbat apare in `omise_cazute`.
    #[test]
    fn omiterea_unui_obiect_schimbat_se_vede() {
        let r = rand("dbo.p", "a", "b");
        let mut v = Revizuire::noua();
        v.omite(&r);
        assert!(v.omise_cazute(std::slice::from_ref(&r)).is_empty());
        let schimbat = rand("dbo.p", "a", "c");
        assert!(!v.e_omis(&schimbat));
        assert_eq!(
            v.omise_cazute(&[schimbat]),
            [("Client1".to_string(), "dbo.p".to_string())]
        );
    }

    #[test]
    fn golirea_sterge_doar_bifele() {
        let r = rand("dbo.p", "a", "b");
        let mut v = Revizuire::noua();
        v.marcheaza(&r);
        v.marcheaza_deployat(&r);
        v.pune_nota(&r.baza, &r.obiect, "a ramas aici");
        let mut o = rand("dbo.q", "a", "b");
        o.obiect = "dbo.q".into();
        v.omite(&o);
        assert_eq!(v.goleste_bifele(), 2);
        assert!(!v.e_revizuit(&r) && !v.e_omis(&o));
        assert!(v.a_fost_deployat(&r));
        assert_eq!(v.nota(&r.baza, &r.obiect), Some("a ramas aici"));
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

    /// C3 / C4 din `cercetare/audit-qa-2026-11-18.md`: ziua 1 compari Ref cu
    /// Client1 + Client2 si revizuiesti 200 de obiecte in Client2; ziua 2
    /// compari doar cu Client1, iar prima bifa (sau „Deconecteaza") curata
    /// tot ce tine de Client2 — n-are randuri. Fara niciun mesaj. Un „omis"
    /// pierdut inseamna obiect ajuns inapoi in deploy.
    #[test]
    fn curatarea_nu_atinge_bazele_din_afara_compararii() {
        let mut rev = Revizuire::noua();
        rev.marcheaza(&rand("dbo.p", "aaa", "bbb"));
        let mut in_client2 = rand("dbo.q", "ccc", "ddd");
        in_client2.baza = "Client2".into();
        rev.marcheaza(&in_client2);
        rev.omite(&in_client2);
        let mut omis2 = rand("dbo.r", "eee", "fff");
        omis2.baza = "Client2".into();
        rev.omite(&omis2);

        // ziua 2: se compara doar cu Client1
        let randuri = vec![rand("dbo.p", "aaa", "bbb")];
        let scoase = rev.curata(&randuri, &["Client1".to_string()]);
        assert_eq!(scoase, 0, "nimic de scos: Client2 nu e in comparare");
        assert!(rev.e_omis(&omis2), "omisul din Client2 a ramas");
        assert!(rev.e_revizuit(&rand("dbo.p", "aaa", "bbb")));

        // iar cand Client2 **e** in comparare, curatarea lucreaza ca pana acum
        let scoase = rev.curata(&randuri, &["Client1".to_string(), "Client2".to_string()]);
        assert_eq!(scoase, 2, "dbo.q si dbo.r nu mai au randuri in Client2");
    }

    #[test]
    fn curatarea_scoate_bifele_care_nu_mai_corespund() {
        let mut rev = Revizuire::noua();
        rev.marcheaza(&rand("dbo.p", "aaa", "bbb"));
        rev.marcheaza(&rand("dbo.q", "ccc", "ddd"));
        rev.marcheaza(&rand("dbo.sters", "eee", "fff"));
        let randuri = vec![rand("dbo.p", "aaa", "bbb"), rand("dbo.q", "ccc", "ALTCEVA")];
        assert_eq!(
            rev.curata(&randuri, &["Client1".to_string()]),
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
        // omiterea ramane, ca deploy-ul sa spuna ca a cazut (N4)
        assert_eq!(
            v.curata(std::slice::from_ref(&schimbat), &["Client1".to_string()]),
            0
        );
        assert_eq!(v.omise_cazute(&[schimbat]).len(), 1);
        assert_eq!(v.curata(&[], &["Client1".to_string()]), 1);
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
        v.curata(std::slice::from_ref(&schimbat), &["Client1".to_string()]);
        assert!(v.a_fost_deployat(&schimbat));
        let mut s = Sesiune::noua("srv", "Windows", "Ref", "t");
        v.in_sesiune(&mut s);
        assert!(Revizuire::din_sesiune(&s).e_deployat(&r));
    }

    #[test]
    fn nota_se_tine_pe_nume_supravietuieste_schimbarii_si_curatarii() {
        let mut v = Revizuire::noua();
        assert!(v.pune_nota("Client1", "dbo.p", "a ramas aici\n"));
        assert!(
            !v.pune_nota("Client1", "dbo.p", "a ramas aici"),
            "acelasi text"
        );
        assert_eq!(v.nota("Client1", "dbo.p"), Some("a ramas aici"));
        assert!(!v.e_goala());
        // obiectul se schimba pe server: bifele cad, nota ramane
        let schimbat = rand_test("a2", "b");
        v.curata(std::slice::from_ref(&schimbat), &["Client1".to_string()]);
        let mut s = Sesiune::noua("srv", "Windows", "Ref", "t");
        v.in_sesiune(&mut s);
        let v2 = Revizuire::din_sesiune(&s);
        assert_eq!(v2.nota("Client1", "dbo.p"), Some("a ramas aici"));
        assert_eq!(v2.note(), vec![("Client1", "dbo.p", "a ramas aici")]);
        assert_eq!(v2.nota("Client2", "dbo.p"), None);
        // textul gol o sterge
        let mut v3 = v2.clone();
        assert!(v3.pune_nota("Client1", "dbo.p", "  \n "));
        assert!(!v3.pune_nota("Client1", "dbo.p", ""));
        assert!(v3.e_goala());
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
            exacta_curent: None,
        }
    }
}
