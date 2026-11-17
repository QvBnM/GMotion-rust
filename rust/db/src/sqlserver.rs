//! Sursa de obiecte de pe un SQL Server real, prin TDS nativ (`tiberius`).
//!
//! **Se strica astfel:** conexiunea nu se deschide, autentificarea e refuzata,
//! sau interogarea cade. Toate au cod propriu (`GM-DB-001`, `002`, `004`), ca
//! din raportul utilizatorului sa se vada imediat care din cele trei e.
//!
//! Aici se opreste asincronia: `tiberius` e async, dar tipul de aici tine un
//! runtime `tokio` propriu si expune metode sincrone. Vezi `sursa.rs` pentru de
//! ce.
//!
//! **Fara ODBC.** Versiunea Python construia un text de conexiune cu
//! `DRIVER={ODBC Driver 18 for SQL Server}` si cerea driverul instalat pe
//! masina tinta. TDS vorbit direct scoate cerinta. Vezi `JURNAL.md`.

use std::time::Duration;

use tiberius::{AuthMethod, Client, Config, SqlBrowser};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::eroare::{valideaza_nume, Eroare, Redactat, Rezultat};
use crate::model::{Autentificare, InfoServer, Obiect, Tinta};
use crate::sursa::SursaObiecte;

/// Interogarea care aduce obiectele comparabile. Tipurile vin din
/// `gmotion_core::TIPURI_OBIECTE`, ca sa existe o singura definitie a ceea ce
/// „se compara": `P` procedura, `V` view, `FN`/`IF`/`TF` functii, `TR` trigger.
fn interogare_obiecte() -> String {
    let tipuri = gmotion_core::TIPURI_OBIECTE
        .iter()
        .map(|t| format!("'{t}'"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "SELECT s.name, o.name, o.type, m.definition \
         FROM sys.objects o \
         JOIN sys.schemas s ON s.schema_id = o.schema_id \
         JOIN sys.sql_modules m ON m.object_id = o.object_id \
         WHERE o.type IN ({tipuri}) \
         ORDER BY s.name, o.name"
    )
}

/// `database_id > 4` sare peste master, tempdb, model si msdb. `state = 0`
/// inseamna ONLINE: o baza in restaurare ar da eroare la citire.
const INTEROGARE_BAZE: &str =
    "SELECT name FROM sys.databases WHERE database_id > 4 AND state = 0 ORDER BY name";

const INTEROGARE_INFO: &str = "SELECT \
     CAST(SERVERPROPERTY('Edition') AS NVARCHAR(200)), \
     CAST(SERVERPROPERTY('ProductVersion') AS NVARCHAR(200)), \
     DATEDIFF(SECOND, (SELECT sqlserver_start_time FROM sys.dm_os_sys_info), GETDATE()), \
     (SELECT COUNT(*) FROM sys.databases WHERE database_id > 4)";

/// Cat asteptam deschiderea conexiunii. Versiunea Python folosea 30 s.
const TIMEOUT_CONECTARE: Duration = Duration::from_secs(30);

pub struct SursaSqlServer {
    runtime: tokio::runtime::Runtime,
    tinta: Tinta,
    auth: Autentificare,
    /// Baza la care e legata conexiunea curenta. `tiberius` leaga clientul de o
    /// baza, deci schimbarea de baza cere reconectare.
    conexiune: Option<(String, Client<tokio_util::compat::Compat<TcpStream>>)>,
}

impl SursaSqlServer {
    /// Nu deschide nicio conexiune: doar valideaza. Conexiunea se face la
    /// primul apel care are nevoie de ea.
    pub fn nou(tinta: Tinta, auth: Autentificare) -> Rezultat<Self> {
        valideaza_nume(&tinta.server)?;
        if matches!(auth, Autentificare::Integrata) && !auth_integrata_disponibila() {
            return Err(Eroare::AuthIntegrataIndisponibila);
        }
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .map_err(|e| Eroare::Conectare {
                server: tinta.server.clone(),
                detaliu: format!("nu pot porni runtime-ul: {e}"),
            })?;
        Ok(SursaSqlServer {
            runtime,
            tinta,
            auth,
            conexiune: None,
        })
    }

    fn configuratie(&self, baza: &str) -> Rezultat<Config> {
        valideaza_nume(baza)?;
        let mut c = Config::new();
        // `ADO string` ar fi mai scurt, dar numele de server si baza ar intra
        // intr-un text cu `;` ca separator — exact locul in care un nume
        // ostil poate injecta. Setarea campurilor evita problema.
        let a = interpreteaza_server(&self.tinta.server, self.tinta.port)?;
        c.host(&a.gazda);
        match (a.port, &a.instanta) {
            (Some(p), _) => c.port(p),
            // Portul il spune SQL Browser. Cu port dat, Browser-ul nu se intreaba.
            (None, Some(i)) => c.instance_name(i),
            (None, None) => {}
        }
        c.database(baza);
        // Certificate autosemnate sunt norma pe serverele interne, iar
        // versiunea Python folosea TrustServerCertificate=yes. Se pastreaza
        // comportamentul; criptarea rămâne activa.
        c.trust_cert();
        c.authentication(match &self.auth {
            Autentificare::Sql { utilizator, parola } => AuthMethod::sql_server(utilizator, parola),
            Autentificare::Integrata => auth_integrata()?,
        });
        Ok(c)
    }

    fn conectare(&mut self, baza: &str) -> Rezultat<()> {
        if self.conexiune.as_ref().is_some_and(|(b, _)| b == baza) {
            return Ok(());
        }
        let config = self.configuratie(baza)?;
        let server = self.tinta.server.clone();
        let instanta = interpreteaza_server(&server, self.tinta.port)?
            .instanta
            .filter(|_| self.tinta.port.is_none());

        tracing::info!(
            target: "gmotion::db",
            %server, %baza, auth = self.auth.eticheta(),
            "deschid conexiunea"
        );

        let client = self.runtime.block_on(async {
            let tcp = deschide_tcp(&config, &server, instanta.as_deref()).await?;

            Client::connect(config, tcp.compat_write())
                .await
                .map_err(|e| clasifica(&server, e))
        })?;

        self.conexiune = Some((baza.to_string(), client));
        Ok(())
    }

    /// Verifica conexiunea si intoarce ce raporteaza serverul despre el insusi.
    /// Pentru ecranul de conectare si pentru `gmotion diagnostic`.
    pub fn verifica(&mut self, baza: &str) -> Rezultat<InfoServer> {
        self.conectare(baza)?;
        let server = self.tinta.server.clone();
        let (_, client) = self.conexiune.as_mut().expect("conectare a reușit");
        self.runtime.block_on(async {
            let rand = client
                .query(INTEROGARE_INFO, &[])
                .await
                .map_err(|e| clasifica(&server, e))?
                .into_row()
                .await
                .map_err(|e| clasifica(&server, e))?;
            let Some(r) = rand else {
                return Err(Eroare::Interogare {
                    baza: baza.to_string(),
                    detaliu: "interogarea de informatii nu a intors niciun rand".into(),
                });
            };
            Ok(InfoServer {
                editie: r.get::<&str, _>(0).unwrap_or("necunoscuta").to_string(),
                sistem: r.get::<&str, _>(1).unwrap_or("necunoscut").to_string(),
                pornit_de_secunde: r.get::<i32, _>(2).map(i64::from),
                nr_baze: r.get::<i32, _>(3),
            })
        })
    }
}

/// `Debug` scris de mana, nu derivat: un `derive` ar tipari parola la primul
/// `dbg!` sau intr-un mesaj de panica. Tipul tine acreditari, deci nu are voie
/// sa aiba o reprezentare care le scurge.
impl std::fmt::Debug for SursaSqlServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SursaSqlServer")
            .field("server", &self.tinta.server)
            .field("port", &self.tinta.port)
            .field("auth", &self.auth.eticheta())
            .field("conectat_la", &self.conexiune.as_ref().map(|(b, _)| b))
            .finish_non_exhaustive()
    }
}

