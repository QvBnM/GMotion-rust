//! De unde vin obiectele: ce alege utilizatorul in ecranul de conectare se
//! transforma aici in surse adevarate si intr-o comparare.
//!
//! **Se strica astfel:** parola ajunge intr-un mesaj de eroare sau intr-o
//! sesiune salvata, sau ecranul spune „conectat" cand nu e. De-aia construirea
//! sursei sta intr-un singur loc, si ce se arata vine din ce chiar s-a
//! intamplat (bazele citite), nu din ce s-a cerut.

use gmotion_db::{Autentificare, Certificat, SursaFixture, SursaObiecte, SursaSqlServer, Tinta};
use gmotion_fereastra::model_conectare::{Conectare, Parte, StareConexiune};
use gmotion_modul_diff_obiecte as modul;

/// Deschide sursa unei parti, dupa ce s-a ales din ecran.
///
/// Parola se ia din `Acreditari` chiar aici si nu mai iese: nici in jurnal,
/// nici in starea afisata.
pub fn deschide(c: &Conectare, p: &Parte) -> Result<Box<dyn SursaObiecte>, String> {
    if let Some(cale) = &c.instantaneu {
        return SursaFixture::din_fisier(cale)
            .map(|s| Box::new(s) as Box<dyn SursaObiecte>)
            .map_err(|e| e.to_string());
    }
    let auth = if p.auth_windows {
        Autentificare::Integrata
    } else {
        Autentificare::Sql {
            utilizator: p.acreditari.utilizator.clone(),
            parola: p.acreditari.parola().to_string(),
        }
    };
    let (server, port) = imparte_portul(&p.server);
    SursaSqlServer::nou(
        Tinta {
            server,
            port,
            certificat: certificat(c),
        },
        auth,
    )
    .map(|s| Box::new(s) as Box<dyn SursaObiecte>)
    .map_err(|e| e.to_string())
}

/// Ce alege ecranul ajunge in conexiune. Pana la 2026-11-18 aici era mereu
/// `Nevalidat`, desi ecranul putea spune „se verifica".
fn certificat(c: &Conectare) -> Certificat {
    if c.verifica_certificatul {
        Certificat::Sistem
    } else {
        Certificat::Nevalidat
    }
}

/// `srv.domeniu.local,1433` -> (`srv.domeniu.local`, 1433). Forma cu virgula e
/// cea pe care o scrie lumea (si cea din macheta), nu una inventata aici.
fn imparte_portul(server: &str) -> (String, Option<u16>) {
    match server.rsplit_once(',') {
        Some((s, p)) => match p.trim().parse::<u16>() {
            Ok(port) => (s.trim().to_string(), Some(port)),
            Err(_) => (server.trim().to_string(), None),
        },
        None => (server.trim().to_string(), None),
    }
}

/// Citeste bazele unei parti si intoarce starea de afisat.
///
/// Nu spune „conectat" decat daca chiar a citit ceva: asta e singurul lucru
/// care dovedeste conexiunea.
pub fn citeste_bazele(c: &Conectare, p: &Parte) -> (Vec<String>, StareConexiune) {
    let mut sursa = match deschide(c, p) {
        Ok(s) => s,
        Err(e) => return (Vec::new(), StareConexiune::Eroare(e)),
    };
    match sursa.baze() {
        Ok(baze) => {
            let stare = StareConexiune::Conectat(format!(
                "conectat · {} {}",
                baze.len(),
                if baze.len() == 1 {
                    "baza"
                } else if baze.len() < 20 {
                    "baze"
                } else {
                    "de baze"
                }
            ));
            (baze, stare)
        }
        Err(e) => (Vec::new(), StareConexiune::Eroare(e.to_string())),
    }
}

/// Rezultatul unei comparari pornite din ecranul de conectare.
pub struct Comparata {
    pub randuri: Vec<modul::Rand>,
    pub sursa: modul::Baza,
    /// Bazele comparate, in ordinea bifata. Una singura e cazul obisnuit.
    pub tinte: Vec<(String, modul::Baza)>,
    pub server: String,
}

