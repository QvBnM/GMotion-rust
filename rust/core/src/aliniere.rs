//! Alinierea pe linii a doua texte, si compunerea textului final din ea.
//!
//! **Se strica astfel:** panourile editorului nu se mai aliniaza (blocul din
//! stanga nu e langa perechea lui), sau textul final compus dupa „Aplica"
//! pierde ori dubleaza linii — si atunci deploy-ul scrie in baza alt cod decat
//! cel vazut pe ecran. Editorul (`fereastra`) si compunerea (`diff-obiecte`,
//! deci si `cli`) folosesc **aceeasi** aliniere, de aici: un singur motor.
//!
//! Blocurile, imperecherea si bucatile schimbate vin din [`diff_aliniat`];
//! aici doar se aseaza pe linii, cu liniile dintre blocuri (goale, sau
//! comentarii pe care segmentarea nu le pastreaza) completate din textul
//! original, ca sa se vada tot obiectul.

#[cfg(test)]
use crate::diff::diff_aliniat;
use crate::diff::{RandDiff, TipDiferenta};
use crate::segmentare::{linii_start_blocuri, split_statements};

/// O linie dintr-o parte, cu numarul ei real (de la 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linie {
    pub nr: usize,
    pub text: String,
}

/// Un rand al alinierii: o linie in stanga (referinta / sursa), una in
/// dreapta (tinta / baza), sau umplutura (`None`) intr-una din parti.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandAliniat {
    pub stanga: Option<Linie>,
    pub dreapta: Option<Linie>,
    pub tip: TipDiferenta,
    /// Indexul in [`Aliniere::diff`] al blocului din care face parte randul;
    /// `None` pe liniile dintre blocuri, luate din textul original.
    pub bloc: Option<usize>,
    /// Pozitia liniei in textul blocului (`0` = prima linie a blocului).
    pub linie_in_bloc: usize,
    /// Indexul (de la 0) liniei din dreapta inaintea careia sta randul. Pe un
    /// rand fara linie in dreapta, acolo s-ar insera ce se aduce din stanga.
    pub poz_dreapta: usize,
}

/// Rezultatul alinierii.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aliniere {
    pub randuri: Vec<RandAliniat>,
    /// Blocurile date de motor, in ordine. `RandAliniat::bloc` arata aici.
    pub diff: Vec<RandDiff>,
}

impl Aliniere {
    /// Indexurile (in `diff`) blocurilor care difera, in ordine: ce se poate
    /// aplica.
    pub fn diferente(&self) -> Vec<usize> {
        self.diff
            .iter()
            .enumerate()
            .filter(|(_, d)| d.tip != TipDiferenta::Egal)
            .map(|(i, _)| i)
            .collect()
    }
}

/// Aliniaza doua texte deja pregatite pentru afisare
/// ([`crate::pregateste_afisare`]).
pub fn aliniaza(text_stanga: &str, text_dreapta: &str) -> Aliniere {
    aliniaza_cu(text_stanga, text_dreapta, false)
}

