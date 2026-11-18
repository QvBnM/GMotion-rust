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

pub mod harta;
pub mod jetoane;
pub mod model_conectare;
pub mod model_deploy;
pub mod model_editor;
pub mod model_lista;

slint::include_modules!();

pub use jetoane::Mod;
use model_editor::{Bucata, Marcaj, ModelEditor, RandAfisat, Sintaxa, Stare};
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

/// Lista jetoanelor din `Tema`: campul din `Paleta`, setter-ul si getter-ul
/// Slint. O singura lista, din care se umple tema si se face testul care
/// prinde un jeton uitat (ar ramane transparent).
macro_rules! cu_jetoanele {
    ($m:ident) => {
        $m!(
        fundal set_fundal get_fundal,
        suprafata set_suprafata get_suprafata,
        strat set_strat get_strat,
        cod_fundal set_cod_fundal get_cod_fundal,
        panou_sursa set_panou_sursa get_panou_sursa,
        cap_sursa set_cap_sursa get_cap_sursa,
        cap_baza set_cap_baza get_cap_baza,
        contur_baza set_contur_baza get_contur_baza,
        linie set_linie get_linie,
        linie_slaba set_linie_slaba get_linie_slaba,
        text set_text get_text,
        text_secundar set_text_secundar get_text_secundar,
        text_tert set_text_tert get_text_tert,
        selectie set_selectie get_selectie,
        pe_selectie set_pe_selectie get_pe_selectie,
        accent set_accent get_accent,
        pe_accent set_pe_accent get_pe_accent,
        accent_text set_accent_text get_accent_text,
        tine set_tine get_tine,
        pe_tine set_pe_tine get_pe_tine,
        mediu_prod set_mediu_prod get_mediu_prod,
        mediu_test set_mediu_test get_mediu_test,
        mediu_dev set_mediu_dev get_mediu_dev,
        pe_mediu set_pe_mediu get_pe_mediu,
        mod_text set_mod_text get_mod_text,
        mod_fundal set_mod_fundal get_mod_fundal,
        mod_bara set_mod_bara get_mod_bara,
        mod_inlinie set_mod_inlinie get_mod_inlinie,
        add_text set_add_text get_add_text,
        add_fundal set_add_fundal get_add_fundal,
        add_bara set_add_bara get_add_bara,
        add_inlinie set_add_inlinie get_add_inlinie,
        del_text set_del_text get_del_text,
        del_fundal set_del_fundal get_del_fundal,
        del_bara set_del_bara get_del_bara,
        del_inlinie set_del_inlinie get_del_inlinie,
        set_bara set_set_bara get_set_bara,
        ident set_ident get_ident,
        hasura_a set_hasura_a get_hasura_a,
        hasura_b set_hasura_b get_hasura_b,
        s_cuvant set_s_cuvant get_s_cuvant,
        s_sir set_s_sir get_s_sir,
        s_numar set_s_numar get_s_numar,
        s_variabila set_s_variabila get_s_variabila,
        s_comentariu set_s_comentariu get_s_comentariu
        )
    };
}

/// Pune paleta modului ales in globalul `Tema`. Se cheama inainte de orice
/// afisare si din nou la fiecare comutare de mod: in `.slint` culorile nu au
/// valori implicite, tocmai ca sa nu existe o a doua lista de culori pe care
/// nu o verifica nimeni.
pub fn aplica_tema<'a, T>(ui: &'a T, m: Mod)
where
    T: slint::ComponentHandle,
    Tema<'a>: slint::Global<'a, T>,
{
    let t = ui.global::<Tema>();
    let p = m.paleta();
    t.set_font_ui(jetoane::FONT_UI.into());
    t.set_font_mono(jetoane::FONT_MONO.into());
    t.set_intunecat(m == Mod::Intunecat);
    macro_rules! pune {
        ($($camp:ident $set:ident $get:ident),* $(,)?) => {
            // destructurare fara `..`: un camp din `Paleta` lipsa din lista
            // de mai sus nu compileaza, deci nu poate ramane nepus in tema
            let jetoane::Paleta { $($camp),* } = *p;
            $( t.$set(culoare($camp)); )*
        };
    }
    cu_jetoanele!(pune);
}

/// Modul pus in `Tema` de ultimul [`aplica_tema`].
pub fn mod_curent<'a, T>(ui: &'a T) -> Mod
where
    T: slint::ComponentHandle,
    Tema<'a>: slint::Global<'a, T>,
{
    if ui.global::<Tema>().get_intunecat() {
        Mod::Intunecat
    } else {
        Mod::Luminos
    }
}

/// Tigla hasurii de pe partea fara linii: dungi la 135°, din paleta modului.
pub fn tigla_hasura(m: Mod) -> Image {
    let (pas, pixeli) = tigla_pixeli(m.paleta());
    let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(pas, pas);
    buf.make_mut_slice().copy_from_slice(&pixeli);
    Image::from_rgba8(buf)
}

/// Pixelii tiglei, separat de Slint ca sa se poata verifica: dungile trebuie
/// sa se imbine cand tigla se repeta (de-aia pasul imparte latura exact).
fn tigla_pixeli(paleta: &jetoane::Paleta) -> (u32, Vec<Rgba8Pixel>) {
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
                p(paleta.hasura_b)
            } else {
                p(paleta.hasura_a)
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
                dr_marcaj: 0,
                inceput: false,
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
                    dr_marcaj: match rand.marcaj {
                        Marcaj::Niciunul => 0,
                        Marcaj::Adus => 1,
                        Marcaj::ScrisDeTine => 2,
                    },
                    // primul rand al unei diferente, sau al unui sir de linii
                    // aduse: acolo sta sageata (sau bifa) din canal
                    inceput: rand.inceput_bloc
                        || (rand.marcaj != Marcaj::Niciunul
                            && (i == 0 || model.randuri[i - 1].marcaj == Marcaj::Niciunul)),
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
    /// „procedura", „vedere", „functie"...
    pub tip: &'a str,
    pub sursa_baza: &'a str,
    pub sursa_server: &'a str,
    pub tinta_baza: &'a str,
    pub tinta_server: &'a str,
    pub mediu: Mediu,
    /// „obiectul 4 din 212 nerevizuite"
    pub stare_obiecte: &'a str,
    pub revizuit: bool,
    /// Obiectul e in planul de deploy.
    pub in_plan: bool,
}

