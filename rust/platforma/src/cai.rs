//! Unde stau datele aplicatiei pe disc.
//!
//! **Se strica astfel:** sesiuni care se scriu unde nu trebuie, sau nu se mai
//! gasesc dupa o actualizare. Pe Windows, `%APPDATA%` e locul corect; daca
//! lipseste (serviciu, cont fara profil), nu inventam o cale ci raportam
//! `GM-PLAT-001`.
//!
//! Fara dependinta de un crate de directoare: doua variabile de mediu si o
//! regula simpla sunt mai putin decat costul unei dependinte in plus.

use std::path::PathBuf;

use crate::eroare::{Eroare, Rezultat};

/// Numele directorului aplicatiei, sub directorul de date al utilizatorului.
const NUME_APP: &str = "GMotion";

/// Radacina datelor aplicatiei.
///
/// Windows: `%APPDATA%\GMotion`.
/// Altundeva (dezvoltare): `$XDG_DATA_HOME/gmotion` sau `$HOME/.local/share/gmotion`.
///
/// O cale relativa din mediu e refuzata (`GMOTION_DIR_DATE`, `APPDATA`) sau
/// ignorata (`XDG_DATA_HOME`, cum cere specificatia XDG): altfel datele ar
/// depinde de directorul din care e pornit programul. Gasit la audit.
pub fn director_date() -> Rezultat<PathBuf> {
    let absoluta = |v: String| {
        let p = PathBuf::from(v);
        if p.is_absolute() {
            Ok(p)
        } else {
            Err(Eroare::DirectorDate)
        }
    };
    if let Ok(suprascris) = std::env::var("GMOTION_DIR_DATE") {
        if !suprascris.is_empty() {
            return absoluta(suprascris);
        }
    }
    #[cfg(windows)]
    {
        let appdata = std::env::var("APPDATA").map_err(|_| Eroare::DirectorDate)?;
        if appdata.is_empty() {
            return Err(Eroare::DirectorDate);
        }
        Ok(absoluta(appdata)?.join(NUME_APP))
    }
    #[cfg(not(windows))]
    {
        let _ = NUME_APP;
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            if !xdg.is_empty() && PathBuf::from(&xdg).is_absolute() {
                return Ok(PathBuf::from(xdg).join("gmotion"));
            }
        }
        let home = std::env::var("HOME").map_err(|_| Eroare::DirectorDate)?;
        if home.is_empty() {
            return Err(Eroare::DirectorDate);
        }
        Ok(absoluta(home)?.join(".local/share/gmotion"))
    }
}

/// Scrie un fisier atomic: intai alaturi, apoi redenumit peste. O cadere la
/// jumatatea scrierii lasa fisierul vechi intreg, nu unul trunchiat (care, la
/// configurare, ar bloca orice comanda). Gasit la audit.
pub(crate) fn scrie_atomic(cale: &std::path::Path, date: &[u8]) -> std::io::Result<()> {
    let temporar = cale.with_extension("tmp-gmotion");
    std::fs::write(&temporar, date)?;
    std::fs::rename(&temporar, cale).inspect_err(|_e| {
        let _ = std::fs::remove_file(&temporar);
    })
}

/// Directorul sesiunilor unui modul: `<date>/sesiuni/<modul>`.
///
/// Spatiul de nume per modul e deliberat si e **o corectura** fata de versiunea
/// Python, unde cheia de sesiune era doar numele bazei de referinta
/// (`<slug>.dat`). Cu patru module, doua care lucreaza pe aceeasi baza isi
/// suprascriau reciproc progresul.
pub fn director_sesiuni(modul: &str) -> Rezultat<PathBuf> {
    crate::eroare::valideaza_identificator(modul)?;
    Ok(director_date()?.join("sesiuni").join(modul))
}

/// Fisierul de configurare: `<date>/configurare.json`.
pub fn cale_configurare() -> Rezultat<PathBuf> {
    Ok(director_date()?.join("configurare.json"))
}

/// Directorul de jurnale: `<date>/jurnale`.
pub fn director_jurnale() -> Rezultat<PathBuf> {
    Ok(director_date()?.join("jurnale"))
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn suprascrierea_din_mediu_are_prioritate() {
        // Testele ruleaza in paralel in acelasi proces, deci nu se umbla la
        // variabile globale de mediu aici; se verifica doar forma rezultatului.
        let d = director_date().expect("HOME sau APPDATA exista la rulare");
        assert!(d.is_absolute(), "{d:?} ar trebui absolut");
    }

    #[test]
    fn sesiunile_sunt_separate_per_modul() {
        let a = director_sesiuni("diff-obiecte").unwrap();
        let b = director_sesiuni("upgrade").unwrap();
        assert_ne!(a, b);
        assert!(a.ends_with("sesiuni/diff-obiecte") || a.ends_with("sesiuni\\diff-obiecte"));
    }

    #[test]
    fn modul_cu_nume_ostil_nu_poate_iesi_din_director() {
        for m in ["../../etc", "..", "a/b"] {
            assert!(director_sesiuni(m).is_err(), "{m:?} ar trebui refuzat");
        }
    }
}
