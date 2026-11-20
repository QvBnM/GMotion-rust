//! Ce ecran e activ, ce obiect e deschis, ce face fiecare comanda.
//!
//! **Se strica astfel:** bifa pusa in editor nu se vede in lista (sau invers),
//! `j` / `k` din editor te intorc in lista in loc sa treaca la obiectul
//! urmator, sau obiectul deschis nu mai e cel selectat dupa un filtru. Toate
//! sunt bug-uri de stare, nu de desen — de-aia starea sta aici, fara Slint, si
//! se testeaza ca atare.

use gmotion_fereastra::model_arbore::Arbore;
use gmotion_fereastra::model_editor::{
    construieste_cu, marcheaza_textul_final, previzualizare, LiniePrevizualizare, ModelEditor,
};
use gmotion_fereastra::model_lista::{
    Deploy, Detalii, FiltruRevizuire, Impact, Lista, Obiect, Ordine, Stare,
};
use gmotion_fereastra::model_taburi::{Activ, Cheie, Pas, Refuz, Taburi};
use gmotion_modul_diff_obiecte::{self as modul, Baza, Editari, Rand, Revizuire, TextFinal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ecran {
    Lista,
    Editor,
    /// Planul de deploy: ce s-a pus „in plan", garda si scrierea.
    Deploy,
}

/// Tot ce poate cere utilizatorul, din oricare ecran.
///
/// Fara `Eq`: `FiltreazaScor` tine procente.
#[derive(Debug, Clone, PartialEq)]
pub enum Comanda {
    /// Deschide / inchide un nod din arborele listei (cheia din `Nod`).
    ComutaNod(String),
    /// Cautarea din coloana editorului: filtreaza doar arborele; obiectul
    /// deschis si lista raman cum sunt.
    CautaInArbore(String),
    MutaSelectia(i32),
    Selecteaza(usize),
    /// Clic pe un rand (sau pe patratul lui de pe harta): il selecteaza si
    /// deschide panoul de detalii; pe randul deja deschis, inchide panoul.
    ClicPeRand(usize),
    /// Butonul X din panoul de detalii, sau Esc.
    InchideDetalii,
    /// Nota obiectului `(baza, schema.nume)` (text gol = sterge). Obiectul
    /// vine explicit, nu din selectie: campul poate pierde focul dupa ce
    /// selectia s-a mutat deja pe alt rand.
    PuneNota(String, String, String),
    Deschide,
    // ---- taburile editorului (etapa 16; regulile in `model_taburi`)
    /// Fixeaza obiectul selectat intr-un tab (dublu clic in coloana listei).
    FixeazaSelectia,
    /// Dublu clic pe tabul de previzualizare.
    FixeazaPrevizualizarea,
    /// Clic pe un tab: `None` = previzualizarea, `Some(i)` = al i-lea fixat.
    ActiveazaTab(Option<usize>),
    /// X-ul unui tab.
    InchideTab(Option<usize>),
    /// `Ctrl+W`: tabul activ; activ devine vecinul din stanga.
    InchideTabActiv,
    InapoiLaLista,
    ObiectUrmator,
    ObiectAnterior,
    ComutaRevizuit,
    /// Bifa „Revizuit" de pe un **anumit** rand vizibil (clic cu mouse-ul).
    /// Randul devine si cel selectat: altfel bifa unui rand ar bifa alt
    /// obiect decat cel apasat (V9 din auditul QA).
    ComutaRevizuitLa(usize),
    Filtreaza(Option<Stare>),
    FiltreazaTip(Option<String>),
    /// Filtrul pe baza comparata (cand sunt mai multe).
    FiltreazaBaza(Option<String>),
    Cauta(String),
    Sorteaza(Ordine),
    /// Filtrul pe ce ai facut cu obiectul (nerevizuite, omise, deployate...).
    FiltreazaRevizuire(Option<FiltruRevizuire>),
    /// Intervalul de similaritate, in procente.
    FiltreazaScor(f64, f64),
    /// Doar obiectele cu impactul asta (`None` = toate).
    FiltreazaImpact(Option<Impact>),
    /// Un bloc tras si lasat (`bloc`: indexul lui in diff, ca in editor).
    /// `loc` `None` = lasat pe locul sugerat (ca „Aplica", doar blocul asta);
    /// altfel unde l-a lasat omul: peste un bloc il inlocuieste, intre
    /// blocuri se insereaza.
    Plaseaza {
        bloc: usize,
        loc: Option<gmotion_core::Loc>,
    },
    /// Clic pe un nume din „Folosit de" / „Foloseste": obiectul `schema.nume`
    /// din aceeasi baza devine selectia listei, daca e in lista.
    SelecteazaDependenta(String),
    /// „In plan" pe tot ce se vede; `true` = si pe ce s-a deployat deja.
    PuneVizibileleInPlan(bool),
    /// Pune sau scoate obiectul selectat din plan.
    ComutaInPlan,
    /// Scoate din plan obiectul dat, ca `baza|schema.nume` (din ecranul de
    /// deploy, unde randul stie din ce baza e).
    ScoateDinPlan(String, String),
    /// Deschide planul de deploy; ramane pe loc daca planul e gol.
    DeschideDeploy,
    // ---- editorul: textul final (vezi `modul::Editari`)
    /// Aplica diferenta cu numarul asta (de la 0, ca in editor); `None` =
    /// diferenta curenta.
    AplicaDiferenta(Option<usize>),
    AplicaToate,
    Reset,
    /// Ctrl+Z: ultima schimbare a textului final, pe orice obiect.
    Anuleaza,
    /// „Editeaza": textul final scris de mana, cu cheia obiectului pentru
    /// care s-a scris (`baza/schema.nume`, vezi `Aplicatie::id_obiect`).
    /// Textul venit pentru alt obiect se refuza: campul din Slint isi rupe
    /// legatura cu modelul la prima tasta, deci dupa „‹ ›" tinea textul
    /// obiectului dinainte (F-A1 din auditul QA).
    SalveazaText {
        id: String,
        text: String,
    },
    /// „Omite" (`o`): analizat, lasat in afara deploy-ului.
    ComutaOmis,
    /// „Uneste ciclul cursor": blocul de cursor se ia intreg.
    ComutaUnireaCursorului(bool),
    /// Diferenta curenta s-a mutat (n / p, clic pe harta).
    MergiLaDiferenta(usize),
    // ---- modul ghidat: ia diferentele pe rand
    PornesteGhidat,
    OpresteGhidat,
    /// Ce se pune in baza pentru blocul curent: din sursa (`true`) sau
    /// ramane cum e in tinta (`false`).
    GhidatAlege(bool),
    /// Aplica alegerea si treci la urmatoarea diferenta.
    GhidatAplica,
    /// Lasa diferenta asa si treci la urmatoarea.
    GhidatSari,
}

/// Ce se spune dupa o comanda pe taburi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MesajTab {
    /// `j` / `k`: nu mai e niciun obiect nefixat in directia asta.
    Capat,
    /// Al noualea tab, cu toate cele 8 avand text final editat: refuzat.
    ToateEditate,
}

/// Coloana listei din editor (etapa 16): latimea trasa, pinul, partea si
/// banda. Supravietuiesc inchiderii, in sesiune (cheia `coloana`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StareColoana {
    pub latime: f32,
    pub fixata: bool,
    pub dreapta: bool,
    pub banda: bool,
}

impl StareColoana {
    pub fn in_sesiune(&self, s: &mut gmotion_platforma::Sesiune) {
        s.date.insert(
            "coloana".to_string(),
            serde_json::json!({
                "latime": self.latime,
                "fixata": self.fixata,
                "dreapta": self.dreapta,
                "banda": self.banda,
            }),
        );
    }

    /// Ce s-a salvat; latimea se tine intre 200 si 520 px, ca in fereastra.
    /// O valoare lipsa sau stricata inseamna implicitul ferestrei.
    pub fn din_sesiune(s: &gmotion_platforma::Sesiune) -> Option<Self> {
        let v = s.date.get("coloana")?.as_object()?;
        let b = |k: &str| v.get(k).and_then(serde_json::Value::as_bool);
        Some(StareColoana {
            latime: (v.get("latime")?.as_f64()? as f32).clamp(200.0, 520.0),
            fixata: b("fixata")?,
            dreapta: b("dreapta")?,
            banda: b("banda")?,
        })
    }
}

/// Taburile fixate, in sesiune (cheia `taburi`), ca lista de (baza, obiect).
pub fn taburi_in_sesiune(t: &Taburi, s: &mut gmotion_platforma::Sesiune) {
    let v = t.de_salvat();
    if v.is_empty() {
        s.date.remove("taburi");
    } else if let Ok(j) = serde_json::to_value(v) {
        s.date.insert("taburi".to_string(), j);
    }
}

/// Taburile fixate salvate; o valoare stricata inseamna fara taburi.
pub fn taburi_din_sesiune(s: &gmotion_platforma::Sesiune) -> Taburi {
    let v: Vec<Cheie> = s
        .date
        .get("taburi")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    Taburi::din_sesiune(v)
}

/// Modul ghidat: diferentele, una cate una, cu o alegere pentru fiecare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ghidat {
    /// Alegerea pentru diferenta curenta: din sursa (implicit) sau din tinta.
    pub din_sursa: bool,
    pub aplicate: usize,
    pub sarite: usize,
    /// S-au terminat diferentele: se arata dialogul de final.
    pub terminat: bool,
}

/// Pentru fiecare obiect al unei baze, cine il foloseste.
type DependenteBaza = std::rc::Rc<modul::Dependente>;

/// Ce arata panoul de detalii al listei (vezi `gmotion_fereastra::PanouDetalii`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanouDetalii {
    pub deschis: bool,
    pub baza: String,
    pub obiect: String,
    /// 0 scazut, 1 mediu, 2 ridicat.
    pub impact: Option<i32>,
    pub motive: String,
    pub nota: String,
    /// Cine foloseste obiectul, in Modificata (`schema.nume`).
    pub folosit_de: Vec<String>,
    /// Ce foloseste obiectul.
    pub foloseste: Vec<String>,
}

impl PanouDetalii {
    /// Forma pe care o primeste fereastra.
    pub fn pentru_fereastra(&self) -> gmotion_fereastra::PanouDetalii<'_> {
        gmotion_fereastra::PanouDetalii {
            deschis: self.deschis,
            baza: &self.baza,
            obiect: &self.obiect,
            impact: self.impact,
            motive: &self.motive,
            nota: &self.nota,
            folosit_de: &self.folosit_de,
            foloseste: &self.foloseste,
        }
    }
}

/// Cheia modelului tinut minte in `Aplicatie::model`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CheieModel {
    baza: String,
    id: String,
    /// Amprenta textului din dreapta (final sau al bazei).
    text: String,
    manual: bool,
    uneste_cursorul: bool,
}

/// Starea aplicatiei: lista, ecranul activ si progresul revizuirii.
pub struct Aplicatie {
    lista: Lista,
    ecran: Ecran,
    revizuire: Revizuire,
    /// Randurile compararii, pe `schema.nume`: de acolo vin amprentele.
    randuri: Vec<Rand>,
    sursa: Baza,
    /// Bazele comparate, pe nume. De cand o referinta se compara cu mai
    /// multe, obiectul se cauta in baza randului lui, nu intr-una singura.
    tinte: Vec<(String, Baza)>,
    /// Textele finale (Aplica, Reset, scris de mana), salvate in sesiune.
    editari: Editari,
    /// Pentru Ctrl+Z: obiectul si textul lui final de dinainte de fiecare
    /// schimbare. Nu se salveaza: anularea tine cat sesiunea de lucru.
    istoric: Vec<(Rand, Option<TextFinal>)>,
    /// Diferenta curenta din editor; ramane pe loc dupa „Aplica", cand
    /// diferenta aplicata dispare si urmatoarea ii ia locul.
    diferenta: usize,
    ghidat: Option<Ghidat>,
    /// Cine foloseste fiecare obiect, pe baza comparata; lexarea unei baze
    /// intregi se face o data, la prima cerere pe ea (`impact_obiect`).
    dependente: std::collections::BTreeMap<String, DependenteBaza>,
    /// Aceleasi legaturi, in Martor: ce **foloseste** un obiect se ia din
    /// codul care s-ar deploia, deci din Martor (in afara de Extra).
    dependente_martor: DependenteBaza,
    /// „Uneste ciclul cursor": schimba taierea blocurilor in editor.
    uneste_cursorul: bool,
    /// Modelul editorului, tinut minte pe cheia lui. Se refacea la **fiecare
    /// desenare**: 2,4 s pentru un obiect de 30.000 de linii, deci „Aplica"
    /// insemna 7-10 secunde de fereastra inghetata, iar o bifa (`r`, `i`) tot
    /// 2,4 s, desi nu schimba textul (P1 din auditul QA).
    model: std::cell::RefCell<Option<(CheieModel, ModelEditor)>>,
    /// De cate ori s-a construit modelul. Numai asa se poate proba ca
    /// desenarea nu-l reface (P1); altfel testul ar masura timpi.
    refaceri_model: std::cell::Cell<u64>,
    /// A cata comparare e asta. Dat de un numarator, la construire: ce s-a
    /// calculat pe o comparare (planul de deploy, garda lui) nu se poate
    /// refolosi dupa o recomparare, si uitarea era bug-ul C2 din auditul QA.
    id_comparare: u64,
    /// Ultima comanda „In plan" a fost refuzata (obiectul nu are cod de
    /// deploiat). Ecranul spune de ce; se sterge la urmatoarea comanda (F-A18).
    refuzat_in_plan: bool,
    /// Taburile editorului. Obiectul din tabul activ e mereu si selectia
    /// listei (`selecteaza_cheie`): asa raman valabile toate comenzile care
    /// lucreaza pe selectie.
    taburi: Taburi,
    /// Ce trebuie spus dupa ultima comanda pe taburi; se sterge la urmatoarea.
    mesaj_tab: Option<MesajTab>,
    /// Panoul de detalii din dreapta listei e deschis.
    detalii: bool,
    /// Arborele listei (etapa 19): ce baze si dosare sunt inchise.
    arbore: Arbore,
}

/// Un obiect de calculat pe alt fir: cheia lui si cele doua texte.
#[derive(Debug, Clone)]
pub struct LucruDetalii {
    pub cheie: (String, String),
    pub a: String,
    pub b: String,
    pub setari: Vec<&'static str>,
}

impl LucruDetalii {
    /// „Difera in" si `+N −M`, ca in lista. Fara stare: merge pe orice fir.
    pub fn calculeaza(&self) -> Detalii {
        let d = modul::detalii_obiect(&self.a, &self.b, &self.setari);
        Detalii {
            difera_in: d.difera_in,
            linii_plus: d.linii_plus,
            linii_minus: d.linii_minus,
        }
    }
}

