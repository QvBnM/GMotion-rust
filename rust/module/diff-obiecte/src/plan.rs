//! Planul de deploy: ce intra, cu amprenta fiecarei parti din momentul
//! compararii, si garda „baza n-a mai fost atinsa".
//!
//! **Se strica astfel:** garda spune „neatins" despre un obiect schimbat intre
//! comparare si deploy — si atunci deploy-ul suprascrie, fara ca nimeni sa se
//! fi uitat, munca altcuiva. Sau planul contine alt obiect decat cel ales.
//! De-aia amprenta e **exacta** (textul din baza plus setarile), nu hash-ul
//! normalizat al compararii: o schimbare de spatii sau de `QUOTED_IDENTIFIER`
//! tot o schimbare e, si deploy-ul ar sterge-o.

use std::collections::BTreeMap;
use std::fmt;

use gmotion_core::calc_hash;
use serde::{Deserialize, Serialize};

use crate::comparare::{Baza, Categorie, Comparare, ObiectPregatit, Rand};
use crate::deploy::{aplica_create_or_alter, DeDeployat, PasRevenire};
use crate::eroare::{Eroare, Rezultat};

/// Amprenta exacta a unui obiect: definitia neatinsa si setarile de creare.
///
/// Diferita de `ObiectPregatit::hash`, care e pe forma normalizata si e facut
/// sa nu vada spatiile. Aici tocmai spatiile conteaza: deploy-ul inlocuieste
/// textul exact.
pub fn amprenta_exacta(o: &ObiectPregatit) -> String {
    let setari = match o.setari {
        Some(s) => format!(
            "{}{}",
            u8::from(s.ansi_nulls),
            u8::from(s.quoted_identifier)
        ),
        None => "--".to_string(),
    };
    // Tipul intra si el: acelasi text cu alt tip nu se poate, dar daca se
    // poate, nu e acelasi obiect.
    calc_hash(&format!("{}\0{setari}\0{}", o.tip, o.definitie))
}

/// Un obiect din plan, intr-o baza tinta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pas {
    pub baza: String,
    pub obiect: String,
    pub tip: String,
    /// Amprenta exacta a obiectului in referinta la comparare.
    pub referinta: String,
    /// Amprenta exacta in tinta la comparare; `None` — nu exista acolo, deci
    /// deploy-ul il creeaza si revenirea il sterge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tinta: Option<String>,
    /// Tipul obiectului in tinta, cand exista si difera de `tip`: o procedura
    /// in martor si un view in modificata nu se pot deploia cu `CREATE OR
    /// ALTER` (F-D3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tip_tinta: Option<String>,
    /// Amprenta textului final cu care s-a facut planul (`None`: fara text
    /// final, deci codul din martor). `deploy --plan` o verifica: un text
    /// schimbat dupa plan nu pleaca nevazut (F-D5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_final: Option<String>,
}

/// Ce intra in deploy, cu amprentele de la comparare.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub referinta: String,
    /// Cand s-a facut compararea pe care se sprijina planul. Dat din afara,
    /// ca peste tot in modul.
    pub facut_la: String,
    pub pasi: Vec<Pas>,
    /// Randuri alese dar nedeployabile: `extra` (exista doar in tinta) sau
    /// criptate intr-una din parti.
    pub sarite: usize,
}

/// Care parte s-a schimbat de la comparare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Parte {
    /// Codul care se deployeaza nu mai e cel revizuit.
    Referinta,
    /// Baza pe care se deployeaza a fost atinsa.
    Tinta,
    /// Textul final (din editor sau `gmotion editare`) nu mai e cel din plan.
    #[serde(rename = "text_final")]
    TextFinal,
}

/// Cum s-a schimbat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FelSchimbare {
    Modificat,
    /// Nu exista la comparare, exista acum.
    Aparut,
    /// Exista la comparare, nu mai exista acum.
    Disparut,
}

/// O schimbare gasita de garda. Se spune pe nume, nu „ceva s-a schimbat".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schimbare {
    pub baza: String,
    pub obiect: String,
    pub parte: Parte,
    pub fel: FelSchimbare,
}

impl fmt::Display for Schimbare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fel = match self.fel {
            FelSchimbare::Modificat => "a fost modificat",
            FelSchimbare::Aparut => "a aparut",
            FelSchimbare::Disparut => "a disparut",
        };
        match self.parte {
            Parte::Tinta => write!(f, "{}: {} {fel} de la comparare", self.baza, self.obiect),
            Parte::Referinta => write!(
                f,
                "{} {fel} in Before de la comparare (deploy pe {})",
                self.obiect, self.baza
            ),
            Parte::TextFinal => write!(
                f,
                "textul final al lui {} (deploy pe {}) {fel} de la plan",
                self.obiect, self.baza
            ),
        }
    }
}

/// Obiectele unei baze tinta, dupa nume.
/// Obiectul din baza, cautat intai exact, apoi **fara sa conteze
/// majusculele** — daca varianta e singura.
///
/// Colatia implicita la SQL Server e case-insensitive, deci un
/// `CREATE OR ALTER PROCEDURE dbo.GetX` scris peste un `dbo.getx` existent il
/// **modifica**. Cautarea exacta facea ca planul sa-l creada inexistent:
/// revenirea il stergea, iar codul lui vechi nu ajungea in niciun script
/// (C8 din auditul QA). Cu mai multe variante nu se ghiceste: se cade pe
/// cautarea exacta.
///
/// Cautarea fara majuscule trece printr-un index facut o data pe baza: pana
/// pe 2026-11-25 se parcurgea toata baza pentru fiecare obiect care lipsea
/// din ea, iar planul cu „Toate in plan" pe 12.000 de obiecte tinea firul
/// ferestrei 1,1 s.
struct Cautare<'a> {
    baza: &'a Baza,
    /// numele cu litere mici -> numele exact, `None` daca sunt mai multe
    mici: std::collections::HashMap<String, Option<&'a str>>,
}

