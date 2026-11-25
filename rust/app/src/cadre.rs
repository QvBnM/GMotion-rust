//! Monitorul de cadre: cate cadre s-au desenat, percentila 99 a duratei lor
//! si cate au trecut de 16,7 ms, pe ferestre de o secunda. Ascuns implicit;
//! se aprinde cu `--fps` sau `GMOTION_FPS=1` (Mihai, 2026-11-25).
//!
//! **Se strica astfel:** cifra minte — monitorul insusi tine fereastra in
//! desenare (asa era pana pe 2026-11-25: `animation-tick()` dadea ~70
//! cadre/s in repaus), se numara si cadrul pe care il provoaca propriul text,
//! se ia intervalul dintre cadre drept durata desenarii, sau media ascunde
//! varfurile (media 14 ms cu un varf de 127 ms „arata bine"). De-aia calculul
//! e aici, pur, cu teste; `main.rs` doar ii da momentele, din winit.
//!
//! Durata unui cadru = lucrul firului ferestrei pentru el: evenimentele
//! (clic, tasta, mouse — cu tot Rust-ul chemat din ele) primite de la
//! cadrul de dinainte, plus desenarea (`RedrawRequested`).

use std::time::{Duration, Instant};

/// Pragul unui cadru la 60 de cadre/s.
pub const PRAG_CADRU: Duration = Duration::from_micros(16_667);

/// Cadrele dintr-o fereastra de masurare.
#[derive(Debug, Default, Clone)]
pub struct StatisticaCadre {
    inceput: Option<Instant>,
    /// Evenimentul in curs (inceput, nesfarsit).
    eveniment: Option<Instant>,
    /// Lucrul evenimentelor de la ultimul cadru incoace.
    lucru: Duration,
    /// Desenarea in curs.
    desenare: Option<Instant>,
    durate: Vec<Duration>,
    /// Urmatorul cadru fara niciun eveniment e al monitorului (i s-a
    /// schimbat textul): nu se numara, altfel s-ar intretine singur.
    ignora_propriul: bool,
}

/// Ce arata monitorul pentru o fereastra incheiata.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaportCadre {
    /// Cadre pe secunda, pe durata ferestrei.
    pub pe_secunda: f64,
    /// Percentila 99 a duratei cadrelor, in ms.
    pub p99_ms: f64,
    /// Cel mai lung cadru, in ms.
    pub cel_mai_lung_ms: f64,
    /// Media, in ms.
    pub mediu_ms: f64,
    /// Cate cadre au trecut de [`PRAG_CADRU`].
    pub peste_prag: u32,
    pub cadre: u32,
}

impl StatisticaCadre {
    /// Incepe un eveniment al ferestrei (mouse, tastatura, redimensionare).
    pub fn eveniment_inceput(&mut self, acum: Instant) {
        self.eveniment = Some(acum);
    }

    /// S-a terminat evenimentul inceput cu `eveniment_inceput`.
    pub fn eveniment_sfarsit(&mut self, acum: Instant) {
        if let Some(t0) = self.eveniment.take() {
            self.lucru += acum.saturating_duration_since(t0);
            self.ignora_propriul = false;
        }
    }

    /// Incepe desenarea unui cadru.
    pub fn inainte(&mut self, acum: Instant) {
        self.inceput.get_or_insert(acum);
        self.desenare = Some(acum);
    }

    /// S-a terminat desenarea inceputa cu `inainte`. Un `dupa` fara
    /// `inainte` nu se numara.
    pub fn dupa(&mut self, acum: Instant) {
        let Some(t0) = self.desenare.take() else {
            return;
        };
        let d = acum.saturating_duration_since(t0) + std::mem::take(&mut self.lucru);
        if std::mem::take(&mut self.ignora_propriul) {
            return;
        }
        self.durate.push(d);
    }

    /// Monitorul isi schimba textul: cadrul care urmeaza, daca nu vine din
    /// alt eveniment, e al lui.
    pub fn ignora_urmatorul(&mut self) {
        self.ignora_propriul = true;
    }

