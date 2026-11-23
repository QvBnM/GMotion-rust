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

use gmotion_app::fir::Lucru;
use gmotion_app::pornire;
use gmotion_app::rutare::{Aplicatie, Comanda, Ecran};
use gmotion_fereastra::dublu_clic::DubluClic;
use gmotion_fereastra::model_conectare::{Conectare, Recenta, ServerGasit, StareConexiune};
use gmotion_fereastra::model_lista::{FiltruRevizuire, Stare as StareObiect};
use gmotion_fereastra::{
    aplica_tema, goleste_previzualizarea, incarca_conectare, incarca_deploy, incarca_editarea,
    incarca_editor, incarca_jurnal, incarca_lista, incarca_previzualizare, AntetEditor, AntetLista,
    EditorDiferente, FereastraConectare, FereastraDeploy, FereastraLista, Mod, StareEditare,
};
use gmotion_modul_diff_obiecte as modul;
use slint::ComponentHandle;

/// Bazele unui server si ce s-a aflat despre el incercand sa le citeasca.
type BazeCitite = (Vec<String>, StareConexiune);

/// Ce stie aplicatia in orice moment.
struct Stare {
    conectare: Conectare,
    /// Lipseste pana la prima comparare.
    app: Option<Aplicatie>,
    sesiuni: gmotion_platforma::Sesiuni,
    /// Ce scrie pe ecran despre sursa (server sau cale de instantaneu).
    server: String,
    /// Baza de referinta **cu care a rulat compararea**, nu ce scrie in
    /// formular acum. Formularul se poate schimba cat ruleaza (si se poate
    /// schimba de mana oricand dupa): sesiunea, deploy-ul si antetul se
    /// sprijina pe asta (F-A3 din auditul QA).
    referinta: String,
    /// Planul de deploy deschis, cu cheia lui: cand se schimba ce e „in
    /// plan", textele finale sau compararea, planul se reface de la zero —
    /// deci si fara garda. Decizia e in `deploy::asigura_planul`, nu aici.
    deploy: Option<gmotion_app::deploy::Deschis>,
    /// Compararea care ruleaza acum, pe alt fir.
    rulare: Option<gmotion_app::fir::Rulare>,
    /// Celelalte lucruri lungi, si ele pe alt fir: citirea bazelor, testul de
    /// conexiune, cautarea de servere si garda de deploy. Pe firul ferestrei,
    /// un server inaccesibil tinea aplicatia „Not responding" pana la timeout
    /// (P4 din auditul QA). `bool` = pentru care parte a ecranului.
    baze_cerute: Option<(bool, Lucru<BazeCitite>)>,
    testare: Option<(bool, Lucru<StareConexiune>)>,
    cautare_servere: Option<(bool, Lucru<Vec<ServerGasit>>)>,
    /// Starea coloanei din editor citita din sesiune, de pus in fereastra la
    /// urmatoarea desenare (etapa 16).
    coloana_noua: Option<gmotion_app::rutare::StareColoana>,
    garda: Option<Lucru<pornire::Citit>>,
    /// „Difera in" si `+N −M` pentru toate obiectele modificate, calculate pe
    /// alt fir dupa comparare (etapa 19). Un receptor nou opreste firul vechi.
    detalii: Option<std::sync::mpsc::Receiver<DetaliiCalculate>>,
    /// Cand s-a salvat ultima data progresul (revizuirea, textele finale):
    /// „salvat la 14:52" din editor.
    salvat_la: String,
    /// Ce se spune in bara listei la urmatoarea desenare: o sesiune pusa
    /// deoparte (F-A15), o recomparare esuata (C7).
    mesaj_lista: Option<String>,
    /// Revizuirea si textele finale de la ultima citire / salvare: baza
    /// unirii in trei cai cu ce a scris intre timp cli in sesiune.
    salvate_ultima: (modul::Revizuire, modul::Editari),
    /// Compararea de acum e una lipita (offline): nu se poate reface din
    /// formularul de conectare.
    offline: bool,
}

impl Stare {
    /// Partea aleasa, ca sa nu se repete `if sursa { ... } else { ... }` la
    /// fiecare camp din ecranul de conectare.
    ///
    /// `None` cat ruleaza o comparare: formularul e desenat blocat, dar
    /// primeste tastatura, si o tasta in campul bazei schimba sub noi
    /// referinta cu care s-a pornit (F-A3 din auditul QA). Intoarce `Option`
    /// tocmai ca sa nu se poata uita: compilatorul cere tratat cazul la
    /// fiecare camp.
    fn parte(&mut self, sursa: bool) -> Option<&mut gmotion_fereastra::model_conectare::Parte> {
        if self.rulare.is_some() {
            return None;
        }
        Some(if sursa {
            &mut self.conectare.sursa
        } else {
            &mut self.conectare.baza
        })
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

/// O sesiune care nu se poate citi se muta deoparte (nu se scrie peste ea la
/// prima bifa) si se spune pe ecran unde a ajuns (F-A15).
fn pune_deoparte(
    sesiuni: &gmotion_platforma::Sesiuni,
    server: &str,
    referinta: &str,
    eroare: &gmotion_platforma::Eroare,
    limba: gmotion_core::Limba,
) -> String {
    tracing::warn!(target: "gmotion::app", %eroare, "sesiunea salvata nu se poate citi");
    let unde = match sesiuni.pune_deoparte(server, referinta) {
        Ok(Some(c)) => c.display().to_string(),
        Ok(None) => String::new(),
        Err(e) => {
            tracing::warn!(target: "gmotion::app", eroare = %e, "sesiunea nu s-a putut pune deoparte");
            String::new()
        }
    };
    limba
        .alege(
            &format!("Progresul salvat nu s-a putut citi ({eroare}); l-am pus deoparte in {unde} si incep o sesiune noua."),
            &format!("The saved progress could not be read ({eroare}); it was moved to {unde} and a new session starts."),
        )
        .to_string()
}

fn argument(nume: &str) -> Option<String> {
    let a: Vec<String> = std::env::args().collect();
    a.iter()
        .position(|x| x == nume)
        .and_then(|i| a.get(i + 1))
        .cloned()
}

/// Modul de previzualizare: instantaneul sintetic cu obiecte de toate
/// felurile, ca Mihai sa probeze functiile fara baza de date.
///
/// Se cauta langa binar si in radacina proiectului, in ordinea asta: o
/// instalare are fisierul langa `gmotion-ui.exe`, iar in dezvoltare el sta in
/// `referinta/instantanee/`.
fn cale_previzualizare() -> Option<std::path::PathBuf> {
    const NUME: &str = "previzualizare.json";
    let mut locuri: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            locuri.push(d.join(NUME));
            locuri.push(d.join("instantanee").join(NUME));
        }
    }
    for sus in ["", "../", "../../", "../../../"] {
        locuri.push(std::path::PathBuf::from(format!("{sus}referinta/instantanee")).join(NUME));
    }
    locuri.into_iter().find(|c| c.is_file())
}

/// Unde se scrie deploy-ul: in directorul de date al aplicatiei, cu
/// referinta si momentul in nume; langa binar, daca acela nu se poate afla.
fn director_deploy(referinta: &str) -> std::path::PathBuf {
    let nume = format!(
        "{}_{}",
        referinta.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        gmotion_platforma::timp::acum_compact()
    );
    let baza = gmotion_platforma::cai::director_date()
        .map(|d| d.join("deploy").join(&nume))
        .unwrap_or_else(|_| std::path::PathBuf::from(format!("deploy_{nume}")));
    // a doua scriere in aceeasi secunda nu cade pe „director negol"
    let mut cale = baza.clone();
    let mut n = 2;
    while cale.exists() {
        cale = std::path::PathBuf::from(format!("{}_{n}", baza.display()));
        n += 1;
    }
    cale
}

