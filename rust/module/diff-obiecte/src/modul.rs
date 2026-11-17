//! Declaratia modulului catre platforma.
//!
//! **Se strica astfel:** modulul nu se inregistreaza, sau isi declara o metoda
//! care se ciocneste cu una comuna. Ambele sunt prinse de `Registru` la
//! inregistrare, deci la pornire, nu la rulare.

use gmotion_platforma::{CerinteBaze, Ecran, Modul};

/// Ecranele modulului. **Provizoriu**, ca si contractul din platforma:
/// interfata nu exista inca.
const ECRANE: [Ecran; 3] = [
    Ecran {
        id: "lista",
        titlu: "Obiecte",
    },
    Ecran {
        id: "editor",
        titlu: "Diferente",
    },
    Ecran {
        id: "deploy",
        titlu: "Deploy",
    },
];

/// Metodele specifice, publicate de platforma drept `diff-obiecte/<metoda>`.
/// Niciuna nu are voie sa fie una din `METODE_PLATFORMA`.
const METODE: [&str; 5] = ["compara", "rezultate", "diferente", "export", "deploy"];

#[derive(Debug, Default, Clone, Copy)]
pub struct ModulDiffObiecte;

impl Modul for ModulDiffObiecte {
    fn nume(&self) -> &str {
        crate::NUME
    }
    fn titlu(&self) -> &str {
        "Diferente intre obiecte"
    }
    fn iconita(&self) -> &str {
        "git-compare"
    }
    fn cerinte(&self) -> CerinteBaze {
        CerinteBaze::ReferintaPlusUna
    }
    fn ecrane(&self) -> &[Ecran] {
        &ECRANE
    }
    fn ecran_intrare(&self) -> &str {
        "lista"
    }
    fn metode(&self) -> &[&'static str] {
        &METODE
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use gmotion_platforma::{Registru, Ruta, METODE_PLATFORMA};

    #[test]
    fn se_inregistreaza_si_se_ruteaza() {
        let mut r = Registru::nou();
        r.inregistreaza(Box::new(ModulDiffObiecte)).unwrap();
        assert_eq!(r.nume_module(), ["diff-obiecte"]);
        for m in METODE {
            match r.ruteaza(&format!("diff-obiecte/{m}")).unwrap() {
                Ruta::Modul { metoda, .. } => assert_eq!(metoda, m),
                Ruta::Platforma(_) => panic!("{m} nu e metoda de platforma"),
            }
        }
    }

    #[test]
    fn nicio_metoda_nu_calca_platforma() {
        for m in METODE {
            assert!(
                !METODE_PLATFORMA.contains(&m),
                "{m} e deja oferita de platforma"
            );
        }
    }

    #[test]
    fn cere_referinta_plus_cel_putin_o_baza() {
        let m = ModulDiffObiecte;
        assert!(m.cerinte().verifica(m.nume(), "Ref", &["A".into()]).is_ok());
        assert!(m.cerinte().verifica(m.nume(), "Ref", &[]).is_err());
    }
}
