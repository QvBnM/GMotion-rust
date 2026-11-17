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

#[derive(Default)]
struct Tampon {
    linii: std::collections::VecDeque<String>,
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
    let t = tampon().lock().expect("tamponul de jurnal nu e otravit");
    let total = t.linii.len();
    if de_la >= total {
        return (Vec::new(), total);
    }
    (t.linii.iter().skip(de_la).cloned().collect(), total)
}

/// Adauga o linie in tampon. Publica fiindca o folosesc si straturile de
/// deasupra pentru mesaje destinate utilizatorului, nu doar `tracing`.
pub fn adauga_linie(linie: impl Into<String>) {
    let mut t = tampon().lock().expect("tamponul de jurnal nu e otravit");
    if t.linii.len() >= MAX_LINII {
        t.linii.pop_front();
    }
    t.linii.push_back(linie.into());
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
pub fn porneste() {
    static PORNIT: OnceLock<()> = OnceLock::new();
    if PORNIT.set(()).is_err() {
        return;
    }
    let filtru = std::env::var("GMOTION_JURNAL")
        .or_else(|_| std::env::var("RUST_LOG"))
        .unwrap_or_else(|_| "info".to_string());
    let env = tracing_subscriber::EnvFilter::try_new(&filtru)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
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
                .with_writer(std::io::stderr),
        )
        .with(StratTampon)
        .try_init();
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn linii_cere_doar_ce_e_nou() {
        adauga_linie("prima");
        let (_, pana_aici) = linii(0);
        adauga_linie("a doua");
        let (noi, urmator) = linii(pana_aici);
        assert_eq!(noi, vec!["a doua"]);
        assert_eq!(urmator, pana_aici + 1);
        // index peste total nu da panica si nu intoarce nimic
        assert_eq!(linii(urmator + 100).0.len(), 0);
    }

    #[test]
    fn tamponul_nu_creste_nemarginit() {
        for i in 0..(MAX_LINII + 50) {
            adauga_linie(format!("linia {i}"));
        }
        let (toate, _) = linii(0);
        assert!(
            toate.len() <= MAX_LINII,
            "tamponul a crescut la {}",
            toate.len()
        );
    }

    #[test]
    fn pornirea_repetata_nu_da_panica() {
        porneste();
        porneste();
        porneste();
    }
}
