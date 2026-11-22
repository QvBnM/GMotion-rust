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

/// Un instantaneu propriu fiecarui test: `cargo test` ruleaza testele in
/// paralel, iar un director comun ar fi sters de sub celalalt test — exact
/// genul de test care trece azi si cade aleator peste o luna.
fn instantaneu(eticheta: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("gmotion-flux-{eticheta}-{}", std::process::id()));
    std::fs::remove_dir_all(&d).ok();
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
    let cale = instantaneu("conectare-editor");
    let mut c = Conectare::noua();

    // 1. instantaneu deschis: nu se cer servere, dar se cer bazele
    c.instantaneu = Some(cale.to_string_lossy().to_string());
    assert_eq!(
        c.ce_lipseste().as_deref(),
        Some("alege baza martor din instantaneu")
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
    let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());
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
        "un sir"
    );

    // 5. deschiderea din lista duce in editor, pe acelasi obiect
    app.comanda(Comanda::Selecteaza(mesaj));
    app.comanda(Comanda::Deschide);
    assert_eq!(app.ecran(), Ecran::Editor);
    assert_eq!(app.obiect().unwrap().nume, "pMesaj");
    assert_eq!(
        app.model_editor()
            .expect("modelul editorului")
            .diferente
            .len(),
        1
    );

    // 6. bifa din editor se vede in progresul listei
    assert!(app.comanda(Comanda::ComutaRevizuit));
    assert_eq!(app.lista().progres_revizuire(), (1, 3));

    // 7. obiectul care exista doar in martor se deschide si el, cu tot codul
    // de adus (F-A9)
    let doar_sursa = (0..app.lista().vizibile().len())
        .find(|&p| app.lista().obiect(p).unwrap().nume == "pNou")
        .expect("obiectul exista");
    app.comanda(Comanda::InapoiLaLista);
    app.comanda(Comanda::Selecteaza(doar_sursa));
    app.comanda(Comanda::Deschide);
    assert_eq!(app.ecran(), Ecran::Editor);
    let model = app.model_editor().expect("modelul");
    assert!(!model.diferente.is_empty(), "tot codul e de adus");

    std::fs::remove_dir_all(cale.parent().unwrap()).ok();
}

#[test]
fn din_lista_pana_la_scripturile_de_deploy() {
    use gmotion_app::deploy::{Garda, StareDeploy};
    let cale = instantaneu("deploy");
    let mut c = Conectare::noua();
    c.instantaneu = Some(cale.to_string_lossy().to_string());
    c.sursa.baza = "CharismaDev".into();
    c.baza.baza = "CharismaProd".into();
    let r = pornire::compara(&c).expect("compararea merge");
    let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());

    // 1. doua obiecte in plan: unul modificat, unul nou
    for nume in ["pStoc", "pNou"] {
        let p = (0..app.lista().vizibile().len())
            .find(|&p| app.lista().obiect(p).unwrap().nume == nume)
            .unwrap();
        app.comanda(Comanda::Selecteaza(p));
        app.comanda(Comanda::ComutaInPlan);
    }
    app.comanda(Comanda::DeschideDeploy);
    assert_eq!(app.ecran(), Ecran::Deploy);
    let mut st = StareDeploy::nou(app.plan("CharismaDev", "2026-11-18 14:00"));
    let pasi = st.pasi_in_ordine(app.sursa());
    assert_eq!(pasi.len(), 2);

    // 2. garda, pe ce se reciteste acum din instantaneu
    let recitit = pornire::compara(&c).expect("recitirea merge");
    st.aplica_verificarea(recitit.sursa, recitit.tinte, "14:05");
    assert!(matches!(st.garda, Garda::Ok { .. }), "{:?}", st.garda);

    // 3. scrierea: scripturile si revenirea, pe disc
    let dir = cale.parent().unwrap().join("deploy");
    st.scrie(&dir, "2026-11-18 14:05", &Default::default());
    let raport = st
        .rezultat
        .as_ref()
        .unwrap()
        .as_ref()
        .expect("scrierea merge");
    assert_eq!(raport.obiecte, 2);
    let script: String = raport
        .fisiere
        .iter()
        .map(|f| std::fs::read_to_string(f).unwrap())
        .collect();
    assert!(script.contains("dbo.pStoc") && script.contains("dbo.pNou"));
    assert!(
        !script.contains("dbo.pMesaj"),
        "pMesaj n-a fost pus in plan"
    );
    assert!(!raport.revenire.is_empty(), "revenirea e scrisa");
    std::fs::remove_dir_all(cale.parent().unwrap()).ok();
}

