//! Binarul cu interfata: `gmotion-ui`.
//!
//! **Se strica astfel:** fereastra nu porneste pe o masina fara GPU, sau
//! comenzile ajung in alta stare decat ce se vede. Prima se rezolva cu
//! randarea software a lui Slint; a doua, tinand starea in `rutare`, testata
//! fara fereastra — aici raman doar desenul si legaturile.
//!
//! Pana la ecranul de conectare (etapa urmatoare), sursa se da in linia de
//! comanda, si numai ca instantaneu:
//!
//!   gmotion-ui --sursa instantaneu.json --referinta Referinta --baza Client1

use std::cell::RefCell;
use std::rc::Rc;

use gmotion_app::rutare::{Aplicatie, Comanda, Ecran};
use gmotion_db::SursaObiecte;
use gmotion_fereastra::model_lista::{Ordine, Stare};
use gmotion_fereastra::{
    aplica_tema, goleste_previzualizarea, incarca_editor, incarca_lista, incarca_previzualizare,
    AntetEditor, AntetLista, EditorDiferente, FereastraLista, Mediu,
};
use gmotion_modul_diff_obiecte as modul;
use slint::ComponentHandle;

struct Argumente {
    sursa: String,
    referinta: String,
    baza: String,
}

fn argumente() -> Result<Argumente, String> {
    let a: Vec<String> = std::env::args().collect();
    let ia = |nume: &str| -> Option<String> {
        a.iter()
            .position(|x| x == nume)
            .and_then(|i| a.get(i + 1))
            .cloned()
    };
    match (ia("--sursa"), ia("--referinta"), ia("--baza")) {
        (Some(sursa), Some(referinta), Some(baza)) => Ok(Argumente {
            sursa,
            referinta,
            baza,
        }),
        _ => Err(
            "folosire: gmotion-ui --sursa instantaneu.json --referinta BAZA --baza BAZA\n\
                  (conectarea la server vine cu ecranul de conectare)"
                .into(),
        ),
    }
}

