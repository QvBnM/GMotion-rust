//! Descoperirea instantelor SQL Server in reteaua locala, prin SSRP
//! (SQL Server Resolution Protocol) pe UDP 1434.
//!
//! **Se strica astfel:** un server care exista nu apare in lista. Cauzele sunt
//! aproape mereu de mediu, nu de cod — UDP 1434 blocat de firewall, subnet
//! traversat fara IP helper, instanta cu `Hide Instance = Yes`, sau serviciul
//! SQL Browser oprit. De-aia parsarea raspunsului e separata de trimiterea pe
//! retea: partea care se poate strica din cod e testabila fara retea.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::{Duration, Instant};

use crate::eroare::{Eroare, Rezultat};
use crate::model::Instanta;

/// Portul pe care ascultă serviciul SQL Browser.
const PORT_BROWSER: u16 = 1434;

/// `0x02` = CLNT_BCAST_EX: „toate instantele din retea, raportati-va".
const CERERE_BROADCAST: [u8; 1] = [0x02];

/// Parseaza un raspuns SSRP intr-o lista de instante.
///
/// Formatul: 3 octeti de antet, apoi blocuri `cheie;valoare;...` separate prin
/// `;;`. Cheile care ne interesează: `ServerName`, `InstanceName`, `tcp`,
/// `Version`.
///
/// Separat de retea **tocmai** ca sa poată fi testat: raspunsurile reale sunt
/// greu de obtinut, iar formatul are destule capcane (instanta implicita se
/// numeste `MSSQLSERVER` si nu se scrie in nume; `tcp` poate lipsi).
pub fn parseaza_raspuns(date: &[u8], ip: &str) -> Vec<Instanta> {
    if date.len() <= 3 {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&date[3..]);
    let mut out = Vec::new();

    for bloc in text.split(";;") {
        let parti: Vec<&str> = bloc.split(';').collect();
        let mut nume: Option<&str> = None;
        let mut instanta: Option<&str> = None;
        let mut tcp: Option<&str> = None;
        let mut versiune: Option<&str> = None;

        for par in parti.chunks(2) {
            if par.len() < 2 {
                continue;
            }
            match par[0] {
                "ServerName" => nume = Some(par[1]),
                "InstanceName" => instanta = Some(par[1]),
                "tcp" => tcp = Some(par[1]),
                "Version" => versiune = Some(par[1]),
                _ => {}
            }
        }

        let Some(server) = nume.filter(|s| !s.is_empty()) else {
            continue;
        };

        // Instanta implicita se numeste MSSQLSERVER si NU se scrie in nume:
        // `SRV01`, nu `SRV01\MSSQLSERVER`. Altfel conexiunea ar cauta o
        // instanta numita care nu exista.
        let server = match instanta {
            Some(i) if !i.is_empty() && i != "MSSQLSERVER" => format!("{server}\\{i}"),
            _ => server.to_string(),
        };

        out.push(Instanta {
            server,
            ip: ip.to_string(),
            port: tcp.and_then(|p| p.parse().ok()),
            versiune: versiune.filter(|v| !v.is_empty()).map(str::to_string),
        });
    }
    out
}

/// Cauta instante prin broadcast pe UDP 1434.
///
/// Asteapta pana la `timeout`, strangand toate raspunsurile. Nu e o eroare sa
/// nu gaseasca nimic: intr-o retea cu SQL Browser oprit e rezultatul normal.
/// Duplicatele (acelasi server pe mai multe interfete) se elimina.
pub fn cauta_instante(timeout: Duration) -> Rezultat<Vec<Instanta>> {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).map_err(|e| {
        Eroare::Descoperire {
            detaliu: format!("nu pot deschide socketul UDP: {e}"),
        }
    })?;
    socket
        .set_broadcast(true)
        .map_err(|e| Eroare::Descoperire {
            detaliu: format!("nu pot activa broadcast: {e}"),
        })?;

    socket
        .send_to(
            &CERERE_BROADCAST,
            SocketAddr::from((Ipv4Addr::BROADCAST, PORT_BROWSER)),
        )
        .map_err(|e| Eroare::Descoperire {
            detaliu: format!("nu pot trimite pe UDP {PORT_BROWSER}: {e}"),
        })?;

    let start = Instant::now();
    let mut gasite: Vec<Instanta> = Vec::new();
    let mut tampon = [0u8; 4096];

    loop {
        let scurs = start.elapsed();
        if scurs >= timeout {
            break;
        }
        // Timeout zero inseamna „blocheaza la infinit" pe unele sisteme, deci
        // se asigura un minim.
        let ramas = (timeout - scurs).max(Duration::from_millis(1));
        if socket.set_read_timeout(Some(ramas)).is_err() {
            break;
        }
        match socket.recv_from(&mut tampon) {
            Ok((n, de_la)) => {
                for inst in parseaza_raspuns(&tampon[..n], &de_la.ip().to_string()) {
                    if !gasite.iter().any(|g| g.server == inst.server) {
                        tracing::debug!(target: "gmotion::db", server = %inst.server, ip = %inst.ip, "instanta gasita");
                        gasite.push(inst);
                    }
                }
            }
            // Timeout sau socket intrerupt: am terminat de ascultat.
            Err(_) => break,
        }
    }

    gasite.sort_by(|a, b| a.server.cmp(&b.server));
    tracing::info!(target: "gmotion::db", nr = gasite.len(), "descoperire incheiata");
    Ok(gasite)
}

