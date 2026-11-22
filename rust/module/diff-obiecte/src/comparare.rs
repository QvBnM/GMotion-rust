//! Compararea unei baze fata de referinta: ce lipseste, ce e in plus, ce e
//! identic, ce difera.
//!
//! **Se strica astfel:** un obiect e pus in categoria gresita — cel mai rau,
//! raportat ca identic cand nu e, fiindca atunci nu-l mai deschide nimeni ca sa
//! vada. De-aia clasificarea trece prin hash **si** prin scor, in ordinea asta.
//!
//! Portat din `_comparatie` din `mostenire/app.py`, cu aceeasi ordine de
//! decizie. Diferenta: aici nu se tin token-ii in memorie pentru fiecare obiect
//! din fiecare baza.

use std::collections::BTreeMap;

use gmotion_core::{
    calc_hash, hinturi, normalizeaza, pregateste_afisare, rezumat_rapid, scor_similaritate,
};
use gmotion_db::{Obiect, Setari};
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
    /// Criptat (`WITH ENCRYPTION`) in referinta sau in baza comparata: codul
    /// nu se poate citi, deci nu se compara si nu se deployeaza. Pana la audit
    /// obiectele criptate se sareau la citire, deci in tinta pareau „lipsa" si
    /// deploy-ul le-ar fi suprascris.
    Criptat,
}

impl Categorie {
    pub fn eticheta(self) -> &'static str {
        match self {
            Categorie::Lipsa => "lipsa",
            Categorie::Diferit => "diferit",
            Categorie::Extra => "extra",
            Categorie::Criptat => "criptat",
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
    pub criptate: usize,
}

/// Forma pregatita a unui obiect: ce e nevoie pentru comparare, calculat o
/// singura data.
///
/// `afisare` se pastreaza fiindca diff-ul se face pe ea, nu pe forma
/// normalizata. `hash` se calculeaza din forma normalizata.
#[derive(Debug, Clone)]
pub struct ObiectPregatit {
    pub tip: String,
    /// Textul exact din baza. **Asta** se deployeaza: `afisare` are spatiile
    /// de la capat de linie si `\r\n` schimbate, inclusiv in string-uri (gasit
    /// la audit; v3.4 deploya tot `afisare`).
    pub definitie: String,
    /// `ANSI_NULLS` / `QUOTED_IDENTIFIER` cu care a fost creat; se refac la
    /// deploy.
    pub setari: Option<Setari>,
    pub afisare: String,
    pub hash: String,
    /// Hinturile de blocare (`gmotion_core::hinturi`): normalizarea le sterge,
    /// deci un obiect care difera doar prin ele ar iesi identic (F-M3).
    pub hinturi: String,
}

impl ObiectPregatit {
    pub fn din(o: &Obiect) -> Self {
        let normalizat = normalizeaza(&o.definitie);
        ObiectPregatit {
            tip: o.tip.clone(),
            definitie: o.definitie.clone(),
            setari: o.setari,
            afisare: pregateste_afisare(&o.definitie),
            hash: calc_hash(&normalizat),
            hinturi: hinturi(&o.definitie),
        }
    }
}

/// Setarile de sesiune care difera intre doua versiuni ale aceluiasi obiect.
///
/// Nu se vad in text, dar schimba comportamentul la executie: pe serverul
/// real, ~21% din obiecte aveau `QUOTED_IDENTIFIER OFF`, iar un deploy rulat
/// din SSMS le-ar fi schimbat. Daca setarile unei parti nu se stiu
/// (instantaneu vechi), nu se afirma nimic: lista goala.
pub fn setari_diferite(a: Option<Setari>, b: Option<Setari>) -> Vec<&'static str> {
    let (Some(a), Some(b)) = (a, b) else {
        return Vec::new();
    };
    let mut v = Vec::new();
    if a.quoted_identifier != b.quoted_identifier {
        v.push("QUOTED_IDENTIFIER");
    }
    if a.ansi_nulls != b.ansi_nulls {
        v.push("ANSI_NULLS");
    }
    v
}

/// Ce difera intre doua versiuni fara sa se vada in textul normalizat: setarile
/// de creare si hinturile de blocare (`hinturi`). Apare in „Difera in" ca o
/// eticheta proprie, ca setarile.
pub fn diferente_ascunse(a: &ObiectPregatit, b: &ObiectPregatit) -> Vec<&'static str> {
    let mut v = setari_diferite(a.setari, b.setari);
    if a.hinturi != b.hinturi {
        v.push("hinturi");
    }
    v
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

