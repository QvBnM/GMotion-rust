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
use crate::normalizare::{scor_maxim_posibil, scor_pe_tokeni, tokeni_scor, HINTURI};
use crate::segmentare::{cheie_bloc, segmenteaza, split_statements, Segmentare};
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
/// e ieftin; de cand matricea se reteaza pe lungimi
/// ([`scor_maxim_posibil`]), cele mai multe perechi nici nu se calculeaza,
/// deci plafonul a urcat de la 4096 (64x64) la 16384 (128x128).
/// Cand plafonul chiar se atinge, se **spune**: vezi
/// [`diff_aliniat_cu_raport`] si `Aliniere::segmente_pozitionale`.
const MAX_PRODUS_ALINIERE: usize = 16_384;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipDiferenta {
    Egal,
    Modificat,
    Sters,
    Adaugat,
    /// Acelasi bloc, in alt loc. Vezi [`RandDiff::mutat_cu`].
    Mutat,
}

impl TipDiferenta {
    pub fn eticheta(self) -> &'static str {
        match self {
            TipDiferenta::Egal => "egal",
            TipDiferenta::Modificat => "modificat",
            TipDiferenta::Sters => "sters",
            TipDiferenta::Adaugat => "adaugat",
            TipDiferenta::Mutat => "mutat",
        }
    }

    /// Blocul exista in martor (randul are text in stanga).
    pub fn e_in_martor(self) -> bool {
        !matches!(self, TipDiferenta::Adaugat)
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
    /// Pentru un rand `Mutat`: pozitia (in acelasi vector de randuri) a
    /// celuilalt capat al mutarii — randul din tinta pentru randul din
    /// martor, si invers. `None` pe orice alt fel de rand.
    ///
    /// Mutarea se raporteaza pe **doua** randuri, nu pe unul: un singur rand
    /// cu text pe amandoua parti ar strica ordinea pe care se sprijina tot
    /// restul (randurile cu text in dreapta vin in ordinea blocurilor tintei —
    /// `plasare::plasari` si `aliniere` numara pe asta), fiindca blocul mutat
    /// e, prin definitie, in alta ordine.
    pub mutat_cu: Option<usize>,
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
    diff_intra_cu(a, b, true)
}

