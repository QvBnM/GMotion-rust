//! Harta, ecranul de ansamblu deschis din lista (`h`): cate o coloana pe tip
//! de obiect, despartite prin linii, care umplu impreuna latimea ecranului.
//! In fiecare coloana: titlul cu numarul, bara cu proportia starilor si cate
//! un patrat pe obiect, in ordinea din lista, pe blocuri de cate `BLOC`
//! randuri. Patratele si bara se picteaza aici, in pixeli — nu mii de
//! elemente Slint.
//!
//! **Se strica astfel:** un patrat arata (sau selecteaza la clic) alt obiect
//! decat al lui, conturul selectiei sta pe alt patrat, coloanele ies din
//! ecran sau lasa o parte din el goala. Toate se decid aici, cu teste
//! fara fereastra: `celula_la` (clic → rand) e inversa lui `pozitie_randului`.

use crate::jetoane::{Culoare, Paleta};

/// Marimea patratelor, in pixeli logici: latura, pasul dintre doua si
/// spatiul in plus dintre doua blocuri de randuri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scara {
    pub latura: u32,
    pub pas: u32,
    pub spatiu_bloc: u32,
}

/// Treptele de zoom, de la cea mai mare la cea mai mica. Cand obiectele nu
/// incap in inaltimea ecranului, harta coboara o treapta; abia la ultima
/// apare derularea.
pub const SCARI: [Scara; 7] = [
    // cele mari: harta are tot ecranul; cu putine obiecte, patratele cresc
    // pana o umplu, ca in macheta
    Scara {
        latura: 20,
        pas: 25,
        spatiu_bloc: 8,
    },
    Scara {
        latura: 17,
        pas: 21,
        spatiu_bloc: 7,
    },
    Scara {
        latura: 14,
        pas: 18,
        spatiu_bloc: 6,
    },
    Scara {
        latura: 11,
        pas: 14,
        spatiu_bloc: 5,
    },
    Scara {
        latura: 8,
        pas: 10,
        spatiu_bloc: 4,
    },
    Scara {
        latura: 6,
        pas: 8,
        spatiu_bloc: 3,
    },
    Scara {
        latura: 4,
        pas: 5,
        spatiu_bloc: 2,
    },
];

impl Scara {
    /// Treapta cu latura data (fereastra tine doar latura).
    pub fn cu_latura(latura: u32) -> Option<Scara> {
        SCARI.iter().copied().find(|s| s.latura == latura)
    }
}

/// Cate randuri de patrate intr-un bloc.
pub const BLOC: u32 = 5;
/// Cat de inalta poate fi harta inainte sa se micsoreze patratele, cand
/// fereastra inca n-are marime (altfel se ia inaltimea ecranului hartii).
pub const INALTIME_MAXIMA: f32 = 640.0;
/// Cat de mult creste latimea unei coloane cu numarul de obiecte: radacina
/// a patra, ca in macheta lui Mihai — tipul mare e mai lat, dar coloanele
/// raman apropiate (2100 / 480 / 250 / 150 de obiecte dau 36 / 25 / 21 /
/// 18 %, nu 70 / 16 / 8 / 5 %).
const PUTERE: f32 = 0.25;
/// Capul unei coloane, deasupra patratelor: titlul (20), bara la `Y_BARA`,
/// groasa de `GROSIME_BARA`, apoi un spatiu.
pub const CAP: f32 = 44.0;
pub const Y_BARA: f32 = 28.0;
pub const GROSIME_BARA: u32 = 4;
/// Spatiul dintre doua coloane (linia despartitoare sta la mijloc) si dintre
/// doua randuri de coloane.
pub const SPATIU_X: f32 = 40.0;
pub const SPATIU_Y: f32 = 20.0;
/// Latimea estimata a unui caracter din titlu (14 px) si a unei cifre din
/// numar (mono 12,5 px), plus spatiul minim dintre ele.
const LITERA: f32 = 8.0;
const CIFRA: f32 = 8.0;
const INTRE: f32 = 16.0;
/// Cat de ingusta poate fi o coloana, oricat de scurt i-ar fi titlul.
pub const LATIME_MINIMA: f32 = 120.0;

/// Un obiect de pe harta: randul lui din lista afisata, starea (codul din
/// `RandListaUi.stare`: 0 modificat, 1 doar in sursa, 2 doar in baza,
/// 3 identic, 4 criptat) si daca e revizuit (patrat estompat).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Celula {
    pub rand: usize,
    pub stare: i32,
    pub revizuit: bool,
}

