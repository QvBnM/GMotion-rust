//! Lexer T-SQL minimal: unde e cod, unde e string, comentariu sau identificator.
//!
//! **Se strica astfel:** o regula menita codului (stergerea comentariilor,
//! litere mici, `CREATE` -> `CREATE OR ALTER`) loveste in interiorul unui
//! string sau al unui comentariu. Simptome reale gasite la audit: `'--- x'`
//! facea normalizarea sa stearga tot codul de dupa (modificari raportate
//! „identic"), iar un `CREATE` dintr-un comentariu era rescris in locul celui
//! adevarat.
//!
//! Nu e un parser: recunoaste doar granitele. Pozitiile sunt offset-uri in
//! octeti in textul original, mereu pe granita de caracter.

/// Ce fel de bucata de text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fel {
    /// Cod obisnuit: cuvinte cheie, nume simple, operatori, spatii.
    Cod,
    /// `'...'`, cu `''` ca apostrof in interior. Prefixul `N` ramane in cod.
    Sir,
    /// `[...]`, cu `]]` ca `]` in interior.
    IdentificatorParanteze,
    /// `"..."`, cu `""` in interior. Identificator sau string, dupa
    /// `QUOTED_IDENTIFIER`; oricum, interiorul nu e cod.
    IdentificatorGhilimele,
    /// `-- ...` pana la sfarsitul liniei, fara `\n`.
    ComentariuLinie,
    /// `/* ... */`, cu imbricare, ca in T-SQL.
    ComentariuBloc,
}

/// O bucata: felul si intervalul `[start, sfarsit)` in octeti.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
    pub fel: Fel,
    pub start: usize,
    pub sfarsit: usize,
}

/// Imparte textul in segmente consecutive care il acopera complet.
///
/// Un string, identificator sau comentariu neinchis se intinde pana la final —
/// textul e oricum invalid, si asa nu se trateaza ca cod ce nu e cod.
pub fn segmente(text: &str) -> Vec<Segment> {
    let b = text.as_bytes();
    let n = b.len();
    let mut rez: Vec<Segment> = Vec::new();
    let mut cod_start = 0usize;
    let mut i = 0usize;

    fn inchide_cod(rez: &mut Vec<Segment>, pana_la: usize, cod_start: usize) {
        if pana_la > cod_start {
            rez.push(Segment {
                fel: Fel::Cod,
                start: cod_start,
                sfarsit: pana_la,
            });
        }
    }

    // Toate delimitatorii sunt ASCII, deci cautarea pe octeti e sigura:
    // un octet ASCII nu poate aparea in interiorul unui caracter UTF-8.
    while i < n {
        let (fel, sfarsit) = match b[i] {
            b'-' if i + 1 < n && b[i + 1] == b'-' => {
                let mut j = i + 2;
                while j < n && b[j] != b'\n' {
                    j += 1;
                }
                (Fel::ComentariuLinie, j)
            }
            b'/' if i + 1 < n && b[i + 1] == b'*' => {
                let mut adancime = 1usize;
                let mut j = i + 2;
                while j < n && adancime > 0 {
                    if b[j] == b'/' && j + 1 < n && b[j + 1] == b'*' {
                        adancime += 1;
                        j += 2;
                    } else if b[j] == b'*' && j + 1 < n && b[j + 1] == b'/' {
                        adancime -= 1;
                        j += 2;
                    } else {
                        j += 1;
                    }
                }
                (Fel::ComentariuBloc, j)
            }
            b'\'' => (Fel::Sir, pana_la_inchidere(b, i + 1, b'\'')),
            b'[' => (
                Fel::IdentificatorParanteze,
                pana_la_inchidere(b, i + 1, b']'),
            ),
            b'"' => (
                Fel::IdentificatorGhilimele,
                pana_la_inchidere(b, i + 1, b'"'),
            ),
            _ => {
                i += 1;
                continue;
            }
        };
        inchide_cod(&mut rez, i, cod_start);
        rez.push(Segment {
            fel,
            start: i,
            sfarsit,
        });
        i = sfarsit;
        cod_start = sfarsit;
    }
    inchide_cod(&mut rez, n, cod_start);
    rez
}

/// Pozitia de dupa delimitatorul de inchidere; delimitatorul dublat e caracter
/// in interior (`''`, `]]`, `""`).
fn pana_la_inchidere(b: &[u8], mut j: usize, inchidere: u8) -> usize {
    while j < b.len() {
        if b[j] == inchidere {
            if j + 1 < b.len() && b[j + 1] == inchidere {
                j += 2;
                continue;
            }
            return j + 1;
        }
        j += 1;
    }
    b.len()
}

