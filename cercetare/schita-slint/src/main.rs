//! Schita de fezabilitate a noii directii vizuale (2026-11-18): trei ecrane
//! legate, cu toate functiile din v3.4, desenate cu Slint (randare software,
//! ca in aplicatie) si scrise in PNG.
//!
//!   cargo run --release -- conectare iesire.png [--progres]
//!   cargo run --release -- lista     iesire.png
//!   cargo run --release -- panza     iesire.png [--ghidat] [--paleta]
//!
//! Cercetare, nu cod livrat. Lista foloseste o baza generata (21.183 de
//! obiecte); editorul foloseste diferenta adevarata, din motor, pe
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
const BAZA_2: &str = "CharismaProd";
const OBIECT: &str = "TESTSIMILITUDINE_AnalizaVanzari";
const W: u32 = 1440;
const H: u32 = 900;

struct Platforma(Rc<MinimalSoftwareWindow>);
impl slint::platform::Platform for Platforma {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}

fn fereastra() -> Rc<MinimalSoftwareWindow> {
    let f = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Platforma(f.clone()))).expect("platforma");
    f
}

/// Arata fereastra, randeaza primul cadru, scrie PNG-ul; intoarce cat a durat cadrul.
fn captura(f: &MinimalSoftwareWindow, cale: &str) -> f64 {
    f.set_size(slint::PhysicalSize::new(W, H));
    let mut buf = vec![PremultipliedRgbaColor::default(); (W * H) as usize];
    let t = Instant::now();
    f.draw_if_needed(|rd| {
        rd.render(&mut buf, W as usize);
    });
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let mut img = image::RgbaImage::new(W, H);
    for (i, p) in buf.iter().enumerate() {
        img.put_pixel(
            i as u32 % W,
            i as u32 / W,
            image::Rgba([p.red, p.green, p.blue, p.alpha]),
        );
    }
    img.save(cale).expect("PNG-ul nu se scrie");
    ms
}

fn model_din<T: Clone + 'static>(v: Vec<T>) -> ModelRc<T> {
    Rc::new(VecModel::from(v)).into()
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

// ------------------------------------------------------------- motorul

fn instantaneu() -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(INSTANTANEU).expect("instantaneul"))
        .expect("json")
}

fn definitie(baza: &str, nume: &str) -> String {
    instantaneu()["baze"][baza]
        .as_array()
        .and_then(|a| a.iter().find(|o| o["nume"] == nume))
        .and_then(|o| o["definitie"].as_str())
        .unwrap_or("")
        .to_string()
}

fn bucati(p: &Parte) -> ModelRc<BucataUi> {
    model_din(
        p.bucati
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
            .collect(),
    )
}

fn nr(p: &Parte) -> slint::SharedString {
    p.nr.map(|n| n.to_string()).unwrap_or_default().into()
}

// ------------------------------------------------------------- conectarea