#[test]
fn o_baza_lipsa_din_instantaneu_nu_duce_in_lista() {
    let cale = instantaneu("baza-lipsa");
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

/// Editorul pana in deploy: „Editeaza" (scris de mana) pe pStoc, in plan,
/// garda, scrierea — iar scriptul are textul final, nu codul din sursa.
#[test]
fn textul_final_din_editor_ajunge_in_scriptul_de_deploy() {
    use gmotion_app::deploy::StareDeploy;
    let cale = instantaneu("text-final");
    let mut c = Conectare::noua();
    c.instantaneu = Some(cale.to_string_lossy().to_string());
    c.sursa.baza = "CharismaDev".into();
    c.baza.baza = "CharismaProd".into();
    let r = pornire::compara(&c).expect("compararea merge");
    let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());
    let p = (0..app.lista().vizibile().len())
        .find(|&p| app.lista().obiect(p).unwrap().nume == "pStoc")
        .unwrap();
    app.comanda(Comanda::Selecteaza(p));
    app.comanda(Comanda::Deschide);
    assert_eq!(app.ecran(), Ecran::Editor);
    assert!(app.comanda(Comanda::SalveazaText {
        id: app.id_obiect().expect("un obiect deschis"),
        text: "CREATE PROC dbo.pStoc AS\nSELECT cantitate FROM Stoc WHERE activ = 7".into()
    }));
    app.comanda(Comanda::ComutaInPlan);
    let mut st = StareDeploy::nou(app.plan("CharismaDev", "acum"));
    let recitit = pornire::compara(&c).expect("recitirea merge");
    st.aplica_verificarea(recitit.sursa, recitit.tinte, "acum");
    let dir = cale.parent().unwrap().join("deploy");
    st.scrie(&dir, "acum", &app.texte_finale());
    let raport = st
        .rezultat
        .as_ref()
        .unwrap()
        .as_ref()
        .expect("scrierea merge");
    let script = std::fs::read_to_string(&raport.fisiere[0]).unwrap();
    assert!(
        script.contains("WHERE activ = 7"),
        "textul final:\n{script}"
    );
    assert!(!script.contains("WHERE activ = 1"), "nu codul din sursa");
    std::fs::remove_dir_all(cale.parent().unwrap()).ok();
}

/// Modul de previzualizare: `gmotion-ui --previzualizare` trebuie sa intre
/// direct in lista, pe instantaneul sintetic, fara server. Testul verifica
/// instantaneul insusi: daca nu mai acopera toate categoriile si tipurile,
/// previzualizarea nu-si mai face treaba, si asta se vede abia la Mihai pe
/// ecran.
#[test]
fn instantaneul_de_previzualizare_acopera_tot_ce_trebuie_probat() {
    use gmotion_fereastra::model_lista::Stare as StareLista;

    let cale = std::path::Path::new("../../referinta/instantanee/previzualizare.json");
    assert!(
        cale.is_file(),
        "lipseste {}; ruleaza unelte/genereaza-previzualizare.py",
        cale.display()
    );

    let mut c = Conectare::noua();
    c.instantaneu = Some(cale.display().to_string());
    let (baze, stare) = pornire::citeste_bazele(&c, &c.sursa);
    assert!(stare.e_conectat(), "{stare:?}");
    assert!(baze.len() >= 3, "mai multe baze comparate: {baze:?}");

    // ca in fereastra: prima baza e referinta, restul se bifeaza
    let (prima, restul) = baze.split_first().expect("cel putin o baza");
    c.sursa.baza = prima.clone();
    for b in restul {
        c.baza.comuta_baza(b);
    }
    assert!(c.poate_compara(), "{:?}", c.ce_lipseste());

    let r = pornire::compara(&c).expect("compararea merge pe instantaneu");
    assert_eq!(r.tinte.len(), baze.len() - 1);
    let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());
    app.asigura_detalii(0, 200);

    // toate categoriile, ca sa se vada fiecare culoare si fiecare filtru
    let stari: Vec<StareLista> = app
        .lista()
        .contoare_stari()
        .into_iter()
        .filter(|(_, n)| *n > 0)
        .map(|(s, _)| s)
        .collect();
    for ceruta in [
        StareLista::Modificat,
        StareLista::DoarSursa,
        StareLista::DoarBaza,
        StareLista::Identic,
        StareLista::Criptat,
    ] {
        assert!(stari.contains(&ceruta), "lipseste categoria {ceruta:?}");
    }

    // toate tipurile din machete
    let tipuri: Vec<String> = app
        .lista()
        .obiecte()
        .iter()
        .map(|o| o.tip.clone())
        .collect();
    for t in ["P", "V", "FN", "IF", "TF", "TR"] {
        assert!(tipuri.iter().any(|x| x == t), "lipseste tipul {t}");
    }

    // obiecte mari (de aia se vede daca lista se impiedica) si scoruri variate
    let linii_max = app
        .lista()
        .obiecte()
        .iter()
        .filter_map(|o| o.detalii.as_ref())
        .map(|d| d.linii_plus + d.linii_minus)
        .max()
        .unwrap_or(0);
    assert!(
        linii_max >= 10,
        "niciun obiect mare: {linii_max} linii mutate"
    );
    let scoruri: Vec<f64> = app
        .lista()
        .obiecte()
        .iter()
        .filter_map(|o| o.scor)
        .collect();
    assert!(
        scoruri.iter().any(|s| *s < 70.0) && scoruri.iter().any(|s| *s > 95.0),
        "scorurile nu acopera si rescrieri, si retusuri: {scoruri:?}"
    );

    // codul dezordonat chiar e dezordonat (tab-uri, comentarii imbricate, GO)
    let dezordonat = app
        .sursa()
        .values()
        .any(|o| o.afisare.contains("/*") && o.afisare.contains("GO"));
    assert!(dezordonat, "lipseste obiectul cu cod scris dezordonat");
}

