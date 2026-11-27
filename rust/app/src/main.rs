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
use std::time::Instant;

use gmotion_app::pornire;
use gmotion_app::rutare::{Comanda, Ecran};
use gmotion_app::stare::{self, Progres, Stare};
use gmotion_fereastra::dublu_clic::DubluClic;
use gmotion_fereastra::model_conectare::Conectare;
use gmotion_fereastra::model_lista::{FiltruRevizuire, Stare as StareObiect};
use gmotion_fereastra::{
    aplica_tema, goleste_previzualizarea, incarca_conectare, incarca_deploy, incarca_editarea,
    incarca_editor, incarca_jurnal, incarca_lista, incarca_previzualizare, AntetEditor, AntetLista,
    Aplicatie, AreAplicatie, EditorDiferente, FereastraConectare, FereastraDeploy, FereastraLista,
    Mod, StareEditare,
};
use gmotion_modul_diff_obiecte as modul;
use slint::ComponentHandle;

/// Ce scrie `gmotion-ui --help`. Pana la auditul 2026-11-23 binarul n-avea
/// `--help` si inghitea in tacere orice argument scris gresit: un `--tena`
/// pornea fereastra ca si cum nimic nu s-ar fi intamplat (C-S7).
const AJUTOR_CLI: &str = "\
gmotion-ui — comparare si deploy de obiecte SQL Server, cu fereastra.

Folosire:
  gmotion-ui [optiuni]

Optiuni:
  --sursa <CALE>        porneste de la un instantaneu `.json`, fara server
  --referinta <BAZA>    baza martor, cu --sursa si --baza: compara direct
  --baza <BAZA>         baza modificata, cu --sursa si --referinta
  --previzualizare      instantaneul sintetic din instalare, fara server
  --tema <intunecat|luminos>
  --limba <ro|en>
  --fps                 monitorul de cadre (cadre/s, cel mai lung cadru);
                        la fel cu variabila GMOTION_FPS=1
  --gpu                 desenarea pe placa video (OpenGL), in locul celei
                        software; la fel cu GMOTION_GPU=1. Daca nu porneste
                        (RDP, driver vechi), ramane cea software
  -h, --help            textul asta

Fara optiuni porneste ecranul de conectare. Tema si limba se tin minte
intre porniri; optiunea le schimba doar pentru rularea asta.
Tot ce face fereastra se poate face si din `gmotion` (linia de comanda).
";

/// Optiunile pe care le stie binarul. Un argument care nu e aici si nu e
/// valoarea uneia dintre ele e o greseala de scriere, si se spune.
const OPTIUNI: [&str; 9] = [
    "--fps",
    "--gpu",
    "--sursa",
    "--referinta",
    "--baza",
    "--previzualizare",
    "--tema",
    "--limba",
    "--ajutor",
];

/// Argumentele care nu sunt recunoscute. Valoarea unei optiuni cu valoare nu
/// se numara: vine imediat dupa ea.
fn argumente_necunoscute() -> Vec<String> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let cu_valoare = ["--sursa", "--referinta", "--baza", "--tema", "--limba"];
    let mut rele = Vec::new();
    let mut i = 0;
    while i < a.len() {
        let x = &a[i];
        if cu_valoare.contains(&x.as_str()) {
            i += 2;
            continue;
        }
        if !OPTIUNI.contains(&x.as_str()) && x != "-h" && x != "--help" {
            rele.push(x.clone());
        }
        i += 1;
    }
    rele
}

/// Monitorul de cadre (`gmotion_app::cadre`): momentele vin din winit, prin
/// filtrul de evenimente al ferestrei — fiecare eveniment (lucrul lui) si
/// fiecare `RedrawRequested` (desenarea). Nu provoaca nicio desenare: in
/// repaus nu e niciun cadru si nu se scrie nimic. O data pe secunda, daca s-a
/// desenat ceva, textul ajunge in fereastra si in jurnal (`gmotion::cadre`).
/// Intoarce ceasul, care trebuie tinut in viata.
fn porneste_monitorul_cadre(
    aplicatie: &Aplicatie,
    stare: &Rc<RefCell<Stare>>,
) -> Option<slint::Timer> {
    use slint::winit_030::{winit, EventResult, WinitWindowAccessor};
    let argumente: Vec<String> = std::env::args().collect();
    let variabila = std::env::var("GMOTION_FPS").ok();
    if !gmotion_app::cadre::cerut(&argumente, variabila.as_deref()) {
        return None;
    }
    // pe firul ferestrei; un `thread_local`, ca apelul pus in coada buclei
    // (`invoke_from_event_loop` cere `Send`) sa nu care nimic cu el
    thread_local! {
        static STATISTICA: RefCell<gmotion_app::cadre::StatisticaCadre> =
            RefCell::new(gmotion_app::cadre::StatisticaCadre::default());
    }
    // Sfarsitul unui eveniment / al desenarii: primul dintre urmatorul
    // eveniment al ferestrei si un apel pus in coada buclei (care ruleaza
    // dupa ce winit a terminat ce avea in mana, inainte de orice asteptare).
    let sfarsit = || {
        STATISTICA.with_borrow_mut(|st| {
            let acum = std::time::Instant::now();
            st.eveniment_sfarsit(acum);
            st.dupa(acum);
        })
    };
    aplicatie.window().on_winit_window_event(move |_, ev| {
        STATISTICA.with_borrow_mut(|st| {
            let acum = std::time::Instant::now();
            st.eveniment_sfarsit(acum);
            st.dupa(acum);
            if matches!(ev, winit::event::WindowEvent::RedrawRequested) {
                st.inainte(acum);
            } else {
                st.eveniment_inceput(acum);
            }
        });
        let _ = slint::invoke_from_event_loop(sfarsit);
        EventResult::Propagate
    });
    aplicatie
        .global::<gmotion_fereastra::Cadre>()
        .set_arata(true);
    let a = aplicatie.as_weak();
    let stare = stare.clone();
    let ceas = slint::Timer::default();
    ceas.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(1),
        move || {
            let Some(r) = STATISTICA.with_borrow_mut(|st| st.incheie(std::time::Instant::now())) else {
                return;
            };
            let limba = stare.borrow().conectare.limba;
            tracing::info!(target: "gmotion::cadre", cadre = r.cadre, p99_ms = format!("{:.1}", r.p99_ms), peste_16_7 = r.peste_prag, cel_mai_lung_ms = format!("{:.1}", r.cel_mai_lung_ms), mediu_ms = format!("{:.1}", r.mediu_ms), pe_secunda = format!("{:.0}", r.pe_secunda), "cadre");
            if let Some(a) = a.upgrade() {
                let g = a.global::<gmotion_fereastra::Cadre>();
                let text: slint::SharedString = r.text(limba).into();
                if g.get_text() != text || g.get_lent() != r.lent() {
                    // textul nou provoaca un cadru: al monitorului, nu se numara
                    STATISTICA.with_borrow_mut(|st| st.ignora_urmatorul());
                    g.set_text(text);
                    g.set_lent(r.lent());
                }
            }
        },
    );
    Some(ceas)
}

fn main() {
    if std::env::args().any(|a| a == "--help" || a == "-h" || a == "--ajutor") {
        print!("{AJUTOR_CLI}");
        return;
    }
    let rele = argumente_necunoscute();
    if !rele.is_empty() {
        eprintln!("Nu cunosc: {}\n", rele.join(", "));
        eprint!("{AJUTOR_CLI}");
        std::process::exit(2);
    }
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

/// Arata fereastra ceruta si le ascunde pe celelalte — **in ordinea asta**.
///
/// Slint inchide bucla de evenimente cand nu mai e nicio fereastra vizibila.
/// „Ascunde tot, apoi arata" lasa, pentru o clipa, zero ferestre: aplicatia
/// se inchide singura exact cand treci de pe conectare pe lista. De-aia
/// trecerea dintre ecrane are un singur loc, aici.
/// O actiune a utilizatorului (buton, camp, tasta legata), in jurnalul de
/// sesiune: ecranul, numele callback-ului si argumentele (taiate la 200 de
/// caractere; parola nu se scrie niciodata). Cele foarte dese (mouse peste
/// harta) merg la `trace`, ca sa nu ingroape restul.
/// Ultima actiune a utilizatorului, pentru ceasul de garda: daca interfata
/// ingheata, jurnalul spune dupa ce.
static ULTIMA_ACTIUNE: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
/// Cand a batut ultima data ceasul firului de interfata (ms de la pornire).
static BATAIE_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Ceasul de garda: un fir care se uita daca firul interfetei mai bate. Pe
/// Windows o fereastra care nu raspunde ~5 s e omorata de sistem
/// (-805306369, 2026-11-25) si jurnalul nu spunea nimic; acum spune cat a
/// stat si dupa ce actiune.
fn porneste_garda(pornire: Instant) {
    use std::sync::atomic::Ordering;
    let _ = std::thread::Builder::new()
        .name("garda-interfata".into())
        .spawn(move || {
            let mut raportat = false;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let acum = pornire.elapsed().as_millis() as u64;
                let ultima = BATAIE_MS.load(Ordering::Relaxed);
                let stat = acum.saturating_sub(ultima);
                if stat > PRAG_INGHETARE_MS && !raportat {
                    let dupa = ULTIMA_ACTIUNE
                        .lock()
                        .map(|a| a.clone())
                        .unwrap_or_default();
                    tracing::error!(target: "gmotion::app", ms = stat, dupa = %dupa, "interfata nu raspunde");
                    raportat = true;
                } else if stat <= PRAG_INGHETARE_MS && raportat {
                    tracing::warn!(target: "gmotion::app", "interfata raspunde din nou");
                    raportat = false;
                }
            }
        });
}

