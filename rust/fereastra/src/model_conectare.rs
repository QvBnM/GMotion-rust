//! Modelul ecranului de conectare: ce s-a completat, ce lipseste, ce se poate
//! apasa, si ce comparari recente se pot relua.
//!
//! **Se strica astfel:** butonul „Compara" e apasabil cu campuri lipsa (si
//! erorile apar abia dupa ce astepti conectarea), inversarea pierde parolele
//! pe drum, sau — cel mai rau — parola ajunge intr-un jurnal. De-aia starea
//! sta aici, se testeaza fara fereastra, iar tipul care tine parola **nu are
//! `Debug` derivat**.

use gmotion_core::Limba;

/// Cum se autentifica o parte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Auth {
    /// Autentificare integrata Windows: fara utilizator si parola.
    Windows,
    Sql,
}

/// Utilizatorul si parola pentru autentificare SQL.
///
/// **Fara `Debug` derivat**: ar tipari parola la primul `dbg!` sau in orice
/// mesaj de panica. Vezi `rust/db/src/sqlserver.rs`, acelasi motiv.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Acreditari {
    pub utilizator: String,
    parola: String,
}

impl Acreditari {
    pub fn noi(utilizator: impl Into<String>, parola: impl Into<String>) -> Self {
        Acreditari {
            utilizator: utilizator.into(),
            parola: parola.into(),
        }
    }

    /// Parola, pentru singurul loc care are voie sa o foloseasca: construirea
    /// conexiunii. Nu se pune in structuri care se serializeaza.
    pub fn parola(&self) -> &str {
        &self.parola
    }

    pub fn pune_parola(&mut self, p: impl Into<String>) {
        self.parola = p.into();
    }

    pub fn are_parola(&self) -> bool {
        !self.parola.is_empty()
    }
}

impl std::fmt::Debug for Acreditari {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Acreditari")
            .field("utilizator", &self.utilizator)
            .field("parola", &"<redactata>")
            .finish()
    }
}

/// Un server gasit in retea. Etichetele vin din raspunsul SSRP
/// (`gmotion_db::Instanta`), nu din ghicitul numelui.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerGasit {
    pub nume: String,
    /// Instanta implicita (fara `\instanta` in nume).
    pub implicita: bool,
    pub cluster: bool,
}

impl ServerGasit {
    pub fn nou(nume: impl Into<String>) -> Self {
        ServerGasit {
            nume: nume.into(),
            ..Default::default()
        }
    }
}

/// Unde a ajuns incercarea de conectare a unei parti.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StareConexiune {
    #[default]
    Neincercat,
    SeConecteaza,
    /// Cautarea serverelor in retea ruleaza (pe alt fir): fereastra nu
    /// ingheata, iar starea spune ce se intampla (15 F).
    SeCauta,
    /// „conectat · SQL Server 2019 · 21.204 obiecte"
    Conectat(String),
    /// Mesajul exact, cu codul lui. Niciodata parola.
    Eroare(String),
}

impl StareConexiune {
    pub fn text(&self) -> &str {
        self.text_in(Limba::Romana)
    }

    /// `text`, in limba data. Mesajele de la server raman cum au venit.
    pub fn text_in(&self, l: Limba) -> &str {
        match self {
            StareConexiune::Neincercat => "",
            StareConexiune::SeConecteaza => l.alege("se conecteaza…", "connecting…"),
            StareConexiune::SeCauta => l.alege(
                "se cauta servere in retea…",
                "searching the network for servers…",
            ),
            StareConexiune::Conectat(t) => t,
            StareConexiune::Eroare(t) => t,
        }
    }

    pub fn e_conectat(&self) -> bool {
        matches!(self, StareConexiune::Conectat(_))
    }
}

/// O parte a compararii: de unde vine codul (sursa) sau unde ajunge (baza).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Parte {
    pub server: String,
    pub acreditari: Acreditari,
    pub baza: String,
    /// Bazele citite de pe server, pentru lista de alegere.
    pub baze: Vec<String>,
    /// Bazele bifate, cand aceeasi referinta se compara cu mai multe. Goala
    /// inseamna „doar `baza`"; prima bifata e mereu si `baza`, ca restul
    /// ecranului (titlul) sa aiba ce arata.
    pub alese: Vec<String>,
    pub stare: StareConexiune,
    pub auth_windows: bool,
    /// Instantele gasite in retea (SQL Browser), de ales cu un clic. Goala
    /// pana se apasa cautarea.
    pub servere_gasite: Vec<ServerGasit>,
}

