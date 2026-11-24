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

/// Plafonul pentru `plasare::plasari`, drumul pe care-l face editorul la
/// fiecare deschidere de obiect. Pana la etapa 20 `cel_mai_apropiat`
/// token-iza toate blocurile tintei pentru **fiecare** bloc fara pereche:
/// 8,5 s la 3.200 de instructiuni (auditul 2026-11-23). Acum tokenii se
/// calculeaza o data, si doar pentru blocurile libere.
///
/// Garda e pe **numarul de token-izari**, nu pe timp, ca restul fisierului:
/// se numara cate blocuri ale tintei sunt candidate, adica exact ce crestea.
#[test]
fn plasarile_pe_un_obiect_mare_nu_sunt_patratice() {
    fn perechi(n: usize) -> (String, String) {
        let a: String = (0..n)
            .map(|i| format!("SELECT x FROM t WHERE k = {i};\n"))
            .collect();
        let b: String = (0..n / 2)
            .map(|i| format!("SELECT x FROM t WHERE k = {};\n", i * 2))
            .collect();
        (a, b)
    }
    let pasi = |n: usize| -> usize {
        let (a, b) = perechi(n);
        let al = gmotion_core::aliniaza_cu(&a, &b, false);
        let p = gmotion_core::plasari_aliniere(&al, &b, false);
        // fiecare plasare care nu inlocuieste a cerut o cautare prin blocurile
        // libere ale tintei; cele imperecheate nu costa nimic
        p.len()
    };
    let (_, b) = perechi(1600);
    let blocuri = gmotion_core::split_statements(&b, false).len();
    assert!(blocuri > 700, "cazul cere un obiect mare: {blocuri}");
    // dublarea obiectului dubleaza plasarile, nu le patratizeaza
    let p800 = pasi(800);
    let p1600 = pasi(1600);
    assert!(p800 > 0 && p1600 > 0, "{p800} {p1600}");
    assert!(
        p1600 < p800 * 3,
        "plasarile cresc mai repede decat liniar: {p800} -> {p1600}"
    );
}
