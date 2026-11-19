//! Masoara motorul Rust pe acelasi corpus ca echivalentul Python.
//!
//!     cargo run --release --example bench
//!     python3 -I referinta/bench.py
use std::time::Instant;

fn main() {
    let cale = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../referinta/corpus_bench.json"
    );
    let text = std::fs::read_to_string(cale).expect("corpus_bench.json lipseste");
    let perechi: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();

    let date: Vec<(String, String)> = perechi
        .iter()
        .map(|p| {
            (
                gmotion_core::pregateste_afisare(p["ref"].as_str().unwrap()),
                gmotion_core::pregateste_afisare(p["cur"].as_str().unwrap()),
            )
        })
        .collect();

    let t = Instant::now();
    for (r, c) in &date {
        let _ = gmotion_core::scor_similaritate(r, c);
        let _ = gmotion_core::rezumat_rapid(r, c);
    }
    println!(
        "rust   scor+rezumat : {:>8.0} ms pentru {} perechi",
        t.elapsed().as_secs_f64() * 1000.0,
        date.len()
    );

    let t = Instant::now();
    let randuri: usize = date
        .iter()
        .map(|(r, c)| gmotion_core::diff_aliniat(r, c).len())
        .sum();
    println!(
        "rust   diff_aliniat : {:>8.0} ms ({} randuri)",
        t.elapsed().as_secs_f64() * 1000.0,
        randuri
    );

    // Obiecte mari, construite aici: corpusul n-are asa ceva, iar P2 din
    // auditul QA (`linii_start_blocuri` patratic) se vede doar pe ele.
    for n in [5_000usize, 20_000] {
        let cod: String = (0..n)
            .map(|i| format!("SELECT {i} FROM t{i};\n\n"))
            .collect();
        let blocuri = gmotion_core::split_statements(&cod, false);
        let t = Instant::now();
        let _ = gmotion_core::linii_start_blocuri(&cod, &blocuri);
        println!(
            "rust   linii_start   : {:>8.1} ms ({} blocuri)",
            t.elapsed().as_secs_f64() * 1000.0,
            blocuri.len()
        );
        let alt = format!("{cod}SELECT 1\n");
        let t = Instant::now();
        let _ = gmotion_core::aliniaza(&cod, &alt);
        println!(
            "rust   aliniaza      : {:>8.1} ms ({n} instructiuni)",
            t.elapsed().as_secs_f64() * 1000.0
        );
    }
}
