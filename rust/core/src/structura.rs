//! Verificarea structurii unui text T-SQL, fara server: ce nu se inchide.
//!
//! **Se strica astfel:** spune „nu se compileaza" despre un text bun (fals
//! pozitiv: `BEGIN TRAN` luat drept bloc, un `END` dintr-un sir), sau tace
//! despre unul stricat. Nu e un parser: prinde doar ce strica o plasare
//! gresita de bloc — `BEGIN`/`END`, `TRY`/`CATCH`, `CASE`, paranteze, siruri
//! si comentarii neinchise, `GO` in mijlocul obiectului.

use crate::lexer::{segmente, Fel};

/// O problema de structura, cu linia ei (de la 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problema {
    pub linie: usize,
    pub fel: FelProblema,
}

/// Ce nu se inchide. Textul pentru om il face interfata, in limba ei.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FelProblema {
    /// Un `BEGIN` (sau `BEGIN TRY` / `BEGIN CATCH` / `CASE`) fara `END`-ul lui.
    DeschisNeinchis(Bloc),
    /// Un `END` (sau `END TRY` / `END CATCH`) fara deschidere.
    InchisFaraDeschidere(Bloc),
    /// `END TRY` cand era deschis altceva (de exemplu un `BEGIN` simplu).
    InchisAltceva { asteptat: Bloc, gasit: Bloc },
    /// O paranteza `(` fara `)`.
    ParantezaNeinchisa,
    /// O `)` fara `(`.
    ParantezaInPlus,
    /// Un sir `'...'` care nu se inchide.
    SirNeinchis,
    /// Un identificator `[...]` sau `"..."` care nu se inchide.
    IdentificatorNeinchis,
    /// Un comentariu `/* ... */` care nu se inchide.
    ComentariuNeinchis,
    /// O linie `GO`: in mijlocul unui obiect, taie scriptul in doua.
    Go,
}

/// Felul unui bloc deschis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bloc {
    Begin,
    Try,
    Catch,
    Case,
}

impl Bloc {
    /// Cum se scrie in cod, pentru mesaje: `BEGIN`, `BEGIN TRY`...
    pub fn cod(self) -> &'static str {
        match self {
            Bloc::Begin => "BEGIN",
            Bloc::Try => "BEGIN TRY",
            Bloc::Catch => "BEGIN CATCH",
            Bloc::Case => "CASE",
        }
    }

    /// Cum se inchide: `END`, `END TRY`...
    pub fn inchidere(self) -> &'static str {
        match self {
            Bloc::Begin | Bloc::Case => "END",
            Bloc::Try => "END TRY",
            Bloc::Catch => "END CATCH",
        }
    }
}

impl core::fmt::Display for Problema {
    /// In romana, pentru `cli` si jurnal; fereastra are textele ei, in limba
    /// aleasa.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "linia {}: ", self.linie)?;
        match &self.fel {
            FelProblema::DeschisNeinchis(b) => write!(f, "{} fara {}", b.cod(), b.inchidere()),
            FelProblema::InchisFaraDeschidere(b) => {
                write!(f, "{} in plus, fara deschidere", b.inchidere())
            }
            FelProblema::InchisAltceva { asteptat, gasit } => {
                write!(f, "{} inchide un {}", gasit.inchidere(), asteptat.cod())
            }
            FelProblema::ParantezaNeinchisa => write!(f, "o paranteza ( nu se inchide"),
            FelProblema::ParantezaInPlus => write!(f, "o paranteza ) in plus"),
            FelProblema::SirNeinchis => write!(f, "un sir '...' nu se inchide"),
            FelProblema::IdentificatorNeinchis => write!(f, "un nume [...] nu se inchide"),
            FelProblema::ComentariuNeinchis => write!(f, "un comentariu /* nu se inchide"),
            FelProblema::Go => write!(f, "GO in mijlocul obiectului"),
        }
    }
}

