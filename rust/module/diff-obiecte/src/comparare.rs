//! Compararea unei baze fata de referinta: ce lipseste, ce e in plus, ce e
//! identic, ce difera.
//!
//! **Se strica astfel:** un obiect e pus in categoria greșita — cel mai rau,
//! raportat ca identic cand nu e, fiindca atunci nu-l mai deschide nimeni ca sa
//! vada. De-aia clasificarea trece prin hash **si** prin scor, in ordinea asta.
//!
//! Portat din `_comparatie` din `mostenire/app.py`, cu aceeasi ordine de
//! decizie. Diferenta: aici nu se tin token-ii in memorie pentru fiecare obiect
//! din fiecare baza.

use std::collections::BTreeMap;

use gmotion_core::{calc_hash, normalizeaza, pregateste_afisare, rezumat_rapid, scor_similaritate};
use gmotion_db::Obiect;
use serde::{Deserialize, Serialize};

/// In ce categorie cade un obiect comparat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Categorie {
    /// Exista in referinta, lipseste din baza comparata.
    Lipsa,
    /// Exista in ambele, dar difera.
    Diferit,
    /// Exista doar in baza comparata.
    Extra,
}

impl Categorie {
    pub fn eticheta(self) -> &'static str {
        match self {
            Categorie::Lipsa => "lipsa",
            Categorie::Diferit => "diferit",
            Categorie::Extra => "extra",
        }
    }
}

/// Un rand din lista de rezultate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rand {
    pub id: usize,
    /// `schema.nume`.
    pub obiect: String,
    pub baza: String,
    /// `None` pentru `Lipsa` si `Extra`: nu exista cu ce compara.
    pub scor: Option<f64>,
    pub rezumat: String,
    pub tip: String,
    pub categorie: Categorie,
    /// Hash-urile celor doua versiuni, pentru reluarea unei sesiuni: daca
    /// obiectul s-a schimbat intre timp pe server, progresul salvat nu mai e
    /// valabil pentru el.
    pub hash_referinta: Option<String>,
    pub hash_curent: Option<String>,
}

/// Cat de departe a ajuns compararea. Platforma o publica prin metoda comuna
/// `progres`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progres {
    pub mesaj: String,
    pub procent: u8,
    pub gata: bool,
}

/// Numarul de obiecte pe categorii, pentru rezumatul de sus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Numaratoare {
    pub total_referinta: usize,
    pub identice: usize,
    pub diferite: usize,
    pub lipsa: usize,
    pub extra: usize,
}

/// Forma pregatita a unui obiect: ce e nevoie pentru comparare, calculat o
/// singura data.
///
/// `afisare` se pastreaza fiindca diff-ul se face pe ea, nu pe forma
/// normalizata. `hash` se calculeaza din forma normalizata.
#[derive(Debug, Clone)]
pub struct ObiectPregatit {
    pub tip: String,
    pub afisare: String,
    pub hash: String,
}

impl ObiectPregatit {
    pub fn din(o: &Obiect) -> Self {
        let normalizat = normalizeaza(&o.definitie);
        ObiectPregatit {
            tip: o.tip.clone(),
            afisare: pregateste_afisare(&o.definitie),
            hash: calc_hash(&normalizat),
        }
    }
}

/// Obiectele unei baze, indexate pe `schema.nume`.
pub type Baza = BTreeMap<String, ObiectPregatit>;

pub fn pregateste(obiecte: &[Obiect]) -> Baza {
    obiecte
        .iter()
        .map(|o| (o.id(), ObiectPregatit::din(o)))
        .collect()
}

/// Rezultatul unei comparari complete.
#[derive(Debug, Clone)]
pub struct Comparare {
    pub referinta: String,
    pub baze: Vec<String>,
    pub randuri: Vec<Rand>,
    pub numaratoare: Numaratoare,
}

