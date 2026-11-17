//! Export CSV al listei de rezultate.
//!
//! **Se strica astfel:** fisierul se deschide in Excel cu diacriticele stricate
//! sau cu totul pe o coloana. Ambele au cauza cunoscuta si fixa: Excel vrea BOM
//! UTF-8 si separator `;` in setarile romanesti. Portat identic din
//! `exporta_csv` din `mostenire/app.py`, care folosea `utf-8-sig` si `;`.

use std::io::Write;
use std::path::Path;

use crate::comparare::Rand;
use crate::eroare::{Eroare, Rezultat};

/// Byte Order Mark UTF-8. Fara el, Excel citeste fisierul ca ANSI si strica
/// diacriticele.
const BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

const CAP: [&str; 8] = [
    "Obiect",
    "Baza",
    "Tip",
    "Categorie",
    "Similaritate %",
    "Diferente",
    "Rezolvat",
    "Omis",
];

/// Un camp CSV: se pun ghilimele doar daca e nevoie, iar ghilimelele din
/// conținut se dubleaza.
///
/// Un camp care incepe cu `=`, `+`, `-`, `@`, tab sau CR e luat de Excel drept
/// formula (`=HYPERLINK(...)`, `@SUM(...)`); numele de baze si obiecte vin din
/// baze sau din instantanee, deci nu sunt de incredere. Primeste un apostrof
/// in fata, cum recomanda OWASP. Gasit la audit.
fn camp(v: &str) -> String {
    let neutralizat;
    let v = if v.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        neutralizat = format!("'{v}");
        neutralizat.as_str()
    } else {
        v
    };
    if v.contains(';') || v.contains('"') || v.contains('\n') || v.contains('\r') {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_string()
    }
}

/// Starea de lucru a utilizatorului pe un rand, care nu vine din comparare.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stare {
    pub rezolvat: bool,
    pub omis: bool,
}

pub fn scrie_csv(cale: &Path, randuri: &[Rand], stare: impl Fn(usize) -> Stare) -> Rezultat<usize> {
    if randuri.is_empty() {
        return Err(Eroare::Export {
            cale: cale.display().to_string(),
            detaliu: "nimic de exportat".into(),
        });
    }
    let mut f = std::fs::File::create(cale).map_err(|e| Eroare::Export {
        cale: cale.display().to_string(),
        detaliu: e.to_string(),
    })?;
    let mut scrie = |date: &[u8]| -> Rezultat<()> {
        f.write_all(date).map_err(|e| Eroare::Export {
            cale: cale.display().to_string(),
            detaliu: e.to_string(),
        })
    };

    scrie(BOM)?;
    scrie(CAP.join(";").as_bytes())?;
    scrie(b"\r\n")?;

    for r in randuri {
        let s = stare(r.id);
        let rand = [
            camp(&r.obiect),
            camp(&r.baza),
            camp(&r.tip),
            camp(r.categorie.eticheta()),
            // Virgula zecimala, ca Excel romanesc sa o citeasca drept numar.
            r.scor
                .map(|x| format!("{x:.2}").replace('.', ","))
                .unwrap_or_default(),
            camp(&r.rezumat),
            if s.rezolvat { "DA" } else { "NU" }.to_string(),
            if s.omis { "DA" } else { "NU" }.to_string(),
        ];
        scrie(rand.join(";").as_bytes())?;
        scrie(b"\r\n")?;
    }
    Ok(randuri.len())
}

#[cfg(test)]
mod teste {
    use super::*;
    use crate::comparare::Categorie;

    fn rand(id: usize, obiect: &str, scor: Option<f64>, rezumat: &str) -> Rand {
        Rand {
            id,
            obiect: obiect.into(),
            baza: "Client1".into(),
            scor,
            rezumat: rezumat.into(),
            tip: "P".into(),
            categorie: if scor.is_some() {
                Categorie::Diferit
            } else {
                Categorie::Lipsa
            },
            hash_referinta: None,
            hash_curent: None,
        }
    }

