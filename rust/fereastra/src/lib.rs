//! Interfata nativa GMotion.
//!
//! **Se strica astfel:** ce vede utilizatorul nu corespunde motorului (alt
//! aliniament, alte culori decat ce a decis `core`), sau fereastra nu se
//! deseneaza deloc pe un server fara GPU. Desenarea e prin Slint, cu randare
//! software (merge si prin RDP); modelele se construiesc aici, in Rust, si se
//! testeaza fara fereastra.

pub mod model_editor;

slint::include_modules!();

use model_editor::{Bucata, ModelEditor, Sintaxa, Stare};
use slint::{ModelRc, SharedString, VecModel};

fn bucati_ui(b: &[Bucata]) -> ModelRc<BucataUi> {
    ModelRc::new(VecModel::from(
        b.iter()
            .map(|x| BucataUi {
                text: SharedString::from(x.text.as_str()),
                sintaxa: match x.sintaxa {
                    Sintaxa::Text => 0,
                    Sintaxa::CuvantCheie => 1,
                    Sintaxa::Variabila => 2,
                    Sintaxa::Numar => 3,
                    Sintaxa::Sir => 4,
                    Sintaxa::Comentariu => 5,
                },
                schimbat: x.schimbat,
            })
            .collect::<Vec<_>>(),
    ))
}

fn cod_stare(s: Stare) -> i32 {
    match s {
        Stare::Egal => 0,
        Stare::Modificat => 1,
        Stare::Sters => 2,
        Stare::Adaugat => 3,
    }
}

/// Datele afisate in antetul editorului.
pub struct AntetEditor<'a> {
    pub obiect: &'a str,
    pub baza_ref: &'a str,
    pub baza_tinta: &'a str,
    pub scor: Option<f64>,
    pub pozitie_obiect: &'a str,
}

/// Umple editorul cu modelul unui obiect.
pub fn incarca_editor(ui: &EditorDiferente, antet: &AntetEditor, model: &ModelEditor) {
    let nr =
        |n: Option<usize>| n.map_or(SharedString::new(), |n| SharedString::from(n.to_string()));
    let randuri: Vec<RandUi> = model
        .randuri
        .iter()
        .map(|r| RandUi {
            st_nr: nr(r.stanga.nr),
            st: bucati_ui(&r.stanga.bucati),
            dr_nr: nr(r.dreapta.nr),
            dr: bucati_ui(&r.dreapta.bucati),
            stare: cod_stare(r.stare),
            inceput: r.inceput_bloc,
        })
        .collect();
    let total = model.randuri.len().max(1) as f32;
    let marcaje: Vec<MarcajUi> = model
        .diferente
        .iter()
        .map(|&i| MarcajUi {
            pozitie: i as f32 / total,
            stare: cod_stare(model.randuri[i].stare),
        })
        .collect();
    let (mut m, mut a, mut s) = (0, 0, 0);
    for &i in &model.diferente {
        match model.randuri[i].stare {
            Stare::Modificat => m += 1,
            Stare::Adaugat => a += 1,
            Stare::Sters => s += 1,
            Stare::Egal => {}
        }
    }
    ui.set_obiect(antet.obiect.into());
    ui.set_baza_ref(antet.baza_ref.to_uppercase().into());
    ui.set_baza_tinta(antet.baza_tinta.to_uppercase().into());
    ui.set_scor(antet.scor.map_or("—".into(), |s| format!("{s:.0}%").into()));
    ui.set_pozitie_obiect(antet.pozitie_obiect.into());
    ui.set_rezumat(format!("{m} mod, {a} add, {s} sterse").into());
    ui.set_randuri(ModelRc::new(VecModel::from(randuri)));
    ui.set_marcaje(ModelRc::new(VecModel::from(marcaje)));
    ui.set_total_diferente(model.diferente.len() as i32);
    ui.set_start_diferente(ModelRc::new(VecModel::from(
        model
            .diferente
            .iter()
            .map(|&i| i as i32)
            .collect::<Vec<_>>(),
    )));
    ui.set_diferenta_curenta(0);
}
