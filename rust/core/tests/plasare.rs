//! Paritate cu stratul de plasare din v3.4.
//!
//! Fixture-ul `referinta/plasare_js.json` e scos din pagina v3.4 adevarata,
//! rulata intr-un browser fara cap pe aceleasi cazuri
//! (`unelte/regenereaza-fixture-plasare.sh`). Daca testul cade, presupunerea
//! implicita e ca portul Rust e gresit — nu se regenereaza fixture-ul ca sa
//! treaca.

use std::path::PathBuf;

use gmotion_core::plasare::{sugereaza, Tinta};
use gmotion_core::{diff_aliniat, pregateste_afisare, split_statements};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Caz {
    nume: String,
    #[serde(rename = "ref")]
    referinta: String,
    cur: String,
}

#[derive(Debug, Deserialize)]
struct SugestieJs {
    pozitie: usize,
    tip: String,
    idx: i64,
    scor: f64,
    randuri: Vec<usize>,
    text: String,
}

#[derive(Debug, Deserialize)]
struct CazJs {
    nume: String,
    sugestii: Vec<SugestieJs>,
}

fn citeste<T: serde::de::DeserializeOwned>(fisier: &str) -> T {
    let cale = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../referinta")
        .join(fisier);
    let text = std::fs::read_to_string(&cale)
        .unwrap_or_else(|e| panic!("nu pot citi {}: {e}", cale.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("{} nu e JSON valid: {e}", cale.display()))
}

/// Cazurile in care segmentarea v4 diverge deliberat de v3.4 (`BEGIN` si
/// `END` au blocuri proprii, JURNAL 2026-11-21; aceleasi cazuri ca in
/// `DIVERGENTE_DELIBERATE` din `conformitate.rs`). Plasarea lucreaza pe
/// blocuri, deci pe alte blocuri nu mai are cu ce sa se compare; pentru ele se
/// cere doar sa diverga in continuare, ca lista sa nu ramana veche.
const SEGMENTARE_DIFERITA: &[&str] = &[
    "header_simplu",
    "declare_in_header",
    "if_begin_end_compus",
    "cursor_complet",
    "end_singur",
];

#[test]
fn plasarea_e_identica_cu_v34() {
    let cazuri: Vec<Caz> = citeste("cazuri.json");
    let asteptate: Vec<CazJs> = citeste("plasare_js.json");
    assert_eq!(
        cazuri.len(),
        asteptate.len(),
        "cazuri.json si plasare_js.json au lungimi diferite — regenereaza fixture-ul"
    );

    let mut probleme: Vec<String> = Vec::new();
    for (caz, a) in cazuri.iter().zip(&asteptate) {
        assert_eq!(caz.nume, a.nume, "fixture-ul e in alta ordine");
        let exceptat = SEGMENTARE_DIFERITA.contains(&caz.nume.as_str());
        let inainte = probleme.len();
        let referinta = pregateste_afisare(&caz.referinta);
        let curent = pregateste_afisare(&caz.cur);
        let randuri = diff_aliniat(&referinta, &curent);
        let blocuri = split_statements(&curent, false);

        for s in &a.sugestii {
            let Some(got) = sugereaza(&randuri, s.pozitie, &blocuri) else {
                probleme.push(format!(
                    "  {} / randul {}: Rust nu sugereaza nimic, v3.4 da {} {}",
                    caz.nume, s.pozitie, s.tip, s.idx
                ));
                continue;
            };
            let (tip, idx, scor) = match got.tinta {
                Tinta::Bloc { indice, scor } => ("blk", indice as i64, scor),
                Tinta::Gol { indice, scor } => ("gap", indice as i64, scor),
            };
            if tip != s.tip || idx != s.idx {
                probleme.push(format!(
                    "  {} / randul {}: rust {} {}, js {} {}",
                    caz.nume, s.pozitie, tip, idx, s.tip, s.idx
                ));
                continue;
            }
            if (scor - s.scor).abs() > 0.005 {
                probleme.push(format!(
                    "  {} / randul {} [scor]: rust {scor}, js {}",
                    caz.nume, s.pozitie, s.scor
                ));
            }
            // v3.4 raporteaza `RandDiff::id`, noi pozitii: le aducem la
            // acelasi numitor inainte de comparatie
            let asteptate_poz: Vec<usize> = s
                .randuri
                .iter()
                .filter_map(|id| randuri.iter().position(|r| r.id == *id as i64))
                .collect();
            if got.randuri != asteptate_poz {
                probleme.push(format!(
                    "  {} / randul {} [grup]: rust {:?}, js {:?}",
                    caz.nume, s.pozitie, got.randuri, asteptate_poz
                ));
            }
            if got.text != s.text {
                probleme.push(format!(
                    "  {} / randul {} [text]: rust {:?}, js {:?}",
                    caz.nume, s.pozitie, got.text, s.text
                ));
            }
        }
        if exceptat {
            if probleme.len() == inainte {
                probleme.push(format!(
                    "  {}: listat in SEGMENTARE_DIFERITA, dar nu mai diverge — scoate-l din lista",
                    caz.nume
                ));
            } else {
                probleme.truncate(inainte);
            }
        }
    }

    assert!(
        probleme.is_empty(),
        "{} nepotriviri de plasare fata de v3.4:\n{}",
        probleme.len(),
        probleme.join("\n")
    );
}
