//! Unde ar ajunge un bloc din martor, in textul tintei: inlocuieste un bloc
//! anume, sau se insereaza intr-un gol dintre blocuri.
//!
//! Port al stratului de plasare din v3.4 (`mostenire/ui.py`: `_sugereaza`,
//! `_celMaiApropiat`, `_gapSugerat`, `_grupPlasare`), cu pragurile lui. Fara
//! el nu se pot face „Aplica pe pozitia sugerata", drag & drop, „Plaseaza
//! aici", randul-fantoma din tinta si modul ghidat: toate intreaba „unde cade
//! blocul asta?", si cineva trebuie sa raspunda.
//!
//! **Se strica astfel:** blocul se aseaza peste alt bloc decat corespondentul
//! lui, sau se insereaza la inceputul obiectului desi corespondentul exista in
//! tinta. Simptomul se vede abia dupa „Aplica", in textul final.

use crate::diff::{RandDiff, TipDiferenta};
use crate::normalizare::{normalizeaza, round_py};
use crate::seqmatch::SequenceMatcher;
use once_cell::sync::Lazy;
use regex::Regex;

/// Peste acest scor, blocul din tinta e corespondentul blocului din martor si
/// se **inlocuieste**; sub el, blocul se insereaza intr-un gol. (v3.4)
const PRAG_INLOCUIRE: f64 = 55.0;

/// Sub atata diferenta intre primii doi candidati, scorul nu mai decide:
/// doua instructiuni aproape identice structural (`UPDATE ... WHERE ...`) pot
/// avea scoruri foarte apropiate si foarte mari, iar alegerea „cel mai mare"
/// devine o moneda aruncata. Atunci conteaza pozitia. (v3.4)
const DIFERENTA_AMBIGUA: f64 = 8.0;

/// Cand scorurile sunt ambigue, un vecin al pozitiei preferate trebuie sa aiba
/// macar atata ca sa fie ales. (v3.4)
const PRAG_VECIN: f64 = 40.0;

/// Cand scorurile sunt ambigue si niciun vecin nu se califica, se renunta la
/// sugestie daca nici cel mai bun candidat nu trece de atat. (v3.4)
const PRAG_AMBIGUU: f64 = 70.0;

/// Plafon de siguranta la gruparea blocurilor care se muta impreuna. (v3.4)
const MAX_BLOCURI_GRUPATE: usize = 6;