/// Unde se scrie exportul CSV: in directorul de date, langa deploy-uri.
/// Ca la deploy, numele are momentul in el, ca doua exporturi sa nu se calce.
fn cale_csv(referinta: &str) -> std::path::PathBuf {
    let nume = format!(
        "{}_{}.csv",
        referinta.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        gmotion_platforma::timp::acum_compact()
    );
    gmotion_platforma::cai::director_date()
        .map(|d| d.join("export").join(&nume))
        .unwrap_or_else(|_| std::path::PathBuf::from(nume))
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
fn actiune(ecran: &str, nume: &str, argumente: &str, des: bool) {
    let a: String = argumente.chars().take(200).collect();
    if des {
        tracing::trace!(target: "gmotion::actiuni", ecran, nume, argumente = %a, "actiune");
    } else {
        tracing::info!(target: "gmotion::actiuni", ecran, nume, argumente = %a, "actiune");
    }
}

fn arata<T: ComponentHandle>(nou: &T, celelalte: [&slint::Window; 3]) {
    // fiecare ecran e o fereastra separata: cea noua preia pozitia, marimea
    // si starea (maximizata / tot ecranul) celei de dinainte, altfel pe
    // Windows „Deschide" parea ca porneste alta aplicatie, in alt loc
    let w = nou.window();
    if !w.is_visible() {
        tracing::debug!(target: "gmotion::app", marime = ?w.size(), pozitie = ?w.position(), "arat alt ecran");
        if let Some(veche) = celelalte.iter().find(|v| v.is_visible()) {
            // geometria preluata, ca o fereastra care sare sa se vada in jurnal
            tracing::debug!(
                target: "gmotion::app",
                marime = ?veche.size(),
                pozitie = ?veche.position(),
                maximizata = veche.is_maximized(),
                tot_ecranul = veche.is_fullscreen(),
                "preiau geometria ferestrei de dinainte"
            );
            if veche.is_fullscreen() {
                w.set_fullscreen(true);
            } else if veche.is_maximized() {
                w.set_maximized(true);
            } else {
                w.set_maximized(false);
                w.set_size(veche.size());
                w.set_position(veche.position());
            }
        }
    }
    let _ = nou.show();
    for w in celelalte {
        let _ = w.hide();
    }
}

/// Ce trimite firul de detalii: cheia obiectului si detaliile lui.
type DetaliiCalculate = ((String, String), gmotion_fereastra::model_lista::Detalii);

/// Porneste calculul detaliilor pe alt fir, pentru toate obiectele modificate
/// (intai cele vizibile). Firul se opreste singur cand receptorul e aruncat
/// (o comparare noua, deconectarea).
fn porneste_detalii(app: &Aplicatie) -> std::sync::mpsc::Receiver<DetaliiCalculate> {
    let lucrari = app.lucrari_detalii();
    let (tx, rx) = std::sync::mpsc::channel();
    tracing::debug!(target: "gmotion::app", obiecte = lucrari.len(), "calculez detaliile listei pe alt fir");
    std::thread::spawn(move || {
        for l in lucrari {
            if tx.send((l.cheie.clone(), l.calculeaza())).is_err() {
                break;
            }
        }
    });
    rx
}

/// Compararile salvate, cele mai noi intai.
fn recente(sesiuni: &gmotion_platforma::Sesiuni, limba: gmotion_core::Limba) -> Vec<Recenta> {
    let mut s = sesiuni.lista().unwrap_or_default();
    s.sort_by(|a, b| b.salvata.cmp(&a.salvata));
    s.into_iter()
        .take(8)
        .map(|s| {
            let baza = s.baze.first().cloned().unwrap_or_default();
            let revizuite = modul::Revizuire::din_sesiune(&s).numar();
            Recenta {
                progres: if revizuite == 0 {
                    String::new()
                } else {
                    format!("{revizuite} {}", limba.alege("revizuite", "reviewed"))
                },
                cand: s.salvata.clone(),
                sursa: s.referinta.clone(),
                server: s.server.clone(),
                baza,
            }
        })
        .collect()
}

fn ruleaza() -> Result<(), String> {
    // jurnalul de sesiune, si in fisier: fiecare actiune, cu durata ei, ca o
    // proba de mana sa se poata trimite si citi dupa
    match gmotion_platforma::cai::director_jurnale()
        .ok()
        .and_then(|d| gmotion_platforma::jurnal::porneste_cu_fisier(&d))
    {
        Some(cale) => {
            tracing::info!(target: "gmotion::app", fisier = %cale.display(), versiune = env!("CARGO_PKG_VERSION"), "jurnalul sesiunii se scrie in fisier")
        }
        None => gmotion_platforma::jurnal::porneste(),
    }
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
    conectare.recente = recente(&sesiuni, conectare.limba);
    conectare.instantaneu = argument("--sursa");
    if previzualizare && conectare.instantaneu.is_none() {
        match cale_previzualizare() {
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

    let stare = Rc::new(RefCell::new(Stare {
        conectare,
        app: None,
        sesiuni,
        server: String::new(),
        referinta: String::new(),
        deploy: None,
        rulare: None,
        baze_cerute: None,
        testare: None,
        cautare_servere: None,
        coloana_noua: None,
        garda: None,
        detalii: None,
        salvat_la: String::new(),
        mesaj_lista: None,
        salvate_ultima: Default::default(),
        offline: false,
    }));

    let conectare_ui = FereastraConectare::new().map_err(|e| e.to_string())?;
    gmotion_fereastra::leaga_scara(&conectare_ui);
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
    // setarile din configurare care schimba comportamentul (F-B5)
    stare.borrow_mut().conectare.timeout_conectare_s = Some(cfg.timeout_conectare_s);
    let uneste_implicit = cfg.uneste_cursor;
    // plierea liniilor identice: ce s-a ales ultima data (implicit, tot codul)
    editor_ui.set_pliaza_identicul(cfg.pliaza_identicul);

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
            let Some(cale) = cale_cfg.as_ref() else {
                return;
            };
            // se reciteste, ca la tema: altfel ar suprascrie ce s-a schimbat
            // de mana intre timp
            match gmotion_platforma::Configurare::citeste(cale) {
                Ok(mut c) => {
                    c.limba = l.nume().to_string();
                    if let Err(e) = c.scrie(cale) {
                        tracing::warn!(target: "gmotion::app", eroare = %e, "limba nu s-a salvat");
                    }
                }
                Err(e) => {
                    tracing::warn!(target: "gmotion::app", eroare = %e, "limba nu s-a salvat: configurarea e stricata");
                }
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
    // N din „Cate N obiecte per fisier": din configurare, schimbabil din ecran
    let cate_per_fisier = Rc::new(std::cell::Cell::new(cfg.obiecte_per_script.max(1)));
    let deseneaza_brut = {
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
            // borrowed", proba pe Windows 2026-11-19); acum a doua asteapta.
            let Ok(mut s) = stare.try_borrow_mut() else {
                tracing::debug!(target: "gmotion::app", "redesenare ceruta din alta redesenare; amanata");
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
            c_ui.set_text_ajutor(gmotion_fereastra::ajutor(limba).into());
            l_ui.set_text_ajutor(gmotion_fereastra::ajutor(limba).into());
            if s.app.is_none() {
                // ATENTIE la ordine: intai se arata fereastra noua, abia apoi
                // se ascund celelalte. Slint inchide bucla cand nu mai e
                // nicio fereastra vizibila, deci „ascunde tot, apoi arata"
                // inchide aplicatia pe loc.
                arata(&c_ui, [l_ui.window(), e_ui.window(), d_ui.window()]);
                // „Aceeasi conexiune": ce s-a schimbat la martor ajunge si la
                // modificata, inainte sa se deseneze
                s.conectare.sincronizeaza();
                incarca_conectare(&c_ui, &s.conectare);
                // progresul compararii, cat ruleaza pe alt fir
                match s.rulare.as_ref() {
                    Some(r) => {
                        c_ui.set_ruleaza(true);
                        c_ui.set_progres(r.fractie());
                        c_ui.set_progres_text(r.text_in(limba).into());
                        c_ui.set_progres_timp(r.timp().into());
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
                        director_deploy(referinta)
                    });
                }
            }
            // referinta cu care a rulat compararea, nu ce scrie acum in
            // formular: formularul se poate schimba intre timp (F-A3)
            let referinta = s.referinta.clone();
            let baze = s
                .app
                .as_ref()
                .map(|a| a.baze())
                .unwrap_or_else(|| vec![s.conectare.baza.baza.clone()]);
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
            // restul vin de pe firul de detalii; obiectul selectat le are pe loc
            if let Some(p) = app.lista().selectat() {
                app.asigura_detalii(p, p + 1);
            }
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
            if let Some(m) = s.mesaj_lista.take() {
                l_ui.set_mesaj_csv(m.into());
            }
            // recompararea ruleaza pe alt fir; lista ramane, dar spune asta (C7)
            if s.rulare.is_some() {
                l_ui.set_mesaj_csv(
                    limba
                        .alege(
                            "Recompar... (lista se actualizeaza cand termina)",
                            "Comparing again... (the list updates when done)",
                        )
                        .into(),
                );
            }
            let app = s.app.as_mut().expect("verificat mai sus");
            if app.refuzat_in_plan() {
                l_ui.set_mesaj_csv(
                    limba
                        .alege(
                            "Nu intra in plan: obiectul n-are cod de deploiat (exista doar in baza modificata sau e criptat).",
                            "Not added to the plan: the object has no code to deploy (it exists only in the modified database or is encrypted).",
                        )
                        .into(),
                );
            }
            match app.texte_obiect() {
                Some((a, b, setari)) => incarca_previzualizare(&l_ui, a, b, &setari),
                None => goleste_previzualizarea(&l_ui),
            }
            gmotion_fereastra::incarca_detalii(&l_ui, &app.panou_detalii().pentru_fereastra());
            if app.ecran() == Ecran::Editor {
                let mut pozitie = format!(
                    "{} {} {} {}",
                    limba.alege("obiectul", "object"),
                    app.lista().selectat().map_or(0, |p| p + 1),
                    limba.alege("din", "of"),
                    app.lista().vizibile().len()
                );
                // ce s-a intamplat cu ultima comanda pe taburi (etapa 16)
                match app.mesaj_tab() {
                    Some(gmotion_app::rutare::MesajTab::Capat) => pozitie.push_str(limba.alege(
                        " · niciun alt obiect nefixat in directia asta",
                        " · no other unpinned object in this direction",
                    )),
                    Some(gmotion_app::rutare::MesajTab::FaraEditor) => pozitie.push_str(limba.alege(
                        " · obiectul ales exista doar intr-o parte sau e criptat: nu se deschide in editor",
                        " · the chosen object exists on one side only or is encrypted: it does not open in the editor",
                    )),
                    Some(gmotion_app::rutare::MesajTab::ToateEditate) => {
                        pozitie.push_str(limba.alege(
                            " · 8 taburi fixate, toate editate: inchide unul intai",
                            " · 8 pinned tabs, all edited: close one first",
                        ))
                    }
                    None => {}
                }
                // `salvat` se ia inainte: `model_editor` tine un imprumut
                // pe `app`, iar `s.salvat_la` e tot din `s`
                let salvat = if s.salvat_la.is_empty() {
                    String::new()
                } else {
                    format!(
                        "{} {}",
                        limba.alege(
                            "Modificarile se salveaza singure, ultima data la",
                            "Changes are saved automatically, last at"
                        ),
                        s.salvat_la
                    )
                };
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
                    gmotion_fereastra::incarca_coloana_lista(&e_ui, app.lista(), app.arbore());
                    gmotion_fereastra::incarca_taburi(&e_ui, app.taburi(), |c| {
                        app.are_text_final(c)
                    });
                    e_ui.set_are_cursor(app.are_cursor());
                    e_ui.set_uneste_cursorul(app.uneste_cursorul());
                    let (editat, _) = app.stare_text_final();
                    let (text_sursa, text_final) = app.texte_curente().unwrap_or_default();
                    drop(model);
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
                arata(&e_ui, [l_ui.window(), d_ui.window(), c_ui.window()]);
            } else if app.ecran() == Ecran::Deploy {
                let sursa = app.sursa().clone();
                if let Some((st, dir)) = s.deploy.as_ref().map(|d| (&d.stare, &d.director)) {
                    // dupa scriere, directorul scris; inainte, cel in care se va scrie
                    let director = match &st.rezultat {
                        Some(Ok(r)) => r.director.display().to_string(),
                        _ => dir.display().to_string(),
                    };
                    let e = gmotion_app::deploy::ecran(
                        st,
                        &sursa,
                        &director,
                        cate_per_fisier.get(),
                        limba,
                    );
                    incarca_deploy(&d_ui, &e);
                }
                arata(&d_ui, [l_ui.window(), e_ui.window(), c_ui.window()]);
            } else {
                arata(&l_ui, [e_ui.window(), d_ui.window(), c_ui.window()]);
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
        move || {
            // coloana listei din editor (etapa 16): starea ei e in fereastra
            let coloana = e_ui.upgrade().map(|u| gmotion_app::rutare::StareColoana {
                latime: u.get_latime_lista_aleasa(),
                fixata: u.get_lista_fixata(),
                dreapta: u.get_lista_dreapta(),
                banda: u.get_lista_banda(),
            });
            let mut s = stare.borrow_mut();
            let referinta = s.referinta.clone();
            // bazele comparate chiar acum, toate, nu doar prima bifata
            let baze = s
                .app
                .as_ref()
                .map(|a| a.baze())
                .unwrap_or_else(|| vec![s.conectare.baza.baza.clone()]);
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
            // o sesiune ilizibila se pune deoparte, nu se scrie peste ea (F-A15)
            let citita = match s.sesiuni.incarca(&server, &referinta) {
                Ok(c) => c,
                Err(e) => {
                    let m = pune_deoparte(&s.sesiuni, &server, &referinta, &e, s.conectare.limba);
                    s.mesaj_lista = Some(m);
                    None
                }
            };
            let mut sesiune = citita.unwrap_or_else(|| {
                gmotion_platforma::Sesiune::noua(server, auth, referinta.clone(), String::new())
            });
            // unire in trei cai cu ce e acum pe disc: ce a scris `gmotion`
            // din linia de comanda cat fereastra era deschisa nu se pierde
            // (auditul 2026-11-23)
            let (baza_rev, baza_ed) = s.salvate_ultima.clone();
            let revizuire = modul::Revizuire::uneste(
                &baza_rev,
                &revizuire,
                &modul::Revizuire::din_sesiune(&sesiune),
            );
            let editari =
                modul::Editari::uneste(&baza_ed, &editari, &modul::Editari::din_sesiune(&sesiune));
            revizuire.in_sesiune(&mut sesiune);
            editari.in_sesiune(&mut sesiune);
            if let Some(a) = s.app.as_mut() {
                a.preia_starea(revizuire.clone(), editari.clone());
            }
            s.salvate_ultima = (revizuire, editari);
            if let Some(a) = s.app.as_ref() {
                gmotion_app::rutare::taburi_in_sesiune(a.taburi(), &mut sesiune);
            }
            if let Some(c) = coloana {
                c.in_sesiune(&mut sesiune);
            }
            // se adauga, nu se inlocuiesc: sesiunea tine munca facuta pe
            // toate bazele comparate vreodata cu referinta asta (C3)
            sesiune.adauga_baze(&baze);
            match s.sesiuni.salveaza(&sesiune) {
                Ok(_) => {
                    // ora, nu data: „ultima data la 14:52"
                    let acum = gmotion_platforma::timp::acum();
                    // ceasul din `platforma::timp` e UTC: se spune, altfel
                    // „14:52" pare ora locala (F-B7)
                    s.salvat_la = format!("{} UTC", acum.split(' ').nth(1).unwrap_or(&acum));
                }
                Err(e) => {
                    tracing::warn!(target: "gmotion::app", eroare = %e, "progresul nu s-a salvat");
                }
            }
        }
    };

    let salveaza = Rc::new(salveaza);

    let comanda = {
        let stare = stare.clone();
        let deseneaza = deseneaza.clone();
        let salveaza = salveaza.clone();
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
                salveaza();
            }
            deseneaza();
            let ms = t.elapsed().as_millis() as u64;
            // starea dupa comanda: unde esti si ce e selectat
            let dupa = stare.borrow().app.as_ref().map(|a| {
                format!(
                    "ecran={:?} obiect={:?} vizibile={} diferenta={}",
                    a.ecran(),
                    a.obiect().map(|o| o.cheie()),
                    a.lista().vizibile().len(),
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

    // --- ecranul de conectare
    {
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_server_schimbat(move |sursa, text| {
            actiune(
                "conectare",
                "server_schimbat",
                &format!("{:?}", (&sursa, &text)),
                false,
            );
            {
                let mut st = s.borrow_mut();
                let Some(p) = st.parte(sursa) else { return };
                p.server = text.to_string();
                // serverul s-a schimbat: ce stiam despre el nu mai e valabil
                p.baze.clear();
                p.baza.clear();
                p.stare = StareConexiune::Neincercat;
                drop(st);
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_auth_schimbat(move |sursa, windows| {
            actiune(
                "conectare",
                "auth_schimbat",
                &format!("{:?}", (&sursa, &windows)),
                false,
            );
            {
                let mut st = s.borrow_mut();
                let Some(p) = st.parte(sursa) else { return };
                p.auth_windows = windows;
                p.baze.clear();
                p.stare = StareConexiune::Neincercat;
                drop(st);
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_utilizator_schimbat(move |sursa, text| {
            actiune(
                "conectare",
                "utilizator_schimbat",
                &format!("{:?}", (&sursa, &text)),
                false,
            );
            {
                if let Some(p) = s.borrow_mut().parte(sursa) {
                    p.acreditari.utilizator = text.to_string();
                }
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_parola_schimbata(move |sursa, text| {
            actiune("conectare", "parola_schimbata", "", false);
            {
                if let Some(p) = s.borrow_mut().parte(sursa) {
                    p.acreditari.pune_parola(text.to_string());
                }
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_baza_schimbata(move |sursa, text| {
            actiune(
                "conectare",
                "baza_schimbata",
                &format!("{:?}", (&sursa, &text)),
                false,
            );
            {
                let mut st = s.borrow_mut();
                if st.parte(sursa).is_none() {
                    return;
                }
                if sursa {
                    st.conectare.sursa.baza = text.to_string();
                } else {
                    // o singura baza si la modificata (etapa 17)
                    st.conectare.alege_baza_modificata(&text);
                }
                drop(st);
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_cere_bazele(move |sursa| {
            actiune(
                "conectare",
                "cere_bazele",
                &format!("{:?}", (&sursa,)),
                false,
            );
            {
                let c = s.borrow().conectare.clone();
                if let Some(ce) = c.ce_lipseste_pentru_baze(sursa) {
                    let mut st = s.borrow_mut();
                    if let Some(tinta) = st.parte(sursa) {
                        tinta.baze = Vec::new();
                        tinta.stare = StareConexiune::Eroare(
                            gmotion_fereastra::model_conectare::completeaza(ce, c.limba),
                        );
                    }
                    drop(st);
                    d();
                    return;
                }
                // pe alt fir: un server care nu raspunde ar tine fereastra pana
                // la timeout (30 s), fara sa se poata apasa nimic (P4)
                let mut st = s.borrow_mut();
                if st.baze_cerute.is_some() {
                    return;
                }
                if let Some(tinta) = st.parte(sursa) {
                    tinta.stare = StareConexiune::SeConecteaza;
                }
                st.baze_cerute = Some((
                    sursa,
                    Lucru::porneste(move || {
                        let p = if sursa { &c.sursa } else { &c.baza };
                        Ok(pornire::citeste_bazele(&c, p))
                    }),
                ));
                drop(st);
                d();
            }
        });
        // cautarea serverelor in retea (SQL Browser), ca `gmotion servere`
        let s = stare.clone();
        let d = deseneaza.clone();
        let timeout = std::time::Duration::from_millis(cfg.timeout_descoperire_ms);
        conectare_ui.on_cauta_servere(move |sursa| {
            actiune(
                "conectare",
                "cauta_servere",
                &format!("{:?}", (&sursa,)),
                false,
            );
            {
                tracing::info!(target: "gmotion::app", "caut servere in retea");
                let mut st = s.borrow_mut();
                if st.cautare_servere.is_some() {
                    return;
                }
                if let Some(p) = st.parte(sursa) {
                    p.stare = StareConexiune::SeCauta;
                }
                st.cautare_servere = Some((
                    sursa,
                    Lucru::porneste(move || {
                        gmotion_db::cauta_instante(timeout)
                            .map(|v| {
                                v.into_iter()
                                    .map(|i| ServerGasit {
                                        nume: i.server,
                                        implicita: i.implicita,
                                        cluster: i.cluster,
                                    })
                                    .collect()
                            })
                            .map_err(|e| e.to_string())
                    }),
                ));
                drop(st);
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_server_ales(move |sursa, server| {
            actiune(
                "conectare",
                "server_ales",
                &format!("{:?}", (&sursa, &server)),
                false,
            );
            {
                let mut st = s.borrow_mut();
                let Some(p) = st.parte(sursa) else { return };
                p.server = server.to_string();
                p.servere_gasite.clear();
                p.baze.clear();
                p.baza.clear();
                p.stare = StareConexiune::Neincercat;
                drop(st);
                d();
            }
        });
        let a = alege_tema.clone();
        let d = deseneaza.clone();
        let al = alege_limba.clone();
        let d2 = deseneaza.clone();
        conectare_ui.on_alege_limba(move |i| {
            actiune("conectare", "alege_limba", &format!("{:?}", (&i,)), false);
            {
                al(i);
                d2();
            }
        });
        conectare_ui.on_alege_tema(move |intunecat| {
            actiune(
                "conectare",
                "alege_tema",
                &format!("{:?}", (&intunecat,)),
                false,
            );
            {
                a(intunecat);
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_aceeasi_conexiune_comutata(move || {
            actiune("conectare", "aceeasi_conexiune_comutata", "", false);
            {
                let mut st = s.borrow_mut();
                if st.rulare.is_some() {
                    return;
                }
                let da = !st.conectare.aceeasi_conexiune;
                st.conectare.pune_aceeasi_conexiune(da);
                drop(st);
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_schimba_certificatul(move || {
            actiune("conectare", "schimba_certificatul", "", false);
            {
                // doua stari, ca in v3.4: nevalidat (implicit) sau depozitul
                // sistemului. Calea unei autoritati se da din linia de comanda.
                let mut st = s.borrow_mut();
                st.conectare.verifica_certificatul = !st.conectare.verifica_certificatul;
                drop(st);
                d();
            }
        });
        let d = deseneaza.clone();
        let c_ui = conectare_ui.as_weak();
        conectare_ui.on_deschide_instantaneu(move || {
            actiune("conectare", "deschide_instantaneu", "", false);
            {
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
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        let c_ui = conectare_ui.as_weak();
        conectare_ui.on_alege_instantaneul(move |cale| {
            actiune(
                "conectare",
                "alege_instantaneul",
                &format!("{:?}", (&cale,)),
                false,
            );
            {
                let cale = cale.trim().to_string();
                if cale.is_empty() {
                    return;
                }
                let mut st = s.borrow_mut();
                st.conectare.instantaneu = Some(cale.clone());
                // bazele vin din fisier: daca nu se citeste, se spune pe loc
                let c = st.conectare.clone();
                let (baze, stare_noua) = pornire::citeste_bazele(&c, &c.sursa);
                let reusit = stare_noua.e_conectat();
                st.conectare.sursa.baze.clone_from(&baze);
                st.conectare.baza.baze = baze;
                st.conectare.sursa.stare = stare_noua.clone();
                st.conectare.baza.stare = stare_noua;
                if !reusit {
                    st.conectare.instantaneu = None;
                } else {
                    st.conectare.sursa.baza = String::new();
                    st.conectare.baza.alese.clear();
                    st.conectare.baza.baza = String::new();
                }
                drop(st);
                if let Some(ui) = c_ui.upgrade() {
                    if reusit {
                        ui.set_alege_fisierul(false);
                    }
                }
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_compara(move || {
            actiune("conectare", "compara", "", false);
            {
                let c = s.borrow().conectare.clone();
                tracing::info!(target: "gmotion::app", "pornesc compararea");
                s.borrow_mut().rulare = Some(gmotion_app::fir::Rulare::porneste(c));
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_testeaza(move |sursa| {
            actiune("conectare", "testeaza", &format!("{:?}", (&sursa,)), false);
            {
                let c = s.borrow().conectare.clone();
                tracing::info!(target: "gmotion::app", sursa, "testez conexiunea");
                let mut st = s.borrow_mut();
                if st.testare.is_some() {
                    return;
                }
                if let Some(p) = st.parte(sursa) {
                    p.stare = StareConexiune::SeConecteaza;
                }
                st.testare = Some((
                    sursa,
                    Lucru::porneste(move || {
                        let p = if sursa { &c.sursa } else { &c.baza };
                        Ok(match pornire::testeaza(&c, p) {
                            Ok(t) => StareConexiune::Conectat(t),
                            Err(e) => StareConexiune::Eroare(e),
                        })
                    }),
                ));
                drop(st);
                d();
            }
        });
        let d = deseneaza.clone();
        conectare_ui.on_sterge_jurnalul(move || {
            actiune("conectare", "sterge_jurnalul", "", false);
            {
                gmotion_platforma::jurnal::goleste();
                d();
            }
        });
        let c_ui = conectare_ui.as_weak();
        let d = deseneaza.clone();
        conectare_ui.on_deschide_offline(move || {
            actiune("conectare", "deschide_offline", "", false);
            {
                if let Some(ui) = c_ui.upgrade() {
                    ui.set_offline(true);
                    ui.set_offline_eroare(Default::default());
                }
                d();
            }
        });
        let c_ui = conectare_ui.as_weak();
        let d = deseneaza.clone();
        conectare_ui.on_inchide_offline(move || {
            actiune("conectare", "inchide_offline", "", false);
            {
                if let Some(ui) = c_ui.upgrade() {
                    ui.set_offline(false);
                }
                d();
            }
        });
        let s = stare.clone();
        let c_ui = conectare_ui.as_weak();
        let d = deseneaza.clone();
        conectare_ui.on_compara_offline(move |a, b| {
            actiune(
                "conectare",
                "compara_offline",
                &format!("{:?}", (&a, &b)),
                false,
            );
            {
                // limba se ia inainte: un `borrow` in conditia lui `match` ar
                // tine pana la capat si ar cadea la `borrow_mut` de dedesubt
                let limba = s.borrow().conectare.limba;
                match pornire::compara_offline(&a, &b, limba) {
                    Ok(r) => {
                        let mut st = s.borrow_mut();
                        st.server = r.server;
                        st.referinta = r.referinta.clone();
                        // fara stare citita: unirea ia de pe disc tot ce nu e al ei
                        st.salvate_ultima = Default::default();
                        let mut app =
                            Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());
                        // „Uneste ciclul cursor" porneste cum spune configurarea (F-B5)
                        app.comanda(Comanda::ComutaUnireaCursorului(uneste_implicit));
                        st.detalii = Some(porneste_detalii(&app));
                        st.app = Some(app);
                        // formularul de conectare ramane cum era: antetul ia
                        // bazele din aplicatie, iar „Recompara" nu porneste o
                        // comparare pe server cu numele lipite (auditul
                        // 2026-11-23)
                        st.offline = true;
                        drop(st);
                        if let Some(ui) = c_ui.upgrade() {
                            ui.set_offline(false);
                        }
                    }
                    Err(e) => {
                        if let Some(ui) = c_ui.upgrade() {
                            ui.set_offline_eroare(e.into());
                        }
                    }
                }
                d();
            }
        });
        conectare_ui.on_ajutor(|| {
            tracing::info!(target: "gmotion::app", "deschid ajutorul");
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_ignora_recenta(move |i| { actiune("conectare", "ignora_recenta", &format!("{:?}", (&i,)), false); {
            let mut st = s.borrow_mut();
            if let Some(r) = st
                .conectare
                .recente
                .get(usize::try_from(i).unwrap_or(0))
                .cloned()
            {
                match st.sesiuni.sterge(&r.server, &r.sursa) {
                    Ok(_) => tracing::info!(target: "gmotion::app", referinta = %r.sursa, "comparare ignorata"),
                    Err(e) => tracing::warn!(target: "gmotion::app", eroare = %e, "nu pot sterge sesiunea"),
                }
                let recente_noi = recente(&st.sesiuni, st.conectare.limba);
                st.conectare.recente = recente_noi;
            }
            drop(st);
            d();
        } });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_anuleaza(move || {
            actiune("conectare", "anuleaza", "", false);
            {
                if let Some(r) = s.borrow_mut().rulare.as_mut() {
                    r.anuleaza();
                }
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        conectare_ui.on_reia(move |i| {
            actiune("conectare", "reia", &format!("{:?}", (&i,)), false);
            {
                let mut st = s.borrow_mut();
                if let Some(r) = st
                    .conectare
                    .recente
                    .get(usize::try_from(i).unwrap_or(0))
                    .cloned()
                {
                    st.conectare.reia(&r);
                }
                drop(st);
                d();
            }
        });
    }

    // --- lista
    {
        // harta (`h`): deschiderea, reasezarea si bula sunt ale ferestrei;
        // clicul pe patrat schimba selectia, deci trece prin `Comanda`
        gmotion_fereastra::leaga_harta(&lista_ui);
        let c = comanda.clone();
        let l = lista_ui.as_weak();
        lista_ui.on_clic_harta(move |grup, x, y| {
            actiune(
                "lista",
                "clic_harta",
                &format!("{:?}", (&grup, &x, &y)),
                false,
            );
            let Some(ui) = l.upgrade() else { return };
            if let Some(r) = gmotion_fereastra::rand_din_harta(&ui, grup, x, y) {
                // harta se inchide inainte, ca redesenarea sa nu o mai picteze
                ui.set_arata_harta(false);
                c(Comanda::Selecteaza(r));
            }
        });
        let c = comanda.clone();
        lista_ui.on_alege_randul(move |i| {
            actiune("lista", "alege_randul", &format!("{:?}", (&i,)), false);
            c(Comanda::Selecteaza(i.max(0) as usize))
        });
        let c = comanda.clone();
        let dc = RefCell::new(DubluClic::default());
        lista_ui.on_clic_rand(move |i| {
            actiune("lista", "clic_rand", &format!("{:?}", (&i,)), false);
            c(Comanda::ClicPeRand(i.max(0) as usize));
            // al doilea clic repede pe acelasi rand deschide obiectul (ca si
            // inainte: primul clic deschide detaliile, al doilea le inchide)
            if dc.borrow_mut().apasat(i, Instant::now()) {
                tracing::debug!(target: "gmotion::actiuni", rand = i, "dublu clic in lista");
                c(Comanda::Selecteaza(i.max(0) as usize));
                c(Comanda::Deschide);
            }
        });
        let c = comanda.clone();
        lista_ui.on_inchide_detalii(move || {
            actiune("lista", "inchide_detalii", "", false);
            c(Comanda::InchideDetalii)
        });
        let c = comanda.clone();
        lista_ui.on_deschide_obiectul(move || {
            actiune("lista", "deschide_obiectul", "", false);
            c(Comanda::Deschide)
        });
        let c = comanda.clone();
        lista_ui.on_comuta_omis(move || {
            actiune("lista", "comuta_omis", "", false);
            c(Comanda::ComutaOmis)
        });
        let c = comanda.clone();
        lista_ui.on_nota_editata(move |baza, id, text| {
            actiune(
                "lista",
                "nota_editata",
                &format!("{:?}", (&baza, &id, &text)),
                false,
            );
            {
                c(Comanda::PuneNota(
                    baza.to_string(),
                    id.to_string(),
                    text.to_string(),
                ))
            }
        });
        lista_ui.on_ajutor(|| {
            tracing::info!(target: "gmotion::app", "deschid ajutorul");
        });
        let l = lista_ui.as_weak();
        lista_ui.on_cere_deconectarea(move || {
            actiune("lista", "cere_deconectarea", "", false);
            {
                if let Some(ui) = l.upgrade() {
                    ui.set_intreaba_deconectarea(true);
                }
            }
        });
        let l = lista_ui.as_weak();
        lista_ui.on_renunta_la_deconectare(move || {
            actiune("lista", "renunta_la_deconectare", "", false);
            {
                if let Some(ui) = l.upgrade() {
                    ui.set_intreaba_deconectarea(false);
                }
            }
        });
        let s = stare.clone();
        let l = lista_ui.as_weak();
        let sv = salveaza.clone();
        let d = deseneaza.clone();
        lista_ui.on_deconecteaza(move || {
            actiune("lista", "deconecteaza", "", false);
            {
                // progresul se salveaza inainte de a arunca starea: altfel
                // „Deconecteaza" ar insemna „pierde ce ai revizuit"
                sv();
                let mut st = s.borrow_mut();
                st.app = None;
                st.detalii = None;
                st.deploy = None;
                st.rulare = None;
                st.referinta = String::new();
                // nimic din conexiunea de dinainte nu ramane pe ecran (F-A16)
                st.salvat_la = String::new();
                st.garda = None;
                st.mesaj_lista = None;
                st.salvate_ultima = Default::default();
                st.offline = false;
                let recente_noi = recente(&st.sesiuni, st.conectare.limba);
                st.conectare.recente = recente_noi;
                drop(st);
                if let Some(ui) = l.upgrade() {
                    ui.set_intreaba_deconectarea(false);
                    ui.set_mesaj_csv("".into());
                    ui.set_cautare("".into());
                }
                tracing::info!(target: "gmotion::app", "deconectat");
                d();
            }
        });
        // recompararea: aceeasi conectare, revizuirea de pana acum ramane
        let s = stare.clone();
        let d = deseneaza.clone();
        lista_ui.on_recompara(move || {
            actiune("lista", "recompara", "", false);
            {
                // un al doilea clic cat ruleaza ar porni inca un fir, iar
                // primul ar arde CPU pana la capat (C7)
                if s.borrow().rulare.is_some() {
                    return;
                }
                if s.borrow().offline {
                    let mut st = s.borrow_mut();
                    let l = st.conectare.limba;
                    st.mesaj_lista = Some(
                        l.alege(
                            "O comparare lipita nu se reface: lipeste din nou textele.",
                            "A pasted comparison cannot be rerun: paste the texts again.",
                        )
                        .to_string(),
                    );
                    drop(st);
                    d();
                    return;
                }
                let c = s.borrow().conectare.clone();
                tracing::info!(target: "gmotion::app", "recompar");
                // acelasi drum ca prima comparare: pe alt fir, cu progres. Ceasul
                // pastreaza revizuirea de pana acum.
                s.borrow_mut().rulare = Some(gmotion_app::fir::Rulare::porneste(c));
                d();
            }
        });
        let a = alege_tema.clone();
        let d = deseneaza.clone();
        let al = alege_limba.clone();
        let d2 = deseneaza.clone();
        lista_ui.on_alege_limba(move |i| {
            actiune("lista", "alege_limba", &format!("{:?}", (&i,)), false);
            {
                al(i);
                d2();
            }
        });
        lista_ui.on_alege_tema(move |intunecat| {
            actiune(
                "lista",
                "alege_tema",
                &format!("{:?}", (&intunecat,)),
                false,
            );
            {
                a(intunecat);
                d();
            }
        });
        let c = comanda.clone();
        lista_ui.on_comuta_revizuit(move |i| {
            actiune("lista", "comuta_revizuit", &format!("{:?}", (&i,)), false);
            {
                // indicele randului apasat, nu cel selectat (V9)
                match usize::try_from(i) {
                    Ok(p) => c(Comanda::ComutaRevizuitLa(p)),
                    Err(_) => c(Comanda::ComutaRevizuit),
                }
            }
        });
        let c = comanda.clone();
        lista_ui.on_cauta(move |t| {
            actiune("lista", "cauta", &format!("{:?}", (&t,)), false);
            c(Comanda::Cauta(t.to_string()))
        });
        let c = comanda.clone();
        lista_ui.on_alege_categoria(move |i| {
            actiune("lista", "alege_categoria", &format!("{:?}", (&i,)), false);
            {
                c(Comanda::Filtreaza(
                    usize::try_from(i)
                        .ok()
                        .and_then(|i| StareObiect::TOATE.get(i).copied()),
                ))
            }
        });
        let c = comanda.clone();
        let s = stare.clone();
        lista_ui.on_alege_tipul(move |i| {
            actiune("lista", "alege_tipul", &format!("{:?}", (&i,)), false);
            {
                let tip = usize::try_from(i).ok().and_then(|i| {
                    s.borrow()
                        .app
                        .as_ref()
                        .and_then(|a| a.lista().contoare_tipuri().get(i).map(|(t, _)| t.clone()))
                });
                c(Comanda::FiltreazaTip(tip))
            }
        });
        let c = comanda.clone();
        // clic pe un cap de coloana: sorteaza dupa ea; al doilea clic inverseaza
        lista_ui.on_sorteaza_coloana(move |cod| {
            actiune(
                "lista",
                "sorteaza_coloana",
                &format!("{:?}", (&cod,)),
                false,
            );
            {
                c(Comanda::Sorteaza(gmotion_fereastra::ordine_din_cod(cod)));
            }
        });
        let s = stare.clone();
        let l = lista_ui.as_weak();
        let d = deseneaza.clone();
        let dc = RefCell::new(DubluClic::default());
        lista_ui.on_clic_margine_coloana(move |cod| {
            actiune(
                "lista",
                "clic_margine_coloana",
                &format!("{:?}", (&cod,)),
                false,
            );
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
        let c = comanda.clone();
        lista_ui.on_alege_baza(move |i| {
            actiune("lista", "alege_baza", &format!("{:?}", (&i,)), false);
            {
                let baza = usize::try_from(i).ok().and_then(|i| {
                    s.borrow()
                        .app
                        .as_ref()
                        .and_then(|a| a.lista().contoare_baze().get(i).map(|(b, _)| b.clone()))
                });
                c(Comanda::FiltreazaBaza(baza))
            }
        });
        let c = comanda.clone();
        lista_ui.on_alege_starea(move |i| {
            actiune("lista", "alege_starea", &format!("{:?}", (&i,)), false);
            {
                let f = usize::try_from(i)
                    .ok()
                    .and_then(|i| FiltruRevizuire::TOATE.get(i).copied());
                c(Comanda::FiltreazaRevizuire(f));
            }
        });
        let c = comanda.clone();
        lista_ui.on_alege_impactul(move |i| {
            actiune("lista", "alege_impactul", &format!("{:?}", (&i,)), false);
            let f = usize::try_from(i).ok().and_then(|i| {
                gmotion_fereastra::model_lista::Impact::TOATE
                    .get(i)
                    .copied()
            });
            c(Comanda::FiltreazaImpact(f));
        });
        let c = comanda.clone();
        lista_ui.on_alege_dependenta(move |nume| {
            actiune(
                "lista",
                "alege_dependenta",
                &format!("{:?}", (&nume,)),
                false,
            );
            c(Comanda::SelecteazaDependenta(nume.to_string()));
        });
        let c = comanda.clone();
        lista_ui.on_schimba_scorul(move |min, max| {
            actiune(
                "lista",
                "schimba_scorul",
                &format!("{:?}", (&min, &max)),
                false,
            );
            {
                // campurile sunt numerice, dar pot fi goale cat se scrie in ele
                let n = |t: &str, implicit: f64| t.trim().parse::<f64>().unwrap_or(implicit);
                c(Comanda::FiltreazaScor(n(&min, 0.0), n(&max, 100.0)));
            }
        });
        let c = comanda.clone();
        lista_ui.on_pune_vizibilele_in_plan(move |cu| {
            actiune(
                "lista",
                "pune_vizibilele_in_plan",
                &format!("{:?}", (&cu,)),
                false,
            );
            c(Comanda::PuneVizibileleInPlan(cu))
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        let l = lista_ui.as_weak();
        lista_ui.on_exporta_csv(move || { actiune("lista", "exporta_csv", "", false); {
            let st = s.borrow();
            let Some(app) = st.app.as_ref() else {
                return;
            };
            let cale = cale_csv(&st.referinta);
            let mesaj = match cale.parent().map(std::fs::create_dir_all) {
                Some(Err(e)) => format!(
                    "{}: {e}",
                    st.conectare.limba.alege("nu pot scrie", "cannot write")
                ),
                _ => match app.exporta_csv(&cale) {
                    Ok(n) => {
                        tracing::info!(target: "gmotion::app", cale = %cale.display(), randuri = n, "CSV scris");
                        format!(
                            "{n} {} {}",
                            st.conectare.limba.alege("randuri in", "rows in"),
                            cale.display()
                        )
                    }
                    Err(e) => e,
                },
            };
            drop(st);
            if let Some(l) = l.upgrade() {
                l.set_mesaj_csv(mesaj.into());
            }
            d();
        } });
    }

    // --- planul de deploy
    {
        let c = comanda.clone();
        lista_ui.on_adauga_in_plan(move || {
            actiune("lista", "adauga_in_plan", "", false);
            c(Comanda::ComutaInPlan)
        });
        let c = comanda.clone();
        lista_ui.on_deschide_deploy(move || {
            actiune("lista", "deschide_deploy", "", false);
            c(Comanda::DeschideDeploy)
        });
        let c = comanda.clone();
        editor_ui.on_adauga_in_plan(move || {
            actiune("editor", "adauga_in_plan", "", false);
            c(Comanda::ComutaInPlan)
        });
        let c = comanda.clone();
        deploy_ui.on_inapoi_lista(move || {
            actiune("deploy", "inapoi_lista", "", false);
            c(Comanda::InapoiLaLista)
        });
        let c = comanda.clone();
        deploy_ui.on_scoate_din_plan(move |baza, id| {
            actiune(
                "deploy",
                "scoate_din_plan",
                &format!("{:?}", (&baza, &id)),
                false,
            );
            {
                c(Comanda::ScoateDinPlan(baza.to_string(), id.to_string()))
            }
        });
        let a = alege_tema.clone();
        let d = deseneaza.clone();
        let al = alege_limba.clone();
        let d2 = deseneaza.clone();
        deploy_ui.on_alege_limba(move |i| {
            actiune("deploy", "alege_limba", &format!("{:?}", (&i,)), false);
            {
                al(i);
                d2();
            }
        });
        deploy_ui.on_alege_tema(move |intunecat| {
            actiune(
                "deploy",
                "alege_tema",
                &format!("{:?}", (&intunecat,)),
                false,
            );
            {
                a(intunecat);
                d();
            }
        });
        // garda: se recitesc ambele baze si se compara amprentele
        let s = stare.clone();
        let d = deseneaza.clone();
        deploy_ui.on_verifica_baza(move || {
            actiune("deploy", "verifica_baza", "", false);
            {
                let c = s.borrow().conectare.clone();
                tracing::info!(target: "gmotion::app", "garda: recitesc bazele");
                let mut st = s.borrow_mut();
                if st.garda.is_some() {
                    return;
                }
                if let Some(dep) = st.deploy.as_mut().map(|d| &mut d.stare) {
                    dep.verificarea_a_pornit();
                }
                // `reciteste`, nu `compara`: garzii ii trebuie doar obiectele de
                // acum, iar compararea intreaga tinea minute pe o baza mare (P4)
                st.garda = Some(Lucru::porneste(move || pornire::reciteste(&c)));
                drop(st);
                d();
            }
        });
        let s = stare.clone();
        let d = deseneaza.clone();
        let sv = salveaza.clone();
        deploy_ui.on_scrie_scripturile(move || { actiune("deploy", "scrie_scripturile", "", false); {
            let mut st = s.borrow_mut();
            let referinta = st.referinta.clone();
            let mut scris = None;
            // textele finale se iau acum, din aplicatie, nu dintr-un
            // instantaneu luat la deschiderea planului (C1 din auditul QA)
            let texte = st.app.as_ref().map(|a| a.texte_finale()).unwrap_or_default();
            if let Some(d) = st.deploy.as_mut() {
                let dir = d.director.clone();
                let dep = &mut d.stare;
                dep.scrie(&dir, &gmotion_platforma::timp::acum(), &texte);
                if matches!(dep.rezultat, Some(Ok(_))) {
                    // o a doua scriere (alta impartire) merge in alt director
                    d.director = director_deploy(&referinta);
                }
                let dep = &d.stare;
                if let Some(Ok(r)) = &dep.rezultat {
                    tracing::info!(target: "gmotion::app", director = %r.director.display(), fisiere = r.fisiere.len(), "deploy scris");
                    scris = Some(dep.plan.clone());
                }
            }
            // obiectele scrise devin „deployate" in lista si in sesiune;
            // doar dupa o scriere reusita, nu dupa o incercare
            let de_salvat = match (scris, st.app.as_mut()) {
                (Some(plan), Some(app)) => app.marcheaza_deployate(&plan),
                _ => false,
            };
            drop(st);
            if de_salvat {
                sv();
            }
            d();
        } });
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
        let c = comanda.clone();
        editor_ui.on_inapoi_lista(move || {
            actiune("editor", "inapoi_lista", "", false);
            c(Comanda::InapoiLaLista)
        });
        let c = comanda.clone();
        editor_ui.on_obiect_urmator(move || {
            actiune("editor", "obiect_urmator", "", false);
            c(Comanda::ObiectUrmator)
        });
        let c = comanda.clone();
        editor_ui.on_obiect_anterior(move || {
            actiune("editor", "obiect_anterior", "", false);
            c(Comanda::ObiectAnterior)
        });
        let c = comanda.clone();
        editor_ui.on_comuta_revizuit(move || {
            actiune("editor", "comuta_revizuit", "", false);
            c(Comanda::ComutaRevizuit)
        });
        // clic in coloana din stanga: schimba obiectul pe loc, fara sa pleci
        // din editor (pachetul B)
        let c = comanda.clone();
        let dc = RefCell::new(DubluClic::default());
        editor_ui.on_alege_randul(move |i| {
            actiune("editor", "alege_randul", &format!("{:?}", (&i,)), false);
            c(Comanda::Selecteaza(i.max(0) as usize));
            // dublu clic in coloana: obiectul ramane intr-un tab al lui
            if dc.borrow_mut().apasat(i, Instant::now()) {
                tracing::debug!(target: "gmotion::actiuni", rand = i, "dublu clic in coloana");
                c(Comanda::FixeazaSelectia);
            }
        });
        // ---- taburile (etapa 16); regulile sunt in `rutare` / `model_taburi`
        let c = comanda.clone();
        editor_ui.on_fixeaza_randul(move |i| {
            actiune("editor", "fixeaza_randul", &format!("{:?}", (&i,)), false);
            {
                c(Comanda::Selecteaza(i.max(0) as usize));
                c(Comanda::FixeazaSelectia);
            }
        });
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
        let c = comanda.clone();
        editor_ui.on_comuta_nod(move |cheie| {
            actiune("editor", "comuta_nod", &format!("{:?}", (&cheie,)), false);
            c(Comanda::ComutaNod(cheie.to_string()))
        });
        let c = comanda.clone();
        editor_ui.on_inchide_tab(move |cod| {
            actiune("editor", "inchide_tab", &format!("{:?}", (&cod,)), false);
            c(Comanda::InchideTab(usize::try_from(cod).ok()))
        });
        let c = comanda.clone();
        editor_ui.on_fixeaza_previzualizarea(move || {
            actiune("editor", "fixeaza_previzualizarea", "", false);
            c(Comanda::FixeazaPrevizualizarea)
        });
        let c = comanda.clone();
        editor_ui.on_inchide_tab_activ(move || {
            actiune("editor", "inchide_tab_activ", "", false);
            c(Comanda::InchideTabActiv)
        });
        // Deploy (N) din bara de sus a editorului (etapa 18)
        let c = comanda.clone();
        editor_ui.on_deschide_deploy(move || {
            actiune("editor", "deschide_deploy", "", false);
            c(Comanda::DeschideDeploy)
        });
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
        let c = comanda.clone();
        editor_ui.on_cauta(move |t| {
            actiune("editor", "cauta", &format!("{:?}", (&t,)), false);
            // doar arborele din coloana: obiectul deschis ramane
            c(Comanda::CautaInArbore(t.to_string()))
        });
        let e = editor_ui.as_weak();
        let d = deseneaza.clone();
        // `[` (si «): coloana devine banda ingusta cu contoare, sau inapoi
        // (etapa 16); fara coloana deloc, `[` o readuce la latimea trasa
        let sv = salveaza.clone();
        editor_ui.on_ascunde_lista(move || {
            actiune("editor", "ascunde_lista", "", false);
            {
                if let Some(ui) = e.upgrade() {
                    if ui.get_latime_lista() > 0.0 {
                        ui.set_lista_banda(!ui.get_lista_banda());
                    } else {
                        ui.set_latime_lista(ui.get_latime_lista_aleasa());
                    }
                }
                sv();
                d();
            }
        });
        // pin, partea, banda, latimea: se tin minte in sesiune
        let sv = salveaza.clone();
        let d = deseneaza.clone();
        editor_ui.on_coloana_schimbata(move || {
            actiune("editor", "coloana_schimbata", "", false);
            {
                sv();
                d();
            }
        });
        // plierea si alaturat / unificat stau in fereastra; modelul se
        // reconstruieste la redesenare, din ce arata fereastra acum
        let c = comanda.clone();
        editor_ui.on_comuta_omis(move || {
            actiune("editor", "comuta_omis", "", false);
            c(Comanda::ComutaOmis)
        });
        let c = comanda.clone();
        editor_ui.on_comuta_unirea_cursorului(move |cu| {
            actiune(
                "editor",
                "comuta_unirea_cursorului",
                &format!("{:?}", (&cu,)),
                false,
            );
            c(Comanda::ComutaUnireaCursorului(cu))
        });
        let c = comanda.clone();
        editor_ui
            .on_aplica_diferenta(move |k| c(Comanda::AplicaDiferenta(usize::try_from(k).ok())));
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
        let c = comanda.clone();
        editor_ui.on_aplica_toate(move || {
            actiune("editor", "aplica_toate", "", false);
            c(Comanda::AplicaToate)
        });
        let c = comanda.clone();
        editor_ui.on_reset(move || {
            actiune("editor", "reset", "", false);
            c(Comanda::Reset)
        });
        let c = comanda.clone();
        editor_ui.on_anuleaza(move || {
            actiune("editor", "anuleaza", "", false);
            c(Comanda::Anuleaza)
        });
        let c = comanda.clone();
        editor_ui.on_salveaza_textul(move |id, t| {
            actiune(
                "editor",
                "salveaza_textul",
                &format!("{:?}", (&id, &t)),
                false,
            );
            {
                c(Comanda::SalveazaText {
                    id: id.to_string(),
                    text: t.to_string(),
                })
            }
        });
        let c = comanda.clone();
        editor_ui.on_ghidat_porneste(move || {
            actiune("editor", "ghidat_porneste", "", false);
            c(Comanda::PornesteGhidat)
        });
        let c = comanda.clone();
        editor_ui.on_ghidat_opreste(move || {
            actiune("editor", "ghidat_opreste", "", false);
            c(Comanda::OpresteGhidat)
        });
        let c = comanda.clone();
        editor_ui.on_ghidat_alege(move |din_sursa| {
            actiune(
                "editor",
                "ghidat_alege",
                &format!("{:?}", (&din_sursa,)),
                false,
            );
            c(Comanda::GhidatAlege(din_sursa))
        });
        let c = comanda.clone();
        editor_ui.on_ghidat_aplica(move || {
            actiune("editor", "ghidat_aplica", "", false);
            c(Comanda::GhidatAplica)
        });
        let c = comanda.clone();
        editor_ui.on_ghidat_sari(move || {
            actiune("editor", "ghidat_sari", "", false);
            c(Comanda::GhidatSari)
        });
        // „Re-sincronizeaza": se compara din nou textul curent (modelul se
        // reconstruieste la redesenare)
        let d = deseneaza.clone();
        editor_ui.on_resincronizeaza(move || {
            actiune("editor", "resincronizeaza", "", false);
            {
                tracing::info!(target: "gmotion::app", "re-sincronizez obiectul");
                d();
            }
        });
        editor_ui.on_editeaza_textul_final(|| {
            tracing::info!(target: "gmotion::app", "editez textul final de mana");
        });
        // n / p muta doar desenul (fara redesenare: pe un obiect mare,
        // reconstruirea modelului la fiecare tasta s-ar simti); aplicatia
        // tine minte pozitia ca s-o pastreze dupa „Aplica"
        let s = stare.clone();
        editor_ui.on_diferenta_schimbata(move |k| {
            actiune(
                "editor",
                "diferenta_schimbata",
                &format!("{:?}", (&k,)),
                true,
            );
            {
                if let Some(a) = s.borrow_mut().app.as_mut() {
                    a.comanda(Comanda::MergiLaDiferenta(k.max(0) as usize));
                }
            }
        });
        let d = deseneaza.clone();
        let cale = gmotion_platforma::cai::cale_configurare().ok();
        editor_ui.on_comuta_plierea(move |pliat| {
            actiune("editor", "comuta_plierea", &format!("{:?}", (&pliat,)), false);
            // tinuta minte ca tema: se reciteste configurarea, ca sa nu
            // suprascrie ce s-a schimbat de mana intre timp
            if let Some(cale) = cale.as_ref() {
                match gmotion_platforma::Configurare::citeste(cale) {
                    Ok(mut c) => {
                        c.pliaza_identicul = pliat;
                        if let Err(e) = c.scrie(cale) {
                            tracing::warn!(target: "gmotion::app", eroare = %e, "plierea nu s-a salvat");
                        }
                    }
                    Err(e) => {
                        tracing::warn!(target: "gmotion::app", eroare = %e, "plierea nu s-a salvat: configurarea e stricata");
                    }
                }
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
        // amanat: `comuta-wrap` vine si din `changed coloane-wrap`, adica din
        // mijlocul unei redesenari (cand Rust citeste latimea)
        editor_ui.on_comuta_wrap(move |pornit| {
            actiune("editor", "comuta_wrap", &format!("{:?}", (&pornit,)), false);
            slint::Timer::single_shot(std::time::Duration::ZERO, d.clone());
        });
        let a = alege_tema.clone();
        let d = deseneaza.clone();
        let al = alege_limba.clone();
        let d2 = deseneaza.clone();
        editor_ui.on_alege_limba(move |i| {
            actiune("editor", "alege_limba", &format!("{:?}", (&i,)), false);
            {
                al(i);
                d2();
            }
        });
        editor_ui.on_alege_tema(move |intunecat| {
            actiune(
                "editor",
                "alege_tema",
                &format!("{:?}", (&intunecat,)),
                false,
            );
            {
                a(intunecat);
                // hasura e pictata in culorile modului
                d();
            }
        });
    }

    // ceasul care aduce progresul compararii de pe celalalt fir. Bate tot
    // timpul, dar nu face nimic cat nu ruleaza nimic.
    let ceas = slint::Timer::default();
    {
        let s = stare.clone();
        let d = deseneaza.clone();
        ceas.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(120),
            move || {
                // celelalte lucruri de pe alte fire: bazele, testul,
                // serverele, garda. Fiecare isi pune rezultatul in model si
                // isi goleste slotul (P4).
                let mut s_a_schimbat = false;
                {
                    let mut st = s.borrow_mut();
                    if let Some((sursa, l)) = st.baze_cerute.as_mut() {
                        let (sursa, gata) = (*sursa, l.verifica());
                        if let Some(r) = gata {
                            st.baze_cerute = None;
                            s_a_schimbat = true;
                            let (baze, stare_noua) = match r {
                                Ok(x) => x,
                                Err(e) => (Vec::new(), StareConexiune::Eroare(e)),
                            };
                            tracing::info!(target: "gmotion::app", sursa, baze = baze.len(), stare = ?stare_noua, "baze citite");
                            if let Some(p) = st.parte(sursa) {
                                p.baze = baze;
                                p.stare = stare_noua;
                            }
                        }
                    }
                    if let Some((sursa, l)) = st.testare.as_mut() {
                        let (sursa, gata) = (*sursa, l.verifica());
                        if let Some(r) = gata {
                            st.testare = None;
                            s_a_schimbat = true;
                            let stare_noua = r.unwrap_or_else(StareConexiune::Eroare);
                            tracing::info!(target: "gmotion::app", sursa, stare = ?stare_noua, "testul conexiunii s-a terminat");
                            if let Some(p) = st.parte(sursa) {
                                p.stare = stare_noua;
                            }
                        }
                    }
                    let st_limba = st.conectare.limba;
                    if let Some((sursa, l)) = st.cautare_servere.as_mut() {
                        let (sursa, gata) = (*sursa, l.verifica());
                        if let Some(r) = gata {
                            st.cautare_servere = None;
                            s_a_schimbat = true;
                            match r {
                                Ok(v) if v.is_empty() => {
                                    tracing::info!(target: "gmotion::app", "niciun server gasit in retea");
                                    if let Some(p) = st.parte(sursa) {
                                        p.servere_gasite.clear();
                                        let l = st_limba;
                                        p.stare = StareConexiune::Eroare(
                                            l.alege(
                                                "niciun server nu a raspuns in retea; scrie-l de mana",
                                                "no server answered on the network; type it in",
                                            )
                                            .into(),
                                        );
                                    }
                                }
                                Ok(v) => {
                                    tracing::info!(target: "gmotion::app", gasite = v.len(), "servere gasite");
                                    if let Some(p) = st.parte(sursa) {
                                        p.servere_gasite = v;
                                        p.stare = StareConexiune::Neincercat;
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(target: "gmotion::app", eroare = %e, "cautarea serverelor a esuat");
                                    if let Some(p) = st.parte(sursa) {
                                        p.servere_gasite.clear();
                                        p.stare = StareConexiune::Eroare(e);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(l) = st.garda.as_mut() {
                        if let Some(r) = l.verifica() {
                            st.garda = None;
                            s_a_schimbat = true;
                            let acum = gmotion_platforma::timp::acum();
                            tracing::info!(target: "gmotion::app", reusita = r.is_ok(), "verificarea de garda s-a terminat");
                            if let Some(dep) = st.deploy.as_mut().map(|d| &mut d.stare) {
                                match r {
                                    Ok(x) => dep.aplica_verificarea(x.sursa, x.tinte, &acum),
                                    Err(e) => dep.verificare_esuata(e),
                                }
                            }
                        }
                    }
                }
                // detaliile de pe alt fir: cate au venit, pe obiectul lor
                {
                    let mut st = s.borrow_mut();
                    let st = &mut *st;
                    if let (Some(rx), Some(app)) = (st.detalii.as_ref(), st.app.as_mut()) {
                        let mut lot = Vec::new();
                        let mut gata = false;
                        loop {
                            match rx.try_recv() {
                                Ok((cheie, d)) => lot.push((cheie, d)),
                                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                    gata = true;
                                    break;
                                }
                            }
                        }
                        if gata {
                            tracing::debug!(target: "gmotion::app", "detaliile listei sunt calculate");
                            st.detalii = None;
                        }
                        s_a_schimbat |= !lot.is_empty();
                        app.pune_detalii_multe(lot);
                    }
                }
                if s_a_schimbat {
                    d();
                }
                let gata = {
                    let mut st = s.borrow_mut();
                    match st.rulare.as_mut() {
                        Some(r) => r.verifica(),
                        None => return,
                    }
                };
                match gata {
                    // inca ruleaza: doar redesenam bara si ceasul
                    None => d(),
                    Some(rezultat) => {
                        let mut st = s.borrow_mut();
                        st.rulare = None;
                        match rezultat {
                            Ok(r) => {
                                tracing::info!(
                                    target: "gmotion::app",
                                    obiecte = r.randuri.len(),
                                    recomparare = st.app.is_some(),
                                    "compararea s-a terminat"
                                );
                                let referinta = r.referinta.clone();
                                // la recomparare pastram ce s-a revizuit deja;
                                // la prima comparare, ce s-a salvat in sesiune
                                let vederea = st.app.as_ref().map(|a| a.lista().clone());
                                let ecranul = st.app.as_ref().map(|a| a.ecran());
                                let (revizuire, editari, taburi) = match st.app.as_mut() {
                                    Some(a) => (
                                        a.revizuire_de_salvat().clone(),
                                        a.editari().clone(),
                                        a.taburi().clone(),
                                    ),
                                    None => {
                                        // cheia sesiunii tine si serverul; una
                                        // ilizibila se pune deoparte (F-A15)
                                        let salvata = match st.sesiuni.incarca(&r.server, &referinta) {
                                            Ok(c) => c,
                                            Err(e) => {
                                                let m = pune_deoparte(
                                                    &st.sesiuni,
                                                    &r.server,
                                                    &referinta,
                                                    &e,
                                                    st.conectare.limba,
                                                );
                                                st.mesaj_lista = Some(m);
                                                None
                                            }
                                        };
                                        (
                                            salvata
                                                .as_ref()
                                                .map(modul::Revizuire::din_sesiune)
                                                .unwrap_or_default(),
                                            salvata
                                                .as_ref()
                                                .map(modul::Editari::din_sesiune)
                                                .unwrap_or_default(),
                                            // taburile fixate si coloana supravietuiesc
                                            // inchiderii (etapa 16)
                                            {
                                                st.coloana_noua = salvata.as_ref().and_then(
                                                    gmotion_app::rutare::StareColoana::din_sesiune,
                                                );
                                                salvata
                                                    .as_ref()
                                                    .map(gmotion_app::rutare::taburi_din_sesiune)
                                                    .unwrap_or_default()
                                            },
                                        )
                                    }
                                };
                                st.server = r.server;
                                st.referinta = referinta;
                                st.salvate_ultima = (revizuire.clone(), editari.clone());
                                st.offline = false;
                                // o baza fara niciun obiect vizibil (drepturi
                                // lipsa sau baza noua) se spune in lista
                                if !r.sursa.is_empty() {
                                    let goale: Vec<&str> = r
                                        .tinte
                                        .iter()
                                        .filter(|(_, b)| b.is_empty())
                                        .map(|(n, _)| n.as_str())
                                        .collect();
                                    if !goale.is_empty() {
                                        let l = st.conectare.limba;
                                        st.mesaj_lista = Some(format!(
                                            "{}: {}",
                                            l.alege(
                                                "Atentie, fara niciun obiect vizibil (baza noua sau drepturi lipsa)",
                                                "Warning, no visible objects (new database or missing permissions)"
                                            ),
                                            goale.join(", ")
                                        ));
                                    }
                                }
                                let mut app =
                                    Aplicatie::noua(r.randuri, r.sursa, r.tinte, revizuire)
                                        .cu_editari(editari)
                                        .cu_taburi(taburi);
                                if let Some(v) = &vederea {
                                    app = app.cu_vederea(v);
                                }
                                if let Some(e) = ecranul {
                                    app = app.cu_ecranul(e);
                                }
                                // ca la offline: comutatorul porneste din configurare
                                app.comanda(Comanda::ComutaUnireaCursorului(uneste_implicit));
                                st.detalii = Some(porneste_detalii(&app));
                                st.app = Some(app);
                            }
                            Err(e) if e == pornire::ANULAT => {
                                tracing::info!(target: "gmotion::app", "compararea a fost anulata");
                                let l = st.conectare.limba;
                                st.conectare.sursa.stare = StareConexiune::Eroare(
                                    l.alege("comparare anulata", "comparison cancelled").into(),
                                );
                            }
                            Err(e) => {
                                tracing::warn!(target: "gmotion::app", eroare = %e, "compararea a esuat");
                                // la recomparare omul e in lista: acolo trebuie spus
                                if st.app.is_some() {
                                    let l = st.conectare.limba;
                                    st.mesaj_lista = Some(format!(
                                        "{}: {e}",
                                        l.alege("Recompararea a esuat", "Comparing again failed")
                                    ));
                                }
                                st.conectare.sursa.stare = StareConexiune::Eroare(e);
                            }
                        }
                        drop(st);
                        d();
                    }
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
        let c = stare.borrow().conectare.clone();
        let (baze, stare_citire) = pornire::citeste_bazele(&c, &c.sursa);
        let mut st = stare.borrow_mut();
        st.conectare.sursa.baze.clone_from(&baze);
        st.conectare.baza.baze.clone_from(&baze);
        st.conectare.sursa.stare = stare_citire.clone();
        st.conectare.baza.stare = stare_citire;
        // referinta e prima baza; restul se compara cu ea
        if let Some((prima, restul)) = baze.split_first() {
            st.conectare.sursa.baza = prima.clone();
            for b in restul {
                st.conectare.baza.comuta_baza(b);
            }
        }
        let c = st.conectare.clone();
        drop(st);
        if c.poate_compara() {
            tracing::info!(target: "gmotion::app", "previzualizare: compar instantaneul sintetic");
            stare.borrow_mut().rulare = Some(gmotion_app::fir::Rulare::porneste(c));
        } else {
            tracing::warn!(target: "gmotion::app", "previzualizarea nu poate porni: {:?}", c.ce_lipseste());
        }
    }

    deseneaza();
    slint::run_event_loop().map_err(|e| e.to_string())?;
    Ok(())
}
