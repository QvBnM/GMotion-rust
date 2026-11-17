//! Interfata nativa GMotion.
//!
//! **Se strica astfel:** ce vede utilizatorul nu corespunde motorului (alt
//! aliniament, alte culori decat ce a decis `core`), sau fereastra nu se
//! deseneaza deloc pe un server fara GPU. Desenarea e prin Slint, cu randare
//! software (merge si prin RDP); modelele se construiesc aici, in Rust, si se
//! testeaza fara fereastra.
//!
//! Culorile si fonturile vin din `jetoane` (designul aprobat) si se pun in
//! globalul `Tema` cu [`aplica_tema`] — in `.slint` nu se scrie nicio culoare.

pub mod jetoane;
pub mod model_editor;

slint::include_modules!();

use model_editor::{Bucata, ModelEditor, RandAfisat, Sintaxa, Stare};
use slint::{Color, Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};

/// Mediul bazei tinta: da culoarea benzii de sus si a etichetei.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mediu {
    Dev,
    Test,
    Prod,
}

impl Mediu {
    fn cod(self) -> i32 {
        match self {
            Mediu::Dev => 0,
            Mediu::Test => 1,
            Mediu::Prod => 2,
        }
    }

    /// Eticheta in mono de langa numele bazei.
    pub fn eticheta(self) -> &'static str {
        match self {
            Mediu::Dev => "DEV",
            Mediu::Test => "TEST",
            Mediu::Prod => "PROD",
        }
    }
}

fn culoare(c: jetoane::Culoare) -> Color {
    Color::from_rgb_u8(
        ((c >> 16) & 0xff) as u8,
        ((c >> 8) & 0xff) as u8,
        (c & 0xff) as u8,
    )
}

/// Pune jetoanele aprobate in globalul `Tema`. De apelat o data, inainte de
/// orice afisare: in `.slint` culorile nu au valori implicite, tocmai ca sa nu
/// existe o a doua lista de culori pe care nu o verifica nimeni.
pub fn aplica_tema(ui: &EditorDiferente) {
    use jetoane as j;
    let t = ui.global::<Tema>();
    t.set_font_ui(j::FONT_UI.into());
    t.set_font_mono(j::FONT_MONO.into());
    t.set_fundal(culoare(j::FUNDAL));
    t.set_suprafata(culoare(j::SUPRAFATA));
    t.set_cod_fundal(culoare(j::COD_FUNDAL));
    t.set_linie(culoare(j::LINIE));
    t.set_linie_slaba(culoare(j::LINIE_SLABA));
    t.set_text(culoare(j::TEXT));
    t.set_text_secundar(culoare(j::TEXT_SECUNDAR));
    t.set_text_tert(culoare(j::TEXT_TERT));
    t.set_selectie(culoare(j::SELECTIE));
    t.set_mediu_prod(culoare(j::MEDIU_PROD));
    t.set_mediu_test(culoare(j::MEDIU_TEST));
    t.set_mediu_dev(culoare(j::MEDIU_DEV));
    t.set_mod_text(culoare(j::MOD_TEXT));
    t.set_mod_fundal(culoare(j::MOD_FUNDAL));
    t.set_mod_bara(culoare(j::MOD_BARA));
    t.set_mod_inlinie(culoare(j::MOD_INLINIE));
    t.set_add_text(culoare(j::ADD_TEXT));
    t.set_add_fundal(culoare(j::ADD_FUNDAL));
    t.set_add_bara(culoare(j::ADD_BARA));
    t.set_add_inlinie(culoare(j::ADD_INLINIE));
    t.set_del_text(culoare(j::DEL_TEXT));
    t.set_del_fundal(culoare(j::DEL_FUNDAL));
    t.set_del_bara(culoare(j::DEL_BARA));
    t.set_del_inlinie(culoare(j::DEL_INLINIE));
    t.set_hasura_a(culoare(j::HASURA_A));
    t.set_hasura_b(culoare(j::HASURA_B));
    t.set_s_cuvant(culoare(j::S_CUVANT));
    t.set_s_sir(culoare(j::S_SIR));
    t.set_s_numar(culoare(j::S_NUMAR));
    t.set_s_variabila(culoare(j::S_VARIABILA));
    t.set_s_comentariu(culoare(j::S_COMENTARIU));
    ui.set_tigla_hasura(tigla_hasura());
}