/// „1 diferenta" / „4 diferente" / „21 de diferente".
fn numar_diferente(n: usize) -> String {
    match n {
        0 => "fara diferente".to_string(),
        1 => "1 diferenta".to_string(),
        2..=19 => format!("{n} diferente"),
        _ => format!("{n} de diferente"),
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

    ui.set_tigla_hasura(tigla_hasura(mod_curent(ui)));
    ui.set_obiect_schema(antet.schema.into());
    ui.set_obiect_nume(antet.nume.into());
    ui.set_tip_obiect(antet.tip.into());
    ui.set_in_plan(antet.in_plan);
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

/// Starea editarii obiectului deschis: textul final, omis, modul ghidat.
/// Separat de [`AntetEditor`], care descrie obiectul, nu ce faci cu el.
pub struct StareEditare<'a> {
    pub omis: bool,
    /// Exista un text final (deci „Reset" are ce face).
    pub editat: bool,
    pub se_poate_anula: bool,
    /// Diferenta curenta, care ramane pe loc dupa „Aplica".
    pub diferenta: usize,
    /// Modul ghidat: `(din_sursa, aplicate, sarite, terminat)`.
    pub ghidat: Option<(bool, usize, usize, bool)>,
    /// Ce copiaza „Copiaza" si ce se editeaza de mana.
    pub text_sursa: &'a str,
    pub text_final: &'a str,
    /// „salvat la 14:52", sau gol.
    pub salvat: &'a str,
}

/// Pune starea editarii in editor. Se cheama dupa [`incarca_editor`].
pub fn incarca_editarea(ui: &EditorDiferente, e: &StareEditare) {
    ui.set_omis(e.omis);
    ui.set_editat(e.editat);
    ui.set_se_poate_anula(e.se_poate_anula);
    ui.set_text_sursa(e.text_sursa.into());
    ui.set_text_final(e.text_final.into());
    ui.set_salvat(e.salvat.into());
    let total = ui.get_total_diferente().max(0) as usize;
    match e.ghidat {
        Some((din_sursa, aplicate, sarite, terminat)) => {
            ui.set_ghidat(true);
            ui.set_ghidat_din_sursa(din_sursa);
            ui.set_ghidat_terminat(terminat);
            ui.set_ghidat_titlu(
                if terminat {
                    format!("Mod ghidat incheiat: {aplicate} aplicate, {sarite} lasate cum erau")
                } else {
                    format!("Mod ghidat, diferenta {} din {total}", e.diferenta + 1)
                }
                .into(),
            );
        }
        None => {
            ui.set_ghidat(false);
            ui.set_ghidat_terminat(false);
        }
    }
    if total > 0 {
        ui.invoke_mergi_la(e.diferenta.min(total - 1) as i32);
    }
}

/// Datele din antetul listei.
pub struct AntetLista<'a> {
    pub sursa_baza: &'a str,
    pub tinta_baza: &'a str,
    /// „srv-dev → srv-prod · comparat la 14:32"
    pub servere: &'a str,
    pub mediu: Mediu,
    /// Cate obiecte sunt in planul de deploy.
    pub in_plan: usize,
}

/// Coloana sortata, ca numar pentru `.slint` (`global Sortare`). Ordinea e
/// aceeasi cu cea din `model_lista::Ordine`; daca una se muta, se muta in
/// ambele locuri — testul `codurile_de_sortare_se_potrivesc` cade altfel.
fn cod_ordine(o: model_lista::Ordine) -> i32 {
    use model_lista::Ordine;
    match o {
        Ordine::Nume => 0,
        Ordine::Tip => 1,
        Ordine::Linii => 2,
        Ordine::Scor => 3,
        Ordine::DiferaIn => 4,
        Ordine::Revizuire => 5,
        Ordine::Deploy => 6,
        Ordine::Baza => 7,
    }
}

/// Inversul lui `cod_ordine`: ce coloana a apasat utilizatorul.
pub fn ordine_din_cod(c: i32) -> model_lista::Ordine {
    use model_lista::Ordine;
    match c {
        1 => Ordine::Tip,
        2 => Ordine::Linii,
        3 => Ordine::Scor,
        4 => Ordine::DiferaIn,
        5 => Ordine::Revizuire,
        6 => Ordine::Deploy,
        7 => Ordine::Baza,
        _ => Ordine::Nume,
    }
}

/// Coloana „Deploy" din v3.4: omis bate orice (obiectul e lasat deoparte
/// deliberat), apoi ce s-a trimis, apoi ce e pregatit.
fn cod_deploy(o: &model_lista::Obiect) -> i32 {
    use model_lista::Deploy;
    if o.omis {
        return 4;
    }
    match o.deploy {
        Deploy::Deployat => 2,
        Deploy::VersiuneNoua => 3,
        Deploy::Nu if o.in_plan => 1,
        Deploy::Nu => 0,
    }
}

