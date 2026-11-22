//! Impactul estimat al deploy-ului unui obiect: cat de mult poate strica in
//! baza comparata daca schimbarea e gresita.
//!
//! **Se strica astfel:** un obiect folosit de jumatate din baza apare
//! „Scazut", iar utilizatorul il trece fara sa se uite. Estimarea e grosiera
//! si o spune: fiecare nivel vine cu motivele lui, ca sa poata fi verificat.
//! Cine foloseste obiectul se decide prin `deploy::referinte` (acelasi lexer
//! ca ordinea de deploy), nu a doua oara aici.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::comparare::{Baza, Categorie, Rand};

/// Nivelul, in ordinea gravitatii.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Nivel {
    Scazut,
    Mediu,
    Ridicat,
}

impl Nivel {
    pub fn eticheta(self) -> &'static str {
        match self {
            Nivel::Scazut => "Scazut",
            Nivel::Mediu => "Mediu",
            Nivel::Ridicat => "Ridicat",
        }
    }
}

/// Nivelul si de ce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Impact {
    pub nivel: Nivel,
    /// Cate obiecte din baza comparata il folosesc.
    pub folosit_de: usize,
    /// Motivele, pe scurt, in ordinea in care conteaza.
    pub motive: Vec<String>,
}

/// Legaturile dintre obiectele unei baze, in ambele sensuri.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dependente {
    /// Pentru fiecare obiect, cine il foloseste (`schema.nume`, in ordine).
    pub folosit_de: BTreeMap<String, Vec<String>>,
    /// Pentru fiecare obiect, ce obiecte din aceeasi baza foloseste.
    pub foloseste: BTreeMap<String, Vec<String>>,
}

/// Legaturile dintre obiectele bazei. Lexeaza tot codul bazei **o data**
/// (prin `deploy::referinte`, acelasi lexer ca ordinea de deploy), deci se
/// calculeaza o singura data si se tine minte de cine il cheama.
pub fn dependente(baza: &Baza) -> Dependente {
    let perechi: Vec<(&str, &str)> = baza
        .iter()
        .map(|(nume, o)| (nume.as_str(), o.definitie.as_str()))
        .collect();
    let mut d = Dependente::default();
    for (i, deps) in crate::deploy::referinte(&perechi).into_iter().enumerate() {
        for j in deps {
            d.folosit_de
                .entry(perechi[j].0.to_string())
                .or_default()
                .push(perechi[i].0.to_string());
            d.foloseste
                .entry(perechi[i].0.to_string())
                .or_default()
                .push(perechi[j].0.to_string());
        }
    }
    for v in d.folosit_de.values_mut().chain(d.foloseste.values_mut()) {
        v.sort();
        v.dedup();
    }
    d
}

/// Pentru fiecare obiect din baza, cine il foloseste. Vezi [`dependente`].
pub fn folosit_de(baza: &Baza) -> BTreeMap<String, Vec<String>> {
    dependente(baza).folosit_de
}

