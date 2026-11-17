//! Binarul cu interfata: `gmotion-ui`.
//!
//! **Se strica astfel:** fereastra nu porneste pe o masina fara GPU, sau
//! comenzile ajung in alta stare decat ce se vede. Prima se rezolva cu
//! randarea software a lui Slint; a doua, tinand starea in `rutare` si
//! pornirea in `pornire`, amandoua testate fara fereastra — aici raman doar
//! desenul si legaturile.
//!
//! Pornit fara argumente, incepe cu ecranul de conectare. Pentru probe si
//! capturi se poate da sursa de la inceput:
//!
//!   gmotion-ui --sursa instantaneu.json --referinta Referinta --baza Client1

use std::cell::RefCell;
use std::rc::Rc;

use gmotion_app::pornire;
use gmotion_app::rutare::{Aplicatie, Comanda, Ecran};
use gmotion_fereastra::model_conectare::{Conectare, Recenta, StareConexiune};
use gmotion_fereastra::model_lista::{Ordine, Stare as StareObiect};
use gmotion_fereastra::{
    aplica_tema, goleste_previzualizarea, incarca_conectare, incarca_editor, incarca_lista,
    incarca_previzualizare, mediu_dupa_nume, AntetEditor, AntetLista, EditorDiferente,
    FereastraConectare, FereastraLista,
};
use gmotion_modul_diff_obiecte as modul;
use slint::ComponentHandle;

/// Ce stie aplicatia in orice moment.
struct Stare {
    conectare: Conectare,
    /// Lipseste pana la prima comparare.
    app: Option<Aplicatie>,
    sesiuni: gmotion_platforma::Sesiuni,
    /// Ce scrie pe ecran despre sursa (server sau cale de instantaneu).
    server: String,
}