/// Compara fiecare baza cu referinta.
///
/// Ordinea de decizie, aceeasi ca in versiunea Python, si conteaza:
///
/// 1. obiectul lipseste din baza -> `Lipsa`;
/// 2. hash-urile sunt egale -> identic, nu se mai calculeaza nimic;
/// 3. scorul e 100 -> identic (normalizarea a inghitit diferenta, deci e
///    cosmetica);
/// 4. altfel -> `Diferit`, cu scor si rezumat.
///
/// Pasul 2 inainte de 3 e optimizarea care conteaza: pe baze aproape identice,
/// majoritatea obiectelor se opresc la comparatia de hash-uri.
pub fn compara(
    referinta: &str,
    ref_obiecte: &Baza,
    altele: &[(String, Baza)],
    mut progres: impl FnMut(Progres),
) -> Comparare {
    let mut randuri = Vec::new();
    let mut n = Numaratoare {
        total_referinta: ref_obiecte.len(),
        ..Numaratoare::default()
    };
    let mut id = 0usize;
    let total = ref_obiecte.len().max(1);

    progres(Progres {
        mesaj: format!(
            "Referinta '{referinta}': {} obiecte. Incep compararea...",
            ref_obiecte.len()
        ),
        procent: 5,
        gata: false,
    });

    for (idx, (nume, r)) in ref_obiecte.iter().enumerate() {
        if idx % 1000 == 0 && idx > 0 {
            let procent = 5 + (85 * idx / total) as u8;
            progres(Progres {
                mesaj: format!("Compar obiectele: {idx}/{total}"),
                procent,
                gata: false,
            });
        }
        for (baza, obiecte) in altele {
            let Some(c) = obiecte.get(nume) else {
                n.lipsa += 1;
                randuri.push(Rand {
                    id,
                    obiect: nume.clone(),
                    baza: baza.clone(),
                    scor: None,
                    rezumat: "lipseste complet".into(),
                    tip: r.tip.clone(),
                    categorie: Categorie::Lipsa,
                    hash_referinta: Some(r.hash.clone()),
                    hash_curent: None,
                });
                id += 1;
                continue;
            };
            if c.hash == r.hash {
                n.identice += 1;
                continue;
            }
            let scor = scor_similaritate(&r.afisare, &c.afisare);
            if scor >= 100.0 {
                // Hash-uri diferite dar scor 100: diferenta e cosmetica, adica
                // exact ce normalizarea sterge. Nu merita raportata.
                n.identice += 1;
                continue;
            }
            n.diferite += 1;
            randuri.push(Rand {
                id,
                obiect: nume.clone(),
                baza: baza.clone(),
                scor: Some(scor),
                rezumat: rezumat_rapid(&r.afisare, &c.afisare),
                tip: r.tip.clone(),
                categorie: Categorie::Diferit,
                hash_referinta: Some(r.hash.clone()),
                hash_curent: Some(c.hash.clone()),
            });
            id += 1;
        }
    }

    progres(Progres {
        mesaj: "Caut obiecte care exista doar in bazele comparate...".into(),
        procent: 92,
        gata: false,
    });

    for (baza, obiecte) in altele {
        for (nume, o) in obiecte {
            if ref_obiecte.contains_key(nume) {
                continue;
            }
            n.extra += 1;
            randuri.push(Rand {
                id,
                obiect: nume.clone(),
                baza: baza.clone(),
                scor: None,
                rezumat: "exista doar in aceasta baza".into(),
                tip: o.tip.clone(),
                categorie: Categorie::Extra,
                hash_referinta: None,
                hash_curent: Some(o.hash.clone()),
            });
            id += 1;
        }
    }

    progres(Progres {
        mesaj: format!(
            "Gata. {} obiecte in referinta: {} identice, {} diferite, {} lipsa, {} extra.",
            n.total_referinta, n.identice, n.diferite, n.lipsa, n.extra
        ),
        procent: 100,
        gata: true,
    });

    Comparare {
        referinta: referinta.to_string(),
        baze: altele.iter().map(|(b, _)| b.clone()).collect(),
        randuri,
        numaratoare: n,
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    fn ob(nume: &str, def: &str) -> Obiect {
        Obiect {
            schema: "dbo".into(),
            nume: nume.into(),
            tip: "P".into(),
            definitie: def.into(),
        }
    }

    fn fara_progres() -> impl FnMut(Progres) {
        |_| {}
    }

    #[test]
    fn identic_lipsa_extra_diferit() {
        let r = pregateste(&[
            ob("pA", "CREATE PROC dbo.pA AS SELECT 1"),
            ob("pB", "CREATE PROC dbo.pB AS SELECT 2"),
            ob("pC", "CREATE PROC dbo.pC AS SELECT 3"),
        ]);
        let t = pregateste(&[
            ob("pA", "CREATE PROC dbo.pA AS SELECT 1"),
            ob("pB", "CREATE PROC dbo.pB AS SELECT 999"),
            ob("pZ", "CREATE PROC dbo.pZ AS SELECT 0"),
        ]);

        let c = compara("Ref", &r, &[("T".to_string(), t)], fara_progres());

        assert_eq!(c.numaratoare.identice, 1, "pA");
        assert_eq!(c.numaratoare.diferite, 1, "pB");
        assert_eq!(c.numaratoare.lipsa, 1, "pC");
        assert_eq!(c.numaratoare.extra, 1, "pZ");
        assert_eq!(c.numaratoare.total_referinta, 3);

        let g = |cat| {
            c.randuri
                .iter()
                .filter(|x| x.categorie == cat)
                .map(|x| x.obiect.as_str())
                .collect::<Vec<_>>()
        };
        assert_eq!(g(Categorie::Diferit), ["dbo.pB"]);
        assert_eq!(g(Categorie::Lipsa), ["dbo.pC"]);
        assert_eq!(g(Categorie::Extra), ["dbo.pZ"]);

        // id-urile sunt unice si contigue
        let mut ids: Vec<usize> = c.randuri.iter().map(|x| x.id).collect();
        ids.sort_unstable();
        assert_eq!(ids, (0..c.randuri.len()).collect::<Vec<_>>());
    }

    #[test]
    fn diferenta_cosmetica_nu_se_raporteaza() {
        // Acelasi cod, alta aranjare in pagina si un comentariu in plus:
        // hash-urile difera, dar normalizarea le aduce la aceeasi forma.
        let r = pregateste(&[ob("pA", "CREATE PROC dbo.pA AS\nSELECT a FROM t")]);
        let t = pregateste(&[ob(
            "pA",
            "CREATE PROC dbo.pA AS\n   SELECT   a   FROM t  -- comentariu",
        )]);
        let c = compara("Ref", &r, &[("T".to_string(), t)], fara_progres());
        assert_eq!(c.numaratoare.identice, 1);
        assert_eq!(c.numaratoare.diferite, 0);
        assert!(c.randuri.is_empty(), "nimic de raportat: {:?}", c.randuri);
    }

    #[test]
    fn hash_urile_ajung_in_rand_pentru_reluarea_sesiunii() {
        let r = pregateste(&[ob("pA", "CREATE PROC dbo.pA AS SELECT 1")]);
        let t = pregateste(&[ob("pA", "CREATE PROC dbo.pA AS SELECT 2")]);
        let c = compara("Ref", &r, &[("T".to_string(), t)], fara_progres());
        let x = &c.randuri[0];
        assert!(x.hash_referinta.is_some() && x.hash_curent.is_some());
        assert_ne!(x.hash_referinta, x.hash_curent);
    }

    #[test]
    fn mai_multe_baze_dau_cate_un_rand_fiecare() {
        let r = pregateste(&[ob("pA", "SELECT 1")]);
        let a = pregateste(&[ob("pA", "SELECT 2")]);
        let b = pregateste(&[ob("pA", "SELECT 3")]);
        let c = compara(
            "Ref",
            &r,
            &[("A".to_string(), a), ("B".to_string(), b)],
            fara_progres(),
        );
        assert_eq!(c.randuri.len(), 2);
        assert_eq!(c.baze, ["A", "B"]);
        let baze: Vec<&str> = c.randuri.iter().map(|x| x.baza.as_str()).collect();
        assert_eq!(baze, ["A", "B"]);
    }

    #[test]
    fn progresul_ajunge_la_suta_si_se_termina() {
        let r = pregateste(&[ob("pA", "SELECT 1")]);
        let t = pregateste(&[ob("pA", "SELECT 1")]);
        let mut pasi = Vec::new();
        compara("Ref", &r, &[("T".to_string(), t)], |p| pasi.push(p));
        let ultim = pasi.last().expect("cel putin un pas");
        assert_eq!(ultim.procent, 100);
        assert!(ultim.gata);
        assert!(pasi.iter().all(|p| p.procent <= 100));
    }

    #[test]
    fn referinta_goala_nu_da_panica() {
        let r = pregateste(&[]);
        let t = pregateste(&[ob("pA", "SELECT 1")]);
        let c = compara("Ref", &r, &[("T".to_string(), t)], fara_progres());
        assert_eq!(c.numaratoare.extra, 1);
        assert_eq!(c.numaratoare.total_referinta, 0);
    }

    #[test]
    fn fara_baze_de_comparat_nu_da_randuri() {
        let r = pregateste(&[ob("pA", "SELECT 1")]);
        let c = compara("Ref", &r, &[], fara_progres());
        assert!(c.randuri.is_empty());
        assert_eq!(c.numaratoare.total_referinta, 1);
    }
}
