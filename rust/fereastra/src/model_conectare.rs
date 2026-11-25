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
    /// Serverele gasite care se potrivesc cu ce scrie in campul de server
    /// (etapa 33: cu 15 gasite, lista trebuie sa se poata filtra). Fara
    /// diferenta intre litere mari si mici. Campul gol, sau exact numele unui
    /// server gasit (tocmai ales din lista), le arata pe toate: dupa alegere
    /// omul trebuie sa poata alege altul.
    pub fn servere_filtrate(&self) -> Vec<&ServerGasit> {
        let t = self.server.trim().to_lowercase();
        let ales = self
            .servere_gasite
            .iter()
            .any(|s| s.nume.to_lowercase() == t);
        self.servere_gasite
            .iter()
            .filter(|s| t.is_empty() || ales || s.nume.to_lowercase().contains(&t))
            .collect()
    }

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
    /// Bifa „Aceeasi conexiune" (etapa 17): modificata foloseste serverul,
    /// autentificarea si bazele martorului, iar pe ecran ramane doar lista ei
    /// de baze. Copia se face in `sincronizeaza`, nu se citeste de doua ori:
    /// restul aplicatiei vede tot doua parti complete.
    pub aceeasi_conexiune: bool,
    /// De ce n-a mers ultima comparare (sau ca a fost anulata), langa
    /// butonul Compara. Camp propriu: pus pe starea martorului, orice eroare
    /// (si „baza X nu exista" la modificata) cerea „conecteaza-te la martor"
    /// (F-A12).
    pub eroare_comparare: Option<String>,
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
            // de cele mai multe ori martorul si modificata sunt pe acelasi
            // server (in v3.4 era singurul caz): bifa porneste pusa
            aceeasi_conexiune: true,
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

    /// Pune sau scoate bifa „Aceeasi conexiune". Scoasa, modificata isi
    /// pastreaza ce a primit de la martor, ca sa nu se tasteze din nou.
    pub fn pune_aceeasi_conexiune(&mut self, da: bool) {
        self.aceeasi_conexiune = da;
        self.sincronizeaza();
    }

    /// Cu bifa pusa, copiaza conexiunea martorului in modificata: serverul,
    /// autentificarea, acreditarile, bazele citite si starea. Baza aleasa la
    /// modificata ramane doar daca exista in lista si nu e chiar a martorului.
    /// Se cheama inainte de fiecare desenare, deci ce schimba martorul ajunge
    /// si in modificata. Fara bifa nu face nimic.
    /// „Continua" pe o comparare recenta: serverul, martorul si modificata ei.
    /// Inainte se puneau doar numele bazelor: serverul de acum ramanea, iar cu
    /// „Aceeasi conexiune" modificata se golea la urmatoarea desenare, fiindca
    /// bazele nu erau inca citite (auditul 2026-11-23).
    /// Care sesiune recenta se arata pe ecranul de conectare: cea a perechii
    /// alese acum (martor si modificata), daca exista; altfel cea mai noua.
    /// `bool` spune daca e potrivita. `None` fara nicio sesiune.
    pub fn recenta_potrivita(&self) -> Option<(usize, bool)> {
        if self.recente.is_empty() {
            return None;
        }
        let (a, b) = (self.sursa.baza.as_str(), self.baza.baza.as_str());
        if !a.is_empty() && !b.is_empty() {
            if let Some(i) = self
                .recente
                .iter()
                .position(|r| r.sursa == a && r.baza == b)
            {
                return Some((i, true));
            }
        }
        // doar martorul ales: ultima sesiune cu el
        if !a.is_empty() {
            if let Some(i) = self.recente.iter().position(|r| r.sursa == a) {
                return Some((i, true));
            }
        }
        Some((0, false))
    }

    pub fn reia(&mut self, r: &Recenta) {
        if self.instantaneu.is_none() && !r.server.is_empty() && r.server != self.sursa.server {
            self.sursa.server.clone_from(&r.server);
            // bazele citite erau ale altui server
            self.sursa.baze.clear();
            self.baza.baze.clear();
        }
        self.sursa.baza.clone_from(&r.sursa);
        self.baza.alese.clear();
        self.baza.baza.clone_from(&r.baza);
        self.sincronizeaza();
    }

    pub fn sincronizeaza(&mut self) {
        if !self.aceeasi_conexiune || self.instantaneu.is_some() {
            return;
        }
        let s = &self.sursa;
        let b = &mut self.baza;
        b.server.clone_from(&s.server);
        b.auth_windows = s.auth_windows;
        b.acreditari.clone_from(&s.acreditari);
        b.baze.clone_from(&s.baze);
        b.stare.clone_from(&s.stare);
        b.servere_gasite.clear();
        b.alese.clear();
        // bazele inca necitite (lista goala) nu sunt un motiv sa golesti
        // alegerea: abia citirea spune daca baza exista
        let ramane = (b.baze.is_empty() || b.baze.iter().any(|x| x == &b.baza))
            && !b.baza.eq_ignore_ascii_case(&s.baza);
        if !ramane {
            b.baza.clear();
        }
    }

    /// Alege baza modificatei: una singura (Mihai, 2026-11-19). Cu „Aceeasi
    /// conexiune", baza martorului nu se poate alege: ar fi aceeasi baza.
    pub fn alege_baza_modificata(&mut self, nume: &str) {
        if self.e_baza_martorului(nume) {
            return;
        }
        self.baza.baza = nume.to_string();
        self.baza.alese.clear();
    }

    /// In lista modificatei, randul asta e chiar baza martorului (si deci nu
    /// se poate alege): doar cu „Aceeasi conexiune", unde listele sunt una.
    pub fn e_baza_martorului(&self, nume: &str) -> bool {
        self.aceeasi_conexiune
            && self.instantaneu.is_none()
            && !self.sursa.baza.is_empty()
            && self.sursa.baza.eq_ignore_ascii_case(nume)
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
                        "alege baza Before din instantaneu",
                        "choose the Before database from the snapshot",
                    )
                    .into(),
                ),
                (_, true) => Some(
                    l.alege(
                        "alege baza After din instantaneu",
                        "choose the modified database from the snapshot",
                    )
                    .into(),
                ),
                _ => self.acelasi_lucru(l),
            };
        }
        for (parte, ro, en) in [
            (&self.sursa, "Before", "Before"),
            (&self.baza, "After", "After"),
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
                    "Before si After sunt aceeasi baza",
                    "Before and After are the same database",
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
    /// Pe ecranul de conectare se vede o singura sesiune (Mihai, 2026-11-25):
    /// a perechii alese, altfel a martorului ales, altfel cea mai noua.
    #[test]
    fn sesiunea_aratata_e_cea_potrivita_altfel_cea_mai_noua() {
        let r = |a: &str, b: &str| Recenta {
            sursa: a.into(),
            server: "S".into(),
            baza: b.into(),
            cand: String::new(),
            progres: String::new(),
        };
        let mut c = Conectare::noua();
        assert_eq!(c.recenta_potrivita(), None);
        c.recente = vec![r("A", "B"), r("GetsBet", "Copy"), r("GetsBet", "Alta")];
        assert_eq!(
            c.recenta_potrivita(),
            Some((0, false)),
            "nimic ales: cea mai noua"
        );
        c.sursa.baza = "GetsBet".into();
        assert_eq!(c.recenta_potrivita(), Some((1, true)), "doar martorul");
        c.baza.baza = "Alta".into();
        assert_eq!(c.recenta_potrivita(), Some((2, true)), "perechea");
        c.sursa.baza = "Nimic".into();
        assert_eq!(c.recenta_potrivita(), Some((0, false)));
    }

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

    fn martor_conectat() -> Conectare {
        let mut c = Conectare::noua();
        c.sursa = parte_gata("SRV-DEV", "Dev");
        c.sursa.baze = vec!["Dev".into(), "Prod".into(), "Test".into()];
        c.sursa.auth_windows = false;
        c.sursa.acreditari = Acreditari::noi("deploy_ro", "secret");
        c
    }

    #[test]
    fn continua_pe_o_recenta_pune_serverul_si_ambele_baze() {
        let mut c = Conectare {
            aceeasi_conexiune: true,
            ..Default::default()
        };
        c.sursa.server = "SRV-ACUM".into();
        c.sursa.baze = vec!["A".into(), "B".into()];
        let r = Recenta {
            sursa: "Martor".into(),
            server: "SRV-VECHI".into(),
            baza: "Client".into(),
            cand: String::new(),
            progres: String::new(),
        };
        c.reia(&r);
        assert_eq!(c.sursa.server, "SRV-VECHI");
        assert_eq!(c.sursa.baza, "Martor");
        assert_eq!(c.baza.baza, "Client");
        c.sincronizeaza();
        assert_eq!(c.baza.baza, "Client", "nu se goleste inainte de citire");
        assert_eq!(c.baza.server, "SRV-VECHI");
    }

    #[test]
    fn aceeasi_conexiune_copiaza_martorul_in_modificata() {
        let mut c = martor_conectat();
        c.baza = parte_gata("ALT", "Altceva");
        c.pune_aceeasi_conexiune(true);
        assert_eq!(c.baza.server, "SRV-DEV");
        assert!(!c.baza.auth_windows);
        assert_eq!(c.baza.acreditari.utilizator, "deploy_ro");
        assert_eq!(c.baza.acreditari.parola(), "secret");
        assert_eq!(c.baza.baze, c.sursa.baze);
        assert_eq!(c.baza.stare, c.sursa.stare);
        // „Altceva" nu e pe serverul martorului: nu ramane aleasa
        assert_eq!(c.baza.baza, "");
        c.alege_baza_modificata("Prod");
        assert!(c.poate_compara(), "{:?}", c.ce_lipseste());

        // martorul se schimba: modificata urmeaza la urmatoarea sincronizare
        c.sursa.server = "SRV-NOU".into();
        c.sursa.baze = vec!["Prod".into(), "Altele".into()];
        c.sincronizeaza();
        assert_eq!(c.baza.server, "SRV-NOU");
        assert_eq!(c.baza.baza, "Prod", "Prod exista si acolo: ramane");
        c.sursa.baze = vec!["Altele".into()];
        c.sincronizeaza();
        assert_eq!(c.baza.baza, "", "Prod nu mai exista: se goleste");
    }

    #[test]
    fn fara_bifa_modificata_isi_tine_conexiunea() {
        let mut c = martor_conectat();
        c.aceeasi_conexiune = false;
        c.baza = parte_gata("ALT", "Altceva");
        c.sincronizeaza();
        assert_eq!(c.baza.server, "ALT");
        // bifa scoasa dupa ce a fost pusa: ramane ce s-a copiat
        c.pune_aceeasi_conexiune(true);
        c.pune_aceeasi_conexiune(false);
        c.sursa.server = "SRV-NOU".into();
        c.sincronizeaza();
        assert_eq!(c.baza.server, "SRV-DEV");
    }

    #[test]
    fn la_modificata_se_alege_o_singura_baza() {
        let mut c = martor_conectat();
        c.pune_aceeasi_conexiune(true);
        c.alege_baza_modificata("Prod");
        c.alege_baza_modificata("Test");
        assert_eq!(c.baza.baze_alese(), ["Test"]);
        // baza martorului nu se poate alege cand conexiunea e aceeasi
        assert!(c.e_baza_martorului("dev"));
        c.alege_baza_modificata("Dev");
        assert_eq!(c.baza.baza, "Test");
        // martorul trece pe baza aleasa la modificata: aceea se goleste
        c.sursa.baza = "Test".into();
        c.sincronizeaza();
        assert_eq!(c.baza.baza, "");
        // fara bifa, pe alt server, acelasi nume se poate alege
        c.pune_aceeasi_conexiune(false);
        c.baza.server = "ALT".into();
        assert!(!c.e_baza_martorului("Test"));
        c.alege_baza_modificata("Test");
        assert_eq!(c.baza.baza, "Test");
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
            Some("completeaza serverul pentru Before")
        );
        c.sursa.server = "SRV-DEV".into();
        assert_eq!(c.ce_lipseste().as_deref(), Some("alege baza pentru Before"));
        c.sursa.baza = "Dev".into();
        assert_eq!(c.ce_lipseste().as_deref(), Some("conecteaza-te la Before"));
        c.sursa.stare = StareConexiune::Conectat("conectat".into());
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("completeaza serverul pentru After")
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
            Some("Before si After sunt aceeasi baza"),
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
            Some("Before si After sunt aceeasi baza"),
            "acelasi server si aceeasi baza, indiferent de majuscule"
        );
        // alt server, acelasi nume de baza: e o comparare buna
        c.baza.server = "ALT-SRV".into();
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
            Some("alege baza Before din instantaneu")
        );
        c.sursa.baza = "A".into();
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("alege baza After din instantaneu")
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
            Some("fill in the server for Before")
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
            Some("choose the Before database from the snapshot")
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

    #[test]
    fn serverele_gasite_se_filtreaza_dupa_camp() {
        let mut p = Parte {
            servere_gasite: [
                "TS2409\\MIHAIG",
                "TS2409\\SQL2019",
                "SRV-PROD",
                "srv-test\\A",
            ]
            .into_iter()
            .map(ServerGasit::nou)
            .collect(),
            ..Default::default()
        };
        let nume = |p: &Parte| -> Vec<String> {
            p.servere_filtrate()
                .iter()
                .map(|s| s.nume.clone())
                .collect()
        };
        assert_eq!(nume(&p).len(), 4, "campul gol: toate");
        p.server = "ts2409".into();
        assert_eq!(nume(&p), vec!["TS2409\\MIHAIG", "TS2409\\SQL2019"]);
        p.server = "SRV".into();
        assert_eq!(nume(&p), vec!["SRV-PROD", "srv-test\\A"]);
        p.server = "nimic".into();
        assert!(nume(&p).is_empty());
        // tocmai ales din lista: toate, ca sa se poata alege altul
        p.server = "srv-prod".into();
        assert_eq!(nume(&p).len(), 4);
    }
}
