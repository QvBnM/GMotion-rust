//! Jetoanele temei, in doua palete cu aceleasi nume: `INTUNECAT` si
//! `LUMINOS` (`cercetare/schita-slint/`, decizia din 2026-11-18 in
//! `JURNAL.md`). De aici se umple globalul `Tema` din Slint si de aici se
//! verifica contrastul, pe amandoua.
//!
//! **Se strica astfel:** o culoare scrisa de mana in `.slint` divergeaza de
//! paleta (si nimic nu o mai verifica), un text ajunge pe un fundal pe care nu
//! se citeste, sau o paleta primeste un jeton pe care cealalta nu-l are.
//! De-aia culorile nu stau in `.slint`, iar `Paleta` e o singura structura.

/// O culoare opaca, `0xRRGGBB`.
pub type Culoare = u32;

/// Modul ferestrei. Acelasi set de jetoane, alte valori.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mod {
    /// Implicit: stilul hartii, ales primul de Mihai. Butonul de tema il
    /// schimba, iar alegerea se tine minte in configurare.
    #[default]
    Intunecat,
    Luminos,
}

impl Mod {
    pub fn paleta(self) -> &'static Paleta {
        match self {
            Mod::Intunecat => &INTUNECAT,
            Mod::Luminos => &LUMINOS,
        }
    }

    /// `intunecat` / `luminos`, ca la `gmotion-ui --tema`.
    pub fn din_nume(nume: &str) -> Option<Mod> {
        match nume.trim().to_lowercase().as_str() {
            "intunecat" | "întunecat" | "dark" => Some(Mod::Intunecat), // diacritice-permise: date de test
            "luminos" | "light" => Some(Mod::Luminos),
            _ => None,
        }
    }

    pub fn nume(self) -> &'static str {
        match self {
            Mod::Intunecat => "intunecat",
            Mod::Luminos => "luminos",
        }
    }
}

/// Toate culorile unei palete. Starile: `mod_*` modificat, `add_*` doar in
/// sursa (se aduce in baza), `del_*` doar in baza; `*_text` text si numere
/// de linie, `*_fundal` randul, `*_bara` jgheabul si patratul de pe harta,
/// `*_inlinie` caracterele schimbate. `s_*` sintaxa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paleta {
    /// fereastra
    pub fundal: Culoare,
    /// antete, carduri, panouri
    pub suprafata: Culoare,
    /// un nivel deasupra suprafetei: rand peste care treci, buton secundar
    pub strat: Culoare,
    /// zona de cod; panoul Baza din editor
    pub cod_fundal: Culoare,
    /// panoul Sursa din editor (doar citire)
    pub panou_sursa: Culoare,
    /// capul panoului Sursa
    pub cap_sursa: Culoare,
    /// capul panoului Baza (editabil)
    pub cap_baza: Culoare,
    /// conturul panoului Baza
    pub contur_baza: Culoare,
    /// separatoare, contururi
    pub linie: Culoare,
    /// intre randuri
    pub linie_slaba: Culoare,
    /// textul principal
    pub text: Culoare,
    /// etichete, contoare
    pub text_secundar: Culoare,
    /// numere de linie, schema, indicii
    pub text_tert: Culoare,
    /// randul selectat, plin
    pub selectie: Culoare,
    /// textul pe randul selectat
    pub pe_selectie: Culoare,
    /// actiunea principala (chihlimbar)
    pub accent: Culoare,
    /// textul pe accent
    pub pe_accent: Culoare,
    /// text in culoarea accentului (link, „referinta")
    pub accent_text: Culoare,
    /// ce ai scris tu, „editabil"
    pub tine: Culoare,
    /// textul pe eticheta „editabil"
    pub pe_tine: Culoare,
    /// eticheta PROD (fundal)
    pub mediu_prod: Culoare,
    /// eticheta TEST (fundal)
    pub mediu_test: Culoare,
    /// eticheta DEV (fundal)
    pub mediu_dev: Culoare,
    /// textul pe eticheta de mediu
    pub pe_mediu: Culoare,
    pub mod_text: Culoare,
    pub mod_fundal: Culoare,
    pub mod_bara: Culoare,
    pub mod_inlinie: Culoare,
    pub add_text: Culoare,
    pub add_fundal: Culoare,
    pub add_bara: Culoare,
    pub add_inlinie: Culoare,
    pub del_text: Culoare,
    pub del_fundal: Culoare,
    pub del_bara: Culoare,
    pub del_inlinie: Culoare,
    /// obiect care difera doar in setari SET (patratul de pe harta)
    pub set_bara: Culoare,
    /// patrat / bara pentru „identic"
    pub ident: Culoare,
    /// hasura de pe partea fara linii, dunga 1
    pub hasura_a: Culoare,
    /// hasura, dunga 2
    pub hasura_b: Culoare,
    pub s_cuvant: Culoare,
    pub s_sir: Culoare,
    pub s_numar: Culoare,
    pub s_variabila: Culoare,
    pub s_comentariu: Culoare,
}

