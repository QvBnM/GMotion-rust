//! Fotografiaza lista cu panoul de detalii, prin comenzile adevarate: intai
//! lista pe toata latimea, apoi clic pe un rand (panoul deschis, cu impact si
//! nota), apoi panoul tras mai lat.
//!
//!   cargo run --release -p gmotion-app --example captura_detalii -- \
//!       instantaneu.json SURSA BAZA inchis.png deschis.png lat.png \
//!       [schema.obiect] [--tema intunecat|luminos]
use std::rc::Rc;

use gmotion_app::rutare::{Aplicatie, Comanda};
use gmotion_db::SursaObiecte;
use gmotion_fereastra::{
    aplica_tema, incarca_detalii, incarca_lista, incarca_previzualizare, AntetLista,
    FereastraLista, Mediu, Mod,
};
use gmotion_modul_diff_obiecte as modul;
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

fn scrie(fereastra: &MinimalSoftwareWindow, w: u32, h: u32, cale: &str) {
    fereastra.request_redraw();
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
    if a.len() < 7 {
        eprintln!(
            "folosire: captura_detalii instantaneu.json SURSA BAZA inchis.png deschis.png lat.png [schema.obiect] [--tema intunecat|luminos]"
        );
        std::process::exit(2);
    }
    let tema = match a
        .iter()
        .position(|x| x == "--tema")
        .and_then(|i| a.get(i + 1))
    {
        Some(t) if t == "luminos" => Mod::Luminos,
        _ => Mod::Intunecat,
    };
    let mut fisier = gmotion_db::SursaFixture::din_fisier(&a[1]).expect("instantaneul se citeste");
    let server = fisier.descriere();
    let sursa = modul::pregateste(&fisier.obiecte(&a[2]).expect("baza sursa"));
    let tinta = modul::pregateste(&fisier.obiecte(&a[3]).expect("baza tinta"));
    let comparare = modul::compara(&a[2], &sursa, &[(a[3].clone(), tinta.clone())], |_| {});
    let mut app = Aplicatie::noua(comparare.randuri, sursa, tinta, modul::Revizuire::noua());
    app.asigura_detalii(0, 40);

    let (w, h) = (1440u32, 900u32);
    let fereastra = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(fereastra.clone()))).expect("platforma");
    let ui = FereastraLista::new().expect("fereastra listei");
    aplica_tema(&ui, tema);
    let incarca = |ui: &FereastraLista, app: &Aplicatie| {
        incarca_lista(
            ui,
            &AntetLista {
                sursa_baza: &a[2],
                tinta_baza: &a[3],
                servere: &server,
                mediu: Mediu::Prod,
                in_plan: app.lista().in_plan().len(),
            },
            app.lista(),
        );
        if let Some((s, t, setari)) = app.texte_obiect() {
            incarca_previzualizare(ui, s, t, &setari);
        }
        incarca_detalii(ui, &app.panou_detalii().pentru_fereastra());
    };

    incarca(&ui, &app);
    ui.show().expect("se arata");
    fereastra.set_size(slint::PhysicalSize::new(w, h));
    scrie(&fereastra, w, h, &a[4]);

    let p = a
        .get(7)
        .filter(|x| !x.starts_with("--"))
        .and_then(|cerut| {
            (0..app.lista().vizibile().len())
                .find(|&p| app.lista().obiect(p).is_some_and(|o| o.id() == *cerut))
        })
        .unwrap_or(2);
    app.comanda(Comanda::ClicPeRand(p));
    let id = app.obiect().map(|o| o.id()).unwrap_or_default();
    app.comanda(Comanda::PuneNota(
        id,
        "Am ramas aici: de verificat cu echipa de facturare.".into(),
    ));
    app.comanda(Comanda::ComutaRevizuit);
    incarca(&ui, &app);
    assert!(ui.get_detalii_deschise(), "clicul pe rand deschide panoul");
    scrie(&fereastra, w, h, &a[5]);

    ui.set_latime_detalii(640.0);
    scrie(&fereastra, w, h, &a[6]);
    println!(
        "{} ({}, {}, {})",
        app.obiect().map(|o| o.id()).unwrap_or_default(),
        a[4],
        a[5],
        a[6]
    );
}