impl<'a> Cautare<'a> {
    fn noua(baza: &'a Baza) -> Self {
        let mut mici: std::collections::HashMap<String, Option<&'a str>> =
            std::collections::HashMap::with_capacity(baza.len());
        for n in baza.keys() {
            mici.entry(n.to_ascii_lowercase())
                .and_modify(|v| *v = None)
                .or_insert(Some(n.as_str()));
        }
        Cautare { baza, mici }
    }

    fn gaseste(&self, nume: &str) -> Option<&'a ObiectPregatit> {
        if let Some(o) = self.baza.get(nume) {
            return Some(o);
        }
        let exact = (*self.mici.get(&nume.to_ascii_lowercase())?)?;
        self.baza.get(exact)
    }
}

/// Cautarile pe fiecare baza tinta, facute o data.
fn cautari(altele: &[(String, Baza)]) -> Vec<(&str, Cautare<'_>)> {
    altele
        .iter()
        .map(|(n, b)| (n.as_str(), Cautare::noua(b)))
        .collect()
}

fn in_baza<'a, 'b>(c: &'b [(&str, Cautare<'a>)], baza: &str) -> Option<&'b Cautare<'a>> {
    c.iter().find(|(n, _)| *n == baza).map(|(_, c)| c)
}

impl Plan {
    /// Planul din randurile alese ale unei comparari.
    ///
    /// Intra doar ce se poate deploia: un rand `extra` n-are cod in referinta,
    /// unul criptat n-are cod de citit; amandoua se numara la `sarite`.
    /// Obiectele identice nu sunt randuri, deci nu intra niciodata.
    pub fn din_comparare(
        c: &Comparare,
        referinta: &Baza,
        altele: &[(String, Baza)],
        mut alege: impl FnMut(&Rand) -> bool,
        facut_la: &str,
    ) -> Plan {
        let mut pasi = Vec::new();
        let mut sarite = 0usize;
        let cauta = cautari(altele);
        for r in c.randuri.iter().filter(|r| alege(r)) {
            // tabelele doar se compara (etapa 28, Mihai): nu intra in deploy
            if matches!(r.categorie, Categorie::Extra | Categorie::Criptat)
                || r.tip.trim().eq_ignore_ascii_case("U")
            {
                sarite += 1;
                continue;
            }
            let Some(o) = referinta.get(&r.obiect) else {
                // randurile care nu sunt `extra` vin din referinta
                sarite += 1;
                continue;
            };
            if o.definitie.is_empty() {
                sarite += 1;
                continue;
            }
            let in_tinta = in_baza(&cauta, &r.baza).and_then(|b| b.gaseste(&r.obiect));
            let tinta = in_tinta.map(amprenta_exacta);
            let tip_tinta = in_tinta
                .map(|t| t.tip.trim().to_string())
                .filter(|t| t != r.tip.trim());
            pasi.push(Pas {
                baza: r.baza.clone(),
                obiect: r.obiect.clone(),
                tip: r.tip.clone(),
                referinta: amprenta_exacta(o),
                tinta,
                tip_tinta,
                text_final: None,
            });
        }
        Plan {
            referinta: c.referinta.clone(),
            facut_la: facut_la.to_string(),
            pasi,
            sarite,
        }
    }

    /// Garda: recitite acum, sunt obiectele din plan exact cum erau la
    /// comparare? Lista goala inseamna da.
    ///
    /// `referinta` si `altele` trebuie sa fie **recitite**, nu cele de la
    /// comparare — altfel garda compara un lucru cu el insusi. O baza tinta
    /// care lipseste din `altele` e tratata ca goala: tot ce era in ea a
    /// „disparut", deci garda opreste.
    pub fn verifica(&self, referinta: &Baza, altele: &[(String, Baza)]) -> Vec<Schimbare> {
        let mut schimbari = Vec::new();
        let cauta = cautari(altele);
        for p in &self.pasi {
            let acum_ref = referinta.get(&p.obiect).map(amprenta_exacta);
            if let Some(fel) = fel_schimbare(Some(&p.referinta), acum_ref.as_ref()) {
                schimbari.push(Schimbare {
                    baza: p.baza.clone(),
                    obiect: p.obiect.clone(),
                    parte: Parte::Referinta,
                    fel,
                });
            }
            let acum_tinta = in_baza(&cauta, &p.baza)
                .and_then(|b| b.gaseste(&p.obiect))
                .map(amprenta_exacta);
            if let Some(fel) = fel_schimbare(p.tinta.as_ref(), acum_tinta.as_ref()) {
                schimbari.push(Schimbare {
                    baza: p.baza.clone(),
                    obiect: p.obiect.clone(),
                    parte: Parte::Tinta,
                    fel,
                });
            }
        }
        if !schimbari.is_empty() {
            tracing::warn!(
                target: "gmotion::modul",
                schimbari = schimbari.len(),
                "garda: baza s-a schimbat de la comparare"
            );
        }
        schimbari
    }

