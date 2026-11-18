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
//!
//! Modul ferestrei (acelasi pe toate ecranele): `--tema intunecat|luminos`.

use std::cell::RefCell;
use std::rc::Rc;

use gmotion_app::pornire;
use gmotion_app::rutare::{Aplicatie, Comanda, Ecran};
use gmotion_fereastra::model_conectare::{Conectare, Recenta, StareConexiune};
use gmotion_fereastra::model_lista::{Ordine, Stare as StareObiect};
use gmotion_fereastra::{
    aplica_tema, goleste_previzualizarea, incarca_conectare, incarca_deploy, incarca_editarea,
    incarca_editor, incarca_jurnal, incarca_lista, incarca_previzualizare, mediu_dupa_nume,
    AntetEditor, AntetLista, EditorDiferente, FereastraConectare, FereastraDeploy, FereastraLista,
    Mod, StareEditare,
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
    /// Planul de deploy deschis si obiectele din care s-a facut: cand se
    /// schimba ce e „in plan", planul se reface (si garda se pierde).
    deploy: Option<(Vec<String>, gmotion_app::deploy::StareDeploy)>,
    /// Cand s-a salvat ultima data progresul (revizuirea, textele finale):
    /// „salvat la 14:52" din editor.
    salvat_la: String,
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
    // mesaj de om la iesirea de erori, nu un `Debug` al tipului de eroare:
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

/// Unde se scrie deploy-ul: in directorul de date al aplicatiei, cu
/// referinta si momentul in nume; langa binar, daca acela nu se poate afla.
fn director_deploy(referinta: &str) -> std::path::PathBuf {
    let nume = format!(
        "{}_{}",
        referinta.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        gmotion_platforma::timp::acum_compact()
    );
    gmotion_platforma::cai::director_date()
        .map(|d| d.join("deploy").join(&nume))
        .unwrap_or_else(|_| std::path::PathBuf::from(format!("deploy_{nume}")))
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
        deploy: None,
        salvat_la: String::new(),
    }));

    let conectare_ui = FereastraConectare::new().map_err(|e| e.to_string())?;
    let lista_ui = FereastraLista::new().map_err(|e| e.to_string())?;
    let editor_ui = EditorDiferente::new().map_err(|e| e.to_string())?;
    let deploy_ui = FereastraDeploy::new().map_err(|e| e.to_string())?;
    // configurarea: un fisier stricat nu opreste fereastra, dar se spune
    let cale_cfg = gmotion_platforma::cai::cale_configurare().ok();
    let cfg = match cale_cfg
        .as_ref()
        .map(gmotion_platforma::Configurare::citeste)
    {
        Some(Ok(c)) => c,
        Some(Err(e)) => {
            tracing::warn!(target: "gmotion::app", eroare = %e, "configurarea nu se citeste; folosesc implicitele");
            gmotion_platforma::Configurare::default()
        }
        None => gmotion_platforma::Configurare::default(),
    };

    // modul (intunecat / luminos), acelasi pe toate ecranele: argumentul,
    // altfel ce s-a ales ultima data cu butonul de tema, altfel implicitul
    let mod_ales = match argument("--tema") {
        Some(n) => Mod::din_nume(&n)
            .ok_or_else(|| format!("tema necunoscuta: „{n}”; alege intunecat sau luminos"))?,
        None => Mod::din_nume(&cfg.tema).unwrap_or_default(),
    };
    aplica_tema(&conectare_ui, mod_ales);
    aplica_tema(&lista_ui, mod_ales);
    aplica_tema(&editor_ui, mod_ales);
    aplica_tema(&deploy_ui, mod_ales);

    // butonul de tema: schimba modul pe toate ecranele si il tine minte
    let alege_tema = {
        let (c_ui, l_ui, e_ui, d_ui) = (
            conectare_ui.as_weak(),
            lista_ui.as_weak(),
            editor_ui.as_weak(),
            deploy_ui.as_weak(),
        );
        let cale_cfg = cale_cfg.clone();
        Rc::new(move |intunecat: bool| {
            let m = if intunecat {
                Mod::Intunecat
            } else {
                Mod::Luminos
            };
            if let Some(u) = c_ui.upgrade() {
                aplica_tema(&u, m);
            }
            if let Some(u) = l_ui.upgrade() {
                aplica_tema(&u, m);
            }
            if let Some(u) = e_ui.upgrade() {
                aplica_tema(&u, m);
            }
            if let Some(u) = d_ui.upgrade() {
                aplica_tema(&u, m);
            }
            let Some(cale) = cale_cfg.as_ref() else {
                return;
            };
            // se reciteste: altfel ar suprascrie ce s-a schimbat de mana intre timp
            match gmotion_platforma::Configurare::citeste(cale) {
                Ok(mut c) => {
                    c.tema = m.nume().to_string();
                    if let Err(e) = c.scrie(cale) {
                        tracing::warn!(target: "gmotion::app", eroare = %e, "tema nu s-a salvat");
                    }
                }
                Err(e) => {
                    tracing::warn!(target: "gmotion::app", eroare = %e, "tema nu s-a salvat: configurarea e stricata");
                }
            }
        })
    };

    // o singura functie de redesenat: altfel un ecran ramane in urma celuilalt
    let cate_per_fisier = cfg.obiecte_per_script;
    let deseneaza = {
        let stare = stare.clone();
        let (c_ui, l_ui, e_ui, d_ui) = (
            conectare_ui.as_weak(),
            lista_ui.as_weak(),
            editor_ui.as_weak(),
            deploy_ui.as_weak(),
        );
        move || {
            let mut s = stare.borrow_mut();
            let (Some(c_ui), Some(l_ui), Some(e_ui), Some(d_ui)) = (
                c_ui.upgrade(),
                l_ui.upgrade(),
                e_ui.upgrade(),
                d_ui.upgrade(),
            ) else {
                return;
            };
            if s.app.is_none() {
                incarca_conectare(&c_ui, &s.conectare);
                incarca_jurnal(&c_ui, &gmotion_platforma::jurnal::linii(0).0);
                let _ = l_ui.hide();
                let _ = e_ui.hide();
                let _ = d_ui.hide();
                let _ = c_ui.show();
                return;
            }
            // planul de deploy: se reface doar cand s-a schimbat ce e in plan
            let alese = s
                .app
                .as_ref()
                .map(|a| a.lista().in_plan())
                .unwrap_or_default();
            let facut = s.deploy.as_ref().map(|(a, _)| a.clone());
            if s.app.as_ref().is_some_and(|a| a.ecran() == Ecran::Deploy)
                && facut.as_ref() != Some(&alese)
            {
                let (r, t) = (
                    s.conectare.sursa.baza.clone(),
                    s.conectare.baza.baza.clone(),
                );
                let plan = s.app.as_ref().map(|a| {
                    (
                        a.plan(&r, &t, &gmotion_platforma::timp::acum()),
                        a.texte_finale(),
                    )
                });
                s.deploy = plan.map(|(p, texte)| {
                    (
                        alese.clone(),
                        gmotion_app::deploy::StareDeploy::nou(p).cu_texte_finale(texte),
                    )
                });
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
                    in_plan: app.lista().in_plan().len(),
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
                            in_plan: o.in_plan,
                        },
                        &model,
                    );
                    let (editat, _) = app.stare_text_final();
                    let (text_sursa, text_final) = app.texte_curente().unwrap_or_default();
                    let salvat = if s.salvat_la.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "Modificarile se salveaza singure, ultima data la {}",
                            s.salvat_la
                        )
                    };
                    let app = s.app.as_ref().expect("verificat mai sus");
                    incarca_editarea(
                        &e_ui,
                        &StareEditare {
                            omis: o.omis,
                            editat,
                            se_poate_anula: app.se_poate_anula(),
                            diferenta: app.diferenta_curenta(),
                            ghidat: app
                                .ghidat()
                                .map(|g| (g.din_sursa, g.aplicate, g.sarite, g.terminat)),
                            text_sursa: &text_sursa,
                            text_final: &text_final,
                            salvat: &salvat,
                        },
                    );
                }
                let _ = l_ui.hide();
                let _ = d_ui.hide();
                let _ = e_ui.show();
            } else if app.ecran() == Ecran::Deploy {
                let sursa = app.sursa().clone();
                if let Some((_, st)) = s.deploy.as_ref() {
                    let director = director_deploy(&referinta).display().to_string();
                    let e =
                        gmotion_app::deploy::ecran(st, &sursa, &baza, &director, cate_per_fisier);
                    incarca_deploy(&d_ui, &e, mediu);
                }
                let _ = l_ui.hide();
                let _ = e_ui.hide();
                let _ = d_ui.show();
            } else {
                let _ = e_ui.hide();
                let _ = d_ui.hide();
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
            let editari = s
                .app
                .as_mut()
                .map(|a| a.editari_de_salvat().clone())
                .unwrap_or_default();
            let mut sesiune = s
                .sesiuni
                .incarca(&referinta)
                .ok()
                .flatten()
                .unwrap_or_else(|| {
                    gmotion_platforma::Sesiune::noua(server, auth, referinta.clone(), String::new())
                });
            revizuire.in_sesiune(&mut sesiune);
            editari.in_sesiune(&mut sesiune);
            sesiune.baze = baze;
            match s.sesiuni.salveaza(&sesiune) {
                Ok(_) => {
                    // ora, nu data: „ultima data la 14:52"
                    let acum = gmotion_platforma::timp::acum();
                    s.salvat_la = acum.split(' ').nth(1).unwrap_or(&acum).to_string();
                }
                Err(e) => {
                    tracing::warn!(target: "gmotion::app", eroare = %e, "progresul nu s-a salvat");
                }
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
            let (baze, stare_noua) = match c.ce_lipseste_pentru_baze(sursa) {
                Some(ce) => (
                    Vec::new(),
                    StareConexiune::Eroare(format!("completeaza {ce}")),
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
        // cautarea serverelor in retea (SQL Browser), ca `gmotion servere`
        let s = stare.clone();
        let d = deseneaza.clone();
        let timeout = std::time::Duration::from_millis(cfg.timeout_descoperire_ms);
        conectare_ui.on_cauta_servere(move |sursa| {
            tracing::info!(target: "gmotion::app", "caut servere in retea");
            let gasite = gmotion_db::cauta_instante(timeout);
            let mut st = s.borrow_mut();
            let p = st.parte(sursa);
            match gasite {
                Ok(v) if v.is_empty() => {
                    p.servere_gasite.clear();
                    p.stare = StareConexiune::Eroare(
                        "niciun server nu a raspuns in retea; scrie-l de mana".into(),
                    );
                }
                Ok(v) => {
                    tracing::info!(target: "gmotion::app", gasite = v.len(), "servere gasite");
                    p.servere_gasite = v.into_iter().map(|i| i.server).collect();
                }
                Err(e) => {
                    p.servere_gasite.clear();
                    p.stare = StareConexiune::Eroare(e.to_string());
                }
            }
            drop(st);
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_server_ales(move |sursa, server| {
            let mut st = s.borrow_mut();
            let p = st.parte(sursa);
            p.server = server.to_string();
            p.servere_gasite.clear();
            p.baze.clear();
            p.baza.clear();
            p.stare = StareConexiune::Neincercat;
            drop(st);
            d();
        });
        let a = alege_tema.clone();
        let d = deseneaza.clone();
        conectare_ui.on_alege_tema(move |intunecat| {
            a(intunecat);
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
            st.conectare.verifica_certificatul = !st.conectare.verifica_certificatul;
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
                    let salvata = st.sesiuni.incarca(&c.sursa.baza).ok().flatten();
                    let revizuire = salvata
                        .as_ref()
                        .map(modul::Revizuire::din_sesiune)
                        .unwrap_or_default();
                    let editari = salvata
                        .as_ref()
                        .map(modul::Editari::din_sesiune)
                        .unwrap_or_default();
                    st.server = r.server;
                    st.app = Some(
                        Aplicatie::noua(r.randuri, r.sursa, r.tinta, revizuire).cu_editari(editari),
                    );
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
        gmotion_fereastra::leaga_harta(&lista_ui);
        // recompararea: aceeasi conectare, revizuirea de pana acum ramane
        let s = stare.clone();
        let d = deseneaza.clone();
        lista_ui.on_recompara(move || {
            let c = s.borrow().conectare.clone();
            tracing::info!(target: "gmotion::app", "recompar");
            match pornire::compara(&c) {
                Ok(r) => {
                    let mut st = s.borrow_mut();
                    let revizuire = st
                        .app
                        .as_mut()
                        .map(|a| a.revizuire_de_salvat().clone())
                        .unwrap_or_default();
                    let editari = st
                        .app
                        .as_ref()
                        .map(|a| a.editari().clone())
                        .unwrap_or_default();
                    st.server = r.server;
                    st.app = Some(
                        Aplicatie::noua(r.randuri, r.sursa, r.tinta, revizuire).cu_editari(editari),
                    );
                }
                Err(e) => {
                    tracing::warn!(target: "gmotion::app", eroare = %e, "recompararea a esuat");
                }
            }
            d();
        });
        let c = comanda.clone();
        let l = lista_ui.as_weak();
        lista_ui.on_clic_harta(move |grup, x, y| {
            let rand = l
                .upgrade()
                .and_then(|ui| gmotion_fereastra::rand_din_harta(&ui, grup, x, y));
            if let Some(r) = rand {
                c(Comanda::Selecteaza(r));
            }
        });
        let a = alege_tema.clone();
        let d = deseneaza.clone();
        lista_ui.on_alege_tema(move |intunecat| {
            a(intunecat);
            // harta e pictata in culorile modului: se repicteaza
            d();
        });
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

    // --- planul de deploy
    {
        let c = comanda.clone();
        lista_ui.on_adauga_in_plan(move || c(Comanda::ComutaInPlan));
        let c = comanda.clone();
        lista_ui.on_deschide_deploy(move || c(Comanda::DeschideDeploy));
        let c = comanda.clone();
        editor_ui.on_adauga_in_plan(move || c(Comanda::ComutaInPlan));
        let c = comanda.clone();
        deploy_ui.on_inapoi_lista(move || c(Comanda::InapoiLaLista));
        let c = comanda.clone();
        deploy_ui.on_scoate_din_plan(move |id| c(Comanda::ScoateDinPlan(id.to_string())));
        let a = alege_tema.clone();
        let d = deseneaza.clone();
        deploy_ui.on_alege_tema(move |intunecat| {
            a(intunecat);
            d();
        });
        // garda: se recitesc ambele baze si se compara amprentele
        let s = stare.clone();
        let d = deseneaza.clone();
        deploy_ui.on_verifica_baza(move || {
            let c = s.borrow().conectare.clone();
            tracing::info!(target: "gmotion::app", "garda: recitesc bazele");
            let recitit = pornire::compara(&c);
            let mut st = s.borrow_mut();
            if let Some((_, dep)) = st.deploy.as_mut() {
                match recitit {
                    Ok(r) => dep.aplica_verificarea(
                        r.sursa,
                        &c.baza.baza,
                        r.tinta,
                        &gmotion_platforma::timp::acum(),
                    ),
                    Err(e) => dep.verificare_esuata(e),
                }
            }
            drop(st);
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        deploy_ui.on_scrie_scripturile(move || {
            let mut st = s.borrow_mut();
            let (referinta, tinta) = (st.conectare.sursa.baza.clone(), st.conectare.baza.baza.clone());
            if let Some((_, dep)) = st.deploy.as_mut() {
                let dir = director_deploy(&referinta);
                dep.scrie(&dir, &tinta, &gmotion_platforma::timp::acum());
                if let Some(Ok(r)) = &dep.rezultat {
                    tracing::info!(target: "gmotion::app", director = %r.director.display(), fisiere = r.fisiere.len(), "deploy scris");
                }
            }
            drop(st);
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        deploy_ui.on_alege_impartirea(move |i| {
            if let Some((_, dep)) = s.borrow_mut().deploy.as_mut() {
                dep.alege_impartirea(gmotion_app::deploy::impartire_din(i, cate_per_fisier));
            }
            d();
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
        // plierea si alaturat / unificat stau in fereastra; modelul se
        // reconstruieste la redesenare, din ce arata fereastra acum
        let c = comanda.clone();
        editor_ui.on_comuta_omis(move || c(Comanda::ComutaOmis));
        let c = comanda.clone();
        editor_ui
            .on_aplica_diferenta(move |k| c(Comanda::AplicaDiferenta(usize::try_from(k).ok())));
        let c = comanda.clone();
        editor_ui.on_aplica_toate(move || c(Comanda::AplicaToate));
        let c = comanda.clone();
        editor_ui.on_reset(move || c(Comanda::Reset));
        let c = comanda.clone();
        editor_ui.on_anuleaza(move || c(Comanda::Anuleaza));
        let c = comanda.clone();
        editor_ui.on_salveaza_textul(move |t| c(Comanda::SalveazaText(t.to_string())));
        let c = comanda.clone();
        editor_ui.on_ghidat_porneste(move || c(Comanda::PornesteGhidat));
        let c = comanda.clone();
        editor_ui.on_ghidat_opreste(move || c(Comanda::OpresteGhidat));
        let c = comanda.clone();
        editor_ui.on_ghidat_alege(move |din_sursa| c(Comanda::GhidatAlege(din_sursa)));
        let c = comanda.clone();
        editor_ui.on_ghidat_aplica(move || c(Comanda::GhidatAplica));
        let c = comanda.clone();
        editor_ui.on_ghidat_sari(move || c(Comanda::GhidatSari));
        // „Re-sincronizeaza": se compara din nou textul curent (modelul se
        // reconstruieste la redesenare)
        let d = deseneaza.clone();
        editor_ui.on_resincronizeaza(move || {
            tracing::info!(target: "gmotion::app", "re-sincronizez obiectul");
            d();
        });
        editor_ui.on_editeaza_textul_final(|| {
            tracing::info!(target: "gmotion::app", "editez textul final de mana");
        });
        // n / p muta doar desenul (fara redesenare: pe un obiect mare,
        // reconstruirea modelului la fiecare tasta s-ar simti); aplicatia
        // tine minte pozitia ca s-o pastreze dupa „Aplica"
        let s = stare.clone();
        editor_ui.on_diferenta_schimbata(move |k| {
            if let Some(a) = s.borrow_mut().app.as_mut() {
                a.comanda(Comanda::MergiLaDiferenta(k.max(0) as usize));
            }
        });
        let d = deseneaza.clone();
        editor_ui.on_comuta_plierea(move |_| d());
        let d = deseneaza.clone();
        editor_ui.on_comuta_alaturat(move |_| d());
        let a = alege_tema.clone();
        let d = deseneaza.clone();
        editor_ui.on_alege_tema(move |intunecat| {
            a(intunecat);
            // hasura e pictata in culorile modului
            d();
        });
    }

    deseneaza();
    slint::run_event_loop().map_err(|e| e.to_string())?;
    Ok(())
}