/// Ca [`diff_intra`]; cu `filtru_cosmetic = false` se aprinde orice bucata
/// schimbata, si spatiile sau `dbo.`.
fn diff_intra_cu(a: &str, b: &str, filtru_cosmetic: bool) -> (Vec<Segment>, Vec<Segment>) {
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
        if !filtru_cosmetic {
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
            continue;
        }
        let in_literal = la[op.i1..op.i2].iter().any(|&x| x) || lb[op.j1..op.j2].iter().any(|&x| x);
        // in string, reindentarea (spatii care trec de rand, de ambele parti)
        // e cosmetica, ca in `norm`; pe acelasi rand, tab in loc de spatiu sau
        // un spatiu aparut ramane schimbare (Mihai, 2026-11-22)
        let doar_spatii = |v: &str| !v.is_empty() && v.chars().all(char::is_whitespace);
        let trece_de_rand = |v: &str| v.contains(['\n', '\r']);
        if schimbat
            && in_literal
            && doar_spatii(&va)
            && doar_spatii(&vb)
            && trece_de_rand(&va)
            && trece_de_rand(&vb)
        {
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

    // Retezare exacta: o pereche a carei margine din lungimi e sub prag nu
    // poate trece pragul, iar DP-ul aduna scorul numai cand trece. Deci zero
    // in loc de o comparatie, cu **acelasi** rezultat.
    let mut s = vec![vec![0f64; m]; n];
    for i in 0..n {
        for j in 0..m {
            if scor_maxim_posibil(tv[i].len(), tn[j].len()) < PRAG_PERECHE {
                continue;
            }
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
    /// pozitia celuilalt capat al mutarii, in plan
    mutat_cu: Option<usize>,
}

impl RandPlan {
    fn nou(tip: TipDiferenta, i: Option<usize>, j: Option<usize>) -> Self {
        Self {
            tip,
            i,
            j,
            mutat_cu: None,
        }
    }
}

/// Marcheaza ca **mutate** blocurile ramase fara pereche care au, de cealalta
/// parte, un bloc cu exact aceeasi forma normalizata.
///
/// `SequenceMatcher` imperecheaza blocurile egale care au ramas in ordine; ce
/// pica pe „sters" plus „adaugat" cu cheie identica e deci, prin definitie, un
/// bloc reordonat. Pana la 2026-11-24 motorul nu avea notiunea: reordonarea —
/// refactorul cel mai frecvent — ieseau doua diferente fara nicio legatura
/// vizibila intre ele, si cine se uita nu putea sti ca textul n-a fost atins.
///
/// „Mutat" cere text normalizat **identic**: un bloc mutat si schimbat
/// ramane „sters" plus „adaugat", ca pana acum. Cheia o spune (`cheie_bloc` =
/// 128 de biti de SHA-256 pe forma normalizata), deci nu costa nicio
/// comparatie.
fn marcheaza_mutarile(plan: &mut [RandPlan], kv: &[String], kn: &[String]) {
    use std::collections::HashMap;
    let mut dupa_cheie: HashMap<&str, Vec<usize>> = HashMap::new();
    for (p, r) in plan.iter().enumerate() {
        if r.tip == TipDiferenta::Adaugat {
            if let Some(j) = r.j {
                dupa_cheie.entry(kn[j].as_str()).or_default().push(p);
            }
        }
    }
    if dupa_cheie.is_empty() {
        return;
    }
    let mut perechi: Vec<(usize, usize)> = Vec::new();
    for (p, r) in plan.iter().enumerate() {
        if r.tip != TipDiferenta::Sters {
            continue;
        }
        let Some(i) = r.i else { continue };
        if let Some(candidati) = dupa_cheie.get_mut(kv[i].as_str()) {
            if !candidati.is_empty() {
                perechi.push((p, candidati.remove(0)));
            }
        }
    }
    for (st, dr) in perechi {
        plan[st].tip = TipDiferenta::Mutat;
        plan[st].mutat_cu = Some(dr);
        plan[dr].tip = TipDiferenta::Mutat;
        plan[dr].mutat_cu = Some(st);
    }
}

/// Imperecherea blocurilor, fara sa se uite in interiorul lor.
///
/// Decizia „egal" se ia pe cheile de bloc, nu renormalizand textul: cheie
/// egala inseamna text normalizat egal (invarianta 1b din `CRATE.md`), iar
/// cheile sunt deja calculate.
/// Perechile de blocuri cu **aceeasi** cheie dintre doua liste de blocuri
/// libere, lacom: fiecare bloc din stanga ia primul bloc nefolosit cu cheia lui.
///
/// Rostul: blocurile identice pe care `SequenceMatcher` nu le-a imperecheat.
/// Ratcliff-Obershelp nu e un LCS exhaustiv, ci recursiv pe cea mai lunga
/// potrivire, deci un bloc care si-a schimbat locul *peste* potrivirea aleasa
/// nu mai e privit niciodata. Pana la etapa 30, imperecherea pe scor il lua
/// atunci drept pereche a altui bloc si il raporta **modificat** — adica
/// „instructiunea asta s-a schimbat" despre o instructiune neatinsa. Masurat pe
/// corpusul real (`tests/calitate.rs`): doua blocuri inversate ieseau
/// „2 modificate" pe 23 din 25 de obiecte.
fn perechi_identice(
    kv: &[String],
    kn: &[String],
    liberi_i: &[usize],
    liberi_j: &[usize],
) -> Vec<(usize, usize)> {
    use std::collections::HashMap;
    let mut dupa_cheie: HashMap<&str, Vec<usize>> = HashMap::new();
    for &jj in liberi_j {
        dupa_cheie.entry(kn[jj].as_str()).or_default().push(jj);
    }
    let mut out = Vec::new();
    for &ii in liberi_i {
        if let Some(c) = dupa_cheie.get_mut(kv[ii].as_str()) {
            if !c.is_empty() {
                out.push((ii, c.remove(0)));
            }
        }
    }
    out
}

/// Cel mai lung subsir crescator pe `j` (pe `i` e deja crescator): perechile
/// care se pot pastra **in ordine**. Ele devin ancore (`egal`); restul s-au
/// incrucisat, deci sunt mutari.
///
/// `O(n log n)`, prin cozi de pacienta: pe un obiect cu multe blocuri identice
/// nepotrivite, varianta patratica ar fi fost chiar aici costul.
fn ancore_in_ordine(perechi: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if perechi.len() < 2 {
        return perechi.to_vec();
    }
    // `capete[l]` = indicele perechii cu cel mai mic `j` care incheie un subsir
    // crescator de lungime `l + 1`
    let mut capete: Vec<usize> = Vec::new();
    let mut de_la: Vec<usize> = vec![usize::MAX; perechi.len()];
    for (k, &(_, j)) in perechi.iter().enumerate() {
        let l = capete.partition_point(|&c| perechi[c].1 < j);
        if l > 0 {
            de_la[k] = capete[l - 1];
        }
        if l == capete.len() {
            capete.push(k);
        } else {
            capete[l] = k;
        }
    }
    let mut out = Vec::with_capacity(capete.len());
    let mut k = *capete.last().expect("cel putin o pereche");
    loop {
        out.push(perechi[k]);
        if de_la[k] == usize::MAX {
            break;
        }
        k = de_la[k];
    }
    out.reverse();
    out
}

/// Randurile pentru un interval, dintr-o lista de perechi deja hotarate.
///
/// Randurile cu text in stanga ies in ordinea blocurilor martorului, cele cu
/// text in dreapta in ordinea blocurilor tintei — pe asta se sprijina si
/// `aliniere`, si `plasare::plasari`, care numara blocurile tintei mergand
/// prin randuri.
fn emite(
    plan: &mut Vec<RandPlan>,
    i: std::ops::Range<usize>,
    j: std::ops::Range<usize>,
    perechi: &[(usize, usize, TipDiferenta)],
) {
    let (mut ii, mut jj) = (i.start, j.start);
    for &(pi, pj, tip) in perechi {
        while ii < pi {
            plan.push(RandPlan::nou(TipDiferenta::Sters, Some(ii), None));
            ii += 1;
        }
        while jj < pj {
            plan.push(RandPlan::nou(TipDiferenta::Adaugat, None, Some(jj)));
            jj += 1;
        }
        plan.push(RandPlan::nou(tip, Some(pi), Some(pj)));
        ii = pi + 1;
        jj = pj + 1;
    }
    while ii < i.end {
        plan.push(RandPlan::nou(TipDiferenta::Sters, Some(ii), None));
        ii += 1;
    }
    while jj < j.end {
        plan.push(RandPlan::nou(TipDiferenta::Adaugat, None, Some(jj)));
        jj += 1;
    }
}

/// Imperecherea blocurilor, fara sa se uite in interiorul lor.
///
/// Trei straturi, in ordinea increderii:
/// 1. **ancorele lui `SequenceMatcher`** — rulurile de blocuri identice ramase
///    in ordine; ele taie obiectul in goluri.
/// 2. **blocurile identice din goluri** — aceeasi cheie, deci acelasi text
///    normalizat (invarianta 1b din `CRATE.md`). Cele care se pot pastra in
///    ordine devin si ele ancore; cele incrucisate sunt **mutari**. Stratul
///    asta e mai tare decat scorul si se aplica inaintea lui: un bloc identic
///    nu are ce cauta intr-o pereche „modificat".
/// 3. **scorul**, pe ce a ramas liber in fiecare gol (`perechi_replace`).
fn plan_blocuri(
    sv: &[String],
    sn: &[String],
    kv: &[String],
    kn: &[String],
    pozitionale: &mut usize,
) -> Vec<RandPlan> {
    let (n, m) = (kv.len(), kn.len());
    let mut fixe: Vec<(usize, usize)> = Vec::new();
    let mut luat_i = vec![false; n];
    let mut luat_j = vec![false; m];
    for op in SequenceMatcher::new(kv, kn).opcodes() {
        if op.tag == Tag::Equal {
            for k in 0..(op.i2 - op.i1) {
                fixe.push((op.i1 + k, op.j1 + k));
                luat_i[op.i1 + k] = true;
                luat_j[op.j1 + k] = true;
            }
        }
    }

    // golurile dintre ancore, ca intervale pe amandoua partile
    let mut goluri: Vec<(std::ops::Range<usize>, std::ops::Range<usize>)> = Vec::new();
    let (mut i0, mut j0) = (0usize, 0usize);
    for &(a, b) in &fixe {
        goluri.push((i0..a, j0..b));
        (i0, j0) = (a + 1, b + 1);
    }
    goluri.push((i0..n, j0..m));

    // stratul 2: blocurile identice ramase libere, cautate **peste tot**, nu
    // doar in acelasi gol: un bloc si geamanul lui cad de obicei in goluri
    // diferite (sunt de o parte si de alta a unei ancore), si exact aia e o
    // reordonare. Perechea se scoate oricum din imperecherea pe scor — un bloc
    // identic nu are ce cauta intr-o pereche „modificat" — iar daca nu poate
    // fi ancora, ramane „sters" plus „adaugat" si `marcheaza_mutarile` le
    // uneste la final in `Mutat`.
    let liberi_i: Vec<usize> = (0..n).filter(|&k| !luat_i[k]).collect();
    let liberi_j: Vec<usize> = (0..m).filter(|&k| !luat_j[k]).collect();
    let identice = perechi_identice(kv, kn, &liberi_i, &liberi_j);
    let mut identice_luate_i = vec![false; n];
    let mut identice_luate_j = vec![false; m];
    for &(a, b) in &identice {
        identice_luate_i[a] = true;
        identice_luate_j[b] = true;
    }
    // Ancora noua doar cand amandoua capetele sunt in acelasi gol: altfel ar
    // sari peste o ancora a lui `SequenceMatcher` si ordinea s-ar inversa.
    // Golul unui bloc liber se afla prin cautare binara in ancore, nu
    // filtrand toate perechile pentru fiecare gol (ar fi patratic pe un obiect
    // cu multe blocuri identice nepotrivite).
    let golul_lui = |a: usize, b: usize| -> Option<usize> {
        let ga = fixe.partition_point(|&(x, _)| x < a);
        let gb = fixe.partition_point(|&(_, y)| y < b);
        (ga == gb).then_some(ga)
    };
    let mut in_gol: Vec<Vec<(usize, usize)>> = vec![Vec::new(); goluri.len()];
    for &(a, b) in &identice {
        if let Some(g) = golul_lui(a, b) {
            in_gol[g].push((a, b));
        }
    }
    let ancore_noi: Vec<Vec<(usize, usize)>> = in_gol.iter().map(|p| ancore_in_ordine(p)).collect();

    let mut plan: Vec<RandPlan> = Vec::new();
    for (k, (gi, gj)) in goluri.iter().enumerate() {
        let ancore = &ancore_noi[k];
        // subgolurile dintre ancorele noi, unde decide scorul
        let mut perechi: Vec<(usize, usize, TipDiferenta)> = ancore
            .iter()
            .map(|&(a, b)| (a, b, TipDiferenta::Egal))
            .collect();
        let mut margini: Vec<(usize, usize)> = vec![(gi.start, gj.start)];
        margini.extend(ancore.iter().map(|&(a, b)| (a + 1, b + 1)));
        let capete: Vec<(usize, usize)> = ancore
            .iter()
            .copied()
            .chain(std::iter::once((gi.end, gj.end)))
            .collect();
        for (&(ia, ja), &(ib, jb)) in margini.iter().zip(capete.iter()) {
            let libere_i: Vec<usize> = (ia..ib).filter(|&k| !identice_luate_i[k]).collect();
            let libere_j: Vec<usize> = (ja..jb).filter(|&k| !identice_luate_j[k]).collect();
            if libere_i.is_empty() || libere_j.is_empty() {
                continue;
            }
            let bv: Vec<String> = libere_i.iter().map(|&k| sv[k].clone()).collect();
            let bn: Vec<String> = libere_j.iter().map(|&k| sn[k].clone()).collect();
            if bv.len() * bn.len() > MAX_PRODUS_ALINIERE {
                *pozitionale += 1;
            }
            for (a, b) in perechi_replace(&bv, &bn) {
                let (ri, rj) = (libere_i[a], libere_j[b]);
                let tip = if kv[ri] == kn[rj] {
                    TipDiferenta::Egal
                } else {
                    TipDiferenta::Modificat
                };
                perechi.push((ri, rj, tip));
            }
        }
        perechi.sort_by_key(|p| p.0);
        emite(&mut plan, gi.clone(), gj.clone(), &perechi);
        // ancora lui `SequenceMatcher` de dupa gol
        if let Some(&(a, b)) = fixe.get(k) {
            plan.push(RandPlan::nou(TipDiferenta::Egal, Some(a), Some(b)));
        }
    }
    marcheaza_mutarile(&mut plan, kv, kn);
    plan
}

/// Tot ce iese dintr-o comparare, calculat o singura data.
///
/// Inainte, `aliniaza_cu` segmenta amandoua textele **inca o data** dupa
/// `diff_aliniat_cu_raport` (care le segmentase deja), doar ca sa afle
/// pozitiile blocurilor: patru segmentari pentru o aliniere, iar segmentarea
/// e 60% din costul diff-ului (masurat 2026-11-24). Aici blocurile ies o data,
/// cu pozitiile lor.
#[derive(Debug, Clone)]
pub struct Comparare {
    pub randuri: Vec<RandDiff>,
    /// Cate segmente `replace` au ramas pe imperechere pozitionala, fiind
    /// prea mari pentru imperecherea pe scor ([`MAX_PRODUS_ALINIERE`]).
    /// Acolo „modificat" poate insemna de fapt doua blocuri fara legatura —
    /// cine afiseaza trebuie sa poata spune asta, nu sa degradeze in tacere.
    pub segmente_pozitionale: usize,
    pub referinta: Segmentare,
    pub tinta: Segmentare,
}

/// Comparatia completa: blocurile amandoua parti, cu pozitii, si randurile.
pub fn compara(text_referinta: &str, text_curent: &str, uneste_cursor: bool) -> Comparare {
    let referinta = segmenteaza(text_referinta, uneste_cursor);
    let tinta = segmenteaza(text_curent, uneste_cursor);
    let kv: Vec<String> = referinta
        .blocuri
        .iter()
        .map(|b| cheie_bloc(&b.text))
        .collect();
    let kn: Vec<String> = tinta.blocuri.iter().map(|b| cheie_bloc(&b.text)).collect();
    let sv: Vec<String> = referinta.blocuri.iter().map(|b| b.text.clone()).collect();
    let sn: Vec<String> = tinta.blocuri.iter().map(|b| b.text.clone()).collect();
    let mut segmente_pozitionale = 0usize;
    let plan = plan_blocuri(&sv, &sn, &kv, &kn, &mut segmente_pozitionale);
    Comparare {
        randuri: randuri_din_plan(plan, &sv, &sn),
        segmente_pozitionale,
        referinta,
        tinta,
    }
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
    compara(text_referinta, text_curent, uneste_cursor).randuri
}

/// Ca [`diff_aliniat_cu`], plus cate segmente `replace` au ramas pe
/// imperechere pozitionala. Infasuratoare peste [`compara`].
pub fn diff_aliniat_cu_raport(
    text_referinta: &str,
    text_curent: &str,
    uneste_cursor: bool,
) -> (Vec<RandDiff>, usize) {
    let c = compara(text_referinta, text_curent, uneste_cursor);
    (c.randuri, c.segmente_pozitionale)
}

/// Randurile afisabile dintr-un plan: id-urile stabile si, pe blocurile
/// modificate, bucatile schimbate din interior.
fn randuri_din_plan(plan: Vec<RandPlan>, sv: &[String], sn: &[String]) -> Vec<RandDiff> {
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
                let (mut ss, mut ds) = diff_intra(&sv[i], &sn[j]);
                // blocul difera (normalizarea o spune), dar evidentierea a
                // socotit cosmetic tot ce s-a schimbat (`Sales.dbo.X` /
                // `Sales.X`, `a,b` / `a, b`): fara nimic aprins, editorul
                // spunea „fara diferente" la un obiect listat diferit (F-M1).
                // Atunci se aprinde si ce e cosmetic, ca sa se vada ce difera.
                let nimic = |v: &[Segment]| v.iter().all(|s| s.schimbat.is_none());
                if nimic(&ss) && nimic(&ds) {
                    (ss, ds) = diff_intra_cu(&sv[i], &sn[j], false);
                }
                (Some(ss), Some(ds))
            }
            _ => (None, None),
        };
        randuri.push(RandDiff {
            id,
            tip: r.tip,
            mutat_cu: r.mutat_cu,
            stanga,
            dreapta,
            seg_stanga,
            seg_dreapta,
        });
    }
    randuri
}

