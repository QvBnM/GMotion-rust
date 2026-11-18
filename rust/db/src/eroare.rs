//! Erorile stratului de acces la date, cu coduri stabile.
//!
//! **Se strica astfel:** un utilizator raporteaza „nu merge" fara nimic de
//! care sa te prinzi. De-aia fiecare eroare are un cod (`GM-DB-003`) care apare
//! si in jurnal si in mesajul de pe ecran: din raportul utilizatorului ajungi
//! direct la varianta de aici. Un cod alocat nu se refoloseste si nu se
//! renumeroteaza — vezi registrul din `JURNAL.md`.

use std::fmt;

/// Rezultatul oricarei operatii de acces la date.
pub type Rezultat<T> = Result<T, Eroare>;

#[derive(Debug, thiserror::Error)]
pub enum Eroare {
    #[error("[GM-DB-001] nu pot deschide conexiunea catre {server}: {detaliu}")]
    Conectare { server: String, detaliu: String },

    #[error("[GM-DB-002] autentificarea a fost refuzata pe {server}: {detaliu}")]
    Autentificare { server: String, detaliu: String },

    #[error("[GM-DB-003] autentificarea integrata Windows nu e disponibila pe acest sistem")]
    AuthIntegrataIndisponibila,

    #[error("[GM-DB-004] interogarea a esuat pe baza {baza}: {detaliu}")]
    Interogare { baza: String, detaliu: String },

    #[error("[GM-DB-005] baza {baza} nu exista pe {server} sau nu e accesibila")]
    BazaLipsa { server: String, baza: String },

    #[error("[GM-DB-006] descoperirea instantelor a esuat: {detaliu}")]
    Descoperire { detaliu: String },

    #[error("[GM-DB-007] instantaneul {cale} nu poate fi citit: {detaliu}")]
    CitireInstantaneu { cale: String, detaliu: String },

    #[error("[GM-DB-008] instantaneul {cale} nu poate fi scris: {detaliu}")]
    ScriereInstantaneu { cale: String, detaliu: String },

    #[error("[GM-DB-009] instantaneul {cale} are versiunea {gasita}, dar aceasta versiune de GMotion citeste {asteptata}")]
    VersiuneInstantaneu {
        cale: String,
        gasita: u32,
        asteptata: u32,
    },

    #[error("[GM-DB-010] nume invalid: {0:?}. Sunt permise doar litere, cifre, _ , - , . si \\")]
    NumeInvalid(String),

    #[error("[GM-DB-011] numele de baza {0:?} e gol, prea lung (peste 128 de caractere) sau contine caractere de control")]
    BazaInvalida(String),
}

impl Eroare {
    /// Codul stabil, fara paranteze drepte. Pentru jurnal si pentru rapoarte.
    pub fn cod(&self) -> &'static str {
        match self {
            Eroare::Conectare { .. } => "GM-DB-001",
            Eroare::Autentificare { .. } => "GM-DB-002",
            Eroare::AuthIntegrataIndisponibila => "GM-DB-003",
            Eroare::Interogare { .. } => "GM-DB-004",
            Eroare::BazaLipsa { .. } => "GM-DB-005",
            Eroare::Descoperire { .. } => "GM-DB-006",
            Eroare::CitireInstantaneu { .. } => "GM-DB-007",
            Eroare::ScriereInstantaneu { .. } => "GM-DB-008",
            Eroare::VersiuneInstantaneu { .. } => "GM-DB-009",
            Eroare::NumeInvalid(_) => "GM-DB-010",
            Eroare::BazaInvalida(_) => "GM-DB-011",
        }
    }

    /// Mai are rost sa reincerci? Un timeout de retea, da; o parola gresita, nu.
    pub fn merita_reincercat(&self) -> bool {
        matches!(self, Eroare::Conectare { .. } | Eroare::Descoperire { .. })
    }
}

/// Numele de **server**: ajunge in campurile de configurare si in rezolvarea
/// DNS, deci forma e stricta.
///
/// Permis: litere, cifre, `_`, `-`, `.`, `\` (instanta), `,` (port). Orice
/// altceva — ghilimele, `;`, spatiu, `--` — e refuzat.
pub fn valideaza_nume(n: &str) -> Rezultat<()> {
    if n.is_empty() || n.len() > 256 {
        return Err(Eroare::NumeInvalid(n.to_string()));
    }
    let ok = n
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '\\' | ','));
    if ok {
        Ok(())
    } else {
        Err(Eroare::NumeInvalid(n.to_string()))
    }
}