    /// Garda ca eroare: `GM-MOD-012` cu fiecare schimbare pe nume.
    /// Pune in plan amprenta textelor finale de acum (cheie: baza, obiect).
    pub fn cu_texte_finale(mut self, texte: &BTreeMap<(String, String), String>) -> Self {
        for p in &mut self.pasi {
            p.text_final = texte
                .get(&(p.baza.clone(), p.obiect.clone()))
                .map(|t| calc_hash(t));
        }
        self
    }

    /// Garda pentru textele finale: sunt cele cu care s-a facut planul? Lista
    /// goala inseamna da. Un text aparut, disparut (Reset) sau schimbat dupa
    /// plan opreste deploy-ul, ca o baza atinsa.
    pub fn verifica_texte(&self, texte: &BTreeMap<(String, String), String>) -> Vec<Schimbare> {
        self.pasi
            .iter()
            .filter_map(|p| {
                let acum = texte
                    .get(&(p.baza.clone(), p.obiect.clone()))
                    .map(|t| calc_hash(t));
                let fel = match (&p.text_final, &acum) {
                    (a, b) if a == b => return None,
                    (None, Some(_)) => FelSchimbare::Aparut,
                    (Some(_), None) => FelSchimbare::Disparut,
                    _ => FelSchimbare::Modificat,
                };
                Some(Schimbare {
                    baza: p.baza.clone(),
                    obiect: p.obiect.clone(),
                    parte: Parte::TextFinal,
                    fel,
                })
            })
            .collect()
    }

    pub fn verifica_sau_opreste(
        &self,
        referinta: &Baza,
        altele: &[(String, Baza)],
    ) -> Rezultat<()> {
        let s = self.verifica(referinta, altele);
        if s.is_empty() {
            return Ok(());
        }
        Err(Eroare::BazaAtinsa {
            schimbari: s
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; "),
        })
    }

    /// Ce se scrie in scriptul de deploy: codul din referinta, pe baze.
    ///
    /// Se cheama dupa garda, cu obiectele recitite; un obiect din plan care
    /// nu mai e in referinta e `GM-MOD-012`, nu un pas sarit in tacere.
    pub fn de_deployat(&self, referinta: &Baza) -> Rezultat<BTreeMap<String, Vec<DeDeployat>>> {
        self.de_deployat_cu(referinta, &BTreeMap::new())
    }

    /// Ca [`Plan::de_deployat`], dar un obiect cu text final (editat in
    /// editor: blocuri aplicate, scris de mana) se deployeaza cu **textul
    /// final**, pe `(baza, schema.nume)`. Setarile raman cele din referinta.
    pub fn de_deployat_cu(
        &self,
        referinta: &Baza,
        texte_finale: &BTreeMap<(String, String), String>,
    ) -> Rezultat<BTreeMap<String, Vec<DeDeployat>>> {
        self.de_deployat_cu_tinte(referinta, &[], texte_finale)
    }

    /// Ca [`Plan::de_deployat_cu`], dar cu bazele tinta la indemana: un text
    /// final egal cu **afisarea tintei** inseamna „pastreaza ce e in baza"
    /// (modul ghidat, „din tinta" la tot), deci obiectul nu intra in script.
    ///
    /// Fara asta, un deploy care dupa decizia omului „nu schimba nimic"
    /// rescria totusi obiectul, cu alte capete de linie si fara spatiile de
    /// la capat — iar la urmatoarea comparare garda il oprea pe un obiect pe
    /// care nu-l atinsese nimeni (A-4 din auditul 2026-11-23). Simetric cu
    /// filtrul de pe referinta, de mai jos.
    pub fn de_deployat_cu_tinte(
        &self,
        referinta: &Baza,
        altele: &[(String, Baza)],
        texte_finale: &BTreeMap<(String, String), String>,
    ) -> Rezultat<BTreeMap<String, Vec<DeDeployat>>> {
        let mut pe_baza: BTreeMap<String, Vec<DeDeployat>> = BTreeMap::new();
        for p in &self.pasi {
            let o = referinta.get(&p.obiect).ok_or_else(|| Eroare::BazaAtinsa {
                schimbari: format!("{} a disparut din Before", p.obiect),
            })?;
            if let Some(t) = &p.tip_tinta {
                return Err(Eroare::TipSchimbat {
                    obiect: p.obiect.clone(),
                    baza: p.baza.clone(),
                    martor: p.tip.trim().to_string(),
                    modificata: t.clone(),
                });
            }
            // un text final gol ar ajunge „criptat" in verificare, cu un mesaj
            // fals; e o greseala de editare, si se spune asa (B2)
            if texte_finale
                .get(&(p.baza.clone(), p.obiect.clone()))
                .is_some_and(|t| t.trim().is_empty())
            {
                return Err(Eroare::TextFinalGol {
                    obiect: p.obiect.clone(),
                });
            }
            // „pastreaza tinta" la tot: nimic de scris pe obiectul asta
            if let Some(t) = texte_finale.get(&(p.baza.clone(), p.obiect.clone())) {
                let la_fel_cu_tinta = altele
                    .iter()
                    .find(|(b, _)| *b == p.baza)
                    .and_then(|(_, baza)| baza.get(&p.obiect))
                    .is_some_and(|o| t.as_str() == gmotion_core::pregateste_afisare(&o.definitie));
                if la_fel_cu_tinta {
                    continue;
                }
            }
            pe_baza.entry(p.baza.clone()).or_default().push(DeDeployat {
                obiect: p.obiect.clone(),
                tip: p.tip.clone(),
                cod: aplica_create_or_alter(
                    texte_finale
                        .get(&(p.baza.clone(), p.obiect.clone()))
                        // un text final egal cu **afisarea** referintei nu e
                        // o editare: „Aplica toate" pune exact textul vazut in
                        // editor, iar afisarea nu are spatiile de la capat de
                        // linie si liniile goale de la capete. Scris asa,
                        // obiectul ar pleca in baza altfel decat referinta cu
                        // care l-am declarat identic (C5 din auditul QA).
                        .filter(|t| t.as_str() != gmotion_core::pregateste_afisare(&o.definitie))
                        .map_or(o.definitie.as_str(), String::as_str),
                    &p.obiect,
                ),
                setari: o.setari,
            });
        }
        Ok(pe_baza)
    }

