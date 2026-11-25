//! Alinierea pe obiecte reale care au aratat-o gresit in fereastra.
//!
//! **Se strica astfel:** o ancora proasta (un `END` legat de alt `END`) taie
//! obiectul in doua: in stanga doar Before, in dreapta doar After, cu goluri
//! in fata, chiar si pe instructiuni identice (BO_Reconcile, Mihai
//! 2026-11-25). Textele sunt cele din proba, in `tests/date/`.

use gmotion_core::{diff_aliniat, pregateste_afisare, TipDiferenta};

fn alinierea(before: &str, after: &str) -> Vec<gmotion_core::RandDiff> {
    diff_aliniat(&pregateste_afisare(before), &pregateste_afisare(after))
}

#[test]
fn bo_reconcile_identicele_si_end_final_sunt_perechi() {
    let r = alinierea(
        include_str!("date/bo_reconcile_before.sql"),
        include_str!("date/bo_reconcile_after.sql"),
    );
    let pereche = |text: &str| {
        r.iter().any(|x| {
            x.tip == TipDiferenta::Egal
                && x.stanga.as_deref().is_some_and(|s| s.contains(text))
                && x.dreapta.as_deref().is_some_and(|s| s.contains(text))
        })
    };
    let tipuri: Vec<&str> = r.iter().map(|x| x.tip.eticheta()).collect();
    assert!(pereche("set @SessionId = newid()"), "{tipuri:?}");
    assert!(pereche("select * from #BOSyncMaster"), "{tipuri:?}");
    // UPDATE-ul care a fost doar rupt pe doua linii e pereche (modificat)
    assert!(
        r.iter().any(|x| x.tip == TipDiferenta::Modificat
            && x.stanga
                .as_deref()
                .is_some_and(|s| s.starts_with("update BOSync_Master"))
            && x.dreapta
                .as_deref()
                .is_some_and(|s| s.starts_with("update BOSync_Master"))),
        "{tipuri:?}"
    );
    // `END`-ul final cu `END`-ul final
    let ultimul = r.last().expect("randuri");
    assert_eq!(ultimul.tip, TipDiferenta::Egal, "{tipuri:?}");
    assert_eq!(ultimul.stanga.as_deref(), Some("end"));
    // si niciun `END` din mijlocul lui Before legat de cel din After
    let end_perechi = r
        .iter()
        .filter(|x| x.stanga.as_deref() == Some("end") && x.dreapta.is_some())
        .count();
    assert_eq!(end_perechi, 1, "{tipuri:?}");
}

/// O vedere care a devenit tabel (EUTransactionType, Mihai 2026-11-25):
/// antetele sunt pereche, deci stau alaturi, nu unul sus si celalalt jos.
#[test]
fn eutransactiontype_antetele_sunt_pereche() {
    let r = alinierea(
        include_str!("date/eutransactiontype_before.sql"),
        include_str!("date/eutransactiontype_after.sql"),
    );
    let tipuri: Vec<&str> = r.iter().map(|x| x.tip.eticheta()).collect();
    let primul = r.first().expect("randuri");
    assert_eq!(primul.tip, TipDiferenta::Modificat, "{tipuri:?}");
    assert!(primul
        .stanga
        .as_deref()
        .is_some_and(|s| s.starts_with("create view")));
    assert!(primul
        .dreapta
        .as_deref()
        .is_some_and(|s| s.starts_with("CREATE TABLE")));
    assert!(
        !r.iter().any(|x| x.tip == TipDiferenta::Adaugat),
        "nimic doar in After: {tipuri:?}"
    );
}
