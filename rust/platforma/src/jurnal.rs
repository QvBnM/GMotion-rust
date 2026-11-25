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
/// tampon care creste nemarginit intr-o aplicatie care ramane deschisa toata
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

    /// Uita tot ce s-a adunat, pastrand indexul absolut: cine citeste
    /// incremental nu primeste iar liniile vechi.
    fn goleste(&mut self) {
        self.aruncate += self.linii.len();
        self.linii.clear();
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

/// Goleste tamponul („Sterge log" din ecranul de conectare). Indexul
/// absolut **nu** se da inapoi: cine tine un index de unde a citit nu
/// primeste de doua ori aceleasi linii.
pub fn goleste() {
    tampon().lock().unwrap_or_else(|e| e.into_inner()).goleste();
}

/// Adauga o linie in tampon. Publica fiindca o folosesc si straturile de
/// deasupra pentru mesaje destinate utilizatorului, nu doar `tracing`.
pub fn adauga_linie(linie: impl Into<String>) {
    let linie = curata_linie(&linie.into());
    tampon()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .adauga(linie, MAX_LINII);
}

/// Scriitor care scapa caracterele de control (ESC, CR...) inainte de
/// terminal sau fisier: un nume de server dintr-un instantaneu, cu ESC in el,
/// rescria terminalul (auditul 2026-11-23). Randul nou si tab-ul raman: sunt
/// ale formatului.
struct Curatat<W>(W);

impl<W: std::io::Write> std::io::Write for Curatat<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // o secventa de culoare (ESC [ cifre ; m), pusa de formatul jurnalului
        // pe terminal, trece; orice alt ESC (stergere de ecran, mutare de
        // cursor) se scapa
        let e_culoare = |i: usize| {
            buf.get(i + 1) == Some(&b'[')
                && buf[i + 2..]
                    .iter()
                    .position(|&c| !(c.is_ascii_digit() || c == b';'))
                    .is_some_and(|k| buf[i + 2 + k] == b'm')
        };
        let mut out = Vec::with_capacity(buf.len());
        for (i, &b) in buf.iter().enumerate() {
            if b == 0x1b && e_culoare(i) {
                out.push(b);
            } else if (b < 0x20 && b != b'\n' && b != b'\t') || b == 0x7f {
                out.extend_from_slice(format!("\\x{b:02x}").as_bytes());
            } else {
                out.push(b);
            }
        }
        self.0.write_all(&out)?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

/// Strat de `tracing` care scrie si in tamponul din memorie.
struct StratTampon;

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for StratTampon {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        /// Mesajul si **toate** campurile. Pana la audit se pastra doar
        /// mesajul, iar fiecare apel pune datele in campuri
        /// (`%server, %baza, "deschid conexiunea"`), deci tamponul arata
        /// „deschid conexiunea" fara server sau baza — inutil la depanare.
        #[derive(Default)]
        struct Culege {
            mesaj: String,
            campuri: Vec<String>,
        }
        impl tracing::field::Visit for Culege {
            fn record_str(&mut self, camp: &tracing::field::Field, val: &str) {
                if camp.name() == "message" {
                    self.mesaj = val.to_string();
                } else {
                    self.campuri.push(format!("{}={}", camp.name(), val));
                }
            }
            fn record_debug(&mut self, camp: &tracing::field::Field, val: &dyn std::fmt::Debug) {
                // `format_args!` (mesajul obisnuit) are `Debug` fara ghilimele;
                // inainte se taiau ghilimelele de la capete, si cu ele si pe
                // cele care faceau parte din mesaj.
                if camp.name() == "message" {
                    self.mesaj = format!("{val:?}");
                } else {
                    self.campuri.push(format!("{}={:?}", camp.name(), val));
                }
            }
        }
        // doar ce scrie aplicatia: liniile bibliotecilor (fereastra, TLS,
        // X11) raman in fisier, nu in jurnalul din fereastra (2026-11-25)
        if !event.metadata().target().starts_with("gmotion") {
            return;
        }
        let mut c = Culege::default();
        event.record(&mut c);
        if c.mesaj.is_empty() && c.campuri.is_empty() {
            return;
        }
        let mut linie = format!("[{}] {}", event.metadata().level(), c.mesaj);
        for camp in &c.campuri {
            linie.push(' ');
            linie.push_str(camp);
        }
        adauga_linie(linie);
    }
}

/// Lungimea maxima a unei linii din tampon, in caractere. Limita pe numarul de
/// linii nu ajuta daca o linie are 50 MB.
const MAX_CARACTERE_LINIE: usize = 4096;

