//! Erorile modulului, cu coduri `GM-MOD-*`.
//!
//! **Se strica astfel:** acelasi lucru ca la celelalte crate-uri — un raport
//! fara nimic de care sa te prinzi. Registrul e in `JURNAL.md`.

pub type Rezultat<T> = Result<T, Eroare>;

#[derive(Debug, thiserror::Error)]
pub enum Eroare {
    #[error("[GM-MOD-001] nu s-a rulat nicio comparare")]
    NicioComparare,

    #[error("[GM-MOD-002] niciun obiect de deployat. Obiectele marcate exista doar in bazele After, deci nu exista cod de adus din Before")]
    NimicDeDeployat,

    #[error("[GM-MOD-003] nu pot scrie in {cale}: {detaliu}")]
    ScriereDeploy { cale: String, detaliu: String },

    #[error("[GM-MOD-004] obiectul cu id {0} nu exista in rezultate")]
    RandNecunoscut(usize),

    #[error("[GM-MOD-005] nu pot exporta in {cale}: {detaliu}")]
    Export { cale: String, detaliu: String },

    #[error("[GM-MOD-006] {0}")]
    Platforma(String),

    #[error("[GM-MOD-007] {0}")]
    Date(String),

    #[error("[GM-MOD-008] numele {nume:?} contine caractere de control (rand nou, tab etc.); intr-un script de deploy ar putea deveni cod executabil, deci nu se scrie")]
    NumeNesigur { nume: String },

    #[error("[GM-MOD-009] definitia lui {obiect} are linia `GO` (liniile {linii}); sqlcmd si SSMS taie batch-ul acolo, chiar si in interiorul unui string, deci restul s-ar executa separat. Obiectul trebuie deployat de mana")]
    GoInDefinitie { obiect: String, linii: String },

    #[error("[GM-MOD-010] directorul {cale} nu e gol; fisierele dintr-o rulare veche s-ar amesteca cu cele noi. Alege un director nou sau gol")]
    DirectorNegol { cale: String },

    #[error("[GM-MOD-011] {obiect} e criptat (WITH ENCRYPTION) intr-una din baze; codul nu se poate citi, deci nu se compara si nu se deployeaza")]
    ObiectCriptat { obiect: String },

    #[error("[GM-MOD-013] {obiect} apare de doua ori in planul pentru {baza}; ar fi scris de doua ori, iar revenirea ar avea doua DROP-uri pentru el. Scoate pasul repetat din plan")]
    ObiectRepetat { obiect: String, baza: String },

    #[error("[GM-MOD-014] definitia lui {obiect} creeaza {gasit}, nu {obiect}; scriptul ar face alt obiect decat cel din plan, iar revenirea ar sterge unul pe care deploy-ul nu l-a creat. Scrie numele intreg, `[schema].[nume]`, in textul final")]
    NumeDinDefinitie { obiect: String, gasit: String },

    #[error("[GM-MOD-015] codul de deploy al lui {obiect} nu incepe cu `CREATE {asteptat}` (gasit: {gasit}); s-ar executa ca SQL oarecare dupa `USE`, nu ar crea obiectul. Corecteaza textul final sau sterge-l (Reset)")]
    CodFaraCreate {
        obiect: String,
        asteptat: String,
        gasit: String,
    },

    #[error("[GM-MOD-016] textul final al lui {obiect} e gol; ar sterge codul obiectului. Sterge textul final (Reset) daca vrei codul din Before")]
    TextFinalGol { obiect: String },

    #[error("[GM-MOD-017] {obiect} e {martor} in Before si {modificata} in {baza}; `CREATE OR ALTER` nu poate schimba tipul unui obiect, deci scriptul ar cadea. Obiectul se deployeaza de mana (DROP, apoi CREATE)")]
    TipSchimbat {
        obiect: String,
        baza: String,
        martor: String,
        modificata: String,
    },

    #[error("[GM-MOD-018] o decizie salvata nu mai e valabila, deci deploy-ul nu se scrie: {lista}. Obiectul s-a schimbat de cand ai decis (sau l-ai omis dupa plan): revizuieste-l in editor (omite din nou sau scoate omiterea; refa textul final sau Reset) si apoi deploy")]
    DecizieCazuta { lista: String },

    #[error("[GM-MOD-012] baza s-a schimbat de la comparare, deci deploy-ul nu se scrie: {schimbari}. Compara din nou si revizuieste ce s-a schimbat")]
    BazaAtinsa { schimbari: String },
}

impl Eroare {
    pub fn cod(&self) -> &'static str {
        match self {
            Eroare::NicioComparare => "GM-MOD-001",
            Eroare::NimicDeDeployat => "GM-MOD-002",
            Eroare::ScriereDeploy { .. } => "GM-MOD-003",
            Eroare::RandNecunoscut(_) => "GM-MOD-004",
            Eroare::Export { .. } => "GM-MOD-005",
            Eroare::Platforma(_) => "GM-MOD-006",
            Eroare::Date(_) => "GM-MOD-007",
            Eroare::NumeNesigur { .. } => "GM-MOD-008",
            Eroare::GoInDefinitie { .. } => "GM-MOD-009",
            Eroare::DirectorNegol { .. } => "GM-MOD-010",
            Eroare::ObiectCriptat { .. } => "GM-MOD-011",
            Eroare::BazaAtinsa { .. } => "GM-MOD-012",
            Eroare::ObiectRepetat { .. } => "GM-MOD-013",
            Eroare::NumeDinDefinitie { .. } => "GM-MOD-014",
            Eroare::CodFaraCreate { .. } => "GM-MOD-015",
            Eroare::TextFinalGol { .. } => "GM-MOD-016",
            Eroare::TipSchimbat { .. } => "GM-MOD-017",
            Eroare::DecizieCazuta { .. } => "GM-MOD-018",
        }
    }
}