/// Estimeaza impactul deploy-ului randului, stiind cate obiecte din baza
/// comparata il folosesc.
///
/// Puncte: folosit de 1-4 obiecte +1, de 5 sau mai multe +2; schimbare medie
/// (similar 70-95%) +1, mare (sub 70%) +2; trigger +1 (ruleaza la fiecare
/// scriere in tabel, fara sa-l cheme cineva). 0 = Scazut, 1-2 = Mediu,
/// 3+ = Ridicat. Un obiect nou nu inlocuieste nimic, deci schimbarea nu
/// conteaza; unul care nu se deployeaza (doar in baza, criptat) e Scazut.
pub fn estimeaza(r: &Rand, folosit_de: usize) -> Impact {
    let mut motive = Vec::new();
    match r.categorie {
        Categorie::Extra => {
            return Impact {
                nivel: Nivel::Scazut,
                folosit_de,
                motive: vec!["Extra: nu se deployeaza".into()],
            }
        }
        Categorie::Criptat => {
            return Impact {
                nivel: Nivel::Scazut,
                folosit_de,
                motive: vec!["criptat: nu se deployeaza".into()],
            }
        }
        _ => {}
    }

    let mut puncte = 0u8;
    match folosit_de {
        0 => {}
        1 => {
            puncte += 1;
            motive.push("folosit de 1 obiect".into());
        }
        n => {
            puncte += if n >= 5 { 2 } else { 1 };
            motive.push(format!("folosit de {n} obiecte"));
        }
    }
    match (r.categorie, r.scor) {
        (Categorie::Lipsa, _) => motive.push("obiect nou".into()),
        (_, Some(s)) if s < 70.0 => {
            puncte += 2;
            motive.push(format!("schimbare mare ({s:.0}% similar)"));
        }
        (_, Some(s)) if s < 95.0 => {
            puncte += 1;
            motive.push(format!("schimbare medie ({s:.0}% similar)"));
        }
        (_, Some(_)) => motive.push("schimbare mica".into()),
        (_, None) => {}
    }
    if r.tip.trim() == "TR" {
        puncte += 1;
        motive.push("trigger: ruleaza la fiecare scriere".into());
    }
    if folosit_de == 0 && r.categorie != Categorie::Lipsa {
        motive.push("nefolosit de alte obiecte".into());
    }
    let nivel = match puncte {
        0 => Nivel::Scazut,
        1 | 2 => Nivel::Mediu,
        _ => Nivel::Ridicat,
    };
    Impact {
        nivel,
        folosit_de,
        motive,
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use crate::comparare::pregateste;

    fn rand(obiect: &str, tip: &str, categorie: Categorie, scor: Option<f64>) -> Rand {
        Rand {
            id: 1,
            obiect: obiect.into(),
            baza: "B".into(),
            scor,
            rezumat: String::new(),
            tip: tip.into(),
            categorie,
            hash_referinta: None,
            hash_curent: None,
            exacta_curent: None,
        }
    }

    fn obiect(nume: &str, cod: &str) -> gmotion_db::Obiect {
        let (schema, nume) = nume.split_once('.').unwrap_or(("dbo", nume));
        gmotion_db::Obiect {
            schema: schema.into(),
            nume: nume.into(),
            tip: "P".into(),
            definitie: cod.into(),
            setari: None,
        }
    }

    #[test]
    fn cine_foloseste_vine_din_cod_nu_din_comentarii() {
        let baza = pregateste(&[
            obiect(
                "dbo.f",
                "CREATE FUNCTION dbo.f() RETURNS INT AS BEGIN RETURN 1 END",
            ),
            obiect("dbo.a", "CREATE PROC dbo.a AS SELECT dbo.f()"),
            obiect("dbo.b", "CREATE PROC dbo.b AS EXEC a; SELECT [dbo].[f]()"),
            obiect("dbo.c", "CREATE PROC dbo.c AS -- dbo.f\nSELECT 'dbo.a'"),
        ]);
        let cine = folosit_de(&baza);
        assert_eq!(cine["dbo.f"], vec!["dbo.a", "dbo.b"]);
        assert_eq!(cine["dbo.a"], vec!["dbo.b"]);
        assert!(!cine.contains_key("dbo.c"));
        assert!(!cine.contains_key("dbo.b"));
        // si invers: ce foloseste fiecare, o singura data per nume
        let d = dependente(&baza);
        assert_eq!(d.foloseste["dbo.b"], vec!["dbo.a", "dbo.f"]);
        assert_eq!(d.foloseste["dbo.a"], vec!["dbo.f"]);
        assert!(
            !d.foloseste.contains_key("dbo.c"),
            "doar comentarii si siruri"
        );
        assert!(!d.foloseste.contains_key("dbo.f"));
    }

    #[test]
    fn nivelurile() {
        let mic = rand("dbo.p", "P", Categorie::Diferit, Some(98.0));
        assert_eq!(estimeaza(&mic, 0).nivel, Nivel::Scazut);
        assert_eq!(estimeaza(&mic, 3).nivel, Nivel::Mediu);
        assert_eq!(estimeaza(&mic, 5).nivel, Nivel::Mediu);

        let mare = rand("dbo.p", "P", Categorie::Diferit, Some(57.0));
        assert_eq!(estimeaza(&mare, 0).nivel, Nivel::Mediu);
        assert_eq!(estimeaza(&mare, 1).nivel, Nivel::Ridicat);

        let mediu = rand("dbo.p", "P", Categorie::Diferit, Some(80.0));
        assert_eq!(estimeaza(&mediu, 7).nivel, Nivel::Ridicat);

        let trigger = rand("dbo.t", "TR ", Categorie::Diferit, Some(91.0));
        let i = estimeaza(&trigger, 0);
        assert_eq!(i.nivel, Nivel::Mediu);
        assert!(i.motive.iter().any(|m| m.starts_with("trigger")));
    }

    #[test]
    fn obiectele_noi_si_cele_care_nu_se_deployeaza() {
        let nou = rand("dbo.n", "P", Categorie::Lipsa, None);
        assert_eq!(estimeaza(&nou, 0).nivel, Nivel::Scazut);
        assert_eq!(estimeaza(&nou, 0).motive, vec!["obiect nou"]);
        let extra = rand("dbo.x", "P", Categorie::Extra, None);
        assert_eq!(estimeaza(&extra, 9).nivel, Nivel::Scazut);
        let criptat = rand("dbo.x", "P", Categorie::Criptat, None);
        assert_eq!(estimeaza(&criptat, 9).nivel, Nivel::Scazut);
    }

    #[test]
    fn motivele_spun_numarul_si_scorul() {
        let r = rand("dbo.p", "P", Categorie::Diferit, Some(57.4));
        let i = estimeaza(&r, 6);
        assert_eq!(i.folosit_de, 6);
        assert_eq!(
            i.motive,
            vec!["folosit de 6 obiecte", "schimbare mare (57% similar)"]
        );
    }
}