/// Umple ecranul listei din modelul ei. Nu calculeaza nimic: filtrele,
/// contoarele si selectia sunt deja decise in `model_lista`.
pub fn incarca_lista(ui: &FereastraLista, antet: &AntetLista, lista: &model_lista::Lista) {
    use model_lista::Stare;
    let cod_stare = |s: Stare| match s {
        Stare::Modificat => 0,
        Stare::DoarSursa => 1,
        Stare::DoarBaza => 2,
        Stare::Identic => 3,
        Stare::Criptat => 4,
    };
    let randuri: Vec<RandListaUi> = lista
        .vizibile()
        .iter()
        .map(|&i| {
            let o = &lista.obiecte()[i];
            let (plus, minus) = match &o.detalii {
                Some(d) => (format!("+{}", d.linii_plus), format!("−{}", d.linii_minus)),
                None => (String::new(), String::new()),
            };
            RandListaUi {
                schema: o.schema.as_str().into(),
                nume: o.nume.as_str().into(),
                baza: o.baza.as_str().into(),
                tip: gmotion_core::nume_tip(&o.tip).into(),
                litera: o.stare.litera().into(),
                stare: cod_stare(o.stare),
                plus: plus.into(),
                minus: minus.into(),
                difera_in: o
                    .detalii
                    .as_ref()
                    .map_or("", |d| d.difera_in.as_str())
                    .into(),
                revizuit: o.revizuit,
                in_plan: o.in_plan,
                omis: o.omis,
                scor: o.scor.unwrap_or(0.0) as f32 / 100.0,
                scor_text: o.scor.map_or(String::new(), |x| format!("{x:.0}%")).into(),
                deploy: cod_deploy(o),
            }
        })
        .collect();
    let categorii: Vec<CategorieUi> = lista
        .contoare_stari()
        .into_iter()
        .map(|(s, n)| CategorieUi {
            litera: s.litera().into(),
            nume: s.categorie().into(),
            numar: n.to_string().into(),
            stare: cod_stare(s),
        })
        .collect();
    let tipuri: Vec<TipUi> = lista
        .contoare_tipuri()
        .into_iter()
        .map(|(t, n)| TipUi {
            nume: nume_tip_plural(&t).into(),
            numar: n.to_string().into(),
        })
        .collect();
    let (revizuite, total) = lista.progres_revizuire();

    ui.set_tigla_hasura(tigla_hasura(mod_curent(ui)));
    ui.set_sursa_baza(antet.sursa_baza.into());
    ui.set_tinta_baza(antet.tinta_baza.into());
    ui.set_servere(antet.servere.into());
    ui.set_mediu(antet.mediu.cod());
    ui.set_eticheta_mediu(antet.mediu.eticheta().into());
    ui.set_numar_deploy(antet.in_plan.to_string().into());
    ui.set_progres_revizuite(format!("{revizuite}/{total}").into());
    ui.set_progres_fractie(if total == 0 {
        0.0
    } else {
        revizuite as f32 / total as f32
    });
    ui.set_categorie_activa(
        lista
            .filtru()
            .stare
            .and_then(|s| model_lista::Stare::TOATE.iter().position(|&x| x == s))
            .map_or(-1, |i| i as i32),
    );
    ui.set_tip_activ(
        lista
            .filtru()
            .tip
            .as_deref()
            .and_then(|t| lista.contoare_tipuri().iter().position(|(x, _)| x == t))
            .map_or(-1, |i| i as i32),
    );
    let baze: Vec<(String, usize)> = lista.contoare_baze();
    ui.set_baze(ModelRc::new(VecModel::from(
        baze.iter()
            .map(|(b, n)| TipUi {
                nume: b.as_str().into(),
                numar: n.to_string().into(),
            })
            .collect::<Vec<_>>(),
    )));
    ui.set_baza_activa(
        lista
            .filtru()
            .baza
            .as_deref()
            .and_then(|b| baze.iter().position(|(x, _)| x == b))
            .map_or(-1, |i| i as i32),
    );
    ui.set_nume_baze(ModelRc::new(VecModel::from(
        baze.iter()
            .map(|(b, _)| SharedString::from(b.as_str()))
            .collect::<Vec<_>>(),
    )));
    ui.set_mai_multe_baze(baze.len() > 1);
    ui.set_rezumat_lista(lista.rezumat_vizibile().into());
    ui.set_coloana_sortata(cod_ordine(lista.ordine()));
    ui.set_sens_descrescator(lista.descrescator());
    ui.set_stari(ModelRc::new(VecModel::from(
        model_lista::FiltruRevizuire::TOATE
            .iter()
            .map(|f| SharedString::from(f.eticheta()))
            .collect::<Vec<_>>(),
    )));
    ui.set_stare_activa(lista.filtru().revizuire.map_or(-1, |f| {
        model_lista::FiltruRevizuire::TOATE
            .iter()
            .position(|&x| x == f)
            .map_or(-1, |i| i as i32)
    }));
    ui.set_scor_min(format!("{:.0}", lista.filtru().scor_min).into());
    ui.set_scor_max(format!("{:.0}", lista.filtru().scor_max).into());
    let (deployate, total_deploy) = lista.progres_deploy();
    ui.set_progres_deployate(format!("{deployate}/{total_deploy}").into());
    ui.set_progres_deploy(if total_deploy == 0 {
        0.0
    } else {
        deployate as f32 / total_deploy as f32
    });
    ui.set_selectat(lista.selectat().map_or(0, |p| p as i32));
    incarca_harta(ui, &randuri, lista.selectat());
    ui.set_randuri(ModelRc::new(VecModel::from(randuri)));
    ui.set_categorii(ModelRc::new(VecModel::from(categorii)));
    ui.set_tipuri(ModelRc::new(VecModel::from(tipuri)));

    match lista.obiect_selectat() {
        Some(o) => {
            ui.set_obiect_schema(o.schema.as_str().into());
            ui.set_obiect_nume(o.nume.as_str().into());
            ui.set_obiect_revizuit(o.revizuit);
            ui.set_obiect_in_plan(o.in_plan);
            ui.set_ce_difera(
                o.detalii
                    .as_ref()
                    .map_or(String::new(), |d| format!("difera in {}", d.difera_in))
                    .into(),
            );
        }
        None => {
            ui.set_obiect_schema(SharedString::new());
            ui.set_obiect_nume(SharedString::new());
            ui.set_ce_difera(SharedString::new());
            ui.set_obiect_revizuit(false);
        }
    }
}

/// Numele tipului la plural, pentru bara laterala („Proceduri"), ca in
/// machete. Singularul vine din `gmotion_core::nume_tip`.
fn nume_tip_plural(tip: &str) -> &'static str {
    match gmotion_core::nume_tip(tip) {
        "procedura" => "Proceduri",
        "view" => "View-uri",
        "functie" => "Functii",
        "trigger" => "Triggere",
        _ => "Alte obiecte",
    }
}

/// Pune in previzualizare diferenta obiectului selectat: aceleasi randuri ca
/// in editor (un singur motor), pliate, plus propozitia care spune ce s-a
/// schimbat („un singur caracter, intr-un sir: Nu → nu").
///
/// Textele sunt cele pregatite pentru afisare (`pregateste_afisare`).
pub fn incarca_previzualizare(
    ui: &FereastraLista,
    text_sursa: &str,
    text_baza: &str,
    setari: &[&str],
) {
    let model = model_editor::construieste(text_sursa, text_baza);
    let (randuri, _) = randuri_ui(&model, true, false);
    ui.set_previzualizare(ModelRc::new(VecModel::from(randuri)));
    ui.set_ce_difera(gmotion_core::clasificare::propozitie(text_sursa, text_baza, setari).into());
}

/// Goleste previzualizarea (niciun obiect selectat).
pub fn goleste_previzualizarea(ui: &FereastraLista) {
    ui.set_previzualizare(ModelRc::new(VecModel::from(Vec::<RandUi>::new())));
}

/// Ce scrie in campul bazelor cand se pot bifa mai multe: numele, daca sunt
/// una sau doua, altfel „N baze" — un sir lung ar iesi din camp oricum.
fn eticheta_baze(p: &model_conectare::Parte) -> String {
    let alese = p.baze_alese();
    match alese.len() {
        0 => String::new(),
        1 | 2 => alese.join(", "),
        n => format!("{n} baze"),
    }
}

/// Umple ecranul de conectare din modelul lui.
/// Umple ecranul de deploy. Numarul mare si propozitia vin din pasi
/// (`model_deploy::rezumat`), ca sa nu poata spune altceva decat lista.
pub fn incarca_deploy(ui: &FereastraDeploy, e: &model_deploy::EcranDeploy, mediu: Mediu) {
    use model_deploy::{GardaDeploy, RezultatDeploy};
    ui.set_referinta(e.referinta.as_str().into());
    ui.set_tinta(e.tinta.as_str().into());
    ui.set_mediu(mediu.cod());
    ui.set_numar(e.pasi.len().to_string().into());
    ui.set_rezumat(model_deploy::rezumat(&e.pasi).into());
    ui.set_mai_multe_baze(e.baze.len() > 1);
    let pasi: Vec<PasDeployUi> = e
        .pasi
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let (schema, nume) = p.obiect.split_once('.').unwrap_or(("dbo", &p.obiect));
            PasDeployUi {
                nr: (i + 1).to_string().into(),
                creeaza: p.creeaza,
                schema: schema.into(),
                nume: nume.into(),
                tip: p.tip.as_str().into(),
                id: p.obiect.as_str().into(),
                baza: p.baza.as_str().into(),
            }
        })
        .collect();
    ui.set_pasi(ModelRc::new(VecModel::from(pasi)));
    ui.set_sarite(
        match e.sarite {
            0 => String::new(),
            1 => "Un obiect ales nu intra: exista doar in baza sau e criptat.".into(),
            n => format!("{n} obiecte alese nu intra: exista doar in baza sau sunt criptate."),
        }
        .into(),
    );
    let (fel, schimbari) = match &e.garda {
        GardaDeploy::Neverificata => (0, Vec::new()),
        GardaDeploy::Ok { .. } => (1, Vec::new()),
        GardaDeploy::Schimbata(v) => (2, v.iter().take(6).cloned().collect()),
        GardaDeploy::Eroare(_) => (3, Vec::new()),
    };
    ui.set_garda(fel);
    ui.set_text_garda(model_deploy::text_garda(&e.garda, &e.tinta).into());
    ui.set_schimbari(ModelRc::new(VecModel::from(
        schimbari
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>(),
    )));
    ui.set_impartire(e.impartire);
    ui.set_cate(e.cate.to_string().into());
    ui.set_director(e.director.as_str().into());
    ui.set_poate_scrie(e.poate_scrie);
    ui.set_rezultat(match e.rezultat {
        RezultatDeploy::Nescris => 0,
        RezultatDeploy::Scris { .. } => 1,
        RezultatDeploy::Eroare(_) => 2,
    });
    ui.set_text_rezultat(model_deploy::text_rezultat(&e.rezultat).into());
}

