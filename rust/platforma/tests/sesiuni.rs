//! Fluxul de sesiuni pe disc, cu accent pe bug-ul pe care platforma trebuie
//! sa-l faca imposibil: doua module care lucreaza pe aceeasi baza de referinta
//! nu au voie sa-si suprascrie progresul.
//!
//! In versiunea Python cheia era doar numele bazei (`_cale_sesiune(referinta)`
//! -> `<slug>.dat`), deci coliziunea era garantata de la al doilea modul.

use gmotion_platforma::{Sesiune, Sesiuni};

/// `GMOTION_DIR_DATE` e globala in proces, iar `cargo test` ruleaza testele in
/// paralel. Fara serializare, doua teste care o schimba se calca reciproc si
/// poarta cade aleator — un test instabil e mai rau decat niciun test. De-aia
/// fiecare test din acest fisier ia lacatul si il tine pana la final.
static LACAT: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Lacatul plus directorul propriu al testului. Se tine cat traieste valoarea
/// intoarsa.
fn pregateste(eticheta: &str) -> (std::sync::MutexGuard<'static, ()>, std::path::PathBuf) {
    // `unwrap_or_else` pe otravire: daca un test anterior a dat panica,
    // lacatul e otravit, dar directorul fiecarui test e separat, deci se poate
    // continua.
    let lacat = LACAT.lock().unwrap_or_else(|e| e.into_inner());
    (lacat, director(eticheta))
}

fn director(eticheta: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("gmotion-ses-{}-{}", eticheta, std::process::id()));
    std::fs::remove_dir_all(&d).ok();
    std::fs::create_dir_all(&d).unwrap();
    std::env::set_var("GMOTION_DIR_DATE", &d);
    d
}

fn sesiune(referinta: &str, nota: &str) -> Sesiune {
    let mut s = Sesiune::noua("SRV01", "SQL", referinta, "2026-11-17T10:00:00Z");
    s.utilizator = "sa".into();
    s.baze = vec!["Client1".into(), "Client2".into()];
    s.date
        .insert("nota".into(), serde_json::Value::String(nota.into()));
    s
}

#[test]
fn tot_fluxul_de_sesiuni() {
    // Un singur test pentru tot, fiindca GMOTION_DIR_DATE e global in proces:
    // doua teste care-l schimba in paralel s-ar calca reciproc. Mai bine un
    // test lung si corect decat patru care pica aleator.
    let (_lacat, dir) = pregateste("flux");

    let diff = Sesiuni::pentru_modul("diff-obiecte").unwrap();
    let upgrade = Sesiuni::pentru_modul("upgrade").unwrap();

    // --- bug-ul din versiunea Python: aceeasi referinta, doua module ---
    diff.salveaza(&sesiune("Referinta", "din diff")).unwrap();
    upgrade
        .salveaza(&sesiune("Referinta", "din upgrade"))
        .unwrap();

    let a = diff
        .incarca("SRV01", "Referinta")
        .unwrap()
        .expect("diff are sesiune");
    let b = upgrade
        .incarca("SRV01", "Referinta")
        .unwrap()
        .expect("upgrade are sesiune");
    assert_eq!(a.date["nota"], "din diff", "upgrade a suprascris diff");
    assert_eq!(b.date["nota"], "din upgrade");
    assert_ne!(diff.director(), upgrade.director());

    // --- dus-intors complet ---
    assert_eq!(a.server, "SRV01");
    assert_eq!(a.utilizator, "sa");
    assert_eq!(a.baze, vec!["Client1", "Client2"]);

    // --- sesiune inexistenta nu e eroare ---
    assert!(diff.incarca("SRV01", "BazaCareNuExista").unwrap().is_none());

    // --- listare, sortata, doar sesiunile modulului ---
    diff.salveaza(&sesiune("Alta", "a doua")).unwrap();
    let lista = diff.lista().unwrap();
    assert_eq!(lista.len(), 2, "doar sesiunile lui diff");
    assert_eq!(lista[0].referinta, "Alta", "sortate dupa referinta");
    assert_eq!(lista[1].referinta, "Referinta");
    assert_eq!(upgrade.lista().unwrap().len(), 1);

    // --- un fisier stricat nu ascunde celelalte sesiuni ---
    std::fs::write(diff.director().join("stricat.dat"), b"gunoi").unwrap();
    assert_eq!(
        diff.lista().unwrap().len(),
        2,
        "fisierul stricat trebuie sarit, nu sa faca listarea sa cada"
    );

    // --- stergere ---
    assert!(diff.sterge("SRV01", "Alta").unwrap());
    assert!(
        !diff.sterge("SRV01", "Alta").unwrap(),
        "a doua stergere nu e eroare"
    );
    assert_eq!(diff.lista().unwrap().len(), 1);

    // --- parola nu ajunge pe disc, in nicio forma ---
    let brut = std::fs::read(diff.director().join(format!(
        "{}.dat",
        gmotion_platforma::slug_sesiune("SRV01", "Referinta")
    )))
    .unwrap();
    let text = String::from_utf8_lossy(&brut);
    for cuvant in ["parola", "password", "secret"] {
        assert!(
            !text.contains(cuvant),
            "{cuvant} apare in fisierul de sesiune"
        );
    }

    std::env::remove_var("GMOTION_DIR_DATE");
    std::fs::remove_dir_all(&dir).ok();
}

