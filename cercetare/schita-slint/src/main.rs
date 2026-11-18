//! Schita de fezabilitate pentru directiile alese pe 2026-11-18:
//! Harta = ecranul cu lista obiectelor, Panza = interiorul editorului.
//! Deseneaza ambele ecrane cu Slint (randare software, ca in aplicatie) si le
//! scrie in PNG, ca sa se vada ca designul chiar se poate face in Rust.
//!
//!   cargo run --release -- harta  iesire.png
//!   cargo run --release -- panza  iesire.png [--paleta]
//!
//! Cercetare, nu cod livrat: datele hartii sunt generate (o baza de 21.183 de
//! obiecte), diferenta din panou si din panza e cea adevarata, din motor, pe
//! `referinta/instantanee/testsimilitudine.json`.
use std::rc::Rc;
use std::time::Instant;

use gmotion_fereastra::model_editor::{self, ModelEditor, Parte, RandAfisat, Sintaxa, Stare};
use slint::platform::software_renderer::{
    MinimalSoftwareWindow, PremultipliedRgbaColor, RepaintBufferType,
};
use slint::{Color, ComponentHandle, Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel};

slint::include_modules!();

const INSTANTANEU: &str = "../../referinta/instantanee/testsimilitudine.json";
const SURSA: &str = "CharismaERP_GetsBet";
const BAZA: &str = "CharismaERP_GetsBet_Copy";
const OBIECT: &str = "TESTSIMILITUDINE_AnalizaVanzari";

struct Platforma(Rc<MinimalSoftwareWindow>);
impl slint::platform::Platform for Platforma {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}

fn scrie_png(fereastra: &MinimalSoftwareWindow, w: u32, h: u32, cale: &str) -> f64 {
    let mut buf = vec![PremultipliedRgbaColor::default(); (w * h) as usize];
    let t = Instant::now();
    fereastra.draw_if_needed(|rd| {
        rd.render(&mut buf, w as usize);
    });
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let mut img = image::RgbaImage::new(w, h);
    for (i, p) in buf.iter().enumerate() {
        img.put_pixel(
            i as u32 % w,
            i as u32 / w,
            image::Rgba([p.red, p.green, p.blue, p.alpha]),
        );
    }
    img.save(cale).expect("PNG-ul nu se scrie");
    ms
}

// ------------------------------------------------------------- motorul

fn definitie(baza: &str, nume: &str) -> String {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(INSTANTANEU).expect("instantaneul"))
            .expect("json");
    v["baze"][baza]
        .as_array()
        .and_then(|a| a.iter().find(|o| o["nume"] == nume))
        .and_then(|o| o["definitie"].as_str())
        .unwrap_or("")
        .to_string()
}