/// `#tmp`, `##tmp` si `@tmp` se potrivesc ca acelasi identificator: altfel
/// exact refactorul urmarit (tabel temporar -> variabila tabel) isi scade
/// singur scorul si cade sub prag.
static RE_SIGILIU: Lazy<Regex> = Lazy::new(|| Regex::new(r"(^|[\s(,])[#@]{1,2}(\w)").unwrap());
static RE_SEPARATOARE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([(),;'])").unwrap());
static RE_SPATII: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

/// Unde ar ajunge blocul.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tinta {
    /// Inlocuieste blocul `indice` din tinta.
    Bloc { indice: usize, scor: f64 },
    /// Se insereaza inaintea blocului `indice` (`indice == numarul de blocuri`
    /// inseamna la sfarsit). Nu s-a gasit corespondent.
    Gol { indice: usize, scor: f64 },
}

impl Tinta {
    pub fn scor(self) -> f64 {
        match self {
            Tinta::Bloc { scor, .. } | Tinta::Gol { scor, .. } => scor,
        }
    }
}

/// Ce se plaseaza si unde.
#[derive(Debug, Clone, PartialEq)]
pub struct Sugestie {
    /// Randurile din diff care se muta impreuna (un `IF ... BEGIN` isi ia
    /// corpul cu el). Primul e cel cerut.
    ///
    /// Sunt **pozitii** in `randuri`, nu `RandDiff::id` ca in v3.4: fereastra
    /// adreseaza blocurile prin pozitie (`RandAliniat::bloc`), iar id-urile
    /// negative ale randurilor doar-adaugate n-ar avea ce cauta intr-o lista
    /// de mutat.
    pub randuri: Vec<usize>,
    /// Textul care s-ar plasa: blocul singur la inlocuire, grupul intreg la
    /// inserare intr-un gol.
    pub text: String,
    pub tinta: Tinta,
}

/// Token-urile folosite la potrivirea blocurilor. **Nu** sunt cele de la
/// scorul din lista (`normalizare::tokeni_scor`): aici parantezele, virgulele
/// si apostrofurile devin token-uri separate, iar sigiliile de tabel se
/// ignora.
pub fn tokeni_potrivire(text: &str) -> Vec<String> {
    let n = normalizeaza(text);
    let n = RE_SIGILIU.replace_all(&n, "$1$2");
    let n = RE_SEPARATOARE.replace_all(&n, " $1 ");
    let n = RE_SPATII.replace_all(&n, " ");
    n.trim().split(' ').map(|t| t.to_string()).collect()
}

fn scor_potrivire(a: &[String], b: &[String]) -> f64 {
    round_py(SequenceMatcher::new(a, b).ratio() * 100.0, 2)
}

/// Blocul din `blocuri` care seamana cel mai bine cu `text`, daca se poate
/// alege unul. `indice_preferat` e pozitia la care blocul ar cadea dupa
/// ordinea lui, si rupe egalitatile.
///
/// Returneaza `None` cand scorurile sunt prea apropiate ca sa insemne ceva
/// (vezi `DIFERENTA_AMBIGUA`).
pub fn cel_mai_apropiat(
    text: &str,
    blocuri: &[String],
    indice_preferat: Option<usize>,
) -> Option<(usize, f64)> {
    if blocuri.is_empty() {
        return None;
    }
    let ta = tokeni_potrivire(text);
    let mut rez: Vec<(usize, f64)> = blocuri
        .iter()
        .enumerate()
        .map(|(k, b)| (k, scor_potrivire(&ta, &tokeni_potrivire(b))))
        .collect();
    // sortare stabila: la scor egal ramane ordinea blocurilor
    rez.sort_by(|a, b| b.1.total_cmp(&a.1));

    let (best_idx, best_scor) = rez[0];
    let al_doilea = rez.get(1).map_or(-1.0, |r| r.1);
    if best_scor - al_doilea < DIFERENTA_AMBIGUA {
        if let Some(pref) = indice_preferat {
            if let Some(&(k, s)) = rez
                .iter()
                .find(|&&(k, s)| k.abs_diff(pref) <= 1 && s >= PRAG_VECIN)
            {
                return Some((k, s));
            }
        }
        if best_scor < PRAG_AMBIGUU {
            return None;
        }
    }
    Some((best_idx, best_scor))
}

/// Golul in care ar cadea randul `pozitie`: cate blocuri din tinta sunt deja
/// consumate de randurile dinaintea lui.
fn gol_sugerat(randuri: &[RandDiff], pozitie: usize, n_blocuri_tinta: usize) -> usize {
    let inainte = randuri[..pozitie.min(randuri.len())]
        .iter()
        .filter(|r| r.dreapta.is_some())
        .count();
    inainte.min(n_blocuri_tinta)
}

/// Blocurile care se muta impreuna cu cel cerut: un `IF ... BEGIN` neinchis isi
/// ia corpul, pana se echilibreaza `BEGIN` / `END`.
///
/// Antetul (primul rand) nu se grupeaza niciodata: ar inghiti tot obiectul.
fn grup_plasare(randuri: &[RandDiff], pozitie: usize) -> (Vec<usize>, String) {
    let singur = |p: usize| (vec![p], randuri[p].stanga.clone().unwrap_or_default());
    if pozitie >= randuri.len() || pozitie == 0 {
        return singur(pozitie.min(randuri.len().saturating_sub(1)));
    }
    let Some(st0) = randuri[pozitie].stanga.as_deref() else {
        return singur(pozitie);
    };
    let mut adancime = crate::segmentare::adancime_begin_end(st0);
    if adancime <= 0 {
        return singur(pozitie);
    }
    let mut indici = vec![pozitie];
    let mut texte = vec![st0.to_string()];
    let mut k = pozitie;
    while adancime > 0 && k + 1 < randuri.len() && indici.len() < MAX_BLOCURI_GRUPATE {
        k += 1;
        // randurile doar-adaugate nu fac parte din secventa din martor
        let Some(st) = randuri[k].stanga.as_deref() else {
            continue;
        };
        adancime += crate::segmentare::adancime_begin_end(st);
        indici.push(k);
        texte.push(st.to_string());
    }
    if adancime > 0 {
        // nu s-a echilibrat in limita de siguranta: mutam blocul singur,
        // neechilibrat, in loc sa inghitim tot obiectul
        return singur(pozitie);
    }
    (indici, texte.join("\n"))
}

/// Unde ar ajunge randul `pozitie` din diff, in `blocuri_tinta`.
///
/// `blocuri_tinta` sunt blocurile textului **curent** al tintei
/// ([`crate::split_statements`]) — in editor textul se schimba sub tine, deci
/// plasarea se recalculeaza pe ce e acum pe ecran, nu pe ce era la comparare.
///
/// `None` pentru randurile fara nimic in martor (ce exista doar in tinta nu
/// are ce sa plaseze).
pub fn sugereaza(
    randuri: &[RandDiff],
    pozitie: usize,
    blocuri_tinta: &[String],
) -> Option<Sugestie> {
    let rand = randuri.get(pozitie)?;
    let text_bloc = rand.stanga.clone()?;
    let (indici, text_grup) = grup_plasare(randuri, pozitie);
    let preferat = gol_sugerat(randuri, pozitie, blocuri_tinta.len());

    // Antetul (`CREATE PROCEDURE ... AS`) e mereu unic si mereu primul bloc,
    // si in martor si in tinta: se potriveste direct, fara scor. Altfel, daca
    // s-au adaugat parametri, scorul pe token-uri scade sub prag si sugestia
    // ar deveni „insereaza la inceput", desi antetul corespunzator exista.
    let tinta = if pozitie == 0 && !blocuri_tinta.is_empty() {
        Tinta::Bloc {
            indice: 0,
            scor: 100.0,
        }
    } else {
        match cel_mai_apropiat(&text_bloc, blocuri_tinta, Some(preferat)) {
            Some((indice, scor)) if scor >= PRAG_INLOCUIRE => Tinta::Bloc { indice, scor },
            // scor -1 inseamna „n-a fost ales niciun candidat" (ca in v3.4),
            // nu „candidat cu scor zero"
            alt => Tinta::Gol {
                indice: preferat,
                scor: alt.map_or(-1.0, |(_, s)| s),
            },
        }
    };

    // La inlocuire se muta blocul singur (schimb 1:1, sigur). La inserare
    // intr-un gol se aduce grupul intreg, presupunand ca lipseste tot.
    let (randuri_mutate, text) = match tinta {
        Tinta::Bloc { .. } => (vec![pozitie], text_bloc),
        Tinta::Gol { .. } => (indici, text_grup),
    };

    Some(Sugestie {
        randuri: randuri_mutate,
        text,
        tinta,
    })
}

/// Unde ajunge un bloc care difera, dupa ce cele doua straturi s-au impacat.
#[derive(Debug, Clone, PartialEq)]
pub struct Plasare {
    /// Pozitia randului in diff.
    pub rand: usize,
    /// Inlocuieste blocul `bloc_tinta`, sau se insereaza inaintea lui.
    pub inlocuieste: bool,
    /// Indicele blocului din tinta (egal cu numarul de blocuri = la sfarsit).
    pub bloc_tinta: usize,
    /// Potrivirea; `-1` cand niciun candidat nu s-a calificat.
    pub scor: f64,
    /// Randurile care se muta impreuna.
    pub randuri: Vec<usize>,
}

/// Unde ajunge fiecare bloc din martor care difera, in ordinea lor.
///
/// **Doua straturi, o singura concluzie.** Cand diff-ul a imperecheat deja
/// blocul cu unul din tinta (randul e `modificat`), aia e plasarea: perechea
/// aleasa de motor bate o a doua parere, altfel „Aplica" ar scrie in alt loc
/// decat arata editorul. [`sugereaza`] raspunde doar pentru blocurile ramase
/// fara pereche — exact intrebarea pe care diff-ul n-o acopera.
pub fn plasari(randuri: &[RandDiff], blocuri_tinta: &[String]) -> Vec<Plasare> {
    let mut out = Vec::new();
    let mut bloc_tinta_urmator = 0usize;
    for (pozitie, r) in randuri.iter().enumerate() {
        // blocurile din tinta se consuma in ordine
        let perechea = r.dreapta.as_ref().map(|_| {
            let k = bloc_tinta_urmator;
            bloc_tinta_urmator += 1;
            k
        });
        if r.stanga.is_none() || r.tip == TipDiferenta::Egal {
            continue;
        }
        let (inlocuieste, bloc_tinta, scor, mutate) = match perechea {
            Some(k) => (
                true,
                k,
                crate::normalizare::scor_similaritate(
                    r.stanga.as_deref().unwrap_or(""),
                    r.dreapta.as_deref().unwrap_or(""),
                ),
                vec![pozitie],
            ),
            None => {
                let Some(s) = sugereaza(randuri, pozitie, blocuri_tinta) else {
                    continue;
                };
                match s.tinta {
                    Tinta::Bloc { indice, scor } => (true, indice, scor, s.randuri),
                    Tinta::Gol { indice, scor } => (false, indice, scor, s.randuri),
                }
            }
        };
        out.push(Plasare {
            rand: pozitie,
            inlocuieste,
            bloc_tinta,
            scor,
            randuri: mutate,
        });
    }
    out
}

#[cfg(test)]
mod teste {
    use super::*;
    use crate::{diff_aliniat, split_statements};

    fn pregateste(martor: &str, tinta: &str) -> (Vec<RandDiff>, Vec<String>) {
        (diff_aliniat(martor, tinta), split_statements(tinta, false))
    }

    #[test]
    fn sigiliul_de_tabel_nu_scade_scorul() {
        assert_eq!(
            tokeni_potrivire("INSERT INTO #Clienti (a, b)"),
            tokeni_potrivire("INSERT INTO @Clienti (a, b)")
        );
    }

    #[test]
    fn parantezele_si_virgulele_sunt_token_uri_separate() {
        assert_eq!(
            tokeni_potrivire("select a,b from t"),
            vec!["select", "a", ",", "b", "from", "t"]
        );
    }

    /// Cazul din comentariul lui v3.4: antetul primeste parametri noi, scorul
    /// cade sub prag, dar antetul corespunzator exista si e tot primul.
    #[test]
    fn antetul_se_potriveste_direct_chiar_daca_scorul_cade() {
        let martor = "CREATE PROCEDURE p\n @A int\nAS\n\nSELECT 1";
        let tinta = concat!(
            "CREATE PROCEDURE p(\n @A INT,\n @B INT,\n @C INT,\n",
            " @D VARCHAR(255) = '',\n @E INT)\nAS\nBEGIN\n\nSELECT 1"
        );
        let (randuri, blocuri) = pregateste(martor, tinta);
        let s = sugereaza(&randuri, 0, &blocuri).expect("antetul are ce plasa");
        assert_eq!(
            s.tinta,
            Tinta::Bloc {
                indice: 0,
                scor: 100.0
            },
            "antetul trebuie sa cada peste antet, nu intr-un gol"
        );
        // si fara regula, scorul chiar ar fi sub prag
        let scor = cel_mai_apropiat(randuri[0].stanga.as_deref().unwrap(), &blocuri, Some(0));
        assert!(
            scor.is_none_or(|(_, s)| s < PRAG_INLOCUIRE),
            "cazul isi pierde rostul daca scorul trece pragul: {scor:?}"
        );
    }

    /// Al doilea caz din comentariile lui v3.4: `UPDATE ... WHERE ...` aproape
    /// identice structural, cu scoruri mari si foarte apropiate. Atunci scorul
    /// nu mai decide — decide pozitia, prin vecinul pozitiei preferate.
    #[test]
    fn la_scoruri_ambigue_decide_pozitia() {
        let blocuri: Vec<String> = vec![
            "UPDATE Clienti SET activ = 1 WHERE id = @id".to_string(),
            "UPDATE Clienti SET activ = 1 WHERE id = @id AND tip = 1 AND zona = 3".to_string(),
            "UPDATE Clienti SET activ = 1 WHERE id = @id AND tip = 2 AND zona = 3".to_string(),
        ];
        let text = "UPDATE Clienti SET activ = 1 WHERE id = @id AND tip = 2 AND zona = 4";
        let ta = tokeni_potrivire(text);
        let s: Vec<f64> = blocuri
            .iter()
            .map(|b| scor_potrivire(&ta, &tokeni_potrivire(b)))
            .collect();
        assert!(
            s[2] >= s[1] && s[2] - s[1] < DIFERENTA_AMBIGUA,
            "cazul cere primii doi la mai putin de {DIFERENTA_AMBIGUA} puncte: {s:?}"
        );
        assert!(s[1] >= PRAG_VECIN, "vecinul trebuie sa se califice: {s:?}");

        assert_eq!(
            cel_mai_apropiat(text, &blocuri, Some(0)).map(|x| x.0),
            Some(1),
            "cu pozitia preferata 0, castiga vecinul ei, nu scorul: {s:?}"
        );
        assert_eq!(
            cel_mai_apropiat(text, &blocuri, None).map(|x| x.0),
            Some(2),
            "fara pozitie preferata, castiga scorul: {s:?}"
        );
    }

    #[test]
    fn fara_corespondent_se_sugereaza_un_gol() {
        let martor = "SELECT 1\n\nEXEC dbo.ceva_cu_totul_nou @x = 1, @y = 2";
        let tinta = "SELECT 1";
        let (randuri, blocuri) = pregateste(martor, tinta);
        let poz = randuri
            .iter()
            .position(|r| r.stanga.as_deref().is_some_and(|t| t.starts_with("EXEC")))
            .expect("blocul din martor");
        let s = sugereaza(&randuri, poz, &blocuri).expect("are ce plasa");
        assert!(
            matches!(s.tinta, Tinta::Gol { indice: 1, .. }),
            "dupa singurul bloc din tinta: {:?}",
            s.tinta
        );
    }

    #[test]
    fn ce_exista_doar_in_tinta_nu_are_ce_plasa() {
        let (randuri, blocuri) = pregateste("SELECT 1", "SELECT 1\n\nEXEC p");
        let poz = randuri
            .iter()
            .position(|r| r.stanga.is_none())
            .expect("rand doar in tinta");
        assert_eq!(sugereaza(&randuri, poz, &blocuri), None);
    }

    #[test]
    fn un_if_begin_isi_ia_corpul_cand_se_insereaza() {
        let martor = "SELECT 1\n\nIF @x = 1\nBEGIN\n\nUPDATE t SET a = 1\n\nEND";
        let tinta = "SELECT 1";
        let (randuri, blocuri) = pregateste(martor, tinta);
        let poz = randuri
            .iter()
            .position(|r| r.stanga.as_deref().is_some_and(|t| t.starts_with("IF")))
            .expect("blocul IF");
        let s = sugereaza(&randuri, poz, &blocuri).expect("are ce plasa");
        assert!(matches!(s.tinta, Tinta::Gol { .. }), "{:?}", s.tinta);
        assert!(
            s.randuri.len() > 1,
            "IF ... BEGIN se muta cu corpul lui: {:?}",
            s.randuri
        );
        assert!(s.text.contains("UPDATE"), "{:?}", s.text);
    }
}
