//! Compararea pe alt fir, cu progres si Anuleaza.
//!
//! **Se strica astfel:** fereastra pare inghetata cat se citesc bazele (12 s
//! pe o baza adevarata), sau — mai rau — ramane „se compara…" dupa ce firul a
//! murit. De-aia firul trimite **mereu** o ultima veste (`Gata` sau `Esuata`),
//! iar partea de fereastra se uita doar la ce a primit, nu la ce presupune.
//!
//! Nimic din Slint nu intra aici: firul lucreaza cu `Conectare` si trimite
//! structuri simple.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::Instant;

use gmotion_fereastra::model_conectare::Conectare;

use crate::pornire::{self, Comparata, Pas};

/// Ce trimite firul inapoi.
pub enum Veste {
    /// Se citeste baza asta (a cata din cate).
    Progres(Pas),
    Gata(Box<Comparata>),
    /// Mesajul erorii; `pornire::ANULAT` daca s-a apasat Anuleaza.
    Esuata(String),
}

/// Un lucru lung care nu are voie sa tina fereastra: se porneste pe alt fir
/// si se intreaba la fiecare bataie de ceas, ca si compararea.
///
/// Exista fiindca „citeste bazele", „Testeaza conexiunea", „cauta servere" si
/// garda de deploy rulau **pe firul ferestrei**: un server inaccesibil o lasa
/// „Not responding" pana la timeout (30 s), iar garda pe o baza mare, minute
/// (P4 din auditul QA).
pub struct Lucru<T> {
    vesti: Receiver<Result<T, String>>,
    pornit_la: Instant,
}