/// O coloana asezata: titlul, pozitia, latimea si inaltimea randului ei de
/// coloane (pentru linia despartitoare), cate patrate pe rand si celulele,
/// in ordinea din tabel (pe randuri).
#[derive(Debug, Clone, PartialEq)]
pub struct Grup {
    pub titlu: String,
    pub x: f32,
    pub y: f32,
    pub latime: f32,
    pub inaltime: f32,
    pub coloane: u32,
    pub scara: Scara,
    pub celule: Vec<Celula>,
}

impl Grup {
    /// Cate randuri de patrate are coloana.
    pub fn randuri(&self) -> u32 {
        (self.celule.len() as u32).div_ceil(self.coloane.max(1))
    }
}

/// Cat ocupa `n` patrate alaturate pe orizontala.
fn intindere(s: Scara, n: u32) -> f32 {
    (n.max(1) * s.pas - (s.pas - s.latura)) as f32
}

/// Unde incepe randul `r` de patrate, fata de coltul grilei.
fn y_rand(s: Scara, r: u32) -> u32 {
    r * s.pas + (r / BLOC) * s.spatiu_bloc
}

/// Cat ocupa `r` randuri de patrate pe verticala, cu spatiile dintre blocuri.
fn inaltime_grila(s: Scara, r: u32) -> f32 {
    (y_rand(s, r.max(1) - 1) + s.latura) as f32
}

/// Latimea capului unei coloane cu `n` obiecte: titlul si numarul alaturi.
pub fn latime_titlu(titlu: &str, n: usize) -> f32 {
    (titlu.chars().count() as f32 * LITERA + INTRE + n.to_string().len() as f32 * CIFRA)
        .max(LATIME_MINIMA)
}

/// Cate patrate incap alaturi in latimea `w`.
fn incap(s: Scara, w: f32) -> u32 {
    ((w + (s.pas - s.latura) as f32) / s.pas as f32).floor() as u32
}

/// Latimile coloanelor de pe un rand, ca impreuna cu spatiile dintre ele sa
/// umple `latime`: pe masura lui `n^PUTERE`, dar niciuna sub capul ei.
fn latimi(linie: &[(String, Vec<Celula>, f32)], latime: f32) -> Vec<f32> {
    let n = linie.len();
    let util = (latime - SPATIU_X * n.saturating_sub(1) as f32).max(0.0);
    let pondere: Vec<f32> = linie
        .iter()
        .map(|g| (g.1.len().max(1) as f32).powf(PUTERE))
        .collect();
    // o coloana care ar iesi mai ingusta decat capul ei primeste capul, iar
    // restul se reimparte celorlalte; se repeta pana nu mai cade niciuna
    let mut la_cap = vec![false; n];
    loop {
        let (liber, suma) = impartire(linie, &pondere, &la_cap, util);
        let mut schimbat = false;
        for i in 0..n {
            if !la_cap[i] && liber * pondere[i] / suma < linie[i].2 {
                la_cap[i] = true;
                schimbat = true;
            }
        }
        if !schimbat {
            return (0..n)
                .map(|i| {
                    if la_cap[i] {
                        linie[i].2
                    } else {
                        (liber * pondere[i] / suma).floor()
                    }
                })
                .collect();
        }
    }
}

/// Cat ramane de impartit dupa coloanele fixate la cap, si suma ponderilor
/// celorlalte.
fn impartire(
    linie: &[(String, Vec<Celula>, f32)],
    pondere: &[f32],
    la_cap: &[bool],
    util: f32,
) -> (f32, f32) {
    let fix: f32 = (0..linie.len())
        .filter(|&i| la_cap[i])
        .map(|i| linie[i].2)
        .sum();
    let suma: f32 = (0..linie.len())
        .filter(|&i| !la_cap[i])
        .map(|i| pondere[i])
        .sum();
    ((util - fix).max(0.0), suma.max(f32::MIN_POSITIVE))
}

/// Aseaza coloanele in `latime`. Coloanele stau alaturi si umplu impreuna
/// latimea; fiecare ia o parte pe masura lui `n^PUTERE` (vezi `PUTERE`), iar
/// patratele umplu randul coloanei lor. Doar cand nici capetele nu incap
/// alaturi, coloanele trec pe randul urmator.
///
/// Patratele iau cea mai mare treapta din `SCARI` la care harta intra in
/// `inaltime_maxima` (zoom automat); daca nici cea mai mica nu intra, ramane
/// cea mai mica, cu derulare. Intoarce si inaltimea totala.
pub fn aseaza(
    grupuri: Vec<(String, Vec<Celula>)>,
    latime: f32,
    inaltime_maxima: f32,
) -> (Vec<Grup>, f32) {
    let mut asezare = (Vec::new(), 0.0);
    for s in SCARI {
        asezare = aseaza_la(grupuri.clone(), latime, s);
        if asezare.1 <= inaltime_maxima {
            break;
        }
    }
    asezare
}

