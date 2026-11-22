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

use tiberius::{AuthMethod, Client, Config, Row};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::eroare::{valideaza_baza, valideaza_nume, Eroare, Redactat, Rezultat};
use crate::model::{Autentificare, Certificat, InfoServer, Obiect, Setari, Tinta};
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
        "SELECT s.name, o.name, o.type, m.definition, \
         m.uses_ansi_nulls, m.uses_quoted_identifier, \
         CONVERT(nvarchar(19), o.modify_date, 120) \
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

/// Cat asteptam deschiderea conexiunii: TCP, TLS si login, impreuna.
/// Versiunea Python folosea 30 s. Pana la audit acoperea doar TCP-ul; un
/// server care accepta conexiunea si apoi tace bloca programul la nesfarsit.
const TIMEOUT_CONECTARE: Duration = Duration::from_secs(30);

/// Cat poate dura o interogare. Citirea a ~21.000 de obiecte dureaza ~10 s pe
/// o retea locala; limita e larga, dar exista: un server care moare la
/// jumatatea raspunsului nu mai blocheaza programul.
const TIMEOUT_INTEROGARE: Duration = Duration::from_secs(600);

/// Cat asteptam raspunsul SQL Browser pentru o instanta numita.
const TIMEOUT_BROWSER: Duration = Duration::from_secs(2);

/// Portul pe care asculta SQL Browser.
const PORT_BROWSER: u16 = 1434;

type ClientTds = Client<tokio_util::compat::Compat<TcpStream>>;

pub struct SursaSqlServer {
    runtime: tokio::runtime::Runtime,
    tinta: Tinta,
    auth: Autentificare,
    /// Baza la care e legata conexiunea curenta. `tiberius` leaga clientul de o
    /// baza, deci schimbarea de baza cere reconectare. Dupa orice eroare
    /// conexiunea se arunca: una rupta la jumatatea unui raspuns nu mai e
    /// sigura (raman token-uri necitite).
    conexiune: Option<(String, ClientTds)>,
    /// `TIMEOUT_CONECTARE`, schimbabil doar in teste (un test de 30 s nu).
    timeout_conectare: Duration,
}

