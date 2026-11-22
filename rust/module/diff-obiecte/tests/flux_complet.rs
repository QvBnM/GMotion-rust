//! Fluxul complet al modulului, pe un instantaneu: citire -> comparare ->
//! diff -> deploy -> export. Fara baza de date si fara interfata.
//!
//! Daca asta trece, modulul e functional cap-coada. E si forma in care un bug
//! raportat de pe o baza reala devine test de regresie: se schimba doar
//! instantaneul.

use std::collections::BTreeMap;

use gmotion_db::{Instantaneu, Obiect, SursaFixture, SursaObiecte};
use gmotion_modul_diff_obiecte as modul;
use modul::{Categorie, DeDeployat, ModImpartire};

fn ob(nume: &str, tip: &str, def: &str) -> Obiect {
    Obiect {
        schema: "dbo".into(),
        nume: nume.into(),
        tip: tip.into(),
        definitie: def.into(),
        setari: None,
        modificat: None,
        creat: None,
    }
}

/// Referinta are patru obiecte. Client1: unul identic, unul modificat, unul
/// lipsa, plus unul propriu. Client2: totul identic in afara de unul cu
/// diferenta doar cosmetica, deci nu trebuie raportat.
fn instantaneu() -> Instantaneu {
    let mut i = Instantaneu::nou("SRV-TEST", "2026-11-17T10:00:00Z");
    i.baze.insert(
        "Referinta".into(),
        vec![
            ob("pIdentic", "P", "CREATE PROCEDURE dbo.pIdentic AS\nSELECT 1"),
            ob(
                "pModificat",
                "P",
                "CREATE PROCEDURE dbo.pModificat AS\nSELECT a FROM t WHERE x = 1\n\nSELECT b FROM u",
            ),
            ob("pLipsa", "P", "CREATE PROCEDURE dbo.pLipsa AS\nSELECT 3"),
            ob("vCosmetic", "V", "CREATE VIEW dbo.vCosmetic AS\nSELECT a FROM t"),
        ],
    );
    i.baze.insert(
        "Client1".into(),
        vec![
            ob("pIdentic", "P", "CREATE PROCEDURE dbo.pIdentic AS\nSELECT 1"),
            ob(
                "pModificat",
                "P",
                "CREATE PROCEDURE dbo.pModificat AS\nSELECT a FROM t WHERE x = 2\n\nSELECT b FROM u",
            ),
            ob("vCosmetic", "V", "CREATE VIEW dbo.vCosmetic AS\nSELECT a FROM t"),
            ob("pNumaiAici", "P", "CREATE PROCEDURE dbo.pNumaiAici AS\nSELECT 9"),
        ],
    );
    i.baze.insert(
        "Client2".into(),
        vec![
            ob("pIdentic", "P", "CREATE PROCEDURE dbo.pIdentic AS\nSELECT 1"),
            ob(
                "pModificat",
                "P",
                "CREATE PROCEDURE dbo.pModificat AS\nSELECT a FROM t WHERE x = 1\n\nSELECT b FROM u",
            ),
            ob("pLipsa", "P", "CREATE PROCEDURE dbo.pLipsa AS\nSELECT 3"),
            // acelasi cod, alta aranjare si un comentariu: hash diferit, cod identic
            ob(
                "vCosmetic",
                "V",
                "CREATE VIEW dbo.vCosmetic AS\n   SELECT   a   FROM t   -- nimic nou",
            ),
        ],
    );
    i
}

