//! Ce arata ecranul de deploy, fara fereastra: propozitia de rezumat, pasii
//! numerotati, starea garzii si a scrierii.
//!
//! **Se strica astfel:** propozitia spune alt numar decat pasii de dedesubt,
//! garda pare trecuta cand nu e, sau butonul de scriere pare activ cand nu
//! se poate scrie. Decizia „se poate scrie" vine din `app` (unde e garda);
//! aici doar se spune pe ecran, fara sa o refaca.

use gmotion_core::Limba;

/// Un pas din plan, in ordinea din script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasDeploy {
    /// `true` = obiectul nu exista in baza; deploy-ul il creeaza.
    pub creeaza: bool,
    /// Baza in care se scrie.
    pub baza: String,
    /// `schema.nume`.
    pub obiect: String,
    /// Codul tipului (`P`, `V`...); numele pentru om il pune fereastra, in
    /// limba aleasa (`gmotion_core::nume_tip_in`).
    pub tip: String,
}

/// Ce a spus garda „baza n-a mai fost atinsa".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GardaDeploy {
    Neverificata,
    /// Recitirea ruleaza acum, pe alt fir. Pe o baza mare tine minute, si
    /// pana la auditul QA tinea fereastra blocata (P4).
    SeVerifica,
    Ok {
        la: String,
    },
    Schimbata(Vec<String>),
    Eroare(String),
}

/// Cum a iesit scrierea.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RezultatDeploy {
    Nescris,
    Scris {
        director: String,
        fisiere: usize,
        revenire: usize,
    },
    Eroare(String),
}

/// Tot ecranul de deploy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcranDeploy {
    pub referinta: String,
    /// Bazele tinta, scrise pentru om: „Client1" sau „3 baze".
    pub tinta: String,
    /// Numele fiecarei baze tinta; randurile le arata cand sunt mai multe.
    pub baze: Vec<String>,
    pub pasi: Vec<PasDeploy>,
    /// Alese dar nedeployabile (exista doar in baza sau sunt criptate).
    pub sarite: usize,
    pub garda: GardaDeploy,
    /// 0 un fisier per baza (implicit), 1 un singur fisier, 2 per obiect,
    /// 3 cate N obiecte.
    pub impartire: i32,
    /// Cate obiecte per fisier, la impartirea 3.
    pub cate: usize,
    pub poate_scrie: bool,
    /// Unde se scrie, inainte de scriere.
    pub director: String,
    pub rezultat: RezultatDeploy,
}

/// „3 obiecte: 2 modificate, 1 nou" — din pasi, ca sa nu poata minti.
pub fn rezumat(pasi: &[PasDeploy], l: Limba) -> String {
    if pasi.is_empty() {
        return l.alege("Planul e gol.", "The plan is empty.").to_string();
    }
    let noi = pasi.iter().filter(|p| p.creeaza).count();
    let modificate = pasi.len() - noi;
    let mut parti = Vec::new();
    if modificate > 0 {
        parti.push(l.numar(
            modificate,
            ("modificat", "modificate"),
            ("changed", "changed"),
        ));
    }
    if noi > 0 {
        parti.push(l.numar(noi, ("nou", "noi"), ("new", "new")));
    }
    format!(
        "{}: {}",
        l.numar(pasi.len(), ("obiect", "obiecte"), ("object", "objects")),
        parti.join(", ")
    )
}

/// Ce spune garda, intr-o propozitie.
pub fn text_garda(g: &GardaDeploy, tinta: &str, l: Limba) -> String {
    let en = l == Limba::Engleza;
    match g {
        GardaDeploy::Neverificata if en => format!(
            "Before writing, the objects in {tinta} are read again and compared with what you reviewed."
        ),
        GardaDeploy::Neverificata => format!(
            "Inainte de scriere se recitesc obiectele din {tinta} si se compara cu ce ai revizuit."
        ),
        GardaDeploy::SeVerifica => l
            .alege(
                "Se recitesc obiectele din baza, ca sa fie comparate cu ce ai revizuit...",
                "Reading the objects again, to compare them with what you reviewed...",
            )
            .to_string(),
        GardaDeploy::Ok { la } if en => {
            format!("Checked at {la}: the objects in the plan are exactly as you compared them.")
        }
        GardaDeploy::Ok { la } => {
            format!("Verificat la {la}: obiectele din plan sunt exact cum le-ai comparat.")
        }
        GardaDeploy::Schimbata(v) if en => format!(
            "{} since the comparison. Compare again and review again.",
            if v.len() == 1 {
                "One object has changed".to_string()
            } else {
                format!("{} objects have changed", v.len())
            }
        ),
        GardaDeploy::Schimbata(v) => format!(
            "{} de la comparare. Recompara si revizuieste din nou.",
            if v.len() == 1 {
                "Un obiect s-a schimbat".to_string()
            } else {
                format!("{} s-au schimbat", l.numar(v.len(), ("obiect", "obiecte"), ("", "")))
            }
        ),
        GardaDeploy::Eroare(e) => format!("{}: {e}", l.alege("Nu s-a putut verifica", "Could not check")),
    }
}