impl SursaObiecte for SursaSqlServer {
    fn descriere(&self) -> String {
        let parola = match &self.auth {
            Autentificare::Sql { parola, .. } => format!(", parola {}", Redactat(parola)),
            Autentificare::Integrata => String::new(),
        };
        format!(
            "{}{} (auth {}{})",
            self.tinta.server,
            self.tinta.port.map(|p| format!(":{p}")).unwrap_or_default(),
            self.auth.eticheta(),
            parola
        )
    }

    fn baze(&mut self) -> Rezultat<Vec<String>> {
        // `master` exista mereu si nu cere drepturi speciale pentru lista.
        self.conectare("master")?;
        let server = self.tinta.server.clone();
        let (_, client) = self.conexiune.as_mut().expect("conectare a reușit");
        self.runtime.block_on(async {
            let randuri = client
                .query(INTEROGARE_BAZE, &[])
                .await
                .map_err(|e| clasifica(&server, e))?
                .into_first_result()
                .await
                .map_err(|e| clasifica(&server, e))?;
            Ok(randuri
                .iter()
                .filter_map(|r| r.get::<&str, _>(0).map(str::to_string))
                .collect())
        })
    }

    fn obiecte(&mut self, baza: &str) -> Rezultat<Vec<Obiect>> {
        self.conectare(baza)?;
        let interogare = interogare_obiecte();
        let baza_nume = baza.to_string();
        let (_, client) = self.conexiune.as_mut().expect("conectare a reușit");
        self.runtime.block_on(async {
            let randuri = client
                .query(&interogare, &[])
                .await
                .map_err(|e| Eroare::Interogare {
                    baza: baza_nume.clone(),
                    detaliu: e.to_string(),
                })?
                .into_first_result()
                .await
                .map_err(|e| Eroare::Interogare {
                    baza: baza_nume.clone(),
                    detaliu: e.to_string(),
                })?;

            let mut out = Vec::with_capacity(randuri.len());
            for r in &randuri {
                // Un obiect fara definitie (criptat cu WITH ENCRYPTION) se
                // sare: nu se poate compara, si nu e un motiv sa cada tot.
                let Some(definitie) = r.get::<&str, _>(3) else {
                    tracing::warn!(
                        target: "gmotion::db",
                        baza = %baza_nume,
                        obiect = r.get::<&str, _>(1).unwrap_or("?"),
                        "obiect fara definitie, probabil criptat - sarit"
                    );
                    continue;
                };
                out.push(Obiect {
                    schema: r.get::<&str, _>(0).unwrap_or("dbo").to_string(),
                    nume: r.get::<&str, _>(1).unwrap_or_default().to_string(),
                    tip: r.get::<&str, _>(2).unwrap_or_default().trim().to_string(),
                    definitie: definitie.to_string(),
                });
            }
            Ok(out)
        })
    }
}