#[test]
fn comparare_diff_deploy_export() {
    let mut sursa = SursaFixture::din_instantaneu(instantaneu());

    // --- citire prin trait, exact ca de pe un server real ---
    let referinta = modul::pregateste(&sursa.obiecte("Referinta").unwrap());
    let altele: Vec<(String, modul::Baza)> = ["Client1", "Client2"]
        .iter()
        .map(|b| {
            (
                (*b).to_string(),
                modul::pregateste(&sursa.obiecte(b).unwrap()),
            )
        })
        .collect();

    let mut progres = Vec::new();
    let c = modul::compara("Referinta", &referinta, &altele, |p| progres.push(p));

    // --- clasificare ---
    let n = c.numaratoare;
    assert_eq!(n.total_referinta, 4);
    assert_eq!(n.lipsa, 1, "pLipsa doar din Client1");
    assert_eq!(n.extra, 1, "pNumaiAici doar in Client1");
    assert_eq!(n.diferite, 1, "pModificat doar in Client1");
    // Client1: pIdentic, vCosmetic = 2. Client2: pIdentic, pModificat, pLipsa,
    // vCosmetic (cosmetic, deci identic) = 4.
    assert_eq!(n.identice, 6);

    let cauta = |cat: Categorie| -> Vec<(&str, &str)> {
        c.randuri
            .iter()
            .filter(|r| r.categorie == cat)
            .map(|r| (r.obiect.as_str(), r.baza.as_str()))
            .collect()
    };
    assert_eq!(cauta(Categorie::Diferit), [("dbo.pModificat", "Client1")]);
    assert_eq!(cauta(Categorie::Lipsa), [("dbo.pLipsa", "Client1")]);
    assert_eq!(cauta(Categorie::Extra), [("dbo.pNumaiAici", "Client1")]);

    assert!(progres.last().unwrap().gata);
    assert_eq!(progres.last().unwrap().procent, 100);

    // --- diff la cerere, pe obiectul modificat ---
    let rand = c
        .randuri
        .iter()
        .find(|r| r.categorie == Categorie::Diferit)
        .unwrap();
    let r_afis = &referinta[&rand.obiect].afisare;
    let c_afis = &altele.iter().find(|(b, _)| *b == rand.baza).unwrap().1[&rand.obiect].afisare;
    let (diferente, rezumat) = modul::diferente_obiect(r_afis, c_afis);
    assert!(diferente.iter().any(|d| d.fel == modul::Fel::Modificat));
    assert!(!rezumat.is_empty());

    // --- deploy: codul din referinta, cu CREATE OR ALTER ---
    let mut pe_baza: BTreeMap<String, Vec<DeDeployat>> = BTreeMap::new();
    let mut sarite = 0usize;
    for r in &c.randuri {
        if r.categorie == Categorie::Extra {
            // exista doar in tinta: nu exista cod in referinta de adus
            sarite += 1;
            continue;
        }
        let cod = modul::aplica_create_or_alter(&referinta[&r.obiect].definitie, &r.obiect);
        pe_baza.entry(r.baza.clone()).or_default().push(DeDeployat {
            obiect: r.obiect.clone(),
            tip: r.tip.clone(),
            cod,
            setari: referinta[&r.obiect].setari,
        });
    }
    assert_eq!(sarite, 1);

    let dir = std::env::temp_dir().join(format!("gmotion-flux-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    let raport = modul::deploy::scrie(
        &dir,
        "Referinta",
        &pe_baza,
        ModImpartire::PerBaza,
        sarite,
        "2026-11-17 10:00",
    )
    .unwrap();

    assert_eq!(raport.fisiere.len(), 1, "doar Client1 are ce deploia");
    assert_eq!(raport.obiecte, 2, "pModificat si pLipsa");
    let sql = std::fs::read_to_string(&raport.fisiere[0]).unwrap();
    assert!(sql.contains("USE [Client1];"));
    assert_eq!(
        sql.matches("CREATE OR ALTER PROCEDURE").count(),
        2,
        "fiecare obiect trebuie rulabil pe o baza care il are sau nu:\n{sql}"
    );
    assert!(sql.contains("dbo.pModificat") && sql.contains("dbo.pLipsa"));
    // codul deployat e cel din REFERINTA, nu din tinta
    assert!(
        sql.contains("WHERE x = 1"),
        "s-a deployat codul din tinta:\n{sql}"
    );
    assert!(!sql.contains("WHERE x = 2"));

    // --- export CSV ---
    let csv = dir.join("export.csv");
    let scrise = modul::scrie_csv(&csv, &c.randuri, |id| modul::Stare {
        rezolvat: id == 0,
        omis: false,
    })
    .unwrap();
    assert_eq!(scrise, c.randuri.len());
    let brut = std::fs::read(&csv).unwrap();
    assert!(brut.starts_with(&[0xEF, 0xBB, 0xBF]), "lipseste BOM-ul");
    let text = String::from_utf8(brut[3..].to_vec()).unwrap();
    assert_eq!(
        text.lines().count(),
        c.randuri.len() + 1,
        "cap plus randuri"
    );
    assert!(
        text.contains("dbo.pNumaiAici"),
        "si obiectele extra se exporta"
    );

    std::fs::remove_dir_all(&dir).ok();
}

/// Modulul se inregistreaza la platforma si rutele lui functioneaza.
#[test]
fn se_inregistreaza_la_platforma() {
    use gmotion_platforma::{Registru, Ruta};

    let mut r = Registru::nou();
    r.inregistreaza(Box::new(modul::ModulDiffObiecte)).unwrap();

    match r.ruteaza("diff-obiecte/compara").unwrap() {
        Ruta::Modul { modul: m, metoda } => {
            assert_eq!(m.nume(), "diff-obiecte");
            assert_eq!(metoda, "compara");
        }
        Ruta::Platforma(_) => panic!("ar trebui sa fie metoda de modul"),
    }
    // metodele comune raman la platforma
    assert!(matches!(
        r.ruteaza("cauta_servere").unwrap(),
        Ruta::Platforma("cauta_servere")
    ));
}
