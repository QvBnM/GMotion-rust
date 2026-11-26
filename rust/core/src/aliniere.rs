//! Alinierea pe linii a doua texte, si compunerea textului final din ea.
//!
//! **Se strica astfel:** panourile editorului nu se mai aliniaza (blocul din
//! stanga nu e langa perechea lui), sau textul final compus dupa „Aplica"
//! pierde ori dubleaza linii — si atunci deploy-ul scrie in baza alt cod decat
//! cel vazut pe ecran. Editorul (`fereastra`) si compunerea (`diff-obiecte`,
//! deci si `cli`) folosesc **aceeasi** aliniere, de aici: un singur motor.
//!
//! Blocurile, imperecherea si bucatile schimbate vin din [`diff_aliniat`];
//! aici doar se aseaza pe linii, cu liniile dintre blocuri (goale, sau
//! comentarii pe care segmentarea nu le pastreaza) completate din textul
//! original, ca sa se vada tot obiectul.

#[cfg(test)]
use crate::diff::diff_aliniat;
use crate::diff::{RandDiff, TipDiferenta};
use crate::segmentare::{split_statements, Bloc};

/// O linie dintr-o parte, cu numarul ei real (de la 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linie {
    pub nr: usize,
    pub text: String,
}

/// Un rand al alinierii: o linie in stanga (referinta / sursa), una in
/// dreapta (tinta / baza), sau umplutura (`None`) intr-una din parti.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandAliniat {
    pub stanga: Option<Linie>,
    pub dreapta: Option<Linie>,
    pub tip: TipDiferenta,
    /// Indexul in [`Aliniere::diff`] al blocului din care face parte randul;
    /// `None` pe liniile dintre blocuri, luate din textul original.
    pub bloc: Option<usize>,
    /// Pozitia liniei in textul blocului, pe fiecare parte (`0` = prima linie
    /// a blocului), sau `None` unde partea e umplutura.
    ///
    /// Doua campuri, nu unul: de cand liniile se imperecheaza intre ele si nu
    /// pe pozitie, linia a treia din stanga poate sta langa a cincea din
    /// dreapta.
    pub linie_in_bloc_stanga: Option<usize>,
    pub linie_in_bloc_dreapta: Option<usize>,
    /// Primul, respectiv ultimul rand al blocului din care face parte randul.
    /// Interfata deseneaza chenarul blocului din ele; fara ele ar trebui sa
    /// se uite la randul vecin, adica sa refaca aici o socoteala a motorului.
    pub primul_din_bloc: bool,
    pub ultimul_din_bloc: bool,
    /// Linia e goala sau numai comentariu, deci n-a intrat in imperechere:
    /// normalizarea o ignora cand decide „egal / modificat". Interfata o poate
    /// arata altfel („ignorat la comparare") sau o poate plia.
    pub ignorata: bool,
    /// Indexul (de la 0) liniei din dreapta inaintea careia sta randul. Pe un
    /// rand fara linie in dreapta, acolo s-ar insera ce se aduce din stanga.
    pub poz_dreapta: usize,
}

/// Rezultatul alinierii.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aliniere {
    pub randuri: Vec<RandAliniat>,
    /// Blocurile date de motor, in ordine. `RandAliniat::bloc` arata aici.
    pub diff: Vec<RandDiff>,
    /// Blocurile prea mari pentru imperecherea pe scor a liniilor
    /// (`MAX_PRODUS_LINII`): acolo liniile sunt puse pozitional, ca inainte.
    /// Se raporteaza, ca sa nu degradeze in tacere.
    pub blocuri_nealiniate: Vec<usize>,
    /// Cate segmente `replace` au fost prea mari pentru imperecherea pe scor a
    /// **blocurilor** (`diff::MAX_PRODUS_ALINIERE`). Acolo „modificat" poate
    /// insemna doua blocuri fara legatura, imperecheate doar pozitional.
    pub segmente_pozitionale: usize,
}

impl Aliniere {
    /// Indexurile (in `diff`) blocurilor care difera, in ordine: ce se poate
    /// aplica.
    pub fn diferente(&self) -> Vec<usize> {
        self.diff
            .iter()
            .enumerate()
            .filter(|(_, d)| d.tip != TipDiferenta::Egal)
            .map(|(i, _)| i)
            .collect()
    }
}

/// Sub acest scor, doua linii dintr-un segment `replace` nu se imperecheaza:
/// se arata ca „lipseste" plus „in plus", ceea ce descrie mai bine ce s-a
/// intamplat decat o pereche fortata intre doua linii fara legatura.
const PRAG_LINIE: f64 = 40.0;

/// Peste acest produs `n*m` nu se mai cauta perechi prin programare dinamica
/// intr-un segment `replace` de linii; se cade pe imperechere pozitionala.
/// Blocurile atat de mari sunt rare, si cand se intampla **se spune**: vezi
/// [`Aliniere::blocuri_nealiniate`]. Urcat de la 4096 odata cu retezarea pe
/// lungimi din [`crate::normalizare::scor_maxim_posibil`].
const MAX_PRODUS_LINII: usize = 16_384;

/// Liniile care nu intra in imperechere: goale, sau numai comentariu.
///
/// Comentariile nu conteaza la „egal / modificat" (normalizarea le sterge),
/// deci nu au ce cauta nici la imperecherea liniilor: daca intra, trag
/// alinierea dupa ele si `AS` ajunge langa alt rand. Se reaseaza dupa, la
/// locul lor. Granitele vin din `lexer`, ca peste tot.
fn linii_ignorate(bloc: &str) -> Vec<bool> {
    use crate::lexer::{segmente, Fel};
    let comentarii: Vec<(usize, usize)> = segmente(bloc)
        .into_iter()
        .filter(|s| matches!(s.fel, Fel::ComentariuLinie | Fel::ComentariuBloc))
        .map(|s| (s.start, s.sfarsit))
        .collect();
    let mut out = Vec::new();
    let mut poz = 0usize;
    for l in bloc.split('\n') {
        let a = poz + (l.len() - l.trim_start().len());
        let b = poz + l.trim_end().len();
        let gol = l.trim().is_empty();
        out.push(gol || comentarii.iter().any(|&(x, y)| x <= a && b <= y));
        poz += l.len() + 1;
    }
    out
}

