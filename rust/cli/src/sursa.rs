//! De unde vin obiectele: de pe un server sau din un instantaneu.
//!
//! **Se strica astfel:** o comanda merge pe server dar nu pe instantaneu (sau
//! invers), si atunci se pierde exact proprietatea pentru care exista
//! `SursaObiecte` — ca tot ce face aplicatia sa poată fi reluat offline.
//!
//! De-aia construirea sursei e intr-un singur loc, si fiecare comanda care are
//! nevoie de obiecte o foloseste. Daca apare o comanda care accepta doar
//! server, asta e bug-ul.

use gmotion_db::{Autentificare, Certificat, SursaFixture, SursaObiecte, SursaSqlServer, Tinta};

use crate::acreditari::ArgumenteAuth;

#[derive(Debug, Clone, clap::Args)]
pub struct ArgumenteSursa {
    /// Serverul SQL: `SRV01`, `SRV01\SQLEXPRESS` sau `srv.domeniu.local`.
    #[arg(long, value_name = "SERVER", conflicts_with = "sursa")]
    pub server: Option<String>,

    /// Port, daca nu e cel implicit.
    #[arg(long, requires = "server")]
    pub port: Option<u16>,

    /// Verificarea certificatului serverului: `nevalidat` (implicit, ca in
    /// v3.4: criptat, dar oricine se pune la mijloc poate citi parola),
    /// `sistem` (depozitul de certificate al sistemului), sau calea unei
    /// autoritati de certificare (`.pem`/`.crt`).
    #[arg(
        long,
        value_name = "MOD",
        requires = "server",
        default_value = "nevalidat"
    )]
    pub certificat: String,

    /// Citeste dintr-un instantaneu in loc de pe un server. Asa se reia un bug
    /// de pe o baza reala, pe orice masina.
    #[arg(long, value_name = "CALE")]
    pub sursa: Option<std::path::PathBuf>,

    #[command(flatten)]
    pub auth: ArgumenteAuth,
}

impl ArgumenteSursa {
    /// Ce se scrie in campul `server` al unui instantaneu: numele dat, cu
    /// portul in forma `SRV,PORT` daca a fost dat. **Nu** `descriere()` — aceea
    /// e pentru jurnal si a dus o data informatii despre parola intr-un
    /// fisier commitat.
    pub fn eticheta_server(&self) -> Option<String> {
        self.server.as_ref().map(|s| match self.port {
            Some(p) => format!("{s},{p}"),
            None => s.clone(),
        })
    }

    pub fn deschide(&self) -> Result<Box<dyn SursaObiecte>, String> {
        match (&self.sursa, &self.server) {
            (Some(cale), _) => {
                let s = SursaFixture::din_fisier(cale).map_err(|e| e.to_string())?;
                tracing::info!(target: "gmotion::cli", sursa = %s.descriere(), "citesc din instantaneu");
                Ok(Box::new(s))
            }
            (None, Some(server)) => {
                let auth: Autentificare = self.auth.rezolva()?;
                let s = SursaSqlServer::nou(
                    Tinta {
                        server: server.clone(),
                        port: self.port,
                        certificat: certificat(&self.certificat),
                    },
                    auth,
                )
                .map_err(|e| e.to_string())?;
                tracing::info!(target: "gmotion::cli", sursa = %s.descriere(), "citesc de pe server");
                Ok(Box::new(s))
            }
            (None, None) => Err("da ori --server <SERVER>, ori --sursa <instantaneu.json>".into()),
        }
    }
}

fn certificat(mod_: &str) -> Certificat {
    match mod_ {
        "nevalidat" => Certificat::Nevalidat,
        "sistem" => Certificat::Sistem,
        cale => Certificat::Autoritate(cale.into()),
    }
}

/// Imparte o lista data pe linia de comanda: `A,B,C`.
///
/// Spatiile se taie si elementele goale se arunca, fiindca `--baze "A, B,"` e
/// ce scrie un om.
/// Fara dubluri, in ordinea data: `--baze A,A` compara A de doua ori, dubla
/// numaratorile si scria fiecare bloc de doua ori in deploy. Gasit la audit.
pub fn lista(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for x in s.split(',').map(str::trim).filter(|x| !x.is_empty()) {
        if !out.iter().any(|y| y == x) {
            out.push(x.to_string());
        }
    }
    out
}

#[cfg(test)]
mod teste {
    use super::*;

    /// `unwrap_err()` ar cere `Debug` pe `Box<dyn SursaObiecte>`, care nu-l are
    /// si nici nu trebuie sa-l aiba: ar insemna o reprezentare a unui tip care
    /// tine acreditari.
    fn eroarea(a: &ArgumenteSursa) -> String {
        match a.deschide() {
            Err(e) => e,
            Ok(_) => panic!("ar fi trebuit sa eșueze"),
        }
    }

    #[test]
    fn lista_tolereaza_spatii_si_virgule_in_plus() {
        assert_eq!(lista("A,B"), ["A", "B"]);
        assert_eq!(lista("A, B , C"), ["A", "B", "C"]);
        assert_eq!(lista("A,,B,"), ["A", "B"]);
        assert_eq!(lista(""), Vec::<String>::new());
        assert_eq!(lista("  "), Vec::<String>::new());
        assert_eq!(lista("A,B,A, B"), ["A", "B"]);
    }

    #[test]
    fn eticheta_de_server_nu_are_nimic_despre_autentificare() {
        let mut a = ArgumenteSursa {
            server: Some(r"TS2409\MIHAIG".into()),
            port: None,
            certificat: "nevalidat".into(),
            sursa: None,
            auth: ArgumenteAuth {
                utilizator: Some("sa".into()),
                parola_fisier: None,
                parola_stdin: false,
            },
        };
        assert_eq!(a.eticheta_server().as_deref(), Some(r"TS2409\MIHAIG"));
        a.port = Some(1500);
        assert_eq!(a.eticheta_server().as_deref(), Some(r"TS2409\MIHAIG,1500"));
        a.server = None;
        assert_eq!(a.eticheta_server(), None);
    }

    #[test]
    fn fara_server_si_fara_sursa_e_eroare_explicata() {
        let a = ArgumenteSursa {
            server: None,
            port: None,
            certificat: "nevalidat".into(),
            sursa: None,
            auth: ArgumenteAuth {
                utilizator: None,
                parola_fisier: None,
                parola_stdin: false,
            },
        };
        let e = eroarea(&a);
        assert!(e.contains("--server"), "{e}");
        assert!(e.contains("--sursa"), "{e}");
    }

    #[test]
    fn instantaneu_inexistent_da_codul_de_la_db() {
        let a = ArgumenteSursa {
            server: None,
            port: None,
            certificat: "nevalidat".into(),
            sursa: Some(std::path::PathBuf::from("/nu/exista.json")),
            auth: ArgumenteAuth {
                utilizator: None,
                parola_fisier: None,
                parola_stdin: false,
            },
        };
        let e = eroarea(&a);
        assert!(e.contains("GM-DB-007"), "codul nu trebuie pierdut: {e}");
    }
}