/// Ca [`aliniaza`], dar cu „uneste ciclul cursorului" pornit: blocurile se
/// taie altfel, deci si alinierea. Optiunea trebuie sa ajunga **si** la diff,
/// **si** la inceputurile de blocuri — altfel cele doua n-ar mai vorbi despre
/// aceleasi blocuri.
pub fn aliniaza_cu(text_stanga: &str, text_dreapta: &str, uneste_cursor: bool) -> Aliniere {
    let diff = crate::diff::diff_aliniat_cu(text_stanga, text_dreapta, uneste_cursor);
    let start_st = linii_start_blocuri(text_stanga, &split_statements(text_stanga, uneste_cursor));
    let start_dr =
        linii_start_blocuri(text_dreapta, &split_statements(text_dreapta, uneste_cursor));
    let original_st: Vec<&str> = text_stanga.lines().collect();
    let original_dr: Vec<&str> = text_dreapta.lines().collect();

    let mut randuri: Vec<RandAliniat> = Vec::new();
    let (mut bloc_st, mut bloc_dr) = (0usize, 0usize);
    let (mut ultim_st, mut ultim_dr) = (0usize, 0usize);

    let completeaza = |randuri: &mut Vec<RandAliniat>, s: (usize, usize), d: (usize, usize)| {
        let n = s.1.saturating_sub(s.0).max(d.1.saturating_sub(d.0));
        for i in 0..n {
            let linie = |k: usize, original: &[&str]| Linie {
                nr: k + 1,
                text: original.get(k).copied().unwrap_or("").to_string(),
            };
            let ls = (s.0 + i < s.1).then_some(s.0 + i);
            let ld = (d.0 + i < d.1).then_some(d.0 + i);
            randuri.push(RandAliniat {
                stanga: ls.map(|k| linie(k, &original_st)),
                dreapta: ld.map(|k| linie(k, &original_dr)),
                tip: TipDiferenta::Egal,
                bloc: None,
                linie_in_bloc: 0,
                poz_dreapta: ld.unwrap_or(d.1),
            });
        }
    };

    for (indice, rd) in diff.iter().enumerate() {
        let s_st = rd.stanga.as_ref().map(|_| {
            let s = start_st.get(bloc_st).map_or(ultim_st, |&s| s.max(ultim_st));
            bloc_st += 1;
            s
        });
        let s_dr = rd.dreapta.as_ref().map(|_| {
            let s = start_dr.get(bloc_dr).map_or(ultim_dr, |&s| s.max(ultim_dr));
            bloc_dr += 1;
            s
        });
        completeaza(
            &mut randuri,
            (ultim_st, s_st.unwrap_or(ultim_st)),
            (ultim_dr, s_dr.unwrap_or(ultim_dr)),
        );
        let linii_st: Vec<&str> = rd
            .stanga
            .as_deref()
            .map_or(Vec::new(), |t| t.split('\n').collect());
        let linii_dr: Vec<&str> = rd
            .dreapta
            .as_deref()
            .map_or(Vec::new(), |t| t.split('\n').collect());
        let inceput_dr = s_dr.unwrap_or(ultim_dr);
        for i in 0..linii_st.len().max(linii_dr.len()) {
            let linie = |linii: &[&str], start: Option<usize>| match (linii.get(i), start) {
                (Some(t), Some(s)) => Some(Linie {
                    nr: s + i + 1,
                    text: (*t).to_string(),
                }),
                _ => None,
            };
            let dreapta = linie(&linii_dr, s_dr);
            randuri.push(RandAliniat {
                stanga: linie(&linii_st, s_st),
                poz_dreapta: dreapta
                    .as_ref()
                    .map_or(inceput_dr + linii_dr.len(), |l| l.nr - 1),
                dreapta,
                tip: rd.tip,
                bloc: Some(indice),
                linie_in_bloc: i,
            });
        }
        if let Some(s) = s_st {
            ultim_st = s + linii_st.len();
        }
        if let Some(s) = s_dr {
            ultim_dr = s + linii_dr.len();
        }
    }
    completeaza(
        &mut randuri,
        (ultim_st, original_st.len()),
        (ultim_dr, original_dr.len()),
    );
    Aliniere { randuri, diff }
}

/// Textul din dreapta, cu blocurile pentru care `aplica` spune `true` luate
/// din stanga.
///
/// Se lucreaza pe **liniile originale**, prin numerele lor: ce nu e aplicat
/// ramane octet cu octet ce era in dreapta (cu niciun bloc aplicat, rezultatul
/// e exact `text_dreapta`), iar un bloc aplicat aduce liniile originale din
/// stanga. Textul blocurilor din motor nu se foloseste: nu e garantat subsir
/// al originalului (blocurile unite se lipesc cu un singur `\n`), deci ar
/// pierde sau ar inventa linii.
pub fn compune(
    a: &Aliniere,
    text_stanga: &str,
    text_dreapta: &str,
    aplica: impl Fn(usize) -> bool,
) -> String {
    let original_st: Vec<&str> = text_stanga.lines().collect();
    let original_dr: Vec<&str> = text_dreapta.lines().collect();
    let mut iesire: Vec<&str> = Vec::with_capacity(original_dr.len());
    let mut urmatoarea_dr = 0usize;
    let mut i = 0;
    while i < a.randuri.len() {
        let r = &a.randuri[i];
        let Some(bloc) = r
            .bloc
            .filter(|&b| a.diff[b].tip != TipDiferenta::Egal && aplica(b))
        else {
            i += 1;
            continue;
        };
        // tot blocul, ca interval de randuri
        let j = (i..a.randuri.len())
            .find(|&k| a.randuri[k].bloc != Some(bloc))
            .unwrap_or(a.randuri.len());
        let randuri = &a.randuri[i..j];
        let inceput = randuri
            .iter()
            .filter_map(|r| r.dreapta.as_ref().map(|l| l.nr - 1))
            .min()
            .unwrap_or(r.poz_dreapta);
        let sfarsit = randuri
            .iter()
            .filter_map(|r| r.dreapta.as_ref().map(|l| l.nr))
            .max()
            .unwrap_or(inceput);
        let inceput = inceput.max(urmatoarea_dr);
        iesire.extend_from_slice(
            &original_dr[urmatoarea_dr.min(original_dr.len())..inceput.min(original_dr.len())],
        );
        for l in randuri.iter().filter_map(|r| r.stanga.as_ref()) {
            iesire.push(
                original_st
                    .get(l.nr - 1)
                    .copied()
                    .unwrap_or(l.text.as_str()),
            );
        }
        urmatoarea_dr = sfarsit.max(inceput);
        i = j;
    }
    if urmatoarea_dr < original_dr.len() {
        iesire.extend_from_slice(&original_dr[urmatoarea_dr..]);
    }
    iesire.join("\n")
}