fn main() {
    // mesaj de om la ieșirea de erori, nu un `Debug` al tipului de eroare:
    // asta e tot ce vede cineva care porneste gresit aplicatia
    if let Err(e) = ruleaza() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn ruleaza() -> Result<(), String> {
    gmotion_platforma::jurnal::porneste();
    let arg = match argumente() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    let mut fisier = gmotion_db::SursaFixture::din_fisier(&arg.sursa).map_err(|e| e.to_string())?;
    let server = fisier.descriere();
    let sursa = modul::pregateste(&fisier.obiecte(&arg.referinta).map_err(|e| e.to_string())?);
    let tinta = modul::pregateste(&fisier.obiecte(&arg.baza).map_err(|e| e.to_string())?);
    let comparare = modul::compara(
        &arg.referinta,
        &sursa,
        &[(arg.baza.clone(), tinta.clone())],
        |_| {},
    );

    // progresul salvat, daca exista
    let sesiuni =
        gmotion_platforma::Sesiuni::pentru_modul(modul::NUME).map_err(|e| e.to_string())?;
    let sesiune = sesiuni.incarca(&arg.referinta).map_err(|e| e.to_string())?;
    let revizuire = sesiune
        .as_ref()
        .map(modul::Revizuire::din_sesiune)
        .unwrap_or_default();

    let aplicatie = Rc::new(RefCell::new(Aplicatie::noua(
        comparare.randuri,
        sursa,
        tinta,
        revizuire,
    )));
    aplicatie.borrow_mut().asigura_detalii(0, 40);

    let lista_ui = FereastraLista::new().map_err(|e| e.to_string())?;
    let editor_ui = EditorDiferente::new().map_err(|e| e.to_string())?;
    aplica_tema(&lista_ui);
    aplica_tema(&editor_ui);

    let mediu = Mediu::Prod;
    let antet = move |a: &Aplicatie| AntetLista {
        sursa_baza: "",
        tinta_baza: "",
        servere: "",
        mediu,
        in_plan: a
            .lista()
            .obiecte()
            .iter()
            .filter(|o| o.stare.difera())
            .count(),
    };

    // o singura functie de redesenat: altfel un ecran ramane in urma celuilalt
    let deseneaza = {
        let aplicatie = aplicatie.clone();
        let lista_ui = lista_ui.as_weak();
        let editor_ui = editor_ui.as_weak();
        let referinta = arg.referinta.clone();
        let baza = arg.baza.clone();
        let server = server.clone();
        move || {
            let mut a = aplicatie.borrow_mut();
            a.asigura_detalii(0, 60);
            let (Some(lista_ui), Some(editor_ui)) = (lista_ui.upgrade(), editor_ui.upgrade())
            else {
                return;
            };
            incarca_lista(
                &lista_ui,
                &AntetLista {
                    sursa_baza: &referinta,
                    tinta_baza: &baza,
                    servere: &server,
                    ..antet(&a)
                },
                a.lista(),
            );
            match a.texte_obiect() {
                Some((s, t, setari)) => incarca_previzualizare(&lista_ui, s, t, &setari),
                None => goleste_previzualizarea(&lista_ui),
            }
            if a.ecran() == Ecran::Editor {
                if let (Some(o), Some(model)) = (a.obiect(), a.model_editor()) {
                    let pozitie = format!(
                        "obiectul {} din {} nerevizuite",
                        a.lista().selectat().map_or(0, |p| p + 1),
                        a.lista().vizibile().len()
                    );
                    incarca_editor(
                        &editor_ui,
                        &AntetEditor {
                            schema: &o.schema,
                            nume: &o.nume,
                            tip: gmotion_core::nume_tip(&o.tip),
                            sursa_baza: &referinta,
                            sursa_server: &server,
                            tinta_baza: &baza,
                            tinta_server: &server,
                            mediu,
                            stare_obiecte: &pozitie,
                            revizuit: o.revizuit,
                        },
                        &model,
                    );
                }
                let _ = editor_ui.show();
                let _ = lista_ui.hide();
            } else {
                let _ = editor_ui.hide();
                let _ = lista_ui.show();
            }
        }
    };

    // fiecare comanda trece prin acelasi drum: starea, apoi desenul
    let salveaza = {
        let aplicatie = aplicatie.clone();
        let referinta = arg.referinta.clone();
        let baze = vec![arg.baza.clone()];
        let server_etichetat = server.clone();
        move || {
            let mut a = aplicatie.borrow_mut();
            let mut s = sesiune.clone().unwrap_or_else(|| {
                gmotion_platforma::Sesiune::noua(
                    server_etichetat.clone(),
                    "instantaneu",
                    referinta.clone(),
                    String::new(),
                )
            });
            a.revizuire_de_salvat().in_sesiune(&mut s);
            s.baze = baze.clone();
            if let Err(e) = sesiuni.salveaza(&s) {
                tracing::warn!(target: "gmotion::app", eroare = %e, "progresul nu s-a salvat");
            }
        }
    };
    let comanda = {
        let aplicatie = aplicatie.clone();
        let deseneaza = deseneaza.clone();
        let salveaza = Rc::new(salveaza);
        move |c: Comanda| {
            let schimbat = aplicatie.borrow_mut().comanda(c);
            if schimbat {
                salveaza();
            }
            deseneaza();
        }
    };

    // --- lista
    {
        let c = comanda.clone();
        lista_ui.on_alege_randul(move |i| c(Comanda::Selecteaza(i.max(0) as usize)));
        let c = comanda.clone();
        lista_ui.on_deschide_obiectul(move || c(Comanda::Deschide));
        let c = comanda.clone();
        lista_ui.on_comuta_revizuit(move |_| c(Comanda::ComutaRevizuit));
        let c = comanda.clone();
        lista_ui.on_cauta(move |t| c(Comanda::Cauta(t.to_string())));
        let c = comanda.clone();
        lista_ui.on_alege_categoria(move |i| {
            c(Comanda::Filtreaza(
                Stare::TOATE
                    .get(usize::try_from(i).unwrap_or(usize::MAX))
                    .copied(),
            ))
        });
        let c = comanda.clone();
        let aplicatie_tipuri = aplicatie.clone();
        lista_ui.on_alege_tipul(move |i| {
            let tip = usize::try_from(i)
                .ok()
                .and_then(|i| {
                    aplicatie_tipuri
                        .borrow()
                        .lista()
                        .contoare_tipuri()
                        .get(i)
                        .cloned()
                })
                .map(|(t, _)| t);
            c(Comanda::FiltreazaTip(tip))
        });
        let c = comanda.clone();
        let aplicatie_sortare = aplicatie.clone();
        lista_ui.on_sorteaza_dupa_linii(move || {
            let acum = aplicatie_sortare.borrow().lista().ordine();
            c(Comanda::Sorteaza(if acum == Ordine::LiniiDesc {
                Ordine::Nume
            } else {
                Ordine::LiniiDesc
            }))
        });
    }

    // --- editor
    {
        let c = comanda.clone();
        editor_ui.on_inapoi_lista(move || c(Comanda::InapoiLaLista));
        let c = comanda.clone();
        editor_ui.on_obiect_urmator(move || c(Comanda::ObiectUrmator));
        let c = comanda.clone();
        editor_ui.on_obiect_anterior(move || c(Comanda::ObiectAnterior));
        let c = comanda.clone();
        editor_ui.on_comuta_revizuit(move || c(Comanda::ComutaRevizuit));
    }

    deseneaza();
    lista_ui.run().map_err(|e| e.to_string())?;
    Ok(())
}
