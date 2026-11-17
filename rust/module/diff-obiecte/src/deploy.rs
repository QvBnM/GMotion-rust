//! Generatorul de scripturi de deploy.
//!
//! **Se strica astfel:** produce SQL greșit, si se observa **dupa** ce a rulat
//! pe o baza. E partea cu cel mai mare cost al unei greseli din tot modulul, de
//! aceea are cele mai multe teste si nicio scurtatura.
//!
//! Portat din `genereaza_deployer`, `_scrie_deployer` si
//! `_aplica_create_or_alter` din `mostenire/app.py`. Deciziile pastrate:
//! obiectele `extra` nu se pot deploia (nu exista cod in referinta de adus),
//! iar primul `CREATE`/`ALTER` devine `CREATE OR ALTER`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::eroare::{Eroare, Rezultat};

/// Cum se imparte deploy-ul in fisiere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModImpartire {
    /// Tot intr-un singur fisier, toate bazele.
    Unul,
    /// Un fisier per baza. Implicit.
    #[default]
    PerBaza,
    /// Un fisier per obiect.
    PerObiect,
    /// Cate `n` obiecte per fisier.
    CateN(usize),
}

/// Un obiect de scris, cu codul final deja decis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeDeployat {
    pub obiect: String,
    pub tip: String,
    pub cod: String,
}

/// Ce s-a scris.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raport {
    pub director: PathBuf,
    pub fisiere: Vec<PathBuf>,
    pub obiecte: usize,
    /// Obiecte marcate dar nedeployabile: cele `extra`.
    pub sarite: usize,
}

/// Numele scurt al tipului, pentru numele de fisier.
fn tip_nume(t: &str) -> &'static str {
    match t.trim() {
        "P" => "PROC",
        "V" => "VIEW",
        "FN" | "TF" | "IF" => "FUNC",
        "TR" => "TRIGGER",
        _ => "OBJ",
    }
}

/// Reduce un nume la ceva folosibil ca nume de fisier, pastrand punctul din
/// `schema.obiect`.
fn slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}

/// Transforma primul `CREATE` sau `ALTER <tip>` in `CREATE OR ALTER`.
///
/// Un script care face `CREATE` pe un obiect existent eșueaza; unul care face
/// `ALTER` pe un obiect inexistent, la fel. `CREATE OR ALTER` merge in ambele
/// cazuri, deci acelasi script poate fi rulat si pe o baza careia lipseste
/// obiectul si pe una care il are intr-o versiune veche.
///
/// Nu atinge codul care e deja `CREATE OR ALTER`. Inlocuieste doar **prima**
/// apariție: restul pot fi in comentarii sau in SQL dinamic.
pub fn aplica_create_or_alter(cod: &str) -> String {
    if are_create_or_alter(cod) {
        return cod.to_string();
    }
    let minuscule = cod.to_lowercase();

    // Prima apariție a unui `CREATE` sau `ALTER` urmat de un tip de obiect pe
    // care GMotion compara. `CREATE TABLE` nu intra: `CREATE OR ALTER TABLE` nu
    // exista in T-SQL.
    let mut candidate: Vec<(usize, usize)> = Vec::new();
    for (cuvant, lungime) in [("create", 6usize), ("alter", 5usize)] {
        for poz in gaseste_cuvinte(&minuscule, cuvant) {
            candidate.push((poz, lungime));
        }
    }
    candidate.sort_unstable();

    const TIPURI: [(&str, &str); 5] = [
        ("procedure", "PROCEDURE"),
        ("proc", "PROCEDURE"),
        ("view", "VIEW"),
        ("function", "FUNCTION"),
        ("trigger", "TRIGGER"),
    ];

    for (poz, lungime_kw) in candidate {
        // sari peste spatiile dintre cuvantul cheie si tipul de obiect
        let mut i = poz + lungime_kw;
        while minuscule[i..].starts_with(char::is_whitespace) {
            i += minuscule[i..].chars().next().map_or(1, char::len_utf8);
        }
        let rest = &minuscule[i..];
        for (mic, mare) in TIPURI {
            if rest.starts_with(mic) && !continua_identificatorul(rest.as_bytes().get(mic.len())) {
                let sfarsit = i + mic.len();
                return format!("{}CREATE OR ALTER {}{}", &cod[..poz], mare, &cod[sfarsit..]);
            }
        }
    }
    cod.to_string()
}

fn continua_identificatorul(c: Option<&u8>) -> bool {
    matches!(c, Some(b) if (*b as char).is_ascii_alphanumeric() || *b == b'_')
}

