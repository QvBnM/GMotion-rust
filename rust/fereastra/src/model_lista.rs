//! Modelul listei de diferente: ce obiecte se vad, in ce ordine, cu ce
//! contoare, si care e selectat.
//!
//! **Se strica astfel:** contorul din bara laterala spune altceva decat
//! tabelul (filtrul aplicat la unul si nu la altul), selectia se muta singura
//! cand se schimba filtrul, sau cautarea ascunde obiecte care se potrivesc.
//! Toate trei se vad abia pe o lista mare, de-aia modelul se testeaza fara
//! fereastra.
//!
//! Nu stie nimic despre module: primeste randuri gata clasificate. Asa ramane
//! testabil si nu leaga fereastra de `diff-obiecte`.

/// In ce stare e un obiect fata de baza comparata. Literele sunt cele din
/// machete: starea nu se spune doar prin culoare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stare {
    Modificat,
    DoarSursa,
    DoarBaza,
    Identic,
    Criptat,
}

impl Stare {
    pub fn litera(self) -> &'static str {
        match self {
            Stare::Modificat => "M",
            Stare::DoarSursa => "S",
            Stare::DoarBaza => "B",
            Stare::Identic => "=",
            Stare::Criptat => "#",
        }
    }

    /// Numele categoriei in bara laterala, ca in macheta.
    pub fn categorie(self) -> &'static str {
        match self {
            Stare::Modificat => "Modificate",
            Stare::DoarSursa => "Doar in sursa",
            Stare::DoarBaza => "Doar in baza",
            Stare::Identic => "Identice",
            Stare::Criptat => "Criptate",
        }
    }

    /// Cum se citeste starea unui rand (pentru cititoarele de ecran).
    pub fn descriere(self) -> &'static str {
        match self {
            Stare::Modificat => "modificat",
            Stare::DoarSursa => "doar in sursa",
            Stare::DoarBaza => "doar in baza",
            Stare::Identic => "identic",
            Stare::Criptat => "criptat",
        }
    }

    /// Obiectul chiar difera (intra in progresul revizuirii si in deploy).
    pub fn difera(self) -> bool {
        matches!(self, Stare::Modificat | Stare::DoarSursa | Stare::DoarBaza)
    }

    /// Ordinea din bara laterala.
    pub const TOATE: [Stare; 5] = [
        Stare::Modificat,
        Stare::DoarSursa,
        Stare::DoarBaza,
        Stare::Identic,
        Stare::Criptat,
    ];
}

/// Ce se stie despre un obiect dupa alinierea lui. Lipseste pana cand lista
/// chiar are nevoie: pe zeci de mii de obiecte, alinierea fiecaruia ar fi
/// scumpa si aproape toata aruncata (vezi `diff-obiecte::detalii_obiect`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Detalii {
    pub difera_in: String,
    pub linii_plus: usize,
    pub linii_minus: usize,
}

/// Ce s-a intamplat cu obiectul la deploy (coloana Deploy din v3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Deploy {
    #[default]
    Nu,
    /// Scris intr-un deploy, si obiectul e tot cel de atunci.
    Deployat,
    /// Scris intr-un deploy, dar s-a schimbat de atunci: e o versiune noua.
    VersiuneNoua,
}

/// Filtrul pe ce ai facut cu obiectul (v3.4: „Stare").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiltruRevizuire {
    Nerevizuite,
    Revizuite,
    Omise,
    InPlan,
    Deployate,
    Nedeployate,
}

impl FiltruRevizuire {
    pub const TOATE: [FiltruRevizuire; 6] = [
        FiltruRevizuire::Nerevizuite,
        FiltruRevizuire::Revizuite,
        FiltruRevizuire::Omise,
        FiltruRevizuire::InPlan,
        FiltruRevizuire::Deployate,
        FiltruRevizuire::Nedeployate,
    ];