#[cfg(test)]
mod teste {
    use super::*;

    fn bloc_cu_tip(a: &Aliniere, tip: TipDiferenta) -> usize {
        a.diferente()
            .into_iter()
            .find(|&d| a.diff[d].tip == tip)
            .unwrap_or_else(|| panic!("niciun bloc {tip:?}: {:?}", a.diff))
    }

    #[test]
    fn fara_nimic_aplicat_ramane_exact_dreapta() {
        let st = "CREATE PROC p AS\n\n-- antet\nSELECT 1\n\nUPDATE t SET a = 1";
        let dr = "CREATE PROC p AS\n\n\n-- alt antet\nSELECT 2\nUPDATE t SET a = 1\n\n-- final";
        let a = aliniaza(st, dr);
        assert_eq!(compune(&a, st, dr, |_| false), dr);
    }

    #[test]
    fn un_bloc_modificat_aplicat_inlocuieste_doar_blocul() {
        let st = "SELECT 1\n\nUPDATE t\nSET a = 1\n\nDELETE x";
        let dr = "SELECT 1\n\nUPDATE t\nSET a = 2,\n    b = 3\n\nDELETE x";
        let a = aliniaza(st, dr);
        let b = bloc_cu_tip(&a, TipDiferenta::Modificat);
        assert_eq!(
            compune(&a, st, dr, |x| x == b),
            "SELECT 1\n\nUPDATE t\nSET a = 1\n\nDELETE x"
        );
    }

    #[test]
    fn un_bloc_doar_in_stanga_se_insereaza_la_locul_lui() {
        let st = "SELECT 1\n\nINSERT INTO t VALUES (1)\n\nDELETE x";
        let dr = "SELECT 1\n\nDELETE x";
        let a = aliniaza(st, dr);
        let b = bloc_cu_tip(&a, TipDiferenta::Sters);
        let rez = compune(&a, st, dr, |x| x == b);
        assert!(rez.contains("INSERT INTO t VALUES (1)"), "{rez}");
        let (p_select, p_insert, p_delete) = (
            rez.find("SELECT 1").unwrap(),
            rez.find("INSERT").unwrap(),
            rez.find("DELETE").unwrap(),
        );
        assert!(p_select < p_insert && p_insert < p_delete, "ordinea: {rez}");
    }

    #[test]
    fn un_bloc_doar_in_dreapta_aplicat_dispare() {
        let st = "SELECT 1\n\nDELETE x";
        let dr = "SELECT 1\n\nPRINT 'doar aici'\n\nDELETE x";
        let a = aliniaza(st, dr);
        let b = bloc_cu_tip(&a, TipDiferenta::Adaugat);
        let rez = compune(&a, st, dr, |x| x == b);
        assert!(!rez.contains("doar aici"), "{rez}");
        assert!(rez.contains("SELECT 1") && rez.contains("DELETE x"));
    }

    /// Liniile dintre blocuri (goale, comentarii) nu sunt in niciun bloc, deci
    /// raman cum sunt in dreapta; codul, in schimb, e cel din stanga, in
    /// ordinea din stanga. „Aplica toate" din editor pune textul exact al
    /// stangii, nu compunerea asta.
    #[test]
    fn toate_aplicate_dau_codul_din_stanga() {
        let st = "SELECT a\nFROM t\n\nUPDATE t SET x = 1\n\nEXEC p";
        let dr = "SELECT b\nFROM t\n\nPRINT 1\n\nEXEC p";
        let a = aliniaza(st, dr);
        let rez = compune(&a, st, dr, |_| true);
        let cod = |t: &str| {
            t.lines()
                .filter(|l| !l.trim().is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        };
        assert_eq!(cod(&rez), cod(st), "{rez}");
        assert!(diff_aliniat(st, &rez)
            .iter()
            .all(|r| r.tip == TipDiferenta::Egal));
    }

    #[test]
    fn numerele_de_linie_sunt_cele_reale_cu_umplutura() {
        let a = aliniaza(
            "SELECT 1\n\nUPDATE t\nSET a = 1",
            "SELECT 1\n\nUPDATE t\nSET a = 2,\n    b = 3",
        );
        let b = a
            .randuri
            .iter()
            .position(|r| r.tip == TipDiferenta::Modificat)
            .unwrap();
        let bloc: Vec<&RandAliniat> = a.randuri[b..]
            .iter()
            .take_while(|r| r.tip != TipDiferenta::Egal)
            .collect();
        assert_eq!(bloc.len(), 3);
        assert_eq!(bloc[0].stanga.as_ref().map(|l| l.nr), Some(3));
        assert_eq!(bloc[2].stanga, None, "umplutura in stanga");
        assert_eq!(bloc[2].dreapta.as_ref().map(|l| l.nr), Some(5));
        assert_eq!(bloc[2].linie_in_bloc, 2);
    }
}
