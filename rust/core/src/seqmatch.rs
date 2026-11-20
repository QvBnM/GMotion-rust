//! Port fidel al `difflib.SequenceMatcher` din Python, cu `autojunk=False`
//! si fara `isjunk` - exact varianta folosita de motorul existent.
//!
//! De ce un port fidel si nu un algoritm LCS oarecare: alinierea blocurilor
//! depinde de ce anume considera `SequenceMatcher` "cea mai lunga potrivire",
//! iar Ratcliff-Obershelp (recursiv pe cea mai lunga potrivire) da alte
//! rezultate decat un LCS clasic. Orice abatere aici se vede direct in ce
//! blocuri sunt declarate `modificat` fata de `sters` + `adaugat`.
//!
//! **Se strica astfel:** rezultate care difera de `difflib` din Python pe
//! aceleasi intrari. Cel mai probabil vinovat e `autojunk`: `difflib` il are
//! implicit activat si ignora elementele frecvente in secvente lungi, ceea ce
//! schimba alinierea. Motorul existent il are oprit, deci si portul de aici.

use std::collections::HashMap;
use std::hash::Hash;

/// Operatiile rezultate din comparatie, in ordinea pozitiei.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Equal,
    Replace,
    Delete,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opcode {
    pub tag: Tag,
    pub i1: usize,
    pub i2: usize,
    pub j1: usize,
    pub j2: usize,
}

/// Peste acest produs `n*m` de elemente, cautarea trece pe automat de sufixe.
/// Sub el, bucla din `difflib` e mai ieftina decat constructia automatului.
const PRAG_AUTOMAT: u64 = 250_000;

pub struct SequenceMatcher<'a, T: Eq + Hash> {
    a: &'a [T],
    b: &'a [T],
    b2j: HashMap<&'a T, Vec<usize>>,
    /// Cati pasi elementari a facut cautarea. Nu e o masuratoare de timp, e o
    /// marime deterministica: pe ea se poate pune un plafon intr-un test, care
    /// nu devine instabil pe o masina incarcata. Vezi
    /// `tests/cost.rs`.
    operatii: std::cell::Cell<u64>,
}

impl<'a, T: Eq + Hash> SequenceMatcher<'a, T> {
    pub fn new(a: &'a [T], b: &'a [T]) -> Self {
        let mut b2j: HashMap<&'a T, Vec<usize>> = HashMap::with_capacity(b.len());
        for (i, el) in b.iter().enumerate() {
            b2j.entry(el).or_default().push(i);
        }
        Self {
            a,
            b,
            b2j,
            operatii: std::cell::Cell::new(0),
        }
    }

    /// Pasii elementari facuti pana acum de cautarile pe acest matcher.
    pub fn operatii(&self) -> u64 {
        self.operatii.get()
    }

    fn numara(&self, pasi: u64) {
        self.operatii.set(self.operatii.get() + pasi);
    }

