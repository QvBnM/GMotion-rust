//! Teste pe binarul adevarat, nu pe functii interne.
//!
//! Rostul: ce am verificat o data de mana sa rămână verificat. Probele manuale
//! care nu devin teste se pierd, si atunci urmatoarea regresie se descopera la
//! utilizator.
//!
//! Acoperite anume:
//! - `--json` scoate **numai** JSON la ieșirea standard (jurnalul si progresul
//!   la erori). Prima versiune a jurnalului scria la stdout si facea ieșirea
//!   neparsabila;
//! - codul de ieșire e nenul la eșec, altfel o rulare programata ar raporta
//!   succes;
//! - parola nu se accepta ca argument de linie de comanda.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const EXE: &str = env!("CARGO_BIN_EXE_gmotion");

fn dir(eticheta: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("gmotion-cli-{eticheta}-{}", std::process::id()));
    std::fs::remove_dir_all(&d).ok();
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn ruleaza(args: &[&str]) -> Output {
    ruleaza_in(args, None)
}

/// Ca `ruleaza`, dar cu directorul de date al aplicatiei intr-un loc al
/// testului: sesiunile salvate nu au voie sa ajunga in cel real.
fn ruleaza_in(args: &[&str], dir_date: Option<&Path>) -> Output {
    let mut c = Command::new(EXE);
    if let Some(d) = dir_date {
        c.env("GMOTION_DIR_DATE", d);
    }
    c.args(args)
        // Fara jurnal in teste: nu schimba ieșirea standard, dar face
        // diagnosticarea unui eșec mai curata.
        .env("GMOTION_JURNAL", "error")
        .env_remove("GMOTION_PAROLA")
        .output()
        .expect("binarul gmotion exista")
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// Instantaneu cu: un obiect modificat, unul lipsa, unul doar in tinta, si unul
/// cu diferenta pur cosmetica (care nu trebuie raportat).
fn scrie_instantaneu(cale: &Path) {
    let json = r#"{
      "versiune": 1,
      "server": "SRV-TEST",
      "creat": "2026-11-17T10:00:00Z",
      "nota": "",
      "baze": {
        "Referinta": [
          {"schema":"dbo","nume":"pClienti","tip":"P","definitie":"CREATE PROCEDURE dbo.pClienti AS\nSELECT nume FROM Clienti WHERE activ = 1"},
          {"schema":"dbo","nume":"vFacturi","tip":"V","definitie":"CREATE VIEW dbo.vFacturi AS\nSELECT id FROM Facturi"},
          {"schema":"dbo","nume":"pStoc","tip":"P","definitie":"CREATE PROCEDURE dbo.pStoc AS\nSELECT * FROM Stoc"}
        ],
        "Client1": [
          {"schema":"dbo","nume":"pClienti","tip":"P","definitie":"CREATE PROCEDURE dbo.pClienti AS\nSELECT nume FROM Clienti WHERE activ = 0"},
          {"schema":"dbo","nume":"vFacturi","tip":"V","definitie":"CREATE VIEW dbo.vFacturi AS\n   SELECT id FROM Facturi   -- cosmetic"},
          {"schema":"dbo","nume":"pLocal","tip":"P","definitie":"CREATE PROCEDURE dbo.pLocal AS\nSELECT 1"}
        ]
      }
    }"#;
    std::fs::write(cale, json).unwrap();
}

#[test]
fn diagnostic_merge_fara_nimic_configurat() {
    let o = ruleaza(&["diagnostic"]);
    assert!(o.status.success(), "{}", err(&o));
    let t = out(&o);
    assert!(t.contains("GMotion"));
    assert!(t.contains("Tipuri comparate"));
    // tipurile vin din core: un singur loc de adevar
    for tip in gmotion_core::TIPURI_OBIECTE {
        assert!(t.contains(tip), "lipseste tipul {tip} din diagnostic:\n{t}");
    }
}

#[test]
fn diagnostic_json_e_parsabil() {
    let o = ruleaza(&["diagnostic", "--json"]);
    assert!(o.status.success(), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).expect("JSON valid");
    assert!(v["versiune"].is_string());
    assert!(v["configurare_activa"]["timeout_conectare_s"].is_number());
}