impl Comparata {
    /// Prima tinta: ecranele care arata o singura baza (antetul, deploy-ul
    /// pe o baza) au nevoie de un nume de aratat.
    pub fn prima_tinta(&self) -> &str {
        self.tinte.first().map_or("", |(n, _)| n.as_str())
    }
}

/// Citeste sursa si fiecare baza bifata, si compara. `progres` primeste
/// pasii pe rand (ce baza se citeste, a cata din cate) si intoarce `false`
/// daca utilizatorul a apasat „Anuleaza".
pub fn compara_cu(
    c: &Conectare,
    mut progres: impl FnMut(Pas) -> bool,
) -> Result<Comparata, String> {
    let baze = c.baza.baze_alese();
    if baze.is_empty() {
        return Err("nicio baza de comparat".to_string());
    }
    let mut sursa_db = deschide(c, &c.sursa)?;
    let server = sursa_db.descriere();
    if !progres(Pas {
        baza: c.sursa.baza.clone(),
        facut: 0,
        din: baze.len() + 1,
    }) {
        return Err(ANULAT.to_string());
    }
    let obiecte_sursa = sursa_db.obiecte(&c.sursa.baza).map_err(|e| e.to_string())?;
    let sursa = modul::pregateste(&obiecte_sursa);

    let mut tinte: Vec<(String, modul::Baza)> = Vec::new();
    for (i, nume) in baze.iter().enumerate() {
        if !progres(Pas {
            baza: nume.clone(),
            facut: i + 1,
            din: baze.len() + 1,
        }) {
            return Err(ANULAT.to_string());
        }
        // pe instantaneu toate bazele vin din acelasi fisier, deci nu se
        // deschide o a doua conexiune pentru fiecare
        let obiecte = if c.instantaneu.is_some() {
            sursa_db.obiecte(nume).map_err(|e| e.to_string())?
        } else {
            let mut db = deschide(c, &c.baza)?;
            db.obiecte(nume).map_err(|e| e.to_string())?
        };
        tinte.push((nume.clone(), modul::pregateste(&obiecte)));
    }

    let comparare = modul::compara(&c.sursa.baza, &sursa, &tinte, |_| {});
    Ok(Comparata {
        randuri: comparare.randuri,
        sursa,
        tinte,
        server,
    })
}

/// Unde a ajuns compararea: ce baza se citeste acum, a cata din cate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pas {
    pub baza: String,
    pub facut: usize,
    pub din: usize,
}

/// Mesajul cu care se incheie o comparare oprita de utilizator. Se recunoaste
/// dupa text in fereastra, ca sa nu se arate ca eroare.
pub const ANULAT: &str = "comparare anulata";

/// Compararea fara progres si fara anulare.
pub fn compara(c: &Conectare) -> Result<Comparata, String> {
    compara_cu(c, |_| true)
}

#[cfg(test)]
mod teste {
    use super::*;
    use gmotion_fereastra::model_conectare::Acreditari;

    #[test]
    fn alegerea_certificatului_ajunge_in_conexiune() {
        let mut c = Conectare::noua();
        assert_eq!(certificat(&c), Certificat::Nevalidat);
        c.verifica_certificatul = true;
        assert_eq!(certificat(&c), Certificat::Sistem);
    }

    #[test]
    fn portul_se_desparte_de_server_ca_in_machete() {
        assert_eq!(
            imparte_portul("srv-prod.firma.local,1433"),
            ("srv-prod.firma.local".to_string(), Some(1433))
        );
        assert_eq!(
            imparte_portul("SRV-DEV\\SQL2019"),
            ("SRV-DEV\\SQL2019".to_string(), None)
        );
        // o virgula care nu e port ramane in numele serverului
        assert_eq!(
            imparte_portul("srv,instanta"),
            ("srv,instanta".to_string(), None)
        );
        assert_eq!(
            imparte_portul("  srv , 1433 "),
            ("srv".to_string(), Some(1433))
        );
    }

    fn instantaneu(cale: &std::path::Path) {
        std::fs::write(
            cale,
            r#"{"versiune":1,"server":"SRV","creat":"2026-11-17T10:00:00Z","nota":"","baze":{
                "A":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 1"}],
                "B":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 2"}]}}"#,
        )
        .unwrap();
    }