fn conectare(cale: &str, progres: bool) {
    let f = fereastra();
    let ui = Conectare::new().expect("fereastra");
    ui.set_server("srv-dev\\SQL2019".into());
    ui.set_utilizator("gmotion".into());
    ui.set_parola("parola-de-proba".into());
    ui.set_servere_gasite(model_din(vec![
        ServerUi {
            nume: "srv-dev\\SQL2019".into(),
            eticheta: "IMPLICITĂ".into(),
        },
        ServerUi {
            nume: "srv-prod\\SQL2019".into(),
            eticheta: "CLUSTER".into(),
        },
        ServerUi {
            nume: "srv-test".into(),
            eticheta: "".into(),
        },
    ]));
    ui.set_stare_server("Conectat".into());
    ui.set_detalii_server(
        "SQL Server 2019 Enterprise (15.0.4345), Windows Server 2019, pornit de 41 de zile, 8 baze."
            .into(),
    );
    ui.set_certificat("Certificatul serverului nu e verificat.".into());
    ui.set_server_baze("srv-dev\\SQL2019".into());
    let baze = [
        (SURSA, "DEV", true, false),
        (BAZA, "TEST", false, true),
        (BAZA_2, "PROD", false, true),
        ("CharismaERP_Arhiva2024", "DEV", false, false),
        ("CharismaERP_Demo", "DEV", false, false),
        ("CharismaTest", "TEST", false, false),
        ("ReportServer", "DEV", false, false),
        ("SSISDB", "DEV", false, false),
    ];
    ui.set_baze(model_din(
        baze.iter()
            .map(|&(n, m, r, b)| BazaUi {
                nume: n.into(),
                mediu: m.into(),
                referinta: r,
                bifata: b,
            })
            .collect(),
    ));
    ui.set_rezumat_pornire(format!("{SURSA} față de 2 baze").into());
    ui.set_nr_sesiuni("3".into());
    ui.set_sesiune_gasita(SesiuneUi {
        referinta: SURSA.into(),
        server: "srv-dev\\SQL2019".into(),
        progres: "124 din 336 rezolvate, 7 la deploy".into(),
        cand: "ultima dată ieri, 17:40".into(),
    });
    let mut jurnal = vec![
        (
            "14:31:02",
            0,
            "Rețea",
            "Se caută servere în rețea (SQL Browser, UDP 1434)",
        ),
        (
            "14:31:03",
            1,
            "Rețea",
            "3 servere găsite: srv-dev\\SQL2019, srv-prod\\SQL2019, srv-test",
        ),
        (
            "14:31:09",
            0,
            "Conexiune",
            "Conectare la srv-dev\\SQL2019 cu utilizatorul gmotion",
        ),
        (
            "14:31:09",
            2,
            "Conexiune",
            "Certificatul serverului nu e verificat (TrustServerCertificate)",
        ),
        (
            "14:31:10",
            1,
            "Conexiune",
            "Conectat: SQL Server 2019 Enterprise (15.0.4345), 8 baze",
        ),
        (
            "14:31:22",
            0,
            "Baze",
            "Referință CharismaERP_GetsBet; de comparat: CharismaERP_GetsBet_Copy, CharismaProd",
        ),
    ];
    if progres {
        jurnal.extend([
            (
                "14:31:25",
                0,
                "Citire",
                "Se citesc obiectele din CharismaERP_GetsBet",
            ),
            ("14:31:29", 1, "Citire", "21.183 de obiecte citite în 4,1 s"),
            (
                "14:31:29",
                0,
                "Citire",
                "Se citesc obiectele din CharismaERP_GetsBet_Copy",
            ),
        ]);
        ui.set_ruleaza(true);
        ui.set_progres(0.62);
        ui.set_progres_text(
            "Se citesc obiectele din CharismaERP_GetsBet_Copy: 8.214 din 13.240".into(),
        );
        ui.set_progres_timp("7,4 s".into());
    }
    // banda arata ultimele randuri; tot jurnalul ramane in fisier
    let de_la = jurnal.len().saturating_sub(5);
    ui.set_jurnal(model_din(
        jurnal
            .into_iter()
            .skip(de_la)
            .map(|(o, n, e, t)| LogUi {
                ora: o.into(),
                nivel: n,
                etapa: e.into(),
                text: t.into(),
            })
            .collect(),
    ));
    ui.show().expect("fereastra nu se arata");
    let ms = captura(&f, cale);
    println!("conectare: primul cadru {ms:.1} ms -> {cale}");
}

// ------------------------------------------------------------- lista

const LATURA: u32 = 14;
const PAS: u32 = 17;
const RANDURI_HARTA: u32 = 6;

/// cat: 1 modificat, 2 lipsa din baza, 3 extra in baza, 4 doar SET
struct Obiect {
    schema: String,
    nume: String,
    baza: &'static str,
    tip: &'static str,
    cat: u8,
    sim: f32,
    diferente: String,
    stare: i32,
    modificat: String,
    deploy: i32,
    grup: usize,
    in_grup: usize,
}

struct Grup {
    titlu: String,
    total: usize,
    obiecte: Vec<usize>,
}