/// Cele doua parti ale unui rand, cat tine socoteala.
struct Parti<'a> {
    stanga: Option<&'a str>,
    dreapta: Option<&'a str>,
}

/// Cat s-a schimbat dintr-un obiect, pe blocuri.
///
/// Scorul din lista (`scor_similaritate`) e Ratcliff-Obershelp pe tot
/// obiectul: o instructiune schimbata din patruzeci da ~98, orice ar fi
/// schimbat in ea. Numarul asta spune altceva — **cat** din obiect e atins,
/// ponderat pe marimea blocurilor — si e descompus pe feluri, ca sa nu
/// trebuiasca crezut pe cuvant.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Masura {
    pub egale: usize,
    pub modificate: usize,
    /// O mutare se numara o data, desi are doua randuri.
    pub mutate: usize,
    pub sterse: usize,
    pub adaugate: usize,
    /// Procentul din obiect care **nu** e identic, ponderat pe marimea
    /// blocurilor (token-uri). Un bloc modificat intra cu partea lui
    /// schimbata, nu intreg.
    pub schimbat: f64,
}

/// [`Masura`] pe randuri deja calculate.
///
/// Un bloc **mutat** se socoteste neschimbat la `schimbat`: textul lui e
/// acelasi, s-a mutat doar. Faptul ca s-a mutat se vede in `mutate` si in
/// rezumat — care e exact rostul lui `Mutat`.
pub fn masoara(randuri: &[RandDiff]) -> Masura {
    masoara_din(
        randuri
            .iter()
            .map(|r| (r.tip, r.stanga.as_deref(), r.dreapta.as_deref())),
    )
}