impl SursaSqlServer {
    /// Nu deschide nicio conexiune: doar valideaza. Conexiunea se face la
    /// primul apel care are nevoie de ea.
    pub fn nou(tinta: Tinta, auth: Autentificare) -> Rezultat<Self> {
        valideaza_nume(&tinta.server)?;
        if matches!(auth, Autentificare::Integrata) && !auth_integrata_disponibila() {
            return Err(Eroare::AuthIntegrataIndisponibila);
        }
        if let Certificat::Autoritate(cale) = &tinta.certificat {
            if !cale.is_file() {
                return Err(Eroare::Conectare {
                    server: tinta.server.clone(),
                    detaliu: format!("certificatul autoritatii {} nu exista", cale.display()),
                });
            }
        }
        if tinta.certificat == Certificat::Nevalidat {
            tracing::warn!(
                target: "gmotion::db",
                server = %tinta.server,
                "certificatul serverului NU se verifica (ca in v3.4); pe o retea nesigura parola poate fi interceptata. Vezi --certificat"
            );
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
            timeout_conectare: TIMEOUT_CONECTARE,
        })
    }

    /// Cat se asteapta deschiderea conexiunii; implicit 30 s, ca in v3.4.
    /// Vine din `timeout_conectare_s` din configurare (F-B5).
    pub fn cu_timeout_conectare(mut self, d: Duration) -> Self {
        self.timeout_conectare = d;
        self
    }

    fn configuratie(&self, baza: &str, gazda: &str, port: u16) -> Rezultat<Config> {
        valideaza_baza(baza)?;
        let mut c = Config::new();
        // `ADO string` ar fi mai scurt, dar numele de server si baza ar intra
        // intr-un text cu `;` ca separator — exact locul in care un nume
        // ostil poate injecta. Setarea campurilor evita problema.
        c.host(gazda);
        c.port(port);
        c.database(baza);
        match &self.tinta.certificat {
            Certificat::Nevalidat => c.trust_cert(),
            Certificat::Sistem => {}
            Certificat::Autoritate(cale) => c.trust_cert_ca(cale.display()),
        }
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
        self.conexiune = None;
        valideaza_baza(baza)?;
        let server = self.tinta.server.clone();
        let a = interpreteaza_server(&server, self.tinta.port)?;

        tracing::info!(
            target: "gmotion::db",
            %server, %baza, auth = self.auth.eticheta(),
            "deschid conexiunea"
        );

        let client = self.runtime.block_on(async {
            tokio::time::timeout(self.timeout_conectare, async {
                let port = port_tinta(&a, &server).await?;
                let config = self.configuratie(baza, &a.gazda, port)?;
                deschide_client(config, &server, baza).await
            })
            .await
            .map_err(|_| timeout_conectare(&server, self.timeout_conectare))?
        })?;

        self.conexiune = Some((baza.to_string(), client));
        Ok(())
    }

    /// Ruleaza o interogare pe baza data si intoarce primul set de randuri.
    /// Orice eroare (si timeout-ul) arunca conexiunea.
    fn interogheaza(&mut self, baza: &str, sql: &str) -> Rezultat<Vec<Row>> {
        self.conectare(baza)?;
        let Some((_, client)) = self.conexiune.as_mut() else {
            return Err(Eroare::Interogare {
                baza: baza.to_string(),
                detaliu: "conexiunea lipseste dupa conectare".into(),
            });
        };
        let eroare = |detaliu: String| Eroare::Interogare {
            baza: baza.to_string(),
            detaliu,
        };
        let rez = self.runtime.block_on(async {
            tokio::time::timeout(TIMEOUT_INTEROGARE, async {
                client
                    .query(sql, &[])
                    .await
                    .map_err(|e| eroare(e.to_string()))?
                    .into_first_result()
                    .await
                    .map_err(|e| eroare(e.to_string()))
            })
            .await
            .map_err(|_| eroare(format!("timeout dupa {} s", TIMEOUT_INTEROGARE.as_secs())))?
        });
        if rez.is_err() {
            self.conexiune = None;
        }
        rez
    }

    /// Verifica conexiunea si intoarce ce raporteaza serverul despre el insusi.
    /// Pentru ecranul de conectare si pentru `gmotion diagnostic`.
    pub fn verifica(&mut self, baza: &str) -> Rezultat<InfoServer> {
        let randuri = self.interogheaza(baza, INTEROGARE_INFO)?;
        let Some(r) = randuri.first() else {
            return Err(Eroare::Interogare {
                baza: baza.to_string(),
                detaliu: "interogarea de informatii nu a intors niciun rand".into(),
            });
        };
        Ok(InfoServer {
            editie: text(r, 0, baza)?.unwrap_or("necunoscuta").to_string(),
            sistem: text(r, 1, baza)?.unwrap_or("necunoscut").to_string(),
            pornit_de_secunde: intreg(r, 2, baza)?.map(i64::from),
            nr_baze: intreg(r, 3, baza)?,
        })
    }
}

/// O coloana text. `try_get`, nu `get`: `get` da panica daca serverul
/// intoarce alt tip decat cel asteptat, iar serverul (sau cineva intre noi si
/// el) nu e de incredere. Gasit la audit.
fn text<'a>(r: &'a Row, i: usize, baza: &str) -> Rezultat<Option<&'a str>> {
    r.try_get::<&str, _>(i).map_err(|e| Eroare::Interogare {
        baza: baza.to_string(),
        detaliu: format!("coloana {i}: {e}"),
    })
}

fn bit(r: &Row, i: usize, baza: &str) -> Rezultat<Option<bool>> {
    r.try_get::<bool, _>(i).map_err(|e| Eroare::Interogare {
        baza: baza.to_string(),
        detaliu: format!("coloana {i}: {e}"),
    })
}

