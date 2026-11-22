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

pub mod dublu_clic;
pub mod harta;
pub mod jetoane;
pub mod model_arbore;
pub mod model_conectare;
pub mod model_deploy;
pub mod model_editor;
pub mod model_lista;
pub mod model_taburi;
pub mod scara;
pub mod texte;

slint::include_modules!();

pub use jetoane::Mod;
use model_editor::{Bucata, Marcaj, ModelEditor, RandAfisat, Sintaxa, Stare};
use slint::{Color, Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};

/// Pune textele limbii alese in globalul `Texte`. Se cheama inainte de orice
/// afisare si din nou la fiecare schimbare de limba, ca `aplica_tema`.
pub fn aplica_limba<'a, T>(ui: &'a T, l: texte::Limba)
where
    T: slint::ComponentHandle,
    Texte<'a>: slint::Global<'a, T>,
{
    let g = ui.global::<Texte>();
    macro_rules! pune {
        ($($camp:ident $set:ident),* $(,)?) => {
            // destructurare fara `..`: un camp din `Texte` lipsa din lista
            // nu compileaza, deci nu poate ramane nepus
            let texte::Texte { $($camp),* } = *texte::texte(l);
            $( g.$set($camp.into()); )*
        };
    }
    cu_textele!(pune);
}

/// Textul ferestrei de ajutor (F1), in limba data. Intr-un singur loc, ca
/// ecranele sa nu spuna lucruri diferite despre aceleasi taste.
pub fn ajutor(l: texte::Limba) -> &'static str {
    l.alege(AJUTOR, AJUTOR_EN)
}

/// Textul ecranului „Despre": versiunea, componentele terte cu licentele lor
/// si unde sunt jurnalul si datele, ca o problema sa se poata trimite cu
/// fisierul potrivit. Atribuirea Slint o face `AboutSlint`, sub text.
pub fn text_despre(
    l: texte::Limba,
    versiune: &str,
    jurnal: Option<&std::path::Path>,
    date: Option<&std::path::Path>,
) -> String {
    let cale = |c: Option<&std::path::Path>| {
        c.map_or_else(
            || l.alege("(necunoscut)", "(unknown)").to_string(),
            |c| c.display().to_string(),
        )
    };
    format!(
        "GMotion {versiune}\n\n{}\n\n{}\n\
         \u{2022} Slint ({}): Slint Royalty-free License 2.0.\n\
         \u{2022} {}: SIL Open Font License 1.1.\n\
         \u{2022} tiberius ({}): MIT / Apache-2.0.\n\
         \u{2022} {}\n\n\
         {}: {}\n{}: {}",
        l.alege(
            "Compara obiectele SQL Server (proceduri, functii, view-uri, triggere) dintre un Martor si o baza Modificata si pregateste deploy-ul, cu revenire.",
            "Compares SQL Server objects (procedures, functions, views, triggers) between a Witness and a Modified database and prepares the deploy, with rollback.",
        ),
        l.alege("Componente terte:", "Third-party components:"),
        l.alege("interfata", "user interface"),
        l.alege("Fonturile Geist si Geist Mono", "The Geist and Geist Mono fonts"),
        l.alege("conexiunea SQL Server", "SQL Server connection"),
        l.alege(
            "restul bibliotecilor Rust: MIT si/sau Apache-2.0, lista completa in Cargo.lock.",
            "the other Rust libraries: MIT and/or Apache-2.0, full list in Cargo.lock.",
        ),
        l.alege("Jurnalul acestei sesiuni", "This session's log"),
        cale(jurnal),
        l.alege("Datele aplicatiei", "Application data"),
        cale(date),
    )
}

/// Ajutorul in romana.
pub const AJUTOR: &str = "\
GMotion compara obiectele programabile (proceduri, functii, view-uri, \
triggere) dintr-o baza martor cu una sau mai multe baze modificate, si \
scrie scripturile de deploy. Citeste; nu scrie nimic pe server.\n\
\n\
CONECTARE\n\
  Alege serverul si baza martor in stanga, serverul si bazele \
modificate in dreapta. Bifeaza mai multe baze ca sa compari acelasi \
martor cu toate deodata.\n\
  Testeaza conexiunea  spune ce raspunde serverul (editie, sistem, cate baze).\n\
  Comparare offline    doua bucati de cod lipite, fara server.\n\
  Esc                  anuleaza compararea care ruleaza.\n\
\n\
LISTA\n\
  Ctrl+J K / sageti  obiectul urmator, anterior\n\
  Enter          deschide obiectul in editor\n\
  Ctrl+R         revizuit\n\
  Ctrl+I         pune sau scoate din planul de deploy\n\
  Ctrl+O         omite obiectul\n\
  Ctrl+F         cauta\n\
  Ctrl+H         harta: toate obiectele dintr-o privire; Esc inapoi\n\
  Clic pe capul unei coloane sorteaza; al doilea clic inverseaza.\n\
\n\
EDITOR\n\
  Esc            inapoi la lista\n\
  Ctrl+J K       obiectul urmator, anterior (si PageDown, PageUp)\n\
  Ctrl+N P       diferenta urmatoare, anterioara (si F7, Shift+F7)\n\
  Ctrl+U         alaturat / unificat\n\
  Ctrl+L         rupe liniile lungi\n\
  Shift+sageti   deruleaza codul pe orizontala (liniile lungi); Home inapoi\n\
  Ctrl+E         editeaza textul final de mana\n\
  Ctrl+G         modul ghidat (fiecare diferenta pe rand)\n\
  Ctrl+R / I / O revizuit, in plan, omite\n\
  Ctrl+B         ascunde sau arata coloana listei\n\
  Ctrl+W         inchide tabul\n\
  Ctrl+Z         anuleaza ultima schimbare a textului final\n\
\n\
DEPLOY\n\
  Planul se verifica inainte de scriere (ca bazele n-au mai fost atinse de \
la comparare), iar revenirea se scrie inaintea scripturilor.\n\
\n\
F1 deschide si inchide fereastra asta. F11 trece pe tot ecranul.";

/// Ajutorul in engleza; aceleasi taste, aceeasi ordine ca `AJUTOR`.
pub const AJUTOR_EN: &str = "\
GMotion compares programmable objects (procedures, functions, views, \
triggers) in a witness database with one or more modified databases, and \
writes the deploy scripts. It only reads; it writes nothing on the server.\n\
\n\
CONNECTION\n\
  Choose the server and the witness database on the left, the server and the \
modified databases on the right. Tick several databases to compare the same \
witness with all of them at once.\n\
  Test connection      shows what the server answers (edition, system, databases).\n\
  Offline comparison   two pieces of pasted code, no server.\n\
  Esc                  cancels the running comparison.\n\
\n\
LIST\n\
  Ctrl+J K / arrows  next, previous object\n\
  Enter          open the object in the editor\n\
  Ctrl+R         reviewed\n\
  Ctrl+I         add to or remove from the deploy plan\n\
  Ctrl+O         skip the object\n\
  Ctrl+F         search\n\
  Ctrl+H         map: every object at a glance; Esc back\n\
  Click a column header to sort; a second click reverses.\n\
\n\
EDITOR\n\
  Esc            back to the list\n\
  Ctrl+J K       next, previous object (also PageDown, PageUp)\n\
  Ctrl+N P       next, previous difference (also F7, Shift+F7)\n\
  Ctrl+U         side by side / unified\n\
  Ctrl+L         wrap long lines\n\
  Shift+arrows   scroll the code horizontally (long lines); Home back\n\
  Ctrl+E         edit the final text by hand\n\
  Ctrl+G         guided mode (each difference in turn)\n\
  Ctrl+R / I / O reviewed, in plan, skip\n\
  Ctrl+B         hide or show the list column\n\
  Ctrl+W         close the tab\n\
  Ctrl+Z         undo the last change to the final text\n\
\n\
DEPLOY\n\
  The plan is checked before writing (that the databases were not touched \
since the comparison), and the rollback is written before the scripts.\n\
\n\
F1 opens and closes this window. F11 goes full screen.";

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
        harta_mod set_harta_mod get_harta_mod,
        harta_add set_harta_add get_harta_add,
        harta_del set_harta_del get_harta_del,
        harta_gol set_harta_gol get_harta_gol,
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

/// Pune randurile unei liste Slint **pe loc**, cand numarul lor nu s-a
/// schimbat: un model nou la fiecare redesenare recrea randurile, deci al
/// doilea clic al unui dublu clic cadea pe alt rand si dublu clicul nu se
/// forma niciodata (gasit pe Windows, 2026-11-19). Intoarce modelul de pus,
/// sau `None` daca s-a actualizat cel vechi.
fn pe_loc<T: Clone + PartialEq + 'static>(vechi: ModelRc<T>, noi: Vec<T>) -> Option<ModelRc<T>> {
    use slint::Model;
    if let Some(v) = vechi.as_any().downcast_ref::<VecModel<T>>() {
        if v.row_count() == noi.len() {
            for (i, r) in noi.into_iter().enumerate() {
                if v.row_data(i).as_ref() != Some(&r) {
                    v.set_row_data(i, r);
                }
            }
            return None;
        }
    }
    Some(ModelRc::new(VecModel::from(noi)))
}

/// Limba pusa in `Texte` de ultimul [`aplica_limba`]; inainte de el, romana.
pub fn limba_curenta<'a, T>(ui: &'a T) -> texte::Limba
where
    T: slint::ComponentHandle,
    Texte<'a>: slint::Global<'a, T>,
{
    texte::Limba::din_nume(&ui.global::<Texte>().get_cod_limba()).unwrap_or_default()
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

/// Codul culorii de sintaxa pentru `.slint` (`BucataUi::sintaxa`).
fn cod_sintaxa(s: Sintaxa) -> i32 {
    match s {
        Sintaxa::Text => 0,
        Sintaxa::CuvantCheie => 1,
        Sintaxa::Variabila => 2,
        Sintaxa::Numar => 3,
        Sintaxa::Sir => 4,
        Sintaxa::Comentariu => 5,
    }
}

fn bucati_ui(b: &[Bucata]) -> ModelRc<BucataUi> {
    ModelRc::new(VecModel::from(
        b.iter()
            .map(|x| BucataUi {
                text: SharedString::from(x.text.as_str()),
                sintaxa: cod_sintaxa(x.sintaxa),
                schimbat: x.schimbat,
            })
            .collect::<Vec<_>>(),
    ))
}

/// Latimea estimata a unui caracter mono de 12,5 px, in pixeli logici (Slint
/// nu masoara un text fara sa-l deseneze). Aceeasi in `.slint`, la
/// `coloane-wrap`.
const CARACTER_MONO: f32 = 7.52;

/// Cate coloane ocupa un caracter pe ecran: tabul se socoteste 4.
fn coloane_caracter(c: char) -> usize {
    if c == '\t' {
        4
    } else {
        1
    }
}

/// Bucatile unei linii, rupte in randuri de cel mult `coloane` (wrap, etapa
/// 19). Se rupe dupa ultimul spatiu din a doua jumatate a randului, ca un
/// cuvant sa nu se taie; fara spatiu acolo, la margine. O bucata taiata isi
/// pastreaza culoarea si marcajul. Intoarce mereu cel putin un rand (gol,
/// pentru linia goala). Fara `coloane`, linia ramane intreaga.
fn rupe_linia(bucati: &[Bucata], coloane: Option<usize>) -> Vec<Vec<Bucata>> {
    let Some(max) = coloane.filter(|&c| c >= 8) else {
        return vec![bucati.to_vec()];
    };
    // fiecare caracter, cu bucata lui
    let car: Vec<(char, usize)> = bucati
        .iter()
        .enumerate()
        .flat_map(|(k, b)| b.text.chars().map(move |c| (c, k)))
        .collect();
    // unde incepe fiecare rand afisat
    let mut inceputuri = vec![0usize];
    let (mut start, mut ocupat) = (0usize, 0usize);
    let mut i = 0usize;
    while i < car.len() {
        let w = coloane_caracter(car[i].0);
        if ocupat + w > max && i > start {
            // ultimul spatiu din a doua jumatate a randului
            let rupe = (start + 1..i)
                .rev()
                .take_while(|&k| k > start + max / 2)
                .find(|&k| car[k - 1].0 == ' ')
                .unwrap_or(i);
            inceputuri.push(rupe);
            start = rupe;
            ocupat = car[start..i]
                .iter()
                .map(|&(c, _)| coloane_caracter(c))
                .sum();
            continue;
        }
        ocupat += w;
        i += 1;
    }
    inceputuri.push(car.len());
    inceputuri
        .windows(2)
        .map(|g| {
            let mut rand: Vec<Bucata> = Vec::new();
            for &(c, k) in &car[g[0]..g[1]] {
                match rand.last_mut() {
                    // bucatile vecine cu acelasi aspect se lipesc
                    Some(b)
                        if b.sintaxa == bucati[k].sintaxa && b.schimbat == bucati[k].schimbat =>
                    {
                        b.text.push(c)
                    }
                    _ => rand.push(Bucata {
                        text: c.to_string(),
                        ..bucati[k].clone()
                    }),
                }
            }
            rand
        })
        .collect()
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

/// Liniile previzualizarii la hover, cum le deseneaza `EditorDiferente`.
pub fn previzualizare_ui(
    linii: &[model_editor::LiniePrevizualizare],
) -> ModelRc<LiniePrevizualizareUi> {
    ModelRc::new(VecModel::from(
        linii
            .iter()
            .map(|l| LiniePrevizualizareUi {
                nr: l
                    .nr
                    .map_or_else(SharedString::new, |n| n.to_string().into()),
                text: l.text.as_str().into(),
                adusa: l.adusa,
            })
            .collect::<Vec<_>>(),
    ))
}

/// Locul unde se lasa un bloc, cum il poarta un rand din Slint: >= 0
/// inlocuieste blocul cu indexul asta, <= -2 se insereaza inaintea liniei
/// `-(cod + 2)`, -1 nimic. Inversul e [`loc_din_ui`].
pub fn loc_ui(loc: Option<gmotion_core::Loc>) -> i32 {
    match loc {
        Some(gmotion_core::Loc::Inlocuieste(b)) => i32::try_from(b).unwrap_or(-1),
        Some(gmotion_core::Loc::Insereaza(l)) => i32::try_from(l).map_or(-1, |l| -l - 2),
        None => -1,
    }
}

/// Codul primit inapoi din Slint, ca loc (vezi [`loc_ui`]).
pub fn loc_din_ui(cod: i32) -> Option<gmotion_core::Loc> {
    match cod {
        c if c >= 0 => Some(gmotion_core::Loc::Inlocuieste(c as usize)),
        c if c <= -2 => Some(gmotion_core::Loc::Insereaza((-c - 2) as usize)),
        _ => None,
    }
}

/// Randurile de afisat si randul de inceput al fiecarei diferente, **in
/// coordonatele listei afisate** (plierea schimba indicii, iar navigarea
/// deruleaza lista, nu modelul).
/// Numai pentru probe si exemple: randurile asa cum ajung in Slint.
#[doc(hidden)]
pub fn randuri_ui_pentru_test(model: &ModelEditor, pliat: bool) -> (Vec<RandUi>, Vec<i32>) {
    randuri_ui(model, pliat, false, texte::Limba::Romana, None)
}

fn randuri_ui(
    model: &ModelEditor,
    pliat: bool,
    unificat: bool,
    limba: texte::Limba,
    wrap: Option<usize>,
) -> (Vec<RandUi>, Vec<i32>) {
    let (randuri, start, _) = randuri_ui_cu_tinte(model, pliat, unificat, limba, wrap);
    (randuri, start)
}

/// Locul in care ajunge fiecare diferenta, pe randuri afisate: `[de_la,
/// pana_la)`, sau `(-1, -1)` cand n-are loc. Pe diferenta, nu pe rand: doua
/// blocuri pot cadea in acelasi loc (blocurile 4 si 5 pe acelasi
/// rand-fantoma), iar un singur numar pe rand il tinea doar pe ultimul.
type Tinte = Vec<(i32, i32)>;

fn randuri_ui_cu_tinte(
    model: &ModelEditor,
    pliat: bool,
    unificat: bool,
    limba: texte::Limba,
    wrap: Option<usize>,
) -> (Vec<RandUi>, Vec<i32>, Tinte) {
    // un rand al modelului devine unul sau mai multe randuri afisate: cu
    // wrap, linia lunga continua pe randuri fara numar, iar partea mai
    // scurta primeste randuri goale, ca stanga si dreapta sa ramana aliniate
    let pune = |randuri: &mut Vec<RandUi>, sablon: RandUi, st: &[Bucata], dr: &[Bucata]| {
        let st_r = rupe_linia(st, wrap);
        let dr_r = rupe_linia(dr, wrap);
        let n = st_r.len().max(dr_r.len());
        let st_lipsa = sablon.st_nr.is_empty();
        let dr_lipsa = sablon.dr_nr.is_empty();
        for j in 0..n {
            let parte =
                |v: &Vec<Vec<Bucata>>| v.get(j).map_or_else(|| bucati_ui(&[]), |b| bucati_ui(b));
            let mut r = RandUi {
                st: parte(&st_r),
                dr: parte(&dr_r),
                ..sablon.clone()
            };
            if j > 0 {
                r.st_nr = SharedString::new();
                r.dr_nr = SharedString::new();
                r.inceput = false;
                r.continuare = true;
                r.st_lipsa = st_lipsa;
                r.dr_lipsa = dr_lipsa;
            }
            randuri.push(r);
        }
    };
    // un obiect fara nicio diferenta se arata intreg: pliat, ramanea un
    // singur rand („5 linii identice") si nu se vedea codul deloc (Mihai,
    // 2026-11-19: „vreau sa vad indiferent")
    let pliat = pliat && !model.diferente.is_empty();
    let afisate = if pliat {
        model_editor::plieaza(model, 3, limba)
    } else {
        (0..model.randuri.len()).map(RandAfisat::Linie).collect()
    };
    let a_cata = model_editor::diferenta_fiecarui_rand(model);
    let gol = || ModelRc::new(VecModel::from(Vec::<BucataUi>::new()));
    let nr =
        |n: Option<usize>| n.map_or(SharedString::new(), |n| SharedString::from(n.to_string()));

    let mut randuri = Vec::with_capacity(afisate.len());
    let mut start = vec![-1i32; model.diferente.len()];
    // unde a ajuns fiecare rand al modelului in lista afisata (plierea si
    // wrap-ul schimba indicii): din asta ies legaturile din canal
    let mut afisat_pentru = vec![-1i32; model.randuri.len()];
    // drumul invers: din randul afisat in cel al modelului
    let mut model_pentru: Vec<usize> = Vec::with_capacity(afisate.len());
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
                linii_bloc: 0,
                pereche: gol(),
                pereche_nr: SharedString::new(),
                continuare: false,
                st_lipsa: false,
                dr_lipsa: false,
                eticheta: SharedString::new(),
                fantoma: false,
                delta_tinta: 0,
                tinta_gol: false,
                loc: -1,
                bloc: -1,
                text_st: SharedString::new(),
                inserare: false,
                cutie_st: false,
                cutie_dr: false,
                cutie_jos: false,
                nr_diferenta: 0,
            }),
            &RandAfisat::Linie(i) => {
                let rand = &model.randuri[i];
                // eticheta blocului sta pe un rand al ei, deasupra, in ambele
                // panouri: pusa peste cod acoperea linia, iar cu wrap se
                // repeta pe fiecare bucata (sesizat de Mihai, 2026-11-20).
                // `diferenta: -1`: randul nu e al blocului (nu se numara in
                // linia din canal si nu se aprinde la tragere)
                if !rand.eticheta.is_empty() {
                    randuri.push(RandUi {
                        fel: 2,
                        pliere: SharedString::new(),
                        st_nr: SharedString::new(),
                        st: gol(),
                        dr_nr: SharedString::new(),
                        dr: gol(),
                        stare: cod_stare(rand.stare),
                        diferenta: -1,
                        dr_marcaj: 0,
                        inceput: false,
                        linii_bloc: 0,
                        pereche: gol(),
                        pereche_nr: SharedString::new(),
                        continuare: false,
                        st_lipsa: false,
                        dr_lipsa: false,
                        eticheta: SharedString::from(rand.eticheta.as_str()),
                        fantoma: false,
                        delta_tinta: 0,
                        tinta_gol: false,
                        loc: -1,
                        bloc: -1,
                        text_st: SharedString::new(),
                        inserare: false,
                        cutie_st: false,
                        cutie_dr: false,
                        cutie_jos: false,
                        nr_diferenta: 0,
                    });
                }
                afisat_pentru[i] = randuri.len() as i32;
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
                    linii_bloc: 0,
                    pereche: gol(),
                    pereche_nr: SharedString::new(),
                    continuare: false,
                    st_lipsa: false,
                    dr_lipsa: false,
                    eticheta: SharedString::new(),
                    fantoma: rand.fantoma,
                    // se completeaza la final: tinta se stie abia cand se
                    // cunosc toate randurile afisate
                    delta_tinta: 0,
                    tinta_gol: false,
                    loc: loc_ui(rand.loc),
                    bloc: match (rand.stare, rand.bloc) {
                        (Stare::Egal, _) | (_, None) => -1,
                        (_, Some(b)) => b as i32,
                    },
                    text_st: rand
                        .stanga
                        .bucati
                        .iter()
                        .map(|b| b.text.as_str())
                        .collect::<String>()
                        .trim()
                        .into(),
                    inserare: rand.inserare.is_some(),
                    cutie_st: false,
                    cutie_dr: false,
                    cutie_jos: false,
                    nr_diferenta: 0,
                };
                if !unificat || rand.stare != Stare::Modificat {
                    pune(
                        &mut randuri,
                        baza,
                        &rand.stanga.bucati,
                        &rand.dreapta.bucati,
                    );
                    continue;
                }
                // unificat: blocul modificat se vede de doua ori, intai cum e
                // in sursa, apoi cum e in baza
                // cealalta versiune ramane pe rand, in `pereche`: la hover
                // se vede fara sa comuti pe alaturat (v3.4 facea la fel)
                if rand.stanga.nr.is_some() {
                    let sablon = RandUi {
                        dr_nr: SharedString::new(),
                        pereche: bucati_ui(&rand.dreapta.bucati),
                        pereche_nr: nr(rand.dreapta.nr),
                        ..baza.clone()
                    };
                    pune(&mut randuri, sablon, &rand.stanga.bucati, &[]);
                }
                if rand.dreapta.nr.is_some() {
                    let sablon = RandUi {
                        st_nr: SharedString::new(),
                        pereche: bucati_ui(&rand.stanga.bucati),
                        pereche_nr: nr(rand.stanga.nr),
                        ..baza
                    };
                    pune(&mut randuri, sablon, &[], &rand.dreapta.bucati);
                }
            }
        }
    }
    // cate randuri afisate tine fiecare diferenta: linia de legatura din
    // canal se deseneaza de la primul la ultimul rand al blocului
    let mut cate = vec![0i32; model.diferente.len()];
    for r in &randuri {
        if r.diferenta >= 0 {
            if let Some(n) = cate.get_mut(r.diferenta as usize) {
                *n += 1;
            }
        }
    }
    for r in randuri.iter_mut().filter(|r| r.inceput && r.diferenta >= 0) {
        r.linii_bloc = cate.get(r.diferenta as usize).copied().unwrap_or(0);
    }
    // din ce rand al modelului vine fiecare rand afisat
    for (i, m) in afisat_pentru.iter().enumerate() {
        if *m >= 0 {
            while model_pentru.len() <= *m as usize {
                model_pentru.push(usize::MAX);
            }
            model_pentru[*m as usize] = i;
        }
    }
    let model_al_randului = |afisat: usize| -> Option<usize> {
        afisat_pentru
            .iter()
            .enumerate()
            .filter(|(_, &a)| a >= 0 && a as usize <= afisat)
            .max_by_key(|(_, &a)| a)
            .map(|(m, _)| m)
    };

    // legatura din canal: de la primul rand al diferentei la randul unde
    // ajunge blocul. Pe randuri afisate, nu pe randuri de model — plierea si
    // wrap-ul le muta.
    let mut tinte: Tinte = vec![(-1, -1); model.tinte.len()];
    for (k, tinta) in model.tinte.iter().enumerate() {
        let (Some(tinta), Some(&de_la)) = (tinta, start.get(k)) else {
            continue;
        };
        if de_la < 0 {
            continue;
        }
        let pana_la = afisat_pentru.get(*tinta).copied().unwrap_or(-1);
        if pana_la < 0 {
            continue;
        }
        let gol = model.randuri.get(*tinta).is_some_and(|r| r.fantoma);
        if let Some(r) = randuri.get_mut(de_la as usize) {
            r.delta_tinta = pana_la - de_la;
            r.tinta_gol = gol;
        }
        // tot blocul tinta se aprinde cat timp se trage, nu doar primul rand:
        // altfel pare ca blocul ar intra pe o singura linie
        let bloc_tinta = model.randuri.get(*tinta).and_then(|r| r.bloc);
        // doar randurile de cod: eticheta blocului urmator ar parea, dupa
        // pozitie, inca un rand al blocului tinta
        // locul de inserare se aprinde intreg (toate randurile lui), nu doar
        // randul cu textul
        let grup_inserare = model.randuri.get(*tinta).and_then(|r| r.inserare);
        let in_loc: Vec<usize> = (0..randuri.len())
            .filter(|&i| randuri[i].fel == 0)
            .filter(|&i| match (grup_inserare, bloc_tinta) {
                (Some(g), _) => model_al_randului(i)
                    .and_then(|m| model.randuri.get(m))
                    .is_some_and(|m| m.inserare == Some(g)),
                (None, b) => match b {
                    Some(b) => model_al_randului(i)
                        .and_then(|m| model.randuri.get(m))
                        .is_some_and(|m| m.bloc == Some(b)),
                    None => i as i32 == pana_la,
                },
            })
            .collect();
        if let (Some(&a), Some(&z)) = (in_loc.first(), in_loc.last()) {
            tinte[k] = (a as i32, z as i32 + 1);
        }
    }
    marcheaza_cutiile(&mut randuri);
    (randuri, start, tinte)
}

