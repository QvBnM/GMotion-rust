//! Alinierea blocurilor si diferentele din interiorul unui bloc.
//!
//! Alinierea se face prin programare dinamica pe similaritate
//! (`perechi_replace`), nu pozitional ca in `mostenire/motor.py`: doua blocuri
//! care si-au schimbat ordinea trebuie imperecheate intre ele, nu declarate
//! „sters" plus „adaugat".
//!
//! **Se strica astfel:** un bloc modificat apare drept sters si adaugat
//! separat (sau invers, doua blocuri fara legatura apar imperecheate);
//! evidentierea in interiorul liniei marcheaza alte token-uri decat cele
//! schimbate; rezumatul raporteaza blocuri care nu exista.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::normalizare::{norm, scor_pe_tokeni, tokeni_scor, HINTURI};
use crate::segmentare::{cheie_bloc, split_statements};
use crate::seqmatch::{SequenceMatcher, Tag};

static RE_TOKEN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s+|[A-Za-z_@#][0-9A-Za-z_]*|[^\sA-Za-z_@#]").unwrap());

/// Segmentele care arata diferite caracter-cu-caracter, dar pe care
/// normalizarea le ignora oricum: `[dbo].`, parantezele drepte,
/// `COLLATE database_default`, hint-urile de lock.
///
/// Cand un bloc e `modificat` dintr-un motiv real aflat in alta parte a lui,
/// diferenta pe token-uri le-ar scoate la iveala ca "diferente" - desi
/// utilizatorul le vede deja ca zgomot.
static RE_COSMETIC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        concat!(
            r"(?i)^(?:\[dbo\]\.?|dbo\.|\[|\]|collate\s+database_default|",
            r"with\s*\(\s*(?:{h})(?:\s*,\s*(?:{h}))*\s*\)|(?:{h})|\s)+$"
        ),
        h = HINTURI
    ))
    .unwrap()
});

/// Pragul de similaritate peste care doua blocuri din acelasi segment
/// `replace` se considera o pereche (modificare), nu stergere + adaugare.
const PRAG_PERECHE: f64 = 44.0;

/// Peste acest produs `n*m` nu mai facem aliniere prin programare dinamica;
/// pentru segmente mari costul nu se justifica si pastram imperecherea
/// pozitionala.
const MAX_PRODUS_ALINIERE: usize = 144;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipDiferenta {
    Egal,
    Modificat,
    Sters,
    Adaugat,
}

impl TipDiferenta {
    pub fn eticheta(self) -> &'static str {
        match self {
            TipDiferenta::Egal => "egal",
            TipDiferenta::Modificat => "modificat",
            TipDiferenta::Sters => "sters",
            TipDiferenta::Adaugat => "adaugat",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FelSchimbare {
    /// Prezent in referinta, lipseste sau e inlocuit in tinta.
    Sters,
    /// Prezent in tinta, nou fata de referinta.
    Adaugat,
}

/// O bucata de text dintr-un bloc, marcata sau nu ca diferita.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub schimbat: Option<FelSchimbare>,
}

/// Un rand din comparatia aliniata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandDiff {
    /// Identificator STABIL intre recalculari: pentru randurile ancorate in
    /// referinta (`egal`, `sters`, `modificat`) e pozitia absoluta a blocului
    /// in referinta, care nu se schimba cat timp referinta e fixa. Randurile
    /// doar-adaugate nu au ancora, deci primesc id-uri negative, unice pe
    /// randare, dintr-un spatiu separat de numere.
    pub id: i64,
    pub tip: TipDiferenta,
    pub stanga: Option<String>,
    pub dreapta: Option<String>,
    pub seg_stanga: Option<Vec<Segment>>,
    pub seg_dreapta: Option<Vec<Segment>>,
}

fn tokenizeaza(s: &str) -> Vec<&str> {
    RE_TOKEN.find_iter(s).map(|m| m.as_str()).collect()
}

fn este_cosmetic(s: &str) -> bool {
    !s.is_empty() && RE_COSMETIC.is_match(s)
}

