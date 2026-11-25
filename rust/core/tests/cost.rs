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
/// 8,5 s la 3.200 de instructiuni (auditul 2026-11-23).
///
/// Cazul greu nu e „tinta are jumatate din blocuri" — acolo blocurile tintei
/// sunt toate imperecheate, deci niciunul nu e candidat si cautarea nu costa
/// nimic. Greu e cand **amandoua** partile au blocuri fara pereche si fara
/// nicio asemanare intre ele: atunci fiecare bloc al martorului se compara cu
/// fiecare bloc liber al tintei. Garda e pe **numarul de comparatii**
/// (`RaportPlasare::comparatii`), nu pe timp.
/// `g` grupuri de forma „o ancora, doua blocuri numai in martor, doua blocuri
/// numai in tinta". Ancorele tin golurile mici (sub `MAX_PRODUS_ALINIERE`,
/// altfel imperecherea cade pozitional si **tot** iese „modificat", deci
/// nimic nu mai e fara pereche si cautarea nu mai costa nimic), dar blocurile
/// fara pereche se adun& peste tot obiectul: `2g` in martor contra `2g` libere
/// in tinta, adica `4g^2` comparatii.
fn fara_nicio_pereche(g: usize) -> (String, String) {
    let mut a = String::new();
    let mut b = String::new();
    for i in 0..g {
        let ancora = format!("SELECT comun_{i} FROM dbo.Comun_{i};\n");
        a.push_str(&ancora);
        b.push_str(&ancora);
        a.push_str(&format!("SELECT coloana_{i} FROM dbo.Tabela_{i};\n"));
        a.push_str(&format!("DELETE FROM dbo.Vechi_{i} WHERE id = {i};\n"));
        b.push_str(&format!("EXEC dbo.procedura_{i} @parametru = {i};\n"));
        b.push_str(&format!("PRINT 'mesajul numarul {i}';\n"));
    }
    (a, b)
}

#[test]
fn plasarile_pe_un_obiect_mare_nu_sunt_patratice() {
    let comparatii = |n: usize| -> usize {
        let (a, b) = fara_nicio_pereche(n);
        let al = gmotion_core::aliniaza_cu(&a, &b, false);
        gmotion_core::plasari_aliniere_cu_raport(&al, &b, false)
            .1
            .comparatii
    };
    let (p60, p120) = (comparatii(60), comparatii(120));
    assert!(p60 > 1000 && p120 > p60, "{p60} -> {p120}");
    // patratic, dar nu mai rau: dublarea obiectului nu inmulteste cu 8
    assert!(
        p120 < p60 * 5,
        "comparatiile cresc mai repede decat patratic: {p60} -> {p120}"
    );
}

/// Plafonul de cost al lui `plasari` **se atinge** si se raporteaza: fara
/// proba asta, plafonul ar fi cod pe care nu l-a vazut nimeni lucrand (B-5 din
/// auditul 2026-11-23-clever-cerf cerea plafonul; un plafon netestat nu apara
/// nimic).
#[test]
fn plafonul_de_cost_al_plasarilor_se_vede_lucrand() {
    let (a, b) = fara_nicio_pereche(250);
    let al = gmotion_core::aliniaza_cu(&a, &b, false);
    let (plasari, raport) = gmotion_core::plasari_aliniere_cu_raport(&al, &b, false);
    assert!(
        raport.degradate > 0,
        "plafonul nu s-a atins: {raport:?} pe {} plasari",
        plasari.len()
    );
    // degradat nu inseamna pierdut: fiecare bloc care difera are tot o plasare
    assert!(
        plasari.len() >= 500,
        "{} plasari, {raport:?}",
        plasari.len()
    );
}

/// Segmentarea unei instructiuni care nu se termina niciodata (`UPDATE` fara
/// `SET`, `MERGE` fara `;`) ramane **liniara**. Pana pe 2026-11-24 starea se
/// recalcula renormalizand instructiunea intreaga la fiecare linie: 94 ms la
/// 500 de linii, 7.020 ms la 4.000. Masura e numarul de normalizari
/// (`Segmentare::normalizari`), nu timpul.
#[test]
fn instructiunea_neterminata_nu_e_patratica() {
    for prefix in [
        "UPDATE dbo.Tinta",
        "MERGE dbo.t AS x USING dbo.s AS y ON x.id = y.id",
    ] {
        let text = |n: usize| -> String {
            let mut s = format!("{prefix}\n");
            for i in 0..n {
                s.push_str(&format!("SELECT {i} FROM dbo.T{i}\n\n"));
            }
            s
        };
        let normalizari = |n: usize| gmotion_core::segmenteaza(&text(n), false).normalizari;
        let (n1, n2) = (normalizari(500), normalizari(1000));
        assert!(n1 > 0 && n2 > 0, "{prefix}: {n1} {n2}");
        // liniar: dublarea liniilor dubleaza normalizarile. Patratic ar da x4.
        assert!(
            n2 < n1 * 3,
            "{prefix}: normalizarile cresc patratic, {n1} -> {n2}"
        );
        // si plafonul taie instructiunea, ca sa nu inghita obiectul
        let s = gmotion_core::segmenteaza(&text(1000), false);
        assert_eq!(s.instructiuni_retezate, 1, "{prefix}");
        assert!(s.blocuri.len() > 100, "{prefix}: {}", s.blocuri.len());
    }
}