/// Tigla hasurii de pe partea fara linii: dungi la 135°, din jetoane.
pub fn tigla_hasura() -> Image {
    let (pas, pixeli) = tigla_pixeli();
    let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(pas, pas);
    buf.make_mut_slice().copy_from_slice(&pixeli);
    Image::from_rgba8(buf)
}

/// Pixelii tiglei, separat de Slint ca sa se poata verifica: dungile trebuie
/// sa se imbine cand tigla se repeta (de-aia pasul imparte latura exact).
fn tigla_pixeli() -> (u32, Vec<Rgba8Pixel>) {
    const PAS: u32 = 7;
    let p = |c: jetoane::Culoare| Rgba8Pixel {
        r: ((c >> 16) & 0xff) as u8,
        g: ((c >> 8) & 0xff) as u8,
        b: (c & 0xff) as u8,
        a: 255,
    };
    let mut v = Vec::with_capacity((PAS * PAS) as usize);
    for y in 0..PAS {
        for x in 0..PAS {
            v.push(if (x + y) % PAS == 0 {
                p(jetoane::HASURA_B)
            } else {
                p(jetoane::HASURA_A)
            });
        }
    }
    (PAS, v)
}

fn bucati_ui(b: &[Bucata]) -> ModelRc<BucataUi> {
    ModelRc::new(VecModel::from(
        b.iter()
            .map(|x| BucataUi {
                text: SharedString::from(x.text.as_str()),
                sintaxa: match x.sintaxa {
                    Sintaxa::Text => 0,
                    Sintaxa::CuvantCheie => 1,
                    Sintaxa::Variabila => 2,
                    Sintaxa::Numar => 3,
                    Sintaxa::Sir => 4,
                    Sintaxa::Comentariu => 5,
                },
                schimbat: x.schimbat,
            })
            .collect::<Vec<_>>(),
    ))
}

/// Codul de stare pentru `.slint`: 0 egal, 1 modificat, **2 doar in sursa**,
/// **3 doar in baza**.
///
/// Motorul numeste blocurile fata de referinta (`Sters` = e in sursa si
/// lipseste din baza). Editorul le arata insa in sensul deploy-ului, ca in
/// machete: ce e doar in sursa se va **adauga** in baza (verde), ce e doar in
/// baza se va **scoate** (rosu). De-aia cele doua se incruciseaza aici — intr-un
/// singur loc, nu in `.slint`.
fn cod_stare(s: Stare) -> i32 {
    match s {
        Stare::Egal => 0,
        Stare::Modificat => 1,
        Stare::Sters => 2,
        Stare::Adaugat => 3,
    }
}

