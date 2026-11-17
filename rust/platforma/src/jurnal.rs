//! Jurnalul: o tinta per crate, filtrabila din mediu.
//!
//! **Se strica astfel:** cand ceva merge prost nu ai ce citi, sau ai prea mult
//! si nu gasesti nimic. De-aia fiecare crate are tinta lui
//! (`gmotion::core`, `gmotion::db`, `gmotion::platforma`, `gmotion::modul`):
//! la depanare se porneste doar partea suspecta.
//!
//! Tine si un tampon in memorie cu ultimele linii, pentru metoda comuna
//! `jurnal` — interfata si CLI-ul arata ce s-a intamplat fara sa citeasca
//! fisiere.

use std::sync::{Arc, Mutex, OnceLock};

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Cate linii se tin in memorie. Peste atat, cele mai vechi se arunca: un
/// tampon care creste nemarginit intr-o aplicatie care rămâne deschisa toata
/// ziua e o scurgere de memorie, nu o facilitate.
const MAX_LINII: usize = 2000;

/// Indexurile date apelantului sunt **absolute** (a cata linie de la pornire),
/// nu pozitii in coada: dupa ce tamponul se umple, coada are mereu `MAX_LINII`
/// elemente, si un index pozitional ar spune „nimic nou" la nesfarsit.
#[derive(Default)]
struct Tampon {
    linii: std::collections::VecDeque<String>,
    /// Cate linii au fost aruncate de la inceput = indexul absolut al primei
    /// linii din coada.
    aruncate: usize,
}

impl Tampon {
    fn adauga(&mut self, linie: String, max: usize) {
        if self.linii.len() >= max {
            self.linii.pop_front();
            self.aruncate += 1;
        }
        self.linii.push_back(linie);
    }

    /// Liniile de la indexul absolut `de_la`. Daca unele au fost deja aruncate,
    /// se incepe cu cea mai veche pastrata.
    fn de_la(&self, de_la: usize) -> (Vec<String>, usize) {
        let total = self.aruncate + self.linii.len();
        let sari = de_la.saturating_sub(self.aruncate);
        if sari >= self.linii.len() {
            return (Vec::new(), total);
        }
        (self.linii.iter().skip(sari).cloned().collect(), total)
    }
}

static TAMPON: OnceLock<Arc<Mutex<Tampon>>> = OnceLock::new();

fn tampon() -> &'static Arc<Mutex<Tampon>> {
    TAMPON.get_or_init(|| Arc::new(Mutex::new(Tampon::default())))
}

/// Liniile de jurnal de la indexul dat incolo, plus indexul urmator.
///
/// Apelantul tine indexul si cere doar ce e nou — exact ce facea `log_lines`
/// din versiunea Python, dar fara sa retrimita tot de fiecare data.
pub fn linii(de_la: usize) -> (Vec<String>, usize) {
    tampon()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .de_la(de_la)
}

/// Adauga o linie in tampon. Publica fiindca o folosesc si straturile de
/// deasupra pentru mesaje destinate utilizatorului, nu doar `tracing`.
pub fn adauga_linie(linie: impl Into<String>) {
    tampon()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .adauga(linie.into(), MAX_LINII);
}

/// Strat de `tracing` care scrie si in tamponul din memorie.
struct StratTampon;

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for StratTampon {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        struct Culege(String);
        impl tracing::field::Visit for Culege {
            fn record_debug(&mut self, camp: &tracing::field::Field, val: &dyn std::fmt::Debug) {
                if camp.name() == "message" {
                    self.0 = format!("{val:?}").trim_matches('"').to_string();
                }
            }
        }
        let mut c = Culege(String::new());
        event.record(&mut c);
        if !c.0.is_empty() {
            adauga_linie(format!("[{}] {}", event.metadata().level(), c.0));
        }
    }
}

/// Porneste jurnalul. Sigur de chemat de mai multe ori: a doua chemare nu face
/// nimic, in loc sa dea panica — o aplicatie cu interfata si un CLI in acelasi
/// binar ar putea incerca de doua ori.
///
/// Filtrul vine din `GMOTION_JURNAL` sau `RUST_LOG`; implicit `info`.
/// Filtrul cand nu e dat altul in `GMOTION_JURNAL` / `RUST_LOG`. `tiberius`
/// scrie ~8 linii INFO la fiecare conexiune (TLS, schimbari de context); la
/// prima rulare pe o baza reala acopereau progresul. Avertismentele raman.
const FILTRU_IMPLICIT: &str = "info,tiberius=warn";