impl From<gmotion_db::Eroare> for Eroare {
    fn from(e: gmotion_db::Eroare) -> Self {
        // Codul original se pastreaza in text, ca sa nu se pierda informatia
        // la traversarea granitei.
        Eroare::Date(e.to_string())
    }
}

impl From<gmotion_platforma::Eroare> for Eroare {
    fn from(e: gmotion_platforma::Eroare) -> Self {
        Eroare::Platforma(e.to_string())
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn coduri_unice_si_prezente_in_mesaj() {
        // Lista e scrisa de mana, deci se poate uita o varianta — si s-a
        // uitat: pana la auditul 2026-11-23 avea 12 din 18, fara chiar
        // ultimele adaugate. `match`-ul de mai jos e exhaustiv, deci
        // compilatorul cade daca apare o varianta noua si nu e pusa aici.
        fn exemplu(e: &Eroare) -> Eroare {
            match e {
                Eroare::NicioComparare => Eroare::NicioComparare,
                Eroare::NimicDeDeployat => Eroare::NimicDeDeployat,
                Eroare::ScriereDeploy { .. } => Eroare::ScriereDeploy {
                    cale: "c".into(),
                    detaliu: "d".into(),
                },
                Eroare::RandNecunoscut(_) => Eroare::RandNecunoscut(1),
                Eroare::Export { .. } => Eroare::Export {
                    cale: "c".into(),
                    detaliu: "d".into(),
                },
                Eroare::Platforma(_) => Eroare::Platforma("p".into()),
                Eroare::Date(_) => Eroare::Date("d".into()),
                Eroare::NumeNesigur { .. } => Eroare::NumeNesigur { nume: "n".into() },
                Eroare::GoInDefinitie { .. } => Eroare::GoInDefinitie {
                    obiect: "o".into(),
                    linii: "1".into(),
                },
                Eroare::DirectorNegol { .. } => Eroare::DirectorNegol { cale: "c".into() },
                Eroare::ObiectCriptat { .. } => Eroare::ObiectCriptat { obiect: "o".into() },
                Eroare::BazaAtinsa { .. } => Eroare::BazaAtinsa {
                    schimbari: "s".into(),
                },
                Eroare::ObiectRepetat { .. } => Eroare::ObiectRepetat {
                    obiect: "o".into(),
                    baza: "b".into(),
                },
                Eroare::NumeDinDefinitie { .. } => Eroare::NumeDinDefinitie {
                    obiect: "o".into(),
                    gasit: "g".into(),
                },
                Eroare::CodFaraCreate { .. } => Eroare::CodFaraCreate {
                    obiect: "o".into(),
                    asteptat: "PROCEDURE".into(),
                    gasit: "g".into(),
                },
                Eroare::TextFinalGol { .. } => Eroare::TextFinalGol { obiect: "o".into() },
                Eroare::TipSchimbat { .. } => Eroare::TipSchimbat {
                    obiect: "o".into(),
                    martor: "P".into(),
                    modificata: "V".into(),
                    baza: "b".into(),
                },
                Eroare::DecizieCazuta { .. } => Eroare::DecizieCazuta { lista: "l".into() },
            }
        }
        let toate = [
            Eroare::NicioComparare,
            Eroare::NimicDeDeployat,
            Eroare::ScriereDeploy {
                cale: "c".into(),
                detaliu: "d".into(),
            },
            Eroare::RandNecunoscut(1),
            Eroare::Export {
                cale: "c".into(),
                detaliu: "d".into(),
            },
            Eroare::Platforma("p".into()),
            Eroare::Date("d".into()),
            Eroare::NumeNesigur { nume: "n".into() },
            Eroare::GoInDefinitie {
                obiect: "o".into(),
                linii: "1".into(),
            },
            Eroare::DirectorNegol { cale: "c".into() },
            Eroare::ObiectCriptat { obiect: "o".into() },
            Eroare::BazaAtinsa {
                schimbari: "s".into(),
            },
            Eroare::ObiectRepetat {
                obiect: "o".into(),
                baza: "b".into(),
            },
            Eroare::NumeDinDefinitie {
                obiect: "o".into(),
                gasit: "g".into(),
            },
            Eroare::CodFaraCreate {
                obiect: "o".into(),
                asteptat: "PROCEDURE".into(),
                gasit: "g".into(),
            },
            Eroare::TextFinalGol { obiect: "o".into() },
            Eroare::TipSchimbat {
                obiect: "o".into(),
                martor: "P".into(),
                modificata: "V".into(),
                baza: "b".into(),
            },
            Eroare::DecizieCazuta { lista: "l".into() },
        ];
        // fiecare varianta din lista se regaseste in `exemplu`: daca enum-ul
        // creste, compilatorul cere si acolo o ramura noua
        for e in &toate {
            assert_eq!(exemplu(e).cod(), e.cod());
        }
        let mut c: Vec<&str> = toate.iter().map(|e| e.cod()).collect();
        c.sort_unstable();
        let n = c.len();
        c.dedup();
        assert_eq!(c.len(), n);
        for e in &toate {
            assert!(e.to_string().contains(e.cod()));
        }
    }

    #[test]
    fn codul_original_nu_se_pierde_la_traversare() {
        let db = gmotion_db::Eroare::BazaLipsa {
            server: "s".into(),
            baza: "b".into(),
        };
        let m: Eroare = db.into();
        assert_eq!(m.cod(), "GM-MOD-007");
        assert!(m.to_string().contains("GM-DB-005"), "codul original: {m}");
    }
}