    pub fn eticheta(self) -> &'static str {
        match self {
            FiltruRevizuire::Nerevizuite => "Nerevizuite",
            FiltruRevizuire::Revizuite => "Revizuite",
            FiltruRevizuire::Omise => "Omise",
            FiltruRevizuire::InPlan => "In plan",
            FiltruRevizuire::Deployate => "Deployate",
            FiltruRevizuire::Nedeployate => "Nedeployate",
        }
    }

    fn potriveste(self, o: &Obiect) -> bool {
        match self {
            // nerezolvate: nici revizuite, nici omise
            FiltruRevizuire::Nerevizuite => !o.revizuit && !o.omis,
            FiltruRevizuire::Revizuite => o.revizuit,
            FiltruRevizuire::Omise => o.omis,
            FiltruRevizuire::InPlan => o.in_plan,
            FiltruRevizuire::Deployate => o.deploy == Deploy::Deployat,
            FiltruRevizuire::Nedeployate => o.deploy != Deploy::Deployat,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Obiect {
    pub schema: String,
    pub nume: String,
    /// Codul din `sys.objects.type` (`P`, `V`, `FN`...); numele pentru
    /// utilizator vine din `gmotion_core::nume_tip`.
    pub tip: String,
    pub stare: Stare,
    pub detalii: Option<Detalii>,
    pub revizuit: bool,
    /// Pus in planul de deploy (butonul „In plan").
    pub in_plan: bool,
    /// Omis („Omite"): analizat, dar lasat in afara deploy-ului.
    pub omis: bool,
    /// Similaritatea cu sursa, 0-100; doar pentru obiectele modificate.
    pub scor: Option<f64>,
    pub deploy: Deploy,
}

impl Obiect {
    pub fn id(&self) -> String {
        format!("{}.{}", self.schema, self.nume)
    }

    /// Se poate deploia: are cod in sursa care se poate citi. Un obiect doar
    /// in baza n-are ce scrie, unul criptat n-are cod.
    pub fn se_poate_deploia(&self) -> bool {
        matches!(self.stare, Stare::Modificat | Stare::DoarSursa)
    }
}

/// Coloana dupa care se sorteaza; sensul il tine `Lista`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ordine {
    /// Alfabetic pe `schema.nume`.
    #[default]
    Nume,
    Tip,
    /// Liniile mutate (`+N −M`); implicit, cele mai multe primele.
    Linii,
    /// Similaritatea; implicit, cele mai diferite primele.
    Scor,
    DiferaIn,
    /// Revizuit / omis / nerevizuit.
    Revizuire,
    Deploy,
}

impl Ordine {
    /// Sensul cu care incepe o coloana: liniile, descrescator (cele mai
    /// schimbate intai); restul, crescator.
    fn implicit_descrescator(self) -> bool {
        matches!(self, Ordine::Linii)
    }
}

#[derive(Debug, Clone)]
pub struct Filtru {
    pub stare: Option<Stare>,
    /// Codul tipului (`P`, `V`...).
    pub tip: Option<String>,
    pub cautare: String,
    pub revizuire: Option<FiltruRevizuire>,
    /// Intervalul de similaritate, in procente. Cat timp e 0-100, nu se
    /// filtreaza nimic; altfel, obiectele fara scor (doar intr-o parte,
    /// criptate) nu se mai vad.
    pub scor_min: f64,
    pub scor_max: f64,
}

impl Default for Filtru {
    fn default() -> Self {
        Filtru {
            stare: None,
            tip: None,
            cautare: String::new(),
            revizuire: None,
            scor_min: 0.0,
            scor_max: 100.0,
        }
    }
}

impl Filtru {
    fn scor_activ(&self) -> bool {
        self.scor_min > 0.0 || self.scor_max < 100.0
    }
}

/// Lista, cu filtrul si selectia ei.
#[derive(Debug, Clone)]
pub struct Lista {
    obiecte: Vec<Obiect>,
    filtru: Filtru,
    ordine: Ordine,
    descrescator: bool,
    /// Indici in `obiecte`, in ordinea afisata.
    vizibile: Vec<usize>,
    /// Pozitia in `vizibile`.
    selectat: Option<usize>,
}

impl Lista {
    pub fn noua(obiecte: Vec<Obiect>) -> Self {
        let mut l = Lista {
            obiecte,
            filtru: Filtru::default(),
            ordine: Ordine::default(),
            descrescator: false,
            vizibile: Vec::new(),
            selectat: None,
        };
        l.recalculeaza();
        l
    }

    pub fn obiecte(&self) -> &[Obiect] {
        &self.obiecte
    }

    pub fn filtru(&self) -> &Filtru {
        &self.filtru
    }

    pub fn ordine(&self) -> Ordine {
        self.ordine
    }

    pub fn descrescator(&self) -> bool {
        self.descrescator
    }

    /// Indicii obiectelor afisate, in ordinea afisata.
    pub fn vizibile(&self) -> &[usize] {
        &self.vizibile
    }

    pub fn obiect(&self, pozitie: usize) -> Option<&Obiect> {
        self.vizibile.get(pozitie).map(|&i| &self.obiecte[i])
    }

    pub fn selectat(&self) -> Option<usize> {
        self.selectat
    }

    pub fn obiect_selectat(&self) -> Option<&Obiect> {
        self.selectat.and_then(|p| self.obiect(p))
    }

    pub fn selecteaza(&mut self, pozitie: usize) {
        if pozitie < self.vizibile.len() {
            self.selectat = Some(pozitie);
        }
    }

    /// Muta selectia cu `pas` randuri, fara sa iasa din lista.
    pub fn muta_selectia(&mut self, pas: i32) {
        if self.vizibile.is_empty() {
            self.selectat = None;
            return;
        }
        let acum = self.selectat.unwrap_or(0) as i32;
        let nou = (acum + pas).clamp(0, self.vizibile.len() as i32 - 1);
        self.selectat = Some(nou as usize);
    }

    pub fn filtreaza_stare(&mut self, stare: Option<Stare>) {
        self.filtru.stare = stare;
        self.recalculeaza();
    }

    pub fn filtreaza_tip(&mut self, tip: Option<String>) {
        self.filtru.tip = tip;
        self.recalculeaza();
    }

    pub fn cauta(&mut self, text: &str) {
        self.filtru.cautare = text.to_string();
        self.recalculeaza();
    }

    /// Sorteaza dupa coloana; a doua oara pe aceeasi coloana, sensul se
    /// inverseaza (ca in v3.4, clic pe capul coloanei).
    pub fn sorteaza(&mut self, ordine: Ordine) {
        if ordine == self.ordine {
            self.descrescator = !self.descrescator;
        } else {
            self.ordine = ordine;
            self.descrescator = ordine.implicit_descrescator();
        }
        self.recalculeaza();
    }

    pub fn filtreaza_revizuire(&mut self, f: Option<FiltruRevizuire>) {
        self.filtru.revizuire = f;
        self.recalculeaza();
    }

    /// Intervalul de similaritate, in procente; capetele se ordoneaza si se
    /// taie la 0-100, ca un interval scris invers sa nu goleasca lista.
    pub fn filtreaza_scor(&mut self, min: f64, max: f64) {
        let (a, b) = (min.clamp(0.0, 100.0), max.clamp(0.0, 100.0));
        self.filtru.scor_min = a.min(b);
        self.filtru.scor_max = a.max(b);
        self.recalculeaza();
    }

    /// Starea de deploy a obiectului `schema.nume`.
    pub fn pune_deploy(&mut self, id: &str, d: Deploy) {
        for o in self.obiecte.iter_mut().filter(|o| o.id() == id) {
            o.deploy = d;
        }
    }

    /// Pune in plan tot ce se vede si se poate deploia, in afara de ce e omis
    /// si — daca nu se cere altfel — de ce s-a deployat deja (v3.4: „include
    /// si ce a fost deja trimis la deploy"). Intoarce cate s-au adaugat.
    pub fn pune_vizibilele_in_plan(&mut self, cu_deployate: bool) -> usize {
        let mut n = 0;
        for &i in &self.vizibile {
            let o = &mut self.obiecte[i];
            if o.in_plan || o.omis || !o.se_poate_deploia() {
                continue;
            }
            if o.deploy == Deploy::Deployat && !cu_deployate {
                continue;
            }
            o.in_plan = true;
            n += 1;
        }
        n
    }

    pub fn comuta_revizuit(&mut self, pozitie: usize) {
        if let Some(&i) = self.vizibile.get(pozitie) {
            self.obiecte[i].revizuit = !self.obiecte[i].revizuit;
        }
    }

    /// Starea de revizuire a obiectului vizibil: revizuit, omis, sau niciuna.
    /// Cele doua se exclud; un obiect omis iese si din plan.
    pub fn pune_revizuirea(&mut self, pozitie: usize, revizuit: bool, omis: bool) {
        if let Some(&i) = self.vizibile.get(pozitie) {
            let o = &mut self.obiecte[i];
            o.revizuit = revizuit && !omis;
            o.omis = omis;
            if omis {
                o.in_plan = false;
            }
        }
    }

    /// Pune sau scoate obiectul vizibil din plan. Intoarce `false` (si nu
    /// schimba nimic) pentru un obiect care nu se poate deploia.
    pub fn comuta_in_plan(&mut self, pozitie: usize) -> bool {
        let Some(&i) = self.vizibile.get(pozitie) else {
            return false;
        };
        if !self.obiecte[i].se_poate_deploia() {
            return false;
        }
        self.obiecte[i].in_plan = !self.obiecte[i].in_plan;
        true
    }

    /// Scoate din plan obiectul cu `schema.nume` dat, vizibil sau nu.
    pub fn scoate_din_plan(&mut self, id: &str) {
        for o in self.obiecte.iter_mut().filter(|o| o.id() == id) {
            o.in_plan = false;
        }
    }

    /// Obiectele din plan, `schema.nume`, in ordinea din lista completa.
    pub fn in_plan(&self) -> Vec<String> {
        self.obiecte
            .iter()
            .filter(|o| o.in_plan)
            .map(Obiect::id)
            .collect()
    }

    /// Pune detaliile calculate pentru un obiect vizibil.
    pub fn pune_detalii(&mut self, pozitie: usize, d: Detalii) {
        if let Some(&i) = self.vizibile.get(pozitie) {
            self.obiecte[i].detalii = Some(d);
        }
    }

    /// Pozitiile din intervalul `[de_la, pana_la)` carora le lipsesc detaliile:
    /// exact ce trebuie calculat pentru randurile care tocmai au intrat pe
    /// ecran.
    pub fn fara_detalii(&self, de_la: usize, pana_la: usize) -> Vec<usize> {
        let pana_la = pana_la.min(self.vizibile.len());
        (de_la.min(pana_la)..pana_la)
            .filter(|&p| {
                let o = &self.obiecte[self.vizibile[p]];
                o.detalii.is_none() && o.stare == Stare::Modificat
            })
            .collect()
    }

    /// Contoarele din bara laterala: se numara pe **toate** obiectele, nu pe
    /// cele filtrate — altfel filtrul si-ar ascunde propriile categorii.
    pub fn contoare_stari(&self) -> Vec<(Stare, usize)> {
        Stare::TOATE
            .iter()
            .map(|&s| (s, self.obiecte.iter().filter(|o| o.stare == s).count()))
            .collect()
    }

    /// Tipurile prezente, cu numarul de obiecte **care difera**, in ordinea in
    /// care apar in machete (proceduri, functii, view-uri, triggere).
    pub fn contoare_tipuri(&self) -> Vec<(String, usize)> {
        const ORDINE: [&str; 6] = ["P", "FN", "IF", "TF", "V", "TR"];
        let mut rez: Vec<(String, usize)> = Vec::new();
        for o in self.obiecte.iter().filter(|o| o.stare.difera()) {
            match rez.iter_mut().find(|(t, _)| *t == o.tip) {
                Some((_, n)) => *n += 1,
                None => rez.push((o.tip.clone(), 1)),
            }
        }
        let rang = |t: &str| ORDINE.iter().position(|x| *x == t).unwrap_or(ORDINE.len());
        rez.sort_by(|a, b| rang(&a.0).cmp(&rang(&b.0)).then(a.0.cmp(&b.0)));
        rez
    }

    /// Cate obiecte care difera au fost deployate (si sunt inca cele de
    /// atunci), din cate.
    pub fn progres_deploy(&self) -> (usize, usize) {
        let difera: Vec<&Obiect> = self.obiecte.iter().filter(|o| o.stare.difera()).collect();
        (
            difera
                .iter()
                .filter(|o| o.deploy == Deploy::Deployat)
                .count(),
            difera.len(),
        )
    }

    /// Cate obiecte care difera au fost revizuite, din cate.
    pub fn progres_revizuire(&self) -> (usize, usize) {
        let difera: Vec<&Obiect> = self.obiecte.iter().filter(|o| o.stare.difera()).collect();
        (difera.iter().filter(|o| o.revizuit).count(), difera.len())
    }

    /// „212 nerevizuite din 312 modificate" — despre ce se vede acum.
    pub fn rezumat_vizibile(&self) -> String {
        let nerevizuite = self
            .vizibile
            .iter()
            .filter(|&&i| self.obiecte[i].stare.difera() && !self.obiecte[i].revizuit)
            .count();
        let total = self.vizibile.len();
        format!("{nerevizuite} nerevizuite din {total}")
    }

    fn recalculeaza(&mut self) {
        // obiectul selectat ramane selectat daca mai e vizibil: altfel
        // previzualizarea de dedesubt ar sari pe alt obiect la fiecare filtru
        let id_selectat = self.obiect_selectat().map(Obiect::id);
        let cautare = self.filtru.cautare.to_lowercase();
        let mut v: Vec<usize> = self
            .obiecte
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                self.filtru.stare.is_none_or(|s| o.stare == s)
                    && self.filtru.tip.as_deref().is_none_or(|t| o.tip == t)
                    && (cautare.is_empty() || o.id().to_lowercase().contains(&cautare))
                    && self.filtru.revizuire.is_none_or(|f| f.potriveste(o))
                    && (!self.filtru.scor_activ()
                        || o.scor.is_some_and(|x| {
                            x >= self.filtru.scor_min && x <= self.filtru.scor_max
                        }))
            })
            .map(|(i, _)| i)
            .collect();
        let o = &self.obiecte;
        let linii = |i: usize| -> usize {
            o[i].detalii
                .as_ref()
                .map_or(0, |d| d.linii_plus + d.linii_minus)
        };
        let rang_revizuire = |i: usize| -> u8 {
            if o[i].revizuit {
                2
            } else if o[i].omis {
                1
            } else {
                0
            }
        };
        let rang_deploy = |i: usize| -> u8 {
            match o[i].deploy {
                Deploy::Nu => u8::from(o[i].in_plan),
                Deploy::VersiuneNoua => 2,
                Deploy::Deployat => 3,
            }
        };
        let ordine = self.ordine;
        v.sort_by(|&a, &b| {
            let c = match ordine {
                Ordine::Nume => std::cmp::Ordering::Equal,
                Ordine::Tip => o[a].tip.cmp(&o[b].tip),
                Ordine::Linii => linii(a).cmp(&linii(b)),
                // fara scor (doar intr-o parte): la capat, ca „necunoscut"
                Ordine::Scor => o[a]
                    .scor
                    .unwrap_or(101.0)
                    .total_cmp(&o[b].scor.unwrap_or(101.0)),
                Ordine::DiferaIn => {
                    let d = |i: usize| o[i].detalii.as_ref().map(|d| d.difera_in.clone());
                    d(a).cmp(&d(b))
                }
                Ordine::Revizuire => rang_revizuire(a).cmp(&rang_revizuire(b)),
                Ordine::Deploy => rang_deploy(a).cmp(&rang_deploy(b)),
            };
            // numele departajeaza mereu crescator, ca ordinea sa fie stabila
            let c = if self.descrescator { c.reverse() } else { c };
            c.then_with(|| o[a].id().cmp(&o[b].id()))
        });
        if ordine == Ordine::Nume && self.descrescator {
            v.reverse();
        }
        self.vizibile = v;
        self.selectat = match id_selectat {
            Some(id) => self
                .vizibile
                .iter()
                .position(|&i| self.obiecte[i].id() == id)
                .or(if self.vizibile.is_empty() {
                    None
                } else {
                    Some(0)
                }),
            None if self.vizibile.is_empty() => None,
            None => self.selectat.map(|p| p.min(self.vizibile.len() - 1)),
        };
    }
}

