//! Cat tine lista pe o comparare uriasa: criteriul „masurata pe 50.000 de
//! obiecte" din etapa 9.
//!
//! Masoara separat ce creste cu numarul de obiecte (modelul, filtrarea,
//! umplerea listei) si ce nu trebuie sa creasca (desenarea unui cadru —
//! `ListView` deseneaza doar randurile vizibile). Daca timpul pe cadru creste
//! cu numarul de randuri, virtualizarea s-a rupt, si asta e tot rostul
//! masuratorii.
//!
//!   cargo run --release -p gmotion-fereastra --example masura_lista [-- 1000 50000]
use std::rc::Rc;
use std::time::Instant;

use gmotion_fereastra::{
    aplica_tema, incarca_lista, model_lista, AntetLista, FereastraLista, Mediu,
};
use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType, TargetPixel,
};
use slint::ComponentHandle;

struct Platforma(Rc<MinimalSoftwareWindow>);
impl slint::platform::Platform for Platforma {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}

/// `n` obiecte, cu amestecul de stari si tipuri al unei comparari adevarate:
/// cele mai multe identice, cateva mii modificate.
fn obiecte(n: usize) -> Vec<model_lista::Obiect> {
    use model_lista::Stare;
    (0..n)
        .map(|i| {
            let stare = match i % 10 {
                0 | 1 => Stare::Modificat,
                2 => Stare::DoarSursa,
                3 => Stare::DoarBaza,
                4 if i % 1000 == 4 => Stare::Criptat,
                _ => Stare::Identic,
            };
            model_lista::Obiect {
                baza: "Prod".into(),
                schema: if i % 7 == 0 { "rpt" } else { "dbo" }.into(),
                nume: format!("usp_Obiect_{i:06}"),
                tip: ["P", "P", "P", "V", "FN", "TR"][i % 6].into(),
                stare,
                detalii: (stare == Stare::Modificat && i % 100 == 0).then(|| {
                    model_lista::Detalii {
                        difera_in: "cod, siruri".into(),
                        linii_plus: i % 97,
                        linii_minus: i % 31,
                    }
                }),
                revizuit: i % 13 == 0,
                in_plan: false,
                omis: false,
                scor: None,
                deploy: Default::default(),
            }
        })
        .collect()
}

fn main() {
    let arg: Vec<usize> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let dimensiuni = if arg.is_empty() {
        vec![1_000usize, 10_000, 50_000]
    } else {
        arg
    };

    let (w, h) = (1440u32, 900u32);
    let fereastra = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(fereastra.clone()))).expect("platforma");
    let ui = FereastraLista::new().expect("fereastra nu se creeaza");
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
    let ms = |t: Instant| t.elapsed().as_secs_f64() * 1000.0;

    println!(
        "{:>8}  {:>9}  {:>8}  {:>9}  {:>10}  {:>9}  {:>13}",
        "obiecte", "model ms", "filtru ms", "cautare ms", "incarca ms", "cadru1 ms", "cadru-jos ms"
    );
    for n in dimensiuni {
        let t = Instant::now();
        let mut lista = model_lista::Lista::noua(obiecte(n));
        let model_ms = ms(t);

        let t = Instant::now();
        lista.filtreaza_stare(Some(model_lista::Stare::Modificat));
        let filtru_ms = ms(t);

        let t = Instant::now();
        lista.cauta("usp_Obiect_01");
        let cautare_ms = ms(t);
        lista.cauta("");
        // masuram cazul greu: fara filtru, toate obiectele in lista
        lista.filtreaza_stare(None);

        let antet = AntetLista {
            sursa_baza: "CharismaDev",
            tinta_baza: "CharismaProd",
            servere: "srv-dev → srv-prod · comparat la 14:32",
            mediu: Mediu::Prod,
            in_plan: 12,
        };
        let t = Instant::now();
        incarca_lista(&ui, &antet, &lista);
        let incarca_ms = ms(t);

        let cadru1 = deseneaza();
        // derulare la capatul listei: acolo se vede daca ListView deseneaza
        // doar randurile vizibile sau toata lista
        ui.invoke_muta_selectia(lista.vizibile().len() as i32);
        let cadru_jos = deseneaza();

        println!(
            "{:>8}  {:>9.1}  {:>9.1}  {:>10.1}  {:>10.1}  {:>9.1}  {:>13.1}",
            lista.vizibile().len(),
            model_ms,
            filtru_ms,
            cautare_ms,
            incarca_ms,
            cadru1,
            cadru_jos
        );
    }
}
