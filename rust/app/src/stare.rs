//! Starea binarului cu interfata, fara Slint: ce s-a conectat, ce ruleaza pe
//! alte fire, ce se salveaza in sesiune.
//!
//! **Se strica astfel:** dupa o comparare, o recomparare sau o deconectare,
//! ecranul arata o stare care nu mai e cea adevarata (revizuirea pierduta,
//! eroarea pusa pe alt camp, ceva ramas de la conexiunea de dinainte). Statea
//! in `main.rs`, in inchideri legate de ferestre, deci fara teste; aici se
//! testeaza fara fereastra.

use gmotion_fereastra::model_conectare::{Conectare, Recenta, StareConexiune};
use gmotion_modul_diff_obiecte as modul;

use crate::fir::Lucru;
use crate::pornire;
use crate::rutare::{Aplicatie, Comanda, StareColoana};

/// Bazele unui server si ce s-a aflat despre el incercand sa le citeasca.
pub type BazeCitite = (Vec<String>, StareConexiune);

/// Ce trimite firul de detalii: cheia obiectului si detaliile lui.
pub type DetaliiCalculate = ((String, String), gmotion_fereastra::model_lista::Detalii);

/// Ce stie aplicatia in orice moment.
pub struct Stare {
    pub conectare: Conectare,
    /// Lipseste pana la prima comparare.
    pub app: Option<Aplicatie>,
    pub sesiuni: gmotion_platforma::Sesiuni,
    /// Ce scrie pe ecran despre sursa (server sau cale de instantaneu).
    pub server: String,
    /// Baza de referinta **cu care a rulat compararea**, nu ce scrie in
    /// formular acum. Formularul se poate schimba cat ruleaza (si se poate
    /// schimba de mana oricand dupa): sesiunea, deploy-ul si antetul se
    /// sprijina pe asta (F-A3 din auditul QA).
    pub referinta: String,
    /// Planul de deploy deschis, cu cheia lui: cand se schimba ce e „in
    /// plan", textele finale sau compararea, planul se reface de la zero —
    /// deci si fara garda. Decizia e in `deploy::asigura_planul`, nu aici.
    pub deploy: Option<crate::deploy::Deschis>,
    /// Compararea care ruleaza acum, pe alt fir.
    pub rulare: Option<crate::fir::Rulare>,
    /// Celelalte lucruri lungi, si ele pe alt fir: citirea bazelor, testul de
    /// conexiune, cautarea de servere si garda de deploy. Pe firul ferestrei,
    /// un server inaccesibil tinea aplicatia „Not responding" pana la timeout
    /// (P4 din auditul QA). `bool` = pentru care parte a ecranului.
    pub baze_cerute: Option<(bool, Lucru<BazeCitite>)>,
    pub testare: Option<(bool, Lucru<StareConexiune>)>,
    pub cautare_servere: Option<(
        bool,
        Lucru<Vec<gmotion_fereastra::model_conectare::ServerGasit>>,
    )>,
    /// Starea coloanei din editor citita din sesiune, de pus in fereastra la
    /// urmatoarea desenare (etapa 16).
    pub coloana_noua: Option<StareColoana>,
    pub garda: Option<Lucru<pornire::Citit>>,
    /// „Difera in" si `+N −M` pentru toate obiectele modificate, calculate pe
    /// alt fir dupa comparare (etapa 19). Un receptor nou opreste firul vechi.
    pub detalii: Option<std::sync::mpsc::Receiver<DetaliiCalculate>>,
    /// Cand s-a salvat ultima data progresul (revizuirea, textele finale):
    /// „salvat la 14:52" din editor.
    pub salvat_la: String,
    /// „Uneste ciclul cursor" din configurare: cu el porneste fiecare
    /// comparare (F-B5).
    pub uneste_implicit: bool,
    /// Bara de mesaje a listei: exportul CSV, o recomparare esuata sau
    /// anulata. Pe ecranul listei, o eroare pusa pe formularul de conectare
    /// nu se vedea deloc (F-A8).
    pub mesaj_lista: String,
    /// Folderul ales cu dialogul nativ pentru deploy (etapa 25); `None` =
    /// directorul de date. Deploy-ul merge intr-un subfolder nou al lui, cu
    /// referinta si momentul in nume, ca un folder plin sa nu fie refuzat.
    pub folder_deploy: Option<std::path::PathBuf>,
    /// „Verifica pe server" (etapa 26), pe alt fir: pentru ce cheie de obiect
    /// si ce cod s-a trimis.
    pub verificare: Option<(CheieVerificare, Lucru<gmotion_db::VerificareServer>)>,
    /// Ultimul raspuns al serverului; se arata cat timp obiectul si codul
    /// sunt aceleasi.
    pub rezultat_verificare: Option<(
        CheieVerificare,
        Result<gmotion_db::VerificareServer, String>,
    )>,
}

/// Ce s-a verificat pe server: obiectul (`baza`, `schema.nume`) si codul trimis.
pub type CheieVerificare = ((String, String), String);

/// Ce s-a intamplat cu compararea care rula, la o bataie de ceas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Progres {
    /// Nu ruleaza nicio comparare.
    Nimic,
    /// Inca ruleaza: se redeseneaza doar bara.
    InCurs,
    /// S-a terminat (bine, anulata sau cu eroare); starea e deja pusa.
    Terminata,
}

impl Stare {
    /// Starea de pornire: doar formularul de conectare si sesiunile.
    pub fn noua(conectare: Conectare, sesiuni: gmotion_platforma::Sesiuni) -> Stare {
        Stare {
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
            uneste_implicit: false,
            mesaj_lista: String::new(),
            folder_deploy: None,
            verificare: None,
            rezultat_verificare: None,
        }
    }

    /// Partea aleasa, ca sa nu se repete `if sursa { ... } else { ... }` la
    /// fiecare camp din ecranul de conectare.
    ///
    /// `None` cat ruleaza o comparare: formularul e desenat blocat, dar
    /// primeste tastatura, si o tasta in campul bazei schimba sub noi
    /// referinta cu care s-a pornit (F-A3 din auditul QA). Intoarce `Option`
    /// tocmai ca sa nu se poata uita: compilatorul cere tratat cazul la
    /// fiecare camp.
    pub fn parte(&mut self, sursa: bool) -> Option<&mut gmotion_fereastra::model_conectare::Parte> {
        if self.rulare.is_some() {
            return None;
        }
        Some(if sursa {
            &mut self.conectare.sursa
        } else {
            &mut self.conectare.baza
        })
    }

    /// Bazele comparate chiar acum, toate; inainte de prima comparare, cea
    /// din formular.
    pub fn baze(&self) -> Vec<String> {
        self.app
            .as_ref()
            .map(|a| a.baze())
            .unwrap_or_else(|| vec![self.conectare.baza.baza.clone()])
    }

