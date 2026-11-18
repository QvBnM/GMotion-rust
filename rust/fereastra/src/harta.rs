//! Harta de deasupra listei: fiecare obiect afisat e un patrat, grupat pe tip,
//! colorat dupa stare. Patratele se picteaza aici, in pixeli, intr-o imagine
//! pe grup — nu sute de elemente Slint.
//!
//! **Se strica astfel:** un patrat arata (sau selecteaza la clic) alt obiect
//! decat randul lui din tabel, conturul selectiei sta pe alt patrat, sau
//! grupurile ies din ecran. Toate trei se decid aici, cu teste fara fereastra:
//! `celula_la` (clic → rand) e inversa lui `pozitie_randului` (rand → patrat).

use crate::jetoane::{Culoare, Paleta};

/// Latura unui patrat si pasul dintre doua, in pixeli logici.
pub const LATURA: u32 = 12;
pub const PAS: u32 = 15;
/// Cate patrate pe coloana, in fiecare grup.
pub const RANDURI: u32 = 4;
/// Inaltimea titlului unui grup, deasupra patratelor.
pub const TITLU: f32 = 20.0;
/// Spatiul dintre grupuri, pe orizontala si intre doua randuri de grupuri.
pub const SPATIU_X: f32 = 24.0;
pub const SPATIU_Y: f32 = 12.0;
/// Cat de lat e cel mai ingust grup (sa incapa titlul).
pub const LATIME_MINIMA: f32 = 96.0;

/// Un obiect de pe harta: randul lui din lista afisata, starea (codul din
/// `RandListaUi.stare`: 0 modificat, 1 doar in sursa, 2 doar in baza,
/// 3 identic, 4 criptat) si daca e revizuit (patrat estompat).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Celula {
    pub rand: usize,
    pub stare: i32,
    pub revizuit: bool,
}

/// Un grup asezat: titlul, pozitia si celulele, in ordinea de pe harta
/// (pe coloane: primele `RANDURI` celule sunt prima coloana).
#[derive(Debug, Clone, PartialEq)]
pub struct Grup {
    pub titlu: String,
    pub x: f32,
    pub y: f32,
    pub latime: f32,
    pub celule: Vec<Celula>,
}

impl Grup {
    pub fn coloane(&self) -> u32 {
        (self.celule.len() as u32).div_ceil(RANDURI).max(1)
    }
}

/// Inaltimea unui rand de grupuri: titlul si patratele.
pub fn inaltime_grup() -> f32 {
    TITLU + (RANDURI * PAS - (PAS - LATURA)) as f32
}

/// Aseaza grupurile de la stanga la dreapta si trece pe randul urmator cand
/// nu mai incap in `latime`. In fiecare grup, celulele se ordoneaza dupa
/// stare, iar cele revizuite stau la coada starii lor. Intoarce si
/// inaltimea totala.
pub fn aseaza(grupuri: Vec<(String, Vec<Celula>)>, latime: f32) -> (Vec<Grup>, f32) {
    let mut out = Vec::new();
    let (mut x, mut y) = (0.0f32, 0.0f32);
    for (titlu, mut celule) in grupuri {
        if celule.is_empty() {
            continue;
        }
        celule.sort_by_key(|c| (c.stare, c.revizuit, c.rand));
        let coloane = (celule.len() as u32).div_ceil(RANDURI);
        let w = ((coloane * PAS - (PAS - LATURA)) as f32).max(LATIME_MINIMA);
        if x > 0.0 && x + w > latime {
            x = 0.0;
            y += inaltime_grup() + SPATIU_Y;
        }
        out.push(Grup {
            titlu,
            x,
            y,
            latime: w,
            celule,
        });
        x += w + SPATIU_X;
    }
    let inaltime = if out.is_empty() {
        0.0
    } else {
        y + inaltime_grup()
    };
    (out, inaltime)
}

/// Al catelea patrat din grup e sub punctul (x, y), in coordonatele imaginii
/// grupului (patratele merg pe coloane). Singurul loc unde se face calculul:
/// fereastra trimite coordonatele, nu il repeta.
pub fn indice_la(x: f32, y: f32) -> Option<usize> {
    if x < 0.0 || y < 0.0 {
        return None;
    }
    let (col, rand) = (x as u32 / PAS, y as u32 / PAS);
    if rand >= RANDURI {
        return None;
    }
    Some((col * RANDURI + rand) as usize)
}

/// Celula de sub punctul (x, y), in coordonatele imaginii grupului.
pub fn celula_la(g: &Grup, x: f32, y: f32) -> Option<Celula> {
    indice_la(x, y).and_then(|k| g.celule.get(k).copied())
}

/// Unde e patratul unui rand din lista: coltul lui, in coordonatele hartii.
pub fn pozitie_randului(grupuri: &[Grup], rand: usize) -> Option<(f32, f32)> {
    grupuri.iter().find_map(|g| {
        g.celule.iter().position(|c| c.rand == rand).map(|k| {
            let k = k as u32;
            (
                g.x + ((k / RANDURI) * PAS) as f32,
                g.y + TITLU + ((k % RANDURI) * PAS) as f32,
            )
        })
    })
}

fn culoare_starii(p: &Paleta, stare: i32) -> Culoare {
    match stare {
        0 => p.mod_bara,
        1 => p.add_bara,
        2 => p.del_bara,
        3 => p.ident,
        _ => p.text_tert,
    }
}

fn amesteca(a: Culoare, b: Culoare, f: f32) -> [u8; 3] {
    let c = |s: u32| -> f32 { ((a >> s) & 0xff) as f32 * f + ((b >> s) & 0xff) as f32 * (1.0 - f) };
    [c(16) as u8, c(8) as u8, c(0) as u8]
}

