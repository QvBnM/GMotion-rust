//! Diff-ul unui obiect, calculat la cerere.
//!
//! **Se strica astfel:** diff-ul nu corespunde cu ce arata lista — un obiect
//! raportat ca „modificat" care se deschide si arata identic, sau invers. In
//! versiunea Python asta se putea intampla **usor**, fiindca lista venea din
//! `motor.py` si editorul din portul JS, iar cele doua divergeau. Aici amandoua
//! cheama acelasi `gmotion_core`.
//!
//! La cerere, nu la comparare: diff-ul complet pe mii de obiecte ar fi scump si
//! aproape tot ar fi aruncat. Compararea foloseste doar hash si scor.
//!
//! **`id` nu e pozitia in lista.** `gmotion_core` da fiecarui rand un id stabil
//! intre recalculari: pentru randurile ancorate in referinta e pozitia blocului
//! in referinta, iar cele doar-adaugate primesc id-uri negative. Asta e cheia
//! prin care se tin minte deciziile utilizatorului („linia asta o iau, asta
//! nu") si supravietuiesc unei recalculari. Daca s-ar folosi indexul din lista,
//! orice schimbare in tinta ar muta deciziile pe alte linii.

use gmotion_core::{diff_aliniat, rezumat_diff, FelSchimbare, RandDiff, TipDiferenta};
use serde::{Deserialize, Serialize};

/// Cum e marcata o bucata de text in interiorul unei linii.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Marcaj {
    Sters,
    Adaugat,
}

impl Marcaj {
    fn din(f: FelSchimbare) -> Self {
        match f {
            FelSchimbare::Sters => Marcaj::Sters,
            FelSchimbare::Adaugat => Marcaj::Adaugat,
        }
    }
}

/// O bucata de text dintr-o linie, marcata sau nu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bucata {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marcaj: Option<Marcaj>,
}

/// Ce s-a intamplat cu un rand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Fel {
    Egal,
    Modificat,
    Sters,
    Adaugat,
}

impl Fel {
    fn din(t: TipDiferenta) -> Self {
        match t {
            TipDiferenta::Egal => Fel::Egal,
            TipDiferenta::Modificat => Fel::Modificat,
            TipDiferenta::Sters => Fel::Sters,
            TipDiferenta::Adaugat => Fel::Adaugat,
        }
    }

    pub fn eticheta(self) -> &'static str {
        match self {
            Fel::Egal => "egal",
            Fel::Modificat => "modificat",
            Fel::Sters => "sters",
            Fel::Adaugat => "adaugat",
        }
    }
}

/// Un rand de diff, in forma in care il consuma un ecran sau linia de comanda.
///
/// Tipurile proprii, nu cele din `gmotion_core`: `core` nu depinde de `serde`
/// (trebuie sa ramana minim, fiindca merge si pe wasm), iar forma serializata e
/// o decizie a modulului, nu a motorului.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diferenta {
    /// Id stabil intre recalculari. **Nu** indexul din lista.
    pub id: i64,
    pub fel: Fel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stanga: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dreapta: Option<String>,
    /// Bucatile liniei din stanga, cand exista marcaje in interiorul liniei.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucati_stanga: Option<Vec<Bucata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucati_dreapta: Option<Vec<Bucata>>,
}

fn bucati(segmente: &Option<Vec<gmotion_core::Segment>>) -> Option<Vec<Bucata>> {
    segmente.as_ref().map(|v| {
        v.iter()
            .map(|s| Bucata {
                text: s.text.clone(),
                marcaj: s.schimbat.map(Marcaj::din),
            })
            .collect()
    })
}

impl Diferenta {
    fn din(r: &RandDiff) -> Self {
        Diferenta {
            id: r.id,
            fel: Fel::din(r.tip),
            stanga: r.stanga.clone(),
            dreapta: r.dreapta.clone(),
            bucati_stanga: bucati(&r.seg_stanga),
            bucati_dreapta: bucati(&r.seg_dreapta),
        }
    }
}

/// Diferentele dintre doua versiuni ale aceluiasi obiect, plus rezumatul.
///
/// `referinta` si `curent` sunt forme deja pregatite pentru afisare
/// (`gmotion_core::pregateste_afisare`), nu text brut din baza.
pub fn diferente_obiect(referinta: &str, curent: &str) -> (Vec<Diferenta>, String) {
    let randuri = diff_aliniat(referinta, curent);
    let rezumat = rezumat_diff(&randuri);
    (randuri.iter().map(Diferenta::din).collect(), rezumat)
}

/// Ce arata lista despre un obiect diferit, dincolo de scor: in ce difera si
/// cate linii se adauga / se scot daca se deployeaza.
///
/// La cerere, ca si diff-ul: pe o comparare cu zeci de mii de obiecte,
/// alinierea fiecarui obiect ar fi scumpa si aproape toata aruncata. Lista o
/// cere pentru randurile pe care chiar le arata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Detalii {
    /// „cod, siruri", „un sir", „comentariu", „QUOTED_IDENTIFIER", „—".
    pub difera_in: String,
    /// Linii care ar intra in baza (sunt in sursa, nu si in baza).
    pub linii_plus: usize,
    /// Linii care ar iesi din baza.
    pub linii_minus: usize,
}