/// `aseaza`, la o treapta data.
pub fn aseaza_la(grupuri: Vec<(String, Vec<Celula>)>, latime: f32, s: Scara) -> (Vec<Grup>, f32) {
    let grupuri = grupuri
        .into_iter()
        .filter(|(_, c)| !c.is_empty())
        .map(|(t, c)| {
            let w = latime_titlu(&t, c.len());
            (t, c, w)
        });

    // randurile de coloane: cate capete incap alaturi
    let mut linii: Vec<Vec<(String, Vec<Celula>, f32)>> = Vec::new();
    let mut ocupat = 0.0f32;
    for g in grupuri {
        let w = g.2;
        match linii.last_mut() {
            Some(l) if ocupat + SPATIU_X + w <= latime => {
                ocupat += SPATIU_X + w;
                l.push(g);
            }
            _ => {
                ocupat = w;
                linii.push(vec![g]);
            }
        }
    }

    let mut out = Vec::new();
    let mut y = 0.0f32;
    for (i, linie) in linii.into_iter().enumerate() {
        if i > 0 {
            y += SPATIU_Y;
        }
        let w = latimi(&linie, latime);
        let inceput = out.len();
        let mut x = 0.0f32;
        let mut randuri = 1;
        for ((titlu, celule, _), w) in linie.into_iter().zip(w) {
            let coloane = incap(s, w).min(celule.len() as u32).max(1);
            let g = Grup {
                titlu,
                x,
                y,
                latime: w,
                inaltime: 0.0,
                coloane,
                scara: s,
                celule,
            };
            randuri = randuri.max(g.randuri());
            x += g.latime + SPATIU_X;
            out.push(g);
        }
        let inalt = CAP + inaltime_grila(s, randuri);
        for g in &mut out[inceput..] {
            g.inaltime = inalt;
        }
        y += inalt;
    }
    (out, y)
}

/// Al catelea patrat al unei coloane cu `coloane` patrate pe rand, la
/// treapta `s`, e sub punctul (x, y), in coordonatele grilei. Singurul loc
/// unde se face calculul: fereastra trimite coordonatele, nu il repeta.
pub fn indice_la(s: Scara, coloane: u32, x: f32, y: f32) -> Option<usize> {
    if x < 0.0 || y < 0.0 {
        return None;
    }
    let (x, y) = (x as u32, y as u32);
    let inalt_bloc = BLOC * s.pas + s.spatiu_bloc;
    let bloc = y / inalt_bloc;
    let in_bloc = (y - bloc * inalt_bloc) / s.pas;
    let col = x / s.pas;
    if col >= coloane || in_bloc >= BLOC {
        return None;
    }
    Some(((bloc * BLOC + in_bloc) * coloane + col) as usize)
}

/// Celula de sub punctul (x, y), in coordonatele grilei.
pub fn celula_la(g: &Grup, x: f32, y: f32) -> Option<Celula> {
    indice_la(g.scara, g.coloane, x, y).and_then(|k| g.celule.get(k).copied())
}

/// Unde e patratul `k` al unei grile cu `coloane` patrate pe rand, fata de
/// coltul ei. Inversa lui `indice_la`.
pub fn coltul_in_grila(s: Scara, coloane: u32, k: usize) -> (u32, u32) {
    let (k, c) = (k as u32, coloane.max(1));
    ((k % c) * s.pas, y_rand(s, k / c))
}

fn coltul(g: &Grup, k: usize) -> (u32, u32) {
    coltul_in_grila(g.scara, g.coloane, k)
}

/// Unde e patratul unui rand din lista: coltul lui, in coordonatele hartii.
pub fn pozitie_randului(grupuri: &[Grup], rand: usize) -> Option<(f32, f32)> {
    grupuri.iter().find_map(|g| {
        g.celule.iter().position(|c| c.rand == rand).map(|k| {
            let (cx, cy) = coltul(g, k);
            (g.x + cx as f32, g.y + CAP + cy as f32)
        })
    })
}

fn culoare_starii(p: &Paleta, stare: i32) -> Culoare {
    match stare {
        0 => p.harta_mod,
        1 => p.harta_add,
        2 => p.harta_del,
        3 => p.harta_gol,
        _ => p.text_tert,
    }
}