/// Cat de lata e harta din lista, in pixeli logici: fereastra de 1440 minus
/// marginile cartusului. Pe o fereastra mai ingusta, ultimul grup trece pe
/// randul urmator mai devreme; nu iese din cartus decat pe orizontala.
pub const LATIME_HARTA: f32 = 1372.0;

/// Harta de deasupra tabelului: obiectele afisate, pe tipuri, pictate in
/// culorile modului curent; conturul pe patratul randului selectat.
fn incarca_harta(ui: &FereastraLista, randuri: &[RandListaUi], selectat: Option<usize>) {
    // grupurile, in ordinea tipurilor dupa cate obiecte au
    let mut pe_tip: Vec<(String, Vec<harta::Celula>)> = Vec::new();
    for (i, r) in randuri.iter().enumerate() {
        let c = harta::Celula {
            rand: i,
            stare: r.stare,
            revizuit: r.revizuit,
        };
        match pe_tip
            .iter_mut()
            .find(|(t, _)| t.as_str() == r.tip.as_str())
        {
            Some((_, v)) => v.push(c),
            None => pe_tip.push((r.tip.to_string(), vec![c])),
        }
    }
    pe_tip.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
    let (grupuri, inaltime) = harta::aseaza(pe_tip, LATIME_HARTA);
    let paleta = mod_curent(ui).paleta();
    let ui_grupuri: Vec<GrupHartaUi> = grupuri
        .iter()
        .map(|g| {
            let (w, h, px) = harta::picteaza(g, paleta);
            let mut buf = SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
            for (d, s) in buf.make_mut_slice().iter_mut().zip(&px) {
                *d = slint::Rgba8Pixel {
                    r: s[0],
                    g: s[1],
                    b: s[2],
                    a: s[3],
                };
            }
            GrupHartaUi {
                titlu: g.titlu.as_str().into(),
                numar: g.celule.len().to_string().into(),
                x: g.x,
                y: g.y,
                latime: g.latime,
                imagine: Image::from_rgba8(buf),
                randuri: ModelRc::new(VecModel::from(
                    g.celule.iter().map(|c| c.rand as i32).collect::<Vec<_>>(),
                )),
            }
        })
        .collect();
    let sel = selectat.and_then(|s| harta::pozitie_randului(&grupuri, s));
    ui.set_sel_pe_harta(sel.is_some());
    if let Some((x, y)) = sel {
        ui.set_sel_x(x);
        ui.set_sel_y(y);
    }
    ui.set_harta(ModelRc::new(VecModel::from(ui_grupuri)));
    ui.set_harta_inaltime(inaltime);
}

/// Randul din tabel al patratului de la (x, y) din grupul `grup` al hartii,
/// in coordonatele imaginii grupului. Calculul e in `harta::indice_la`.
pub fn rand_din_harta(ui: &FereastraLista, grup: i32, x: f32, y: f32) -> Option<usize> {
    use slint::Model;
    let g = ui.get_harta().row_data(usize::try_from(grup).ok()?)?;
    let k = harta::indice_la(x, y)?;
    usize::try_from(g.randuri.row_data(k)?).ok()
}

/// Leaga bula de pe harta: numele patratului de sub mouse. Nu tine de starea
/// aplicatiei, deci se leaga aici, o data, dupa crearea ferestrei.
pub fn leaga_harta(ui: &FereastraLista) {
    let slab = ui.as_weak();
    ui.on_peste_harta(move |grup, x, y| {
        use slint::Model;
        let Some(ui) = slab.upgrade() else { return };
        let r = rand_din_harta(&ui, grup, x, y).and_then(|i| ui.get_randuri().row_data(i));
        let Some(r) = r else {
            ui.set_bula("".into());
            return;
        };
        let Some(g) = usize::try_from(grup)
            .ok()
            .and_then(|i| ui.get_harta().row_data(i))
        else {
            return;
        };
        // originea zonei hartii in fereastra: antetul (60) + filtrele (56) +
        // marginea cartusului (14) pe verticala, 16 + 18 pe orizontala
        ui.set_bula(format!("{}.{}", r.schema, r.nume).into());
        ui.set_bula_x(34.0 + g.x + x);
        ui.set_bula_y(130.0 + g.y + 20.0 + y);
    });
}

/// Cate linii de jurnal arata banda de jos a ecranului de conectare.
pub const LINII_JURNAL: usize = 4;

/// O linie din jurnalul aplicatiei (`[NIVEL] mesaj camp=valoare`), pentru
/// banda de jos: nivelul devine culoare (0 info, 2 atentie, 3 eroare), iar
/// prefixul dispare din text.
pub fn linie_jurnal(linie: &str) -> (i32, String) {
    // doar nivelurile lui `tracing` sunt prefix; `[GM-DB-001]` e parte din mesaj
    for (prefix, nivel) in [
        ("[ERROR] ", 3),
        ("[WARN] ", 2),
        ("[INFO] ", 0),
        ("[DEBUG] ", 0),
        ("[TRACE] ", 0),
    ] {
        if let Some(rest) = linie.strip_prefix(prefix) {
            return (nivel, rest.to_string());
        }
    }
    (0, linie.to_string())
}

/// Pune ultimele [`LINII_JURNAL`] linii din jurnal in banda de jos.
pub fn incarca_jurnal(ui: &FereastraConectare, linii: &[String]) {
    let de_la = linii.len().saturating_sub(LINII_JURNAL);
    let v: Vec<LinieJurnalUi> = linii[de_la..]
        .iter()
        .map(|l| {
            let (nivel, text) = linie_jurnal(l);
            LinieJurnalUi {
                nivel,
                text: text.into(),
            }
        })
        .collect();
    ui.set_jurnal(ModelRc::new(VecModel::from(v)));
}

