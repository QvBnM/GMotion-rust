//! Randeaza ecranul de conectare fara ecran, intr-un PNG, cu datele din
//! macheta (`cercetare/machete/conectare-noua/`), ca sa se poata compara.
//!
//!   cargo run --release -p gmotion-fereastra --example captura_conectare -- iesire.png \
//!       [--tema intunecat|luminos] [--limba ro|en] [--alta] [--latime N --inaltime N] [--sistem 1.5]
//!
//! `--latime` / `--inaltime` sunt pixeli de ecran; `--sistem` e marirea din
//! Windows. Scara se pune ca in aplicatie (`scara.rs`).
use std::rc::Rc;

use gmotion_fereastra::model_conectare::{
    Acreditari, Conectare, Parte, Recenta, ServerGasit, StareConexiune,
};
use gmotion_fereastra::{aplica_tema, incarca_conectare, incarca_jurnal, FereastraConectare, Mod};
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
    let cale = a.get(1).cloned().unwrap_or_else(|| "conectare.png".into());

    // datele din macheta `cercetare/machete/conectare-noua/`; `--alta`
    // fotografiaza varianta cu modificata pe alt server
    let alta = a.iter().any(|x| x == "--alta");
    let baze: Vec<String> = [
        "CharismaDev",
        "CharismaProd",
        "CharismaTest",
        "Charisma_Copy",
        "CharismaWMS",
        "CharismaHR",
        "CharismaHR_Test",
        "CharismaFin",
        "CharismaFin_2024",
        "CharismaB2B",
        "CharismaB2B_Dev",
        "Charisma_Raportare",
        "master_copie",
    ]
    .iter()
    .map(|b| b.to_string())
    .collect();
    let mut c = Conectare::noua();
    c.sursa = Parte {
        server: "SRV-DEV\\SQL2019".into(),
        acreditari: Acreditari::noi("deploy_ro", "nu se vede"),
        baza: "CharismaDev".into(),
        baze: baze.clone(),
        stare: StareConexiune::Conectat("conectat · SQL Server 2019 · 14 baze".into()),
        auth_windows: false,
        ..Parte::default()
    };
    if alta {
        c.pune_aceeasi_conexiune(false);
        c.baza = Parte {
            server: "srv-prod.firma.local,1433".into(),
            acreditari: Acreditari::noi("deploy_ro", "nu se vede"),
            baza: "CharismaProd".into(),
            baze: vec![
                "CharismaProd".into(),
                "CharismaProd_Arhiva".into(),
                "CharismaProd_Test".into(),
            ],
            stare: StareConexiune::Conectat("conectat · SQL Server 2016 SP2 · 3 baze".into()),
            auth_windows: false,
            ..Parte::default()
        };
    } else {
        c.pune_aceeasi_conexiune(true);
        c.alege_baza_modificata("CharismaProd");
    }
    // sa nu se foloseasca `ServerGasit`: in macheta lista e inchisa
    let _ = ServerGasit::nou("");
    c.recente = vec![
        Recenta {
            sursa: "CharismaDev".into(),
            server: "SRV01".into(),
            baza: "CharismaProd".into(),
            cand: "ieri, 17:40".into(),
            progres: "124 din 336 revizuite".into(),
        },
        Recenta {
            sursa: "CharismaDev".into(),
            server: "SRV01".into(),
            baza: "CharismaTest".into(),
            cand: "luni, 10:12".into(),
            progres: "deploy facut".into(),
        },
    ];

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
    let ui = FereastraConectare::new().expect("fereastra nu se creeaza");
    let tema = a
        .iter()
        .position(|x| x == "--tema")
        .and_then(|i| a.get(i + 1))
        .and_then(|n| Mod::din_nume(n))
        .unwrap_or_default();
    aplica_tema(&ui, tema);
    // `--limba ro|en`: pana la auditul 2026-11-23 numai `captura_lista` o
    // accepta, deci interfata in engleza nu se putea vedea pe 3 din 4 ecrane
    // (C-S9). Butonul de limba are nevoie si de lista limbilor, altfel se
    // deseneaza gol.
    let limba = a
        .iter()
        .position(|x| x == "--limba")
        .and_then(|i| a.get(i + 1))
        .and_then(|n| gmotion_core::Limba::din_nume(n))
        .unwrap_or_default();
    gmotion_fereastra::aplica_limba(&ui, limba);
    {
        use gmotion_core::Limba;
        let etichete: Vec<slint::SharedString> = Limba::toate()
            .iter()
            .map(|x| slint::SharedString::from(x.eticheta()))
            .collect();
        ui.set_limbi(slint::ModelRc::new(slint::VecModel::from(etichete)));
        ui.set_limba_aleasa(Limba::toate().iter().position(|x| *x == limba).unwrap_or(0) as i32);
    }
    incarca_conectare(&ui, &c);
    incarca_jurnal(
        &ui,
        &[
            "[INFO] deschid conexiunea server=SRV-DEV\\SQL2019 baza=master".into(),
            "[WARN] certificatul serverului nu se verifica".into(),
            "[INFO] baze citite sursa=true baze=14".into(),
            "[INFO] actiune ecran=conectare nume=baza_schimbata".into(),
        ],
    );
    ui.show().expect("fereastra nu se arata");
    let sistem: f32 = a
        .iter()
        .position(|x| x == "--sistem")
        .and_then(|i| a.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.0);
    let k = gmotion_fereastra::scara::scara(w as f32 / sistem, h as f32 / sistem, sistem);
    let factor = sistem * k;
    ui.window()
        .dispatch_event(slint::platform::WindowEvent::ScaleFactorChanged {
            scale_factor: factor,
        });
    fereastra.set_size(slint::PhysicalSize::new(w, h));
    ui.set_scund((h as f32 / factor) < 820.0);
    println!("scara {k:.2}, factor {factor:.2}");
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
