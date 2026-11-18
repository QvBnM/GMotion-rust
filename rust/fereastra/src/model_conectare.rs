//! Modelul ecranului de conectare: ce s-a completat, ce lipseste, ce se poate
//! apasa, si ce comparari recente se pot relua.
//!
//! **Se strica astfel:** butonul „Compara" e apasabil cu campuri lipsa (si
//! erorile apar abia dupa ce astepti conectarea), inversarea pierde parolele
//! pe drum, sau — cel mai rau — parola ajunge intr-un jurnal. De-aia starea
//! sta aici, se testeaza fara fereastra, iar tipul care tine parola **nu are
//! `Debug` derivat**.

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

/// Unde a ajuns incercarea de conectare a unei parti.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StareConexiune {
    #[default]
    Neincercat,
    SeConecteaza,
    /// „conectat · SQL Server 2019 · 21.204 obiecte"
    Conectat(String),
    /// Mesajul exact, cu codul lui. Niciodata parola.
    Eroare(String),
}

impl StareConexiune {
    pub fn text(&self) -> &str {
        match self {
            StareConexiune::Neincercat => "",
            StareConexiune::SeConecteaza => "se conecteaza…",
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
    pub stare: StareConexiune,
    pub auth_windows: bool,
}

impl Parte {
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

/// O comparare facuta inainte, gata de reluat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recenta {
    pub sursa: String,
    pub baza: String,
    /// `PROD` / `TEST` / `DEV`, sau gol.
    pub mediu: String,
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
    /// Textul despre certificat, ca in macheta: „Certificatul serverului nu se
    /// verifica." Se schimba din legatura „Schimba".
    pub certificat: String,
    pub recente: Vec<Recenta>,
    /// Instantaneul deschis in locul serverelor („Deschide un instantaneu…").
    pub instantaneu: Option<String>,
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
            certificat: "Certificatul serverului nu se verifica.".to_string(),
            ..Conectare::default()
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
        if self.instantaneu.is_some() {
            return match (self.sursa.baza.is_empty(), self.baza.baza.is_empty()) {
                (true, _) => Some("alege baza sursa din instantaneu".into()),
                (_, true) => Some("alege baza de comparat din instantaneu".into()),
                _ => self.acelasi_lucru(),
            };
        }
        for (parte, nume) in [(&self.sursa, "sursa"), (&self.baza, "baza")] {
            if let Some(ce) = parte.ce_lipseste() {
                return Some(format!("completeaza {ce} pentru {nume}"));
            }
            if parte.baza.is_empty() {
                return Some(format!("alege baza pentru {nume}"));
            }
            if !parte.stare.e_conectat() {
                return Some(format!("conecteaza-te la {nume}"));
            }
        }
        self.acelasi_lucru()
    }

    fn acelasi_lucru(&self) -> Option<String> {
        let acelasi_server =
            self.instantaneu.is_some() || self.sursa.server.eq_ignore_ascii_case(&self.baza.server);
        if acelasi_server && self.sursa.baza.eq_ignore_ascii_case(&self.baza.baza) {
            return Some("sursa si baza sunt aceeasi".into());
        }
        None
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
            Some("completeaza serverul pentru sursa")
        );
        c.sursa.server = "SRV-DEV".into();
        assert_eq!(c.ce_lipseste().as_deref(), Some("alege baza pentru sursa"));
        c.sursa.baza = "Dev".into();
        assert_eq!(c.ce_lipseste().as_deref(), Some("conecteaza-te la sursa"));
        c.sursa.stare = StareConexiune::Conectat("conectat".into());
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("completeaza serverul pentru baza")
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
    fn aceeasi_baza_de_doua_ori_nu_e_o_comparare() {
        let mut c = Conectare::noua();
        c.sursa = parte_gata("SRV", "Aceeasi");
        c.baza = parte_gata("srv", "aceeasi");
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("sursa si baza sunt aceeasi"),
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

    #[test]
    fn cu_instantaneu_nu_se_mai_cer_servere() {
        let mut c = Conectare::noua();
        c.instantaneu = Some("/tmp/inst.json".into());
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("alege baza sursa din instantaneu")
        );
        c.sursa.baza = "A".into();
        assert_eq!(
            c.ce_lipseste().as_deref(),
            Some("alege baza de comparat din instantaneu")
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
}
