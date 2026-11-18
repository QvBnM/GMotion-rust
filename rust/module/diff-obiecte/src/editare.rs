//! Textul final al unui obiect: ce ajunge in baza la deploy, dupa „Aplica",
//! „Aplica toate", „Reset" si editarea de mana.
//!
//! **Se strica astfel:** deploy-ul scrie alt text decat cel vazut in editor,
//! sau un text facut peste o versiune a bazei care intre timp s-a schimbat.
//! De-aia textul final e legat de **amprenta** obiectului din momentul
//! editarii (ca bifa de revizuire): daca sursa sau baza s-au schimbat de
//! atunci, editarea nu mai e valabila si nu mai ajunge in deploy.
//!
//! Compunerea vine din `gmotion_core::compune`, aceeasi aliniere pe care o
//! deseneaza editorul: blocul aplicat e exact blocul vazut.

use std::collections::BTreeMap;

use gmotion_core::{aliniaza, compune};
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

    /// Textul final, **numai daca** obiectul e acelasi ca la editare.
    pub fn text_final(&self, r: &Rand) -> Option<&TextFinal> {
        self.baze
            .get(&r.baza)
            .and_then(|b| b.get(&r.obiect))
            .filter(|t| t.amprenta == Amprenta::a_randului(r))
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
            },
        );
    }

    /// Aplica blocurile `blocuri` (indexuri in diff-ul dintre sursa si textul
    /// curent, ca in editor) si intoarce textul rezultat.
    pub fn aplica_blocuri(
        &mut self,
        r: &Rand,
        text_sursa: &str,
        text_baza: &str,
        blocuri: &[usize],
    ) -> String {
        let curent = self.curent(r, text_baza).to_string();
        let a = aliniaza(text_sursa, &curent);
        let text = compune(&a, text_sursa, &curent, |b| blocuri.contains(&b));
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

    /// Scoate editarile care nu mai corespund niciunui rand de acum.
    pub fn curata(&mut self, randuri: &[Rand]) -> usize {
        let mut scoase = 0;
        for (baza, obiecte) in self.baze.iter_mut() {
            obiecte.retain(|obiect, t| {
                let pastreaza = randuri.iter().any(|r| {
                    r.baza == *baza && r.obiect == *obiect && Amprenta::a_randului(r) == t.amprenta
                });
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
        }
    }

    const SURSA: &str = "SELECT 1\n\nUPDATE t SET a = 1\n\nDELETE x WHERE id = 1";
    const BAZA: &str = "SELECT 1\n\nUPDATE t SET a = 2\n\nDELETE x WHERE id = 2";

    fn diferente(sursa: &str, curent: &str) -> Vec<usize> {
        aliniaza(sursa, curent).diferente()
    }

    #[test]
    fn un_bloc_aplicat_schimba_doar_blocul_si_se_tine_minte() {
        let r = rand("a", "b");
        let mut e = Editari::noi();
        let d = diferente(SURSA, BAZA);
        assert_eq!(d.len(), 2);
        let text = e.aplica_blocuri(&r, SURSA, BAZA, &[d[0]]);
        assert_eq!(
            text,
            "SELECT 1\n\nUPDATE t SET a = 1\n\nDELETE x WHERE id = 2"
        );
        assert_eq!(e.curent(&r, BAZA), text);
        // al doilea „Aplica" lucreaza pe textul curent: ramane o diferenta
        let d2 = diferente(SURSA, e.curent(&r, BAZA));
        assert_eq!(d2.len(), 1);
        e.aplica_blocuri(&r, SURSA, BAZA, &[d2[0]]);
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
        let text = e.aplica_blocuri(&r, SURSA, BAZA, &[]);
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
        assert_eq!(e.curata(&[rand("a", "b2")]), 1);
    }

    #[test]
    fn manual_ramane_manual_dupa_un_aplica() {
        let r = rand("a", "b");
        let mut e = Editari::noi();
        e.pune(&r, format!("{BAZA}\n-- scris de mine"), true);
        let d = diferente(SURSA, e.curent(&r, BAZA));
        e.aplica_blocuri(&r, SURSA, BAZA, &[d[0]]);
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