/// Rezumatul si masura dintr-o singura trecere, fara diferentele din
/// interiorul blocurilor (partea scumpa). Asta foloseste lista de obiecte:
/// doua apeluri ar construi planul de doua ori.
pub fn rezumat_si_masura_rapide(text_referinta: &str, text_curent: &str) -> (String, Masura) {
    let sv = split_statements(text_referinta, false);
    let sn = split_statements(text_curent, false);
    let kv: Vec<String> = sv.iter().map(|b| cheie_bloc(b)).collect();
    let kn: Vec<String> = sn.iter().map(|b| cheie_bloc(b)).collect();
    let plan = plan_blocuri(&sv, &sn, &kv, &kn, &mut 0);
    let numara = |t: TipDiferenta| plan.iter().filter(|r| r.tip == t).count();
    let rezumat = formateaza_rezumat(
        numara(TipDiferenta::Modificat),
        numara(TipDiferenta::Adaugat),
        numara(TipDiferenta::Sters),
        plan.iter()
            .filter(|r| r.tip == TipDiferenta::Mutat && r.i.is_some())
            .count(),
    );
    let masura = masoara_din(plan.iter().map(|r| {
        (
            r.tip,
            r.i.map(|i| sv[i].as_str()),
            r.j.map(|j| sn[j].as_str()),
        )
    }));
    (rezumat, masura)
}