fn intreg(r: &Row, i: usize, baza: &str) -> Rezultat<Option<i32>> {
    r.try_get::<i32, _>(i).map_err(|e| Eroare::Interogare {
        baza: baza.to_string(),
        detaliu: format!("coloana {i}: {e}"),
    })
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
    fn identitate(&self) -> String {
        format!(
            "{}{}",
            self.tinta.server,
            self.tinta.port.map(|p| format!(":{p}")).unwrap_or_default()
        )
    }

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
        let randuri = self.interogheaza("master", INTEROGARE_BAZE)?;
        let mut out = Vec::with_capacity(randuri.len());
        for r in &randuri {
            if let Some(n) = text(r, 0, "master")? {
                out.push(n.to_string());
            }
        }
        Ok(out)
    }

    fn obiecte(&mut self, baza: &str) -> Rezultat<Vec<Obiect>> {
        let randuri = self.interogheaza(baza, &interogare_obiecte())?;
        obiecte_din_randuri(&randuri, baza)
    }

    /// Pana la `PARALELISM` baze deodata, pe conexiuni separate, ca v3.4
    /// (4 conexiuni). Secvential, pe o retea cu latenta mare (VPN) citirea a
    /// doua baze CharismaERP a durat 48,6 s fata de 28,9 s in v3.4; pe LAN
    /// diferenta nu se vedea (19 s). Ordinea rezultatului e cea ceruta, iar
    /// prima eroare opreste tot (restul conexiunilor se inchid).
    fn obiecte_multe(
        &mut self,
        baze: &[String],
        progres: &mut dyn FnMut(usize, usize, &str),
    ) -> Rezultat<Vec<(String, Vec<Obiect>)>> {
        for b in baze {
            valideaza_baza(b)?;
        }
        let n = baze.len();
        let server = self.tinta.server.clone();
        let a = interpreteaza_server(&server, self.tinta.port)?;
        let timeout = self.timeout_conectare;
        tracing::info!(
            target: "gmotion::db",
            %server, baze = n, paralel = PARALELISM.min(n), auth = self.auth.eticheta(),
            "citesc bazele"
        );
        let rez = self.runtime.block_on(async {
            let port = tokio::time::timeout(timeout, port_tinta(&a, &server))
                .await
                .map_err(|_| timeout_conectare(&server, timeout))??;
            let mut configuratii = Vec::with_capacity(n);
            for b in baze {
                configuratii.push(self.configuratie(b, &a.gazda, port)?);
            }
            let mut configuratii = configuratii.into_iter();

            let mut in_lucru = tokio::task::JoinSet::new();
            let mut rezultate: Vec<Option<Vec<Obiect>>> = (0..n).map(|_| None).collect();
            let mut urmator = 0usize;
            let mut porneste = |in_lucru: &mut tokio::task::JoinSet<_>, i: usize| {
                let (config, baza, server) =
                    (configuratii.next(), baze[i].clone(), server.clone());
                in_lucru.spawn(async move {
                    let Some(config) = config else {
                        return (i, Err(Eroare::Conectare { server, detaliu: "configuratie lipsa".into() }));
                    };
                    (i, citeste_baza(config, &server, &baza, timeout).await)
                });
            };
            while urmator < n && in_lucru.len() < PARALELISM {
                progres(urmator, n, &baze[urmator]);
                porneste(&mut in_lucru, urmator);
                urmator += 1;
            }
            while let Some(gata) = in_lucru.join_next().await {
                let (i, r) = gata.map_err(|e| Eroare::Conectare {
                    server: server.clone(),
                    detaliu: format!("citirea s-a intrerupt: {e}"),
                })?;
                let ob = r?; // prima eroare: JoinSet-ul se arunca, conexiunile se inchid
                tracing::debug!(target: "gmotion::db", baza = %baze[i], nr = ob.len(), "obiecte citite");
                rezultate[i] = Some(ob);
                if urmator < n {
                    progres(urmator, n, &baze[urmator]);
                    porneste(&mut in_lucru, urmator);
                    urmator += 1;
                }
            }
            Ok(rezultate)
        })?;
        progres(n, n, "");
        Ok(baze
            .iter()
            .cloned()
            .zip(rez.into_iter().map(Option::unwrap_or_default))
            .collect())
    }
}

/// Cate baze se citesc deodata. Ca in v3.4.
const PARALELISM: usize = 4;

fn timeout_conectare(server: &str, durata: Duration) -> Eroare {
    Eroare::Conectare {
        server: server.to_string(),
        detaliu: format!(
            "timeout dupa {} s (TCP, TLS si autentificare)",
            durata.as_secs()
        ),
    }
}