    /// Scriptul de revenire, din definitiile **actuale** ale bazelor tinta:
    /// ce exista se reface exact cum e acum (cu setarile lui), ce nu exista
    /// — deci deploy-ul il creeaza — se sterge.
    ///
    /// `referinta` da codul care se va deploia peste obiectele noi: din el
    /// se afla in ce ordine se pot sterge.
    pub fn revenire(
        &self,
        referinta: &Baza,
        altele: &[(String, Baza)],
    ) -> BTreeMap<String, Vec<PasRevenire>> {
        let mut pe_baza: BTreeMap<String, Vec<PasRevenire>> = BTreeMap::new();
        let cauta = cautari(altele);
        for p in &self.pasi {
            let pas = match in_baza(&cauta, &p.baza).and_then(|b| b.gaseste(&p.obiect)) {
                Some(o) => PasRevenire::Reface(DeDeployat {
                    obiect: p.obiect.clone(),
                    tip: o.tip.clone(),
                    cod: aplica_create_or_alter(&o.definitie, &p.obiect),
                    setari: o.setari,
                }),
                None => PasRevenire::Sterge(DeDeployat {
                    obiect: p.obiect.clone(),
                    tip: p.tip.clone(),
                    cod: referinta
                        .get(&p.obiect)
                        .map(|o| aplica_create_or_alter(&o.definitie, &p.obiect))
                        .unwrap_or_default(),
                    setari: None,
                }),
            };
            pe_baza.entry(p.baza.clone()).or_default().push(pas);
        }
        pe_baza
    }
}

fn fel_schimbare(atunci: Option<&String>, acum: Option<&String>) -> Option<FelSchimbare> {
    match (atunci, acum) {
        (None, None) => None,
        (Some(a), Some(b)) if a == b => None,
        (Some(_), Some(_)) => Some(FelSchimbare::Modificat),
        (None, Some(_)) => Some(FelSchimbare::Aparut),
        (Some(_), None) => Some(FelSchimbare::Disparut),
    }
}

#[cfg(test)]
mod teste_a4 {
    use super::*;