    #[test]
    fn dintr_un_instantaneu_se_citesc_bazele_si_se_compara() {
        let d = std::env::temp_dir().join(format!("gmotion-pornire-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let cale = d.join("inst.json");
        instantaneu(&cale);

        let mut c = Conectare::noua();
        c.instantaneu = Some(cale.to_string_lossy().to_string());
        let (baze, stare) = citeste_bazele(&c, &c.sursa);
        assert_eq!(baze, ["A", "B"]);
        assert!(stare.e_conectat(), "{stare:?}");
        assert_eq!(stare.text(), "conectat · 2 baze");

        c.sursa.baza = "A".into();
        c.baza.baza = "B".into();
        let r = compara(&c).expect("compararea merge");
        assert_eq!(r.randuri.len(), 1);
        assert_eq!(r.randuri[0].obiect, "dbo.p");
        assert_eq!(r.prima_tinta(), "B");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn o_referinta_se_compara_cu_mai_multe_baze_deodata() {
        let d = std::env::temp_dir().join(format!("gmotion-multi-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let cale = d.join("inst.json");
        std::fs::write(
            &cale,
            r#"{"versiune":1,"server":"SRV","creat":"2026-11-17T10:00:00Z","nota":"","baze":{
                "Ref":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 1"}],
                "C1":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 2"}],
                "C2":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 3"}],
                "C3":[]}}"#,
        )
        .unwrap();
        let mut c = Conectare::noua();
        c.instantaneu = Some(cale.to_string_lossy().to_string());
        c.sursa.baza = "Ref".into();
        c.baza.comuta_baza("C1");
        c.baza.comuta_baza("C2");

        let pasi = std::cell::RefCell::new(Vec::new());
        let r = compara_cu(&c, |p| {
            pasi.borrow_mut().push(p.baza.clone());
            true
        })
        .expect("compararea merge");
        assert_eq!(
            *pasi.borrow(),
            ["Ref", "C1", "C2"],
            "progresul spune ce baza se citeste, referinta intai"
        );
        assert_eq!(r.tinte.len(), 2);
        assert_eq!(r.prima_tinta(), "C1");
        // acelasi obiect apare o data pentru fiecare baza
        let baze: Vec<&str> = r.randuri.iter().map(|x| x.baza.as_str()).collect();
        assert_eq!(baze, ["C1", "C2"]);

        // „Anuleaza" opreste fara sa citeasca restul
        let citite = std::cell::Cell::new(0);
        let e = compara_cu(&c, |_| {
            citite.set(citite.get() + 1);
            citite.get() < 2
        })
        .map(|_| ())
        .expect_err("anulata");
        assert_eq!(e, ANULAT);
        assert_eq!(citite.get(), 2, "s-a oprit la a doua intrebare");

        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn fara_nicio_baza_bifata_nu_e_nimic_de_comparat() {
        let mut c = Conectare::noua();
        c.instantaneu = Some("/nu/conteaza.json".into());
        c.sursa.baza = "Ref".into();
        assert_eq!(
            compara(&c).map(|_| ()).expect_err("nimic de comparat"),
            "nicio baza de comparat"
        );
    }

    #[test]
    fn un_instantaneu_inexistent_da_eroare_nu_conectat() {
        let mut c = Conectare::noua();
        c.instantaneu = Some("/nu/exista/inst.json".into());
        let (baze, stare) = citeste_bazele(&c, &c.sursa);
        assert!(baze.is_empty());
        assert!(!stare.e_conectat(), "{stare:?}");
        assert!(stare.text().contains("GM-DB-007"), "{}", stare.text());
    }

    #[test]
    fn parola_nu_apare_in_eroarea_de_conectare() {
        let mut c = Conectare::noua();
        c.sursa.server = "server.care.nu.exista.invalid".into();
        c.sursa.auth_windows = false;
        c.sursa.acreditari = Acreditari::noi("deploy_ro", "parola-secreta");
        let (_, stare) = citeste_bazele(&c, &c.sursa);
        assert!(!stare.e_conectat());
        assert!(
            !stare.text().contains("parola-secreta"),
            "parola in mesajul de eroare: {}",
            stare.text()
        );
    }
}
