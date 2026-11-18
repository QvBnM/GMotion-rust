//! Ce ecran e activ, ce obiect e deschis, ce face fiecare comanda.
//!
//! **Se strica astfel:** bifa pusa in editor nu se vede in lista (sau invers),
//! `j` / `k` din editor te intorc in lista in loc sa treaca la obiectul
//! urmator, sau obiectul deschis nu mai e cel selectat dupa un filtru. Toate
//! sunt bug-uri de stare, nu de desen — de-aia starea sta aici, fara Slint, si
//! se testeaza ca atare.

use gmotion_fereastra::model_editor::{construieste, marcheaza_textul_final, ModelEditor};
use gmotion_fereastra::model_lista::{
    Deploy, Detalii, FiltruRevizuire, Lista, Obiect, Ordine, Stare,
};
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
    MutaSelectia(i32),
    Selecteaza(usize),
    Deschide,
    InapoiLaLista,
    ObiectUrmator,
    ObiectAnterior,
    ComutaRevizuit,
    Filtreaza(Option<Stare>),
    FiltreazaTip(Option<String>),
    Cauta(String),
    Sorteaza(Ordine),
    /// Filtrul pe ce ai facut cu obiectul (nerevizuite, omise, deployate...).
    FiltreazaRevizuire(Option<FiltruRevizuire>),
    /// Intervalul de similaritate, in procente.
    FiltreazaScor(f64, f64),
    /// „In plan" pe tot ce se vede; `true` = si pe ce s-a deployat deja.
    PuneVizibileleInPlan(bool),
    /// Pune sau scoate obiectul selectat din plan.
    ComutaInPlan,
    /// Scoate din plan obiectul `schema.nume` (din ecranul de deploy).
    ScoateDinPlan(String),
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
    /// „Editeaza": textul final scris de mana.
    SalveazaText(String),
    /// „Omite" (`o`): analizat, lasat in afara deploy-ului.
    ComutaOmis,
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

/// Starea aplicatiei: lista, ecranul activ si progresul revizuirii.
pub struct Aplicatie {
    lista: Lista,
    ecran: Ecran,
    revizuire: Revizuire,
    /// Randurile compararii, pe `schema.nume`: de acolo vin amprentele.
    randuri: Vec<Rand>,
    sursa: Baza,
    tinta: Baza,
    /// Textele finale (Aplica, Reset, scris de mana), salvate in sesiune.
    editari: Editari,
    /// Pentru Ctrl+Z: obiectul si textul lui final de dinainte de fiecare
    /// schimbare. Nu se salveaza: anularea tine cat sesiunea de lucru.
    istoric: Vec<(Rand, Option<TextFinal>)>,
    /// Diferenta curenta din editor; ramane pe loc dupa „Aplica", cand
    /// diferenta aplicata dispare si urmatoarea ii ia locul.
    diferenta: usize,
    ghidat: Option<Ghidat>,
}

impl Aplicatie {
    /// `randuri` sunt randurile compararii; `sursa` si `tinta`, obiectele
    /// pregatite ale celor doua baze.
    pub fn noua(randuri: Vec<Rand>, sursa: Baza, tinta: Baza, revizuire: Revizuire) -> Self {
        let obiecte: Vec<Obiect> = randuri
            .iter()
            .map(|r| {
                let (schema, nume) = r.obiect.split_once('.').unwrap_or(("dbo", &r.obiect));
                Obiect {
                    schema: schema.to_string(),
                    nume: nume.to_string(),
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
                }
            })
            .collect();
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
            tinta,
            editari: Editari::noi(),
            istoric: Vec::new(),
            diferenta: 0,
            ghidat: None,
        }
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
        self.editari.curata(&self.randuri);
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