/// Randurile de afisat si randul de inceput al fiecarei diferente, **in
/// coordonatele listei afisate** (plierea schimba indicii, iar navigarea
/// deruleaza lista, nu modelul).
fn randuri_ui(model: &ModelEditor, pliat: bool, unificat: bool) -> (Vec<RandUi>, Vec<i32>) {
    let afisate = if pliat {
        model_editor::plieaza(model, 3)
    } else {
        (0..model.randuri.len()).map(RandAfisat::Linie).collect()
    };
    let a_cata = model_editor::diferenta_fiecarui_rand(model);
    let gol = || ModelRc::new(VecModel::from(Vec::<BucataUi>::new()));
    let nr =
        |n: Option<usize>| n.map_or(SharedString::new(), |n| SharedString::from(n.to_string()));

    let mut randuri = Vec::with_capacity(afisate.len());
    let mut start = vec![-1i32; model.diferente.len()];
    for r in &afisate {
        match r {
            RandAfisat::Pliere { eticheta, .. } => randuri.push(RandUi {
                fel: 1,
                pliere: SharedString::from(eticheta.as_str()),
                st_nr: SharedString::new(),
                st: gol(),
                dr_nr: SharedString::new(),
                dr: gol(),
                stare: 0,
                diferenta: -1,
            }),
            &RandAfisat::Linie(i) => {
                let rand = &model.randuri[i];
                let k = a_cata[i];
                if let Some(k) = k {
                    if start[k] < 0 {
                        start[k] = randuri.len() as i32;
                    }
                }
                let baza = RandUi {
                    fel: 0,
                    pliere: SharedString::new(),
                    st_nr: nr(rand.stanga.nr),
                    st: bucati_ui(&rand.stanga.bucati),
                    dr_nr: nr(rand.dreapta.nr),
                    dr: bucati_ui(&rand.dreapta.bucati),
                    stare: cod_stare(rand.stare),
                    diferenta: k.map_or(-1, |k| k as i32),
                };
                if !unificat || rand.stare != Stare::Modificat {
                    randuri.push(baza);
                    continue;
                }
                // unificat: blocul modificat se vede de doua ori, intai cum e
                // in sursa, apoi cum e in baza
                if rand.stanga.nr.is_some() {
                    randuri.push(RandUi {
                        dr_nr: SharedString::new(),
                        dr: gol(),
                        ..baza.clone()
                    });
                }
                if rand.dreapta.nr.is_some() {
                    randuri.push(RandUi {
                        st_nr: SharedString::new(),
                        st: gol(),
                        ..baza
                    });
                }
            }
        }
    }
    (randuri, start)
}

/// Datele afisate in antetul editorului.
pub struct AntetEditor<'a> {
    pub schema: &'a str,
    pub nume: &'a str,
    /// „procedură", „vedere", „functie"...
    pub tip: &'a str,
    pub sursa_baza: &'a str,
    pub sursa_server: &'a str,
    pub tinta_baza: &'a str,
    pub tinta_server: &'a str,
    pub mediu: Mediu,
    /// „obiectul 4 din 212 nerevizuite"
    pub stare_obiecte: &'a str,
    pub revizuit: bool,
}

/// „1 diferență" / „4 diferențe" / „21 de diferențe".
fn numar_diferente(n: usize) -> String {
    match n {
        0 => "fără diferențe".to_string(),
        1 => "1 diferență".to_string(),
        2..=19 => format!("{n} diferențe"),
        _ => format!("{n} de diferențe"),
    }
}

/// „srv-dev · 47 linii".
fn detaliu_panou(server: &str, linii: usize) -> String {
    let l = if linii == 1 {
        "1 linie".to_string()
    } else if linii < 20 {
        format!("{linii} linii")
    } else {
        format!("{linii} de linii")
    };
    if server.is_empty() {
        l
    } else {
        format!("{server} · {l}")
    }
}

/// Umple editorul cu modelul unui obiect. Nu atinge jetoanele: `aplica_tema`
/// se cheama o data, la pornire.
pub fn incarca_editor(ui: &EditorDiferente, antet: &AntetEditor, model: &ModelEditor) {
    let (randuri, start) = randuri_ui(model, ui.get_pliaza_identicul(), !ui.get_alaturat());
    let total = randuri.len().max(1) as f32;
    let mut marcaje: Vec<MarcajUi> = Vec::new();
    for (k, &s) in start.iter().enumerate() {
        if s < 0 {
            continue;
        }
        let pana = start
            .get(k + 1)
            .copied()
            .filter(|&u| u > s)
            .unwrap_or(s + 1);
        let stare = model.randuri[model.diferente[k]].stare;
        marcaje.push(MarcajUi {
            pozitie: s as f32 / total,
            inaltime: ((pana - s).min(3) as f32) / total,
            stare: cod_stare(stare),
        });
    }
    let linii = |t: &ModelEditor, stanga: bool| {
        t.randuri
            .iter()
            .filter(|r| {
                if stanga {
                    r.stanga.nr.is_some()
                } else {
                    r.dreapta.nr.is_some()
                }
            })
            .count()
    };

    ui.set_obiect_schema(antet.schema.into());
    ui.set_obiect_nume(antet.nume.into());
    ui.set_tip_obiect(antet.tip.into());
    ui.set_rezumat(numar_diferente(model.diferente.len()).into());
    ui.set_sursa_baza(antet.sursa_baza.into());
    ui.set_sursa_detaliu(detaliu_panou(antet.sursa_server, linii(model, true)).into());
    ui.set_tinta_baza(antet.tinta_baza.into());
    ui.set_tinta_detaliu(detaliu_panou(antet.tinta_server, linii(model, false)).into());
    ui.set_mediu(antet.mediu.cod());
    ui.set_eticheta_mediu(antet.mediu.eticheta().into());
    ui.set_stare_obiecte(antet.stare_obiecte.into());
    ui.set_revizuit(antet.revizuit);
    ui.set_randuri(ModelRc::new(VecModel::from(randuri)));
    ui.set_marcaje(ModelRc::new(VecModel::from(marcaje)));
    ui.set_total_diferente(model.diferente.len() as i32);
    ui.set_start_diferente(ModelRc::new(VecModel::from(start)));
    ui.set_diferenta_curenta(0);
}