fn amesteca(a: Culoare, b: Culoare, f: f32) -> [u8; 4] {
    let c = |s: u32| -> u8 {
        (((a >> s) & 0xff) as f32 * f + ((b >> s) & 0xff) as f32 * (1.0 - f)) as u8
    };
    [c(16), c(8), c(0), 255]
}

/// Pixelii grilei unei coloane (RGBA, rand cu rand) si dimensiunea imaginii,
/// in pixeli logici. Pentru ecran se foloseste `picteaza_la`.
pub fn picteaza(g: &Grup, p: &Paleta) -> (u32, u32, Vec<[u8; 4]>) {
    picteaza_la(g, p, 1.0)
}

/// Ca `picteaza`, dar la `factor` pixeli fizici pe pixel logic (scalarea
/// ecranului: 1,25, 1,5, 2). Pictata in pixeli logici, imaginea se marea pe
/// un ecran scalat si patratele ieseau neclare (15 D). Coltul fiecarui patrat
/// se rotunjeste la pixel, deci la factori neintregi doua patrate pot diferi
/// cu un pixel, dar raman nete.
pub fn picteaza_la(g: &Grup, p: &Paleta, factor: f32) -> (u32, u32, Vec<[u8; 4]>) {
    let f = factor.max(0.5);
    let fizic = |v: u32| -> u32 { (v as f32 * f).round() as u32 };
    let (w, h) = (
        fizic(intindere(g.scara, g.coloane) as u32).max(1),
        fizic(inaltime_grila(g.scara, g.randuri()) as u32).max(1),
    );
    let mut px = vec![[0u8; 4]; (w * h) as usize];
    let l = fizic(g.scara.latura).max(1);
    // colturi rotunjite, aproximate; patratele mici raman drepte;
    // la 8 px un colt de 2 face din patrat un cerc; acolo doar se tesesc
    let colt = match g.scara.latura {
        11.. => fizic(2),
        8..=10 => fizic(1),
        _ => 0,
    };
    for (k, c) in g.celule.iter().enumerate() {
        let (cx, cy) = coltul(g, k);
        let (cx, cy) = (fizic(cx), fizic(cy));
        let k_f = if c.revizuit { 0.4 } else { 1.0 };
        let culoare = amesteca(culoare_starii(p, c.stare), p.suprafata, k_f);
        for y in 0..l {
            for x in 0..l {
                let (dx, dy) = (x.min(l - 1 - x), y.min(l - 1 - y));
                if dx + dy < colt {
                    continue;
                }
                let (px_x, px_y) = (cx + x, cy + y);
                if px_x < w && px_y < h {
                    px[(px_y * w + px_x) as usize] = culoare;
                }
            }
        }
    }
    (w, h, px)
}

/// Bucatile barei de proportie: modificate, doar in sursa, doar in baza,
/// restul (identice si criptate), fiecare cu numarul ei de obiecte.
pub fn proportii(g: &Grup) -> [usize; 4] {
    let mut n = [0; 4];
    for c in &g.celule {
        n[c.stare.clamp(0, 3) as usize] += 1;
    }
    n
}

/// Spatiul dintre doua bucati ale barei.
const GOL_BARA: u32 = 3;

/// Pixelii barei de proportie, lata cat coloana: fiecare stare ia o bucata
/// pe masura numarului ei de obiecte; o stare fara obiecte nu apare.
pub fn picteaza_bara(g: &Grup, p: &Paleta) -> (u32, u32, Vec<[u8; 4]>) {
    picteaza_bara_la(g, p, 1.0)
}

/// Ca `picteaza_bara`, la `factor` pixeli fizici pe pixel logic.
pub fn picteaza_bara_la(g: &Grup, p: &Paleta, factor: f32) -> (u32, u32, Vec<[u8; 4]>) {
    let f = factor.max(0.5);
    let fizic = |v: u32| -> u32 { (v as f32 * f).round() as u32 };
    let (w, h) = (fizic(g.latime as u32).max(1), fizic(GROSIME_BARA).max(1));
    let gol = fizic(GOL_BARA);
    let mut px = vec![[0u8; 4]; (w * h) as usize];
    let n = proportii(g);
    let culori = [p.harta_mod, p.harta_add, p.harta_del, p.harta_gol];
    let bucati = n.iter().filter(|&&k| k > 0).count() as u32;
    let total = n.iter().sum::<usize>().max(1) as f32;
    let util = w.saturating_sub(gol * bucati.saturating_sub(1)) as f32;
    let mut x0 = 0u32;
    let mut cumul = 0usize;
    for (k, &cate) in n.iter().enumerate() {
        if cate == 0 {
            continue;
        }
        cumul += cate;
        // capatul se ia din suma cumulata: bucatile umplu exact bara
        let gol_pana_aici = gol * (n[..k].iter().filter(|&&c| c > 0).count() as u32);
        let x1 = ((cumul as f32 / total * util).round() as u32 + gol_pana_aici).min(w);
        let culoare = amesteca(culori[k], culori[k], 1.0);
        for y in 0..h {
            for x in x0..x1 {
                // capetele bucatii, rotunjite
                let colt = (y == 0 || y == h - 1) && (x == x0 || x + 1 == x1);
                if !colt {
                    px[(y * w + x) as usize] = culoare;
                }
            }
        }
        x0 = x1 + gol;
    }
    (w, h, px)
}