pub fn porneste() {
    static PORNIT: OnceLock<()> = OnceLock::new();
    if PORNIT.set(()).is_err() {
        return;
    }
    let filtru = std::env::var("GMOTION_JURNAL")
        .or_else(|_| std::env::var("RUST_LOG"))
        .unwrap_or_else(|_| FILTRU_IMPLICIT.to_string());
    let env = tracing_subscriber::EnvFilter::try_new(&filtru)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(FILTRU_IMPLICIT));
    let _ = tracing_subscriber::registry()
        .with(env)
        // La ieșirea de ERORI, nu la cea standard. Linia de comanda scrie
        // JSON la ieșirea standard, iar o linie de jurnal amestecata acolo il
        // face neparsabil. S-a intamplat: prima versiune folosea implicitul
        // `fmt::layer()`, care scrie la stdout, si `gmotion ... --json` nu se
        // putea citi.
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                // Culori doar pe terminal: redirectat intr-un fisier sau
                // capturat de alt program, codurile ANSI devin gunoi.
                .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
                .with_writer(std::io::stderr),
        )
        .with(StratTampon)
        .try_init();
}

#[cfg(test)]
mod teste {
    use super::*;

    // Logica se testeaza pe un `Tampon` local. Tamponul global e partajat cu
    // toate testele din proces (si cu jurnalul lor, dupa `porneste`), deci un
    // test pe el care presupune ca nu scrie nimeni altcineva cade aleator:
    // asa a cazut `linii_cere_doar_ce_e_nou`, ~1 rulare din 100.

    #[test]
    fn linii_cere_doar_ce_e_nou() {
        let mut t = Tampon::default();
        t.adauga("prima".into(), 10);
        let (_, pana_aici) = t.de_la(0);
        t.adauga("a doua".into(), 10);
        let (noi, urmator) = t.de_la(pana_aici);
        assert_eq!(noi, vec!["a doua"]);
        assert_eq!(urmator, pana_aici + 1);
        // index peste total nu da panica si nu intoarce nimic
        assert_eq!(t.de_la(urmator + 100).0.len(), 0);
    }

    #[test]
    fn tamponul_nu_creste_nemarginit() {
        let mut t = Tampon::default();
        for i in 0..(MAX_LINII + 50) {
            t.adauga(format!("linia {i}"), MAX_LINII);
        }
        let (toate, total) = t.de_la(0);
        assert_eq!(toate.len(), MAX_LINII);
        assert_eq!(toate[0], "linia 50", "se arunca cele mai vechi");
        assert_eq!(total, MAX_LINII + 50, "indexul numara si ce s-a aruncat");
    }

    #[test]
    fn dupa_ce_tamponul_e_plin_liniile_noi_se_vad_in_continuare() {
        let mut t = Tampon::default();
        for i in 0..5 {
            t.adauga(format!("linia {i}"), 5);
        }
        let (_, pana_aici) = t.de_la(0);
        t.adauga("noua".into(), 5);
        let (noi, urmator) = t.de_la(pana_aici);
        assert_eq!(noi, vec!["noua"], "jurnalul a inghetat la tamponul plin");
        assert_eq!(urmator, pana_aici + 1);
    }

    #[test]
    fn un_apelant_ramas_in_urma_primeste_ce_s_a_pastrat() {
        let mut t = Tampon::default();
        for i in 0..8 {
            t.adauga(format!("linia {i}"), 3);
        }
        let (noi, urmator) = t.de_la(1);
        assert_eq!(noi, vec!["linia 5", "linia 6", "linia 7"]);
        assert_eq!(urmator, 8);
    }

    #[test]
    fn functiile_globale_vad_ce_s_a_adaugat() {
        // Robust la scrieri concurente: verifica doar ca linia proprie apare
        // dupa index si ca indexul avanseaza.
        let (_, inainte) = linii(0);
        adauga_linie("marcaj-unic-de-test-7f3a");
        let (noi, dupa) = linii(inainte);
        assert!(
            noi.iter().any(|l| l == "marcaj-unic-de-test-7f3a"),
            "{noi:?}"
        );
        assert!(dupa > inainte);
    }

    #[test]
    fn filtrul_implicit_e_valid() {
        assert!(tracing_subscriber::EnvFilter::try_new(FILTRU_IMPLICIT).is_ok());
    }

    #[test]
    fn pornirea_repetata_nu_da_panica() {
        porneste();
        porneste();
        porneste();
    }
}