#[cfg(test)]
mod teste {
    use super::*;
    use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
    use slint::{ComponentHandle, Model};
    use std::rc::Rc;

    struct Platforma(Rc<MinimalSoftwareWindow>);
    impl slint::platform::Platform for Platforma {
        fn create_window_adapter(
            &self,
        ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
            Ok(self.0.clone())
        }
    }

    /// O fereastra fara ecran, pentru teste. Platforma Slint se tine pe fir
    /// (`thread_local`), iar `cargo test` da fiecarui test firul lui, deci
    /// nu e nevoie de lacat: a doua chemare pe acelasi fir doar esueaza, si
    /// atunci fereastra existenta e buna.
    fn fereastra() -> EditorDiferente {
        let w = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        let _ = slint::platform::set_platform(Box::new(Platforma(w)));
        let ui = EditorDiferente::new().expect("fereastra de test");
        aplica_tema(&ui);
        ui
    }

    #[test]
    fn aplica_tema_pune_fiecare_jeton_in_slint() {
        use jetoane as j;
        let ui = fereastra();
        let t = ui.global::<Tema>();
        let perechi: Vec<(&str, slint::Color, j::Culoare)> = vec![
            ("fundal", t.get_fundal(), j::FUNDAL),
            ("suprafata", t.get_suprafata(), j::SUPRAFATA),
            ("cod-fundal", t.get_cod_fundal(), j::COD_FUNDAL),
            ("linie", t.get_linie(), j::LINIE),
            ("linie-slaba", t.get_linie_slaba(), j::LINIE_SLABA),
            ("text", t.get_text(), j::TEXT),
            ("text-secundar", t.get_text_secundar(), j::TEXT_SECUNDAR),
            ("text-tert", t.get_text_tert(), j::TEXT_TERT),
            ("selectie", t.get_selectie(), j::SELECTIE),
            ("mediu-prod", t.get_mediu_prod(), j::MEDIU_PROD),
            ("mediu-test", t.get_mediu_test(), j::MEDIU_TEST),
            ("mediu-dev", t.get_mediu_dev(), j::MEDIU_DEV),
            ("mod-text", t.get_mod_text(), j::MOD_TEXT),
            ("mod-fundal", t.get_mod_fundal(), j::MOD_FUNDAL),
            ("mod-bara", t.get_mod_bara(), j::MOD_BARA),
            ("mod-inlinie", t.get_mod_inlinie(), j::MOD_INLINIE),
            ("add-text", t.get_add_text(), j::ADD_TEXT),
            ("add-fundal", t.get_add_fundal(), j::ADD_FUNDAL),
            ("add-bara", t.get_add_bara(), j::ADD_BARA),
            ("add-inlinie", t.get_add_inlinie(), j::ADD_INLINIE),
            ("del-text", t.get_del_text(), j::DEL_TEXT),
            ("del-fundal", t.get_del_fundal(), j::DEL_FUNDAL),
            ("del-bara", t.get_del_bara(), j::DEL_BARA),
            ("del-inlinie", t.get_del_inlinie(), j::DEL_INLINIE),
            ("hasura-a", t.get_hasura_a(), j::HASURA_A),
            ("hasura-b", t.get_hasura_b(), j::HASURA_B),
            ("s-cuvant", t.get_s_cuvant(), j::S_CUVANT),
            ("s-sir", t.get_s_sir(), j::S_SIR),
            ("s-numar", t.get_s_numar(), j::S_NUMAR),
            ("s-variabila", t.get_s_variabila(), j::S_VARIABILA),
            ("s-comentariu", t.get_s_comentariu(), j::S_COMENTARIU),
        ];
        for (nume, c, asteptat) in perechi {
            assert_eq!(c.alpha(), 255, "jetonul {nume} a ramas nepus (transparent)");
            assert_eq!(
                (c.red(), c.green(), c.blue()),
                (
                    ((asteptat >> 16) & 0xff) as u8,
                    ((asteptat >> 8) & 0xff) as u8,
                    (asteptat & 0xff) as u8
                ),
                "jetonul {nume}"
            );
        }
        assert_eq!(t.get_font_ui(), j::FONT_UI);
        assert_eq!(t.get_font_mono(), j::FONT_MONO);
    }