/// Cap-coada pe cod scris urat: se ia obiectul cu ciclu de cursor din
/// instantaneul de previzualizare, se apasa „Aplica toate", si rezultatul
/// trebuie sa fie **acelasi cod ca in martor**, normalizat.
///
/// Invarianta e pe forma normalizata, nu pe text: un comentariu lipit de cod
/// muta granitele segmentarii, deci „zero diferente" ar fi prea tare (vezi
/// jurnalul etapei 14).
#[test]
fn pe_cod_deformat_aplica_toate_aduce_martorul_in_baza() {
    let cale = std::path::Path::new("../../referinta/instantanee/previzualizare.json");
    assert!(cale.is_file(), "lipseste {}", cale.display());

    let mut c = Conectare::noua();
    c.instantaneu = Some(cale.display().to_string());
    let (baze, _) = pornire::citeste_bazele(&c, &c.sursa);
    let (prima, restul) = baze.split_first().expect("cel putin o baza");
    c.sursa.baza = prima.clone();
    c.baza.comuta_baza(&restul[0]);

    let r = pornire::compara(&c).expect("compararea merge");
    let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());

    // obiectul cu cursor: ala e cel scris urat
    let pozitie = (0..app.lista().vizibile().len())
        .find(|&p| {
            app.lista()
                .obiect(p)
                .is_some_and(|o| o.nume == "uspReproceseazaFacturi")
        })
        .expect("obiectul cu cursor e in lista");
    app.comanda(Comanda::Selecteaza(pozitie));
    app.comanda(Comanda::Deschide);
    assert_eq!(app.ecran(), Ecran::Editor);

    assert!(
        !app.model_editor()
            .expect("editorul are model")
            .diferente
            .is_empty(),
        "obiectul chiar difera, altfel testul nu probeaza nimic"
    );
    // codul e deformat: majuscule amestecate si tab-uri, in ambele parti
    let (martor, baza) = app.texte_curente().expect("texte");
    for text in [&martor, &baza] {
        assert!(text.contains('\t'), "codul de proba are tab-uri");
        assert!(
            text.contains("WHile") || text.contains("while"),
            "codul de proba are cuvinte cheie scrise amestecat"
        );
    }
    assert!(app.are_cursor(), "obiectul are un ciclu de cursor");

    app.comanda(Comanda::AplicaToate);
    let (martor_dupa, final_dupa) = app.texte_curente().expect("texte dupa aplicare");
    assert_eq!(
        gmotion_core::norm(&martor_dupa),
        gmotion_core::norm(&final_dupa),
        "dupa Aplica toate, baza trebuie sa spuna acelasi lucru ca martorul"
    );

    // si cu ciclul de cursor unit, rezultatul e acelasi
    app.comanda(Comanda::Reset);
    app.comanda(Comanda::ComutaUnireaCursorului(true));
    app.comanda(Comanda::AplicaToate);
    let (martor_unit, final_unit) = app.texte_curente().expect("texte");
    assert_eq!(
        gmotion_core::norm(&martor_unit),
        gmotion_core::norm(&final_unit),
        "unirea ciclului de cursor schimba taierea blocurilor, nu rezultatul"
    );
}

