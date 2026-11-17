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
/// `schema.obiect`. Doua nume diferite pot da acelasi slug (`Baza Mea` /
/// `Baza_Mea`); `NumeFisiere` le face unice.
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
    out.trim_matches(|c| c == '_' || c == '.').to_string()
}

/// Numele pe care Windows le trateaza ca dispozitive, cu orice extensie.
const NUME_REZERVATE_WINDOWS: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Da nume de fisier unice in cadrul unei rulari. Pana la audit, `Baza Mea` si
/// `Baza_Mea` scriau amandoua `Baza_Mea.sql`, a doua peste prima, iar raportul
/// spunea ca s-au scris doua fisiere.
#[derive(Default)]
struct NumeFisiere {
    folosite: std::collections::HashSet<String>,
}

impl NumeFisiere {
    /// `baza` e partea dinaintea extensiei, deja trecuta prin `slug`.
    fn unic(&mut self, baza: &str, extensie: &str) -> String {
        let mut baza = if baza.is_empty() {
            "fara_nume".to_string()
        } else {
            baza.to_string()
        };
        let tulpina = baza.split('.').next().unwrap_or("").to_ascii_uppercase();
        if NUME_REZERVATE_WINDOWS.contains(&tulpina.as_str()) {
            baza = format!("_{baza}");
        }
        let mut nume = format!("{baza}.{extensie}");
        let mut k = 2;
        // Windows nu deosebeste majusculele in numele de fisier.
        while !self.folosite.insert(nume.to_lowercase()) {
            nume = format!("{baza}_{k}.{extensie}");
            k += 1;
        }
        nume
    }
}

/// Un cuvant din codul propriu-zis (nu din string sau comentariu), cu pozitia
/// lui in octeti in textul original.
struct Cuvant<'a> {
    text: &'a str,
    start: usize,
    sfarsit: usize,
}

/// Cuvintele din segmentele de cod, in ordine. Pozitiile sunt in textul
/// original: nimic nu se recalculeaza pe o copie cu litere mici (asa dadea
/// panica pe `K`, al carui minuscul are alta lungime in octeti).
fn cuvinte_de_cod(cod: &str) -> Vec<Cuvant<'_>> {
    let e_cuvant = |c: char| c.is_alphanumeric() || matches!(c, '_' | '@' | '#' | '$');
    let mut rez = Vec::new();
    for seg in gmotion_core::lexer::segmente(cod) {
        if seg.fel != gmotion_core::lexer::Fel::Cod {
            continue;
        }
        let bucata = &cod[seg.start..seg.sfarsit];
        let mut start: Option<usize> = None;
        for (i, c) in bucata
            .char_indices()
            .chain(std::iter::once((bucata.len(), ' ')))
        {
            match (start, e_cuvant(c)) {
                (None, true) => start = Some(i),
                (Some(s), false) => {
                    rez.push(Cuvant {
                        text: &bucata[s..i],
                        start: seg.start + s,
                        sfarsit: seg.start + i,
                    });
                    start = None;
                }
                _ => {}
            }
        }
    }
    rez
}