#[cfg(test)]
mod teste {
    use super::*;

    fn obiect(nume: &str, tip: &str, stare: Stare) -> Obiect {
        Obiect {
            schema: "dbo".into(),
            nume: nume.into(),
            tip: tip.into(),
            stare,
            detalii: None,
            revizuit: false,
            in_plan: false,
            omis: false,
            scor: None,
            deploy: Default::default(),
        }
    }

    fn lista_proba() -> Lista {
        Lista::noua(vec![
            obiect("usp_Stoc", "P", Stare::Modificat),
            obiect("usp_Import", "P", Stare::DoarSursa),
            obiect("fn_Curs", "FN", Stare::DoarBaza),
            obiect("vw_Facturi", "V", Stare::Modificat),
            obiect("usp_Vechi", "P", Stare::Identic),
            obiect("trg_Audit", "TR", Stare::Criptat),
        ])
    }

    #[test]
    fn in_plan_intra_doar_ce_se_poate_deploia() {
        let mut l = Lista::noua(vec![
            obiect("pMod", "P", Stare::Modificat),
            obiect("pNou", "P", Stare::DoarSursa),
            obiect("pExtra", "P", Stare::DoarBaza),
            obiect("pCriptat", "P", Stare::Criptat),
        ]);
        l.filtreaza_stare(None);
        let poz = |l: &Lista, n: &str| {
            (0..l.vizibile().len())
                .find(|&p| l.obiect(p).unwrap().nume == n)
                .unwrap()
        };
        for n in ["pMod", "pNou"] {
            let p = poz(&l, n);
            assert!(l.comuta_in_plan(p), "{n} se poate deploia");
        }
        for n in ["pExtra", "pCriptat"] {
            let p = poz(&l, n);
            assert!(!l.comuta_in_plan(p), "{n} n-are cod de deploiat");
        }
        assert_eq!(
            l.in_plan(),
            vec!["dbo.pMod".to_string(), "dbo.pNou".to_string()]
        );
        l.scoate_din_plan("dbo.pMod");
        assert_eq!(l.in_plan(), vec!["dbo.pNou".to_string()]);
        // a doua apasare scoate
        let p = poz(&l, "pNou");
        assert!(l.comuta_in_plan(p));
        assert!(l.in_plan().is_empty());
    }

