//! Contractul de modul si registrul: cum se declara o unealta si cum ajung
//! apelurile la ea.
//!
//! **Se strica astfel:** metodele comune se rescriu in fiecare modul, sau un
//! modul ajunge la starea altuia. In versiunea Python clasa `Api` expunea
//! metodele modulului de diff direct la nivelul de sus (`test_conexiune`,
//! `cauta_servere`, `progres`, `log_lines` lingă `rezultate` si
//! `genereaza_deployer`). Cu patru module, ori se ciocnesc numele, ori se
//! copiaza codul comun de patru ori.
//!
//! Aici: ce e comun stă la platforma si se scrie o data; ce e specific se
//! inregistreaza cu prefix de modul, deci `diff-obiecte/rezultate` si
//! `upgrade/rezultate` pot coexista.
//!
//! **Provizoriu:** partea de ecrane e declarativa si minimala, fiindca
//! interfata nu exista inca. Se va largi la etapa de fereastra, si atunci
//! modulul 2 va fi testul real al contractului — daca are nevoie de schimbari
//! in platforma, contractul de aici era greșit.

use std::collections::BTreeMap;

use crate::eroare::{valideaza_identificator, Eroare, Rezultat};

/// De ce are nevoie un modul de la platforma, in termeni de baze.
///
/// Platforma verifica inainte de a porni modulul, ca modulul sa nu trebuiasca
/// sa valideze si sa raporteze el.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CerinteBaze {
    /// O singura baza (ex: o unealta de inspectie).
    Una,
    /// O baza de referinta plus una sau mai multe de comparat.
    ReferintaPlusUna,
    /// Doua baze si o mapare intre ele.
    DouaCuMapare,
}

impl CerinteBaze {
    pub fn descriere(self) -> &'static str {
        match self {
            CerinteBaze::Una => "o singura baza",
            CerinteBaze::ReferintaPlusUna => "o baza de referinta si cel putin una de comparat",
            CerinteBaze::DouaCuMapare => "doua baze si o mapare intre ele",
        }
    }

    /// Verifica o selectie. `referinta` e numele bazei de referinta, `baze`
    /// cele de comparat (fara referinta).
    pub fn verifica(self, modul: &str, referinta: &str, baze: &[String]) -> Rezultat<()> {
        let potrivit = match self {
            CerinteBaze::Una => !referinta.is_empty() && baze.is_empty(),
            CerinteBaze::ReferintaPlusUna => !referinta.is_empty() && !baze.is_empty(),
            CerinteBaze::DouaCuMapare => !referinta.is_empty() && baze.len() == 1,
        };
        if potrivit {
            return Ok(());
        }
        Err(Eroare::CerinteNeindeplinite {
            modul: modul.to_string(),
            cerinta: self.descriere().to_string(),
            primit: format!(
                "referinta {}, {} baze de comparat",
                if referinta.is_empty() {
                    "lipsa"
                } else {
                    "setata"
                },
                baze.len()
            ),
        })
    }
}

/// Un ecran al unui modul. Deocamdata doar identificator si titlu: forma
/// completa se decide la etapa de interfata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ecran {
    pub id: &'static str,
    pub titlu: &'static str,
}

/// Ce declara un modul despre sine.
///
/// Nu conține logica: logica e in crate-ul modulului. Asta e doar ce trebuie sa
/// stie platforma ca sa-l poată inregistra, ruta si porni.
pub trait Modul {
    /// Identificator: litere mici, cifre, cratima. Devine prefix de ruta **si**
    /// nume de director pentru sesiuni, deci e validat strict.
    fn nume(&self) -> &str;

    /// Titlul afisat utilizatorului.
    fn titlu(&self) -> &str;

    /// Nume de iconita, rezolvat de interfata.
    fn iconita(&self) -> &str;

    fn cerinte(&self) -> CerinteBaze;

    fn ecrane(&self) -> &[Ecran];

    /// Ecranul de intrare. Trebuie sa fie unul din [`Modul::ecrane`].
    fn ecran_intrare(&self) -> &str;

    /// Metodele pe care modulul le expune interfetei, fara prefix. Platforma le
    /// publica drept `<nume>/<metoda>`.
    fn metode(&self) -> &[&'static str];
}

/// Metodele comune, scrise o singura data la nivelul platformei. Un modul nu
/// are voie sa declare una din astea: ar ascunde-o pentru el, si atunci
/// comportamentul ar depinde de modulul activ.
pub const METODE_PLATFORMA: [&str; 8] = [
    "test_conexiune",
    "cauta_servere",
    "baze",
    "progres",
    "jurnal",
    "sesiuni",
    "module",
    "diagnostic",
];

/// Modulele inregistrate si rutarea catre ele.
#[derive(Default)]
pub struct Registru {
    module: BTreeMap<String, Box<dyn Modul>>,
}

impl Registru {
    pub fn nou() -> Self {
        Registru::default()
    }