fn masoara_din<'a>(
    randuri: impl Iterator<Item = (TipDiferenta, Option<&'a str>, Option<&'a str>)>,
) -> Masura {
    let mut m = Masura::default();
    let (mut potrivit, mut total) = (0f64, 0f64);
    for (tip, stanga, dreapta) in randuri {
        let r = Parti { stanga, dreapta };
        let tok = |t: &Option<&str>| t.map_or(0usize, |x| tokeni_scor(x).len());
        match tip {
            TipDiferenta::Egal => {
                m.egale += 1;
                let n = (tok(&r.stanga) + tok(&r.dreapta)) as f64;
                potrivit += n;
                total += n;
            }
            TipDiferenta::Mutat => {
                m.mutate += usize::from(r.stanga.is_some());
                let n = (tok(&r.stanga) + tok(&r.dreapta)) as f64;
                potrivit += n;
                total += n;
            }
            TipDiferenta::Modificat => {
                m.modificate += 1;
                let (a, b) = (
                    tokeni_scor(r.stanga.unwrap_or("")),
                    tokeni_scor(r.dreapta.unwrap_or("")),
                );
                let n = (a.len() + b.len()) as f64;
                potrivit += n * scor_pe_tokeni(&a, &b) / 100.0;
                total += n;
            }
            TipDiferenta::Sters => {
                m.sterse += 1;
                total += tok(&r.stanga) as f64;
            }
            TipDiferenta::Adaugat => {
                m.adaugate += 1;
                total += tok(&r.dreapta) as f64;
            }
        }
    }
    m.schimbat = if total <= 0.0 {
        0.0
    } else {
        crate::normalizare::round_py(100.0 * (1.0 - potrivit / total), 2)
    };
    m
}