    #[test]
    fn tigla_hasurii_are_ambele_culori_si_se_imbina() {
        let (pas, px) = tigla_pixeli();
        assert_eq!(px.len(), (pas * pas) as usize);
        let dunga = |c: jetoane::Culoare| Rgba8Pixel {
            r: ((c >> 16) & 0xff) as u8,
            g: ((c >> 8) & 0xff) as u8,
            b: (c & 0xff) as u8,
            a: 255,
        };
        let (a, b) = (dunga(jetoane::HASURA_A), dunga(jetoane::HASURA_B));
        assert!(px.contains(&a) && px.contains(&b), "ambele culori");
        // cate un pixel de dunga pe fiecare rand si pe fiecare coloana: asta
        // face dunga continua cand tigla se repeta
        for y in 0..pas {
            let pe_rand = (0..pas).filter(|x| px[(y * pas + x) as usize] == b).count();
            assert_eq!(pe_rand, 1, "randul {y}");
        }
        for x in 0..pas {
            let pe_coloana = (0..pas).filter(|y| px[(y * pas + x) as usize] == b).count();
            assert_eq!(pe_coloana, 1, "coloana {x}");
        }
        assert_eq!(tigla_hasura().size().width, pas);
    }

    fn model_de_proba() -> model_editor::ModelEditor {
        let a: Vec<String> = (1..=40).map(|i| format!("SELECT {i}")).collect();
        let mut b = a.clone();
        b[20] = "SELECT 999".to_string();
        model_editor::construieste(&a.join("\n\n"), &b.join("\n\n"))
    }

    #[test]
    fn incarca_editor_umple_antetul_si_randurile() {
        let ui = fereastra();
        let model = model_de_proba();
        incarca_editor(
            &ui,
            &AntetEditor {
                schema: "dbo",
                nume: "usp_Test",
                tip: "procedură",
                sursa_baza: "Dev",
                sursa_server: "srv-dev",
                tinta_baza: "Prod",
                tinta_server: "srv-prod",
                mediu: Mediu::Prod,
                stare_obiecte: "obiectul 1 din 2 nerevizuite",
                revizuit: false,
            },
            &model,
        );
        assert_eq!(ui.get_obiect_schema(), "dbo");
        assert_eq!(ui.get_eticheta_mediu(), "PROD");
        assert_eq!(ui.get_mediu(), 2);
        assert_eq!(ui.get_rezumat(), "1 diferență");
        assert_eq!(ui.get_total_diferente(), 1);
        assert!(ui.get_randuri().row_count() > 0);
        assert_eq!(ui.get_marcaje().row_count(), 1);
        // randul de start al diferentei e in coordonatele listei afisate
        let start = ui.get_start_diferente().row_data(0).expect("start");
        assert!(start >= 0 && (start as usize) < ui.get_randuri().row_count());
        let r = ui
            .get_randuri()
            .row_data(start as usize)
            .expect("randul diferentei");
        assert_eq!(r.fel, 0);
        assert_eq!(r.stare, 1, "blocul e modificat");
        assert_eq!(r.diferenta, 0);
    }