/// Pixelii unui grup (RGBA, rand cu rand) si dimensiunea imaginii. Fondul e
/// transparent; un patrat revizuit e estompat spre `suprafata`.
pub fn picteaza(g: &Grup, p: &Paleta) -> (u32, u32, Vec<[u8; 4]>) {
    let coloane = g.coloane();
    let (w, h) = (
        coloane * PAS - (PAS - LATURA),
        RANDURI * PAS - (PAS - LATURA),
    );
    let mut px = vec![[0u8; 4]; (w * h) as usize];
    for (k, c) in g.celule.iter().enumerate() {
        let k = k as u32;
        let (cx, cy) = ((k / RANDURI) * PAS, (k % RANDURI) * PAS);
        let f = if c.revizuit { 0.35 } else { 1.0 };
        let [r, gr, b] = amesteca(culoare_starii(p, c.stare), p.suprafata, f);
        for y in 0..LATURA {
            for x in 0..LATURA {
                let (dx, dy) = (x.min(LATURA - 1 - x), y.min(LATURA - 1 - y));
                // colturi rotunjite, aproximate
                if dx + dy < 2 {
                    continue;
                }
                px[((cy + y) * w + cx + x) as usize] = [r, gr, b, 255];
            }
        }
    }
    (w, h, px)
}

#[cfg(test)]
mod teste {
    use super::*;
    use crate::jetoane::{INTUNECAT, LUMINOS};

    fn celule(n: usize, de_la: usize) -> Vec<Celula> {
        (0..n)
            .map(|i| Celula {
                rand: de_la + i,
                stare: 0,
                revizuit: false,
            })
            .collect()
    }

    #[test]
    fn clic_pe_patrat_da_randul_lui_si_inapoi() {
        let (g, _) = aseaza(
            vec![
                ("procedura".into(), celule(10, 0)),
                ("functie".into(), celule(3, 10)),
            ],
            1000.0,
        );
        for gr in &g {
            for c in &gr.celule {
                let (x, y) = pozitie_randului(&g, c.rand).expect("fiecare rand are patrat");
                // un punct din mijlocul patratului, in coordonatele imaginii
                let gasit = celula_la(gr, x - gr.x + 6.0, y - gr.y - TITLU + 6.0);
                assert_eq!(gasit.map(|c| c.rand), Some(c.rand), "randul {}", c.rand);
            }
        }
        assert_eq!(pozitie_randului(&g, 99), None);
    }

    #[test]
    fn clic_in_afara_patratelor_nu_alege_nimic() {
        let (g, _) = aseaza(vec![("procedura".into(), celule(5, 0))], 1000.0);
        // a doua coloana are un singur patrat (5 = 4 + 1)
        assert_eq!(celula_la(&g[0], PAS as f32 + 1.0, PAS as f32 + 1.0), None);
        assert_eq!(celula_la(&g[0], 1.0, (RANDURI * PAS) as f32 + 1.0), None);
        assert_eq!(celula_la(&g[0], -1.0, 1.0), None);
    }

    #[test]
    fn grupurile_trec_pe_randul_urmator_cand_nu_incap() {
        // 40 de celule = 10 coloane = 147 px; trei grupuri pe 400 px: doua pe primul rand
        let (g, h) = aseaza(
            vec![
                ("a".into(), celule(40, 0)),
                ("b".into(), celule(40, 40)),
                ("c".into(), celule(40, 80)),
            ],
            400.0,
        );
        assert_eq!((g[0].y, g[1].y), (0.0, 0.0));
        assert!(
            g[2].y > 0.0 && g[2].x == 0.0,
            "al treilea grup trece dedesubt"
        );
        assert_eq!(h, g[2].y + inaltime_grup());
        assert!(g[1].x >= g[0].x + g[0].latime, "nu se suprapun");
    }

    #[test]
    fn grupurile_goale_dispar_si_cele_mici_au_loc_de_titlu() {
        let (g, h) = aseaza(
            vec![("gol".into(), vec![]), ("unul".into(), celule(1, 0))],
            500.0,
        );
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].latime, LATIME_MINIMA);
        assert!(h > 0.0);
        assert_eq!(aseaza(vec![], 500.0).1, 0.0);
    }

    #[test]
    fn in_grup_starea_vine_intai_si_revizuitele_la_coada() {
        let c = vec![
            Celula {
                rand: 0,
                stare: 2,
                revizuit: false,
            },
            Celula {
                rand: 1,
                stare: 0,
                revizuit: true,
            },
            Celula {
                rand: 2,
                stare: 0,
                revizuit: false,
            },
        ];
        let (g, _) = aseaza(vec![("t".into(), c)], 500.0);
        let ordine: Vec<usize> = g[0].celule.iter().map(|c| c.rand).collect();
        assert_eq!(ordine, vec![2, 1, 0]);
    }

    #[test]
    fn pictura_are_culoarea_starii_si_urmeaza_modul() {
        let c = vec![Celula {
            rand: 0,
            stare: 1,
            revizuit: false,
        }];
        let (g, _) = aseaza(vec![("t".into(), c)], 500.0);
        let centru = |p: &Paleta| {
            let (w, _, px) = picteaza(&g[0], p);
            px[(6 * w + 6) as usize]
        };
        let rgb = |c: Culoare| {
            [
                ((c >> 16) & 0xff) as u8,
                ((c >> 8) & 0xff) as u8,
                (c & 0xff) as u8,
                255,
            ]
        };
        assert_eq!(centru(&LUMINOS), rgb(LUMINOS.add_bara));
        assert_eq!(centru(&INTUNECAT), rgb(INTUNECAT.add_bara));
        // coltul e transparent: fondul hartii se vede
        let (_, _, px) = picteaza(&g[0], &LUMINOS);
        assert_eq!(px[0][3], 0);
    }
}
