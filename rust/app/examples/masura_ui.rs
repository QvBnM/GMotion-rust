//! Cat tine firul ferestrei dupa o comparare mare: primirea compararii,
//! prima desenare a listei, si fiecare bataie de ceas cat sosesc detaliile.
//! Pe Windows, la 12.000 de obiecte, firul statea 2-8 s („interfata nu
//! raspunde", 2026-11-25): primirea detaliilor si dependentele erau O(n²) /
//! pe firul ferestrei. Afiseaza doar ce trece de 50 ms (30 ms la detalii).
//!
//!   cargo run --profile proba -p gmotion-app --example masura_ui -- instantaneu.json
//!
//! Instantaneul trebuie sa aiba bazele `Ref` si `C1`.
use std::rc::Rc;
use std::time::Instant;

use gmotion_app::stare::{Progres, Stare};
use gmotion_fereastra::{AntetLista, FereastraLista};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};

struct Platforma(Rc<MinimalSoftwareWindow>);
impl slint::platform::Platform for Platforma {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let w = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(w.clone()))).unwrap();
    let ui = FereastraLista::new().unwrap();
    ui.window().set_size(slint::PhysicalSize::new(1920, 1080));
    gmotion_fereastra::aplica_tema(&ui, gmotion_fereastra::Mod::Intunecat);
    let d = std::env::temp_dir().join("gm-masura-ui");
    let ses = gmotion_platforma::Sesiuni::in_director("diff-obiecte", d).unwrap();
    let mut c = gmotion_fereastra::model_conectare::Conectare::noua();
    c.instantaneu = Some(a[1].clone());
    c.sursa.baza = "Ref".into();
    c.baza.comuta_baza("C1");
    let mut s = Stare::noua(c, ses);
    s.compara();
    loop {
        let t = Instant::now();
        let p = s.verifica_compararea();
        let ms = t.elapsed().as_millis();
        if ms > 50 {
            println!("verifica_compararea {ms} ms");
        }
        if matches!(p, Progres::Terminata) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let deseneaza = |s: &mut Stare, eticheta: &str| {
        let t0 = Instant::now();
        let app = s.app.as_mut().unwrap();
        if let Some(p) = app.lista().selectat() {
            app.asigura_detalii(p, p + 1);
        }
        let t = Instant::now();
        gmotion_fereastra::incarca_lista(
            &ui,
            &AntetLista {
                sursa_baza: "Ref",
                tinta_baza: "C1",
                servere: "S",
                in_plan: 0,
            },
            app.lista(),
        );
        let l = t.elapsed().as_millis();
        let t = Instant::now();
        gmotion_fereastra::incarca_fundal_harta(&ui);
        let f = t.elapsed().as_millis();
        let t = Instant::now();
        match app.texte_obiect() {
            Some((x, y, setari)) => gmotion_fereastra::incarca_previzualizare(&ui, x, y, &setari),
            None => gmotion_fereastra::goleste_previzualizarea(&ui),
        }
        let pz = t.elapsed().as_millis();
        let t = Instant::now();
        let mut panou = app.panou_detalii();
        gmotion_fereastra::incarca_detalii(
            &ui,
            &panou.pentru_fereastra(gmotion_core::Limba::Romana),
        );
        let pd = t.elapsed().as_millis();
        let t = Instant::now();
        w.request_redraw();
        w.draw_if_needed(|_| {});
        let dr = t.elapsed().as_millis();
        println!(
            "{eticheta}: total {} ms (lista {l}, fundal {f}, previz {pz}, panou {pd}, cadru {dr})",
            t0.elapsed().as_millis()
        );
    };
    deseneaza(&mut s, "prima");
    for i in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(120));
        let t = Instant::now();
        let sch = s.verifica_lucrarile();
        let ms = t.elapsed().as_millis();
        if ms > 30 {
            println!("verifica_lucrarile {ms} ms");
        }
        if sch {
            deseneaza(&mut s, &format!("dupa detalii {i}"));
        }
    }
    // bifa „Revizuit" pe un rand: comanda, salvarea sesiunii, desenarea
    for k in 0..3 {
        let t = Instant::now();
        s.app
            .as_mut()
            .unwrap()
            .comanda(gmotion_app::rutare::Comanda::ComutaRevizuitLa(14 + k));
        let c = t.elapsed().as_millis();
        let t = Instant::now();
        s.salveaza(None);
        let sv = t.elapsed().as_millis();
        println!("revizuit {k}: comanda {c} ms, salvare {sv} ms");
        deseneaza(&mut s, "dupa revizuit");
    }
    // „Toate in plan", salvarea si planul de deploy, pe toate modificatele
    let t = Instant::now();
    s.app
        .as_mut()
        .unwrap()
        .comanda(gmotion_app::rutare::Comanda::PuneVizibileleInPlan(false));
    let c = t.elapsed().as_millis();
    let t = Instant::now();
    s.salveaza(None);
    let sv = t.elapsed().as_millis();
    let t = Instant::now();
    let p = s.app.as_ref().unwrap().plan("Ref", "2026-11-25 12:00:00");
    let pl = t.elapsed().as_millis();
    println!(
        "toate in plan: comanda {c} ms, salvare {sv} ms, plan {pl} ms ({} pasi)",
        p.pasi.len()
    );
    deseneaza(&mut s, "dupa plan");
}