/// Scorul unui obiect diferit nu poate fi afisat 100: rotunjirea la doua
/// zecimale ar spune „identic" despre ceva care nu e.
fn scor_afisat(scor: f64) -> f64 {
    if scor >= 100.0 {
        99.99
    } else {
        scor
    }
}

/// Compara fiecare baza cu referinta.
///
/// Ordinea de decizie, aceeasi ca in versiunea Python, si conteaza:
///
/// 1. obiectul lipseste din baza -> `Lipsa`;
/// 2. hash-urile formelor normalizate sunt egale -> identic;
/// 3. altfel -> `Diferit`, cu scor si rezumat.
///
/// Versiunea Python mai avea un pas: „scor 100 -> identic". Scorul e rotunjit
/// la doua zecimale, deci pe obiecte mari (10.000+ token-uri) un `NOT` sters
/// dadea 99,995 -> 100 -> „identic", si modificarea nu ajungea nici in lista,
/// nici in deploy. Scos la auditul din 2026-11-17; hash-ul e deja calculat pe
/// forma normalizata, deci ce e cosmetic e prins la pasul 2.
///
/// Pasul 2 inainte de 3 e optimizarea care conteaza: pe baze aproape identice,
/// majoritatea obiectelor se opresc la comparatia de hash-uri.
pub fn compara(
    referinta: &str,
    ref_obiecte: &Baza,
    altele: &[(String, Baza)],
    progres: impl FnMut(Progres),
) -> Comparare {
    match compara_cu_oprire(referinta, ref_obiecte, altele, progres, || true) {
        Some(c) => c,
        None => unreachable!("fara oprire, compararea ajunge la capat"),
    }
}

/// Ca [`compara`], dar se opreste daca `continua()` spune `false`: se
/// intreaba inainte de fiecare obiect din referinta, deci oprirea vine cel
/// mult dupa un obiect. `None` = oprita.
///
/// Fara asta, „Anuleaza" din fereastra intorcea ecranul pe loc, dar firul
/// compararii ardea procesorul pana la capat (P8 din auditul QA).
pub fn compara_cu_oprire(
    referinta: &str,
    ref_obiecte: &Baza,
    altele: &[(String, Baza)],
    mut progres: impl FnMut(Progres),
    continua: impl Fn() -> bool,
) -> Option<Comparare> {
    let mut randuri = Vec::new();
    let mut n = Numaratoare {
        total_referinta: ref_obiecte.len(),
        ..Numaratoare::default()
    };
    let mut id = 0usize;
    let total = ref_obiecte.len().max(1);

    progres(Progres {
        mesaj: format!(
            "Martor '{referinta}': {} obiecte. Incep compararea...",
            ref_obiecte.len()
        ),
        procent: 5,
        gata: false,
    });

    for (idx, (nume, r)) in ref_obiecte.iter().enumerate() {
        if !continua() {
            tracing::info!(target: "gmotion::modul", facute = idx, "compararea a fost oprita");
            return None;
        }
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
                // o varianta care difera doar prin majuscule exista in baza:
                // pe o colatie case-insensitive (implicita la SQL Server)
                // deploy-ul o va **modifica**, nu va crea un obiect nou, deci
                // randul trebuie s-o spuna (C8 din auditul QA)
                let varianta = obiecte
                    .keys()
                    .find(|n| n.eq_ignore_ascii_case(nume))
                    .cloned();
                randuri.push(Rand {
                    id,
                    obiect: nume.clone(),
                    baza: baza.clone(),
                    scor: None,
                    rezumat: match &varianta {
                        Some(v) => format!("lipseste; in baza exista {v} (difera majusculele)"),
                        None => "lipseste complet".to_string(),
                    },
                    tip: r.tip.clone(),
                    categorie: Categorie::Lipsa,
                    hash_referinta: Some(r.hash.clone()),
                    hash_curent: None,
                });
                id += 1;
                continue;
            };
            if r.definitie.is_empty() || c.definitie.is_empty() {
                n.criptate += 1;
                randuri.push(Rand {
                    id,
                    obiect: nume.clone(),
                    baza: baza.clone(),
                    scor: None,
                    rezumat: "criptat: nu se poate compara".into(),
                    tip: r.tip.clone(),
                    categorie: Categorie::Criptat,
                    hash_referinta: Some(r.hash.clone()),
                    hash_curent: Some(c.hash.clone()),
                });
                id += 1;
                continue;
            }
            if c.hash == r.hash {
                // acelasi text, dar setarile de creare pot diferi: cu
                // `QUOTED_IDENTIFIER OFF` un `"a"` e sir, cu ON e coloana.
                // Nu se vede in text, deci pana la auditul QA (C6) obiectul
                // era numarat identic si nu se putea deploia.
                let setari = diferente_ascunse(r, c);
                if setari.is_empty() {
                    n.identice += 1;
                    continue;
                }
                n.diferite += 1;
                randuri.push(Rand {
                    id,
                    obiect: nume.clone(),
                    baza: baza.clone(),
                    // textul e acelasi: un procent de similaritate ar minti
                    scor: None,
                    rezumat: format!("difera doar in: {}", setari.join(", ")),
                    tip: r.tip.clone(),
                    categorie: Categorie::Diferit,
                    hash_referinta: Some(r.hash.clone()),
                    hash_curent: Some(c.hash.clone()),
                });
                id += 1;
                continue;
            }
            let scor = scor_afisat(scor_similaritate(&r.afisare, &c.afisare));
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
            "Gata. {} obiecte in martor: {} identice, {} diferite, {} lipsa, {} extra.",
            n.total_referinta, n.identice, n.diferite, n.lipsa, n.extra
        ),
        procent: 100,
        gata: true,
    });

    Some(Comparare {
        referinta: referinta.to_string(),
        baze: altele.iter().map(|(b, _)| b.clone()).collect(),
        randuri,
        numaratoare: n,
    })
}