pub fn incarca_conectare(ui: &FereastraConectare, c: &model_conectare::Conectare) {
    use model_conectare::StareConexiune;
    let fel = |s: &StareConexiune| match s {
        StareConexiune::Neincercat => 0,
        StareConexiune::SeConecteaza => 1,
        StareConexiune::Conectat(_) => 2,
        StareConexiune::Eroare(_) => 3,
    };
    let parte = |p: &model_conectare::Parte, eticheta: &str, rol: &str, mediu: Mediu| ParteUi {
        eticheta: eticheta.into(),
        rol: rol.into(),
        mediu: mediu.eticheta().into(),
        cod_mediu: mediu.cod(),
        server: p.server.as_str().into(),
        auth_windows: p.auth_windows,
        utilizator: p.acreditari.utilizator.as_str().into(),
        // campul de parola arata doar cat de lunga e, niciodata continutul
        parola: "•"
            .repeat(if p.acreditari.are_parola() { 10 } else { 0 })
            .into(),
        baza: p.baza.as_str().into(),
        baze: ModelRc::new(VecModel::from(
            p.baze
                .iter()
                .map(|b| SharedString::from(b.as_str()))
                .collect::<Vec<_>>(),
        )),
        alese: ModelRc::new(VecModel::from(
            p.baze.iter().map(|b| p.e_aleasa(b)).collect::<Vec<bool>>(),
        )),
        eticheta_baze: eticheta_baze(p).into(),
        stare: p.stare.text().into(),
        fel_stare: fel(&p.stare),
        servere_gasite: ModelRc::new(VecModel::from(
            p.servere_gasite
                .iter()
                .map(|s| SharedString::from(s.as_str()))
                .collect::<Vec<_>>(),
        )),
    };
    let recente: Vec<RecentaUi> = c
        .recente
        .iter()
        .map(|r| RecentaUi {
            sursa: r.sursa.as_str().into(),
            baza: r.baza.as_str().into(),
            mediu: r.mediu.as_str().into(),
            cod_mediu: mediu_dupa_nume(&r.baza).cod(),
            cand: r.cand.as_str().into(),
            progres: r.progres.as_str().into(),
        })
        .collect();
    let mediu_tinta = mediu_dupa_nume(&c.baza.baza);
    ui.set_sursa(parte(
        &c.sursa,
        "Sursa",
        "de unde vine codul",
        mediu_dupa_nume(&c.sursa.baza),
    ));
    ui.set_baza(parte(&c.baza, "Baza", "unde ajunge", mediu_tinta));
    ui.set_mediu(mediu_tinta.cod());
    ui.set_certificat(c.text_certificat().into());
    ui.set_instantaneu(c.instantaneu.as_deref().unwrap_or("").into());
    ui.set_recente(ModelRc::new(VecModel::from(recente)));
    ui.set_ce_lipseste(c.ce_lipseste().unwrap_or_default().into());
}