/// O linie de jurnal sigura de afisat: caracterele de control devin `\n`,
/// `\r`, `\t` sau `\u{..}` (numele vin si din baze si de pe retea; un rand nou
/// fabrica o linie de jurnal falsa, un ESC rescrie terminalul), iar liniile
/// prea lungi se taie.
fn curata_linie(s: &str) -> String {
    let mut out = String::with_capacity(s.len().min(MAX_CARACTERE_LINIE));
    for (i, c) in s.chars().enumerate() {
        if i >= MAX_CARACTERE_LINIE {
            out.push_str(" …[taiat]");
            break;
        }
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // si caracterele de directie (U+202E inverseaza textul afisat) si
            // separatorii de rand Unicode: nu sunt `is_control`, dar fac o
            // linie sa arate altceva decat contine (F-B7)
            c if c.is_control()
                || matches!(c, '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{2028}' | '\u{2029}') =>
            {
                out.push_str(&format!("\\u{{{:x}}}", c as u32))
            }
            c => out.push(c),
        }
    }
    out
}

/// Porneste jurnalul. Sigur de chemat de mai multe ori: a doua chemare nu face
/// nimic, in loc sa dea panica — o aplicatie cu interfata si un CLI in acelasi
/// binar ar putea incerca de doua ori.
///
/// Filtrul vine din `GMOTION_JURNAL` sau `RUST_LOG`; implicit `info`.
/// Filtrul cand nu e dat altul in `GMOTION_JURNAL` / `RUST_LOG`. `tiberius`
/// scrie ~8 linii INFO la fiecare conexiune (TLS, schimbari de context); la
/// prima rulare pe o baza reala acopereau progresul. Avertismentul lui despre
/// certificat („Trusting the server certificate without validation") vine la
/// fiecare conexiune si spune ce spune deja `gmotion::db`, o data pe sesiune:
/// de la `tiberius` raman doar erorile (2026-11-25).
const FILTRU_IMPLICIT: &str = "info,tiberius=error";

pub fn porneste() {
    porneste_intern(None);
}

/// Ca `porneste`, dar scrie jurnalul si intr-un fisier nou din `director`
/// (`gmotion-<moment>.log`), la nivel `debug` pentru tintele `gmotion`, ca o
/// sesiune de proba (fiecare buton apasat, fiecare comanda cu durata ei) sa
/// se poata trimite si citi dupa. Se pastreaza doar ultimele `PASTRATE`
/// fisiere. Intoarce calea fisierului, sau `None` daca nu s-a putut crea —
/// jurnalul pe ecran merge oricum.
pub fn porneste_cu_fisier(director: &std::path::Path) -> Option<std::path::PathBuf> {
    let cale = fisier_nou(director);
    let fisier = cale.as_ref().and_then(|c| std::fs::File::create(c).ok());
    let scris = fisier.is_some();
    porneste_intern(fisier);
    if scris {
        cale
    } else {
        None
    }
}

/// Cate fisiere de jurnal se pastreaza; cele mai vechi se sterg.
pub const PASTRATE: usize = 10;

/// Calea unui fisier nou de jurnal, dupa ce le sterge pe cele vechi peste
/// `PASTRATE - 1`. Numele are momentul, deci sortarea alfabetica e cronologica.
fn fisier_nou(director: &std::path::Path) -> Option<std::path::PathBuf> {
    std::fs::create_dir_all(director).ok()?;
    let mut vechi: Vec<std::path::PathBuf> = std::fs::read_dir(director)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("gmotion-") && n.ends_with(".log"))
        })
        .collect();
    vechi.sort();
    while vechi.len() >= PASTRATE {
        let _ = std::fs::remove_file(vechi.remove(0));
    }
    Some(director.join(format!("gmotion-{}.log", crate::timp::acum_compact())))
}

