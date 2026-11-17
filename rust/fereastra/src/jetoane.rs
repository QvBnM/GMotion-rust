//! Jetoanele temei luminoase (`cercetare/design-aprobat.md`), intr-un singur
//! loc: de aici se umple globalul `Tema` din Slint, si de aici se verifica
//! contrastul.
//!
//! **Se strica astfel:** o culoare scrisa de mana in `.slint` divergeaza de
//! cea aprobata (si nimic nu o mai verifica), sau un text ajunge pe un fundal
//! pe care nu se citeste. De-aia culorile nu stau in `.slint`, ci aici, unde
//! testul de contrast le vede.

/// O culoare opaca, `0xRRGGBB`.
pub type Culoare = u32;

// --- suprafete
pub const FUNDAL: Culoare = 0xf5f4f0;
pub const SUPRAFATA: Culoare = 0xfbfaf7;
pub const COD_FUNDAL: Culoare = 0xffffff;
pub const LINIE: Culoare = 0xdedbd3;
pub const LINIE_SLABA: Culoare = 0xebe8e1;

// --- text
pub const TEXT: Culoare = 0x1c1b18;
pub const TEXT_SECUNDAR: Culoare = 0x6b675e;
/// Mai intunecat decat `#9c978b` din macheta: aceea avea 2,9:1 pe fundalul de
/// cod. Decizia lui Mihai (2026-11-17), vezi `JURNAL.md`.
pub const TEXT_TERT: Culoare = 0x716c61;
pub const SELECTIE: Culoare = 0x2b4a6f;

// --- mediul tintei
pub const MEDIU_PROD: Culoare = 0xb3261e;
pub const MEDIU_TEST: Culoare = 0x2b4a6f;
pub const MEDIU_DEV: Culoare = 0x6b675e;

// --- diferente: modificat / adaugat (doar in sursa) / sters (doar in baza)
pub const MOD_TEXT: Culoare = 0x8a5a00;
pub const MOD_FUNDAL: Culoare = 0xf8efd9;
pub const MOD_BARA: Culoare = 0xc48a1a;
pub const MOD_INLINIE: Culoare = 0xecd49a;

pub const ADD_TEXT: Culoare = 0x3d6b35;
pub const ADD_FUNDAL: Culoare = 0xe8f0e0;
pub const ADD_BARA: Culoare = 0x5a8a4c;
pub const ADD_INLINIE: Culoare = 0xc9dfb6;

pub const DEL_TEXT: Culoare = 0xa3392b;
pub const DEL_FUNDAL: Culoare = 0xf8e6e1;
pub const DEL_BARA: Culoare = 0xb8543f;
pub const DEL_INLINIE: Culoare = 0xefc3b8;

/// Hasura de pe partea fara linii (135°, doua culori).
pub const HASURA_A: Culoare = 0xffffff;
pub const HASURA_B: Culoare = 0xebe8e1;

// --- sintaxa
pub const S_CUVANT: Culoare = 0x2a4d8f;
pub const S_SIR: Culoare = 0x7a3e0f;
pub const S_NUMAR: Culoare = 0x7a3e0f;
pub const S_VARIABILA: Culoare = 0x6a3d8f;
/// Mai intunecat decat `#8a857a` din macheta (3,7:1 pe cod). Vezi `TEXT_TERT`.
pub const S_COMENTARIU: Culoare = 0x6a665b;

// --- tipografie
pub const FONT_UI: &str = "IBM Plex Sans";
pub const FONT_MONO: &str = "IBM Plex Mono";