#[cfg(test)]
mod teste {
    use super::*;
    use gmotion_core::normalizare::tokeni_scor;

    fn ob(nume: &str, def: &str) -> Obiect {
        Obiect {
            schema: "dbo".into(),
            nume: nume.into(),
            tip: "P".into(),
            definitie: def.into(),
            setari: None,
        }
    }

    fn fara_progres() -> impl FnMut(Progres) {
        |_| {}
    }

    #[test]
    fn compararea_se_opreste_cand_i_se_cere() {
        let r = pregateste(&[
            ob("p1", "SELECT 1"),
            ob("p2", "SELECT 2"),
            ob("p3", "SELECT 3"),
        ]);
        let c = pregateste(&[
            ob("p1", "SELECT 10"),
            ob("p2", "SELECT 20"),
            ob("p3", "SELECT 30"),
        ]);
        let altele = vec![("C".to_string(), c)];
        let intrebari = std::cell::Cell::new(0);
        let oprita = compara_cu_oprire("R", &r, &altele, fara_progres(), || {
            intrebari.set(intrebari.get() + 1);
            intrebari.get() <= 1
        });
        assert!(oprita.is_none());
        assert_eq!(
            intrebari.get(),
            2,
            "oprirea vine dupa un obiect, nu la capat"
        );
        let intreaga = compara_cu_oprire("R", &r, &altele, fara_progres(), || true).unwrap();
        assert_eq!(
            intreaga.randuri.len(),
            compara("R", &r, &altele, fara_progres()).randuri.len()
        );
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

    /// Regresie din audit: pe un obiect de 10.000+ token-uri, un `NOT` sters
    /// dadea scor 99,995 -> rotunjit 100 -> „identic", si modificarea nu
    /// ajungea nici in lista, nici in deploy (la fel in v3.4).
    #[test]
    fn modificarea_unui_cuvant_intr_un_obiect_mare_se_raporteaza() {
        let corp: String = (0..1200)
            .map(|i| format!("UPDATE t SET c{i} = {i} WHERE id = @id\n"))
            .collect();
        let a = format!("CREATE PROC dbo.pMare AS\nIF NOT EXISTS (SELECT 1 FROM t)\n{corp}");
        let b = format!("CREATE PROC dbo.pMare AS\nIF EXISTS (SELECT 1 FROM t)\n{corp}");
        assert!(tokeni_scor(&a).len() > 10_000);
        let r = pregateste(&[ob("pMare", &a)]);
        let t = pregateste(&[ob("pMare", &b)]);
        let c = compara("Ref", &r, &[("T".to_string(), t)], fara_progres());
        assert_eq!(c.numaratoare.diferite, 1, "{:?}", c.numaratoare);
        let scor = c.randuri[0].scor.unwrap();
        assert!(
            scor < 100.0,
            "un obiect diferit nu se afiseaza cu scor 100: {scor}"
        );
    }

    /// Regresie din audit: un obiect criptat in tinta nu e „lipsa" (deploy-ul
    /// l-ar suprascrie), ci `criptat`.
    #[test]
    fn obiectul_criptat_nu_apare_lipsa() {
        let r = pregateste(&[ob("pV", "CREATE PROC dbo.pV AS SELECT 1")]);
        let t = pregateste(&[ob("pV", "")]);
        let c = compara("Ref", &r, &[("T".to_string(), t)], fara_progres());
        assert_eq!(c.numaratoare.lipsa, 0);
        assert_eq!(c.numaratoare.criptate, 1);
        assert_eq!(c.randuri[0].categorie, Categorie::Criptat);
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

    #[test]
    fn setarile_diferite_se_spun_pe_nume_si_doar_cand_se_stiu() {
        let on = Setari {
            ansi_nulls: true,
            quoted_identifier: true,
        };
        let qi_off = Setari {
            quoted_identifier: false,
            ..on
        };
        assert!(setari_diferite(Some(on), Some(on)).is_empty());
        assert_eq!(
            setari_diferite(Some(on), Some(qi_off)),
            ["QUOTED_IDENTIFIER"]
        );
        assert_eq!(
            setari_diferite(
                Some(on),
                Some(Setari {
                    ansi_nulls: false,
                    quoted_identifier: false
                })
            ),
            ["QUOTED_IDENTIFIER", "ANSI_NULLS"]
        );
        // instantaneu vechi, fara setari: nu se afirma nimic
        assert!(setari_diferite(None, Some(qi_off)).is_empty());
        assert!(setari_diferite(Some(on), None).is_empty());
    }

    /// C6 din `cercetare/audit-qa-2026-11-18.md`: acelasi text cu alt
    /// `QUOTED_IDENTIFIER` se numara „identic", deci nu apare in lista si nu
    /// se poate deploia. Cu QI OFF, `"a"` e un sir; cu ON, o coloana — alt
    /// F-M3 (decizia lui Mihai): hinturile raman zgomot la comparare, dar un
    /// obiect care difera **doar** prin ele nu mai iese identic: o reparatie
    /// de concurenta (`UPDLOCK, HOLDLOCK`) nu s-ar fi deployat niciodata.
    #[test]
    fn diferenta_doar_in_hinturi_nu_e_identic() {
        let o = |def: &str| {
            pregateste(&[Obiect {
                schema: "dbo".into(),
                nume: "pH".into(),
                tip: "P".into(),
                definitie: def.into(),
                setari: None,
            }])
        };
        let r = o("CREATE PROC dbo.pH AS SELECT a FROM t WITH (NOLOCK)");
        let t = o("CREATE PROC dbo.pH AS SELECT a FROM t WITH (UPDLOCK, HOLDLOCK)");
        assert_eq!(r["dbo.pH"].hash, t["dbo.pH"].hash, "la normalizare, zgomot");
        let c = compara("Ref", &r, &[("Tinta".to_string(), t.clone())], |_| {});
        assert_eq!(c.numaratoare.identice, 0);
        assert_eq!(c.randuri[0].categorie, Categorie::Diferit);
        assert_eq!(c.randuri[0].rezumat, "difera doar in: hinturi");
        assert_eq!(diferente_ascunse(&r["dbo.pH"], &t["dbo.pH"]), ["hinturi"]);
        // acelasi hint, scris altfel: tot identic
        let t2 = o("create proc dbo.pH as select a from t with ( nolock )");
        let c = compara("Ref", &r, &[("Tinta".to_string(), t2)], |_| {});
        assert_eq!(c.numaratoare.identice, 1);
    }

    /// comportament la executie, pe ~21% din obiectele serverului real.
    #[test]
    fn acelasi_text_cu_alte_setari_e_diferit_nu_identic() {
        let on = Setari {
            ansi_nulls: true,
            quoted_identifier: true,
        };
        let cod = "CREATE PROC dbo.pSet AS SELECT \"a\"";
        let r = pregateste(&[Obiect {
            schema: "dbo".into(),
            nume: "pSet".into(),
            tip: "P".into(),
            definitie: cod.into(),
            setari: Some(on),
        }]);
        let t = pregateste(&[Obiect {
            schema: "dbo".into(),
            nume: "pSet".into(),
            tip: "P".into(),
            definitie: cod.into(),
            setari: Some(Setari {
                quoted_identifier: false,
                ..on
            }),
        }]);
        let c = compara("Ref", &r, &[("Tinta".to_string(), t)], |_| {});
        assert_eq!(c.numaratoare.identice, 0, "setarile nu sunt o bagatela");
        assert_eq!(c.randuri.len(), 1, "{:?}", c.randuri);
        assert_eq!(c.randuri[0].categorie, Categorie::Diferit);
        assert!(
            c.randuri[0].rezumat.contains("QUOTED_IDENTIFIER"),
            "rezumatul spune ce difera: {:?}",
            c.randuri[0].rezumat
        );
    }
}