/// De la cat timp fara bataie se considera ca interfata a inghetat.
const PRAG_INGHETARE_MS: u64 = 2000;

fn actiune(ecran: &str, nume: &str, argumente: &str, des: bool) {
    if let Ok(mut u) = ULTIMA_ACTIUNE.lock() {
        *u = format!("{ecran}.{nume}");
    }
    let a: String = argumente.chars().take(200).collect();
    if des {
        tracing::trace!(target: "gmotion::actiuni", ecran, nume, argumente = %a, "actiune");
    } else {
        tracing::info!(target: "gmotion::actiuni", ecran, nume, argumente = %a, "actiune");
    }
}

/// Pune fereastra pe monitorul cursorului, centrata, sau unde a fost lasata
/// data trecuta daca monitorul acela mai exista. Inainte de `show`: Slint o
/// creeaza direct acolo, fara sa sara.
fn pune_geometria(a: &Aplicatie, cfg: &gmotion_platforma::Configurare) {
    use gmotion_app::monitor::{alege_geometria, citeste_sistemul, Dreptunghi};
    let salvata = match cfg.fereastra.as_slice() {
        [x, y, l, i] if *l > 0 && *i > 0 => Some(Dreptunghi {
            x: *x,
            y: *y,
            latime: *l as u32,
            inaltime: *i as u32,
        }),
        _ => None,
    };
    let (monitoare, cursor) = citeste_sistemul();
    let ales = alege_geometria(salvata, &monitoare, cursor);
    tracing::debug!(target: "gmotion::app", monitoare = monitoare.len(), reluata = salvata.is_some(), geometrie = ?ales, "pozitia ferestrei la pornire");
    if let Some(g) = ales {
        a.window()
            .set_size(slint::PhysicalSize::new(g.latime, g.inaltime));
        a.window()
            .set_position(slint::PhysicalPosition::new(g.x, g.y));
    }
    if cfg.fereastra_maximizata {
        a.window().set_maximized(true);
    }
}

/// Cat dureaza trecerea de la un ecran la altul: destul cat sa se vada ca
/// ecranul se schimba pe loc, nu atat cat sa astepti dupa el.
const DURATA_TRANZITIE_MS: i64 = 180;

/// Face vizibil ecranul `ecran` din fereastra unica. Fereastra nu se mai
/// inchide si nu se mai redeschide: ramane pe acelasi monitor, in aceeasi
/// pozitie, iar ecranele trec unul in altul (opacitate, `durata-tranzitie`).
/// Pana la 2026-11-25 fiecare ecran era o fereastra separata, si „Deschide"
/// o muta unde voia sistemul.
fn arata(a: &Aplicatie, ecran: i32) {
    if a.get_ecran() != ecran {
        tracing::debug!(target: "gmotion::app", din = a.get_ecran(), in_ = ecran, "schimb ecranul");
        a.set_ecran(ecran);
    }
    if !a.window().is_visible() {
        let _ = a.show();
    }
}