impl Parte {
    /// Bazele cu care se compara efectiv: cele bifate, sau cea aleasa.
    pub fn baze_alese(&self) -> Vec<String> {
        if self.alese.is_empty() {
            if self.baza.is_empty() {
                Vec::new()
            } else {
                vec![self.baza.clone()]
            }
        } else {
            self.alese.clone()
        }
    }

    /// Bifeaza sau debifeaza o baza. `baza` ramane prima bifata, ca ecranele
    /// care arata o singura baza sa nu ramana cu una debifata.
    pub fn comuta_baza(&mut self, nume: &str) {
        match self.alese.iter().position(|b| b == nume) {
            Some(i) => {
                self.alese.remove(i);
            }
            None => self.alese.push(nume.to_string()),
        }
        self.baza = self.alese.first().cloned().unwrap_or_default();
    }

    pub fn e_aleasa(&self, nume: &str) -> bool {
        self.alese.iter().any(|b| b == nume) || (self.alese.is_empty() && self.baza == nume)
    }

    pub fn auth(&self) -> Auth {
        if self.auth_windows {
            Auth::Windows
        } else {
            Auth::Sql
        }
    }

    /// Ce lipseste ca sa se poata conecta. `None` = se poate incerca.
    pub fn ce_lipseste(&self) -> Option<&'static str> {
        if self.server.trim().is_empty() {
            return Some("serverul");
        }
        if !self.auth_windows {
            if self.acreditari.utilizator.trim().is_empty() {
                return Some("utilizatorul");
            }
            if !self.acreditari.are_parola() {
                return Some("parola");
            }
        }
        None
    }
}

/// (gazda, instanta, port) pentru comparat: litere mici, gazda locala sub un
/// singur nume, domeniul taiat (`srv.firma.local` = `srv`), instanta implicita
/// si portul implicit (1433) scrise la fel ca lipsa lor.
fn identitate_instanta(server: &str) -> (String, String, u16) {
    let s = server.trim().to_lowercase();
    let (fara_port, port) = match s.rsplit_once(',') {
        Some((a, p)) => match p.trim().parse::<u16>() {
            Ok(n) => (a.trim().to_string(), n),
            Err(_) => (s.clone(), 1433),
        },
        None => (s.clone(), 1433),
    };
    let (gazda, instanta) = match fara_port.split_once('\\') {
        Some((g, i)) => (g.to_string(), i.to_string()),
        None => (fara_port, String::new()),
    };
    let instanta = if instanta == "mssqlserver" {
        String::new()
    } else {
        instanta
    };
    let gazda = match gazda.as_str() {
        "." | "(local)" | "localhost" | "127.0.0.1" | "::1" => "localhost".to_string(),
        g if g.parse::<std::net::Ipv4Addr>().is_ok() => g.to_string(),
        g => g.split('.').next().unwrap_or(g).to_string(),
    };
    (gazda, instanta, port)
}

/// Numele campului lipsa (`Parte::ce_lipseste`) in limba data.
pub fn camp_in(ce: &str, l: Limba) -> &str {
    match (ce, l) {
        (_, Limba::Romana) => ce,
        ("serverul", _) => "the server",
        ("utilizatorul", _) => "the user name",
        ("parola", _) => "the password",
        (alt, _) => alt,
    }
}

/// „completeaza serverul" / „fill in the server": ce se spune cand lipseste
/// un camp inainte de citirea bazelor.
pub fn completeaza(ce: &str, l: Limba) -> String {
    match l {
        Limba::Romana => format!("completeaza {ce}"),
        Limba::Engleza => format!("fill in {}", camp_in(ce, l)),
    }
}