/// Bucatile schimbate, asezate pe linii: `rezultat[i]` sunt intervalele (in
/// caractere) marcate pe linia `i` a blocului.
///
/// Segmentele din [`RandDiff`] sunt pe bloc, nu pe linie, iar cine deseneaza
/// are nevoie de ele pe linie. Fara functia asta, fiecare consumator isi face
/// propria taiere pe `\n` — si a doua implementare a aceleiasi socoteli e
/// exact ce evitam.
///
/// **Pozitiile sunt in caractere, nu in octeti** — spre deosebire de
/// [`crate::lexer::Segment`], care e in octeti. Aici se deseneaza text, deci
/// coloana cerut e cea vazuta de om. Un `&text[a..b]` cu perechile astea da
/// panica pe orice caracter multi-octet: se indexeaza printr-un
/// `Vec<char>` (asa face `cli::evidentiat`).
pub fn marcaje_pe_linii(segmente: &[Segment]) -> Vec<Vec<(usize, usize)>> {
    let mut linii: Vec<Vec<(usize, usize)>> = vec![Vec::new()];
    let mut coloana = 0usize;
    for s in segmente {
        for (k, bucata) in s.text.split('\n').enumerate() {
            if k > 0 {
                linii.push(Vec::new());
                coloana = 0;
            }
            let n = bucata.chars().count();
            if s.schimbat.is_some() && n > 0 {
                let linie = linii.last_mut().expect("are mereu o linie");
                match linie.last_mut() {
                    // doua marcaje lipite se unesc, altfel ar iesi doua perechi
                    // de paranteze una langa alta
                    Some(ultim) if ultim.1 == coloana => ultim.1 = coloana + n,
                    _ => linie.push((coloana, coloana + n)),
                }
            }
            coloana += n;
        }
    }
    linii
}

