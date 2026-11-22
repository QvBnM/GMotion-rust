//! De unde vine parola.
//!
//! **Se strica astfel:** parola ajunge intr-un loc din care o poate citi
//! altcineva. De-aia e un fisier separat, cu o singura treaba.
//!
//! **Nu exista `--parola <valoare>`**, deliberat. Un argument de linie de
//! comanda ajunge in istoricul shell-ului (`~/.bash_history`,
//! `ConsoleHost_history.txt`) si e vizibil in lista de procese pentru orice
//! utilizator de pe masina, cat timp comanda ruleaza. Caile acceptate:
//!
//! 1. variabila de mediu `GMOTION_PAROLA`;
//! 2. `--parola-fisier <cale>` — util pentru rulari programate;
//! 3. `--parola-stdin` — se citeste prima linie de la intrarea standard.
//!
//! Autentificarea integrata Windows nu are nevoie de niciuna.

use std::path::PathBuf;

use gmotion_db::Autentificare;

/// Numele variabilei de mediu. Un singur loc de adevar, folosit si in mesajele
/// de eroare.
pub const VAR_PAROLA: &str = "GMOTION_PAROLA";

/// `Debug` e scris de mana: dupa rezolvare tine parola (`rezolvata`).
#[derive(Clone, clap::Args)]
pub struct ArgumenteAuth {
    /// Utilizator SQL Server. Lipsa lui inseamna autentificare integrata
    /// Windows.
    #[arg(long, value_name = "UTILIZATOR")]
    pub utilizator: Option<String>,

    /// Citeste parola din acest fisier (prima linie).
    #[arg(long, value_name = "CALE", conflicts_with = "parola_stdin")]
    pub parola_fisier: Option<PathBuf>,

    /// Citeste parola de la intrarea standard (prima linie).
    #[arg(long)]
    pub parola_stdin: bool,

    /// Autentificarea, rezolvata o singura data pe rulare: o comanda deschide
    /// sursa de mai multe ori (identitate, comparare, garda), iar
    /// `--parola-stdin` se poate citi o singura data. A doua citire era goala
    /// (B3 din `cercetare/audit-qa-2026-11-22.md`).
    #[arg(skip)]
    pub rezolvata: std::sync::OnceLock<Autentificare>,
}

impl std::fmt::Debug for ArgumenteAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArgumenteAuth")
            .field("utilizator", &self.utilizator)
            .field("parola_fisier", &self.parola_fisier)
            .field("parola_stdin", &self.parola_stdin)
            .field("rezolvata", &self.rezolvata.get().is_some())
            .finish()
    }
}

impl ArgumenteAuth {
    /// Construieste autentificarea. Mesajele de eroare spun **exact** ce sa
    /// faca utilizatorul, fiindca asta e primul loc in care se blocheaza
    /// cineva care foloseste unealta prima data.
    pub fn rezolva(&self) -> Result<Autentificare, String> {
        if let Some(a) = self.rezolvata.get() {
            return Ok(a.clone());
        }
        let a = self.rezolva_o_data()?;
        Ok(self.rezolvata.get_or_init(|| a).clone())
    }

    fn rezolva_o_data(&self) -> Result<Autentificare, String> {
        let Some(utilizator) = self.utilizator.clone() else {
            if self.parola_fisier.is_some() || self.parola_stdin {
                return Err(
                    "[GM-CLI-001] ai dat o parola dar niciun --utilizator. Pentru \
                    autentificare integrata Windows nu da parola; pentru SQL \
                    Auth da si --utilizator"
                        .into(),
                );
            }
            return Ok(Autentificare::Integrata);
        };

        let parola = if let Some(cale) = &self.parola_fisier {
            let text = std::fs::read_to_string(cale).map_err(|e| {
                format!(
                    "[GM-CLI-002] nu pot citi parola din {}: {e}",
                    cale.display()
                )
            })?;
            prima_linie(&text)
        } else if self.parola_stdin {
            // O singura linie: tastata in terminal, citirea pana la sfarsitul
            // intrarii astepta Ctrl-D / Ctrl-Z si parea agatata. Gasit la audit.
            let mut text = String::new();
            std::io::stdin().read_line(&mut text).map_err(|e| {
                format!("[GM-CLI-002] nu pot citi parola de la intrarea standard: {e}")
            })?;
            prima_linie(&text)
        } else {
            match std::env::var(VAR_PAROLA) {
                Ok(p) if !p.is_empty() => p,
                _ => {
                    return Err(format!(
                        "[GM-CLI-002] lipseste parola pentru utilizatorul {utilizator}. Pune-o in \
                         variabila {VAR_PAROLA}, sau foloseste --parola-fisier / \
                         --parola-stdin.\n\
                         Nu exista --parola ca argument: ar ajunge in istoricul \
                         shell-ului si in lista de procese."
                    ))
                }
            }
        };

        if parola.is_empty() {
            return Err("[GM-CLI-002] parola citita e goala".into());
        }
        Ok(Autentificare::Sql { utilizator, parola })
    }
}

