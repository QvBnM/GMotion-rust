//! Randeaza ecranul de conectare fara ecran, intr-un PNG, cu datele din
//! machetă (`cercetare/machete/Conectare.dc.html`), ca sa se poata compara.
//!
//!   cargo run --release -p gmotion-fereastra --example captura_conectare -- iesire.png
use std::rc::Rc;

use gmotion_fereastra::model_conectare::{Acreditari, Conectare, Parte, Recenta, StareConexiune};
use gmotion_fereastra::{aplica_tema, incarca_conectare, FereastraConectare};
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
    let cale = a.get(1).cloned().unwrap_or_else(|| "conectare.png".into());

    let mut c = Conectare::noua();
    c.sursa = Parte {
        server: "SRV-DEV\\SQL2019".into(),
        baza: "CharismaDev".into(),
        baze: vec!["CharismaDev".into(), "CharismaDev_Copy".into()],
        stare: StareConexiune::Conectat("conectat · SQL Server 2019 · 21.204 obiecte".into()),
        auth_windows: true,
        ..Parte::default()
    };
    c.baza = Parte {
        server: "srv-prod.firma.local,1433".into(),
        acreditari: Acreditari::noi("deploy_ro", "nu se vede"),
        baza: "CharismaProd".into(),
        baze: vec!["CharismaProd".into()],
        stare: StareConexiune::Conectat("conectat · SQL Server 2016 SP2 · 21.183 obiecte".into()),
        auth_windows: false,
    };
    c.recente = vec![
        Recenta {
            sursa: "CharismaDev".into(),
            baza: "CharismaProd".into(),
            mediu: "PROD".into(),
            cand: "ieri, 17:40".into(),
            progres: "124 din 336 revizuite".into(),
        },
        Recenta {
            sursa: "CharismaDev".into(),
            baza: "CharismaTest".into(),
            mediu: "TEST".into(),
            cand: "luni, 10:12".into(),
            progres: "deploy făcut".into(),
        },
        Recenta {
            sursa: "Charisma_Copy".into(),
            baza: "CharismaDev".into(),
            mediu: "DEV".into(),
            cand: "12 noiembrie".into(),
            progres: "96 din 1.204 revizuite".into(),
        },
    ];

    let (w, h) = (1440u32, 900u32);
    let fereastra = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(fereastra.clone()))).expect("platforma");
    let ui = FereastraConectare::new().expect("fereastra nu se creeaza");
    aplica_tema(&ui);
    incarca_conectare(&ui, &c);
    ui.show().expect("fereastra nu se arata");
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
    img.save(&cale).expect("PNG-ul nu se scrie");
    println!(
        "{} -> {cale}",
        if c.poate_compara() {
            "se poate compara"
        } else {
            "lipseste ceva"
        }
    );
}
