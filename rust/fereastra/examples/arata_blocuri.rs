//! Randurile editorului pentru doua fisiere, ca text: ce e in stanga, ce e in
//! dreapta, eticheta si blocul. Pentru depanarea alinierii blocurilor.
//!
//!   cargo run -p gmotion-fereastra --example arata_blocuri -- before.sql after.sql
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let x = std::fs::read_to_string(&a[1]).expect("before");
    let y = std::fs::read_to_string(&a[2]).expect("after");
    let m = gmotion_fereastra::model_editor::construieste(&x, &y);
    for (i, r) in m.randuri.iter().enumerate() {
        let s: String = r.stanga.text().chars().take(38).collect();
        let d: String = r.dreapta.text().chars().take(38).collect();
        println!(
            "{i:3} {:>3} {s:<38} | {:>3} {d:<38} {:?} bloc={:?} fant={} ins={:?}",
            r.stanga.nr.map_or(String::new(), |n| n.to_string()),
            r.dreapta.nr.map_or(String::new(), |n| n.to_string()),
            r.stare,
            r.bloc,
            r.fantoma,
            r.inserare
        );
    }
    println!("diferente {:?} tinte {:?}", m.diferente, m.tinte);
}
