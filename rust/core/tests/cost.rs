//! Garda de cost pentru motor.
//!
//! Plafon pe **numarul de pasi**, nu pe timp: un test pe timpi trece azi si
//! cade aleator peste o luna, pe o masina incarcata. Pasii sunt deterministi,
//! deci plafonul inseamna ceva.
//!
//! Istoria pe care o pazeste: pana la etapa 20, `scor_similaritate` trecea,
//! pentru fiecare token, prin toate aparitiile lui din celalalt text. Pe SQL,
//! unde `select` si `from` apar de mii de ori, costul crestea cubic — 38 s pe
//! un obiect de 16.000 de linii, 400 s pe 32.000.

use gmotion_core::normalizare::tokeni_scor;
use gmotion_core::seqmatch::SequenceMatcher;

fn obiect(n_blocuri: usize, modifica: bool) -> String {
    let mut s = String::from("CREATE PROCEDURE p AS\nBEGIN\n");
    for i in 0..n_blocuri {
        let v = if modifica && i % 37 == 0 { i + 1 } else { i };
        s.push_str(&format!(
            "SELECT col_{i}, alt_{i} FROM dbo.Tabela_{i} WITH (NOLOCK)\n  WHERE id = {v} AND activ = 1\n  ORDER BY col_{i}\n\n"
        ));
    }
    s.push_str("END\n");
    s
}

/// 8.000 de linii, ~32.000 de token-uri. Bucla din difflib facea aici
/// ~960.000.000 de pasi; automatul de sufixe face sub un milion.
#[test]
fn scorul_pe_un_obiect_mare_nu_redevine_patratic() {
    let a = gmotion_core::pregateste_afisare(&obiect(2000, false));
    let b = gmotion_core::pregateste_afisare(&obiect(2000, true));
    let ta = tokeni_scor(&a);
    let tb = tokeni_scor(&b);
    assert!(ta.len() > 30_000, "cazul cere un obiect mare: {}", ta.len());

    let sm = SequenceMatcher::new(&ta, &tb);
    let potriviri: usize = sm.matching_blocks().iter().map(|x| x.2).sum();
    assert!(potriviri > 0);

    const PLAFON: u64 = 5_000_000;
    assert!(
        sm.operatii() < PLAFON,
        "cautarea a facut {} pasi, peste plafonul de {PLAFON}: a redevenit patratica in aparitii",
        sm.operatii()
    );
}