    #[test]
    fn contoarele_se_numara_pe_tot_nu_pe_ce_e_filtrat() {
        let mut l = lista_proba();
        let inainte = l.contoare_stari();
        l.filtreaza_stare(Some(Stare::Modificat));
        assert_eq!(l.vizibile().len(), 2);
        assert_eq!(
            l.contoare_stari(),
            inainte,
            "filtrul nu are voie sa-si ascunda propriile categorii"
        );
        assert_eq!(
            inainte,
            vec![
                (Stare::Modificat, 2),
                (Stare::DoarSursa, 1),
                (Stare::DoarBaza, 1),
                (Stare::Identic, 1),
                (Stare::Criptat, 1)
            ]
        );
    }

    #[test]
    fn tipurile_numara_doar_ce_difera_in_ordinea_din_machete() {
        let l = lista_proba();
        assert_eq!(
            l.contoare_tipuri(),
            vec![
                ("P".to_string(), 2),
                ("FN".to_string(), 1),
                ("V".to_string(), 1)
            ],
            "obiectul identic si cel criptat nu intra"
        );
    }

    #[test]
    fn progresul_revizuirii_numara_doar_obiectele_care_difera() {
        let mut l = lista_proba();
        assert_eq!(l.progres_revizuire(), (0, 4));
        let p = l
            .vizibile()
            .iter()
            .position(|&i| l.obiecte()[i].nume == "usp_Stoc")
            .unwrap();
        l.comuta_revizuit(p);
        assert_eq!(l.progres_revizuire(), (1, 4));
        l.comuta_revizuit(p);
        assert_eq!(l.progres_revizuire(), (0, 4));
    }