fn formateaza_rezumat(modificate: usize, adaugate: usize, sterse: usize, mutate: usize) -> String {
    let mut p: Vec<String> = Vec::with_capacity(4);
    if modificate > 0 {
        p.push(format!("{} modificate", modificate));
    }
    if adaugate > 0 {
        p.push(format!("{} adaugate", adaugate));
    }
    if sterse > 0 {
        p.push(format!("{} sterse", sterse));
    }
    // o mutare e un singur lucru, desi are doua randuri
    if mutate > 0 {
        p.push(format!("{} mutate", mutate));
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
    rezumat_si_masura_rapide(text_referinta, text_curent).0
}

/// Rezumat din randurile deja calculate de `diff_aliniat`.
pub fn rezumat_diff(randuri: &[RandDiff]) -> String {
    let numara = |t: TipDiferenta| randuri.iter().filter(|r| r.tip == t).count();
    formateaza_rezumat(
        numara(TipDiferenta::Modificat),
        numara(TipDiferenta::Adaugat),
        numara(TipDiferenta::Sters),
        randuri
            .iter()
            .filter(|r| r.tip == TipDiferenta::Mutat && r.stanga.is_some())
            .count(),
    )
}

#[cfg(test)]
mod teste_b11 {
    /// `diff_aliniat_cu` e API public si nu avea test propriu (B-11 din
    /// auditul 2026-11-23): cu `uneste_cursor` oprit da exact `diff_aliniat`,
    /// iar cu el pornit ciclul de cursor iese un singur bloc.
    #[test]
    fn diff_aliniat_cu_respecta_unirea_cursorului() {
        let a = "DECLARE c CURSOR LOCAL FOR\nSELECT id FROM t\nOPEN c\nFETCH NEXT FROM c\nCLOSE c";
        let b = "DECLARE c CURSOR LOCAL FOR\nSELECT id FROM t2\nOPEN c\nFETCH NEXT FROM c\nCLOSE c";
        assert_eq!(
            super::diff_aliniat_cu(a, b, false),
            super::diff_aliniat(a, b),
            "fara unire e acelasi lucru ca `diff_aliniat`"
        );
        let unit = super::diff_aliniat_cu(a, b, true);
        assert!(
            unit.len() <= super::diff_aliniat(a, b).len(),
            "unirea nu poate da mai multe blocuri: {} vs {}",
            unit.len(),
            super::diff_aliniat(a, b).len()
        );
    }
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
    /// Doua spatii in loc de unul, in string, pe acelasi rand, sunt alte date
    /// (Mihai, 2026-11-22; v3.4 le socotea egale). Reindentarea nu.
    #[test]
    fn spatiile_din_string_se_marcheaza_cele_din_cod_nu() {
        assert_eq!(
            tipuri(&diff_aliniat("PRINT 'a b'", "PRINT 'a  b'")),
            vec!["modificat"]
        );
        assert_eq!(
            tipuri(&diff_aliniat("PRINT 'a\n  b'", "PRINT 'a\n\tb'")),
            vec!["egal"]
        );
        let r = diff_aliniat(
            "PRINT 'mesaj catre partenerul'",
            "PRINT 'mesaj catre partenerul '",
        );
        let seg = r[0].seg_dreapta.as_ref().expect("segmente");
        assert!(seg.iter().any(|s| s.schimbat.is_some()), "{seg:?}");
        // un spatiu in cod nu se aprinde cand mai e altceva de aratat...
        let r = diff_aliniat(
            "SELECT a,b FROM t WHERE x = 1",
            "SELECT a,  b FROM t WHERE x = 2",
        );
        let aprinse: Vec<&str> = r[0]
            .seg_dreapta
            .iter()
            .flatten()
            .filter(|g| g.schimbat.is_some())
            .map(|g| g.text.as_str())
            .collect();
        assert!(
            !aprinse.is_empty() && aprinse.iter().all(|t| !t.trim().is_empty()),
            "{aprinse:?}"
        );
        // ...dar cand e singura diferenta se aprinde, altfel blocul
        // „modificat" n-ar arata nimic (F-M1)
        let r = diff_aliniat("SELECT a,b FROM t", "SELECT a,  b FROM t");
        assert!(
            r.iter().any(|x| x
                .seg_dreapta
                .as_ref()
                .is_some_and(|s| s.iter().any(|g| g.schimbat.is_some()))),
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
    /// (tab in loc de spatii) nu se evidentiaza; un spatiu aparut in string,
    /// sau un tab in loc de spatiu pe acelasi rand, da (Mihai, 2026-11-22).
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
            "SET @s = ') W\n\t\tJOIN t X ON W.a=X.a ' + @x",
            "SET @s = ') W\n            JOIN t X ON W.a=X.a ' + @y",
        );
        assert_eq!(m, ["@x", "@y"], "doar variabila difera: {m:?}");
        let m = marcat("PRINT 'a b'", "PRINT 'ab'");
        assert!(!m.is_empty(), "spatiul disparut din string se vede");
        let m = marcat("PRINT 'a\tb'", "PRINT 'a b'");
        assert!(
            !m.is_empty(),
            "tabul in loc de spatiu, pe acelasi rand, se vede"
        );
    }

    /// F-M1: un bloc modificat are mereu ceva aprins.
    #[test]
    fn un_bloc_modificat_are_mereu_ceva_aprins() {
        for (a, b) in [
            (
                "SELECT a FROM Sales.dbo.Orders",
                "SELECT a FROM Sales.Orders",
            ),
            ("SELECT [a b] FROM t", "SELECT a b FROM t"),
            ("SELECT * FROM f(nolock)", "SELECT * FROM f()"),
        ] {
            let r = diff_aliniat(a, b);
            assert_eq!(r[0].tip, TipDiferenta::Modificat, "{a}");
            let aprinse = r[0]
                .seg_stanga
                .iter()
                .chain(r[0].seg_dreapta.iter())
                .flatten()
                .filter(|s| s.schimbat.is_some())
                .count();
            assert!(aprinse > 0, "{a:?} / {b:?}: nimic aprins");
        }
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
            "       NULL AS LocalDiskSavePath,\n",
            "       NULL AS LocalDiskSaveName"
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
    fn marcajele_se_taie_pe_linii() {
        let r = diff_aliniat(
            "SELECT a\nFROM vechi\nWHERE x = 1",
            "SELECT a\nFROM nou\nWHERE x = 1",
        );
        let seg = r[0].seg_dreapta.as_ref().expect("segmente");
        let m = marcaje_pe_linii(seg);
        assert_eq!(m.len(), 3, "trei linii: {m:?}");
        assert!(m[0].is_empty(), "prima linie e neatinsa: {m:?}");
        assert!(!m[1].is_empty(), "a doua linie are marcaj: {m:?}");
        assert!(m[2].is_empty(), "a treia linie e neatinsa: {m:?}");
        // marcajul cade pe `nou`, in coloanele lui
        let linie = "FROM nou";
        let (a, b) = m[1][0];
        assert_eq!(&linie[a..b], "nou", "{m:?}");
    }

    #[test]
    fn rezumat_fara_diferente() {
        let t = "SELECT 1";
        assert_eq!(rezumat_rapid(t, t), "diferente minore");
        assert_eq!(rezumat_diff(&diff_aliniat(t, t)), "diferente minore");
    }

    /// Reordonarea: acelasi text, alt loc. Pana la etapa 30 ieseau „sters" si
    /// „adaugat", doua diferente fara nicio legatura vizibila.
    #[test]
    fn blocul_reordonat_e_mutat() {
        let a = "SELECT 1 FROM dbo.a;\n\nSELECT 2 FROM dbo.b;\n\nSELECT 3 FROM dbo.c;";
        let b = "SELECT 3 FROM dbo.c;\n\nSELECT 1 FROM dbo.a;\n\nSELECT 2 FROM dbo.b;";
        let r = diff_aliniat(a, b);
        let mutate: Vec<&RandDiff> = r.iter().filter(|x| x.tip == TipDiferenta::Mutat).collect();
        assert_eq!(mutate.len(), 2, "doua randuri pentru o mutare: {r:#?}");
        // capetele se arata reciproc
        let st = r
            .iter()
            .position(|x| x.tip == TipDiferenta::Mutat && x.stanga.is_some());
        let dr = r
            .iter()
            .position(|x| x.tip == TipDiferenta::Mutat && x.dreapta.is_some());
        assert_eq!(r[st.unwrap()].mutat_cu, dr);
        assert_eq!(r[dr.unwrap()].mutat_cu, st);
        assert_eq!(rezumat_diff(&r), "1 mutate");
    }

    /// Un bloc mutat **si** schimbat ramane „sters" + „adaugat": „mutat"
    /// inseamna text identic, altfel ar fi o afirmatie pe care motorul nu o
    /// poate sustine fara scor.
    #[test]
    fn mutat_cere_text_identic() {
        let a = "SELECT 1 FROM dbo.a;\n\nSELECT 2 FROM dbo.b;";
        let b = "SELECT 2 FROM dbo.b WHERE x = 1;\n\nSELECT 1 FROM dbo.a;";
        let r = diff_aliniat(a, b);
        assert!(!r.iter().any(|x| x.tip == TipDiferenta::Mutat), "{r:#?}");
    }

    #[test]
    fn masura_descompune_schimbarea() {
        let a = "SELECT 1 FROM dbo.a;\n\nSELECT 2 FROM dbo.b;\n\nSELECT 3 FROM dbo.c;";
        let b = "SELECT 1 FROM dbo.a;\n\nSELECT 2 FROM dbo.b;\n\nSELECT 4 FROM dbo.c;";
        let m = masoara(&diff_aliniat(a, b));
        assert_eq!((m.egale, m.modificate, m.sterse, m.adaugate), (2, 1, 0, 0));
        // o bucata mica dintr-un bloc din trei: departe de „2% diferit" al
        // scorului pe tot obiectul, dar tot mic
        assert!(m.schimbat > 0.0 && m.schimbat < 20.0, "{m:?}");

        let m = masoara(&diff_aliniat(a, a));
        assert_eq!(m.schimbat, 0.0);
        assert_eq!(m.egale, 3);
    }

    /// O mutare nu schimba nimic din text, deci `schimbat` ramane zero — dar se
    /// numara ca mutare.
    #[test]
    fn mutarea_nu_e_text_schimbat() {
        let a = "SELECT 1 FROM dbo.a;\n\nSELECT 2 FROM dbo.b;\n\nSELECT 3 FROM dbo.c;";
        let b = "SELECT 3 FROM dbo.c;\n\nSELECT 1 FROM dbo.a;\n\nSELECT 2 FROM dbo.b;";
        let m = masoara(&diff_aliniat(a, b));
        assert_eq!(m.schimbat, 0.0, "{m:?}");
        assert_eq!(m.mutate, 1);
    }

    #[test]
    fn ancorele_in_ordine_dau_cel_mai_lung_subsir_crescator() {
        // doua perechi incrucisate: se pastreaza una
        let a = ancore_in_ordine(&[(0, 5), (1, 2)]);
        assert_eq!(a.len(), 1);
        // deja crescator: se pastreaza tot
        assert_eq!(
            ancore_in_ordine(&[(0, 1), (1, 3), (2, 7)]),
            vec![(0, 1), (1, 3), (2, 7)]
        );
        // un singur intrus: cade intrusul, nu restul
        assert_eq!(
            ancore_in_ordine(&[(0, 1), (1, 9), (2, 3), (3, 4)]),
            vec![(0, 1), (2, 3), (3, 4)]
        );
        assert!(ancore_in_ordine(&[]).is_empty());
        assert_eq!(ancore_in_ordine(&[(4, 4)]), vec![(4, 4)]);
        // rezultatul e mereu crescator pe amandoua
        let lung: Vec<(usize, usize)> = (0..200).map(|k| (k, (k * 7919) % 211)).collect();
        let r = ancore_in_ordine(&lung);
        assert!(
            r.windows(2).all(|w| w[0].0 < w[1].0 && w[0].1 < w[1].1),
            "{r:?}"
        );
    }

    /// Un bloc identic de cealalta parte a unei ancore nu poate fi ancora el
    /// insusi (ar sari peste cea veche si ar inversa ordinea), deci e o mutare.
    /// Si, mai important, **nu** intra in imperecherea pe scor: inainte era
    /// luat drept pereche a vecinului si raportat „modificat".
    #[test]
    fn blocul_identic_de_dincolo_de_ancora_e_mutare_nu_modificare() {
        let a = "SELECT unu FROM dbo.a;\n\nSELECT ancora FROM dbo.z;\n\nSELECT doi FROM dbo.b;";
        let b = "SELECT doi FROM dbo.b;\n\nSELECT ancora FROM dbo.z;\n\nSELECT unu FROM dbo.a;";
        let r = diff_aliniat(a, b);
        let m = masoara(&r);
        assert_eq!(
            (m.modificate, m.sterse, m.adaugate),
            (0, 0, 0),
            "nimic nu s-a schimbat, doar s-a mutat: {r:#?}"
        );
        assert!(m.mutate >= 1, "{m:?}");
        assert_eq!(m.schimbat, 0.0);
    }
}