/// O baza de 21.183 de obiecte, cu cele care difera fata de doua baze.
fn catalog() -> (Vec<Grup>, Vec<Obiect>) {
    let mut seed: u64 = 11;
    let mut rnd = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as f64 / (1u64 << 31) as f64
    };
    let def = [
        ("dbo", "Proceduri", 9412, 141, "usp_", "PROC"),
        ("dbo", "Funcții", 2210, 38, "fn_", "FN"),
        ("dbo", "View-uri", 3104, 27, "vw_", "VIEW"),
        ("rpt", "Proceduri", 1873, 52, "usp_Raport", "PROC"),
        ("dbo", "Triggere", 612, 13, "trg_", "TRG"),
        ("imp", "Proceduri", 1544, 31, "usp_Import", "PROC"),
        ("anaf", "Proceduri", 980, 22, "usp_Anaf", "PROC"),
        ("dbo", "Tipuri", 1448, 12, "tt_", "TYPE"),
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
        "Sold",
    ];
    let mut obiecte = Vec::new();
    let mut grupuri = Vec::new();
    for (g, &(sch, titlu, total, dif, pref, tip)) in def.iter().enumerate() {
        let mut ale_grupului: Vec<Obiect> = (0..dif)
            .map(|i| {
                let r = rnd();
                let cat = if r < 0.86 {
                    1
                } else if r < 0.93 {
                    2
                } else if r < 0.97 {
                    3
                } else {
                    4
                };
                let (sim, diferente) = match cat {
                    1 => {
                        let m = 1 + (rnd() * 12.0) as u32;
                        let a = (rnd() * 4.0) as u32;
                        let s = (rnd() * 2.0) as u32;
                        let mut d = format!("{m} modificate");
                        if a > 0 {
                            d += &format!(", {a} adăugate");
                        }
                        if s > 0 {
                            d += &format!(", {s} șterse");
                        }
                        (0.62 + rnd() as f32 * 0.37, d)
                    }
                    2 => (0.0, "lipsește din bază".to_string()),
                    3 => (0.0, "doar în bază".to_string()),
                    _ => (1.0, "doar QUOTED_IDENTIFIER".to_string()),
                };
                let stare = if rnd() < 0.37 {
                    1
                } else if rnd() < 0.06 {
                    2
                } else {
                    0
                };
                let deploy = match (stare, rnd()) {
                    (2, _) => 4,
                    (1, x) if x < 0.3 => 1,
                    (1, x) if x < 0.45 => 2,
                    (1, x) if x < 0.5 => 3,
                    _ => 0,
                };
                Obiect {
                    schema: format!("{sch}."),
                    nume: format!(
                        "{pref}{}{}",
                        radacini[i % radacini.len()],
                        i / radacini.len() + 1
                    ),
                    baza: if rnd() < 0.55 { BAZA } else { BAZA_2 },
                    tip,
                    cat,
                    sim,
                    diferente,
                    stare,
                    modificat: format!(
                        "2026-{:02}-{:02}",
                        1 + (rnd() * 10.0) as u32,
                        1 + (rnd() * 27.0) as u32
                    ),
                    deploy,
                    grup: g,
                    in_grup: 0,
                }
            })
            .collect();
        // pe harta: starea intai, rezolvatele la coada fiecarei stari
        ale_grupului.sort_by_key(|o| (o.cat, o.stare != 0));
        let mut idx = Vec::new();
        for (j, mut o) in ale_grupului.into_iter().enumerate() {
            o.in_grup = j;
            idx.push(obiecte.len());
            obiecte.push(o);
        }
        grupuri.push(Grup {
            titlu: format!("{sch} · {titlu}"),
            total,
            obiecte: idx,
        });
    }
    (grupuri, obiecte)
}

fn culoare_cat(cat: u8) -> [u8; 3] {
    match cat {
        1 => [0xff, 0xb5, 0x47],
        2 => [0x3d, 0xdc, 0x97],
        3 => [0xff, 0x6b, 0x81],
        _ => [0x8f, 0x9b, 0xff],
    }
}