/// Distinge o parola greșita de un server inaccesibil. Fara asta, ambele ar
/// aparea ca `GM-DB-001` si utilizatorul n-ar sti daca sa verifice reteaua sau
/// acreditarile.
fn clasifica(server: &str, e: tiberius::error::Error) -> Eroare {
    let text = e.to_string();
    let e_auth = matches!(&e, tiberius::error::Error::Server(t)
        if matches!(t.code(), 18456 | 18452 | 4060 | 18451));
    if e_auth {
        Eroare::Autentificare {
            server: server.to_string(),
            detaliu: text,
        }
    } else {
        Eroare::Conectare {
            server: server.to_string(),
            detaliu: text,
        }
    }
}

/// Autentificarea integrata Windows (SSPI) exista doar pe Windows. Pe Linux
/// `tiberius` o ofera prin Kerberos/GSSAPI, dar asta ar cere o configurare de
/// domeniu care nu are rost in dezvoltare — deci aici se raporteaza cinstit ca
/// nu e disponibila, in loc sa eșueze mai tarziu cu un mesaj obscur.
pub fn auth_integrata_disponibila() -> bool {
    cfg!(windows)
}

/// Unde se deschide conexiunea TCP, din ce a scris utilizatorul.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Adresa {
    gazda: String,
    port: Option<u16>,
    instanta: Option<String>,
}

