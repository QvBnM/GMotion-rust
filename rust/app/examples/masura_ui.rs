//! Cat tine firul ferestrei dupa o comparare mare: primirea compararii,
//! prima desenare a listei, si fiecare bataie de ceas cat sosesc detaliile.
//! Pe Windows, la 12.000 de obiecte, firul statea 2-8 s („interfata nu
//! raspunde", 2026-11-25): primirea detaliilor si dependentele erau O(n²) /
//! pe firul ferestrei. Afiseaza doar ce trece de 50 ms (30 ms la detalii).
//!
//!   cargo run --profile proba -p gmotion-app --example masura_ui -- instantaneu.json
//!
//! Instantaneul trebuie sa aiba bazele `Ref` si `C1`; unul de 12.000 de
//! obiecte: `python3 -I unelte/genereaza-mare.py 12000 mare.json`.
//!
//! La final, derularea (2026-11-25, „scroll-ul rapid nu e fluid"): lista
//! randata in pixeli la 1920x1080, cu rotita, lent si rapid; pentru fiecare,
//! media, p95 si cel mai lung cadru.
//!
//! Apoi editorul (etapa 33: „redesenare lenta ms=119" la `alege_randul`):
//! schimbarea obiectului din Object Explorer, tab nou, clic pe tab, wrap,
//! Ctrl+W — pentru fiecare, Rust-ul (ce face `main.rs` la redesenare pe
//! ecranul editorului) si desenarea in pixeli, separat.
use std::rc::Rc;
use std::time::Instant;