    /// Salveaza progresul (revizuirea, textele finale, taburile, coloana) in
    /// sesiunea referintei. Nimic de salvat inainte de prima comparare.
    pub fn salveaza(&mut self, coloana: Option<StareColoana>) {
        let referinta = self.referinta.clone();
        // bazele comparate chiar acum, toate, nu doar prima bifata
        let baze = self.baze();
        let server = self.server.clone();
        let auth = if self.conectare.instantaneu.is_some() {
            "instantaneu"
        } else if self.conectare.sursa.auth_windows {
            "Windows"
        } else {
            "SQL"
        };
        let Some(revizuire) = self.app.as_mut().map(|a| a.revizuire_de_salvat().clone()) else {
            return;
        };
        let editari = self
            .app
            .as_mut()
            .map(|a| a.editari_de_salvat().clone())
            .unwrap_or_default();
        let mut sesiune = self
            .sesiuni
            .incarca(&server, &referinta)
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                gmotion_platforma::Sesiune::noua(server, auth, referinta.clone(), String::new())
            });
        revizuire.in_sesiune(&mut sesiune);
        editari.in_sesiune(&mut sesiune);
        if let Some(a) = self.app.as_ref() {
            crate::rutare::taburi_in_sesiune(a.taburi(), &mut sesiune);
        }
        if let Some(c) = coloana {
            c.in_sesiune(&mut sesiune);
        }
        // se adauga, nu se inlocuiesc: sesiunea tine munca facuta pe
        // toate bazele comparate vreodata cu referinta asta (C3)
        sesiune.adauga_baze(&baze);
        match self.sesiuni.salveaza(&sesiune) {
            Ok(_) => {
                // ora, nu data: „ultima data la 14:52"
                let acum = gmotion_platforma::timp::acum();
                self.salvat_la = acum.split(' ').nth(1).unwrap_or(&acum).to_string();
            }
            Err(e) => {
                tracing::warn!(target: "gmotion::app", eroare = %e, "progresul nu s-a salvat");
            }
        }
    }

    /// Inapoi la ecranul de conectare. Progresul se salveaza **inainte**, de
    /// cine cheama: altfel „Deconecteaza" ar insemna „pierde ce ai revizuit".
    pub fn deconecteaza(&mut self) {
        self.app = None;
        self.detalii = None;
        self.deploy = None;
        self.rulare = None;
        self.referinta = String::new();
        // nimic de la conexiunea de dinainte nu ramane pe ecran (F-A16)
        self.salvat_la = String::new();
        self.mesaj_lista = String::new();
        self.conectare.eroare_comparare = None;
        self.conectare.recente = recente(&self.sesiuni);
        tracing::info!(target: "gmotion::app", "deconectat");
    }

    /// O comparare facuta (pe server, din instantaneu sau offline) devine
    /// aplicatia curenta.
    ///
    /// La recomparare pastram ce s-a revizuit deja si vederea listei; la
    /// prima comparare, ce s-a salvat in sesiune.
    pub fn primeste_compararea(&mut self, r: pornire::Comparata) {
        tracing::info!(
            target: "gmotion::app",
            obiecte = r.randuri.len(),
            recomparare = self.app.is_some(),
            "compararea s-a terminat"
        );
        let referinta = r.referinta.clone();
        let vederea = self.app.as_ref().map(|a| a.lista().clone());
        let (revizuire, editari, taburi) = match self.app.as_mut() {
            Some(a) => (
                a.revizuire_de_salvat().clone(),
                a.editari().clone(),
                a.taburi().clone(),
            ),
            None => {
                // cheia sesiunii tine si serverul
                let salvata = self.sesiune_salvata(&r.server, &referinta);
                // taburile fixate si coloana supravietuiesc inchiderii (etapa 16)
                self.coloana_noua = salvata.as_ref().and_then(StareColoana::din_sesiune);
                (
                    salvata
                        .as_ref()
                        .map(modul::Revizuire::din_sesiune)
                        .unwrap_or_default(),
                    salvata
                        .as_ref()
                        .map(modul::Editari::din_sesiune)
                        .unwrap_or_default(),
                    salvata
                        .as_ref()
                        .map(crate::rutare::taburi_din_sesiune)
                        .unwrap_or_default(),
                )
            }
        };
        self.server = r.server;
        self.referinta = referinta;
        let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, revizuire)
            .cu_editari(editari)
            .cu_taburi(taburi);
        if let Some(v) = &vederea {
            app = app.cu_vederea(v);
        }
        // ca la offline: comutatorul porneste din configurare
        app.comanda(Comanda::ComutaUnireaCursorului(self.uneste_implicit));
        self.detalii = Some(porneste_detalii(&app));
        self.app = Some(app);
    }

    /// Sesiunea salvata pentru `server` / `referinta`. Una care nu se poate
    /// citi nu se mai inlocuieste tacut la prima salvare: se pune deoparte
    /// (`.corupt`) si se spune pe lista (F-A15).
    fn sesiune_salvata(
        &mut self,
        server: &str,
        referinta: &str,
    ) -> Option<gmotion_platforma::Sesiune> {
        match self.sesiuni.incarca(server, referinta) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(target: "gmotion::app", eroare = %e, "sesiunea salvata nu se poate citi");
                let l = self.conectare.limba;
                let unde = match self.sesiuni.pune_deoparte(server, referinta) {
                    Ok(Some(c)) => format!(
                        " {} {}.",
                        l.alege("Copia ei e in", "A copy is in"),
                        c.display()
                    ),
                    _ => String::new(),
                };
                self.mesaj_lista = format!(
                    "{}: {e}.{unde}",
                    l.alege(
                        "Progresul salvat nu s-a putut citi, revizuirea porneste de la zero",
                        "The saved progress could not be read, the review starts over"
                    )
                );
                None
            }
        }
    }

    /// Compararea de pe celalalt fir, la o bataie de ceas.
    pub fn verifica_compararea(&mut self) -> Progres {
        let gata = match self.rulare.as_mut() {
            Some(r) => r.verifica(),
            None => return Progres::Nimic,
        };
        let Some(rezultat) = gata else {
            return Progres::InCurs;
        };
        self.rulare = None;
        let l = self.conectare.limba;
        // o recomparare se face de pe lista: acolo se spune ce s-a intamplat,
        // iar lista veche ramane (F-A8). Prima comparare, langa buton (F-A12).
        let recomparare = self.app.is_some();
        let mesaj = match rezultat {
            Ok(r) => {
                self.conectare.eroare_comparare = None;
                self.mesaj_lista = String::new();
                self.primeste_compararea(*r);
                return Progres::Terminata;
            }
            Err(e) if e == pornire::ANULAT => {
                tracing::info!(target: "gmotion::app", recomparare, "compararea a fost anulata");
                if recomparare {
                    l.alege(
                        "Recompararea a fost anulata; lista e cea de dinainte.",
                        "The comparison was cancelled; the list is the previous one.",
                    )
                    .to_string()
                } else {
                    l.alege("Comparare anulata.", "Comparison cancelled.")
                        .to_string()
                }
            }
            Err(e) => {
                tracing::warn!(target: "gmotion::app", eroare = %e, recomparare, "compararea a esuat");
                if recomparare {
                    format!(
                        "{}: {e}",
                        l.alege(
                            "Recompararea a esuat; lista e cea de dinainte",
                            "The comparison failed; the list is the previous one"
                        )
                    )
                } else {
                    format!(
                        "{}: {e}",
                        l.alege("Compararea a esuat", "The comparison failed")
                    )
                }
            }
        };
        if recomparare {
            self.mesaj_lista = mesaj;
        } else {
            self.conectare.eroare_comparare = Some(mesaj);
        }
        Progres::Terminata
    }

    /// Celelalte lucruri de pe alte fire: bazele, testul, serverele, garda,
    /// detaliile listei. Fiecare isi pune rezultatul in model si isi goleste
    /// slotul (P4). `true` daca s-a schimbat ceva de desenat.
    pub fn verifica_lucrarile(&mut self) -> bool {
        let mut s_a_schimbat = false;
        if let Some((sursa, l)) = self.baze_cerute.as_mut() {
            let (sursa, gata) = (*sursa, l.verifica());
            if let Some(r) = gata {
                self.baze_cerute = None;
                s_a_schimbat = true;
                let (baze, stare_noua) = match r {
                    Ok(x) => x,
                    Err(e) => (Vec::new(), StareConexiune::Eroare(e)),
                };
                tracing::info!(target: "gmotion::app", sursa, baze = baze.len(), stare = ?stare_noua, "baze citite");
                if let Some(p) = self.parte(sursa) {
                    p.baze = baze;
                    p.stare = stare_noua;
                }
            }
        }
        if let Some((sursa, l)) = self.testare.as_mut() {
            let (sursa, gata) = (*sursa, l.verifica());
            if let Some(r) = gata {
                self.testare = None;
                s_a_schimbat = true;
                let stare_noua = r.unwrap_or_else(StareConexiune::Eroare);
                tracing::info!(target: "gmotion::app", sursa, stare = ?stare_noua, "testul conexiunii s-a terminat");
                if let Some(p) = self.parte(sursa) {
                    p.stare = stare_noua;
                }
            }
        }
        let limba = self.conectare.limba;
        if let Some((sursa, l)) = self.cautare_servere.as_mut() {
            let (sursa, gata) = (*sursa, l.verifica());
            if let Some(r) = gata {
                self.cautare_servere = None;
                s_a_schimbat = true;
                match r {
                    Ok(v) if v.is_empty() => {
                        tracing::info!(target: "gmotion::app", "niciun server gasit in retea");
                        if let Some(p) = self.parte(sursa) {
                            p.servere_gasite.clear();
                            p.stare = StareConexiune::Eroare(
                                limba
                                    .alege(
                                        "niciun server nu a raspuns in retea; scrie-l de mana",
                                        "no server answered on the network; type it in",
                                    )
                                    .into(),
                            );
                        }
                    }
                    Ok(v) => {
                        tracing::info!(target: "gmotion::app", gasite = v.len(), "servere gasite");
                        if let Some(p) = self.parte(sursa) {
                            p.servere_gasite = v;
                            p.stare = StareConexiune::Neincercat;
                        }
                    }
                    Err(e) => {
                        tracing::warn!(target: "gmotion::app", eroare = %e, "cautarea serverelor a esuat");
                        if let Some(p) = self.parte(sursa) {
                            p.servere_gasite.clear();
                            p.stare = StareConexiune::Eroare(e);
                        }
                    }
                }
            }
        }
        if let Some((_, l)) = self.verificare.as_mut() {
            if let Some(r) = l.verifica() {
                if let Some((cheie, _)) = self.verificare.take() {
                    tracing::info!(target: "gmotion::app", reusita = r.is_ok(), "verificarea pe server s-a terminat");
                    self.rezultat_verificare = Some((cheie, r));
                    s_a_schimbat = true;
                }
            }
        }
        if let Some(l) = self.garda.as_mut() {
            if let Some(r) = l.verifica() {
                self.garda = None;
                s_a_schimbat = true;
                let acum = gmotion_platforma::timp::acum();
                tracing::info!(target: "gmotion::app", reusita = r.is_ok(), "verificarea de garda s-a terminat");
                if let Some(dep) = self.deploy.as_mut().map(|d| &mut d.stare) {
                    match r {
                        Ok(x) => dep.aplica_verificarea(x.sursa, x.tinte, &acum),
                        Err(e) => dep.verificare_esuata(e),
                    }
                }
            }
        }
        // detaliile de pe alt fir: cate au venit, pe obiectul lor
        if let (Some(rx), Some(app)) = (self.detalii.as_ref(), self.app.as_mut()) {
            let mut venite = 0usize;
            let mut gata = false;
            loop {
                match rx.try_recv() {
                    Ok((cheie, d)) => {
                        app.pune_detalii(&cheie, d);
                        venite += 1;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        gata = true;
                        break;
                    }
                }
            }
            if gata {
                tracing::debug!(target: "gmotion::app", "detaliile listei sunt calculate");
                self.detalii = None;
            }
            if venite > 0 {
                app.detalii_sosite();
                s_a_schimbat = true;
            }
        }
        s_a_schimbat
    }

    // --- ecranul de conectare: fiecare camp si buton, fara fereastra

    /// Serverul scris de mana: ce stiam despre cel vechi nu mai e valabil.
    pub fn server_schimbat(&mut self, sursa: bool, text: &str) {
        let Some(p) = self.parte(sursa) else { return };
        p.server = text.to_string();
        p.baze.clear();
        p.baza.clear();
        p.stare = StareConexiune::Neincercat;
    }

    /// Serverul ales din lista celor gasite in retea.
    pub fn server_ales(&mut self, sursa: bool, server: &str) {
        let Some(p) = self.parte(sursa) else { return };
        p.server = server.to_string();
        p.servere_gasite.clear();
        p.baze.clear();
        p.baza.clear();
        p.stare = StareConexiune::Neincercat;
    }

    pub fn auth_schimbat(&mut self, sursa: bool, windows: bool) {
        let Some(p) = self.parte(sursa) else { return };
        p.auth_windows = windows;
        p.baze.clear();
        p.stare = StareConexiune::Neincercat;
    }

    pub fn utilizator_schimbat(&mut self, sursa: bool, text: &str) {
        if let Some(p) = self.parte(sursa) {
            p.acreditari.utilizator = text.to_string();
        }
    }

    pub fn parola_schimbata(&mut self, sursa: bool, text: &str) {
        if let Some(p) = self.parte(sursa) {
            p.acreditari.pune_parola(text.to_string());
        }
    }

    pub fn baza_schimbata(&mut self, sursa: bool, text: &str) {
        if self.parte(sursa).is_none() {
            return;
        }
        if sursa {
            self.conectare.sursa.baza = text.to_string();
        } else {
            // o singura baza si la modificata (etapa 17)
            self.conectare.alege_baza_modificata(text);
        }
    }

    pub fn comuta_aceeasi_conexiune(&mut self) {
        if self.rulare.is_some() {
            return;
        }
        let da = !self.conectare.aceeasi_conexiune;
        self.conectare.pune_aceeasi_conexiune(da);
    }

    /// Doua stari, ca in v3.4: nevalidat (implicit) sau depozitul sistemului.
    /// Calea unei autoritati se da din linia de comanda.
    pub fn comuta_certificatul(&mut self) {
        self.conectare.verifica_certificatul = !self.conectare.verifica_certificatul;
    }

    /// Instantaneul ales: bazele vin din fisier, iar daca nu se citeste, se
    /// spune pe loc si instantaneul nu ramane ales. `true` daca s-a citit.
    pub fn alege_instantaneul(&mut self, cale: &str) -> bool {
        let cale = cale.trim();
        if cale.is_empty() {
            return false;
        }
        self.conectare.instantaneu = Some(cale.to_string());
        let c = self.conectare.clone();
        let (baze, stare_noua) = pornire::citeste_bazele(&c, &c.sursa);
        let reusit = stare_noua.e_conectat();
        self.conectare.sursa.baze.clone_from(&baze);
        self.conectare.baza.baze = baze;
        self.conectare.sursa.stare = stare_noua.clone();
        self.conectare.baza.stare = stare_noua;
        if !reusit {
            self.conectare.instantaneu = None;
        } else {
            self.conectare.sursa.baza = String::new();
            self.conectare.baza.alese.clear();
            self.conectare.baza.baza = String::new();
        }
        reusit
    }

    /// Bazele serverului, pe alt fir: un server care nu raspunde ar tine
    /// fereastra pana la timeout (30 s), fara sa se poata apasa nimic (P4).
    pub fn cere_bazele(&mut self, sursa: bool) {
        let c = self.conectare.clone();
        if let Some(ce) = c.ce_lipseste_pentru_baze(sursa) {
            if let Some(tinta) = self.parte(sursa) {
                tinta.baze = Vec::new();
                tinta.stare = StareConexiune::Eroare(
                    gmotion_fereastra::model_conectare::completeaza(ce, c.limba),
                );
            }
            return;
        }
        if self.baze_cerute.is_some() {
            return;
        }
        if let Some(tinta) = self.parte(sursa) {
            tinta.stare = StareConexiune::SeConecteaza;
        }
        self.baze_cerute = Some((
            sursa,
            Lucru::porneste(move || {
                let p = if sursa { &c.sursa } else { &c.baza };
                Ok(pornire::citeste_bazele(&c, p))
            }),
        ));
    }

    /// „Testeaza conexiunea", pe alt fir.
    pub fn testeaza(&mut self, sursa: bool) {
        let c = self.conectare.clone();
        tracing::info!(target: "gmotion::app", sursa, "testez conexiunea");
        if self.testare.is_some() {
            return;
        }
        if let Some(p) = self.parte(sursa) {
            p.stare = StareConexiune::SeConecteaza;
        }
        self.testare = Some((
            sursa,
            Lucru::porneste(move || {
                let p = if sursa { &c.sursa } else { &c.baza };
                Ok(match pornire::testeaza(&c, p) {
                    Ok(t) => StareConexiune::Conectat(t),
                    Err(e) => StareConexiune::Eroare(e),
                })
            }),
        ));
    }

    /// Serverele din retea (SQL Browser), ca `gmotion servere`, pe alt fir.
    pub fn cauta_servere(&mut self, sursa: bool, timeout: std::time::Duration) {
        tracing::info!(target: "gmotion::app", "caut servere in retea");
        if self.cautare_servere.is_some() {
            return;
        }
        if let Some(p) = self.parte(sursa) {
            p.stare = StareConexiune::SeCauta;
        }
        self.cautare_servere = Some((
            sursa,
            Lucru::porneste(move || {
                gmotion_db::cauta_instante(timeout)
                    .map(|v| {
                        v.into_iter()
                            .map(|i| gmotion_fereastra::model_conectare::ServerGasit {
                                nume: i.server,
                                implicita: i.implicita,
                                cluster: i.cluster,
                            })
                            .collect()
                    })
                    .map_err(|e| e.to_string())
            }),
        ));
    }

    /// Porneste compararea (si recompararea) pe alt fir, cu progres. O
    /// recomparare pastreaza revizuirea: vezi `primeste_compararea`.
    pub fn compara(&mut self) {
        tracing::info!(target: "gmotion::app", recomparare = self.app.is_some(), "pornesc compararea");
        self.conectare.eroare_comparare = None;
        self.rulare = Some(crate::fir::Rulare::porneste(self.conectare.clone()));
    }

    pub fn anuleaza(&mut self) {
        if let Some(r) = self.rulare.as_mut() {
            r.anuleaza();
        }
    }

    /// O comparare recenta pune bazele ei in formular.
    pub fn reia(&mut self, i: i32) {
        if let Some(r) = self
            .conectare
            .recente
            .get(usize::try_from(i).unwrap_or(0))
            .cloned()
        {
            self.conectare.sursa.baza = r.sursa;
            self.conectare.baza.baza = r.baza;
        }
    }

    /// O comparare recenta, uitata: sesiunea ei se sterge.
    pub fn ignora_recenta(&mut self, i: i32) {
        let Some(r) = self
            .conectare
            .recente
            .get(usize::try_from(i).unwrap_or(0))
            .cloned()
        else {
            return;
        };
        match self.sesiuni.sterge(&r.server, &r.sursa) {
            Ok(_) => {
                tracing::info!(target: "gmotion::app", referinta = %r.sursa, "comparare ignorata")
            }
            Err(e) => tracing::warn!(target: "gmotion::app", eroare = %e, "nu pot sterge sesiunea"),
        }
        self.conectare.recente = recente(&self.sesiuni);
    }

    /// Compararea a doua texte lipite (offline). `Err` cu mesajul pentru om.
    pub fn compara_offline(&mut self, a: &str, b: &str) -> Result<(), String> {
        let r = pornire::compara_offline(a, b, self.conectare.limba)?;
        self.server = r.server;
        self.referinta = r.referinta.clone();
        let mut app = Aplicatie::noua(r.randuri, r.sursa, r.tinte, modul::Revizuire::noua());
        // „Uneste ciclul cursor" porneste cum spune configurarea (F-B5)
        app.comanda(Comanda::ComutaUnireaCursorului(self.uneste_implicit));
        self.detalii = Some(porneste_detalii(&app));
        self.app = Some(app);
        // numele bazelor din antet: offline nu vine de pe server
        self.conectare.sursa.baza = self.referinta.clone();
        self.conectare.baza.baza = "Lipit".into();
        Ok(())
    }

    /// Exportul CSV al listei vizibile, in directorul de date; ce s-a
    /// intamplat ajunge in bara listei. Nimic inainte de prima comparare.
    pub fn exporta_csv(&mut self) {
        let Some(app) = self.app.as_ref() else {
            return;
        };
        let cale = cale_csv(&self.referinta);
        self.mesaj_lista = match cale.parent().map(std::fs::create_dir_all) {
            Some(Err(e)) => format!(
                "{}: {e}",
                self.conectare.limba.alege("nu pot scrie", "cannot write")
            ),
            _ => match app.exporta_csv(&cale) {
                Ok(n) => {
                    tracing::info!(target: "gmotion::app", cale = %cale.display(), randuri = n, "CSV scris");
                    format!("{n} randuri in {}", cale.display())
                }
                Err(e) => e,
            },
        };
    }

    /// Recompararea care ruleaza, pentru bara listei: textul pasului (cu
    /// timpul) si cat s-a facut. `None` cand nu ruleaza nimic sau cand e prima
    /// comparare (aceea se vede pe ecranul de conectare).
    pub fn progres_recomparare(&self, limba: gmotion_core::Limba) -> Option<(String, f32)> {
        self.app.as_ref()?;
        let r = self.rulare.as_ref()?;
        Some((format!("{} · {}", r.text_in(limba), r.timp()), r.fractie()))
    }

    /// Unde se va scrie deploy-ul acum: in folderul ales, daca s-a ales.
    pub fn director_deploy_curent(&self) -> std::path::PathBuf {
        director_deploy_in(self.folder_deploy.as_deref(), &self.referinta)
    }

    /// Directorul in care s-a scris ultimul deploy reusit, pentru „Deschide
    /// folderul".
    pub fn director_scris(&self) -> Option<std::path::PathBuf> {
        match self.deploy.as_ref()?.stare.rezultat.as_ref()? {
            Ok(r) => Some(r.director.clone()),
            Err(_) => None,
        }
    }

    /// Ce s-ar trimite la verificarea pe server pentru obiectul deschis:
    /// cheia, codul in forma care se deployeaza (`CREATE OR ALTER`, numele
    /// din plan) si setarile lui de creare. `None` fara obiect deschis.
    fn de_verificat(&self) -> Option<(CheieVerificare, Option<gmotion_db::Setari>)> {
        let app = self.app.as_ref()?;
        let cheie = app.obiect()?.cheie();
        let (_, final_) = app.texte_curente()?;
        let cod = modul::aplica_create_or_alter(&final_, &cheie.1);
        let setari = app.sursa().get(&cheie.1).and_then(|o| o.setari);
        Some(((cheie, cod), setari))
    }

    /// „Verifica pe server": textul final (sau al bazei) al obiectului
    /// deschis, trimis fara efecte la baza modificata, pe alt fir.
    pub fn verifica_pe_server(&mut self) {
        if self.verificare.is_some() {
            return;
        }
        let Some((cheie, setari)) = self.de_verificat() else {
            return;
        };
        let c = self.conectare.clone();
        let (baza, cod) = (cheie.0 .0.clone(), cheie.1.clone());
        tracing::info!(target: "gmotion::app", %baza, obiect = %cheie.0 .1, "verific pe server");
        self.verificare = Some((
            cheie,
            Lucru::porneste(move || {
                let mut s = pornire::deschide(&c, &c.baza)?;
                s.verifica_cod(&baza, &cod, setari)
                    .map_err(|e| e.to_string())
            }),
        ));
    }

    /// Ce spune editorul despre verificarea pe server: textul si daca e de
    /// bine. `None` cand nu e nimic de spus pentru obiectul si codul de acum
    /// (un raspuns vechi nu mai e al lor).
    pub fn text_verificare(&self, limba: gmotion_core::Limba) -> Option<(String, bool)> {
        let (cheie, _) = self.de_verificat()?;
        if self.verificare.as_ref().is_some_and(|(k, _)| *k == cheie) {
            return Some((
                limba
                    .alege("Se verifica pe server…", "Checking on the server…")
                    .to_string(),
                true,
            ));
        }
        let (k, r) = self.rezultat_verificare.as_ref()?;
        if *k != cheie {
            return None;
        }
        Some(match r {
            Err(e) => (
                format!(
                    "{}: {e}",
                    limba.alege("Nu s-a putut verifica", "Could not check")
                ),
                false,
            ),
            Ok(v) => match v.sintaxa.as_ref().or(v.compilare.as_ref()) {
                None => (
                    limba
                        .alege(
                            "Serverul accepta textul final (sintaxa si compilarea)",
                            "The server accepts the final text (syntax and compilation)",
                        )
                        .to_string(),
                    true,
                ),
                Some(m) => (
                    format!(
                        "{} {} {} {}: {}",
                        limba.alege("Serverul: eroarea", "Server: error"),
                        m.numar,
                        limba.alege("la linia", "at line"),
                        m.linie,
                        m.mesaj
                    ),
                    false,
                ),
            },
        })
    }

    /// Ce scrie in bara listei: un „In plan" refuzat bate mesajul tinut.
    pub fn text_bara_lista(&self, limba: gmotion_core::Limba) -> String {
        self.app
            .as_ref()
            .and_then(|a| text_refuz_in_plan(a, limba))
            .map(str::to_string)
            .unwrap_or_else(|| self.mesaj_lista.clone())
    }

    // --- planul de deploy

    /// Garda: se recitesc ambele baze, pe alt fir. `reciteste`, nu `compara`:
    /// garzii ii trebuie doar obiectele de acum, iar compararea intreaga tinea
    /// minute pe o baza mare (P4).
    pub fn porneste_garda(&mut self) {
        if self.garda.is_some() {
            return;
        }
        tracing::info!(target: "gmotion::app", "garda: recitesc bazele");
        if let Some(dep) = self.deploy.as_mut().map(|d| &mut d.stare) {
            dep.verificarea_a_pornit();
        }
        let c = self.conectare.clone();
        self.garda = Some(Lucru::porneste(move || pornire::reciteste(&c)));
    }

    /// Scrie scripturile planului deschis in `director`. `true` daca s-au
    /// scris si obiectele au fost marcate „deployate" (atunci se salveaza
    /// sesiunea); dupa o incercare esuata, nimic nu se marcheaza.
    pub fn scrie_deploy(&mut self, director: &std::path::Path) -> bool {
        // textele finale se iau acum, din aplicatie, nu dintr-un instantaneu
        // luat la deschiderea planului (C1 din auditul QA)
        let texte = self
            .app
            .as_ref()
            .map(|a| a.texte_finale())
            .unwrap_or_default();
        let mut scris = None;
        if let Some(dep) = self.deploy.as_mut().map(|d| &mut d.stare) {
            dep.scrie(director, &gmotion_platforma::timp::acum(), &texte);
            if let Some(Ok(r)) = &dep.rezultat {
                tracing::info!(target: "gmotion::app", director = %r.director.display(), fisiere = r.fisiere.len(), "deploy scris");
                scris = Some(dep.plan.clone());
            }
        }
        match (scris, self.app.as_mut()) {
            (Some(plan), Some(app)) => app.marcheaza_deployate(&plan),
            _ => false,
        }
    }

    /// Antetul editorului: „obiectul 3 din 12", plus ce s-a intamplat cu
    /// ultima comanda pe taburi (etapa 16). Gol inainte de prima comparare.
    pub fn text_pozitie(&self, limba: gmotion_core::Limba) -> String {
        let Some(app) = self.app.as_ref() else {
            return String::new();
        };
        let mut pozitie = format!(
            "{} {} {} {}",
            limba.alege("obiectul", "object"),
            app.lista().selectat().map_or(0, |p| p + 1),
            limba.alege("din", "of"),
            app.lista().vizibile().len()
        );
        match app.mesaj_tab() {
            Some(crate::rutare::MesajTab::Capat) => pozitie.push_str(limba.alege(
                " · niciun alt obiect nefixat in directia asta",
                " · no other unpinned object in this direction",
            )),
            Some(crate::rutare::MesajTab::ToateEditate) => pozitie.push_str(limba.alege(
                " · 8 taburi fixate, toate editate: inchide unul intai",
                " · 8 pinned tabs, all edited: close one first",
            )),
            None => {}
        }
        pozitie
    }

    /// `--previzualizare`: se intra direct in lista, cu toate bazele
    /// instantaneului bifate; referinta e prima baza.
    pub fn porneste_previzualizarea(&mut self) {
        let c = self.conectare.clone();
        let (baze, stare_citire) = pornire::citeste_bazele(&c, &c.sursa);
        self.conectare.sursa.baze.clone_from(&baze);
        self.conectare.baza.baze.clone_from(&baze);
        self.conectare.sursa.stare = stare_citire.clone();
        self.conectare.baza.stare = stare_citire;
        if let Some((prima, restul)) = baze.split_first() {
            self.conectare.sursa.baza = prima.clone();
            for b in restul {
                self.conectare.baza.comuta_baza(b);
            }
        }
        if self.conectare.poate_compara() {
            tracing::info!(target: "gmotion::app", "previzualizare: compar instantaneul sintetic");
            self.compara();
        } else {
            tracing::warn!(target: "gmotion::app", "previzualizarea nu poate porni: {:?}", self.conectare.ce_lipseste());
        }
    }

    /// „Modificarile se salveaza singure, ultima data la 14:52"; gol inainte
    /// de prima salvare.
    pub fn text_salvat(&self, limba: gmotion_core::Limba) -> String {
        if self.salvat_la.is_empty() {
            String::new()
        } else {
            format!(
                "{} {}",
                limba.alege(
                    "Modificarile se salveaza singure, ultima data la",
                    "Changes are saved automatically, last at"
                ),
                self.salvat_la
            )
        }
    }
}