fn porneste_intern(fisier: Option<std::fs::File>) {
    use tracing_subscriber::Layer;
    static PORNIT: OnceLock<()> = OnceLock::new();
    if PORNIT.set(()).is_err() {
        return;
    }
    let filtru = std::env::var("GMOTION_JURNAL")
        .or_else(|_| std::env::var("RUST_LOG"))
        .unwrap_or_else(|_| FILTRU_IMPLICIT.to_string());
    let env = || {
        tracing_subscriber::EnvFilter::try_new(&filtru)
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(FILTRU_IMPLICIT))
    };
    // fisierul are filtrul lui: tot ce e `gmotion` pana la debug, restul info
    let strat_fisier = fisier.map(|f| {
        tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_ansi(false)
            .with_writer(Mutex::new(Curatat(f)))
            .with_filter(tracing_subscriber::EnvFilter::new(
                "info,tiberius=error,gmotion=debug",
            ))
    });
    let _ = tracing_subscriber::registry()
        // La iesirea de ERORI, nu la cea standard. Linia de comanda scrie
        // JSON la iesirea standard, iar o linie de jurnal amestecata acolo il
        // face neparsabil. S-a intamplat: prima versiune folosea implicitul
        // `fmt::layer()`, care scrie la stdout, si `gmotion ... --json` nu se
        // putea citi.
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                // Culori doar pe terminal: redirectat intr-un fisier sau
                // capturat de alt program, codurile ANSI devin gunoi.
                .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
                .with_writer(|| Curatat(std::io::stderr()))
                .with_filter(env()),
        )
        // banda de jurnal din fereastra vede ce vede si terminalul, nu debug
        .with(StratTampon.with_filter(env()))
        .with(strat_fisier)
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
    fn scriitorul_scapa_caracterele_de_control() {
        use std::io::Write;
        let mut v = Vec::new();
        Curatat(&mut v)
            .write_all(b"server S\x1b[2J;rm\tok\n")
            .unwrap();
        assert_eq!(String::from_utf8(v).unwrap(), "server S\\x1b[2J;rm\tok\n");
        let mut v = Vec::new();
        Curatat(&mut v).write_all(b"\x1b[2mINFO\x1b[0m").unwrap();
        assert_eq!(v, b"\x1b[2mINFO\x1b[0m", "culorile trec");
    }

    #[test]
    fn caracterele_de_directie_se_scapa() {
        let l = curata_linie("a\u{202e}b\u{2028}c");
        assert_eq!(l, "a\\u{202e}b\\u{2028}c");
    }

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
    fn golirea_uita_liniile_dar_nu_da_indexul_inapoi() {
        let mut t = Tampon::default();
        t.adauga("prima".into(), 10);
        t.adauga("a doua".into(), 10);
        let (_, inainte) = t.de_la(0);
        t.goleste();
        let (linii, total) = t.de_la(0);
        assert!(linii.is_empty(), "tamponul e gol");
        assert_eq!(total, inainte, "indexul merge mai departe de unde era");
        t.adauga("a treia".into(), 10);
        assert_eq!(t.de_la(inainte).0, vec!["a treia"], "si liniile noi se vad");
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

    /// Regresii din audit: control in linii, linii nelimitate.
    #[test]
    fn liniile_se_curata_si_se_limiteaza() {
        assert_eq!(
            curata_linie("Baza\n[ERROR] fals\u{1b}[31m"),
            "Baza\\n[ERROR] fals\\u{1b}[31m"
        );
        let lunga = "x".repeat(MAX_CARACTERE_LINIE * 3);
        assert!(curata_linie(&lunga).chars().count() < MAX_CARACTERE_LINIE + 20);
        assert_eq!(
            curata_linie("\"citat\" la capete \""),
            "\"citat\" la capete \""
        );
    }

    /// Jurnalul din fereastra arata doar ce scrie aplicatia: winit, TLS si
    /// celelalte biblioteci raman in fisier (2026-11-25).
    #[test]
    fn tamponul_primeste_doar_tintele_aplicatiei() {
        use tracing_subscriber::layer::SubscriberExt;
        let sub = tracing_subscriber::registry().with(StratTampon);
        tracing::subscriber::with_default(sub, || {
            tracing::warn!(target: "winit::x11", "biblioteca-7f3 vorbeste");
            tracing::info!(target: "gmotion::app", "aplicatia-7f3 vorbeste");
        });
        let (toate, _) = linii(0);
        assert!(
            toate.iter().any(|l| l.contains("aplicatia-7f3")),
            "{toate:?}"
        );
        assert!(
            !toate.iter().any(|l| l.contains("biblioteca-7f3")),
            "{toate:?}"
        );
    }

    /// Regresie din audit: campurile se pierdeau, iar ghilimelele reale de la
    /// capetele mesajului erau taiate.
    #[test]
    fn stratul_pastreaza_campurile_si_ghilimelele() {
        use tracing_subscriber::layer::SubscriberExt;
        let sub = tracing_subscriber::registry().with(StratTampon);
        tracing::subscriber::with_default(sub, || {
            tracing::info!(referinta = %"RefUnica-9c1", cale = "/x", "sesiune salvata");
            tracing::info!("\"citat\" la capete");
        });
        let (toate, _) = linii(0);
        assert!(
            toate.iter().any(|l| l.contains("sesiune salvata")
                && l.contains("referinta=RefUnica-9c1")
                && l.contains("cale=/x")),
            "{toate:?}"
        );
        assert!(
            toate.iter().any(|l| l.ends_with("\"citat\" la capete")),
            "{toate:?}"
        );
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

    #[test]
    fn fisierele_vechi_de_jurnal_se_sterg_peste_limita() {
        let d = std::env::temp_dir().join(format!("gmotion-jurnale-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        for i in 0..PASTRATE + 3 {
            std::fs::write(d.join(format!("gmotion-2026111{i:02}.log")), "x").unwrap();
        }
        std::fs::write(d.join("altceva.txt"), "x").unwrap();
        let nou = fisier_nou(&d).unwrap();
        assert!(nou.starts_with(&d));
        let ramase = std::fs::read_dir(&d)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".log"))
            .count();
        assert_eq!(ramase, PASTRATE - 1, "loc pentru cel nou");
        assert!(d.join("altceva.txt").exists(), "nu atinge alte fisiere");
        assert!(
            !d.join("gmotion-202611100.log").exists(),
            "cele mai vechi pleaca"
        );
        let _ = std::fs::remove_dir_all(&d);
    }
}