/// O comparare facuta inainte, gata de reluat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recenta {
    pub sursa: String,
    /// Serverul (sau instantaneul) pe care s-a facut compararea. Nu se
    /// deseneaza, dar face parte din cheia sesiunii: fara el, „nu mai arata
    /// compararea asta" ar sterge sesiunea altui server (F-B4).
    pub server: String,
    pub baza: String,
    /// „ieri, 17:40" sau data exacta.
    pub cand: String,
    /// „124 din 336 revizuite", sau gol daca nu s-a revizuit nimic.
    pub progres: String,
}

/// Starea ecranului.
#[derive(Debug, Clone, Default)]
pub struct Conectare {
    pub sursa: Parte,
    pub baza: Parte,
    /// Se verifica certificatul serverului cu depozitul sistemului? Implicit
    /// nu, ca in v3.4. Se schimba din legatura „Schimba". **Asta** ajunge in
    /// conexiune, nu doar textul: pana la 2026-11-18 ecranul spunea „se
    /// verifica", iar conexiunea tot nu verifica.
    pub verifica_certificatul: bool,
    pub recente: Vec<Recenta>,
    /// Instantaneul deschis in locul serverelor („Deschide un instantaneu…").
    pub instantaneu: Option<String>,
    /// Limba mesajelor facute in Rust pentru ecranul asta (stari, erori de
    /// validare); o tine la zi comutatorul de limba.
    pub limba: Limba,
    /// `timeout_conectare_s` din configurare; `None` = implicitul din `db`.
    pub timeout_conectare_s: Option<u64>,
}

impl Conectare {
    pub fn noua() -> Self {
        Conectare {
            sursa: Parte {
                auth_windows: true,
                ..Parte::default()
            },
            baza: Parte {
                auth_windows: true,
                ..Parte::default()
            },
            ..Conectare::default()
        }
    }