#[cfg(test)]
mod teste {
    use super::*;

    use crate::jetoane::{INTUNECAT, LUMINOS};

    const S: Scara = SCARI[0];

    fn celule(n: usize, de_la: usize) -> Vec<Celula> {
        (0..n)
            .map(|i| Celula {
                rand: de_la + i,
                stare: 0,
                revizuit: false,
            })
            .collect()
    }

    /// Cazul din captura: 34 / 9 / 3 / 3 obiecte, pe latimea ferestrei.
    fn captura(latime: f32) -> (Vec<Grup>, f32) {
        aseaza_la(
            vec![
                ("procedura".into(), celule(34, 0)),
                ("functie".into(), celule(9, 34)),
                ("trigger".into(), celule(3, 43)),
                ("view".into(), celule(3, 46)),
            ],
            latime,
            S,
        )
    }

    fn verifica_inversa(g: &[Grup]) {
        for gr in g {
            for c in &gr.celule {
                let (x, y) = pozitie_randului(g, c.rand).expect("fiecare rand are patrat");
                // un punct din mijlocul patratului, in coordonatele imaginii
                let m = gr.scara.latura as f32 / 2.0;
                let gasit = celula_la(gr, x - gr.x + m, y - gr.y - CAP + m);
                assert_eq!(gasit.map(|c| c.rand), Some(c.rand), "randul {}", c.rand);
            }
        }
    }

    fn verifica_incadrarea(g: &[Grup], latime: f32) {
        for (i, a) in g.iter().enumerate() {
            assert!(
                a.latime >= intindere(S, a.coloane),
                "{}: patratele ies din grup",
                a.titlu
            );
            assert!(
                a.latime >= latime_titlu(&a.titlu, a.celule.len()),
                "{}: titlul nu incape",
                a.titlu
            );
            let singur = g.iter().filter(|b| b.y == a.y).count() == 1;
            assert!(
                singur || a.x + a.latime <= latime,
                "{}: iese din cartus",
                a.titlu
            );
            for b in &g[i + 1..] {
                let alaturi = a.x + a.latime <= b.x || b.x + b.latime <= a.x;
                assert!(
                    a.y != b.y || alaturi,
                    "{} si {} se suprapun",
                    a.titlu,
                    b.titlu
                );
            }
        }
    }

    #[test]
    fn clic_pe_patrat_da_randul_lui_si_inapoi() {
        for latime in [1372.0, 600.0, 300.0, 120.0] {
            let (g, _) = captura(latime);
            verifica_inversa(&g);
        }
        assert_eq!(pozitie_randului(&captura(1372.0).0, 99), None);
    }

    #[test]
    fn coloanele_au_latimi_apropiate_mai_late_pentru_tipul_mai_mare() {
        // ca in macheta: 2100 / 480 / 250 / 150 de obiecte
        let (g, _) = aseaza_la(multe_ca_in_macheta(), 1372.0, S);
        assert!(g.iter().all(|x| x.y == 0.0), "toate pe un rand");
        for p in g.windows(2) {
            assert!(
                p[0].latime >= p[1].latime,
                "{} sub {}",
                p[0].titlu,
                p[1].titlu
            );
        }
        // procedura de 14 ori mai multe decat triggere, dar nici de doua ori mai lata
        let raport = g[0].latime / g[3].latime;
        assert!(raport > 1.5 && raport < 2.5, "raport {raport}");
        verifica_incadrarea(&g, 1372.0);
    }

    #[test]
    fn pe_ecran_ingust_grupurile_cresc_in_inaltime_si_raman_in_cartus() {
        for latime in [600.0, 400.0, 300.0, 200.0, 120.0] {
            let (g, h) = captura(latime);
            verifica_incadrarea(&g, latime);
            assert!(h >= captura(1372.0).1);
        }
        let (g, _) = captura(600.0);
        assert!(g[0].randuri() > 1, "procedura nu mai incape pe un rand");
        assert!(g.iter().all(|x| x.y == 0.0), "titlurile inca incap alaturi");
    }

