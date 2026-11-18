//! Randeaza ecranul listei fara ecran, intr-un PNG, pornind de la o comparare
//! adevarata dintr-un instantaneu.
//!
//!   cargo run --release -p gmotion-app --example captura_lista -- \
//!       instantaneu.json SURSA BAZE iesire.png [schema.obiect-selectat] [--tema intunecat|luminos] [--limba ro|en] [--toate] [--latime 960]
//!
//! `BAZE` poate fi o lista separata prin virgula: asa se fotografiaza si
//! coloana „Baza".
use std::rc::Rc;

use gmotion_app::pornire;
use gmotion_app::rutare::{Aplicatie, Comanda};
use gmotion_db::SursaObiecte;
use gmotion_fereastra::{
    aplica_tema, incarca_lista, incarca_previzualizare, model_lista, AntetLista, FereastraLista,
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

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 5 {
        eprintln!("folosire: captura_lista instantaneu.json SURSA BAZA iesire.png [schema.obiect]");
        std::process::exit(2);
    }
    let fisier = gmotion_db::SursaFixture::din_fisier(&a[1]).expect("instantaneul nu se citeste");
    let server = fisier.descriere();

    // aceeasi cale ca aplicatia: conectare -> comparare -> `Aplicatie`.
    // Captura n-are voie sa-si construiasca singura lista; a doua
    // implementare a aratat deja altceva decat aplicatia (scorul lipsea).
    let mut c = gmotion_fereastra::model_conectare::Conectare::noua();
    c.instantaneu = Some(a[1].clone());
    c.sursa.baza = a[2].clone();
    for baza in a[3].split(',') {
        c.baza.comuta_baza(baza.trim());
    }
    let r = pornire::compara(&c).expect("compararea merge pe instantaneu");
    let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());

    // implicit doar modificatele; `--toate` arata toate starile (harta cu
    // toate culorile)
    if !a.iter().any(|x| x == "--toate") {
        app.comanda(Comanda::Filtreaza(Some(model_lista::Stare::Modificat)));
    }
    // detaliile se calculeaza doar pentru randurile de pe ecran, ca in
    // aplicatie: aici incap ~12
    app.asigura_detalii(0, 12);
    // un obiect revizuit, ca sa se vada bifa si progresul
    app.comanda(Comanda::Selecteaza(1));
    app.comanda(Comanda::ComutaRevizuit);
    let cerut = a.get(5).filter(|x| !x.starts_with("--"));
    let pozitie = match cerut {
        Some(cerut) => (0..app.lista().vizibile().len()).find(|&p| {
            app.lista()
                .obiect(p)
                .is_some_and(|o| o.id().eq_ignore_ascii_case(cerut))
        }),
        None => Some(0),
    };
    app.comanda(Comanda::Selecteaza(pozitie.unwrap_or(0)));
    // antetul arata o singura tinta, ca in aplicatie
    let baze = app.baze();
    let tinta_baza = match baze.as_slice() {
        [una] => una.clone(),
        multe => format!("{} baze", multe.len()),
    };
    let lista = app.lista();

    // `--latime N`: fereastra mai ingusta, ca sa se vada cum curge antetul
    let w = a
        .iter()
        .position(|x| x == "--latime")
        .and_then(|i| a.get(i + 1))
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1440);
    let h = 900u32;
    let fereastra = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(fereastra.clone()))).expect("platforma");
    let ui = FereastraLista::new().expect("fereastra nu se creeaza");
    let tema = a
        .iter()
        .position(|x| x == "--tema")
        .and_then(|i| a.get(i + 1))
        .and_then(|n| gmotion_fereastra::Mod::din_nume(n))
        .unwrap_or_default();
    aplica_tema(&ui, tema);
    let limba = a
        .iter()
        .position(|x| x == "--limba")
        .and_then(|i| a.get(i + 1))
        .and_then(|n| gmotion_fereastra::texte::Limba::din_nume(n))
        .unwrap_or_default();
    gmotion_fereastra::aplica_limba(&ui, limba);
    gmotion_fereastra::leaga_harta(&ui);
    incarca_lista(
        &ui,
        &AntetLista {
            sursa_baza: &a[2],
            tinta_baza: &tinta_baza,
            servere: &server,
            in_plan: lista.in_plan().len(),
        },
        lista,
    );
    if let Some((sursa, tinta, setari)) = app.texte_obiect() {
        incarca_previzualizare(&ui, sursa, tinta, &setari);
    }
    // bula de pe harta, ca sa se vada in captura
    let pas = gmotion_fereastra::harta::Scara::cu_latura(ui.get_patrat() as u32)
        .unwrap_or(gmotion_fereastra::harta::SCARI[0])
        .pas as f32;
    ui.invoke_peste_harta(0, 2.0 * pas + 2.0, 2.0);
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
    println!(
        "{} obiecte, {} vizibile -> {}",
        lista.obiecte().len(),
        lista.vizibile().len(),
        a[4]
    );
}
