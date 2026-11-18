//! Randeaza ecranul listei fara ecran, intr-un PNG, pornind de la o comparare
//! adevarata dintr-un instantaneu.
//!
//!   cargo run --release -p gmotion-fereastra --example captura_lista -- \
//!       instantaneu.json SURSA BAZA iesire.png [schema.obiect-selectat]
use std::rc::Rc;

use gmotion_db::SursaObiecte;
use gmotion_fereastra::{
    aplica_tema, incarca_lista, incarca_previzualizare, model_lista, AntetLista, FereastraLista,
    Mediu,
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
    let mut fisier =
        gmotion_db::SursaFixture::din_fisier(&a[1]).expect("instantaneul nu se citeste");
    let server = fisier.descriere();
    let sursa = modul::pregateste(&fisier.obiecte(&a[2]).expect("baza sursa"));
    let tinta = modul::pregateste(&fisier.obiecte(&a[3]).expect("baza tinta"));
    let comparare = modul::compara(&a[2], &sursa, &[(a[3].clone(), tinta.clone())], |_| {});

    // randurile comparate -> modelul listei
    let obiecte: Vec<model_lista::Obiect> = comparare
        .randuri
        .iter()
        .map(|r| {
            let (schema, nume) = r.obiect.split_once('.').unwrap_or(("dbo", &r.obiect));
            model_lista::Obiect {
                schema: schema.to_string(),
                nume: nume.to_string(),
                tip: r.tip.clone(),
                stare: match r.categorie {
                    modul::Categorie::Diferit => model_lista::Stare::Modificat,
                    modul::Categorie::Lipsa => model_lista::Stare::DoarSursa,
                    modul::Categorie::Extra => model_lista::Stare::DoarBaza,
                    modul::Categorie::Criptat => model_lista::Stare::Criptat,
                },
                detalii: None,
                revizuit: false,
            }
        })
        .chain(
            // obiectele identice nu ajung in `randuri`, dar bara laterala le numara
            sursa
                .iter()
                .filter(|(id, _)| !comparare.randuri.iter().any(|r| &r.obiect == *id))
                .map(|(id, o)| {
                    let (schema, nume) = id.split_once('.').unwrap_or(("dbo", id));
                    model_lista::Obiect {
                        schema: schema.to_string(),
                        nume: nume.to_string(),
                        tip: o.tip.clone(),
                        stare: model_lista::Stare::Identic,
                        detalii: None,
                        revizuit: false,
                    }
                }),
        )
        .collect();

    let mut lista = model_lista::Lista::noua(obiecte);
    lista.filtreaza_stare(Some(model_lista::Stare::Modificat));

    // detaliile se calculeaza doar pentru randurile de pe ecran, ca in
    // aplicatie: aici incap ~10
    for p in lista.fara_detalii(0, 12) {
        let Some(o) = lista.obiect(p) else { continue };
        let id = o.id();
        let (Some(s), Some(t)) = (sursa.get(&id), tinta.get(&id)) else {
            continue;
        };
        let setari = modul::setari_diferite(s.setari, t.setari);
        let d = modul::detalii_obiect(&s.afisare, &t.afisare, &setari);
        lista.pune_detalii(
            p,
            model_lista::Detalii {
                difera_in: d.difera_in,
                linii_plus: d.linii_plus,
                linii_minus: d.linii_minus,
            },
        );
    }
    // un obiect revizuit, ca sa se vada bifa si progresul
    lista.comuta_revizuit(1);
    if let Some(cerut) = a.get(5) {
        if let Some(p) = (0..lista.vizibile().len()).find(|&p| {
            lista
                .obiect(p)
                .is_some_and(|o| o.id().eq_ignore_ascii_case(cerut))
        }) {
            lista.selecteaza(p);
        }
    } else {
        lista.selecteaza(0);
    }

    let (w, h) = (1440u32, 900u32);
    let fereastra = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(fereastra.clone()))).expect("platforma");
    let ui = FereastraLista::new().expect("fereastra nu se creeaza");
    aplica_tema(&ui, gmotion_fereastra::Mod::default());
    incarca_lista(
        &ui,
        &AntetLista {
            sursa_baza: &a[2],
            tinta_baza: &a[3],
            servere: &server,
            mediu: Mediu::Prod,
            in_plan: lista.obiecte().iter().filter(|o| o.stare.difera()).count(),
        },
        &lista,
    );
    if let Some(o) = lista.obiect_selectat() {
        let id = o.id();
        if let (Some(s), Some(t)) = (sursa.get(&id), tinta.get(&id)) {
            let setari = modul::setari_diferite(s.setari, t.setari);
            incarca_previzualizare(&ui, &s.afisare, &t.afisare, &setari);
        }
    }
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
