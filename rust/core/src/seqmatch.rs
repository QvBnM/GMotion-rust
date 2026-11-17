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

pub struct SequenceMatcher<'a, T: Eq + Hash> {
    a: &'a [T],
    b: &'a [T],
    b2j: HashMap<&'a T, Vec<usize>>,
}

impl<'a, T: Eq + Hash> SequenceMatcher<'a, T> {
    pub fn new(a: &'a [T], b: &'a [T]) -> Self {
        let mut b2j: HashMap<&'a T, Vec<usize>> = HashMap::with_capacity(b.len());
        for (i, el) in b.iter().enumerate() {
            b2j.entry(el).or_default().push(i);
        }
        Self { a, b, b2j }
    }

    fn indici(&self, el: &T) -> &[usize] {
        self.b2j.get(el).map(|v| v.as_slice()).unwrap_or(&[])
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
        let (mut besti, mut bestj, mut bestsize) = (alo, blo, 0usize);

        // j2len[j] = lungimea celei mai lungi potriviri care se termina la a[i-1], b[j]
        let mut j2len: HashMap<usize, usize> = HashMap::new();
        for i in alo..ahi {
            let mut newj2len: HashMap<usize, usize> = HashMap::new();
            for &j in self.indici(&a[i]) {
                if j < blo {
                    continue;
                }
                if j >= bhi {
                    break;
                }
                // in JS `j2len[j-1]` pe j=0 da undefined -> 0
                let anterior = j
                    .checked_sub(1)
                    .and_then(|jm| j2len.get(&jm).copied())
                    .unwrap_or(0);
                let k = anterior + 1;
                newj2len.insert(j, k);
                if k > bestsize {
                    besti = i + 1 - k;
                    bestj = j + 1 - k;
                    bestsize = k;
                }
            }
            j2len = newj2len;
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