/// Mediul ghicit din numele bazei: `...Prod` e productie, `...Test` e test,
/// restul dev. Banda de sus si eticheta nu au alta sursa de adevar pana cand
/// mediul nu se configureaza explicit (decizie de luat cu Mihai).
pub fn mediu_dupa_nume(baza: &str) -> Mediu {
    let b = baza.to_lowercase();
    if b.contains("prod") {
        Mediu::Prod
    } else if b.contains("test") || b.contains("uat") {
        Mediu::Test
    } else {
        Mediu::Dev
    }
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
        aplica_tema(&ui, Mod::Luminos);
        ui
    }

    /// Apasa o tasta in fereastra, ca un utilizator (apasat si eliberat).
    fn apasa(ui: &EditorDiferente, tasta: &str) {
        use slint::platform::WindowEvent;
        let t: SharedString = tasta.into();
        ui.window()
            .dispatch_event(WindowEvent::KeyPressed { text: t.clone() });
        ui.window()
            .dispatch_event(WindowEvent::KeyReleased { text: t });
    }

    /// Tastele editorului ajung la callback-urile lor: aceleasi pe care le
    /// cheama butoanele. Un buton sau o tasta fara efect e exact ce a trecut
    /// neobservat prin etapele 8-11.
    #[test]
    fn tastele_editorului_cheama_actiunile() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let ui = fereastra();
        ui.show().expect("fereastra se arata");
        let apeluri: Rc<RefCell<Vec<&'static str>>> = Rc::default();
        macro_rules! inregistreaza {
            ($($on:ident => $nume:literal),* $(,)?) => {
                $( let a = apeluri.clone(); ui.$on(move || a.borrow_mut().push($nume)); )*
            };
        }
        inregistreaza!(
            on_comuta_omis => "omite",
            on_ghidat_porneste => "ghidat",
            on_ghidat_opreste => "opreste",
            on_ghidat_aplica => "aplica",
            on_ghidat_sari => "sari",
            on_anuleaza => "anuleaza",
            on_editeaza_textul_final => "editeaza",
            on_inapoi_lista => "lista",
        );
        apasa(&ui, "o");
        apasa(&ui, "g");
        apasa(&ui, "e");
        assert!(ui.get_editeaza_mod(), "e deschide editarea de mana");
        ui.set_editeaza_mod(false);
        ui.invoke_focus_taste();
        // Ctrl+Z: Control tinut apasat, apoi z
        use slint::platform::{Key, WindowEvent};
        ui.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Control.into(),
        });
        apasa(&ui, "z");
        ui.window().dispatch_event(WindowEvent::KeyReleased {
            text: Key::Control.into(),
        });
        // in modul ghidat, Enter aplica si Esc sare (nu te intoarce in lista)
        ui.set_ghidat(true);
        apasa(&ui, &SharedString::from(Key::Return));
        apasa(&ui, &SharedString::from(Key::Escape));
        // la final, Enter inchide
        ui.set_ghidat_terminat(true);
        apasa(&ui, &SharedString::from(Key::Return));
        // in afara modului ghidat, Esc e lista
        ui.set_ghidat(false);
        apasa(&ui, &SharedString::from(Key::Escape));
        assert_eq!(
            *apeluri.borrow(),
            ["omite", "ghidat", "editeaza", "anuleaza", "aplica", "sari", "opreste", "lista"]
        );
    }

    /// Capetele de coloana, filtrul pe stare, intervalul de similaritate si
    /// Export CSV cheama fiecare actiunea lui. Un buton desenat si nelegat
    /// arata perfect in captura — asa au trecut etapele 8-11 cu 7 butoane
    /// moarte.
    #[test]
    fn butoanele_listei_cheama_actiunile() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let w = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        let _ = slint::platform::set_platform(Box::new(Platforma(w)));
        let ui = FereastraLista::new().expect("fereastra de test");
        aplica_tema(&ui, Mod::Luminos);
        let apeluri: Rc<RefCell<Vec<String>>> = Rc::default();
        let a = apeluri.clone();
        ui.on_sorteaza_coloana(move |c| a.borrow_mut().push(format!("sort{c}")));
        let a = apeluri.clone();
        ui.on_alege_starea(move |i| a.borrow_mut().push(format!("stare{i}")));
        let a = apeluri.clone();
        ui.on_schimba_scorul(move |min, max| a.borrow_mut().push(format!("scor{min}-{max}")));
        let a = apeluri.clone();
        ui.on_exporta_csv(move || a.borrow_mut().push("csv".into()));
        let a = apeluri.clone();
        ui.on_cauta(move |t| a.borrow_mut().push(format!("cauta{t}")));
        let a = apeluri.clone();
        ui.on_pune_vizibilele_in_plan(move |cu| a.borrow_mut().push(format!("plan{cu}")));

        ui.invoke_sorteaza_coloana(cod_ordine(model_lista::Ordine::Scor));
        ui.invoke_alege_starea(2);
        ui.invoke_schimba_scorul("40".into(), "90".into());
        ui.invoke_exporta_csv();
        ui.invoke_pune_vizibilele_in_plan(true);
        assert_eq!(
            *apeluri.borrow(),
            ["sort3", "stare2", "scor40-90", "csv", "plantrue"],
            "fiecare element nou din bara de filtre are actiune"
        );

        // Ctrl+F duce cursorul in cautare: proba e ca ce tastezi dupa aia
        // ajunge acolo, nu ca scurtatura „a fost apasata"
        ui.show().expect("fereastra se arata");
        use slint::platform::{Key, WindowEvent};
        let tasta = |t: SharedString| {
            ui.window()
                .dispatch_event(WindowEvent::KeyPressed { text: t.clone() });
            ui.window()
                .dispatch_event(WindowEvent::KeyReleased { text: t });
        };
        ui.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Control.into(),
        });
        tasta("f".into());
        ui.window().dispatch_event(WindowEvent::KeyReleased {
            text: Key::Control.into(),
        });
        tasta("p".into());
        assert_eq!(ui.get_cautare(), "p", "textul merge in campul de cautare");
        assert!(
            apeluri.borrow().iter().any(|a| a == "cautap"),
            "si anunta modelul: {:?}",
            apeluri.borrow()
        );
    }

    /// Codurile de sortare din `.slint` (`global Sortare`) si `Ordine` sunt
    /// aceeasi lista. Daca una se muta si cealalta nu, clicul pe „Tip" ar
    /// sorta dupa linii, fara ca nimic sa cada.
    #[test]
    fn codurile_de_sortare_se_potrivesc() {
        use model_lista::Ordine;
        for o in [
            Ordine::Nume,
            Ordine::Tip,
            Ordine::Linii,
            Ordine::Scor,
            Ordine::DiferaIn,
            Ordine::Revizuire,
            Ordine::Deploy,
        ] {
            assert_eq!(ordine_din_cod(cod_ordine(o)), o, "dus-intors pentru {o:?}");
        }
        // un cod necunoscut nu da panica: cade pe sortarea implicita
        assert_eq!(ordine_din_cod(99), Ordine::Nume);
    }

    #[test]
    fn starea_editarii_ajunge_in_fereastra() {
        let ui = fereastra();
        let model = model_editor::construieste(
            "SELECT 1\n\nUPDATE t SET a = 1",
            "SELECT 1\n\nUPDATE t SET a = 2",
        );
        incarca_editor(
            &ui,
            &AntetEditor {
                schema: "dbo",
                nume: "p",
                tip: "procedura",
                sursa_baza: "A",
                sursa_server: "",
                tinta_baza: "B",
                tinta_server: "",
                mediu: Mediu::Dev,
                stare_obiecte: "",
                revizuit: false,
                in_plan: false,
            },
            &model,
        );
        incarca_editarea(
            &ui,
            &StareEditare {
                omis: true,
                editat: true,
                se_poate_anula: true,
                diferenta: 0,
                ghidat: Some((true, 0, 0, false)),
                text_sursa: "sursa",
                text_final: "final",
                salvat: "salvat la 14:52",
            },
        );
        assert!(ui.get_omis() && ui.get_editat() && ui.get_se_poate_anula() && ui.get_ghidat());
        assert_eq!(ui.get_ghidat_titlu(), "Mod ghidat, diferenta 1 din 1");
        assert_eq!(ui.get_text_final(), "final");
        incarca_editarea(
            &ui,
            &StareEditare {
                omis: false,
                editat: false,
                se_poate_anula: false,
                diferenta: 0,
                ghidat: Some((true, 1, 0, true)),
                text_sursa: "",
                text_final: "",
                salvat: "",
            },
        );
        assert!(ui.get_ghidat_terminat());
        assert!(
            ui.get_ghidat_titlu().contains("1 aplicate"),
            "{}",
            ui.get_ghidat_titlu()
        );
    }

    #[test]
    fn aplica_tema_pune_fiecare_jeton_in_slint_in_ambele_moduri() {
        let ui = fereastra();
        let t = ui.global::<Tema>();
        for m in [Mod::Intunecat, Mod::Luminos] {
            aplica_tema(&ui, m);
            let p = m.paleta();
            assert_eq!(mod_curent(&ui), m);
            macro_rules! verifica {
                ($($camp:ident $set:ident $get:ident),* $(,)?) => {
                    $(
                        let c = t.$get();
                        let nume = stringify!($camp);
                        assert_eq!(c.alpha(), 255, "{m:?}: jetonul {nume} a ramas nepus (transparent)");
                        assert_eq!(c, culoare(p.$camp), "{m:?}: jetonul {nume}");
                    )*
                };
            }
            cu_jetoanele!(verifica);
            assert_eq!(t.get_font_ui(), jetoane::FONT_UI);
            assert_eq!(t.get_font_mono(), jetoane::FONT_MONO);
        }
    }

    #[test]
    fn tigla_hasurii_are_ambele_culori_si_se_imbina() {
        let (pas, px) = tigla_pixeli(&jetoane::LUMINOS);
        assert_eq!(px.len(), (pas * pas) as usize);
        let dunga = |c: jetoane::Culoare| Rgba8Pixel {
            r: ((c >> 16) & 0xff) as u8,
            g: ((c >> 8) & 0xff) as u8,
            b: (c & 0xff) as u8,
            a: 255,
        };
        let (a, b) = (
            dunga(jetoane::LUMINOS.hasura_a),
            dunga(jetoane::LUMINOS.hasura_b),
        );
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
        assert_eq!(tigla_hasura(Mod::Luminos).size().width, pas);
        // hasura urmeaza modul: pe fond intunecat, dungile sunt intunecate
        let (_, inchis) = tigla_pixeli(&jetoane::INTUNECAT);
        assert!(
            !inchis.contains(&a),
            "tigla intunecata are culori din paleta luminoasa"
        );
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
                tip: "procedura",
                sursa_baza: "Dev",
                sursa_server: "srv-dev",
                tinta_baza: "Prod",
                tinta_server: "srv-prod",
                mediu: Mediu::Prod,
                stare_obiecte: "obiectul 1 din 2 nerevizuite",
                revizuit: false,
                in_plan: false,
            },
            &model,
        );
        assert_eq!(ui.get_obiect_schema(), "dbo");
        assert_eq!(ui.get_eticheta_mediu(), "PROD");
        assert_eq!(ui.get_mediu(), 2);
        assert_eq!(ui.get_rezumat(), "1 diferenta");
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
            tip: "procedura",
            sursa_baza: "Dev",
            sursa_server: "",
            tinta_baza: "Prod",
            tinta_server: "",
            mediu: Mediu::Test,
            stare_obiecte: "",
            revizuit: false,
            in_plan: false,
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
            tip: "procedura",
            sursa_baza: "Dev",
            sursa_server: "",
            tinta_baza: "Prod",
            tinta_server: "",
            mediu: Mediu::Dev,
            stare_obiecte: "",
            revizuit: false,
            in_plan: false,
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
        assert_eq!(numar_diferente(0), "fara diferente");
        assert_eq!(numar_diferente(1), "1 diferenta");
        assert_eq!(numar_diferente(4), "4 diferente");
        assert_eq!(numar_diferente(21), "21 de diferente");
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

    fn fereastra_lista() -> FereastraLista {
        let w = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        let _ = slint::platform::set_platform(Box::new(Platforma(w)));
        let ui = FereastraLista::new().expect("fereastra de test");
        aplica_tema(&ui, Mod::Luminos);
        ui
    }

    fn lista_de_proba() -> model_lista::Lista {
        use model_lista::{Obiect, Stare};
        let o = |nume: &str, tip: &str, stare: Stare| Obiect {
            schema: "dbo".into(),
            nume: nume.into(),
            baza: "CharismaProd".into(),
            tip: tip.into(),
            stare,
            detalii: None,
            revizuit: false,
            in_plan: false,
            omis: false,
            scor: None,
            deploy: Default::default(),
        };
        model_lista::Lista::noua(vec![
            o("usp_Stoc", "P", Stare::Modificat),
            o("usp_Import", "P", Stare::DoarSursa),
            o("vw_Facturi", "V", Stare::Identic),
        ])
    }

    fn antet_lista() -> AntetLista<'static> {
        AntetLista {
            sursa_baza: "CharismaDev",
            tinta_baza: "CharismaProd",
            servere: "srv-dev → srv-prod",
            mediu: Mediu::Prod,
            in_plan: 2,
        }
    }

    /// Coloanele noi din lista (Similar, Deploy) si starea barei de filtre
    /// vin din model, nu din desen.
    #[test]
    fn coloanele_de_scor_si_deploy_vin_din_model() {
        use model_lista::{Deploy, FiltruRevizuire, Ordine};
        let ui = fereastra_lista();
        let mut lista = lista_de_proba();
        lista.pune_deploy("CharismaProd", "dbo.usp_Stoc", Deploy::Deployat);
        lista.pune_deploy("CharismaProd", "dbo.usp_Import", Deploy::VersiuneNoua);
        lista.sorteaza(Ordine::Scor);
        lista.filtreaza_revizuire(Some(FiltruRevizuire::Nerevizuite));
        lista.filtreaza_scor(30.0, 80.0);
        incarca_lista(&ui, &antet_lista(), &lista);

        assert_eq!(ui.get_coloana_sortata(), 3, "coloana Similar");
        assert!(!ui.get_sens_descrescator(), "scorul incepe crescator");
        assert_eq!(ui.get_stare_activa(), 0, "Nerevizuite e prima optiune");
        assert_eq!(ui.get_stari().row_count(), 6);
        assert_eq!(ui.get_scor_min(), "30");
        assert_eq!(ui.get_scor_max(), "80");
        // filtrul de scor e activ, deci obiectele fara scor nu se mai vad
        assert_eq!(ui.get_randuri().row_count(), 0);

        // fara filtrul de scor: coloanele randurilor
        lista.filtreaza_scor(0.0, 100.0);
        incarca_lista(&ui, &antet_lista(), &lista);
        let rand = |n: &str| {
            ui.get_randuri()
                .iter()
                .find(|r| r.nume == n)
                .unwrap_or_else(|| panic!("randul {n}"))
        };
        assert_eq!(rand("usp_Stoc").deploy, 2, "trimis");
        assert_eq!(rand("usp_Import").deploy, 3, "versiune noua");
        assert_eq!(rand("vw_Facturi").deploy, 0, "neintrodus");
        assert_eq!(
            rand("usp_Stoc").scor_text,
            "",
            "fara scor calculat, coloana ramane goala, nu „0%”"
        );
        // „% deploy" numara doar ce difera si e neschimbat
        assert_eq!(ui.get_progres_deployate(), "1/2");
    }

    #[test]
    fn omisul_bate_starea_de_deploy_in_coloana() {
        use model_lista::Deploy;
        let ui = fereastra_lista();
        let mut lista = lista_de_proba();
        lista.pune_deploy("CharismaProd", "dbo.usp_Stoc", Deploy::Deployat);
        let p = (0..lista.vizibile().len())
            .find(|&p| lista.obiect(p).unwrap().nume == "usp_Stoc")
            .unwrap();
        lista.pune_revizuirea(p, false, true);
        incarca_lista(&ui, &antet_lista(), &lista);
        let r = ui
            .get_randuri()
            .iter()
            .find(|r| r.nume == "usp_Stoc")
            .expect("randul");
        assert_eq!(r.deploy, 4, "omis: lasat deoparte deliberat");
        assert!(r.omis);
    }

    #[test]
    fn lista_arata_aceleasi_cifre_ca_modelul() {
        let ui = fereastra_lista();
        let mut lista = lista_de_proba();
        lista.selecteaza(0);
        incarca_lista(&ui, &antet_lista(), &lista);

        assert_eq!(ui.get_randuri().row_count(), 3);
        assert_eq!(ui.get_eticheta_mediu(), "PROD");
        assert_eq!(ui.get_numar_deploy(), "2");
        assert_eq!(ui.get_progres_revizuite(), "0/2");
        // categoriile sunt toate cinci, in ordinea din machete
        let categorii: Vec<String> = ui
            .get_categorii()
            .iter()
            .map(|c| c.nume.to_string())
            .collect();
        assert_eq!(
            categorii,
            [
                "Modificate",
                "Doar in sursa",
                "Doar in baza",
                "Identice",
                "Criptate"
            ]
        );
        assert_eq!(ui.get_categorii().row_data(0).unwrap().numar, "1");
        // tipurile numara doar ce difera: o procedura modificata, una doar in sursa
        let tipuri: Vec<(String, String)> = ui
            .get_tipuri()
            .iter()
            .map(|t| (t.nume.to_string(), t.numar.to_string()))
            .collect();
        assert_eq!(tipuri, [("Proceduri".to_string(), "2".to_string())]);
        // numele tipului e cel pentru utilizator, nu codul din sys.objects
        let r = ui
            .get_randuri()
            .iter()
            .find(|r| r.nume == "usp_Stoc")
            .expect("randul obiectului modificat");
        assert_eq!(r.tip, "procedura");
        assert_eq!(r.litera, "M");
        // lista e sortata alfabetic, deci primul rand e altul
        assert_eq!(ui.get_randuri().row_data(0).unwrap().nume, "usp_Import");
    }

    #[test]
    fn filtrul_din_model_se_vede_in_bara_laterala() {
        let ui = fereastra_lista();
        let mut lista = lista_de_proba();
        incarca_lista(&ui, &antet_lista(), &lista);
        assert_eq!(ui.get_categorie_activa(), -1, "fara filtru");
        lista.filtreaza_stare(Some(model_lista::Stare::DoarSursa));
        incarca_lista(&ui, &antet_lista(), &lista);
        assert_eq!(ui.get_categorie_activa(), 1);
        assert_eq!(ui.get_randuri().row_count(), 1);
        assert_eq!(ui.get_randuri().row_data(0).unwrap().nume, "usp_Import");
    }

    #[test]
    fn previzualizarea_spune_ce_s_a_schimbat() {
        let ui = fereastra_lista();
        let a = gmotion_core::pregateste_afisare("RAISERROR('Nu s-au gasit date', 16, 1);");
        let b = gmotion_core::pregateste_afisare("RAISERROR('nu s-au gasit date', 16, 1);");
        incarca_previzualizare(&ui, &a, &b, &[]);
        assert_eq!(
            ui.get_ce_difera(),
            "un singur caracter, intr-un sir: Nu → nu"
        );
        assert!(ui.get_previzualizare().row_count() > 0);
        goleste_previzualizarea(&ui);
        assert_eq!(ui.get_previzualizare().row_count(), 0);
    }

    #[test]
    fn randurile_fara_detalii_calculate_au_coloanele_goale_nu_zerouri() {
        let ui = fereastra_lista();
        let mut lista = lista_de_proba();
        incarca_lista(&ui, &antet_lista(), &lista);
        let r = ui
            .get_randuri()
            .iter()
            .find(|r| r.nume == "usp_Stoc")
            .expect("randul obiectului modificat");
        assert_eq!(
            (r.plus.as_str(), r.minus.as_str(), r.difera_in.as_str()),
            ("", "", "")
        );
        let p = lista
            .vizibile()
            .iter()
            .position(|&i| lista.obiecte()[i].nume == "usp_Stoc")
            .unwrap();
        lista.pune_detalii(
            p,
            model_lista::Detalii {
                difera_in: "un sir".into(),
                linii_plus: 1,
                linii_minus: 1,
            },
        );
        incarca_lista(&ui, &antet_lista(), &lista);
        let r = ui
            .get_randuri()
            .iter()
            .find(|r| r.nume == "usp_Stoc")
            .unwrap();
        assert_eq!((r.plus.as_str(), r.minus.as_str()), ("+1", "−1"));
        assert_eq!(r.difera_in, "un sir");
    }

    #[test]
    fn linia_de_jurnal_isi_pierde_prefixul_si_da_nivelul() {
        assert_eq!(
            linie_jurnal("[INFO] deschid conexiunea server=X"),
            (0, "deschid conexiunea server=X".into())
        );
        assert_eq!(
            linie_jurnal("[WARN] certificat neverificat"),
            (2, "certificat neverificat".into())
        );
        assert_eq!(
            linie_jurnal("[ERROR] [GM-DB-001] nu raspunde"),
            (3, "[GM-DB-001] nu raspunde".into())
        );
        // fara prefix: ramane cum e, ca informatie
        assert_eq!(linie_jurnal("fara nivel"), (0, "fara nivel".into()));
        assert_eq!(
            linie_jurnal("[GM-DB-001] cod, nu nivel"),
            (0, "[GM-DB-001] cod, nu nivel".into())
        );
    }

    #[test]
    fn banda_de_jurnal_arata_doar_ultimele_linii() {
        let ui = fereastra_conectare();
        let linii: Vec<String> = (1..=8).map(|i| format!("[INFO] pasul {i}")).collect();
        incarca_jurnal(&ui, &linii);
        let j = ui.get_jurnal();
        assert_eq!(j.row_count(), LINII_JURNAL);
        assert_eq!(j.row_data(0).unwrap().text, "pasul 5");
        assert_eq!(j.row_data(LINII_JURNAL - 1).unwrap().text, "pasul 8");
        incarca_jurnal(&ui, &[]);
        assert_eq!(ui.get_jurnal().row_count(), 0);
    }

    fn fereastra_conectare() -> FereastraConectare {
        let w = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        let _ = slint::platform::set_platform(Box::new(Platforma(w)));
        let ui = FereastraConectare::new().expect("fereastra de test");
        aplica_tema(&ui, Mod::Luminos);
        ui
    }

    #[test]
    fn ecranul_de_conectare_spune_ce_lipseste_si_nu_arata_parola() {
        use model_conectare::{Acreditari, Conectare, Parte, StareConexiune};
        let ui = fereastra_conectare();
        let mut c = Conectare::noua();
        incarca_conectare(&ui, &c);
        assert_eq!(ui.get_ce_lipseste(), "completeaza serverul pentru sursa");

        c.sursa = Parte {
            server: "SRV-DEV".into(),
            acreditari: Acreditari::noi("deploy_ro", "parola-secreta"),
            baza: "CharismaDev".into(),
            baze: vec!["CharismaDev".into(), "Altceva".into()],
            stare: StareConexiune::Conectat("conectat · 21.204 obiecte".into()),
            auth_windows: false,
            servere_gasite: vec!["SRV-DEV\\SQL2019".into(), "SRV-PROD".into()],
            alese: Vec::new(),
        };
        c.baza = Parte {
            server: "SRV-PROD".into(),
            baza: "CharismaProd".into(),
            baze: vec!["CharismaProd".into()],
            stare: StareConexiune::Conectat("conectat".into()),
            auth_windows: true,
            ..Parte::default()
        };
        incarca_conectare(&ui, &c);
        assert_eq!(ui.get_ce_lipseste(), "", "se poate compara");
        assert_eq!(ui.get_sursa().utilizator, "deploy_ro");
        assert!(
            !ui.get_sursa().parola.contains("parola-secreta"),
            "parola nu ajunge in fereastra"
        );
        assert_eq!(ui.get_sursa().fel_stare, 2);
        // ce a gasit cautarea in retea ajunge in lista de sub camp
        assert_eq!(ui.get_sursa().servere_gasite.row_count(), 2);
        assert_eq!(ui.get_baza().servere_gasite.row_count(), 0);
        assert_eq!(ui.get_baza().baza, "CharismaProd");
        // mediul tintei da banda si eticheta
        assert_eq!(ui.get_mediu(), 2);
        assert_eq!(ui.get_baza().mediu, "PROD");
        assert_eq!(ui.get_sursa().mediu, "DEV");
    }

    #[test]
    fn mediul_se_ghiceste_din_numele_bazei() {
        assert_eq!(mediu_dupa_nume("CharismaProd").eticheta(), "PROD");
        assert_eq!(mediu_dupa_nume("charisma_test").eticheta(), "TEST");
        assert_eq!(mediu_dupa_nume("CharismaUAT").eticheta(), "TEST");
        assert_eq!(mediu_dupa_nume("CharismaDev").eticheta(), "DEV");
        assert_eq!(mediu_dupa_nume("").eticheta(), "DEV");
    }
}
