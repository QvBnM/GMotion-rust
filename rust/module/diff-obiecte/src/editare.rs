//! Textul final al unui obiect: ce ajunge in baza la deploy, dupa „Aplica",
//! „Aplica toate", „Reset" si editarea de mana.
//!
//! **Se strica astfel:** deploy-ul scrie alt text decat cel vazut in editor,
//! sau un text facut peste o versiune a bazei care intre timp s-a schimbat.
//! De-aia textul final e legat de **amprenta** obiectului din momentul
//! editarii (ca bifa de revizuire): daca sursa sau baza s-au schimbat de
//! atunci, editarea nu mai e valabila si nu mai ajunge in deploy.
//!
//! Compunerea vine din `gmotion_core::aplica_blocuri`, aceeasi aliniere si
//! aceeasi plasare pe care le deseneaza editorul: blocul aplicat ajunge exact
//! unde l-a aratat ecranul.

use std::collections::BTreeMap;

use gmotion_platforma::Sesiune;
use serde::{Deserialize, Serialize};

use crate::comparare::Rand;
use crate::revizuire::Amprenta;

/// Cheia sub care sta starea in `Sesiune::date`.
pub const CHEIE_SESIUNE: &str = "editari";

/// Textul final al unui obiect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextFinal {
    pub text: String,
    /// Scris de mana (nu doar blocuri aplicate din sursa). Editorul marcheaza
    /// altfel liniile scrise de tine.
    #[serde(default)]
    pub manual: bool,
    /// Amprenta obiectului cand s-a facut editarea.
    pub amprenta: Amprenta,
    /// Amprenta exacta a bazei comparate cand s-a facut editarea. Amprenta
    /// de mai sus e pe textul normalizat, deci nu vede un comentariu adaugat
    /// in baza (un hotfix); textul final, facut peste versiunea veche, l-ar
    /// fi sters la deploy (F-D5). `None` in sesiunile de dinainte.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exacta: Option<String>,
}

/// Textele finale, pe baza comparata si obiect.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Editari {
    #[serde(default)]
    baze: BTreeMap<String, BTreeMap<String, TextFinal>>,
}

impl Editari {
    pub fn noi() -> Self {
        Self::default()
    }

    /// Unirea in trei cai, ca [`crate::Revizuire::uneste`]: textele scrise
    /// din cli cat fereastra era deschisa nu se mai pierd la salvare.
    pub fn uneste(baza: &Self, ai_mei: &Self, ai_lor: &Self) -> Self {
        Editari {
            baze: crate::revizuire::uneste_harta(&baza.baze, &ai_mei.baze, &ai_lor.baze),
        }
    }

    /// Textul final, **numai daca** obiectul e acelasi ca la editare.
    pub fn text_final(&self, r: &Rand) -> Option<&TextFinal> {
        self.baze
            .get(&r.baza)
            .and_then(|b| b.get(&r.obiect))
            .filter(|t| t.amprenta == Amprenta::a_randului(r))
            .filter(|t| match (&t.exacta, &r.exacta_curent) {
                (Some(a), Some(b)) => a == b,
                _ => true,
            })
    }

    /// Textul din dreapta pe care lucreaza editorul acum: textul final, daca
    /// exista si e valabil, altfel textul bazei.
    pub fn curent<'a>(&'a self, r: &Rand, text_baza: &'a str) -> &'a str {
        self.text_final(r).map_or(text_baza, |t| t.text.as_str())
    }

    /// Pune textul final.
    ///
    /// Si cand e exact textul bazei: aia e decizia „pastreaza tinta" (modul
    /// ghidat cu „din tinta" la toate diferentele), si trebuie sa ajunga la
    /// deploy. Pana la auditul QA se stergea, ca „nu schimba nimic" — iar
    /// deploy-ul, fara text final, scria **toata sursa** peste exact ce
    /// alesesei sa pastrezi (F-A5). „Reset" e singurul care sterge decizia.
    pub fn pune(&mut self, r: &Rand, text: String, manual: bool) {
        let manual = manual || self.text_final(r).is_some_and(|t| t.manual);
        self.baze.entry(r.baza.clone()).or_default().insert(
            r.obiect.clone(),
            TextFinal {
                text,
                manual,
                amprenta: Amprenta::a_randului(r),
                exacta: r.exacta_curent.clone(),
            },
        );
    }