    /// Textul despre certificat, ca in macheta. Gol pe un instantaneu: acolo
    /// nu exista server, deci nici certificat.
    pub fn text_certificat(&self) -> &'static str {
        self.text_certificat_in(Limba::Romana)
    }

    /// `text_certificat`, in limba data.
    pub fn text_certificat_in(&self, l: Limba) -> &'static str {
        if self.instantaneu.is_some() {
            ""
        } else if self.verifica_certificatul {
            l.alege(
                "Certificatul serverului se verifica cu depozitul sistemului.",
                "The server certificate is checked against the system store.",
            )
        } else {
            l.alege(
                "Certificatul serverului nu se verifica.",
                "The server certificate is not checked.",
            )
        }
    }

    /// Ce lipseste ca sa se poata citi bazele unei parti. Pe un instantaneu,
    /// nimic: bazele vin din fisier, nu de pe un server.
    pub fn ce_lipseste_pentru_baze(&self, sursa: bool) -> Option<&'static str> {
        if self.instantaneu.is_some() {
            return None;
        }
        if sursa {
            self.sursa.ce_lipseste()
        } else {
            self.baza.ce_lipseste()
        }
    }

    /// Inverseaza cele doua parti, cu tot cu acreditari si baze citite: altfel
    /// ar trebui tastate din nou, exact cand omul a observat ca a pus invers.
    pub fn inverseaza(&mut self) {
        std::mem::swap(&mut self.sursa, &mut self.baza);
    }

    /// Ce lipseste ca sa se poata compara. `None` = butonul e apasabil.
    ///
    /// Mesajul e pentru om si spune **ce** lipseste, nu ca „formularul e
    /// invalid": asta e tot rostul lui.
    pub fn ce_lipseste(&self) -> Option<String> {
        self.ce_lipseste_in(Limba::Romana)
    }

    /// `ce_lipseste`, in limba data.
    pub fn ce_lipseste_in(&self, l: Limba) -> Option<String> {
        if self.instantaneu.is_some() {
            return match (self.sursa.baza.is_empty(), self.baza.baza.is_empty()) {
                (true, _) => Some(
                    l.alege(
                        "alege baza martor din instantaneu",
                        "choose the witness database from the snapshot",
                    )
                    .into(),
                ),
                (_, true) => Some(
                    l.alege(
                        "alege baza modificata din instantaneu",
                        "choose the modified database from the snapshot",
                    )
                    .into(),
                ),
                _ => self.acelasi_lucru(l),
            };
        }
        for (parte, ro, en) in [
            (&self.sursa, "martor", "the witness"),
            (&self.baza, "modificata", "the modified side"),
        ] {
            let nume = l.alege(ro, en);
            if let Some(ce) = parte.ce_lipseste() {
                return Some(match l {
                    Limba::Romana => format!("completeaza {ce} pentru {nume}"),
                    Limba::Engleza => format!("fill in {} for {nume}", camp_in(ce, l)),
                });
            }
            if parte.baza.is_empty() {
                return Some(format!(
                    "{} {nume}",
                    l.alege("alege baza pentru", "choose the database for")
                ));
            }
            if !parte.stare.e_conectat() {
                return Some(format!(
                    "{} {nume}",
                    l.alege("conecteaza-te la", "connect to")
                ));
            }
        }
        self.acelasi_lucru(l)
    }

    fn acelasi_lucru(&self, l: Limba) -> Option<String> {
        let acelasi_server =
            self.instantaneu.is_some() || self.sursa.server.eq_ignore_ascii_case(&self.baza.server);
        if acelasi_server
            && self
                .baza
                .baze_alese()
                .iter()
                .any(|b| self.sursa.baza.eq_ignore_ascii_case(b))
        {
            return Some(
                l.alege(
                    "martorul si modificata sunt aceeasi baza",
                    "the witness and the modified database are the same",
                )
                .into(),
            );
        }
        None
    }

    /// Martorul si modificata stau pe **aceeasi instanta** de SQL Server
    /// (15 F): alt nume pentru acelasi server (`.` / `localhost` /
    /// `127.0.0.1`, numele scurt fata de cel cu domeniu, portul implicit
    /// scris sau nu) se recunoaste. Pe instantaneu nu se spune nimic.
    pub fn aceeasi_instanta(&self) -> bool {
        if self.instantaneu.is_some()
            || self.sursa.server.trim().is_empty()
            || self.baza.server.trim().is_empty()
        {
            return false;
        }
        identitate_instanta(&self.sursa.server) == identitate_instanta(&self.baza.server)
    }

    pub fn poate_compara(&self) -> bool {
        self.ce_lipseste().is_none()
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    fn parte_gata(server: &str, baza: &str) -> Parte {
        Parte {
            server: server.into(),
            acreditari: Acreditari::default(),
            baza: baza.into(),
            baze: vec![baza.into()],
            stare: StareConexiune::Conectat("conectat".into()),
            auth_windows: true,
            servere_gasite: Vec::new(),
            alese: Vec::new(),
        }
    }

    #[test]
    fn parola_nu_apare_in_debug() {
        let a = Acreditari::noi("deploy_ro", "parola-secreta");
        let text = format!("{a:?}");
        assert!(!text.contains("parola-secreta"), "{text}");
        assert!(text.contains("redactata"), "{text}");
        assert!(
            text.contains("deploy_ro"),
            "utilizatorul nu e secret: {text}"
        );
        // si prin structurile care o contin
        let p = Parte {
            acreditari: a,
            ..parte_gata("SRV", "B")
        };
        assert!(!format!("{p:?}").contains("parola-secreta"));
        let mut c = Conectare::noua();
        c.sursa = p;
        assert!(!format!("{c:?}").contains("parola-secreta"));
    }

    #[test]
    fn butonul_spune_ce_lipseste_nu_doar_ca_nu_merge() {
        let mut c = Conectare::noua();
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("completeaza serverul pentru martor")
        );
        c.sursa.server = "SRV-DEV".into();
        assert_eq!(c.ce_lipseste().as_deref(), Some("alege baza pentru martor"));
        c.sursa.baza = "Dev".into();
        assert_eq!(c.ce_lipseste().as_deref(), Some("conecteaza-te la martor"));
        c.sursa.stare = StareConexiune::Conectat("conectat".into());
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("completeaza serverul pentru modificata")
        );
        c.baza = parte_gata("SRV-PROD", "Prod");
        assert!(c.poate_compara(), "{:?}", c.ce_lipseste());
    }

    #[test]
    fn autentificarea_sql_cere_utilizator_si_parola() {
        let mut p = parte_gata("SRV", "B");
        p.auth_windows = false;
        assert_eq!(p.ce_lipseste(), Some("utilizatorul"));
        p.acreditari.utilizator = "deploy_ro".into();
        assert_eq!(p.ce_lipseste(), Some("parola"));
        p.acreditari.pune_parola("x");
        assert_eq!(p.ce_lipseste(), None);
        // autentificarea Windows nu cere nimic in plus
        p.auth_windows = true;
        p.acreditari = Acreditari::default();
        assert_eq!(p.ce_lipseste(), None);
    }

    #[test]
    fn bifele_aleg_mai_multe_baze_si_prima_ramane_cea_aratata() {
        let mut p = parte_gata("SRV", "Client1");
        p.baze = vec!["Client1".into(), "Client2".into(), "Client3".into()];
        assert_eq!(p.baze_alese(), ["Client1"], "fara bife, baza aleasa");
        assert!(p.e_aleasa("Client1") && !p.e_aleasa("Client2"));

        p.comuta_baza("Client2");
        assert_eq!(p.baze_alese(), ["Client2"]);
        assert_eq!(p.baza, "Client2", "prima bifata e si cea aratata");
        p.comuta_baza("Client3");
        assert_eq!(p.baze_alese(), ["Client2", "Client3"]);
        assert_eq!(p.baza, "Client2");
        // debifarea primei o muta pe urmatoarea
        p.comuta_baza("Client2");
        assert_eq!(p.baze_alese(), ["Client3"]);
        assert_eq!(p.baza, "Client3");
        // fara nicio bifa nu mai e nimic de comparat
        p.comuta_baza("Client3");
        assert!(p.baze_alese().is_empty());
        assert_eq!(p.baza, "");
    }

    #[test]
    fn sursa_bifata_si_printre_tinte_e_tot_aceeasi_comparare() {
        let mut c = Conectare::noua();
        c.sursa = parte_gata("SRV", "Referinta");
        c.baza = parte_gata("SRV", "Client1");
        c.baza.comuta_baza("Client1");
        c.baza.comuta_baza("Referinta");
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("martorul si modificata sunt aceeasi baza"),
            "referinta bifata printre tinte nu are ce compara cu ea insasi"
        );
    }

    #[test]
    fn aceeasi_baza_de_doua_ori_nu_e_o_comparare() {
        let mut c = Conectare::noua();
        c.sursa = parte_gata("SRV", "Aceeasi");
        c.baza = parte_gata("srv", "aceeasi");
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("martorul si modificata sunt aceeasi baza"),
            "acelasi server si aceeasi baza, indiferent de majuscule"
        );
        // alt server, acelasi nume de baza: e o comparare buna
        c.baza.server = "ALT-SRV".into();
        assert!(c.poate_compara());
    }

    #[test]
    fn inversarea_muta_tot_inclusiv_acreditarile_si_bazele_citite() {
        let mut c = Conectare::noua();
        c.sursa = parte_gata("SRV-DEV", "Dev");
        c.baza = Parte {
            auth_windows: false,
            acreditari: Acreditari::noi("deploy_ro", "x"),
            ..parte_gata("SRV-PROD", "Prod")
        };
        c.inverseaza();
        assert_eq!(c.sursa.server, "SRV-PROD");
        assert_eq!(c.sursa.acreditari.utilizator, "deploy_ro");
        assert!(c.sursa.acreditari.are_parola(), "parola nu se pierde");
        assert_eq!(c.baza.server, "SRV-DEV");
        assert_eq!(c.baza.baze, ["Dev"]);
        assert!(c.poate_compara());
    }

    /// Bug de pe Windows (2026-11-18): pornit cu `--sursa`, lista de baze
    /// cerea server si scria „completeaza serverul".
    #[test]
    fn pe_instantaneu_bazele_nu_cer_server() {
        let mut c = Conectare::noua();
        assert_eq!(c.ce_lipseste_pentru_baze(true), Some("serverul"));
        c.instantaneu = Some("/tmp/inst.json".into());
        assert_eq!(c.ce_lipseste_pentru_baze(true), None);
        assert_eq!(c.ce_lipseste_pentru_baze(false), None);
    }

    #[test]
    fn textul_certificatului_urmeaza_alegerea_si_dispare_pe_instantaneu() {
        let mut c = Conectare::noua();
        assert!(!c.verifica_certificatul, "implicit ca in v3.4");
        assert!(c.text_certificat().contains("nu se verifica"));
        c.verifica_certificatul = true;
        assert!(c.text_certificat().contains("depozitul sistemului"));
        c.instantaneu = Some("/tmp/inst.json".into());
        assert_eq!(c.text_certificat(), "");
    }

    #[test]
    fn cu_instantaneu_nu_se_mai_cer_servere() {
        let mut c = Conectare::noua();
        c.instantaneu = Some("/tmp/inst.json".into());
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("alege baza martor din instantaneu")
        );
        c.sursa.baza = "A".into();
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("alege baza modificata din instantaneu")
        );
        c.baza.baza = "B".into();
        assert!(c.poate_compara(), "{:?}", c.ce_lipseste());
        // aceeasi baza din acelasi instantaneu: tot nu e o comparare
        c.baza.baza = "A".into();
        assert!(!c.poate_compara());
    }

    #[test]
    fn starea_conexiunii_are_text_pentru_fiecare_caz() {
        assert_eq!(StareConexiune::Neincercat.text(), "");
        assert_eq!(StareConexiune::SeConecteaza.text(), "se conecteaza…");
        assert!(StareConexiune::Conectat("conectat · 21.204 obiecte".into()).e_conectat());
        assert_eq!(
            StareConexiune::Eroare("[GM-DB-002] autentificare".into()).text(),
            "[GM-DB-002] autentificare"
        );
        assert!(!StareConexiune::Eroare("x".into()).e_conectat());
    }

    #[test]
    fn mesajele_conectarii_urmeaza_limba() {
        let e = Limba::Engleza;
        let mut c = Conectare::noua();
        assert_eq!(
            c.ce_lipseste_in(e).as_deref(),
            Some("fill in the server for the witness")
        );
        assert_eq!(
            c.text_certificat_in(e),
            "The server certificate is not checked."
        );
        assert_eq!(StareConexiune::SeConecteaza.text_in(e), "connecting…");
        assert_eq!(completeaza("parola", e), "fill in the password");
        assert_eq!(completeaza("parola", Limba::Romana), "completeaza parola");
        c.instantaneu = Some("/tmp/i.json".into());
        assert_eq!(
            c.ce_lipseste_in(e).as_deref(),
            Some("choose the witness database from the snapshot")
        );
    }

    #[test]
    fn aceeasi_instanta_se_recunoaste_sub_alte_nume() {
        let cu = |a: &str, b: &str| {
            let mut c = Conectare::noua();
            c.sursa.server = a.into();
            c.baza.server = b.into();
            c.aceeasi_instanta()
        };
        assert!(cu("SRV-PROD", "srv-prod"));
        assert!(cu("srv-prod.firma.local,1433", "SRV-PROD"));
        assert!(cu(".", "localhost"));
        assert!(cu("(local)\\MSSQLSERVER", "127.0.0.1"));
        assert!(cu("SRV\\SQL2019", "srv.firma.local\\sql2019"));
        assert!(
            !cu("SRV\\SQL2019", "SRV"),
            "alta instanta pe aceeasi masina"
        );
        assert!(!cu("SRV,1433", "SRV,1434"), "alt port");
        assert!(!cu("10.0.0.1", "10.0.0.2"));
        assert!(!cu("SRV-DEV", "SRV-PROD"));
        assert!(!cu("SRV", ""), "fara server, nu se spune nimic");
        let mut c = Conectare::noua();
        c.instantaneu = Some("/tmp/i.json".into());
        c.sursa.server = "A".into();
        c.baza.server = "A".into();
        assert!(!c.aceeasi_instanta(), "pe instantaneu nu exista instanta");
        assert_eq!(
            StareConexiune::SeCauta.text_in(Limba::Engleza),
            "searching the network for servers…"
        );
    }
}