/// Interpreteaza numele de server cum o face si driverul ODBC folosit de
/// versiunea Python: `GAZDA`, `GAZDA\INSTANTA`, `GAZDA,PORT`, iar `.` e
/// calculatorul curent. Alte forme ODBC (`tcp:`, `(local)`, spatii) nu trec de
/// `valideaza_nume`, care ruleaza inainte, in `SursaSqlServer::nou`.
///
/// Fara asta, `SRV01\SQLEXPRESS` ajungea intreg la DNS („No such host is
/// known") — regresie fata de Python, gasita la prima rulare pe o baza reala.
fn interpreteaza_server(server: &str, port: Option<u16>) -> Rezultat<Adresa> {
    let gresit = |detaliu: String| Eroare::Conectare {
        server: server.to_string(),
        detaliu,
    };
    let (rest, port_din_nume) = match server.rsplit_once(',') {
        Some((r, p)) => {
            let p: u16 = p
                .parse()
                .map_err(|_| gresit(format!("portul {p:?} dupa virgula nu e un numar valid")))?;
            (r, Some(p))
        }
        None => (server, None),
    };
    let port = match (port, port_din_nume) {
        (Some(a), Some(b)) if a != b => {
            return Err(gresit(format!(
                "portul e dat de doua ori, diferit: --port {a} si ,{b} in nume"
            )))
        }
        (a, b) => a.or(b),
    };
    let (gazda, instanta) = match rest.split_once('\\') {
        Some((g, "")) => return Err(gresit(format!("lipseste numele instantei dupa {g:?}\\"))),
        Some((g, i)) => (g, Some(i.to_string())),
        None => (rest, None),
    };
    let gazda = match gazda {
        "" | "." => "localhost",
        g => g,
    };
    Ok(Adresa {
        gazda: gazda.to_string(),
        port,
        instanta,
    })
}

/// Deschide socket-ul TCP. Cu instanta numita, portul se afla intai de la SQL
/// Browser (UDP 1434, sau portul din `config` cand e dat explicit).
async fn deschide_tcp(
    config: &Config,
    server: &str,
    instanta: Option<&str>,
) -> Rezultat<TcpStream> {
    let timeout = || Eroare::Conectare {
        server: server.to_string(),
        detaliu: format!("timeout dupa {} s", TIMEOUT_CONECTARE.as_secs()),
    };
    let Some(instanta) = instanta else {
        let tcp = tokio::time::timeout(TIMEOUT_CONECTARE, TcpStream::connect(config.get_addr()))
            .await
            .map_err(|_| timeout())?
            .map_err(|e| Eroare::Conectare {
                server: server.to_string(),
                detaliu: e.to_string(),
            })?;
        tcp.set_nodelay(true).ok();
        return Ok(tcp);
    };
    tokio::time::timeout(TIMEOUT_CONECTARE, TcpStream::connect_named(config))
        .await
        .map_err(|_| timeout())?
        .map_err(|e| Eroare::Conectare {
            server: server.to_string(),
            detaliu: format!(
                "{e}. Instanta numita {instanta:?}: portul ei il spune serviciul \
                 SQL Browser (UDP 1434). Verifica ca SQL Browser ruleaza pe server \
                 si ca instanta exista, sau da portul direct cu --port"
            ),
        })
}

