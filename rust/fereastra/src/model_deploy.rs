//! Ce arata ecranul de deploy, fara fereastra: propozitia de rezumat, pasii
//! numerotati, starea garzii si a scrierii.
//!
//! **Se strica astfel:** propozitia spune alt numar decat pasii de dedesubt,
//! garda pare trecuta cand nu e, sau butonul de scriere pare activ cand nu
//! se poate scrie. Decizia „se poate scrie" vine din `app` (unde e garda);
//! aici doar se spune pe ecran, fara sa o refaca.

/// Un pas din plan, in ordinea din script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasDeploy {
    /// `true` = obiectul nu exista in baza; deploy-ul il creeaza.
    pub creeaza: bool,
    /// `schema.nume`.
    pub obiect: String,
    /// Numele tipului pentru utilizator („procedura").
    pub tip: String,
}

/// Ce a spus garda „baza n-a mai fost atinsa".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GardaDeploy {
    Neverificata,
    Ok { la: String },
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
    pub tinta: String,
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

fn plural(n: usize, unu: &str, multe: &str) -> String {
    match n {
        1 => format!("1 {unu}"),
        2..=19 => format!("{n} {multe}"),
        _ => format!("{n} de {multe}"),
    }
}

/// „3 obiecte: 2 modificate, 1 nou" — din pasi, ca sa nu poata minti.
pub fn rezumat(pasi: &[PasDeploy]) -> String {
    let noi = pasi.iter().filter(|p| p.creeaza).count();
    let modificate = pasi.len() - noi;
    let mut parti = Vec::new();
    if modificate > 0 {
        parti.push(if modificate == 1 {
            "1 modificat".to_string()
        } else {
            plural(modificate, "modificat", "modificate")
        });
    }
    if noi > 0 {
        parti.push(if noi == 1 {
            "1 nou".to_string()
        } else {
            plural(noi, "nou", "noi")
        });
    }
    if pasi.is_empty() {
        return "Planul e gol.".to_string();
    }
    format!(
        "{}: {}",
        plural(pasi.len(), "obiect", "obiecte"),
        parti.join(", ")
    )
}

/// Ce spune garda, intr-o propozitie.
pub fn text_garda(g: &GardaDeploy, tinta: &str) -> String {
    match g {
        GardaDeploy::Neverificata => {
            format!("Inainte de scriere se recitesc obiectele din {tinta} si se compara cu ce ai revizuit.")
        }
        GardaDeploy::Ok { la } => {
            format!("Verificat la {la}: obiectele din plan sunt exact cum le-ai comparat.")
        }
        GardaDeploy::Schimbata(v) => format!(
            "{} de la comparare. Recompara si revizuieste din nou.",
            if v.len() == 1 {
                "Un obiect s-a schimbat".to_string()
            } else {
                format!("{} obiecte s-au schimbat", v.len())
            }
        ),
        GardaDeploy::Eroare(e) => format!("Nu s-a putut verifica: {e}"),
    }
}

/// Ce spune rezultatul scrierii, intr-o propozitie.
pub fn text_rezultat(r: &RezultatDeploy) -> String {
    match r {
        RezultatDeploy::Nescris => String::new(),
        RezultatDeploy::Scris {
            director,
            fisiere,
            revenire,
        } => format!(
            "{} in {director}. Revenirea, salvata inainte: {}.",
            plural(*fisiere, "fisier scris", "fisiere scrise"),
            plural(*revenire, "fisier", "fisiere")
        ),
        RezultatDeploy::Eroare(e) => format!("Nu s-a scris nimic: {e}"),
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    fn pas(creeaza: bool, obiect: &str) -> PasDeploy {
        PasDeploy {
            creeaza,
            obiect: obiect.into(),
            tip: "procedura".into(),
        }
    }

    #[test]
    fn rezumatul_numara_din_pasi() {
        assert_eq!(rezumat(&[]), "Planul e gol.");
        assert_eq!(rezumat(&[pas(false, "dbo.a")]), "1 obiect: 1 modificat");
        assert_eq!(rezumat(&[pas(true, "dbo.a")]), "1 obiect: 1 nou");
        assert_eq!(
            rezumat(&[pas(false, "dbo.a"), pas(false, "dbo.b"), pas(true, "dbo.c")]),
            "3 obiecte: 2 modificate, 1 nou"
        );
        let multe: Vec<PasDeploy> = (0..21)
            .map(|i| pas(i % 3 == 0, &format!("dbo.p{i}")))
            .collect();
        assert_eq!(rezumat(&multe), "21 de obiecte: 14 modificate, 7 noi");
    }

    #[test]
    fn garda_spune_ce_s_a_intamplat() {
        assert!(text_garda(&GardaDeploy::Neverificata, "Prod").contains("Prod"));
        assert_eq!(
            text_garda(&GardaDeploy::Ok { la: "14:05".into() }, "Prod"),
            "Verificat la 14:05: obiectele din plan sunt exact cum le-ai comparat."
        );
        assert!(
            text_garda(&GardaDeploy::Schimbata(vec!["x".into()]), "Prod").starts_with("Un obiect")
        );
        assert!(text_garda(
            &GardaDeploy::Schimbata(vec!["x".into(), "y".into()]),
            "Prod"
        )
        .starts_with("2 obiecte"));
        assert!(text_garda(&GardaDeploy::Eroare("timeout".into()), "Prod").contains("timeout"));
    }

    #[test]
    fn rezultatul_spune_unde_si_cate() {
        assert_eq!(text_rezultat(&RezultatDeploy::Nescris), "");
        assert_eq!(
            text_rezultat(&RezultatDeploy::Scris {
                director: "C:\\deploy".into(),
                fisiere: 1,
                revenire: 1
            }),
            "1 fisier scris in C:\\deploy. Revenirea, salvata inainte: 1 fisier."
        );
        assert!(text_rezultat(&RezultatDeploy::Eroare("disc plin".into())).contains("disc plin"));
    }
}