/// Portul TCP la care se conecteaza: cel dat, cel spus de SQL Browser pentru
/// o instanta numita, sau 1433.
async fn port_tinta(a: &Adresa, server: &str) -> Rezultat<u16> {
    Ok(match (a.port, &a.instanta) {
        (Some(p), _) => p,
        (None, Some(i)) => rezolva_port_instanta(&a.gazda, PORT_BROWSER, i, server).await?,
        (None, None) => 1433,
    })
}

/// TCP, TLS si login. Timeout-ul il pune apelantul.
async fn deschide_client(config: Config, server: &str, baza: &str) -> Rezultat<ClientTds> {
    let tcp = TcpStream::connect(config.get_addr())
        .await
        .map_err(|e| Eroare::Conectare {
            server: server.to_string(),
            detaliu: e.to_string(),
        })?;
    tcp.set_nodelay(true).ok();
    Client::connect(config, tcp.compat_write())
        .await
        .map_err(|e| clasifica_conectare(server, baza, e))
}

/// O baza, pe o conexiune proprie: deschidere, interogare, conversie.
async fn citeste_baza(
    config: Config,
    server: &str,
    baza: &str,
    timeout: Duration,
) -> Rezultat<Vec<Obiect>> {
    let mut client = tokio::time::timeout(timeout, deschide_client(config, server, baza))
        .await
        .map_err(|_| timeout_conectare(server, timeout))??;
    let eroare = |detaliu: String| Eroare::Interogare {
        baza: baza.to_string(),
        detaliu,
    };
    let sql = interogare_obiecte();
    let randuri = tokio::time::timeout(TIMEOUT_INTEROGARE, async {
        client
            .query(sql.as_str(), &[])
            .await
            .map_err(|e| eroare(e.to_string()))?
            .into_first_result()
            .await
            .map_err(|e| eroare(e.to_string()))
    })
    .await
    .map_err(|_| eroare(format!("timeout dupa {} s", TIMEOUT_INTEROGARE.as_secs())))??;
    obiecte_din_randuri(&randuri, baza)
}

/// Randurile interogarii de obiecte, ca `Obiect`.
fn obiecte_din_randuri(randuri: &[Row], baza: &str) -> Rezultat<Vec<Obiect>> {
    let mut out = Vec::with_capacity(randuri.len());
    for r in randuri {
        let schema = text(r, 0, baza)?.unwrap_or("dbo");
        let nume = text(r, 1, baza)?.unwrap_or_default();
        // Obiect criptat (WITH ENCRYPTION): definitia e NULL. Se pastreaza
        // cu definitie goala, ca sa nu para „lipsa" in tinta — altfel
        // deploy-ul ar suprascrie un obiect criptat al furnizorului.
        let definitie = text(r, 3, baza)?;
        if definitie.is_none() {
            tracing::warn!(
                target: "gmotion::db",
                %baza, obiect = %nume,
                "obiect fara definitie (criptat): nu se compara si nu se deployeaza"
            );
        }
        out.push(Obiect {
            schema: schema.to_string(),
            nume: nume.to_string(),
            tip: text(r, 2, baza)?.unwrap_or_default().trim().to_string(),
            definitie: definitie.unwrap_or_default().to_string(),
            setari: match (bit(r, 4, baza)?, bit(r, 5, baza)?) {
                (Some(ansi_nulls), Some(quoted_identifier)) => Some(Setari {
                    ansi_nulls,
                    quoted_identifier,
                }),
                _ => None,
            },
            modificat: text(r, 6, baza)?.map(str::to_string),
        });
    }
    Ok(out)
}