    /// Obiectul curent are text final (deci „Reset" are ce face), si e scris
    /// si de mana?
    pub fn stare_text_final(&self) -> (bool, bool) {
        self.obiect()
            .and_then(|o| self.rand(&o.id()))
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
        let id = self.obiect()?.id();
        let r = self.rand(&id)?;
        let (a, b) = self.texte(&id)?;
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

    fn rand(&self, id: &str) -> Option<&Rand> {
        self.randuri.iter().find(|r| r.obiect == id)
    }

    /// Textele pregatite ale obiectului selectat, daca exista in ambele baze.
    fn texte(&self, id: &str) -> Option<(&str, &str)> {
        match (self.sursa.get(id), self.tinta.get(id)) {
            (Some(a), Some(b)) => Some((a.afisare.as_str(), b.afisare.as_str())),
            _ => None,
        }
    }

    /// Modelul editorului pentru obiectul curent. `None` pentru obiectele care
    /// exista doar intr-o parte: nu au cu ce fi comparate.
    ///
    /// In dreapta e **textul final** (daca exista), nu textul bazei: editorul
    /// arata mereu sursa fata de ce va ajunge in baza, iar liniile schimbate
    /// fata de baza sunt marcate (adus / scris de tine).
    pub fn model_editor(&self) -> Option<ModelEditor> {
        let id = self.obiect()?.id();
        let (a, b) = self.texte(&id)?;
        let Some(r) = self.rand(&id) else {
            return Some(construieste(a, b));
        };
        let curent = self.editari.curent(r, b);
        let mut m = construieste(a, curent);
        let manual = self.editari.text_final(r).is_some_and(|t| t.manual);
        marcheaza_textul_final(&mut m, b, curent, manual);
        Some(m)
    }

    /// Propozitia pentru previzualizare si detaliile randului curent.
    pub fn texte_obiect(&self) -> Option<(&str, &str, Vec<&'static str>)> {
        let id = self.obiect()?.id();
        let (a, b) = self.texte(&id)?;
        let setari = match (self.sursa.get(&id), self.tinta.get(&id)) {
            (Some(x), Some(y)) => modul::setari_diferite(x.setari, y.setari),
            _ => Vec::new(),
        };
        Some((a, b, setari))
    }

    /// Calculeaza detaliile („Difera in", `+N −M`) pentru randurile de pe
    /// ecran, daca lipsesc. Restul listei ramane necalculat.
    pub fn asigura_detalii(&mut self, de_la: usize, pana_la: usize) {
        for p in self.lista.fara_detalii(de_la, pana_la) {
            let Some(o) = self.lista.obiect(p) else {
                continue;
            };
            let id = o.id();
            let (Some(a), Some(b)) = (self.sursa.get(&id), self.tinta.get(&id)) else {
                continue;
            };
            let setari = modul::setari_diferite(a.setari, b.setari);
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
        match c {
            Comanda::MutaSelectia(pas) => self.lista.muta_selectia(pas),
            Comanda::Selecteaza(p) => self.lista.selecteaza(p),
            Comanda::Deschide => {
                self.diferenta = 0;
                self.ghidat = None;
                // un obiect care exista doar intr-o parte n-are ce arata in
                // editor: ecranul ramane pe lista
                if self.model_editor().is_some() {
                    self.ecran = Ecran::Editor;
                }
            }
            Comanda::InapoiLaLista => self.ecran = Ecran::Lista,
            // din editor, obiectul urmator se deschide direct: altfel ar
            // trebui drum dus-intors prin lista pentru fiecare obiect
            Comanda::ObiectUrmator => {
                self.treci(1);
                self.diferenta = 0;
                self.ghidat = None;
            }
            Comanda::ObiectAnterior => {
                self.treci(-1);
                self.diferenta = 0;
                self.ghidat = None;
            }
            Comanda::ComutaRevizuit => return self.comuta_revizuit(),
            Comanda::Filtreaza(s) => self.lista.filtreaza_stare(s),
            Comanda::FiltreazaTip(t) => self.lista.filtreaza_tip(t),
            Comanda::Cauta(t) => self.lista.cauta(&t),
            Comanda::Sorteaza(o) => self.lista.sorteaza(o),
            Comanda::FiltreazaRevizuire(f) => self.lista.filtreaza_revizuire(f),
            Comanda::FiltreazaScor(min, max) => self.lista.filtreaza_scor(min, max),
            Comanda::PuneVizibileleInPlan(cu_deployate) => {
                self.lista.pune_vizibilele_in_plan(cu_deployate);
            }
            Comanda::ComutaInPlan => {
                if let Some(p) = self.lista.selectat() {
                    self.lista.comuta_in_plan(p);
                }
            }
            Comanda::ScoateDinPlan(id) => {
                self.lista.scoate_din_plan(&id);
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
            Comanda::AplicaToate => {
                return self.schimba_textul(|e, r, sursa, baza| e.aplica_toate(r, sursa, baza));
            }
            Comanda::Reset => {
                return self.schimba_textul(|e, r, _, _| {
                    e.reset(r);
                });
            }
            Comanda::SalveazaText(text) => {
                let text = gmotion_core::pregateste_afisare(&text);
                return self.schimba_textul(move |e, r, _, baza| e.pune(r, text, true, baza));
            }
            Comanda::Anuleaza => {
                if let Some((r, inainte)) = self.istoric.pop() {
                    self.editari.restaureaza(&r, inainte);
                    return true;
                }
            }
            Comanda::ComutaOmis => return self.comuta_omis(),
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
        let Some(id) = self.obiect().map(|o| o.id()) else {
            return false;
        };
        let (Some(r), Some((a, b))) = (self.rand(&id).cloned(), self.texte(&id)) else {
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
        // dupa o schimbare, diferenta curenta nu poate trece de ultima
        let n = self.numar_diferente();
        self.diferenta = self.diferenta.min(n.saturating_sub(1));
        true
    }

    /// Aplica diferenta `k` (indexul din editor) a obiectului curent.
    fn aplica(&mut self, k: usize) -> bool {
        let Some(bloc) = self.model_editor().and_then(|m| m.blocuri.get(k).copied()) else {
            return false;
        };
        self.schimba_textul(|e, r, a, b| {
            e.aplica_blocuri(r, a, b, &[bloc]);
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
            self.lista.pune_deploy(&pas.obiect, Deploy::Deployat);
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
            .filter_map(|o| self.rand(&o.id()))
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
        let (id, era) = (o.id(), o.omis);
        let Some(rand) = self.rand(&id).cloned() else {
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
    /// `referinta` si `tinta` sunt numele bazelor; `facut_la`, momentul.
    pub fn plan(&self, referinta: &str, tinta: &str, facut_la: &str) -> modul::Plan {
        let alese = self.lista.in_plan();
        let comparare = modul::Comparare {
            referinta: referinta.to_string(),
            baze: vec![tinta.to_string()],
            randuri: self.randuri.clone(),
            numaratoare: modul::Numaratoare::default(),
        };
        modul::Plan::din_comparare(
            &comparare,
            &self.sursa,
            &[(tinta.to_string(), self.tinta.clone())],
            |r| alese.iter().any(|a| a == &r.obiect),
            facut_la,
        )
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
        let (id, era) = (o.id(), o.revizuit);
        let Some(rand) = self.rand(&id).cloned() else {
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

    /// Starea de revizuire, curatata de bifele care nu mai corespund, gata de
    /// salvat in sesiune.
    pub fn revizuire_de_salvat(&mut self) -> &Revizuire {
        self.revizuire.curata(&self.randuri);
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
        Aplicatie::noua(comparare.randuri, sursa, tinta, Revizuire::noua())
    }

    #[test]
    fn planul_si_deploy_ul_urmeaza_ce_e_in_plan() {
        let mut app = aplicatie();
        // planul gol: deploy-ul nu se deschide
        app.comanda(Comanda::DeschideDeploy);
        assert_eq!(app.ecran(), Ecran::Lista);
        app.comanda(Comanda::Selecteaza(0));
        app.comanda(Comanda::ComutaInPlan);
        let id = app.obiect().unwrap().id();
        assert_eq!(app.lista().in_plan(), vec![id.clone()]);
        app.comanda(Comanda::DeschideDeploy);
        assert_eq!(app.ecran(), Ecran::Deploy);
        let plan = app.plan("Ref", "Baza", "acum");
        assert_eq!(plan.pasi.len(), 1);
        assert_eq!(plan.pasi[0].obiect, id);
        // scos ultimul obiect: inapoi in lista, planul e gol
        app.comanda(Comanda::ScoateDinPlan(id));
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
        assert_eq!(a.lista().in_plan(), ["dbo.pA"]);
        a.comanda(Comanda::Cauta(String::new()));
        a.comanda(Comanda::PuneVizibileleInPlan(false));
        assert_eq!(
            a.lista().in_plan(),
            ["dbo.pA", "dbo.pB", "dbo.pDoarSursa"],
            "toate se pot deploia"
        );
    }

    #[test]
    fn dupa_deploy_obiectele_din_plan_sunt_deployate_si_tin_minte() {
        let mut a = aplicatie();
        a.comanda(Comanda::PuneVizibileleInPlan(false));
        let plan = a.plan("Referinta", "Client1", "acum");
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
        let b = Aplicatie::noua(c.randuri, sursa, tinta, salvata.clone());
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
        let b = Aplicatie::noua(c.randuri, sursa, tinta, salvata);
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
        let mut a = Aplicatie::noua(c.randuri, sursa, tinta, Revizuire::noua());
        a.comanda(Comanda::Deschide);
        (a, SURSA, BAZA)
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
        let m = a.model_editor().unwrap();
        assert!(m
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
        assert!(a.comanda(Comanda::SalveazaText(text.clone())));
        assert_eq!(a.stare_text_final(), (true, true));
        let t = a.texte_finale();
        assert_eq!(t[&("Client1".to_string(), "dbo.pE".to_string())], text);
        // acelasi text inca o data nu e o schimbare
        assert!(!a.comanda(Comanda::SalveazaText(text)));
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
}
