//! Fluxul de sesiuni pe disc, cu accent pe bug-ul pe care platforma trebuie
//! sa-l facă imposibil: doua module care lucreaza pe aceeasi baza de referinta
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

/// Lacatul plus directorul propriu al testului. Se tine cat trăiește valoarea
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
    // test lung si corect decat patru care pică aleator.
    let (_lacat, dir) = pregateste("flux");

    let diff = Sesiuni::pentru_modul("diff-obiecte").unwrap();
    let upgrade = Sesiuni::pentru_modul("upgrade").unwrap();

    // --- bug-ul din versiunea Python: aceeasi referinta, doua module ---
    diff.salveaza(&sesiune("Referinta", "din diff")).unwrap();
    upgrade
        .salveaza(&sesiune("Referinta", "din upgrade"))
        .unwrap();

    let a = diff
        .incarca("Referinta")
        .unwrap()
        .expect("diff are sesiune");
    let b = upgrade
        .incarca("Referinta")
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
    assert!(diff.incarca("BazaCareNuExista").unwrap().is_none());

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
        "fisierul stricat trebuie sarit, nu sa facă listarea sa cada"
    );

    // --- stergere ---
    assert!(diff.sterge("Alta").unwrap());
    assert!(!diff.sterge("Alta").unwrap(), "a doua stergere nu e eroare");
    assert_eq!(diff.lista().unwrap().len(), 1);

    // --- parola nu ajunge pe disc, in nicio forma ---
    let brut = std::fs::read(
        diff.director()
            .join(format!("{}.dat", gmotion_platforma::slug("Referinta"))),
    )
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
        "BAZĂ-cu-diacritice",
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
        let citit = s.incarca(referinta).unwrap().expect(referinta);
        assert_eq!(citit.date["marcaj"], referinta);
    }

    std::env::remove_var("GMOTION_DIR_DATE");
    std::fs::remove_dir_all(&dir).ok();
}