    fn scrie_in_temp(randuri: &[Rand]) -> (std::path::PathBuf, Vec<u8>) {
        let d = std::env::temp_dir().join(format!(
            "gmotion-csv-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&d).unwrap();
        let cale = d.join("export.csv");
        scrie_csv(&cale, randuri, |_| Stare::default()).unwrap();
        let brut = std::fs::read(&cale).unwrap();
        (d, brut)
    }

    #[test]
    fn are_bom_si_separator_punct_virgula() {
        let (d, brut) = scrie_in_temp(&[rand(0, "dbo.pA", Some(87.5), "3 linii modificate")]);
        assert!(
            brut.starts_with(BOM),
            "lipseste BOM-ul: Excel ar strica diacriticele"
        );
        let text = String::from_utf8(brut[3..].to_vec()).unwrap();
        assert!(text.starts_with("Obiect;Baza;Tip;"));
        assert!(text.contains("\r\n"), "Excel vrea CRLF");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn scorul_are_virgula_zecimala() {
        let (d, brut) = scrie_in_temp(&[rand(0, "dbo.pA", Some(87.5), "x")]);
        let text = String::from_utf8(brut[3..].to_vec()).unwrap();
        assert!(text.contains("87,50"), "scorul trebuie cu virgula: {text}");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn scor_lipsa_da_celula_goala() {
        let (d, brut) = scrie_in_temp(&[rand(0, "dbo.pA", None, "lipseste complet")]);
        let text = String::from_utf8(brut[3..].to_vec()).unwrap();
        let linie = text.lines().nth(1).unwrap();
        assert!(
            linie.contains(";;"),
            "celula de scor trebuie goala: {linie}"
        );
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn campurile_cu_separator_sau_ghilimele_sunt_protejate() {
        let (d, brut) = scrie_in_temp(&[
            rand(0, "dbo.p;cu;separator", Some(50.0), "rezumat \"citat\""),
            rand(1, "dbo.pNormal", Some(50.0), "linia 1\nlinia 2"),
        ]);
        let text = String::from_utf8(brut[3..].to_vec()).unwrap();
        assert!(text.contains("\"dbo.p;cu;separator\""));
        assert!(text.contains("\"rezumat \"\"citat\"\"\""));
        assert!(text.contains("\"linia 1\nlinia 2\""));
        std::fs::remove_dir_all(&d).ok();
    }

    /// Regresie din audit: `@SUM(1+1)` ajungea brut intr-o celula.
    #[test]
    fn formulele_nu_ajung_in_excel() {
        for v in ["=HYPERLINK(\"http://x\")", "@SUM(1+1)", "+1", "-2", "\tx"] {
            assert!(
                camp(v).trim_start_matches('"').starts_with('\''),
                "{v:?} -> {:?}",
                camp(v)
            );
        }
        assert_eq!(camp("dbo.pNormal"), "dbo.pNormal");
    }

    #[test]
    fn diacriticele_trec_intacte() {
        let (d, brut) = scrie_in_temp(&[rand(0, "dbo.pÎncărcare", Some(50.0), "ștersă")]);
        let text = String::from_utf8(brut[3..].to_vec()).unwrap();
        assert!(text.contains("dbo.pÎncărcare"));
        assert!(text.contains("ștersă"));
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn starea_utilizatorului_ajunge_in_fisier() {
        let d = std::env::temp_dir().join(format!("gmotion-csv-stare-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let cale = d.join("e.csv");
        let randuri = [
            rand(0, "dbo.pA", Some(1.0), "x"),
            rand(1, "dbo.pB", Some(2.0), "y"),
        ];
        scrie_csv(&cale, &randuri, |id| Stare {
            rezolvat: id == 0,
            omis: id == 1,
        })
        .unwrap();
        let text = String::from_utf8(std::fs::read(&cale).unwrap()[3..].to_vec()).unwrap();
        let linii: Vec<&str> = text.lines().collect();
        assert!(linii[1].ends_with("DA;NU"), "{}", linii[1]);
        assert!(linii[2].ends_with("NU;DA"), "{}", linii[2]);
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn export_gol_e_eroare() {
        let e = scrie_csv(std::path::Path::new("/tmp/nu-conteaza.csv"), &[], |_| {
            Stare::default()
        })
        .unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-005");
    }
}
