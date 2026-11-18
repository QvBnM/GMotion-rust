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
    pub tinta: modul::Baza,
    pub server: String,
}

/// Citeste ambele baze si compara. Sincron: ecranul de conectare nu are
/// progres in machete, iar pe instantaneu dureaza oricum sub o secunda.
/// (Pe server real, progresul e lucru de etapa viitoare.)
pub fn compara(c: &Conectare) -> Result<Comparata, String> {
    let mut sursa_db = deschide(c, &c.sursa)?;
    let server = sursa_db.descriere();
    let obiecte_sursa = sursa_db.obiecte(&c.sursa.baza).map_err(|e| e.to_string())?;
    let obiecte_tinta = if c.instantaneu.is_some() {
        sursa_db.obiecte(&c.baza.baza).map_err(|e| e.to_string())?
    } else {
        let mut tinta_db = deschide(c, &c.baza)?;
        tinta_db.obiecte(&c.baza.baza).map_err(|e| e.to_string())?
    };
    let sursa = modul::pregateste(&obiecte_sursa);
    let tinta = modul::pregateste(&obiecte_tinta);
    let comparare = modul::compara(
        &c.sursa.baza,
        &sursa,
        &[(c.baza.baza.clone(), tinta.clone())],
        |_| {},
    );
    Ok(Comparata {
        randuri: comparare.randuri,
        sursa,
        tinta,
        server,
    })
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
        std::fs::remove_dir_all(&d).ok();
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