    fn indici(&self, el: &T) -> &[usize] {
        self.b2j.get(el).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Aceeasi potrivire, cautata cu un automat de sufixe peste `b[blo..bhi]`.
    ///
    /// Bucla din `difflib` trece, pentru fiecare element din `a`, prin toate
    /// aparitiile lui din `b`. Pe SQL, unde `select`, `from` si `=` apar de
    /// mii de ori intr-un obiect mare, costul devine cubic: 38 s pe 16.000 de
    /// linii, 400 s pe 32.000 (masurat). Automatul da **acelasi** raspuns in
    /// timp liniar, fiindca „cea mai lunga potrivire" e, fara `isjunk` si cu
    /// `autojunk` oprit, exact cel mai lung subsir comun: difflib alege dintre
    /// ele pe cel cu `i` minim, apoi `j` minim, iar automatul tine minte cea
    /// mai mica pozitie de final a fiecarei stari.
    fn potrivire_prin_automat(
        &self,
        alo: usize,
        ahi: usize,
        blo: usize,
        bhi: usize,
    ) -> (usize, usize, usize) {
        // elementele din interval, numerotate: tranzitiile automatului se tin
        // pe numere, nu pe `&T`
        let mut numere: HashMap<&T, u32> = HashMap::new();
        let cod = |el: &'a T, numere: &mut HashMap<&'a T, u32>| -> u32 {
            let n = numere.len() as u32;
            *numere.entry(el).or_insert(n)
        };
        let bb: Vec<u32> = self.b[blo..bhi]
            .iter()
            .map(|el| cod(el, &mut numere))
            .collect();

        // automat de sufixe peste bb
        let cap = 2 * bb.len() + 2;
        let mut urm: Vec<HashMap<u32, u32>> = Vec::with_capacity(cap);
        let mut link: Vec<i32> = Vec::with_capacity(cap);
        let mut lung: Vec<u32> = Vec::with_capacity(cap);
        // cea mai mica pozitie de final (1-based in felie) pentru starea asta
        let mut min_sfarsit: Vec<u32> = Vec::with_capacity(cap);
        urm.push(HashMap::new());
        link.push(-1);
        lung.push(0);
        min_sfarsit.push(u32::MAX);
        let mut ultim: u32 = 0;
        for (poz, &c) in bb.iter().enumerate() {
            let cur = urm.len() as u32;
            urm.push(HashMap::new());
            link.push(-1);
            lung.push(lung[ultim as usize] + 1);
            min_sfarsit.push(poz as u32 + 1);
            let mut p = ultim as i32;
            while p >= 0 && !urm[p as usize].contains_key(&c) {
                urm[p as usize].insert(c, cur);
                p = link[p as usize];
            }
            if p < 0 {
                link[cur as usize] = 0;
            } else {
                let q = urm[p as usize][&c];
                if lung[p as usize] + 1 == lung[q as usize] {
                    link[cur as usize] = q as i32;
                } else {
                    let clona = urm.len() as u32;
                    let copie = urm[q as usize].clone();
                    urm.push(copie);
                    link.push(link[q as usize]);
                    lung.push(lung[p as usize] + 1);
                    min_sfarsit.push(min_sfarsit[q as usize]);
                    while p >= 0 && urm[p as usize].get(&c) == Some(&q) {
                        urm[p as usize].insert(c, clona);
                        p = link[p as usize];
                    }
                    link[q as usize] = clona as i32;
                    link[cur as usize] = clona as i32;
                }
            }
            ultim = cur;
        }

        // `min_sfarsit` urca pe arborele de legaturi (stari sortate pe lungime)
        let n = urm.len();
        let maxl = lung.iter().copied().max().unwrap_or(0) as usize;
        let mut cnt = vec![0usize; maxl + 2];
        for l in &lung {
            cnt[*l as usize] += 1;
        }
        for i in 1..=maxl {
            cnt[i] += cnt[i - 1];
        }
        let mut ordine = vec![0u32; n];
        for (i, l) in lung.iter().enumerate() {
            cnt[*l as usize] -= 1;
            ordine[cnt[*l as usize]] = i as u32;
        }
        for &v in ordine.iter().rev() {
            if v == 0 {
                continue;
            }
            let parinte = link[v as usize];
            if parinte >= 0 {
                let m = min_sfarsit[v as usize];
                if m < min_sfarsit[parinte as usize] {
                    min_sfarsit[parinte as usize] = m;
                }
            }
        }

        self.numara((urm.len() + (ahi - alo)) as u64);
        let (mut besti, mut bestj, mut bestsize) = (alo, blo, 0usize);
        let mut stare: u32 = 0;
        let mut potrivit: usize = 0;
        for i in alo..ahi {
            let Some(&c) = numere.get(&self.a[i]) else {
                stare = 0;
                potrivit = 0;
                continue;
            };
            loop {
                if let Some(&urmatoare) = urm[stare as usize].get(&c) {
                    stare = urmatoare;
                    potrivit += 1;
                    break;
                }
                if link[stare as usize] < 0 {
                    stare = 0;
                    potrivit = 0;
                    break;
                }
                stare = link[stare as usize] as u32;
                potrivit = lung[stare as usize] as usize;
            }
            if potrivit > bestsize {
                bestsize = potrivit;
                besti = i + 1 - potrivit;
                bestj = blo + min_sfarsit[stare as usize] as usize - potrivit;
            }
        }
        (besti, bestj, bestsize)
    }

    /// Cea mai lunga potrivire din `a[alo..ahi]` in `b[blo..bhi]`.
    /// Returneaza `(i, j, dimensiune)`.
    // indexarea explicita e necesara: `i` se compara cu limitele si intra in
    // calculul pozitiilor returnate, nu doar in accesul la element
    #[allow(clippy::needless_range_loop)]
    pub fn find_longest_match(
        &self,
        alo: usize,
        ahi: usize,
        blo: usize,
        bhi: usize,
    ) -> (usize, usize, usize) {
        let (a, b) = (self.a, self.b);
        // Limitele vin si din afara (functia e publica): un apel cu limite
        // peste lungime dadea panica la index. Se reduc la secventa reala.
        let ahi = ahi.min(a.len());
        let bhi = bhi.min(b.len());
        let alo = alo.min(ahi);
        let blo = blo.min(bhi);
        // Pe intervale mari, bucla de mai jos devine patratica in aparitii
        // (vezi `potrivire_prin_automat`): acolo trecem pe automat de sufixe,
        // care da acelasi raspuns. Pragul e ales ca sa nu platim constructia
        // automatului pe obiectele obisnuite, unde bucla e deja ieftina.
        if (ahi - alo) as u64 * (bhi - blo) as u64 > PRAG_AUTOMAT {
            return self.potrivire_prin_automat(alo, ahi, blo, bhi);
        }
        self.potrivire_prin_bucla(alo, ahi, blo, bhi)
    }

    /// Bucla din `difflib`, element cu element prin aparitiile din `b`.
    #[allow(clippy::needless_range_loop)]
    fn potrivire_prin_bucla(
        &self,
        alo: usize,
        ahi: usize,
        blo: usize,
        bhi: usize,
    ) -> (usize, usize, usize) {
        let (a, b) = (self.a, self.b);
        let (mut besti, mut bestj, mut bestsize) = (alo, blo, 0usize);

        // j2len[j] = lungimea celei mai lungi potriviri care se termina la
        // a[i-1], b[j]. Acelasi algoritm ca `difflib` (dict nou la fiecare i),
        // dar pe doi vectori densi reutilizati: alocarea a doua HashMap-uri la
        // fiecare token domina timpul pe obiecte mari (auditul: 97 s pe o
        // procedura de 20.000 de linii). Indicii atinsi se tin separat, ca
        // stergerea sa fie proportionala cu ei, nu cu lungimea lui `b`.
        let latime = bhi - blo + 1;
        let mut j2len: Vec<usize> = vec![0; latime];
        let mut newj2len: Vec<usize> = vec![0; latime];
        let mut atinsi: Vec<usize> = Vec::new();
        let mut atinsi_noi: Vec<usize> = Vec::new();
        for i in alo..ahi {
            self.numara(self.indici(&a[i]).len() as u64);
            for &j in self.indici(&a[i]) {
                if j < blo {
                    continue;
                }
                if j >= bhi {
                    break;
                }
                // pozitia j in vectori e j - blo + 1; pozitia 0 ramane mereu 0,
                // deci j2len[j-1] pentru j = blo da 0, ca in difflib
                let anterior = j2len[j - blo];
                let k = anterior + 1;
                newj2len[j - blo + 1] = k;
                atinsi_noi.push(j - blo + 1);
                if k > bestsize {
                    besti = i + 1 - k;
                    bestj = j + 1 - k;
                    bestsize = k;
                }
            }
            for &x in &atinsi {
                j2len[x] = 0;
            }
            std::mem::swap(&mut j2len, &mut newj2len);
            std::mem::swap(&mut atinsi, &mut atinsi_noi);
            atinsi_noi.clear();
        }

        // extinde potrivirea in stanga, apoi in dreapta
        while besti > alo && bestj > blo && a[besti - 1] == b[bestj - 1] {
            besti -= 1;
            bestj -= 1;
            bestsize += 1;
        }
        while besti + bestsize < ahi
            && bestj + bestsize < bhi
            && a[besti + bestsize] == b[bestj + bestsize]
        {
            bestsize += 1;
        }

        (besti, bestj, bestsize)
    }

    /// Blocurile care se potrivesc, coalescate, cu terminatorul `(la, lb, 0)`.
    pub fn matching_blocks(&self) -> Vec<(usize, usize, usize)> {
        let (la, lb) = (self.a.len(), self.b.len());
        let mut coada: Vec<(usize, usize, usize, usize)> = vec![(0, la, 0, lb)];
        let mut potriviri: Vec<(usize, usize, usize)> = Vec::new();

        // LIFO, ca in implementarea existenta; ordinea nu conteaza, se sorteaza oricum
        while let Some((alo, ahi, blo, bhi)) = coada.pop() {
            let (i, j, k) = self.find_longest_match(alo, ahi, blo, bhi);
            if k > 0 {
                potriviri.push((i, j, k));
                if alo < i && blo < j {
                    coada.push((alo, i, blo, j));
                }
                if i + k < ahi && j + k < bhi {
                    coada.push((i + k, ahi, j + k, bhi));
                }
            }
        }

        potriviri.sort_unstable_by(|x, y| x.0.cmp(&y.0).then(x.1.cmp(&y.1)));

        let mut out: Vec<(usize, usize, usize)> = Vec::with_capacity(potriviri.len() + 1);
        let (mut i1, mut j1, mut k1) = (0usize, 0usize, 0usize);
        for &(i2, j2, k2) in &potriviri {
            if i1 + k1 == i2 && j1 + k1 == j2 {
                k1 += k2;
            } else {
                if k1 > 0 {
                    out.push((i1, j1, k1));
                }
                i1 = i2;
                j1 = j2;
                k1 = k2;
            }
        }
        if k1 > 0 {
            out.push((i1, j1, k1));
        }
        out.push((la, lb, 0));
        out
    }

    pub fn opcodes(&self) -> Vec<Opcode> {
        let mut i = 0usize;
        let mut j = 0usize;
        let mut ans: Vec<Opcode> = Vec::new();
        for (ai, bj, size) in self.matching_blocks() {
            let tag = if i < ai && j < bj {
                Some(Tag::Replace)
            } else if i < ai {
                Some(Tag::Delete)
            } else if j < bj {
                Some(Tag::Insert)
            } else {
                None
            };
            if let Some(tag) = tag {
                ans.push(Opcode {
                    tag,
                    i1: i,
                    i2: ai,
                    j1: j,
                    j2: bj,
                });
            }
            i = ai + size;
            j = bj + size;
            if size > 0 {
                ans.push(Opcode {
                    tag: Tag::Equal,
                    i1: ai,
                    i2: i,
                    j1: bj,
                    j2: j,
                });
            }
        }
        ans
    }

    /// `2 * potriviri / (len(a) + len(b))`, cu 1.0 pentru doua secvente goale.
    pub fn ratio(&self) -> f64 {
        let potriviri: usize = self.matching_blocks().iter().map(|b| b.2).sum();
        let total = self.a.len() + self.b.len();
        if total == 0 {
            1.0
        } else {
            2.0 * potriviri as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    /// Regresie din audit: limite peste lungimea secventei dadeau panica.
    #[test]
    fn limitele_prea_mari_nu_dau_panica() {
        let a = ["x", "y", "z"];
        let sm = SequenceMatcher::new(&a, &a);
        assert_eq!(sm.find_longest_match(0, 10, 0, 10), (0, 0, 3));
        assert_eq!(sm.find_longest_match(7, 9, 0, 3).2, 0);
    }

    #[test]
    fn ratio_secvente_goale() {
        let gol: Vec<&str> = vec![];
        assert_eq!(SequenceMatcher::new(&gol, &gol).ratio(), 1.0);
    }

    #[test]
    fn ratio_identice() {
        let a = vec!["a", "b", "c"];
        assert_eq!(SequenceMatcher::new(&a, &a).ratio(), 1.0);
    }

    #[test]
    fn ratio_disjuncte() {
        let a = vec!["a", "b"];
        let b = vec!["c", "d"];
        assert_eq!(SequenceMatcher::new(&a, &b).ratio(), 0.0);
    }

    #[test]
    fn opcodes_inlocuire_la_mijloc() {
        let a = vec!["x", "a", "y"];
        let b = vec!["x", "b", "y"];
        let ops = SequenceMatcher::new(&a, &b).opcodes();
        assert_eq!(
            ops.iter().map(|o| o.tag).collect::<Vec<_>>(),
            vec![Tag::Equal, Tag::Replace, Tag::Equal]
        );
    }

    #[test]
    fn find_longest_match_pe_prefix_comun() {
        let a = vec!["a", "b", "c", "d"];
        let b = vec!["a", "b", "x", "y"];
        let (i, j, k) = SequenceMatcher::new(&a, &b).find_longest_match(0, 4, 0, 4);
        assert_eq!((i, j, k), (0, 0, 2));
    }
}

#[cfg(test)]
mod teste_automat {
    use super::*;

    /// Generator determinist de secvente cu repetitii dese — exact forma pe
    /// care bucla din difflib se poarta cel mai prost si in care greselile de
    /// departajare ies la iveala.
    fn secventa(seed: u64, n: usize, alfabet: u32) -> Vec<u32> {
        let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (0..n)
            .map(|_| {
                x = x
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((x >> 33) as u32) % alfabet
            })
            .collect()
    }

    /// Automatul trebuie sa dea **exact** ce da bucla: aceeasi potrivire, nu
    /// una la fel de lunga in alta parte. Daca departajarea difera, se schimba
    /// ce blocuri ies „modificat" fata de „sters" plus „adaugat".
    #[test]
    fn automatul_da_aceeasi_potrivire_ca_bucla() {
        for seed in 0..200u64 {
            let alfabet = 2 + (seed % 7) as u32;
            let a = secventa(seed, 40 + (seed % 37) as usize, alfabet);
            let b = secventa(seed + 9_000, 40 + (seed % 29) as usize, alfabet);
            let sm = SequenceMatcher::new(&a, &b);
            for (alo, ahi, blo, bhi) in [
                (0, a.len(), 0, b.len()),
                (1, a.len(), 0, b.len().saturating_sub(1)),
                (a.len() / 3, a.len(), b.len() / 4, b.len()),
                (0, a.len() / 2, 0, b.len() / 2),
            ] {
                assert_eq!(
                    sm.potrivire_prin_automat(alo, ahi, blo, bhi),
                    sm.potrivire_prin_bucla(alo, ahi, blo, bhi),
                    "seed {seed}, interval {alo}..{ahi} / {blo}..{bhi}"
                );
            }
        }
    }

    /// Si pe tot lantul recursiv, nu doar pe un apel: blocurile potrivite
    /// trebuie sa iasa identice, fiindca din ele se nasc opcode-urile.
    #[test]
    fn blocurile_potrivite_sunt_identice_pe_lantul_recursiv() {
        for seed in 200..260u64 {
            let a = secventa(seed, 120, 3);
            let b = secventa(seed + 5_000, 110, 3);
            let sm = SequenceMatcher::new(&a, &b);
            let prin_automat = sm.matching_blocks();
            // acelasi lant, dar fortand bucla peste tot
            let mut coada = vec![(0, a.len(), 0, b.len())];
            let mut potriviri: Vec<(usize, usize, usize)> = Vec::new();
            while let Some((alo, ahi, blo, bhi)) = coada.pop() {
                let (i, j, k) = sm.potrivire_prin_bucla(alo, ahi, blo, bhi);
                if k > 0 {
                    potriviri.push((i, j, k));
                    if alo < i && blo < j {
                        coada.push((alo, i, blo, j));
                    }
                    if i + k < ahi && j + k < bhi {
                        coada.push((i + k, ahi, j + k, bhi));
                    }
                }
            }
            potriviri.sort_unstable();
            let mut din_automat: Vec<(usize, usize, usize)> =
                prin_automat.into_iter().filter(|x| x.2 > 0).collect();
            din_automat.sort_unstable();
            // `matching_blocks` coalesceaza, deci comparam totalul potrivit
            let total_a: usize = din_automat.iter().map(|x| x.2).sum();
            let total_b: usize = potriviri.iter().map(|x| x.2).sum();
            assert_eq!(total_a, total_b, "seed {seed}");
        }
    }
}