/// Numele de baze cu caractere care nu pot aparea intr-un nume de fisier.
#[test]
fn nume_de_baze_dificile() {
    let (_lacat, dir) = pregateste("nume");
    let s = Sesiuni::pentru_modul("diff-obiecte").unwrap();

    for referinta in [
        "Baza Cu Spatii",
        "baza/cu/slash",
        "baza\\cu\\backslash",
        "BAZĂ-cu-diacritice", // diacritice-permise: date de test
        "..",
        "a:b*c?d",
        "Baza.2026",
    ] {
        let mut ses = Sesiune::noua("SRV", "SQL", referinta, "2026-11-17T00:00:00Z");
        ses.date.insert(
            "marcaj".into(),
            serde_json::Value::String(referinta.to_string()),
        );
        let cale = s.salveaza(&ses).expect(referinta);
        assert!(
            cale.starts_with(s.director()),
            "{referinta:?} a scris in afara directorului: {cale:?}"
        );
        let citit = s.incarca("SRV", referinta).unwrap().expect(referinta);
        assert_eq!(citit.date["marcaj"], referinta);
    }

    std::env::remove_var("GMOTION_DIR_DATE");
    std::fs::remove_dir_all(&dir).ok();
}

/// F-B4 din `cercetare/audit-qa-2026-11-18.md`: cheia sesiunii era doar
/// referinta, deci `PROD\Ref` si `TEST\Ref` isi suprascriau progresul, iar
/// `incarca` intorcea sesiunea celuilalt server. Si: o sesiune scrisa sub
/// numele vechi se muta singura pe numele nou, ca progresul de pana acum sa
/// nu para pierdut.
#[test]
fn doua_servere_cu_aceeasi_referinta_au_sesiuni_separate() {
    let (_lacat, dir) = pregateste("doua-servere");
    let s = Sesiuni::pentru_modul("diff-obiecte").unwrap();

    let mut prod = sesiune("Ref", "de pe PROD");
    prod.server = "SRV-PROD".into();
    s.salveaza(&prod).unwrap();
    let mut test = sesiune("Ref", "de pe TEST");
    test.server = "SRV-TEST".into();
    s.salveaza(&test).unwrap();

    assert_eq!(
        s.incarca("SRV-PROD", "Ref").unwrap().unwrap().date["nota"],
        "de pe PROD",
        "sesiunea lui PROD a fost suprascrisa de a lui TEST"
    );
    assert_eq!(
        s.incarca("SRV-TEST", "Ref").unwrap().unwrap().date["nota"],
        "de pe TEST"
    );
    // un server pe care nu s-a lucrat n-are sesiune, nu o imprumuta
    assert!(s.incarca("SRV-ALTUL", "Ref").unwrap().is_none());
    // stergerea e tot pe pereche
    assert!(s.sterge("SRV-TEST", "Ref").unwrap());
    assert!(s.incarca("SRV-PROD", "Ref").unwrap().is_some());

    // --- migrarea unei sesiuni scrise sub numele vechi (doar pe referinta)
    let vechi = s
        .director()
        .join(format!("{}.dat", gmotion_platforma::slug("Veche")));
    let nou = s.director().join(format!(
        "{}.dat",
        gmotion_platforma::slug_sesiune("SRV-PROD", "Veche")
    ));
    let mut veche = sesiune("Veche", "scrisa inainte");
    veche.server = "SRV-PROD".into();
    std::fs::rename(s.salveaza(&veche).unwrap(), &vechi).unwrap();
    assert!(!nou.exists());
    assert_eq!(
        s.incarca("SRV-PROD", "Veche").unwrap().unwrap().date["nota"],
        "scrisa inainte",
        "sesiunea veche trebuie gasita si mutata, nu pierduta"
    );
    assert!(nou.exists() && !vechi.exists(), "mutata, nu copiata");

    std::env::remove_var("GMOTION_DIR_DATE");
    std::fs::remove_dir_all(&dir).ok();
}

/// F-A15: o sesiune ilizibila se pune deoparte, nu se suprascrie; dupa asta
/// `incarca` vede „nicio sesiune", iar listarea n-o ia drept sesiune.
#[test]
fn sesiunea_ilizibila_se_pune_deoparte() {
    let (_lacat, dir) = pregateste("deoparte");
    let s = Sesiuni::pentru_modul("diff-obiecte").unwrap();
    assert_eq!(s.pune_deoparte("SRV01", "Ref").unwrap(), None);
    let cale = s.salveaza(&sesiune("Ref", "munca")).unwrap();
    let continut = std::fs::read(&cale).unwrap();
    std::fs::write(&cale, &continut[..continut.len() / 2]).unwrap();
    assert!(s.incarca("SRV01", "Ref").is_err());

    let pusa = s.pune_deoparte("SRV01", "Ref").unwrap().expect("exista");
    assert!(!cale.exists());
    assert_eq!(
        std::fs::read(&pusa).unwrap(),
        &continut[..continut.len() / 2]
    );
    assert_eq!(s.incarca("SRV01", "Ref").unwrap(), None);
    assert!(s.lista().unwrap().is_empty());
    std::fs::remove_dir_all(&dir).ok();
}

/// Acelasi server scris altfel (`SRV`, `srv`, `SRV,1433`) e aceeasi sesiune;
/// una scrisa inainte de normalizare se muta singura pe cheia noua.
#[test]
fn serverul_scris_altfel_e_aceeasi_sesiune() {
    let (_lacat, dir) = pregateste("server-normalizat");
    let s = Sesiuni::pentru_modul("diff-obiecte").unwrap();
    s.salveaza(&sesiune("Ref", "munca")).unwrap(); // server "SRV01"
    for forma in ["SRV01", "srv01", "SRV01,1433", "SRV01:1433", " srv01 "] {
        let g = s.incarca(forma, "Ref").unwrap();
        assert!(g.is_some(), "{forma:?}");
    }
    assert!(
        s.incarca("SRV01,1500", "Ref").unwrap().is_none(),
        "alt port, alt server"
    );
    assert_eq!(gmotion_platforma::server_normalizat("SRV,1433"), "srv");
    std::fs::remove_dir_all(&dir).ok();
}
