//! Cat tine harta din lista la 1.000 / 10.000 / 50.000 de obiecte care
//! difera (criteriul 15 D): asezarea pe coloane si pictarea patratelor, la
//! scalarea 1x si 2x a ecranului, plus treapta de zoom si inaltimea la care
//! ajunge (de la ea incolo harta se deruleaza).
//!
//!   cargo run --release -p gmotion-fereastra --example masura_harta [-- 4000 50000]
use std::time::Instant;

use gmotion_fereastra::harta::{self, Celula};

/// `n` obiecte care difera, pe patru tipuri, cu amestecul de stari al unei
/// comparari adevarate.
fn grupuri(n: usize) -> Vec<(String, Vec<Celula>)> {
    let tipuri = ["procedura", "functie", "view", "trigger"];
    let mut v: Vec<(String, Vec<Celula>)> =
        tipuri.iter().map(|t| (t.to_string(), Vec::new())).collect();
    for i in 0..n {
        let t = match i % 10 {
            0..=5 => 0,
            6 | 7 => 1,
            8 => 2,
            _ => 3,
        };
        v[t].1.push(Celula {
            rand: i,
            stare: (i % 7 % 3) as i32,
            revizuit: i % 5 == 0,
        });
    }
    v.sort_by_key(|(_, c)| std::cmp::Reverse(c.len()));
    v
}

fn main() {
    let arg: Vec<usize> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let dimensiuni = if arg.is_empty() {
        vec![1_000usize, 4_000, 10_000, 50_000]
    } else {
        arg
    };
    let paleta = gmotion_fereastra::Mod::default().paleta();
    let ms = |t: Instant| t.elapsed().as_secs_f64() * 1000.0;
    println!(
        "{:>8}  {:>10}  {:>11}  {:>11}  {:>7}  {:>10}",
        "obiecte", "aseaza ms", "picteaza 1x", "picteaza 2x", "latura", "inaltime"
    );
    for n in dimensiuni {
        let g = grupuri(n);
        let t = Instant::now();
        let (asezate, inaltime) = harta::aseaza(g, 1372.0, harta::INALTIME_MAXIMA);
        let aseaza_ms = ms(t);
        let mut pictat = [0.0f64; 2];
        for (k, factor) in [1.0f32, 2.0].into_iter().enumerate() {
            let t = Instant::now();
            for g in &asezate {
                std::hint::black_box(harta::picteaza_la(g, paleta, factor));
                std::hint::black_box(harta::picteaza_bara_la(g, paleta, factor));
            }
            pictat[k] = ms(t);
        }
        println!(
            "{:>8}  {:>10.1}  {:>11.1}  {:>11.1}  {:>7}  {:>10.0}",
            n,
            aseaza_ms,
            pictat[0],
            pictat[1],
            asezate.first().map_or(0, |g| g.scara.latura),
            inaltime
        );
    }
}