/// Numaratorul compararilor din sesiunea asta.
fn urmatorul_id_comparare() -> u64 {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl Aplicatie {
    /// `randuri` sunt randurile compararii; `sursa`, obiectele pregatite ale
    /// referintei, `tinte`, ale fiecarei baze comparate.
    pub fn noua(
        randuri: Vec<Rand>,
        sursa: Baza,
        tinte: Vec<(String, Baza)>,
        revizuire: Revizuire,
    ) -> Self {
        // Legaturile dintre obiecte, o data per baza: lista le arata pe
        // fiecare rand (coloanele „Folosit de" si „Impact"), deci nu se mai
        // pot calcula lenes, la deschiderea panoului.
        let dependente: std::collections::BTreeMap<String, DependenteBaza> = tinte
            .iter()
            .map(|(n, b)| (n.clone(), std::rc::Rc::new(modul::impact::dependente(b))))
            .collect();
        let dependente_martor = std::rc::Rc::new(modul::impact::dependente(&sursa));
        let cati = |baza: &str, id: &str| -> usize {
            dependente
                .get(baza)
                .and_then(|d| d.folosit_de.get(id))
                .map_or(0, Vec::len)
        };
        let obiecte: Vec<Obiect> = randuri
            .iter()
            .map(|r| {
                let (schema, nume) = r.obiect.split_once('.').unwrap_or(("dbo", &r.obiect));
                let folosit_de = cati(&r.baza, &r.obiect);
                Obiect {
                    schema: schema.to_string(),
                    nume: nume.to_string(),
                    baza: r.baza.clone(),
                    tip: r.tip.clone(),
                    stare: match r.categorie {
                        modul::Categorie::Diferit => Stare::Modificat,
                        modul::Categorie::Lipsa => Stare::DoarSursa,
                        modul::Categorie::Extra => Stare::DoarBaza,
                        modul::Categorie::Criptat => Stare::Criptat,
                    },
                    detalii: None,
                    revizuit: revizuire.e_revizuit(r),
                    in_plan: false,
                    omis: revizuire.e_omis(r),
                    scor: r.scor,
                    deploy: if revizuire.e_deployat(r) {
                        Deploy::Deployat
                    } else if revizuire.a_fost_deployat(r) {
                        Deploy::VersiuneNoua
                    } else {
                        Deploy::Nu
                    },
                    folosit_de: Some(folosit_de),
                    impact: Some(match modul::impact::estimeaza(r, folosit_de).nivel {
                        modul::Nivel::Scazut => Impact::Scazut,
                        modul::Nivel::Mediu => Impact::Mediu,
                        modul::Nivel::Ridicat => Impact::Ridicat,
                    }),
                }
            })
            .collect();
        // obiectele identice nu ajung in randurile compararii (nu e nimic de
        // spus despre ele), dar lista le numara si le filtreaza: fara ele,
        // categoria „Identice" ar arata mereu 0, iar cautarea n-ar gasi un
        // obiect care exista si e la fel.
        let mut obiecte = obiecte;
        for (nume_baza, tinta) in &tinte {
            for (id, o) in &sursa {
                let are_rand = randuri
                    .iter()
                    .any(|r| &r.baza == nume_baza && &r.obiect == id);
                if are_rand || !tinta.contains_key(id) {
                    continue;
                }
                let (schema, nume) = id.split_once('.').unwrap_or(("dbo", id));
                obiecte.push(Obiect {
                    schema: schema.to_string(),
                    nume: nume.to_string(),
                    baza: nume_baza.clone(),
                    tip: o.tip.clone(),
                    stare: Stare::Identic,
                    detalii: None,
                    revizuit: false,
                    in_plan: false,
                    omis: false,
                    scor: None,
                    deploy: Deploy::Nu,
                    folosit_de: Some(cati(nume_baza, id)),
                    impact: None,
                });
            }
        }
        let mut lista = Lista::noua(obiecte);
        if !lista.vizibile().is_empty() {
            lista.selecteaza(0);
        }
        Aplicatie {
            lista,
            ecran: Ecran::Lista,
            revizuire,
            randuri,
            sursa,
            tinte,
            editari: Editari::noi(),
            istoric: Vec::new(),
            diferenta: 0,
            ghidat: None,
            dependente,
            dependente_martor,
            uneste_cursorul: false,
            model: std::cell::RefCell::default(),
            refaceri_model: std::cell::Cell::new(0),
            id_comparare: urmatorul_id_comparare(),
            refuzat_in_plan: false,
            arbore: Arbore::default(),
            detalii: false,
            taburi: Taburi::noi(),
            mesaj_tab: None,
        }
    }

    /// A cata comparare e asta: doua `Aplicatie` nu au niciodata acelasi id.
    pub fn id_comparare(&self) -> u64 {
        self.id_comparare
    }

    /// Amprenta textelor finale valabile: se schimba la orice editare care
    /// ajunge in deploy, si cu ea se invalideaza planul deschis.
    pub fn amprenta_texte_finale(&self) -> String {
        let mut t = String::new();
        for ((baza, obiect), text) in self.texte_finale() {
            t.push_str(&baza);
            t.push('\0');
            t.push_str(&obiect);
            t.push('\0');
            t.push_str(&text);
            t.push('\u{1}');
        }
        gmotion_core::calc_hash(&t)
    }

    /// Panoul de detalii e deschis (si are un obiect de aratat).
    pub fn detalii_deschise(&self) -> bool {
        self.detalii && self.obiect().is_some()
    }

    /// Impactul estimat al deploy-ului obiectului selectat, si cine il
    /// foloseste in baza comparata.
    pub fn impact_obiect(&self) -> Option<(modul::Impact, Vec<String>)> {
        let (baza, id) = self.cheia_curenta()?;
        let r = self.rand(&baza, &id)?;
        let (cine, _) = self.dependente_obiect()?;
        Some((modul::impact::estimeaza(r, cine.len()), cine))
    }

    /// Legaturile obiectului selectat: cine il foloseste in Modificata, si ce
    /// foloseste el (din Martor; pentru Extra, din Modificata, singurul loc
    /// unde exista).
    pub fn dependente_obiect(&self) -> Option<(Vec<String>, Vec<String>)> {
        let (baza, id) = self.cheia_curenta()?;
        let in_tinta = self.dependente.get(&baza)?;
        let folosit_de = in_tinta.folosit_de.get(&id).cloned().unwrap_or_default();
        let extra = self.obiect().is_some_and(|o| o.stare == Stare::DoarBaza);
        let de_unde = if extra {
            in_tinta.as_ref()
        } else {
            self.dependente_martor.as_ref()
        };
        let foloseste = de_unde.foloseste.get(&id).cloned().unwrap_or_default();
        Some((folosit_de, foloseste))
    }

    /// Tot ce arata panoul de detalii, gata de pus in fereastra. Impactul se
    /// calculeaza doar cu panoul deschis: prima data lexeaza toata baza.
    pub fn panou_detalii(&self) -> PanouDetalii {
        let deschis = self.detalii_deschise();
        // doar cu panoul deschis: prima cerere pe o baza o lexeaza toata
        let impact = if deschis { self.impact_obiect() } else { None };
        let dependente = if deschis {
            self.dependente_obiect().unwrap_or_default()
        } else {
            (Vec::new(), Vec::new())
        };
        PanouDetalii {
            deschis,
            baza: self.obiect().map(|o| o.baza.clone()).unwrap_or_default(),
            obiect: self.obiect().map(|o| o.id()).unwrap_or_default(),
            impact: impact.as_ref().map(|(i, _)| match i.nivel {
                modul::Nivel::Scazut => 0,
                modul::Nivel::Mediu => 1,
                modul::Nivel::Ridicat => 2,
            }),
            motive: impact
                .as_ref()
                .map(|(i, _)| i.motive.join(" · "))
                .unwrap_or_default(),
            // din dependente, nu din impact: si un obiect identic (fara
            // impact, nu se deployeaza) poate fi folosit de altele
            folosit_de: dependente.0,
            foloseste: dependente.1,
            nota: self.nota_obiect().to_string(),
        }
    }

    /// Nota obiectului selectat (gol daca n-are).
    pub fn nota_obiect(&self) -> &str {
        self.cheia_curenta()
            .and_then(|(b, id)| self.revizuire.nota(&b, &id))
            .unwrap_or("")
    }

    /// Textele finale salvate (din sesiune). Cele care nu mai corespund
    /// obiectelor de acum se ignora singure (sunt legate de amprenta).
    pub fn cu_editari(mut self, editari: Editari) -> Self {
        self.editari = editari;
        self
    }

    pub fn editari(&self) -> &Editari {
        &self.editari
    }

    /// Textele finale gata de salvat, fara cele care nu mai corespund.
    pub fn editari_de_salvat(&mut self) -> &Editari {
        self.editari.curata(&self.randuri, &self.baze());
        &self.editari
    }

    /// Textele finale valabile, pe `(baza, schema.nume)`: ce primeste
    /// deploy-ul in locul codului din sursa.
    pub fn texte_finale(&self) -> std::collections::BTreeMap<(String, String), String> {
        self.editari.pentru_deploy(&self.randuri)
    }

    pub fn diferenta_curenta(&self) -> usize {
        self.diferenta
    }

    pub fn ghidat(&self) -> Option<&Ghidat> {
        self.ghidat.as_ref()
    }

    pub fn uneste_cursorul(&self) -> bool {
        self.uneste_cursorul
    }

    /// Obiectul curent are un cursor, deci butonul „Uneste ciclul cursor"
    /// are ce face.
    pub fn are_cursor(&self) -> bool {
        self.obiect()
            .and_then(|o| self.texte(&o.baza, &o.id()))
            .is_some_and(|(a, b)| gmotion_core::are_cursor(a) || gmotion_core::are_cursor(b))
    }

    /// Obiectul curent are text final (deci „Reset" are ce face), si e scris
    /// si de mana?
    pub fn stare_text_final(&self) -> (bool, bool) {
        self.obiect()
            .and_then(|o| self.rand(&o.baza, &o.id()))
            .and_then(|r| self.editari.text_final(r))
            .map_or((false, false), |t| (true, t.manual))
    }

    /// Se poate anula ceva cu Ctrl+Z.
    pub fn se_poate_anula(&self) -> bool {
        !self.istoric.is_empty()
    }

    /// Textul curent al obiectului din dreapta (final sau al bazei) si cel al
    /// sursei: ce arata „Copiaza" si ce se editeaza de mana.
    pub fn texte_curente(&self) -> Option<(String, String)> {
        let (baza, id) = self.cheia_curenta()?;
        let r = self.rand(&baza, &id)?;
        let (a, b) = self.texte(&baza, &id)?;
        Some((a.to_string(), self.editari.curent(r, b).to_string()))
    }

    pub fn ecran(&self) -> Ecran {
        self.ecran
    }

    pub fn lista(&self) -> &Lista {
        &self.lista
    }

    pub fn revizuire(&self) -> &Revizuire {
        &self.revizuire
    }

    /// Obiectul selectat (in lista) sau deschis (in editor) — acelasi, mereu:
    /// editorul se deschide pe selectie, iar `j` / `k` din editor **muta
    /// selectia**, ca intoarcerea in lista sa cada unde trebuie.
    pub fn obiect(&self) -> Option<&Obiect> {
        self.lista.obiect_selectat()
    }

    /// Cheia obiectului deschis, `baza/schema.nume`. Pleaca spre fereastra si
    /// se intoarce cu textul scris de mana, ca sa nu poata ajunge pe alt
    /// obiect (F-A1).
    pub fn id_obiect(&self) -> Option<String> {
        self.obiect()
            .map(|o| format!("{}/{}.{}", o.baza, o.schema, o.nume))
    }

    fn rand(&self, baza: &str, id: &str) -> Option<&Rand> {
        self.randuri
            .iter()
            .find(|r| r.baza == baza && r.obiect == id)
    }

    /// Obiectele pregatite ale unei baze comparate.
    fn tinta(&self, baza: &str) -> Option<&Baza> {
        self.tinte.iter().find(|(n, _)| n == baza).map(|(_, b)| b)
    }

    /// Textele pregatite ale obiectului, daca exista in ambele baze.
    fn texte(&self, baza: &str, id: &str) -> Option<(&str, &str)> {
        match (self.sursa.get(id), self.tinta(baza).and_then(|t| t.get(id))) {
            (Some(a), Some(b)) => Some((a.afisare.as_str(), b.afisare.as_str())),
            _ => None,
        }
    }

    /// Baza si `schema.nume` ale obiectului curent.
    fn cheia_curenta(&self) -> Option<(String, String)> {
        self.obiect().map(Obiect::cheie)
    }

    /// Modelul editorului pentru obiectul curent. `None` pentru obiectele care
    /// exista doar intr-o parte: nu au cu ce fi comparate.
    ///
    /// In dreapta e **textul final** (daca exista), nu textul bazei: editorul
    /// arata mereu sursa fata de ce va ajunge in baza, iar liniile schimbate
    /// fata de baza sunt marcate (adus / scris de tine).
    pub fn model_editor(&self) -> Option<std::cell::Ref<'_, ModelEditor>> {
        let cheie = self.cheie_model()?;
        {
            let mut c = self.model.borrow_mut();
            if c.as_ref().map(|(k, _)| k) != Some(&cheie) {
                *c = Some((cheie, self.construieste_modelul()?));
            }
        }
        std::cell::Ref::filter_map(self.model.borrow(), |x| x.as_ref().map(|(_, m)| m)).ok()
    }

    /// De ce depinde modelul editorului: obiectul, textul lui final de acum
    /// si taierea blocurilor. Altceva (bifa, „in plan", filtrul listei) nu-l
    /// schimba, deci nu-l reface.
    fn cheie_model(&self) -> Option<CheieModel> {
        let (baza, id) = self.cheia_curenta()?;
        let (_, b) = self.texte(&baza, &id)?;
        let (text, manual) = match self.rand(&baza, &id) {
            Some(r) => (
                self.editari.curent(r, b),
                self.editari.text_final(r).is_some_and(|t| t.manual),
            ),
            None => (b, false),
        };
        Some(CheieModel {
            baza,
            id,
            text: gmotion_core::calc_hash(text),
            manual,
            uneste_cursorul: self.uneste_cursorul,
        })
    }

    /// De cate ori s-a construit modelul editorului de la pornire. Pe un
    /// obiect de 30.000 de linii o construire tine 2,4 s, deci numarul asta
    /// e si masura inghetarilor (P1 din auditul QA).
    pub fn refaceri_model(&self) -> u64 {
        self.refaceri_model.get()
    }

    fn construieste_modelul(&self) -> Option<ModelEditor> {
        self.refaceri_model.set(self.refaceri_model.get() + 1);
        // un obiect criptat n-are text de comparat: editorul arata un diff
        // fals („totul sters"), iar „Aplica toate" salva un text final gol
        // (F-A14 din auditul QA)
        if self.obiect().is_some_and(|o| o.stare == Stare::Criptat) {
            return None;
        }
        let (baza, id) = self.cheia_curenta()?;
        let (a, b) = self.texte(&baza, &id)?;
        let Some(r) = self.rand(&baza, &id) else {
            return Some(construieste_cu(a, b, self.uneste_cursorul));
        };
        let curent = self.editari.curent(r, b);
        let mut m = construieste_cu(a, curent, self.uneste_cursorul);
        let manual = self.editari.text_final(r).is_some_and(|t| t.manual);
        marcheaza_textul_final(&mut m, b, curent, manual);
        Some(m)
    }

    /// Propozitia pentru previzualizare si detaliile randului curent.
    pub fn texte_obiect(&self) -> Option<(&str, &str, Vec<&'static str>)> {
        let (baza, id) = self.cheia_curenta()?;
        let (a, b) = self.texte(&baza, &id)?;
        let setari = match (
            self.sursa.get(&id),
            self.tinta(&baza).and_then(|t| t.get(&id)),
        ) {
            (Some(x), Some(y)) => modul::diferente_ascunse(x, y),
            _ => Vec::new(),
        };
        Some((a, b, setari))
    }

    /// Ce trebuie calculat ca toate obiectele modificate sa aiba „Difera in"
    /// si `+N −M`: textele copiate, ca sa mearga pe alt fir. Pe firul
    /// ferestrei, 60 de obiecte calculate la un filtru faceau 1,2 s (proba
    /// pe Windows, 2026-11-19).
    pub fn lucrari_detalii(&self) -> Vec<LucruDetalii> {
        self.lista
            .chei_fara_detalii()
            .into_iter()
            .filter_map(|(baza, id)| {
                let a = self.sursa.get(&id)?;
                let b = self.tinta(&baza).and_then(|t| t.get(&id))?;
                Some(LucruDetalii {
                    setari: modul::diferente_ascunse(a, b),
                    a: a.afisare.clone(),
                    b: b.afisare.clone(),
                    cheie: (baza, id),
                })
            })
            .collect()
    }

    /// Detaliile calculate pe alt fir, puse pe obiectul lor.
    pub fn pune_detalii(&mut self, cheie: &(String, String), d: Detalii) {
        self.lista.pune_detalii_cheie(cheie, d);
    }

    /// Calculeaza detaliile („Difera in", `+N −M`) pentru randurile de pe
    /// ecran, daca lipsesc. Restul listei ramane necalculat.
    pub fn asigura_detalii(&mut self, de_la: usize, pana_la: usize) {
        for p in self.lista.fara_detalii(de_la, pana_la) {
            let Some(o) = self.lista.obiect(p) else {
                continue;
            };
            let (baza, id) = o.cheie();
            let (Some(a), Some(b)) = (
                self.sursa.get(&id),
                self.tinta(&baza).and_then(|t| t.get(&id)),
            ) else {
                continue;
            };
            let setari = modul::diferente_ascunse(a, b);
            let d = modul::detalii_obiect(&a.afisare, &b.afisare, &setari);
            self.lista.pune_detalii(
                p,
                Detalii {
                    difera_in: d.difera_in,
                    linii_plus: d.linii_plus,
                    linii_minus: d.linii_minus,
                },
            );
        }
    }

    /// Executa o comanda. Intoarce `true` daca starea de revizuire s-a
    /// schimbat, deci sesiunea trebuie salvata.
    pub fn comanda(&mut self, c: Comanda) -> bool {
        // arborele listei (etapa 19): starea nodurilor nu e a sesiunii
        if let Comanda::ComutaNod(cheie) = &c {
            self.arbore.comuta(cheie);
            return false;
        }
        if let Comanda::CautaInArbore(text) = &c {
            self.arbore.cauta(text);
            return false;
        }
        let inainte = self.lista.selectat();
        let rez = self.executa(c);
        // obiectul selectat (j / k, tab, alt ecran) nu sta intr-un dosar inchis
        if let Some(p) = self.lista.selectat().filter(|p| Some(*p) != inainte) {
            self.arbore.deschide_calea(&self.lista, p);
        }
        rez
    }

    /// Arborele din lista: ce noduri sunt inchise.
    pub fn arbore(&self) -> &Arbore {
        &self.arbore
    }

    fn executa(&mut self, c: Comanda) -> bool {
        self.refuzat_in_plan = false;
        self.mesaj_tab = None;
        match c {
            Comanda::MutaSelectia(pas) => self.lista.muta_selectia(pas),
            Comanda::Selecteaza(p) => {
                self.lista.selecteaza(p);
                // in editor, clicul in coloana listei incarca obiectul in
                // previzualizare (sau ii activeaza tabul, daca e fixat)
                if self.ecran == Ecran::Editor {
                    if let Some(c) = self.cheia_curenta() {
                        self.taburi.deschide(c);
                        self.dupa_schimbarea_tabului();
                    }
                }
            }
            // fixatele stau in sesiune: orice schimbare a lor se salveaza
            Comanda::FixeazaSelectia => {
                if let Some(c) = self.cheia_curenta() {
                    let editate = self.chei_editate();
                    let r = self.taburi.fixeaza(c, |k| editate.contains(k));
                    self.rezultat_fixare(r);
                }
                return true;
            }
            Comanda::FixeazaPrevizualizarea => {
                let editate = self.chei_editate();
                let r = self.taburi.fixeaza_previzualizarea(|k| editate.contains(k));
                self.rezultat_fixare(r);
                return true;
            }
            Comanda::ActiveazaTab(i) => {
                self.taburi
                    .activeaza(i.map_or(Activ::Previzualizare, Activ::Fixat));
                self.dupa_schimbarea_tabului();
            }
            Comanda::InchideTab(i) => {
                self.taburi
                    .inchide(i.map_or(Activ::Previzualizare, Activ::Fixat));
                self.dupa_schimbarea_tabului();
                return true;
            }
            Comanda::InchideTabActiv => {
                self.taburi.inchide_activ();
                self.dupa_schimbarea_tabului();
                return true;
            }
            Comanda::ClicPeRand(p) => {
                if self.detalii && self.lista.selectat() == Some(p) {
                    self.detalii = false;
                } else {
                    self.lista.selecteaza(p);
                    self.detalii = self.lista.selectat() == Some(p);
                }
            }
            Comanda::InchideDetalii => self.detalii = false,
            Comanda::PuneNota(baza, id, text) => {
                if self.rand(&baza, &id).is_none() {
                    return false;
                }
                return self.revizuire.pune_nota(&baza, &id, &text);
            }
            Comanda::Deschide => {
                self.diferenta = 0;
                self.ghidat = None;
                self.istoric.clear();
                // un obiect care exista doar intr-o parte n-are ce arata in
                // editor: ecranul ramane pe lista
                if self.model_editor().is_some() {
                    self.ecran = Ecran::Editor;
                    if let Some(c) = self.cheia_curenta() {
                        self.taburi.deschide(c);
                    }
                    // la intoarcere, lista arata obiectul de unde ai plecat
                    self.detalii = true;
                }
            }
            Comanda::InapoiLaLista => self.ecran = Ecran::Lista,
            // din editor, obiectul urmator se deschide direct: altfel ar
            // trebui drum dus-intors prin lista pentru fiecare obiect
            Comanda::ObiectUrmator if self.ecran == Ecran::Editor => self.pas_tab(1),
            Comanda::ObiectAnterior if self.ecran == Ecran::Editor => self.pas_tab(-1),
            Comanda::ObiectUrmator => {
                self.treci(1);
                self.diferenta = 0;
                self.ghidat = None;
                // Ctrl+Z e al obiectului deschis, ca in v3.4 (`ui.py:3205`):
                // cu un istoric global, anularea schimba textul altui obiect,
                // si nu se vede nimic pe ecran (F-A4 din auditul QA)
                self.istoric.clear();
            }
            Comanda::ObiectAnterior => {
                self.treci(-1);
                self.diferenta = 0;
                self.ghidat = None;
                self.istoric.clear();
            }
            Comanda::ComutaRevizuit => return self.comuta_revizuit(),
            Comanda::ComutaRevizuitLa(p) => {
                if self.lista.obiect(p).is_none() {
                    return false;
                }
                self.lista.selecteaza(p);
                return self.comuta_revizuit();
            }
            Comanda::Filtreaza(s) => self.lista.filtreaza_stare(s),
            Comanda::FiltreazaTip(t) => self.lista.filtreaza_tip(t),
            Comanda::FiltreazaBaza(b) => self.lista.filtreaza_baza(b),
            Comanda::Cauta(t) => self.lista.cauta(&t),
            Comanda::Sorteaza(o) => self.lista.sorteaza(o),
            Comanda::FiltreazaRevizuire(f) => self.lista.filtreaza_revizuire(f),
            Comanda::FiltreazaScor(min, max) => self.lista.filtreaza_scor(min, max),
            Comanda::FiltreazaImpact(i) => self.lista.filtreaza_impact(i),
            Comanda::SelecteazaDependenta(id) => {
                if let Some(baza) = self.obiect().map(|o| o.baza.clone()) {
                    // daca nu e vizibil, `selecteaza_cheie` goleste filtrele;
                    // daca nu e in lista deloc (identic intr-o comparare
                    // veche, de pilda), selectia ramane pe loc
                    self.lista.selecteaza_cheie(&(baza, id));
                }
            }
            Comanda::PuneVizibileleInPlan(cu_deployate) => {
                self.lista.pune_vizibilele_in_plan(cu_deployate);
            }
            Comanda::ComutaInPlan => {
                if let Some(p) = self.lista.selectat() {
                    // ca in v3.4: un obiect fara cod de deploiat (doar in baza
                    // modificata, criptat) nu intra, dar se spune de ce
                    self.refuzat_in_plan = !self.lista.comuta_in_plan(p);
                }
            }
            Comanda::ScoateDinPlan(baza, id) => {
                self.lista.scoate_din_plan(&baza, &id);
                // planul golit nu mai are ce arata
                if self.ecran == Ecran::Deploy && self.lista.in_plan().is_empty() {
                    self.ecran = Ecran::Lista;
                }
            }
            Comanda::DeschideDeploy => {
                if !self.lista.in_plan().is_empty() {
                    self.ecran = Ecran::Deploy;
                }
            }
            Comanda::AplicaDiferenta(k) => {
                let k = k.unwrap_or(self.diferenta);
                return self.aplica(k);
            }
            Comanda::Plaseaza { bloc, loc } => {
                let uneste = self.uneste_cursorul;
                return self.schimba_textul(|e, r, a, b| {
                    match loc {
                        None => e.aplica_blocuri(r, a, b, uneste, &[bloc]),
                        Some(l) => e.aplica_blocuri_la(r, a, b, uneste, &[bloc], l),
                    };
                });
            }
            Comanda::AplicaToate => {
                let schimbat = self.schimba_textul(|e, r, sursa, _| e.aplica_toate(r, sursa));
                // ca in v3.4: „Aplica toate" inseamna ca obiectul a fost
                // revizuit (F-A18); o bifa pusa deja ramane
                if schimbat && !self.obiect().is_some_and(|o| o.revizuit) {
                    self.comuta_revizuit();
                }
                return schimbat;
            }
            Comanda::Reset => {
                return self.schimba_textul(|e, r, _, _| {
                    e.reset(r);
                });
            }
            Comanda::SalveazaText { id, text } => {
                if self.id_obiect() != Some(id.clone()) {
                    tracing::warn!(
                        target: "gmotion::app",
                        trimis = %id, curent = ?self.id_obiect(),
                        "text scris de mana pentru alt obiect: refuzat"
                    );
                    return false;
                }
                let text = gmotion_core::pregateste_afisare(&text);
                return self.schimba_textul(move |e, r, _, _| e.pune(r, text, true));
            }
            Comanda::Anuleaza => {
                if let Some((r, inainte)) = self.istoric.pop() {
                    self.editari.restaureaza(&r, inainte);
                    return true;
                }
            }
            Comanda::ComutaOmis => return self.comuta_omis(),
            Comanda::ComutaUnireaCursorului(cu) => {
                self.uneste_cursorul = cu;
                // blocurile se taie altfel, deci diferenta curenta poate sa
                // nu mai existe
                let n = self.numar_diferente();
                self.diferenta = self.diferenta.min(n.saturating_sub(1));
            }
            Comanda::MergiLaDiferenta(k) => self.diferenta = k,
            Comanda::PornesteGhidat => {
                self.ghidat = Some(Ghidat {
                    din_sursa: true,
                    aplicate: 0,
                    sarite: 0,
                    terminat: false,
                });
                self.diferenta = 0;
                self.verifica_ghidat();
            }
            Comanda::OpresteGhidat => self.ghidat = None,
            Comanda::GhidatAlege(din_sursa) => {
                if let Some(g) = self.ghidat.as_mut() {
                    g.din_sursa = din_sursa;
                }
            }
            Comanda::GhidatAplica => {
                let Some(g) = self.ghidat.clone() else {
                    return false;
                };
                if g.terminat {
                    return false;
                }
                if g.din_sursa {
                    // diferenta aplicata dispare: urmatoarea ii ia locul, deci
                    // indexul ramane pe loc
                    let schimbat = self.aplica(self.diferenta);
                    if let Some(g) = self.ghidat.as_mut() {
                        g.aplicate += 1;
                        g.din_sursa = true;
                    }
                    self.verifica_ghidat();
                    return schimbat;
                }
                // „din tinta": blocul ramane cum e, ca o diferenta sarita
                self.sari_ghidat();
            }
            Comanda::GhidatSari => self.sari_ghidat(),
            // tratata in `comanda`, inainte
            Comanda::ComutaNod(_) | Comanda::CautaInArbore(_) => {}
        }
        false
    }

    /// Cate diferente are obiectul curent, asa cum le arata editorul.
    fn numar_diferente(&self) -> usize {
        self.model_editor().map_or(0, |m| m.diferente.len())
    }

    fn sari_ghidat(&mut self) {
        if let Some(g) = self.ghidat.as_mut() {
            if g.terminat {
                return;
            }
            g.sarite += 1;
            g.din_sursa = true;
        }
        self.diferenta += 1;
        self.verifica_ghidat();
    }

    /// Modul ghidat se termina cand nu mai e nicio diferenta dupa cea curenta.
    fn verifica_ghidat(&mut self) {
        let n = self.numar_diferente();
        if let Some(g) = self.ghidat.as_mut() {
            if self.diferenta >= n {
                g.terminat = true;
                self.diferenta = n.saturating_sub(1);
            }
        }
    }

    /// Schimba textul final al obiectului curent, cu intrare in istoric.
    /// Intoarce `true` (de salvat) daca s-a schimbat ceva.
    fn schimba_textul(&mut self, f: impl FnOnce(&mut Editari, &Rand, &str, &str)) -> bool {
        let Some((baza, id)) = self.cheia_curenta() else {
            return false;
        };
        let (Some(r), Some((a, b))) = (self.rand(&baza, &id).cloned(), self.texte(&baza, &id))
        else {
            return false;
        };
        let (a, b) = (a.to_string(), b.to_string());
        let inainte = self.editari.text_final(&r).cloned();
        f(&mut self.editari, &r, &a, &b);
        let dupa = self.editari.text_final(&r).cloned();
        if dupa == inainte {
            return false;
        }
        self.istoric.push((r, inainte));
        // textul final editat pe obiectul din previzualizare il fixeaza: un
        // clic in lista nu are voie sa-l inlocuiasca (regula din etapa 16)
        if self.taburi.activ() == Activ::Previzualizare
            && self.taburi.previzualizare() == Some(&(baza.clone(), id.clone()))
        {
            let editate = self.chei_editate();
            let r = self.taburi.fixeaza_previzualizarea(|k| editate.contains(k));
            self.rezultat_fixare(r);
        }
        // dupa o schimbare, diferenta curenta nu poate trece de ultima
        let n = self.numar_diferente();
        self.diferenta = self.diferenta.min(n.saturating_sub(1));
        true
    }

    /// Previzualizarea la hover a diferentei `k`: cum ar arata textul final
    /// daca ai aplica-o, fara sa schimbe nimic. Trece prin aceeasi
    /// `aplica_blocuri` ca [`Comanda::AplicaDiferenta`], deci arata exact ce
    /// s-ar scrie.
    pub fn previzualizare(&self, k: usize) -> Option<Vec<LiniePrevizualizare>> {
        let grup = self.model_editor()?.grupuri.get(k).cloned()?;
        let (sursa, curent) = self.texte_curente()?;
        let rezultat = gmotion_core::aplica_blocuri(&sursa, &curent, self.uneste_cursorul, &grup);
        Some(previzualizare(&curent, &rezultat, 2, 12))
    }

    /// Aplica diferenta `k` (indexul din editor) a obiectului curent: tot
    /// grupul ei, daca e un grup de instructiuni consecutive.
    fn aplica(&mut self, k: usize) -> bool {
        let Some(grup) = self.model_editor().and_then(|m| m.grupuri.get(k).cloned()) else {
            return false;
        };
        let uneste = self.uneste_cursorul;
        self.schimba_textul(|e, r, a, b| {
            e.aplica_blocuri(r, a, b, uneste, &grup);
        })
    }

    /// Dupa un deploy scris: obiectele din plan sunt deployate. Starea se
    /// tine pe amprenta (`Revizuire`), nu pe nume, deci un obiect schimbat
    /// pe server devine singur „versiune noua". Intoarce `true` daca s-a
    /// schimbat ceva, deci sesiunea trebuie salvata.
    pub fn marcheaza_deployate(&mut self, plan: &modul::Plan) -> bool {
        let mut schimbat = false;
        for pas in &plan.pasi {
            let Some(r) = self
                .randuri
                .iter()
                .find(|r| r.baza == pas.baza && r.obiect == pas.obiect)
                .cloned()
            else {
                continue;
            };
            self.revizuire.marcheaza_deployat(&r);
            self.lista
                .pune_deploy(&pas.baza, &pas.obiect, Deploy::Deployat);
            schimbat = true;
        }
        schimbat
    }

    #[cfg(test)]
    fn randuri_vizibile_toate(&self) -> Vec<Rand> {
        self.randuri.clone()
    }

    /// Randurile compararii pentru obiectele vizibile acum, in ordinea
    /// afisata: exact ce vede utilizatorul, nu toata compararea.
    pub fn randuri_vizibile(&self) -> Vec<Rand> {
        (0..self.lista.vizibile().len())
            .filter_map(|p| self.lista.obiect(p))
            .filter_map(|o| self.rand(&o.baza, &o.id()))
            .cloned()
            .collect()
    }

    /// Exporta lista **filtrata** in CSV (acelasi fisier ca `gmotion export`).
    /// Intoarce cate randuri s-au scris.
    pub fn exporta_csv(&self, cale: &std::path::Path) -> Result<usize, String> {
        let randuri = self.randuri_vizibile();
        let stare: std::collections::BTreeMap<usize, modul::export::Stare> = randuri
            .iter()
            .map(|r| {
                (
                    r.id,
                    modul::export::Stare {
                        rezolvat: self.revizuire.e_revizuit(r),
                        omis: self.revizuire.e_omis(r),
                    },
                )
            })
            .collect();
        modul::scrie_csv(cale, &randuri, |id| {
            stare.get(&id).copied().unwrap_or_default()
        })
        .map_err(|e| e.to_string())
    }

    fn comuta_omis(&mut self) -> bool {
        let Some(p) = self.lista.selectat() else {
            return false;
        };
        let Some(o) = self.lista.obiect(p) else {
            return false;
        };
        let ((baza, id), era) = (o.cheie(), o.omis);
        let Some(rand) = self.rand(&baza, &id).cloned() else {
            return false;
        };
        if era {
            self.revizuire.scoate_omis(&rand.baza, &rand.obiect);
        } else {
            self.revizuire.omite(&rand);
        }
        self.lista.pune_revizuirea(p, false, !era);
        true
    }

    /// Planul de deploy din ce s-a pus „in plan", cu amprentele compararii.
    /// `referinta` e numele bazei sursa; `facut_la`, momentul. Bazele tinta
    /// sunt cele comparate: planul le acopera pe toate deodata.
    pub fn plan(&self, referinta: &str, facut_la: &str) -> modul::Plan {
        let alese = self.lista.in_plan();
        let comparare = modul::Comparare {
            referinta: referinta.to_string(),
            baze: self.tinte.iter().map(|(n, _)| n.clone()).collect(),
            randuri: self.randuri.clone(),
            numaratoare: modul::Numaratoare::default(),
        };
        modul::Plan::din_comparare(
            &comparare,
            &self.sursa,
            &self.tinte,
            |r| alese.iter().any(|(b, o)| b == &r.baza && o == &r.obiect),
            facut_la,
        )
    }

    /// Bazele comparate, in ordine.
    pub fn baze(&self) -> Vec<String> {
        self.tinte.iter().map(|(n, _)| n.clone()).collect()
    }

    /// Obiectele pregatite ale bazelor comparate: de aici se recitesc la garda.
    pub fn tinte(&self) -> &[(String, Baza)] {
        &self.tinte
    }

    /// Obiectele pregatite ale sursei: de aici se scrie codul in deploy.
    pub fn sursa(&self) -> &Baza {
        &self.sursa
    }

    /// Trece la obiectul urmator / anterior care se poate deschide, ramanand
    /// in editor.
    fn treci(&mut self, pas: i32) {
        let inceput = self.lista.selectat();
        for _ in 0..self.lista.vizibile().len() {
            self.lista.muta_selectia(pas);
            if self.lista.selectat() == inceput {
                break;
            }
            if self.ecran == Ecran::Lista || self.model_editor().is_some() {
                return;
            }
        }
        // nu exista alt obiect de deschis: ramane unde era
        if let Some(p) = inceput {
            self.lista.selecteaza(p);
        }
    }

    fn comuta_revizuit(&mut self) -> bool {
        let Some(p) = self.lista.selectat() else {
            return false;
        };
        let Some(o) = self.lista.obiect(p) else {
            return false;
        };
        let ((baza, id), era) = (o.cheie(), o.revizuit);
        let Some(rand) = self.rand(&baza, &id).cloned() else {
            return false;
        };
        if era {
            self.revizuire.scoate(&rand.baza, &rand.obiect);
        } else {
            self.revizuire.marcheaza(&rand);
        }
        // revizuit scoate omis (se exclud)
        self.lista.pune_revizuirea(p, !era, false);
        true
    }

    /// Obiectul are text final (punctul de pe tabul lui).
    pub fn are_text_final(&self, (baza, id): &Cheie) -> bool {
        self.rand(baza, id)
            .is_some_and(|r| self.editari.text_final(r).is_some())
    }

    /// Taburile editorului.
    pub fn taburi(&self) -> &Taburi {
        &self.taburi
    }

    /// Ce trebuie spus dupa ultima comanda pe taburi.
    pub fn mesaj_tab(&self) -> Option<MesajTab> {
        self.mesaj_tab
    }

    /// Filtrul, ordinea si selectia listei de dinainte (recomparare).
    pub fn cu_vederea(mut self, vechi: &Lista) -> Self {
        self.lista.preia_vederea(vechi);
        self
    }

    /// Taburile de dinainte (recomparare, sau din sesiune). Cele ale
    /// obiectelor care nu mai difera se inchid.
    pub fn cu_taburi(mut self, mut t: Taburi) -> Self {
        let exista: std::collections::BTreeSet<Cheie> =
            self.lista.obiecte().iter().map(|o| o.cheie()).collect();
        t.pastreaza(|c| exista.contains(c));
        self.taburi = t;
        self
    }

    /// Fixatele cu text final editat: pe ele nu le inchide al noualea tab.
    fn chei_editate(&self) -> Vec<Cheie> {
        self.taburi
            .fixate()
            .into_iter()
            .filter(|(b, id)| {
                self.rand(b, id)
                    .is_some_and(|r| self.editari.text_final(r).is_some())
            })
            .cloned()
            .collect()
    }

    fn rezultat_fixare(&mut self, r: Result<Option<Cheie>, Refuz>) {
        match r {
            Ok(_) => self.dupa_schimbarea_tabului(),
            Err(Refuz::ToateEditate) => self.mesaj_tab = Some(MesajTab::ToateEditate),
        }
    }

    /// Obiectul din tabul activ devine selectia listei; fara niciun tab,
    /// editorul n-are ce arata, deci se intoarce in lista.
    fn dupa_schimbarea_tabului(&mut self) {
        let Some(c) = self.taburi.cheie_activa().cloned() else {
            self.ecran = Ecran::Lista;
            return;
        };
        if self.cheia_curenta().as_ref() != Some(&c) {
            self.lista.selecteaza_cheie(&c);
            self.diferenta = 0;
            self.ghidat = None;
            self.istoric.clear();
        }
    }

    /// `j` / `k` in editor: urmatorul obiect nefixat care se poate deschide,
    /// in ordinea listei; la capat se ramane pe loc, cu mesaj.
    fn pas_tab(&mut self, pas: i32) {
        // in ordinea arborelui din coloana (Object Explorer), ca `j` sa mearga
        // la obiectul de dedesubt, nu la urmatorul din lista sortata
        let chei: Vec<Cheie> = self
            .arbore
            .noduri(&self.lista, gmotion_fereastra::texte::Limba::Romana)
            .iter()
            .filter_map(|n| n.pozitie)
            .filter_map(|p| self.lista.obiect(p).map(|o| o.cheie()))
            .filter(|(b, id)| self.texte(b, id).is_some())
            .collect();
        match self.taburi.urmatorul(&chei, pas) {
            Pas::Obiect(c) => {
                self.taburi.deschide(c);
                self.dupa_schimbarea_tabului();
            }
            Pas::Capat => self.mesaj_tab = Some(MesajTab::Capat),
        }
    }

    /// Ultimul „In plan" a fost refuzat: obiectul n-are cod de deploiat.
    pub fn refuzat_in_plan(&self) -> bool {
        self.refuzat_in_plan
    }

    /// Starea de revizuire, curatata de bifele care nu mai corespund, gata de
    /// salvat in sesiune.
    pub fn revizuire_de_salvat(&mut self) -> &Revizuire {
        self.revizuire.curata(&self.randuri, &self.baze());
        &self.revizuire
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use gmotion_db::Obiect as ObiectDb;

    fn obiect_db(nume: &str, definitie: &str) -> ObiectDb {
        ObiectDb {
            schema: "dbo".into(),
            nume: nume.into(),
            tip: "P".into(),
            definitie: definitie.into(),
            setari: None,
        }
    }

    /// Doua baze cu: un obiect modificat, unul modificat altfel, unul care
    /// exista doar in sursa.
    fn aplicatie() -> Aplicatie {
        let sursa = modul::pregateste(&[
            obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1"),
            obiect_db("pB", "CREATE PROC dbo.pB AS\nSELECT 2"),
            obiect_db("pDoarSursa", "CREATE PROC dbo.pDoarSursa AS\nSELECT 3"),
        ]);
        let tinta = modul::pregateste(&[
            obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 99"),
            obiect_db("pB", "CREATE PROC dbo.pB AS\nSELECT 98"),
        ]);
        let comparare = modul::compara(
            "Referinta",
            &sursa,
            &[("Client1".to_string(), tinta.clone())],
            |_| {},
        );
        Aplicatie::noua(
            comparare.randuri,
            sursa,
            vec![("Client1".to_string(), tinta)],
            Revizuire::noua(),
        )
    }

    /// Etapa 19: nodurile arborelui din editor se inchid / deschid prin
    /// comanda, iar obiectul selectat isi deschide singur dosarul.
    #[test]
    fn arborele_se_comuta_si_selectia_isi_deschide_calea() {
        use gmotion_fereastra::texte::Limba;
        let mut app = aplicatie();
        let toate = app.arbore().noduri(app.lista(), Limba::Romana).len();
        let dosar = app.arbore().noduri(app.lista(), Limba::Romana)[1]
            .cheie
            .clone();
        app.comanda(Comanda::ComutaNod(dosar.clone()));
        assert!(!app.arbore().deschis(&dosar));
        assert!(app.arbore().noduri(app.lista(), Limba::Romana).len() < toate);
        app.comanda(Comanda::Selecteaza(1));
        assert!(app.arbore().deschis(&dosar), "selectia a redeschis dosarul");
        assert_eq!(app.arbore().noduri(app.lista(), Limba::Romana).len(), toate);
    }

    /// Detaliile calculate pe alt fir ajung pe obiectul lor, ca si cele
    /// calculate pe loc.
    #[test]
    fn detaliile_de_pe_alt_fir_sunt_aceleasi_ca_pe_loc() {
        let mut pe_loc = aplicatie();
        pe_loc.asigura_detalii(0, 10);
        let mut app = aplicatie();
        let lucrari = app.lucrari_detalii();
        assert_eq!(lucrari.len(), 2, "pA si pB sunt modificate");
        let rezultate: Vec<_> = std::thread::spawn(move || {
            lucrari
                .iter()
                .map(|l| (l.cheie.clone(), l.calculeaza()))
                .collect()
        })
        .join()
        .expect("firul se termina");
        for (c, d) in rezultate {
            app.pune_detalii(&c, d);
        }
        assert!(app.lucrari_detalii().is_empty());
        let detalii = |a: &Aplicatie| -> Vec<Option<Detalii>> {
            a.lista()
                .obiecte()
                .iter()
                .map(|o| o.detalii.clone())
                .collect()
        };
        assert_eq!(detalii(&app), detalii(&pe_loc));
    }

    /// Proba pe Windows (2026-11-19): cautarea din coloana editorului
    /// schimba obiectul deschis la fiecare tasta, iar una fara rezultat il
    /// pierdea. Acum filtreaza doar arborele.
    #[test]
    fn cautarea_din_editor_nu_schimba_obiectul_deschis() {
        use gmotion_fereastra::texte::Limba;
        let mut a = aplicatie();
        a.comanda(Comanda::Selecteaza(0));
        a.comanda(Comanda::Deschide);
        let deschis = a.cheia_curenta();
        let vizibile = a.lista().vizibile().len();
        for t in ["pB", "zzz", "das\\", ""] {
            a.comanda(Comanda::CautaInArbore(t.into()));
            assert_eq!(a.cheia_curenta(), deschis, "cautand {t:?}");
            assert_eq!(a.taburi().cheie_activa(), deschis.as_ref());
            assert_eq!(
                a.lista().vizibile().len(),
                vizibile,
                "lista nu se filtreaza"
            );
        }
        a.comanda(Comanda::CautaInArbore("pB".into()));
        let obiecte: Vec<String> = a
            .arbore()
            .noduri(a.lista(), Limba::Romana)
            .iter()
            .filter(|n| n.pozitie.is_some())
            .map(|n| n.eticheta.clone())
            .collect();
        assert_eq!(obiecte, ["dbo.pB"]);
    }

    #[test]
    fn recompararea_pastreaza_filtrul_listei() {
        let mut veche = aplicatie();
        veche.comanda(Comanda::Cauta("pB".into()));
        assert_eq!(veche.lista().vizibile().len(), 1);
        let noua = aplicatie().cu_vederea(veche.lista());
        assert_eq!(noua.lista().vizibile().len(), 1, "filtrul ramane");
        assert_eq!(
            noua.cheia_curenta().map(|(_, id)| id),
            Some("dbo.pB".to_string())
        );
    }

    #[test]
    fn panoul_de_detalii_se_deschide_la_clic_si_se_inchide_la_al_doilea() {
        let mut app = aplicatie();
        assert!(
            !app.detalii_deschise(),
            "la pornire lista e pe toata latimea"
        );
        app.comanda(Comanda::ClicPeRand(1));
        assert!(app.detalii_deschise());
        assert_eq!(app.lista().selectat(), Some(1));
        // clic pe alt rand: panoul ramane, arata noul rand
        app.comanda(Comanda::ClicPeRand(0));
        assert!(app.detalii_deschise());
        assert_eq!(app.lista().selectat(), Some(0));
        // al doilea clic pe acelasi rand: se inchide, selectia ramane
        app.comanda(Comanda::ClicPeRand(0));
        assert!(!app.detalii_deschise());
        assert_eq!(app.lista().selectat(), Some(0));
        app.comanda(Comanda::ClicPeRand(0));
        app.comanda(Comanda::InchideDetalii);
        assert!(!app.detalii_deschise());
        // j / k nu deschid panoul
        app.comanda(Comanda::MutaSelectia(1));
        assert!(!app.detalii_deschise());
        // un rand care nu exista nu deschide nimic
        app.comanda(Comanda::ClicPeRand(99));
        assert!(!app.detalii_deschise());
    }

    #[test]
    fn deschide_duce_in_editor_si_intoarcerea_pastreaza_obiectul() {
        let mut app = aplicatie();
        app.comanda(Comanda::Selecteaza(1));
        let c = app.cheia_curenta();
        app.comanda(Comanda::Deschide);
        assert_eq!(app.ecran(), Ecran::Editor);
        app.comanda(Comanda::InapoiLaLista);
        assert_eq!(app.ecran(), Ecran::Lista);
        assert_eq!(app.cheia_curenta(), c);
    }

    #[test]
    fn nota_se_pune_pe_obiectul_dat_nu_pe_selectie() {
        let mut app = aplicatie();
        app.comanda(Comanda::Selecteaza(0));
        let (ba, a) = app.obiect().unwrap().cheie();
        app.comanda(Comanda::Selecteaza(1));
        let b = app.obiect().unwrap().id();
        let nota = |t: &str| Comanda::PuneNota(ba.clone(), a.clone(), t.into());
        assert!(app.comanda(nota("de verificat")));
        assert_eq!(app.nota_obiect(), "", "selectia e pe {b}, nota pe {a}");
        app.comanda(Comanda::Selecteaza(0));
        assert_eq!(app.nota_obiect(), "de verificat");
        assert!(!app.comanda(nota("de verificat")));
        assert!(!app.comanda(Comanda::PuneNota(
            ba.clone(),
            "dbo.nuExista".into(),
            "x".into()
        )));
        assert!(!app.comanda(Comanda::PuneNota("altaBaza".into(), a.clone(), "x".into())));
        assert!(app.comanda(nota("")));
        assert_eq!(app.nota_obiect(), "");
    }

    /// Etapa 22: fiecare rand din lista stie cine il foloseste si impactul,
    /// fara sa deschizi panoul; panoul arata **care** obiecte, in ambele
    /// sensuri; clic pe un nume il selecteaza; filtrul pe impact merge.
    #[test]
    fn dependentele_si_impactul_in_lista_si_in_panou() {
        let sursa = modul::pregateste(&[
            obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1"),
            obiect_db("pB", "CREATE PROC dbo.pB AS\nEXEC dbo.pA"),
            obiect_db("pC", "CREATE PROC dbo.pC AS\nEXEC dbo.pA\nEXEC dbo.pB"),
        ]);
        let tinta = modul::pregateste(&[
            obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 2"),
            obiect_db("pB", "CREATE PROC dbo.pB AS\nEXEC dbo.pA"),
            obiect_db("pC", "CREATE PROC dbo.pC AS\nEXEC dbo.pA"),
            obiect_db("pX", "CREATE PROC dbo.pX AS\nEXEC dbo.pB"),
        ]);
        let c = modul::compara("Ref", &sursa, &[("Client1".into(), tinta.clone())], |_| {});
        let mut app = Aplicatie::noua(
            c.randuri,
            sursa,
            vec![("Client1".into(), tinta)],
            Revizuire::noua(),
        );
        let rand = |app: &Aplicatie, id: &str| {
            app.lista()
                .obiecte()
                .iter()
                .find(|o| o.id() == id)
                .cloned()
                .unwrap_or_else(|| panic!("{id} lipseste din lista"))
        };
        // in Modificata, pA e folosit de pB, pC; pB de pX
        assert_eq!(rand(&app, "dbo.pA").folosit_de, Some(2));
        assert_eq!(
            rand(&app, "dbo.pB").folosit_de,
            Some(1),
            "identic, dar folosit"
        );
        assert_eq!(
            rand(&app, "dbo.pB").impact,
            None,
            "identicul nu se deployeaza"
        );
        assert_eq!(rand(&app, "dbo.pX").stare, Stare::DoarBaza);
        assert_eq!(rand(&app, "dbo.pX").impact, Some(Impact::Scazut), "Extra");
        assert!(rand(&app, "dbo.pA").impact.is_some());

        // panoul: pC din Martor foloseste pA si pB (codul care s-ar deploia)
        app.comanda(Comanda::Filtreaza(None));
        let p_c = app
            .lista()
            .vizibile()
            .iter()
            .position(|&i| app.lista().obiecte()[i].id() == "dbo.pC")
            .expect("pC vizibil");
        app.comanda(Comanda::ClicPeRand(p_c));
        let p = app.panou_detalii();
        assert_eq!(p.foloseste, ["dbo.pA", "dbo.pB"]);
        assert!(p.folosit_de.is_empty());
        // Extra: ce foloseste vine din Modificata, singurul loc unde exista
        app.comanda(Comanda::SelecteazaDependenta("dbo.pX".into()));
        assert_eq!(app.obiect().map(|o| o.id()), Some("dbo.pX".into()));
        assert_eq!(app.panou_detalii().foloseste, ["dbo.pB"]);
        // clic pe „Foloseste: dbo.pB" il selecteaza, chiar daca filtrul il ascundea
        app.comanda(Comanda::Filtreaza(Some(Stare::DoarBaza)));
        app.comanda(Comanda::SelecteazaDependenta("dbo.pB".into()));
        assert_eq!(app.obiect().map(|o| o.id()), Some("dbo.pB".into()));
        assert_eq!(app.panou_detalii().folosit_de, ["dbo.pX"]);
        // un nume care nu e in lista lasa selectia pe loc
        app.comanda(Comanda::SelecteazaDependenta("dbo.nuExista".into()));
        assert_eq!(app.obiect().map(|o| o.id()), Some("dbo.pB".into()));

        // filtrul pe impact
        app.comanda(Comanda::Filtreaza(None));
        app.comanda(Comanda::FiltreazaImpact(Some(Impact::Scazut)));
        assert!(app
            .lista()
            .vizibile()
            .iter()
            .all(|&i| app.lista().obiecte()[i].impact == Some(Impact::Scazut)));
        assert!(!app.lista().vizibile().is_empty());
        app.comanda(Comanda::FiltreazaImpact(None));
    }

    #[test]
    fn impactul_numara_cine_foloseste_obiectul_in_baza_comparata() {
        let sursa = modul::pregateste(&[
            obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1"),
            obiect_db("pB", "CREATE PROC dbo.pB AS\nEXEC dbo.pA"),
        ]);
        let tinta = modul::pregateste(&[
            obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 2"),
            obiect_db("pB", "CREATE PROC dbo.pB AS\nEXEC dbo.pA\nSELECT 5"),
        ]);
        let c = modul::compara("Ref", &sursa, &[("Client1".into(), tinta.clone())], |_| {});
        let mut app = Aplicatie::noua(
            c.randuri,
            sursa,
            vec![("Client1".into(), tinta)],
            Revizuire::noua(),
        );
        let mut vazute = Vec::new();
        for p in 0..2 {
            app.comanda(Comanda::Selecteaza(p));
            let (i, cine) = app.impact_obiect().unwrap();
            vazute.push((app.obiect().unwrap().id(), i.folosit_de, cine));
        }
        vazute.sort();
        // panoul inchis nu calculeaza impactul; deschis, il arata
        assert_eq!(app.panou_detalii().impact, None);
        app.comanda(Comanda::ClicPeRand(0));
        let p = app.panou_detalii();
        assert!(
            p.deschis && p.impact.is_some() && !p.motive.is_empty(),
            "{p:?}"
        );
        assert_eq!(
            vazute,
            vec![
                ("dbo.pA".to_string(), 1, vec!["dbo.pB".to_string()]),
                ("dbo.pB".to_string(), 0, vec![]),
            ]
        );
    }

    #[test]
    fn planul_si_deploy_ul_urmeaza_ce_e_in_plan() {
        let mut app = aplicatie();
        // planul gol: deploy-ul nu se deschide
        app.comanda(Comanda::DeschideDeploy);
        assert_eq!(app.ecran(), Ecran::Lista);
        app.comanda(Comanda::Selecteaza(0));
        app.comanda(Comanda::ComutaInPlan);
        let (baza, id) = app.obiect().unwrap().cheie();
        assert_eq!(app.lista().in_plan(), [(baza.clone(), id.clone())]);
        app.comanda(Comanda::DeschideDeploy);
        assert_eq!(app.ecran(), Ecran::Deploy);
        let plan = app.plan("Ref", "acum");
        assert_eq!(plan.pasi.len(), 1);
        assert_eq!(plan.pasi[0].obiect, id);
        assert_eq!(plan.pasi[0].baza, baza);
        // scos ultimul obiect: inapoi in lista, planul e gol
        app.comanda(Comanda::ScoateDinPlan(baza, id));
        assert_eq!(app.ecran(), Ecran::Lista);
        assert!(app.lista().in_plan().is_empty());
    }

    #[test]
    fn filtrele_noi_ajung_la_lista_prin_comenzi() {
        let mut a = aplicatie();
        a.comanda(Comanda::FiltreazaRevizuire(Some(
            FiltruRevizuire::Nerevizuite,
        )));
        assert_eq!(a.lista().vizibile().len(), 3);
        a.comanda(Comanda::Selecteaza(0));
        a.comanda(Comanda::ComutaRevizuit);
        assert_eq!(
            a.lista().vizibile().len(),
            3,
            "ca in v3.4: bifa nu smulge randul de sub cursor; filtrul se \
             re-aplica la urmatoarea atingere"
        );
        a.comanda(Comanda::FiltreazaRevizuire(Some(
            FiltruRevizuire::Nerevizuite,
        )));
        assert_eq!(a.lista().vizibile().len(), 2, "acum iese");
        a.comanda(Comanda::FiltreazaRevizuire(Some(
            FiltruRevizuire::Revizuite,
        )));
        assert_eq!(a.lista().vizibile().len(), 1);
        a.comanda(Comanda::FiltreazaRevizuire(None));
        assert_eq!(a.lista().vizibile().len(), 3);
        // pDoarSursa n-are scor: iese cand filtrul de scor e activ
        a.comanda(Comanda::FiltreazaScor(0.0, 99.0));
        assert_eq!(a.lista().vizibile().len(), 2);
        a.comanda(Comanda::FiltreazaScor(0.0, 100.0));
        assert_eq!(a.lista().vizibile().len(), 3);
    }

    #[test]
    fn in_plan_pe_tot_ce_se_vede_respecta_filtrul() {
        let mut a = aplicatie();
        a.comanda(Comanda::Cauta("pA".into()));
        a.comanda(Comanda::PuneVizibileleInPlan(false));
        assert_eq!(
            a.lista().in_plan(),
            [("Client1".to_string(), "dbo.pA".to_string())]
        );
        a.comanda(Comanda::Cauta(String::new()));
        a.comanda(Comanda::PuneVizibileleInPlan(false));
        let plan: Vec<String> = a.lista().in_plan().into_iter().map(|(_, o)| o).collect();
        assert_eq!(
            plan,
            ["dbo.pA", "dbo.pB", "dbo.pDoarSursa"],
            "toate se pot deploia"
        );
    }

    #[test]
    fn dupa_deploy_obiectele_din_plan_sunt_deployate_si_tin_minte() {
        let mut a = aplicatie();
        a.comanda(Comanda::PuneVizibileleInPlan(false));
        let plan = a.plan("Referinta", "acum");
        assert_eq!(plan.pasi.len(), 3);
        assert!(a.marcheaza_deployate(&plan), "e de salvat in sesiune");
        assert_eq!(a.lista().progres_deploy(), (3, 3));
        a.comanda(Comanda::FiltreazaRevizuire(Some(
            FiltruRevizuire::Nedeployate,
        )));
        assert!(a.lista().vizibile().is_empty());

        // starea e pe amprenta: aceeasi comparare o regaseste, una schimbata nu
        let randuri: Vec<Rand> = a.randuri_vizibile_toate();
        let rev = a.revizuire().clone();
        for r in &randuri {
            assert!(rev.e_deployat(r));
            let mut schimbat = r.clone();
            schimbat.hash_curent = Some("altceva".into());
            assert!(!rev.e_deployat(&schimbat), "versiune noua, nu deployat");
            assert!(rev.a_fost_deployat(&schimbat));
        }
    }

    #[test]
    fn exportul_csv_scrie_doar_randurile_vizibile() {
        let mut a = aplicatie();
        a.comanda(Comanda::Cauta("pA".into()));
        let dir = std::env::temp_dir().join(format!("gmotion-export-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cale = dir.join("lista.csv");
        assert_eq!(a.exporta_csv(&cale).unwrap(), 1);
        let text = std::fs::read_to_string(&cale).unwrap();
        assert!(text.contains("dbo.pA") && !text.contains("dbo.pB"));
        // starea utilizatorului intra in fisier
        a.comanda(Comanda::Selecteaza(0));
        a.comanda(Comanda::ComutaOmis);
        a.exporta_csv(&cale).unwrap();
        let text = std::fs::read_to_string(&cale).unwrap();
        assert!(text.trim_end().ends_with("NU;DA"), "coloana Omis: {text}");
        // o lista filtrata la zero nu scrie un fisier gol, da eroare
        a.comanda(Comanda::Cauta("nu-exista".into()));
        assert!(a.exporta_csv(&cale).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Aceeasi referinta, doua baze: obiectul apare o data pentru fiecare,
    /// fiecare rand isi vede textul din baza lui, si planul le acopera pe
    /// amandoua.
    #[test]
    fn doua_baze_dau_doua_randuri_cu_texte_diferite() {
        let sursa = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1")]);
        let c1 = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 11")]);
        let c2 = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 22")]);
        let tinte = vec![("C1".to_string(), c1), ("C2".to_string(), c2)];
        let comparare = modul::compara("Ref", &sursa, &tinte, |_| {});
        let mut a = Aplicatie::noua(comparare.randuri, sursa, tinte, Revizuire::noua());

        assert_eq!(a.lista().vizibile().len(), 2);
        assert_eq!(a.baze(), ["C1", "C2"]);

        // fiecare rand arata textul bazei lui, nu al primei
        for (p, asteptat) in [(0, "SELECT 11"), (1, "SELECT 22")] {
            a.comanda(Comanda::Selecteaza(p));
            let (_, baza) = a.texte_curente().expect("texte");
            assert!(
                baza.contains(asteptat),
                "randul {p} ar trebui sa arate {asteptat}, arata:\n{baza}"
            );
        }

        // planul le ia pe amandoua, fiecare cu baza ei
        a.comanda(Comanda::PuneVizibileleInPlan(false));
        let plan = a.plan("Ref", "acum");
        let baze: Vec<&str> = plan.pasi.iter().map(|p| p.baza.as_str()).collect();
        assert_eq!(baze, ["C1", "C2"]);

        // filtrul pe baza taie una din ele
        a.comanda(Comanda::FiltreazaBaza(Some("C2".into())));
        assert_eq!(a.lista().vizibile().len(), 1);
        assert_eq!(a.obiect().unwrap().baza, "C2");

        // „scoate din plan" scoate doar randul bazei date
        a.comanda(Comanda::ScoateDinPlan("C1".into(), "dbo.pA".into()));
        assert_eq!(
            a.lista().in_plan(),
            [("C2".to_string(), "dbo.pA".to_string())]
        );
    }

    #[test]
    fn bifa_pusa_pe_o_baza_nu_se_intinde_la_cealalta() {
        let sursa = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1")]);
        let c1 = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 11")]);
        let c2 = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 22")]);
        let tinte = vec![("C1".to_string(), c1), ("C2".to_string(), c2)];
        let comparare = modul::compara("Ref", &sursa, &tinte, |_| {});
        let mut a = Aplicatie::noua(comparare.randuri, sursa, tinte, Revizuire::noua());
        a.comanda(Comanda::Selecteaza(0));
        a.comanda(Comanda::ComutaRevizuit);
        assert!(a.lista().obiect(0).unwrap().revizuit);
        assert!(
            !a.lista().obiect(1).unwrap().revizuit,
            "acelasi obiect in alta baza e alta treaba"
        );
        assert_eq!(a.lista().progres_revizuire(), (1, 2));
    }

    /// „Uneste ciclul cursor": blocurile se taie altfel, deci editorul arata
    /// mai putine diferente — una singura pe tot ciclul, in loc de cate una
    /// pe fiecare instructiune.
    #[test]
    fn unirea_ciclului_de_cursor_strange_blocurile_intr_unul() {
        const CICLU: &str = "CREATE PROCEDURE dbo.pC AS\n\
            DECLARE c CURSOR FOR SELECT id FROM dbo.T;\n\
            OPEN c;\n\
            FETCH NEXT FROM c INTO @id;\n\
            WHILE @@FETCH_STATUS = 0\n\
            BEGIN\n\
                FETCH NEXT FROM c INTO @id;\n\
            END\n\
            CLOSE c;\n\
            DEALLOCATE c;";
        let sursa = modul::pregateste(&[obiect_db("pC", CICLU)]);
        let tinta = modul::pregateste(&[obiect_db(
            "pC",
            &CICLU.replace(
                "SELECT id FROM dbo.T",
                "SELECT id FROM dbo.T WHERE activ = 1",
            ),
        )]);
        let tinte = vec![("Client1".to_string(), tinta)];
        let c = modul::compara("Ref", &sursa, &tinte, |_| {});
        let mut a = Aplicatie::noua(c.randuri, sursa, tinte, Revizuire::noua());
        a.comanda(Comanda::Selecteaza(0));

        assert!(a.are_cursor(), "obiectul are cursor, deci butonul are rost");
        assert!(!a.uneste_cursorul());
        let separat = a.model_editor().expect("model").diferente.len();

        a.comanda(Comanda::ComutaUnireaCursorului(true));
        assert!(a.uneste_cursorul());
        let unit = a.model_editor().expect("model").diferente.len();
        assert_eq!(unit, 1, "tot ciclul devine o singura diferenta");
        assert!(
            separat >= unit,
            "unirea nu poate da mai multe blocuri: {separat} -> {unit}"
        );

        a.comanda(Comanda::ComutaUnireaCursorului(false));
        assert_eq!(a.model_editor().expect("model").diferente.len(), separat);
    }

    #[test]
    fn butonul_de_cursor_nu_apare_pe_obiectele_fara_cursor() {
        let mut a = aplicatie();
        a.comanda(Comanda::Selecteaza(0));
        assert!(!a.are_cursor(), "pA n-are cursor");
    }

    #[test]
    fn deschiderea_duce_in_editor_si_esc_inapoi() {
        let mut a = aplicatie();
        assert_eq!(a.ecran(), Ecran::Lista);
        a.comanda(Comanda::Deschide);
        assert_eq!(a.ecran(), Ecran::Editor);
        assert_eq!(a.obiect().unwrap().nume, "pA");
        a.comanda(Comanda::InapoiLaLista);
        assert_eq!(a.ecran(), Ecran::Lista);
        assert_eq!(a.obiect().unwrap().nume, "pA", "selectia a ramas unde era");
    }

    #[test]
    fn din_editor_se_trece_la_obiectul_urmator_fara_sa_treci_prin_lista() {
        let mut a = aplicatie();
        a.comanda(Comanda::Deschide);
        a.comanda(Comanda::ObiectUrmator);
        assert_eq!(a.ecran(), Ecran::Editor, "ramanem in editor");
        assert_eq!(a.obiect().unwrap().nume, "pB");
        a.comanda(Comanda::ObiectAnterior);
        assert_eq!(a.obiect().unwrap().nume, "pA");
    }

    #[test]
    fn un_obiect_doar_intr_o_parte_nu_deschide_editorul() {
        let mut a = aplicatie();
        let p = (0..a.lista().vizibile().len())
            .find(|&p| a.lista().obiect(p).unwrap().nume == "pDoarSursa")
            .expect("obiectul exista in lista");
        a.comanda(Comanda::Selecteaza(p));
        assert!(a.model_editor().is_none());
        a.comanda(Comanda::Deschide);
        assert_eq!(a.ecran(), Ecran::Lista, "nu avem cu ce compara");
    }

    #[test]
    fn urmatorul_obiect_sare_peste_cele_care_nu_se_pot_deschide() {
        let mut a = aplicatie();
        // pB e ultimul care se poate deschide; dupa el, pDoarSursa
        let p = (0..a.lista().vizibile().len())
            .find(|&p| a.lista().obiect(p).unwrap().nume == "pB")
            .unwrap();
        a.comanda(Comanda::Selecteaza(p));
        a.comanda(Comanda::Deschide);
        assert_eq!(a.ecran(), Ecran::Editor);
        a.comanda(Comanda::ObiectUrmator);
        assert_eq!(
            a.obiect().unwrap().nume,
            "pB",
            "nu exista alt obiect de deschis, deci ramanem pe el"
        );
        assert_eq!(a.ecran(), Ecran::Editor);
    }

    #[test]
    fn bifa_pusa_in_editor_se_vede_in_lista_si_in_starea_salvata() {
        let mut a = aplicatie();
        a.comanda(Comanda::Deschide);
        assert!(a.comanda(Comanda::ComutaRevizuit), "s-a schimbat ceva");
        assert!(a.obiect().unwrap().revizuit);
        assert_eq!(a.lista().progres_revizuire(), (1, 3));
        assert_eq!(a.revizuire().numar(), 1);

        // si inapoi: scoaterea bifei
        assert!(a.comanda(Comanda::ComutaRevizuit));
        assert!(!a.obiect().unwrap().revizuit);
        assert_eq!(a.revizuire().numar(), 0);
    }

    #[test]
    fn bifele_vechi_se_reiau_la_pornire_si_cad_daca_obiectul_s_a_schimbat() {
        let mut a = aplicatie();
        a.comanda(Comanda::ComutaRevizuit);
        let salvata = a.revizuire_de_salvat().clone();

        // aceeasi comparare: bifa se reia
        let sursa = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1")]);
        let tinta = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 99")]);
        let c = modul::compara(
            "Referinta",
            &sursa,
            &[("Client1".to_string(), tinta.clone())],
            |_| {},
        );
        let b = Aplicatie::noua(
            c.randuri,
            sursa,
            vec![("Client1".to_string(), tinta)],
            salvata.clone(),
        );
        assert!(b.obiect().unwrap().revizuit);

        // obiectul s-a schimbat pe server: bifa nu se mai reia
        let sursa = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1")]);
        let tinta = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 77")]);
        let c = modul::compara(
            "Referinta",
            &sursa,
            &[("Client1".to_string(), tinta.clone())],
            |_| {},
        );
        let b = Aplicatie::noua(
            c.randuri,
            sursa,
            vec![("Client1".to_string(), tinta)],
            salvata,
        );
        assert!(!b.obiect().unwrap().revizuit);
    }

    #[test]
    fn detaliile_se_calculeaza_doar_pentru_randurile_cerute() {
        let mut a = aplicatie();
        assert!(a.lista().obiect(0).unwrap().detalii.is_none());
        a.asigura_detalii(0, 1);
        assert_eq!(
            a.lista()
                .obiect(0)
                .unwrap()
                .detalii
                .as_ref()
                .unwrap()
                .difera_in,
            "cod"
        );
        assert!(
            a.lista().obiect(1).unwrap().detalii.is_none(),
            "randul necerut ramane necalculat"
        );
    }

    #[test]
    fn filtrul_si_cautarea_nu_pierd_obiectul_deschis() {
        let mut a = aplicatie();
        a.comanda(Comanda::Cauta("pB".into()));
        a.comanda(Comanda::Deschide);
        assert_eq!(a.obiect().unwrap().nume, "pB");
        a.comanda(Comanda::Cauta(String::new()));
        assert_eq!(
            a.obiect().unwrap().nume,
            "pB",
            "obiectul deschis ramane selectat cand cautarea se goleste"
        );
    }

    /// Un obiect cu trei diferente, pentru editare si modul ghidat.
    fn aplicatie_editare() -> (Aplicatie, &'static str, &'static str) {
        const SURSA: &str =
            "CREATE PROC dbo.pE AS\nSELECT 1\n\nUPDATE t SET a = 1\n\nDELETE x WHERE id = 1";
        const BAZA: &str =
            "CREATE PROC dbo.pE AS\nSELECT 2\n\nUPDATE t SET a = 2\n\nDELETE x WHERE id = 2";
        let sursa = modul::pregateste(&[obiect_db("pE", SURSA)]);
        let tinta = modul::pregateste(&[obiect_db("pE", BAZA)]);
        let c = modul::compara(
            "Ref",
            &sursa,
            &[("Client1".to_string(), tinta.clone())],
            |_| {},
        );
        let mut a = Aplicatie::noua(
            c.randuri,
            sursa,
            vec![("Client1".to_string(), tinta)],
            Revizuire::noua(),
        );
        a.comanda(Comanda::Deschide);
        (a, SURSA, BAZA)
    }

    /// Un obiect in care UPDATE-ul a urcat in tinta: blocul din sursa iese
    /// fara pereche, iar plasarea il trimite peste UPDATE-ul din tinta.
    fn aplicatie_mutat() -> (Aplicatie, &'static str) {
        const SURSA: &str = "CREATE PROC dbo.pM AS\n\
            SELECT a FROM dbo.T1;\n\
            SELECT b FROM dbo.T2;\n\
            UPDATE dbo.T3 SET x = 1, y = 2, z = 4 WHERE id = 5 AND activ = 1;";
        const BAZA: &str = "CREATE PROC dbo.pM AS\n\
            UPDATE dbo.T3 SET x = 1, y = 3, z = 4 WHERE id = 5 AND activ = 1;\n\
            SELECT a FROM dbo.T1;\n\
            SELECT b FROM dbo.T2;";
        let sursa = modul::pregateste(&[obiect_db("pM", SURSA)]);
        let tinta = modul::pregateste(&[obiect_db("pM", BAZA)]);
        let c = modul::compara(
            "Ref",
            &sursa,
            &[("Client1".to_string(), tinta.clone())],
            |_| {},
        );
        let mut a = Aplicatie::noua(
            c.randuri,
            sursa,
            vec![("Client1".to_string(), tinta)],
            Revizuire::noua(),
        );
        a.comanda(Comanda::Deschide);
        (a, BAZA)
    }

    /// Diferenta blocului fara pereche, si randul din model pe care il
    /// aprinde ecranul cand e tras (`ModelEditor::tinte`).
    fn diferenta_mutata(a: &Aplicatie) -> (usize, String) {
        let m = a.model_editor().expect("model");
        let k = (0..m.diferente.len())
            .find(|&k| {
                m.randuri[m.diferente[k]].stare == gmotion_fereastra::model_editor::Stare::Sters
            })
            .expect("un bloc doar in sursa");
        let tinta = m.tinte[k].expect("o tinta aprinsa");
        let aprins: String = m.randuri[tinta]
            .dreapta
            .bucati
            .iter()
            .map(|b| b.text.as_str())
            .collect();
        (k, aprins)
    }

    /// Drag & drop: blocul tras ajunge **exact** in locul aprins in timpul
    /// tragerii (acolo unde inlocuieste), nu inca o data la pozitia lui din
    /// sursa. Lasarea cheama `aplica-diferenta`, deci asta e comanda.
    #[test]
    fn blocul_lasat_ajunge_in_locul_aprins_si_ctrl_z_il_scoate() {
        let (mut a, baza) = aplicatie_mutat();
        let (k, aprins) = diferenta_mutata(&a);
        assert!(
            aprins.contains("y = 3"),
            "se aprinde UPDATE-ul din tinta: {aprins}"
        );

        assert!(a.comanda(Comanda::AplicaDiferenta(Some(k))));
        let (_, curent) = a.texte_curente().expect("texte");
        assert_eq!(curent.matches("UPDATE").count(), 1, "dublat:\n{curent}");
        assert!(
            curent.contains("y = 2") && !curent.contains("y = 3"),
            "{curent}"
        );
        assert!(
            curent.lines().nth(1).is_some_and(|l| l.contains("UPDATE")),
            "pe locul aprins:\n{curent}"
        );

        assert!(a.comanda(Comanda::Anuleaza), "Ctrl+Z dupa lasare");
        assert_eq!(a.texte_curente().expect("texte").1, baza);
    }

    /// Instructiunile consecutive doar in Martor, care merg in acelasi loc,
    /// sunt o singura diferenta: un „Aplica" le aduce pe toate, in ordine.
    #[test]
    fn un_grup_de_instructiuni_se_aplica_dintr_o_data() {
        const SURSA: &str = "CREATE PROC dbo.pG AS\n\
            SELECT a FROM dbo.T;\n\
            DECLARE @x INT;\n\
            SET @x = 1;\n\
            SELECT @x;\n\
            PRINT 'gata';";
        const BAZA: &str = "CREATE PROC dbo.pG AS\n\
            SELECT a FROM dbo.T;\n\
            PRINT 'gata';";
        let sursa = modul::pregateste(&[obiect_db("pG", SURSA)]);
        let tinta = modul::pregateste(&[obiect_db("pG", BAZA)]);
        let c = modul::compara("Ref", &sursa, &[("Client1".into(), tinta.clone())], |_| {});
        let mut a = Aplicatie::noua(
            c.randuri,
            sursa,
            vec![("Client1".into(), tinta)],
            Revizuire::noua(),
        );
        a.comanda(Comanda::Deschide);
        let m = a.model_editor().expect("model");
        assert_eq!(m.diferente.len(), 1, "trei instructiuni, o diferenta");
        assert_eq!(m.grupuri[0].len(), 3);
        drop(m);
        assert!(a.comanda(Comanda::AplicaDiferenta(Some(0))));
        let (_, curent) = a.texte_curente().expect("texte");
        let pozitie = |t: &str| {
            curent
                .find(t)
                .unwrap_or_else(|| panic!("{t} lipseste:\n{curent}"))
        };
        assert!(
            pozitie("DECLARE @x") < pozitie("SET @x = 1")
                && pozitie("SET @x = 1") < pozitie("SELECT @x")
        );
        assert!(pozitie("SELECT @x") < pozitie("PRINT 'gata'"));
        assert_eq!(a.model_editor().expect("model").diferente.len(), 0);
    }

    /// Previzualizarea la hover arata exact liniile pe care le-ar scrie
    /// „Aplica", si nu schimba nimic.
    #[test]
    fn previzualizarea_arata_ce_scrie_aplica_si_nu_schimba_nimic() {
        let (mut a, baza) = aplicatie_mutat();
        let (k, _) = diferenta_mutata(&a);
        let p = a.previzualizare(k).expect("previzualizare");
        assert_eq!(a.texte_curente().expect("texte").1, baza, "n-a scris nimic");
        assert!(!a.se_poate_anula(), "nici in istoric");
        let aduse: Vec<&str> = p
            .iter()
            .filter(|l| l.adusa)
            .map(|l| l.text.as_str())
            .collect();
        assert!(
            aduse.iter().any(|l| l.contains("y = 2")),
            "se vede blocul adus: {p:?}"
        );
        a.comanda(Comanda::AplicaDiferenta(Some(k)));
        let (_, dupa) = a.texte_curente().expect("texte");
        let linii: Vec<&str> = dupa.split('\n').collect();
        for l in p.iter().filter(|l| l.nr.is_some()) {
            let nr = l.nr.unwrap_or_default();
            assert_eq!(
                linii.get(nr - 1).copied(),
                Some(l.text.as_str()),
                "linia {nr}"
            );
        }
        assert!(a.previzualizare(99).is_none(), "diferenta inexistenta");
    }

    /// Tragerea libera: blocul lasat la sfarsitul Modificatei se insereaza
    /// acolo; lasat peste alt bloc il inlocuieste; lasat pe locul sugerat e
    /// ca „Aplica". Ctrl+Z scoate fiecare.
    #[test]
    fn blocul_tras_se_lasa_oriunde_si_ctrl_z_il_scoate() {
        let (mut a, _sursa, baza) = aplicatie_editare();
        let m = a.model_editor().expect("model");
        let bloc = m.grupuri[0][0];
        // locul fiecarui rand vine din model: un rand cu linie intr-un bloc
        // al Modificatei il inlocuieste
        let peste = m
            .randuri
            .iter()
            .find(|r| {
                r.dreapta.nr.is_some() && matches!(r.loc, Some(gmotion_core::Loc::Inlocuieste(_)))
            })
            .and_then(|r| r.loc)
            .expect("un rand care inlocuieste");
        drop(m);
        let linii = baza.lines().count();

        // la sfarsit
        assert!(a.comanda(Comanda::Plaseaza {
            bloc,
            loc: Some(gmotion_core::Loc::Insereaza(linii)),
        }));
        let (sursa, curent) = a.texte_curente().expect("texte");
        assert!(
            curent.starts_with(baza),
            "Modificata ramane intreaga in fata"
        );
        assert!(curent.len() > baza.len(), "si blocul vine la urma");
        assert!(a.comanda(Comanda::Anuleaza));
        assert_eq!(a.texte_curente().expect("texte").1, baza);

        // peste un bloc: liniile lui dispar, vin cele ale blocului tras
        assert!(a.comanda(Comanda::Plaseaza {
            bloc,
            loc: Some(peste)
        }));
        let (_, curent) = a.texte_curente().expect("texte");
        assert_ne!(curent, baza);
        assert!(curent.lines().count() <= linii + sursa.lines().count());
        assert!(a.comanda(Comanda::Anuleaza));

        // pe locul sugerat: exact ce face „Aplica"
        assert!(a.comanda(Comanda::Plaseaza { bloc, loc: None }));
        let lasat = a.texte_curente().expect("texte").1;
        a.comanda(Comanda::Anuleaza);
        a.comanda(Comanda::AplicaDiferenta(Some(0)));
        assert_eq!(a.texte_curente().expect("texte").1, lasat);
    }

    /// Modul ghidat aplica prin aceeasi plasare: aceeasi tinta ca la tragere.
    #[test]
    fn modul_ghidat_aplica_pe_locul_dat_de_plasare() {
        let (mut a, _) = aplicatie_mutat();
        let (k, _) = diferenta_mutata(&a);
        a.comanda(Comanda::PornesteGhidat);
        for _ in 0..k {
            a.comanda(Comanda::GhidatSari);
        }
        assert!(a.comanda(Comanda::GhidatAplica));
        let (_, curent) = a.texte_curente().expect("texte");
        assert_eq!(curent.matches("UPDATE").count(), 1, "dublat:\n{curent}");
        assert!(curent.contains("y = 2"), "{curent}");
    }

    /// Cu „uneste ciclul cursor" pornit, editorul numara alte blocuri; aplicarea
    /// trebuie sa numere la fel, altfel scrie alt bloc decat cel apasat.
    #[test]
    fn cu_cursorul_unit_aplica_blocul_apasat() {
        const SURSA: &str = "CREATE PROC dbo.pC AS\n\
            DECLARE c CURSOR FOR SELECT id FROM dbo.T;\n\
            OPEN c;\n\
            FETCH NEXT FROM c INTO @id;\n\
            CLOSE c;\n\
            DEALLOCATE c;\n\
            SELECT 1;";
        let baza = SURSA
            .replace(
                "SELECT id FROM dbo.T",
                "SELECT id FROM dbo.T WHERE activ = 1",
            )
            .replace("SELECT 1;", "SELECT 2;");
        let sursa = modul::pregateste(&[obiect_db("pC", SURSA)]);
        let tinta = modul::pregateste(&[obiect_db("pC", &baza)]);
        let c = modul::compara(
            "Ref",
            &sursa,
            &[("Client1".to_string(), tinta.clone())],
            |_| {},
        );
        let mut a = Aplicatie::noua(
            c.randuri,
            sursa,
            vec![("Client1".to_string(), tinta)],
            Revizuire::noua(),
        );
        a.comanda(Comanda::Deschide);
        a.comanda(Comanda::ComutaUnireaCursorului(true));
        assert_eq!(a.model_editor().expect("model").diferente.len(), 2);
        // a doua diferenta e `SELECT 1`; fara aceeasi taiere s-ar aplica alt bloc
        assert!(a.comanda(Comanda::AplicaDiferenta(Some(1))));
        let (_, curent) = a.texte_curente().expect("texte");
        assert!(curent.contains("SELECT 1;"), "{curent}");
        assert!(
            curent.contains("WHERE activ = 1"),
            "ciclul ramane neatins:\n{curent}"
        );
    }

    #[test]
    fn aplica_o_diferenta_apoi_ctrl_z_apoi_reset() {
        let (mut a, sursa, baza) = aplicatie_editare();
        let n = a.model_editor().unwrap().diferente.len();
        assert_eq!(n, 3);
        assert!(a.comanda(Comanda::AplicaDiferenta(Some(1))), "de salvat");
        assert_eq!(a.model_editor().unwrap().diferente.len(), 2);
        let (_, curent) = a.texte_curente().unwrap();
        assert!(
            curent.contains("SET a = 1") && curent.contains("SELECT 2"),
            "{curent}"
        );
        assert_eq!(a.stare_text_final(), (true, false));
        // liniile aduse sunt marcate
        assert!(a
            .model_editor()
            .unwrap()
            .randuri
            .iter()
            .any(|r| r.marcaj == gmotion_fereastra::model_editor::Marcaj::Adus));
        // Ctrl+Z: inapoi la baza
        assert!(a.se_poate_anula());
        assert!(a.comanda(Comanda::Anuleaza));
        assert_eq!(a.texte_curente().unwrap().1, baza);
        assert!(!a.se_poate_anula());
        // Aplica toate, apoi Reset
        a.comanda(Comanda::AplicaToate);
        assert_eq!(a.texte_curente().unwrap().1, sursa);
        assert_eq!(a.model_editor().unwrap().diferente.len(), 0);
        a.comanda(Comanda::Reset);
        assert_eq!(a.texte_curente().unwrap().1, baza);
        assert_eq!(a.stare_text_final(), (false, false));
        // doua anulari: inapoi la „aplica toate", apoi la baza
        a.comanda(Comanda::Anuleaza);
        assert_eq!(a.texte_curente().unwrap().1, sursa);
    }

    #[test]
    fn textul_scris_de_mana_e_marcat_si_ajunge_in_deploy() {
        let (mut a, _, baza) = aplicatie_editare();
        let text = format!("{baza}\n-- scris de mine");
        let id = a.id_obiect().expect("un obiect deschis");
        assert!(a.comanda(Comanda::SalveazaText {
            id: id.clone(),
            text: text.clone()
        }));
        assert_eq!(a.stare_text_final(), (true, true));
        let t = a.texte_finale();
        assert_eq!(t[&("Client1".to_string(), "dbo.pE".to_string())], text);
        // acelasi text inca o data nu e o schimbare
        assert!(!a.comanda(Comanda::SalveazaText {
            id: id.clone(),
            text: text.clone()
        }));
        // F-A1: un text venit pentru alt obiect nu se scrie nicaieri
        assert!(!a.comanda(Comanda::SalveazaText {
            id: "Client1/dbo.altul".to_string(),
            text: format!("{text}\n-- pentru altul")
        }));
        assert_eq!(
            a.texte_finale()[&("Client1".to_string(), "dbo.pE".to_string())],
            text,
            "textul obiectului deschis a ramas al lui"
        );
    }

    #[test]
    fn omis_si_revizuit_se_exclud_si_omis_scoate_din_plan() {
        let mut a = aplicatie();
        a.comanda(Comanda::ComutaInPlan);
        assert!(a.obiect().unwrap().in_plan);
        assert!(a.comanda(Comanda::ComutaOmis));
        let o = a.obiect().unwrap();
        assert!(o.omis && !o.revizuit && !o.in_plan);
        assert!(a.revizuire().e_omis(&a.randuri[0]));
        a.comanda(Comanda::ComutaRevizuit);
        let o = a.obiect().unwrap();
        assert!(o.revizuit && !o.omis, "revizuit scoate omis");
        a.comanda(Comanda::ComutaOmis);
        a.comanda(Comanda::ComutaOmis);
        let o = a.obiect().unwrap();
        assert!(!o.omis && !o.revizuit);
    }

    #[test]
    fn modul_ghidat_ia_diferentele_pe_rand_pana_la_final() {
        let (mut a, _, _) = aplicatie_editare();
        a.comanda(Comanda::PornesteGhidat);
        assert_eq!(a.diferenta_curenta(), 0);
        assert!(a.ghidat().is_some_and(|g| g.din_sursa && !g.terminat));
        // prima: aplicata din sursa; diferenta dispare, urmatoarea vine in loc
        a.comanda(Comanda::GhidatAplica);
        assert_eq!(a.model_editor().unwrap().diferente.len(), 2);
        assert_eq!(a.diferenta_curenta(), 0);
        // a doua: „din tinta" = ramane cum e, se trece mai departe
        a.comanda(Comanda::GhidatAlege(false));
        a.comanda(Comanda::GhidatAplica);
        assert_eq!(a.diferenta_curenta(), 1);
        // a treia: sarita; nu mai e nimic, deci s-a terminat
        a.comanda(Comanda::GhidatSari);
        let g = a.ghidat().unwrap();
        assert!(g.terminat);
        assert_eq!((g.aplicate, g.sarite), (1, 2));
        assert_eq!(a.model_editor().unwrap().diferente.len(), 2);
        a.comanda(Comanda::OpresteGhidat);
        assert!(a.ghidat().is_none());
    }

    #[test]
    fn ghidat_cu_toate_aplicate_se_termina_fara_diferente() {
        let (mut a, sursa, _) = aplicatie_editare();
        a.comanda(Comanda::PornesteGhidat);
        for _ in 0..3 {
            a.comanda(Comanda::GhidatAplica);
        }
        assert!(a.ghidat().unwrap().terminat);
        assert_eq!(a.ghidat().unwrap().aplicate, 3);
        assert_eq!(a.model_editor().unwrap().diferente.len(), 0);
        let (_, curent) = a.texte_curente().unwrap();
        assert_eq!(
            gmotion_core::normalizeaza(&curent),
            gmotion_core::normalizeaza(sursa)
        );
    }

    /// V9 din `cercetare/audit-qa-2026-11-18.md`: bifa „Revizuit" de pe un
    /// rand bifa obiectul **selectat**, nu pe cel apasat — legatura din
    /// fereastra ignora indicele trimis de `lista.slint`.
    #[test]
    fn bifa_unui_rand_bifeaza_randul_ei_nu_pe_cel_selectat() {
        let mut a = aplicatie();
        a.comanda(Comanda::Selecteaza(0));
        let nume = |a: &Aplicatie, p: usize| a.lista().obiect(p).unwrap().nume.clone();
        let (primul, al_doilea) = (nume(&a, 0), nume(&a, 1));
        assert_ne!(primul, al_doilea);

        assert!(a.comanda(Comanda::ComutaRevizuitLa(1)));
        assert!(
            a.lista().obiect(1).unwrap().revizuit,
            "{al_doilea} e randul apasat"
        );
        assert!(
            !a.lista().obiect(0).unwrap().revizuit,
            "{primul} era doar selectat, nu apasat"
        );
        // si randul apasat devine cel selectat, ca sa nu se mai desincronizeze
        assert_eq!(a.lista().selectat(), Some(1));
        // al doilea clic scoate bifa
        assert!(a.comanda(Comanda::ComutaRevizuitLa(1)));
        assert!(!a.lista().obiect(1).unwrap().revizuit);
        // un rand care nu exista nu face nimic
        assert!(!a.comanda(Comanda::ComutaRevizuitLa(999)));
    }

    /// P1 din `cercetare/audit-qa-2026-11-18.md`: modelul editorului
    /// (alinierea completa) se refacea la **fiecare desenare**, si inca o
    /// data in `numar_diferente`, in `aplica` si in `treci`. Pe un obiect de
    /// 30.000 de linii o construire tine 2,4 s: „Aplica" insemna 7-10 secunde
    /// de fereastra inghetata, iar `r` sau `i` (bifa, in plan) tot 2,4 s,
    /// desi nu schimba textul.
    #[test]
    fn modelul_editorului_se_reface_doar_cand_se_schimba_textul() {
        let (mut a, _, _) = aplicatie_editare();
        let n0 = a.refaceri_model();
        let diferente = a.model_editor().unwrap().diferente.len();
        assert!(diferente > 0, "obiectul de proba chiar difera");
        let n1 = a.refaceri_model();
        assert!(
            n1 <= n0 + 1,
            "cel mult o construire: {n0} -> {n1} (deschiderea obiectului il construise deja)"
        );

        // desenari repetate: acelasi model
        for _ in 0..5 {
            assert_eq!(a.model_editor().unwrap().diferente.len(), diferente);
        }
        assert_eq!(a.refaceri_model(), n1, "desenarea nu reface nimic");

        // bifa si „in plan" nu ating textul, deci nici modelul
        a.comanda(Comanda::ComutaRevizuit);
        a.comanda(Comanda::ComutaInPlan);
        a.model_editor().unwrap();
        assert_eq!(a.refaceri_model(), n1, "bifa nu reface modelul");

        // o schimbare de text il reface, o singura data
        a.comanda(Comanda::AplicaToate);
        assert_eq!(a.model_editor().unwrap().diferente.len(), 0);
        let n2 = a.refaceri_model();
        assert_eq!(n2, n1 + 1, "o singura refacere dupa Aplica toate");
        a.model_editor().unwrap();
        assert_eq!(a.refaceri_model(), n2);

        // si unirea ciclului de cursor taie blocurile altfel: se reface
        a.comanda(Comanda::ComutaUnireaCursorului(true));
        a.model_editor().unwrap();
        assert!(a.refaceri_model() > n2);
    }

    /// F-A4 si F-A14 din `cercetare/audit-qa-2026-11-18.md`: Ctrl+Z avea un
    /// istoric **global**, deci anula o schimbare de pe alt obiect, fara sa se
    /// vada nimic pe ecran (v3.4 golea istoricul la deschiderea unui obiect);
    /// iar un obiect criptat se deschidea in editor cu un diff fals „totul
    /// sters", unde „Aplica toate" salva un text final gol.
    #[test]
    fn anularea_e_a_obiectului_deschis_iar_criptatele_nu_intra_in_editor() {
        // doua obiecte (pA, pB): testul cere chiar trecerea pe ALT obiect; cu
        // unul singur, `j` ramane pe loc (etapa 16) si nu dovedea nimic
        let mut a = aplicatie();
        a.comanda(Comanda::Deschide);
        let id = a.id_obiect().expect("un obiect deschis");
        assert!(a.comanda(Comanda::SalveazaText {
            id,
            text: "CREATE PROC dbo.pA AS\nSELECT 1\n-- scris de mine".into()
        }));
        assert!(a.se_poate_anula());
        // alt obiect: ce s-a facut pe cel dinainte nu se mai anuleaza de aici
        a.comanda(Comanda::ObiectUrmator);
        assert_eq!(a.obiect().unwrap().nume, "pB");
        assert!(
            !a.se_poate_anula(),
            "istoricul e al obiectului deschis, nu global"
        );
        assert!(!a.comanda(Comanda::Anuleaza), "si nu face nimic pe ascuns");

        // un obiect criptat nu are model de editor
        let mut criptate = aplicatie_criptat();
        criptate.comanda(Comanda::Selecteaza(0));
        assert_eq!(
            criptate.lista().obiect(0).map(|o| o.stare),
            Some(Stare::Criptat)
        );
        assert!(
            criptate.model_editor().is_none(),
            "un obiect criptat n-are ce arata in editor"
        );
        criptate.comanda(Comanda::Deschide);
        assert_eq!(criptate.ecran(), Ecran::Lista, "deci ramane in lista");
    }

    /// O comparare cu un obiect criptat (definitie goala intr-o parte).
    fn aplicatie_criptat() -> Aplicatie {
        let sursa = modul::pregateste(&[obiect_db("pCriptat", "")]);
        let tinta = modul::pregateste(&[obiect_db(
            "pCriptat",
            "CREATE PROC dbo.pCriptat AS SELECT 1",
        )]);
        let comparare = modul::compara(
            "Referinta",
            &sursa,
            &[("Client1".to_string(), tinta.clone())],
            |_| {},
        );
        Aplicatie::noua(
            comparare.randuri,
            sursa,
            vec![("Client1".to_string(), tinta)],
            Revizuire::noua(),
        )
    }

    /// F-A18: ca in v3.4, „Aplica toate" marcheaza obiectul revizuit, iar
    /// „In plan" pe un obiect fara cod spune ca a fost refuzat.
    #[test]
    fn aplica_toate_bifeaza_si_in_plan_refuzat_se_spune() {
        let (mut a, _, _) = aplicatie_editare();
        assert!(!a.obiect().unwrap().revizuit);
        assert!(a.comanda(Comanda::AplicaToate));
        assert!(a.obiect().unwrap().revizuit, "Aplica toate = revizuit");
        assert_eq!(a.revizuire().numar(), 1, "si in starea salvata");

        let mut b = aplicatie();
        let p = (0..b.lista().vizibile().len())
            .find(|&p| b.lista().obiect(p).unwrap().nume == "pA")
            .unwrap();
        b.comanda(Comanda::Selecteaza(p));
        b.comanda(Comanda::ComutaInPlan);
        assert!(!b.refuzat_in_plan(), "pA se poate deploia");

        // un obiect doar in baza modificata n-are cod de deploiat
        let sursa = modul::pregateste(&[obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1")]);
        let tinta = modul::pregateste(&[
            obiect_db("pA", "CREATE PROC dbo.pA AS\nSELECT 1"),
            obiect_db("pExtra", "CREATE PROC dbo.pExtra AS\nSELECT 2"),
        ]);
        let t = vec![("Client1".to_string(), tinta)];
        let c = modul::compara("Ref", &sursa, &t, |_| {});
        let mut x = Aplicatie::noua(c.randuri, sursa, t, Revizuire::noua());
        let p = (0..x.lista().vizibile().len())
            .find(|&p| x.lista().obiect(p).unwrap().nume == "pExtra")
            .expect("pExtra e in lista");
        x.comanda(Comanda::Selecteaza(p));
        x.comanda(Comanda::ComutaInPlan);
        assert!(x.refuzat_in_plan(), "refuzul se spune");
        assert!(x.lista().in_plan().is_empty());
        x.comanda(Comanda::MutaSelectia(0));
        assert!(!x.refuzat_in_plan(), "si dispare la urmatoarea comanda");
    }

    /// Etapa 16: taburile in aplicatie. Obiectul din tabul activ e si
    /// selectia listei; editarea fixeaza; `j` sare peste fixate.
    #[test]
    fn taburile_urmeaza_regulile_in_aplicatie() {
        let mut a = aplicatie();
        a.comanda(Comanda::Deschide);
        assert_eq!(
            a.taburi().previzualizare().map(|c| c.1.as_str()),
            Some("dbo.pA")
        );
        // dublu clic: pA fixat; j incarca pB in previzualizare
        a.comanda(Comanda::FixeazaSelectia);
        a.comanda(Comanda::ObiectUrmator);
        assert_eq!(a.obiect().unwrap().nume, "pB");
        assert_eq!(a.taburi().fixate().len(), 1);
        // clic pe tabul fixat: selectia listei il urmeaza
        a.comanda(Comanda::ActiveazaTab(Some(0)));
        assert_eq!(a.obiect().unwrap().nume, "pA");
        // j de pe pA: pB nu e fixat, deci el; inca un j: capat (pDoarSursa
        // nu se poate deschide), mesaj si ramane pe loc
        a.comanda(Comanda::ObiectUrmator);
        assert_eq!(a.obiect().unwrap().nume, "pB");
        a.comanda(Comanda::ObiectUrmator);
        assert_eq!(a.mesaj_tab(), Some(MesajTab::Capat));
        assert_eq!(a.obiect().unwrap().nume, "pB");
        // editarea textului final in previzualizare il fixeaza
        let id = a.id_obiect().unwrap();
        a.comanda(Comanda::SalveazaText {
            id,
            text: "CREATE PROC dbo.pB AS\nSELECT 2".into(),
        });
        assert_eq!(a.taburi().fixate().len(), 2);
        assert_eq!(a.taburi().previzualizare(), None);
        // Ctrl+W pe pB: activ devine pA (vecinul din stanga)
        a.comanda(Comanda::InchideTabActiv);
        assert_eq!(a.obiect().unwrap().nume, "pA");
        // ultimul tab inchis: inapoi in lista
        a.comanda(Comanda::InchideTabActiv);
        assert_eq!(a.ecran(), Ecran::Lista);
    }

    #[test]
    fn tabul_activat_cu_filtru_care_il_ascunde_se_arata_oricum() {
        let mut a = aplicatie();
        a.comanda(Comanda::Deschide);
        a.comanda(Comanda::FixeazaSelectia);
        a.comanda(Comanda::Cauta("pB".into()));
        a.comanda(Comanda::Selecteaza(0));
        assert_eq!(a.obiect().unwrap().nume, "pB");
        a.comanda(Comanda::ActiveazaTab(Some(0)));
        assert_eq!(
            a.obiect().unwrap().nume,
            "pA",
            "filtrul s-a golit ca sa-l arate"
        );
    }

    #[test]
    fn taburile_trec_prin_sesiune_si_prin_recomparare() {
        let mut a = aplicatie();
        a.comanda(Comanda::Deschide);
        a.comanda(Comanda::FixeazaSelectia);
        let mut s = gmotion_platforma::Sesiune::noua("SRV", "SQL", "Ref", "acum");
        taburi_in_sesiune(a.taburi(), &mut s);
        let t = taburi_din_sesiune(&s);
        assert_eq!(t.de_salvat(), a.taburi().de_salvat());
        // recomparare in care pA nu mai difera: tabul lui se inchide
        let sursa = modul::pregateste(&[obiect_db("pB", "CREATE PROC dbo.pB AS\nSELECT 2")]);
        let tinta = modul::pregateste(&[obiect_db("pB", "CREATE PROC dbo.pB AS\nSELECT 9")]);
        let tinte = vec![("Client1".to_string(), tinta)];
        let c = modul::compara("Ref", &sursa, &tinte, |_| {});
        let b = Aplicatie::noua(c.randuri, sursa, tinte, Revizuire::noua()).cu_taburi(t);
        assert!(b.taburi().fixate().is_empty());
        // fara taburi in sesiune: niciunul, fara eroare
        taburi_in_sesiune(&Taburi::noi(), &mut s);
        assert!(!s.date.contains_key("taburi"));
    }

    #[test]
    fn starea_coloanei_trece_prin_sesiune() {
        let mut s = gmotion_platforma::Sesiune::noua("SRV", "SQL", "Ref", "acum");
        assert_eq!(
            StareColoana::din_sesiune(&s),
            None,
            "nimic salvat: implicitul"
        );
        let c = StareColoana {
            latime: 900.0,
            fixata: false,
            dreapta: true,
            banda: true,
        };
        c.in_sesiune(&mut s);
        let r = StareColoana::din_sesiune(&s).unwrap();
        assert_eq!(r.latime, 520.0, "latimea ramane intre 200 si 520");
        assert!(!r.fixata && r.dreapta && r.banda);
        s.date
            .insert("coloana".into(), serde_json::json!("stricat"));
        assert_eq!(StareColoana::din_sesiune(&s), None);
    }
}
