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

/// Stock_StockProcessDispatchAfterProcess_UpdateItemCost (Mihai,
/// 2026-11-26): comentariile au locul lor.
/// - in blocurile modificate, `-- detail --` / `-- header --` identice stau
///   pe acelasi rand (ieseau o data doar in stanga, o data doar in dreapta);
/// - comentariul lipit de o instructiune e al ei: `/* Situatia de lucrari
///   interna */` intra in blocul `update r` doar din Before (era umplutura,
///   aratata ca neschimbata, iar „Aplica" nu-l aducea);
/// - antetul de documentatie doar din After sta singur, fara sa decaleze
///   liniile de dupa.
#[test]
fn updateitemcost_comentariile_au_locul_lor() {
    let a = pregateste_afisare(include_str!("date/updateitemcost_before.sql"));
    let b = pregateste_afisare(include_str!("date/updateitemcost_after.sql"));
    let al = gmotion_core::aliniaza(&a, &b);
    let r = &al.randuri;
    let text = |l: &Option<gmotion_core::Linie>| l.as_ref().map(|x| x.text.trim().to_string());
    // nicio linie `-- detail --` / `-- header --` singura intr-un bloc
    // modificat
    for x in r.iter().filter(|x| x.tip == TipDiferenta::Modificat) {
        for c in ["-- detail --", "-- header --", "/* Bon de consum */"] {
            let st = text(&x.stanga).as_deref() == Some(c);
            let dr = text(&x.dreapta).as_deref() == Some(c);
            assert!(st == dr, "{c} pe un singur rand: {x:?}");
        }
    }
    // comentariul de sectiune e in blocul doar din Before
    let sectiune = r
        .iter()
        .find(|x| text(&x.stanga).as_deref() == Some("/* Situatia de lucrari interna */"))
        .expect("comentariul e in aliniere");
    assert_eq!(sectiune.tip, TipDiferenta::Sters, "{sectiune:?}");
    assert!(
        sectiune.bloc.is_some() && sectiune.primul_din_bloc,
        "{sectiune:?}"
    );
    // antetul doar din After: randuri doar in dreapta, inainte de CREATE
    let create = r
        .iter()
        .position(|x| text(&x.dreapta).is_some_and(|t| t.starts_with("CREATE PROCEDURE")))
        .expect("CREATE");
    assert!(
        r[..create].iter().all(|x| x.stanga.is_none()),
        "antetul din After e singur"
    );
    assert!(
        text(&r[create].stanga).is_some_and(|t| t.starts_with("create PROCEDURE")),
        "CREATE langa create"
    );
}

/// Liniile dintre blocuri se pun alaturi dupa continut: un comentariu in plus
/// intr-o parte nu mai decaleaza comentariile identice de dupa el.
#[test]
fn umplutura_se_aliniaza_dupa_continut() {
    let a = "SELECT 1 FROM t\n\n-- a\n-- b\n\nSELECT 2 FROM t";
    let b = "SELECT 1 FROM t\n\n-- nou\n-- a\n-- b\n\nSELECT 2 FROM t";
    let al = gmotion_core::aliniaza(a, b);
    let pe_acelasi_rand = |c: &str| {
        al.randuri.iter().any(|x| {
            x.stanga.as_ref().is_some_and(|l| l.text == c)
                && x.dreapta.as_ref().is_some_and(|l| l.text == c)
        })
    };
    assert!(
        pe_acelasi_rand("-- a") && pe_acelasi_rand("-- b"),
        "{:?}",
        al.randuri
    );
    assert!(al
        .randuri
        .iter()
        .any(|x| x.stanga.is_none() && x.dreapta.as_ref().is_some_and(|l| l.text == "-- nou")));
}