impl Stare {
    /// Partea aleasa, ca sa nu se repete `if sursa { ... } else { ... }` la
    /// fiecare camp din ecranul de conectare.
    fn parte(&mut self, sursa: bool) -> &mut gmotion_fereastra::model_conectare::Parte {
        if sursa {
            &mut self.conectare.sursa
        } else {
            &mut self.conectare.baza
        }
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

fn argument(nume: &str) -> Option<String> {
    let a: Vec<String> = std::env::args().collect();
    a.iter()
        .position(|x| x == nume)
        .and_then(|i| a.get(i + 1))
        .cloned()
}

/// Compararile salvate, cele mai noi intai.
fn recente(sesiuni: &gmotion_platforma::Sesiuni) -> Vec<Recenta> {
    let mut s = sesiuni.lista().unwrap_or_default();
    s.sort_by(|a, b| b.salvata.cmp(&a.salvata));
    s.into_iter()
        .take(8)
        .map(|s| {
            let baza = s.baze.first().cloned().unwrap_or_default();
            let revizuite = modul::Revizuire::din_sesiune(&s).numar();
            Recenta {
                mediu: mediu_dupa_nume(&baza).eticheta().to_string(),
                progres: if revizuite == 0 {
                    String::new()
                } else {
                    format!("{revizuite} revizuite")
                },
                cand: s.salvata.clone(),
                sursa: s.referinta.clone(),
                baza,
            }
        })
        .collect()
}

fn ruleaza() -> Result<(), String> {
    gmotion_platforma::jurnal::porneste();
    let sesiuni =
        gmotion_platforma::Sesiuni::pentru_modul(modul::NUME).map_err(|e| e.to_string())?;

    let mut conectare = Conectare::noua();
    conectare.recente = recente(&sesiuni);
    conectare.instantaneu = argument("--sursa");
    if conectare.instantaneu.is_some() {
        if let (Some(r), Some(b)) = (argument("--referinta"), argument("--baza")) {
            conectare.sursa.baza = r;
            conectare.baza.baza = b;
        }
    }

    let stare = Rc::new(RefCell::new(Stare {
        conectare,
        app: None,
        sesiuni,
        server: String::new(),
    }));

    let conectare_ui = FereastraConectare::new().map_err(|e| e.to_string())?;
    let lista_ui = FereastraLista::new().map_err(|e| e.to_string())?;
    let editor_ui = EditorDiferente::new().map_err(|e| e.to_string())?;
    aplica_tema(&conectare_ui);
    aplica_tema(&lista_ui);
    aplica_tema(&editor_ui);

    // o singura functie de redesenat: altfel un ecran ramane in urma celuilalt
    let deseneaza = {
        let stare = stare.clone();
        let (c_ui, l_ui, e_ui) = (
            conectare_ui.as_weak(),
            lista_ui.as_weak(),
            editor_ui.as_weak(),
        );
        move || {
            let mut s = stare.borrow_mut();
            let (Some(c_ui), Some(l_ui), Some(e_ui)) =
                (c_ui.upgrade(), l_ui.upgrade(), e_ui.upgrade())
            else {
                return;
            };
            if s.app.is_none() {
                incarca_conectare(&c_ui, &s.conectare);
                let _ = l_ui.hide();
                let _ = e_ui.hide();
                let _ = c_ui.show();
                return;
            }
            let _ = c_ui.hide();
            let (referinta, baza) = (
                s.conectare.sursa.baza.clone(),
                s.conectare.baza.baza.clone(),
            );
            let mediu = mediu_dupa_nume(&baza);
            let server = s.server.clone();
            let app = s.app.as_mut().expect("verificat mai sus");
            app.asigura_detalii(0, 60);
            incarca_lista(
                &l_ui,
                &AntetLista {
                    sursa_baza: &referinta,
                    tinta_baza: &baza,
                    servere: &server,
                    mediu,
                    in_plan: app
                        .lista()
                        .obiecte()
                        .iter()
                        .filter(|o| o.stare.difera())
                        .count(),
                },
                app.lista(),
            );
            match app.texte_obiect() {
                Some((a, b, setari)) => incarca_previzualizare(&l_ui, a, b, &setari),
                None => goleste_previzualizarea(&l_ui),
            }
            if app.ecran() == Ecran::Editor {
                let pozitie = format!(
                    "obiectul {} din {} nerevizuite",
                    app.lista().selectat().map_or(0, |p| p + 1),
                    app.lista().vizibile().len()
                );
                if let (Some(o), Some(model)) = (app.obiect().cloned(), app.model_editor()) {
                    incarca_editor(
                        &e_ui,
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
                let _ = l_ui.hide();
                let _ = e_ui.show();
            } else {
                let _ = e_ui.hide();
                let _ = l_ui.show();
            }
        }
    };

    let salveaza = {
        let stare = stare.clone();
        move || {
            let mut s = stare.borrow_mut();
            let referinta = s.conectare.sursa.baza.clone();
            let baze = vec![s.conectare.baza.baza.clone()];
            let server = s.server.clone();
            let auth = if s.conectare.instantaneu.is_some() {
                "instantaneu"
            } else if s.conectare.sursa.auth_windows {
                "Windows"
            } else {
                "SQL"
            };
            let Some(revizuire) = s.app.as_mut().map(|a| a.revizuire_de_salvat().clone()) else {
                return;
            };
            let mut sesiune = s
                .sesiuni
                .incarca(&referinta)
                .ok()
                .flatten()
                .unwrap_or_else(|| {
                    gmotion_platforma::Sesiune::noua(server, auth, referinta.clone(), String::new())
                });
            revizuire.in_sesiune(&mut sesiune);
            sesiune.baze = baze;
            if let Err(e) = s.sesiuni.salveaza(&sesiune) {
                tracing::warn!(target: "gmotion::app", eroare = %e, "progresul nu s-a salvat");
            }
        }
    };

    let comanda = {
        let stare = stare.clone();
        let deseneaza = deseneaza.clone();
        let salveaza = Rc::new(salveaza);
        move |c: Comanda| {
            let schimbat = stare
                .borrow_mut()
                .app
                .as_mut()
                .map(|a| a.comanda(c))
                .unwrap_or(false);
            if schimbat {
                salveaza();
            }
            deseneaza();
        }
    };

    // --- ecranul de conectare
    {
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_server_schimbat(move |sursa, text| {
            let mut st = s.borrow_mut();
            let p = st.parte(sursa);
            p.server = text.to_string();
            // serverul s-a schimbat: ce stiam despre el nu mai e valabil
            p.baze.clear();
            p.baza.clear();
            p.stare = StareConexiune::Neincercat;
            drop(st);
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_auth_schimbat(move |sursa, windows| {
            let mut st = s.borrow_mut();
            let p = st.parte(sursa);
            p.auth_windows = windows;
            p.baze.clear();
            p.stare = StareConexiune::Neincercat;
            drop(st);
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_utilizator_schimbat(move |sursa, text| {
            s.borrow_mut().parte(sursa).acreditari.utilizator = text.to_string();
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_parola_schimbata(move |sursa, text| {
            s.borrow_mut()
                .parte(sursa)
                .acreditari
                .pune_parola(text.to_string());
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_baza_schimbata(move |sursa, text| {
            s.borrow_mut().parte(sursa).baza = text.to_string();
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_cere_bazele(move |sursa| {
            let c = s.borrow().conectare.clone();
            let p = if sursa { &c.sursa } else { &c.baza };
            let (baze, stare_noua) = match p.ce_lipseste() {
                Some(ce) => (
                    Vec::new(),
                    StareConexiune::Eroare(format!("completează {ce}")),
                ),
                None => pornire::citeste_bazele(&c, p),
            };
            let mut st = s.borrow_mut();
            let tinta = st.parte(sursa);
            tinta.baze = baze;
            tinta.stare = stare_noua;
            drop(st);
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_inverseaza(move || {
            s.borrow_mut().conectare.inverseaza();
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_schimba_certificatul(move || {
            // doua stari, ca in v3.4: nevalidat (implicit) sau depozitul
            // sistemului. Calea unei autoritati se da din linia de comanda.
            let mut st = s.borrow_mut();
            st.conectare.certificat = if st.conectare.certificat.contains("nu se verifică") {
                "Certificatul serverului se verifică cu depozitul sistemului.".into()
            } else {
                "Certificatul serverului nu se verifică.".into()
            };
            drop(st);
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_deschide_instantaneu(move || {
            // fara selector de fisiere: ar fi o dependinta noua, si nu se
            // poate proba pe Linux fara ecran. Vezi STARE.md.
            let mut st = s.borrow_mut();
            if st.conectare.instantaneu.is_none() {
                st.conectare.sursa.stare = StareConexiune::Eroare(
                    "porneste cu --sursa <instantaneu.json> ca sa deschizi un instantaneu".into(),
                );
            }
            drop(st);
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_compara(move || {
            let c = s.borrow().conectare.clone();
            match pornire::compara(&c) {
                Ok(r) => {
                    let mut st = s.borrow_mut();
                    let revizuire = st
                        .sesiuni
                        .incarca(&c.sursa.baza)
                        .ok()
                        .flatten()
                        .map(|x| modul::Revizuire::din_sesiune(&x))
                        .unwrap_or_default();
                    st.server = r.server;
                    st.app = Some(Aplicatie::noua(r.randuri, r.sursa, r.tinta, revizuire));
                }
                Err(e) => {
                    s.borrow_mut().conectare.sursa.stare = StareConexiune::Eroare(e);
                }
            }
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_reia(move |i| {
            let mut st = s.borrow_mut();
            if let Some(r) = st
                .conectare
                .recente
                .get(usize::try_from(i).unwrap_or(0))
                .cloned()
            {
                st.conectare.sursa.baza = r.sursa;
                st.conectare.baza.baza = r.baza;
            }
            drop(st);
            d();
        });
    }

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
                usize::try_from(i)
                    .ok()
                    .and_then(|i| StareObiect::TOATE.get(i).copied()),
            ))
        });
        let c = comanda.clone();
        let s = stare.clone();
        lista_ui.on_alege_tipul(move |i| {
            let tip = usize::try_from(i).ok().and_then(|i| {
                s.borrow()
                    .app
                    .as_ref()
                    .and_then(|a| a.lista().contoare_tipuri().get(i).map(|(t, _)| t.clone()))
            });
            c(Comanda::FiltreazaTip(tip))
        });
        let c = comanda.clone();
        let s = stare.clone();
        lista_ui.on_sorteaza_dupa_linii(move || {
            let acum = s
                .borrow()
                .app
                .as_ref()
                .map(|a| a.lista().ordine())
                .unwrap_or_default();
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
    slint::run_event_loop().map_err(|e| e.to_string())?;
    Ok(())
}