/// Pozitiile unui cuvant intreg (delimitat), in text deja minuscul.
fn gaseste_cuvinte(text: &str, cuvant: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut de_la = 0usize;
    while let Some(rel) = text[de_la..].find(cuvant) {
        let poz = de_la + rel;
        let inainte_ok = poz == 0 || !continua_identificatorul(text.as_bytes().get(poz - 1));
        let dupa_ok = !continua_identificatorul(text.as_bytes().get(poz + cuvant.len()));
        if inainte_ok && dupa_ok {
            out.push(poz);
        }
        de_la = poz + 1;
    }
    out
}

fn are_create_or_alter(cod: &str) -> bool {
    let m = cod.to_lowercase();
    gaseste_cuvinte(&m, "create").iter().any(|&p| {
        let rest = m[p + 6..].trim_start();
        rest.starts_with("or")
            && !continua_identificatorul(rest.as_bytes().get(2))
            && rest[2..].trim_start().starts_with("alter")
    })
}

fn antet(baza: &str, referinta: &str, acum: &str) -> String {
    format!("-- DEPLOYER {baza} | referinta: {referinta} | {acum}\nUSE [{baza}];\nGO\n\n")
}

fn bloc(obiect: &str, cod: &str) -> String {
    format!("-- ═══ {obiect} ═══\n{cod}\nGO\n\n")
}

