//! Marcajul de timp pentru nume de fisiere, antete de script si sesiuni.
//!
//! **Se strica astfel:** doua binare scriu date diferite pentru acelasi
//! moment (de-aia e un singur loc, folosit de `cli` si de fereastra), sau
//! conversia zilelor in data greseste anii bisecti.

/// Marcajul de timp pentru nume de fisiere si antete. Secundele de la epoca,
/// fara dependinta de un crate de date: nu avem nevoie de calendar, doar de
/// ceva monoton si lizibil.
pub fn acum() -> String {
    let s = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Conversie minima la data: suficient pentru un antet de script.
    let zile = s / 86_400;
    let ora = (s % 86_400) / 3600;
    let minut = (s % 3600) / 60;
    let (an, luna, zi) = din_zile(zile);
    format!("{an:04}-{luna:02}-{zi:02} {ora:02}:{minut:02}")
}

/// Zile de la 1970-01-01 -> (an, luna, zi). Algoritmul civil de la Howard
/// Hinnant, forma pentru ere de 400 de ani.
fn din_zile(zile: u64) -> (i64, u32, u32) {
    let z = zile as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

/// Marcaj compact pentru nume de fisiere: `20261117_1030`.
pub fn acum_compact() -> String {
    acum().replace(['-', ':'], "").replace(' ', "_")
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn conversia_de_zile_e_corecta_pe_repere_cunoscute() {
        assert_eq!(din_zile(0), (1970, 1, 1));
        assert_eq!(din_zile(1), (1970, 1, 2));
        assert_eq!(din_zile(365), (1971, 1, 1));
        // 2000 a fost an bisect (divizibil cu 400)
        assert_eq!(din_zile(10_957), (2000, 1, 1));
        assert_eq!(din_zile(11_016), (2000, 2, 29));
        assert_eq!(din_zile(20_819), (2027, 1, 1));
    }

    #[test]
    fn marcajul_de_timp_are_forma_asteptata() {
        let a = acum();
        assert_eq!(a.len(), 16, "{a}");
        assert_eq!(a.as_bytes()[4], b'-');
        assert_eq!(a.as_bytes()[10], b' ');
        assert_eq!(a.as_bytes()[13], b':');
        let c = acum_compact();
        assert_eq!(c.len(), 13, "{c}");
        assert!(c.contains('_'));
        assert!(!c.contains('-') && !c.contains(':') && !c.contains(' '));
    }
}