/// Liniile de umplutura dintre doua blocuri (`[s.0, s.1)` in stanga,
/// `[d.0, d.1)` in dreapta), puse alaturi: cele identice (fara spatiile de la
/// capete) pereche, in ordine; in rest, una langa alta cat ajung, apoi cele
/// ramase singure.
fn perechi_umplutura(
    st: &[&str],
    s: (usize, usize),
    dr: &[&str],
    d: (usize, usize),
) -> Vec<(Option<usize>, Option<usize>)> {
    use crate::seqmatch::{SequenceMatcher, Tag};
    let (s1, d1) = (s.1.max(s.0), d.1.max(d.0));
    let chei = |v: &[&str], a: usize, b: usize| -> Vec<String> {
        (a..b)
            .map(|k| v.get(k).copied().unwrap_or("").trim().to_string())
            .collect()
    };
    let (ka, kb) = (chei(st, s.0, s1), chei(dr, d.0, d1));
    let mut out = Vec::new();
    for op in SequenceMatcher::new(&ka, &kb).opcodes() {
        let (n, m) = (op.i2 - op.i1, op.j2 - op.j1);
        let lung = if op.tag == Tag::Equal { n } else { n.max(m) };
        for k in 0..lung {
            out.push((
                (k < n).then_some(s.0 + op.i1 + k),
                (k < m).then_some(d.0 + op.j1 + k),
            ));
        }
    }
    out
}