/// C1 si C2 din `cercetare/audit-qa-2026-11-18.md`: planul de deploy deschis
/// o data se refolosea cat timp multimea „in plan" nu se schimba, deci
/// scrierea punea in script **textul final de la prima deschidere** (C1), si
/// supravietuia unei recomparari, cu garda ei veche (C2).
///
/// Testul merge pe acelasi drum ca `main.rs`: deschide deploy-ul, se intoarce
/// in editor, schimba textul, redeschide deploy-ul — fara sa atinga ce e in
/// plan.
#[test]
fn deploy_redeschis_dupa_o_editare_scrie_textul_de_acum() {
    use gmotion_app::deploy::Garda;
    let cale = instantaneu("deploy-text-vechi");
    let mut c = Conectare::noua();
    c.instantaneu = Some(cale.to_string_lossy().to_string());
    c.sursa.baza = "CharismaDev".into();
    c.baza.baza = "CharismaProd".into();
    let r = pornire::compara(&c).expect("compararea merge");
    let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());

    let pozitie = |app: &Aplicatie, nume: &str| {
        (0..app.lista().vizibile().len())
            .find(|&p| app.lista().obiect(p).unwrap().nume == nume)
            .unwrap()
    };

    // 1. pStoc in plan, apoi ecranul de deploy
    let p = pozitie(&app, "pStoc");
    app.comanda(Comanda::Selecteaza(p));
    app.comanda(Comanda::ComutaInPlan);
    app.comanda(Comanda::DeschideDeploy);
    let mut deschis = None;
    gmotion_app::deploy::asigura_planul(&mut deschis, &app, "CharismaDev", "2026-11-18 14:00");
    assert_eq!(deschis.as_ref().unwrap().stare.plan.pasi.len(), 1);

    // 2. inapoi in editor, textul schimbat de mana
    app.comanda(Comanda::InapoiLaLista);
    app.comanda(Comanda::Selecteaza(pozitie(&app, "pStoc")));
    app.comanda(Comanda::Deschide);
    assert!(app.comanda(Comanda::SalveazaText {
        id: app.id_obiect().expect("un obiect deschis"),
        text: "CREATE PROC dbo.pStoc AS\nSELECT cantitate FROM Stoc WHERE activ = 7".into()
    }));

    // 3. deploy din nou: aceeasi multime „in plan", deci aici se refolosea
    app.comanda(Comanda::DeschideDeploy);
    gmotion_app::deploy::asigura_planul(&mut deschis, &app, "CharismaDev", "2026-11-18 14:10");
    let st = &mut deschis.as_mut().unwrap().stare;
    assert!(
        matches!(st.garda, Garda::Neverificata),
        "planul refacut porneste fara garda: {:?}",
        st.garda
    );
    let recitit = pornire::compara(&c).expect("recitirea merge");
    st.aplica_verificarea(recitit.sursa, recitit.tinte, "14:10");
    let dir = cale.parent().unwrap().join("deploy");
    st.scrie(&dir, "2026-11-18 14:10", &app.texte_finale());
    let raport = st
        .rezultat
        .as_ref()
        .unwrap()
        .as_ref()
        .expect("scrierea merge");
    let script = std::fs::read_to_string(&raport.fisiere[0]).unwrap();
    assert!(
        script.contains("WHERE activ = 7"),
        "in script trebuie sa ajunga textul de acum:\n{script}"
    );
    assert!(
        !script.contains("WHERE activ = 1"),
        "nu textul de la prima deschidere a planului:\n{script}"
    );
    std::fs::remove_dir_all(cale.parent().unwrap()).ok();
}

/// C2: dupa o recomparare, `Aplicatie` e alta — planul si garda de dinainte
/// nu mai sunt ale ei, oricat de identica ar fi multimea „in plan".
#[test]
fn recompararea_pierde_planul_si_garda_de_dinainte() {
    use gmotion_app::deploy::Garda;
    let cale = instantaneu("deploy-recompara");
    let mut c = Conectare::noua();
    c.instantaneu = Some(cale.to_string_lossy().to_string());
    c.sursa.baza = "CharismaDev".into();
    c.baza.baza = "CharismaProd".into();
    let r = pornire::compara(&c).expect("compararea merge");
    let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());
    let p = (0..app.lista().vizibile().len())
        .find(|&p| app.lista().obiect(p).unwrap().nume == "pStoc")
        .unwrap();
    app.comanda(Comanda::Selecteaza(p));
    app.comanda(Comanda::ComutaInPlan);
    app.comanda(Comanda::DeschideDeploy);
    let mut deschis = None;
    gmotion_app::deploy::asigura_planul(&mut deschis, &app, "CharismaDev", "14:00");
    let recitit = pornire::compara(&c).expect("recitirea merge");
    let st = &mut deschis.as_mut().unwrap().stare;
    st.aplica_verificarea(recitit.sursa, recitit.tinte, "14:05");
    assert!(matches!(st.garda, Garda::Ok { .. }));

    // recompararea: alta `Aplicatie`, cu acelasi obiect in plan
    let r2 = pornire::compara(&c).expect("recompararea merge");
    let mut app2 = Aplicatie::noua(r2.randuri, r2.sursa, r2.tinte, modul::Revizuire::noua());
    let p = (0..app2.lista().vizibile().len())
        .find(|&p| app2.lista().obiect(p).unwrap().nume == "pStoc")
        .unwrap();
    app2.comanda(Comanda::Selecteaza(p));
    app2.comanda(Comanda::ComutaInPlan);
    app2.comanda(Comanda::DeschideDeploy);
    gmotion_app::deploy::asigura_planul(&mut deschis, &app2, "CharismaDev", "14:10");
    assert!(
        matches!(deschis.as_ref().unwrap().stare.garda, Garda::Neverificata),
        "garda de dinainte de recomparare nu mai e valabila: {:?}",
        deschis.as_ref().unwrap().stare.garda
    );
    std::fs::remove_dir_all(cale.parent().unwrap()).ok();
}