/// Erorile de la deschiderea conexiunii: parola gresita, baza inaccesibila sau
/// server inaccesibil, fiecare cu codul ei, ca utilizatorul sa stie daca sa
/// verifice reteaua, acreditarile sau numele bazei. Erorile de interogare nu
/// trec pe aici (sunt `GM-DB-004`): pana la audit treceau, si o interogare
/// respinsa aparea ca „nu pot deschide conexiunea", marcata ca reincercabila.
fn clasifica_conectare(server: &str, baza: &str, e: tiberius::error::Error) -> Eroare {
    let cod = match &e {
        tiberius::error::Error::Server(t) => Some(t.code()),
        _ => None,
    };
    match cod {
        Some(18456 | 18452 | 18451) => Eroare::Autentificare {
            server: server.to_string(),
            detaliu: e.to_string(),
        },
        // 4060: „Cannot open database requested by the login": baza nu exista,
        // e offline, sau utilizatorul nu are acces la ea.
        Some(4060) => Eroare::BazaLipsa {
            server: server.to_string(),
            baza: baza.to_string(),
        },
        _ => Eroare::Conectare {
            server: server.to_string(),
            detaliu: e.to_string(),
        },
    }
}

/// Autentificarea integrata Windows (SSPI) exista doar pe Windows. Pe Linux
/// `tiberius` o ofera prin Kerberos/GSSAPI, dar asta ar cere o configurare de
/// domeniu care nu are rost in dezvoltare — deci aici se raporteaza cinstit ca
/// nu e disponibila, in loc sa esueze mai tarziu cu un mesaj obscur.
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