    /// A-4 din auditul 2026-11-23: „pastreaza tinta" la tot inseamna ca
    /// textul final e chiar afisarea tintei. Obiectul nu mai intra in script:
    /// altfel un deploy care „nu schimba nimic" rescria obiectul cu alte
    /// capete de linie si fara spatiile de la capat, iar la urmatoarea
    /// comparare garda il oprea degeaba.
    #[test]
    fn textul_final_egal_cu_tinta_nu_intra_in_script() {
        let ob = |nume: &str, def: &str| gmotion_db::Obiect {
            schema: "dbo".into(),
            nume: nume.into(),
            tip: "P".into(),
            definitie: def.into(),
            setari: None,
            modificat: None,
            creat: None,
        };
        let referinta = crate::comparare::pregateste(&[ob("pA", "CREATE PROC dbo.pA AS SELECT 1")]);
        // tinta are acelasi cod, dar cu CRLF si un spatiu la capat de linie
        let tinta = crate::comparare::pregateste(&[ob(
            "pA",
            "CREATE PROC dbo.pA AS\r\nSELECT 2   \r\n\r\n",
        )]);
        let altele = vec![("C1".to_string(), tinta.clone())];
        let plan = Plan {
            referinta: "Ref".into(),
            facut_la: "2026-11-24 10:00".into(),
            pasi: vec![Pas {
                baza: "C1".into(),
                obiect: "dbo.pA".into(),
                tip: "P".into(),
                referinta: amprenta_exacta(referinta.get("dbo.pA").expect("in martor")),
                tinta: Some(amprenta_exacta(tinta.get("dbo.pA").expect("in tinta"))),
                tip_tinta: None,
                text_final: None,
            }],
            sarite: 0,
        };
        let def_tinta = &tinta.get("dbo.pA").expect("obiectul e in tinta").definitie;
        let mut texte = BTreeMap::new();
        texte.insert(
            ("C1".to_string(), "dbo.pA".to_string()),
            gmotion_core::pregateste_afisare(def_tinta),
        );
        let pe_baza = plan
            .de_deployat_cu_tinte(&referinta, &altele, &texte)
            .expect("plan valid");
        assert!(
            pe_baza.get("C1").is_none_or(Vec::is_empty),
            "nimic de scris: {pe_baza:?}"
        );
        // iar un text final care chiar difera intra, ca pana acum
        let mut alt = BTreeMap::new();
        alt.insert(
            ("C1".to_string(), "dbo.pA".to_string()),
            "CREATE PROC dbo.pA AS SELECT 3".to_string(),
        );
        let pe_baza = plan
            .de_deployat_cu_tinte(&referinta, &altele, &alt)
            .expect("plan valid");
        assert_eq!(pe_baza.get("C1").map(Vec::len), Some(1));
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use crate::comparare::{compara, pregateste};
    use gmotion_db::{Obiect, Setari};

    fn ob(nume: &str, def: &str) -> Obiect {
        Obiect {
            schema: "dbo".into(),
            nume: nume.into(),
            tip: "P".into(),
            definitie: def.into(),
            setari: Some(Setari {
                ansi_nulls: true,
                quoted_identifier: true,
            }),
            modificat: None,
            creat: None,
        }
    }

    /// Referinta: pA, pB, pC. Tinta: pA diferit, pB lipsa, pC identic, pX extra.
    fn scena() -> (Baza, Vec<(String, Baza)>) {
        let r = pregateste(&[
            ob("pA", "CREATE PROCEDURE dbo.pA AS SELECT 1"),
            ob("pB", "CREATE PROCEDURE dbo.pB AS SELECT 2"),
            ob("pC", "CREATE PROCEDURE dbo.pC AS SELECT 3"),
        ]);
        let t = pregateste(&[
            ob("pA", "CREATE PROCEDURE dbo.pA AS SELECT 100"),
            ob("pC", "CREATE PROCEDURE dbo.pC AS SELECT 3"),
            ob("pX", "CREATE PROCEDURE dbo.pX AS SELECT 9"),
        ]);
        (r, vec![("Tinta".to_string(), t)])
    }

    /// C5 din auditul QA, partea a doua: „Aplica toate" pune ca text final
    /// exact textul **de afisare** al referintei. Daca deploy-ul il scrie ca
    /// text final, obiectul pleaca in baza fara spatiile de la capat de linie
    /// si fara liniile goale de la capete — adica altfel decat referinta cu
    /// care a fost declarat identic. Textul final egal cu afisarea referintei
    /// nu e o editare: se deployeaza definitia neatinsa.
    /// Etapa 28: un tabel diferit nu intra in plan, se numara la sarite.
    #[test]
    fn tabelele_nu_intra_in_plan() {
        let ob = |nume: &str, tip: &str, def: &str| gmotion_db::Obiect {
            schema: "dbo".into(),
            nume: nume.into(),
            tip: tip.into(),
            definitie: def.into(),
            setari: None,
            modificat: None,
            creat: None,
        };
        let r = crate::pregateste(&[
            ob("T", "U", "CREATE TABLE [dbo].[T] (\n    [a] int NULL\n)"),
            ob("p", "P", "CREATE PROC dbo.p AS SELECT 1"),
        ]);
        let a = crate::pregateste(&[
            ob("T", "U", "CREATE TABLE [dbo].[T] (\n    [a] bigint NULL\n)"),
            ob("p", "P", "CREATE PROC dbo.p AS SELECT 2"),
        ]);
        let altele = vec![("C".to_string(), a)];
        let c = crate::compara("R", &r, &altele, |_| {});
        assert_eq!(c.randuri.len(), 2, "tabelul difera si el");
        let p = Plan::din_comparare(&c, &r, &altele, |_| true, "t");
        assert_eq!(p.pasi.len(), 1);
        assert_eq!(p.pasi[0].obiect, "dbo.p");
        assert_eq!(p.sarite, 1);
    }

    #[test]
    fn textul_final_egal_cu_afisarea_referintei_deployeaza_definitia_neatinsa() {
        let def = "CREATE PROCEDURE dbo.pA AS\r\nSELECT 'a   \r\nb'   \r\n\r\n";
        let r = pregateste(&[ob("pA", def)]);
        let t = pregateste(&[ob("pA", "CREATE PROCEDURE dbo.pA AS SELECT 0")]);
        let a = vec![("Tinta".to_string(), t)];
        let p = plan(&r, &a);
        let mut texte = BTreeMap::new();
        texte.insert(
            ("Tinta".to_string(), "dbo.pA".to_string()),
            gmotion_core::pregateste_afisare(def),
        );
        let pe_baza = p.de_deployat_cu(&r, &texte).expect("se poate deploia");
        assert_eq!(
            pe_baza["Tinta"][0].cod,
            crate::deploy::aplica_create_or_alter(def, "dbo.pA"),
            "definitia referintei, octet cu octet"
        );
    }

    /// F-D4: definitia venita din `sys.sql_modules` pastreaza numele dinainte
    /// de `sp_rename`. Planul lucreaza pe numele din `sys.objects`, deci codul
    /// deployat trebuie sa-l creeze pe acela — altfel revenirea sterge un
    /// obiect pe care deploy-ul nu l-a creat.
    #[test]
    fn codul_deployat_creeaza_obiectul_din_plan_nu_pe_cel_din_definitie() {
        let r = pregateste(&[ob("pNou", "CREATE PROCEDURE dbo.pVechi AS SELECT 1")]);
        let t = pregateste(&[ob("pNou", "CREATE PROCEDURE dbo.pVechi AS SELECT 2")]);
        let a = vec![("Tinta".to_string(), t)];
        let p = plan(&r, &a);
        let pe_baza = p.de_deployat(&r).expect("se poate deploia");
        assert!(
            pe_baza["Tinta"][0].cod.contains("[dbo].[pNou]"),
            "{}",
            pe_baza["Tinta"][0].cod
        );
        // si revenirea reface obiectul din plan, nu pe cel din definitie
        let rev = p.revenire(&r, &a);
        let crate::plan::PasRevenire::Reface(d) = &rev["Tinta"][0] else {
            panic!("obiectul exista in tinta, deci se reface");
        };
        assert!(d.cod.contains("[dbo].[pNou]"), "{}", d.cod);
    }

    /// C8 din `cercetare/audit-qa-2026-11-18.md`: referinta are `dbo.GetX`,
    /// tinta `dbo.getx`. Compararea le vede ca lipsa + extra, deploy-ul scrie
    /// `CREATE OR ALTER PROCEDURE dbo.GetX` — iar pe o colatie
    /// case-insensitive (implicita la SQL Server) asta **modifica** `getx`.
    /// Revenirea facea `DROP PROCEDURE [dbo].[GetX]`, deci stergea obiectul
    /// care exista inainte, si codul lui vechi nu era in niciun script.
    #[test]
    fn revenirea_reface_obiectul_care_difera_doar_prin_majuscule() {
        let r = pregateste(&[ob("GetX", "CREATE PROCEDURE dbo.GetX AS SELECT 1")]);
        let t = pregateste(&[ob(
            "getx",
            "CREATE PROCEDURE dbo.getx AS SELECT 999 -- important",
        )]);
        let a = vec![("Tinta".to_string(), t)];
        let c = compara("Ref", &r, &a, |_| {});
        let p = Plan::din_comparare(&c, &r, &a, |x| x.obiect == "dbo.GetX", "t");
        assert_eq!(p.pasi.len(), 1);
        assert!(
            p.pasi[0].tinta.is_some(),
            "obiectul exista in tinta, scris cu alte majuscule"
        );
        // si randul din lista o spune, nu doar „lipseste complet"
        let rand = c
            .randuri
            .iter()
            .find(|x| x.obiect == "dbo.GetX")
            .expect("randul exista");
        assert!(
            rand.rezumat.contains("dbo.getx") && rand.rezumat.contains("majuscule"),
            "{}",
            rand.rezumat
        );
        let rev = p.revenire(&r, &a);
        match &rev["Tinta"][0] {
            PasRevenire::Reface(d) => assert!(
                d.cod.contains("SELECT 999 -- important"),
                "revenirea pune codul care era acolo: {}",
                d.cod
            ),
            PasRevenire::Sterge(_) => {
                panic!("revenirea ar sterge un obiect care exista dinainte")
            }
        }
        // si garda vede acum obiectul: o schimbare pe el opreste scrierea
        let atinsa = pregateste(&[ob("getx", "CREATE PROCEDURE dbo.getx AS SELECT 0")]);
        let schimbari = p.verifica(&r, &[("Tinta".to_string(), atinsa)]);
        assert_eq!(schimbari.len(), 1, "{schimbari:?}");
    }

    fn plan(r: &Baza, a: &[(String, Baza)]) -> Plan {
        let c = compara("Ref", r, a, |_| {});
        Plan::din_comparare(&c, r, a, |_| true, "2026-11-18 10:00")
    }

    #[test]
    fn planul_ia_doar_ce_se_poate_deploia() {
        let (r, a) = scena();
        let p = plan(&r, &a);
        let nume: Vec<&str> = p.pasi.iter().map(|x| x.obiect.as_str()).collect();
        assert_eq!(nume, ["dbo.pA", "dbo.pB"], "pC e identic, pX e extra");
        assert_eq!(p.sarite, 1, "pX");
        assert!(p.pasi[0].tinta.is_some(), "pA exista in tinta");
        assert!(p.pasi[1].tinta.is_none(), "pB nu exista in tinta");
    }

    #[test]
    fn alegerea_filtreaza_randurile() {
        let (r, a) = scena();
        let c = compara("Ref", &r, &a, |_| {});
        let p = Plan::din_comparare(&c, &r, &a, |x| x.obiect == "dbo.pB", "t");
        assert_eq!(p.pasi.len(), 1);
        assert_eq!(p.pasi[0].obiect, "dbo.pB");
        assert_eq!(p.sarite, 0);
    }

    #[test]
    fn neatinsa_trece() {
        let (r, a) = scena();
        let p = plan(&r, &a);
        assert!(p.verifica(&r, &a).is_empty());
        assert!(p.verifica_sau_opreste(&r, &a).is_ok());
    }

    #[test]
    fn tinta_modificata_e_spusa_pe_nume() {
        let (r, mut a) = scena();
        let p = plan(&r, &a);
        a[0].1 = pregateste(&[
            ob("pA", "CREATE PROCEDURE dbo.pA AS SELECT 200"),
            ob("pC", "CREATE PROCEDURE dbo.pC AS SELECT 3"),
        ]);
        let s = p.verifica(&r, &a);
        assert_eq!(
            s,
            vec![Schimbare {
                baza: "Tinta".into(),
                obiect: "dbo.pA".into(),
                parte: Parte::Tinta,
                fel: FelSchimbare::Modificat,
            }]
        );
        let e = p.verifica_sau_opreste(&r, &a).unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-012");
        assert!(
            e.to_string().contains("Tinta: dbo.pA a fost modificat"),
            "{e}"
        );
    }

    #[test]
    fn obiect_aparut_in_tinta_opreste() {
        // pB lipsea: deploy-ul l-ar fi creat. Intre timp l-a creat altcineva.
        let (r, mut a) = scena();
        let p = plan(&r, &a);
        a[0].1.extend(pregateste(&[ob(
            "pB",
            "CREATE PROCEDURE dbo.pB AS SELECT 7",
        )]));
        let s = p.verifica(&r, &a);
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].parte, s[0].fel), (Parte::Tinta, FelSchimbare::Aparut));
    }

    #[test]
    fn obiect_disparut_din_tinta_opreste() {
        let (r, mut a) = scena();
        let p = plan(&r, &a);
        a[0].1.remove("dbo.pA");
        let s = p.verifica(&r, &a);
        assert_eq!(s.len(), 1);
        assert_eq!(
            (s[0].obiect.as_str(), s[0].fel),
            ("dbo.pA", FelSchimbare::Disparut)
        );
    }

    #[test]
    fn referinta_schimbata_opreste() {
        // Codul care s-ar deploia nu mai e cel revizuit.
        let (mut r, a) = scena();
        let p = plan(&r, &a);
        r = pregateste(&[
            ob("pA", "CREATE PROCEDURE dbo.pA AS SELECT 1 -- altfel"),
            ob("pB", "CREATE PROCEDURE dbo.pB AS SELECT 2"),
            ob("pC", "CREATE PROCEDURE dbo.pC AS SELECT 3"),
        ]);
        let s = p.verifica(&r, &a);
        assert_eq!(s.len(), 1);
        assert_eq!(
            (s[0].parte, s[0].fel),
            (Parte::Referinta, FelSchimbare::Modificat)
        );
    }

    #[test]
    fn schimbarea_doar_de_spatii_sau_setari_tot_opreste() {
        // Hash-ul compararii nu le vede; deploy-ul le-ar suprascrie.
        let (r, mut a) = scena();
        let p = plan(&r, &a);
        let mut cu_spatii = ob("pA", "CREATE PROCEDURE dbo.pA AS SELECT 100   ");
        a[0].1 = pregateste(&[
            cu_spatii.clone(),
            ob("pC", "CREATE PROCEDURE dbo.pC AS SELECT 3"),
        ]);
        assert_eq!(p.verifica(&r, &a).len(), 1, "spatiile de la capat");

        cu_spatii.definitie = "CREATE PROCEDURE dbo.pA AS SELECT 100".into();
        cu_spatii.setari = Some(Setari {
            ansi_nulls: true,
            quoted_identifier: false,
        });
        a[0].1 = pregateste(&[cu_spatii, ob("pC", "CREATE PROCEDURE dbo.pC AS SELECT 3")]);
        assert_eq!(p.verifica(&r, &a).len(), 1, "QUOTED_IDENTIFIER");
    }

    /// Cautarea prin index da exact ce dadea parcurgerea: exact intai,
    /// apoi fara majuscule doar daca varianta e singura.
    #[test]
    fn cautarea_fara_majuscule_prin_index() {
        let b = pregateste(&[
            ob("GetX", "CREATE PROC dbo.GetX AS SELECT 1"),
            ob("Dublu", "CREATE PROC dbo.Dublu AS SELECT 1"),
            ob("DUBLU", "CREATE PROC dbo.DUBLU AS SELECT 2"),
        ]);
        let c = Cautare::noua(&b);
        assert!(c.gaseste("dbo.GetX").is_some());
        assert_eq!(
            c.gaseste("dbo.getx").map(|o| o.definitie.as_str()),
            Some("CREATE PROC dbo.GetX AS SELECT 1")
        );
        assert!(
            c.gaseste("dbo.dublu").is_none(),
            "doua variante: nu se ghiceste"
        );
        assert!(c.gaseste("dbo.DUBLU").is_some(), "exact: da");
        assert!(c.gaseste("dbo.lipsa").is_none());
    }

    #[test]
    fn baza_tinta_lipsa_opreste() {
        let (r, a) = scena();
        let p = plan(&r, &a);
        let s = p.verifica(&r, &[]);
        assert_eq!(s.len(), 1, "pA a disparut; pB lipsea si inainte");
        assert_eq!(s[0].fel, FelSchimbare::Disparut);
    }

    #[test]
    fn obiectele_neplanificate_nu_conteaza() {
        // pC (identic) si pX (extra) nu sunt in plan: schimbarea lor nu e
        // treaba deploy-ului.
        let (r, mut a) = scena();
        let p = plan(&r, &a);
        a[0].1.insert(
            "dbo.pX".into(),
            ObiectPregatit::din(&ob("pX", "CREATE PROCEDURE dbo.pX AS SELECT 0")),
        );
        a[0].1.remove("dbo.pC");
        assert!(p.verifica(&r, &a).is_empty());
    }

    #[test]
    fn de_deployat_ia_codul_din_referinta() {
        let (r, a) = scena();
        let p = plan(&r, &a);
        let d = p.de_deployat(&r).unwrap();
        let t = &d["Tinta"];
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].cod, "CREATE OR ALTER PROCEDURE dbo.pA AS SELECT 1");
        assert!(p.de_deployat(&Baza::new()).is_err());
    }

    /// F-D5: un text final schimbat, aparut sau sters dupa plan se vede.
    #[test]
    fn textele_finale_din_plan_se_verifica() {
        let (r, a) = scena();
        let cheie = ("Tinta".to_string(), "dbo.pA".to_string());
        let la_plan = BTreeMap::from([(
            cheie.clone(),
            "CREATE PROCEDURE dbo.pA AS SELECT 7".to_string(),
        )]);
        let p = plan(&r, &a).cu_texte_finale(&la_plan);
        assert!(p.verifica_texte(&la_plan).is_empty());
        let schimbat = BTreeMap::from([(
            cheie.clone(),
            "CREATE PROCEDURE dbo.pA AS DELETE FROM t".to_string(),
        )]);
        let s = p.verifica_texte(&schimbat);
        assert_eq!(
            (s.len(), s[0].parte, s[0].fel),
            (1, Parte::TextFinal, FelSchimbare::Modificat)
        );
        assert!(
            s[0].to_string().contains("textul final al lui dbo.pA"),
            "{}",
            s[0]
        );
        assert_eq!(
            p.verifica_texte(&BTreeMap::new())[0].fel,
            FelSchimbare::Disparut
        );
        let fara = plan(&r, &a);
        assert_eq!(fara.verifica_texte(&la_plan)[0].fel, FelSchimbare::Aparut);
        assert!(fara.verifica_texte(&BTreeMap::new()).is_empty());
    }

    /// F-D3: procedura in martor, view in modificata. Randul o spune, iar
    /// deploy-ul refuza inainte de orice fisier; un plan vechi, fara campul
    /// nou, se citeste la fel.
    #[test]
    fn tipul_schimbat_se_spune_si_opreste_deploy_ul() {
        let r = pregateste(&[ob("pA", "CREATE PROCEDURE dbo.pA AS SELECT 1")]);
        let mut v = ob("pA", "CREATE VIEW dbo.pA AS SELECT 1 AS a");
        v.tip = "V ".into();
        let a = vec![("Tinta".to_string(), pregateste(&[v]))];
        let c = compara("Ref", &r, &a, |_| {});
        assert!(
            c.randuri[0]
                .rezumat
                .starts_with("tip schimbat: P in Before, V"),
            "{}",
            c.randuri[0].rezumat
        );
        let p = plan(&r, &a);
        assert_eq!(p.pasi[0].tip_tinta.as_deref(), Some("V"));
        let e = p.de_deployat_cu(&r, &BTreeMap::new()).unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-017");
        let json = serde_json::to_string(&p).unwrap();
        let vechi = json.replace(",\"tip_tinta\":\"V\"", "");
        assert_ne!(json, vechi);
        let citit: Plan = serde_json::from_str(&vechi).unwrap();
        assert_eq!(citit.pasi[0].tip_tinta, None);
        let (r, a) = scena();
        assert!(plan(&r, &a).pasi.iter().all(|p| p.tip_tinta.is_none()));
    }

    /// B2: un text final gol e spus ca atare, nu ca „criptat".
    #[test]
    fn textul_final_gol_e_refuzat_cu_motivul_lui() {
        let (r, a) = scena();
        let p = plan(&r, &a);
        let texte = BTreeMap::from([(
            ("Tinta".to_string(), "dbo.pA".to_string()),
            "  \n".to_string(),
        )]);
        let e = p.de_deployat_cu(&r, &texte).unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-016");
    }

    #[test]
    fn textul_final_inlocuieste_codul_din_referinta() {
        let (r, a) = scena();
        let p = plan(&r, &a);
        let texte = BTreeMap::from([(
            ("Tinta".to_string(), "dbo.pA".to_string()),
            "ALTER PROCEDURE dbo.pA AS SELECT 'final'".to_string(),
        )]);
        let d = p.de_deployat_cu(&r, &texte).unwrap();
        assert_eq!(
            d["Tinta"][0].cod,
            "CREATE OR ALTER PROCEDURE dbo.pA AS SELECT 'final'"
        );
        assert_eq!(
            d["Tinta"][1].cod, "CREATE OR ALTER PROCEDURE dbo.pB AS SELECT 2",
            "fara text final: codul din referinta"
        );
    }

    #[test]
    fn revenirea_reface_ce_exista_si_sterge_ce_se_creeaza() {
        let (r, a) = scena();
        let p = plan(&r, &a);
        let rev = p.revenire(&r, &a);
        let t = &rev["Tinta"];
        assert_eq!(t.len(), 2);
        match &t[0] {
            PasRevenire::Reface(o) => {
                assert_eq!(o.obiect, "dbo.pA");
                assert_eq!(
                    o.cod, "CREATE OR ALTER PROCEDURE dbo.pA AS SELECT 100",
                    "revenirea e codul din TINTA, nu din referinta"
                );
            }
            alt => panic!("pA trebuia refacut, nu {alt:?}"),
        }
        match &t[1] {
            PasRevenire::Sterge(o) => {
                assert_eq!((o.obiect.as_str(), o.tip.as_str()), ("dbo.pB", "P"))
            }
            alt => panic!("pB trebuia sters, nu {alt:?}"),
        }
    }

    #[test]
    fn planul_supravietuieste_json() {
        let (r, a) = scena();
        let p = plan(&r, &a);
        let text = serde_json::to_string(&p).unwrap();
        let inapoi: Plan = serde_json::from_str(&text).unwrap();
        assert_eq!(inapoi, p);
        assert!(inapoi.verifica(&r, &a).is_empty());
    }
}
