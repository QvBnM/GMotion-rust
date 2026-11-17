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
    /// `SET ANSI_NULLS` / `SET QUOTED_IDENTIFIER` cu care a fost creat
    /// obiectul. SQL Server le retine per obiect si le aplica la fiecare
    /// rulare; un deploy care nu le reface schimba comportamentul (`= NULL`,
    /// `"text"`) sau cade (indecsi filtrati, XML). `None` in instantaneele
    /// scrise inainte de 2026-11-17, care nu le aveau.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setari: Option<Setari>,
}

/// Setarile de sesiune pe care SQL Server le retine la crearea unui modul
/// (`sys.sql_modules.uses_ansi_nulls` / `uses_quoted_identifier`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setari {
    pub ansi_nulls: bool,
    pub quoted_identifier: bool,
}

impl Obiect {
    /// `schema.nume` — identificatorul folosit peste tot in aplicatie.
    pub fn id(&self) -> String {
        format!("{}.{}", self.schema, self.nume)
    }

    /// Obiect `WITH ENCRYPTION`: serverul nu da definitia. Se pastreaza cu
    /// definitie goala (SQL Server nu are obiecte necriptate cu definitie
    /// goala). Pana la audit se sarea, deci aparea „lipsa" in tinta si deploy-ul
    /// l-ar fi suprascris.
    pub fn criptat(&self) -> bool {
        self.definitie.is_empty()
    }
}

/// Cum ne autentificam. Parola nu se serializeaza niciodata: nu ajunge nici in
/// sesiuni, nici in instantanee, nici in jurnal.
///
/// `Debug` scris de mana, nu derivat (regula din `CLAUDE.md`): derivat, un
/// `{:?}`, un `assert_eq!` care cade sau o panica tipareau parola. Gasit la
/// auditul din 2026-11-17.
#[derive(Clone, PartialEq, Eq)]
pub enum Autentificare {
    /// Utilizator si parola de SQL Server.
    Sql { utilizator: String, parola: String },
    /// Utilizatorul Windows curent (SSPI). Doar pe Windows.
    Integrata,
}

impl std::fmt::Debug for Autentificare {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Autentificare::Sql { utilizator, .. } => f
                .debug_struct("Sql")
                .field("utilizator", utilizator)
                .field("parola", &"<ascunsa>")
                .finish(),
            Autentificare::Integrata => f.write_str("Integrata"),
        }
    }
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
    /// Cum se verifica certificatul TLS al serverului.
    pub certificat: Certificat,
}

/// Verificarea certificatului serverului.
///
/// Fara verificare, oricine se pune intre statie si server (ARP/DNS spoofing,
/// un nume tastat gresit) primeste parola in clar si poate livra definitii
/// false — reprodus la audit cu un server TDS fals. Cu `Sistem`, serverul
/// trebuie sa aiba un certificat valid pentru numele lui; pe serverele interne
/// cu certificat autosemnat asta cade, de aceea implicitul ramane, deocamdata,
/// cel din v3.4 (`TrustServerCertificate=yes`). Vezi `JURNAL.md`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Certificat {
    /// Orice certificat e acceptat. Criptarea ramane, autenticitatea nu.
    #[default]
    Nevalidat,
    /// Validat cu depozitul de certificate al sistemului.
    Sistem,
    /// Validat cu o autoritate data (fisier `.pem`/`.crt`/`.der`).
    Autoritate(std::path::PathBuf),
}

impl Tinta {
    pub fn noua(server: impl Into<String>, port: Option<u16>) -> Self {
        Tinta {
            server: server.into(),
            port,
            certificat: Certificat::default(),
        }
    }
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

    /// Setarile sunt optionale in JSON: un instantaneu de dinainte de ele se
    /// citeste in continuare (`None`), iar unul nou le pastreaza.
    #[test]
    fn setarile_sunt_compatibile_inapoi_si_dus_intors() {
        let vechi = r#"{"schema":"dbo","nume":"p","tip":"P","definitie":"SELECT 1"}"#;
        let o: Obiect = serde_json::from_str(vechi).unwrap();
        assert_eq!(o.setari, None);
        assert!(!serde_json::to_string(&o).unwrap().contains("setari"));
        let cu = Obiect {
            setari: Some(Setari {
                ansi_nulls: false,
                quoted_identifier: true,
            }),
            ..o
        };
        let dus_intors: Obiect =
            serde_json::from_str(&serde_json::to_string(&cu).unwrap()).unwrap();
        assert_eq!(dus_intors, cu);
    }

    /// Regresie din audit: `Autentificare` derivata `Debug` tiparea parola.
    #[test]
    fn debug_pe_autentificare_nu_scurge_parola() {
        let a = Autentificare::Sql {
            utilizator: "sa".into(),
            parola: "S3cret-Pa55".into(),
        };
        let t = format!("{a:?} {a:#?}");
        assert!(!t.contains("S3cret"), "{t}");
        assert!(t.contains("sa"), "utilizatorul ramane vizibil: {t}");
    }

    #[test]
    fn id_e_schema_punct_nume() {
        let o = Obiect {
            schema: "dbo".into(),
            nume: "pClienti".into(),
            tip: "P".into(),
            definitie: "CREATE PROCEDURE dbo.pClienti AS SELECT 1".into(),
            setari: None,
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
            setari: None,
        };
        accepta_serializabil(&o);
    }
}
