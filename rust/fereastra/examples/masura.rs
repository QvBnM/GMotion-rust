//! Cat tine editorul pe un obiect uriase: criteriul „un obiect de 20.000 de
//! linii nu blocheaza" din etapa 8.
//!
//! Masoara separat ce creste cu obiectul (modelul, lista de randuri) si ce nu
//! trebuie sa creasca (desenarea unui cadru — `ListView` deseneaza doar
//! randurile vizibile). Daca timpul pe cadru creste cu numarul de randuri,
//! virtualizarea s-a rupt, si asta e tot rostul masuratorii.
//!
//!   cargo run --release -p gmotion-fereastra --example masura [-- linii...]
use std::rc::Rc;
use std::time::Instant;

use gmotion_fereastra::{
    aplica_tema, incarca_editor, model_editor, AntetEditor, EditorDiferente, Mediu,
};
use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType, TargetPixel,
};
use slint::{ComponentHandle, Model};

struct Platforma(Rc<MinimalSoftwareWindow>);
impl slint::platform::Platform for Platforma {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}

/// Doua versiuni ale unui obiect de `linii` linii, cu o diferenta la fiecare
/// 500 de linii (destul ca plierea sa aiba ce pliat si navigarea ce navigat).
fn obiect(linii: usize) -> (String, String) {
    let mut a = String::from("CREATE OR ALTER PROCEDURE dbo.usp_Uriasa\nAS\nBEGIN\n");
    let mut b = a.clone();
    for i in 0..linii {
        let bloc =
            format!("    -- pasul {i}\n    UPDATE dbo.Linii SET Valoare = {i} WHERE Id = {i};\n\n");
        a.push_str(&bloc);
        if i % 500 == 0 {
            b.push_str(&bloc.replace(&format!("Valoare = {i}"), &format!("Valoare = {i} + 1")));
        } else {
            b.push_str(&bloc);
        }
    }
    a.push_str("END\n");
    b.push_str("END\n");
    (a, b)
}

fn main() {
    let arg: Vec<usize> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let dimensiuni = if arg.is_empty() {
        vec![200usize, 2_000, 20_000]
    } else {
        arg
    };

    let (w, h) = (1440u32, 900u32);
    let fereastra = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(fereastra.clone()))).expect("platforma");
    let ui = EditorDiferente::new().expect("fereastra nu se creeaza");
    aplica_tema(&ui, gmotion_fereastra::Mod::default());
    ui.show().expect("fereastra nu se arata");
    fereastra.set_size(slint::PhysicalSize::new(w, h));
    let mut buf = vec![PremultipliedRgbaColor::background(); (w * h) as usize];
    let mut deseneaza = || {
        let t = Instant::now();
        fereastra.request_redraw();
        fereastra.draw_if_needed(|rd| {
            rd.render(&mut buf, w as usize);
        });
        t.elapsed().as_secs_f64() * 1000.0
    };

    println!(
        "{:>8}  {:>9}  {:>7}  {:>9}  {:>10}  {:>9}  {:>14}",
        "randuri", "model ms", "pliat", "afisate", "incarca ms", "cadru1 ms", "cadru-mijloc ms"
    );
    for n in dimensiuni {
        for pliat in [true, false] {
            let (a, b) = obiect(n / 2);
            ui.set_pliaza_identicul(pliat);
            let t = Instant::now();
            let model = model_editor::construieste(&a, &b);
            let model_ms = t.elapsed().as_secs_f64() * 1000.0;

            let t = Instant::now();
            incarca_editor(
                &ui,
                &AntetEditor {
                    schema: "dbo",
                    nume: "usp_Uriasa",
                    tip: "procedura",
                    sursa_baza: "CharismaDev",
                    sursa_server: "srv-dev",
                    tinta_baza: "CharismaProd",
                    tinta_server: "srv-prod",
                    mediu: Mediu::Prod,
                    stare_obiecte: "obiectul 1 din 1 nerevizuite",
                    revizuit: false,
                    in_plan: false,
                },
                &model,
            );
            let incarca_ms = t.elapsed().as_secs_f64() * 1000.0;
            let cadru1 = deseneaza();
            // derulare la o diferenta de la mijloc: acolo se vede daca ListView
            // deseneaza doar randurile vizibile sau tot obiectul
            ui.invoke_mergi_la((model.diferente.len() / 2) as i32);
            let cadru_mijloc = deseneaza();
            println!(
                "{:>8}  {:>9.1}  {:>7}  {:>9}  {:>10.1}  {:>9.1}  {:>14.1}",
                model.randuri.len(),
                model_ms,
                if pliat { "da" } else { "nu" },
                ui.get_randuri().iter().count(),
                incarca_ms,
                cadru1,
                cadru_mijloc
            );
        }
    }
}
