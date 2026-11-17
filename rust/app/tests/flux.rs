//! Drumul intreg, fara fereastra: din ecranul de conectare pana in editor.
//!
//! **Se strica astfel:** fiecare bucata merge separat, dar legate nu — ecranul
//! de conectare da o comparare pe care lista nu o poate arata, sau obiectul
//! deschis nu e cel selectat. Testul de unitate nu prinde asta, fiindca
//! fiecare parte e testata cu datele ei.

use gmotion_app::pornire;
use gmotion_app::rutare::{Aplicatie, Comanda, Ecran};
use gmotion_fereastra::model_conectare::Conectare;
use gmotion_fereastra::model_lista::Stare;
use gmotion_modul_diff_obiecte as modul;

fn instantaneu() -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("gmotion-flux-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let cale = d.join("inst.json");
    std::fs::write(
        &cale,
        r#"{"versiune":1,"server":"SRV-TEST","creat":"2026-11-17T10:00:00Z","nota":"","baze":{
          "CharismaDev":[
            {"schema":"dbo","nume":"pStoc","tip":"P","definitie":"CREATE PROC dbo.pStoc AS\nSELECT cantitate FROM Stoc WHERE activ = 1"},
            {"schema":"dbo","nume":"pMesaj","tip":"P","definitie":"CREATE PROC dbo.pMesaj AS\nRAISERROR('Nu s-au gasit date', 16, 1)"},
            {"schema":"dbo","nume":"pNou","tip":"P","definitie":"CREATE PROC dbo.pNou AS\nSELECT 1"}],
          "CharismaProd":[
            {"schema":"dbo","nume":"pStoc","tip":"P","definitie":"CREATE PROC dbo.pStoc AS\nSELECT cantitate FROM Stoc WHERE activ = 0"},
            {"schema":"dbo","nume":"pMesaj","tip":"P","definitie":"CREATE PROC dbo.pMesaj AS\nRAISERROR('nu s-au gasit date', 16, 1)"}]}}"#,
    )
    .unwrap();
    cale
}

#[test]
fn din_conectare_pana_in_editor() {
    let cale = instantaneu();
    let mut c = Conectare::noua();

    // 1. instantaneu deschis: nu se cer servere, dar se cer bazele
    c.instantaneu = Some(cale.to_string_lossy().to_string());
    assert_eq!(
        c.ce_lipseste().as_deref(),
        Some("alege baza sursă din instantaneu")
    );
    let (baze, stare) = pornire::citeste_bazele(&c, &c.sursa);
    assert_eq!(baze, ["CharismaDev", "CharismaProd"]);
    assert!(stare.e_conectat(), "{stare:?}");

    // 2. alegerea bazelor face butonul apasabil
    c.sursa.baza = "CharismaDev".into();
    c.baza.baza = "CharismaProd".into();
    assert!(c.poate_compara(), "{:?}", c.ce_lipseste());

    // 3. compararea
    let r = pornire::compara(&c).expect("compararea merge");
    let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinta, modul::Revizuire::noua());
    app.asigura_detalii(0, 10);

    // lista are cele trei obiecte, clasificate
    let stari: Vec<Stare> = app
        .lista()
        .vizibile()
        .iter()
        .map(|&i| app.lista().obiecte()[i].stare)
        .collect();
    assert_eq!(stari.len(), 3);
    assert_eq!(
        stari.iter().filter(|s| **s == Stare::Modificat).count(),
        2,
        "pStoc si pMesaj difera"
    );
    assert_eq!(
        stari.iter().filter(|s| **s == Stare::DoarSursa).count(),
        1,
        "pNou e doar in sursa"
    );

    // 4. „Difera in" spune unde e diferenta, nu doar cat de mare e
    let mesaj = (0..app.lista().vizibile().len())
        .find(|&p| app.lista().obiect(p).unwrap().nume == "pMesaj")
        .expect("obiectul exista");
    assert_eq!(
        app.lista()
            .obiect(mesaj)
            .unwrap()
            .detalii
            .as_ref()
            .unwrap()
            .difera_in,
        "un șir"
    );

    // 5. deschiderea din lista duce in editor, pe acelasi obiect
    app.comanda(Comanda::Selecteaza(mesaj));
    app.comanda(Comanda::Deschide);
    assert_eq!(app.ecran(), Ecran::Editor);
    assert_eq!(app.obiect().unwrap().nume, "pMesaj");
    let model = app.model_editor().expect("modelul editorului");
    assert_eq!(model.diferente.len(), 1);

    // 6. bifa din editor se vede in progresul listei
    assert!(app.comanda(Comanda::ComutaRevizuit));
    assert_eq!(app.lista().progres_revizuire(), (1, 3));

    // 7. obiectul care exista doar in sursa nu deschide editorul
    let doar_sursa = (0..app.lista().vizibile().len())
        .find(|&p| app.lista().obiect(p).unwrap().nume == "pNou")
        .expect("obiectul exista");
    app.comanda(Comanda::InapoiLaLista);
    app.comanda(Comanda::Selecteaza(doar_sursa));
    app.comanda(Comanda::Deschide);
    assert_eq!(app.ecran(), Ecran::Lista, "nu avem cu ce compara");

    std::fs::remove_dir_all(cale.parent().unwrap()).ok();
}

#[test]
fn o_baza_lipsa_din_instantaneu_nu_duce_in_lista() {
    let cale = instantaneu();
    let mut c = Conectare::noua();
    c.instantaneu = Some(cale.to_string_lossy().to_string());
    c.sursa.baza = "CharismaDev".into();
    c.baza.baza = "BazaCareNuExista".into();
    // `Comparata` nu are `Debug` (contine definitiile bazelor): se despacheteaza
    let e = match pornire::compara(&c) {
        Err(e) => e,
        Ok(_) => panic!("baza nu exista, compararea ar fi trebuit sa esueze"),
    };
    assert!(e.contains("BazaCareNuExista"), "{e}");
    std::fs::remove_dir_all(cale.parent().unwrap()).ok();
}