#[test]
fn fluxul_complet_pe_instantaneu() {
    let d = dir("flux");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let i = inst.to_str().unwrap();

    // --- baze ---
    let o = ruleaza(&["baze", "--sursa", i]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(
        out(&o).lines().collect::<Vec<_>>(),
        ["Client1", "Referinta"]
    );

    // --- compara, text ---
    let o = ruleaza(&["compara", "--sursa", i, "--referinta", "Referinta"]);
    assert!(o.status.success(), "{}", err(&o));
    let t = out(&o);
    assert!(t.contains("dbo.pClienti"), "{t}");
    assert!(t.contains("dbo.pStoc"), "{t}");
    assert!(t.contains("dbo.pLocal"), "{t}");
    assert!(
        !t.contains("dbo.vFacturi"),
        "diferenta cosmetica nu trebuie raportata:\n{t}"
    );

    // --- compara, JSON: numai JSON la ieșirea standard ---
    let o = ruleaza(&[
        "compara",
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "--json",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o))
        .unwrap_or_else(|e| panic!("ieșirea standard nu e JSON curat ({e}):\n{}", out(&o)));
    assert_eq!(v["referinta"], "Referinta");
    assert_eq!(v["diferite"], 1);
    assert_eq!(v["lipsa"], 1);
    assert_eq!(v["extra"], 1);
    // O singura baza comparata (Client1) si 3 obiecte in referinta:
    // pClienti diferit, vFacturi identic (diferenta cosmetica), pStoc lipsa.
    assert_eq!(v["identice"], 1);
    let categorii: Vec<&str> = v["randuri"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["categorie"].as_str().unwrap())
        .collect();
    assert_eq!(categorii, ["diferit", "lipsa", "extra"]);

    // --- diferente pe un obiect ---
    let o = ruleaza(&[
        "diferente",
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "--baza",
        "Client1",
        "--obiect",
        "dbo.pClienti",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let t = out(&o);
    // un bloc modificat apare cu ambele versiuni si cu bucata schimbata marcata
    assert!(
        t.lines().any(|l| l.contains(" - ") && l.contains("[-1-]")),
        "lipseste versiunea veche marcata:\n{t}"
    );
    assert!(
        t.lines().any(|l| l.contains(" + ") && l.contains("{+0+}")),
        "lipseste versiunea noua marcata:\n{t}"
    );

    // --- deploy ---
    let dep = d.join("dep");
    let o = ruleaza(&[
        "deploy",
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "-d",
        dep.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let sql = std::fs::read_to_string(dep.join("Client1.sql")).unwrap();
    assert!(sql.contains("USE [Client1];"));
    assert_eq!(sql.matches("CREATE OR ALTER PROCEDURE").count(), 2);
    // codul vine din REFERINTA, nu din tinta
    assert!(
        sql.contains("activ = 1"),
        "s-a deployat codul din tinta:\n{sql}"
    );
    assert!(!sql.contains("activ = 0"));
    // obiectul care exista doar in tinta nu se deploiaza
    assert!(!sql.contains("dbo.pLocal"), "{sql}");
    // In mod text progresul merge la ieșirea standard; doar pe --json se muta
    // la erori, ca sa nu strice JSON-ul.
    assert!(out(&o).contains("1 sarite"), "{}", out(&o));

    // --- export ---
    let csv = d.join("e.csv");
    let o = ruleaza(&[
        "export",
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "-o",
        csv.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let brut = std::fs::read(&csv).unwrap();
    assert!(
        brut.starts_with(&[0xEF, 0xBB, 0xBF]),
        "lipseste BOM-ul UTF-8"
    );
    let text = String::from_utf8(brut[3..].to_vec()).unwrap();
    assert!(
        text.contains("93,75") || text.contains(";diferit;"),
        "{text}"
    );
    assert!(text.contains("\r\n"));

    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn modurile_de_impartire_a_deploy_ului() {
    let d = dir("moduri");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let i = inst.to_str().unwrap();

    for (mod_arg, asteptat) in [("unul", 1usize), ("per-baza", 1), ("per-obiect", 2)] {
        let out_dir = d.join(mod_arg);
        let o = ruleaza(&[
            "deploy",
            "--sursa",
            i,
            "--referinta",
            "Referinta",
            "-d",
            out_dir.to_str().unwrap(),
            "--mod-fisiere",
            mod_arg,
        ]);
        assert!(o.status.success(), "{mod_arg}: {}", err(&o));
        let n = std::fs::read_dir(&out_dir).unwrap().count();
        assert_eq!(n, asteptat, "modul {mod_arg} a scris {n} fisiere");
    }
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn doar_anumite_obiecte() {
    let d = dir("doar");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let dep = d.join("dep");
    let o = ruleaza(&[
        "deploy",
        "--sursa",
        inst.to_str().unwrap(),
        "--referinta",
        "Referinta",
        "-d",
        dep.to_str().unwrap(),
        "--doar",
        "dbo.pStoc",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let sql = std::fs::read_to_string(dep.join("Client1.sql")).unwrap();
    assert!(sql.contains("dbo.pStoc"));
    assert!(!sql.contains("dbo.pClienti"), "{sql}");
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn instantaneu_scris_de_cli_se_poate_relua() {
    let d = dir("dus-intors");
    let primul = d.join("unu.json");
    scrie_instantaneu(&primul);
    let al_doilea = d.join("doi.json");

    // Un instantaneu citit dintr-un instantaneu: fluxul e acelasi, deci se
    // poate verifica fara server.
    let o = ruleaza(&[
        "instantaneu",
        "--sursa",
        primul.to_str().unwrap(),
        "-o",
        al_doilea.to_str().unwrap(),
        "--nota",
        "proba",
    ]);
    assert!(o.status.success(), "{}", err(&o));

    let o = ruleaza(&[
        "compara",
        "--sursa",
        al_doilea.to_str().unwrap(),
        "--referinta",
        "Referinta",
        "--json",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(v["diferite"], 1);
    assert_eq!(v["lipsa"], 1);

    let text = std::fs::read_to_string(&al_doilea).unwrap();
    assert!(
        text.contains("\"nota\": \"proba\""),
        "nota trebuie pastrata"
    );
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn eșecurile_dau_cod_nenul_si_scriu_pe_erori() {
    let d = dir("esecuri");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let i = inst.to_str().unwrap();

    let cazuri: Vec<(Vec<&str>, &str)> = vec![
        (
            vec!["baze", "--sursa", "/cale/inexistenta.json"],
            "GM-DB-007",
        ),
        (
            vec!["compara", "--sursa", i, "--referinta", "Inexistenta"],
            "nu exista in sursa",
        ),
        (vec!["baze"], "--sursa"),
        (
            vec!["baze", "--server", "SRV01", "--utilizator", "sa"],
            "GMOTION_PAROLA",
        ),
    ];
    for (args, asteptat) in cazuri {
        let o = ruleaza(&args);
        assert!(
            !o.status.success(),
            "{args:?} ar trebui sa eșueze, dar a ieșit cu 0"
        );
        assert!(
            err(&o).contains(asteptat),
            "{args:?}: lipseste {asteptat:?} din mesaj:\n{}",
            err(&o)
        );
        assert!(
            out(&o).is_empty(),
            "{args:?}: eroarea nu are ce caută la ieșirea standard:\n{}",
            out(&o)
        );
    }
    std::fs::remove_dir_all(&d).ok();
}

#[test]
fn parola_nu_se_accepta_ca_argument() {
    // Daca cineva adauga vreodata `--parola`, testul asta trebuie sa-l
    // opreasca: un argument ajunge in istoricul shell-ului si in lista de
    // procese.
    let o = ruleaza(&[
        "baze",
        "--server",
        "SRV01",
        "--utilizator",
        "sa",
        "--parola",
        "x",
    ]);
    assert!(!o.status.success());
    let e = err(&o);
    assert!(
        e.contains("unexpected argument") || e.contains("--parola"),
        "`--parola` nu trebuie sa existe:\n{e}"
    );
}

#[test]
fn server_si_sursa_nu_se_pot_da_impreuna() {
    let o = ruleaza(&["baze", "--server", "SRV01", "--sursa", "x.json"]);
    assert!(!o.status.success());
    assert!(err(&o).contains("cannot be used with"), "{}", err(&o));
}

#[test]
fn ajutorul_spune_de_ce_nu_exista_parola_ca_argument() {
    let o = ruleaza(&["--help"]);
    assert!(o.status.success());
    let t = out(&o);
    assert!(t.contains("GMOTION_PAROLA"), "{t}");
    assert!(t.contains("istoricul"), "{t}");
    assert!(
        t.contains("--sursa"),
        "ajutorul trebuie sa arate fluxul cu instantaneu:\n{t}"
    );
}

/// Jurnalul nu pune coduri de culoare cand nu scrie pe un terminal. La prima
/// rulare pe Windows, `etapa6-rezultat.txt` era plin de `[2m...[0m`.
#[test]
fn jurnalul_redirectat_nu_are_coduri_de_culoare() {
    let d = dir("culori");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let o = Command::new(EXE)
        .args(["baze", "--sursa", inst.to_str().unwrap()])
        .env("GMOTION_JURNAL", "info")
        .env_remove("NO_COLOR")
        .output()
        .expect("binarul gmotion exista");
    assert!(o.status.success(), "{}", err(&o));
    let e = err(&o);
    assert!(
        e.contains("INFO"),
        "testul are nevoie de jurnal pe erori:\n{e}"
    );
    assert!(
        !e.contains('\u{1b}'),
        "coduri ANSI in ieșire redirectata:\n{e:?}"
    );
}

/// Deploy-ul scrie definitia exacta din baza, nu textul de afisare. Inainte
/// (si in v3.4), spatiile de la capat de linie si `\r\n` se schimbau si in
/// interiorul string-urilor, deci procedurile care construiesc text de latime
/// fixa sau SQL dinamic se schimbau tacut in productie. Gasit la audit.
#[test]
fn deploy_scrie_definitia_originala_octet_cu_octet() {
    let d = dir("deploy-brut");
    let inst = d.join("inst.json");
    let definitie = "CREATE PROCEDURE dbo.pSpatii AS\r\nSELECT 'linia1   \r\nlinia2' AS x   \r\n";
    let json = serde_json::json!({
        "versiune": 1, "server": "S", "creat": "t", "nota": "",
        "baze": {
            "Ref": [{"schema": "dbo", "nume": "pSpatii", "tip": "P", "definitie": definitie}],
            "T": []
        }
    });
    std::fs::write(&inst, json.to_string()).unwrap();
    let iesire = d.join("out");
    let o = ruleaza(&[
        "deploy",
        "--sursa",
        inst.to_str().unwrap(),
        "--referinta",
        "Ref",
        "-d",
        iesire.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let fisier = std::fs::read_dir(&iesire)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let text = std::fs::read_to_string(fisier).unwrap();
    let asteptat = definitie.replacen("CREATE PROCEDURE", "CREATE OR ALTER PROCEDURE", 1);
    assert!(
        text.contains(&asteptat),
        "definitia s-a schimbat:\n{text:?}"
    );
}

/// `--detalii` pe binarul adevarat: in ce difera obiectul, nu doar cat de
/// mult. Pe instantaneul de proba, `pClienti` difera intr-o cifra din cod
/// (`activ = 1` fata de `activ = 0`).
#[test]
fn compara_cu_detalii_spune_in_ce_difera_obiectul() {
    let d = dir("detalii");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let i = inst.to_str().unwrap();

    let o = ruleaza(&[
        "compara",
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "--detalii",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let t = out(&o);
    assert!(t.contains("DIFERA IN"), "lipseste coloana:\n{t}");
    assert!(t.contains("cod"), "pClienti difera in cod:\n{t}");

    // JSON: detaliile sunt legate de rand prin id, si numai cu --detalii
    let o = ruleaza(&[
        "compara",
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "--detalii",
        "--json",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o))
        .unwrap_or_else(|e| panic!("ieșirea standard nu e JSON curat ({e}):\n{}", out(&o)));
    let det = v["detalii"].as_array().expect("campul detalii");
    assert_eq!(det.len(), 1, "doar obiectul diferit are detalii: {det:?}");
    assert_eq!(det[0]["obiect"], "dbo.pClienti");
    assert_eq!(det[0]["difera_in"], "cod");
    assert_eq!(det[0]["linii_plus"], 1);
    assert_eq!(det[0]["linii_minus"], 1);
    let id = det[0]["id"].as_u64().unwrap();
    let rand = v["randuri"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"].as_u64() == Some(id))
        .expect("id-ul din detalii exista intre randuri");
    assert_eq!(rand["obiect"], "dbo.pClienti");

    // fara --detalii, nici campul, nici coloana
    let o = ruleaza(&[
        "compara",
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "--json",
    ]);
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert!(v.get("detalii").is_none(), "{v}");
}

/// Bifa de revizuit, pe binarul adevarat: se salveaza, se reia, si **cade
/// singura** cand obiectul s-a schimbat pe server. Ultima parte e tot rostul
/// amprentei, deci se probeaza, nu se presupune.
#[test]
fn revizuirea_se_tine_minte_si_cade_cand_obiectul_se_schimba() {
    let d = dir("revizuire");
    let date = d.join("date");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let i = inst.to_str().unwrap();
    let args = |extra: &[&str]| -> Vec<String> {
        let mut v: Vec<String> = vec![
            "revizuire".into(),
            "--sursa".into(),
            i.to_string(),
            "--referinta".into(),
            "Referinta".into(),
        ];
        v.extend(extra.iter().map(|x| x.to_string()));
        v
    };
    let ruleaza_rev = |extra: &[&str]| -> Output {
        let v = args(extra);
        let refs: Vec<&str> = v.iter().map(String::as_str).collect();
        ruleaza_in(&refs, Some(&date))
    };

    // la inceput, nimic bifat
    let o = ruleaza_rev(&["--json"]);
    assert!(o.status.success(), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).expect("JSON valid");
    assert_eq!(v["revizuite"], 0);
    assert_eq!(v["cazute"], 0);

    // marcam obiectul modificat
    let o = ruleaza_rev(&["--marcheaza", "dbo.pClienti"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = ruleaza_rev(&["--json"]);
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(v["revizuite"], 1, "bifa trebuie sa se fi salvat: {v}");
    assert_eq!(v["obiecte"][0], "dbo.pClienti");

    // acelasi obiect, alta definitie in baza comparata: bifa cade
    let schimbat = d.join("schimbat.json");
    let text = std::fs::read_to_string(&inst).unwrap();
    std::fs::write(
        &schimbat,
        text.replace("WHERE activ = 0", "WHERE activ = 0 AND sters = 0"),
    )
    .unwrap();
    let o = ruleaza_in(
        &[
            "revizuire",
            "--sursa",
            schimbat.to_str().unwrap(),
            "--referinta",
            "Referinta",
            "--json",
        ],
        Some(&date),
    );
    assert!(o.status.success(), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(
        v["revizuite"], 0,
        "obiectul s-a schimbat, bifa nu mai e valabila"
    );
    assert_eq!(v["cazute"], 1);
    assert_eq!(v["obiecte"].as_array().unwrap().len(), 0);

    // un obiect care nu e in comparare: eroare clara, nu bifa tacuta
    let o = ruleaza_rev(&["--marcheaza", "dbo.nuExista"]);
    assert!(!o.status.success());
    assert!(err(&o).contains("dbo.nuExista"), "{}", err(&o));

    // scoaterea si golirea
    let o = ruleaza_rev(&["--scoate", "dbo.pClienti"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = ruleaza_rev(&["--json"]);
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(v["revizuite"], 0);
    let o = ruleaza_rev(&["--goleste"]);
    assert!(o.status.success(), "{}", err(&o));
}