    #[test]
    fn grupurile_trec_pe_randul_urmator_doar_cand_nu_incap_titlurile() {
        let (g, h) = captura(200.0);
        assert!(g.iter().any(|x| x.y > 0.0));
        let ultimul = g.iter().map(|x| x.y).fold(0.0, f32::max);
        assert!(h > ultimul);
        verifica_incadrarea(&g, 200.0);
    }

    #[test]
    fn singura_coloana_ia_toata_latimea() {
        let (g, _) = aseaza_la(vec![("functie".into(), celule(9, 0))], 1000.0, S);
        assert_eq!((g[0].latime, g[0].coloane, g[0].randuri()), (1000.0, 9, 1));
    }

    #[test]
    fn coloanele_umplu_cartusul_pe_fiecare_rand() {
        for latime in [1372.0, 900.0, 600.0, 300.0] {
            let (g, _) = captura(latime);
            for y in g.iter().map(|x| x.y) {
                let rand: Vec<&Grup> = g.iter().filter(|x| x.y == y).collect();
                let ultimul = rand[rand.len() - 1];
                let dreapta = ultimul.x + ultimul.latime;
                // restul se imparte rotunjit in jos: ramane sub un pixel pe coloana
                assert!(
                    dreapta <= latime && latime - dreapta < rand.len() as f32,
                    "la {latime}: randul se termina la {dreapta}"
                );
                assert!(rand.iter().all(|x| x.inaltime == rand[0].inaltime));
            }
        }
    }

