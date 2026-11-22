//! Verificarea structurii pe cod adevarat: obiectele din instantaneul real
//! (`referinta/instantanee/testsimilitudine.json`, proceduri din productie)
//! sunt toate valide, deci nu au voie sa iasa probleme. Un fals pozitiv aici
//! ar fi un avertisment rosu pe fiecare obiect al utilizatorului.

#[test]
fn obiectele_reale_nu_au_probleme_de_structura() {
    let cale = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../referinta/instantanee/testsimilitudine.json"
    );
    let text = std::fs::read_to_string(cale).expect("instantaneul real");
    let v: serde_json::Value = serde_json::from_str(&text).expect("JSON");
    let mut verificate = 0;
    for (baza, obiecte) in v["baze"].as_object().expect("baze") {
        for o in obiecte.as_array().expect("obiecte") {
            let def = o["definitie"].as_str().unwrap_or("");
            let probleme = gmotion_core::structura::verifica(def);
            assert!(
                probleme.is_empty(),
                "{baza}/{}: {probleme:?}",
                o["nume"].as_str().unwrap_or("?")
            );
            verificate += 1;
        }
    }
    assert!(verificate >= 30, "{verificate} obiecte");
}