    #[test]
    fn plierea_scurteaza_lista_si_muta_startul_diferentei() {
        let ui = fereastra();
        let model = model_de_proba();
        let antet = AntetEditor {
            schema: "dbo",
            nume: "usp_Test",
            tip: "procedură",
            sursa_baza: "Dev",
            sursa_server: "",
            tinta_baza: "Prod",
            tinta_server: "",
            mediu: Mediu::Test,
            stare_obiecte: "",
            revizuit: false,
        };
        ui.set_pliaza_identicul(false);
        incarca_editor(&ui, &antet, &model);
        let intreg = ui.get_randuri().row_count();
        let start_intreg = ui.get_start_diferente().row_data(0).expect("start");
        assert_eq!(intreg, model.randuri.len());

        ui.set_pliaza_identicul(true);
        incarca_editor(&ui, &antet, &model);
        let pliat = ui.get_randuri().row_count();
        let start_pliat = ui.get_start_diferente().row_data(0).expect("start");
        assert!(pliat < intreg, "{pliat} randuri pliate din {intreg}");
        assert!(
            start_pliat < start_intreg,
            "startul diferentei urmeaza lista afisata: {start_pliat} vs {start_intreg}"
        );
        let r = ui
            .get_randuri()
            .row_data(start_pliat as usize)
            .expect("rand");
        assert_eq!(r.stare, 1);
    }

    #[test]
    fn unificat_arata_blocul_modificat_de_doua_ori() {
        let ui = fereastra();
        let model = model_de_proba();
        let antet = AntetEditor {
            schema: "dbo",
            nume: "usp_Test",
            tip: "procedură",
            sursa_baza: "Dev",
            sursa_server: "",
            tinta_baza: "Prod",
            tinta_server: "",
            mediu: Mediu::Dev,
            stare_obiecte: "",
            revizuit: false,
        };
        ui.set_pliaza_identicul(false);
        ui.set_alaturat(true);
        incarca_editor(&ui, &antet, &model);
        let alaturat = ui.get_randuri().row_count();
        ui.set_alaturat(false);
        incarca_editor(&ui, &antet, &model);
        let unificat = ui.get_randuri().row_count();
        assert!(
            unificat > alaturat,
            "blocul modificat ocupa doua randuri: {unificat} vs {alaturat}"
        );
        // in unificat fiecare rand are o singura parte
        for i in 0..unificat {
            let r = ui.get_randuri().row_data(i).expect("rand");
            assert!(
                r.st_nr.is_empty() || r.dr_nr.is_empty() || r.stare == 0,
                "randul {i} are ambele parti intr-un bloc diferit"
            );
        }
    }

    #[test]
    fn numeralele_sunt_in_romana() {
        assert_eq!(numar_diferente(0), "fără diferențe");
        assert_eq!(numar_diferente(1), "1 diferență");
        assert_eq!(numar_diferente(4), "4 diferențe");
        assert_eq!(numar_diferente(21), "21 de diferențe");
        assert_eq!(detaliu_panou("srv-dev", 47), "srv-dev · 47 de linii");
        assert_eq!(detaliu_panou("", 1), "1 linie");
        assert_eq!(detaliu_panou("srv", 9), "srv · 9 linii");
    }

    #[test]
    fn starile_se_dau_in_sensul_deploy_ului() {
        // ce e doar in sursa se va adauga in baza: verde (2)
        assert_eq!(cod_stare(Stare::Sters), 2);
        // ce e doar in baza se va scoate: rosu (3)
        assert_eq!(cod_stare(Stare::Adaugat), 3);
        assert_eq!(cod_stare(Stare::Modificat), 1);
        assert_eq!(cod_stare(Stare::Egal), 0);
    }
}
