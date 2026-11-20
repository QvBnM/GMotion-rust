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

use crate::lexer::Fel;
use crate::normalizare::{scor_pe_tokeni, tokeni_scor, HINTURI};
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

/// Peste acest produs `n*m` nu mai facem imperechere prin programare dinamica
/// intr-un segment `replace`; pastram imperecherea pozitionala.
///
/// Era 144 (12x12), de pe vremea cand fiecare scor costa cat o comparatie
/// difflib completa. De cand `seqmatch` foloseste automatul de sufixe, scorul
/// e ieftin, iar un segment de 64x64 se imperecheaza corect fara sa se simta.
/// Cand plafonul chiar se atinge, se **spune**: vezi
/// [`diff_aliniat_cu_raport`] si `Aliniere::segmente_pozitionale`.
const MAX_PRODUS_ALINIERE: usize = 4096;

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

/// Pentru fiecare token din `tokenizeaza(s)`: e in interiorul unui string sau
/// al unui identificator in ghilimele? Acolo spatiile si majusculele conteaza,
/// deci o schimbare nu e cosmetica. Pana la a doua rulare pe Windows, un spatiu
/// in plus intr-un mesaj (`'... partenerul '`) era raportat „modificat", dar
/// nu era marcat nicaieri in linie.
fn tokeni_in_literal(s: &str) -> Vec<bool> {
    let literale: Vec<(usize, usize)> = crate::lexer::segmente(s)
        .into_iter()
        .filter(|g| matches!(g.fel, Fel::Sir | Fel::IdentificatorGhilimele))
        .map(|g| (g.start, g.sfarsit))
        .collect();
    // segmentele vin in ordinea pozitiei, deci cautare binara: scanarea
    // liniara pe fiecare token facea functia patratica in blocurile mari
    RE_TOKEN
        .find_iter(s)
        .map(|m| {
            let poz = m.start();
            let k = literale.partition_point(|&(a, _)| a <= poz);
            k > 0 && poz < literale[k - 1].1
        })
        .collect()
}

fn este_cosmetic(s: &str) -> bool {
    !s.is_empty() && RE_COSMETIC.is_match(s)
}

