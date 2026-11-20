//! Cat costa scorul pe obiecte mari, cu si fara automat de sufixe.
//!
//!     cargo run --release --example masura_scor
use std::time::Instant;

fn obiect(n: usize, modifica: bool) -> String {
    let mut s = String::from("CREATE PROCEDURE p AS\nBEGIN\n");
    for i in 0..n {
        let v = if modifica && i % 37 == 0 { i + 1 } else { i };
        s.push_str(&format!(
            "SELECT col_{i}, alt_{i} FROM dbo.Tabela_{i} WITH (NOLOCK)\n  WHERE id = {v} AND activ = 1\n  ORDER BY col_{i}\n\n"
        ));
    }
    s.push_str("END\n");
    s
}

fn main() {
    for n in [500usize, 1000, 2000, 4000] {
        let a = gmotion_core::pregateste_afisare(&obiect(n, false));
        let b = gmotion_core::pregateste_afisare(&obiect(n, true));
        let t = Instant::now();
        let scor = gmotion_core::scor_similaritate(&a, &b);
        println!(
            "{:>6} linii  scor {:>6.2}  in {:>8.1} ms",
            a.lines().count(),
            scor,
            t.elapsed().as_secs_f64() * 1000.0
        );
    }
}