/// Diferentele din interiorul unui bloc, pe token-uri.
/// Returneaza `(segmente_referinta, segmente_tinta)`.
fn diff_intra(a: &str, b: &str) -> (Vec<Segment>, Vec<Segment>) {
    let ta = tokenizeaza(a);
    let tb = tokenizeaza(b);
    let sm = SequenceMatcher::new(&ta, &tb);

    let mut sr: Vec<Segment> = Vec::new();
    let mut sc: Vec<Segment> = Vec::new();

    for op in sm.opcodes() {
        let va: String = ta[op.i1..op.i2].concat();
        let vb: String = tb[op.j1..op.j2].concat();
        let mut schimbat = op.tag != Tag::Equal;
        if schimbat
            && (va.is_empty() || este_cosmetic(&va))
            && (vb.is_empty() || este_cosmetic(&vb))
        {
            schimbat = false;
        }
        if !va.is_empty() {
            sr.push(Segment {
                text: va,
                schimbat: schimbat.then_some(FelSchimbare::Sters),
            });
        }
        if !vb.is_empty() {
            sc.push(Segment {
                text: vb,
                schimbat: schimbat.then_some(FelSchimbare::Adaugat),
            });
        }
    }

    (sr, sc)
}

/// Imperecheaza blocurile dintr-un segment `replace` dupa similaritate, prin
/// programare dinamica, fara incrucisari.
///
/// Fara asta, imperecherea e pur pozitionala: daca in tinta a fost inserat un
/// bloc la mijloc, toate blocurile de dupa apar ca "modificate" desi sunt
/// neatinse. Blocurile prea diferite intre ele nu sunt forțate intr-o pereche -
/// ramân stergere + adaugare, care descrie corect ce s-a intamplat.
fn perechi_replace(bv: &[String], bn: &[String]) -> Vec<(usize, usize)> {
    let (n, m) = (bv.len(), bn.len());
    if n == 0 || m == 0 {
        return Vec::new();
    }
    if n * m > MAX_PRODUS_ALINIERE {
        return (0..n.min(m)).map(|i| (i, i)).collect();
    }

    // token-urile se calculeaza o singura data per bloc, nu la fiecare pereche
    let tv: Vec<Vec<String>> = bv.iter().map(|b| tokeni_scor(b)).collect();
    let tn: Vec<Vec<String>> = bn.iter().map(|b| tokeni_scor(b)).collect();

    let mut s = vec![vec![0f64; m]; n];
    for i in 0..n {
        for j in 0..m {
            s[i][j] = scor_pe_tokeni(&tv[i], &tn[j]);
        }
    }

    // d[i][j] = scorul cumulat maxim; p[i][j] = direcția din care s-a ajuns
    let mut d = vec![vec![0f64; m + 1]; n + 1];
    let mut p = vec![vec![0u8; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            let scor = s[i - 1][j - 1];
            let diag = d[i - 1][j - 1] + if scor >= PRAG_PERECHE { scor } else { 0.0 };
            let sus = d[i - 1][j];
            let stanga = d[i][j - 1];
            if diag >= sus && diag >= stanga {
                d[i][j] = diag;
                p[i][j] = 1;
            } else if sus >= stanga {
                d[i][j] = sus;
                p[i][j] = 2;
            } else {
                d[i][j] = stanga;
                p[i][j] = 3;
            }
        }
    }

    let mut perechi: Vec<(usize, usize)> = Vec::new();
    let (mut i, mut j) = (n, m);
    while i > 0 && j > 0 {
        match p[i][j] {
            1 => {
                if s[i - 1][j - 1] >= PRAG_PERECHE {
                    perechi.push((i - 1, j - 1));
                }
                i -= 1;
                j -= 1;
            }
            2 => i -= 1,
            _ => j -= 1,
        }
    }
    perechi.reverse();
    perechi
}