/// Diferentele din interiorul unui bloc, pe token-uri.
/// Returneaza `(segmente_referinta, segmente_tinta)`.
fn diff_intra(a: &str, b: &str) -> (Vec<Segment>, Vec<Segment>) {
    let ta = tokenizeaza(a);
    let tb = tokenizeaza(b);
    let la = tokeni_in_literal(a);
    let lb = tokeni_in_literal(b);
    let sm = SequenceMatcher::new(&ta, &tb);

    let mut sr: Vec<Segment> = Vec::new();
    let mut sc: Vec<Segment> = Vec::new();

    for op in sm.opcodes() {
        let va: String = ta[op.i1..op.i2].concat();
        let vb: String = tb[op.j1..op.j2].concat();
        let mut schimbat = op.tag != Tag::Equal;
        let in_literal = la[op.i1..op.i2].iter().any(|&x| x) || lb[op.j1..op.j2].iter().any(|&x| x);
        // in string, spatii schimbate pe spatii (tab in loc de spatii,
        // indentare) sunt cosmetice, ca in `norm`; un spatiu aparut sau
        // disparut ramane schimbare
        let doar_spatii = |v: &str| !v.is_empty() && v.chars().all(char::is_whitespace);
        if schimbat && in_literal && doar_spatii(&va) && doar_spatii(&vb) {
            schimbat = false;
        }
        if schimbat
            && !in_literal
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
/// neatinse. Blocurile prea diferite intre ele nu sunt fortate intr-o pereche -
/// raman stergere + adaugare, care descrie corect ce s-a intamplat.
fn perechi_replace(bv: &[String], bn: &[String]) -> Vec<(usize, usize)> {
    let (n, m) = (bv.len(), bn.len());
    if n == 0 || m == 0 {
        return Vec::new();
    }
    if n * m > MAX_PRODUS_ALINIERE {
        return (0..n.min(m)).map(|i| (i, i)).collect();
    }
    // Un bloc fata de un singur bloc: nu exista alt candidat cu care sa se
    // confunde, deci pragul n-are pe cine sa apere. Fara regula asta, un
    // `SELECT` care a primit patru coloane (scor 24) iesea „sters" plus
    // „adaugat", pe randuri diferite, in loc de „modificat" fata in fata.
    if n == 1 && m == 1 {
        return vec![(0, 0)];
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

    // d[i][j] = scorul cumulat maxim; p[i][j] = directia din care s-a ajuns
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

/// Un rand al planului: ce bloc din referinta si ce bloc din tinta ajung pe el.
///
/// **Singurul loc in care se decide imperecherea blocurilor.** Si diff-ul
/// complet, si rezumatul din lista citesc de aici: doua implementari ale
/// aceleiasi imperecheri inseamna doua numere diferite pentru acelasi obiect
/// (F-M4 din auditul din 2026-11-18: lista spunea „1 modificate", editorul
/// „1 adaugate, 1 sterse").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RandPlan {
    tip: TipDiferenta,
    /// indicele blocului in referinta
    i: Option<usize>,
    /// indicele blocului in tinta
    j: Option<usize>,
}

/// Imperecherea blocurilor, fara sa se uite in interiorul lor.
///
/// Decizia „egal" se ia pe cheile de bloc, nu renormalizand textul: cheie
/// egala inseamna text normalizat egal (invarianta 1b din `CRATE.md`), iar
/// cheile sunt deja calculate.
fn plan_blocuri(
    sv: &[String],
    sn: &[String],
    kv: &[String],
    kn: &[String],
    pozitionale: &mut usize,
) -> Vec<RandPlan> {
    let mut plan: Vec<RandPlan> = Vec::new();
    for op in SequenceMatcher::new(kv, kn).opcodes() {
        match op.tag {
            Tag::Equal => {
                for k in 0..(op.i2 - op.i1) {
                    plan.push(RandPlan {
                        tip: TipDiferenta::Egal,
                        i: Some(op.i1 + k),
                        j: Some(op.j1 + k),
                    });
                }
            }
            Tag::Replace => {
                let bv = &sv[op.i1..op.i2];
                let bn = &sn[op.j1..op.j2];
                if bv.len() * bn.len() > MAX_PRODUS_ALINIERE {
                    *pozitionale += 1;
                }
                let perechi = perechi_replace(bv, bn);
                let (mut pi, mut ii, mut jj) = (0usize, 0usize, 0usize);
                while ii < bv.len() || jj < bn.len() {
                    let pr = perechi.get(pi).copied();
                    if pr == Some((ii, jj)) {
                        let egal = kv[op.i1 + ii] == kn[op.j1 + jj];
                        plan.push(RandPlan {
                            tip: if egal {
                                TipDiferenta::Egal
                            } else {
                                TipDiferenta::Modificat
                            },
                            i: Some(op.i1 + ii),
                            j: Some(op.j1 + jj),
                        });
                        ii += 1;
                        jj += 1;
                        pi += 1;
                        continue;
                    }
                    let (urm_i, urm_j) = pr.unwrap_or((bv.len(), bn.len()));
                    if ii < urm_i {
                        plan.push(RandPlan {
                            tip: TipDiferenta::Sters,
                            i: Some(op.i1 + ii),
                            j: None,
                        });
                        ii += 1;
                        continue;
                    }
                    if jj < urm_j {
                        plan.push(RandPlan {
                            tip: TipDiferenta::Adaugat,
                            i: None,
                            j: Some(op.j1 + jj),
                        });
                        jj += 1;
                        continue;
                    }
                    break;
                }
            }
            Tag::Delete => {
                for k in op.i1..op.i2 {
                    plan.push(RandPlan {
                        tip: TipDiferenta::Sters,
                        i: Some(k),
                        j: None,
                    });
                }
            }
            Tag::Insert => {
                for k in op.j1..op.j2 {
                    plan.push(RandPlan {
                        tip: TipDiferenta::Adaugat,
                        i: None,
                        j: Some(k),
                    });
                }
            }
        }
    }
    plan
}

/// Blocurile si cheile lor, pentru ambele parti.
fn blocuri_si_chei(
    text_referinta: &str,
    text_curent: &str,
    uneste_cursor: bool,
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let sv = split_statements(text_referinta, uneste_cursor);
    let sn = split_statements(text_curent, uneste_cursor);
    let kv: Vec<String> = sv.iter().map(|b| cheie_bloc(b)).collect();
    let kn: Vec<String> = sn.iter().map(|b| cheie_bloc(b)).collect();
    (sv, sn, kv, kn)
}

/// Comparatia aliniata, bloc cu bloc.
pub fn diff_aliniat(text_referinta: &str, text_curent: &str) -> Vec<RandDiff> {
    diff_aliniat_cu(text_referinta, text_curent, false)
}

/// Ca [`diff_aliniat`], dar cu optiunea de segmentare „uneste ciclul
/// cursorului" (`DECLARE ... CURSOR` pana la `DEALLOCATE` intr-un singur
/// bloc), ca in v3.4.
pub fn diff_aliniat_cu(
    text_referinta: &str,
    text_curent: &str,
    uneste_cursor: bool,
) -> Vec<RandDiff> {
    diff_aliniat_cu_raport(text_referinta, text_curent, uneste_cursor).0
}

/// Ca [`diff_aliniat_cu`], plus cate segmente `replace` au fost prea mari
/// pentru imperecherea pe scor si au ramas pe imperechere pozitionala
/// (`MAX_PRODUS_ALINIERE`). Acolo „modificat" poate insemna de fapt doua
/// blocuri fara legatura — cine afiseaza trebuie sa poata spune asta, nu sa
/// degradeze in tacere.
pub fn diff_aliniat_cu_raport(
    text_referinta: &str,
    text_curent: &str,
    uneste_cursor: bool,
) -> (Vec<RandDiff>, usize) {
    let (sv, sn, kv, kn) = blocuri_si_chei(text_referinta, text_curent, uneste_cursor);
    let mut pozitionale = 0usize;
    let plan = plan_blocuri(&sv, &sn, &kv, &kn, &mut pozitionale);

    let mut randuri: Vec<RandDiff> = Vec::with_capacity(plan.len());
    let mut contor_inserturi: i64 = 0;
    for r in plan {
        // `id` STABIL intre recalculari: pozitia blocului in referinta. Randurile
        // doar-adaugate n-au ancora, deci primesc id-uri negative, unice pe randare.
        let id = match r.i {
            Some(i) => i as i64,
            None => {
                contor_inserturi += 1;
                -contor_inserturi
            }
        };
        let stanga = r.i.map(|i| sv[i].clone());
        let dreapta = r.j.map(|j| sn[j].clone());
        let (seg_stanga, seg_dreapta) = match (r.tip, r.i, r.j) {
            (TipDiferenta::Modificat, Some(i), Some(j)) => {
                let (ss, ds) = diff_intra(&sv[i], &sn[j]);
                (Some(ss), Some(ds))
            }
            _ => (None, None),
        };
        randuri.push(RandDiff {
            id,
            tip: r.tip,
            stanga,
            dreapta,
            seg_stanga,
            seg_dreapta,
        });
    }

    (randuri, pozitionale)
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

/// Rezumat pentru lista: aceeasi imperechere ca [`diff_aliniat`], dar fara
/// diferentele din interiorul blocurilor (partea scumpa: token-izare si
/// `SequenceMatcher` per bloc modificat).
///
/// Numerele sunt **identice** cu ale editorului, prin constructie: amandoua
/// citesc din [`plan_blocuri`].
pub fn rezumat_rapid(text_referinta: &str, text_curent: &str) -> String {
    let (sv, sn, kv, kn) = blocuri_si_chei(text_referinta, text_curent, false);
    let plan = plan_blocuri(&sv, &sn, &kv, &kn, &mut 0);
    let numara = |t: TipDiferenta| plan.iter().filter(|r| r.tip == t).count();
    formateaza_rezumat(
        numara(TipDiferenta::Modificat),
        numara(TipDiferenta::Adaugat),
        numara(TipDiferenta::Sters),
    )
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
    use crate::normalizare::scor_similaritate;

    fn tipuri(r: &[RandDiff]) -> Vec<&'static str> {
        r.iter().map(|x| x.tip.eticheta()).collect()
    }

    #[test]
    fn identice_dau_doar_egal() {
        let t = "SELECT 1\n\nSELECT 2";
        assert_eq!(tipuri(&diff_aliniat(t, t)), vec!["egal", "egal"]);
    }

    /// Regresie din audit: cu 32 de biti de MD5 in cheie, aceste doua blocuri
    /// aveau aceeasi cheie (coliziune gasita in 0,1 s) si se raportau `egal`.
    #[test]
    fn blocuri_diferite_cu_cheie_veche_identica_nu_sunt_egale() {
        let a = "DELETE FROM Clienti WHERE activ = 0 AND id = 32750";
        let b = "DELETE FROM Clienti WHERE activ = 0 AND id = 79586";
        assert_ne!(cheie_bloc(a), cheie_bloc(b));
        assert_eq!(tipuri(&diff_aliniat(a, b)), vec!["modificat"]);
    }

    /// Un spatiu aparut intr-un string se marcheaza; acelasi spatiu in cod nu.
    /// (Doua spatii in loc de unul, in string, sunt egale, ca in v3.4.)
    #[test]
    fn spatiile_din_string_se_marcheaza_cele_din_cod_nu() {
        assert_eq!(
            tipuri(&diff_aliniat("PRINT 'a b'", "PRINT 'a  b'")),
            vec!["egal"]
        );
        let r = diff_aliniat(
            "PRINT 'mesaj catre partenerul'",
            "PRINT 'mesaj catre partenerul '",
        );
        let seg = r[0].seg_dreapta.as_ref().expect("segmente");
        assert!(seg.iter().any(|s| s.schimbat.is_some()), "{seg:?}");
        let r = diff_aliniat("SELECT a,b FROM t", "SELECT a,  b FROM t");
        assert!(
            r.iter().all(|x| x
                .seg_dreapta
                .as_ref()
                .is_none_or(|s| s.iter().all(|g| g.schimbat.is_none()))),
            "{r:?}"
        );
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

    /// Proba pe Windows (2026-11-19): SQL dinamic reindentat intr-un string
    /// (tab in loc de spatii) nu se evidentiaza; un spatiu aparut in string
    /// da, fiindca schimba textul.
    #[test]
    fn spatiile_schimbate_in_string_nu_se_evidentiaza() {
        let marcat = |a: &str, b: &str| -> Vec<String> {
            let (ss, sc) = diff_intra(a, b);
            ss.iter()
                .chain(sc.iter())
                .filter(|s| s.schimbat.is_some())
                .map(|s| s.text.clone())
                .collect()
        };
        let m = marcat(
            "SET @s = ') W\n\t\tJOIN t X ON W.a=X.a\t' + @x",
            "SET @s = ') W\n            JOIN t X ON W.a=X.a ' + @y",
        );
        assert_eq!(m, ["@x", "@y"], "doar variabila difera: {m:?}");
        let m = marcat("PRINT 'a b'", "PRINT 'ab'");
        assert!(!m.is_empty(), "spatiul disparut din string se vede");
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

    /// Cazul martor / upgrade al lui Mihai (2026-11-19): acelasi `SELECT`, cu
    /// coloane in plus. Scorul dintre cele doua blocuri e 24, sub pragul de
    /// perechere — dar segmentul `replace` are un bloc de fiecare parte, deci
    /// nu exista alt candidat cu care sa se confunde. Pana la etapa 20 ieseau
    /// „sters" plus „adaugat", pe randuri care nu se vedeau impreuna.
    #[test]
    fn un_bloc_fata_de_un_bloc_se_imperecheaza_oricat_de_diferit() {
        let martor = "SELECT 'RCA_Multiple.doc' AS Template";
        let modificata = concat!(
            "SELECT 'RCA_Multiple.doc' AS Template,\n",
            "       CAST(0 AS BIT) AS SaveOnLocalDisk,\n",
            "       NULL AS LocalDiskSavePath\n",
            "END"
        );
        assert!(
            scor_similaritate(martor, modificata) < PRAG_PERECHE,
            "cazul isi pierde rostul daca scorul trece pragul"
        );
        let r = diff_aliniat(martor, modificata);
        assert_eq!(tipuri(&r), vec!["modificat"]);

        let marcat = |s: &Option<Vec<Segment>>| -> Vec<String> {
            s.as_ref()
                .map(|v| {
                    v.iter()
                        .filter(|x| x.schimbat.is_some())
                        .map(|x| x.text.clone())
                        .collect()
                })
                .unwrap_or_default()
        };
        assert!(
            marcat(&r[0].seg_stanga).is_empty(),
            "nimic nu dispare din martor: {:?}",
            marcat(&r[0].seg_stanga)
        );
        let dreapta = marcat(&r[0].seg_dreapta).join("");
        assert!(
            dreapta.contains(','),
            "virgula noua se marcheaza: {dreapta:?}"
        );
        assert!(
            !dreapta.contains("RCA_Multiple.doc"),
            "ce n-a fost atins nu se marcheaza: {dreapta:?}"
        );
    }

    #[test]
    fn rezumat_fara_diferente() {
        let t = "SELECT 1";
        assert_eq!(rezumat_rapid(t, t), "diferente minore");
        assert_eq!(rezumat_diff(&diff_aliniat(t, t)), "diferente minore");
    }
}