/// C5 din auditul QA, cap-coada: un sir pe mai multe linii, cu spatii la
/// capatul liniei si `\r\n` in interior, trebuie sa ajunga in scriptul de
/// deploy **octet cu octet**, si dupa „Aplica toate", si dupa „Aplica" pe un
/// bloc. Altfel GMotion schimba singur datele produse de procedura.
#[test]
fn un_sir_pe_mai_multe_linii_supravietuieste_aplicarii_si_deploy_ului() {
    use gmotion_app::deploy::StareDeploy;
    let d = std::env::temp_dir().join(format!("gmotion-sir-{}", std::process::id()));
    std::fs::remove_dir_all(&d).ok();
    std::fs::create_dir_all(&d).unwrap();
    let cale = d.join("inst.json");
    // definitia martorului: sirul are spatii la capat de linie si CRLF
    let def_martor = "CREATE PROC dbo.pRaport AS\r\nSET @s = 'linia 1   \r\nlinia 2'\r\nSELECT 1";
    let def_baza = "CREATE PROC dbo.pRaport AS\r\nSET @s = 'altceva'\r\nSELECT 0";
    // scris de mana: `\r` si `\n` ca secvente JSON, ca sa ajunga in definitie
    // exact octetii de mai sus
    let in_json = |t: &str| t.replace('\r', "\\r").replace('\n', "\\n");
    std::fs::write(
        &cale,
        format!(
            r#"{{"versiune":1,"server":"SRV","creat":"2026-11-18T10:00:00Z","nota":"","baze":{{
              "Dev":[{{"schema":"dbo","nume":"pRaport","tip":"P","definitie":"{}"}}],
              "Prod":[{{"schema":"dbo","nume":"pRaport","tip":"P","definitie":"{}"}}]}}}}"#,
            in_json(def_martor),
            in_json(def_baza)
        ),
    )
    .unwrap();

    let mut c = Conectare::noua();
    c.instantaneu = Some(cale.to_string_lossy().to_string());
    c.sursa.baza = "Dev".into();
    c.baza.baza = "Prod".into();
    let r = pornire::compara(&c).expect("compararea merge");
    let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());
    app.comanda(Comanda::Selecteaza(0));
    app.comanda(Comanda::Deschide);
    assert_eq!(app.ecran(), Ecran::Editor);

    // 1. editorul arata sirul neatins
    let (martor, _) = app.texte_curente().expect("texte");
    assert!(
        martor.contains("'linia 1   \r\nlinia 2'"),
        "editorul arata sirul neatins:\n{martor:?}"
    );

    // 2. Aplica toate, apoi deploy
    app.comanda(Comanda::AplicaToate);
    app.comanda(Comanda::ComutaInPlan);
    let mut st = StareDeploy::nou(app.plan("Dev", "14:00"));
    let recitit = pornire::compara(&c).expect("recitirea merge");
    st.aplica_verificarea(recitit.sursa, recitit.tinte, "14:05");
    st.scrie(&d.join("deploy"), "14:05", &app.texte_finale());
    let raport = st
        .rezultat
        .as_ref()
        .unwrap()
        .as_ref()
        .expect("scrierea merge");
    let script = std::fs::read_to_string(&raport.fisiere[0]).unwrap();
    assert!(
        script.contains("'linia 1   \r\nlinia 2'"),
        "sirul pleaca in baza octet cu octet:\n{script:?}"
    );
    std::fs::remove_dir_all(&d).ok();
}
