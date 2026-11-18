//! Randeaza editorul fara ecran, intr-un PNG: comparatia cu machetele si cu
//! capturile v3.4 (`referinta/captura_v34/`).
//!
//!   cargo run -p gmotion-fereastra --example captura -- \
//!       instantaneu.json SURSA BAZA schema.obiect iesire.png [--unificat] [--fara-pliere] [--mediu prod] [--font-mono NUME] [--tema intunecat|luminos]
use std::rc::Rc;

use gmotion_fereastra::{
    aplica_tema, incarca_editor, model_editor, AntetEditor, EditorDiferente, Mediu,
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
             [--unificat] [--fara-pliere] [--mediu dev|test|prod] [--font-mono NUME]"
        );
        std::process::exit(2);
    }
    let are = |f: &str| a.iter().any(|x| x == f);
    let mediu = match a
        .iter()
        .position(|x| x == "--mediu")
        .map(|i| a[i + 1].as_str())
    {
        Some("dev") => Mediu::Dev,
        Some("test") => Mediu::Test,
        _ => Mediu::Prod,
    };

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

    let (w, h) = (1440u32, 900u32);
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
    // proba ca fonturile incorporate chiar se folosesc: cu un nume inexistent
    // randarea trebuie sa iasa diferita (altfel desenam oricum cu rezerva)
    if let Some(i) = a.iter().position(|x| x == "--font-mono") {
        ui.global::<gmotion_fereastra::Tema>()
            .set_font_mono(a[i + 1].as_str().into());
    }
    ui.set_alaturat(!are("--unificat"));
    ui.set_pliaza_identicul(!are("--fara-pliere"));
    let (schema, nume) = a[4].split_once('.').unwrap_or(("dbo", &a[4]));
    incarca_editor(
        &ui,
        &AntetEditor {
            schema,
            nume,
            tip: gmotion_core::nume_tip(&tip),
            sursa_baza: &a[2],
            sursa_server: &server,
            tinta_baza: &a[3],
            tinta_server: &server,
            mediu,
            stare_obiecte: "obiectul 4 din 18 nerevizuite",
            revizuit: false,
            in_plan: false,
        },
        &model,
    );
    ui.show().expect("fereastra nu se arata");
    fereastra.set_size(slint::PhysicalSize::new(w, h));
    scrie_png(&fereastra, w, h, &a[5]);
    println!(
        "{} randuri in model, {} diferente -> {}",
        model.randuri.len(),
        model.diferente.len(),
        a[5]
    );
}