/// Problemele de structura ale textului, in ordinea liniilor. Gol = nimic
/// gasit (ceea ce nu inseamna ca serverul il accepta).
pub fn verifica(text: &str) -> Vec<Problema> {
    let linie_la = |poz: usize| text[..poz.min(text.len())].matches('\n').count() + 1;
    let mut probleme = Vec::new();

    // cuvintele si parantezele din cod, cu pozitia lor; ce e in siruri si
    // comentarii nu conteaza
    let mut jetoane: Vec<(usize, String)> = Vec::new();
    for s in segmente(text) {
        let bucata = &text[s.start..s.sfarsit];
        match s.fel {
            Fel::Cod => {
                let mut start: Option<usize> = None;
                for (i, c) in bucata
                    .char_indices()
                    .chain(std::iter::once((bucata.len(), ' ')))
                {
                    let e_cuvant = c.is_alphanumeric() || matches!(c, '_' | '@' | '#' | '$');
                    match (start, e_cuvant) {
                        (None, true) => start = Some(i),
                        (Some(a), false) => {
                            jetoane.push((s.start + a, bucata[a..i].to_ascii_uppercase()));
                            start = None;
                        }
                        _ => {}
                    }
                    if c == '(' || c == ')' || c == ';' {
                        jetoane.push((s.start + i, c.to_string()));
                    }
                }
            }
            Fel::Sir if !inchis(bucata, '\'', '\'') => probleme.push(Problema {
                linie: linie_la(s.start),
                fel: FelProblema::SirNeinchis,
            }),
            Fel::IdentificatorParanteze if !inchis(bucata, '[', ']') => probleme.push(Problema {
                linie: linie_la(s.start),
                fel: FelProblema::IdentificatorNeinchis,
            }),
            Fel::IdentificatorGhilimele if !inchis(bucata, '"', '"') => probleme.push(Problema {
                linie: linie_la(s.start),
                fel: FelProblema::IdentificatorNeinchis,
            }),
            Fel::ComentariuBloc if !(bucata.len() >= 4 && bucata.ends_with("*/")) => {
                probleme.push(Problema {
                    linie: linie_la(s.start),
                    fel: FelProblema::ComentariuNeinchis,
                })
            }
            _ => {}
        }
    }

    // blocurile si parantezele, cu o stiva
    let mut blocuri: Vec<(Bloc, usize)> = Vec::new();
    let mut paranteze: Vec<usize> = Vec::new();
    let urmatorul = |i: usize| jetoane.get(i + 1).map(|(_, c)| c.as_str());
    for (i, (poz, c)) in jetoane.iter().enumerate() {
        match c.as_str() {
            "(" => paranteze.push(*poz),
            ")" => {
                if paranteze.pop().is_none() {
                    probleme.push(Problema {
                        linie: linie_la(*poz),
                        fel: FelProblema::ParantezaInPlus,
                    });
                }
            }
            "BEGIN" => match urmatorul(i) {
                // instructiuni, nu blocuri
                Some("TRAN" | "TRANSACTION" | "DISTRIBUTED" | "DIALOG" | "CONVERSATION") => {}
                Some("TRY") => blocuri.push((Bloc::Try, *poz)),
                Some("CATCH") => blocuri.push((Bloc::Catch, *poz)),
                _ => blocuri.push((Bloc::Begin, *poz)),
            },
            "CASE" => blocuri.push((Bloc::Case, *poz)),
            "END" => {
                let gasit = match urmatorul(i) {
                    // `END CONVERSATION` e o instructiune
                    Some("CONVERSATION") => continue,
                    Some("TRY") => Bloc::Try,
                    Some("CATCH") => Bloc::Catch,
                    _ => Bloc::Begin,
                };
                match blocuri.pop() {
                    None => probleme.push(Problema {
                        linie: linie_la(*poz),
                        fel: FelProblema::InchisFaraDeschidere(gasit),
                    }),
                    Some((deschis, linie_deschis)) => {
                        let potrivit = match gasit {
                            // un `END` simplu inchide un `BEGIN` sau un `CASE`
                            Bloc::Begin => matches!(deschis, Bloc::Begin | Bloc::Case),
                            alt => deschis == alt,
                        };
                        if !potrivit {
                            probleme.push(Problema {
                                linie: linie_la(*poz),
                                fel: FelProblema::InchisAltceva {
                                    asteptat: deschis,
                                    gasit,
                                },
                            });
                            let _ = linie_deschis;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    for (bloc, poz) in blocuri {
        probleme.push(Problema {
            linie: linie_la(poz),
            fel: FelProblema::DeschisNeinchis(bloc),
        });
    }
    for poz in paranteze {
        probleme.push(Problema {
            linie: linie_la(poz),
            fel: FelProblema::ParantezaNeinchisa,
        });
    }
    // Un `GO` pe **ultima** linie nu taie obiectul in doua: dupa el nu mai
    // vine nimic. Patru obiecte reale din `referinta/instantanee/` il au, dupa
    // `END`, si apareau ca probleme de structura (COSMETIC, auditul
    // 2026-11-23).
    let ultima_cu_cod = text
        .split('\n')
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, _)| i + 1)
        .last()
        .unwrap_or(0);
    for linie in crate::lexer::linii_go(text) {
        if linie >= ultima_cu_cod {
            continue;
        }
        probleme.push(Problema {
            linie,
            fel: FelProblema::Go,
        });
    }
    probleme.sort_by_key(|p| p.linie);
    probleme
}

/// Un sir sau identificator e inchis daca se termina cu delimitatorul de
/// inchidere dupa cel de deschidere. `'abc'` da, `'abc` nu; `'` singur nu.
fn inchis(bucata: &str, deschide: char, inchide: char) -> bool {
    // prefixul `N` al sirurilor unicode ramane in cod, deci bucata incepe cu
    // delimitatorul
    let mut c = bucata.chars();
    c.next() == Some(deschide) && bucata.len() >= 2 && bucata.ends_with(inchide)
}

#[cfg(test)]
mod teste {
    use super::*;

    fn feluri(text: &str) -> Vec<(usize, FelProblema)> {
        verifica(text)
            .into_iter()
            .map(|p| (p.linie, p.fel))
            .collect()
    }

    #[test]
    fn o_procedura_buna_nu_are_probleme() {
        let t = "CREATE PROCEDURE dbo.p AS\nBEGIN\n  BEGIN TRAN\n  IF @a = 1\n  BEGIN\n    SELECT CASE WHEN x = 1 THEN 'END' ELSE 'BEGIN' END FROM t -- END\n  END\n  BEGIN TRY\n    SELECT (1 + (2))\n  END TRY\n  BEGIN CATCH\n    ROLLBACK\n  END CATCH\n  /* BEGIN */\n  COMMIT\n  END CONVERSATION @h\nEND";
        assert_eq!(feluri(t), []);
    }

    #[test]
    fn un_begin_fara_end_si_un_end_in_plus() {
        assert_eq!(
            feluri("CREATE PROC p AS\nBEGIN\n  IF 1 = 1\n  BEGIN\n    SELECT 1\nEND"),
            [(2, FelProblema::DeschisNeinchis(Bloc::Begin))]
        );
        assert_eq!(
            feluri("CREATE PROC p AS\nBEGIN\n  SELECT 1\nEND\nEND"),
            [(5, FelProblema::InchisFaraDeschidere(Bloc::Begin))]
        );
    }

    #[test]
    fn try_inchis_cu_end_simplu_e_alt_bloc() {
        assert_eq!(
            feluri("BEGIN TRY\n  SELECT 1\nEND\nBEGIN CATCH\nEND CATCH"),
            [(
                3,
                FelProblema::InchisAltceva {
                    asteptat: Bloc::Try,
                    gasit: Bloc::Begin
                }
            )]
        );
    }

    #[test]
    fn paranteze_siruri_comentarii_si_go() {
        assert_eq!(feluri("SELECT (1"), [(1, FelProblema::ParantezaNeinchisa)]);
        assert_eq!(feluri("SELECT 1)"), [(1, FelProblema::ParantezaInPlus)]);
        assert_eq!(
            feluri("SELECT 'abc\nFROM t"),
            [(1, FelProblema::SirNeinchis)]
        );
        assert_eq!(feluri("SELECT N'ok', 'x'"), []);
        assert_eq!(
            feluri("SELECT [a\nFROM t"),
            [(1, FelProblema::IdentificatorNeinchis)]
        );
        assert_eq!(
            feluri("SELECT 1 /* nu se\ninchide"),
            [(1, FelProblema::ComentariuNeinchis)]
        );
        assert_eq!(feluri("SELECT 1\nGO\nSELECT 2"), [(2, FelProblema::Go)]);
        // `GO` pe ultima linie nu taie nimic: dupa el nu mai vine cod
        assert_eq!(feluri("SELECT 1\nGO"), []);
        assert_eq!(feluri("SELECT 1\nGO\n"), []);
        assert_eq!(feluri("SELECT 1\nGO\n   \n"), []);
    }

    #[test]
    fn un_bloc_pus_in_mijlocul_unui_if_se_vede() {
        // cazul din sesizare: instructiuni puse intre `IF ... BEGIN` si corpul
        // lui, cu `END`-ul blocului adus si el
        let stricat = "CREATE PROC p AS\nBEGIN\n  IF @a = 1\n  BEGIN\n    SELECT 1\n  END\n  END\n  SELECT 2\nEND";
        assert_eq!(
            feluri(stricat),
            [(9, FelProblema::InchisFaraDeschidere(Bloc::Begin))]
        );
    }

    #[test]
    fn problema_se_scrie_cu_linia() {
        let p = verifica("BEGIN\nSELECT 1");
        assert_eq!(p[0].to_string(), "linia 1: BEGIN fara END");
    }

    #[test]
    fn textul_blocurilor_pentru_mesaje() {
        assert_eq!(Bloc::Try.cod(), "BEGIN TRY");
        assert_eq!(Bloc::Catch.inchidere(), "END CATCH");
        assert_eq!(Bloc::Case.inchidere(), "END");
    }
}