/// Comparatia aliniata, bloc cu bloc.
// indicele blocului devine `id`-ul stabil al randului, deci nu poate fi
// inlocuit cu un iterator pe elemente
#[allow(clippy::needless_range_loop)]
pub fn diff_aliniat(text_referinta: &str, text_curent: &str) -> Vec<RandDiff> {
    let sv = split_statements(text_referinta, false);
    let sn = split_statements(text_curent, false);
    let kv: Vec<String> = sv.iter().map(|b| cheie_bloc(b)).collect();
    let kn: Vec<String> = sn.iter().map(|b| cheie_bloc(b)).collect();

    let mut randuri: Vec<RandDiff> = Vec::new();
    let mut contor_inserturi: i64 = 0;
    let mut urmator_id_insert = || {
        contor_inserturi += 1;
        -contor_inserturi
    };

    for op in SequenceMatcher::new(&kv, &kn).opcodes() {
        match op.tag {
            Tag::Equal => {
                for k in 0..(op.i2 - op.i1) {
                    randuri.push(RandDiff {
                        id: (op.i1 + k) as i64,
                        tip: TipDiferenta::Egal,
                        stanga: Some(sv[op.i1 + k].clone()),
                        dreapta: Some(sn[op.j1 + k].clone()),
                        seg_stanga: None,
                        seg_dreapta: None,
                    });
                }
            }
            Tag::Replace => {
                let bv = &sv[op.i1..op.i2];
                let bn = &sn[op.j1..op.j2];
                let perechi = perechi_replace(bv, bn);
                let (mut pi, mut ii, mut jj) = (0usize, 0usize, 0usize);

                while ii < bv.len() || jj < bn.len() {
                    let pr = perechi.get(pi).copied();
                    if pr == Some((ii, jj)) {
                        if norm(&bv[ii]) == norm(&bn[jj]) {
                            randuri.push(RandDiff {
                                id: (op.i1 + ii) as i64,
                                tip: TipDiferenta::Egal,
                                stanga: Some(bv[ii].clone()),
                                dreapta: Some(bn[jj].clone()),
                                seg_stanga: None,
                                seg_dreapta: None,
                            });
                        } else {
                            let (ss, ds) = diff_intra(&bv[ii], &bn[jj]);
                            randuri.push(RandDiff {
                                id: (op.i1 + ii) as i64,
                                tip: TipDiferenta::Modificat,
                                stanga: Some(bv[ii].clone()),
                                dreapta: Some(bn[jj].clone()),
                                seg_stanga: Some(ss),
                                seg_dreapta: Some(ds),
                            });
                        }
                        ii += 1;
                        jj += 1;
                        pi += 1;
                        continue;
                    }
                    let (urm_i, urm_j) = pr.unwrap_or((bv.len(), bn.len()));
                    if ii < urm_i {
                        randuri.push(RandDiff {
                            id: (op.i1 + ii) as i64,
                            tip: TipDiferenta::Sters,
                            stanga: Some(bv[ii].clone()),
                            dreapta: None,
                            seg_stanga: None,
                            seg_dreapta: None,
                        });
                        ii += 1;
                        continue;
                    }
                    if jj < urm_j {
                        randuri.push(RandDiff {
                            id: urmator_id_insert(),
                            tip: TipDiferenta::Adaugat,
                            stanga: None,
                            dreapta: Some(bn[jj].clone()),
                            seg_stanga: None,
                            seg_dreapta: None,
                        });
                        jj += 1;
                        continue;
                    }
                    break;
                }
            }
            Tag::Delete => {
                for k in op.i1..op.i2 {
                    randuri.push(RandDiff {
                        id: k as i64,
                        tip: TipDiferenta::Sters,
                        stanga: Some(sv[k].clone()),
                        dreapta: None,
                        seg_stanga: None,
                        seg_dreapta: None,
                    });
                }
            }
            Tag::Insert => {
                for k in op.j1..op.j2 {
                    randuri.push(RandDiff {
                        id: urmator_id_insert(),
                        tip: TipDiferenta::Adaugat,
                        stanga: None,
                        dreapta: Some(sn[k].clone()),
                        seg_stanga: None,
                        seg_dreapta: None,
                    });
                }
            }
        }
    }

    randuri
}

fn formateaza_rezumat(modificate: usize, adaugate: usize, sterse: usize) -> String {
    let mut p: Vec<String> = Vec::with_capacity(3);
    if modificate > 0 {
        p.push(format!("{} modificate", modificate));
    }
    if adaugate > 0 {
        p.push(format!("{} adaugate", adaugate));
    }
    if sterse > 0 {
        p.push(format!("{} sterse", sterse));
    }
    if p.is_empty() {
        "diferente minore".to_string()
    } else {
        p.join(", ")
    }
}

/// Rezumat pentru lista: numara doar la nivel de blocuri, fara diferentele din
/// interiorul lor (partea scumpa). Pe obiecte mari e de ordine de magnitudine
/// mai rapid decat `diff_aliniat`, cu acelasi rezultat numeric.
pub fn rezumat_rapid(text_referinta: &str, text_curent: &str) -> String {
    let sv = split_statements(text_referinta, false);
    let sn = split_statements(text_curent, false);
    let kv: Vec<String> = sv.iter().map(|b| cheie_bloc(b)).collect();
    let kn: Vec<String> = sn.iter().map(|b| cheie_bloc(b)).collect();

    let (mut modificate, mut adaugate, mut sterse) = (0usize, 0usize, 0usize);
    for op in SequenceMatcher::new(&kv, &kn).opcodes() {
        match op.tag {
            Tag::Replace => {
                let comun = (op.i2 - op.i1).min(op.j2 - op.j1);
                for k in 0..comun {
                    if norm(&sv[op.i1 + k]) != norm(&sn[op.j1 + k]) {
                        modificate += 1;
                    }
                }
                sterse += (op.i2 - op.i1) - comun;
                adaugate += (op.j2 - op.j1) - comun;
            }
            Tag::Delete => sterse += op.i2 - op.i1,
            Tag::Insert => adaugate += op.j2 - op.j1,
            Tag::Equal => {}
        }
    }

    formateaza_rezumat(modificate, adaugate, sterse)
}