    #[test]
    fn selectia_ramane_pe_acelasi_obiect_cand_se_schimba_filtrul() {
        let mut l = lista_proba();
        let p = l
            .vizibile()
            .iter()
            .position(|&i| l.obiecte()[i].nume == "vw_Facturi")
            .unwrap();
        l.selecteaza(p);
        assert_eq!(l.obiect_selectat().unwrap().nume, "vw_Facturi");
        l.filtreaza_stare(Some(Stare::Modificat));
        assert_eq!(
            l.obiect_selectat().unwrap().nume,
            "vw_Facturi",
            "obiectul e tot vizibil, deci selectia nu se misca"
        );
        // cand iese din filtru, selectia cade pe primul rand, nu ramane in aer
        l.filtreaza_stare(Some(Stare::DoarSursa));
        assert_eq!(l.obiect_selectat().unwrap().nume, "usp_Import");
    }

    #[test]
    fn cautarea_gaseste_dupa_schema_si_nume_indiferent_de_majuscule() {
        let mut l = lista_proba();
        l.cauta("USP_");
        let nume: Vec<&str> = l
            .vizibile()
            .iter()
            .map(|&i| l.obiecte()[i].nume.as_str())
            .collect();
        assert_eq!(nume, ["usp_Import", "usp_Stoc", "usp_Vechi"]);
        l.cauta("dbo.fn");
        assert_eq!(l.vizibile().len(), 1);
        l.cauta("");
        assert_eq!(l.vizibile().len(), 6);
    }