/// Fondul bleumarin al hartii; starile in culori vii.
pub const INTUNECAT: Paleta = Paleta {
    fundal: 0x0d1120,
    suprafata: 0x151a2e,
    strat: 0x1d2440,
    cod_fundal: 0x0f1424,
    panou_sursa: 0x131930,
    cap_sursa: 0x1a2038,
    cap_baza: 0x221d3d,
    contur_baza: 0x4b3a78,
    linie: 0x2a3255,
    linie_slaba: 0x1e2542,
    text: 0xeef1ff,
    text_secundar: 0xa3acd0,
    text_tert: 0x8a93b8,
    selectie: 0x3a4680,
    pe_selectie: 0xffffff,
    accent: 0xffb547,
    pe_accent: 0x1a1200,
    accent_text: 0xffb547,
    tine: 0xb98cff,
    pe_tine: 0x140a26,
    mediu_prod: 0xd92b2b,
    mediu_test: 0x3561d6,
    mediu_dev: 0x3a4466,
    pe_mediu: 0xffffff,
    mod_text: 0xffb547,
    mod_fundal: 0x312b29,
    mod_bara: 0xffb547,
    mod_inlinie: 0x44372c,
    add_text: 0x3ddc97,
    add_fundal: 0x152e33,
    add_bara: 0x3ddc97,
    add_inlinie: 0x183c3b,
    del_text: 0xff6b81,
    del_fundal: 0x2e1f30,
    del_bara: 0xff6b81,
    del_inlinie: 0x49293a,
    set_bara: 0x8f9bff,
    ident: 0x232a45,
    hasura_a: 0x0f1424,
    hasura_b: 0x1a2038,
    s_cuvant: 0x8fb4ff,
    s_sir: 0xffcf8a,
    s_numar: 0xffcf8a,
    s_variabila: 0xd6a8ff,
    s_comentariu: 0x8a93b8,
};