/// Transforma primul `CREATE` sau `ALTER <tip>` in `CREATE OR ALTER`.
///
/// Un script care face `CREATE` pe un obiect existent eșueaza; unul care face
/// `ALTER` pe un obiect inexistent, la fel. `CREATE OR ALTER` merge in ambele
/// cazuri, deci acelasi script poate fi rulat si pe o baza careia lipseste
/// obiectul si pe una care il are intr-o versiune veche.
///
/// Se uita **doar la cod**: un `CREATE` dintr-un comentariu (`-- vechi: CREATE
/// PROCEDURE ...`) sau dintr-un string nu se atinge. Pana la audit, regula
/// lovea primul `CREATE` din text, deci rescria comentariul si lasa `CREATE`-ul
/// adevarat, iar scriptul cadea pe orice baza care avea deja obiectul.
/// `CREATE TABLE` nu intra: `CREATE OR ALTER TABLE` nu exista in T-SQL.
pub fn aplica_create_or_alter(cod: &str) -> String {
    const TIPURI: [(&str, &str); 5] = [
        ("procedure", "PROCEDURE"),
        ("proc", "PROCEDURE"),
        ("view", "VIEW"),
        ("function", "FUNCTION"),
        ("trigger", "TRIGGER"),
    ];
    let cuv = cuvinte_de_cod(cod);
    for (i, c) in cuv.iter().enumerate() {
        let e_create = c.text.eq_ignore_ascii_case("create");
        if !e_create && !c.text.eq_ignore_ascii_case("alter") {
            continue;
        }
        let urm = |k: usize| cuv.get(i + k).map(|x| x.text);
        if e_create
            && urm(1).is_some_and(|w| w.eq_ignore_ascii_case("or"))
            && urm(2).is_some_and(|w| w.eq_ignore_ascii_case("alter"))
        {
            return cod.to_string(); // deja CREATE OR ALTER
        }
        let Some(tip) = cuv.get(i + 1) else {
            continue;
        };
        if let Some((_, mare)) = TIPURI
            .iter()
            .find(|(mic, _)| tip.text.eq_ignore_ascii_case(mic))
        {
            return format!(
                "{}CREATE OR ALTER {}{}",
                &cod[..c.start],
                mare,
                &cod[tip.sfarsit..]
            );
        }
    }
    cod.to_string()
}

/// `]` dublat: singurul caracter care poate iesi dintr-un `[identificator]`.
/// Pana la audit, o baza numita `X]; DROP DATABASE Prod; --` scria in script
/// `USE [X]; DROP DATABASE Prod; --];`.
fn paranteze(nume: &str) -> String {
    format!("[{}]", nume.replace(']', "]]"))
}

/// Un nume care ajunge in script (in `USE [..]` sau intr-un comentariu `--`)
/// nu are voie sa contina caractere de control: un rand nou incheie
/// comentariul, iar o linie `GO` aparuta asa taie batch-ul si face restul
/// executabil.
fn verifica_nume(nume: &str) -> Rezultat<()> {
    if nume.chars().any(char::is_control) {
        return Err(Eroare::NumeNesigur {
            nume: nume.to_string(),
        });
    }
    Ok(())
}

/// Tot ce poate face un script periculos se verifica **inainte** de a scrie
/// ceva: un deploy partial e mai rau decat niciunul.
fn verifica_inainte(referinta: &str, pe_baza: &BTreeMap<String, Vec<DeDeployat>>) -> Rezultat<()> {
    verifica_nume(referinta)?;
    for (baza, obiecte) in pe_baza {
        verifica_nume(baza)?;
        for o in obiecte {
            verifica_nume(&o.obiect)?;
            if o.cod.trim().is_empty() {
                return Err(Eroare::ObiectCriptat {
                    obiect: o.obiect.clone(),
                });
            }
            let linii = gmotion_core::lexer::linii_go(&o.cod);
            if !linii.is_empty() {
                return Err(Eroare::GoInDefinitie {
                    obiect: o.obiect.clone(),
                    linii: linii
                        .iter()
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                });
            }
        }
    }
    Ok(())
}

fn antet(baza: &str, referinta: &str, acum: &str) -> String {
    format!(
        "-- DEPLOYER {baza} | referinta: {referinta} | {acum}\nUSE {};\nGO\n\n",
        paranteze(baza)
    )
}

fn bloc(obiect: &str, cod: &str) -> String {
    format!("-- ═══ {obiect} ═══\n{cod}\nGO\n\n")
}