    /// Aplica blocurile `blocuri` (indexuri in diff-ul dintre sursa si textul
    /// curent, ca in editor, cu aceeasi optiune `uneste_cursor`) acolo unde
    /// spune plasarea (`gmotion_core::aplica_blocuri`) si intoarce textul.
    pub fn aplica_blocuri(
        &mut self,
        r: &Rand,
        text_sursa: &str,
        text_baza: &str,
        uneste_cursor: bool,
        blocuri: &[usize],
    ) -> String {
        let curent = self.curent(r, text_baza).to_string();
        let text = gmotion_core::aplica_blocuri(text_sursa, &curent, uneste_cursor, blocuri);
        self.pune(r, text.clone(), false);
        text
    }

    /// Ca [`Self::aplica_blocuri`], dar blocurile merg unde le-a lasat omul
    /// (`gmotion_core::aplica_blocuri_la`): tragerea pe alt loc decat cel
    /// sugerat.
    pub fn aplica_blocuri_la(
        &mut self,
        r: &Rand,
        text_sursa: &str,
        text_baza: &str,
        uneste_cursor: bool,
        blocuri: &[usize],
        loc: gmotion_core::Loc,
    ) -> String {
        let curent = self.curent(r, text_baza).to_string();
        let text =
            gmotion_core::aplica_blocuri_la(text_sursa, &curent, uneste_cursor, blocuri, loc);
        self.pune(r, text.clone(), false);
        text
    }

    /// „Aplica toate": textul exact al sursei, nu o compunere — altfel
    /// comentariile si liniile goale dintre blocuri ar ramane din baza.
    pub fn aplica_toate(&mut self, r: &Rand, text_sursa: &str) {
        self.pune(r, text_sursa.to_string(), false);
    }

    /// Pune inapoi o stare luata inainte cu [`Editari::text_final`] (Ctrl+Z):
    /// `None` inseamna ca nu exista text final.
    pub fn restaureaza(&mut self, r: &Rand, inainte: Option<TextFinal>) {
        match inainte {
            Some(t) => {
                self.baze
                    .entry(r.baza.clone())
                    .or_default()
                    .insert(r.obiect.clone(), t);
            }
            None => {
                self.reset(r);
            }
        }
    }

    /// „Reset": inapoi la textul bazei. Intoarce `true` daca era ceva.
    pub fn reset(&mut self, r: &Rand) -> bool {
        let scos = self
            .baze
            .get_mut(&r.baza)
            .is_some_and(|b| b.remove(&r.obiect).is_some());
        self.baze.retain(|_, b| !b.is_empty());
        scos
    }

    /// Scoate editarile care nu mai corespund niciunui rand de acum, **doar
    /// pentru bazele din compararea de acum** (`baze_comparate`).
    ///
    /// O baza care nu e comparata acum n-are randuri: fara filtrul pe baze,
    /// textele finale scrise alta data pentru ea se stergeau toate, iar
    /// deploy-ul ei scria codul referintei in locul lor (C3/C4 din auditul QA).
    pub fn curata(&mut self, randuri: &[Rand], baze_comparate: &[String]) -> usize {
        let mut scoase = 0;
        // index, nu `any` pe toate randurile pentru fiecare text (P6)
        let index: std::collections::HashMap<(&str, &str), Amprenta> = randuri
            .iter()
            .map(|r| {
                (
                    (r.baza.as_str(), r.obiect.as_str()),
                    Amprenta::a_randului(r),
                )
            })
            .collect();
        for (baza, obiecte) in self.baze.iter_mut() {
            if !baze_comparate.iter().any(|b| b == baza) {
                continue;
            }
            obiecte.retain(|obiect, t| {
                // un text final pe alta versiune ramane cat obiectul e in
                // comparare: deploy-ul spune ca a cazut (`cazute`), in loc sa
                // dispara tacut la prima salvare (N5); Reset il sterge
                let _ = &t.amprenta;
                let pastreaza = index.contains_key(&(baza.as_str(), obiect.as_str()));
                scoase += usize::from(!pastreaza);
                pastreaza
            });
        }
        self.baze.retain(|_, b| !b.is_empty());
        scoase
    }

    pub fn numar(&self) -> usize {
        self.baze.values().map(BTreeMap::len).sum()
    }

    /// Textele finale valabile pentru randurile date, pe `(baza, obiect)`:
    /// ce primeste deploy-ul.
    pub fn pentru_deploy(&self, randuri: &[Rand]) -> BTreeMap<(String, String), String> {
        randuri
            .iter()
            .filter_map(|r| {
                self.text_final(r)
                    .map(|t| ((r.baza.clone(), r.obiect.clone()), t.text.clone()))
            })
            .collect()
    }

