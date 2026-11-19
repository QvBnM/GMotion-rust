//! Arborele din lista, ca Object Explorer din SSMS: baza › dosar pe tip ›
//! obiect, aplatizat in randurile pe care le deseneaza fereastra.
//! Categoria de bug: obiect care lipseste din arbore sau apare in alt dosar,
//! numar gresit pe dosar, nod care se deschide / inchide singur la redesenare.
use std::collections::BTreeSet;

use crate::model_lista::Lista;
use crate::texte::Limba;

/// Dosarele de sub o baza, in ordinea in care se arata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dosar {
    Proceduri,
    Functii,
    Viewuri,
    Triggere,
    Altele,
}

impl Dosar {
    /// Dosarul unui cod de tip din `sys.objects` (`P`, `FN`, `V`...).
    pub fn din_tip(tip: &str) -> Dosar {
        match tip.trim() {
            "P" | "PC" | "X" | "RF" => Dosar::Proceduri,
            "FN" | "IF" | "TF" | "FS" | "FT" | "AF" => Dosar::Functii,
            "V" => Dosar::Viewuri,
            "TR" | "TA" => Dosar::Triggere,
            _ => Dosar::Altele,
        }
    }

    pub fn nume(self, l: Limba) -> &'static str {
        match self {
            Dosar::Proceduri => l.alege("Proceduri", "Stored Procedures"),
            Dosar::Functii => l.alege("Functii", "Functions"),
            Dosar::Viewuri => l.alege("View-uri", "Views"),
            Dosar::Triggere => l.alege("Triggere", "Triggers"),
            Dosar::Altele => l.alege("Altele", "Other"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fel {
    Baza,
    Dosar,
    Obiect,
}

/// Un rand din arbore.
#[derive(Debug, Clone, PartialEq)]
pub struct Nod {
    /// 0 baza, 1 dosar, 2 obiect.
    pub nivel: u8,
    pub fel: Fel,
    /// Identitatea nodului, stabila intre redesenari (`b:Baza`, `d:Baza/1`,
    /// `o:Baza/schema.nume`).
    pub cheie: String,
    /// Numele bazei sau al dosarului; la obiect, `schema.nume`.
    pub eticheta: String,
    /// Cate obiecte vizibile are dedesubt; 0 la obiect.
    pub numar: usize,
    pub deschis: bool,
    /// La obiect: pozitia lui in lista vizibila.
    pub pozitie: Option<usize>,
}

/// Un dosar si pozitiile (in lista vizibila) obiectelor din el.
type DosarCuPozitii = (Dosar, Vec<usize>);

/// Ce noduri sunt inchise. Implicit totul e deschis, ca la o comparare
/// noua sa se vada obiectele fara niciun clic.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Arbore {
    inchise: BTreeSet<String>,
}

fn cheie_baza(baza: &str) -> String {
    format!("b:{baza}")
}

fn cheie_dosar(baza: &str, d: Dosar) -> String {
    format!("d:{baza}/{}", d as u8)
}

impl Arbore {
    /// Deschide un nod inchis sau il inchide pe unul deschis.
    pub fn comuta(&mut self, cheie: &str) {
        if !self.inchise.remove(cheie) {
            self.inchise.insert(cheie.to_string());
        }
    }

    pub fn deschis(&self, cheie: &str) -> bool {
        !self.inchise.contains(cheie)
    }

    /// Deschide baza si dosarul obiectului de pe `pozitie` (selectat cu
    /// `j` / `k` sau din alt ecran): selectia nu sta niciodata ascunsa.
    pub fn deschide_calea(&mut self, lista: &Lista, pozitie: usize) {
        if let Some(o) = lista.obiect(pozitie) {
            self.inchise.remove(&cheie_baza(&o.baza));
            self.inchise
                .remove(&cheie_dosar(&o.baza, Dosar::din_tip(&o.tip)));
        }
    }

    /// Randurile arborelui pentru obiectele vizibile ale listei (filtrele si
    /// cautarea se aplica deci si aici), in ordinea listei in fiecare dosar.
    pub fn noduri(&self, lista: &Lista, l: Limba) -> Vec<Nod> {
        // baza -> dosar -> pozitii, bazele in ordinea primei aparitii
        let mut baze: Vec<(String, Vec<DosarCuPozitii>)> = Vec::new();
        for (p, &i) in lista.vizibile().iter().enumerate() {
            let o = &lista.obiecte()[i];
            let b = match baze.iter().position(|(n, _)| *n == o.baza) {
                Some(b) => b,
                None => {
                    baze.push((o.baza.clone(), Vec::new()));
                    baze.len() - 1
                }
            };
            let d = Dosar::din_tip(&o.tip);
            let dosare = &mut baze[b].1;
            match dosare.iter_mut().find(|(x, _)| *x == d) {
                Some((_, v)) => v.push(p),
                None => dosare.push((d, vec![p])),
            }
        }
        let mut v = Vec::new();
        for (baza, mut dosare) in baze {
            dosare.sort_by_key(|(d, _)| *d);
            let cb = cheie_baza(&baza);
            let deschisa = self.deschis(&cb);
            v.push(Nod {
                nivel: 0,
                fel: Fel::Baza,
                eticheta: baza.clone(),
                numar: dosare.iter().map(|(_, p)| p.len()).sum(),
                deschis: deschisa,
                cheie: cb,
                pozitie: None,
            });
            if !deschisa {
                continue;
            }
            for (d, pozitii) in dosare {
                let cd = cheie_dosar(&baza, d);
                let deschis = self.deschis(&cd);
                v.push(Nod {
                    nivel: 1,
                    fel: Fel::Dosar,
                    eticheta: d.nume(l).to_string(),
                    numar: pozitii.len(),
                    deschis,
                    cheie: cd,
                    pozitie: None,
                });
                if !deschis {
                    continue;
                }
                for p in pozitii {
                    let Some(o) = lista.obiect(p) else { continue };
                    v.push(Nod {
                        nivel: 2,
                        fel: Fel::Obiect,
                        eticheta: o.id(),
                        numar: 0,
                        deschis: false,
                        cheie: format!("o:{}/{}", o.baza, o.id()),
                        pozitie: Some(p),
                    });
                }
            }
        }
        v
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use crate::model_lista::{Obiect, Stare};

    fn o(baza: &str, nume: &str, tip: &str) -> Obiect {
        Obiect {
            schema: "dbo".into(),
            nume: nume.into(),
            baza: baza.into(),
            tip: tip.into(),
            stare: Stare::Modificat,
            detalii: None,
            revizuit: false,
            in_plan: false,
            omis: false,
            scor: None,
            deploy: Default::default(),
        }
    }

    fn lista() -> Lista {
        Lista::noua(vec![
            o("B1", "vw_a", "V"),
            o("B1", "usp_a", "P"),
            o("B1", "fn_a", "FN"),
            o("B1", "usp_b", "P"),
            o("B2", "usp_c", "P"),
            o("B2", "trg_a", "TR"),
        ])
    }

    fn rezumat(n: &[Nod]) -> Vec<String> {
        n.iter()
            .map(|x| {
                format!(
                    "{}{} {}",
                    "  ".repeat(x.nivel as usize),
                    x.eticheta,
                    x.numar
                )
            })
            .collect()
    }

    #[test]
    fn baza_dosar_obiect_cu_numere() {
        let n = Arbore::default().noduri(&lista(), Limba::Romana);
        let r = rezumat(&n);
        assert_eq!(r[0], "B1 4");
        assert_eq!(r[1], "  Proceduri 2");
        assert!(r[2].starts_with("    dbo.usp_"), "{r:?}");
        assert!(r.contains(&"  Functii 1".to_string()), "{r:?}");
        assert!(r.contains(&"  View-uri 1".to_string()), "{r:?}");
        assert!(r.contains(&"B2 2".to_string()), "{r:?}");
        assert!(r.contains(&"  Triggere 1".to_string()), "{r:?}");
        // 2 baze + 5 dosare + 6 obiecte
        assert_eq!(n.len(), 13);
        // fiecare obiect apare o data, cu pozitia lui din lista vizibila
        let mut p: Vec<usize> = n.iter().filter_map(|x| x.pozitie).collect();
        p.sort_unstable();
        assert_eq!(p, (0..6).collect::<Vec<_>>());
        let n = Arbore::default().noduri(&lista(), Limba::Engleza);
        assert!(rezumat(&n).contains(&"  Stored Procedures 2".to_string()));
    }

    #[test]
    fn nodul_inchis_isi_ascunde_copiii_si_se_redeschide() {
        let l = lista();
        let mut a = Arbore::default();
        let dosar = a.noduri(&l, Limba::Romana)[1].cheie.clone();
        a.comuta(&dosar);
        let n = a.noduri(&l, Limba::Romana);
        assert_eq!(n.len(), 11, "cele doua proceduri se ascund");
        assert!(!n[1].deschis);
        assert_eq!(n[1].numar, 2, "numarul ramane pe dosarul inchis");
        a.comuta("b:B2");
        // B1 cu trei dosare si doua obiecte, plus B2 inchis
        assert_eq!(a.noduri(&l, Limba::Romana).len(), 7);
        a.comuta(&dosar);
        a.comuta("b:B2");
        assert_eq!(a.noduri(&l, Limba::Romana).len(), 13);
    }

    #[test]
    fn filtrul_listei_se_aplica_arborelui() {
        let mut l = lista();
        l.cauta("usp");
        let n = Arbore::default().noduri(&l, Limba::Romana);
        assert_eq!(
            rezumat(&n),
            [
                "B1 2",
                "  Proceduri 2",
                "    dbo.usp_a 0",
                "    dbo.usp_b 0",
                "B2 1",
                "  Proceduri 1",
                "    dbo.usp_c 0"
            ]
        );
    }

    #[test]
    fn selectia_isi_deschide_calea() {
        let l = lista();
        let mut a = Arbore::default();
        a.comuta("b:B2");
        let p = (0..6)
            .find(|&p| l.obiect(p).is_some_and(|o| o.nume == "trg_a"))
            .expect("trg_a in lista");
        a.comuta(&format!("d:B2/{}", Dosar::Triggere as u8));
        a.deschide_calea(&l, p);
        let n = a.noduri(&l, Limba::Romana);
        assert!(n.iter().any(|x| x.pozitie == Some(p)), "obiectul se vede");
    }

    #[test]
    fn tipurile_ajung_in_dosarul_lor() {
        assert_eq!(Dosar::din_tip("P "), Dosar::Proceduri);
        assert_eq!(Dosar::din_tip("IF"), Dosar::Functii);
        assert_eq!(Dosar::din_tip("TF"), Dosar::Functii);
        assert_eq!(Dosar::din_tip("V"), Dosar::Viewuri);
        assert_eq!(Dosar::din_tip("TR"), Dosar::Triggere);
        assert_eq!(Dosar::din_tip("SN"), Dosar::Altele);
    }
}