/// Directorul de iesire trebuie sa nu existe sau sa fie gol. Pana la audit,
/// fisierele unei rulari vechi ramaneau langa cele noi, iar rulate in ordine
/// alfabetica aplicau o versiune veche dupa una noua.
fn pregateste_director(director: &Path) -> Rezultat<()> {
    let eroare = |e: std::io::Error| Eroare::ScriereDeploy {
        cale: director.display().to_string(),
        detaliu: e.to_string(),
    };
    match std::fs::read_dir(director) {
        Ok(mut intrari) => {
            if intrari.next().is_some() {
                return Err(Eroare::DirectorNegol {
                    cale: director.display().to_string(),
                });
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(director).map_err(eroare)
        }
        Err(e) => Err(eroare(e)),
    }
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
    verifica_inainte(referinta, pe_baza)?;
    pregateste_director(director)?;

    let mut fisiere = Vec::new();
    let mut nume_fisiere = NumeFisiere::default();
    let mut scrie_fisier = |baza_nume: String, conținut: String| -> Rezultat<()> {
        let nume = nume_fisiere.unic(&baza_nume, "sql");
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
                        "{:03}_{}_{}_{}",
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
                        "{}_part{:02}_obj{}-{}",
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
            scrie_fisier("deploy_complet".to_string(), text)?;
        }
        ModImpartire::PerBaza => {
            for (baza, obiecte) in pe_baza {
                let mut text = antet(baza, referinta, acum);
                for o in obiecte {
                    text.push_str(&bloc(&o.obiect, &o.cod));
                }
                scrie_fisier(slug(baza), text)?;
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
        // Punctele se pastreaza in interior, pentru `schema.obiect`; la
        // capete se taie, ca un nume de fisier sa nu inceapa cu `..`.
        assert_eq!(slug("../etc"), "etc");
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

    fn o(obiect: &str, cod: &str) -> DeDeployat {
        DeDeployat {
            obiect: obiect.into(),
            tip: "P".into(),
            cod: cod.into(),
        }
    }

    fn o_baza(baza: &str, obiect: DeDeployat) -> BTreeMap<String, Vec<DeDeployat>> {
        BTreeMap::from([(baza.to_string(), vec![obiect])])
    }

    // ---- regresii din auditul de la f561bf9 ----

    #[test]
    fn create_din_comentariu_sau_string_nu_se_rescrie() {
        for (intrare, asteptat) in [
            (
                "-- vechi: CREATE PROCEDURE dbo.pCom_old\nCREATE PROCEDURE dbo.pCom AS SELECT 1",
                "-- vechi: CREATE PROCEDURE dbo.pCom_old\nCREATE OR ALTER PROCEDURE dbo.pCom AS SELECT 1",
            ),
            (
                "/* exemplu: ALTER VIEW dbo.v */ CREATE PROCEDURE dbo.p AS SELECT 1",
                "/* exemplu: ALTER VIEW dbo.v */ CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "/* use CREATE OR ALTER next time */\nCREATE PROCEDURE dbo.pStr AS SELECT 1",
                "/* use CREATE OR ALTER next time */\nCREATE OR ALTER PROCEDURE dbo.pStr AS SELECT 1",
            ),
            (
                "CREATE PROC /* x */ dbo.p AS SELECT 'ALTER VIEW v'",
                "CREATE OR ALTER PROCEDURE /* x */ dbo.p AS SELECT 'ALTER VIEW v'",
            ),
        ] {
            assert_eq!(aplica_create_or_alter(intrare), asteptat, "{intrare:?}");
        }
    }

    #[test]
    fn caracterele_care_isi_schimba_lungimea_la_minuscule_nu_strica_nimic() {
        // `K` (semnul Kelvin) si `İ`: minusculele au alta lungime in octeti.
        // Inainte: panica, respectiv `CCREATE OR ALTER PROCEDUREdbo...`.
        assert_eq!(
            aplica_create_or_alter("-- 5\u{212A}\nCREATE PROCEDURE dbo.p AS SELECT 1"),
            "-- 5\u{212A}\nCREATE OR ALTER PROCEDURE dbo.p AS SELECT 1"
        );
        assert_eq!(
            aplica_create_or_alter("-- İstanbul\nCREATE PROCEDURE dbo.pTurc AS SELECT 1"),
            "-- İstanbul\nCREATE OR ALTER PROCEDURE dbo.pTurc AS SELECT 1"
        );
    }

    #[test]
    fn numele_bazei_nu_poate_iesi_din_use() {
        let d = dir("use");
        let baza = "X]; DROP DATABASE Prod; --";
        let r = scrie(
            &d,
            "Ref",
            &o_baza(baza, o("dbo.p", "CREATE OR ALTER PROC dbo.p AS SELECT 1")),
            ModImpartire::PerBaza,
            0,
            "t",
        )
        .unwrap();
        let text = std::fs::read_to_string(&r.fisiere[0]).unwrap();
        assert!(
            text.contains("USE [X]]; DROP DATABASE Prod; --];"),
            "{text}"
        );
        assert!(
            !text.lines().any(|l| l.trim_start().starts_with("DROP")),
            "{text}"
        );
    }

    #[test]
    fn nume_cu_caractere_de_control_sunt_refuzate_inainte_de_orice_scriere() {
        let cod = "CREATE OR ALTER PROC dbo.p AS SELECT 1";
        for (baza, obiect) in [
            ("B\nGO\nDROP DATABASE Prod\nGO\n--", "dbo.p"),
            ("B", "dbo.q\nGO\nDROP TABLE dbo.Facturi\nGO\n--"),
            ("B\r", "dbo.p"),
        ] {
            let d = dir("control");
            let e = scrie(
                &d,
                "Ref",
                &o_baza(baza, o(obiect, cod)),
                ModImpartire::Unul,
                0,
                "t",
            )
            .unwrap_err();
            assert_eq!(e.cod(), "GM-MOD-008", "{baza:?} / {obiect:?}");
            assert!(!d.exists(), "nimic nu se scrie cand se refuza");
        }
    }

    #[test]
    fn go_in_definitie_e_refuzat() {
        let d = dir("go");
        let cod =
            "CREATE OR ALTER PROC dbo.pGo AS\nSELECT '\nGO\nDROP TABLE dbo.Clienti\nGO\n' AS x";
        let e = scrie(
            &d,
            "Ref",
            &o_baza("B", o("dbo.pGo", cod)),
            ModImpartire::Unul,
            0,
            "t",
        )
        .unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-009");
        assert!(e.to_string().contains("3, 5"), "{e}");
        assert!(!d.exists());
    }

    #[test]
    fn obiect_fara_cod_e_refuzat() {
        let d = dir("gol");
        let e = scrie(
            &d,
            "Ref",
            &o_baza("B", o("dbo.pCriptat", "")),
            ModImpartire::Unul,
            0,
            "t",
        )
        .unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-011");
    }

    #[test]
    fn nume_de_baze_care_dau_acelasi_fisier_nu_se_suprascriu() {
        let d = dir("coliziune");
        let cod = "CREATE OR ALTER PROC dbo.p AS SELECT 1";
        let mut m = BTreeMap::new();
        for b in ["Baza Mea", "Baza_Mea", "baza_mea", "CON", "Bază", "Bazâ"] {
            m.insert(b.to_string(), vec![o("dbo.p", cod)]);
        }
        let r = scrie(&d, "Ref", &m, ModImpartire::PerBaza, 0, "t").unwrap();
        assert_eq!(r.fisiere.len(), 6);
        let mut nume: Vec<String> = r
            .fisiere
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_lowercase())
            .collect();
        nume.sort();
        nume.dedup();
        assert_eq!(nume.len(), 6, "{nume:?}");
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 6, "pe disc tot 6");
        assert!(r.fisiere.iter().all(|f| {
            let s = f.file_name().unwrap().to_string_lossy().to_uppercase();
            !s.starts_with("CON.")
        }));
    }

    #[test]
    fn director_ne_gol_e_refuzat() {
        let d = dir("negol");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("003_vechi.sql"), "-- vechi").unwrap();
        let e = scrie(&d, "Ref", &exemplu(), ModImpartire::PerObiect, 0, "t").unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-010");
        // director existent dar gol: e in regula
        let d2 = dir("gol-existent");
        std::fs::create_dir_all(&d2).unwrap();
        assert!(scrie(&d2, "Ref", &exemplu(), ModImpartire::PerBaza, 0, "t").is_ok());
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