/// Scrie scripturile. `acum` e dat din afara, nu citit din ceas: altfel
/// rezultatul n-ar fi testabil.
pub fn scrie(
    director: &Path,
    referinta: &str,
    pe_baza: &BTreeMap<String, Vec<DeDeployat>>,
    mod_impartire: ModImpartire,
    sarite: usize,
    acum: &str,
) -> Rezultat<Raport> {
    if pe_baza.is_empty() {
        return Err(Eroare::NimicDeDeployat);
    }
    std::fs::create_dir_all(director).map_err(|e| Eroare::ScriereDeploy {
        cale: director.display().to_string(),
        detaliu: e.to_string(),
    })?;

    let mut fisiere = Vec::new();
    let mut scrie_fisier = |nume: String, conținut: String| -> Rezultat<()> {
        let cale = director.join(&nume);
        std::fs::write(&cale, conținut).map_err(|e| Eroare::ScriereDeploy {
            cale: cale.display().to_string(),
            detaliu: e.to_string(),
        })?;
        fisiere.push(cale);
        Ok(())
    };

    match mod_impartire {
        ModImpartire::PerObiect => {
            for (baza, obiecte) in pe_baza {
                for (i, o) in obiecte.iter().enumerate() {
                    let nume = format!(
                        "{:03}_{}_{}_{}.sql",
                        i + 1,
                        slug(baza),
                        tip_nume(&o.tip),
                        slug(&o.obiect)
                    );
                    scrie_fisier(
                        nume,
                        format!(
                            "{}{}",
                            antet(baza, referinta, acum),
                            bloc(&o.obiect, &o.cod)
                        ),
                    )?;
                }
            }
        }
        ModImpartire::CateN(n) => {
            let n = n.max(1);
            for (baza, obiecte) in pe_baza {
                for (parte, grup) in obiecte.chunks(n).enumerate() {
                    let primul = parte * n + 1;
                    let nume = format!(
                        "{}_part{:02}_obj{}-{}.sql",
                        slug(baza),
                        parte + 1,
                        primul,
                        primul + grup.len() - 1
                    );
                    let mut text = antet(baza, referinta, acum);
                    for o in grup {
                        text.push_str(&bloc(&o.obiect, &o.cod));
                    }
                    scrie_fisier(nume, text)?;
                }
            }
        }
        ModImpartire::Unul => {
            let mut text = String::new();
            for (baza, obiecte) in pe_baza {
                text.push_str(&antet(baza, referinta, acum));
                for o in obiecte {
                    text.push_str(&bloc(&o.obiect, &o.cod));
                }
                text.push('\n');
            }
            scrie_fisier("deploy_complet.sql".to_string(), text)?;
        }
        ModImpartire::PerBaza => {
            for (baza, obiecte) in pe_baza {
                let mut text = antet(baza, referinta, acum);
                for o in obiecte {
                    text.push_str(&bloc(&o.obiect, &o.cod));
                }
                scrie_fisier(format!("{}.sql", slug(baza)), text)?;
            }
        }
    }

    let obiecte = pe_baza.values().map(Vec::len).sum();
    tracing::info!(
        target: "gmotion::modul",
        director = %director.display(), fisiere = fisiere.len(), obiecte,
        "scripturi de deploy scrise"
    );
    Ok(Raport {
        director: director.to_path_buf(),
        fisiere,
        obiecte,
        sarite,
    })
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn create_devine_create_or_alter() {
        for (intrare, asteptat) in [
            (
                "CREATE PROCEDURE dbo.p AS SELECT 1",
                "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "CREATE PROC dbo.p AS SELECT 1",
                "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "ALTER PROCEDURE dbo.p AS SELECT 1",
                "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "create view dbo.v as select 1",
                "CREATE OR ALTER VIEW dbo.v as select 1",
            ),
            (
                "ALTER FUNCTION dbo.f() RETURNS INT AS BEGIN RETURN 1 END",
                "CREATE OR ALTER FUNCTION dbo.f() RETURNS INT AS BEGIN RETURN 1 END",
            ),
            (
                "CREATE TRIGGER t ON tab AFTER INSERT AS SELECT 1",
                "CREATE OR ALTER TRIGGER t ON tab AFTER INSERT AS SELECT 1",
            ),
        ] {
            assert_eq!(aplica_create_or_alter(intrare), asteptat, "pe {intrare:?}");
        }
    }

    #[test]
    fn spatii_si_comentarii_inainte_nu_incurca() {
        let cod = "-- comentariu\n/* bloc */\n\n   CREATE   PROCEDURE  dbo.p AS SELECT 1";
        let rez = aplica_create_or_alter(cod);
        assert!(
            rez.starts_with("-- comentariu"),
            "antetul trebuie pastrat: {rez}"
        );
        assert!(rez.contains("CREATE OR ALTER PROCEDURE"));
        assert!(!rez.contains("CREATE   PROCEDURE"));
    }

    #[test]
    fn deja_create_or_alter_nu_se_atinge() {
        for cod in [
            "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            "create or alter view dbo.v as select 1",
            "CREATE\n  OR\n  ALTER PROCEDURE dbo.p AS SELECT 1",
        ] {
            assert_eq!(aplica_create_or_alter(cod), cod, "pe {cod:?}");
        }
    }

    #[test]
    fn doar_prima_apariție_se_schimba() {
        let cod = "CREATE PROCEDURE dbo.p AS\n-- CREATE PROCEDURE dbo.altceva\nEXEC('CREATE VIEW v AS SELECT 1')";
        let rez = aplica_create_or_alter(cod);
        assert_eq!(rez.matches("CREATE OR ALTER").count(), 1);
        assert!(
            rez.contains("-- CREATE PROCEDURE dbo.altceva"),
            "comentariul nu se atinge"
        );
        assert!(
            rez.contains("EXEC('CREATE VIEW"),
            "SQL-ul dinamic nu se atinge"
        );
    }

    #[test]
    fn cod_fara_create_sau_alter_rămâne_neatins() {
        for cod in [
            "SELECT 1",
            "",
            "-- doar un comentariu",
            "CREATED PROCEDURE x",
        ] {
            assert_eq!(aplica_create_or_alter(cod), cod, "pe {cod:?}");
        }
    }

    #[test]
    fn create_table_nu_se_atinge() {
        // Nu e un tip pe care-l comparam, si `CREATE OR ALTER TABLE` nu exista.
        let cod = "CREATE TABLE t (a INT)";
        assert_eq!(aplica_create_or_alter(cod), cod);
    }

    #[test]
    fn tip_si_slug() {
        assert_eq!(tip_nume("P"), "PROC");
        assert_eq!(tip_nume(" V "), "VIEW");
        assert_eq!(tip_nume("TF"), "FUNC");
        assert_eq!(tip_nume("X"), "OBJ");
        assert_eq!(slug("dbo.pClienti"), "dbo.pClienti");
        assert_eq!(slug("Baza Mea"), "Baza_Mea");
        assert_eq!(slug("a//b"), "a_b");
        // Punctele se pastreaza, pentru `schema.obiect`. Separatorii de cale,
        // nu — deci `..` ramane dar nu poate forma o cale.
        assert_eq!(slug("../etc"), ".._etc");
        for nume in ["../../etc/passwd", "..", "C:\\Windows\\x", "./.."] {
            let s = slug(nume);
            assert!(
                !s.contains('/') && !s.contains('\\') && !s.contains(':'),
                "{nume:?} -> {s:?} ar putea forma o cale"
            );
        }
    }

    fn dir(et: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("gmotion-dep-{et}-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        d
    }

    fn exemplu() -> BTreeMap<String, Vec<DeDeployat>> {
        let mut m = BTreeMap::new();
        m.insert(
            "Client1".to_string(),
            vec![
                DeDeployat {
                    obiect: "dbo.pA".into(),
                    tip: "P".into(),
                    cod: "CREATE OR ALTER PROCEDURE dbo.pA AS SELECT 1".into(),
                },
                DeDeployat {
                    obiect: "dbo.vB".into(),
                    tip: "V".into(),
                    cod: "CREATE OR ALTER VIEW dbo.vB AS SELECT 2".into(),
                },
                DeDeployat {
                    obiect: "dbo.pC".into(),
                    tip: "P".into(),
                    cod: "CREATE OR ALTER PROCEDURE dbo.pC AS SELECT 3".into(),
                },
            ],
        );
        m.insert(
            "Client2".to_string(),
            vec![DeDeployat {
                obiect: "dbo.pA".into(),
                tip: "P".into(),
                cod: "CREATE OR ALTER PROCEDURE dbo.pA AS SELECT 1".into(),
            }],
        );
        m
    }

    #[test]
    fn mod_per_baza() {
        let d = dir("per-baza");
        let r = scrie(
            &d,
            "Ref",
            &exemplu(),
            ModImpartire::PerBaza,
            0,
            "2026-11-17 10:00",
        )
        .unwrap();
        assert_eq!(r.fisiere.len(), 2);
        assert_eq!(r.obiecte, 4);
        let t = std::fs::read_to_string(d.join("Client1.sql")).unwrap();
        assert!(t.starts_with("-- DEPLOYER Client1 | referinta: Ref | 2026-11-17 10:00\n"));
        assert!(t.contains("USE [Client1];\nGO\n"));
        assert_eq!(t.matches("-- ═══").count(), 3);
        assert_eq!(
            t.matches("\nGO\n").count(),
            4,
            "unul de USE plus unul per obiect"
        );
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn mod_unul_singur() {
        let d = dir("unul");
        let r = scrie(&d, "Ref", &exemplu(), ModImpartire::Unul, 0, "a").unwrap();
        assert_eq!(r.fisiere.len(), 1);
        let t = std::fs::read_to_string(d.join("deploy_complet.sql")).unwrap();
        // fiecare baza isi aduce propriul USE: altfel al doilea set ar rula pe
        // baza greșita
        assert_eq!(t.matches("USE [").count(), 2);
        assert!(t.find("USE [Client1]").unwrap() < t.find("USE [Client2]").unwrap());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn mod_per_obiect() {
        let d = dir("per-obiect");
        let r = scrie(&d, "Ref", &exemplu(), ModImpartire::PerObiect, 0, "a").unwrap();
        assert_eq!(r.fisiere.len(), 4);
        assert!(d.join("001_Client1_PROC_dbo.pA.sql").exists());
        assert!(d.join("002_Client1_VIEW_dbo.vB.sql").exists());
        assert!(d.join("003_Client1_PROC_dbo.pC.sql").exists());
        assert!(d.join("001_Client2_PROC_dbo.pA.sql").exists());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn mod_cate_n() {
        let d = dir("cate-n");
        let r = scrie(&d, "Ref", &exemplu(), ModImpartire::CateN(2), 0, "a").unwrap();
        // Client1 are 3 obiecte -> doua fisiere; Client2 are 1 -> unul
        assert_eq!(r.fisiere.len(), 3);
        assert!(d.join("Client1_part01_obj1-2.sql").exists());
        assert!(d.join("Client1_part02_obj3-3.sql").exists());
        assert!(d.join("Client2_part01_obj1-1.sql").exists());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn cate_n_cu_zero_nu_imparte_la_zero() {
        let d = dir("cate-zero");
        let r = scrie(&d, "Ref", &exemplu(), ModImpartire::CateN(0), 0, "a").unwrap();
        assert_eq!(r.fisiere.len(), 4, "0 se trateaza ca 1");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn nimic_de_deployat_e_eroare_nu_un_director_gol() {
        let d = dir("gol");
        let e = scrie(&d, "Ref", &BTreeMap::new(), ModImpartire::PerBaza, 2, "a").unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-002");
        assert!(!d.exists(), "nu se creeaza directorul degeaba");
    }

    #[test]
    fn numele_de_baza_cu_caractere_dificile_nu_scapa_din_director() {
        let d = dir("nume-dificil");
        let mut m = BTreeMap::new();
        m.insert(
            "../evadare".to_string(),
            vec![DeDeployat {
                obiect: "dbo.p".into(),
                tip: "P".into(),
                cod: "SELECT 1".into(),
            }],
        );
        let r = scrie(&d, "Ref", &m, ModImpartire::PerBaza, 0, "a").unwrap();
        assert_eq!(r.fisiere.len(), 1);
        assert_eq!(r.fisiere[0].parent().unwrap(), d.as_path());
        std::fs::remove_dir_all(&d).ok();
    }
}