/// Liniile care contin doar separatorul de batch `GO` (optional cu un numar),
/// in afara comentariilor bloc — exact ce taie `sqlcmd` si SSMS. Atentie:
/// `sqlcmd` taie pe `GO` **si in interiorul unui string** care se intinde pe
/// mai multe linii, deci aici se raporteaza orice linie `GO`, indiferent de
/// context, cu exceptia comentariilor bloc. Numerele de linie sunt 1-based.
pub fn linii_go(text: &str) -> Vec<usize> {
    let seg = segmente(text);
    let in_comentariu_bloc = |poz: usize| {
        seg.iter()
            .any(|s| s.fel == Fel::ComentariuBloc && s.start < poz && poz < s.sfarsit)
    };
    let mut rez = Vec::new();
    let mut poz = 0usize;
    for (nr, linie) in text.split('\n').enumerate() {
        let l = linie.trim_end_matches('\r').trim();
        let mut cuv = l.split_whitespace();
        let e_go = cuv.next().is_some_and(|c| c.eq_ignore_ascii_case("go"))
            && match cuv.next() {
                None => true,
                Some(c) => c.chars().all(|ch| ch.is_ascii_digit()) && cuv.next().is_none(),
            };
        if e_go && !in_comentariu_bloc(poz) {
            rez.push(nr + 1);
        }
        poz += linie.len() + 1;
    }
    rez
}

#[cfg(test)]
mod teste {
    use super::*;

    fn feluri(t: &str) -> Vec<(Fel, &str)> {
        segmente(t)
            .into_iter()
            .map(|s| (s.fel, &t[s.start..s.sfarsit]))
            .collect()
    }

    #[test]
    fn segmentele_acopera_tot_textul_fara_suprapuneri() {
        for t in [
            "",
            "SELECT 1",
            "a 'b' [c] \"d\" -- e\n/* f */ g",
            "'neinchis",
            "/* /* imbricat */ inca */ x",
            "ăîș 'ț' [ș]", // diacritice-permise: date de test
        ] {
            let s = segmente(t);
            let mut poz = 0;
            for x in &s {
                assert_eq!(x.start, poz, "{t:?}");
                assert!(x.sfarsit > x.start, "{t:?}");
                assert!(t.is_char_boundary(x.start) && t.is_char_boundary(x.sfarsit));
                poz = x.sfarsit;
            }
            assert_eq!(poz, t.len(), "{t:?}");
        }
    }

    #[test]
    fn comentariile_din_string_nu_sunt_comentarii() {
        assert_eq!(
            feluri("PRINT '--- start'; DELETE FROM t"),
            vec![
                (Fel::Cod, "PRINT "),
                (Fel::Sir, "'--- start'"),
                (Fel::Cod, "; DELETE FROM t"),
            ]
        );
        assert_eq!(
            feluri("x = '/*' -- c\ny"),
            vec![
                (Fel::Cod, "x = "),
                (Fel::Sir, "'/*'"),
                (Fel::Cod, " "),
                (Fel::ComentariuLinie, "-- c"),
                (Fel::Cod, "\ny"),
            ]
        );
    }

    #[test]
    fn delimitatorii_dublati_raman_in_interior() {
        assert_eq!(feluri("'a''b'")[0], (Fel::Sir, "'a''b'"));
        assert_eq!(feluri("[a]]b]")[0], (Fel::IdentificatorParanteze, "[a]]b]"));
        assert_eq!(
            feluri("\"a\"\"b\"")[0],
            (Fel::IdentificatorGhilimele, "\"a\"\"b\"")
        );
    }

    #[test]
    fn comentariile_bloc_se_imbrica() {
        assert_eq!(
            feluri("/* a /* b */ SELECT 1 */ SELECT 2"),
            vec![
                (Fel::ComentariuBloc, "/* a /* b */ SELECT 1 */"),
                (Fel::Cod, " SELECT 2"),
            ]
        );
    }

    #[test]
    fn go_pe_linie_proprie_inclusiv_in_string() {
        let t = "SELECT 'a\nGO\nb'\n  go 5 \r\n/* \nGO\n */\nGOTO x\nGO";
        assert_eq!(linii_go(t), vec![2, 4, 9]);
    }
}
