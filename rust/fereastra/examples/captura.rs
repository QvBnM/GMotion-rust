//! Randeaza editorul fara ecran, intr-un PNG: comparatia cu machetele si cu
//! capturile v3.4 (`referinta/captura_v34/`).
//!
//!   cargo run -p gmotion-fereastra --example captura -- \
//!       instantaneu.json SURSA BAZA schema.obiect iesire.png [--unificat] [--fara-pliere] [--wrap] [--font-mono NUME] [--tema intunecat|luminos] [--trage N] [--previzualizeaza N] [--editeaza]
use std::rc::Rc;

use gmotion_fereastra::{aplica_tema, incarca_editor, model_editor, AntetEditor, EditorDiferente};
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

/// Deseneaza fereastra intr-un PNG. Public ca sa-l foloseasca si `masura`.
pub fn scrie_png(fereastra: &MinimalSoftwareWindow, w: u32, h: u32, cale: &str) {
    let mut buf = vec![PremultipliedRgbaColor::background(); (w * h) as usize];
    fereastra.draw_if_needed(|rd| {
        rd.render(&mut buf, w as usize);
    });
    let mut img = image::RgbaImage::new(w, h);
    for (i, p) in buf.iter().enumerate() {
        img.put_pixel(
            i as u32 % w,
            i as u32 / w,
            image::Rgba([p.red, p.green, p.blue, p.alpha]),
        );
    }
    img.save(cale).expect("PNG-ul nu se scrie");
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 6 {
        eprintln!(
            "folosire: captura instantaneu.json SURSA BAZA schema.obiect iesire.png \
             [--unificat] [--fara-pliere] [--font-mono NUME]"
        );
        std::process::exit(2);
    }
    let are = |f: &str| a.iter().any(|x| x == f);

    let inst: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&a[1]).expect("instantaneul nu se citeste"))
            .expect("instantaneul nu e JSON");
    let server = inst["server"].as_str().unwrap_or("").to_string();
    let obiect = |baza: &str| -> (String, String) {
        inst["baze"][baza]
            .as_array()
            .expect("baza lipseste din instantaneu")
            .iter()
            .find(|o| {
                format!(
                    "{}.{}",
                    o["schema"].as_str().unwrap_or(""),
                    o["nume"].as_str().unwrap_or("")
                ) == a[4]
            })
            .map(|o| {
                (
                    o["definitie"].as_str().unwrap_or("").to_string(),
                    o["tip"].as_str().unwrap_or("P").to_string(),
                )
            })
            .unwrap_or_default()
    };
    let (def_sursa, tip) = obiect(&a[2]);
    let (def_baza, _) = obiect(&a[3]);
    let (r, t) = (
        gmotion_core::pregateste_afisare(&def_sursa),
        gmotion_core::pregateste_afisare(&def_baza),
    );
    let model = model_editor::construieste(&r, &t);

    // `--latime N` / `--inaltime N`: ecranul se fotografiaza si la rezolutii
    // mici (1366x768 e cel mai comun laptop), ca sa se vada ce iese din el
    let numar = |f: &str, implicit: u32| -> u32 {
        a.iter()
            .position(|x| x == f)
            .and_then(|i| a.get(i + 1))
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(implicit)
    };
    let (w, h) = (numar("--latime", 1440), numar("--inaltime", 900));
    let fereastra = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(fereastra.clone()))).expect("platforma");
    let ui = EditorDiferente::new().expect("fereastra nu se creeaza");
    let tema = a
        .iter()
        .position(|x| x == "--tema")
        .and_then(|i| a.get(i + 1))
        .and_then(|n| gmotion_fereastra::Mod::din_nume(n))
        .unwrap_or_default();
    aplica_tema(&ui, tema);
    gmotion_fereastra::aplica_limba(&ui, Default::default());
    // fara lista, coloana din stanga ar iesi goala in captura
    ui.set_latime_lista(0.0);
    // proba ca fonturile incorporate chiar se folosesc: cu un nume inexistent
    // randarea trebuie sa iasa diferita (altfel desenam oricum cu rezerva)
    if let Some(i) = a.iter().position(|x| x == "--font-mono") {
        ui.global::<gmotion_fereastra::Tema>()
            .set_font_mono(a[i + 1].as_str().into());
    }
    // `--trage N`: fotografiaza editorul in timp ce se trage diferenta N,
    // ca sa se vada tinta aprinsa (criteriul „drag & drop" din etapa 21)
    if let Some(i) = a.iter().position(|x| x == "--trage") {
        if let Some(n) = a.get(i + 1).and_then(|x| x.parse::<i32>().ok()) {
            ui.set_trage_diferenta(n);
            ui.set_trage_x(80.0);
            // blocul luat (primul din grup), textul lui pentru fantoma si,
            // cu `--sub-mouse R`, randul afisat de sub mouse (etapa 23)
            if let Some(&bloc) = usize::try_from(n)
                .ok()
                .and_then(|k| model.grupuri.get(k))
                .and_then(|g| g.first())
            {
                ui.set_trage_bloc(bloc as i32);
                let text = model
                    .randuri
                    .iter()
                    .find(|r| r.bloc == Some(bloc) && r.stanga.nr.is_some())
                    .map(|r| {
                        r.stanga
                            .bucati
                            .iter()
                            .map(|b| b.text.as_str())
                            .collect::<String>()
                    })
                    .unwrap_or_default();
                ui.set_trage_text(text.trim().into());
            }
            let sub = a
                .iter()
                .position(|x| x == "--sub-mouse")
                .and_then(|i| a.get(i + 1))
                .and_then(|x| x.parse::<i32>().ok());
            if let Some(r) = sub {
                ui.set_trage_rand(r);
                ui.set_trage_gx(1000.0);
                ui.set_trage_gy(30.0 + r as f32 * 22.0 + 11.0);
            }
        }
    }
    ui.set_alaturat(!are("--unificat"));
    ui.set_pliaza_identicul(!are("--fara-pliere"));
    let (schema, nume) = a[4].split_once('.').unwrap_or(("dbo", &a[4]));
    incarca_editor(
        &ui,
        &AntetEditor {
            id: &format!("{}/{schema}.{nume}", a[3]),
            schema,
            nume,
            tip: gmotion_core::nume_tip(&tip),
            sursa_baza: &a[2],
            sursa_server: &server,
            tinta_baza: &a[3],
            tinta_server: &server,
            stare_obiecte: "obiectul 4 din 18 nerevizuite",
            revizuit: false,
            in_plan: false,
        },
        &model,
    );
    // `--editeaza`: campul de editare deschis, cu sintaxa cat scrii (etapa
    // 25); textul e cel al Modificatei
    if are("--editeaza") {
        gmotion_fereastra::leaga_editarea(&ui);
        ui.set_text_final(t.as_str().into());
        ui.set_editeaza_mod(true);
    }
    // `--taburi`: bara de taburi (etapa 16), cu trei fixate, unul editat,
    // si obiectul capturat in previzualizare
    if are("--taburi") {
        use gmotion_fereastra::model_taburi::Taburi;
        let mut t = Taburi::noi();
        let k = |n: &str| (a[3].clone(), format!("dbo.{n}"));
        for n in ["uspFactura", "uspIncarcaTot", "ifClientiDupaAn"] {
            let _ = t.fixeaza(k(n), |_| false);
        }
        t.deschide((a[3].clone(), a[4].clone()));
        gmotion_fereastra::incarca_taburi(&ui, &t, |c| c.1 == "dbo.uspIncarcaTot");
    }
    ui.show().expect("fereastra nu se arata");
    fereastra.set_size(slint::PhysicalSize::new(w, h));
    // `--wrap`: liniile rupte la latimea panoului; latimea se stie abia dupa
    // primul cadru, deci editorul se reincarca, ca in aplicatie
    if are("--wrap") {
        ui.set_impacheteaza(true);
        scrie_png(&fereastra, w, h, &a[5]);
        incarca_editor(
            &ui,
            &AntetEditor {
                id: &format!("{}/{schema}.{nume}", a[3]),
                schema,
                nume,
                tip: gmotion_core::nume_tip(&tip),
                sursa_baza: &a[2],
                sursa_server: &server,
                tinta_baza: &a[3],
                tinta_server: &server,
                stare_obiecte: "obiectul 4 din 18 nerevizuite",
                revizuit: false,
                in_plan: false,
            },
            &model,
        );
        fereastra.request_redraw();
    }
    // `--previzualizeaza N`: mouse-ul se plimba peste canal pana ajunge pe
    // sageata diferentei N, ca un om; previzualizarea o calculeaza aceeasi
    // `aplica_blocuri` ca in aplicatie (criteriul „hover" din etapa 21)
    if let Some(n) = a
        .iter()
        .position(|x| x == "--previzualizeaza")
        .and_then(|i| a.get(i + 1))
        .and_then(|x| x.parse::<i32>().ok())
    {
        let (r2, t2, grupuri) = (r.clone(), t.clone(), model.grupuri.clone());
        let slab = ui.as_weak();
        ui.on_previzualizeaza(move |k| {
            let Some(grup) = usize::try_from(k).ok().and_then(|k| grupuri.get(k)) else {
                return;
            };
            let rez = gmotion_core::aplica_blocuri(&r2, &t2, false, grup);
            let linii = model_editor::previzualizare(&t2, &rez, 2, 12);
            if let Some(ui) = slab.upgrade() {
                ui.set_previzualizare_linii(gmotion_fereastra::previzualizare_ui(&linii));
            }
        });
        ui.invoke_mergi_la(n);
        // un cadru, ca pozitiile din ListView sa existe inainte de mouse
        scrie_png(&fereastra, w, h, &a[5]);
        'cauta: for y in (60..h - 40).step_by(4) {
            for x in (w / 3..w).step_by(4) {
                ui.window()
                    .dispatch_event(slint::platform::WindowEvent::PointerMoved {
                        position: slint::LogicalPosition::new(x as f32, y as f32),
                    });
                if ui.get_previzualizare_diferenta() == n {
                    break 'cauta;
                }
            }
        }
        fereastra.request_redraw();
    }
    scrie_png(&fereastra, w, h, &a[5]);
    println!(
        "{} randuri in model, {} diferente -> {}",
        model.randuri.len(),
        model.diferente.len(),
        a[5]
    );
}