/// Ce spune rezultatul scrierii, intr-o propozitie.
pub fn text_rezultat(r: &RezultatDeploy, l: Limba) -> String {
    match r {
        RezultatDeploy::Nescris => String::new(),
        RezultatDeploy::Scris {
            director,
            fisiere,
            revenire,
        } => format!(
            "{} {director}. {}: {}.",
            l.numar(
                *fisiere,
                ("fisier scris in", "fisiere scrise in"),
                ("file written to", "files written to")
            ),
            l.alege("Revenirea, salvata inainte", "Rollback, saved first"),
            l.numar(*revenire, ("fisier", "fisiere"), ("file", "files"))
        ),
        RezultatDeploy::Eroare(e) => {
            format!(
                "{}: {e}",
                l.alege("Nu s-a scris nimic", "Nothing was written")
            )
        }
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    const R: Limba = Limba::Romana;

    fn pas(creeaza: bool, obiect: &str) -> PasDeploy {
        PasDeploy {
            creeaza,
            baza: "Client1".into(),
            obiect: obiect.into(),
            tip: "procedura".into(),
        }
    }

    #[test]
    fn rezumatul_numara_din_pasi() {
        assert_eq!(rezumat(&[], R), "Planul e gol.");
        assert_eq!(rezumat(&[pas(false, "dbo.a")], R), "1 obiect: 1 modificat");
        assert_eq!(rezumat(&[pas(true, "dbo.a")], R), "1 obiect: 1 nou");
        assert_eq!(
            rezumat(
                &[pas(false, "dbo.a"), pas(false, "dbo.b"), pas(true, "dbo.c")],
                R
            ),
            "3 obiecte: 2 modificate, 1 nou"
        );
        let multe: Vec<PasDeploy> = (0..21)
            .map(|i| pas(i % 3 == 0, &format!("dbo.p{i}")))
            .collect();
        assert_eq!(rezumat(&multe, R), "21 de obiecte: 14 modificate, 7 noi");
    }

    #[test]
    fn garda_spune_ce_s_a_intamplat() {
        assert!(text_garda(&GardaDeploy::Neverificata, "Prod", R).contains("Prod"));
        assert_eq!(
            text_garda(&GardaDeploy::Ok { la: "14:05".into() }, "Prod", R),
            "Verificat la 14:05: obiectele din plan sunt exact cum le-ai comparat."
        );
        assert!(
            text_garda(&GardaDeploy::Schimbata(vec!["x".into()]), "Prod", R)
                .starts_with("Un obiect")
        );
        assert!(text_garda(
            &GardaDeploy::Schimbata(vec!["x".into(), "y".into()]),
            "Prod",
            R
        )
        .starts_with("2 obiecte"));
        assert!(text_garda(&GardaDeploy::Eroare("timeout".into()), "Prod", R).contains("timeout"));
    }

    #[test]
    fn rezultatul_spune_unde_si_cate() {
        assert_eq!(text_rezultat(&RezultatDeploy::Nescris, R), "");
        assert_eq!(
            text_rezultat(
                &RezultatDeploy::Scris {
                    director: "C:\\deploy".into(),
                    fisiere: 1,
                    revenire: 1
                },
                R
            ),
            "1 fisier scris in C:\\deploy. Revenirea, salvata inainte: 1 fisier."
        );
        assert!(
            text_rezultat(&RezultatDeploy::Eroare("disc plin".into()), R).contains("disc plin")
        );
    }

    #[test]
    fn in_engleza_aceleasi_cifre() {
        let e = Limba::Engleza;
        assert_eq!(rezumat(&[], e), "The plan is empty.");
        assert_eq!(
            rezumat(
                &[pas(false, "dbo.a"), pas(true, "dbo.b"), pas(true, "dbo.c")],
                e
            ),
            "3 objects: 1 changed, 2 new"
        );
        assert!(
            text_garda(&GardaDeploy::Ok { la: "14:05".into() }, "Prod", e)
                .starts_with("Checked at 14:05")
        );
        assert_eq!(
            text_rezultat(
                &RezultatDeploy::Scris {
                    director: "C:\\deploy".into(),
                    fisiere: 2,
                    revenire: 1
                },
                e
            ),
            "2 files written to C:\\deploy. Rollback, saved first: 1 file."
        );
    }
}
