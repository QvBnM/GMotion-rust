//! Randeaza editorul fara ecran, intr-un PNG, pentru comparatia cu v3.4.
//!
//!   cargo run -p gmotion-fereastra --example captura -- instantaneu.json REF BAZA schema.obiect iesire.png [font-mono]
use std::rc::Rc;

use gmotion_fereastra::{incarca_editor, model_editor, AntetEditor, EditorDiferente, Tema};
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

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let inst: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&a[1]).unwrap()).unwrap();
    let def = |baza: &str| -> String {
        inst["baze"][baza]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| {
                format!(
                    "{}.{}",
                    o["schema"].as_str().unwrap(),
                    o["nume"].as_str().unwrap()
                ) == a[4]
            })
            .map(|o| o["definitie"].as_str().unwrap().to_string())
            .unwrap_or_default()
    };
    let (r, t) = (
        gmotion_core::pregateste_afisare(&def(&a[2])),
        gmotion_core::pregateste_afisare(&def(&a[3])),
    );
    let model = model_editor::construieste(&r, &t);

    let (w, h) = (1400u32, 860u32);
    let fereastra = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(fereastra.clone()))).unwrap();
    let ui = EditorDiferente::new().unwrap();
    if let Some(f) = a.get(6) {
        ui.global::<Tema>().set_font_mono(f.into());
    }
    incarca_editor(
        &ui,
        &AntetEditor {
            obiect: &a[4],
            baza_ref: &a[2],
            baza_tinta: &a[3],
            scor: Some(gmotion_core::scor_similaritate(&r, &t)),
            pozitie_obiect: "13/18",
        },
        &model,
    );
    ui.show().unwrap();
    fereastra.set_size(slint::PhysicalSize::new(w, h));
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
    img.save(&a[5]).unwrap();
    println!(
        "{} randuri, {} diferente -> {}",
        model.randuri.len(),
        model.diferente.len(),
        a[5]
    );
}
