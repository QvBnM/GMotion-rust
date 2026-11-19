//! Fotografiaza drumul adevarat: lista, apoi editorul **deschis din lista**,
//! prin aceleasi comenzi pe care le trimite binarul.
//!
//! Rostul: captura editorului din `gmotion-fereastra` porneste de la un obiect
//! dat de mana. Aici obiectul vine din lista, prin `Comanda::Deschide`, deci
//! se vede ca rutarea chiar duce unde trebuie.
//!
//!   cargo run --release -p gmotion-app --example captura_flux -- \
//!       instantaneu.json SURSA BAZA iesire-lista.png iesire-editor.png [schema.obiect]
use std::rc::Rc;

use gmotion_app::rutare::{Aplicatie, Comanda, Ecran};
use gmotion_db::SursaObiecte;
use gmotion_fereastra::{
    aplica_tema, incarca_editor, incarca_lista, incarca_previzualizare, AntetEditor, AntetLista,
    EditorDiferente, FereastraLista,
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
            "folosire: captura_flux instantaneu.json SURSA BAZA lista.png editor.png [schema.obiect]"
        );
        std::process::exit(2);
    }
    let mut fisier = gmotion_db::SursaFixture::din_fisier(&a[1]).expect("instantaneul se citeste");
    let server = fisier.descriere();
    let sursa = modul::pregateste(&fisier.obiecte(&a[2]).expect("baza sursa"));
    let tinta = modul::pregateste(&fisier.obiecte(&a[3]).expect("baza tinta"));
    let comparare = modul::compara(&a[2], &sursa, &[(a[3].clone(), tinta.clone())], |_| {});

    let mut app = Aplicatie::noua(
        comparare.randuri,
        sursa,
        // numele bazei trebuie sa fie exact cel din comparare, altfel
        // randurile nu-si mai gasesc obiectele si nu se deschide nimic
        vec![(a[3].clone(), tinta)],
        modul::Revizuire::noua(),
    );
    app.asigura_detalii(0, 40);
    if let Some(cerut) = a.get(6).filter(|x| !x.starts_with("--")) {
        if let Some(p) = (0..app.lista().vizibile().len())
            .find(|&p| app.lista().obiect(p).is_some_and(|o| o.id() == *cerut))
        {
            app.comanda(Comanda::Selecteaza(p));
        }
    }
    // o bifa, ca sa se vada si progresul
    app.comanda(Comanda::ComutaRevizuit);
    app.comanda(Comanda::ComutaRevizuit);

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
    let tema = a
        .iter()
        .position(|x| x == "--tema")
        .and_then(|i| a.get(i + 1))
        .and_then(|n| gmotion_fereastra::Mod::din_nume(n))
        .unwrap_or_default();
    let lista_ui = FereastraLista::new().expect("fereastra listei");
    aplica_tema(&lista_ui, tema);
    gmotion_fereastra::aplica_limba(&lista_ui, Default::default());
    incarca_lista(
        &lista_ui,
        &AntetLista {
            sursa_baza: &a[2],
            tinta_baza: &a[3],
            servere: &server,
            in_plan: app
                .lista()
                .obiecte()
                .iter()
                .filter(|o| o.stare.difera())
                .count(),
        },
        app.lista(),
    );
    if let Some((s, t, setari)) = app.texte_obiect() {
        incarca_previzualizare(&lista_ui, s, t, &setari);
    }
    lista_ui.show().expect("se arata");
    fereastra.set_size(slint::PhysicalSize::new(w, h));
    scrie(&fereastra, w, h, &a[4]);
    lista_ui.hide().expect("se ascunde");

    // acum chiar comanda de deschidere, ca din interfata
    app.comanda(Comanda::Deschide);
    // optional, dupa obiect: ce s-a facut in editor inainte de poza
    //   --aplica      prima diferenta adusa din sursa
    //   --ghidat      modul ghidat pornit
    //   --editeaza    panoul Baza in editarea de mana
    let are = |f: &str| a.iter().any(|x| x == f);
    if are("--aplica") {
        app.comanda(Comanda::AplicaDiferenta(Some(0)));
    }
    if are("--ghidat") {
        app.comanda(Comanda::PornesteGhidat);
    }
    assert_eq!(
        app.ecran(),
        Ecran::Editor,
        "Deschide trebuia sa duca in editor"
    );
    let editor_ui = EditorDiferente::new().expect("fereastra editorului");
    aplica_tema(&editor_ui, tema);
    gmotion_fereastra::aplica_limba(&editor_ui, Default::default());
    let o = app.obiect().expect("un obiect deschis").clone();
    let model = app.model_editor().expect("modelul editorului");
    // coloana din stanga: lista stransa, ca in aplicatie
    gmotion_fereastra::incarca_coloana_lista(&editor_ui, app.lista());
    incarca_editor(
        &editor_ui,
        &AntetEditor {
            id: &format!("{}/{}.{}", o.baza, o.schema, o.nume),
            schema: &o.schema,
            nume: &o.nume,
            tip: gmotion_core::nume_tip(&o.tip),
            sursa_baza: &a[2],
            sursa_server: &server,
            tinta_baza: &a[3],
            tinta_server: &server,
            stare_obiecte: &format!(
                "obiectul {} din {} nerevizuite",
                app.lista().selectat().map_or(0, |p| p + 1),
                app.lista().vizibile().len()
            ),
            revizuit: o.revizuit,
            in_plan: false,
        },
        &model,
    );
    let (editat, _) = app.stare_text_final();
    let (text_sursa, text_final) = app.texte_curente().unwrap_or_default();
    gmotion_fereastra::incarca_editarea(
        &editor_ui,
        &gmotion_fereastra::StareEditare {
            omis: o.omis,
            editat,
            se_poate_anula: app.se_poate_anula(),
            diferenta: app.diferenta_curenta(),
            ghidat: app
                .ghidat()
                .map(|g| (g.din_sursa, g.aplicate, g.sarite, g.terminat)),
            text_sursa: &text_sursa,
            text_final: &text_final,
            salvat: if editat {
                "Modificarile se salveaza singure, ultima data la 14:52"
            } else {
                ""
            },
        },
    );
    editor_ui.set_editeaza_mod(are("--editeaza"));
    editor_ui.show().expect("se arata");
    fereastra.set_size(slint::PhysicalSize::new(w, h));
    scrie(&fereastra, w, h, &a[5]);

    println!(
        "{} -> {} ({} si {})",
        o.id(),
        if app.ecran() == Ecran::Editor {
            "editor"
        } else {
            "lista"
        },
        a[4],
        a[5]
    );
}