    /// Inregistreaza un modul, verificand tot ce poate fi verificat devreme:
    /// numele, unicitatea, ecranul de intrare, si ca nu calca metodele comune.
    pub fn inregistreaza(&mut self, m: Box<dyn Modul>) -> Rezultat<()> {
        let nume = m.nume().to_string();
        valideaza_identificator(&nume)?;
        if self.module.contains_key(&nume) {
            return Err(Eroare::ModulDuplicat(nume));
        }
        for e in m.ecrane() {
            valideaza_identificator(e.id)?;
        }
        if !m.ecrane().iter().any(|e| e.id == m.ecran_intrare()) {
            return Err(Eroare::RutaInvalida(format!(
                "{nume}: ecranul de intrare {:?} nu e intre ecranele declarate",
                m.ecran_intrare()
            )));
        }
        for metoda in m.metode() {
            valideaza_identificator(metoda)?;
            if METODE_PLATFORMA.contains(metoda) {
                return Err(Eroare::RutaInvalida(format!(
                    "{nume}: metoda {metoda:?} e una comuna, oferita de platforma"
                )));
            }
        }
        tracing::info!(target: "gmotion::platforma", modul = %nume, "modul inregistrat");
        self.module.insert(nume, m);
        Ok(())
    }

    pub fn nume_module(&self) -> Vec<&str> {
        self.module.keys().map(String::as_str).collect()
    }

    pub fn modul(&self, nume: &str) -> Rezultat<&dyn Modul> {
        self.module
            .get(nume)
            .map(AsRef::as_ref)
            .ok_or_else(|| Eroare::ModulNecunoscut(nume.to_string()))
    }

    /// Imparte o ruta in modul si metoda, verificand ca ambele exista.
    ///
    /// O metoda comuna (`progres`, `cauta_servere`, ...) se scrie fara prefix si
    /// intoarce [`Ruta::Platforma`].
    pub fn ruteaza(&self, ruta: &str) -> Rezultat<Ruta<'_>> {
        // Se intoarce intrarea din tabel, nu textul primit: `Ruta` trebuie sa
        // trăiască independent de durata argumentului.
        if let Some(m) = METODE_PLATFORMA.iter().find(|m| **m == ruta) {
            return Ok(Ruta::Platforma(m));
        }
        let Some((nume, metoda)) = ruta.split_once('/') else {
            return Err(Eroare::RutaInvalida(ruta.to_string()));
        };
        if nume.is_empty() || metoda.is_empty() || metoda.contains('/') {
            return Err(Eroare::RutaInvalida(ruta.to_string()));
        }
        let m = self.modul(nume)?;
        if !m.metode().contains(&metoda) {
            return Err(Eroare::RutaInvalida(format!(
                "{ruta}: modulul {nume} nu expune {metoda:?}"
            )));
        }
        Ok(Ruta::Modul {
            modul: m,
            metoda: m
                .metode()
                .iter()
                .find(|x| **x == metoda)
                .copied()
                .expect("verificat mai sus"),
        })
    }
}

/// Unde duce o ruta.
pub enum Ruta<'a> {
    /// O metoda comuna, tratata de platforma.
    Platforma(&'static str),
    Modul {
        modul: &'a dyn Modul,
        metoda: &'static str,
    },
}

#[cfg(test)]
mod teste {
    use super::*;

    struct ModulTest {
        nume: &'static str,
        metode: Vec<&'static str>,
        ecrane: Vec<Ecran>,
        intrare: &'static str,
    }

    impl ModulTest {
        fn nou(nume: &'static str) -> Self {
            ModulTest {
                nume,
                metode: vec!["rezultate", "elemente"],
                ecrane: vec![Ecran {
                    id: "lista",
                    titlu: "Lista",
                }],
                intrare: "lista",
            }
        }
    }

    impl Modul for ModulTest {
        fn nume(&self) -> &str {
            self.nume
        }
        fn titlu(&self) -> &str {
            "Test"
        }
        fn iconita(&self) -> &str {
            "test"
        }
        fn cerinte(&self) -> CerinteBaze {
            CerinteBaze::ReferintaPlusUna
        }
        fn ecrane(&self) -> &[Ecran] {
            &self.ecrane
        }
        fn ecran_intrare(&self) -> &str {
            self.intrare
        }
        fn metode(&self) -> &[&'static str] {
            &self.metode
        }
    }