fn obiecte_instantaneu() -> Vec<String> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(INSTANTANEU).expect("instantaneul"))
            .expect("json");
    v["baze"][SURSA]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|o| o["nume"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Sursa = referinta = stanga in model; baza = dreapta. Pe ecran, in sensul
/// deploy-ului: „acum" e baza, „dupa deploy" e sursa.
fn model() -> ModelEditor {
    let r = gmotion_core::pregateste_afisare(&definitie(SURSA, OBIECT));
    let t = gmotion_core::pregateste_afisare(&definitie(BAZA, OBIECT));
    model_editor::construieste(&r, &t)
}

fn bucati(p: &Parte) -> ModelRc<BucataUi> {
    let v: Vec<BucataUi> = p
        .bucati
        .iter()
        .map(|b| BucataUi {
            text: b.text.clone().into(),
            s: match b.sintaxa {
                Sintaxa::Text => 0,
                Sintaxa::CuvantCheie => 1,
                Sintaxa::Variabila => 2,
                Sintaxa::Numar => 3,
                Sintaxa::Sir => 4,
                Sintaxa::Comentariu => 5,
            },
            tare: b.schimbat,
        })
        .collect();
    Rc::new(VecModel::from(v)).into()
}

fn nr(p: &Parte) -> slint::SharedString {
    p.nr.map(|n| n.to_string()).unwrap_or_default().into()
}

// ------------------------------------------------------------- Harta

const LATURA: u32 = 14;
const PAS: u32 = 17;

/// k: 1 modificat, 2 doar in sursa, 3 doar in baza, 4 doar SET
struct Celula {
    k: u8,
    revizuit: bool,
    nume: String,
}

struct Grup {
    titlu: String,
    total: usize,
    celule: Vec<Celula>,
}

/// O baza de 21.183 de obiecte, cu 336 care difera: distributia din machete.
fn catalog() -> Vec<Grup> {
    let mut seed: u64 = 7;
    let mut rnd = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as f64 / (1u64 << 31) as f64
    };
    let def = [
        ("dbo", "Proceduri", 9412, 141, "usp_"),
        ("dbo", "Funcții", 2210, 38, "fn_"),
        ("dbo", "View-uri", 3104, 27, "vw_"),
        ("rpt", "Proceduri", 1873, 52, "usp_"),
        ("dbo", "Triggere", 612, 13, "trg_"),
        ("imp", "Proceduri", 1544, 31, "usp_Import"),
        ("anaf", "Proceduri", 980, 22, "usp_Anaf"),
        ("dbo", "Tipuri", 1448, 12, "tt_"),
    ];
    let radacini = [
        "Facturi",
        "Stoc",
        "Comenzi",
        "Parteneri",
        "Plati",
        "Livrari",
        "Cont",
        "Agenti",
        "Preturi",
        "Rapoarte",
    ];
    def.iter()
        .map(|&(sch, tip, total, dif, pref)| {
            let mut celule: Vec<Celula> = (0..dif)
                .map(|i| {
                    let r = rnd();
                    let k = if r < 0.86 {
                        1
                    } else if r < 0.93 {
                        2
                    } else if r < 0.97 {
                        3
                    } else {
                        4
                    };
                    Celula {
                        k,
                        revizuit: rnd() < 0.37,
                        nume: format!(
                            "{sch}.{pref}{}{}",
                            radacini[i % radacini.len()],
                            i / radacini.len() + 1
                        ),
                    }
                })
                .collect();
            // ordinea pe harta: starea intai, revizuitele la coada fiecarei stari
            celule.sort_by_key(|c| (c.k, c.revizuit));
            Grup {
                titlu: format!("{sch} · {tip}"),
                total,
                celule,
            }
        })
        .collect()
}

fn culoare_celula(k: u8) -> [u8; 3] {
    match k {
        1 => [0xff, 0xb5, 0x47],
        2 => [0x3d, 0xdc, 0x97],
        3 => [0xff, 0x6b, 0x81],
        _ => [0x8f, 0x9b, 0xff],
    }
}

/// Patratele unui grup, pictate direct in pixeli: o singura imagine in loc de
/// sute de elemente Slint. Asa harta ramane ieftina si la 20.000 de obiecte.
fn picteaza(celule: &[Celula], coloane: u32, fond: [u8; 3]) -> Image {
    let randuri = (celule.len() as u32).div_ceil(coloane).max(1);
    let (w, h) = (
        coloane * PAS - (PAS - LATURA),
        randuri * PAS - (PAS - LATURA),
    );
    let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
    let px = buf.make_mut_slice();
    for (i, c) in celule.iter().enumerate() {
        let (cx, cy) = ((i as u32 % coloane) * PAS, (i as u32 / coloane) * PAS);
        let [r, g, b] = culoare_celula(c.k);
        let a = if c.revizuit { 0.35 } else { 1.0 };
        let amesteca = |v: u8, f: u8| (v as f32 * a + f as f32 * (1.0 - a)) as u8;
        for y in 0..LATURA {
            for x in 0..LATURA {
                // colturi rotunjite de 3 px, aproximate
                let colt = |d: u32| {
                    d < 1 || (d < 2 && (x.min(LATURA - 1 - x) < 2 && y.min(LATURA - 1 - y) < 2))
                };
                if colt(x.min(LATURA - 1 - x)) && colt(y.min(LATURA - 1 - y)) {
                    continue;
                }
                px[((cy + y) * w + cx + x) as usize] = Rgba8Pixel {
                    r: amesteca(r, fond[0]),
                    g: amesteca(g, fond[1]),
                    b: amesteca(b, fond[2]),
                    a: 255,
                };
            }
        }
    }
    Image::from_rgba8(buf)
}

