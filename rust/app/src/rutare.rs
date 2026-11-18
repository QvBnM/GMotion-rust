//! Ce ecran e activ, ce obiect e deschis, ce face fiecare comanda.
//!
//! **Se strica astfel:** bifa pusa in editor nu se vede in lista (sau invers),
//! `j` / `k` din editor te intorc in lista in loc sa treaca la obiectul
//! urmator, sau obiectul deschis nu mai e cel selectat dupa un filtru. Toate
//! sunt bug-uri de stare, nu de desen — de-aia starea sta aici, fara Slint, si
//! se testeaza ca atare.

use gmotion_fereastra::model_editor::{construieste, ModelEditor};
use gmotion_fereastra::model_lista::{Detalii, Lista, Obiect, Ordine, Stare};
use gmotion_modul_diff_obiecte::{self as modul, Baza, Rand, Revizuire};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ecran {
    Lista,
    Editor,
}

/// Tot ce poate cere utilizatorul, din oricare ecran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comanda {
    MutaSelectia(i32),
    Selecteaza(usize),
    Deschide,
    InapoiLaLista,
    ObiectUrmator,
    ObiectAnterior,
    ComutaRevizuit,
    Filtreaza(Option<Stare>),
    FiltreazaTip(Option<String>),
    Cauta(String),
    Sorteaza(Ordine),
}

/// Starea aplicatiei: lista, ecranul activ si progresul revizuirii.
pub struct Aplicatie {
    lista: Lista,
    ecran: Ecran,
    revizuire: Revizuire,
    /// Randurile compararii, pe `schema.nume`: de acolo vin amprentele.
    randuri: Vec<Rand>,
    sursa: Baza,
    tinta: Baza,
}

impl Aplicatie {
    /// `randuri` sunt randurile compararii; `sursa` si `tinta`, obiectele
    /// pregatite ale celor doua baze.
    pub fn noua(randuri: Vec<Rand>, sursa: Baza, tinta: Baza, revizuire: Revizuire) -> Self {
        let obiecte: Vec<Obiect> = randuri
            .iter()
            .map(|r| {
                let (schema, nume) = r.obiect.split_once('.').unwrap_or(("dbo", &r.obiect));
                Obiect {
                    schema: schema.to_string(),
                    nume: nume.to_string(),
                    tip: r.tip.clone(),
                    stare: match r.categorie {
                        modul::Categorie::Diferit => Stare::Modificat,
                        modul::Categorie::Lipsa => Stare::DoarSursa,
                        modul::Categorie::Extra => Stare::DoarBaza,
                        modul::Categorie::Criptat => Stare::Criptat,
                    },
                    detalii: None,
                    revizuit: revizuire.e_revizuit(r),
                }
            })
            .collect();
        let mut lista = Lista::noua(obiecte);
        if !lista.vizibile().is_empty() {
            lista.selecteaza(0);
        }
        Aplicatie {
            lista,
            ecran: Ecran::Lista,
            revizuire,
            randuri,
            sursa,
            tinta,
        }
    }

    pub fn ecran(&self) -> Ecran {
        self.ecran
    }

    pub fn lista(&self) -> &Lista {
        &self.lista
    }

    pub fn revizuire(&self) -> &Revizuire {
        &self.revizuire
    }

    /// Obiectul selectat (in lista) sau deschis (in editor) — acelasi, mereu:
    /// editorul se deschide pe selectie, iar `j` / `k` din editor **muta
    /// selectia**, ca intoarcerea in lista sa cada unde trebuie.
    pub fn obiect(&self) -> Option<&Obiect> {
        self.lista.obiect_selectat()
    }

    fn rand(&self, id: &str) -> Option<&Rand> {
        self.randuri.iter().find(|r| r.obiect == id)
    }

    /// Textele pregatite ale obiectului selectat, daca exista in ambele baze.
    fn texte(&self, id: &str) -> Option<(&str, &str)> {
        match (self.sursa.get(id), self.tinta.get(id)) {
            (Some(a), Some(b)) => Some((a.afisare.as_str(), b.afisare.as_str())),
            _ => None,
        }
    }

    /// Modelul editorului pentru obiectul curent. `None` pentru obiectele care
    /// exista doar intr-o parte: nu au cu ce fi comparate.
    pub fn model_editor(&self) -> Option<ModelEditor> {
        let id = self.obiect()?.id();
        let (a, b) = self.texte(&id)?;
        Some(construieste(a, b))
    }

    /// Propozitia pentru previzualizare si detaliile randului curent.
    pub fn texte_obiect(&self) -> Option<(&str, &str, Vec<&'static str>)> {
        let id = self.obiect()?.id();
        let (a, b) = self.texte(&id)?;
        let setari = match (self.sursa.get(&id), self.tinta.get(&id)) {
            (Some(x), Some(y)) => modul::setari_diferite(x.setari, y.setari),
            _ => Vec::new(),
        };
        Some((a, b, setari))
    }