/// Blocul ca in v3.4 (Mihai, 2026-11-20): o cutie cu chenar in jurul codului
/// lui, in panoul unde are cod (Martor pentru ce vine de acolo, Modificata
/// pentru ce e doar acolo, amandoua la modificat), cu eticheta pe randul de
/// sus al cutiei. Aici se decide doar ce rand e in cutie, pe ce parte, si
/// care e ultimul; desenul e in `.slint`.
fn marcheaza_cutiile(randuri: &mut [RandUi]) {
    // partile pe care are cod un bloc in starea asta (cod_stare)
    let parti = |stare: i32| match stare {
        1 => (true, true),
        2 => (true, false),
        3 => (false, true),
        _ => (false, false),
    };
    let n = randuri.len();
    for i in 0..n {
        let r = &randuri[i];
        let k = if r.fel == 2 {
            // eticheta sta deasupra primului rand al blocului
            randuri.get(i + 1).map_or(-1, |u| u.diferenta)
        } else if r.fel == 0 {
            r.diferenta
        } else {
            -1
        };
        if k < 0 {
            continue;
        }
        let (st, dr) = parti(r.stare);
        let ultimul = r.fel == 0
            && !randuri
                .get(i + 1)
                .is_some_and(|u| u.fel == 0 && u.diferenta == k);
        let r = &mut randuri[i];
        r.cutie_st = st;
        r.cutie_dr = dr;
        r.cutie_jos = ultimul;
        r.nr_diferenta = k + 1;
    }
}

/// Datele afisate in antetul editorului.
pub struct AntetEditor<'a> {
    /// Cheia obiectului deschis, `baza/schema.nume`. Cand se schimba,
    /// `incarca_editor` **inchide editarea de mana**: altfel campul ramanea cu
    /// textul obiectului dinainte (legatura spre Slint se rupe la prima tasta)
    /// si „Salveaza" il punea pe obiectul nou (F-A1 din auditul QA).
    pub id: &'a str,
    pub schema: &'a str,
    pub nume: &'a str,
    /// „procedura", „vedere", „functie"...
    pub tip: &'a str,
    pub sursa_baza: &'a str,
    pub sursa_server: &'a str,
    pub tinta_baza: &'a str,
    pub tinta_server: &'a str,
    /// „obiectul 4 din 212 nerevizuite"
    pub stare_obiecte: &'a str,
    pub revizuit: bool,
    /// Obiectul e in planul de deploy.
    pub in_plan: bool,
}

/// „1 diferenta" / „4 diferente" / „21 de diferente".
fn numar_diferente(n: usize, l: texte::Limba) -> String {
    if n == 0 {
        return l.alege("fara diferente", "no differences").to_string();
    }
    l.numar(n, ("diferenta", "diferente"), ("difference", "differences"))
}

/// „srv-dev · 47 linii".
fn detaliu_panou(server: &str, linii: usize, limba: texte::Limba) -> String {
    let l = limba.numar(linii, ("linie", "linii"), ("line", "lines"));
    if server.is_empty() {
        l
    } else {
        format!("{server} · {l}")
    }
}

/// Umple editorul cu modelul unui obiect. Nu atinge jetoanele: `aplica_tema`
/// Randurile listei, pentru tabelul intreg si pentru coloana ingusta din
/// editor. Un singur loc: altfel cele doua ar arata altceva despre acelasi
/// obiect.
fn randuri_lista_ui(lista: &model_lista::Lista, limba: texte::Limba) -> Vec<RandListaUi> {
    use model_lista::Stare;
    let cod = |s: Stare| match s {
        Stare::Modificat => 0,
        Stare::DoarSursa => 1,
        Stare::DoarBaza => 2,
        Stare::Identic => 3,
        Stare::Criptat => 4,
    };
    lista
        .vizibile()
        .iter()
        .enumerate()
        .map(|(pozitie, &i)| {
            let o = &lista.obiecte()[i];
            let (plus, minus) = match &o.detalii {
                Some(d) => (
                    format!("+{}", d.linii_plus),
                    format!("\u{2212}{}", d.linii_minus),
                ),
                None => (String::new(), String::new()),
            };
            RandListaUi {
                antet: SharedString::new(),
                pozitie: pozitie as i32,
                nume_scurt: o.nume.as_str().into(),
                schema: o.schema.as_str().into(),
                nume: o.nume.as_str().into(),
                baza: o.baza.as_str().into(),
                tip: gmotion_core::nume_tip_in(&o.tip, limba).into(),
                litera: o.stare.litera().into(),
                stare: cod(o.stare),
                plus: plus.into(),
                minus: minus.into(),
                difera_in: o
                    .detalii
                    .as_ref()
                    .map_or(String::new(), |d| {
                        gmotion_core::clasificare::traduce_eticheta(&d.difera_in, limba)
                    })
                    .into(),
                revizuit: o.revizuit,
                in_plan: o.in_plan,
                omis: o.omis,
                scor: o.scor.unwrap_or(0.0) as f32 / 100.0,
                scor_text: o.scor.map_or(String::new(), |x| format!("{x:.0}%")).into(),
                deploy: cod_deploy(o),
                folosit_de: o.folosit_de.map_or(String::new(), |n| n.to_string()).into(),
                impact: o.impact.map_or(-1, |i| i as i32),
                // fara secunde: in lista ajunge minutul
                modificat: o
                    .modificat
                    .as_deref()
                    .map_or("", |m| m.get(..16).unwrap_or(m))
                    .into(),
            }
        })
        .collect()
}

