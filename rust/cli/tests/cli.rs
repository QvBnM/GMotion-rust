//! Teste pe binarul adevarat, nu pe functii interne.
//!
//! Rostul: ce am verificat o data de mana sa ramana verificat. Probele manuale
//! care nu devin teste se pierd, si atunci urmatoarea regresie se descopera la
//! utilizator.
//!
//! Acoperite anume:
//! - `--json` scoate **numai** JSON la iesirea standard (jurnalul si progresul
//!   la erori). Prima versiune a jurnalului scria la stdout si facea iesirea
//!   neparsabila;
//! - codul de iesire e nenul la esec, altfel o rulare programata ar raporta
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
        // Fara jurnal in teste: nu schimba iesirea standard, dar face
        // diagnosticarea unui esec mai curata.
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

    // --- compara, JSON: numai JSON la iesirea standard ---
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
        .unwrap_or_else(|e| panic!("iesirea standard nu e JSON curat ({e}):\n{}", out(&o)));
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
    // plasarea: unde ajunge blocul daca apesi Aplica. Fara ea, drag & drop si
    // randul-fantoma din editor n-au ce intreba, si nimic nu se poate depana
    // din linia de comanda (CLAUDE.md, sectiunea 3).
    assert!(
        t.contains("Plasarea blocurilor din martor"),
        "lipseste plasarea:\n{t}"
    );
    assert!(
        t.lines()
            .any(|l| l.contains("-> inlocuieste blocul") && l.contains("potrivire")),
        "plasarea nu spune unde cade blocul:\n{t}"
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
    // In mod text progresul merge la iesirea standard; doar pe --json se muta
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
        // Doar fisierele: `revenire/` e subdirector, separat de deploy.
        let n = std::fs::read_dir(&out_dir)
            .unwrap()
            .filter(|e| e.as_ref().unwrap().path().is_file())
            .count();
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
fn esecurile_dau_cod_nenul_si_scriu_pe_erori() {
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
            "{args:?} ar trebui sa esueze, dar a iesit cu 0"
        );
        assert!(
            err(&o).contains(asteptat),
            "{args:?}: lipseste {asteptat:?} din mesaj:\n{}",
            err(&o)
        );
        assert!(
            out(&o).is_empty(),
            "{args:?}: eroarea nu are ce cauta la iesirea standard:\n{}",
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
        "coduri ANSI in iesire redirectata:\n{e:?}"
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
    // Numit explicit: in director sta si `revenire/`, iar ordinea din
    // `read_dir` nu e garantata.
    let text = std::fs::read_to_string(iesire.join("T.sql")).unwrap();
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
        .unwrap_or_else(|e| panic!("iesirea standard nu e JSON curat ({e}):\n{}", out(&o)));
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

/// Garda „baza n-a mai fost atinsa", pe binar: planul tine amprentele de la
/// comparare; daca tinta s-a schimbat intre timp, deploy-ul se opreste cu
/// `GM-MOD-012`, spune obiectul pe nume si nu scrie nimic.
#[test]
fn deploy_cu_plan_se_opreste_daca_baza_s_a_schimbat() {
    let d = dir("plan-garda");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let plan = d.join("plan.json");
    let o = ruleaza(&[
        "plan",
        "--sursa",
        inst.to_str().unwrap(),
        "--referinta",
        "Referinta",
        "-o",
        plan.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let p: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&plan).unwrap()).unwrap();
    assert_eq!(
        p["pasi"].as_array().unwrap().len(),
        2,
        "pClienti si pStoc: {p}"
    );
    assert_eq!(p["sarite"], 1, "pLocal exista doar in tinta");

    // Intre plan si deploy, cineva schimba pClienti pe Client1.
    let text = std::fs::read_to_string(&inst).unwrap();
    let schimbat = d.join("schimbat.json");
    std::fs::write(
        &schimbat,
        text.replace("WHERE activ = 0", "WHERE activ = 2"),
    )
    .unwrap();
    let iesire = d.join("out");
    let o = ruleaza(&[
        "deploy",
        "--sursa",
        schimbat.to_str().unwrap(),
        "--referinta",
        "Referinta",
        "--plan",
        plan.to_str().unwrap(),
        "-d",
        iesire.to_str().unwrap(),
    ]);
    assert!(!o.status.success(), "garda trebuia sa opreasca");
    let e = err(&o);
    assert!(e.contains("GM-MOD-012"), "{e}");
    assert!(e.contains("Client1: dbo.pClienti a fost modificat"), "{e}");
    assert!(!iesire.exists(), "nu trebuia scris nimic");

    // Pe baza neatinsa, acelasi plan trece si scrie si revenirea.
    let o = ruleaza(&[
        "--json",
        "deploy",
        "--sursa",
        inst.to_str().unwrap(),
        "--referinta",
        "Referinta",
        "--plan",
        plan.to_str().unwrap(),
        "-d",
        iesire.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let r: serde_json::Value = serde_json::from_str(&out(&o)).expect("--json scoate numai JSON");
    assert_eq!(r["obiecte"], 2);
    assert_eq!(r["revenire"].as_array().unwrap().len(), 1, "{r}");
}

/// Scriptul de revenire se scrie din definitiile actuale ale tintei: ce
/// exista se reface cum e acum, ce deploy-ul creeaza se sterge.
#[test]
fn deploy_scrie_revenirea_din_tinta() {
    let d = dir("revenire");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let iesire = d.join("out");
    let o = ruleaza(&[
        "deploy",
        "--sursa",
        inst.to_str().unwrap(),
        "--referinta",
        "Referinta",
        "-d",
        iesire.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let rev = std::fs::read_to_string(iesire.join("revenire").join("Client1.sql")).unwrap();
    assert!(rev.contains("USE [Client1];"));
    assert!(rev.contains("WHERE activ = 0"), "codul din tinta:\n{rev}");
    assert!(
        !rev.contains("WHERE activ = 1"),
        "nu codul din referinta:\n{rev}"
    );
    assert!(
        rev.contains("DROP PROCEDURE IF EXISTS [dbo].[pStoc];"),
        "{rev}"
    );
    assert!(
        !rev.contains("pLocal"),
        "extra nu e in deploy, deci nici in revenire"
    );
}

#[test]
fn planul_altei_referinte_e_refuzat() {
    let d = dir("plan-ref");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let plan = d.join("plan.json");
    let o = ruleaza(&[
        "plan",
        "--sursa",
        inst.to_str().unwrap(),
        "--referinta",
        "Referinta",
        "-o",
        plan.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let o = ruleaza(&[
        "deploy",
        "--sursa",
        inst.to_str().unwrap(),
        "--referinta",
        "Client1",
        "--plan",
        plan.to_str().unwrap(),
        "-d",
        d.join("out").to_str().unwrap(),
    ]);
    assert!(!o.status.success());
    assert!(err(&o).contains("martorul"), "{}", err(&o));
}

/// „Aplica", „Omite" si deploy-ul cu textul final, fara fereastra
/// (CLAUDE.md §3), pe binar: textul final se salveaza in sesiune si ajunge
/// in deploy; obiectul omis nu.
#[test]
fn editarea_si_omiterea_ajung_in_deploy() {
    let d = dir("editare");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let date = d.join("date");
    let i = inst.to_str().unwrap();
    let baza = [
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "--baza",
        "Client1",
        "--obiect",
    ];

    // pClienti difera (activ = 1 / 0): o diferenta, numerotata 1
    let mut a: Vec<&str> = vec!["--json", "editare"];
    a.extend(baza);
    a.push("dbo.pClienti");
    let o = ruleaza_in(&a, Some(&date));
    assert!(o.status.success(), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(v["editat"], false);
    assert_eq!(v["diferente"].as_array().unwrap().len(), 1, "{v}");

    // numar inexistent: eroare, cod nenul
    let mut a: Vec<&str> = vec!["editare"];
    a.extend(baza);
    a.extend(["dbo.pClienti", "--aplica", "7"]);
    let o = ruleaza_in(&a, Some(&date));
    assert!(!o.status.success());
    assert!(err(&o).contains("nu exista diferenta 7"), "{}", err(&o));

    // un text scris de mana
    let manual = d.join("manual.sql");
    std::fs::write(
        &manual,
        "CREATE PROCEDURE dbo.pClienti AS\nSELECT nume FROM Clienti WHERE activ = 42",
    )
    .unwrap();
    let mut a: Vec<&str> = vec!["--json", "editare"];
    a.extend(baza);
    a.extend(["dbo.pClienti", "--din-fisier", manual.to_str().unwrap()]);
    let o = ruleaza_in(&a, Some(&date));
    assert!(o.status.success(), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(
        (v["editat"].as_bool(), v["manual"].as_bool()),
        (Some(true), Some(true))
    );

    // pStoc se omite
    let o = ruleaza_in(
        &[
            "revizuire",
            "--sursa",
            i,
            "--referinta",
            "Referinta",
            "--omite",
            "dbo.pStoc",
        ],
        Some(&date),
    );
    assert!(o.status.success(), "{}", err(&o));

    let iesire = d.join("out");
    let o = ruleaza_in(
        &[
            "deploy",
            "--sursa",
            i,
            "--referinta",
            "Referinta",
            "-d",
            iesire.to_str().unwrap(),
        ],
        Some(&date),
    );
    assert!(o.status.success(), "{}", err(&o));
    let sql = std::fs::read_to_string(iesire.join("Client1.sql")).unwrap();
    assert!(sql.contains("WHERE activ = 42"), "textul final:\n{sql}");
    assert!(!sql.contains("WHERE activ = 1"), "nu codul din referinta");
    assert!(!sql.contains("pStoc"), "pStoc e omis:\n{sql}");

    // reset: inapoi la codul din referinta
    let mut a: Vec<&str> = vec!["editare"];
    a.extend(baza);
    a.extend(["dbo.pClienti", "--reset"]);
    assert!(ruleaza_in(&a, Some(&date)).status.success());
    let iesire2 = d.join("out2");
    let o = ruleaza_in(
        &[
            "deploy",
            "--sursa",
            i,
            "--referinta",
            "Referinta",
            "-d",
            iesire2.to_str().unwrap(),
        ],
        Some(&date),
    );
    assert!(o.status.success(), "{}", err(&o));
    let sql = std::fs::read_to_string(iesire2.join("Client1.sql")).unwrap();
    assert!(
        sql.contains("WHERE activ = 1") && !sql.contains("activ = 42"),
        "{sql}"
    );
}

/// Deploy-ul scris din `cli` tine minte obiectele ca „deployate", cu amprenta
/// lor: coloana Deploy din fereastra se umple si cand deploy-ul s-a facut din
/// linia de comanda. Fara asta, cele doua jumatati ale aplicatiei ar tine
/// evidente separate — exact divergenta din v3.4.
#[test]
fn deploy_ul_din_cli_tine_minte_obiectele_deployate() {
    let d = dir("deploy-marcaj");
    let date = d.join("date");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let o = ruleaza_in(
        &[
            "deploy",
            "--sursa",
            inst.to_str().unwrap(),
            "--referinta",
            "Referinta",
            "-d",
            d.join("dep").to_str().unwrap(),
        ],
        Some(&date),
    );
    assert!(o.status.success(), "{}", err(&o));

    let sesiune = cauta_sesiunea(&date);
    let v: serde_json::Value = serde_json::from_str(&sesiune).expect("sesiune JSON valida");
    let deployate = &v["date"]["revizuire"]["deployate"]["Client1"];
    assert!(
        deployate["dbo.pClienti"].is_object() && deployate["dbo.pStoc"].is_object(),
        "obiectele deployate lipsesc din sesiune: {v}"
    );
    assert!(
        deployate["dbo.pLocal"].is_null(),
        "ce exista doar in tinta nu s-a deployat, deci nu se marcheaza: {v}"
    );
    assert!(
        deployate["dbo.vFacturi"].is_null(),
        "obiectul cu diferenta cosmetica nu intra in deploy: {v}"
    );
    assert!(
        deployate["dbo.pStoc"]["curent"].is_null(),
        "pStoc nu exista in tinta: n-are amprenta acolo: {v}"
    );
    // marcajul e pe amprenta, nu pe nume
    assert!(
        deployate["dbo.pClienti"]["referinta"].is_string(),
        "fara amprenta, marcajul n-ar cadea cand obiectul se schimba: {v}"
    );
}

/// Singurul fisier de sesiune din directorul de date al testului, ca JSON.
/// Pe Linux sesiunea se scrie in clar, cu un antet care spune asta.
fn cauta_sesiunea(date: &Path) -> String {
    fn cauta(dir: &Path, gasite: &mut Vec<PathBuf>) {
        let Ok(intrari) = std::fs::read_dir(dir) else {
            return;
        };
        for e in intrari.flatten() {
            let cale = e.path();
            if cale.is_dir() {
                cauta(&cale, gasite);
            } else if cale.extension().is_some_and(|x| x == "dat") {
                gasite.push(cale);
            }
        }
    }
    let mut gasite = Vec::new();
    cauta(date, &mut gasite);
    assert_eq!(gasite.len(), 1, "o singura sesiune salvata: {gasite:?}");
    let text = std::fs::read_to_string(&gasite[0]).unwrap();
    let (antet, json) = text
        .split_once('\0')
        .expect("sesiunea are antetul care spune cum e scrisa");
    assert_eq!(antet, "GMOTION-CLAR", "pe Linux sesiunea se scrie in clar");
    json.to_string()
}

/// `impact` pe binarul adevarat: cine foloseste obiectul vine din codul bazei
/// comparate, iar JSON-ul are nivelul si motivele.
#[test]
fn impactul_arata_cine_foloseste_obiectul() {
    let d = dir("impact");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let text = std::fs::read_to_string(&inst).unwrap();
    std::fs::write(
        &inst,
        text.replace(
            "dbo.pLocal AS\\nSELECT 1",
            "dbo.pLocal AS\\nEXEC dbo.pClienti -- dbo.vFacturi",
        ),
    )
    .unwrap();
    let i = inst.to_str().unwrap();
    let o = ruleaza(&[
        "impact",
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "--obiect",
        "dbo.pClienti",
        "--json",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).expect("JSON valid");
    let x = &v["obiecte"][0];
    assert_eq!(x["obiect"], "dbo.pClienti");
    assert_eq!(x["folosit_de"], 1);
    assert_eq!(x["cine_il_foloseste"][0], "dbo.pLocal");
    assert!(x["nivel"].is_string());

    // si invers: pLocal (Extra, doar in Client1) foloseste pClienti, luat
    // din Client1, singurul loc unde exista pLocal
    let o = ruleaza(&[
        "impact",
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "--obiect",
        "dbo.pLocal",
        "--json",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).expect("JSON valid");
    assert_eq!(v["obiecte"][0]["foloseste"][0], "dbo.pClienti", "{v}");
    let o = ruleaza(&[
        "impact",
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "--obiect",
        "dbo.pLocal",
    ]);
    assert!(out(&o).contains("foloseste  dbo.pClienti"), "{}", out(&o));

    // vFacturi e doar in comentariu: nu e folosit
    let o = ruleaza(&["impact", "--sursa", i, "--referinta", "Referinta"]);
    assert!(o.status.success(), "{}", err(&o));
    let t = out(&o);
    assert!(
        t.contains("dbo.pClienti") && t.contains("folosit de 1 obiect"),
        "{t}"
    );

    let o = ruleaza(&[
        "impact",
        "--sursa",
        i,
        "--referinta",
        "Referinta",
        "--obiect",
        "dbo.nuExista",
    ]);
    assert!(!o.status.success());
}

/// Nota pusa din `cli` se salveaza in sesiune, se arata, si se sterge cu
/// text gol.
#[test]
fn nota_se_pune_si_se_sterge_din_cli() {
    let d = dir("nota");
    let date = d.join("date");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let i = inst.to_str().unwrap();
    let rev = |extra: &[&str]| -> Output {
        let mut v = vec!["revizuire", "--sursa", i, "--referinta", "Referinta"];
        v.extend_from_slice(extra);
        ruleaza_in(&v, Some(&date))
    };
    let o = rev(&["--nota", "dbo.pClienti", "--text", "a ramas aici"]);
    assert!(o.status.success(), "{}", err(&o));
    assert!(out(&o).contains("a ramas aici"), "{}", out(&o));
    let o = rev(&["--json"]);
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(v["note"][0]["obiect"], "dbo.pClienti");
    assert_eq!(v["note"][0]["text"], "a ramas aici");
    let o = rev(&["--nota", "dbo.pClienti", "--text", ""]);
    assert!(o.status.success(), "{}", err(&o));
    let o = rev(&["--json"]);
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(v["note"].as_array().map(Vec::len), Some(0));
    // fara --text, clap refuza
    assert!(!rev(&["--nota", "dbo.pClienti"]).status.success());
}

/// C3 / C4 din `cercetare/audit-qa-2026-11-18.md`: `revizuire --baze T2` sau
/// `editare --baza T2` stergeau munca facuta pe T1, fiindca T1 n-avea randuri
/// in compararea de acum. Un „omis" pierdut inseamna obiect ajuns inapoi in
/// deploy.
#[test]
fn munca_unei_baze_nu_se_pierde_cand_compari_cu_alta() {
    let d = dir("doua-baze");
    let date = d.join("date");
    let inst = d.join("inst.json");
    std::fs::write(
        &inst,
        r#"{"versiune":1,"server":"SRV","creat":"2026-11-18T10:00:00Z","nota":"","baze":{
          "Ref":[
            {"schema":"dbo","nume":"pDiff","tip":"P","definitie":"CREATE PROC dbo.pDiff AS SELECT 1"},
            {"schema":"dbo","nume":"pCom","tip":"P","definitie":"CREATE PROC dbo.pCom AS SELECT 2"}],
          "T1":[
            {"schema":"dbo","nume":"pDiff","tip":"P","definitie":"CREATE PROC dbo.pDiff AS SELECT 11"},
            {"schema":"dbo","nume":"pCom","tip":"P","definitie":"CREATE PROC dbo.pCom AS SELECT 12"}],
          "T2":[
            {"schema":"dbo","nume":"pDiff","tip":"P","definitie":"CREATE PROC dbo.pDiff AS SELECT 21"},
            {"schema":"dbo","nume":"pCom","tip":"P","definitie":"CREATE PROC dbo.pCom AS SELECT 22"}]}}"#,
    )
    .unwrap();
    let i = inst.to_str().unwrap();
    let rev = |baza: &str, extra: &[&str]| -> Output {
        let mut v = vec![
            "revizuire",
            "--sursa",
            i,
            "--referinta",
            "Ref",
            "--baze",
            baza,
        ];
        v.extend_from_slice(extra);
        ruleaza_in(&v, Some(&date))
    };

    // ziua 1: bifa pe T1
    let o = rev("T1", &["--marcheaza", "dbo.pDiff"]);
    assert!(o.status.success(), "{}", err(&o));
    // ziua 2: alta baza, alta treaba
    let o = rev("T2", &["--omite", "dbo.pCom"]);
    assert!(o.status.success(), "{}", err(&o));

    // bifa de pe T1 e intacta
    let o = rev("T1", &["--json"]);
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(
        v["obiecte"].as_array().map(Vec::len),
        Some(1),
        "bifa din T1 trebuie sa fie intacta: {}",
        out(&o)
    );
    // si omisiunea de pe T2
    let o = rev("T2", &["--json"]);
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(
        v["omise"].as_array().map(Vec::len),
        Some(1),
        "omisiunea din T2 trebuie sa fie intacta: {}",
        out(&o)
    );

    // textul final al unei baze nu se pierde cand se editeaza alta
    let ed = |baza: &str, extra: &[&str]| -> Output {
        let mut v = vec![
            "editare",
            "--sursa",
            i,
            "--referinta",
            "Ref",
            "--baza",
            baza,
            "--obiect",
            "dbo.pDiff",
        ];
        v.extend_from_slice(extra);
        ruleaza_in(&v, Some(&date))
    };
    let fisier = d.join("scris.sql");
    std::fs::write(&fisier, "CREATE PROC dbo.pDiff AS SELECT 999").unwrap();
    let o = ed("T1", &["--din-fisier", fisier.to_str().unwrap()]);
    assert!(o.status.success(), "{}", err(&o));
    let o = ed("T2", &["--aplica-toate"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = ed("T1", &["--arata-final"]);
    assert!(
        out(&o).contains("SELECT 999"),
        "textul final al lui T1 trebuie sa fie intact:\n{}",
        out(&o)
    );
    std::fs::remove_dir_all(&d).ok();
}

/// F-C1, F-C2 si F-C3 din `cercetare/audit-qa-2026-11-18.md`:
/// `revizuire --marcheaza` bifa doar **prima** baza (tacut), `export` scria
/// mereu „NU" la Rezolvat si Omis, iar `editare --arata-final` amesteca
/// progresul in iesirea standard, deci fisierul redirectat incepea cu
/// „Citesc referinta...".
#[test]
fn cli_marcheaza_pe_toate_bazele_exporta_starea_si_nu_amesteca_progresul() {
    let d = dir("cli-fc");
    let date = d.join("date");
    let inst = d.join("inst.json");
    std::fs::write(
        &inst,
        r#"{"versiune":1,"server":"SRV","creat":"2026-11-19T10:00:00Z","nota":"","baze":{
          "Ref":[{"schema":"dbo","nume":"pDiff","tip":"P","definitie":"CREATE PROC dbo.pDiff AS SELECT 1"}],
          "T1":[{"schema":"dbo","nume":"pDiff","tip":"P","definitie":"CREATE PROC dbo.pDiff AS SELECT 11"}],
          "T2":[{"schema":"dbo","nume":"pDiff","tip":"P","definitie":"CREATE PROC dbo.pDiff AS SELECT 21"}]}}"#,
    )
    .unwrap();
    let i = inst.to_str().unwrap();

    // F-C1: o singura chemare bifeaza obiectul in ambele baze comparate
    let o = ruleaza_in(
        &[
            "revizuire",
            "--sursa",
            i,
            "--referinta",
            "Ref",
            "--marcheaza",
            "dbo.pDiff",
        ],
        Some(&date),
    );
    assert!(o.status.success(), "{}", err(&o));
    let o = ruleaza_in(
        &["revizuire", "--sursa", i, "--referinta", "Ref", "--json"],
        Some(&date),
    );
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(
        v["revizuite"],
        2,
        "bifa merge pe toate bazele comparate: {}",
        out(&o)
    );

    // F-C2: exportul spune ce s-a revizuit, nu „NU" peste tot
    let csv = d.join("export.csv");
    let o = ruleaza_in(
        &[
            "export",
            "--sursa",
            i,
            "--referinta",
            "Ref",
            "--iesire",
            csv.to_str().unwrap(),
        ],
        Some(&date),
    );
    assert!(o.status.success(), "{}", err(&o));
    let text = std::fs::read_to_string(&csv).unwrap();
    assert!(
        text.lines().skip(1).all(|l| l.contains(";DA")),
        "starea de revizuire ajunge in CSV:\n{text}"
    );

    // F-C3: cu --arata-final, iesirea standard e numai textul
    let o = ruleaza_in(
        &[
            "editare",
            "--sursa",
            i,
            "--referinta",
            "Ref",
            "--baza",
            "T1",
            "--obiect",
            "dbo.pDiff",
            "--arata-final",
        ],
        Some(&date),
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(
        out(&o).starts_with("CREATE PROC"),
        "nimic inaintea textului:\n{}",
        out(&o)
    );
    std::fs::remove_dir_all(&d).ok();
}

/// `editare` numara diferentele ca editorul (instructiunile consecutive care
/// merg in acelasi loc sunt una) si lasa o diferenta unde spune omul
/// (`--la`), ca tragerea in editor (etapa 23). Pe binarul adevarat.
#[test]
fn editarea_numara_pe_grupuri_si_lasa_unde_ii_spui() {
    let d = dir("editare-la");
    let inst = d.join("inst.json");
    let ob = |t: &str| serde_json::json!([{"schema":"dbo","nume":"rptInventar_Situatie","tip":"P","definitie":t}]);
    let v = serde_json::json!({
        "versiune": 1, "server": "srv", "creat": "", "nota": "",
        "baze": {
            "Martor": ob(include_str!("../../../cercetare/machete/editor-lizibil/martor.sql")),
            "Modificata": ob(include_str!("../../../cercetare/machete/editor-lizibil/modificata.sql")),
        }
    });
    std::fs::write(&inst, v.to_string()).unwrap();
    let date = d.join("date");
    let i = inst.to_str().unwrap();
    let baza = [
        "editare",
        "--sursa",
        i,
        "--referinta",
        "Martor",
        "--baza",
        "Modificata",
        "--obiect",
        "dbo.rptInventar_Situatie",
    ];
    let cu = |extra: &[&str], json: bool| {
        let mut a: Vec<&str> = if json { vec!["--json"] } else { vec![] };
        a.extend(baza);
        a.extend_from_slice(extra);
        ruleaza_in(&a, Some(&date))
    };

    let o = cu(&[], true);
    assert!(o.status.success(), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    let dif = v["diferente"].as_array().unwrap();
    assert_eq!(dif.len(), 3, "antet, grupul declare/set/select, @SQL: {v}");
    assert_eq!(dif[1]["instructiuni"], 4, "{v}");

    // --la cere o singura diferenta
    let o = cu(&["--aplica", "1,2", "--la", "3"], false);
    assert!(!o.status.success());
    assert!(err(&o).contains("o singura diferenta"), "{}", err(&o));

    // grupul lasat inaintea liniei 24 (linia goala de dupa `join`)
    let o = cu(&["--aplica", "2", "--la", "24"], false);
    assert!(o.status.success(), "{}", err(&o));
    let o = cu(&["--arata-final"], false);
    let t = out(&o);
    let linii: Vec<&str> = t.lines().collect();
    assert!(linii[22].starts_with("join #Person"), "{t}");
    assert!(
        linii[23].starts_with("declare  @InventoryCommissionName"),
        "{t}"
    );
    assert!(
        linii[26].starts_with("select @InventoryCommissionName = left("),
        "{t}"
    );
    assert_eq!(linii[27], "", "linia goala ramane dupa grup");
}

/// B1 din `cercetare/audit-qa-2026-11-22.md`: o sesiune care exista dar nu se
/// poate citi opreste deploy-ul, planul si exportul. Inainte, deploy-ul iesea
/// cu 0 si scria si obiectele omise.
#[test]
fn sesiunea_care_nu_se_citeste_opreste_deploy_ul() {
    let d = dir("sesiune-stricata");
    let inst = d.join("inst.json");
    scrie_instantaneu(&inst);
    let date = d.join("date");
    let i = inst.to_str().unwrap();
    let o = ruleaza_in(
        &[
            "revizuire",
            "--sursa",
            i,
            "--referinta",
            "Referinta",
            "--omite",
            "dbo.pStoc",
        ],
        Some(&date),
    );
    assert!(o.status.success(), "{}", err(&o));
    let _ = cauta_sesiunea(&date);
    fn sesiuni(dir: &Path, gasite: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let cale = e.path();
            if cale.is_dir() {
                sesiuni(&cale, gasite);
            } else if cale.extension().is_some_and(|x| x == "dat") {
                gasite.push(cale);
            }
        }
    }
    let mut gasite = Vec::new();
    sesiuni(&date, &mut gasite);
    let text = std::fs::read(&gasite[0]).unwrap();
    std::fs::write(&gasite[0], &text[..text.len() / 2]).unwrap();

    let iesire = d.join("out");
    let iesire_s = iesire.to_str().unwrap();
    let csv = d.join("e.csv");
    let plan = d.join("plan.json");
    for args in [
        vec![
            "deploy",
            "--sursa",
            i,
            "--referinta",
            "Referinta",
            "-d",
            iesire_s,
        ],
        vec![
            "plan",
            "--sursa",
            i,
            "--referinta",
            "Referinta",
            "-o",
            plan.to_str().unwrap(),
        ],
        vec![
            "export",
            "--sursa",
            i,
            "--referinta",
            "Referinta",
            "-o",
            csv.to_str().unwrap(),
        ],
    ] {
        let o = ruleaza_in(&args, Some(&date));
        assert!(
            !o.status.success(),
            "{}: trebuia sa cada pe sesiunea stricata\n{}",
            args[0],
            err(&o)
        );
        assert!(err(&o).contains("GM-PLAT-"), "{}: {}", args[0], err(&o));
    }
    assert!(!iesire.exists(), "niciun script scris");
    assert!(!plan.exists() && !csv.exists());
}
