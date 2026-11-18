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

/// Portul pe care asculta serviciul SQL Browser.
const PORT_BROWSER: u16 = 1434;

/// `0x02` = CLNT_BCAST_EX: „toate instantele din retea, raportati-va".
const CERERE_BROADCAST: [u8; 1] = [0x02];

/// Parseaza un raspuns SSRP intr-o lista de instante.
///
/// Formatul: 3 octeti de antet, apoi blocuri `cheie;valoare;...` separate prin
/// `;;`. Cheile care ne intereseaza: `ServerName`, `InstanceName`, `tcp`,
/// `Version`.
///
/// Separat de retea **tocmai** ca sa poata fi testat: raspunsurile reale sunt
/// greu de obtinut, iar formatul are destule capcane (instanta implicita se
/// numeste `MSSQLSERVER` si nu se scrie in nume; `tcp` poate lipsi).
pub fn parseaza_raspuns(date: &[u8], ip: &str) -> Vec<Instanta> {
    // Antetul: 0x05 (SVR_RESP), apoi lungimea continutului pe 2 octeti
    // little-endian. Pana la audit antetul nu se verifica deloc: orice
    // datagrama de pe LAN era parsata ca raspuns.
    if date.len() <= 3 || date[0] != 0x05 {
        return Vec::new();
    }
    // Lungime 0 sau mai mare decat ce a sosit: se ia tot (toleranta pentru
    // implementari neglijente); mai mica: surplusul se ignora.
    let lungime = u16::from_le_bytes([date[1], date[2]]) as usize;
    let sfarsit = if lungime == 0 {
        date.len()
    } else {
        date.len().min(3 + lungime)
    };
    let continut = &date[3..sfarsit];
    let text = String::from_utf8_lossy(continut);
    let mut out = Vec::new();

    for bloc in text.split(";;") {
        let parti: Vec<&str> = bloc.split(';').collect();
        // Valoarea e cea de dupa cheie. Inainte perechile se luau cate doua
        // de la inceput, deci un singur token in plus le decala pe toate si
        // instanta disparea tacut.
        let valoare = |cheie: &str| {
            parti
                .iter()
                .position(|p| p.eq_ignore_ascii_case(cheie))
                .and_then(|i| parti.get(i + 1).copied())
        };
        let nume = valoare("ServerName");
        let instanta = valoare("InstanceName");
        let clusterizat = valoare("IsClustered").is_some_and(|v| v.eq_ignore_ascii_case("Yes"));
        let tcp = valoare("tcp");
        let versiune = valoare("Version");

        let Some(server) = nume.filter(|s| !s.is_empty()) else {
            continue;
        };
        // Numele vin de la oricine de pe retea si ajung pe ecran si in jurnal:
        // un caracter de control ar putea rescrie terminalul.
        if server
            .chars()
            .chain(instanta.unwrap_or("").chars())
            .any(char::is_control)
        {
            continue;
        }

        // Instanta implicita se numeste MSSQLSERVER si NU se scrie in nume:
        // `SRV01`, nu `SRV01\MSSQLSERVER`. Altfel conexiunea ar cauta o
        // instanta numita care nu exista.
        let numita = instanta.is_some_and(|i| !i.is_empty() && i != "MSSQLSERVER");
        let server = if numita {
            format!("{server}\\{}", instanta.unwrap_or(""))
        } else {
            server.to_string()
        };

        out.push(Instanta {
            server,
            ip: ip.to_string(),
            port: tcp.and_then(|p| p.parse().ok()).filter(|p: &u16| *p != 0),
            versiune: versiune.filter(|v| !v.is_empty()).map(str::to_string),
            implicita: !numita,
            cluster: clusterizat,
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
    // Un raspuns SSRP poate avea pana la 64 KB. Cu 4 KB, unul lung se trunchia
    // (port gresit) sau, pe Windows, recv intorcea eroare si oprea tot.
    let mut tampon = vec![0u8; 65_536];

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
            // Timeout: am terminat de ascultat.
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                break
            }
            // Orice alta eroare (datagrama prea mare, ICMP „port unreachable"
            // raportat ca ConnectionReset pe Windows) e a unui singur raspuns,
            // nu a cautarii: o datagrama trimisa de oricine de pe LAN nu are
            // voie sa ascunda restul instantelor.
            Err(e) => {
                tracing::debug!(target: "gmotion::db", eroare = %e, "raspuns SSRP ignorat");
            }
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
        let mut v = vec![0x05];
        v.extend_from_slice(&(corp.len() as u16).to_le_bytes());
        v.extend_from_slice(corp.as_bytes());
        v
    }

    // ---- regresii din auditul de la f561bf9 ----

    #[test]
    fn un_token_in_plus_nu_decaleaza_perechile() {
        let d = raspuns("junk;ServerName;SRV;InstanceName;X;tcp;1500;;");
        let i = parseaza_raspuns(&d, "10.0.0.5");
        assert_eq!(i.len(), 1);
        assert_eq!(i[0].server, "SRV\\X");
        assert_eq!(i[0].port, Some(1500));
    }

    #[test]
    fn datagrame_straine_si_nume_cu_control_se_ignora() {
        let mut fara_antet = raspuns("ServerName;SRV;tcp;1433;;");
        fara_antet[0] = 0x07;
        assert!(parseaza_raspuns(&fara_antet, "1.1.1.1").is_empty());
        let d = raspuns("ServerName;\u{1b}[2J\u{1b}[31mEVIL;tcp;1433;;");
        assert!(parseaza_raspuns(&d, "1.1.1.1").is_empty());
        let d = raspuns("ServerName;SRV;tcp;0;;");
        assert_eq!(
            parseaza_raspuns(&d, "1.1.1.1")[0].port,
            None,
            "portul 0 nu e port"
        );
    }

    #[test]
    fn lungimea_din_antet_taie_surplusul() {
        let mut d = raspuns("ServerName;SRV;tcp;1433;;");
        d.extend_from_slice(b"ServerName;FALS;tcp;1;;");
        let i = parseaza_raspuns(&d, "1.1.1.1");
        assert_eq!(i.len(), 1);
        assert_eq!(i[0].server, "SRV");
    }

    #[test]
    fn instanta_implicita_nu_primeste_sufix() {
        let d = raspuns("ServerName;SRV01;InstanceName;MSSQLSERVER;IsClustered;No;Version;15.0.2000.5;tcp;1433;;");
        let i = parseaza_raspuns(&d, "10.0.0.5");
        assert_eq!(i.len(), 1);
        assert_eq!(i[0].server, "SRV01");
        assert_eq!(i[0].port, Some(1433));
        assert_eq!(i[0].versiune.as_deref(), Some("15.0.2000.5"));
        assert!(i[0].implicita, "MSSQLSERVER inseamna instanta implicita");
        assert!(!i[0].cluster, "IsClustered;No");
    }

    /// Etichetele CLUSTER / IMPLICITA din ecranul de conectare vin de aici,
    /// nu din ghicitul numelui.
    #[test]
    fn eticheta_de_cluster_vine_din_raspuns() {
        let d = raspuns(
            "ServerName;SRV03;InstanceName;SQL2019;IsClustered;Yes;Version;15.0;tcp;1433;;",
        );
        let i = parseaza_raspuns(&d, "10.0.0.7");
        assert_eq!(i[0].server, "SRV03\\SQL2019");
        assert!(i[0].cluster);
        assert!(!i[0].implicita);
    }

    #[test]
    fn instanta_numita_primeste_sufix() {
        let d = raspuns("ServerName;SRV01;InstanceName;SQLEXPRESS;Version;15.0;tcp;49170;;");
        let i = parseaza_raspuns(&d, "10.0.0.5");
        assert_eq!(i[0].server, "SRV01\\SQLEXPRESS");
        assert_eq!(i[0].port, Some(49170));
        assert!(!i[0].implicita);
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
        // port nenumeric: instanta ramane, portul devine None
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
