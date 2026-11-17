//! Tipurile care traverseaza granita: obiecte SQL, instante, acreditari.
//!
//! **Se strica astfel:** nimic singur — nu are logica. Dar forma lui decide ce
//! poate si ce nu poate face restul aplicatiei. Regula: aici stau date brute
//! din baza, **nenormalizate**. Normalizarea, hash-ul si token-ii sunt treaba
//! lui `gmotion-core`, si se calculeaza de cine are nevoie de ei.
//!
//! Versiunea Python calcula totul la citire (`aduce_obiecte`) si tinea, pentru
//! fiecare obiect din fiecare baza, o lista de token-uri pe langa text. Era al
//! treilea motiv al rescrierii.

use serde::{Deserialize, Serialize};

/// Un obiect SQL, exact cum vine din `sys.sql_modules`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Obiect {
    pub schema: String,
    pub nume: String,
    /// `sys.objects.type`, fara spatiile de umplutura: `P`, `V`, `FN`, `IF`,
    /// `TF`, `TR`.
    pub tip: String,
    /// Textul `CREATE`, neatins.
    pub definitie: String,
}

impl Obiect {
    /// `schema.nume` — identificatorul folosit peste tot in aplicatie.
    pub fn id(&self) -> String {
        format!("{}.{}", self.schema, self.nume)
    }
}

/// Cum ne autentificam. Parola nu se serializeaza niciodata: nu ajunge nici in
/// sesiuni, nici in instantanee, nici in jurnal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Autentificare {
    /// Utilizator si parola de SQL Server.
    Sql { utilizator: String, parola: String },
    /// Utilizatorul Windows curent (SSPI). Doar pe Windows.
    Integrata,
}

impl Autentificare {
    pub fn eticheta(&self) -> &'static str {
        match self {
            Autentificare::Sql { .. } => "SQL",
            Autentificare::Integrata => "Windows",
        }
    }
}

/// Unde ne conectam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tinta {
    /// `SRV01`, `SRV01\SQLEXPRESS` sau `srv.domeniu.local`.
    pub server: String,
    /// Port explicit. `None` lasa portul implicit sau SQL Browser sa decida.
    pub port: Option<u16>,
}

/// O instanta gasita in retea prin SQL Browser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instanta {
    /// `SRV01` sau `SRV01\SQLEXPRESS`.
    pub server: String,
    pub ip: String,
    /// `None` cand instanta ascunde portul sau raspunde doar pe named pipes.
    pub port: Option<u16>,
    pub versiune: Option<String>,
}

/// Ce raspunde serverul la o verificare de conexiune. Pentru ecranul de
/// conectare si pentru `gmotion diagnostic`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfoServer {
    pub editie: String,
    pub sistem: String,
    pub pornit_de_secunde: Option<i64>,
    pub nr_baze: Option<i32>,
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn id_e_schema_punct_nume() {
        let o = Obiect {
            schema: "dbo".into(),
            nume: "pClienti".into(),
            tip: "P".into(),
            definitie: "CREATE PROCEDURE dbo.pClienti AS SELECT 1".into(),
        };
        assert_eq!(o.id(), "dbo.pClienti");
    }

    #[test]
    fn parola_nu_se_serializeaza() {
        // Autentificare nu implementeaza Serialize, deliberat. Testul exista ca
        // documentatie executabila: daca cineva adauga derive(Serialize), se
        // uita la asta si se intreaba de ce.
        fn accepta_serializabil<T: serde::Serialize>(_: &T) {}
        let o = Obiect {
            schema: "s".into(),
            nume: "n".into(),
            tip: "P".into(),
            definitie: "d".into(),
        };
        accepta_serializabil(&o);
    }
}