/// Portul unei instante numite, intrebat de la SQL Browser (SSRP, UDP).
///
/// Cod propriu in locul `connect_named` din `tiberius`, din trei motive gasite
/// la audit: (1) un raspuns de 1-2 octeti dadea panica in `tiberius`, deci
/// oricine de pe LAN putea opri aplicatia; (2) socket-ul nu era conectat, deci
/// primul raspuns de la **oricine** alegea portul; (3) nu se verifica numele
/// instantei din raspuns. Aici socket-ul UDP e conectat la gazda intrebata
/// (sistemul arunca datagramele venite de la altcineva), raspunsul trece prin
/// parserul validat din `descoperire`, si se ia doar instanta ceruta.
async fn rezolva_port_instanta(
    gazda: &str,
    port_browser: u16,
    instanta: &str,
    server: &str,
) -> Rezultat<u16> {
    let eroare = |detaliu: String| Eroare::Conectare {
        server: server.to_string(),
        detaliu: format!(
            "{detaliu}. Instanta numita {instanta:?}: portul ei il spune serviciul \
             SQL Browser (UDP {port_browser}). Verifica ca SQL Browser ruleaza pe server \
             si ca instanta exista, sau da portul direct cu --port"
        ),
    };
    let adrese: Vec<std::net::SocketAddr> = tokio::net::lookup_host((gazda, port_browser))
        .await
        .map_err(|e| eroare(format!("nu pot rezolva {gazda}: {e}")))?
        .collect();
    let mut ultima = format!("{gazda} nu are nicio adresa");
    for adr in adrese {
        let local: std::net::SocketAddr = if adr.is_ipv4() {
            (std::net::Ipv4Addr::UNSPECIFIED, 0).into()
        } else {
            (std::net::Ipv6Addr::UNSPECIFIED, 0).into()
        };
        let incercare = async {
            let sock = tokio::net::UdpSocket::bind(local).await?;
            sock.connect(adr).await?;
            let mut cerere = vec![0x04u8];
            cerere.extend_from_slice(instanta.as_bytes());
            sock.send(&cerere).await?;
            let mut tampon = vec![0u8; 65_536];
            let n = sock.recv(&mut tampon).await?;
            tampon.truncate(n);
            Ok::<_, std::io::Error>(tampon)
        };
        match tokio::time::timeout(TIMEOUT_BROWSER, incercare).await {
            Err(_) => {
                ultima = format!(
                    "SQL Browser de la {adr} nu a raspuns in {} s",
                    TIMEOUT_BROWSER.as_secs()
                )
            }
            Ok(Err(e)) => ultima = format!("SQL Browser de la {adr}: {e}"),
            Ok(Ok(raspuns)) => {
                let gasita = crate::descoperire::parseaza_raspuns(&raspuns, &adr.ip().to_string())
                    .into_iter()
                    .find(|i| {
                        i.server
                            .rsplit_once('\\')
                            .is_some_and(|(_, n)| n.eq_ignore_ascii_case(instanta))
                    });
                match gasita.and_then(|i| i.port) {
                    Some(p) => return Ok(p),
                    None => {
                        ultima = format!("raspunsul de la {adr} nu contine instanta cu port TCP")
                    }
                }
            }
        }
    }
    Err(eroare(ultima))
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

    fn sql(server: &str, port: Option<u16>) -> SursaSqlServer {
        SursaSqlServer::nou(
            Tinta::noua(server, port),
            Autentificare::Sql {
                utilizator: "sa".into(),
                parola: "S3cret-Pa55".into(),
            },
        )
        .unwrap()
    }

    #[test]
    fn autoritate_de_certificare_inexistenta_e_refuzata_devreme() {
        let mut tinta = Tinta::noua("srv", None);
        tinta.certificat = Certificat::Autoritate("/nu/exista/ca.pem".into());
        let e = SursaSqlServer::nou(tinta, Autentificare::Integrata).err();
        // pe Linux poate cadea si pe auth integrata; important e ca nu trece
        assert!(e.is_some());
    }

    #[test]
    fn nume_de_server_rupte_dau_eroare_explicata() {
        for (nume, port) in [("srv,abc", None), (r"srv\", None), ("srv,1500", Some(1433))] {
            let e = interpreteaza_server(nume, port).unwrap_err();
            assert_eq!(e.cod(), "GM-DB-001", "{nume:?}");
        }
    }

    fn rt() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    fn raspuns_ssrp(corp: &str) -> Vec<u8> {
        let mut v = vec![5u8];
        v.extend_from_slice(&(corp.len() as u16).to_le_bytes());
        v.extend_from_slice(corp.as_bytes());
        v
    }

    /// SQL Browser fals pe localhost, pe un port efemer. Verifica tot drumul:
    /// intrebarea, raspunsul, portul ales.
    #[test]
    fn instanta_numita_se_rezolva_prin_sql_browser() {
        rt().block_on(async {
            let udp = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let port_browser = udp.local_addr().unwrap().port();
            let browser = tokio::spawn(async move {
                let mut b = [0u8; 256];
                let (n, de) = udp.recv_from(&mut b).await.unwrap();
                let r = "ServerName;TS2409;InstanceName;ALTA;tcp;1111;;\
                         ServerName;TS2409;InstanceName;MIHAIG;IsClustered;No;Version;16.0;tcp;52383;;";
                udp.send_to(&raspuns_ssrp(r), de).await.unwrap();
                b[..n].to_vec()
            });
            let p = rezolva_port_instanta("127.0.0.1", port_browser, "mihaig", "x")
                .await
                .unwrap();
            assert_eq!(p, 52383, "portul instantei cerute, nu al alteia");
            assert_eq!(browser.await.unwrap(), b"\x04mihaig", "intrebarea catre Browser");
        });
    }

    #[test]
    fn browser_care_nu_raspunde_da_mesaj_despre_sql_browser() {
        rt().block_on(async {
            // Socket deschis care nu raspunde niciodata: Browser oprit.
            let mut_udp = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let port = mut_udp.local_addr().unwrap().port();
            let e = rezolva_port_instanta("127.0.0.1", port, "NUEXISTA", "x")
                .await
                .unwrap_err();
            assert_eq!(e.cod(), "GM-DB-001");
            assert!(e.to_string().contains("SQL Browser"), "{e}");
        });
    }

    /// Regresie din audit: un raspuns de 2 octeti dadea panica in `tiberius`
    /// (`&buf[3..len]`), iar release-ul are `panic = "abort"`.
    #[test]
    fn raspuns_scurt_de_la_browser_nu_da_panica() {
        rt().block_on(async {
            let udp = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let port = udp.local_addr().unwrap().port();
            tokio::spawn(async move {
                let mut b = [0u8; 64];
                let (_, de) = udp.recv_from(&mut b).await.unwrap();
                udp.send_to(&[0x05, 0x00], de).await.unwrap();
            });
            let e = rezolva_port_instanta("127.0.0.1", port, "X", "x")
                .await
                .unwrap_err();
            assert_eq!(e.cod(), "GM-DB-001");
        });
    }

    /// Regresie din audit: raspunsul venit de la ALT socket decat cel intrebat
    /// alegea portul (socket neconectat). Acum sistemul il arunca.
    #[test]
    fn raspunsul_de_la_alta_sursa_e_ignorat() {
        rt().block_on(async {
            let browser = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let port = browser.local_addr().unwrap().port();
            let atacator = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
            tokio::spawn(async move {
                let mut b = [0u8; 64];
                let (_, de) = browser.recv_from(&mut b).await.unwrap();
                let r = raspuns_ssrp("ServerName;S;InstanceName;X;tcp;4444;;");
                atacator.send_to(&r, de).await.unwrap();
                // browserul adevarat tace
                tokio::time::sleep(Duration::from_secs(5)).await;
                drop(browser);
            });
            let e = rezolva_port_instanta("127.0.0.1", port, "X", "x")
                .await
                .unwrap_err();
            assert!(e.to_string().contains("nu a raspuns"), "{e}");
        });
    }

    /// Regresie din audit: dupa TCP, nimic nu avea timeout; un server care
    /// accepta conexiunea si tace bloca programul la nesfarsit.
    #[test]
    fn serverul_care_tace_dupa_tcp_da_timeout() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let _tine = std::thread::spawn(move || {
            let _c = listener.accept();
            std::thread::sleep(Duration::from_secs(10));
        });
        // prin `cu_timeout_conectare`, ca din configurare (F-B5)
        let mut s = sql("127.0.0.1", Some(port)).cu_timeout_conectare(Duration::from_secs(1));
        let start = std::time::Instant::now();
        let e = s.baze().unwrap_err();
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "{:?}",
            start.elapsed()
        );
        assert_eq!(e.cod(), "GM-DB-001");
        assert!(e.to_string().contains("timeout"), "{e}");
    }

    /// Citirea paralela: cu un server care tace dupa TCP, patru baze expira
    /// impreuna (o singura asteptare), nu una dupa alta, si progresul a pornit
    /// fiecare baza in ordinea ceruta.
    #[test]
    fn citirea_paralela_expira_impreuna_si_porneste_in_ordine() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let _tine = std::thread::spawn(move || {
            let mut conexiuni = Vec::new();
            for c in listener.incoming().take(8) {
                conexiuni.push(c);
            }
            std::thread::sleep(Duration::from_secs(10));
        });
        let mut s = sql("127.0.0.1", Some(port));
        s.timeout_conectare = Duration::from_secs(1);
        let baze: Vec<String> = ["A", "B", "C", "D"].iter().map(|b| b.to_string()).collect();
        let mut pornite = Vec::new();
        let start = std::time::Instant::now();
        let e = s
            .obiecte_multe(&baze, &mut |i, _, b| pornite.push((i, b.to_string())))
            .unwrap_err();
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "{:?}",
            start.elapsed()
        );
        assert_eq!(e.cod(), "GM-DB-001");
        assert_eq!(
            pornite,
            [
                (0, "A".into()),
                (1, "B".into()),
                (2, "C".into()),
                (3, "D".into())
            ]
        );
    }

    #[test]
    fn citirea_paralela_refuza_numele_invalide_inainte_de_conectare() {
        let mut s = sql("127.0.0.1", Some(1));
        let e = s
            .obiecte_multe(&["ok".into(), "rau\n".into()], &mut |_, _, _| {})
            .unwrap_err();
        assert_eq!(e.cod(), "GM-DB-011");
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
        // „Ultima modificare", ca text fara fus orar: fara tip de data nou
        assert!(q.contains("CONVERT(nvarchar(19), o.modify_date, 120)"));
        assert!(
            q.contains("ORDER BY"),
            "ordinea trebuie stabila intre rulari"
        );
    }

    #[test]
    fn numele_ostile_sunt_refuzate_inainte_de_conectare() {
        for server in ["srv;DROP DATABASE x", "srv'--", "a b"] {
            let e = SursaSqlServer::nou(
                Tinta::noua(server, None),
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
            Tinta::noua("SRV01", None),
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
            Tinta::noua("SRV01", Some(1433)),
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
        let e = SursaSqlServer::nou(Tinta::noua("SRV01", None), Autentificare::Integrata);
        // Pe Linux nu e disponibila: se afla la construire, nu la conectare.
        assert!(e.is_err() || !auth_integrata_disponibila());
    }
}