fn harta(cale: &str) {
    let fereastra = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(fereastra.clone()))).expect("platforma");
    let ui = Harta::new().expect("fereastra");

    let t = Instant::now();
    let grupuri = catalog();
    // trei coloane, fiecare grup in coloana cea mai scurta (zidarie)
    let (stanga, sus, latime_col, spatiu) = (28.0f32, 104.0f32, 272.0f32, 22.0f32);
    let coloane = ((latime_col as u32) + (PAS - LATURA)) / PAS;
    let mut inaltimi = [sus; 3];
    let mut ui_grupuri = Vec::new();
    let mut pozitii = Vec::new();
    for g in &grupuri {
        let c = (0..3)
            .min_by(|&a, &b| inaltimi[a].total_cmp(&inaltimi[b]))
            .unwrap_or(0);
        let randuri = (g.celule.len() as u32).div_ceil(coloane).max(1);
        let h = 24.0 + (randuri * PAS) as f32 + 30.0;
        let (x, y) = (stanga + c as f32 * (latime_col + spatiu), inaltimi[c]);
        inaltimi[c] += h + spatiu;
        pozitii.push((x, y));
        let identice = g.total - g.celule.len();
        ui_grupuri.push(GrupUi {
            x,
            y,
            w: latime_col,
            h,
            titlu: g.titlu.clone().into(),
            contor: format!("{} diferă", g.celule.len()).into(),
            rest: format!("{} identice din {}", mii(identice), mii(g.total)).into(),
            imagine: picteaza(&g.celule, coloane, [0x0d, 0x11, 0x20]),
            proportie: g.celule.len() as f32 / g.total as f32,
        });
    }
    let pictat_ms = t.elapsed().as_secs_f64() * 1000.0;
    ui.set_grupuri(Rc::new(VecModel::from(ui_grupuri)).into());
    ui.set_harta_h(inaltimi.iter().cloned().fold(0.0, f32::max));
    ui.set_sursa(SURSA.into());
    ui.set_tinta(BAZA.into());
    ui.set_nr_mod("312".into());
    ui.set_nr_add("18".into());
    ui.set_nr_del("6".into());
    ui.set_procent("37%".into());
    ui.set_titlu_nr("336".into());
    ui.set_titlu_rest("din 21.183 obiecte diferă".into());
    ui.set_plan_nr("7".into());

    // obiectul selectat: primul modificat din primul grup
    let (gx, gy) = pozitii[0];
    ui.set_sel_x(gx);
    ui.set_sel_y(gy + 24.0);
    ui.set_o_stare("Modificat, procedură".into());
    ui.set_o_culoare(Color::from_rgb_u8(0xff, 0xb5, 0x47));
    ui.set_o_schema("dbo.".into());
    ui.set_o_nume(OBIECT.into());
    ui.set_o_fraza(
        "Diferă în 4 locuri, toate în cod. Comentariile și spațiile nu se numără.".into(),
    );

    // codul unificat din panou, din motorul adevarat
    let m = model();
    let mut linii = Vec::new();
    let gol: ModelRc<BucataUi> = Rc::new(VecModel::<BucataUi>::default()).into();
    for r in model_editor::plieaza(&m, 2) {
        match r {
            RandAfisat::Pliere { eticheta, .. } => linii.push(LinieH {
                nr: "".into(),
                k: 0,
                pliu: eticheta.into(),
                bucati: gol.clone(),
            }),
            RandAfisat::Linie(i) => {
                let rd = &m.randuri[i];
                // minus = baza (acum), plus = sursa (dupa deploy)
                if rd.stare == Stare::Egal {
                    linii.push(LinieH {
                        nr: nr(&rd.dreapta),
                        k: 0,
                        pliu: "".into(),
                        bucati: bucati(&rd.dreapta),
                    });
                    continue;
                }
                if rd.dreapta.nr.is_some() {
                    linii.push(LinieH {
                        nr: nr(&rd.dreapta),
                        k: 1,
                        pliu: "".into(),
                        bucati: bucati(&rd.dreapta),
                    });
                }
                if rd.stanga.nr.is_some() {
                    linii.push(LinieH {
                        nr: nr(&rd.stanga),
                        k: 2,
                        pliu: "".into(),
                        bucati: bucati(&rd.stanga),
                    });
                }
            }
        }
    }
    ui.set_o_linii(Rc::new(VecModel::from(linii)).into());

    // bula de hover: se calculeaza din coordonatele mouse-ului, fara elemente
    let ui_slab = ui.as_weak();
    let nume: Vec<Vec<String>> = grupuri
        .iter()
        .map(|g| g.celule.iter().map(|c| c.nume.clone()).collect())
        .collect();
    let nume = Rc::new(nume);
    let poz = Rc::new(pozitii.clone());
    ui.on_mouse(move |g, x, y| {
        let Some(ui) = ui_slab.upgrade() else { return };
        let (col, rand) = ((x as u32) / PAS, (y as u32) / PAS);
        let i = (rand * coloane + col) as usize;
        let text = nume
            .get(g as usize)
            .and_then(|v| v.get(i))
            .cloned()
            .unwrap_or_default();
        let (gx, gy) = poz[g as usize];
        ui.set_bula(text.into());
        ui.set_bula_x(gx + x);
        ui.set_bula_y(gy + 24.0 + y + 72.0);
    });
    // in captura: bula pe al cincilea patrat din grupul al doilea
    ui.invoke_mouse(1, (4 * PAS + 7) as f32, 7.0);

    ui.show().expect("fereastra nu se arata");
    let (w, h) = (1440, 900);
    fereastra.set_size(slint::PhysicalSize::new(w, h));
    let ms = scrie_png(&fereastra, w, h, cale);
    println!(
        "harta: {} grupuri, picturi {pictat_ms:.1} ms, primul cadru {ms:.1} ms -> {cale}",
        grupuri.len()
    );
}

