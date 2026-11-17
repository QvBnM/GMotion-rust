//! Conformitate cu motorul JS din `ui.py`.
//!
//! Fixture-ul `rust/referinta/rezultate_js.json` e produs de motorul JS extras
//! mecanic din `ui.py`. Portul Rust trebuie sa dea exact aceleasi rezultate -
//! altfel migrarea ar schimba in liniste ce vede utilizatorul.
//!
//! De regenerat fixture-ul cand se schimba `ui.py`:
//!     node rust/referinta/genereaza.mjs

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
    // CARGO_MANIFEST_DIR = rust/core
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust/core are mereu un parinte")
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
/// ar eșua zeci de teste simultan si raportul ar fi ilizibil.
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
            if obtinut != asteptat {
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