use gmotion_app::rutare::Comanda;
use gmotion_app::stare::{Progres, Stare};
use gmotion_fereastra::{AntetLista, AreAplicatie, EditorDiferente, FereastraLista};
use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType, TargetPixel,
};

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
        // MASURA_FARA_FUNDAL=1: fara fundalul hartii, ca sa se vada cat costa
        if std::env::var("MASURA_FARA_FUNDAL").is_err() {
            gmotion_fereastra::incarca_fundal_harta(&ui, app.lista());
        }
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
    // toata lista (ca la Mihai: vizibile=28405), apoi bifa „Revizuit" pe un
    // rand: comanda, salvarea sesiunii, desenarea
    s.app
        .as_mut()
        .unwrap()
        .comanda(gmotion_app::rutare::Comanda::Filtreaza(None));
    deseneaza(&mut s, "toata lista");
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

    // derularea: fiecare cadru randat in pixeli, ca pe ecran
    let mut buf = vec![PremultipliedRgbaColor::background(); 1920 * 1080];
    let mut cadru = || {
        let t = Instant::now();
        w.request_redraw();
        w.draw_if_needed(|r| {
            r.render(&mut buf, 1920);
        });
        t.elapsed().as_secs_f64() * 1000.0
    };
    cadru();
    for (eticheta, pas, cadre) in [("lent", -40.0f32, 60), ("rapid", -400.0, 60)] {
        let mut ms: Vec<f64> = Vec::new();
        for i in 0..cadre {
            // inapoi in sus la jumatate, ca sa nu se opreasca la capat
            let d = if i < cadre / 2 { pas } else { -pas };
            let position = slint::LogicalPosition::new(500.0, 600.0);
            ui.window()
                .dispatch_event(slint::platform::WindowEvent::PointerMoved { position });
            ui.window()
                .dispatch_event(slint::platform::WindowEvent::PointerScrolled {
                    position,
                    delta_x: 0.0,
                    delta_y: d,
                });
            slint::platform::update_timers_and_animations();
            ms.push(cadru());
        }
        ms.sort_by(|a, b| a.total_cmp(b));
        let medie = ms.iter().sum::<f64>() / ms.len() as f64;
        let p95 = ms[(ms.len() * 95 / 100).min(ms.len() - 1)];
        println!(
            "derulare {eticheta}: medie {medie:.1} ms, p95 {p95:.1} ms, max {:.1} ms ({} cadre)",
            ms[ms.len() - 1],
            ms.len()
        );
    }

    // ---- editorul: ce face `main.rs` cand ecranul e al editorului
    let e_ui = EditorDiferente::din(ui.aplicatie());
    let (lat, inalt) = (1920usize, 1080usize);
    let mut buf = vec![PremultipliedRgbaColor::background(); lat * inalt];
    let mut randeaza = || {
        let t = Instant::now();
        w.request_redraw();
        w.draw_if_needed(|r| {
            r.render(&mut buf, lat);
        });
        t.elapsed().as_secs_f64() * 1000.0
    };
    let incarca = |s: &mut Stare| {
        let t = Instant::now();
        let pozitie = s.text_pozitie(gmotion_core::Limba::Romana);
        let app = s.app.as_mut().unwrap();
        if let Some(p) = app.lista().selectat() {
            app.asigura_detalii(p, p + 1);
        }
        if std::env::var("MASURA_FARA_FUNDAL").is_err() {
            gmotion_fereastra::incarca_fundal_harta(&ui, app.lista());
            e_ui.set_fundal_harta(ui.get_fundal_harta());
        }
        let (Some(o), Some(model)) = (app.obiect().cloned(), app.model_editor()) else {
            return t.elapsed().as_secs_f64() * 1000.0;
        };
        let refaceri = app.refaceri_model();
        gmotion_fereastra::incarca_editor(
            &e_ui,
            &gmotion_fereastra::AntetEditor {
                id: &format!("{}/{}.{}", o.baza, o.schema, o.nume),
                schema: &o.schema,
                nume: &o.nume,
                tip: &o.tip,
                sursa_baza: "Ref",
                sursa_server: "S",
                tinta_baza: &o.baza,
                tinta_server: "S",
                stare_obiecte: &pozitie,
                revizuit: o.revizuit,
                in_plan: o.in_plan,
                generatie: Some(refaceri),
            },
            &model,
        );
        drop(model);
        gmotion_fereastra::incarca_coloana_lista(&e_ui, app.lista(), app.arbore());
        gmotion_fereastra::incarca_taburi(&e_ui, app.taburi(), |c| app.are_text_final(c));
        let (text_sursa, text_final) = app.texte_curente().unwrap_or_default();
        let probleme = app.probleme_text_final();
        gmotion_fereastra::incarca_editarea(
            &e_ui,
            &gmotion_fereastra::StareEditare {
                omis: o.omis,
                editat: app.stare_text_final().0,
                se_poate_anula: app.se_poate_anula(),
                diferenta: app.diferenta_curenta(),
                ghidat: None,
                text_sursa: &text_sursa,
                text_final: &text_final,
                salvat: "",
                probleme: &probleme,
            },
        );
        ui.aplicatie().set_ecran(EditorDiferente::ECRAN);
        t.elapsed().as_secs_f64() * 1000.0
    };
    let mut pas = |s: &mut Stare, eticheta: &str, c: Option<Comanda>| {
        let t = Instant::now();
        if let Some(c) = c {
            s.app.as_mut().unwrap().comanda(c);
        }
        let cm = t.elapsed().as_secs_f64() * 1000.0;
        let rs = incarca(s);
        slint::platform::update_timers_and_animations();
        let dr = randeaza();
        println!(
            "editor {eticheta:<22} comanda {cm:5.1} ms, Rust {rs:5.1} ms, desenare {dr:5.1} ms, total {:5.1} ms",
            cm + rs + dr
        );
    };
    let app = s.app.as_mut().unwrap();
    app.comanda(Comanda::Filtreaza(Some(
        gmotion_fereastra::model_lista::Stare::Modificat,
    )));
    app.comanda(Comanda::Selecteaza(0));
    pas(&mut s, "deschide", Some(Comanda::Deschide));
    pas(&mut s, "acelasi, din nou", None);
    for p in [3usize, 5, 10, 12, 14] {
        pas(
            &mut s,
            &format!("explorer: deschide {p}"),
            Some(Comanda::Selecteaza(p)),
        );
    }
    pas(
        &mut s,
        "explorer: tab nou 16",
        Some(Comanda::DeschideInTabNou(Some(16))),
    );
    pas(
        &mut s,
        "explorer: tab nou 18",
        Some(Comanda::DeschideInTabNou(Some(18))),
    );
    pas(
        &mut s,
        "clic pe previzualizare",
        Some(Comanda::ActiveazaTab(None)),
    );
    pas(
        &mut s,
        "clic pe tabul 1",
        Some(Comanda::ActiveazaTab(Some(0))),
    );
    pas(
        &mut s,
        "aplica diferenta",
        Some(Comanda::AplicaDiferenta(None)),
    );
    pas(&mut s, "anuleaza", Some(Comanda::Anuleaza));
    e_ui.set_impacheteaza(false);
    pas(&mut s, "wrap oprit", None);
    e_ui.set_impacheteaza(true);
    pas(&mut s, "wrap pornit", None);
    pas(&mut s, "ctrl+w", Some(Comanda::InchideTabActiv));
    let t = Instant::now();
    s.salveaza(None);
    println!(
        "salvarea pe loc (cea din fundal o muta de pe fir): {:.1} ms",
        t.elapsed().as_secs_f64() * 1000.0
    );
}