fn mii(n: usize) -> String {
    let s = n.to_string();
    let mut o = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            o.push('.');
        }
        o.push(c);
    }
    o
}

// ------------------------------------------------------------- Panza

fn panza(cale: &str, paleta: bool) {
    let fereastra = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(fereastra.clone()))).expect("platforma");
    let ui = Panza::new().expect("fereastra");

    let t = Instant::now();
    let m = model();
    let afisat = model_editor::plieaza(&m, 3);
    let model_ms = t.elapsed().as_secs_f64() * 1000.0;
    let gol: ModelRc<BucataUi> = Rc::new(VecModel::<BucataUi>::default()).into();
    let linii: Vec<LinieP> = afisat
        .iter()
        .map(|r| match r {
            RandAfisat::Pliere { eticheta, .. } => LinieP {
                k: 0,
                pliu: eticheta.clone().into(),
                nr_st: "".into(),
                nr_dr: "".into(),
                st: gol.clone(),
                dr: gol.clone(),
            },
            RandAfisat::Linie(i) => {
                let rd = &m.randuri[*i];
                LinieP {
                    // in sensul deploy-ului: Sters in model = doar in sursa = se adauga
                    k: match rd.stare {
                        Stare::Egal => 0,
                        Stare::Modificat => 1,
                        Stare::Sters => 2,
                        Stare::Adaugat => 3,
                    },
                    pliu: "".into(),
                    nr_st: nr(&rd.dreapta),
                    st: bucati(&rd.dreapta),
                    nr_dr: nr(&rd.stanga),
                    dr: bucati(&rd.stanga),
                }
            }
        })
        .collect();
    ui.set_linii(Rc::new(VecModel::from(linii)).into());

    // rigla: obiectul impartit in 24 de felii, fiecare colorata dupa ce contine
    let n = m.randuri.len().max(1);
    let mut rigla = vec![0; 24];
    for (j, &d) in m.diferente.iter().enumerate() {
        let felie = d * 24 / n;
        rigla[felie] = if j == 0 {
            4
        } else {
            match m.randuri[d].stare {
                Stare::Modificat => 1,
                Stare::Sters => 2,
                Stare::Adaugat => 3,
                Stare::Egal => 0,
            }
        };
    }
    ui.set_rigla(
        Rc::new(VecModel::from(
            rigla.into_iter().map(|k| SemnP { k }).collect::<Vec<_>>(),
        ))
        .into(),
    );

    ui.set_sursa(SURSA.into());
    ui.set_tinta(BAZA.into());
    ui.set_tip(format!("Procedură, diferă în {} locuri, în cod", m.diferente.len()).into());
    ui.set_schema("dbo.".into());
    ui.set_nume(OBIECT.into());
    ui.set_fraza(
        "Comentariile și spațiile nu se numără. Restul liniilor sunt identice și stau pliate."
            .into(),
    );
    ui.set_eticheta_st(format!("{BAZA}, acum").into());
    ui.set_eticheta_dr(format!("{SURSA}, după deploy").into());
    ui.set_revizuite("124".into());
    ui.set_din("din 336 revizuite".into());
    ui.set_arc(arc(124.0 / 336.0).into());
    ui.set_plan("Plan · 7".into());

    if paleta {
        let mut comenzi: Vec<ComandaUi> = obiecte_instantaneu()
            .into_iter()
            .filter(|n| n.contains("Case"))
            .take(5)
            .enumerate()
            .map(|(i, n)| ComandaUi {
                grup: if i == 0 { "Obiecte".into() } else { "".into() },
                text: format!("dbo.{n}").into(),
                sub: "procedură".into(),
                punct: Color::from_rgb_u8(0xf5, 0xa5, 0x24),
                mono: true,
                activ: i == 0,
            })
            .collect();
        for (i, (t, s)) in [
            ("Arată doar ce diferă în cod", ""),
            ("Deschide planul de deploy", "7 obiecte"),
            ("Recompară", "Ctrl R"),
        ]
        .iter()
        .enumerate()
        {
            comenzi.push(ComandaUi {
                grup: if i == 0 { "Comenzi".into() } else { "".into() },
                text: (*t).into(),
                sub: (*s).into(),
                punct: Color::from_argb_u8(0, 0, 0, 0),
                mono: false,
                activ: false,
            });
        }
        ui.set_comenzi(Rc::new(VecModel::from(comenzi)).into());
        ui.set_cautare("case".into());
        ui.set_paleta(true);
    }

    ui.show().expect("fereastra nu se arata");
    let (w, h) = (1440, 900);
    fereastra.set_size(slint::PhysicalSize::new(w, h));
    let ms = scrie_png(&fereastra, w, h, cale);
    println!(
        "panza: {} randuri in model, {} diferente, model {model_ms:.1} ms, primul cadru {ms:.1} ms -> {cale}",
        m.randuri.len(),
        m.diferente.len()
    );
}

/// Arcul inelului de progres, ca drum SVG in cutia 36x36.
fn arc(f: f32) -> String {
    let f = f.clamp(0.001, 0.999);
    let a = f * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    let (x, y) = (18.0 + 15.0 * a.cos(), 18.0 + 15.0 * a.sin());
    format!(
        "M 18 3 A 15 15 0 {} 1 {x:.2} {y:.2}",
        if f > 0.5 { 1 } else { 0 }
    )
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match (a.get(1).map(String::as_str), a.get(2)) {
        (Some("harta"), Some(cale)) => harta(cale),
        (Some("panza"), Some(cale)) => panza(cale, a.iter().any(|x| x == "--paleta")),
        _ => {
            eprintln!("folosire: schita-slint harta|panza iesire.png [--paleta]");
            std::process::exit(2);
        }
    }
}