/// Rezumat din randurile deja calculate de `diff_aliniat`.
pub fn rezumat_diff(randuri: &[RandDiff]) -> String {
    let numara = |t: TipDiferenta| randuri.iter().filter(|r| r.tip == t).count();
    formateaza_rezumat(
        numara(TipDiferenta::Modificat),
        numara(TipDiferenta::Adaugat),
        numara(TipDiferenta::Sters),
    )
}

#[cfg(test)]
mod teste {
    use super::*;

    fn tipuri(r: &[RandDiff]) -> Vec<&'static str> {
        r.iter().map(|x| x.tip.eticheta()).collect()
    }

    #[test]
    fn identice_dau_doar_egal() {
        let t = "SELECT 1\n\nSELECT 2";
        assert_eq!(tipuri(&diff_aliniat(t, t)), vec!["egal", "egal"]);
    }

    #[test]
    fn bloc_sters() {
        let r = diff_aliniat(
            "SELECT 1\n\nUPDATE t SET a = 1\n\nSELECT 2",
            "SELECT 1\n\nSELECT 2",
        );
        assert!(tipuri(&r).contains(&"sters"));
    }

    #[test]
    fn bloc_adaugat() {
        let r = diff_aliniat("SELECT 1\n\nSELECT 2", "SELECT 1\n\nEXEC p\n\nSELECT 2");
        assert!(tipuri(&r).contains(&"adaugat"));
    }

    #[test]
    fn diferenta_cosmetica_nu_se_evidentiaza() {
        // blocul e modificat dintr-un motiv real (1 -> 2), dar `[dbo].` nu
        // trebuie marcat ca diferenta
        let (ss, _) = diff_intra(
            "SELECT a FROM [dbo].[t] WHERE x = 1",
            "SELECT a FROM t WHERE x = 2",
        );
        let marcat: Vec<&str> = ss
            .iter()
            .filter(|s| s.schimbat.is_some())
            .map(|s| s.text.as_str())
            .collect();
        assert!(
            !marcat.iter().any(|t| t.contains("dbo")),
            "prefixul dbo. nu trebuie evidentiat ca diferenta: {:?}",
            marcat
        );
    }

    #[test]
    fn id_urile_de_insert_sunt_negative_si_unice() {
        let r = diff_aliniat("SELECT 1", "EXEC a\n\nEXEC b\n\nSELECT 1");
        let ids: Vec<i64> = r
            .iter()
            .filter(|x| x.tip == TipDiferenta::Adaugat)
            .map(|x| x.id)
            .collect();
        assert!(ids.iter().all(|&i| i < 0), "ids: {:?}", ids);
        let mut unice = ids.clone();
        unice.sort_unstable();
        unice.dedup();
        assert_eq!(unice.len(), ids.len());
    }

    #[test]
    fn insertie_la_mijloc_nu_face_restul_modificat() {
        // regresia pe care o previne `perechi_replace`
        let referinta = "UPDATE t SET a = 1\n\nUPDATE t SET b = 2\n\nUPDATE t SET c = 3";
        let curent = "UPDATE t SET a = 1\n\nEXEC p_nou\n\nUPDATE t SET b = 2\n\nUPDATE t SET c = 3";
        let r = diff_aliniat(referinta, curent);
        assert_eq!(
            r.iter()
                .filter(|x| x.tip == TipDiferenta::Modificat)
                .count(),
            0,
            "blocurile neatinse nu trebuie raportate ca modificate: {:?}",
            tipuri(&r)
        );
    }

    #[test]
    fn rezumat_fara_diferente() {
        let t = "SELECT 1";
        assert_eq!(rezumat_rapid(t, t), "diferente minore");
        assert_eq!(rezumat_diff(&diff_aliniat(t, t)), "diferente minore");
    }
}
