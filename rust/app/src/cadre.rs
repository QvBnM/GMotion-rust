//! Monitorul de cadre: cate cadre pe secunda desenam si cat a tinut cel mai
//! lung, pe ferestre de jumatate de secunda. Ascuns implicit; se aprinde cu
//! `--fps` sau `GMOTION_FPS=1` (Mihai, 2026-11-25: „scroll-ul nu se simte
//! fluid; sa punem un fps monitor pana vedem imbunatatiri").
//!
//! **Se strica astfel:** cifra minte — se numara si cadrele care nu s-au
//! desenat, se ia intervalul dintre cadre drept durata desenarii (la o
//! fereastra in repaus intervalul e de secunde), sau o fereastra goala da
//! impartire la zero. De-aia calculul e aici, pur, cu teste; `main.rs` doar
//! ii da momentele din `set_rendering_notifier`.

use std::time::{Duration, Instant};

/// Cadrele dintr-o fereastra de masurare.
#[derive(Debug, Default, Clone)]
pub struct StatisticaCadre {
    inceput: Option<Instant>,
    desenare_inceputa: Option<Instant>,
    cadre: u32,
    cel_mai_lung: Duration,
    total: Duration,
}

/// Ce arata monitorul pentru o fereastra incheiata.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaportCadre {
    /// Cadre pe secunda, pe durata ferestrei.
    pub pe_secunda: f64,
    /// Cel mai lung cadru (de la inceputul la sfarsitul desenarii), in ms.
    pub cel_mai_lung_ms: f64,
    /// Media desenarii unui cadru, in ms.
    pub mediu_ms: f64,
    pub cadre: u32,
}

impl StatisticaCadre {
    /// Incepe desenarea unui cadru.
    pub fn inainte(&mut self, acum: Instant) {
        self.inceput.get_or_insert(acum);
        self.desenare_inceputa = Some(acum);
    }

    /// S-a terminat desenarea cadrului inceput cu `inainte`. Un `dupa` fara
    /// `inainte` nu se numara.
    pub fn dupa(&mut self, acum: Instant) {
        let Some(t0) = self.desenare_inceputa.take() else {
            return;
        };
        let d = acum.saturating_duration_since(t0);
        self.cadre += 1;
        self.total += d;
        self.cel_mai_lung = self.cel_mai_lung.max(d);
    }

    /// Incheie fereastra de masurare la `acum` si incepe alta. `None` cand
    /// n-a fost niciun cadru (fereastra in repaus).
    pub fn incheie(&mut self, acum: Instant) -> Option<RaportCadre> {
        let inceput = self.inceput?;
        let durata = acum.saturating_duration_since(inceput).as_secs_f64();
        let r = (self.cadre > 0).then(|| RaportCadre {
            pe_secunda: if durata > 0.0 {
                f64::from(self.cadre) / durata
            } else {
                0.0
            },
            cel_mai_lung_ms: self.cel_mai_lung.as_secs_f64() * 1000.0,
            mediu_ms: self.total.as_secs_f64() * 1000.0 / f64::from(self.cadre),
            cadre: self.cadre,
        });
        let in_curs = self.desenare_inceputa;
        *self = StatisticaCadre::default();
        // un cadru inceput si neterminat trece in fereastra urmatoare
        if let Some(t) = in_curs {
            self.inceput = Some(acum);
            self.desenare_inceputa = Some(t);
        }
        r
    }
}

impl RaportCadre {
    /// Textul monitorului: „58 cadre/s · cel mai lung 21 ms · mediu 6,1 ms".
    pub fn text(&self, limba: gmotion_core::Limba) -> String {
        let mediu = format!("{:.1}", self.mediu_ms);
        let mediu = match limba {
            gmotion_core::Limba::Romana => mediu.replace('.', ","),
            _ => mediu,
        };
        format!(
            "{:.0} {} · {} {:.0} ms · {} {mediu} ms",
            self.pe_secunda,
            limba.alege("cadre/s", "fps"),
            limba.alege("cel mai lung", "longest"),
            self.cel_mai_lung_ms,
            limba.alege("mediu", "mean"),
        )
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
    fn numara_cadrele_si_cel_mai_lung_pe_fereastra() {
        let t0 = Instant::now();
        let mut s = StatisticaCadre::default();
        // 30 de cadre in 500 ms, fiecare de 4 ms, unul de 40 ms
        for i in 0..30u64 {
            let start = t0 + ms(i * 16);
            s.inainte(start);
            s.dupa(start + if i == 7 { ms(40) } else { ms(4) });
        }
        let r = s.incheie(t0 + ms(500)).expect("au fost cadre");
        assert_eq!(r.cadre, 30);
        assert!((r.pe_secunda - 60.0).abs() < 0.01, "{r:?}");
        assert!((r.cel_mai_lung_ms - 40.0).abs() < 0.01, "{r:?}");
        assert!((r.mediu_ms - (29.0 * 4.0 + 40.0) / 30.0).abs() < 0.01);
        // fereastra urmatoare porneste de la zero
        assert_eq!(s.incheie(t0 + ms(1000)), None, "repaus: nimic de spus");
    }

    /// Intervalul dintre cadre nu e durata desenarii: o fereastra in repaus
    /// 3 s intre doua cadre scurte nu da „cel mai lung 3000 ms".
    #[test]
    fn repausul_dintre_cadre_nu_se_numara_ca_desenare() {
        let t0 = Instant::now();
        let mut s = StatisticaCadre::default();
        s.inainte(t0);
        s.dupa(t0 + ms(5));
        s.inainte(t0 + ms(3000));
        s.dupa(t0 + ms(3006));
        let r = s.incheie(t0 + ms(3010)).expect("doua cadre");
        assert!(r.cel_mai_lung_ms < 7.0, "{r:?}");
        // si un `dupa` fara `inainte` nu inventeaza un cadru
        s.dupa(t0 + ms(4000));
        assert_eq!(s.incheie(t0 + ms(4100)), None);
    }

    #[test]
    fn textul_si_pornirea() {
        let r = RaportCadre {
            pe_secunda: 58.4,
            cel_mai_lung_ms: 21.2,
            mediu_ms: 6.06,
            cadre: 29,
        };
        assert_eq!(
            r.text(gmotion_core::Limba::Romana),
            "58 cadre/s · cel mai lung 21 ms · mediu 6,1 ms"
        );
        assert_eq!(
            r.text(gmotion_core::Limba::Engleza),
            "58 fps · longest 21 ms · mean 6.1 ms"
        );
        assert!(cerut(&["--fps".into()], None));
        assert!(cerut(&[], Some("1")));
        assert!(!cerut(&[], Some("0")));
        assert!(!cerut(&[], None));
    }
}