/// Fondul alb al panzei; starile mai inchise, ca sa treaca contrastul.
pub const LUMINOS: Paleta = Paleta {
    fundal: 0xf4f5f8,
    suprafata: 0xffffff,
    strat: 0xeceef3,
    cod_fundal: 0xffffff,
    panou_sursa: 0xf4f5f8,
    cap_sursa: 0xeceef3,
    cap_baza: 0xf6f0fd,
    contur_baza: 0xd9c6f3,
    linie: 0xe3e6ec,
    linie_slaba: 0xeef0f3,
    text: 0x0f1115,
    text_secundar: 0x4f5561,
    text_tert: 0x5f6572,
    selectie: 0x1c2640,
    pe_selectie: 0xffffff,
    accent: 0xf5a524,
    pe_accent: 0x1a1200,
    accent_text: 0x8f5b00,
    tine: 0x7828c8,
    pe_tine: 0xffffff,
    mediu_prod: 0xc62828,
    mediu_test: 0x2f5fd0,
    mediu_dev: 0x5f6572,
    pe_mediu: 0xffffff,
    mod_text: 0x8f5b00,
    mod_fundal: 0xfdf1dc,
    mod_bara: 0xf5a524,
    mod_inlinie: 0xfbd9a3,
    add_text: 0x0f6a37,
    add_fundal: 0xdcf7e8,
    add_bara: 0x17c964,
    add_inlinie: 0xa7eac4,
    del_text: 0xc40e4a,
    del_fundal: 0xfee5ee,
    del_bara: 0xf31260,
    del_inlinie: 0xfdd0df,
    set_bara: 0x5b5fd6,
    ident: 0xe3e6ec,
    hasura_a: 0xffffff,
    hasura_b: 0xeef0f3,
    s_cuvant: 0x7828c8,
    s_sir: 0xa83709,
    s_numar: 0xa83709,
    s_variabila: 0x005ac6,
    s_comentariu: 0x5f6572,
};
// --- tipografie: Geist si Geist Mono (SIL OFL 1.1, `fonturi/LICENSE-OFL-1.1-Geist.txt`)
pub const FONT_UI: &str = "Geist";
pub const FONT_MONO: &str = "Geist Mono";

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

    type Jeton = fn(&Paleta) -> Culoare;

    /// Fiecare pereche text / fundal pe care o deseneaza ecranele, in ambele
    /// moduri. Daca adaugi o culoare de text, adauga-i si perechile aici.
    const PERECHI: &[(&str, Jeton, Jeton)] = &[
        ("text pe fundal", |p| p.text, |p| p.fundal),
        ("text pe suprafata", |p| p.text, |p| p.suprafata),
        ("text pe strat", |p| p.text, |p| p.strat),
        ("text pe cod", |p| p.text, |p| p.cod_fundal),
        ("text pe panoul sursa", |p| p.text, |p| p.panou_sursa),
        ("text pe capul sursei", |p| p.text, |p| p.cap_sursa),
        ("text pe capul bazei", |p| p.text, |p| p.cap_baza),
        ("text pe bloc modificat", |p| p.text, |p| p.mod_fundal),
        ("text pe bloc adaugat", |p| p.text, |p| p.add_fundal),
        ("text pe bloc sters", |p| p.text, |p| p.del_fundal),
        ("text pe in-linie modificat", |p| p.text, |p| p.mod_inlinie),
        ("text pe in-linie adaugat", |p| p.text, |p| p.add_inlinie),
        ("text pe in-linie sters", |p| p.text, |p| p.del_inlinie),
        ("secundar pe fundal", |p| p.text_secundar, |p| p.fundal),
        (
            "secundar pe suprafata",
            |p| p.text_secundar,
            |p| p.suprafata,
        ),
        ("secundar pe strat", |p| p.text_secundar, |p| p.strat),
        ("secundar pe cod", |p| p.text_secundar, |p| p.cod_fundal),
        (
            "secundar pe panoul sursa",
            |p| p.text_secundar,
            |p| p.panou_sursa,
        ),
        (
            "secundar pe capul sursei",
            |p| p.text_secundar,
            |p| p.cap_sursa,
        ),
        (
            "secundar pe capul bazei",
            |p| p.text_secundar,
            |p| p.cap_baza,
        ),
        ("tert pe fundal", |p| p.text_tert, |p| p.fundal),
        ("tert pe suprafata", |p| p.text_tert, |p| p.suprafata),
        ("tert pe strat", |p| p.text_tert, |p| p.strat),
        ("tert pe cod", |p| p.text_tert, |p| p.cod_fundal),
        ("tert pe panoul sursa", |p| p.text_tert, |p| p.panou_sursa),
        // in interiorul unui bloc diferit, numarul de linie ia culoarea blocului
        (
            "numar de linie in bloc modificat",
            |p| p.mod_text,
            |p| p.mod_fundal,
        ),
        (
            "numar de linie in bloc adaugat",
            |p| p.add_text,
            |p| p.add_fundal,
        ),
        (
            "numar de linie in bloc sters",
            |p| p.del_text,
            |p| p.del_fundal,
        ),
        ("modificat pe suprafata", |p| p.mod_text, |p| p.suprafata),
        ("adaugat pe suprafata", |p| p.add_text, |p| p.suprafata),
        ("sters pe suprafata", |p| p.del_text, |p| p.suprafata),
        ("modificat pe fundal", |p| p.mod_text, |p| p.fundal),
        ("adaugat pe fundal", |p| p.add_text, |p| p.fundal),
        ("sters pe fundal", |p| p.del_text, |p| p.fundal),
        ("text pe selectie", |p| p.pe_selectie, |p| p.selectie),
        ("text pe accent", |p| p.pe_accent, |p| p.accent),
        (
            "accent ca text pe suprafata",
            |p| p.accent_text,
            |p| p.suprafata,
        ),
        ("accent ca text pe fundal", |p| p.accent_text, |p| p.fundal),
        ("text pe PROD", |p| p.pe_mediu, |p| p.mediu_prod),
        ("text pe TEST", |p| p.pe_mediu, |p| p.mediu_test),
        ("text pe DEV", |p| p.pe_mediu, |p| p.mediu_dev),
        ("text pe „editabil”", |p| p.pe_tine, |p| p.tine),
        ("„scris de tine” pe capul bazei", |p| p.tine, |p| p.cap_baza),
        ("„scris de tine” pe suprafata", |p| p.tine, |p| p.suprafata),
        // sintaxa: pe cele doua panouri si pe fundalurile blocurilor diferite
        ("cuvant cheie pe cod", |p| p.s_cuvant, |p| p.cod_fundal),
        (
            "cuvant cheie pe panoul sursa",
            |p| p.s_cuvant,
            |p| p.panou_sursa,
        ),
        (
            "cuvant cheie pe bloc modificat",
            |p| p.s_cuvant,
            |p| p.mod_fundal,
        ),
        (
            "cuvant cheie pe bloc adaugat",
            |p| p.s_cuvant,
            |p| p.add_fundal,
        ),
        (
            "cuvant cheie pe bloc sters",
            |p| p.s_cuvant,
            |p| p.del_fundal,
        ),
        (
            "cuvant cheie pe in-linie modificat",
            |p| p.s_cuvant,
            |p| p.mod_inlinie,
        ),
        (
            "cuvant cheie pe in-linie adaugat",
            |p| p.s_cuvant,
            |p| p.add_inlinie,
        ),
        (
            "cuvant cheie pe in-linie sters",
            |p| p.s_cuvant,
            |p| p.del_inlinie,
        ),
        ("sir pe cod", |p| p.s_sir, |p| p.cod_fundal),
        ("sir pe panoul sursa", |p| p.s_sir, |p| p.panou_sursa),
        ("sir pe bloc modificat", |p| p.s_sir, |p| p.mod_fundal),
        ("sir pe bloc adaugat", |p| p.s_sir, |p| p.add_fundal),
        ("sir pe bloc sters", |p| p.s_sir, |p| p.del_fundal),
        ("sir pe in-linie modificat", |p| p.s_sir, |p| p.mod_inlinie),
        ("sir pe in-linie adaugat", |p| p.s_sir, |p| p.add_inlinie),
        ("sir pe in-linie sters", |p| p.s_sir, |p| p.del_inlinie),
        ("numar pe cod", |p| p.s_numar, |p| p.cod_fundal),
        (
            "numar pe in-linie modificat",
            |p| p.s_numar,
            |p| p.mod_inlinie,
        ),
        ("variabila pe cod", |p| p.s_variabila, |p| p.cod_fundal),
        (
            "variabila pe panoul sursa",
            |p| p.s_variabila,
            |p| p.panou_sursa,
        ),
        (
            "variabila pe bloc modificat",
            |p| p.s_variabila,
            |p| p.mod_fundal,
        ),
        (
            "variabila pe bloc adaugat",
            |p| p.s_variabila,
            |p| p.add_fundal,
        ),
        (
            "variabila pe bloc sters",
            |p| p.s_variabila,
            |p| p.del_fundal,
        ),
        (
            "variabila pe in-linie modificat",
            |p| p.s_variabila,
            |p| p.mod_inlinie,
        ),
        (
            "variabila pe in-linie adaugat",
            |p| p.s_variabila,
            |p| p.add_inlinie,
        ),
        (
            "variabila pe in-linie sters",
            |p| p.s_variabila,
            |p| p.del_inlinie,
        ),
        ("comentariu pe cod", |p| p.s_comentariu, |p| p.cod_fundal),
        (
            "comentariu pe panoul sursa",
            |p| p.s_comentariu,
            |p| p.panou_sursa,
        ),
        (
            "comentariu pe bloc modificat",
            |p| p.s_comentariu,
            |p| p.mod_fundal,
        ),
        (
            "comentariu pe bloc adaugat",
            |p| p.s_comentariu,
            |p| p.add_fundal,
        ),
        (
            "comentariu pe bloc sters",
            |p| p.s_comentariu,
            |p| p.del_fundal,
        ),
    ];

    /// Abaterile acceptate, cu motivul. Nu se adauga ca sa treaca testul:
    /// fiecare e o decizie, si decizia sta in `JURNAL.md`.
    ///
    /// Un comentariu are gri; cand chiar caracterele lui sunt cele schimbate,
    /// fundalul de in-linie il duce sub 4,5:1. Acolo fundalul mai tare e semnal
    /// suplimentar peste cel al blocului, nu fundalul de referinta al textului.
    /// Decizia lui Mihai, 2026-11-17; pragul de 3,5 ramane pe ambele moduri.
    const ABATERI: &[(&str, Jeton, Jeton, f64)] = &[
        (
            "comentariu pe in-linie modificat",
            |p| p.s_comentariu,
            |p| p.mod_inlinie,
            3.5,
        ),
        (
            "comentariu pe in-linie adaugat",
            |p| p.s_comentariu,
            |p| p.add_inlinie,
            3.5,
        ),
        (
            "comentariu pe in-linie sters",
            |p| p.s_comentariu,
            |p| p.del_inlinie,
            3.5,
        ),
    ];

    const MODURI: [(&str, &Paleta); 2] = [("intunecat", &INTUNECAT), ("luminos", &LUMINOS)];

    #[test]
    fn fiecare_pereche_text_fundal_trece_contrastul_aa_in_ambele_moduri() {
        let mut cazute = Vec::new();
        for (m, p) in MODURI {
            for &(nume, fg, bg) in PERECHI {
                let r = contrast(fg(p), bg(p));
                if r < AA {
                    cazute.push(format!("{m}: {nume}: {r:.2}:1 (minim {AA})"));
                }
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
        for (m, p) in MODURI {
            for &(nume, fg, bg, minim) in ABATERI {
                let r = contrast(fg(p), bg(p));
                assert!(
                    r >= minim,
                    "{m}: {nume}: {r:.2}:1, sub {minim} notat in abateri"
                );
            }
        }
    }

    #[test]
    fn contrastul_e_calculat_ca_in_wcag() {
        // reperele din specificatie: alb/negru = 21:1, o culoare cu ea insasi = 1:1
        assert!((contrast(0xffffff, 0x000000) - 21.0).abs() < 0.01);
        assert!((contrast(LUMINOS.selectie, LUMINOS.selectie) - 1.0).abs() < 1e-9);
        // valoarea din macheta veche, respinsa: text-tert pe fundalul de cod
        assert!((contrast(0x9c978b, 0xffffff) - 2.91).abs() < 0.01);
    }

    #[test]
    fn modurile_se_aleg_dupa_nume() {
        assert_eq!(Mod::din_nume("intunecat"), Some(Mod::Intunecat));
        assert_eq!(Mod::din_nume(" Întunecat "), Some(Mod::Intunecat)); // diacritice-permise: date de test
        assert_eq!(Mod::din_nume("luminos"), Some(Mod::Luminos));
        assert_eq!(Mod::din_nume("verde"), None);
        for m in [Mod::Intunecat, Mod::Luminos] {
            assert_eq!(Mod::din_nume(m.nume()), Some(m));
        }
        assert_eq!(Mod::default(), Mod::Intunecat);
        assert_eq!(Mod::Intunecat.paleta(), &INTUNECAT);
    }

    #[test]
    fn cele_doua_palete_chiar_difera() {
        // o paleta copiata din cealalta ar trece toate testele de contrast
        assert_ne!(INTUNECAT.fundal, LUMINOS.fundal);
        assert!(
            luminanta(INTUNECAT.fundal) < 0.05,
            "fondul intunecat nu e intunecat"
        );
        assert!(
            luminanta(LUMINOS.fundal) > 0.8,
            "fondul luminos nu e luminos"
        );
        assert!(luminanta(INTUNECAT.text) > luminanta(INTUNECAT.fundal));
        assert!(luminanta(LUMINOS.text) < luminanta(LUMINOS.fundal));
    }
}