/// Bara listei, dupa un „In plan" refuzat; `None` daca nu s-a refuzat nimic.
pub fn text_refuz_in_plan(app: &Aplicatie, limba: gmotion_core::Limba) -> Option<&'static str> {
    app.refuzat_in_plan().then(|| {
        limba.alege(
            "Nu intra in plan: obiectul n-are cod de deploiat (exista doar in baza modificata, e criptat sau e tabel; tabelele doar se compara).",
            "Not added to the plan: the object has no code to deploy (it exists only in the modified database, is encrypted or is a table; tables are only compared).",
        )
    })
}

/// Porneste calculul detaliilor pe alt fir, pentru toate obiectele modificate
/// (intai cele vizibile). Firul se opreste singur cand receptorul e aruncat
/// (o comparare noua, deconectarea).
pub fn porneste_detalii(app: &Aplicatie) -> std::sync::mpsc::Receiver<DetaliiCalculate> {
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
pub fn recente(sesiuni: &gmotion_platforma::Sesiuni) -> Vec<Recenta> {
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
                    format!("{revizuite} revizuite")
                },
                cand: s.salvata.clone(),
                sursa: s.referinta.clone(),
                server: s.server.clone(),
                baza,
            }
        })
        .collect()
}

/// Unde se scrie deploy-ul: in directorul de date al aplicatiei, cu
/// referinta si momentul in nume; langa binar, daca acela nu se poate afla.
pub fn director_deploy(referinta: &str) -> std::path::PathBuf {
    director_deploy_in(None, referinta)
}