/// Numele unei **baze de date**: ajunge doar in campul de login TDS (UTF-16),
/// niciodata in text SQL, deci orice nume valid in SQL Server e acceptat —
/// spatii, diacritice, `]`. Pana la audit trecea prin regula de server, deci
/// `Contabilitate 2024` era refuzat si oprea o comparatie pe tot serverul.
/// Refuzate raman doar caracterele de control (nu exista in nume reale si ar
/// strica jurnalele si scripturile) si numele peste limita SQL Server.
pub fn valideaza_baza(n: &str) -> Rezultat<()> {
    if n.is_empty() || n.chars().count() > 128 || n.chars().any(char::is_control) {
        return Err(Eroare::BazaInvalida(n.to_string()));
    }
    Ok(())
}

/// Pentru jurnal: niciodata parola, nici lungimea ei, niciodata textul de
/// conexiune intreg. Spune doar daca exista — atat ajuta la depanare. Lungimea
/// a ajuns o data intr-un instantaneu commitat; vezi `JURNAL.md`.
pub(crate) struct Redactat<'a>(pub &'a str);

impl fmt::Display for Redactat<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "(goala)")
        } else {
            write!(f, "(data, ascunsa)")
        }
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn nume_valide() {
        for n in [
            "SRV01",
            "SRV01\\SQLEXPRESS",
            "srv.domeniu.local,1433",
            "Baza_1-test",
        ] {
            assert!(valideaza_nume(n).is_ok(), "{n} ar trebui valid");
        }
    }

    #[test]
    fn baze_cu_nume_obisnuite_sunt_acceptate() {
        let nume = ["Contabilitate 2024", "My DB", "Facturare_ț", "X]y", "a'b"]; // diacritice-permise: date de test
        for n in nume {
            assert!(valideaza_baza(n).is_ok(), "{n:?} ar trebui acceptat");
        }
        for n in ["", "a\nb", "a\u{1b}[2J", &"x".repeat(129)] {
            assert_eq!(valideaza_baza(n).unwrap_err().cod(), "GM-DB-011", "{n:?}");
        }
    }

    #[test]
    fn nume_invalide() {
        for n in ["", "a b", "x;DROP", "y'--", "baza\"x", "a\nb"] {
            assert!(valideaza_nume(n).is_err(), "{n:?} ar trebui invalid");
        }
    }

    #[test]
    fn parola_nu_apare_in_jurnal() {
        let t = format!("{}", Redactat("parola-secreta"));
        assert!(!t.contains("secreta"));
        assert!(!t.contains("14"), "nici lungimea nu se spune: {t}");
        assert!(!t.chars().any(|c| c.is_ascii_digit()), "{t}");
        assert_ne!(
            t,
            format!("{}", Redactat("")),
            "goala se deosebeste de data"
        );
    }

    #[test]
    fn fiecare_eroare_are_cod_unic() {
        let toate = [
            Eroare::Conectare {
                server: "s".into(),
                detaliu: "d".into(),
            },
            Eroare::Autentificare {
                server: "s".into(),
                detaliu: "d".into(),
            },
            Eroare::AuthIntegrataIndisponibila,
            Eroare::Interogare {
                baza: "b".into(),
                detaliu: "d".into(),
            },
            Eroare::BazaLipsa {
                server: "s".into(),
                baza: "b".into(),
            },
            Eroare::Descoperire {
                detaliu: "d".into(),
            },
            Eroare::CitireInstantaneu {
                cale: "c".into(),
                detaliu: "d".into(),
            },
            Eroare::ScriereInstantaneu {
                cale: "c".into(),
                detaliu: "d".into(),
            },
            Eroare::VersiuneInstantaneu {
                cale: "c".into(),
                gasita: 1,
                asteptata: 2,
            },
            Eroare::NumeInvalid("n".into()),
            Eroare::BazaInvalida("b".into()),
        ];
        let mut coduri: Vec<&str> = toate.iter().map(|e| e.cod()).collect();
        coduri.sort_unstable();
        let n = coduri.len();
        coduri.dedup();
        assert_eq!(coduri.len(), n, "coduri duplicate");
        // codul din mesaj trebuie sa fie acelasi cu cel din cod()
        for e in &toate {
            assert!(
                e.to_string().contains(e.cod()),
                "mesajul lui {} nu contine codul",
                e.cod()
            );
        }
    }
}