#[cfg(windows)]
fn auth_integrata() -> Rezultat<AuthMethod> {
    Ok(AuthMethod::Integrated)
}

#[cfg(not(windows))]
fn auth_integrata() -> Rezultat<AuthMethod> {
    Err(Eroare::AuthIntegrataIndisponibila)
}

#[cfg(test)]
mod teste {
    use super::*;

    fn adresa(gazda: &str, port: Option<u16>, instanta: Option<&str>) -> Adresa {
        Adresa {
            gazda: gazda.into(),
            port,
            instanta: instanta.map(Into::into),
        }
    }

    #[test]
    fn numele_de_server_se_interpreteaza_ca_in_odbc() {
        let cazuri = [
            ("SRV01", None, adresa("SRV01", None, None)),
            (
                r"TS2409\MIHAIG",
                None,
                adresa("TS2409", None, Some("MIHAIG")),
            ),
            ("srv,1500", None, adresa("srv", Some(1500), None)),
            (
                r"srv\inst,1500",
                None,
                adresa("srv", Some(1500), Some("inst")),
            ),
            (
                r".\SQLEXPRESS",
                None,
                adresa("localhost", None, Some("SQLEXPRESS")),
            ),
            (".", None, adresa("localhost", None, None)),
            ("srv", Some(1433), adresa("srv", Some(1433), None)),
            ("srv,1433", Some(1433), adresa("srv", Some(1433), None)),
        ];
        for (nume, port, asteptat) in cazuri {
            assert_eq!(
                interpreteaza_server(nume, port).unwrap(),
                asteptat,
                "{nume:?}"
            );
        }
    }

    /// Prin `nou`, ca sa treaca si de `valideaza_nume`. `get_addr` arata portul
    /// spre care pleaca primul pachet: 1434 = SQL Browser, deci instanta a
    /// ajuns in configuratie.
    #[test]
    fn configuratia_pentru_instanta_numita_intreaba_sql_browser() {
        let sursa = |server: &str, port| {
            SursaSqlServer::nou(
                Tinta {
                    server: server.into(),
                    port,
                },
                Autentificare::Sql {
                    utilizator: "sa".into(),
                    parola: "x".into(),
                },
            )
            .unwrap()
        };
        let c = sursa(r"TS2409\MIHAIG", None)
            .configuratie("master")
            .unwrap();
        assert_eq!(c.get_addr(), "TS2409:1434");
        let c = sursa(r"TS2409\MIHAIG", Some(1500))
            .configuratie("master")
            .unwrap();
        assert_eq!(c.get_addr(), "TS2409:1500", "cu port dat, direct pe el");
        let c = sursa("srv,1500", None).configuratie("master").unwrap();
        assert_eq!(c.get_addr(), "srv:1500");
        let c = sursa("SRV01", None).configuratie("master").unwrap();
        assert_eq!(c.get_addr(), "SRV01:1433");
    }

