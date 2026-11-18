//! Compararea pornita din ecranul de conectare, pe alt fir: fereastra ramane
//! vie, arata progresul si poate anula.
//!
//! **Se strica astfel:** rezultatul unei comparari anulate ajunge totusi in
//! lista (sau peste una pornita dupa ea), ori fereastra asteapta un fir care
//! nu mai trimite nimic. De-aia firul vorbeste doar printr-un canal, iar
//! anularea taie canalul: ce mai trimite firul dupa aceea nu are cine sa
//! primeasca.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;

use gmotion_fereastra::model_conectare::Conectare;
use gmotion_modul_diff_obiecte as modul;

use crate::pornire::{self, Comparata};

/// Ce trimite firul catre fereastra.
pub enum Mesaj {
    Progres(modul::Progres),
    Gata(Result<Comparata, String>),
}

/// O comparare care ruleaza pe alt fir.
pub struct CompararePornita {
    primire: Receiver<Mesaj>,
    anulat: Arc<AtomicBool>,
}

impl CompararePornita {
    /// Porneste compararea pe un fir nou, cu o copie a conectarii: ce se
    /// schimba in formular dupa pornire nu mai ajunge la ea.
    pub fn porneste(c: Conectare) -> Self {
        let (trimite, primire) = mpsc::channel();
        let anulat = Arc::new(AtomicBool::new(false));
        let a = anulat.clone();
        std::thread::spawn(move || {
            let t = trimite.clone();
            let r = pornire::compara_cu(
                &c,
                &mut |p| {
                    // fereastra a renuntat: nu mai are cine sa primeasca
                    let _ = t.send(Mesaj::Progres(p));
                },
                &a,
            );
            let _ = trimite.send(Mesaj::Gata(r));
        });
        CompararePornita { primire, anulat }
    }

    /// Ce a trimis firul de la ultima citire, fara sa astepte. Dupa `Gata`
    /// nu mai vine nimic.
    pub fn citeste(&self) -> Vec<Mesaj> {
        let mut v = Vec::new();
        loop {
            match self.primire.try_recv() {
                Ok(m) => v.push(m),
                Err(TryRecvError::Empty) => break,
                // firul s-a terminat fara `Gata` (n-ar trebui): se spune
                Err(TryRecvError::Disconnected) => {
                    if !v.iter().any(|m| matches!(m, Mesaj::Gata(_))) {
                        v.push(Mesaj::Gata(Err(
                            "compararea s-a oprit fara rezultat".to_string()
                        )));
                    }
                    break;
                }
            }
        }
        v
    }

    /// Anuleaza: firul se opreste la urmatorul pas, iar ce mai trimite se
    /// pierde odata cu canalul.
    pub fn anuleaza(self) {
        self.anulat.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    fn instantaneu(eticheta: &str) -> (std::path::PathBuf, Conectare) {
        let d = std::env::temp_dir().join(format!("gmotion-fir-{eticheta}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let cale = d.join("inst.json");
        std::fs::write(
            &cale,
            r#"{"versiune":1,"server":"SRV","creat":"2026-11-17T10:00:00Z","nota":"","baze":{
                "R":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 1"}],
                "A":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 2"}]}}"#,
        )
        .unwrap();
        let mut c = Conectare::noua();
        c.instantaneu = Some(cale.to_string_lossy().to_string());
        c.sursa.baza = "R".into();
        c.comuta_baza("A");
        (d, c)
    }

    /// Asteapta `Gata`, cu progresul de pe drum.
    fn asteapta(p: &CompararePornita) -> (Vec<modul::Progres>, Result<Comparata, String>) {
        let mut progres = Vec::new();
        for _ in 0..500 {
            for m in p.citeste() {
                match m {
                    Mesaj::Progres(x) => progres.push(x),
                    Mesaj::Gata(r) => return (progres, r),
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("compararea nu s-a terminat in 5 s");
    }

    #[test]
    fn compararea_pe_alt_fir_trimite_progresul_apoi_rezultatul() {
        let (d, c) = instantaneu("gata");
        let p = CompararePornita::porneste(c);
        let (progres, r) = asteapta(&p);
        let r = r.expect("compararea merge");
        assert_eq!(r.randuri.len(), 1);
        assert!(progres.len() >= 3, "pasii de pe drum: {}", progres.len());
        let procente: Vec<u8> = progres.iter().map(|x| x.procent).collect();
        assert!(
            procente.windows(2).all(|w| w[0] <= w[1]),
            "procentul nu da inapoi: {procente:?}"
        );
        assert!(progres.iter().any(|x| x.mesaj.contains("Citesc A")));
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn anulata_inainte_de_primul_pas_nu_compara_nimic() {
        let (d, c) = instantaneu("anulata");
        let anulat = AtomicBool::new(true);
        let r = pornire::compara_cu(&c, &mut |_| {}, &anulat);
        assert_eq!(r.err().as_deref(), Some(pornire::ANULATA));
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn anulata_pe_drum_se_opreste_la_pasul_urmator() {
        let (d, c) = instantaneu("pe-drum");
        let anulat = AtomicBool::new(false);
        let mut pasi = 0;
        let r = pornire::compara_cu(
            &c,
            &mut |_| {
                pasi += 1;
                // Anuleaza apasat in timpul celui de-al doilea pas
                if pasi == 2 {
                    anulat.store(true, Ordering::Relaxed);
                }
            },
            &anulat,
        );
        assert_eq!(r.err().as_deref(), Some(pornire::ANULATA));
        assert_eq!(pasi, 2, "dupa anulare nu se mai citeste nimic");
        std::fs::remove_dir_all(&d).ok();
    }
}