impl<T: Send + 'static> Lucru<T> {
    /// Porneste `f` pe alt fir. Nu blocheaza.
    pub fn porneste(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Lucru<T> {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            // chiar si o panica in `f` trebuie sa lase ecranul sa iasa din
            // „lucreaza…": canalul inchis devine eroare la `verifica`
            let _ = tx.send(f());
        });
        Lucru {
            vesti: rx,
            pornit_la: Instant::now(),
        }
    }

    /// `Some` cand s-a terminat; atunci `Lucru` se arunca.
    pub fn verifica(&mut self) -> Option<Result<T, String>> {
        match self.vesti.try_recv() {
            Ok(r) => Some(r),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                Some(Err("lucrul s-a oprit pe neasteptate".to_string()))
            }
        }
    }

    /// Cat a trecut, ca „0:12".
    pub fn timp(&self) -> String {
        let s = self.pornit_la.elapsed().as_secs();
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// O comparare care ruleaza acum.
pub struct Rulare {
    anulare: Arc<AtomicBool>,
    vesti: Receiver<Veste>,
    pornit_la: Instant,
    /// Ultimul pas primit: din el se deseneaza bara si textul.
    pub pas: Option<Pas>,
    /// `true` dupa ce s-a apasat Anuleaza, ca butonul sa nu mai raspunda.
    pub se_opreste: bool,
}

impl Rulare {
    /// Porneste compararea pe alt fir. Nu blocheaza.
    pub fn porneste(c: Conectare) -> Rulare {
        let anulare = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let steag = anulare.clone();
        std::thread::spawn(move || {
            let tx_progres = tx.clone();
            let rezultat = pornire::compara_cu(&c, |p| {
                // un canal inchis (fereastra a plecat) opreste si firul
                tx_progres.send(Veste::Progres(p)).is_ok() && !steag.load(Ordering::Relaxed)
            });
            let _ = tx.send(match rezultat {
                Ok(r) => Veste::Gata(Box::new(r)),
                Err(e) => Veste::Esuata(e),
            });
        });
        Rulare {
            anulare,
            vesti: rx,
            pornit_la: Instant::now(),
            pas: None,
            se_opreste: false,
        }
    }

    /// Cere oprirea. Firul se opreste la urmatorul pas, deci ecranul ramane
    /// pe progres pana vine vestea finala.
    pub fn anuleaza(&mut self) {
        self.anulare.store(true, Ordering::Relaxed);
        self.se_opreste = true;
    }

    /// Ia vestile adunate. Intoarce `Some` cand compararea s-a terminat
    /// (reusita, esuata sau anulata) — atunci `Rulare` se arunca.
    pub fn verifica(&mut self) -> Option<Result<Box<Comparata>, String>> {
        loop {
            match self.vesti.try_recv() {
                Ok(Veste::Progres(p)) => self.pas = Some(p),
                Ok(Veste::Gata(r)) => return Some(Ok(r)),
                Ok(Veste::Esuata(e)) => return Some(Err(e)),
                Err(TryRecvError::Empty) => return None,
                // firul a murit fara sa spuna nimic: mai bine o eroare decat
                // un ecran blocat pe „se compara…" la nesfarsit
                Err(TryRecvError::Disconnected) => {
                    return Some(Err("compararea s-a oprit pe neasteptate".into()))
                }
            }
        }
    }

    /// Cat a trecut, ca „0:12".
    pub fn timp(&self) -> String {
        let s = self.pornit_la.elapsed().as_secs();
        format!("{}:{:02}", s / 60, s % 60)
    }

    /// Cat din drum s-a facut, 0..1.
    pub fn fractie(&self) -> f32 {
        match &self.pas {
            Some(p) if p.din > 0 => p.facut as f32 / p.din as f32,
            _ => 0.0,
        }
    }

    /// Propozitia de sub bara.
    pub fn text(&self) -> String {
        if self.se_opreste {
            return "se opreste…".to_string();
        }
        match &self.pas {
            Some(p) => format!("Citesc {} ({} din {})", p.baza, p.facut + 1, p.din),
            None => "Se porneste…".to_string(),
        }
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    fn instantaneu(d: &std::path::Path) -> Conectare {
        std::fs::create_dir_all(d).unwrap();
        let cale = d.join("inst.json");
        std::fs::write(
            &cale,
            r#"{"versiune":1,"server":"SRV","creat":"2026-11-17T10:00:00Z","nota":"","baze":{
                "Ref":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 1"}],
                "C1":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 2"}]}}"#,
        )
        .unwrap();
        let mut c = Conectare::noua();
        c.instantaneu = Some(cale.to_string_lossy().to_string());
        c.sursa.baza = "Ref".into();
        c.baza.comuta_baza("C1");
        c
    }

    /// Asteapta pana firul termina, ca fereastra la fiecare bataie de ceas.
    fn asteapta(r: &mut Rulare) -> Result<Box<Comparata>, String> {
        for _ in 0..600 {
            if let Some(x) = r.verifica() {
                return x;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("compararea nu s-a terminat in 3 secunde");
    }

    #[test]
    fn compararea_pe_alt_fir_da_progres_si_rezultat() {
        let d = std::env::temp_dir().join(format!("gmotion-fir-{}", std::process::id()));
        let c = instantaneu(&d);
        let mut r = Rulare::porneste(c);
        assert_eq!(r.text(), "Se porneste…", "inainte de primul pas");
        assert_eq!(r.fractie(), 0.0);
        let rezultat = asteapta(&mut r).expect("compararea merge");
        assert_eq!(rezultat.randuri.len(), 1);
        assert!(r.pas.is_some(), "s-a primit cel putin un pas");
        assert!(r.text().starts_with("Citesc "), "{}", r.text());
        assert!(r.timp().starts_with("0:"), "{}", r.timp());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn anularea_opreste_compararea_si_spune_asta() {
        let d = std::env::temp_dir().join(format!("gmotion-fir-anulat-{}", std::process::id()));
        let c = instantaneu(&d);
        let mut r = Rulare::porneste(c);
        r.anuleaza();
        assert_eq!(r.text(), "se opreste…");
        // firul poate apuca sa termine inainte de a vedea steagul: ambele
        // raspunsuri sunt corecte, important e sa nu ramana agatat
        if let Err(e) = asteapta(&mut r) {
            assert_eq!(e, pornire::ANULAT);
        }
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn o_comparare_esuata_vine_ca_eroare_nu_ca_blocaj() {
        let mut c = Conectare::noua();
        c.instantaneu = Some("/nu/exista/inst.json".into());
        c.sursa.baza = "Ref".into();
        c.baza.comuta_baza("C1");
        let mut r = Rulare::porneste(c);
        let e = asteapta(&mut r).map(|_| ()).expect_err("fisierul lipseste");
        assert!(e.contains("GM-DB-007"), "{e}");
    }

    #[test]
    fn un_lucru_lung_nu_tine_fereastra_si_isi_spune_rezultatul() {
        let mut l = Lucru::porneste(|| {
            std::thread::sleep(std::time::Duration::from_millis(40));
            Ok::<_, String>(21 * 2)
        });
        assert!(l.verifica().is_none(), "nu blocheaza la pornire");
        let mut rezultat = None;
        for _ in 0..200 {
            if let Some(r) = l.verifica() {
                rezultat = Some(r);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(rezultat.expect("s-a terminat"), Ok(42));

        // o eroare vine ca eroare, nu ca blocaj
        let mut l = Lucru::porneste(|| Err::<(), String>("serverul nu raspunde".into()));
        let mut rezultat = None;
        for _ in 0..200 {
            if let Some(r) = l.verifica() {
                rezultat = Some(r);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(rezultat, Some(Err("serverul nu raspunde".to_string())));
    }
}