    #[test]
    fn nume_de_server_rupte_dau_eroare_explicata() {
        for (nume, port) in [("srv,abc", None), (r"srv\", None), ("srv,1500", Some(1433))] {
            let e = interpreteaza_server(nume, port).unwrap_err();
            assert_eq!(e.cod(), "GM-DB-001", "{nume:?}");
        }
    }

    /// SQL Browser fals pe localhost: raspunde cu portul unui `TcpListener`.
    /// Verifica tot drumul: intrebarea catre Browser, raspunsul, conectarea.
    #[test]
    fn instanta_numita_se_rezolva_prin_sql_browser() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let tcp = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port_tcp = tcp.local_addr().unwrap().port();
            let udp = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let port_browser = udp.local_addr().unwrap().port();
            let browser = tokio::spawn(async move {
                let mut b = [0u8; 256];
                let (n, de) = udp.recv_from(&mut b).await.unwrap();
                let r = format!(
                    "ServerName;TS2409;InstanceName;MIHAIG;IsClustered;No;Version;16.0;tcp;{port_tcp};;"
                );
                let mut msg = vec![5u8, 0, 0];
                msg.extend_from_slice(r.as_bytes());
                udp.send_to(&msg, de).await.unwrap();
                b[..n].to_vec()
            });
            let mut c = Config::new();
            c.host("127.0.0.1");
            c.port(port_browser);
            c.instance_name("MIHAIG");
            let s = deschide_tcp(&c, r"127.0.0.1\MIHAIG", Some("MIHAIG")).await.unwrap();
            assert_eq!(s.peer_addr().unwrap().port(), port_tcp, "portul de la Browser");
            assert_eq!(browser.await.unwrap(), b"\x04MIHAIG", "intrebarea catre Browser");
        });
    }

    #[test]
    fn browser_care_nu_raspunde_da_mesaj_despre_sql_browser() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let mut c = Config::new();
            // Socket deschis care nu raspunde niciodata: Browser oprit.
            let mut_udp = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
            c.host("127.0.0.1");
            c.port(mut_udp.local_addr().unwrap().port());
            c.instance_name("NUEXISTA");
            let e = deschide_tcp(&c, r"127.0.0.1\NUEXISTA", Some("NUEXISTA"))
                .await
                .unwrap_err();
            assert_eq!(e.cod(), "GM-DB-001");
            assert!(e.to_string().contains("SQL Browser"), "{e}");
        });
    }

    #[test]
    fn interogarea_acopera_toate_tipurile_din_core() {
        let q = interogare_obiecte();
        for t in gmotion_core::TIPURI_OBIECTE {
            assert!(
                q.contains(&format!("'{t}'")),
                "tipul {t} lipseste din interogare"
            );
        }
        assert!(q.contains("sys.sql_modules"));
        assert!(
            q.contains("ORDER BY"),
            "ordinea trebuie stabila intre rulari"
        );
    }

    #[test]
    fn numele_ostile_sunt_refuzate_inainte_de_conectare() {
        for server in ["srv;DROP DATABASE x", "srv'--", "a b"] {
            let e = SursaSqlServer::nou(
                Tinta {
                    server: server.into(),
                    port: None,
                },
                Autentificare::Sql {
                    utilizator: "u".into(),
                    parola: "p".into(),
                },
            )
            .unwrap_err();
            assert_eq!(e.cod(), "GM-DB-010", "{server} ar trebui refuzat");
        }
    }

    #[test]
    fn debug_nu_scurge_parola() {
        let s = SursaSqlServer::nou(
            Tinta {
                server: "SRV01".into(),
                port: None,
            },
            Autentificare::Sql {
                utilizator: "sa".into(),
                parola: "parola-secreta".into(),
            },
        )
        .unwrap();
        let t = format!("{s:?}");
        assert!(!t.contains("secreta"), "parola a ajuns in Debug: {t}");
        assert!(t.contains("SRV01"));
    }

    #[test]
    fn descrierea_nu_scurge_parola() {
        let s = SursaSqlServer::nou(
            Tinta {
                server: "SRV01".into(),
                port: Some(1433),
            },
            Autentificare::Sql {
                utilizator: "sa".into(),
                parola: "parola-secreta".into(),
            },
        )
        .unwrap();
        let d = s.descriere();
        assert!(!d.contains("secreta"), "parola a ajuns in descriere: {d}");
        assert!(d.contains("SRV01"));
        assert!(d.contains("1433"));
    }

    #[cfg(not(windows))]
    #[test]
    fn auth_integrata_e_refuzata_devreme_pe_linux() {
        let e = SursaSqlServer::nou(
            Tinta {
                server: "SRV01".into(),
                port: None,
            },
            Autentificare::Integrata,
        );
        // Pe Linux nu e disponibila: se afla la construire, nu la conectare.
        assert!(e.is_err() || !auth_integrata_disponibila());
    }
}