/// Luminanta relativa WCAG a unei culori.
fn luminanta(c: Culoare) -> f64 {
    let canal = |v: u32| {
        let x = v as f64 / 255.0;
        if x <= 0.03928 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * canal((c >> 16) & 0xff) + 0.7152 * canal((c >> 8) & 0xff) + 0.0722 * canal(c & 0xff)
}

/// Raportul de contrast WCAG 2.1 intre doua culori opace (1,0 .. 21,0).
pub fn contrast(a: Culoare, b: Culoare) -> f64 {
    let (la, lb) = (luminanta(a), luminanta(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

#[cfg(test)]
mod teste {
    use super::*;

    /// Minimul WCAG AA pentru text normal.
    const AA: f64 = 4.5;

    /// Fiecare pereche text / fundal pe care o deseneaza editorul.
    /// Daca adaugi o culoare de text, adauga-i si perechile aici.
    const PERECHI: &[(&str, Culoare, Culoare)] = &[
        ("text pe fundal", TEXT, FUNDAL),
        ("text pe suprafata", TEXT, SUPRAFATA),
        ("text pe cod", TEXT, COD_FUNDAL),
        ("text pe bloc modificat", TEXT, MOD_FUNDAL),
        ("text pe bloc adaugat", TEXT, ADD_FUNDAL),
        ("text pe bloc sters", TEXT, DEL_FUNDAL),
        ("text pe in-linie modificat", TEXT, MOD_INLINIE),
        ("text pe in-linie adaugat", TEXT, ADD_INLINIE),
        ("text pe in-linie sters", TEXT, DEL_INLINIE),
        ("secundar pe fundal", TEXT_SECUNDAR, FUNDAL),
        ("secundar pe suprafata", TEXT_SECUNDAR, SUPRAFATA),
        ("secundar pe cod", TEXT_SECUNDAR, COD_FUNDAL),
        ("tert pe cod", TEXT_TERT, COD_FUNDAL),
        ("tert pe suprafata", TEXT_TERT, SUPRAFATA),
        ("tert pe fundal", TEXT_TERT, FUNDAL),
        // in interiorul unui bloc diferit, numarul de linie ia culoarea
        // blocului: pe fundalurile tinte, gri-ul tert ar scadea sub AA fara
        // sa se mai deosebeasca de cel secundar.
        ("numar de linie in bloc modificat", MOD_TEXT, MOD_FUNDAL),
        ("numar de linie in bloc adaugat", ADD_TEXT, ADD_FUNDAL),
        ("numar de linie in bloc sters", DEL_TEXT, DEL_FUNDAL),
        ("suprafata pe selectie", SUPRAFATA, SELECTIE),
        ("suprafata pe butonul principal", SUPRAFATA, TEXT),
        ("PROD pe suprafata", MEDIU_PROD, SUPRAFATA),
        ("PROD pe fundal", MEDIU_PROD, FUNDAL),
        ("TEST pe suprafata", MEDIU_TEST, SUPRAFATA),
        ("DEV pe suprafata", MEDIU_DEV, SUPRAFATA),
        ("modificat pe suprafata", MOD_TEXT, SUPRAFATA),
        ("modificat pe cod", MOD_TEXT, COD_FUNDAL),
        ("adaugat pe suprafata", ADD_TEXT, SUPRAFATA),
        ("sters pe suprafata", DEL_TEXT, SUPRAFATA),
        // sintaxa: pe fundalul de cod si pe fundalurile blocurilor diferite
        ("cuvant cheie pe cod", S_CUVANT, COD_FUNDAL),
        ("cuvant cheie pe bloc modificat", S_CUVANT, MOD_FUNDAL),
        ("cuvant cheie pe in-linie modificat", S_CUVANT, MOD_INLINIE),
        ("cuvant cheie pe in-linie sters", S_CUVANT, DEL_INLINIE),
        ("sir pe cod", S_SIR, COD_FUNDAL),
        ("sir pe in-linie modificat", S_SIR, MOD_INLINIE),
        ("sir pe in-linie sters", S_SIR, DEL_INLINIE),
        ("variabila pe cod", S_VARIABILA, COD_FUNDAL),
        ("variabila pe in-linie sters", S_VARIABILA, DEL_INLINIE),
        ("comentariu pe cod", S_COMENTARIU, COD_FUNDAL),
        ("comentariu pe bloc modificat", S_COMENTARIU, MOD_FUNDAL),
        ("comentariu pe bloc adaugat", S_COMENTARIU, ADD_FUNDAL),
        ("comentariu pe bloc sters", S_COMENTARIU, DEL_FUNDAL),
    ];

    /// Abaterile acceptate, cu motivul. Nu se adauga ca sa treaca testul:
    /// fiecare e o decizie, si decizia sta in `JURNAL.md`.
    ///
    /// Un comentariu are gri; cand chiar caracterele lui sunt cele schimbate,
    /// fundalul de in-linie (galben / verde / roz tare) il duce la ~3,7:1.
    /// Acolo fundalul mai tare e semnal suplimentar peste cel al blocului, nu
    /// fundalul de referinta al textului. Decizia lui Mihai, 2026-11-17.
    const ABATERI: &[(&str, Culoare, Culoare, f64)] = &[
        (
            "comentariu pe in-linie modificat",
            S_COMENTARIU,
            MOD_INLINIE,
            3.5,
        ),
        (
            "comentariu pe in-linie adaugat",
            S_COMENTARIU,
            ADD_INLINIE,
            3.5,
        ),
        (
            "comentariu pe in-linie sters",
            S_COMENTARIU,
            DEL_INLINIE,
            3.5,
        ),
    ];

    #[test]
    fn fiecare_pereche_text_fundal_trece_contrastul_aa() {
        let mut cazute = Vec::new();
        for &(nume, fg, bg) in PERECHI {
            let r = contrast(fg, bg);
            if r < AA {
                cazute.push(format!("{nume}: {r:.2}:1 (minim {AA})"));
            }
        }
        assert!(
            cazute.is_empty(),
            "contrast insuficient:\n{}",
            cazute.join("\n")
        );
    }

    #[test]
    fn abaterile_acceptate_sunt_tot_peste_minimul_notat() {
        for &(nume, fg, bg, minim) in ABATERI {
            let r = contrast(fg, bg);
            assert!(r >= minim, "{nume}: {r:.2}:1, sub {minim} notat in abateri");
            assert!(
                r < AA,
                "{nume}: {r:.2}:1 trece acum AA — scoate-l din abateri"
            );
        }
    }

    #[test]
    fn contrastul_e_calculat_ca_in_wcag() {
        // reperele din specificatie: alb/negru = 21:1, o culoare cu ea insasi = 1:1
        assert!((contrast(0xffffff, 0x000000) - 21.0).abs() < 0.01);
        assert!((contrast(SELECTIE, SELECTIE) - 1.0).abs() < 1e-9);
        // valoarea din macheta, respinsa: text-tert vechi pe fundalul de cod
        assert!((contrast(0x9c978b, COD_FUNDAL) - 2.91).abs() < 0.01);
    }
}