    /// Citeste starea dintr-o sesiune; una lipsa sau de neinteles da o stare
    /// goala, ca la revizuire.
    /// Ca [`Self::din_sesiune`], dar o stare care exista si nu se poate
    /// citi e eroare, nu stare goala.
    pub fn din_sesiune_verificat(s: &Sesiune) -> crate::Rezultat<Self> {
        match s.date.get(CHEIE_SESIUNE) {
            None => Ok(Self::default()),
            Some(v) => serde_json::from_value(v.clone()).map_err(|e| {
                crate::Eroare::Date(format!("textele finale din sesiune nu se pot citi: {e}"))
            }),
        }
    }

    /// Textele finale facute pe o versiune care s-a schimbat de atunci: nu
    /// mai ajung in deploy, dar nici nu trebuie sa dispara tacut (N5).
    pub fn cazute(&self, randuri: &[Rand]) -> Vec<(String, String)> {
        randuri
            .iter()
            .filter(|r| {
                self.baze
                    .get(&r.baza)
                    .is_some_and(|b| b.contains_key(&r.obiect))
                    && self.text_final(r).is_none()
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

    pub fn in_sesiune(&self, s: &mut Sesiune) {
        match serde_json::to_value(self) {
            Ok(v) if self.numar() > 0 => {
                s.date.insert(CHEIE_SESIUNE.to_string(), v);
            }
            _ => {
                s.date.remove(CHEIE_SESIUNE);
            }
        }
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use crate::comparare::Categorie;

    fn rand(hr: &str, hc: &str) -> Rand {
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
            modificat: None,
            creat: None,
            exacta_curent: None,
        }
    }

    #[test]
    fn textul_final_cazut_se_vede() {
        let mut r = rand("h1", "h2");
        let mut e = Editari::noi();
        e.pune(&r, "CREATE PROCEDURE dbo.p AS SELECT 1".into(), true);
        assert!(e.cazute(std::slice::from_ref(&r)).is_empty());
        r.hash_curent = Some("h3".into());
        assert_eq!(e.cazute(&[r]).len(), 1);
    }

    /// F-D5: un comentariu adaugat in baza (hash normalizat neschimbat) face
    /// textul final facut peste versiunea veche nevalabil.
    #[test]
    fn un_comentariu_nou_in_baza_invalideaza_textul_final() {
        let mut r = rand("h1", "h2");
        r.exacta_curent = Some("e1".into());
        let mut e = Editari::noi();
        e.pune(&r, "CREATE PROCEDURE dbo.p AS SELECT 1".into(), true);
        assert!(e.text_final(&r).is_some());
        r.exacta_curent = Some("e2-cu-hotfix".into());
        assert!(e.text_final(&r).is_none(), "baza s-a schimbat dupa editare");
        // sesiunile vechi n-au amprenta exacta: raman valabile ca inainte
        let mut vechi = Editari::noi();
        r.exacta_curent = None;
        vechi.pune(&r, "x".into(), true);
        r.exacta_curent = Some("e3".into());
        assert!(vechi.text_final(&r).is_some());
    }

    const SURSA: &str = "SELECT 1\n\nUPDATE t SET a = 1\n\nDELETE x WHERE id = 1";
    const BAZA: &str = "SELECT 1\n\nUPDATE t SET a = 2\n\nDELETE x WHERE id = 2";

    fn diferente(sursa: &str, curent: &str) -> Vec<usize> {
        gmotion_core::aliniaza(sursa, curent).diferente()
    }

    #[test]
    fn un_bloc_aplicat_schimba_doar_blocul_si_se_tine_minte() {
        let r = rand("a", "b");
        let mut e = Editari::noi();
        let d = diferente(SURSA, BAZA);
        assert_eq!(d.len(), 2);
        let text = e.aplica_blocuri(&r, SURSA, BAZA, false, &[d[0]]);
        assert_eq!(
            text,
            "SELECT 1\n\nUPDATE t SET a = 1\n\nDELETE x WHERE id = 2"
        );
        assert_eq!(e.curent(&r, BAZA), text);
        // al doilea „Aplica" lucreaza pe textul curent: ramane o diferenta
        let d2 = diferente(SURSA, e.curent(&r, BAZA));
        assert_eq!(d2.len(), 1);
        e.aplica_blocuri(&r, SURSA, BAZA, false, &[d2[0]]);
        assert_eq!(e.curent(&r, BAZA), SURSA);
    }

    #[test]
    fn aplica_toate_e_textul_exact_al_sursei_si_reset_il_scoate() {
        let r = rand("a", "b");
        let mut e = Editari::noi();
        e.aplica_toate(&r, SURSA);
        assert_eq!(e.curent(&r, BAZA), SURSA);
        assert!(e.reset(&r));
        assert_eq!(e.curent(&r, BAZA), BAZA);
        assert_eq!(e.numar(), 0);
    }

    #[test]
    fn restaurarea_pune_inapoi_starea_de_dinainte() {
        let r = rand("a", "b");
        let mut e = Editari::noi();
        let inainte = e.text_final(&r).cloned();
        e.aplica_toate(&r, SURSA);
        let dupa_toate = e.text_final(&r).cloned();
        e.reset(&r);
        e.restaureaza(&r, dupa_toate);
        assert_eq!(e.curent(&r, BAZA), SURSA);
        e.restaureaza(&r, inainte);
        assert_eq!(e.curent(&r, BAZA), BAZA);
        assert_eq!(e.numar(), 0);
    }

    /// F-A5 din `cercetare/audit-qa-2026-11-18.md`: in modul ghidat, „din
    /// tinta" la toate diferentele da un text final egal cu textul bazei.
    /// Ala se stergea („nu schimba nimic"), deci deploy-ul nu mai avea text
    /// final si scria **toata sursa** peste exact ce ai ales sa pastrezi.
    #[test]
    fn decizia_de_a_pastra_tinta_ajunge_la_deploy() {
        let r = rand("a", "b");
        let mut e = Editari::noi();
        // „din tinta" la toate diferentele: compunerea da exact textul bazei
        let text = e.aplica_blocuri(&r, SURSA, BAZA, false, &[]);
        assert_eq!(text, BAZA, "nimic adus din sursa");
        assert_eq!(
            e.pentru_deploy(std::slice::from_ref(&r))
                .get(&("Client1".to_string(), "dbo.p".to_string()))
                .map(String::as_str),
            Some(BAZA),
            "decizia trebuie sa ajunga la deploy; altfel se scrie toata sursa"
        );
        // „Reset" e singurul care sterge decizia
        e.reset(&r);
        assert_eq!(e.numar(), 0);
        assert!(e.pentru_deploy(std::slice::from_ref(&r)).is_empty());
    }

    #[test]
    fn editarea_cade_cand_obiectul_s_a_schimbat() {
        let mut e = Editari::noi();
        e.aplica_toate(&rand("a", "b"), SURSA);
        assert!(e.text_final(&rand("a", "b")).is_some());
        assert!(
            e.text_final(&rand("a", "b2")).is_none(),
            "baza s-a schimbat"
        );
        assert!(
            e.text_final(&rand("a2", "b")).is_none(),
            "sursa s-a schimbat"
        );
        assert_eq!(e.curent(&rand("a", "b2"), BAZA), BAZA);
        // ramane, ca deploy-ul sa poata spune ca a cazut (N5); dispare doar
        // cand obiectul nu mai e in comparare
        assert_eq!(e.curata(&[rand("a", "b2")], &["Client1".to_string()]), 0);
        assert_eq!(e.cazute(&[rand("a", "b2")]).len(), 1);
        assert_eq!(e.curata(&[], &["Client1".to_string()]), 1);
    }

    #[test]
    fn manual_ramane_manual_dupa_un_aplica() {
        let r = rand("a", "b");
        let mut e = Editari::noi();
        e.pune(&r, format!("{BAZA}\n-- scris de mine"), true);
        let d = diferente(SURSA, e.curent(&r, BAZA));
        e.aplica_blocuri(&r, SURSA, BAZA, false, &[d[0]]);
        assert!(e.text_final(&r).is_some_and(|t| t.manual));
    }

    #[test]
    fn supravietuieste_sesiunii_si_pentru_deploy_da_doar_cele_valabile() {
        let r = rand("a", "b");
        let mut e = Editari::noi();
        e.aplica_toate(&r, SURSA);
        let mut s = Sesiune::noua("srv", "Windows", "Ref", "t");
        e.in_sesiune(&mut s);
        let inapoi = Editari::din_sesiune(&s);
        assert_eq!(inapoi, e);
        let d = inapoi.pentru_deploy(std::slice::from_ref(&r));
        assert_eq!(d[&("Client1".to_string(), "dbo.p".to_string())], SURSA);
        assert!(inapoi.pentru_deploy(&[rand("a", "schimbat")]).is_empty());
        // o sesiune fara editari nu poarta cheia degeaba
        Editari::noi().in_sesiune(&mut s);
        assert!(!s.date.contains_key(CHEIE_SESIUNE));
    }
}
