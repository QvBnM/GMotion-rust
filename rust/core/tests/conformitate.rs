//! Conformitate cu motorul JS din `ui.py`.
//!
//! Fixture-ul `referinta/rezultate_js.json` e produs de motorul JS extras
//! mecanic din `ui.py`. Portul Rust trebuie sa dea exact aceleasi rezultate -
//! altfel migrarea ar schimba in liniste ce vede utilizatorul.
//!
//! De regenerat fixture-ul cand se schimba `ui.py`:
//!     node referinta/genereaza.mjs

use std::collections::BTreeMap;
use std::path::PathBuf;

use gmotion_core::{
    diff_aliniat, normalizeaza, pregateste_afisare, rezumat_diff, rezumat_rapid, scor_similaritate,
    split_statements,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Caz {
    nume: String,
    #[serde(rename = "ref")]
    referinta: String,
    cur: String,
}

#[derive(Debug, Deserialize)]
struct Asteptat {
    nume: String,
    norm_ref: String,
    norm_cur: String,
    scor: f64,
    blocuri_ref: Vec<String>,
    blocuri_cur: Vec<String>,
    rezumat_rapid: String,
    rezumat_diff: String,
    tipuri: Vec<String>,
}

fn dir_referinta() -> PathBuf {
    // CARGO_MANIFEST_DIR = <radacina>/rust/core, iar harness-ul sta in
    // <radacina>/referinta - deci doua niveluri in sus, nu unul.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("rust/core are mereu doi stramosi")
        .join("referinta")
}

fn citeste<T: serde::de::DeserializeOwned>(fisier: &str) -> Vec<T> {
    let cale = dir_referinta().join(fisier);
    let text = std::fs::read_to_string(&cale)
        .unwrap_or_else(|e| panic!("nu pot citi {}: {e}", cale.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("JSON invalid in {}: {e}", cale.display()))
}

/// Un singur test care acopera toate cazurile si raporteaza TOATE nepotrivirile
/// odata. Un test per caz ar fi mai granular, dar la o regresie de segmentare
/// ar esua zeci de teste simultan si raportul ar fi ilizibil.
/// Locurile in care portul se abate **deliberat** de la motorul JS (si de la
/// v3.4): (caz, camp, motiv). Fixture-ul JS nu se regenereaza pentru ele — JS
/// are bug-ul. Fiecare intrare trebuie sa si diverga efectiv: o intrare care nu
/// mai diverge e o eroare, ca lista sa nu ramana in urma codului.
const DIVERGENTE_DELIBERATE: &[(&str, &str, &str)] = &[
    (
        "string_cu_cuvinte_cheie",
        "norm_ref",
        "continutul string-urilor nu se mai trece in litere mici",
    ),
    (
        "string_cu_cuvinte_cheie",
        "norm_cur",
        "continutul string-urilor nu se mai trece in litere mici",
    ),
    (
        "diacritice_si_unicode",
        "norm_ref",
        "continutul string-urilor nu se mai trece in litere mici",
    ),
    (
        "diacritice_si_unicode",
        "norm_cur",
        "continutul string-urilor nu se mai trece in litere mici",
    ),
    (
        "mihai_offline_while_begin",
        "norm_cur",
        "continutul string-urilor nu se mai trece in litere mici",
    ),
];

#[test]
fn portul_rust_e_identic_cu_motorul_js() {
    let cazuri: Vec<Caz> = citeste("cazuri.json");
    let asteptate: Vec<Asteptat> = citeste("rezultate_js.json");

    let dupa_nume: BTreeMap<&str, &Asteptat> =
        asteptate.iter().map(|a| (a.nume.as_str(), a)).collect();

    assert_eq!(
        cazuri.len(),
        asteptate.len(),
        "cazuri.json si rezultate_js.json au lungimi diferite - regenereaza fixture-ul"
    );

    let mut probleme: Vec<String> = Vec::new();

    for caz in &cazuri {
        let a = dupa_nume
            .get(caz.nume.as_str())
            .unwrap_or_else(|| panic!("cazul {:?} lipseste din fixture", caz.nume));

        let referinta = pregateste_afisare(&caz.referinta);
        let curent = pregateste_afisare(&caz.cur);
        let randuri = diff_aliniat(&referinta, &curent);

        let mut verifica = |camp: &str, obtinut: String, asteptat: String| {
            let deliberat = DIVERGENTE_DELIBERATE
                .iter()
                .any(|(c, f, _)| *c == caz.nume && *f == camp);
            if deliberat && obtinut == asteptat {
                probleme.push(format!(
                    "  {} / {}: listat ca divergenta deliberata, dar nu mai diverge — scoate-l din lista",
                    caz.nume, camp
                ));
            } else if !deliberat && obtinut != asteptat {
                probleme.push(format!(
                    "  {} / {}\n      rust: {}\n      js  : {}",
                    caz.nume, camp, obtinut, asteptat
                ));
            }
        };

        verifica("norm_ref", normalizeaza(&caz.referinta), a.norm_ref.clone());
        verifica("norm_cur", normalizeaza(&caz.cur), a.norm_cur.clone());
        verifica(
            "scor",
            format!("{:?}", scor_similaritate(&referinta, &curent)),
            format!("{:?}", a.scor),
        );
        verifica(
            "blocuri_ref",
            format!("{:?}", split_statements(&referinta, false)),
            format!("{:?}", a.blocuri_ref),
        );
        verifica(
            "blocuri_cur",
            format!("{:?}", split_statements(&curent, false)),
            format!("{:?}", a.blocuri_cur),
        );
        verifica(
            "rezumat_rapid",
            rezumat_rapid(&referinta, &curent),
            a.rezumat_rapid.clone(),
        );
        verifica(
            "rezumat_diff",
            rezumat_diff(&randuri),
            a.rezumat_diff.clone(),
        );
        verifica(
            "tipuri",
            format!(
                "{:?}",
                randuri.iter().map(|r| r.tip.eticheta()).collect::<Vec<_>>()
            ),
            format!("{:?}", a.tipuri),
        );
    }

    assert!(
        probleme.is_empty(),
        "{} nepotriviri fata de motorul JS, pe {} cazuri:\n{}",
        probleme.len(),
        cazuri.len(),
        probleme.join("\n")
    );
}