    /// Incheie fereastra de masurare la `acum` si incepe alta. `None` cand
    /// n-a fost niciun cadru (repaus): nimic de scris.
    pub fn incheie(&mut self, acum: Instant) -> Option<RaportCadre> {
        let inceput = self.inceput.take();
        let mut durate = std::mem::take(&mut self.durate);
        if durate.is_empty() {
            return None;
        }
        let durata = acum
            .saturating_duration_since(inceput.unwrap_or(acum))
            .as_secs_f64()
            .max(1e-3);
        durate.sort_unstable();
        let n = durate.len();
        // percentila 99 prin rang (nearest-rank): la sub 100 de cadre e
        // chiar cel mai lung, cum trebuie
        let rang = ((n as f64) * 0.99).ceil() as usize;
        let p99 = durate[rang.clamp(1, n) - 1];
        let total: Duration = durate.iter().sum();
        let ms = |d: Duration| d.as_secs_f64() * 1000.0;
        Some(RaportCadre {
            pe_secunda: n as f64 / durata,
            p99_ms: ms(p99),
            cel_mai_lung_ms: ms(durate[n - 1]),
            mediu_ms: ms(total) / n as f64,
            peste_prag: durate.iter().filter(|d| **d > PRAG_CADRU).count() as u32,
            cadre: n as u32,
        })
    }
}

impl RaportCadre {
    /// Textul monitorului: „12 cadre · p99 9 ms · 0 peste 16,7 ms · max 9 ms".
    pub fn text(&self, limba: gmotion_core::Limba) -> String {
        let prag = match limba {
            gmotion_core::Limba::Romana => "16,7",
            _ => "16.7",
        };
        format!(
            "{} {} · p99 {:.0} ms · {} {} {prag} ms · max {:.0} ms",
            self.cadre,
            limba.alege("cadre", "frames"),
            self.p99_ms,
            self.peste_prag,
            limba.alege("peste", "over"),
            self.cel_mai_lung_ms,
        )
    }

    /// A fost cel putin un cadru peste prag.
    pub fn lent(&self) -> bool {
        self.peste_prag > 0
    }
}

/// Timpii unei redesenari, pe bucati: `porneste`, `marcheaza` dupa fiecare
/// bucata, `scrie` la sfarsit (nivel debug, `gmotion::cadre`). Cu
/// `GMOTION_JURNAL=debug`, o redesenare lenta pe masina lui Mihai spune
/// singura unde a stat (lista, harta, editorul, coloana...).
pub struct Bucati {
    ultima: Instant,
    bucati: Vec<(&'static str, Duration)>,
}

impl Bucati {
    pub fn porneste() -> Self {
        Bucati {
            ultima: Instant::now(),
            bucati: Vec::new(),
        }
    }

    /// S-a terminat bucata `nume`.
    pub fn marcheaza(&mut self, nume: &'static str) {
        let acum = Instant::now();
        self.bucati
            .push((nume, acum.saturating_duration_since(self.ultima)));
        self.ultima = acum;
    }

    /// „lista=3.1 harta=0.2 editor=41.0", in ms.
    pub fn text(&self) -> String {
        self.bucati
            .iter()
            .map(|(n, d)| format!("{n}={:.1}", d.as_secs_f64() * 1000.0))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn scrie(&self) {
        tracing::debug!(target: "gmotion::cadre", bucati = %self.text(), "redesenare pe bucati");
    }
}

/// Monitorul e cerut: `--fps` sau `GMOTION_FPS` cu alta valoare decat `0`.
pub fn cerut(argumente: &[String], variabila: Option<&str>) -> bool {
    argumente.iter().any(|a| a == "--fps") || variabila.is_some_and(|v| !v.is_empty() && v != "0")
}

#[cfg(test)]
mod teste {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn p99_si_cadrele_peste_prag() {
        let t0 = Instant::now();
        let mut s = StatisticaCadre::default();
        // 200 de cadre de 4 ms, doua de 40 ms si unul de 127 ms
        for i in 0..203u64 {
            let start = t0 + ms(i * 4);
            s.inainte(start);
            let d = match i {
                10 | 20 => ms(40),
                30 => ms(127),
                _ => ms(4),
            };
            s.dupa(start + d);
        }
        let r = s.incheie(t0 + ms(1000)).expect("au fost cadre");
        assert_eq!(r.cadre, 203);
        assert_eq!(r.peste_prag, 3);
        // rangul 99% din 203 = 201: al treilea de la capat, 40 ms
        assert!((r.p99_ms - 40.0).abs() < 0.01, "{r:?}");
        assert!((r.cel_mai_lung_ms - 127.0).abs() < 0.01, "{r:?}");
        assert!(r.lent());
        // fereastra urmatoare porneste de la zero
        assert_eq!(s.incheie(t0 + ms(2000)), None, "repaus: nimic de spus");
    }