/// Ca [`director_deploy`], dar intr-un folder ales; `None` = cel implicit.
pub fn director_deploy_in(ales: Option<&std::path::Path>, referinta: &str) -> std::path::PathBuf {
    let nume = format!(
        "{}_{}",
        referinta.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        gmotion_platforma::timp::acum_compact()
    );
    if let Some(d) = ales {
        return d.join(&nume);
    }
    gmotion_platforma::cai::director_date()
        .map(|d| d.join("deploy").join(&nume))
        .unwrap_or_else(|_| std::path::PathBuf::from(format!("deploy_{nume}")))
}

/// Deschide un folder in explorerul sistemului. Fara dependinta: comanda
/// fiecarui sistem.
pub fn deschide_in_explorer(cale: &std::path::Path) -> Result<(), String> {
    let program = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(program)
        .arg(cale)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("{program}: {e}"))
}

/// Unde se scrie exportul CSV: in directorul de date, langa deploy-uri.
/// Ca la deploy, numele are momentul in el, ca doua exporturi sa nu se calce.
pub fn cale_csv(referinta: &str) -> std::path::PathBuf {
    let nume = format!(
        "{}_{}.csv",
        referinta.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        gmotion_platforma::timp::acum_compact()
    );
    gmotion_platforma::cai::director_date()
        .map(|d| d.join("export").join(&nume))
        .unwrap_or_else(|_| std::path::PathBuf::from(nume))
}