    fn registru_cu(nume: &'static str) -> Registru {
        let mut r = Registru::nou();
        r.inregistreaza(Box::new(ModulTest::nou(nume))).unwrap();
        r
    }

    #[test]
    fn doua_module_pot_avea_metode_cu_acelasi_nume() {
        let mut r = registru_cu("diff-obiecte");
        r.inregistreaza(Box::new(ModulTest::nou("upgrade")))
            .unwrap();

        // Exact ciocnirea pe care versiunea Python nu o putea evita.
        for ruta in ["diff-obiecte/rezultate", "upgrade/rezultate"] {
            match r.ruteaza(ruta).unwrap() {
                Ruta::Modul { modul, metoda } => {
                    assert_eq!(metoda, "rezultate");
                    assert!(ruta.starts_with(modul.nume()));
                }
                Ruta::Platforma(_) => panic!("{ruta} nu e metoda de platforma"),
            }
        }
    }

    #[test]
    fn metodele_comune_nu_au_prefix() {
        let r = registru_cu("diff-obiecte");
        for m in METODE_PLATFORMA {
            match r.ruteaza(m).unwrap() {
                Ruta::Platforma(x) => assert_eq!(x, m),
                Ruta::Modul { .. } => panic!("{m} ar trebui tratata de platforma"),
            }
        }
    }

    #[test]
    fn un_modul_nu_poate_calca_o_metoda_comuna() {
        let mut r = Registru::nou();
        let mut m = ModulTest::nou("diff-obiecte");
        m.metode = vec!["progres"];
        let e = r.inregistreaza(Box::new(m)).unwrap_err();
        assert_eq!(e.cod(), "GM-PLAT-009");
    }

    #[test]
    fn rute_invalide() {
        let r = registru_cu("diff-obiecte");
        for ruta in [
            "",
            "rezultate",
            "/rezultate",
            "diff-obiecte/",
            "diff-obiecte/a/b",
            "necunoscut/rezultate",
            "diff-obiecte/inexistenta",
        ] {
            assert!(r.ruteaza(ruta).is_err(), "{ruta:?} ar trebui refuzata");
        }
    }

    #[test]
    fn modul_duplicat_si_nume_invalid() {
        let mut r = registru_cu("diff-obiecte");
        assert_eq!(
            r.inregistreaza(Box::new(ModulTest::nou("diff-obiecte")))
                .unwrap_err()
                .cod(),
            "GM-PLAT-008"
        );
        assert_eq!(
            r.inregistreaza(Box::new(ModulTest::nou("Diff Obiecte")))
                .unwrap_err()
                .cod(),
            "GM-PLAT-006"
        );
    }

    #[test]
    fn ecranul_de_intrare_trebuie_declarat() {
        let mut r = Registru::nou();
        let mut m = ModulTest::nou("diff-obiecte");
        m.intrare = "inexistent";
        assert_eq!(
            r.inregistreaza(Box::new(m)).unwrap_err().cod(),
            "GM-PLAT-009"
        );
    }

    #[test]
    fn cerintele_se_verifica_inainte_de_pornire() {
        let c = CerinteBaze::ReferintaPlusUna;
        assert!(c.verifica("m", "Ref", &["A".into()]).is_ok());
        assert_eq!(
            c.verifica("m", "Ref", &[]).unwrap_err().cod(),
            "GM-PLAT-010"
        );
        assert_eq!(
            c.verifica("m", "", &["A".into()]).unwrap_err().cod(),
            "GM-PLAT-010"
        );

        assert!(CerinteBaze::Una.verifica("m", "Ref", &[]).is_ok());
        assert!(CerinteBaze::Una
            .verifica("m", "Ref", &["A".into()])
            .is_err());
        assert!(CerinteBaze::DouaCuMapare
            .verifica("m", "Ref", &["A".into()])
            .is_ok());
        assert!(CerinteBaze::DouaCuMapare
            .verifica("m", "Ref", &["A".into(), "B".into()])
            .is_err());
    }

    #[test]
    fn mesajul_de_cerinte_spune_si_ce_a_primit() {
        let e = CerinteBaze::ReferintaPlusUna
            .verifica("diff-obiecte", "Ref", &[])
            .unwrap_err()
            .to_string();
        assert!(e.contains("diff-obiecte"));
        assert!(e.contains("cel putin una"));
        assert!(e.contains("0 baze"));
    }
}