fn ruleaza() -> Result<(), String> {
    // jurnalul de sesiune, si in fisier: fiecare actiune, cu durata ei, ca o
    // proba de mana sa se poata trimite si citi dupa
    let cale_jurnal = gmotion_platforma::cai::director_jurnale()
        .ok()
        .and_then(|d| gmotion_platforma::jurnal::porneste_cu_fisier(&d));
    match &cale_jurnal {
        Some(cale) => {
            tracing::info!(target: "gmotion::app", fisier = %cale.display(), versiune = env!("CARGO_PKG_VERSION"), "jurnalul sesiunii se scrie in fisier")
        }
        None => gmotion_platforma::jurnal::porneste(),
    }
    let director_date = gmotion_platforma::cai::director_date().ok();
    // o cadere ajunge si ea in jurnal, cu locul ei, nu doar pe consola
    let vechi = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(target: "gmotion::app", cadere = %info, "aplicatia a cazut");
        vechi(info);
    }));
    let sesiuni =
        gmotion_platforma::Sesiuni::pentru_modul(modul::NUME).map_err(|e| e.to_string())?;

    let previzualizare = std::env::args().any(|a| a == "--previzualizare");
    let mut conectare = Conectare::noua();
    conectare.recente = stare::recente(&sesiuni, conectare.limba);
    conectare.instantaneu = argument("--sursa");
    if previzualizare && conectare.instantaneu.is_none() {
        match stare::cale_previzualizare() {
            Some(c) => conectare.instantaneu = Some(c.display().to_string()),
            None => {
                return Err(
                    "nu gasesc previzualizare.json; ruleaza unelte/genereaza-previzualizare.py"
                        .to_string(),
                )
            }
        }
    }
    if conectare.instantaneu.is_some() {
        if let (Some(r), Some(b)) = (argument("--referinta"), argument("--baza")) {
            conectare.sursa.baza = r;
            conectare.baza.baza = b;
        }
    }

    let stare = Rc::new(RefCell::new(Stare::noua(conectare, sesiuni)));

    // o singura fereastra; ecranele sunt in ea (vezi `arata`)
    // `--gpu`: desenarea pe placa video (etapa 33). Implicit ramane cea
    // software, care merge si pe RDP fara placa video (decizia din JURNAL);
    // daca OpenGL nu porneste, tot ea.
    {
        let argumente: Vec<String> = std::env::args().collect();
        let variabila = std::env::var("GMOTION_GPU").ok();
        if gmotion_app::cadre::gpu_cerut(&argumente, variabila.as_deref()) {
            match slint::BackendSelector::new()
                .backend_name("winit".into())
                .renderer_name("femtovg".into())
                .select()
            {
                Ok(()) => {
                    tracing::info!(target: "gmotion::app", randare = "femtovg", "desenez pe placa video")
                }
                Err(e) => {
                    tracing::warn!(target: "gmotion::app", eroare = %e, "placa video nu porneste; desenez software")
                }
            }
        }
    }
    let aplicatie = Aplicatie::new().map_err(|e| e.to_string())?;
    let conectare_ui = FereastraConectare::din(&aplicatie);
    gmotion_fereastra::leaga_scara(&conectare_ui);
    let lista_ui = FereastraLista::din(&aplicatie);
    let editor_ui = EditorDiferente::din(&aplicatie);
    let deploy_ui = FereastraDeploy::din(&aplicatie);
    // monitorul de cadre: ascuns implicit, `--fps` sau GMOTION_FPS=1
    let _ceas_cadre = porneste_monitorul_cadre(&aplicatie, &stare);
    // configurarea: un fisier stricat nu opreste fereastra, dar se spune
    let cale_cfg = gmotion_platforma::cai::cale_configurare().ok();
    let cfg = stare::citeste_configurarea(cale_cfg.as_deref());
    // setarile din configurare care schimba comportamentul (F-B5)
    stare.borrow_mut().conectare.timeout_conectare_s = Some(cfg.timeout_conectare_s);
    stare.borrow_mut().uneste_implicit = cfg.uneste_cursor;
    // plierea liniilor identice: ce s-a ales ultima data (implicit, tot codul)
    editor_ui.set_pliaza_identicul(cfg.pliaza_identicul);
    pune_geometria(&aplicatie, &cfg);
    // la inchidere se tine minte unde a ramas fereastra
    {
        let a = aplicatie.as_weak();
        let cale_cfg = cale_cfg.clone();
        aplicatie.window().on_close_requested(move || {
            if let (Some(a), Some(cale)) = (a.upgrade(), cale_cfg.as_ref()) {
                let w = a.window();
                let (p, m) = (w.position(), w.size());
                let maximizata = w.is_maximized();
                tracing::info!(target: "gmotion::app", x = p.x, y = p.y, latime = m.width, inaltime = m.height, maximizata, "inchid; tin minte locul ferestrei");
                stare::salveaza_in_configurare(cale, "fereastra", |c| {
                    if !maximizata {
                        c.fereastra = vec![p.x, p.y, m.width as i32, m.height as i32];
                    }
                    c.fereastra_maximizata = maximizata;
                });
            }
            slint::CloseRequestResponse::HideWindow
        });
    }

    // modul (intunecat / luminos), acelasi pe toate ecranele: argumentul,
    // altfel ce s-a ales ultima data cu butonul de tema, altfel implicitul
    let mod_ales = match argument("--tema") {
        Some(n) => Mod::din_nume(&n)
            .ok_or_else(|| format!("tema necunoscuta: „{n}”; alege intunecat sau luminos"))?,
        None => Mod::din_nume(&cfg.tema).unwrap_or_default(),
    };
    // limba: din `--limba`, altfel din configurare, altfel implicitul
    let limba_aleasa = match argument("--limba") {
        Some(n) => gmotion_fereastra::texte::Limba::din_nume(&n)
            .ok_or_else(|| format!("limba necunoscuta: {n}; alege ro sau en"))?,
        None => gmotion_fereastra::texte::Limba::din_nume(&cfg.limba).unwrap_or_default(),
    };
    let pune_limba = |l: gmotion_fereastra::texte::Limba| {
        use gmotion_fereastra::texte::Limba;
        let etichete: Vec<slint::SharedString> = Limba::toate()
            .iter()
            .map(|x| slint::SharedString::from(x.eticheta()))
            .collect();
        let indice = Limba::toate().iter().position(|x| *x == l).unwrap_or(0) as i32;
        macro_rules! pe_fiecare {
            ($($ui:expr),*) => {$({
                gmotion_fereastra::aplica_limba(&$ui, l);
                $ui.set_limbi(slint::ModelRc::new(slint::VecModel::from(etichete.clone())));
                $ui.set_limba_aleasa(indice);
            })*};
        }
        pe_fiecare!(conectare_ui, lista_ui, editor_ui, deploy_ui);
    };
    pune_limba(limba_aleasa);

    aplica_tema(&conectare_ui, mod_ales);
    aplica_tema(&lista_ui, mod_ales);
    aplica_tema(&editor_ui, mod_ales);
    aplica_tema(&deploy_ui, mod_ales);

    // butonul de tema: schimba modul pe toate ecranele si il tine minte
    // comutatorul de limba: schimba textele pe toate ecranele si le tine minte
    let alege_limba = {
        let (c_ui, l_ui, e_ui, d_ui) = (
            conectare_ui.as_weak(),
            lista_ui.as_weak(),
            editor_ui.as_weak(),
            deploy_ui.as_weak(),
        );
        let cale_cfg = cale_cfg.clone();
        Rc::new(move |i: i32| {
            use gmotion_fereastra::texte::Limba;
            let Some(l) = usize::try_from(i)
                .ok()
                .and_then(|i| Limba::toate().get(i).copied())
            else {
                return;
            };
            macro_rules! pe_fiecare {
                ($($ui:expr),*) => {$(
                    if let Some(u) = $ui.upgrade() {
                        gmotion_fereastra::aplica_limba(&u, l);
                        u.set_limba_aleasa(i);
                    }
                )*};
            }
            pe_fiecare!(c_ui, l_ui, e_ui, d_ui);
            if let Some(cale) = cale_cfg.as_ref() {
                stare::salveaza_in_configurare(cale, "limba", |c| c.limba = l.nume().to_string());
            }
        })
    };

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
            macro_rules! pe_fiecare {
                ($($ui:expr),*) => {$(
                    if let Some(u) = $ui.upgrade() {
                        aplica_tema(&u, m);
                    }
                )*};
            }
            pe_fiecare!(c_ui, l_ui, e_ui, d_ui);
            if let Some(cale) = cale_cfg.as_ref() {
                stare::salveaza_in_configurare(cale, "tema", |c| c.tema = m.nume().to_string());
            }
        })
    };

    // o singura functie de redesenat: altfel un ecran ramane in urma celuilalt
    // N din „Cate N obiecte per fisier": din configurare, schimbabil din ecran
    let cate_per_fisier = Rc::new(std::cell::Cell::new(cfg.obiecte_per_script.max(1)));
    let deseneaza_brut = {
        let limba_textelor = std::cell::Cell::new(None::<gmotion_core::Limba>);
        let progres_aratat = std::cell::Cell::new(false);
        let stare = stare.clone();
        let cate_per_fisier = cate_per_fisier.clone();
        let (c_ui, l_ui, e_ui, d_ui) = (
            conectare_ui.as_weak(),
            lista_ui.as_weak(),
            editor_ui.as_weak(),
            deploy_ui.as_weak(),
        );
        move || {
            // o redesenare cheama uneori alta din interiorul ei: un `changed`
            // din `.slint` (latimea de wrap) se declanseaza cand Rust citeste
            // proprietatea. Cu `borrow_mut`, aplicatia cadea („RefCell already
            // borrowed", proba pe Windows 2026-11-19); acum a doua se renunta:
            // cea in curs deseneaza oricum starea de acum. Singurul drum
            // cunoscut (wrap-ul) e amanat cu un Timer, deci nu se pierde nimic.
            let Ok(mut s) = stare.try_borrow_mut() else {
                tracing::debug!(target: "gmotion::app", "redesenare ceruta din alta redesenare; sarita");
                return;
            };
            let (Some(c_ui), Some(l_ui), Some(e_ui), Some(d_ui)) = (
                c_ui.upgrade(),
                l_ui.upgrade(),
                e_ui.upgrade(),
                d_ui.upgrade(),
            ) else {
                return;
            };
            // limba aleasa ajunge si in textele facute in Rust (conectare,
            // ajutor); comutatorul cheama redesenarea dupa ce o schimba
            let limba = gmotion_fereastra::limba_curenta(&c_ui);
            s.conectare.limba = limba;
            if let Some(c) = s.coloana_noua.take() {
                e_ui.set_latime_lista(c.latime);
                e_ui.set_latime_lista_aleasa(c.latime);
                e_ui.set_lista_fixata(c.fixata);
                e_ui.set_lista_dreapta(c.dreapta);
                e_ui.set_lista_banda(c.banda);
            }
            // ajutorul si „Despre" sunt texte lungi: doar cand se schimba limba
            // (pana pe 2026-11-25 se refaceau la fiecare redesenare)
            if limba_textelor.get() != Some(limba) {
                limba_textelor.set(Some(limba));
                c_ui.set_text_ajutor(gmotion_fereastra::ajutor(limba).into());
                l_ui.set_text_ajutor(gmotion_fereastra::ajutor(limba).into());
                let despre = gmotion_fereastra::text_despre(
                    limba,
                    env!("CARGO_PKG_VERSION"),
                    cale_jurnal.as_deref(),
                    director_date.as_deref(),
                );
                c_ui.set_text_despre(despre.as_str().into());
                l_ui.set_text_despre(despre.into());
            }
            if s.app.is_none() {
                // Cat ruleaza compararea, formularul e blocat si nu se schimba:
                // bataia ceasului (8 pe secunda) pune doar progresul. Pana pe
                // 2026-11-25 refacea tot formularul (listele de baze, jurnalul)
                // si dadea cadre de ~26 ms cat dura compararea.
                let doar_progres = s.rulare.is_some() && progres_aratat.get();
                progres_aratat.set(s.rulare.is_some());
                if !doar_progres {
                    arata(c_ui.aplicatie(), FereastraConectare::ECRAN);
                    // „Aceeasi conexiune": ce s-a schimbat la martor ajunge si la
                    // modificata, inainte sa se deseneze
                    s.conectare.sincronizeaza();
                    incarca_conectare(&c_ui, &s.conectare);
                }
                // progresul compararii, cat ruleaza pe alt fir
                match s.rulare.as_ref() {
                    Some(r) => {
                        c_ui.set_ruleaza(true);
                        c_ui.set_progres(r.fractie());
                        c_ui.set_progres_text(r.text_in(limba).into());
                        c_ui.set_progres_timp(r.timp().into());
                        gmotion_fereastra::pune_etape(
                            &c_ui,
                            r.etape(limba)
                                .into_iter()
                                .map(|(nume, durata, stare)| gmotion_fereastra::EtapaUi {
                                    nume: nume.into(),
                                    durata: durata.into(),
                                    stare,
                                })
                                .collect(),
                        );
                    }
                    None => c_ui.set_ruleaza(false),
                }
                incarca_jurnal(&c_ui, &gmotion_platforma::jurnal::linii(0).0);
                return;
            }
            // planul de deploy: se reface cand nu mai e al starii de acum
            {
                // campuri separate, ca sa se poata citi `app` si scrie `deploy`
                let Stare {
                    app,
                    deploy,
                    referinta,
                    ..
                } = &mut *s;
                if let Some(app) = app.as_ref().filter(|a| a.ecran() == Ecran::Deploy) {
                    let acum = gmotion_platforma::timp::acum();
                    gmotion_app::deploy::asigura_planul(deploy, app, referinta, &acum, || {
                        stare::director_deploy(referinta)
                    });
                }
            }
            // referinta cu care a rulat compararea, nu ce scrie acum in
            // formular: formularul se poate schimba intre timp (F-A3)
            let referinta = s.referinta.clone();
            let baze = s.baze();
            let bara = s.text_bara_lista(limba);
            let verificare = s.text_verificare(limba);
            let progres = s.progres_recomparare(limba);
            // antetul arata o singura tinta: cu mai multe, cate sunt
            let baza = match baze.as_slice() {
                [una] => una.clone(),
                multe => format!(
                    "{} {}",
                    multe.len(),
                    s.conectare.limba.alege("baze", "databases")
                ),
            };
            let server = s.server.clone();
            let app = s.app.as_mut().expect("verificat mai sus");
            // timpii pe bucati (debug): o redesenare lenta spune unde a stat
            let mut bucati = gmotion_app::cadre::Bucati::porneste();
            // restul vin de pe firul de detalii; obiectul selectat le are pe loc
            if let Some(p) = app.lista().selectat() {
                app.asigura_detalii(p, p + 1);
            }
            // ecranul listei (randurile, previzualizarea, panoul de detalii)
            // nu se vede din editor: acolo se refacea degeaba la fiecare clic
            // (2-8 ms aici, etapa 33). Iesirea din editor e tot o comanda,
            // deci il reface atunci, la zi.
            let lista_pe_ecran = app.ecran() != Ecran::Editor;
            if lista_pe_ecran {
                incarca_lista(
                    &l_ui,
                    &AntetLista {
                        sursa_baza: &referinta,
                        tinta_baza: &baza,
                        servere: &server,
                        in_plan: app.lista().in_plan().len(),
                    },
                    app.lista(),
                );
            }
            bucati.marcheaza("lista");
            // o data pe mod: dupa prima decodare e doar o copiere de handle
            if let Err(e) = gmotion_fereastra::incarca_fundal(&l_ui) {
                tracing::warn!(target: "gmotion::app", eroare = %e, "fundalul cu unde nu se decodeaza");
            }
            e_ui.set_fundal_decor(l_ui.get_fundal_decor());
            bucati.marcheaza("fundal");
            l_ui.set_mesaj_csv(bara.into());
            l_ui.set_recompara_ruleaza(progres.is_some());
            if let Some((text, fractie)) = progres {
                l_ui.set_recompara_text(text.into());
                l_ui.set_recompara_fractie(fractie);
            }
            if lista_pe_ecran {
                match app.texte_obiect() {
                    Some((a, b, setari)) => incarca_previzualizare(&l_ui, a, b, &setari),
                    None => goleste_previzualizarea(&l_ui),
                }
                let mut panou = app.panou_detalii();
                gmotion_fereastra::incarca_detalii(&l_ui, &panou.pentru_fereastra(limba));
            }
            bucati.marcheaza("previzualizare_si_panou");
            if app.ecran() == Ecran::Editor {
                let pozitie = s.text_pozitie(limba);
                let salvat = s.text_salvat(limba);
                let app = s.app.as_mut().expect("verificat mai sus");
                if let (Some(o), Some(model)) = (app.obiect().cloned(), app.model_editor()) {
                    // dupa `model_editor`: o reconstruire tocmai s-a numarat
                    let refaceri = app.refaceri_model();
                    incarca_editor(
                        &e_ui,
                        &AntetEditor {
                            id: &format!("{}/{}.{}", o.baza, o.schema, o.nume),
                            schema: &o.schema,
                            nume: &o.nume,
                            tip: gmotion_core::nume_tip_in(&o.tip, limba),
                            sursa_baza: &referinta,
                            sursa_server: &server,
                            tinta_baza: &o.baza,
                            tinta_server: &server,
                            stare_obiecte: &pozitie,
                            revizuit: o.revizuit,
                            in_plan: o.in_plan,
                            generatie: Some(refaceri),
                        },
                        &model,
                    );
                    match &verificare {
                        Some((text, bine)) => {
                            e_ui.set_verificare_text(text.as_str().into());
                            e_ui.set_verificare_ok(*bine);
                        }
                        None => e_ui.set_verificare_text(Default::default()),
                    }
                    // blocul lasat de mana in alt loc decat cel sugerat
                    e_ui.set_plasare_text(
                        app.plasat_altundeva()
                            .map(|b| gmotion_fereastra::text_plasat_altundeva(b, limba))
                            .unwrap_or_default()
                            .into(),
                    );
                    bucati.marcheaza("editor");
                    gmotion_fereastra::incarca_coloana_lista(&e_ui, app.lista(), app.arbore());
                    bucati.marcheaza("coloana");
                    gmotion_fereastra::incarca_taburi(&e_ui, app.taburi(), |c| {
                        app.are_text_final(c)
                    });
                    e_ui.set_are_cursor(app.are_cursor());
                    e_ui.set_uneste_cursorul(app.uneste_cursorul());
                    let (editat, _) = app.stare_text_final();
                    let (text_sursa, text_final) = app.texte_curente().unwrap_or_default();
                    drop(model);
                    let probleme = app.probleme_text_final();
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
                            probleme: &probleme,
                        },
                    );
                    bucati.marcheaza("editare");
                }
                arata(e_ui.aplicatie(), EditorDiferente::ECRAN);
                bucati.scrie();
            } else if app.ecran() == Ecran::Deploy {
                let sursa = app.sursa().clone();
                // dupa o scriere reusita, directorul scris; inainte, cel in
                // care se va scrie. Pana la audit se arata mereu al doilea,
                // deci „Deschide folderul" ducea in alta parte.
                let director = s
                    .director_scris()
                    .unwrap_or_else(|| s.director_deploy_curent())
                    .display()
                    .to_string();
                if let Some(st) = s.deploy.as_ref().map(|d| &d.stare) {
                    let e = gmotion_app::deploy::ecran(
                        st,
                        &sursa,
                        &director,
                        cate_per_fisier.get(),
                        limba,
                    );
                    incarca_deploy(&d_ui, &e);
                }
                arata(d_ui.aplicatie(), FereastraDeploy::ECRAN);
            } else {
                arata(l_ui.aplicatie(), FereastraLista::ECRAN);
            }
        }
    };
    // redesenarea, cu timpul ei: peste 100 ms se simte, si intra in jurnal
    let deseneaza = move || {
        let t = std::time::Instant::now();
        deseneaza_brut();
        let ms = t.elapsed().as_millis() as u64;
        if ms > 100 {
            tracing::warn!(target: "gmotion::app", ms, "redesenare lenta");
        } else {
            tracing::trace!(target: "gmotion::app", ms, "redesenare");
        }
    };

    let salveaza = {
        let stare = stare.clone();
        let e_ui = editor_ui.as_weak();
        // `fundal`: fisierul se citeste, se uneste si se scrie pe alt fir
        // (salvarea amanata, etapa 33); altfel pe loc (deconectare,
        // scripturi scrise, inchidere: dupa ele nu mai e cine sa astepte)
        move |fundal: bool| {
            // coloana listei din editor (etapa 16): starea ei e in fereastra
            let coloana = e_ui.upgrade().map(|u| gmotion_app::rutare::StareColoana {
                latime: u.get_latime_lista_aleasa(),
                fixata: u.get_lista_fixata(),
                dreapta: u.get_lista_dreapta(),
                banda: u.get_lista_banda(),
            });
            if fundal {
                stare.borrow_mut().salveaza_in_fundal(coloana);
            } else {
                stare.borrow_mut().salveaza(coloana);
            }
        }
    };

    let salveaza = Rc::new(salveaza);
    // salvarea dupa o comanda e amanata si grupata (`SalvareAmanata`); cele
    // explicite (deconectare, scripturi scrise, inchidere) raman imediate
    let amanata = Rc::new(std::cell::Cell::new(stare::SalvareAmanata::default()));
    let ceas_salvare = Rc::new(slint::Timer::default());
    let salveaza_acum = {
        let salveaza = salveaza.clone();
        let amanata = amanata.clone();
        let ceas = ceas_salvare.clone();
        Rc::new(move |fundal: bool| {
            ceas.stop();
            let mut a = amanata.get();
            a.facuta();
            amanata.set(a);
            salveaza(fundal);
        })
    };
    let cere_salvarea = {
        let amanata = amanata.clone();
        let ceas = Rc::downgrade(&ceas_salvare);
        let salveaza_acum = salveaza_acum.clone();
        move || {
            let mut a = amanata.get();
            a.cere(std::time::Instant::now());
            amanata.set(a);
            let Some(c) = ceas.upgrade() else { return };
            let (amanata, salveaza_acum) = (amanata.clone(), salveaza_acum.clone());
            // repornit la fiecare cerere: se scrie o data, dupa ultima
            c.start(
                slint::TimerMode::SingleShot,
                stare::AMANARE_SALVARE,
                move || {
                    if amanata
                        .get()
                        .e_timpul(std::time::Instant::now(), stare::AMANARE_SALVARE)
                    {
                        salveaza_acum(true);
                    }
                },
            );
        }
    };
    let salveaza = {
        let s = salveaza_acum.clone();
        Rc::new(move || s(false))
    };

    let comanda = {
        let stare = stare.clone();
        let deseneaza = deseneaza.clone();
        let cere_salvarea = cere_salvarea.clone();
        let lista_slaba = lista_ui.as_weak();
        move |c: Comanda| {
            // mesajul din bara listei („Nu intra in plan", CSV scris) e al
            // comenzii de dinainte: altfel ramanea pe ecran la nesfarsit
            // (F-A16); desenarea il pune din nou daca e inca adevarat
            if let Some(l) = lista_slaba.upgrade() {
                l.set_mesaj_csv("".into());
            }
            let t = std::time::Instant::now();
            let text: String = format!("{c:?}").chars().take(200).collect();
            let schimbat = stare
                .borrow_mut()
                .app
                .as_mut()
                .map(|a| a.comanda(c))
                .unwrap_or(false);
            let ms_comanda = t.elapsed().as_millis() as u64;
            if schimbat {
                cere_salvarea();
            }
            deseneaza();
            let ms = t.elapsed().as_millis() as u64;
            // starea dupa comanda: unde esti si ce e selectat
            let dupa = stare.borrow().app.as_ref().map(|a| {
                format!(
                    "ecran={:?} obiect={:?} vizibile={} din={} diferenta={}",
                    a.ecran(),
                    a.obiect().map(|o| o.cheie()),
                    a.lista().vizibile().len(),
                    a.lista().obiecte().len(),
                    a.diferenta_curenta()
                )
            });
            if ms > 200 {
                tracing::warn!(target: "gmotion::actiuni", comanda = %text, ms_comanda, ms_total = ms, stare = ?dupa, "comanda lenta");
            } else {
                tracing::debug!(target: "gmotion::actiuni", comanda = %text, ms_comanda, ms_total = ms, stare = ?dupa, "comanda");
            }
        }
    };

    // Leaga un callback care doar schimba `Stare`: actiunea in jurnal, corpul,
    // apoi redesenarea. Logica sta in `gmotion_app::stare`, unde se testeaza;
    // aici ramane doar legatura (`arg` = argumentele scrise in jurnal).
    // numele ecranului in jurnal, din numele ferestrei: `lista_ui` -> "lista"
    macro_rules! ecran {
        ($ui:ident) => {
            stringify!($ui).trim_end_matches("_ui")
        };
    }
    macro_rules! pe_stare {
        ($ui:ident . $on:ident(|$st:ident $(, $a:ident)*| $corp:expr)) => {
            pe_stare!(@ $ui, $on, true, |$st $(, $a)*| $corp)
        };
        // fara argumente in jurnal: parola nu se scrie niciodata
        (secret $ui:ident . $on:ident(|$st:ident $(, $a:ident)*| $corp:expr)) => {
            pe_stare!(@ $ui, $on, false, |$st $(, $a)*| $corp)
        };
        (@ $ui:ident, $on:ident, $cu_arg:expr, |$st:ident $(, $a:ident)*| $corp:expr) => {{
            let s = stare.clone();
            let d = deseneaza.clone();
            $ui.$on(move |$($a),*| {
                let arg = if $cu_arg { format!("{:?}", ($(&$a,)*)) } else { String::new() };
                actiune(ecran!($ui), stringify!($on).trim_start_matches("on_"), &arg, false);
                {
                    let $st = &mut *s.borrow_mut();
                    $corp;
                }
                d();
            });
        }};
    }

    // Comutatorul de limba si butonul de tema, la fel pe fiecare ecran: schimba
    // toate ecranele, tin minte alegerea, redeseneaza.
    macro_rules! limba_si_tema {
        ($ui:expr, $ecran:literal) => {{
            let al = alege_limba.clone();
            let d = deseneaza.clone();
            $ui.on_alege_limba(move |i| {
                actiune($ecran, "alege_limba", &format!("{:?}", (&i,)), false);
                al(i);
                d();
            });
            let a = alege_tema.clone();
            let d = deseneaza.clone();
            $ui.on_alege_tema(move |intunecat| {
                actiune($ecran, "alege_tema", &format!("{:?}", (&intunecat,)), false);
                a(intunecat);
                // hasura e pictata in culorile modului
                d();
            });
        }};
    }

    // --- ecranul de conectare
    {
        pe_stare!(
            conectare_ui.on_server_schimbat(|st, sursa, text| st.server_schimbat(sursa, &text))
        );
        pe_stare!(
            conectare_ui.on_auth_schimbat(|st, sursa, windows| st.auth_schimbat(sursa, windows))
        );
        pe_stare!(conectare_ui
            .on_utilizator_schimbat(|st, sursa, text| st.utilizator_schimbat(sursa, &text)));
        pe_stare!(secret conectare_ui.on_parola_schimbata(|st, sursa, text| st.parola_schimbata(sursa, &text))
        );
        pe_stare!(conectare_ui.on_baza_schimbata(|st, sursa, text| st.baza_schimbata(sursa, &text)));
        pe_stare!(conectare_ui.on_cere_bazele(|st, sursa| st.cere_bazele(sursa)));
        let timeout = std::time::Duration::from_millis(cfg.timeout_descoperire_ms);
        pe_stare!(conectare_ui.on_cauta_servere(|st, sursa| st.cauta_servere(sursa, timeout)));
        pe_stare!(conectare_ui.on_server_ales(|st, sursa, server| st.server_ales(sursa, &server)));
        limba_si_tema!(conectare_ui, "conectare");
        pe_stare!(conectare_ui.on_aceeasi_conexiune_comutata(|st| st.comuta_aceeasi_conexiune()));
        pe_stare!(conectare_ui.on_schimba_certificatul(|st| st.comuta_certificatul()));
        let d = deseneaza.clone();
        let c_ui = conectare_ui.as_weak();
        conectare_ui.on_deschide_instantaneu(move || {
            actiune("conectare", "deschide_instantaneu", "", false);
            // alegerea se face in aplicatie, nu cu un dialog nativ: acela ar
            // fi o dependinta noua, si nu se poate proba pe Linux fara ecran.
            // Vezi `JURNAL.md`.
            let gasite = pornire::instantanee_gasite();
            tracing::info!(target: "gmotion::app", gasite = gasite.len(), "caut instantanee");
            if let Some(ui) = c_ui.upgrade() {
                ui.set_instantanee(slint::ModelRc::new(slint::VecModel::from(
                    gasite
                        .iter()
                        .map(|c| slint::SharedString::from(c.display().to_string()))
                        .collect::<Vec<_>>(),
                )));
                ui.set_alege_fisierul(true);
            }
            d();
        });
        let c_ui = conectare_ui.as_weak();
        pe_stare!(conectare_ui.on_alege_instantaneul(|st, cale| {
            if st.alege_instantaneul(&cale) {
                if let Some(ui) = c_ui.upgrade() {
                    ui.set_alege_fisierul(false);
                }
            }
        }));
        pe_stare!(conectare_ui.on_compara(|st| st.compara()));
        let c_ui = conectare_ui.as_weak();
        pe_stare!(conectare_ui.on_alege_cu_dialogul(|st| {
            let ales = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file();
            if let Some(cale) = ales {
                if st.alege_instantaneul(&cale.display().to_string()) {
                    if let Some(ui) = c_ui.upgrade() {
                        ui.set_alege_fisierul(false);
                    }
                }
            }
        }));
        pe_stare!(conectare_ui.on_testeaza(|st, sursa| st.testeaza(sursa)));
        let d = deseneaza.clone();
        conectare_ui.on_sterge_jurnalul(move || {
            actiune("conectare", "sterge_jurnalul", "", false);
            gmotion_platforma::jurnal::goleste();
            d();
        });
        let c_ui = conectare_ui.as_weak();
        let d = deseneaza.clone();
        conectare_ui.on_deschide_offline(move || {
            actiune("conectare", "deschide_offline", "", false);
            if let Some(ui) = c_ui.upgrade() {
                ui.set_offline(true);
                ui.set_offline_eroare(Default::default());
            }
            d();
        });
        let c_ui = conectare_ui.as_weak();
        let d = deseneaza.clone();
        conectare_ui.on_inchide_offline(move || {
            actiune("conectare", "inchide_offline", "", false);
            if let Some(ui) = c_ui.upgrade() {
                ui.set_offline(false);
            }
            d();
        });
        let c_ui = conectare_ui.as_weak();
        pe_stare!(conectare_ui.on_compara_offline(|st, a, b| {
            let rezultat = st.compara_offline(&a, &b);
            if let Some(ui) = c_ui.upgrade() {
                match rezultat {
                    Ok(()) => ui.set_offline(false),
                    Err(e) => ui.set_offline_eroare(e.into()),
                }
            }
        }));
        conectare_ui.on_ajutor(|| {
            tracing::info!(target: "gmotion::app", "deschid ajutorul");
        });
        pe_stare!(conectare_ui.on_ignora_recenta(|st, i| st.ignora_recenta(i)));
        pe_stare!(conectare_ui.on_anuleaza(|st| st.anuleaza()));
        pe_stare!(conectare_ui.on_reia(|st, i| st.reia(i)));
    }

    // Leaga un callback care trimite o comanda aplicatiei (`rutare`).
    macro_rules! pe_comanda {
        ($ui:ident . $on:ident(|| $corp:expr)) => {{
            let c = comanda.clone();
            $ui.$on(move || {
                actiune(ecran!($ui), stringify!($on).trim_start_matches("on_"), "", false);
                c($corp);
            });
        }};
        ($ui:ident . $on:ident(|$($a:ident),*| $corp:expr)) => {{
            let c = comanda.clone();
            $ui.$on(move |$($a),*| {
                actiune(ecran!($ui), stringify!($on).trim_start_matches("on_"), &format!("{:?}", ($(&$a,)*)), false);
                c($corp);
            });
        }};
    }
    // `0`-ul unui indice venit din fereastra; negativ = nimic ales
    fn indice(i: i32) -> Option<usize> {
        usize::try_from(i).ok()
    }

    // --- lista
    {
        // harta (`h`): deschiderea, reasezarea si bula sunt ale ferestrei;
        // clicul pe patrat schimba selectia, deci trece prin `Comanda`
        gmotion_fereastra::leaga_harta(&lista_ui);
        // sintaxa cat scrii in campul de editare (etapa 25)
        gmotion_fereastra::leaga_editarea(&editor_ui);
        let c = comanda.clone();
        let l = lista_ui.as_weak();
        lista_ui.on_clic_harta(move |grup, x, y| {
            actiune(
                "lista",
                "clic_harta",
                &format!("{grup:?} {x:?} {y:?}"),
                false,
            );
            let Some(ui) = l.upgrade() else { return };
            if let Some(r) = gmotion_fereastra::rand_din_harta(&ui, grup, x, y) {
                // harta se inchide inainte, ca redesenarea sa nu o mai picteze
                ui.set_arata_harta(false);
                c(Comanda::Selecteaza(r));
            }
        });
        pe_comanda!(lista_ui.on_alege_randul(|i| Comanda::Selecteaza(indice(i).unwrap_or(0))));
        // un clic: selecteaza si deschide / inchide detaliile. Dublul clic nu
        // mai deschide obiectul (Mihai, 2026-11-25): deschide butonul de pe rand
        pe_comanda!(lista_ui.on_clic_rand(|i| Comanda::ClicPeRand(indice(i).unwrap_or(0))));
        let c = comanda.clone();
        lista_ui.on_deschide_randul(move |i| {
            actiune("lista", "deschide_randul", &format!("{:?}", (&i,)), false);
            c(Comanda::Selecteaza(indice(i).unwrap_or(0)));
            c(Comanda::Deschide);
        });
        // numele se taie pe latimea coloanei: o latime noua le reface (amanat,
        // `changed` vine din mijlocul unei aranjari)
        let d = deseneaza.clone();
        lista_ui.on_latime_nume_schimbata(move || {
            slint::Timer::single_shot(std::time::Duration::ZERO, d.clone());
        });
        let d = deseneaza.clone();
        lista_ui.on_previz_redimensionata(move || {
            slint::Timer::single_shot(std::time::Duration::ZERO, d.clone());
        });
        pe_comanda!(lista_ui.on_reseteaza_filtrele(|| Comanda::ResetFiltre));
        pe_comanda!(lista_ui.on_deschide_obiectul(|| Comanda::Deschide));
        // „+" pe rand / „+ Tab nou" in panou (etapa 33); -1 = selectia
        pe_comanda!(lista_ui.on_deschide_in_tab_nou(|i| Comanda::DeschideInTabNou(indice(i))));
        pe_comanda!(lista_ui.on_comuta_omis(|| Comanda::ComutaOmis));
        pe_comanda!(lista_ui.on_nota_editata(|baza, id, text| {
            Comanda::PuneNota(baza.to_string(), id.to_string(), text.to_string())
        }));
        lista_ui.on_ajutor(|| {
            tracing::info!(target: "gmotion::app", "deschid ajutorul");
        });
        let l = lista_ui.as_weak();
        lista_ui.on_cere_deconectarea(move || {
            actiune("lista", "cere_deconectarea", "", false);
            if let Some(ui) = l.upgrade() {
                ui.set_intreaba_deconectarea(true);
            }
        });
        let l = lista_ui.as_weak();
        lista_ui.on_renunta_la_deconectare(move || {
            actiune("lista", "renunta_la_deconectare", "", false);
            if let Some(ui) = l.upgrade() {
                ui.set_intreaba_deconectarea(false);
            }
        });
        let s = stare.clone();
        let l = lista_ui.as_weak();
        let sv = salveaza.clone();
        let d = deseneaza.clone();
        lista_ui.on_deconecteaza(move || {
            actiune("lista", "deconecteaza", "", false);
            // progresul se salveaza inainte de a arunca starea: altfel
            // „Deconecteaza" ar insemna „pierde ce ai revizuit"
            sv();
            s.borrow_mut().deconecteaza();
            if let Some(ui) = l.upgrade() {
                ui.set_intreaba_deconectarea(false);
            }
            d();
        });
        // recompararea: acelasi drum ca prima comparare, revizuirea ramane
        pe_stare!(lista_ui.on_recompara(|st| st.compara()));
        pe_stare!(lista_ui.on_anuleaza_recompararea(|st| st.anuleaza()));
        limba_si_tema!(lista_ui, "lista");
        // indicele randului apasat, nu cel selectat (V9)
        pe_comanda!(lista_ui.on_comuta_revizuit(|i| match indice(i) {
            Some(p) => Comanda::ComutaRevizuitLa(p),
            None => Comanda::ComutaRevizuit,
        }));
        pe_comanda!(lista_ui.on_cauta(|t| Comanda::Cauta(t.to_string())));
        pe_comanda!(lista_ui.on_alege_categoria(|i| {
            Comanda::Filtreaza(indice(i).and_then(|i| StareObiect::TOATE.get(i).copied()))
        }));
        let s = stare.clone();
        pe_comanda!(
            lista_ui.on_alege_tipul(|i| Comanda::FiltreazaTip(indice(i).and_then(|i| {
                s.borrow()
                    .app
                    .as_ref()
                    .and_then(|a| a.lista().contoare_tipuri().get(i).map(|(t, _)| t.clone()))
            })))
        );
        // clic pe un cap de coloana: sorteaza dupa ea; al doilea clic inverseaza
        pe_comanda!(lista_ui.on_sorteaza_coloana(|cod| {
            Comanda::Sorteaza(gmotion_fereastra::ordine_din_cod(cod))
        }));
        let s = stare.clone();
        let l = lista_ui.as_weak();
        let d = deseneaza.clone();
        let dc = RefCell::new(DubluClic::default());
        lista_ui.on_clic_margine_coloana(move |cod| {
            actiune("lista", "clic_margine_coloana", &format!("{cod:?}"), false);
            // dublul clic pe margine = autofit; clicul simplu nu face nimic
            if dc.borrow_mut().apasat(cod, Instant::now()) {
                tracing::debug!(target: "gmotion::actiuni", cod, "autofit coloana");
                let latime = s.borrow().app.as_ref().map(|a| {
                    gmotion_fereastra::latime_autofit(
                        a.lista(),
                        gmotion_fereastra::ordine_din_cod(cod),
                    )
                });
                if let (Some(ui), Some(latime)) = (l.upgrade(), latime) {
                    ui.global::<gmotion_fereastra::Coloane>()
                        .invoke_pune(cod, latime);
                }
                d();
            }
        });
        let s = stare.clone();
        pe_comanda!(
            lista_ui.on_alege_baza(|i| Comanda::FiltreazaBaza(indice(i).and_then(|i| {
                s.borrow()
                    .app
                    .as_ref()
                    .and_then(|a| a.lista().contoare_baze().get(i).map(|(b, _)| b.clone()))
            })))
        );
        pe_comanda!(lista_ui.on_alege_starea(|i| {
            Comanda::FiltreazaRevizuire(
                indice(i).and_then(|i| FiltruRevizuire::TOATE.get(i).copied()),
            )
        }));
        pe_comanda!(lista_ui.on_alege_impactul(|i| {
            Comanda::FiltreazaImpact(indice(i).and_then(|i| {
                gmotion_fereastra::model_lista::Impact::TOATE
                    .get(i)
                    .copied()
            }))
        }));
        // etapa 27: operatia si perioadele de creare / modificare
        pe_comanda!(lista_ui.on_alege_operatia(|i| Comanda::FiltreazaOperatie(
            indice(i).and_then(|i| gmotion_fereastra::model_lista::Operatie::TOATE
                .get(i)
                .copied())
        )));
        pe_comanda!(
            lista_ui.on_alege_creat(|i| Comanda::FiltreazaCreat(indice(i).and_then(|i| {
                gmotion_fereastra::model_lista::Perioada::TOATE
                    .get(i)
                    .copied()
            })))
        );
        pe_comanda!(lista_ui.on_alege_modificat(|i| Comanda::FiltreazaModificat(
            indice(i).and_then(|i| gmotion_fereastra::model_lista::Perioada::TOATE
                .get(i)
                .copied())
        )));
        // campurile sunt numerice, dar pot fi goale cat se scrie in ele
        pe_comanda!(lista_ui.on_schimba_scorul(|min, max| {
            let n = |t: &str, implicit: f64| t.trim().parse::<f64>().unwrap_or(implicit);
            Comanda::FiltreazaScor(n(&min, 0.0), n(&max, 100.0))
        }));
        pe_comanda!(lista_ui.on_pune_vizibilele_in_plan(|cu| Comanda::PuneVizibileleInPlan(cu)));
        pe_stare!(lista_ui.on_exporta_csv(|st| st.exporta_csv()));
    }

    // --- planul de deploy
    {
        pe_comanda!(lista_ui.on_adauga_in_plan(|| Comanda::ComutaInPlan));
        pe_comanda!(lista_ui.on_deschide_deploy(|| Comanda::DeschideDeploy));
        pe_comanda!(editor_ui.on_adauga_in_plan(|| Comanda::ComutaInPlan));
        pe_comanda!(deploy_ui.on_inapoi_lista(|| Comanda::InapoiLaLista));
        pe_comanda!(deploy_ui.on_scoate_din_plan(|baza, id| {
            Comanda::ScoateDinPlan(baza.to_string(), id.to_string())
        }));
        limba_si_tema!(deploy_ui, "deploy");
        // garda: se recitesc ambele baze si se compara amprentele
        pe_stare!(deploy_ui.on_verifica_baza(|st| st.porneste_garda()));
        // textul final la SQL Server, fara efecte (etapa 26)
        pe_stare!(editor_ui.on_verifica_pe_server(|st| st.verifica_pe_server()));
        pe_stare!(editor_ui.on_inchide_verificarea(|st| st.rezultat_verificare = None));
        // dialogul nativ (etapa 25): unde se scrie, si folderul deschis dupa
        pe_stare!(deploy_ui.on_alege_folderul(|st| {
            if let Some(d) = rfd::FileDialog::new().pick_folder() {
                tracing::info!(target: "gmotion::app", folder = %d.display(), "folder de deploy ales");
                st.folder_deploy = Some(d);
            }
        }));
        pe_stare!(deploy_ui.on_deschide_folderul(|st| {
            if let Some(d) = st.director_scris() {
                if let Err(e) = stare::deschide_in_explorer(&d) {
                    tracing::warn!(target: "gmotion::app", eroare = %e, "folderul nu s-a deschis");
                }
            }
        }));
        let s = stare.clone();
        let d = deseneaza.clone();
        let sv = salveaza.clone();
        deploy_ui.on_scrie_scripturile(move || {
            actiune("deploy", "scrie_scripturile", "", false);
            let de_salvat = {
                let mut st = s.borrow_mut();
                let dir = st.director_deploy_curent();
                st.scrie_deploy(&dir)
            };
            // obiectele scrise devin „deployate" in lista si in sesiune
            if de_salvat {
                sv();
            }
            d();
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        let cate = cate_per_fisier.clone();
        deploy_ui.on_alege_impartirea(move |i| {
            actiune("deploy", "alege_impartirea", &format!("{:?}", (&i,)), false);
            {
                if let Some(dep) = s.borrow_mut().deploy.as_mut().map(|d| &mut d.stare) {
                    dep.alege_impartirea(gmotion_app::deploy::impartire_din(i, cate.get()));
                }
                d();
            }
        });
        // N scris in camp: un numar invalid sau 0 se ignora, cat se scrie
        let s = stare.clone();
        let d = deseneaza.clone();
        let cate = cate_per_fisier.clone();
        deploy_ui.on_schimba_cate(move |t| {
            actiune("deploy", "schimba_cate", &format!("{:?}", (&t,)), false);
            {
                let Some(n) = t.trim().parse::<usize>().ok().filter(|&n| n > 0) else {
                    return;
                };
                cate.set(n);
                if let Some(dep) = s.borrow_mut().deploy.as_mut().map(|d| &mut d.stare) {
                    dep.alege_impartirea(gmotion_app::deploy::impartire_din(3, n));
                }
                d();
            }
        });
    }

    // --- editor
    {
        pe_comanda!(editor_ui.on_inapoi_lista(|| Comanda::InapoiLaLista));
        pe_comanda!(editor_ui.on_obiect_urmator(|| Comanda::ObiectUrmator));
        pe_comanda!(editor_ui.on_obiect_anterior(|| Comanda::ObiectAnterior));
        pe_comanda!(editor_ui.on_comuta_revizuit(|| Comanda::ComutaRevizuit));
        // clic in coloana din stanga: schimba obiectul pe loc, fara sa pleci
        // din editor (pachetul B)
        // „Deschide" pe un rand din Object Explorer: in tabul curent (de
        // previzualizare), ca pana acum. Un clic simplu doar alege randul, in
        // `.slint` (etapa 33): tabul deschis nu se mai pierde din greseala
        let c = comanda.clone();
        editor_ui.on_alege_randul(move |i| {
            actiune("editor", "alege_randul", &format!("{:?}", (&i,)), false);
            c(Comanda::Selecteaza(i.max(0) as usize));
        });
        // ---- taburile (etapa 16); regulile sunt in `rutare` / `model_taburi`
        // „+ Tab nou" / dublu clic: tab nou, fara sa atinga tabul deschis
        pe_comanda!(editor_ui.on_fixeaza_randul(|i| Comanda::DeschideInTabNou(indice(i))));
        let c = comanda.clone();
        let dc = RefCell::new(DubluClic::default());
        editor_ui.on_activeaza_tab(move |cod| {
            actiune("editor", "activeaza_tab", &format!("{:?}", (&cod,)), false);
            c(Comanda::ActiveazaTab(usize::try_from(cod).ok()));
            // dublu clic pe previzualizare (codul -1): o fixeaza
            if dc.borrow_mut().apasat(cod, Instant::now()) && cod < 0 {
                tracing::debug!(target: "gmotion::actiuni", "dublu clic pe previzualizare");
                c(Comanda::FixeazaPrevizualizarea);
            }
        });
        // arborele din coloana (Object Explorer): baza / dosar se deschid si
        // se inchid, ca in lista
        pe_comanda!(editor_ui.on_comuta_nod(|cheie| Comanda::ComutaNod(cheie.to_string())));
        pe_comanda!(editor_ui.on_inchide_tab(|cod| Comanda::InchideTab(usize::try_from(cod).ok())));
        pe_comanda!(editor_ui.on_fixeaza_previzualizarea(|| Comanda::FixeazaPrevizualizarea));
        pe_comanda!(editor_ui.on_inchide_tab_activ(|| Comanda::InchideTabActiv));
        // Deploy (N) din bara de sus a editorului (etapa 18)
        pe_comanda!(editor_ui.on_deschide_deploy(|| Comanda::DeschideDeploy));
        // previzualizarea la hover pe sageata: nu e o comanda (nu schimba
        // nimic, nu redeseneaza), doar umple liniile ferestrei
        let e = editor_ui.as_weak();
        let st = stare.clone();
        editor_ui.on_previzualizeaza(move |k| {
            actiune("editor", "previzualizeaza", &format!("{:?}", (&k,)), true);
            // `try_borrow`: hover-ul vine dintr-un `changed`, care poate porni
            // in mijlocul unei desenari (care tine starea imprumutata); un
            // `borrow` simplu ar fi panica, deci in release caderea aplicatiei
            let Ok(stare) = st.try_borrow() else {
                return;
            };
            let linii = usize::try_from(k)
                .ok()
                .and_then(|k| stare.app.as_ref().and_then(|a| a.previzualizare(k)))
                .unwrap_or_default();
            drop(stare);
            if let Some(ui) = e.upgrade() {
                ui.set_previzualizare_linii(gmotion_fereastra::previzualizare_ui(&linii));
            }
        });
        // doar arborele din coloana: obiectul deschis ramane
        pe_comanda!(editor_ui.on_cauta(|t| Comanda::CautaInArbore(t.to_string())));
        let e = editor_ui.as_weak();
        let d = deseneaza.clone();
        // `[` (si «): coloana devine banda ingusta cu contoare, sau inapoi
        // (etapa 16); fara coloana deloc, `[` o readuce la latimea trasa
        let sv = salveaza.clone();
        editor_ui.on_ascunde_lista(move || {
            actiune("editor", "ascunde_lista", "", false);
            if let Some(ui) = e.upgrade() {
                if ui.get_latime_lista() > 0.0 {
                    ui.set_lista_banda(!ui.get_lista_banda());
                } else {
                    ui.set_latime_lista(ui.get_latime_lista_aleasa());
                }
            }
            sv();
            d();
        });
        // pin, partea, banda, latimea: se tin minte in sesiune
        let sv = salveaza.clone();
        let d = deseneaza.clone();
        editor_ui.on_coloana_schimbata(move || {
            actiune("editor", "coloana_schimbata", "", false);
            sv();
            d();
        });
        // plierea si alaturat / unificat stau in fereastra; modelul se
        // reconstruieste la redesenare, din ce arata fereastra acum
        pe_comanda!(editor_ui.on_comuta_omis(|| Comanda::ComutaOmis));
        pe_comanda!(editor_ui.on_comuta_unirea_cursorului(|cu| Comanda::ComutaUnireaCursorului(cu)));
        // prin `pe_comanda!`, ca toate celelalte: scrie si linia in jurnalul
        // de sesiune. Era singura actiune reala fara ea, si tocmai cea care
        // schimba textul ce pleaca in baza — „un buton fara linie in jurnal
        // nu e legat" (C-D5 din auditul 2026-11-23)
        pe_comanda!(
            editor_ui.on_aplica_diferenta(|k| Comanda::AplicaDiferenta(usize::try_from(k).ok()))
        );
        // un bloc tras si lasat: pe locul sugerat (-1) sau unde l-a lasat omul
        let c = comanda.clone();
        editor_ui.on_plaseaza(move |bloc, loc| {
            actiune("editor", "plaseaza", &format!("{:?}", (&bloc, &loc)), false);
            let Ok(bloc) = usize::try_from(bloc) else {
                return;
            };
            c(Comanda::Plaseaza {
                bloc,
                loc: gmotion_fereastra::loc_din_ui(loc),
            })
        });
        pe_comanda!(editor_ui.on_aplica_toate(|| Comanda::AplicaToate));
        pe_comanda!(editor_ui.on_reset(|| Comanda::Reset));
        pe_comanda!(editor_ui.on_anuleaza(|| Comanda::Anuleaza));
        pe_comanda!(editor_ui.on_salveaza_textul(|id, t| {
            Comanda::SalveazaText {
                id: id.to_string(),
                text: t.to_string(),
            }
        }));
        pe_comanda!(editor_ui.on_ghidat_porneste(|| Comanda::PornesteGhidat));
        pe_comanda!(editor_ui.on_ghidat_opreste(|| Comanda::OpresteGhidat));
        pe_comanda!(editor_ui.on_ghidat_alege(|din_sursa| Comanda::GhidatAlege(din_sursa)));
        pe_comanda!(editor_ui.on_ghidat_aplica(|| Comanda::GhidatAplica));
        pe_comanda!(editor_ui.on_ghidat_sari(|| Comanda::GhidatSari));
        // „Re-sincronizeaza": se compara din nou textul curent (modelul se
        // reconstruieste la redesenare)
        let d = deseneaza.clone();
        editor_ui.on_resincronizeaza(move || {
            actiune("editor", "resincronizeaza", "", false);
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
            actiune("editor", "diferenta_schimbata", &format!("{k}"), true);
            if let Some(a) = s.borrow_mut().app.as_mut() {
                a.comanda(Comanda::MergiLaDiferenta(indice(k).unwrap_or(0)));
            }
        });
        let d = deseneaza.clone();
        let cale = gmotion_platforma::cai::cale_configurare().ok();
        editor_ui.on_comuta_plierea(move |pliat| {
            actiune("editor", "comuta_plierea", &format!("{pliat:?}"), false);
            // tinuta minte ca tema: se reciteste configurarea, ca sa nu
            // suprascrie ce s-a schimbat de mana intre timp
            if let Some(cale) = cale.as_ref() {
                stare::salveaza_in_configurare(cale, "plierea", |c| c.pliaza_identicul = pliat);
            }
            d()
        });
        let d = deseneaza.clone();
        editor_ui.on_comuta_alaturat(move |_| {
            actiune("editor", "comuta_alaturat", "", false);
            d()
        });
        // wrap (etapa 19): se redeseneaza cu liniile rupte la latimea panoului
        let d = deseneaza.clone();
        // amanat: o redesenare nu porneste din mijlocul alteia
        editor_ui.on_comuta_wrap(move |pornit| {
            actiune("editor", "comuta_wrap", &format!("{:?}", (&pornit,)), false);
            slint::Timer::single_shot(std::time::Duration::ZERO, d.clone());
        });
        // latimea panoului (nu omul): la pornire si la maximizare veneau doua-
        // trei pe rand, fiecare cu randurile refacute (`comuta_wrap` de doua
        // ori la pornire, cadru de 118 ms dupa comparare). Grupate la 120 ms
        // si doar cu editorul pe ecran: altfel il reface oricum intrarea in el
        let d = deseneaza.clone();
        let st = stare.clone();
        let ceas_wrap = Rc::new(slint::Timer::default());
        editor_ui.on_latime_wrap_schimbata(move || {
            let (d, st) = (d.clone(), st.clone());
            ceas_wrap.start(
                slint::TimerMode::SingleShot,
                std::time::Duration::from_millis(120),
                move || {
                    let in_editor = st
                        .try_borrow()
                        .is_ok_and(|s| s.app.as_ref().is_some_and(|a| a.ecran() == Ecran::Editor));
                    tracing::debug!(target: "gmotion::app", in_editor, "latimea de wrap s-a schimbat");
                    if in_editor {
                        d();
                    }
                },
            );
        });
        limba_si_tema!(editor_ui, "editor");
    }

    // ceasul care aduce progresul compararii de pe celalalt fir. Bate tot
    // timpul, dar nu face nimic cat nu ruleaza nimic.
    let ceas = slint::Timer::default();
    let pornit = Instant::now();
    porneste_garda(pornit);
    {
        let s = stare.clone();
        let d = deseneaza.clone();
        ceas.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(120),
            move || {
                BATAIE_MS.store(
                    pornit.elapsed().as_millis() as u64,
                    std::sync::atomic::Ordering::Relaxed,
                );
                // celelalte lucruri de pe alte fire, apoi compararea
                let s_a_schimbat = s.borrow_mut().verifica_lucrarile();
                // salvarea din fundal: ora „salvat la", sau marcaje din `cli`
                let salvata = s.borrow_mut().verifica_salvarea();
                if s_a_schimbat || salvata {
                    d();
                }
                let progres = s.borrow_mut().verifica_compararea();
                match progres {
                    Progres::Nimic => {}
                    // inca ruleaza: bara si ceasul; s-a terminat: tot ecranul
                    Progres::InCurs | Progres::Terminata => d(),
                }
            },
        );
    }

    // F11: tot ecranul, din oricare fereastra. Slint tine starea pe
    // fereastra, deci se comuta pe cea care a primit tasta.
    macro_rules! ecran_complet {
        ($ui:expr) => {{
            let w = $ui.as_weak();
            $ui.on_ecran_complet(move || {
                if let Some(ui) = w.upgrade() {
                    let f = ui.window().is_fullscreen();
                    ui.window().set_fullscreen(!f);
                }
            });
        }};
    }
    ecran_complet!(conectare_ui);
    ecran_complet!(lista_ui);
    ecran_complet!(editor_ui);
    ecran_complet!(deploy_ui);

    // previzualizarea: se intra direct in lista, cu toate bazele bifate
    if previzualizare {
        stare.borrow_mut().porneste_previzualizarea();
    }

    deseneaza();
    // primul ecran apare pe loc; de aici incolo ecranele trec unul in altul
    {
        let a = aplicatie.as_weak();
        slint::Timer::single_shot(std::time::Duration::from_millis(400), move || {
            if let Some(a) = a.upgrade() {
                a.set_durata_tranzitie(DURATA_TRANZITIE_MS);
            }
        });
    }
    slint::run_event_loop().map_err(|e| e.to_string())?;
    // ce a ramas amanat se scrie la iesire
    if amanata.get().in_asteptare() {
        salveaza();
    }
    // si cea care inca se scrie pe alt fir
    stare.borrow_mut().termina_salvarea();
    Ok(())
}