    #[test]
    fn grupurile_goale_dispar() {
        let (g, h) = aseaza_la(
            vec![("gol".into(), vec![]), ("unul".into(), celule(1, 0))],
            500.0,
            S,
        );
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].latime, 500.0, "singura coloana umple cartusul");
        assert!(h > 0.0);
        assert_eq!(aseaza_la(vec![], 500.0, S).1, 0.0);
    }

    #[test]
    fn clic_in_afara_patratelor_nu_alege_nimic() {
        let g = Grup {
            titlu: "p".into(),
            x: 0.0,
            y: 0.0,
            latime: 120.0,
            inaltime: 0.0,
            coloane: 3,
            scara: S,
            celule: celule(5, 0),
        };
        // 3 coloane: al doilea rand are doua patrate
        let p = S.pas as f32;
        assert_eq!(celula_la(&g, 2.0 * p + 1.0, p + 1.0), None);
        assert_eq!(celula_la(&g, 3.0 * p + 1.0, 1.0), None);
        assert_eq!(celula_la(&g, 1.0, 2.0 * p + 1.0), None);
        assert_eq!(celula_la(&g, -1.0, 1.0), None);
    }

    #[test]
    fn spatiul_dintre_blocuri_nu_alege_nimic_si_blocul_urmator_continua() {
        let bloc = (BLOC * S.pas) as f32;
        assert_eq!(indice_la(S, 4, 1.0, bloc + 1.0), None);
        assert_eq!(
            indice_la(S, 4, 1.0, bloc + S.spatiu_bloc as f32 + 1.0),
            Some((BLOC * 4) as usize)
        );
    }

    #[test]
    fn in_coloana_ordinea_e_cea_din_tabel() {
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
        let (g, _) = aseaza_la(vec![("t".into(), c)], 500.0, S);
        let ordine: Vec<usize> = g[0].celule.iter().map(|c| c.rand).collect();
        assert_eq!(ordine, vec![0, 1, 2]);
    }

    #[test]
    fn pictura_are_culoarea_starii_si_urmeaza_modul() {
        let c = vec![Celula {
            rand: 0,
            stare: 1,
            revizuit: false,
        }];
        let (g, _) = aseaza_la(vec![("t".into(), c)], 500.0, S);
        let centru = |p: &Paleta| {
            let (w, _, px) = picteaza(&g[0], p);
            px[(7 * w + 7) as usize]
        };
        let rgb = |c: Culoare| {
            [
                ((c >> 16) & 0xff) as u8,
                ((c >> 8) & 0xff) as u8,
                (c & 0xff) as u8,
                255,
            ]
        };
        assert_eq!(centru(&LUMINOS), rgb(LUMINOS.harta_add));
        assert_eq!(centru(&INTUNECAT), rgb(INTUNECAT.harta_add));
        // coltul e transparent: fondul hartii se vede
        let (_, _, px) = picteaza(&g[0], &LUMINOS);
        assert_eq!(px[0][3], 0);
    }

    #[test]
    fn pictura_are_exact_marimea_grilei() {
        let (g, _) = captura(300.0);
        for gr in &g {
            let (w, h, px) = picteaza(gr, &LUMINOS);
            assert_eq!(w as f32, intindere(S, gr.coloane));
            assert_eq!(h as f32, inaltime_grila(S, gr.randuri()));
            assert_eq!(px.len(), (w * h) as usize);
        }
    }

    #[test]
    fn bara_imparte_latimea_pe_stari_in_ordine_si_sare_starile_lipsa() {
        let mut c = celule(6, 0); // 6 modificate
        c.extend((6..9).map(|rand| Celula {
            rand,
            stare: 2,
            revizuit: false,
        })); // 3 doar in baza, niciuna doar in sursa
        let (g, _) = aseaza_la(vec![("procedura".into(), c)], 300.0, S);
        assert_eq!(proportii(&g[0]), [6, 0, 3, 0]);
        let (w, h, px) = picteaza_bara(&g[0], &INTUNECAT);
        assert_eq!((w, h), (300, GROSIME_BARA));
        let rand: Vec<[u8; 4]> = px[w as usize..2 * w as usize].to_vec();
        let culoare = |c: Culoare| amesteca(c, c, 1.0);
        let mod_ = rand
            .iter()
            .filter(|p| **p == culoare(INTUNECAT.harta_mod))
            .count();
        let del = rand
            .iter()
            .filter(|p| **p == culoare(INTUNECAT.harta_del))
            .count();
        assert_eq!(
            rand.iter()
                .filter(|p| **p == culoare(INTUNECAT.harta_add))
                .count(),
            0
        );
        // 6:3, pe 300 - un gol
        assert!(
            (mod_ as i32 - 198).abs() <= 1 && (del as i32 - 99).abs() <= 1,
            "{mod_} {del}"
        );
        let primul_del = rand.iter().position(|p| *p == culoare(INTUNECAT.harta_del));
        assert!(
            primul_del
                > rand
                    .iter()
                    .rposition(|p| *p == culoare(INTUNECAT.harta_mod))
        );
        assert_eq!(
            rand[w as usize - 1],
            culoare(INTUNECAT.harta_del),
            "bara se umple pana la capat"
        );
    }

    /// Numerele din macheta lui Mihai, cele mai multe intai (ca in fereastra).
    fn multe_ca_in_macheta() -> Vec<(String, Vec<Celula>)> {
        vec![
            ("procedura".into(), celule(2100, 0)),
            ("functie".into(), celule(480, 2100)),
            ("view".into(), celule(250, 2580)),
            ("trigger".into(), celule(150, 2830)),
        ]
    }

    /// `n` obiecte in patru tipuri, ca intr-o baza mare.
    fn multe(n: usize) -> Vec<(String, Vec<Celula>)> {
        let parti = [
            ("procedura", 6),
            ("functie", 2),
            ("view", 1),
            ("trigger", 1),
        ];
        let mut de_la = 0;
        parti
            .iter()
            .map(|(t, p)| {
                let k = n * p / 10;
                let v = celule(k, de_la);
                de_la += k;
                (t.to_string(), v)
            })
            .collect()
    }

    #[test]
    fn prea_multe_obiecte_micsoreaza_patratele_ca_sa_intre() {
        let (putine, _) = aseaza(multe(50), 1372.0, INALTIME_MAXIMA);
        assert_eq!(putine[0].scara, SCARI[0], "cat incap, patratele raman mari");
        let mut anterioara = 0;
        for n in [50, 500, 1500, 3000, 6000] {
            let (g, h) = aseaza(multe(n), 1372.0, INALTIME_MAXIMA);
            let treapta = SCARI
                .iter()
                .position(|s| *s == g[0].scara)
                .expect("treapta cunoscuta");
            assert!(
                treapta >= anterioara,
                "la {n}: mai multe obiecte, patrate mai mari"
            );
            anterioara = treapta;
            assert!(
                g.iter().all(|x| x.scara == g[0].scara),
                "o singura marime pe harta"
            );
            // cat exista o treapta la care intra, intra
            let ultima = SCARI[SCARI.len() - 1];
            if aseaza_la(multe(n), 1372.0, ultima).1 <= INALTIME_MAXIMA {
                assert!(h <= INALTIME_MAXIMA, "la {n}: {h} peste {INALTIME_MAXIMA}");
            }
            // si nu e mai mica decat trebuie
            if treapta > 0 {
                assert!(aseaza_la(multe(n), 1372.0, SCARI[treapta - 1]).1 > INALTIME_MAXIMA);
            }
            verifica_inversa(&g);
        }
        assert!(anterioara > 0, "la 6000 de obiecte s-a micsorat");
    }

    #[test]
    fn la_fiecare_treapta_clicul_nimereste_patratul() {
        for s in SCARI {
            assert_eq!(Scara::cu_latura(s.latura), Some(s));
            let (g, _) = aseaza_la(multe(400), 700.0, s);
            verifica_inversa(&g);
            for gr in &g {
                let (w, h, _) = picteaza(gr, &LUMINOS);
                assert_eq!(w as f32, intindere(s, gr.coloane));
                assert_eq!(h as f32, inaltime_grila(s, gr.randuri()));
            }
        }
    }

    /// C9 din `cercetare/audit-qa-2026-11-18.md`: pe harta de la `aec038a`,
    /// un grup de peste ~8.700 de obiecte de acelasi tip dadea o imagine mai
    /// lata de 32.767 px si aplicatia **cadea** (`unwrap` pe `None` in
    /// `euclid`) la prima desenare — exact cazul „baza noua / goala" pe un ERP
    /// de 21.000 de obiecte.
    ///
    /// Harta de acum (refacuta la etapa 15) se rupe pe randuri in latimea
    /// data si coboara trepte de zoom, deci nu mai are cum. Testul o tine
    /// asa: nicio imagine mai mare de 8.192 px pe nicio latura, pe 50.000 de
    /// obiecte de un singur tip.
    #[test]
    fn nicio_imagine_a_hartii_nu_trece_de_8192_px() {
        for n in [8_700usize, 21_000, 50_000] {
            let (grupuri, inaltime) = aseaza(
                vec![("procedura".into(), celule(n, 0))],
                1300.0,
                INALTIME_MAXIMA,
            );
            assert!(inaltime.is_finite() && inaltime > 0.0, "{n}: {inaltime}");
            for g in &grupuri {
                for (w, h, px) in [picteaza(g, &LUMINOS), picteaza_bara(g, &LUMINOS)] {
                    assert!(w <= 8192 && h <= 8192, "{n} obiecte: imagine de {w}x{h} px");
                    assert_eq!(px.len(), (w * h) as usize);
                }
            }
            // toate obiectele ajung pe harta, oricat de multe
            let total: usize = grupuri.iter().map(|g| g.celule.len()).sum();
            assert_eq!(total, n);
        }
    }

    /// 15 D: pe un ecran scalat, harta se picteaza la pixelii fizici. La
    /// factor 2 imaginea e de doua ori mai mare, iar un patrat are latura
    /// dubla, nu e o marire a celui de la 1x (care ar fi neclara).
    #[test]
    fn harta_se_picteaza_la_pixelii_fizici() {
        let (g, _) = aseaza_la(vec![("t".into(), celule(7, 0))], 500.0, S);
        let p = &INTUNECAT;
        let (w1, h1, px1) = picteaza(&g[0], p);
        let (w2, h2, px2) = picteaza_la(&g[0], p, 2.0);
        assert_eq!((w2, h2), (w1 * 2, h1 * 2));
        let plini = |px: &[[u8; 4]]| px.iter().filter(|c| c[3] == 255).count();
        // aceeasi forma, ~de patru ori mai multi pixeli; doar colturile
        // rotunjite difera putin (cel mult 16 pixeli pe patrat)
        let (n1, n2) = (plini(&px1), plini(&px2));
        assert!(n2.abs_diff(n1 * 4) <= 7 * 16, "{n1} -> {n2}");
        // primul patrat: rand plin pe toata latura lui fizica, la mijloc
        let l = S.latura * 2;
        let y = l / 2;
        let rand: Vec<_> = (0..l).map(|x| px2[(y * w2 + x) as usize][3]).collect();
        assert!(rand.iter().all(|&a| a == 255), "{rand:?}");
        // la 1,5: dimensiuni rotunjite, fara panica, patratele nete
        let (w3, h3, px3) = picteaza_la(&g[0], p, 1.5);
        assert_eq!(px3.len(), (w3 * h3) as usize);
        assert!(
            px3.iter().all(|c| c[3] == 0 || c[3] == 255),
            "fara pixeli amestecati"
        );
        let (wb, hb, _) = picteaza_bara_la(&g[0], p, 2.0);
        assert_eq!((wb, hb), ((g[0].latime as u32) * 2, GROSIME_BARA * 2));
    }
}