/// Prima linie, fara terminatorii. O parola nu contine `\n`, iar un fisier
/// scris de un editor are mai mult ca sigur o linie noua la final.
/// Prima linie, fara BOM la inceput si fara `\r` la sfarsit. PowerShell 5.1
/// (`Out-File`, `-Encoding utf8`) scrie BOM; un fisier cu terminatii vechi de
/// Mac lasa `\r`. Amandoua ajungeau in parola si dadeau un „login refuzat"
/// fara explicatie. Gasit la audit.
fn prima_linie(text: &str) -> String {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let linie = text.split(['\n', '\r']).next().unwrap_or("");
    linie.to_string()
}

#[cfg(test)]
mod teste {
    use super::*;

    fn fara_parola(utilizator: Option<&str>) -> ArgumenteAuth {
        ArgumenteAuth {
            utilizator: utilizator.map(str::to_string),
            parola_fisier: None,
            parola_stdin: false,
            rezolvata: Default::default(),
        }
    }

    #[test]
    fn fara_utilizator_inseamna_auth_integrata() {
        assert_eq!(
            fara_parola(None).rezolva().unwrap(),
            Autentificare::Integrata
        );
    }

    #[test]
    fn parola_fara_utilizator_e_o_greseala_explicata() {
        let a = ArgumenteAuth {
            utilizator: None,
            parola_fisier: Some(PathBuf::from("/x")),
            parola_stdin: false,
            rezolvata: Default::default(),
        };
        let e = a.rezolva().unwrap_err();
        assert!(e.contains("--utilizator"), "{e}");
    }

    #[test]
    fn utilizator_fara_parola_spune_ce_sa_faca() {
        // Nu depinde de mediu: testul ruleaza doar daca variabila nu e setata,
        // altfel verifica cealalta ramura. Fara umblat la mediul global, ca sa
        // nu fie nevoie de serializare.
        match std::env::var(VAR_PAROLA) {
            Ok(p) if !p.is_empty() => {
                assert!(matches!(
                    fara_parola(Some("sa")).rezolva(),
                    Ok(Autentificare::Sql { .. })
                ));
            }
            _ => {
                let e = fara_parola(Some("sa")).rezolva().unwrap_err();
                assert!(e.contains(VAR_PAROLA), "{e}");
                assert!(e.contains("--parola-fisier"), "{e}");
                assert!(
                    e.contains("istoricul"),
                    "mesajul trebuie sa explice de ce nu exista --parola: {e}"
                );
            }
        }
    }

    #[test]
    fn parola_din_fisier_ignora_linia_noua() {
        let d = std::env::temp_dir().join(format!("gmotion-par-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let cale = d.join("p.txt");
        std::fs::write(&cale, "parola-mea\n").unwrap();
        let a = ArgumenteAuth {
            utilizator: Some("sa".into()),
            parola_fisier: Some(cale),
            parola_stdin: false,
            rezolvata: Default::default(),
        };
        assert_eq!(
            a.rezolva().unwrap(),
            Autentificare::Sql {
                utilizator: "sa".into(),
                parola: "parola-mea".into()
            }
        );
        std::fs::remove_dir_all(&d).ok();
    }

    /// B3 din `cercetare/audit-qa-2026-11-22.md`: o comanda deschide sursa de
    /// mai multe ori; parola se citeste o singura data. Fisierul sters dupa
    /// prima rezolvare dovedeste ca a doua nu mai citeste nimic.
    #[test]
    fn parola_se_citeste_o_singura_data_pe_rulare() {
        let d = std::env::temp_dir().join(format!("gmotion-par1-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let cale = d.join("p.txt");
        std::fs::write(&cale, "parola-mea\n").unwrap();
        let a = ArgumenteAuth {
            utilizator: Some("sa".into()),
            parola_fisier: Some(cale),
            parola_stdin: false,
            rezolvata: Default::default(),
        };
        let prima = a.rezolva().unwrap();
        std::fs::remove_dir_all(&d).unwrap();
        assert_eq!(a.rezolva().unwrap(), prima);
    }

    #[test]
    fn fisier_de_parola_lipsa_da_mesaj_cu_calea() {
        let a = ArgumenteAuth {
            utilizator: Some("sa".into()),
            parola_fisier: Some(PathBuf::from("/cale/inexistenta/p.txt")),
            parola_stdin: false,
            rezolvata: Default::default(),
        };
        let e = a.rezolva().unwrap_err();
        assert!(e.contains("/cale/inexistenta/p.txt"), "{e}");
    }

    #[test]
    fn prima_linie_curata() {
        assert_eq!(prima_linie("abc\n"), "abc");
        assert_eq!(prima_linie("abc\r\ndef"), "abc");
        assert_eq!(prima_linie(""), "");
        assert_eq!(prima_linie("\n"), "");
        assert_eq!(prima_linie("\u{feff}Secret123\r\n"), "Secret123");
        assert_eq!(prima_linie("Secret123\r"), "Secret123");
    }
}