/// Patratele unui grup, pictate in pixeli, pe coloane de cate RANDURI_HARTA:
/// o singura imagine in loc de sute de elemente Slint.
fn picteaza(obiecte: &[&Obiect], fond: [u8; 3]) -> (Image, u32) {
    let coloane = (obiecte.len() as u32).div_ceil(RANDURI_HARTA).max(1);
    let (w, h) = (
        coloane * PAS - (PAS - LATURA),
        RANDURI_HARTA * PAS - (PAS - LATURA),
    );
    let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
    let px = buf.make_mut_slice();
    for (i, o) in obiecte.iter().enumerate() {
        let (cx, cy) = (
            (i as u32 / RANDURI_HARTA) * PAS,
            (i as u32 % RANDURI_HARTA) * PAS,
        );
        let [r, g, b] = culoare_cat(o.cat);
        let a = if o.stare == 0 { 1.0 } else { 0.35 };
        let amesteca = |v: u8, f: u8| (v as f32 * a + f as f32 * (1.0 - a)) as u8;
        for y in 0..LATURA {
            for x in 0..LATURA {
                let (dx, dy) = (x.min(LATURA - 1 - x), y.min(LATURA - 1 - y));
                // colturi rotunjite, aproximate
                if dx + dy < 2 {
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
    (Image::from_rgba8(buf), w)
}

fn lista(cale: &str) {
    let f = fereastra();
    let ui = Lista::new().expect("fereastra");

    let t = Instant::now();
    let (grupuri, obiecte) = catalog();
    // harta: grupurile unul langa altul, pe un singur rand
    // coordonatele sunt in cartusul hartii; cartusul incepe la y = 128 in fereastra
    let (stanga, sus, spatiu, cartus_y) = (40.0f32, 14.0f32, 20.0f32, 128.0f32);
    let mut x = stanga;
    let mut pozitii = Vec::new();
    let mut ui_grupuri = Vec::new();
    for g in &grupuri {
        let ale: Vec<&Obiect> = g.obiecte.iter().map(|&i| &obiecte[i]).collect();
        let (img, w) = picteaza(&ale, [0x15, 0x1a, 0x2e]);
        let w = (w as f32).max(104.0);
        pozitii.push((x, sus));
        ui_grupuri.push(GrupUi {
            x,
            y: sus,
            w,
            titlu: g.titlu.clone().into(),
            contor: format!("{} din {}", ale.len(), mii(g.total)).into(),
            imagine: img,
            proportie: ale.len() as f32 / g.total as f32,
        });
        x += w + spatiu;
    }
    let pictat_ms = t.elapsed().as_secs_f64() * 1000.0;
    let harta_h = 22.0 + (RANDURI_HARTA * PAS) as f32 + 26.0;
    ui.set_grupuri(model_din(ui_grupuri));
    ui.set_harta_h(harta_h);

    // tabelul: sortat pe similaritate, crescator, ca in captura v3.4
    let mut ordine: Vec<usize> = (0..obiecte.len()).collect();
    ordine.sort_by(|&a, &b| obiecte[a].sim.total_cmp(&obiecte[b].sim).then(a.cmp(&b)));
    let ordine: Vec<usize> = ordine
        .into_iter()
        .filter(|&i| obiecte[i].cat == 1)
        .chain((0..obiecte.len()).filter(|&i| obiecte[i].cat != 1))
        .collect();
    let randuri: Vec<RandUi> = ordine
        .iter()
        .enumerate()
        .map(|(n, &i)| {
            let o = &obiecte[i];
            RandUi {
                nr: (n + 1).to_string().into(),
                schema: o.schema.clone().into(),
                nume: o.nume.clone().into(),
                baza: o.baza.into(),
                tip: o.tip.into(),
                cat: o.cat as i32,
                similaritate: o.sim,
                sim_text: if o.cat == 1 {
                    format!("{:.1}%", o.sim * 100.0).replace('.', ",")
                } else {
                    "—".into()
                }
                .into(),
                diferente: o.diferente.clone().into(),
                stare: o.stare,
                modificat: o.modificat.clone().into(),
                deploy: o.deploy,
            }
        })
        .collect();
    let selectat = 2usize;
    let o = &obiecte[ordine[selectat]];
    let (gx, gy) = pozitii[o.grup];
    let (col, rand) = (
        o.in_grup as u32 / RANDURI_HARTA,
        o.in_grup as u32 % RANDURI_HARTA,
    );
    ui.set_sel_x(gx + (col * PAS) as f32);
    ui.set_sel_y(gy + 22.0 + (rand * PAS) as f32);
    ui.set_selectat(selectat as i32);
    ui.set_randuri(model_din(randuri));
    let col = |t: &str, l: f32, s: i32| ColoanaUi {
        titlu: t.into(),
        latime: l,
        sortata: s,
    };
    ui.set_coloane(model_din(vec![
        col("#", 44.0, 0),
        col("Obiect", 0.0, 0),
        col("Bază", 210.0, 0),
        col("Tip", 72.0, 0),
        col("Similaritate", 170.0, 1),
        col("Diferențe", 220.0, 0),
        col("Stare", 64.0, 0),
        col("Ultima modif.", 120.0, 0),
        col("Deploy", 130.0, 0),
        col("Acțiune", 92.0, 0),
    ]));

    let numara = |f: &dyn Fn(&Obiect) -> bool| obiecte.iter().filter(|o| f(o)).count();
    let total_dif = obiecte.len();
    ui.set_sursa(SURSA.into());
    ui.set_tinta("2 baze".into());
    ui.set_nr_dif(numara(&|o| o.cat == 1 || o.cat == 4).to_string().into());
    ui.set_nr_lipsa(numara(&|o| o.cat == 2).to_string().into());
    ui.set_nr_extra(numara(&|o| o.cat == 3).to_string().into());
    ui.set_nr_ok(mii(21_183 - total_dif).into());
    ui.set_rezolvate(numara(&|o| o.stare != 0) as f32 / total_dif as f32);
    ui.set_deployate(numara(&|o| o.deploy == 2) as f32 / total_dif as f32);
    ui.set_cautare("".into());
    ui.set_f_baza("Toate bazele".into());
    ui.set_f_tip("Toate tipurile".into());
    ui.set_f_categorie("Toate categoriile".into());
    ui.set_f_stare("Toate stările".into());
    ui.set_sim_min("0".into());
    ui.set_sim_max("100".into());
    ui.set_nr_deploy(numara(&|o| o.deploy == 1).to_string().into());
    ui.set_subsol(
        format!(
            "{total_dif} obiecte (din {}), sortate după similaritate",
            mii(21_183)
        )
        .into(),
    );

    // bula: din coordonatele mouse-ului, fara elemente pe patrat
    let ui_slab = ui.as_weak();
    let nume: Rc<Vec<Vec<String>>> = Rc::new(
        grupuri
            .iter()
            .map(|g| {
                g.obiecte
                    .iter()
                    .map(|&i| format!("{}{}", obiecte[i].schema, obiecte[i].nume))
                    .collect()
            })
            .collect(),
    );
    let poz = Rc::new(pozitii);
    ui.on_mouse(move |g, x, y| {
        let Some(ui) = ui_slab.upgrade() else { return };
        let i = ((x as u32 / PAS) * RANDURI_HARTA + y as u32 / PAS) as usize;
        let text = nume
            .get(g as usize)
            .and_then(|v| v.get(i))
            .cloned()
            .unwrap_or_default();
        let (gx, gy) = poz[g as usize];
        ui.set_bula(text.into());
        ui.set_bula_x(gx + x);
        ui.set_bula_y(cartus_y + gy + 22.0 + y);
    });
    ui.invoke_mouse(2, (2 * PAS + 7) as f32, (PAS + 7) as f32);

    ui.show().expect("fereastra nu se arata");
    let ms = captura(&f, cale);
    println!(
        "lista: {} obiecte in tabel, picturi {pictat_ms:.1} ms, primul cadru {ms:.1} ms -> {cale}",
        obiecte.len()
    );
}

// ------------------------------------------------------------- panza

fn panza(cale: &str, ghidat: bool, paleta: bool) {
    let f = fereastra();
    let ui = Panza::new().expect("fereastra");

    let t = Instant::now();
    let (def_s, def_b) = (definitie(SURSA, OBIECT), definitie(BAZA, OBIECT));
    let m: ModelEditor = model_editor::construieste(
        &gmotion_core::pregateste_afisare(&def_s),
        &gmotion_core::pregateste_afisare(&def_b),
    );
    let similar = gmotion_core::scor_similaritate(&def_s, &def_b);
    let afisat = model_editor::plieaza(&m, 3);
    let model_ms = t.elapsed().as_secs_f64() * 1000.0;

    // blocul fiecarui rand; primul e deja adus in baza, al doilea e cel curent
    let mut bloc_rand = vec![None; m.randuri.len()];
    let mut b: i32 = -1;
    for (i, r) in m.randuri.iter().enumerate() {
        if r.stare != Stare::Egal {
            if r.inceput_bloc || i == 0 || m.randuri[i - 1].stare == Stare::Egal {
                b += 1;
            }
            bloc_rand[i] = Some(b);
        }
    }
    let nr_blocuri = (b + 1).max(0) as usize;
    let (aplicat, curent) = (0, 1);

    let gol: ModelRc<BucataUi> = model_din(Vec::new());
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
                inceput: false,
                aplicat: false,
                curent: false,
            },
            RandAfisat::Linie(i) => {
                let rd = &m.randuri[*i];
                let bl = bloc_rand[*i];
                let e_aplicat = bl == Some(aplicat);
                let inceput = bl.is_some() && (*i == 0 || bloc_rand[*i - 1] != bl);
                LinieP {
                    // in sensul deploy-ului: Sters in model = doar in sursa
                    k: match rd.stare {
                        Stare::Egal => 0,
                        Stare::Modificat => 1,
                        Stare::Sters => 2,
                        Stare::Adaugat => 3,
                    },
                    pliu: "".into(),
                    nr_st: nr(&rd.stanga),
                    st: bucati(&rd.stanga),
                    // blocul adus: in baza apare textul din sursa
                    nr_dr: if e_aplicat {
                        nr(&rd.stanga)
                    } else {
                        nr(&rd.dreapta)
                    },
                    dr: if e_aplicat {
                        bucati(&rd.stanga)
                    } else {
                        bucati(&rd.dreapta)
                    },
                    inceput,
                    aplicat: e_aplicat,
                    curent: bl == Some(curent),
                }
            }
        })
        .collect();
    ui.set_linii(model_din(linii));

    // rigla: obiectul in 24 de felii
    let n = m.randuri.len().max(1);
    let mut rigla = vec![0; 24];
    for (i, bl) in bloc_rand.iter().enumerate() {
        if let Some(bl) = bl {
            rigla[i * 24 / n] = if *bl == aplicat {
                5
            } else if *bl == curent {
                4
            } else {
                match m.randuri[i].stare {
                    Stare::Modificat => 1,
                    Stare::Sters => 2,
                    Stare::Adaugat => 3,
                    Stare::Egal => 0,
                }
            };
        }
    }
    ui.set_rigla(model_din(rigla.into_iter().map(|k| SemnP { k }).collect()));

    let numara = |s: Stare| {
        let mut v: Vec<i32> = m
            .randuri
            .iter()
            .zip(&bloc_rand)
            .filter(|(r, _)| r.stare == s)
            .filter_map(|(_, b)| *b)
            .collect();
        v.dedup();
        v.len()
    };
    ui.set_sursa(SURSA.into());
    ui.set_tinta(BAZA.into());
    ui.set_mediu("TEST".into());
    ui.set_obiect_nav("17 / 18".into());
    ui.set_tip(format!("Procedură, comparată cu {BAZA}").into());
    ui.set_schema("dbo.".into());
    ui.set_nume(OBIECT.into());
    ui.set_fraza(
        format!(
            "{}% similar. {} modificate, {} de adus, {} de șters. Una e deja adusă.",
            format!("{similar:.2}").replace('.', ","),
            numara(Stare::Modificat),
            numara(Stare::Sters),
            numara(Stare::Adaugat)
        )
        .into(),
    );
    ui.set_salvat("Se aplică în timp real, salvat la 14:52".into());
    ui.set_diferenta_nav(format!("{} / {nr_blocuri}", curent + 1).into());
    ui.set_nerezolvate(format!("{} nerezolvate", nr_blocuri - 1).into());
    ui.set_revizuite("124".into());
    ui.set_din("din 336".into());
    ui.set_arc(arc(124.0 / 336.0).into());
    if ghidat {
        // inaltimea randurilor de dinaintea blocului curent (pliere 38 px, linie 24 px)
        let mut y = 0.0f32;
        for r in &afisat {
            match r {
                RandAfisat::Linie(i) if bloc_rand[*i] == Some(curent) => break,
                RandAfisat::Pliere { .. } => y += 34.0,
                RandAfisat::Linie(_) => y += 24.0,
            }
        }
        ui.set_derulare(-(y - 3.0 * 24.0).max(0.0));
        ui.set_ghidat(true);
        ui.set_ghidat_titlu(
            format!("Mod ghidat, diferența {} din {nr_blocuri}", curent + 1).into(),
        );
    }
    if paleta {
        let obiecte: Vec<String> = instantaneu()["baze"][SURSA]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|o| o["nume"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let mut comenzi: Vec<ComandaUi> = obiecte
            .into_iter()
            .filter(|n| n.contains("Case"))
            .take(4)
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
            ("Aplică toate", ""),
            ("Mod ghidat", "g"),
            ("Copiază rezultatul", ""),
            ("Deployer", "7 obiecte"),
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
        ui.set_comenzi(model_din(comenzi));
        ui.set_cautare("case".into());
        ui.set_paleta(true);
    }

    ui.show().expect("fereastra nu se arata");
    let ms = captura(&f, cale);
    println!(
        "panza: {} randuri, {nr_blocuri} blocuri, similar {similar}%, model {model_ms:.1} ms, primul cadru {ms:.1} ms -> {cale}",
        m.randuri.len()
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
    let are = |f: &str| a.iter().any(|x| x == f);
    match (a.get(1).map(String::as_str), a.get(2)) {
        (Some("conectare"), Some(cale)) => conectare(cale, are("--progres")),
        (Some("lista"), Some(cale)) => lista(cale),
        (Some("panza"), Some(cale)) => panza(cale, are("--ghidat"), are("--paleta")),
        _ => {
            eprintln!("folosire: schita-slint conectare|lista|panza iesire.png [--progres] [--ghidat] [--paleta]");
            std::process::exit(2);
        }
    }
}