/// Umple coloana din stanga a editorului: lista stransa, din acelasi model
/// ca lista intreaga. Fara ea, „Deschide" te scotea din ecran si trebuia sa
/// te intorci cu Esc pentru fiecare obiect (pachetul B al etapei 15).
pub fn incarca_coloana_lista(
    ui: &EditorDiferente,
    lista: &model_lista::Lista,
    arbore: &model_arbore::Arbore,
) {
    let t = ui.global::<Texte>();
    let (nerevizuite, vizibile) = lista.rezumat_vizibile();
    ui.set_rezumat_lista(format!("{nerevizuite} {} {vizibile}", t.get_nerevizuite_din()).into());
    ui.set_selectat_lista(lista.selectat().map_or(0, |p| p as i32));
    let limba = limba_curenta(ui);
    ui.set_numar_obiecte(lista.vizibile().len() as i32);
    // bara de sus (etapa 18): progresul revizuirii, ca in lista mare
    let (revizuite, total) = lista.progres_revizuire();
    ui.set_progres_text(
        format!(
            "{revizuite}/{total} {}",
            limba.alege("revizuite", "reviewed")
        )
        .into(),
    );
    ui.set_progres_fractie(if total == 0 {
        0.0
    } else {
        revizuite as f32 / total as f32
    });
    ui.set_numar_deploy(lista.in_plan().len() as i32);
    // antetul obiectului: +/- si unde difera, din detaliile randului
    match lista.obiect_selectat().and_then(|o| o.detalii.as_ref()) {
        Some(d) => {
            ui.set_plus(format!("+{}", d.linii_plus).into());
            ui.set_minus(format!("\u{2212}{}", d.linii_minus).into());
            let eticheta = gmotion_core::clasificare::traduce_eticheta(&d.difera_in, limba);
            let chips: Vec<SharedString> = eticheta
                .split(", ")
                .filter(|x| !x.is_empty() && *x != "\u{2014}")
                .map(SharedString::from)
                .collect();
            ui.set_difera_in(ModelRc::new(VecModel::from(chips)));
        }
        None => {
            ui.set_plus(SharedString::new());
            ui.set_minus(SharedString::new());
            ui.set_difera_in(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
        }
    }
    // coloana e arborele din lista (Object Explorer, Mihai 2026-11-19):
    // randurile raman in ordinea listei, nodurile le aleg dupa `pozitie`.
    // Numele incape in latimea coloanei: ce ramane dupa indentarea
    // obiectului (2 niveluri), cerc, litera si +/-
    let caractere = ((ui.get_latime_lista() - 140.0) / 7.3).max(8.0) as usize;
    let randuri: Vec<RandListaUi> = randuri_lista_ui(lista, limba)
        .into_iter()
        .map(|mut r| {
            r.nume_scurt = model_taburi::taie_pe_mijloc(&r.nume, caractere).into();
            r
        })
        .collect();
    if let Some(m) = pe_loc(ui.get_randuri_lista(), randuri) {
        ui.set_randuri_lista(m);
    }
    let (noduri, nod_selectat) = noduri_ui(arbore, lista, limba);
    if let Some(m) = pe_loc(ui.get_noduri_lista(), noduri) {
        ui.set_noduri_lista(m);
    }
    ui.set_nod_selectat_lista(nod_selectat);
    ui.set_banda_text(
        lista
            .contoare_stari()
            .iter()
            .filter(|(s, n)| s.difera() && *n > 0)
            .map(|(s, n)| format!("{}\n{n}", s.litera()))
            .collect::<Vec<_>>()
            .join("\n\n")
            .into(),
    );
}

/// se cheama o data, la pornire.
pub fn incarca_editor(ui: &EditorDiferente, antet: &AntetEditor, model: &ModelEditor) {
    let (randuri, start, tinte) = randuri_ui_cu_tinte(
        model,
        ui.get_pliaza_identicul(),
        !ui.get_alaturat(),
        limba_curenta(ui),
        ui.get_impacheteaza()
            .then(|| usize::try_from(ui.get_coloane_wrap()).unwrap_or(0)),
    );
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
    if ui.get_obiect_id() != antet.id {
        // alt obiect: editarea de mana nu se duce cu el
        ui.set_editeaza_mod(false);
        ui.set_obiect_id(antet.id.into());
    }
    ui.set_obiect_schema(antet.schema.into());
    ui.set_obiect_nume(antet.nume.into());
    ui.set_tip_obiect(antet.tip.into());
    ui.set_in_plan(antet.in_plan);
    let limba = limba_curenta(ui);
    ui.set_rezumat(numar_diferente(model.diferente.len(), limba).into());
    ui.set_sursa_baza(antet.sursa_baza.into());
    ui.set_sursa_detaliu(detaliu_panou(antet.sursa_server, linii(model, true), limba).into());
    ui.set_tinta_baza(antet.tinta_baza.into());
    ui.set_tinta_detaliu(detaliu_panou(antet.tinta_server, linii(model, false), limba).into());
    ui.set_stare_obiecte(antet.stare_obiecte.into());
    ui.set_revizuit(antet.revizuit);
    ui.set_randuri(ModelRc::new(VecModel::from(randuri)));
    ui.set_marcaje(ModelRc::new(VecModel::from(marcaje)));
    ui.set_total_diferente(model.diferente.len() as i32);
    ui.set_start_diferente(ModelRc::new(VecModel::from(start)));
    ui.set_tinte_de_la(ModelRc::new(VecModel::from(
        tinte.iter().map(|t| t.0).collect::<Vec<_>>(),
    )));
    ui.set_tinte_pana_la(ModelRc::new(VecModel::from(
        tinte.iter().map(|t| t.1).collect::<Vec<_>>(),
    )));
    ui.set_diferenta_curenta(0);
    // cu wrap nu mai e nimic de derulat pe orizontala
    ui.set_latime_continut(if ui.get_impacheteaza() {
        0.0
    } else {
        latime_continut(model)
    });
    ui.set_derulare_x(0.0);
}

/// Latimea celei mai lungi linii, in pixeli logici.
///
/// Liniile mai lungi decat panoul erau taiate fara elipsa si fara derulare:
/// o diferenta la coloana 90 era invizibila intr-o unealta de diff (V10 din
/// auditul QA). Fereastra deruleaza pe orizontala, si are nevoie sa stie pana
/// unde. Latimea unui caracter mono de 12,5 px e estimata, ca in `harta`:
/// Slint nu da latimea unui text fara sa-l deseneze, iar o supraestimare mica
/// inseamna doar cateva pixeli de derulat in gol.
fn latime_continut(model: &ModelEditor) -> f32 {
    const CARACTER: f32 = CARACTER_MONO;
    let cele_mai_lungi = model
        .randuri
        .iter()
        .map(|r| {
            r.stanga
                .text()
                .chars()
                .count()
                .max(r.dreapta.text().chars().count())
        })
        .max()
        .unwrap_or(0);
    cele_mai_lungi as f32 * CARACTER
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
    /// Ce nu se inchide in textul final (etapa 26): gol daca nu e text final
    /// sau daca e in regula.
    pub probleme: &'a [gmotion_core::structura::Problema],
}

/// Ce nu se inchide in textul final, pentru om: prima problema, cu linia, si
/// cate mai sunt. Gol daca nu e nicio problema.
pub fn text_probleme(p: &[gmotion_core::structura::Problema], l: texte::Limba) -> String {
    use gmotion_core::structura::FelProblema as F;
    let Some(prima) = p.first() else {
        return String::new();
    };
    let ce = match &prima.fel {
        F::DeschisNeinchis(b) => format!(
            "{} {} {}",
            b.cod(),
            l.alege("fara", "without"),
            b.inchidere()
        ),
        F::InchisFaraDeschidere(b) => format!(
            "{} {}",
            b.inchidere(),
            l.alege("in plus, fara deschidere", "too many, nothing to close")
        ),
        F::InchisAltceva { asteptat, gasit } => format!(
            "{} {} {}",
            gasit.inchidere(),
            l.alege("inchide un", "closes a"),
            asteptat.cod()
        ),
        F::ParantezaNeinchisa => l
            .alege("o paranteza ( nu se inchide", "a ( is never closed")
            .into(),
        F::ParantezaInPlus => l.alege("o paranteza ) in plus", "an extra )").into(),
        F::SirNeinchis => l
            .alege(
                "un sir '...' nu se inchide",
                "a '...' string is never closed",
            )
            .into(),
        F::IdentificatorNeinchis => l
            .alege(
                "un nume [...] nu se inchide",
                "a [...] name is never closed",
            )
            .into(),
        F::ComentariuNeinchis => l
            .alege(
                "un comentariu /* nu se inchide",
                "a /* comment is never closed",
            )
            .into(),
        F::Go => l
            .alege("GO in mijlocul obiectului", "GO inside the object")
            .into(),
    };
    let rest = match p.len() {
        1 => String::new(),
        n => format!(" (+{} {})", n - 1, l.alege("altele", "more")),
    };
    format!(
        "{}: {} {}: {ce}{rest}",
        l.alege(
            "Textul final nu se mai compileaza",
            "The final text no longer compiles"
        ),
        l.alege("linia", "line"),
        prima.linie
    )
}

/// Pune starea editarii in editor. Se cheama dupa [`incarca_editor`].
pub fn incarca_editarea(ui: &EditorDiferente, e: &StareEditare) {
    ui.set_omis(e.omis);
    ui.set_editat(e.editat);
    ui.set_se_poate_anula(e.se_poate_anula);
    ui.set_text_sursa(e.text_sursa.into());
    ui.set_text_final(e.text_final.into());
    ui.set_salvat(e.salvat.into());
    ui.set_probleme_structura(text_probleme(e.probleme, limba_curenta(ui)).into());
    let total = ui.get_total_diferente().max(0) as usize;
    match e.ghidat {
        Some((din_sursa, aplicate, sarite, terminat)) => {
            ui.set_ghidat(true);
            ui.set_ghidat_din_sursa(din_sursa);
            ui.set_ghidat_terminat(terminat);
            ui.set_ghidat_titlu(
                match (terminat, limba_curenta(ui)) {
                    (true, texte::Limba::Romana) => format!(
                        "Mod ghidat incheiat: {aplicate} aplicate, {sarite} lasate cum erau"
                    ),
                    (true, texte::Limba::Engleza) => format!(
                        "Guided mode finished: {aplicate} applied, {sarite} left as they were"
                    ),
                    (false, l) => format!(
                        "{} {} {} {total}",
                        l.alege("Mod ghidat, diferenta", "Guided mode, difference"),
                        e.diferenta + 1,
                        l.alege("din", "of")
                    ),
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
        Ordine::FolositDe => 8,
        Ordine::Impact => 9,
        Ordine::Modificat => 10,
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
        8 => Ordine::FolositDe,
        9 => Ordine::Impact,
        10 => Ordine::Modificat,
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

/// Latimea de care are nevoie o coloana ca sa-si arate tot continutul, in
/// pixeli logici („autofit", dublu clic pe marginea capului).
///
/// Latimea unui text nu se poate masura din Slint, deci se socoteste din
/// numarul de caractere si latimea unui caracter mono la 12px. E o
/// aproximare deliberata: mai bine cu cateva zeci de pixeli in plus decat un
/// nume taiat.
pub fn latime_autofit(lista: &model_lista::Lista, coloana: model_lista::Ordine) -> f32 {
    use model_lista::Ordine;
    const CARACTER: f32 = 7.3;
    const PADDING: f32 = 20.0;
    let text = |o: &model_lista::Obiect| -> usize {
        match coloana {
            Ordine::Nume => o.id().chars().count(),
            Ordine::Baza => o.baza.chars().count(),
            Ordine::Tip => gmotion_core::nume_tip(&o.tip).chars().count(),
            Ordine::Linii => o.detalii.as_ref().map_or(0, |d| {
                format!("+{} −{}", d.linii_plus, d.linii_minus)
                    .chars()
                    .count()
            }),
            Ordine::Scor => 10,
            Ordine::DiferaIn => o
                .detalii
                .as_ref()
                .map_or(0, |d| d.difera_in.chars().count()),
            Ordine::Revizuire => 8,
            Ordine::Deploy => "Extra · fara deploy".chars().count(),
            Ordine::FolositDe => o.folosit_de.map_or(1, |n| n.to_string().len()),
            Ordine::Impact => "Ridicat".len(),
            Ordine::Modificat => o.modificat.as_deref().map_or(0, |m| m.chars().count()),
        }
    };
    // doar randurile vizibile: autofit pe ce se vede, ca in orice tabel
    let maxim = lista
        .vizibile()
        .iter()
        .map(|&i| text(&lista.obiecte()[i]))
        .max()
        .unwrap_or(0);
    // si capul coloanei trebuie sa incapa
    let cap = match coloana {
        Ordine::Nume => "Obiect",
        Ordine::Baza => "Baza",
        Ordine::Tip => "Tip",
        Ordine::Linii => "Linii",
        Ordine::Scor => "Similar",
        Ordine::DiferaIn => "Difera in",
        Ordine::Revizuire => "Revizuit",
        Ordine::Deploy => "Deploy",
        Ordine::FolositDe => "Folosit de",
        Ordine::Impact => "Impact",
        Ordine::Modificat => "Ultima modificare",
    }
    .chars()
    .count();
    (maxim.max(cap) as f32) * CARACTER + PADDING
}

/// Umple ecranul listei din modelul ei. Nu calculeaza nimic: filtrele,
/// contoarele si selectia sunt deja decise in `model_lista`.
pub fn incarca_lista(ui: &FereastraLista, antet: &AntetLista, lista: &model_lista::Lista) {
    use model_lista::Stare;
    // etichetele vin din `Texte`, nu din constante in Rust: altfel bara
    // laterala si filtrele ar ramane intr-o singura limba
    let t = ui.global::<Texte>();
    let categorie = |s: Stare| -> SharedString {
        match s {
            Stare::Modificat => t.get_categorie_modificate(),
            Stare::DoarSursa => t.get_categorie_doar_martor(),
            Stare::DoarBaza => t.get_categorie_doar_modificata(),
            Stare::Identic => t.get_categorie_identice(),
            Stare::Criptat => t.get_categorie_criptate(),
        }
    };
    let eticheta_filtru = |f: model_lista::FiltruRevizuire| -> SharedString {
        use model_lista::FiltruRevizuire as F;
        match f {
            F::Nerevizuite => t.get_filtru_nerevizuite(),
            F::Revizuite => t.get_filtru_revizuite(),
            F::Omise => t.get_filtru_omise(),
            F::InPlan => t.get_filtru_in_plan(),
            F::Deployate => t.get_filtru_deployate(),
            F::Nedeployate => t.get_filtru_nedeployate(),
        }
    };
    let randuri = randuri_lista_ui(lista, limba_curenta(ui));
    let cod_categorie = |s: Stare| match s {
        Stare::Modificat => 0,
        Stare::DoarSursa => 1,
        Stare::DoarBaza => 2,
        Stare::Identic => 3,
        Stare::Criptat => 4,
    };
    let categorii: Vec<CategorieUi> = lista
        .contoare_stari()
        .into_iter()
        .map(|(s, n)| CategorieUi {
            litera: s.litera().into(),
            nume: categorie(s),
            numar: n.to_string().into(),
            stare: cod_categorie(s),
        })
        .collect();
    let tipuri: Vec<TipUi> = lista
        .contoare_tipuri()
        .into_iter()
        .map(|(t, n)| TipUi {
            nume: nume_tip_plural(&t, limba_curenta(ui)).into(),
            numar: n.to_string().into(),
        })
        .collect();
    let (revizuite, total) = lista.progres_revizuire();

    ui.set_tigla_hasura(tigla_hasura(mod_curent(ui)));
    ui.set_sursa_baza(antet.sursa_baza.into());
    ui.set_tinta_baza(antet.tinta_baza.into());
    ui.set_servere(antet.servere.into());
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
    // coloana Baza: dupa bazele comparate, nu dupa cele cu diferente. Cu
    // doua baze si diferente doar intr-una, coloana disparea, iar randurile
    // nu mai spuneau din ce baza sunt (F-A17).
    ui.set_mai_multe_baze(
        baze.len() > 1
            || lista
                .obiecte()
                .iter()
                .map(|o| o.baza.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                > 1,
    );
    let (nerevizuite, vizibile) = lista.rezumat_vizibile();
    ui.set_rezumat_lista(format!("{nerevizuite} {} {vizibile}", t.get_nerevizuite_din()).into());
    ui.set_coloana_sortata(cod_ordine(lista.ordine()));
    ui.set_sens_descrescator(lista.descrescator());
    ui.set_stari(ModelRc::new(VecModel::from(
        model_lista::FiltruRevizuire::TOATE
            .iter()
            .map(|&f| eticheta_filtru(f))
            .collect::<Vec<_>>(),
    )));
    ui.set_impacturi(ModelRc::new(VecModel::from(
        model_lista::Impact::TOATE
            .iter()
            .map(|i| match i {
                model_lista::Impact::Ridicat => t.get_impact_ridicat(),
                model_lista::Impact::Mediu => t.get_impact_mediu(),
                model_lista::Impact::Scazut => t.get_impact_scazut(),
            })
            .collect::<Vec<_>>(),
    )));
    ui.set_impact_activ(lista.filtru().impact.map_or(-1, |i| {
        model_lista::Impact::TOATE
            .iter()
            .position(|&x| x == i)
            .map_or(-1, |p| p as i32)
    }));
    // etapa 27: operatia si perioadele
    {
        use model_lista::{Operatie, Perioada};
        let lista_de = |v: Vec<SharedString>| ModelRc::new(VecModel::from(v));
        ui.set_operatii(lista_de(vec![
            t.get_op_create(),
            t.get_op_alter(),
            t.get_op_drop(),
        ]));
        ui.set_perioade_creat(lista_de(vec![
            t.get_creat_azi(),
            t.get_creat_7(),
            t.get_creat_30(),
        ]));
        ui.set_perioade_modificat(lista_de(vec![
            t.get_modificat_azi(),
            t.get_modificat_7(),
            t.get_modificat_30(),
        ]));
        let poz_p = |p: Option<&(Perioada, String)>| {
            p.and_then(|(p, _)| Perioada::TOATE.iter().position(|x| x == p))
                .map_or(-1, |i| i as i32)
        };
        ui.set_creat_activ(poz_p(lista.filtru().creat.as_ref()));
        ui.set_modificat_activ(poz_p(lista.filtru().modificat.as_ref()));
        ui.set_operatie_activa(
            lista
                .filtru()
                .operatie
                .and_then(|o| Operatie::TOATE.iter().position(|x| *x == o))
                .map_or(-1, |i| i as i32),
        );
    }
    ui.set_stare_activa(lista.filtru().revizuire.map_or(-1, |f| {
        model_lista::FiltruRevizuire::TOATE
            .iter()
            .position(|&x| x == f)
            .map_or(-1, |i| i as i32)
    }));
    // campul cautarii urmeaza filtrul: dupa o conectare noua nu mai arata
    // textul de dinainte, care nu mai filtreaza nimic (F-A16). Doar cand
    // difera, ca scrisul in camp sa nu-si piarda cursorul.
    if ui.get_cautare().as_str() != lista.filtru().cautare {
        ui.set_cautare(lista.filtru().cautare.as_str().into());
    }
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
    if let Some(m) = pe_loc(ui.get_randuri(), randuri) {
        ui.set_randuri(m);
    }
    // harta deschisa urmeaza lista: revizuirea, filtrele, selectia
    if ui.get_arata_harta() {
        incarca_harta(ui, lista.selectat());
    }
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
                    .map_or(String::new(), |d| {
                        let l = limba_curenta(ui);
                        format!(
                            "{} {}",
                            l.alege("difera in", "differs in"),
                            gmotion_core::clasificare::traduce_eticheta(&d.difera_in, l)
                        )
                    })
                    .into(),
            );
        }
        None => {
            ui.set_obiect_schema(SharedString::new());
            ui.set_obiect_nume(SharedString::new());
            ui.set_ce_difera(SharedString::new());
            ui.set_obiect_revizuit(false);
            // nimic selectat: butonul nu poate ramane „In plan ✓" de la
            // obiectul de dinainte (F-A16)
            ui.set_obiect_in_plan(false);
        }
    }
}

/// Nodurile arborelui pentru `.slint` si randul obiectului selectat (-1 cand
/// nu se vede), pentru coloana din stanga a editorului.
fn noduri_ui(
    arbore: &model_arbore::Arbore,
    lista: &model_lista::Lista,
    limba: texte::Limba,
) -> (Vec<NodUi>, i32) {
    let noduri = arbore.noduri(lista, limba);
    let selectat = lista.selectat();
    let nod_selectat = noduri
        .iter()
        .position(|n| n.pozitie.is_some() && n.pozitie == selectat)
        .map_or(-1, |i| i as i32);
    let v: Vec<NodUi> = noduri
        .into_iter()
        .map(|n| NodUi {
            nivel: i32::from(n.nivel),
            fel: match n.fel {
                model_arbore::Fel::Baza => 0,
                model_arbore::Fel::Dosar => 1,
                model_arbore::Fel::Obiect => 2,
            },
            eticheta: n.eticheta.into(),
            numar: n.numar as i32,
            deschis: n.deschis,
            cheie: n.cheie.into(),
            pozitie: n.pozitie.map_or(-1, |p| p as i32),
        })
        .collect();
    (v, nod_selectat)
}

/// Ce arata panoul de detalii din lista, pe langa randul selectat. Date
/// simple: fereastra nu stie de module (impactul vine din `diff-obiecte`).
pub struct PanouDetalii<'a> {
    pub deschis: bool,
    /// Baza comparata si `schema.nume` al obiectului; nota se leaga de ele.
    pub baza: &'a str,
    pub obiect: &'a str,
    /// 0 scazut, 1 mediu, 2 ridicat; `None` = nu se stie. Eticheta vine din
    /// `Texte`, in limba aleasa.
    pub impact: Option<i32>,
    pub motive: &'a str,
    pub nota: &'a str,
    /// Cine foloseste obiectul si ce foloseste el (`schema.nume`).
    pub folosit_de: &'a [String],
    pub foloseste: &'a [String],
}

/// Cate nume de dependente incap in panou; restul se spun ca numar.
const NUME_IN_PANOU: usize = 8;

/// Primele `NUME_IN_PANOU` nume, si „+N" pentru cele ramase.
fn nume_taiate(nume: &[String]) -> (ModelRc<SharedString>, SharedString) {
    let arata: Vec<SharedString> = nume
        .iter()
        .take(NUME_IN_PANOU)
        .map(|n| n.as_str().into())
        .collect();
    let rest = nume.len().saturating_sub(NUME_IN_PANOU);
    (
        ModelRc::new(VecModel::from(arata)),
        if rest == 0 {
            SharedString::new()
        } else {
            format!("+{rest}").into()
        },
    )
}

/// Umple panoul de detalii. Nota se pune doar cand s-a schimbat obiectul:
/// rescrisa la fiecare tasta, ar muta cursorul si ar taia spatiul de la
/// capat pe care tocmai l-ai scris.
pub fn incarca_detalii(ui: &FereastraLista, p: &PanouDetalii) {
    ui.set_detalii_deschise(p.deschis);
    ui.set_impact_nivel(p.impact.unwrap_or(-1));
    ui.set_impact_motive(p.motive.into());
    let (folosit, rest) = nume_taiate(p.folosit_de);
    ui.set_folosit_de_lista(folosit);
    ui.set_folosit_de_rest(rest);
    let (foloseste, rest) = nume_taiate(p.foloseste);
    ui.set_foloseste_lista(foloseste);
    ui.set_foloseste_rest(rest);
    if ui.get_nota_id().as_str() != p.obiect || ui.get_nota_baza().as_str() != p.baza {
        ui.set_nota_baza(p.baza.into());
        ui.set_nota_id(p.obiect.into());
        ui.set_nota(p.nota.into());
    }
}

/// Numele tipului la plural, pentru filtrele de tip („Proceduri"), ca in
/// machete. Pe codul din `sys.objects`, nu pe numele singular: cele trei
/// feluri de functii (`FN`, `IF`, `TF`) au filtre separate, si toate trei
/// scriau „Functii" (sesizat de Mihai).
fn nume_tip_plural(tip: &str, l: texte::Limba) -> &'static str {
    match tip.trim().to_ascii_uppercase().as_str() {
        "P" => l.alege("Proceduri", "Procedures"),
        "V" => l.alege("View-uri", "Views"),
        "FN" => l.alege("Functii scalare", "Scalar functions"),
        "IF" => l.alege("Functii inline", "Inline functions"),
        "TF" => l.alege("Functii tabel", "Table functions"),
        "TR" => l.alege("Triggere", "Triggers"),
        "U" => l.alege("Tabele", "Tables"),
        _ => l.alege("Alte obiecte", "Other objects"),
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
    // Lista se redeseneaza la fiecare comanda (filtru, clic, revizuit), iar
    // diff-ul unui obiect mare costa peste o secunda (proba pe Windows,
    // 2026-11-19: 1,2 s la un filtru). Se recalculeaza doar cand obiectul
    // sau limba s-au schimbat, ori latimea de wrap.
    /// Textele, setarile, limba si wrap-ul din care s-a calculat ultima
    /// previzualizare.
    type Calculata = (String, String, Vec<String>, texte::Limba, Option<usize>);
    thread_local! {
        static ULTIMA: std::cell::RefCell<Option<Calculata>> =
            const { std::cell::RefCell::new(None) };
    }
    let limba = limba_curenta(ui);
    // wrap-ul e doar in editor; previzualizarea din panou ramane neimpachetata
    let wrap: Option<usize> = None;
    let acelasi = ULTIMA.with_borrow(|u| {
        u.as_ref().is_some_and(|(a, b, s, l, w)| {
            a == text_sursa
                && b == text_baza
                && s.iter().map(String::as_str).eq(setari.iter().copied())
                && *l == limba
                && *w == wrap
        })
    });
    if acelasi && slint::Model::row_count(&ui.get_previzualizare()) > 0 {
        return;
    }
    let model = model_editor::construieste(text_sursa, text_baza);
    let (randuri, _) = randuri_ui(&model, true, false, limba, wrap);
    ui.set_previzualizare(ModelRc::new(VecModel::from(randuri)));
    ui.set_ce_difera(gmotion_core::clasificare::propozitie(text_sursa, text_baza, setari).into());
    ULTIMA.with_borrow_mut(|u| {
        *u = Some((
            text_sursa.to_string(),
            text_baza.to_string(),
            setari.iter().map(|x| x.to_string()).collect(),
            limba,
            wrap,
        ))
    });
}

/// Goleste previzualizarea (niciun obiect selectat).
pub fn goleste_previzualizarea(ui: &FereastraLista) {
    ui.set_previzualizare(ModelRc::new(VecModel::from(Vec::<RandUi>::new())));
}

/// Ce scrie in campul bazelor cand se pot bifa mai multe: numele, daca sunt
/// una sau doua, altfel „N baze" — un sir lung ar iesi din camp oricum.
fn eticheta_baze(p: &model_conectare::Parte, l: texte::Limba) -> String {
    let alese = p.baze_alese();
    match alese.len() {
        0 => String::new(),
        1 | 2 => alese.join(", "),
        n => l.numar(n, ("baza", "baze"), ("database", "databases")),
    }
}

/// Umple ecranul de conectare din modelul lui.
/// Taburile editorului, cum se deseneaza: fixatele in ordine, apoi
/// previzualizarea (inclinata), ca in macheta. `editat` spune care au text
/// final (punctul de pe tab).
pub fn taburi_ui(
    t: &model_taburi::Taburi,
    editat: impl Fn(&model_taburi::Cheie) -> bool,
) -> Vec<TabUi> {
    use model_taburi::Activ;
    let ui = |c: &model_taburi::Cheie, cod: i32, activ: bool, prev: bool| {
        let scurt = c.1.split_once('.').map_or(c.1.as_str(), |(_, n)| n);
        TabUi {
            cod,
            nume: model_taburi::taie_pe_mijloc(scurt, 22).into(),
            complet: format!("{} / {}", c.0, c.1).into(),
            activ,
            previzualizare: prev,
            editat: editat(c),
        }
    };
    let mut v: Vec<TabUi> = t
        .fixate()
        .into_iter()
        .enumerate()
        .map(|(i, c)| ui(c, i as i32, t.activ() == Activ::Fixat(i), false))
        .collect();
    if let Some(c) = t.previzualizare() {
        v.push(ui(c, -1, t.activ() == Activ::Previzualizare, true));
    }
    v
}

/// Pune taburile in editor.
pub fn incarca_taburi(
    ui: &EditorDiferente,
    t: &model_taburi::Taburi,
    editat: impl Fn(&model_taburi::Cheie) -> bool,
) {
    ui.set_taburi(ModelRc::new(VecModel::from(taburi_ui(t, editat))));
}

/// Umple ecranul de deploy. Numarul mare si propozitia vin din pasi
/// (`model_deploy::rezumat`), ca sa nu poata spune altceva decat lista.
pub fn incarca_deploy(ui: &FereastraDeploy, e: &model_deploy::EcranDeploy) {
    use model_deploy::{GardaDeploy, RezultatDeploy};
    ui.set_referinta(e.referinta.as_str().into());
    ui.set_tinta(e.tinta.as_str().into());
    ui.set_numar(e.pasi.len().to_string().into());
    ui.set_rezumat(model_deploy::rezumat(&e.pasi, limba_curenta(ui)).into());
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
                tip: gmotion_core::nume_tip_in(&p.tip, limba_curenta(ui)).into(),
                id: p.obiect.as_str().into(),
                baza: p.baza.as_str().into(),
            }
        })
        .collect();
    ui.set_pasi(ModelRc::new(VecModel::from(pasi)));
    ui.set_sarite(
        match e.sarite {
            0 => String::new(),
            1 => limba_curenta(ui)
                .alege(
                    "Un obiect ales nu intra: exista doar in baza modificata sau e criptat.",
                    "One chosen object is left out: it exists only in the modified database or is encrypted.",
                )
                .into(),
            n => match limba_curenta(ui) {
                texte::Limba::Romana => format!(
                    "{n} obiecte alese nu intra: exista doar in baza modificata sau sunt criptate."
                ),
                texte::Limba::Engleza => format!(
                    "{n} chosen objects are left out: they exist only in the modified database or are encrypted."
                ),
            },
        }
        .into(),
    );
    let (fel, schimbari) = match &e.garda {
        GardaDeploy::Neverificata => (0, Vec::new()),
        GardaDeploy::SeVerifica => (4, Vec::new()),
        GardaDeploy::Ok { .. } => (1, Vec::new()),
        GardaDeploy::Schimbata(v) => (2, v.iter().take(6).cloned().collect()),
        GardaDeploy::Eroare(_) => (3, Vec::new()),
    };
    ui.set_garda(fel);
    ui.set_text_garda(model_deploy::text_garda(&e.garda, &e.tinta, limba_curenta(ui)).into());
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
    ui.set_text_rezultat(model_deploy::text_rezultat(&e.rezultat, limba_curenta(ui)).into());
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

/// Leaga scara unica de fereastra de conectare (etapa 17, `scara.rs`): la
/// fiecare redimensionare se recalculeaza, dupa ce Slint a terminat
/// evenimentul curent (schimbata in mijlocul lui, ar redimensiona fereastra
/// din interiorul propriei redimensionari).
pub fn leaga_scara(ui: &FereastraConectare) {
    let scara = std::rc::Rc::new(scara::ScaraFereastra::noua());
    let slaba = ui.as_weak();
    let aplica = move || {
        let s = scara.clone();
        let slaba = slaba.clone();
        slint::Timer::single_shot(std::time::Duration::ZERO, move || {
            if let Some(ui) = slaba.upgrade() {
                s.aplica(ui.window());
                let logic = ui.window().size().to_logical(ui.window().scale_factor());
                ui.set_scund(logic.height < 820.0);
            }
        });
    };
    aplica();
    ui.on_dimensiune_schimbata(aplica);
}

pub fn incarca_conectare(ui: &FereastraConectare, c: &model_conectare::Conectare) {
    use model_conectare::StareConexiune;
    let fel = |s: &StareConexiune| match s {
        StareConexiune::Neincercat => 0,
        StareConexiune::SeConecteaza | StareConexiune::SeCauta => 1,
        StareConexiune::Conectat(_) => 2,
        StareConexiune::Eroare(_) => 3,
    };
    let parte = |p: &model_conectare::Parte, eticheta: &str, rol: &str| ParteUi {
        eticheta: eticheta.into(),
        rol: rol.into(),
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
        eticheta_baze: eticheta_baze(p, limba_curenta(ui)).into(),
        stare: p.stare.text_in(limba_curenta(ui)).into(),
        fel_stare: fel(&p.stare),
        servere_gasite: ModelRc::new(VecModel::from(
            p.servere_gasite
                .iter()
                .map(|s| ServerGasitUi {
                    nume: s.nume.as_str().into(),
                    cluster: s.cluster,
                    implicita: s.implicita,
                })
                .collect::<Vec<_>>(),
        )),
    };
    let recente: Vec<RecentaUi> = c
        .recente
        .iter()
        .map(|r| RecentaUi {
            sursa: r.sursa.as_str().into(),
            baza: r.baza.as_str().into(),
            cand: r.cand.as_str().into(),
            progres: r.progres.as_str().into(),
        })
        .collect();
    // numele panourilor vin din `Texte`, ca sa urmeze limba aleasa
    let t = ui.global::<Texte>();
    ui.set_sursa(parte(&c.sursa, &t.get_martor(), &t.get_martor_explicatie()));
    ui.set_baza(parte(
        &c.baza,
        &t.get_modificata(),
        &t.get_modificata_explicatie(),
    ));
    ui.set_certificat(c.text_certificat_in(limba_curenta(ui)).into());
    ui.set_aceeasi_conexiune(c.aceeasi_conexiune && c.instantaneu.is_none());
    ui.set_baza_blocata(
        c.baza
            .baze
            .iter()
            .find(|b| c.e_baza_martorului(b))
            .map(String::as_str)
            .unwrap_or("")
            .into(),
    );
    ui.set_nota_instanta(
        // cu „Aceeasi conexiune" e evident: nu se mai spune
        if c.aceeasi_instanta() && !c.aceeasi_conexiune {
            limba_curenta(ui).alege(
                "Martorul si modificata sunt pe aceeasi instanta",
                "The witness and the modified database are on the same instance",
            )
        } else {
            ""
        }
        .into(),
    );
    ui.set_eroare_comparare(c.eroare_comparare.as_deref().unwrap_or("").into());
    ui.set_instantaneu(c.instantaneu.as_deref().unwrap_or("").into());
    ui.set_recente(ModelRc::new(VecModel::from(recente)));
    ui.set_ce_lipseste(
        c.ce_lipseste_in(limba_curenta(ui))
            .unwrap_or_default()
            .into(),
    );
}

/// Latimea si inaltimea in care se aseaza harta: ecranul ei, fara capul de
/// 60 px si marginile cartusului (16 + 16 in afara, 18 + 18 / 16 + 16
/// inauntru, vezi `lista.slint`). Inainte de prima afisare, 1440 x 900.
fn loc_harta(ui: &FereastraLista) -> (f32, f32) {
    let w = ui.window();
    let m = w.size().to_logical(w.scale_factor());
    let (latime, inaltime) = if m.width > 0.0 {
        (m.width, m.height)
    } else {
        (1440.0, 900.0)
    };
    (
        (latime - 68.0).max(200.0),
        (inaltime - 60.0 - 16.0 - 16.0 - 32.0).max(120.0),
    )
}

/// Harta (`h`): obiectele din lista, pe tipuri, pictate in culorile modului
/// curent, cu conturul pe patratul obiectului selectat. Citeste randurile
/// deja puse in fereastra, ca harta sa arate exact ce arata lista.
pub fn incarca_harta(ui: &FereastraLista, selectat: Option<usize>) {
    use slint::Model;
    let randuri = ui.get_randuri();
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
    let (latime, inaltime) = loc_harta(ui);
    let (grupuri, total) = harta::aseaza(pe_tip, latime, inaltime);
    let paleta = mod_curent(ui).paleta();
    // la scalarea ecranului (125-200%), pictata in pixeli logici harta iesea
    // neclara: se picteaza la pixelii fizici si se afiseaza la marimea logica
    let factor = ui.window().scale_factor().max(1.0);
    ui.set_factor_harta(factor);
    let ui_grupuri: Vec<GrupHartaUi> = grupuri
        .iter()
        .map(|g| GrupHartaUi {
            titlu: g.titlu.as_str().into(),
            numar: g.celule.len().to_string().into(),
            x: g.x,
            y: g.y,
            latime: g.latime,
            inaltime: g.inaltime,
            desparte: g.x > 0.0,
            coloane: g.coloane as i32,
            imagine: imagine(harta::picteaza_la(g, paleta, factor)),
            bara: imagine(harta::picteaza_bara_la(g, paleta, factor)),
            randuri: ModelRc::new(VecModel::from(
                g.celule.iter().map(|c| c.rand as i32).collect::<Vec<_>>(),
            )),
        })
        .collect();
    let sel = selectat.and_then(|s| harta::pozitie_randului(&grupuri, s));
    ui.set_sel_pe_harta(sel.is_some());
    if let Some((x, y)) = sel {
        ui.set_sel_x(x);
        ui.set_sel_y(y);
    }
    // o singura treapta pe toata harta: fereastra tine doar latura ei
    let scara = grupuri.first().map_or(harta::SCARI[0], |g| g.scara);
    ui.set_patrat(scara.latura as f32);
    ui.set_harta(ModelRc::new(VecModel::from(ui_grupuri)));
    ui.set_harta_inaltime(total);
}

/// O imagine Slint din pixelii pictati de `harta`.
fn imagine((w, h, px): (u32, u32, Vec<[u8; 4]>)) -> Image {
    let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
    for (d, s) in buf.make_mut_slice().iter_mut().zip(&px) {
        *d = Rgba8Pixel {
            r: s[0],
            g: s[1],
            b: s[2],
            a: s[3],
        };
    }
    Image::from_rgba8(buf)
}

/// Randul din lista al patratului de la (x, y) din grupul `grup` al hartii,
/// in coordonatele imaginii grupului. Calculul e in `harta::indice_la`.
pub fn rand_din_harta(ui: &FereastraLista, grup: i32, x: f32, y: f32) -> Option<usize> {
    use slint::Model;
    let g = ui.get_harta().row_data(usize::try_from(grup).ok()?)?;
    let k = harta::indice_la(scara_hartii(ui)?, u32::try_from(g.coloane).ok()?, x, y)?;
    usize::try_from(g.randuri.row_data(k)?).ok()
}

/// Treapta de zoom a hartii afisate, din latura pusa in fereastra.
fn scara_hartii(ui: &FereastraLista) -> Option<harta::Scara> {
    harta::Scara::cu_latura(ui.get_patrat().round() as u32)
}

/// Obiectul selectat, daca e unul din randurile afisate.
fn selectat_in_fereastra(ui: &FereastraLista) -> Option<usize> {
    use slint::Model;
    usize::try_from(ui.get_selectat())
        .ok()
        .filter(|&s| s < ui.get_randuri().row_count())
}

/// Leaga harta: deschiderea, reasezarea la redimensionare si bula de sub
/// mouse. Nu tin de starea aplicatiei, deci se leaga aici, o data, dupa
/// crearea ferestrei. Clicul pe patrat schimba selectia, deci e al `app`.
/// Evidentierea de sintaxa cat scrii (etapa 25): la fiecare schimbare a
/// campului de editare, randurile colorate de peste el se refac din text.
/// Peste [`model_editor::LINII_SINTAXA_LA_SCRIERE`] linii raman goale, deci
/// se vede textul simplu al campului.
pub fn leaga_editarea(ui: &EditorDiferente) {
    let slab = ui.as_weak();
    ui.on_text_editat(move |text| {
        if let Some(ui) = slab.upgrade() {
            ui.set_ed_linii(linii_editare_ui(&text));
        }
    });
}

/// Randurile colorate pentru campul de editare.
pub fn linii_editare_ui(text: &str) -> ModelRc<LinieUi> {
    ModelRc::new(VecModel::from(
        model_editor::linii_sintaxa(text)
            .iter()
            .map(|l| {
                let mut caractere = 0;
                let mut taburi = 0;
                let bucati: Vec<BucataEditareUi> = l
                    .iter()
                    .map(|b| {
                        let ui = BucataEditareUi {
                            text: SharedString::from(b.text.as_str()),
                            sintaxa: cod_sintaxa(b.sintaxa),
                            caractere,
                            taburi,
                        };
                        for c in b.text.chars() {
                            if c == '\t' {
                                taburi += 1;
                            } else {
                                caractere += 1;
                            }
                        }
                        ui
                    })
                    .collect();
                LinieUi {
                    bucati: ModelRc::new(VecModel::from(bucati)),
                }
            })
            .collect::<Vec<_>>(),
    ))
}

pub fn leaga_harta(ui: &FereastraLista) {
    let slab = ui.as_weak();
    ui.on_deschide_harta(move || {
        let Some(ui) = slab.upgrade() else { return };
        ui.set_bula("".into());
        incarca_harta(&ui, selectat_in_fereastra(&ui));
        ui.set_arata_harta(true);
    });
    let slab = ui.as_weak();
    ui.on_harta_redimensionata(move || {
        let Some(ui) = slab.upgrade() else { return };
        incarca_harta(&ui, selectat_in_fereastra(&ui));
    });
    let slab = ui.as_weak();
    ui.on_peste_harta(move |grup, x, y| {
        use slint::Model;
        let Some(ui) = slab.upgrade() else { return };
        let gasit = usize::try_from(grup).ok().and_then(|gi| {
            let g = ui.get_harta().row_data(gi)?;
            let col = u32::try_from(g.coloane).ok()?;
            let k = harta::indice_la(scara_hartii(&ui)?, col, x, y)?;
            let rand = ui
                .get_randuri()
                .row_data(usize::try_from(g.randuri.row_data(k)?).ok()?)?;
            Some((g, col, k, rand))
        });
        let (Some((g, col, k, r)), Some(scara)) = (gasit, scara_hartii(&ui)) else {
            ui.set_bula("".into());
            return;
        };
        // coltul patratului, in coordonatele hartii; bula se aseaza in Slint,
        // care ii stie latimea
        let (cx, cy) = harta::coltul_in_grila(scara, col, k);
        ui.set_bula(format!("{}.{}", r.schema, r.nume).into());
        ui.set_bula_stare(r.stare);
        ui.set_hover_x(g.x + cx as f32);
        ui.set_hover_y(g.y + harta::CAP + cy as f32);
    });
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
        aplica_limba(&ui, texte::Limba::default());
        ui
    }

    /// 15 B: marginea dintre coloana listei si editor se trage cu mouse-ul,
    /// in limite, si latimea trasa se tine minte.
    #[test]
    fn marginea_coloanei_de_lista_se_trage_cu_mouse_ul() {
        use slint::platform::{PointerEventButton, WindowEvent};
        use slint::LogicalPosition;
        let ui = fereastra();
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));
        ui.show().expect("fereastra se arata");
        let trage = |de_la: f32, la: f32| {
            let (a, b) = (
                LogicalPosition::new(de_la, 450.0),
                LogicalPosition::new(la, 450.0),
            );
            ui.window()
                .dispatch_event(WindowEvent::PointerMoved { position: a });
            ui.window().dispatch_event(WindowEvent::PointerPressed {
                position: a,
                button: PointerEventButton::Left,
            });
            ui.window()
                .dispatch_event(WindowEvent::PointerMoved { position: b });
            ui.window().dispatch_event(WindowEvent::PointerReleased {
                position: b,
                button: PointerEventButton::Left,
            });
        };
        // 320 px implicit: arborele are nevoie de loc pentru nume (etapa 19)
        assert_eq!(ui.get_latime_lista(), 320.0);
        trage(320.0, 340.0);
        assert_eq!(ui.get_latime_lista(), 340.0, "marginea a urmat mouse-ul");
        assert_eq!(ui.get_latime_lista_aleasa(), 340.0, "si se tine minte");
        trage(340.0, 0.0);
        assert_eq!(ui.get_latime_lista(), 200.0, "nu se strange sub 200 px");
        trage(200.0, 1400.0);
        assert_eq!(ui.get_latime_lista(), 520.0, "cel mult 520 px (etapa 19)");
        let _ = ui.hide();
    }

    /// Etapa 16: fixatele intai, previzualizarea la urma; numele scurt,
    /// taiat pe mijloc; codul care se intoarce in Rust la clic.
    #[test]
    fn taburile_se_deseneaza_din_model() {
        use model_taburi::Taburi;
        let mut t = Taburi::noi();
        let k = |n: &str| ("Client1".to_string(), format!("dbo.{n}"));
        t.deschide(k("uspFoarteFoarteFoarteLungCaNumeDeProcedura"));
        t.fixeaza(k("a"), |_| false).unwrap();
        t.deschide(k("p"));
        let v = taburi_ui(&t, |c| c.1 == "dbo.a");
        assert_eq!(v.len(), 2);
        assert_eq!((v[0].cod, v[0].nume.as_str(), v[0].editat), (0, "a", true));
        assert!(v[1].previzualizare && v[1].activ && v[1].cod == -1);
        assert_eq!(v[1].complet, "Client1 / dbo.p");
        let ui = fereastra();
        incarca_taburi(&ui, &t, |_| false);
        assert_eq!(ui.get_taburi().row_count(), 2);
        // un nume lung se taie pe mijloc, la 22 de caractere
        let mut t = Taburi::noi();
        t.deschide(k("uspFoarteFoarteFoarteLungCaNumeDeProcedura"));
        let v = taburi_ui(&t, |_| false);
        assert_eq!(v[0].nume.chars().count(), 22);
        assert!(v[0].nume.ends_with("umeDeProcedura"), "{}", v[0].nume);
    }

    /// Etapa 16: butoanele din capul coloanei (pin, muta, banda) se apasa cu
    /// mouse-ul; fiecare schimba starea si o anunta pentru sesiune.
    #[test]
    fn butoanele_coloanei_se_apasa_cu_mouse_ul() {
        use slint::platform::{PointerEventButton, WindowEvent};
        use slint::LogicalPosition;
        use std::cell::Cell;
        use std::rc::Rc;
        let ui = fereastra();
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));
        ui.show().expect("fereastra se arata");
        let schimbari = Rc::new(Cell::new(0));
        let s = schimbari.clone();
        ui.on_coloana_schimbata(move || s.set(s.get() + 1));
        let clic = |x: f32, y: f32| {
            let position = LogicalPosition::new(x, y);
            ui.window()
                .dispatch_event(WindowEvent::PointerMoved { position });
            ui.window().dispatch_event(WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            });
            ui.window().dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        };
        // capul coloanei: Esc la dreapta, inaintea lui «, ⇄, pinul (24 px
        // fiecare, cu 4 px intre ele); se cauta pinul de la stanga spre Esc
        let x_pin = (100..240)
            .step_by(2)
            .map(|x| x as f32)
            .find(|&x| {
                // capul coloanei e sub bara de sus (56 px), pe randul de 28 px
                clic(x, 56.0 + 10.0 + 14.0);
                let gasit = !ui.get_lista_fixata();
                if gasit {
                    ui.set_lista_fixata(true);
                }
                gasit
            })
            .expect("pinul raspunde la clic");
        assert!(schimbari.get() >= 1, "pinul anunta schimbarea");
        clic(x_pin + 26.0, 80.0);
        assert!(ui.get_lista_dreapta(), "⇄ muta coloana pe dreapta");
        ui.set_lista_dreapta(false);
        clic(x_pin + 52.0, 80.0);
        assert!(ui.get_lista_banda(), "« o strange ca banda");
        let _ = ui.hide();
    }

    /// Apasa o tasta in fereastra, ca un utilizator (apasat si eliberat).
    fn apasa<T: ComponentHandle>(ui: &T, tasta: &str) {
        use slint::platform::WindowEvent;
        let t: SharedString = tasta.into();
        ui.window()
            .dispatch_event(WindowEvent::KeyPressed { text: t.clone() });
        ui.window()
            .dispatch_event(WindowEvent::KeyReleased { text: t });
    }

    /// O tasta cu Ctrl tinut apasat.
    fn cu_ctrl<T: ComponentHandle>(ui: &T, t: &str) {
        use slint::platform::{Key, WindowEvent};
        ui.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Control.into(),
        });
        apasa(ui, t);
        ui.window().dispatch_event(WindowEvent::KeyReleased {
            text: Key::Control.into(),
        });
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
            on_resincronizeaza => "resincronizeaza",
            on_ghidat_porneste => "ghidat",
            on_ghidat_opreste => "opreste",
            on_ghidat_aplica => "aplica",
            on_ghidat_sari => "sari",
            on_anuleaza => "anuleaza",
            on_editeaza_textul_final => "editeaza",
            on_inapoi_lista => "lista",
            on_obiect_urmator => "j",
            on_obiect_anterior => "k",
            on_ascunde_lista => "ctrl+b",
            on_inchide_tab_activ => "ctrl+w",
            on_fixeaza_previzualizarea => "fixeaza-prev",
        );
        // taburile: clic, X, dublu clic pe un rand din coloana (etapa 16)
        let a = apeluri.clone();
        ui.on_activeaza_tab(move |c| {
            a.borrow_mut()
                .push(if c == -1 { "tab-prev" } else { "tab?" })
        });
        let a = apeluri.clone();
        ui.on_inchide_tab(move |c| a.borrow_mut().push(if c == 2 { "x-tab2" } else { "x?" }));
        let a = apeluri.clone();
        ui.on_fixeaza_randul(move |i| {
            a.borrow_mut()
                .push(if i == 1 { "fixeaza-rand1" } else { "f?" })
        });
        ui.invoke_activeaza_tab(-1);
        ui.invoke_inchide_tab(2);
        ui.invoke_fixeaza_randul(1);
        ui.invoke_fixeaza_previzualizarea();
        assert_eq!(
            *apeluri.borrow(),
            ["tab-prev", "x-tab2", "fixeaza-rand1", "fixeaza-prev"]
        );
        apeluri.borrow_mut().clear();
        // butoanele noi din insula de jos: unirea cursorului si tragerea
        // unui bloc peste panoul din dreapta
        let a = apeluri.clone();
        ui.on_comuta_unirea_cursorului(move |cu| {
            a.borrow_mut().push(if cu { "uneste" } else { "desparte" })
        });
        let a = apeluri.clone();
        ui.on_aplica_diferenta(move |_| a.borrow_mut().push("aplica-bloc"));
        // coloana din stanga: un clic pe un rand schimba obiectul pe loc
        let a = apeluri.clone();
        ui.on_alege_randul(move |i| a.borrow_mut().push(if i == 3 { "rand3" } else { "rand?" }));
        ui.invoke_alege_randul(3);
        ui.invoke_resincronizeaza();
        ui.invoke_comuta_unirea_cursorului(true);
        ui.invoke_comuta_unirea_cursorului(false);
        ui.invoke_aplica_diferenta(0);
        assert_eq!(
            *apeluri.borrow(),
            [
                "rand3",
                "resincronizeaza",
                "uneste",
                "desparte",
                "aplica-bloc"
            ],
            "butoanele noi din editor au actiune"
        );
        apeluri.borrow_mut().clear();

        // o litera singura nu mai face nimic (se scrie in cautare); cu Ctrl, da
        apasa(&ui, "o");
        apasa(&ui, "g");
        apasa(&ui, "j");
        assert!(
            apeluri.borrow().is_empty(),
            "litere fara Ctrl: {:?}",
            apeluri.borrow()
        );
        cu_ctrl(&ui, "o");
        cu_ctrl(&ui, "g");
        cu_ctrl(&ui, "e");
        assert!(ui.get_editeaza_mod(), "Ctrl+E deschide editarea de mana");
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
        // Ctrl+J / K / B si Ctrl+W (etapa 16; Ctrl de la 2026-11-20)
        cu_ctrl(&ui, "j");
        cu_ctrl(&ui, "k");
        cu_ctrl(&ui, "b");
        cu_ctrl(&ui, "w");
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
            [
                "omite", "ghidat", "editeaza", "anuleaza", "j", "k", "ctrl+b", "ctrl+w", "aplica",
                "sari", "opreste", "lista"
            ]
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
        aplica_limba(&ui, texte::Limba::default());
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

        let a = apeluri.clone();
        ui.on_alege_impactul(move |i| a.borrow_mut().push(format!("impact{i}")));
        ui.invoke_alege_impactul(0);
        ui.invoke_sorteaza_coloana(cod_ordine(model_lista::Ordine::Scor));
        ui.invoke_alege_starea(2);
        ui.invoke_schimba_scorul("40".into(), "90".into());
        ui.invoke_exporta_csv();
        ui.invoke_pune_vizibilele_in_plan(true);
        let a = apeluri.clone();
        ui.on_cere_deconectarea(move || a.borrow_mut().push("cere".into()));
        let a = apeluri.clone();
        ui.on_deconecteaza(move || a.borrow_mut().push("deconecteaza".into()));
        let a = apeluri.clone();
        ui.on_renunta_la_deconectare(move || a.borrow_mut().push("ramai".into()));
        let a = apeluri.clone();
        ui.on_alege_baza(move |i| a.borrow_mut().push(format!("baza{i}")));
        ui.invoke_cere_deconectarea();
        ui.invoke_deconecteaza();
        ui.invoke_renunta_la_deconectare();
        ui.invoke_alege_baza(1);
        assert_eq!(
            *apeluri.borrow(),
            [
                "impact0",
                "sort3",
                "stare2",
                "scor40-90",
                "csv",
                "plantrue",
                "cere",
                "deconecteaza",
                "ramai",
                "baza1"
            ],
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

    /// Clicul pe rand, dublu clicul, X-ul, Esc, `o` si nota ajung la
    /// actiunile lor, prin mouse si tastatura adevarate (nu prin `invoke_`):
    /// un `TouchArea` acoperit de alt element ar trece de un test cu
    /// `invoke_`, nu si de asta.
    #[test]
    fn panoul_de_detalii_raspunde_la_mouse_si_tastatura() {
        use slint::platform::{PointerEventButton, WindowEvent};
        use slint::LogicalPosition;
        use std::cell::RefCell;
        let w = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        let _ = slint::platform::set_platform(Box::new(Platforma(w)));
        let ui = FereastraLista::new().expect("fereastra de test");
        aplica_tema(&ui, Mod::Luminos);
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));
        let rand = |nume: &str| RandListaUi {
            schema: "dbo".into(),
            nume: nume.into(),
            tip: "procedura".into(),
            litera: "M".into(),
            ..Default::default()
        };
        ui.set_randuri(ModelRc::new(VecModel::from(vec![rand("pA"), rand("pB")])));
        ui.show().expect("fereastra se arata");
        let apeluri: Rc<RefCell<Vec<String>>> = Rc::default();
        let a = apeluri.clone();
        ui.on_clic_rand(move |i| a.borrow_mut().push(format!("clic{i}")));
        let a = apeluri.clone();
        ui.on_deschide_obiectul(move || a.borrow_mut().push("deschide".into()));
        let a = apeluri.clone();
        ui.on_inchide_detalii(move || a.borrow_mut().push("inchide".into()));
        let a = apeluri.clone();
        ui.on_comuta_omis(move || a.borrow_mut().push("omite".into()));
        let a = apeluri.clone();
        ui.on_nota_editata(move |b, id, t| a.borrow_mut().push(format!("nota {b}/{id}={t}")));

        let clic = |x: f32, y: f32| {
            let position = LogicalPosition::new(x, y);
            ui.window()
                .dispatch_event(WindowEvent::PointerMoved { position });
            ui.window().dispatch_event(WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            });
            ui.window().dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        };
        // unde e primul rand nu se scrie de mana (depinde de antet si de
        // bara de filtre): se cauta, cu clicuri pe coloana „Obiect"
        let y0 = (0..900)
            .step_by(4)
            .map(|y| y as f32)
            .find(|&y| {
                clic(300.0, y);
                let gasit = apeluri.borrow().last().is_some_and(|a| a == "clic0");
                apeluri.borrow_mut().clear();
                gasit
            })
            .expect("primul rand raspunde la clic");
        clic(300.0, y0 + 34.0);
        assert_eq!(*apeluri.borrow(), ["clic1"], "al doilea rand e al doilea");
        apeluri.borrow_mut().clear();

        // dublu clic: din Slint vin doar cele doua clicuri; deschiderea o
        // hotaraste Rust-ul (`dublu_clic`), fiindca `double-clicked` nu
        // ajungea pe Windows. Daca Slint ar deschide si el, s-ar deschide
        // de doua ori.
        std::thread::sleep(std::time::Duration::from_millis(600));
        clic(300.0, y0 + 2.0);
        clic(300.0, y0 + 2.0);
        assert_eq!(*apeluri.borrow(), ["clic0", "clic0"]);
        apeluri.borrow_mut().clear();

        // panoul deschis: Esc il inchide, Ctrl+O omite
        ui.set_detalii_deschise(true);
        ui.set_nota_id("dbo.pA".into());
        let tasta = |t: SharedString| {
            ui.window()
                .dispatch_event(WindowEvent::KeyPressed { text: t.clone() });
            ui.window()
                .dispatch_event(WindowEvent::KeyReleased { text: t });
        };
        tasta(slint::platform::Key::Escape.into());
        cu_ctrl(&ui, "o");
        assert_eq!(*apeluri.borrow(), ["inchide", "omite"]);
        apeluri.borrow_mut().clear();
        ui.set_detalii_deschise(false);
        tasta(slint::platform::Key::Escape.into());
        assert!(apeluri.borrow().is_empty(), "Esc fara panou nu face nimic");
        ui.set_detalii_deschise(true);

        // X-ul din coltul panoului: cautat pe banda de sus a panoului
        let gasit_x = (1100..1440).step_by(3).any(|x| {
            (y0 as i32 - 60..y0 as i32 + 10).step_by(3).any(|y| {
                clic(x as f32, y as f32);
                let e = apeluri.borrow().last().is_some_and(|a| a == "inchide");
                e
            })
        });
        assert!(gasit_x, "X-ul inchide panoul: {:?}", apeluri.borrow());
        apeluri.borrow_mut().clear();

        // nota: textul tastat ajunge cu obiectul lui
        ui.set_nota("".into());
        ui.set_nota_baza("B1".into());
        ui.set_nota_id("dbo.pB".into());
        let gasit_nota = (1000..1440).step_by(6).any(|x| {
            (y0 as i32..900).step_by(6).any(|y| {
                clic(x as f32, y as f32);
                tasta("z".into());
                let e = apeluri.borrow().iter().any(|a| a.starts_with("nota"));
                e
            })
        });
        assert!(gasit_nota, "campul de nota primeste text");
        assert!(
            apeluri.borrow().iter().any(|a| a == "nota B1/dbo.pB=z"),
            "{:?}",
            apeluri.borrow()
        );
    }

    /// Coloana din stanga arata aceleasi randuri ca lista intreaga: daca ar
    /// fi construita separat, cele doua ar putea spune altceva despre acelasi
    /// obiect.
    #[test]
    fn coloana_din_editor_arata_aceleasi_randuri_ca_lista() {
        let ui_lista = fereastra_lista();
        let ui_editor = fereastra();
        aplica_limba(&ui_lista, texte::Limba::default());
        aplica_limba(&ui_editor, texte::Limba::default());
        let mut lista = lista_de_proba();
        lista.selecteaza(1);
        incarca_lista(&ui_lista, &antet_lista(), &lista);
        incarca_coloana_lista(&ui_editor, &lista, &model_arbore::Arbore::default());

        // coloana e arborele (Object Explorer): baza, dosar, obiecte, iar
        // `pozitie` duce la acelasi rand ca in lista mare
        let noduri: Vec<NodUi> = ui_editor.get_noduri_lista().iter().collect();
        assert_eq!(noduri[0].fel, 0, "primul nod e baza");
        assert_eq!(noduri[1].fel, 1, "apoi dosarul");
        let obiecte: Vec<&NodUi> = noduri.iter().filter(|n| n.fel == 2).collect();
        assert_eq!(obiecte.len(), ui_lista.get_randuri().row_count());
        for n in obiecte {
            let a = ui_lista.get_randuri().row_data(n.pozitie as usize).unwrap();
            let b = ui_editor
                .get_randuri_lista()
                .row_data(n.pozitie as usize)
                .unwrap();
            assert_eq!(a.nume, b.nume);
            assert_eq!(a.baza, b.baza);
            assert_eq!(a.stare, b.stare);
            assert_eq!(a.scor_text, b.scor_text);
        }
        let sel = ui_editor.get_nod_selectat_lista();
        assert_eq!(
            noduri[sel as usize].pozitie, 1,
            "nodul selectat e randul selectat"
        );
        // contoarele benzii: doar ce difera
        assert_eq!(ui_editor.get_banda_text(), "M\n1\n\nS\n1");
        assert_eq!(ui_editor.get_selectat_lista(), 1, "acelasi rand selectat");
        assert_eq!(ui_editor.get_rezumat_lista(), ui_lista.get_rezumat_lista());
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
                id: "Prod/dbo.usp_Test",
                schema: "dbo",
                nume: "p",
                tip: "procedura",
                sursa_baza: "A",
                sursa_server: "",
                tinta_baza: "B",
                tinta_server: "",
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
                probleme: &[],
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
                probleme: &[],
            },
        );
        assert!(ui.get_ghidat_terminat());
        assert!(
            ui.get_ghidat_titlu().contains("1 aplicate"),
            "{}",
            ui.get_ghidat_titlu()
        );
    }

    /// Niciun text nu e gol, in nicio limba. Un camp uitat ar ramane gol pe
    /// ecran — exact ce nu se vede intr-o captura.
    ///
    /// Ca ajunge si in Slint o garanteaza `aplica_limba`: destructureaza
    /// `Texte` fara `..`, deci un camp nou care nu e pus in globalul Slint nu
    /// compileaza.
    #[test]
    fn niciun_text_nu_e_gol_in_nicio_limba() {
        for l in texte::Limba::toate() {
            let t = texte::texte(l);
            macro_rules! verifica {
                ($($camp:ident $set:ident),* $(,)?) => {
                    $(
                        assert!(
                            !t.$camp.trim().is_empty(),
                            "textul `{}` e gol in limba {:?}",
                            stringify!($camp),
                            l
                        );
                    )*
                };
            }
            cu_textele!(verifica);
        }
    }

    /// Romana si engleza chiar difera: un tabel copiat de-a gata ar trece
    /// testul de mai sus fara sa traduca nimic.
    #[test]
    fn cele_doua_limbi_nu_sunt_acelasi_tabel() {
        let (ro, en) = (
            texte::texte(texte::Limba::Romana),
            texte::texte(texte::Limba::Engleza),
        );
        assert_eq!(ro.compara, "Compara");
        assert_eq!(en.compara, "Compare");
        assert_eq!(ro.martor, "Martor");
        assert_eq!(en.martor, "Witness");
        assert_ne!(ro.deconecteaza, en.deconecteaza);
    }

    #[test]
    fn aplica_tema_pune_fiecare_jeton_in_slint_in_ambele_moduri() {
        let ui = fereastra();
        let t = ui.global::<Tema>();
        for m in [Mod::Intunecat, Mod::Luminos] {
            aplica_tema(&ui, m);
            aplica_limba(&ui, texte::Limba::default());
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

    fn antet_de_proba() -> AntetEditor<'static> {
        AntetEditor {
            id: "Prod/dbo.usp_Test",
            schema: "dbo",
            nume: "usp_Test",
            tip: "procedura",
            sursa_baza: "Dev",
            sursa_server: "srv-dev",
            tinta_baza: "Prod",
            tinta_server: "srv-prod",
            stare_obiecte: "obiectul 1 din 2 nerevizuite",
            revizuit: false,
            in_plan: false,
        }
    }

    fn model_de_proba() -> model_editor::ModelEditor {
        let a: Vec<String> = (1..=40).map(|i| format!("SELECT {i}")).collect();
        let mut b = a.clone();
        b[20] = "SELECT 999".to_string();
        model_editor::construieste(&a.join("\n\n"), &b.join("\n\n"))
    }

    /// Blocurile 4 si 5 din cazul machetelor cad pe acelasi rand-fantoma si
    /// sunt consecutive: de la etapa 23 sunt **o singura diferenta**, cu un
    /// singur loc aprins la tragere (randul-fantoma) si amandoua blocurile in
    /// grup.
    #[test]
    fn blocurile_consecutive_spre_acelasi_loc_sunt_o_diferenta() {
        let m = model_editor::construieste(
            &gmotion_core::pregateste_afisare(include_str!(
                "../../../cercetare/machete/editor-functii/martor.sql"
            )),
            &gmotion_core::pregateste_afisare(include_str!(
                "../../../cercetare/machete/editor-functii/modificata.sql"
            )),
        );
        let (randuri, _, tinte) = randuri_ui_cu_tinte(&m, true, false, texte::Limba::Romana, None);
        let spre_fantoma: Vec<usize> = (0..tinte.len())
            .filter(|&k| {
                let (a, _) = tinte[k];
                a >= 0 && randuri[a as usize].fantoma
            })
            .collect();
        assert_eq!(spre_fantoma.len(), 1, "{tinte:?}");
        assert_eq!(m.grupuri[spre_fantoma[0]].len(), 2, "{:?}", m.grupuri);
        // blocul care inlocuieste aprinde tot blocul tinta, nu un rand
        let antet = tinte[0];
        assert!(antet.1 - antet.0 > 1, "{antet:?}");
    }

    /// Etapa 21, cu mouse-ul: peste sageata din canal apare previzualizarea
    /// (si dispare cand pleci), iar blocul tras din Martor peste panoul din
    /// dreapta si lasat acolo cheama `aplica-diferenta` pe blocul lui.
    /// „Alege locul" (Mihai, 2026-11-22): un clic pe blocul din Martor imparte
    /// Modificata in chenare; un clic pe un chenar pune blocul acolo; Esc si al
    /// doilea clic pe bloc renunta.
    #[test]
    fn clic_pe_bloc_alege_locul_si_clic_pe_chenar_il_pune_acolo() {
        use slint::platform::{PointerEventButton, WindowEvent};
        use slint::LogicalPosition;
        use std::cell::RefCell;
        let ui = fereastra();
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));
        ui.set_pliaza_identicul(true);
        incarca_editor(&ui, &antet_de_proba(), &model_de_proba());
        ui.show().expect("fereastra se arata");
        let lasate: Rc<RefCell<Vec<(i32, i32)>>> = Rc::default();
        let a = lasate.clone();
        ui.on_plaseaza(move |bloc, loc| a.borrow_mut().push((bloc, loc)));
        let clic = |x: f32, y: f32| {
            let position = LogicalPosition::new(x, y);
            ui.window()
                .dispatch_event(WindowEvent::PointerMoved { position });
            ui.window().dispatch_event(WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            });
            ui.window().dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        };
        // blocul din Martor, cautat cu mouse-ul in panoul din stanga
        let mut gasit = None;
        'cauta: for y in (60..600).step_by(4) {
            for x in [200.0_f32, 400.0] {
                clic(x, y as f32);
                if ui.get_alege_activ() {
                    gasit = Some((x, y as f32));
                    break 'cauta;
                }
            }
        }
        let (x, y) = gasit.expect("un clic pe bloc porneste alegerea");
        assert_eq!(ui.get_alege_diferenta(), 0);
        // al doilea clic pe acelasi bloc renunta, al treilea porneste din nou
        clic(x, y);
        assert!(!ui.get_alege_activ());
        clic(x, y);
        assert!(ui.get_alege_activ());
        // Esc renunta (si nu iese din editor)
        ui.window().dispatch_event(WindowEvent::KeyPressed {
            text: slint::platform::Key::Escape.into(),
        });
        assert!(!ui.get_alege_activ());
        clic(x, y);
        assert!(ui.get_alege_activ());
        // un clic pe un chenar din Modificata pune blocul acolo
        let bloc = ui.get_alege_bloc();
        assert!(bloc >= 0);
        'pune: for yy in (60..800).step_by(4) {
            clic(1200.0, yy as f32);
            if !lasate.borrow().is_empty() {
                break 'pune;
            }
        }
        let l = lasate.borrow();
        assert_eq!(l.len(), 1, "un singur clic, o singura plasare");
        assert_eq!(l[0].0, bloc, "blocul ales, nu altul");
        assert!(!ui.get_alege_activ(), "dupa plasare, modul se inchide");
        let _ = ui.hide();
    }

    #[test]
    fn hover_pe_sageata_previzualizeaza_si_tragerea_aplica_blocul() {
        use slint::platform::{PointerEventButton, WindowEvent};
        use slint::LogicalPosition;
        use std::cell::{Cell, RefCell};
        let ui = fereastra();
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));
        // pliat, ca diferenta sa fie pe ecran fara derulare
        ui.set_pliaza_identicul(true);
        incarca_editor(&ui, &antet_de_proba(), &model_de_proba());
        ui.show().expect("fereastra se arata");
        let cerute: Rc<RefCell<Vec<i32>>> = Rc::default();
        let c = cerute.clone();
        ui.on_previzualizeaza(move |k| c.borrow_mut().push(k));
        let lasate: Rc<RefCell<Vec<(i32, i32)>>> = Rc::default();
        let a = lasate.clone();
        ui.on_plaseaza(move |bloc, loc| a.borrow_mut().push((bloc, loc)));
        let muta = |x: f32, y: f32| {
            ui.window().dispatch_event(WindowEvent::PointerMoved {
                position: LogicalPosition::new(x, y),
            });
        };

        // sageata se cauta cu mouse-ul, ca un om
        let gasit = Cell::new(None);
        'cauta: for y in (60..500).step_by(6) {
            for x in (300..1440).step_by(4) {
                muta(x as f32, y as f32);
                if !cerute.borrow().is_empty() {
                    gasit.set(Some((x as f32, y as f32)));
                    break 'cauta;
                }
            }
        }
        let (x, y) = gasit
            .get()
            .expect("o sageata care previzualizeaza la hover");
        assert_eq!(
            *cerute.borrow(),
            [0],
            "previzualizeaza diferenta de sub mouse"
        );
        assert_eq!(ui.get_previzualizare_diferenta(), 0);
        assert!(
            ui.get_previzualizare_y() > 0.0,
            "fereastra stie unde sa apara"
        );
        ui.set_previzualizare_linii(previzualizare_ui(&[model_editor::LiniePrevizualizare {
            nr: Some(3),
            text: "UPDATE t SET a = 1".into(),
            adusa: true,
        }]));
        muta(x, y + 300.0);
        assert_eq!(ui.get_previzualizare_diferenta(), -1, "pleci, dispare");

        // tragerea: de pe blocul din Martor (stanga sagetii) peste Modificata
        let (de_la, la) = (
            LogicalPosition::new(x - 200.0, y),
            LogicalPosition::new(x + 200.0, y),
        );
        ui.window()
            .dispatch_event(WindowEvent::PointerMoved { position: de_la });
        ui.window().dispatch_event(WindowEvent::PointerPressed {
            position: de_la,
            button: PointerEventButton::Left,
        });
        ui.window()
            .dispatch_event(WindowEvent::PointerMoved { position: la });
        assert_eq!(ui.get_trage_diferenta(), 0, "cat tragi, blocul e tinut");
        let bloc = ui.get_trage_bloc();
        assert!(bloc >= 0, "se trage un bloc anume");
        assert!(ui.get_trage_text() != "", "fantoma arata randul luat");
        assert!(ui.get_trage_rand() >= 0, "si stie randul de sub mouse");
        // ce se asteapta la lasare: locul sugerat (-1) sau locul randului
        let asteptat = if ui.get_trage_pe_sugerat() {
            -1
        } else {
            ui.get_trage_loc()
        };
        assert!(asteptat != -1 || ui.get_trage_pe_sugerat());
        ui.window().dispatch_event(WindowEvent::PointerReleased {
            position: la,
            button: PointerEventButton::Left,
        });
        assert_eq!(
            *lasate.borrow(),
            [(bloc, asteptat)],
            "lasat peste Modificata"
        );
        assert_eq!(ui.get_trage_diferenta(), -1);
        assert_eq!(ui.get_trage_bloc(), -1);
        let _ = ui.hide();
    }

    #[test]
    fn locul_ajunge_in_slint_si_inapoi_neschimbat() {
        use gmotion_core::Loc;
        for loc in [
            Loc::Inlocuieste(0),
            Loc::Inlocuieste(7),
            Loc::Insereaza(0),
            Loc::Insereaza(42),
        ] {
            assert_eq!(loc_din_ui(loc_ui(Some(loc))), Some(loc));
        }
        assert_eq!(loc_ui(None), -1);
        assert_eq!(loc_din_ui(-1), None);
    }

    #[test]
    fn incarca_editor_umple_antetul_si_randurile() {
        let ui = fereastra();
        let model = model_de_proba();
        incarca_editor(
            &ui,
            &AntetEditor {
                id: "Prod/dbo.usp_Test",
                schema: "dbo",
                nume: "usp_Test",
                tip: "procedura",
                sursa_baza: "Dev",
                sursa_server: "srv-dev",
                tinta_baza: "Prod",
                tinta_server: "srv-prod",
                stare_obiecte: "obiectul 1 din 2 nerevizuite",
                revizuit: false,
                in_plan: false,
            },
            &model,
        );
        assert_eq!(ui.get_obiect_schema(), "dbo");
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
            id: "Prod/dbo.p",
            schema: "dbo",
            nume: "usp_Test",
            tip: "procedura",
            sursa_baza: "Dev",
            sursa_server: "",
            tinta_baza: "Prod",
            tinta_server: "",
            stare_obiecte: "",
            revizuit: false,
            in_plan: false,
        };
        ui.set_pliaza_identicul(false);
        incarca_editor(&ui, &antet, &model);
        let intreg = ui.get_randuri().row_count();
        let start_intreg = ui.get_start_diferente().row_data(0).expect("start");
        // fiecare bloc cu eticheta are si randul ei, deasupra
        let etichete = model
            .randuri
            .iter()
            .filter(|r| !r.eticheta.is_empty())
            .count();
        assert_eq!(intreg, model.randuri.len() + etichete);

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
            id: "Prod/dbo.p",
            schema: "dbo",
            nume: "usp_Test",
            tip: "procedura",
            sursa_baza: "Dev",
            sursa_server: "",
            tinta_baza: "Prod",
            tinta_server: "",
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
    fn numeralele_sunt_in_romana_si_in_engleza() {
        const R: texte::Limba = texte::Limba::Romana;
        const E: texte::Limba = texte::Limba::Engleza;
        assert_eq!(numar_diferente(0, R), "fara diferente");
        assert_eq!(numar_diferente(1, R), "1 diferenta");
        assert_eq!(numar_diferente(4, R), "4 diferente");
        assert_eq!(numar_diferente(21, R), "21 de diferente");
        assert_eq!(detaliu_panou("srv-dev", 47, R), "srv-dev · 47 de linii");
        assert_eq!(detaliu_panou("", 1, R), "1 linie");
        assert_eq!(detaliu_panou("srv", 9, R), "srv · 9 linii");
        assert_eq!(numar_diferente(0, E), "no differences");
        assert_eq!(numar_diferente(21, E), "21 differences");
        assert_eq!(detaliu_panou("srv", 1, E), "srv · 1 line");
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
        aplica_limba(&ui, texte::Limba::default());
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
            folosit_de: Some(nume.len() % 3),
            impact: Some(model_lista::Impact::Mediu),
            modificat: None,
            creat: None,
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
            in_plan: 2,
        }
    }

    /// Etapa 22, cu mouse-ul: coloanele „Folosit de" si „Impact" vin din
    /// model, filtrul pe impact se alege din meniu, iar un nume din panou
    /// („Folosit de" / „Foloseste") se apasa si cere obiectul lui.
    #[test]
    fn dependentele_si_impactul_se_vad_si_se_apasa() {
        use slint::platform::{PointerEventButton, WindowEvent};
        use slint::LogicalPosition;
        use std::cell::RefCell;
        let ui = fereastra_lista();
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));
        let mut lista = lista_de_proba();
        lista.filtreaza_impact(Some(model_lista::Impact::Mediu));
        incarca_lista(&ui, &antet_lista(), &lista);
        let r = ui
            .get_randuri()
            .iter()
            .find(|r| r.nume == "usp_Stoc")
            .expect("usp_Stoc");
        assert_eq!(r.folosit_de, "2", "usp_Stoc: 8 % 3");
        assert_eq!(r.impact, 1, "Mediu");
        assert_eq!(
            ui.get_impacturi()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            ["Ridicat", "Mediu", "Scazut"]
        );
        assert_eq!(ui.get_impact_activ(), 1);

        let folosit: Vec<String> = (0..11).map(|i| format!("dbo.p{i:02}")).collect();
        let foloseste = vec!["dbo.fCalc".to_string()];
        incarca_detalii(
            &ui,
            &PanouDetalii {
                deschis: true,
                baza: "CharismaProd",
                obiect: "dbo.usp_Stoc",
                impact: Some(1),
                motive: "folosit de 11 obiecte",
                nota: "",
                folosit_de: &folosit,
                foloseste: &foloseste,
            },
        );
        assert_eq!(ui.get_folosit_de_lista().row_count(), NUME_IN_PANOU);
        assert_eq!(ui.get_folosit_de_rest(), "+3");
        assert_eq!(ui.get_foloseste_rest(), "");

        ui.show().expect("fereastra se arata");
        let cerute: Rc<RefCell<Vec<String>>> = Rc::default();
        let c = cerute.clone();
        ui.on_alege_dependenta(move |n| c.borrow_mut().push(n.to_string()));
        let fixat = ui.clone_strong();
        ui.on_inchide_detalii(move || fixat.set_detalii_deschise(true));
        let clic = |x: f32, y: f32| {
            let position = LogicalPosition::new(x, y);
            ui.window()
                .dispatch_event(WindowEvent::PointerMoved { position });
            ui.window().dispatch_event(WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            });
            ui.window().dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        };
        // numele sunt aliniate la dreapta panoului: se cauta cu clicuri
        'cauta: for y in (150..880).step_by(5) {
            for x in [1380.0, 1340.0] {
                clic(x, y as f32);
                if cerute.borrow().iter().any(|n| n == "dbo.fCalc") {
                    break 'cauta;
                }
            }
        }
        let vazute = cerute.borrow().clone();
        assert!(vazute.contains(&"dbo.p00".to_string()), "{vazute:?}");
        assert!(vazute.contains(&"dbo.fCalc".to_string()), "{vazute:?}");
        let _ = ui.hide();
    }

    /// Cele trei feluri de functii au filtre separate, deci si nume diferite.
    #[test]
    fn felurile_de_functii_au_nume_diferite() {
        for l in texte::Limba::toate() {
            let n: std::collections::BTreeSet<&str> = ["P", "V", "FN", "IF", "TF", "TR", "U", "X"]
                .iter()
                .map(|t| nume_tip_plural(t, l))
                .collect();
            assert_eq!(n.len(), 8, "{l:?}: {n:?}");
        }
        assert_eq!(
            nume_tip_plural("tf ", texte::Limba::Romana),
            "Functii tabel"
        );
    }

    /// F-A16: campul cautarii si butonul „In plan" nu raman de la lista de
    /// dinainte (o conectare noua aduce o lista fara cautare si fara
    /// selectie).
    #[test]
    fn lista_noua_nu_pastreaza_cautarea_si_planul_celei_vechi() {
        let ui = fereastra_lista();
        let mut veche = lista_de_proba();
        veche.cauta("Stoc");
        incarca_lista(&ui, &antet_lista(), &veche);
        assert_eq!(ui.get_cautare(), "Stoc");
        ui.set_obiect_in_plan(true);

        let noua = model_lista::Lista::noua(Vec::new());
        incarca_lista(&ui, &antet_lista(), &noua);
        assert_eq!(
            ui.get_cautare(),
            "",
            "cautarea veche nu mai filtreaza nimic"
        );
        assert!(!ui.get_obiect_in_plan(), "nimic selectat, nimic in plan");
    }

    /// F-A17: doua baze comparate, diferente doar intr-una — coloana Baza
    /// ramane, altfel randurile nu spun din ce baza sunt.
    #[test]
    fn coloana_baza_ramane_cand_doar_o_baza_are_diferente() {
        use model_lista::{Deploy, Obiect, Stare};
        let ob = |baza: &str, stare: Stare| Obiect {
            schema: "dbo".into(),
            nume: "p".into(),
            baza: baza.into(),
            tip: "P".into(),
            stare,
            detalii: None,
            revizuit: false,
            in_plan: false,
            omis: false,
            scor: None,
            deploy: Deploy::Nu,
            folosit_de: None,
            impact: None,
            modificat: None,
            creat: None,
        };
        let ui = fereastra_lista();
        let o_baza =
            model_lista::Lista::noua(vec![ob("C1", Stare::Modificat), ob("C1", Stare::Identic)]);
        incarca_lista(&ui, &antet_lista(), &o_baza);
        assert!(!ui.get_mai_multe_baze());
        let doua =
            model_lista::Lista::noua(vec![ob("C1", Stare::Modificat), ob("C2", Stare::Identic)]);
        incarca_lista(&ui, &antet_lista(), &doua);
        assert!(
            ui.get_mai_multe_baze(),
            "C2 e comparata, chiar daca e identica"
        );
    }

    /// Etapa 25: coloana „Ultima modificare" arata minutul si sorteaza cele
    /// mai recente primele.
    #[test]
    fn ultima_modificare_se_arata_la_minut_si_sorteaza_recentele_intai() {
        use model_lista::{Deploy, Obiect, Ordine, Stare};
        let ob = |nume: &str, cand: Option<&str>| Obiect {
            schema: "dbo".into(),
            nume: nume.into(),
            baza: "C1".into(),
            tip: "P".into(),
            stare: Stare::Modificat,
            detalii: None,
            revizuit: false,
            in_plan: false,
            omis: false,
            scor: None,
            deploy: Deploy::Nu,
            folosit_de: None,
            impact: None,
            modificat: cand.map(str::to_string),
            creat: None,
        };
        let ui = fereastra_lista();
        let mut l = model_lista::Lista::noua(vec![
            ob("pVechi", Some("2025-01-01 08:00:00")),
            ob("pNou", Some("2026-11-22 10:32:15")),
            ob("pNestiut", None),
        ]);
        l.sorteaza(ordine_din_cod(10));
        assert_eq!(l.ordine(), Ordine::Modificat);
        incarca_lista(&ui, &antet_lista(), &l);
        let randuri: Vec<(String, String)> = ui
            .get_randuri()
            .iter()
            .map(|r| (r.nume.to_string(), r.modificat.to_string()))
            .collect();
        assert_eq!(randuri[0], ("pNou".into(), "2026-11-22 10:32".into()));
        assert_eq!(randuri[1].0, "pVechi");
        assert_eq!(randuri[2], ("pNestiut".into(), String::new()));
        assert_eq!(ui.get_coloana_sortata(), 10);
    }

    /// Etapa 25: „Despre" se deschide din Ajutor (antetul ramane ca in
    /// macheta) si se inchide cu Esc.
    #[test]
    fn despre_se_deschide_din_ajutor_si_se_inchide_cu_esc() {
        use slint::platform::{PointerEventButton, WindowEvent};
        use slint::LogicalPosition;
        let ui = fereastra_lista();
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));
        ui.show().expect("fereastra se arata");
        let clic = |x: f32, y: f32| {
            let position = LogicalPosition::new(x, y);
            ui.window()
                .dispatch_event(WindowEvent::PointerMoved { position });
            ui.window().dispatch_event(WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            });
            ui.window().dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        };
        // butonul sta in capul ferestrei de ajutor, langa „Inchide"; se cauta
        let gasit = (180..230).step_by(4).any(|y| {
            (700..1040).step_by(8).any(|x| {
                ui.set_arata_despre(false);
                ui.set_arata_ajutorul(true);
                clic(x as f32, y as f32);
                ui.get_arata_despre()
            })
        });
        assert!(gasit, "butonul Despre din Ajutor");
        assert!(!ui.get_arata_ajutorul(), "Ajutorul s-a inchis");
        ui.window().dispatch_event(WindowEvent::KeyPressed {
            text: slint::platform::Key::Escape.into(),
        });
        assert!(!ui.get_arata_despre(), "Esc inchide Despre");
        let _ = ui.hide();
    }

    /// Etapa 25: campul de editare se coloreaza cand se deschide si la
    /// fiecare tasta, prin `leaga_editarea`.
    #[test]
    fn campul_de_editare_se_coloreaza_la_deschidere_si_la_scriere() {
        use slint::platform::WindowEvent;
        let ui = fereastra();
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));
        leaga_editarea(&ui);
        ui.set_text_final("SELECT 1\nFROM t".into());
        ui.set_editeaza_mod(true);
        ui.show().expect("fereastra se arata");
        assert_eq!(ui.get_ed_linii().row_count(), 2, "la deschidere");
        // o tasta in camp: Enter adauga o linie
        ui.window().dispatch_event(WindowEvent::KeyPressed {
            text: slint::platform::Key::Return.into(),
        });
        ui.window().dispatch_event(WindowEvent::KeyReleased {
            text: slint::platform::Key::Return.into(),
        });
        assert_eq!(ui.get_ed_linii().row_count(), 3, "la scriere");
        let _ = ui.hide();
    }

    /// Etapa 25: fiecare bucata colorata stie cate caractere si cate taburi
    /// are inaintea ei pe linie: din ele se pune exact peste glifele campului.
    #[test]
    fn bucatile_de_editare_isi_stiu_pozitia_in_linie() {
        let linii = linii_editare_ui("\tSELECT @a, 'x'\n\nEND");
        assert_eq!(linii.row_count(), 3);
        let prima: Vec<BucataEditareUi> = linii.row_data(0).unwrap().bucati.iter().collect();
        let select = prima.iter().find(|b| b.text == "SELECT").expect("SELECT");
        assert_eq!((select.caractere, select.taburi, select.sintaxa), (0, 1, 1));
        let var = prima.iter().find(|b| b.text == "@a").expect("@a");
        assert_eq!((var.caractere, var.taburi, var.sintaxa), (7, 1, 2));
        let sir = prima.iter().find(|b| b.text == "'x'").expect("sirul");
        assert_eq!((sir.caractere, sir.sintaxa), (11, 4));
        assert_eq!(
            linii.row_data(1).unwrap().bucati.row_count(),
            0,
            "linia goala"
        );
    }

    /// Mesajul de structura inchis cu × nu revine pentru acelasi text, dar
    /// revine cand problema se schimba; × pe raspunsul serverului cere
    /// inchiderea lui (aplicatia il sterge).
    #[test]
    fn mesajele_se_inchid_cu_x_si_nu_raman_blocate() {
        use std::cell::Cell;
        let ui = fereastra();
        ui.set_probleme_structura("linia 14: BEGIN fara END".into());
        assert!(ui.get_arata_probleme());
        ui.set_probleme_ascunse(ui.get_probleme_structura());
        assert!(!ui.get_arata_probleme(), "× il ascunde");
        ui.set_probleme_structura("linia 20: BEGIN fara END".into());
        assert!(ui.get_arata_probleme(), "alta problema, se arata din nou");
        ui.set_probleme_structura("".into());
        assert!(!ui.get_arata_probleme(), "textul reparat: dispare");
        let inchis = std::rc::Rc::new(Cell::new(false));
        let i = inchis.clone();
        ui.on_inchide_verificarea(move || i.set(true));
        ui.invoke_inchide_verificarea();
        assert!(inchis.get());
    }

    #[test]
    fn problemele_de_structura_se_spun_cu_linia_in_ambele_limbi() {
        let p = gmotion_core::structura::verifica("BEGIN\nSELECT (1\nGO");
        assert_eq!(text_probleme(&[], texte::Limba::Romana), "");
        let ro = text_probleme(&p, texte::Limba::Romana);
        assert!(
            ro.starts_with("Textul final nu se mai compileaza: linia 1: BEGIN fara END"),
            "{ro}"
        );
        assert!(ro.ends_with("(+2 altele)"), "{ro}");
        let en = text_probleme(&p, texte::Limba::Engleza);
        assert!(en.contains("line 1: BEGIN without END"), "{en}");
    }

    #[test]
    fn despre_spune_versiunea_licentele_si_unde_e_jurnalul() {
        let j = std::path::Path::new("/d/jurnale/gmotion-1.log");
        for l in texte::Limba::toate() {
            let t = text_despre(l, "1.2.3", Some(j), None);
            assert!(t.starts_with("GMotion 1.2.3"), "{t}");
            assert!(t.contains("Slint Royalty-free License 2.0"), "{t}");
            assert!(t.contains("SIL Open Font License 1.1"), "{t}");
            assert!(t.contains("/d/jurnale/gmotion-1.log"), "{t}");
        }
        assert!(text_despre(texte::Limba::Romana, "1", None, None).contains("(necunoscut)"));
        assert!(text_despre(texte::Limba::Engleza, "1", None, None).contains("Third-party"));
    }

    /// Etapa 27: filtrele pe operatie si pe date se vad in bara, cu
    /// optiunile in limba aleasa.
    #[test]
    fn filtrele_de_operatie_si_data_se_vad_in_bara() {
        use model_lista::{Operatie, Perioada};
        let ui = fereastra_lista();
        let mut lista = lista_de_proba();
        incarca_lista(&ui, &antet_lista(), &lista);
        assert_eq!(
            (
                ui.get_operatie_activa(),
                ui.get_creat_activ(),
                ui.get_modificat_activ()
            ),
            (-1, -1, -1)
        );
        assert_eq!(ui.get_operatii().row_count(), 3);
        assert!(ui.get_operatii().row_data(2).unwrap().starts_with("DROP"));
        lista.filtreaza_operatie(Some(Operatie::Drop));
        lista.filtreaza_creat(Some(Perioada::Zile7), "2026-11-22");
        lista.filtreaza_modificat(Some(Perioada::Zile30), "2026-11-22");
        incarca_lista(&ui, &antet_lista(), &lista);
        assert_eq!(
            (
                ui.get_operatie_activa(),
                ui.get_creat_activ(),
                ui.get_modificat_activ()
            ),
            (2, 1, 2)
        );
        assert_eq!(
            ui.get_perioade_creat().row_data(1).unwrap(),
            "Creat in ultimele 7 zile"
        );
    }

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
                "Doar in martor",
                "Extra",
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
        // a doua redesenare pe acelasi obiect nu mai recalculeaza diff-ul
        let inainte = ui.get_previzualizare();
        incarca_previzualizare(&ui, &a, &b, &[]);
        assert!(
            inainte == ui.get_previzualizare(),
            "acelasi model, nerecalculat"
        );
        incarca_previzualizare(&ui, &b, &a, &[]);
        assert!(inainte != ui.get_previzualizare(), "alt obiect, recalculat");
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

    /// Butoanele noi din conectare cheama fiecare actiunea lui. Ce se
    /// deseneaza fara sa fie legat arata perfect in captura (etapele 8-11).
    #[test]
    fn butoanele_conectarii_cheama_actiunile() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let ui = fereastra_conectare();
        let apeluri: Rc<RefCell<Vec<String>>> = Rc::default();
        let a = apeluri.clone();
        ui.on_testeaza(move |sursa| a.borrow_mut().push(format!("testeaza{sursa}")));
        let a = apeluri.clone();
        ui.on_sterge_jurnalul(move || a.borrow_mut().push("sterge".into()));
        let a = apeluri.clone();
        ui.on_deschide_offline(move || a.borrow_mut().push("offline".into()));
        let a = apeluri.clone();
        ui.on_compara_offline(move |x, y| a.borrow_mut().push(format!("cmp{x}|{y}")));
        let a = apeluri.clone();
        ui.on_inchide_offline(move || a.borrow_mut().push("inchide".into()));
        let a = apeluri.clone();
        ui.on_anuleaza(move || a.borrow_mut().push("anuleaza".into()));
        let a = apeluri.clone();
        ui.on_ignora_recenta(move |i| a.borrow_mut().push(format!("ignora{i}")));
        let a = apeluri.clone();
        ui.on_ajutor(move || a.borrow_mut().push("ajutor".into()));
        let a = apeluri.clone();
        ui.on_deschide_instantaneu(move || a.borrow_mut().push("instantaneu".into()));
        let a = apeluri.clone();
        ui.on_alege_instantaneul(move |c| a.borrow_mut().push(format!("ales{c}")));

        ui.invoke_testeaza(true);
        ui.invoke_sterge_jurnalul();
        ui.invoke_deschide_offline();
        ui.invoke_compara_offline("A".into(), "B".into());
        ui.invoke_inchide_offline();
        ui.invoke_anuleaza();
        ui.invoke_ignora_recenta(2);
        ui.invoke_ajutor();
        ui.invoke_deschide_instantaneu();
        ui.invoke_alege_instantaneul("/x/y.json".into());
        assert_eq!(
            *apeluri.borrow(),
            [
                "testeazatrue",
                "sterge",
                "offline",
                "cmpA|B",
                "inchide",
                "anuleaza",
                "ignora2",
                "ajutor",
                "instantaneu",
                "ales/x/y.json"
            ]
        );
    }

    /// Etapa 17: bifa „Aceeasi conexiune", randurile din cele doua liste de
    /// baze si Setari, apasate cu mouse-ul, la 1440x900 (scara 1). Baza
    /// martorului din lista modificatei nu raspunde la clic.
    #[test]
    fn conectarea_noua_raspunde_la_clicuri() {
        use model_conectare::{Conectare, Parte, StareConexiune};
        use slint::platform::{PointerEventButton, WindowEvent};
        use slint::LogicalPosition;
        use std::cell::RefCell;
        use std::rc::Rc;

        let ui = fereastra_conectare();
        let mut c = Conectare::noua();
        c.sursa = Parte {
            server: "SRV-DEV".into(),
            baza: "Dev".into(),
            baze: vec!["Dev".into(), "Prod".into(), "Test".into()],
            stare: StareConexiune::Conectat("conectat".into()),
            auth_windows: true,
            ..Parte::default()
        };
        c.sincronizeaza();
        c.alege_baza_modificata("Prod");
        incarca_conectare(&ui, &c);
        assert!(ui.get_aceeasi_conexiune());
        assert_eq!(ui.get_baza_blocata(), "Dev");
        ui.show().expect("fereastra se arata");
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));

        let apeluri: Rc<RefCell<Vec<String>>> = Rc::default();
        let a = apeluri.clone();
        ui.on_baza_schimbata(move |sursa, t| a.borrow_mut().push(format!("baza {sursa} {t}")));
        let a = apeluri.clone();
        ui.on_aceeasi_conexiune_comutata(move || a.borrow_mut().push("bifa".into()));
        let a = apeluri.clone();
        ui.on_schimba_certificatul(move || a.borrow_mut().push("certificat".into()));

        let clic = |x: f32, y: f32| {
            let position = LogicalPosition::new(x, y);
            ui.window()
                .dispatch_event(WindowEvent::PointerMoved { position });
            ui.window().dispatch_event(WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            });
            ui.window().dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        };
        // al doilea rand din lista martorului, apoi primul si al treilea din
        // a modificatei (primul e chiar martorul: nu raspunde)
        clic(300.0, 323.0);
        clic(1000.0, 289.0);
        clic(1000.0, 357.0);
        clic(1320.0, 105.0);
        assert_eq!(
            *apeluri.borrow(),
            ["baza true Prod", "baza false Test", "bifa"]
        );

        // Setari din antet, apoi „Schimba" din fereastra lui
        assert!(!ui.get_arata_setarile());
        clic(1187.0, 30.0);
        assert!(ui.get_arata_setarile(), "Setari se deschide din antet");
        // de la mijloc in jos: un clic in afara ferestrei mici o inchide
        let apasat = (440..520).step_by(4).any(|y| {
            clic(945.0, y as f32);
            apeluri.borrow().last().is_some_and(|x| x == "certificat")
        });
        assert!(apasat, "„Schimba” din Setari schimba certificatul");
    }

    /// Progresul compararii si etichetele serverelor gasite ajung pe ecran.
    #[test]
    fn progresul_si_etichetele_serverelor_ajung_in_fereastra() {
        use model_conectare::{Conectare, ServerGasit};
        let ui = fereastra_conectare();
        let mut c = Conectare::noua();
        c.sursa.server = "SRV".into();
        c.sursa.servere_gasite = vec![
            ServerGasit {
                nume: "SRV01".into(),
                implicita: true,
                cluster: false,
            },
            ServerGasit {
                nume: "SRV02\\SQL2019".into(),
                implicita: false,
                cluster: true,
            },
        ];
        c.baza.baze = vec!["C1".into(), "C2".into(), "C3".into()];
        c.baza.comuta_baza("C1");
        c.baza.comuta_baza("C3");
        incarca_conectare(&ui, &c);

        let g = ui.get_sursa().servere_gasite;
        assert_eq!(g.row_count(), 2);
        assert!(g.row_data(0).unwrap().implicita && !g.row_data(0).unwrap().cluster);
        assert!(g.row_data(1).unwrap().cluster);
        // bifele si eticheta bazelor
        let b = ui.get_baza();
        assert_eq!(b.eticheta_baze, "C1, C3");
        let bifate: Vec<bool> = b.alese.iter().collect();
        assert_eq!(bifate, [true, false, true]);
    }

    /// Eticheta bazelor se scurteaza cand sunt multe: un sir lung ar iesi
    /// din camp.
    #[test]
    fn eticheta_bazelor_se_scurteaza_la_mai_mult_de_doua() {
        use model_conectare::Parte;
        let mut p = Parte::default();
        assert_eq!(eticheta_baze(&p, texte::Limba::Romana), "");
        p.baza = "C1".into();
        assert_eq!(eticheta_baze(&p, texte::Limba::Romana), "C1");
        p.alese = vec!["C1".into(), "C2".into()];
        assert_eq!(eticheta_baze(&p, texte::Limba::Romana), "C1, C2");
        p.alese.push("C3".into());
        assert_eq!(eticheta_baze(&p, texte::Limba::Romana), "3 baze");
    }

    fn fereastra_conectare() -> FereastraConectare {
        let w = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
        let _ = slint::platform::set_platform(Box::new(Platforma(w)));
        let ui = FereastraConectare::new().expect("fereastra de test");
        aplica_tema(&ui, Mod::Luminos);
        aplica_limba(&ui, texte::Limba::default());
        ui
    }

    /// 15 F: ecranul spune cand martorul si modificata sunt pe aceeasi
    /// instanta, si cand cautarea in retea inca ruleaza.
    #[test]
    fn conectarea_spune_aceeasi_instanta_si_cautarea_in_curs() {
        use model_conectare::{Conectare, StareConexiune};
        let ui = fereastra_conectare();
        let mut c = Conectare::noua();
        // doua conexiuni scrise separat, care se dovedesc aceeasi instanta
        c.aceeasi_conexiune = false;
        c.sursa.server = "srv-prod.firma.local".into();
        c.baza.server = "SRV-PROD,1433".into();
        incarca_conectare(&ui, &c);
        assert!(ui.get_nota_instanta().contains("aceeasi instanta"));
        c.baza.server = "SRV-TEST".into();
        c.sursa.stare = StareConexiune::SeCauta;
        incarca_conectare(&ui, &c);
        assert_eq!(ui.get_nota_instanta(), "");
        assert_eq!(ui.get_sursa().stare, "se cauta servere in retea…");
        assert_eq!(ui.get_sursa().fel_stare, 1, "ca o operatie in curs");
    }

    #[test]
    fn ecranul_de_conectare_spune_ce_lipseste_si_nu_arata_parola() {
        use model_conectare::{Acreditari, Conectare, Parte, StareConexiune};
        let ui = fereastra_conectare();
        let mut c = Conectare::noua();
        incarca_conectare(&ui, &c);
        assert_eq!(ui.get_ce_lipseste(), "completeaza serverul pentru martor");

        c.sursa = Parte {
            server: "SRV-DEV".into(),
            acreditari: Acreditari::noi("deploy_ro", "parola-secreta"),
            baza: "CharismaDev".into(),
            baze: vec!["CharismaDev".into(), "Altceva".into()],
            stare: StareConexiune::Conectat("conectat · 21.204 obiecte".into()),
            auth_windows: false,
            servere_gasite: vec![
                model_conectare::ServerGasit {
                    nume: "SRV-DEV\\SQL2019".into(),
                    implicita: false,
                    cluster: true,
                },
                model_conectare::ServerGasit::nou("SRV-PROD"),
            ],
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
    }

    /// F-A1 din `cercetare/audit-qa-2026-11-18.md`: `editeaza-mod` nu se
    /// inchidea la schimbarea obiectului. Campul `TextInput` isi rupe
    /// legatura cu modelul la prima tasta, deci dupa „›" arata tot textul
    /// obiectului dinainte, iar „Salveaza" il punea pe obiectul nou.
    #[test]
    fn editarea_de_mana_se_inchide_la_schimbarea_obiectului() {
        let ui = fereastra();
        let model = model_de_proba();
        let antet = |id: &'static str, nume: &'static str| AntetEditor {
            id,
            schema: "dbo",
            nume,
            tip: "procedura",
            sursa_baza: "Dev",
            sursa_server: "srv-dev",
            tinta_baza: "Prod",
            tinta_server: "srv-prod",
            stare_obiecte: "obiectul 1 din 2 nerevizuite",
            revizuit: false,
            in_plan: false,
        };
        incarca_editor(&ui, &antet("Prod/dbo.pA", "pA"), &model);
        assert_eq!(ui.get_obiect_id(), "Prod/dbo.pA");
        // omul apasa „Editeaza" si scrie
        ui.set_editeaza_mod(true);
        // acelasi obiect, o redesenare: editarea continua
        incarca_editor(&ui, &antet("Prod/dbo.pA", "pA"), &model);
        assert!(ui.get_editeaza_mod(), "o redesenare nu intrerupe editarea");
        // alt obiect: editarea de mana nu se duce cu el
        incarca_editor(&ui, &antet("Prod/dbo.pB", "pB"), &model);
        assert!(
            !ui.get_editeaza_mod(),
            "editarea de mana trebuie sa se inchida la alt obiect"
        );
        assert_eq!(ui.get_obiect_id(), "Prod/dbo.pB");

        // Esc, cat se scrie de mana, renunta la editare si **nu** pleaca din
        // ecran: inainte urca la `inapoi-lista`, cu textul nesalvat
        ui.show().expect("fereastra se arata");
        ui.invoke_focus_taste();
        let plecat = std::rc::Rc::new(std::cell::Cell::new(false));
        let p2 = plecat.clone();
        ui.on_inapoi_lista(move || p2.set(true));
        ui.set_editeaza_mod(true);
        apasa(&ui, "\u{1b}");
        assert!(!ui.get_editeaza_mod(), "Esc inchide editarea de mana");
        assert!(!plecat.get(), "Esc nu pleaca din editor cat se scrie");
        // a doua apasare, fara editare, pleaca din editor ca pana acum
        apasa(&ui, "\u{1b}");
        assert!(plecat.get());
    }

    /// F-A2 din `cercetare/audit-qa-2026-11-18.md`: campurile din conectare nu
    /// mai urmau modelul dupa prima tasta (legatura `text: root.text` se rupe
    /// la editare). Deci dupa „Inverseaza" modelul se schimba, campurile arata
    /// vechile valori, si comparai — si deployai — in directia opusa celei de
    /// pe ecran.
    ///
    /// Testul scrie cu tastatura, apoi schimba modelul, apoi scrie din nou:
    /// ce raporteaza `server-schimbat` arata ce era in camp.
    #[test]
    fn campurile_din_conectare_urmeaza_modelul_si_dupa_ce_s_a_scris_in_ele() {
        use model_conectare::{Conectare, Parte};
        use slint::platform::{PointerEventButton, WindowEvent};
        use slint::LogicalPosition;
        use std::cell::RefCell;
        use std::rc::Rc;

        let ui = fereastra_conectare();
        let mut c = Conectare::noua();
        c.sursa = Parte {
            server: "SRV-DEV".into(),
            ..Parte::default()
        };
        c.baza = Parte {
            server: "SRV-PROD".into(),
            ..Parte::default()
        };
        incarca_conectare(&ui, &c);
        ui.show().expect("fereastra se arata");
        // fereastra de test n-are dimensiune; fara una, aranjarea e degenerata
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));

        // ce a ajuns in model, de la camp
        let scris: Rc<RefCell<Vec<(bool, String)>>> = Rc::default();
        let sc = scris.clone();
        ui.on_server_schimbat(move |sursa, t| sc.borrow_mut().push((sursa, t.to_string())));

        let clic = |x: f32, y: f32| {
            let position = LogicalPosition::new(x, y);
            ui.window()
                .dispatch_event(WindowEvent::PointerMoved { position });
            ui.window().dispatch_event(WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            });
            ui.window().dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        };
        let tasteaza = |t: &str| {
            let t: SharedString = t.into();
            ui.window()
                .dispatch_event(WindowEvent::KeyPressed { text: t.clone() });
            ui.window()
                .dispatch_event(WindowEvent::KeyReleased { text: t });
        };

        // unde e campul serverului sursa nu se scrie de mana: se cauta, cu
        // clicuri pe coloana din stanga si o tasta de proba
        let gasit = (0..800)
            .step_by(4)
            .map(|y| y as f32)
            .find(|&y| {
                clic(450.0, y);
                tasteaza("x");
                let gasit = scris
                    .borrow()
                    .last()
                    .is_some_and(|(sursa, t)| *sursa && t.contains('x'));
                scris.borrow_mut().clear();
                gasit
            })
            .is_some();
        assert!(
            gasit,
            "campul serverului sursa trebuie sa primeasca tastele"
        );

        // „Inverseaza": modelul se schimba sub cursorul ramas in camp
        std::mem::swap(&mut c.sursa, &mut c.baza);
        incarca_conectare(&ui, &c);
        tasteaza("1");
        let raportat = scris.borrow().last().cloned().expect("campul a raportat");
        assert!(
            raportat.1.contains("SRV-PROD"),
            "campul trebuie sa arate valoarea de acum din model, nu pe cea de dinainte: {:?}",
            raportat.1
        );

        // Campul de parola nu se rescrie din model: acolo modelul da doar
        // buline cat lungimea parolei, si rescris cu ele prima tasta de dupa
        // ar trimite bulinele ca parola.
        let parole: Rc<RefCell<Vec<(bool, String)>>> = Rc::default();
        let pa = parole.clone();
        ui.on_parola_schimbata(move |sursa, t| pa.borrow_mut().push((sursa, t.to_string())));
        let mut gasit = false;
        for x in [160.0f32, 360.0, 560.0, 760.0, 960.0] {
            for y in (0..800).step_by(4).map(|y| y as f32) {
                clic(x, y);
                tasteaza("s");
                gasit = parole.borrow().last().is_some_and(|(sursa, _)| *sursa);
                parole.borrow_mut().clear();
                if gasit {
                    break;
                }
            }
            if gasit {
                break;
            }
        }
        assert!(
            gasit,
            "campul parolei pentru martor trebuie sa primeasca tastele"
        );
        // modelul raspunde cu buline (parola exista acum)
        c.sursa.acreditari = model_conectare::Acreditari::noi("u", "s");
        incarca_conectare(&ui, &c);
        tasteaza("e");
        let ultima = parole.borrow().last().cloned().expect("campul a raportat");
        assert!(
            !ultima.1.contains('\u{2022}'),
            "bulinele din model nu au voie sa ajunga parola: {:?}",
            ultima.1
        );
    }

    /// V10 din `cercetare/audit-qa-2026-11-18.md`: liniile mai lungi decat
    /// panoul erau taiate fara elipsa, fara derulare si fara impachetare —
    /// o diferenta la coloana 90 era invizibila intr-o unealta de diff.
    #[test]
    fn liniile_lungi_se_pot_derula_pe_orizontala() {
        let ui = fereastra();
        ui.window().set_size(slint::PhysicalSize::new(1024, 768));
        let lunga = format!("SELECT {} FROM t", "coloana_cu_nume_lung, ".repeat(20));
        let a = format!("CREATE PROC dbo.p AS\n{lunga} WHERE a = 1");
        let b = format!("CREATE PROC dbo.p AS\n{lunga} WHERE a = 2");
        let model = model_editor::construieste(&a, &b);
        incarca_editor(
            &ui,
            &AntetEditor {
                id: "Prod/dbo.p",
                schema: "dbo",
                nume: "p",
                tip: "procedura",
                sursa_baza: "Dev",
                sursa_server: "srv",
                tinta_baza: "Prod",
                tinta_server: "srv",
                stare_obiecte: "obiectul 1 din 1 nerevizuite",
                revizuit: false,
                in_plan: false,
            },
            &model,
        );
        ui.show().expect("fereastra se arata");
        ui.invoke_focus_taste();
        assert!(
            ui.get_latime_continut() > 1024.0,
            "linia de proba e mai lata decat ecranul: {}",
            ui.get_latime_continut()
        );
        assert!(
            ui.get_derulare_maxima() > 0.0,
            "deci se poate derula: {}",
            ui.get_derulare_maxima()
        );
        assert_eq!(ui.get_derulare_x(), 0.0, "incepe de la stanga");

        // Shift + sageti deruleaza, si nu trece de capete
        let cu_shift = |tasta: slint::platform::Key| {
            use slint::platform::WindowEvent;
            let t: SharedString = tasta.into();
            ui.window().dispatch_event(WindowEvent::KeyPressed {
                text: slint::platform::Key::Shift.into(),
            });
            ui.window()
                .dispatch_event(WindowEvent::KeyPressed { text: t.clone() });
            ui.window()
                .dispatch_event(WindowEvent::KeyReleased { text: t });
            ui.window().dispatch_event(WindowEvent::KeyReleased {
                text: slint::platform::Key::Shift.into(),
            });
        };
        cu_shift(slint::platform::Key::RightArrow);
        let dupa = ui.get_derulare_x();
        assert!(dupa > 0.0, "s-a derulat spre dreapta: {dupa}");
        cu_shift(slint::platform::Key::LeftArrow);
        assert_eq!(ui.get_derulare_x(), 0.0, "si inapoi");
        cu_shift(slint::platform::Key::LeftArrow);
        assert_eq!(ui.get_derulare_x(), 0.0, "nu trece de margine");
        for _ in 0..200 {
            cu_shift(slint::platform::Key::RightArrow);
        }
        assert_eq!(
            ui.get_derulare_x(),
            ui.get_derulare_maxima(),
            "nici de cealalta margine"
        );
        // un obiect fara linii lungi nu are ce derula
        let scurt = model_editor::construieste("SELECT 1", "SELECT 2");
        let ui2 = fereastra();
        ui2.window().set_size(slint::PhysicalSize::new(1024, 768));
        incarca_editor(
            &ui2,
            &AntetEditor {
                id: "Prod/dbo.q",
                schema: "dbo",
                nume: "q",
                tip: "procedura",
                sursa_baza: "Dev",
                sursa_server: "srv",
                tinta_baza: "Prod",
                tinta_server: "srv",
                stare_obiecte: "",
                revizuit: false,
                in_plan: false,
            },
            &scurt,
        );
        assert_eq!(ui2.get_derulare_maxima(), 0.0);
    }

    /// Etapa 19: linia se rupe dupa un spatiu, iar o bucata care trece de
    /// margine se taie cu culoarea ei;
    /// Legatura din canal se deseneaza pe randuri **afisate**: plierea si
    /// wrap-ul muta indicii, iar o legatura calculata pe randurile modelului
    /// ar arata spre alt loc decat cel desenat.
    #[test]
    fn legatura_din_canal_arata_spre_randul_afisat() {
        let martor = concat!(
            "CREATE PROC p AS\n\nSELECT 1\n\nSELECT 2\n\nSELECT 3\n\n",
            "EXEC dbo.doar_in_martor @x = 1\n\nSELECT 4"
        );
        let tinta = "CREATE PROC p AS\n\nSELECT 1\n\nSELECT 2\n\nSELECT 3\n\nSELECT 4";
        let model = model_editor::construieste(martor, tinta);
        for pliat in [false, true] {
            let (randuri, start, tinte) =
                randuri_ui_cu_tinte(&model, pliat, false, texte::Limba::Romana, None);
            // locul unde vine blocul fara corespondent e chiar in dreptul lui
            // (etapa 23): legatura e dreapta, pe acelasi rand afisat
            let f = randuri
                .iter()
                .position(|r| r.fantoma)
                .unwrap_or_else(|| panic!("pliat={pliat}: locul de inserare lipseste"));
            let k = randuri[f].diferenta;
            assert!(k >= 0, "pliat={pliat}: randul locului e al diferentei");
            assert_eq!(start[k as usize], f as i32, "pliat={pliat}");
            assert_eq!(
                tinte[k as usize].0, f as i32,
                "pliat={pliat}: tinta in dreptul ei"
            );
            assert!(randuri[f].inserare && randuri[f].dr_nr.is_empty());
            // celelalte legaturi cad pe randuri cu ceva in dreapta
            for (k, &(a, z)) in tinte.iter().enumerate() {
                if a < 0 {
                    continue;
                }
                assert!(z > a && (z as usize) <= randuri.len(), "pliat={pliat}: {k}");
                let r = &randuri[a as usize];
                assert!(
                    r.fantoma || !r.dr_nr.is_empty(),
                    "pliat={pliat}: {k} pe un rand gol"
                );
            }
            assert!(start.iter().any(|&s| s >= 0));
        }
    }

    /// Pastila spune si unde ajunge blocul, nu doar ce e: altfel sageata din
    /// canal ramane o actiune despre care nu stii ce face.
    #[test]
    fn pastila_blocului_ajunge_in_randul_afisat() {
        let martor = "CREATE PROC p AS\n\nSELECT 1\n\nEXEC dbo.doar_aici @x = 1";
        let tinta = "CREATE PROC p(@a int) AS\n\nSELECT 1";
        let model = model_editor::construieste(martor, tinta);
        let (randuri, _) = randuri_ui(&model, false, false, texte::Limba::Romana, None);
        let etichete: Vec<&str> = randuri
            .iter()
            .filter(|r| !r.eticheta.is_empty())
            .map(|r| r.eticheta.as_str())
            .collect();
        assert!(
            etichete.iter().any(|e| e.contains("inlocuieste blocul")),
            "{etichete:?}"
        );
        assert!(
            etichete.iter().any(|e| e.contains("se insereaza")),
            "{etichete:?}"
        );
        assert!(randuri.iter().any(|r| r.fantoma), "lipseste randul fantoma");
    }

    /// tabul ocupa 4 coloane; fara wrap linia ramane intreaga.
    /// Proba pe Windows: un obiect identic se plia intr-un singur rand si nu
    /// i se vedea codul. Fara diferente nu se pliaza nimic.
    #[test]
    fn obiectul_identic_isi_arata_tot_codul() {
        let t = "CREATE PROC dbo.p AS\nSELECT 1\nSELECT 2\nSELECT 3\nSELECT 4";
        let model = model_editor::construieste(t, t);
        let (randuri, _) = randuri_ui(&model, true, false, texte::Limba::Romana, None);
        assert_eq!(randuri.len(), 5, "toate liniile, nu o pliere");
        assert!(randuri.iter().all(|r| r.fel == 0 && !r.st_nr.is_empty()));
        // cu o diferenta, plierea ramane (destule linii identice cat sa
        // treaca de cele 3 de context)
        let lung: String = (1..=12).map(|i| format!("\nSELECT {i}")).collect();
        let a = format!("CREATE PROC dbo.p AS{lung}");
        let model = model_editor::construieste(&a, &a.replace("SELECT 12", "SELECT 99"));
        let (randuri, _) = randuri_ui(&model, true, false, texte::Limba::Romana, None);
        assert!(
            randuri.iter().any(|r| r.fel == 1),
            "contextul identic se pliaza"
        );
    }

    #[test]
    fn linia_se_rupe_la_margine_cu_bucatile_ei() {
        let b = |t: &str, schimbat: bool| Bucata {
            text: t.into(),
            sintaxa: Sintaxa::Text,
            schimbat,
        };
        let linie = [
            b("SELECT ", false),
            b("coloana_lunga", true),
            b(" FROM t", false),
        ];
        let r = rupe_linia(&linie, Some(10));
        let texte: Vec<String> = r
            .iter()
            .map(|x| {
                x.iter()
                    .map(|b| b.text.as_str())
                    .collect::<Vec<_>>()
                    .join("|")
            })
            .collect();
        // rupe dupa spatiu; cuvantul mai lung decat randul se taie la margine
        assert_eq!(texte, ["SELECT ", "coloana_lu", "nga| FROM t"]);
        assert!(r[1][0].schimbat, "bucata taiata isi pastreaza marcajul");
        assert_eq!(rupe_linia(&linie, None).len(), 1);
        assert_eq!(rupe_linia(&[], Some(10)).len(), 1, "linia goala e un rand");
        let tab = [b("\t\t\tx", false)];
        assert_eq!(
            rupe_linia(&tab, Some(10)).len(),
            2,
            "trei taburi = 12 coloane"
        );
    }

    /// Cu wrap, linia lunga continua pe randuri fara numar, iar partea mai
    /// scurta primeste randuri goale: stanga si dreapta raman aliniate, iar
    /// blocul diferentei le numara si pe ele.
    /// Eticheta blocului sta pe randul ei, imediat deasupra primului rand al
    /// blocului, o singura data; codul nu mai are eticheta peste el, nici pe
    /// randurile rupte de wrap (sesizat de Mihai: se suprapunea si se repeta).
    #[test]
    fn eticheta_e_pe_randul_ei_o_singura_data() {
        let a = format!("CREATE PROC dbo.p AS\nSELECT {} FROM t", "a, ".repeat(30));
        let b = format!("CREATE PROC dbo.p AS\nSELECT {} FROM u", "a, ".repeat(30));
        let model = model_editor::construieste(&a, &b);
        for wrap in [None, Some(20)] {
            let (r, start) = randuri_ui(&model, false, false, texte::Limba::Romana, wrap);
            let etichete: Vec<usize> = (0..r.len()).filter(|&i| r[i].fel == 2).collect();
            assert_eq!(etichete.len(), model.diferente.len(), "{wrap:?}");
            assert!(r
                .iter()
                .filter(|x| x.fel == 0)
                .all(|x| x.eticheta.is_empty()));
            for (k, &e) in etichete.iter().enumerate() {
                assert_eq!(e + 1, start[k] as usize, "deasupra blocului {k}");
                assert!(!r[e].eticheta.is_empty() && r[e].diferenta == -1);
            }
        }
    }

    /// Cutia blocului: incepe pe randul etichetei, cuprinde randurile de cod
    /// ale diferentei si se inchide pe ultimul; pe partea unde blocul are cod.
    #[test]
    fn cutia_blocului_incepe_la_eticheta_si_se_inchide_pe_ultimul_rand() {
        let a = format!("CREATE PROC dbo.p AS\nSELECT {} FROM t", "a, ".repeat(30));
        let b = format!("CREATE PROC dbo.p AS\nSELECT {} FROM u", "a, ".repeat(30));
        let model = model_editor::construieste(&a, &b);
        for wrap in [None, Some(20)] {
            let (r, start) = randuri_ui(&model, false, false, texte::Limba::Romana, wrap);
            for (k, &s) in start.iter().enumerate() {
                let s = s as usize;
                let eticheta = &r[s - 1];
                assert_eq!(eticheta.fel, 2);
                assert_eq!(eticheta.nr_diferenta, k as i32 + 1);
                assert!(
                    eticheta.cutie_st && eticheta.cutie_dr,
                    "modificat: ambele parti"
                );
                let ale_ei: Vec<usize> = (0..r.len())
                    .filter(|&i| r[i].fel == 0 && r[i].diferenta == k as i32)
                    .collect();
                assert!(ale_ei.iter().all(|&i| r[i].cutie_st && r[i].cutie_dr));
                let jos: Vec<usize> = ale_ei.iter().copied().filter(|&i| r[i].cutie_jos).collect();
                assert_eq!(
                    jos,
                    [*ale_ei.last().unwrap()],
                    "{wrap:?}: un singur capat, ultimul"
                );
            }
            assert!(r
                .iter()
                .filter(|x| x.diferenta < 0 && x.fel != 2)
                .all(|x| !x.cutie_st && !x.cutie_dr));
        }
    }

    #[test]
    fn wrapul_pastreaza_partile_aliniate() {
        let a = format!("CREATE PROC dbo.p AS\nSELECT {} FROM t", "a, ".repeat(30));
        let b = format!("CREATE PROC dbo.p AS\nSELECT {} FROM u", "a, ".repeat(30));
        let model = model_editor::construieste(&a, &b);
        let (fara, _) = randuri_ui(&model, false, false, texte::Limba::Romana, None);
        let (cu, start) = randuri_ui(&model, false, false, texte::Limba::Romana, Some(20));
        assert!(cu.len() > fara.len(), "linia lunga s-a rupt");
        let continuari: Vec<&RandUi> = cu.iter().filter(|r| r.continuare).collect();
        assert!(!continuari.is_empty());
        assert!(continuari
            .iter()
            .all(|r| r.st_nr.is_empty() && r.dr_nr.is_empty() && !r.inceput));
        assert!(continuari.iter().all(|r| !r.st_lipsa && !r.dr_lipsa));
        // diferenta incepe pe primul ei rand si isi numara continuarile
        let s = start[0] as usize;
        assert!(cu[s].inceput && !cu[s].continuare);
        let din_bloc = cu.iter().filter(|r| r.diferenta == 0).count() as i32;
        assert_eq!(cu[s].linii_bloc, din_bloc);
        // o parte fara linie ramane hasurata si pe continuari
        let model = model_editor::construieste(
            "SELECT 1",
            &format!("SELECT 1\nPRINT '{}'", "x".repeat(60)),
        );
        let (cu, _) = randuri_ui(&model, false, false, texte::Limba::Romana, Some(20));
        assert!(cu.iter().any(|r| r.continuare && r.st_lipsa && !r.dr_lipsa));
    }

    /// Dublu clicul pe un rand din lista: modelul trebuie sa ramana acelasi
    /// intre redesenari, altfel Slint recreeaza randul dupa primul clic.
    #[test]
    fn redesenarea_pastreaza_modelul_randurilor() {
        let ui = fereastra_lista();
        let mut lista = lista_de_proba();
        incarca_lista(&ui, &antet_lista(), &lista);
        let inainte = ui.get_randuri();
        lista.selecteaza(1);
        lista.comuta_revizuit(1);
        incarca_lista(&ui, &antet_lista(), &lista);
        let dupa = ui.get_randuri();
        assert!(inainte == dupa, "acelasi model: randurile nu se recreeaza");
        assert!(
            dupa.row_data(1).unwrap().revizuit,
            "dar datele s-au schimbat"
        );
        // alt numar de randuri: model nou, firesc
        lista.cauta("usp_Stoc");
        incarca_lista(&ui, &antet_lista(), &lista);
        assert_eq!(ui.get_randuri().row_count(), 1);
    }

    /// Harta (etapa 19, dupa macheta lui Mihai): se deschide cu butonul din
    /// bara si cu `h`, se inchide cu Esc, bula urmeaza mouse-ul, iar clicul
    /// pe un patrat ajunge la `clic-harta` cu obiectul lui. Totul prin mouse
    /// si tastatura adevarate: un `TouchArea` acoperit ar trece de `invoke_`.
    #[test]
    fn harta_se_deschide_cu_butonul_si_h_iar_clicul_pe_patrat_alege_obiectul() {
        use slint::platform::{Key, PointerEventButton, WindowEvent};
        use slint::LogicalPosition;
        use std::cell::RefCell;
        let ui = fereastra_lista();
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));
        let lista = lista_de_proba();
        incarca_lista(&ui, &antet_lista(), &lista);
        leaga_harta(&ui);
        ui.show().expect("fereastra se arata");
        let tasta = |t: SharedString| {
            ui.window()
                .dispatch_event(WindowEvent::KeyPressed { text: t.clone() });
            ui.window()
                .dispatch_event(WindowEvent::KeyReleased { text: t });
        };
        let muta = |x: f32, y: f32| {
            ui.window().dispatch_event(WindowEvent::PointerMoved {
                position: LogicalPosition::new(x, y),
            });
        };
        let clic = |x: f32, y: f32| {
            let position = LogicalPosition::new(x, y);
            muta(x, y);
            ui.window().dispatch_event(WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            });
            ui.window().dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        };

        // `h` singur nu face nimic (se scrie in cautare); Ctrl+H o deschide,
        // cu un grup pe tip; Esc o inchide
        tasta("h".into());
        assert!(!ui.get_arata_harta(), "h fara Ctrl nu deschide harta");
        cu_ctrl(&ui, "h");
        assert!(ui.get_arata_harta(), "Ctrl+H deschide harta");
        let tipuri: std::collections::BTreeSet<String> =
            ui.get_randuri().iter().map(|r| r.tip.to_string()).collect();
        assert_eq!(ui.get_harta().row_count(), tipuri.len());
        tasta(Key::Escape.into());
        assert!(!ui.get_arata_harta(), "Esc inchide harta");

        // butonul din bara: primul buton din dreapta progresului
        let x = (600..1440)
            .step_by(4)
            .map(|x| x as f32)
            .find(|&x| {
                clic(x, 60.0 + 22.0);
                ui.get_arata_harta()
            })
            .expect("butonul Harta raspunde la clic");
        assert!(x > 600.0);

        // primul patrat al primului grup: bula, apoi clicul il alege
        let alese: Rc<RefCell<Vec<Option<usize>>>> = Rc::default();
        let a = alese.clone();
        let slab = ui.as_weak();
        ui.on_clic_harta(move |g, x, y| {
            let ui = slab.upgrade().expect("fereastra");
            a.borrow_mut().push(rand_din_harta(&ui, g, x, y));
        });
        let primul = ui.get_harta().row_data(0).expect("un grup");
        let asteptat = primul.randuri.row_data(0).expect("o celula") as usize;
        let y = (80..400)
            .map(|y| y as f32)
            .find(|&y| {
                muta(42.0, y);
                ui.get_bula() != ""
            })
            .expect("bula apare peste primul patrat");
        let r = &ui.get_randuri().row_data(asteptat).expect("randul");
        assert_eq!(ui.get_bula(), format!("{}.{}", r.schema, r.nume));
        clic(42.0, y + 2.0);
        assert_eq!(*alese.borrow(), [Some(asteptat)]);
        let _ = ui.hide();
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

    /// Coloanele noi din lista (Similar, Deploy) si starea barei de filtre
    /// vin din model, nu din desen.
    #[test]
    fn autofitul_tine_cont_si_de_cap_si_de_randuri() {
        use model_lista::Ordine;
        let mut lista = lista_de_proba();
        // capul „Baza" e scurt, dar numele bazei e lung
        let baza = latime_autofit(&lista, Ordine::Baza);
        assert!(baza > 80.0, "numele bazei nu incape: {baza}");
        // o coloana fara continut ramane cat capul ei
        let tip = latime_autofit(&lista, Ordine::Tip);
        assert!(tip > 20.0 && tip < baza, "{tip} vs {baza}");
        // pe o lista filtrata la zero ramane latimea capului
        lista.cauta("nu-exista");
        let gol = latime_autofit(&lista, Ordine::DiferaIn);
        assert!(gol > 20.0, "capul coloanei tot trebuie sa incapa: {gol}");
    }

    /// V9 din `cercetare/audit-qa-2026-11-18.md`: bifa „Revizuit" de pe un
    /// rand bifa alt obiect. `lista.slint` trimitea `comuta-revizuit(i)`, dar
    /// legatura din Rust ignora indicele si bifa randul **selectat**; clicul
    /// pe bifa nu selecteaza randul (TouchArea-ul bifei sta peste cel al
    /// randului). In plus `Bifa` isi scria singura `bifat`, deci randul arata
    /// bifat fara sa fie — si un rand reciclat de `ListView` arata apoi
    /// starea altui obiect.
    #[test]
    fn bifa_unui_rand_nu_bifeaza_alt_rand() {
        use slint::platform::{PointerEventButton, WindowEvent};
        use slint::LogicalPosition;
        use std::cell::RefCell;
        let ui = fereastra_lista();
        ui.window().set_size(slint::PhysicalSize::new(1440, 900));
        let rand = |nume: &str| RandListaUi {
            schema: "dbo".into(),
            nume: nume.into(),
            tip: "procedura".into(),
            litera: "M".into(),
            ..Default::default()
        };
        ui.set_randuri(ModelRc::new(VecModel::from(vec![
            rand("pA"),
            rand("pB"),
            rand("pC"),
        ])));
        ui.set_selectat(0);
        ui.show().expect("fereastra se arata");

        let bifate: Rc<RefCell<Vec<i32>>> = Rc::default();
        let b = bifate.clone();
        ui.on_comuta_revizuit(move |i| b.borrow_mut().push(i));
        let clicuri: Rc<RefCell<Vec<i32>>> = Rc::default();
        let c = clicuri.clone();
        ui.on_clic_rand(move |i| c.borrow_mut().push(i));

        let clic = |x: f32, y: f32| {
            let position = LogicalPosition::new(x, y);
            ui.window()
                .dispatch_event(WindowEvent::PointerMoved { position });
            ui.window().dispatch_event(WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            });
            ui.window().dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        };
        // primul rand: se cauta pe coloana „Obiect", ca in testul de panou
        let y0 = (0..900)
            .step_by(4)
            .map(|y| y as f32)
            .find(|&y| {
                clic(300.0, y);
                let gasit = clicuri.borrow().last() == Some(&0);
                clicuri.borrow_mut().clear();
                gasit
            })
            .expect("primul rand raspunde la clic");
        // bifa randului al treilea: patrat de 13 px in coloana „Revizuit",
        // deci se cauta si pe latime si pe inaltimea randului
        let randul = 2usize;
        let sus = y0 + 34.0 * randul as f32 - 4.0;
        let mut gasit = false;
        for x in (600..1440).step_by(3) {
            for y in 0..12 {
                clic(x as f32, sus + y as f32 * 3.0);
                if !bifate.borrow().is_empty() {
                    gasit = true;
                    break;
                }
            }
            if gasit {
                break;
            }
        }
        assert!(gasit, "bifa randului al treilea primeste clicul");
        assert_eq!(
            *bifate.borrow(),
            [randul as i32],
            "bifa trebuie sa spuna randul ei, nu pe cel selectat"
        );
        // si nu-si schimba singura desenul: starea vine din model
        assert!(
            !ui.get_randuri().row_data(randul).unwrap().revizuit,
            "bifa nu se deseneaza bifata inainte sa raspunda modelul"
        );
    }
}