    /// Calculeaza detaliile („Difera in", `+N −M`) pentru randurile de pe
    /// ecran, daca lipsesc. Restul listei ramane necalculat.
    pub fn asigura_detalii(&mut self, de_la: usize, pana_la: usize) {
        for p in self.lista.fara_detalii(de_la, pana_la) {
            let Some(o) = self.lista.obiect(p) else {
                continue;
            };
            let id = o.id();
            let (Some(a), Some(b)) = (self.sursa.get(&id), self.tinta.get(&id)) else {
                continue;
            };
            let setari = modul::setari_diferite(a.setari, b.setari);
            let d = modul::detalii_obiect(&a.afisare, &b.afisare, &setari);
            self.lista.pune_detalii(
                p,
                Detalii {
                    difera_in: d.difera_in,
                    linii_plus: d.linii_plus,
                    linii_minus: d.linii_minus,
                },
            );
        }
    }

    /// Executa o comanda. Intoarce `true` daca starea de revizuire s-a
    /// schimbat, deci sesiunea trebuie salvata.
    pub fn comanda(&mut self, c: Comanda) -> bool {
        match c {
            Comanda::MutaSelectia(pas) => self.lista.muta_selectia(pas),
            Comanda::Selecteaza(p) => self.lista.selecteaza(p),
            Comanda::Deschide => {
                // un obiect care exista doar intr-o parte n-are ce arata in
                // editor: ecranul ramane pe lista
                if self.model_editor().is_some() {
                    self.ecran = Ecran::Editor;
                }
            }
            Comanda::InapoiLaLista => self.ecran = Ecran::Lista,
            // din editor, obiectul urmator se deschide direct: altfel ar
            // trebui drum dus-intors prin lista pentru fiecare obiect
            Comanda::ObiectUrmator => self.treci(1),
            Comanda::ObiectAnterior => self.treci(-1),
            Comanda::ComutaRevizuit => return self.comuta_revizuit(),
            Comanda::Filtreaza(s) => self.lista.filtreaza_stare(s),
            Comanda::FiltreazaTip(t) => self.lista.filtreaza_tip(t),
            Comanda::Cauta(t) => self.lista.cauta(&t),
            Comanda::Sorteaza(o) => self.lista.sorteaza(o),
        }
        false
    }

    /// Trece la obiectul urmator / anterior care se poate deschide, ramanand
    /// in editor.
    fn treci(&mut self, pas: i32) {
        let inceput = self.lista.selectat();
        for _ in 0..self.lista.vizibile().len() {
            self.lista.muta_selectia(pas);
            if self.lista.selectat() == inceput {
                break;
            }
            if self.ecran == Ecran::Lista || self.model_editor().is_some() {
                return;
            }
        }
        // nu exista alt obiect de deschis: ramane unde era
        if let Some(p) = inceput {
            self.lista.selecteaza(p);
        }
    }

    fn comuta_revizuit(&mut self) -> bool {
        let Some(p) = self.lista.selectat() else {
            return false;
        };
        let Some(o) = self.lista.obiect(p) else {
            return false;
        };
        let (id, era) = (o.id(), o.revizuit);
        let Some(rand) = self.rand(&id).cloned() else {
            return false;
        };
        if era {
            self.revizuire.scoate(&rand.baza, &rand.obiect);
        } else {
            self.revizuire.marcheaza(&rand);
        }
        self.lista.comuta_revizuit(p);
        true
    }