/// Imperecherea liniilor dintr-un segment `replace`, pe scor, fara
/// incrucisari. Acelasi mecanism ca la blocuri (`diff::perechi_replace`), un
/// nivel mai jos.
fn perechi_pe_scor(st: &[&str], dr: &[&str]) -> Vec<(usize, usize)> {
    use crate::normalizare::{scor_pe_tokeni, tokeni_scor};
    let (n, m) = (st.len(), dr.len());
    if n == 0 || m == 0 {
        return Vec::new();
    }
    if n * m > MAX_PRODUS_LINII {
        return (0..n.min(m)).map(|i| (i, i)).collect();
    }
    let tv: Vec<Vec<String>> = st.iter().map(|l| tokeni_scor(l)).collect();
    let tn: Vec<Vec<String>> = dr.iter().map(|l| tokeni_scor(l)).collect();
    // retezare exacta pe lungimi, ca la blocuri (`diff::perechi_replace`)
    let mut s = vec![vec![0f64; m]; n];
    for i in 0..n {
        for j in 0..m {
            if crate::normalizare::scor_maxim_posibil(tv[i].len(), tn[j].len()) < PRAG_LINIE {
                continue;
            }
            s[i][j] = scor_pe_tokeni(&tv[i], &tn[j]);
        }
    }
    let mut d = vec![vec![0f64; m + 1]; n + 1];
    let mut p = vec![vec![0u8; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            let scor = s[i - 1][j - 1];
            let diag = d[i - 1][j - 1] + if scor >= PRAG_LINIE { scor } else { 0.0 };
            let (sus, stanga) = (d[i - 1][j], d[i][j - 1]);
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
    let (mut i, mut j) = (n, m);
    let mut perechi = Vec::new();
    while i > 0 && j > 0 {
        match p[i][j] {
            1 => {
                if s[i - 1][j - 1] >= PRAG_LINIE {
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

/// Cum se aseaza liniile unui bloc fata de ale perechii lui:
/// `(indice in stanga, indice in dreapta)`, in ordinea de afisare.
///
/// **Se strica astfel:** panourile arata doua linii fara legatura una langa
/// alta (`AS` langa `@UserId INT,`), sau o linie dispare din afisare.
fn perechi_linii(bloc_st: &str, bloc_dr: &str) -> Vec<(Option<usize>, Option<usize>, bool)> {
    use crate::normalizare::norm;
    use crate::seqmatch::{SequenceMatcher, Tag};

    let linii_st: Vec<&str> = bloc_st.split('\n').collect();
    let linii_dr: Vec<&str> = bloc_dr.split('\n').collect();
    let ign_st = linii_ignorate(bloc_st);
    let ign_dr = linii_ignorate(bloc_dr);

    let idx_st: Vec<usize> = (0..linii_st.len()).filter(|&i| !ign_st[i]).collect();
    let idx_dr: Vec<usize> = (0..linii_dr.len()).filter(|&i| !ign_dr[i]).collect();
    let txt_st: Vec<&str> = idx_st.iter().map(|&i| linii_st[i]).collect();
    let txt_dr: Vec<&str> = idx_dr.iter().map(|&i| linii_dr[i]).collect();
    let chei_st: Vec<String> = txt_st.iter().map(|l| norm(l)).collect();
    let chei_dr: Vec<String> = txt_dr.iter().map(|l| norm(l)).collect();

    let mut out: Vec<(Option<usize>, Option<usize>, bool)> = Vec::new();
    for op in SequenceMatcher::new(&chei_st, &chei_dr).opcodes() {
        match op.tag {
            Tag::Equal => {
                for k in 0..(op.i2 - op.i1) {
                    out.push((Some(idx_st[op.i1 + k]), Some(idx_dr[op.j1 + k]), false));
                }
            }
            Tag::Replace => {
                let a = &txt_st[op.i1..op.i2];
                let b = &txt_dr[op.j1..op.j2];
                let perechi = perechi_pe_scor(a, b);
                let (mut pi, mut ii, mut jj) = (0usize, 0usize, 0usize);
                while ii < a.len() || jj < b.len() {
                    let pr = perechi.get(pi).copied();
                    if pr == Some((ii, jj)) {
                        out.push((Some(idx_st[op.i1 + ii]), Some(idx_dr[op.j1 + jj]), false));
                        ii += 1;
                        jj += 1;
                        pi += 1;
                        continue;
                    }
                    let (urm_i, urm_j) = pr.unwrap_or((a.len(), b.len()));
                    if ii < urm_i {
                        out.push((Some(idx_st[op.i1 + ii]), None, false));
                        ii += 1;
                        continue;
                    }
                    if jj < urm_j {
                        out.push((None, Some(idx_dr[op.j1 + jj]), false));
                        jj += 1;
                        continue;
                    }
                    break;
                }
            }
            Tag::Delete => {
                for &i in &idx_st[op.i1..op.i2] {
                    out.push((Some(i), None, false));
                }
            }
            Tag::Insert => {
                for &j in &idx_dr[op.j1..op.j2] {
                    out.push((None, Some(j), false));
                }
            }
        }
    }

    // liniile ignorate se pun la loc, dupa linia neignorata care le precede
    let reaseaza =
        |out: &mut Vec<(Option<usize>, Option<usize>, bool)>, ign: &[bool], stanga: bool| {
            for i in (0..ign.len()).rev() {
                if !ign[i] {
                    continue;
                }
                let poz = out
                    .iter()
                    .position(|r| {
                        let v = if stanga { r.0 } else { r.1 };
                        v.is_some_and(|x| x > i)
                    })
                    .unwrap_or(out.len());
                out.insert(
                    poz,
                    if stanga {
                        (Some(i), None, true)
                    } else {
                        (None, Some(i), true)
                    },
                );
            }
        };
    reaseaza(&mut out, &ign_st, true);
    // In dreapta, o linie ignorata (comentariu, linie goala) isi cauta intai
    // geamanul din stanga in locul unde ar ajunge: `-- detail --` langa
    // `-- detail --`. Pana pe 2026-11-26 se puneau separat, deci aceleasi
    // comentarii apareau o data doar in stanga si o data doar in dreapta
    // (Stock_StockProcessDispatchAfterProcess_UpdateItemCost, Mihai).
    for i in (0..ign_dr.len()).filter(|&i| ign_dr[i]) {
        let poz = out
            .iter()
            .position(|r| r.1.is_some_and(|x| x > i))
            .unwrap_or(out.len());
        let mut k = poz;
        while k > 0 && out[k - 1].1.is_none() {
            k -= 1;
        }
        let geaman = (k..poz).find(|&j| {
            out[j].2
                && out[j]
                    .0
                    .is_some_and(|li| linii_st[li].trim() == linii_dr[i].trim())
        });
        match geaman {
            Some(j) => out[j].1 = Some(i),
            None => out.insert(poz, (None, Some(i), true)),
        }
    }
    out
}

/// Aliniaza doua texte deja pregatite pentru afisare
/// ([`crate::pregateste_afisare`]).
pub fn aliniaza(text_stanga: &str, text_dreapta: &str) -> Aliniere {
    aliniaza_cu(text_stanga, text_dreapta, false)
}

/// Ca [`aliniaza`], dar cu „uneste ciclul cursorului" pornit: blocurile se
/// taie altfel, deci si alinierea. Optiunea trebuie sa ajunga **si** la diff,
/// **si** la inceputurile de blocuri — altfel cele doua n-ar mai vorbi despre
/// aceleasi blocuri.
pub fn aliniaza_cu(text_stanga: &str, text_dreapta: &str, uneste_cursor: bool) -> Aliniere {
    let c = crate::diff::compara(text_stanga, text_dreapta, uneste_cursor);
    let (diff, segmente_pozitionale) = (c.randuri, c.segmente_pozitionale);
    let (bl_st, bl_dr) = (&c.referinta.blocuri, &c.tinta.blocuri);
    let original_st: Vec<&str> = text_stanga.lines().collect();
    let original_dr: Vec<&str> = text_dreapta.lines().collect();

    let mut randuri: Vec<RandAliniat> = Vec::new();
    let mut nealiniate: Vec<usize> = Vec::new();
    let (mut bloc_st, mut bloc_dr) = (0usize, 0usize);
    let (mut ultim_st, mut ultim_dr) = (0usize, 0usize);

    let completeaza = |randuri: &mut Vec<RandAliniat>, s: (usize, usize), d: (usize, usize)| {
        // Liniile dintre blocuri (comentarii, linii goale) se pun alaturi dupa
        // continut, nu dupa pozitie: un comentariu care exista doar intr-o
        // parte sta singur pe randul lui, iar cele identice raman pereche
        // (etapa 34; pana atunci, un comentariu in plus intr-o parte decala
        // toate cele de dupa el).
        let linie = |k: usize, original: &[&str]| Linie {
            nr: k + 1,
            text: original.get(k).copied().unwrap_or("").to_string(),
        };
        let perechi = perechi_umplutura(&original_st, s, &original_dr, d);
        for (k, &(ls, ld)) in perechi.iter().enumerate() {
            // fara linie in dreapta: locul e inaintea urmatoarei linii din
            // dreapta a golului
            let urmatoarea = perechi[k..].iter().find_map(|p| p.1).unwrap_or(d.1);
            randuri.push(RandAliniat {
                stanga: ls.map(|k| linie(k, &original_st)),
                dreapta: ld.map(|k| linie(k, &original_dr)),
                tip: TipDiferenta::Egal,
                bloc: None,
                primul_din_bloc: false,
                ultimul_din_bloc: false,
                linie_in_bloc_stanga: None,
                linie_in_bloc_dreapta: None,
                ignorata: false,
                poz_dreapta: ld.unwrap_or(urmatoarea),
            });
        }
    };

    for (indice, rd) in diff.iter().enumerate() {
        // liniile reale ale blocului, din segmentare: `Bloc::linii`. Inainte
        // se pornea de la o linie de start si se reconstruia harta liniilor
        // prin potrivire in textul original (`harta_linii`), ceea ce ghicea de
        // doua ori acelasi lucru.
        let h_st = rd.stanga.as_ref().map(|_| {
            let h = bl_st.get(bloc_st).map(|b: &Bloc| b.linii.clone());
            bloc_st += 1;
            h.unwrap_or_default()
        });
        let h_dr = rd.dreapta.as_ref().map(|_| {
            let h = bl_dr.get(bloc_dr).map(|b: &Bloc| b.linii.clone());
            bloc_dr += 1;
            h.unwrap_or_default()
        });
        let prima = |h: &Option<Vec<usize>>, implicit: usize| {
            h.as_ref()
                .and_then(|v| v.first().copied())
                .map_or(implicit, |l| l.max(implicit))
        };
        let s_st = h_st.as_ref().map(|_| prima(&h_st, ultim_st));
        let s_dr = h_dr.as_ref().map(|_| prima(&h_dr, ultim_dr));
        completeaza(
            &mut randuri,
            (ultim_st, s_st.unwrap_or(ultim_st)),
            (ultim_dr, s_dr.unwrap_or(ultim_dr)),
        );
        let linii_st: Vec<&str> = rd
            .stanga
            .as_deref()
            .map_or(Vec::new(), |t| t.split('\n').collect());
        let linii_dr: Vec<&str> = rd
            .dreapta
            .as_deref()
            .map_or(Vec::new(), |t| t.split('\n').collect());
        let inceput_dr = s_dr.unwrap_or(ultim_dr);
        // Blocurile imperecheate isi aliniaza liniile intre ele; cele
        // dintr-o singura parte raman cum sunt.
        let perechi: Vec<(Option<usize>, Option<usize>, bool)> =
            match (rd.stanga.as_deref(), rd.dreapta.as_deref()) {
                (Some(a), Some(b)) => {
                    if linii_st.len() * linii_dr.len() > MAX_PRODUS_LINII {
                        nealiniate.push(indice);
                    }
                    perechi_linii(a, b)
                }
                _ => (0..linii_st.len().max(linii_dr.len()))
                    .map(|i| {
                        (
                            (i < linii_st.len()).then_some(i),
                            (i < linii_dr.len()).then_some(i),
                            false,
                        )
                    })
                    .collect(),
            };
        // linia din cod a fiecarei linii din bloc: un bloc unit (UNION,
        // cursor) a pierdut liniile goale dintre parti, deci nu e contiguu in
        // cod (N3 din `cercetare/audit-qa-2026-11-23.md`). Vine din
        // segmentare, nu din potrivire pe text.
        let harta_st = h_st;
        let harta_dr = h_dr;
        for (k, (is, id, ignorata)) in perechi.iter().copied().enumerate() {
            let linie = |idx: Option<usize>, linii: &[&str], harta: &Option<Vec<usize>>| match (
                idx.and_then(|i| linii.get(i)),
                idx,
                harta,
            ) {
                (Some(t), Some(i), Some(h)) => Some(Linie {
                    nr: h.get(i).copied().unwrap_or(i) + 1,
                    text: (*t).to_string(),
                }),
                _ => None,
            };
            let dreapta = linie(id, &linii_dr, &harta_dr);
            // Pe un rand fara linie in dreapta, acolo s-ar insera ce se aduce
            // din stanga: inaintea urmatoarei linii din dreapta a blocului.
            let urmatoarea_dr = perechi[k..]
                .iter()
                .find_map(|(_, d, _)| *d)
                .unwrap_or(linii_dr.len());
            randuri.push(RandAliniat {
                primul_din_bloc: k == 0,
                ultimul_din_bloc: k + 1 == perechi.len(),
                stanga: linie(is, &linii_st, &harta_st),
                poz_dreapta: dreapta
                    .as_ref()
                    .map_or(inceput_dr + urmatoarea_dr, |l| l.nr - 1),
                dreapta,
                tip: rd.tip,
                bloc: Some(indice),
                linie_in_bloc_stanga: is,
                linie_in_bloc_dreapta: id,
                ignorata,
            });
        }
        if let Some(h) = &harta_st {
            ultim_st = h.last().map_or(ultim_st, |&l| l + 1);
        }
        if let Some(h) = &harta_dr {
            ultim_dr = h.last().map_or(ultim_dr, |&l| l + 1);
        }
    }
    completeaza(
        &mut randuri,
        (ultim_st, original_st.len()),
        (ultim_dr, original_dr.len()),
    );
    Aliniere {
        randuri,
        diff,
        blocuri_nealiniate: nealiniate,
        segmente_pozitionale,
    }
}

/// Unde ajunge fiecare bloc care difera, pe alinierea `a`.
///
/// Editorul (ce se aprinde, eticheta, randul-fantoma) si [`aplica_blocuri`]
/// (ce se scrie) intreaba aici, cu aceleasi blocuri ale tintei: daca ar lua
/// doua pareri, „Aplica" ar scrie in alt loc decat arata ecranul.
pub fn plasari_aliniere(
    a: &Aliniere,
    text_dreapta: &str,
    uneste_cursor: bool,
) -> Vec<crate::plasare::Plasare> {
    plasari_aliniere_cu_raport(a, text_dreapta, uneste_cursor).0
}

/// Ca [`plasari_aliniere`], plus ce a costat si ce a fost degradat la plafonul
/// de cost (`plasare::RaportPlasare`).
pub fn plasari_aliniere_cu_raport(
    a: &Aliniere,
    text_dreapta: &str,
    uneste_cursor: bool,
) -> (Vec<crate::plasare::Plasare>, crate::plasare::RaportPlasare) {
    let blocuri_tinta = split_statements(text_dreapta, uneste_cursor);
    crate::plasare::plasari_cu_raport(&a.diff, &blocuri_tinta)
}

/// Textul din dreapta, cu blocurile `blocuri` (indexuri in diff, ca in
/// editor) aduse din stanga **acolo unde spune plasarea**: un bloc fara
/// pereche care seamana cu un bloc din dreapta il inlocuieste; unul care se
/// insereaza intr-un gol isi aduce si restul grupului (un `IF ... BEGIN` cu
/// corpul lui).
///
/// E singura cale prin care editorul, modul ghidat, tragerea si `cli` scriu
/// un bloc.
pub fn aplica_blocuri(
    text_stanga: &str,
    text_dreapta: &str,
    uneste_cursor: bool,
    blocuri: &[usize],
) -> String {
    let a = aliniaza_cu(text_stanga, text_dreapta, uneste_cursor);
    let plasari = plasari_aliniere(&a, text_dreapta, uneste_cursor);
    compune_plasat(&a, text_stanga, text_dreapta, &plasari, |b| {
        blocuri.contains(&b)
    })
}

/// Unde lasa omul un bloc tras, cand nu-l lasa pe locul sugerat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loc {
    /// Inlocuieste blocul din dreapta cu indexul asta din diff.
    Inlocuieste(usize),
    /// Se insereaza inaintea liniei din dreapta cu indexul asta (de la 0;
    /// numarul de linii = la sfarsit).
    Insereaza(usize),
}

/// Ce inseamna „aici" pentru linia `linie` (de la 0) din dreapta: daca e
/// intr-un bloc din dreapta, il inlocuieste; altfel (linie goala, comentariu
/// intre blocuri, dupa ultima linie) se insereaza inaintea ei. Asa traduce
/// `cli` o linie data de om.
pub fn loc_la_linie(a: &Aliniere, linie: usize) -> Loc {
    a.randuri
        .iter()
        // `saturating_add`: `linie` vine de la apelant (`cli` o ia de la om),
        // iar `usize::MAX + 1` dadea panica in debug (auditul 2026-11-23)
        .find(|r| {
            r.dreapta
                .as_ref()
                .is_some_and(|l| l.nr == linie.saturating_add(1))
        })
        .and_then(|r| r.bloc)
        .filter(|&b| a.diff[b].dreapta.is_some())
        .map_or(Loc::Insereaza(linie), Loc::Inlocuieste)
}

/// Liniile din dreapta (de la 0, capat exclus) ale blocului `b` din diff.
fn linii_dreapta_bloc(a: &Aliniere, b: usize) -> Option<(usize, usize)> {
    let rr: Vec<&RandAliniat> = a.randuri.iter().filter(|r| r.bloc == Some(b)).collect();
    let primul = rr.first()?;
    let inceput = rr
        .iter()
        .filter_map(|r| r.dreapta.as_ref().map(|l| l.nr - 1))
        .min()
        .unwrap_or(primul.poz_dreapta);
    let sfarsit = rr
        .iter()
        .filter_map(|r| r.dreapta.as_ref().map(|l| l.nr))
        .max()
        .unwrap_or(inceput);
    Some((inceput, sfarsit))
}

/// Ca [`aplica_blocuri`], dar blocurile merg unde le-a lasat omul (`loc`),
/// nu unde spune plasarea: tragerea pe alt loc decat cel sugerat. Liniile
/// blocurilor vin din stanga, in ordine, ca linii originale.
pub fn aplica_blocuri_la(
    text_stanga: &str,
    text_dreapta: &str,
    uneste_cursor: bool,
    blocuri: &[usize],
    loc: Loc,
) -> String {
    let a = aliniaza_cu(text_stanga, text_dreapta, uneste_cursor);
    let original_st = linii_originale(text_stanga);
    let original_dr = linii_originale(text_dreapta);
    // un bloc care exista doar in dreapta n-are nimic de adus: tras peste alt
    // bloc, l-ar sterge (A2 din `cercetare/audit-qa-2026-11-22.md`)
    let mut ordonate: Vec<usize> = blocuri
        .iter()
        .copied()
        .filter(|&b| a.diff.get(b).is_some_and(|d| d.stanga.is_some()))
        .collect();
    ordonate.sort_unstable();
    ordonate.dedup();
    if ordonate.is_empty() {
        return text_dreapta.to_string();
    }
    let aduse: Vec<&str> = ordonate
        .iter()
        .flat_map(|&b| a.randuri.iter().filter(move |r| r.bloc == Some(b)))
        .filter_map(|r| r.stanga.as_ref())
        .map(|l| {
            original_st
                .get(l.nr - 1)
                .copied()
                .unwrap_or(l.text.as_str())
        })
        .collect();
    let n = original_dr.len();
    let (s, e) = match loc {
        Loc::Inlocuieste(t) => linii_dreapta_bloc(&a, t).unwrap_or((n, n)),
        Loc::Insereaza(l) => (l.min(n), l.min(n)),
    };
    let (s, e) = (s.min(n), e.min(n).max(s.min(n)));
    let mut iesire: Vec<&str> = Vec::with_capacity(n + aduse.len());
    iesire.extend_from_slice(&original_dr[..s]);
    iesire.extend(aduse);
    iesire.extend_from_slice(&original_dr[e..]);
    let mut rez = iesire.join("\n");
    // `\n`-ul final al dreptei ramane: ce nu e aplicat e octet cu octet ce
    // era (COSMETIC 12 din auditul 2026-11-23)
    if text_dreapta.ends_with('\n') && !rez.ends_with('\n') {
        rez.push('\n');
    }
    rez
}

/// Textul din dreapta, cu blocurile pentru care `aplica` spune `true` luate
/// din stanga, fiecare la **pozitia lui din diff** (fara plasare).
///
/// Se lucreaza pe **liniile originale**, prin numerele lor: ce nu e aplicat
/// ramane octet cu octet ce era in dreapta (cu niciun bloc aplicat, rezultatul
/// e exact `text_dreapta`), iar un bloc aplicat aduce liniile originale din
/// stanga. Textul blocurilor din motor nu se foloseste: nu e garantat subsir
/// al originalului (blocurile unite se lipesc cu un singur `\n`), deci ar
/// pierde sau ar inventa linii.
pub fn compune(
    a: &Aliniere,
    text_stanga: &str,
    text_dreapta: &str,
    aplica: impl Fn(usize) -> bool,
) -> String {
    compune_plasat(a, text_stanga, text_dreapta, &[], aplica)
}

/// Liniile unui text, cum le numara `lines()`, dar cu `\r`-ul pastrat: un
/// `\r\n` dintr-un sir pe mai multe linii e data, si `pregateste_afisare` il
/// lasa neatins. `lines()` l-ar fi taiat, iar lipirea cu `\n` l-ar fi facut LF
/// la orice aplicare (A3 din `cercetare/audit-qa-2026-11-22.md`).
fn linii_originale(t: &str) -> Vec<&str> {
    let mut v: Vec<&str> = t.split('\n').collect();
    if t.is_empty() || t.ends_with('\n') {
        v.pop();
    }
    v
}

/// Ca [`compune`], dar blocurile din stanga fara pereche merg unde spune
/// `plasari` (vezi [`aplica_blocuri`]).
pub fn compune_plasat(
    a: &Aliniere,
    text_stanga: &str,
    text_dreapta: &str,
    plasari: &[crate::plasare::Plasare],
    aplica: impl Fn(usize) -> bool,
) -> String {
    let original_st = linii_originale(text_stanga);
    let original_dr = linii_originale(text_dreapta);

    // intervalul de randuri al fiecarui bloc din diff
    let mut interval: Vec<Option<(usize, usize)>> = vec![None; a.diff.len()];
    for (k, r) in a.randuri.iter().enumerate() {
        if let Some(b) = r.bloc {
            let e = interval[b].get_or_insert((k, k + 1));
            e.1 = k + 1;
        }
    }
    // liniile din dreapta (de la 0, capat exclus) ale unui bloc
    let linii_dreapta = |b: usize| -> Option<(usize, usize)> {
        let (i, j) = interval[b]?;
        let rr = &a.randuri[i..j];
        let inceput = rr
            .iter()
            .filter_map(|r| r.dreapta.as_ref().map(|l| l.nr - 1))
            .min()
            .unwrap_or(a.randuri[i].poz_dreapta);
        let sfarsit = rr
            .iter()
            .filter_map(|r| r.dreapta.as_ref().map(|l| l.nr))
            .max()
            .unwrap_or(inceput);
        Some((inceput, sfarsit))
    };
    // blocul din diff al fiecarui bloc din dreapta, in ordine (ce numara
    // `plasare` prin `bloc_tinta`)
    let al_ordinalului: Vec<usize> = a
        .diff
        .iter()
        .enumerate()
        .filter(|(_, d)| d.dreapta.is_some())
        .map(|(b, _)| b)
        .collect();

    // ce se aplica: blocurile cerute, plus restul grupului la inserarea in gol
    let mut aplicate: Vec<usize> = (0..a.diff.len())
        .filter(|&b| a.diff[b].tip != TipDiferenta::Egal && aplica(b))
        .collect();
    let din_grup: Vec<usize> = plasari
        .iter()
        .filter(|p| !p.inlocuieste && aplicate.contains(&p.rand))
        .flat_map(|p| p.randuri.iter().copied())
        .filter(|&b| a.diff.get(b).is_some_and(|d| d.tip == TipDiferenta::Sters))
        .collect();
    aplicate.extend(din_grup);
    aplicate.sort_unstable();
    aplicate.dedup();

    // fiecare bloc aplicat devine „inlocuieste liniile [s, e) din dreapta cu
    // liniile lui din stanga" (s == e: inserare)
    let mut operatii: Vec<(usize, usize, usize)> = Vec::new(); // (s, e, bloc)
                                                               // un bloc venit doar ca parte a grupului altuia se insereaza cu grupul,
                                                               // nu unde l-ar pune propria lui plasare: corpul unui `IF` adus in gol
                                                               // pleca altundeva decat arata ecranul (MEDIU 5 din auditul 2026-11-23)
    let doar_din_grup: Vec<usize> = aplicate
        .iter()
        .copied()
        .filter(|&b| a.diff[b].tip != TipDiferenta::Egal && !aplica(b))
        .collect();
    // membrul unui grup se insereaza la locul primului bloc al grupului, in
    // ordine, nu la propria lui pozitie din diff (care poate fi dupa un bloc
    // egal din tinta)
    let liderul: std::collections::HashMap<usize, usize> = plasari
        .iter()
        .filter(|p| !p.inlocuieste)
        .flat_map(|p| p.randuri.iter().map(move |&m| (m, p.rand)))
        .filter(|&(m, l)| m != l)
        .collect();
    for &b in &aplicate {
        let Some(mut loc) = linii_dreapta(b) else {
            continue;
        };
        if let Some(l) = liderul.get(&b).filter(|l| aplicate.contains(l)) {
            if let Some((s, _)) = linii_dreapta(*l) {
                loc = (s, s);
            }
        }
        if a.diff[b].tip == TipDiferenta::Sters && !doar_din_grup.contains(&b) {
            if let Some(p) = plasari.iter().find(|p| p.rand == b && p.inlocuieste) {
                if let Some(l) = al_ordinalului
                    .get(p.bloc_tinta)
                    .and_then(|&t| linii_dreapta(t))
                {
                    loc = l;
                }
            }
        }
        operatii.push((loc.0, loc.1, b));
    }
    // ordinea pe pozitie; la egalitate, ordinea blocurilor
    operatii.sort_by_key(|&(s, _, b)| (s, b));

    let mut iesire: Vec<&str> = Vec::with_capacity(original_dr.len());
    let mut urmatoarea_dr = 0usize;
    for (s, e, b) in operatii {
        let s = s.max(urmatoarea_dr);
        iesire.extend_from_slice(
            &original_dr[urmatoarea_dr.min(original_dr.len())..s.min(original_dr.len())],
        );
        if let Some((i, j)) = interval[b] {
            for l in a.randuri[i..j].iter().filter_map(|r| r.stanga.as_ref()) {
                iesire.push(
                    original_st
                        .get(l.nr - 1)
                        .copied()
                        .unwrap_or(l.text.as_str()),
                );
            }
        }
        urmatoarea_dr = e.max(s);
    }
    if urmatoarea_dr < original_dr.len() {
        iesire.extend_from_slice(&original_dr[urmatoarea_dr..]);
    }
    let mut rez = iesire.join("\n");
    // `\n`-ul final al dreptei ramane: ce nu e aplicat e octet cu octet ce
    // era (COSMETIC 12 din auditul 2026-11-23)
    if text_dreapta.ends_with('\n') && !rez.ends_with('\n') {
        rez.push('\n');
    }
    rez
}

#[cfg(test)]
mod teste {
    use super::*;

    fn bloc_cu_tip(a: &Aliniere, tip: TipDiferenta) -> usize {
        a.diferente()
            .into_iter()
            .find(|&d| a.diff[d].tip == tip)
            .unwrap_or_else(|| panic!("niciun bloc {tip:?}: {:?}", a.diff))
    }

    /// Blocul mutat in tinta (aici, UPDATE-ul urcat inaintea SELECT-urilor)
    /// iese din diff fara pereche. Plasarea il recunoaste in tinta, deci
    /// „Aplica" trebuie sa-l **inlocuiasca** acolo, nu sa-l mai insereze o
    /// data la pozitia lui din martor. Ecranul anunta exact inlocuirea.
    #[test]
    fn blocul_fara_pereche_inlocuieste_unde_arata_plasarea() {
        let st = "CREATE PROCEDURE dbo.p AS\n\
                  SELECT a FROM dbo.T1;\n\
                  SELECT b FROM dbo.T2;\n\
                  UPDATE dbo.T3 SET x = 1, y = 2, z = 4 WHERE id = 5 AND activ = 1;";
        let dr = "CREATE PROCEDURE dbo.p AS\n\
                  UPDATE dbo.T3 SET x = 1, y = 3, z = 4 WHERE id = 5 AND activ = 1;\n\
                  SELECT a FROM dbo.T1;\n\
                  SELECT b FROM dbo.T2;";
        let a = aliniaza(st, dr);
        let plasari = plasari_aliniere(&a, dr, false);
        let p = plasari
            .iter()
            .find(|p| a.diff[p.rand].tip == TipDiferenta::Sters)
            .unwrap_or_else(|| panic!("niciun bloc fara pereche: {:?}", a.diff));
        assert!(
            p.inlocuieste,
            "plasarea trebuia sa gaseasca UPDATE-ul: {p:?}"
        );

        let rez = aplica_blocuri(st, dr, false, &[p.rand]);
        assert_eq!(rez.matches("UPDATE").count(), 1, "dublat:\n{rez}");
        assert!(rez.contains("y = 2") && !rez.contains("y = 3"), "{rez}");
        // pe locul UPDATE-ului din tinta, nu la sfarsit
        assert!(
            rez.lines().nth(1).is_some_and(|l| l.contains("UPDATE")),
            "{rez}"
        );
        // compunerea fara plasare ar fi dublat blocul: de-aia exista
        let fara = compune(&a, st, dr, |b| b == p.rand);
        assert_eq!(fara.matches("UPDATE").count(), 2, "{fara}");
    }

    /// Blocul tras si lasat in alt loc decat cel sugerat: pe un bloc din
    /// dreapta il inlocuieste, intre blocuri se insereaza acolo.
    #[test]
    fn blocul_lasat_oriunde_inlocuieste_sau_se_insereaza_acolo() {
        let st = "CREATE PROC dbo.p AS\nSELECT 1;\nPRINT 'nou';";
        let dr = "CREATE PROC dbo.p AS\nSELECT 1;\n\nSELECT 2;";
        let a = aliniaza(st, dr);
        let nou = a
            .diferente()
            .into_iter()
            .find(|&b| {
                a.diff[b]
                    .stanga
                    .as_deref()
                    .is_some_and(|t| t.contains("PRINT"))
            })
            .expect("blocul PRINT");
        // pe linia goala (index 2): se insereaza inaintea ei
        assert_eq!(loc_la_linie(&a, 2), Loc::Insereaza(2));
        let rez = aplica_blocuri_la(st, dr, false, &[nou], Loc::Insereaza(2));
        assert_eq!(
            rez,
            "CREATE PROC dbo.p AS\nSELECT 1;\nPRINT 'nou';\n\nSELECT 2;"
        );
        // pe SELECT 2 (index 3): il inlocuieste
        let loc = loc_la_linie(&a, 3);
        assert!(matches!(loc, Loc::Inlocuieste(_)), "{loc:?}");
        let rez = aplica_blocuri_la(st, dr, false, &[nou], loc);
        assert_eq!(rez, "CREATE PROC dbo.p AS\nSELECT 1;\n\nPRINT 'nou';");
        // la sfarsit
        let rez = aplica_blocuri_la(st, dr, false, &[nou], Loc::Insereaza(99));
        assert!(rez.ends_with("SELECT 2;\nPRINT 'nou';"), "{rez}");
        // fara blocuri: dreapta neatinsa
        assert_eq!(aplica_blocuri_la(st, dr, false, &[], Loc::Insereaza(1)), dr);
    }

    #[test]
    fn fara_nimic_aplicat_ramane_si_randul_final() {
        let st = "CREATE PROC p AS\nSELECT 1\n";
        let dr = "CREATE PROC p AS\nSELECT 2\n";
        assert_eq!(aplica_blocuri(st, dr, false, &[]), dr);
        assert_eq!(aplica_blocuri_la(st, dr, false, &[], Loc::Insereaza(0)), dr);
    }

    /// A3 din `cercetare/audit-qa-2026-11-22.md`: un `\r\n` dintr-un sir
    /// ramane `\r\n` dupa aplicare, si in blocul neatins, si in cel adus.
    #[test]
    fn aplicarea_pastreaza_crlf_din_siruri() {
        let st = crate::pregateste_afisare("CREATE PROC p AS\r\nSELECT 'a\r\nb'\r\nSELECT 1");
        let dr = crate::pregateste_afisare("CREATE PROC p AS\r\nSELECT 'a\r\nb'\r\nSELECT 2");
        assert!(st.contains("'a\r\nb'"), "{st:?}");
        assert_eq!(aplica_blocuri(&st, &dr, false, &[]), dr);
        let a = aliniaza(&st, &dr);
        let rez = aplica_blocuri(&st, &dr, false, &a.diferente());
        assert_eq!(rez, st);
        let ultim = a.diff.len() - 1;
        let rez = aplica_blocuri_la(&st, &dr, false, &[ultim], Loc::Insereaza(0));
        assert!(rez.contains("'a\r\nb'"), "{rez:?}");
    }

    /// A2 din `cercetare/audit-qa-2026-11-22.md`: un bloc care exista doar in
    /// dreapta, tras peste alt bloc, nu-l sterge.
    #[test]
    fn un_bloc_doar_in_dreapta_tras_peste_altul_nu_sterge_nimic() {
        let st = "CREATE PROC dbo.p AS\nSELECT 1;\nSELECT 2;\nSELECT 3;";
        let dr = "CREATE PROC dbo.p AS\nSELECT 1;\nSELECT 2;\nDELETE FROM t;\nSELECT 3;";
        let a = aliniaza(st, dr);
        let doar_dreapta = (0..a.diff.len())
            .find(|&b| a.diff[b].tip == TipDiferenta::Adaugat)
            .expect("DELETE doar in dreapta");
        let loc = loc_la_linie(&a, 4);
        assert!(matches!(loc, Loc::Inlocuieste(_)), "{loc:?}");
        assert_eq!(aplica_blocuri_la(st, dr, false, &[doar_dreapta], loc), dr);
    }

    #[test]
    fn fara_nimic_aplicat_ramane_exact_dreapta() {
        let st = "CREATE PROC p AS\n\n-- antet\nSELECT 1\n\nUPDATE t SET a = 1";
        let dr = "CREATE PROC p AS\n\n\n-- alt antet\nSELECT 2\nUPDATE t SET a = 1\n\n-- final";
        let a = aliniaza(st, dr);
        assert_eq!(compune(&a, st, dr, |_| false), dr);
    }

    #[test]
    fn un_bloc_modificat_aplicat_inlocuieste_doar_blocul() {
        let st = "SELECT 1\n\nUPDATE t\nSET a = 1\n\nDELETE x";
        let dr = "SELECT 1\n\nUPDATE t\nSET a = 2,\n    b = 3\n\nDELETE x";
        let a = aliniaza(st, dr);
        let b = bloc_cu_tip(&a, TipDiferenta::Modificat);
        assert_eq!(
            compune(&a, st, dr, |x| x == b),
            "SELECT 1\n\nUPDATE t\nSET a = 1\n\nDELETE x"
        );
    }

    #[test]
    fn un_bloc_doar_in_stanga_se_insereaza_la_locul_lui() {
        let st = "SELECT 1\n\nINSERT INTO t VALUES (1)\n\nDELETE x";
        let dr = "SELECT 1\n\nDELETE x";
        let a = aliniaza(st, dr);
        let b = bloc_cu_tip(&a, TipDiferenta::Sters);
        let rez = compune(&a, st, dr, |x| x == b);
        assert!(rez.contains("INSERT INTO t VALUES (1)"), "{rez}");
        let (p_select, p_insert, p_delete) = (
            rez.find("SELECT 1").unwrap(),
            rez.find("INSERT").unwrap(),
            rez.find("DELETE").unwrap(),
        );
        assert!(p_select < p_insert && p_insert < p_delete, "ordinea: {rez}");
    }

    #[test]
    fn un_bloc_doar_in_dreapta_aplicat_dispare() {
        let st = "SELECT 1\n\nDELETE x";
        let dr = "SELECT 1\n\nPRINT 'doar aici'\n\nDELETE x";
        let a = aliniaza(st, dr);
        let b = bloc_cu_tip(&a, TipDiferenta::Adaugat);
        let rez = compune(&a, st, dr, |x| x == b);
        assert!(!rez.contains("doar aici"), "{rez}");
        assert!(rez.contains("SELECT 1") && rez.contains("DELETE x"));
    }

    /// Liniile dintre blocuri (goale, comentarii) nu sunt in niciun bloc, deci
    /// raman cum sunt in dreapta; codul, in schimb, e cel din stanga, in
    /// ordinea din stanga. „Aplica toate" din editor pune textul exact al
    /// stangii, nu compunerea asta.
    #[test]
    fn toate_aplicate_dau_codul_din_stanga() {
        let st = "SELECT a\nFROM t\n\nUPDATE t SET x = 1\n\nEXEC p";
        let dr = "SELECT b\nFROM t\n\nPRINT 1\n\nEXEC p";
        let a = aliniaza(st, dr);
        let rez = compune(&a, st, dr, |_| true);
        let cod = |t: &str| {
            t.lines()
                .filter(|l| !l.trim().is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        };
        assert_eq!(cod(&rez), cod(st), "{rez}");
        assert!(diff_aliniat(st, &rez)
            .iter()
            .all(|r| r.tip == TipDiferenta::Egal));
    }

    #[test]
    fn numerele_de_linie_sunt_cele_reale_cu_umplutura() {
        let a = aliniaza(
            "SELECT 1\n\nUPDATE t\nSET a = 1",
            "SELECT 1\n\nUPDATE t\nSET a = 2,\n    b = 3",
        );
        let b = a
            .randuri
            .iter()
            .position(|r| r.tip == TipDiferenta::Modificat)
            .unwrap();
        let bloc: Vec<&RandAliniat> = a.randuri[b..]
            .iter()
            .take_while(|r| r.tip != TipDiferenta::Egal)
            .collect();
        assert_eq!(bloc.len(), 3);
        assert_eq!(bloc[0].stanga.as_ref().map(|l| l.nr), Some(3));
        assert_eq!(bloc[2].stanga, None, "umplutura in stanga");
        assert_eq!(bloc[2].dreapta.as_ref().map(|l| l.nr), Some(5));
        assert_eq!(bloc[2].linie_in_bloc_dreapta, Some(2));
        assert_eq!(bloc[2].linie_in_bloc_stanga, None);
    }
}

#[cfg(test)]
mod teste_linii {
    use super::*;

    /// Cazul martor / upgrade al lui Mihai (2026-11-19): antetul a primit doi
    /// parametri, si pana la etapa 20 `AS` ajungea langa `@UserId INT,`.
    #[test]
    fn liniile_se_potrivesc_intre_ele_nu_pe_pozitie() {
        let martor = "CREATE PROCEDURE p\n @DocumentId int,\n @SiteId int\nAS";
        let modificata =
            "CREATE PROCEDURE p(\n @DocumentId INT,\n @SiteId INT,\n @UserId INT,\n @Context VARCHAR(255) = '')\nAS\nBEGIN";
        let a = aliniaza(martor, modificata);
        let pereche = |text: &str| -> Option<String> {
            a.randuri
                .iter()
                .find(|r| r.stanga.as_ref().is_some_and(|l| l.text == text))
                .map(|r| {
                    r.dreapta
                        .as_ref()
                        .map_or("<umplutura>".to_string(), |l| l.text.clone())
                })
        };
        assert_eq!(pereche("AS").as_deref(), Some("AS"), "{:?}", a.randuri);
        assert_eq!(pereche(" @SiteId int").as_deref(), Some(" @SiteId INT,"));
        // liniile in plus din dreapta nu fura perechea nimanui
        let noi: Vec<&str> = a
            .randuri
            .iter()
            .filter(|r| r.stanga.is_none() && r.bloc.is_some())
            .filter_map(|r| r.dreapta.as_ref().map(|l| l.text.as_str()))
            .collect();
        assert!(noi.contains(&" @UserId INT,"), "{noi:?}");
        assert!(noi.contains(&"BEGIN"), "{noi:?}");
    }

    /// Comentariile nu trag alinierea dupa ele: raman pe partea lor, iar
    /// codul din jur se potriveste cu codul.
    #[test]
    fn comentariile_nu_intra_in_imperechere() {
        let st = "SELECT a\n-- un comentariu lung, numai in martor\nFROM t";
        let dr = "SELECT a\nFROM t2";
        let a = aliniaza(st, dr);
        let per = |text: &str| {
            a.randuri
                .iter()
                .find(|r| r.stanga.as_ref().is_some_and(|l| l.text == text))
                .and_then(|r| r.dreapta.as_ref().map(|l| l.text.clone()))
        };
        assert_eq!(per("SELECT a").as_deref(), Some("SELECT a"));
        assert_eq!(per("FROM t").as_deref(), Some("FROM t2"));
        assert_eq!(
            per("-- un comentariu lung, numai in martor"),
            None,
            "comentariul n-are pereche, dar isi pastreaza locul"
        );
    }

    /// Liniile fara nicio legatura nu se imperecheaza fortat.
    #[test]
    fn liniile_prea_diferite_raman_separate() {
        let a = aliniaza(
            "SELECT a\nWHERE x = 1",
            "SELECT a\nORDER BY ceva_total_diferit",
        );
        let rand = a
            .randuri
            .iter()
            .find(|r| r.stanga.as_ref().is_some_and(|l| l.text == "WHERE x = 1"))
            .expect("linia din stanga");
        assert!(rand.dreapta.is_none(), "{rand:?}");
    }
}

#[cfg(test)]
mod teste_ignorate {
    use super::*;

    /// Chenarul blocului se deseneaza din ce spune motorul, nu dintr-o
    /// socoteala refacuta in fereastra.
    #[test]
    fn motorul_spune_unde_incepe_si_unde_se_termina_blocul() {
        let a = aliniaza(
            "SELECT 1\n\nUPDATE t SET a = 1",
            "SELECT 1\n\nUPDATE t SET a = 2",
        );
        let blocuri: Vec<usize> = a.randuri.iter().filter_map(|r| r.bloc).collect();
        assert!(!blocuri.is_empty());
        for b in 0..=blocuri.iter().copied().max().unwrap() {
            let ale_lui: Vec<&RandAliniat> =
                a.randuri.iter().filter(|r| r.bloc == Some(b)).collect();
            assert!(ale_lui[0].primul_din_bloc, "blocul {b} nu are inceput");
            assert!(
                ale_lui[ale_lui.len() - 1].ultimul_din_bloc,
                "blocul {b} nu are sfarsit"
            );
            assert_eq!(
                ale_lui.iter().filter(|r| r.primul_din_bloc).count(),
                1,
                "blocul {b} incepe de doua ori"
            );
        }
        // randurile dintre blocuri nu sunt ale niciunui bloc
        assert!(a
            .randuri
            .iter()
            .filter(|r| r.bloc.is_none())
            .all(|r| !r.primul_din_bloc && !r.ultimul_din_bloc));
    }

    /// Liniile ignorate se vad ca atare, ca interfata sa le poata plia sau
    /// desena altfel — nu ca diferente.
    #[test]
    fn liniile_de_comentariu_din_bloc_sunt_marcate_ignorate() {
        let st = "SELECT a\n-- doar in martor\nFROM t";
        let dr = "SELECT a\nFROM t2";
        let a = aliniaza(st, dr);
        let com = a
            .randuri
            .iter()
            .find(|r| {
                r.stanga
                    .as_ref()
                    .is_some_and(|l| l.text == "-- doar in martor")
            })
            .expect("randul comentariului");
        assert!(com.ignorata, "{com:?}");
        let cod = a
            .randuri
            .iter()
            .find(|r| r.stanga.as_ref().is_some_and(|l| l.text == "FROM t"))
            .expect("randul de cod");
        assert!(!cod.ignorata, "{cod:?}");
    }
}