/// Schimba un camp din configurare si o scrie la loc. Se reciteste inainte:
/// altfel ar suprascrie ce s-a schimbat de mana intre timp. O configurare
/// stricata nu se suprascrie; se spune in jurnal.
pub fn salveaza_in_configurare(
    cale: &std::path::Path,
    ce: &str,
    schimba: impl FnOnce(&mut gmotion_platforma::Configurare),
) {
    match gmotion_platforma::Configurare::citeste(cale) {
        Ok(mut c) => {
            schimba(&mut c);
            if let Err(e) = c.scrie(cale) {
                tracing::warn!(target: "gmotion::app", eroare = %e, ce, "nu s-a salvat in configurare");
            }
        }
        Err(e) => {
            tracing::warn!(target: "gmotion::app", eroare = %e, ce, "nu s-a salvat: configurarea e stricata");
        }
    }
}

/// Modul de previzualizare: instantaneul sintetic cu obiecte de toate
/// felurile, ca Mihai sa probeze functiile fara baza de date.
///
/// Se cauta langa binar si in radacina proiectului, in ordinea asta: o
/// instalare are fisierul langa `gmotion-ui.exe`, iar in dezvoltare el sta in
/// `referinta/instantanee/`.
pub fn cale_previzualizare() -> Option<std::path::PathBuf> {
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

/// Configurarea de la `cale`: un fisier stricat nu opreste fereastra, dar se
/// spune in jurnal si se folosesc implicitele.
pub fn citeste_configurarea(cale: Option<&std::path::Path>) -> gmotion_platforma::Configurare {
    match cale.map(gmotion_platforma::Configurare::citeste) {
        Some(Ok(c)) => c,
        Some(Err(e)) => {
            tracing::warn!(target: "gmotion::app", eroare = %e, "configurarea nu se citeste; folosesc implicitele");
            gmotion_platforma::Configurare::default()
        }
        None => gmotion_platforma::Configurare::default(),
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use crate::rutare::Ecran;
    use std::path::PathBuf;

    /// Un director propriu fiecarui test: testele ruleaza in paralel.
    fn director(eticheta: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("gmotion-stare-{eticheta}-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn instantaneu(d: &std::path::Path) -> PathBuf {
        let cale = d.join("inst.json");
        std::fs::write(
            &cale,
            r#"{"versiune":1,"server":"SRV-TEST","creat":"2026-11-17T10:00:00Z","nota":"","baze":{
              "Ref":[
                {"schema":"dbo","nume":"pA","tip":"P","definitie":"CREATE PROC dbo.pA AS\nSELECT 1"},
                {"schema":"dbo","nume":"pB","tip":"P","definitie":"CREATE PROC dbo.pB AS\nSELECT 2"},
                {"schema":"dbo","nume":"pNou","tip":"P","definitie":"CREATE PROC dbo.pNou AS\nSELECT 3"}],
              "Client":[
                {"schema":"dbo","nume":"pA","tip":"P","definitie":"CREATE PROC dbo.pA AS\nSELECT 10"},
                {"schema":"dbo","nume":"pB","tip":"P","definitie":"CREATE PROC dbo.pB AS\nSELECT 20"}]}}"#,
        )
        .unwrap();
        cale
    }

    /// O stare cu instantaneul ales si bazele bifate, gata de comparat.
    fn stare(eticheta: &str) -> (Stare, PathBuf) {
        let d = director(eticheta);
        let sesiuni =
            gmotion_platforma::Sesiuni::in_director(modul::NUME, d.join("sesiuni")).unwrap();
        let mut c = Conectare::noua();
        c.instantaneu = Some(instantaneu(&d).display().to_string());
        c.sursa.baza = "Ref".into();
        c.baza.baza = "Client".into();
        (Stare::noua(c, sesiuni), d)
    }

    fn comparata(s: &Stare) -> pornire::Comparata {
        pornire::compara(&s.conectare).expect("compararea merge")
    }

    /// Asteapta compararea de pe celalalt fir, cum face ceasul ferestrei.
    fn asteapta(s: &mut Stare) {
        for _ in 0..500 {
            if s.verifica_compararea() == Progres::Terminata {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("compararea nu s-a terminat");
    }

    #[test]
    fn compararea_de_pe_fir_ajunge_in_aplicatie() {
        let (mut s, d) = stare("fir");
        assert_eq!(s.verifica_compararea(), Progres::Nimic);
        s.compara();
        assert!(s.parte(true).is_none(), "formularul e blocat cat ruleaza");
        asteapta(&mut s);
        let app = s.app.as_ref().expect("aplicatia");
        assert_eq!(app.ecran(), Ecran::Lista);
        assert_eq!(s.referinta, "Ref");
        assert!(s.detalii.is_some(), "detaliile pornesc pe alt fir");
        assert!(s.rulare.is_none());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn revizuirea_salvata_revine_la_compararea_urmatoare() {
        let (mut s, d) = stare("sesiune");
        let r = comparata(&s);
        s.primeste_compararea(r);
        assert!(s.text_salvat(gmotion_core::Limba::Romana).is_empty());
        let app = s.app.as_mut().unwrap();
        app.comanda(Comanda::Selecteaza(0));
        app.comanda(Comanda::ComutaRevizuit);
        s.salveaza(None);
        assert!(!s.salvat_la.is_empty(), "salvarea spune ora");
        assert!(s
            .text_salvat(gmotion_core::Limba::Romana)
            .contains("ultima data la"));

        // alta pornire a aplicatiei, aceleasi sesiuni
        let sesiuni =
            gmotion_platforma::Sesiuni::in_director(modul::NUME, d.join("sesiuni")).unwrap();
        let mut alta = Stare::noua(s.conectare.clone(), sesiuni);
        let r = comparata(&alta);
        alta.primeste_compararea(r);
        assert_eq!(alta.app.as_ref().unwrap().revizuire().numar(), 1);
        assert_eq!(recente(&alta.sesiuni).len(), 1);
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn recompararea_pastreaza_revizuirea_si_vederea() {
        let (mut s, d) = stare("recomparare");
        let r = comparata(&s);
        s.primeste_compararea(r);
        let app = s.app.as_mut().unwrap();
        app.comanda(Comanda::Selecteaza(1));
        app.comanda(Comanda::ComutaRevizuit);
        app.comanda(Comanda::Cauta("pB".into()));
        let r = comparata(&s);
        s.primeste_compararea(r);
        let app = s.app.as_ref().unwrap();
        assert_eq!(app.revizuire().numar(), 1);
        assert_eq!(app.lista().vizibile().len(), 1, "cautarea ramane");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn deconectarea_arunca_tot_ce_tine_de_conexiune() {
        let (mut s, d) = stare("deconectare");
        let r = comparata(&s);
        s.primeste_compararea(r);
        s.salveaza(None);
        s.deconecteaza();
        assert!(s.app.is_none() && s.detalii.is_none() && s.deploy.is_none());
        assert!(s.referinta.is_empty());
        assert_eq!(s.conectare.recente.len(), 1, "compararea apare la recente");
        // si se poate reveni la ea
        s.conectare.sursa.baza.clear();
        s.reia(0);
        assert_eq!(s.conectare.sursa.baza, "Ref");
        s.ignora_recenta(0);
        assert!(s.conectare.recente.is_empty());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn campurile_conectarii_uita_ce_stiau_despre_serverul_vechi() {
        let (mut s, d) = stare("campuri");
        s.conectare.sursa.baze = vec!["X".into()];
        s.conectare.sursa.stare = StareConexiune::Conectat("ok".into());
        s.server_schimbat(true, "SRV2");
        assert_eq!(s.conectare.sursa.server, "SRV2");
        assert!(s.conectare.sursa.baze.is_empty());
        assert_eq!(s.conectare.sursa.stare, StareConexiune::Neincercat);
        s.utilizator_schimbat(false, "ion");
        assert_eq!(s.conectare.baza.acreditari.utilizator, "ion");
        s.auth_schimbat(false, true);
        assert!(s.conectare.baza.auth_windows);
        s.server_ales(false, "SRV3");
        assert_eq!(s.conectare.baza.server, "SRV3");
        let cert = s.conectare.verifica_certificatul;
        s.comuta_certificatul();
        assert_ne!(s.conectare.verifica_certificatul, cert);
        s.baza_schimbata(true, "Alta");
        assert_eq!(s.conectare.sursa.baza, "Alta");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn un_instantaneu_care_nu_se_citeste_nu_ramane_ales() {
        let (mut s, d) = stare("instantaneu");
        assert!(!s.alege_instantaneul("  "));
        assert!(!s.alege_instantaneul(&d.join("nu-exista.json").display().to_string()));
        assert!(s.conectare.instantaneu.is_none());
        assert!(matches!(s.conectare.sursa.stare, StareConexiune::Eroare(_)));
        assert!(s.alege_instantaneul(&d.join("inst.json").display().to_string()));
        assert_eq!(s.conectare.sursa.baze, ["Client", "Ref"]);
        assert!(s.conectare.sursa.baza.is_empty(), "baza se alege din nou");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn pozitia_din_editor_numara_vizibilele() {
        let (mut s, d) = stare("pozitie");
        assert!(s.text_pozitie(gmotion_core::Limba::Romana).is_empty());
        let r = comparata(&s);
        s.primeste_compararea(r);
        s.app.as_mut().unwrap().comanda(Comanda::Selecteaza(1));
        let t = s.text_pozitie(gmotion_core::Limba::Romana);
        assert!(t.starts_with("obiectul 2 din "), "{t}");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn compararea_offline_porneste_aplicatia() {
        let (mut s, d) = stare("offline");
        s.compara_offline("CREATE PROC p AS SELECT 1", "CREATE PROC p AS SELECT 2")
            .unwrap();
        assert!(s.app.is_some());
        assert_eq!(s.conectare.baza.baza, "Lipit");
        assert!(s.compara_offline("", "").is_err());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn folderul_ales_e_folosit_la_deploy() {
        let (mut s, d) = stare("folder-ales");
        s.referinta = "Ref".into();
        assert!(!s.director_deploy_curent().starts_with(&d));
        s.folder_deploy = Some(d.clone());
        assert!(s.director_deploy_curent().starts_with(&d));
        assert!(s.director_scris().is_none(), "nimic scris inca");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn fara_plan_deschis_nu_se_scrie_si_nu_se_marcheaza_nimic() {
        let (mut s, d) = stare("deploy");
        let r = comparata(&s);
        s.primeste_compararea(r);
        assert!(!s.scrie_deploy(&d.join("out")));
        assert!(!d.join("out").exists());
        s.porneste_garda();
        assert!(s.garda.is_some(), "garda porneste si fara plan");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn configurarea_stricata_nu_se_suprascrie() {
        let d = director("configurare");
        let cale = d.join("configurare.json");
        assert_eq!(
            citeste_configurarea(Some(&cale)).tema,
            gmotion_platforma::Configurare::default().tema
        );
        gmotion_platforma::Configurare::default()
            .scrie(&cale)
            .unwrap();
        salveaza_in_configurare(&cale, "tema", |c| c.tema = "luminos".into());
        assert_eq!(citeste_configurarea(Some(&cale)).tema, "luminos");
        std::fs::write(&cale, "{ stricat").unwrap();
        salveaza_in_configurare(&cale, "tema", |c| c.tema = "intunecat".into());
        assert_eq!(std::fs::read_to_string(&cale).unwrap(), "{ stricat");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn bazele_cerute_vin_de_pe_fir_la_bataia_ceasului() {
        let (mut s, d) = stare("baze");
        s.cere_bazele(true);
        assert!(s.baze_cerute.is_some());
        assert_eq!(s.conectare.sursa.stare, StareConexiune::SeConecteaza);
        let mut gata = false;
        for _ in 0..500 {
            if s.verifica_lucrarile() {
                gata = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(gata && s.baze_cerute.is_none());
        assert_eq!(s.conectare.sursa.baze, ["Client", "Ref"]);
        // fara server si fara instantaneu: se spune ce lipseste, nimic pe fir
        let sesiuni =
            gmotion_platforma::Sesiuni::in_director(modul::NUME, d.join("sesiuni")).unwrap();
        let mut goala = Stare::noua(Conectare::noua(), sesiuni);
        goala.cere_bazele(true);
        assert!(goala.baze_cerute.is_none());
        assert!(matches!(
            goala.conectare.sursa.stare,
            StareConexiune::Eroare(_)
        ));
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn previzualizarea_bifeaza_toate_bazele_si_compara() {
        let (mut s, d) = stare("previzualizare");
        s.conectare.sursa.baza.clear();
        s.conectare.baza.baza.clear();
        s.porneste_previzualizarea();
        assert_eq!(s.conectare.sursa.baza, "Client", "prima baza e referinta");
        assert!(s.rulare.is_some());
        asteapta(&mut s);
        assert!(s.app.is_some());
        std::fs::remove_dir_all(&d).ok();
    }

    /// F-A12: o comparare esuata se spune langa buton, nu pe martor.
    #[test]
    fn prima_comparare_esuata_se_spune_langa_buton() {
        let (mut s, d) = stare("esuata");
        s.conectare.baza.baza = "NuExista".into();
        s.compara();
        asteapta(&mut s);
        assert!(s.app.is_none());
        let e = s.conectare.eroare_comparare.clone().expect("eroarea");
        assert!(e.contains("NuExista"), "{e}");
        assert!(
            !matches!(s.conectare.sursa.stare, StareConexiune::Eroare(_)),
            "martorul n-are nicio vina"
        );
        // o comparare noua sterge eroarea veche
        s.conectare.baza.baza = "Client".into();
        s.compara();
        assert!(s.conectare.eroare_comparare.is_none());
        asteapta(&mut s);
        assert!(s.app.is_some());
        std::fs::remove_dir_all(&d).ok();
    }

    /// F-A8: o recomparare esuata se spune pe lista, iar lista ramane.
    #[test]
    fn recompararea_esuata_lasa_lista_si_spune_pe_ea() {
        let (mut s, d) = stare("recomparare-esuata");
        let r = comparata(&s);
        s.primeste_compararea(r);
        s.app.as_mut().unwrap().comanda(Comanda::ComutaInPlan);
        s.conectare.baza.baza = "NuExista".into();
        s.compara();
        asteapta(&mut s);
        assert!(s.conectare.eroare_comparare.is_none());
        let bara = s.text_bara_lista(gmotion_core::Limba::Romana);
        assert!(bara.starts_with("Recompararea a esuat"), "{bara}");
        assert_eq!(
            s.app.as_ref().unwrap().lista().in_plan().len(),
            1,
            "lista veche"
        );
        // dupa o recomparare reusita, mesajul pleaca, iar planul ramane
        s.conectare.baza.baza = "Client".into();
        s.compara();
        asteapta(&mut s);
        assert!(s.mesaj_lista.is_empty());
        assert_eq!(s.app.as_ref().unwrap().lista().in_plan().len(), 1);
        std::fs::remove_dir_all(&d).ok();
    }

    /// Etapa 25: recompararea isi arata progresul pe lista, iar Anuleaza o
    /// opreste si lasa lista veche.
    #[test]
    fn recompararea_are_progres_pe_lista_si_se_poate_anula() {
        let (mut s, d) = stare("progres-recomparare");
        s.compara();
        assert!(
            s.progres_recomparare(gmotion_core::Limba::Romana).is_none(),
            "prima comparare se vede pe conectare"
        );
        asteapta(&mut s);
        assert!(s.progres_recomparare(gmotion_core::Limba::Romana).is_none());
        s.compara();
        let (text, fractie) = s
            .progres_recomparare(gmotion_core::Limba::Romana)
            .expect("ruleaza");
        assert!(!text.is_empty() && (0.0..=1.0).contains(&fractie), "{text}");
        s.anuleaza();
        assert!(s
            .progres_recomparare(gmotion_core::Limba::Romana)
            .unwrap()
            .0
            .starts_with("se opreste"));
        asteapta(&mut s);
        assert!(s.app.is_some(), "lista ramane, oricum s-ar fi terminat");
        assert!(s.progres_recomparare(gmotion_core::Limba::Romana).is_none());
        std::fs::remove_dir_all(&d).ok();
    }

    /// F-A15: o sesiune care nu se citeste nu se suprascrie tacut.
    #[test]
    fn sesiunea_stricata_se_pune_deoparte_si_se_spune() {
        let (mut s, d) = stare("corupta");
        let r = comparata(&s);
        s.primeste_compararea(r);
        s.salveaza(None);
        let fisier = std::fs::read_dir(d.join("sesiuni"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        std::fs::write(&fisier, b"stricat").unwrap();
        s.deconecteaza();
        s.compara();
        asteapta(&mut s);
        let bara = s.text_bara_lista(gmotion_core::Limba::Romana);
        assert!(
            bara.contains("nu s-a putut citi") && bara.contains(".corupt"),
            "{bara}"
        );
        let mut deoparte = fisier.into_os_string();
        deoparte.push(".corupt");
        assert_eq!(std::fs::read(&deoparte).unwrap(), b"stricat");
        std::fs::remove_dir_all(&d).ok();
    }

    /// F-A16: dupa deconectare nu ramane nimic de la conexiunea veche.
    #[test]
    fn deconectarea_goleste_mesajele() {
        let (mut s, d) = stare("goleste");
        let r = comparata(&s);
        s.primeste_compararea(r);
        s.salveaza(None);
        s.mesaj_lista = "ceva".into();
        s.conectare.eroare_comparare = Some("veche".into());
        s.deconecteaza();
        assert!(s.salvat_la.is_empty() && s.mesaj_lista.is_empty());
        assert!(s.conectare.eroare_comparare.is_none());
        std::fs::remove_dir_all(&d).ok();
    }

    /// Etapa 26: verificarea pe server, pe un instantaneu, se termina cu
    /// refuzul lui (GM-DB-012) aratat in editor; alt obiect nu-l mosteneste.
    #[test]
    fn verificarea_pe_server_pe_instantaneu_spune_ca_nu_se_poate() {
        let (mut s, d) = stare("verificare");
        let r = comparata(&s);
        s.primeste_compararea(r);
        let app = s.app.as_mut().unwrap();
        app.comanda(Comanda::Selecteaza(0));
        app.comanda(Comanda::Deschide);
        assert!(s.text_verificare(gmotion_core::Limba::Romana).is_none());
        s.verifica_pe_server();
        let (t, _) = s
            .text_verificare(gmotion_core::Limba::Romana)
            .expect("in curs");
        assert!(t.starts_with("Se verifica"), "{t}");
        for _ in 0..500 {
            if s.verifica_lucrarile() && s.verificare.is_none() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let (t, bine) = s
            .text_verificare(gmotion_core::Limba::Romana)
            .expect("raspuns");
        assert!(!bine && t.contains("GM-DB-012"), "{t}");
        s.app.as_mut().unwrap().comanda(Comanda::Selecteaza(1));
        assert!(
            s.text_verificare(gmotion_core::Limba::Romana).is_none(),
            "raspunsul era pentru alt obiect"
        );
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn caile_de_iesire_poarta_referinta_curatata() {
        let c = cale_csv("Ref/1");
        let nume = c.file_name().unwrap().to_string_lossy();
        assert!(
            nume.starts_with("Ref_1_") && nume.ends_with(".csv"),
            "{nume}"
        );
        let d = director_deploy("Ref 1");
        let ales = director_deploy_in(Some(std::path::Path::new("/x/y")), "Ref 1");
        assert!(ales.starts_with("/x/y"), "{}", ales.display());
        assert!(ales
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("Ref_1_"));
        assert!(d
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("Ref_1_"));
    }
}
