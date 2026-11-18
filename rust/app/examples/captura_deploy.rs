//! Fotografiaza ecranul de deploy pe drumul adevarat: compararea dintr-un
//! instantaneu, obiecte puse „in plan" prin comenzi, garda pe ce se
//! reciteste, iar optional scrierea intr-un director temporar.
//!
//!   cargo run --release -p gmotion-app --example captura_deploy -- \
//!       instantaneu.json SURSA BAZA iesire.png [--tema intunecat|luminos] [--scris] [--atinsa]
use std::rc::Rc;

use gmotion_app::deploy::{ecran, StareDeploy};
use gmotion_app::pornire;
use gmotion_app::rutare::{Aplicatie, Comanda};
use gmotion_fereastra::model_conectare::Conectare;
use gmotion_fereastra::{aplica_tema, incarca_deploy, FereastraDeploy, Mediu, Mod};
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

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 5 {
        eprintln!("folosire: captura_deploy instantaneu.json SURSA BAZA iesire.png [--tema T] [--scris] [--atinsa]");
        std::process::exit(2);
    }
    let are = |f: &str| a.iter().any(|x| x == f);
    let mut c = Conectare::noua();
    c.instantaneu = Some(a[1].clone());
    c.sursa.baza = a[2].clone();
    c.baza.baza = a[3].clone();
    let r = pornire::compara(&c).expect("compararea");
    let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());
    // primele sase obiecte, in plan, prin aceleasi comenzi ca fereastra
    for p in 0..6.min(app.lista().vizibile().len()) {
        app.comanda(Comanda::Selecteaza(p));
        app.comanda(Comanda::ComutaInPlan);
    }
    app.comanda(Comanda::DeschideDeploy);
    let mut st = StareDeploy::nou(app.plan(&a[2], "2026-11-18 14:41"));
    let recitit = pornire::compara(&c).expect("recitirea");
    if are("--atinsa") {
        // cineva a schimbat in baza primul obiect din plan intre timp
        let mut tinte = recitit.tinte.clone();
        if let (Some(p), Some((_, t))) = (st.plan.pasi.first(), tinte.first_mut()) {
            if let Some(o) = t.get_mut(&p.obiect) {
                o.definitie.push_str("\n-- schimbat de altcineva");
            }
        }
        st.aplica_verificarea(recitit.sursa, tinte, "14:52");
    } else {
        st.aplica_verificarea(recitit.sursa, recitit.tinte, "14:52");
    }
    let director = std::env::temp_dir().join("gmotion-captura-deploy");
    if are("--scris") {
        std::fs::remove_dir_all(&director).ok();
        st.scrie(&director, "2026-11-18 14:53");
    }

    let (w, h) = (1440u32, 900u32);
    let fereastra = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(fereastra.clone()))).expect("platforma");
    let ui = FereastraDeploy::new().expect("fereastra");
    let tema = a
        .iter()
        .position(|x| x == "--tema")
        .and_then(|i| a.get(i + 1))
        .and_then(|n| Mod::din_nume(n))
        .unwrap_or_default();
    aplica_tema(&ui, tema);
    let e = ecran(&st, app.sursa(), &director.display().to_string(), 10);
    incarca_deploy(&ui, &e, Mediu::Prod);
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
    img.save(&a[4]).expect("PNG-ul nu se scrie");
    println!("{} pasi, garda {:?} -> {}", e.pasi.len(), e.garda, a[4]);
    std::fs::remove_dir_all(&director).ok();
}
