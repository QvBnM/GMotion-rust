//! Fluxul care face bug-urile reproductibile: instantaneu scris -> citit ->
//! folosit ca sursa, fara nicio baza de date.
//!
//! Daca asta se rupe, se rupe singurul mod in care un bug de pe serverul
//! utilizatorului poate ajunge sa fie depanat de altcineva.

use gmotion_db::{Instantaneu, Obiect, SursaFixture, SursaObiecte};

fn obiect(schema: &str, nume: &str, tip: &str, def: &str) -> Obiect {
    Obiect {
        schema: schema.into(),
        nume: nume.into(),
        tip: tip.into(),
        definitie: def.into(),
        setari: None,
        modificat: None,
    }
}

fn dir_temporar(eticheta: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "gmotion-db-{}-{}-{:?}",
        eticheta,
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn instantaneu_scris_si_reluat_da_aceleasi_obiecte() {
    let dir = dir_temporar("dus-intors");
    let cale = dir.join("instantaneu.json");

    let mut original = Instantaneu::nou("SRV-PROD\\SQLEXPRESS", "2026-11-17T09:30:00Z");
    original.nota = "bug: comentariu bloc raportat ca doua blocuri sterse".into();
    original.baze.insert(
        "Referinta".into(),
        vec![
            obiect(
                "dbo",
                "pClienti",
                "P",
                "CREATE PROCEDURE dbo.pClienti AS\nSELECT 1",
            ),
            obiect(
                "dbo",
                "vFacturi",
                "V",
                "CREATE VIEW dbo.vFacturi AS\nSELECT 2",
            ),
        ],
    );
    original.baze.insert(
        "Client1".into(),
        vec![obiect(
            "dbo",
            "pClienti",
            "P",
            "CREATE PROCEDURE dbo.pClienti AS\nSELECT 2",
        )],
    );

    original.scrie(&cale).unwrap();

    let mut sursa = SursaFixture::din_fisier(&cale).unwrap();
    assert_eq!(sursa.baze().unwrap(), vec!["Client1", "Referinta"]);

    let ref_ob = sursa.obiecte("Referinta").unwrap();
    assert_eq!(ref_ob.len(), 2);
    assert_eq!(ref_ob[0].id(), "dbo.pClienti");
    assert_eq!(ref_ob[1].tip, "V");
    // definitia trebuie sa treaca prin JSON neatinsa, inclusiv terminatorii
    assert!(ref_ob[0].definitie.contains('\n'));
    assert_eq!(
        ref_ob[0].definitie,
        "CREATE PROCEDURE dbo.pClienti AS\nSELECT 1"
    );

    assert!(sursa.descriere().contains("SRV-PROD"));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn obiecte_multe_merge_prin_trait_fara_baza_de_date() {
    let mut i = Instantaneu::nou("SRV01", "2026-11-17T00:00:00Z");
    for baza in ["A", "B", "C"] {
        i.baze.insert(
            baza.into(),
            vec![obiect("dbo", &format!("p{baza}"), "P", "SELECT 1")],
        );
    }
    let mut sursa = SursaFixture::din_instantaneu(i);
    let baze = sursa.baze().unwrap();

    let mut progres = Vec::new();
    let rez = sursa
        .obiecte_multe(&baze, &mut |i, n, b| progres.push(format!("{i}/{n} {b}")))
        .unwrap();

    assert_eq!(rez.len(), 3);
    assert_eq!(
        rez.iter().map(|(b, _)| b.as_str()).collect::<Vec<_>>(),
        ["A", "B", "C"]
    );
    assert_eq!(progres.len(), 4, "trei baze plus pasul de incheiere");
}

#[test]
fn definitiile_supravietuiesc_caracterelor_dificile() {
    let dir = dir_temporar("caractere");
    let cale = dir.join("i.json");

    let cazuri = [
        "CREATE PROC p AS SELECT 'diacritice ăîâșț'", // diacritice-permise: date de test
        "CREATE PROC p AS SELECT '\"ghilimele\" si \\ backslash'",
        "CREATE PROC p AS\r\nSELECT 'CRLF'",
        "CREATE PROC p AS SELECT 'tab\there'",
        "CREATE PROC p AS SELECT '{\"json\": \"in literal\"}'",
        "CREATE PROC p AS SELECT 'apostrof '' escapat'",
    ];

    let mut i = Instantaneu::nou("SRV01", "2026-11-17T00:00:00Z");
    i.baze.insert(
        "B".into(),
        cazuri
            .iter()
            .enumerate()
            .map(|(n, d)| obiect("dbo", &format!("p{n}"), "P", d))
            .collect(),
    );
    i.scrie(&cale).unwrap();

    let mut s = SursaFixture::din_fisier(&cale).unwrap();
    let citite = s.obiecte("B").unwrap();
    for (n, asteptat) in cazuri.iter().enumerate() {
        assert_eq!(&citite[n].definitie, asteptat, "obiectul p{n} s-a schimbat");
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn instantaneul_e_reproductibil_intre_scrieri() {
    let dir = dir_temporar("reproductibil");
    let (a, b) = (dir.join("a.json"), dir.join("b.json"));

    let mut i = Instantaneu::nou("SRV01", "2026-11-17T00:00:00Z");
    for baza in ["Zeta", "Alfa", "Mu"] {
        i.baze
            .insert(baza.into(), vec![obiect("dbo", "p", "P", "SELECT 1")]);
    }
    i.scrie(&a).unwrap();
    i.scrie(&b).unwrap();

    // Ordine stabila: altfel diff-ul a doua instantanee de pe acelasi server e
    // nefolositor, si asta e exact ce vrei sa poti citi cand urmaresti un bug.
    let (ta, tb) = (
        std::fs::read_to_string(&a).unwrap(),
        std::fs::read_to_string(&b).unwrap(),
    );
    assert_eq!(ta, tb);
    let poz_alfa = ta.find("Alfa").unwrap();
    let poz_mu = ta.find("Mu").unwrap();
    let poz_zeta = ta.find("Zeta").unwrap();
    assert!(
        poz_alfa < poz_mu && poz_mu < poz_zeta,
        "bazele nu sunt sortate"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// Granita dintre `db` si `core`: sursa da text brut, iar motorul il poate
/// consuma direct. Daca asta nu merge, granita e trasa gresit.
#[test]
fn obiectele_din_fixture_intra_direct_in_motor() {
    let mut i = Instantaneu::nou("SRV01", "2026-11-17T00:00:00Z");
    i.baze.insert(
        "Ref".into(),
        vec![obiect(
            "dbo",
            "p",
            "P",
            "CREATE PROC dbo.p AS\nSELECT a FROM t WHERE x = 1",
        )],
    );
    i.baze.insert(
        "Cur".into(),
        vec![obiect(
            "dbo",
            "p",
            "P",
            "CREATE PROC dbo.p AS\nSELECT a FROM t WHERE x = 2",
        )],
    );
    let mut s = SursaFixture::din_instantaneu(i);
    let r = s.obiecte("Ref").unwrap().remove(0);
    let c = s.obiecte("Cur").unwrap().remove(0);

    let scor = gmotion_core::scor_similaritate(
        &gmotion_core::pregateste_afisare(&r.definitie),
        &gmotion_core::pregateste_afisare(&c.definitie),
    );
    assert!(scor > 80.0 && scor < 100.0, "scor neasteptat: {scor}");

    let hash_r = gmotion_core::calc_hash(&gmotion_core::normalizeaza(&r.definitie));
    let hash_c = gmotion_core::calc_hash(&gmotion_core::normalizeaza(&c.definitie));
    assert_ne!(hash_r, hash_c);
}

/// Etapa 25: ultima modificare intra si iese din instantaneu; unul scris
/// inainte (fara camp) se citeste mai departe, cu `None`.
#[test]
fn ultima_modificare_trece_prin_instantaneu_iar_cele_vechi_merg() {
    let dir = dir_temporar("modificat");
    let cale = dir.join("i.json");
    let mut o = obiect("dbo", "p", "P", "CREATE PROC dbo.p AS SELECT 1");
    o.modificat = Some("2026-11-22 10:32:15".into());
    let mut i = Instantaneu::nou("SRV", "2026-11-22T10:00:00Z");
    i.baze.insert("B".into(), vec![o.clone()]);
    i.scrie(&cale).unwrap();
    let citit = Instantaneu::citeste(&cale).unwrap();
    assert_eq!(citit.baze["B"][0].modificat, o.modificat);

    std::fs::write(
        &cale,
        r#"{"versiune":1,"server":"S","creat":"x","nota":"","baze":{"B":[
          {"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS SELECT 1"}]}}"#,
    )
    .unwrap();
    let vechi = Instantaneu::citeste(&cale).unwrap();
    assert_eq!(vechi.baze["B"][0].modificat, None);
    std::fs::remove_dir_all(&dir).ok();
}

/// Etapa 26: un instantaneu nu are server, deci verificarea codului pe server
/// e refuzata clar, cu codul ei, nu tacut.
#[test]
fn verificarea_pe_server_e_refuzata_pe_instantaneu() {
    let mut i = Instantaneu::nou("SRV", "2026-11-22T10:00:00Z");
    i.baze.insert(
        "B".into(),
        vec![obiect("dbo", "p", "P", "CREATE PROC dbo.p AS SELECT 1")],
    );
    let mut s = SursaFixture::din_instantaneu(i);
    let e = s
        .verifica_cod("B", "CREATE OR ALTER PROC dbo.p AS SELECT 1", None)
        .expect_err("fara server");
    assert_eq!(e.cod(), "GM-DB-012");
    assert!(e.to_string().contains("nu e un server"), "{e}");
}