    #[test]
    fn sortarea_pe_linii_pune_primele_obiectele_cu_cele_mai_multe_linii_mutate() {
        let mut l = lista_proba();
        l.filtreaza_stare(Some(Stare::Modificat));
        for p in 0..l.vizibile().len() {
            let n = l.obiect(p).unwrap().nume.clone();
            let plus = if n == "vw_Facturi" { 40 } else { 3 };
            l.pune_detalii(
                p,
                Detalii {
                    difera_in: "cod".into(),
                    linii_plus: plus,
                    linii_minus: 1,
                },
            );
        }
        l.sorteaza(Ordine::Linii);
        assert_eq!(l.obiect(0).unwrap().nume, "vw_Facturi");
        l.sorteaza(Ordine::Nume);
        assert_eq!(l.obiect(0).unwrap().nume, "usp_Stoc");
    }

    #[test]
    fn se_cer_detalii_doar_pentru_randurile_de_pe_ecran_si_doar_o_data() {
        let mut l = lista_proba();
        l.filtreaza_stare(Some(Stare::Modificat));
        assert_eq!(l.fara_detalii(0, 10), vec![0, 1]);
        l.pune_detalii(0, Detalii::default());
        assert_eq!(l.fara_detalii(0, 10), vec![1]);
        // un interval care depaseste lista nu da panica
        assert!(l.fara_detalii(5, 500).is_empty());
    }

    #[test]
    fn rezumatul_vorbeste_despre_ce_se_vede_acum() {
        let mut l = lista_proba();
        l.filtreaza_stare(Some(Stare::Modificat));
        assert_eq!(l.rezumat_vizibile(), "2 nerevizuite din 2");
        l.comuta_revizuit(0);
        assert_eq!(l.rezumat_vizibile(), "1 nerevizuite din 2");
    }

    #[test]
    fn navigarea_cu_tastatura_nu_iese_din_lista() {
        let mut l = lista_proba();
        l.muta_selectia(-1);
        assert_eq!(l.selectat(), Some(0));
        l.muta_selectia(100);
        assert_eq!(l.selectat(), Some(l.vizibile().len() - 1));
        l.cauta("nimic-nu-se-potriveste");
        l.muta_selectia(1);
        assert_eq!(l.selectat(), None, "lista goala nu are selectie");
    }
}