#[cfg(test)]
mod teste {
    use super::*;

    fn raspuns(corp: &str) -> Vec<u8> {
        let mut v = vec![0x05, 0x00, 0x00];
        v.extend_from_slice(corp.as_bytes());
        v
    }

    #[test]
    fn instanta_implicita_nu_primeste_sufix() {
        let d = raspuns("ServerName;SRV01;InstanceName;MSSQLSERVER;IsClustered;No;Version;15.0.2000.5;tcp;1433;;");
        let i = parseaza_raspuns(&d, "10.0.0.5");
        assert_eq!(i.len(), 1);
        assert_eq!(i[0].server, "SRV01");
        assert_eq!(i[0].port, Some(1433));
        assert_eq!(i[0].versiune.as_deref(), Some("15.0.2000.5"));
    }

    #[test]
    fn instanta_numita_primeste_sufix() {
        let d = raspuns("ServerName;SRV01;InstanceName;SQLEXPRESS;Version;15.0;tcp;49170;;");
        let i = parseaza_raspuns(&d, "10.0.0.5");
        assert_eq!(i[0].server, "SRV01\\SQLEXPRESS");
        assert_eq!(i[0].port, Some(49170));
    }

    #[test]
    fn port_ascuns_nu_e_eroare() {
        let d = raspuns("ServerName;SRV02;InstanceName;MSSQLSERVER;Version;13.0;;");
        let i = parseaza_raspuns(&d, "10.0.0.6");
        assert_eq!(i.len(), 1);
        assert_eq!(i[0].port, None);
    }

    #[test]
    fn mai_multe_instante_in_acelasi_raspuns() {
        let d = raspuns(
            "ServerName;SRV03;InstanceName;MSSQLSERVER;tcp;1433;;\
             ServerName;SRV03;InstanceName;DEV;tcp;49171;;",
        );
        let i = parseaza_raspuns(&d, "10.0.0.7");
        assert_eq!(i.len(), 2);
        assert_eq!(i[1].server, "SRV03\\DEV");
    }

    #[test]
    fn raspunsuri_degenerate_nu_dau_panica() {
        for d in [
            vec![],
            vec![0x05],
            vec![0x05, 0x00, 0x00],
            raspuns(""),
            raspuns(";;;;"),
            raspuns("ServerName"),
            raspuns("ServerName;"),
            raspuns("ServerName;;InstanceName;X;;"),
            raspuns("tcp;nuEnumar;ServerName;SRV;;"),
        ] {
            let _ = parseaza_raspuns(&d, "1.2.3.4");
        }
        // cheie fara valoare, nume gol: nu produce instante fantoma
        assert!(parseaza_raspuns(&raspuns("ServerName;;tcp;1433;;"), "1.2.3.4").is_empty());
        // port nenumeric: instanta rămâne, portul devine None
        let i = parseaza_raspuns(&raspuns("tcp;nuEnumar;ServerName;SRV;;"), "1.2.3.4");
        assert_eq!(i[0].port, None);
    }

    #[test]
    fn octeti_invalizi_utf8_nu_dau_panica() {
        let mut d = vec![0x05, 0x00, 0x00];
        d.extend_from_slice(&[0xff, 0xfe, b';', b';']);
        d.extend_from_slice(b"ServerName;SRV;;");
        let i = parseaza_raspuns(&d, "1.2.3.4");
        assert_eq!(i.len(), 1);
    }
}