    /// Starea de revizuire, curatata de bifele care nu mai corespund, gata de
    /// salvat in sesiune.
    pub fn revizuire_de_salvat(&mut self) -> &Revizuire {
        self.revizuire.curata(&self.randuri);
        &self.revizuire
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use gmotion_db::Obiect as ObiectDb;

    fn obiect_db(nume: &str, definitie: &str) -> ObiectDb {
        ObiectDb {
            schema: "dbo".into(),
            nume: nume.into(),
            tip: "P".into(),
            definitie: definitie.into(),
            setari: None,
        }
    }

    /// Doua baze cu: un obiect modificat, unul modificat altfel, unul care
    /// exista doar in sursa.
    fn aplicatie() -> Aplicatie {
        let sursa = modul::pregateste(&[
            obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1"),
            obiect_db("pB", "CREATE PROC dbo.pB AS\nSELECT 2"),
            obiect_db("pDoarSursa", "CREATE PROC dbo.pDoarSursa AS\nSELECT 3"),
        ]);
        let tinta = modul::pregateste(&[
            obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 99"),
            obiect_db("pB", "CREATE PROC dbo.pB AS\nSELECT 98"),
        ]);
        let comparare = modul::compara(
            "Referinta",
            &sursa,
            &[("Client1".to_string(), tinta.clone())],
            |_| {},
        );
        Aplicatie::noua(comparare.randuri, sursa, tinta, Revizuire::noua())
    }

    #[test]
    fn deschiderea_duce_in_editor_si_esc_inapoi() {
        let mut a = aplicatie();
        assert_eq!(a.ecran(), Ecran::Lista);
        a.comanda(Comanda::Deschide);
        assert_eq!(a.ecran(), Ecran::Editor);
        assert_eq!(a.obiect().unwrap().nume, "pA");
        a.comanda(Comanda::InapoiLaLista);
        assert_eq!(a.ecran(), Ecran::Lista);
        assert_eq!(a.obiect().unwrap().nume, "pA", "selectia a ramas unde era");
    }

    #[test]
    fn din_editor_se_trece_la_obiectul_urmator_fara_sa_treci_prin_lista() {
        let mut a = aplicatie();
        a.comanda(Comanda::Deschide);
        a.comanda(Comanda::ObiectUrmator);
        assert_eq!(a.ecran(), Ecran::Editor, "ramanem in editor");
        assert_eq!(a.obiect().unwrap().nume, "pB");
        a.comanda(Comanda::ObiectAnterior);
        assert_eq!(a.obiect().unwrap().nume, "pA");
    }

    #[test]
    fn un_obiect_doar_intr_o_parte_nu_deschide_editorul() {
        let mut a = aplicatie();
        let p = (0..a.lista().vizibile().len())
            .find(|&p| a.lista().obiect(p).unwrap().nume == "pDoarSursa")
            .expect("obiectul exista in lista");
        a.comanda(Comanda::Selecteaza(p));
        assert!(a.model_editor().is_none());
        a.comanda(Comanda::Deschide);
        assert_eq!(a.ecran(), Ecran::Lista, "nu avem cu ce compara");
    }

    #[test]
    fn urmatorul_obiect_sare_peste_cele_care_nu_se_pot_deschide() {
        let mut a = aplicatie();
        // pB e ultimul care se poate deschide; dupa el, pDoarSursa
        let p = (0..a.lista().vizibile().len())
            .find(|&p| a.lista().obiect(p).unwrap().nume == "pB")
            .unwrap();
        a.comanda(Comanda::Selecteaza(p));
        a.comanda(Comanda::Deschide);
        assert_eq!(a.ecran(), Ecran::Editor);
        a.comanda(Comanda::ObiectUrmator);
        assert_eq!(
            a.obiect().unwrap().nume,
            "pB",
            "nu exista alt obiect de deschis, deci ramanem pe el"
        );
        assert_eq!(a.ecran(), Ecran::Editor);
    }

    #[test]
    fn bifa_pusa_in_editor_se_vede_in_lista_si_in_starea_salvata() {
        let mut a = aplicatie();
        a.comanda(Comanda::Deschide);
        assert!(a.comanda(Comanda::ComutaRevizuit), "s-a schimbat ceva");
        assert!(a.obiect().unwrap().revizuit);
        assert_eq!(a.lista().progres_revizuire(), (1, 3));
        assert_eq!(a.revizuire().numar(), 1);

        // si inapoi: scoaterea bifei
        assert!(a.comanda(Comanda::ComutaRevizuit));
        assert!(!a.obiect().unwrap().revizuit);
        assert_eq!(a.revizuire().numar(), 0);
    }

    #[test]
    fn bifele_vechi_se_reiau_la_pornire_si_cad_daca_obiectul_s_a_schimbat() {
        let mut a = aplicatie();
        a.comanda(Comanda::ComutaRevizuit);
        let salvata = a.revizuire_de_salvat().clone();

        // aceeasi comparare: bifa se reia
        let sursa = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1")]);
        let tinta = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 99")]);
        let c = modul::compara(
            "Referinta",
            &sursa,
            &[("Client1".to_string(), tinta.clone())],
            |_| {},
        );
        let b = Aplicatie::noua(c.randuri, sursa, tinta, salvata.clone());
        assert!(b.obiect().unwrap().revizuit);

        // obiectul s-a schimbat pe server: bifa nu se mai reia
        let sursa = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1")]);
        let tinta = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 77")]);
        let c = modul::compara(
            "Referinta",
            &sursa,
            &[("Client1".to_string(), tinta.clone())],
            |_| {},
        );
        let b = Aplicatie::noua(c.randuri, sursa, tinta, salvata);
        assert!(!b.obiect().unwrap().revizuit);
    }

    #[test]
    fn detaliile_se_calculeaza_doar_pentru_randurile_cerute() {
        let mut a = aplicatie();
        assert!(a.lista().obiect(0).unwrap().detalii.is_none());
        a.asigura_detalii(0, 1);
        assert_eq!(
            a.lista()
                .obiect(0)
                .unwrap()
                .detalii
                .as_ref()
                .unwrap()
                .difera_in,
            "cod"
        );
        assert!(
            a.lista().obiect(1).unwrap().detalii.is_none(),
            "randul necerut ramane necalculat"
        );
    }

    #[test]
    fn filtrul_si_cautarea_nu_pierd_obiectul_deschis() {
        let mut a = aplicatie();
        a.comanda(Comanda::Cauta("pB".into()));
        a.comanda(Comanda::Deschide);
        assert_eq!(a.obiect().unwrap().nume, "pB");
        a.comanda(Comanda::Cauta(String::new()));
        assert_eq!(
            a.obiect().unwrap().nume,
            "pB",
            "obiectul deschis ramane selectat cand cautarea se goleste"
        );
    }
}