    /// Lucrul evenimentului (Rust-ul unui clic) intra in cadrul care il
    /// arata: o comanda de 110 ms + o desenare de 9 ms = un cadru de 119 ms.
    #[test]
    fn evenimentul_intra_in_cadrul_urmator() {
        let t0 = Instant::now();
        let mut s = StatisticaCadre::default();
        s.eveniment_inceput(t0);
        s.eveniment_sfarsit(t0 + ms(110));
        s.inainte(t0 + ms(111));
        s.dupa(t0 + ms(120));
        let r = s.incheie(t0 + ms(500)).expect("un cadru");
        assert!((r.cel_mai_lung_ms - 119.0).abs() < 0.01, "{r:?}");
        assert_eq!(r.peste_prag, 1);
    }

    /// Repausul dintre cadre nu e durata unui cadru, iar un `dupa` fara
    /// `inainte` nu inventeaza un cadru.
    #[test]
    fn repausul_nu_se_numara() {
        let t0 = Instant::now();
        let mut s = StatisticaCadre::default();
        s.inainte(t0);
        s.dupa(t0 + ms(5));
        s.inainte(t0 + ms(3000));
        s.dupa(t0 + ms(3006));
        let r = s.incheie(t0 + ms(3010)).expect("doua cadre");
        assert!(r.cel_mai_lung_ms < 7.0, "{r:?}");
        s.dupa(t0 + ms(4000));
        assert_eq!(s.incheie(t0 + ms(4100)), None);
    }

    /// Cadrul provocat de textul monitorului nu se numara: altfel monitorul
    /// ar desena o data pe fereastra la nesfarsit. Un eveniment intre timp
    /// anuleaza scutirea (cadrul e al lui).
    #[test]
    fn cadrul_propriu_nu_se_numara() {
        let t0 = Instant::now();
        let mut s = StatisticaCadre::default();
        s.ignora_urmatorul();
        s.inainte(t0);
        s.dupa(t0 + ms(3));
        assert_eq!(s.incheie(t0 + ms(1000)), None);
        s.ignora_urmatorul();
        s.eveniment_inceput(t0 + ms(1100));
        s.eveniment_sfarsit(t0 + ms(1101));
        s.inainte(t0 + ms(1102));
        s.dupa(t0 + ms(1105));
        assert_eq!(s.incheie(t0 + ms(2000)).map(|r| r.cadre), Some(1));
    }

    #[test]
    fn bucatile_redesenarii() {
        let mut b = Bucati::porneste();
        b.marcheaza("lista");
        b.marcheaza("editor");
        let t = b.text();
        assert!(t.starts_with("lista=") && t.contains(" editor="), "{t}");
        b.scrie();
    }

    #[test]
    fn textul_si_pornirea() {
        let r = RaportCadre {
            pe_secunda: 58.4,
            p99_ms: 21.2,
            cel_mai_lung_ms: 27.0,
            mediu_ms: 6.06,
            peste_prag: 2,
            cadre: 58,
        };
        assert_eq!(
            r.text(gmotion_core::Limba::Romana),
            "58 cadre · p99 21 ms · 2 peste 16,7 ms · max 27 ms"
        );
        assert_eq!(
            r.text(gmotion_core::Limba::Engleza),
            "58 frames · p99 21 ms · 2 over 16.7 ms · max 27 ms"
        );
        assert!(cerut(&["--fps".into()], None));
        assert!(cerut(&[], Some("1")));
        assert!(!cerut(&[], Some("0")));
        assert!(!cerut(&[], None));
    }
}