/// `setari` sunt numele setarilor de sesiune care difera intre cele doua
/// versiuni (`QUOTED_IDENTIFIER`, `ANSI_NULLS`): nu se vad in text, dar
/// schimba comportamentul la executie, deci intra in „Difera in".
pub fn detalii_obiect(referinta: &str, curent: &str, setari: &[&str]) -> Detalii {
    let randuri = diff_aliniat(referinta, curent);
    let clasificare = gmotion_core::clasificare::clasifica_randuri(&randuri);
    let linii = |t: &Option<String>| t.as_deref().map_or(0, |x| x.lines().count());
    let (mut plus, mut minus) = (0usize, 0usize);
    for r in &randuri {
        if r.tip == TipDiferenta::Egal {
            continue;
        }
        plus += linii(&r.stanga);
        minus += linii(&r.dreapta);
    }
    Detalii {
        difera_in: gmotion_core::clasificare::eticheta(clasificare, setari),
        linii_plus: plus,
        linii_minus: minus,
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn obiect_identic_e_tot_egal() {
        let cod = "CREATE PROC dbo.p AS\nSELECT a FROM t";
        let (d, _) = diferente_obiect(cod, cod);
        assert!(d.iter().all(|x| x.fel == Fel::Egal), "{d:?}");
    }

    #[test]
    fn o_linie_schimbata_apare_ca_modificata_cu_marcaje() {
        let (d, rezumat) = diferente_obiect(
            "CREATE PROC dbo.p AS\nSELECT a FROM t WHERE x = 1",
            "CREATE PROC dbo.p AS\nSELECT a FROM t WHERE x = 2",
        );
        let m: Vec<&Diferenta> = d.iter().filter(|x| x.fel == Fel::Modificat).collect();
        assert!(!m.is_empty(), "nimic modificat: {d:?}");
        assert!(!rezumat.is_empty());
        // marcajele din interiorul liniei trebuie sa existe pentru un rand modificat
        assert!(
            m.iter().any(|x| x
                .bucati_dreapta
                .as_ref()
                .is_some_and(|b| b.iter().any(|y| y.marcaj.is_some()))),
            "lipsesc marcajele pe token: {m:?}"
        );
    }

    #[test]
    fn id_ul_e_stabil_intre_recalculari_nu_e_indexul() {
        // Acelasi obiect de referinta, doua tinte diferite: randurile ancorate
        // in referinta trebuie sa pastreze id-ul, chiar daca pozitia in lista
        // se schimba. Asta e ce permite ca deciziile utilizatorului sa
        // supravietuiasca unei recalculari.
        let referinta = "SELECT 1\n\nSELECT 2\n\nSELECT 3";
        let (a, _) = diferente_obiect(referinta, "SELECT 1\n\nSELECT 2\n\nSELECT 3");
        let (b, _) = diferente_obiect(referinta, "SELECT 1\n\nSELECT 99\n\nSELECT 3");

        let id_uri = |d: &[Diferenta]| -> Vec<i64> {
            d.iter().filter(|x| x.id >= 0).map(|x| x.id).collect()
        };
        assert_eq!(
            id_uri(&a),
            id_uri(&b),
            "id-urile ancorate in referinta s-au mutat"
        );
        assert!(!id_uri(&a).is_empty());
    }

    #[test]
    fn randurile_adaugate_au_id_negativ_si_unic() {
        let (d, _) = diferente_obiect("SELECT 1", "SELECT 1\n\nSELECT 2\n\nSELECT 3");
        let negative: Vec<i64> = d.iter().filter(|x| x.id < 0).map(|x| x.id).collect();
        let mut unice = negative.clone();
        unice.sort_unstable();
        unice.dedup();
        assert_eq!(unice.len(), negative.len(), "id-uri negative duplicate");
    }

    #[test]
    fn rezumatul_e_acelasi_pe_care_l_da_motorul() {
        // Aceeasi functie din core care alimenteaza si lista: daca ar fi doua
        // implementari, am fi inapoi la bug-ul din v3.4.
        let a = "CREATE PROC dbo.p AS\nSELECT 1";
        let b = "CREATE PROC dbo.p AS\nSELECT 2";
        let (_, rezumat) = diferente_obiect(a, b);
        assert_eq!(rezumat, gmotion_core::rezumat_diff(&diff_aliniat(a, b)));
    }

    #[test]
    fn se_serializeaza_fara_campuri_goale() {
        let (d, _) = diferente_obiect("SELECT 1", "SELECT 2");
        let text = serde_json::to_string(&d[0]).unwrap();
        assert!(text.contains("\"id\""));
        assert!(!text.contains("null"), "campurile goale se omit: {text}");
    }

    #[test]
    fn intrari_degenerate_nu_dau_panica() {
        for (a, b) in [("", ""), ("", "SELECT 1"), ("SELECT 1", ""), ("\n\n", "  ")] {
            let _ = diferente_obiect(a, b);
        }
    }

    #[test]
    fn detaliile_spun_in_ce_difera_si_cate_linii_se_misca() {
        let d = detalii_obiect(
            "SELECT 1\n\nUPDATE t SET a = 1\nWHERE x = 1",
            "SELECT 1\n\nUPDATE t SET a = 2",
            &[],
        );
        assert_eq!(d.difera_in, "cod");
        assert_eq!((d.linii_plus, d.linii_minus), (2, 1));
    }

    #[test]
    fn un_obiect_care_difera_doar_in_setari_nu_are_linii_mutate() {
        let cod = "CREATE PROC dbo.p AS\nSELECT 1";
        let d = detalii_obiect(cod, cod, &["QUOTED_IDENTIFIER"]);
        assert_eq!(d.difera_in, "QUOTED_IDENTIFIER");
        assert_eq!((d.linii_plus, d.linii_minus), (0, 0));
    }

    #[test]
    fn un_mesaj_schimbat_e_un_sir_nu_cod() {
        let d = detalii_obiect(
            "RAISERROR('nu s-au gasit date', 16, 1);",
            "RAISERROR('Nu s-au gasit date', 16, 1);",
            &[],
        );
        assert_eq!(d.difera_in, "un sir");
    }
}
