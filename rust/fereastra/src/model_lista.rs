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
            Stare::DoarBaza => "E",
            Stare::Identic => "=",
            Stare::Criptat => "#",
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

/// Impactul estimat al deploy-ului (`diff-obiecte::impact::estimeaza`), in
/// ordinea gravitatii. Fereastra nu depinde de module, deci nivelul se
/// copiaza aici; estimarea ramane una singura, acolo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Impact {
    Scazut,
    Mediu,
    Ridicat,
}

impl Impact {
    /// Ordinea din filtru: cele grave intai.
    pub const TOATE: [Impact; 3] = [Impact::Ridicat, Impact::Mediu, Impact::Scazut];
}

/// O perioada pentru filtrele pe data (etapa 27), socotita inapoi de la azi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Perioada {
    Azi,
    Zile7,
    Zile30,
}

impl Perioada {
    pub const TOATE: [Perioada; 3] = [Perioada::Azi, Perioada::Zile7, Perioada::Zile30];

    /// Cate zile inapoi, in afara de azi.
    fn zile(self) -> u32 {
        match self {
            Perioada::Azi => 0,
            Perioada::Zile7 => 7,
            Perioada::Zile30 => 30,
        }
    }

    /// Prima zi din perioada, ca `AAAA-LL-ZZ`, de la `azi` (`AAAA-LL-ZZ...`).
    /// `None` daca `azi` nu e o data.
    pub fn de_la(self, azi: &str) -> Option<String> {
        zile_inainte(azi, self.zile())
    }
}

/// `AAAA-LL-ZZ` cu `n` zile inaintea lui `azi`. Fara biblioteca de date: zile
/// de la 1970 si inapoi (algoritmul „days from civil").
pub fn zile_inainte(azi: &str, n: u32) -> Option<String> {
    let parte = |a: usize, b: usize| azi.get(a..b)?.parse::<i64>().ok();
    let (y, m, d) = (parte(0, 4)?, parte(5, 7)?, parte(8, 10)?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    // din data civila in zile
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let zi = era * 146097 + doe - 719468 - i64::from(n);
    // si inapoi
    let z = zi + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    Some(format!("{y:04}-{m:02}-{d:02}"))
}

/// Ce ar face deploy-ul cu obiectul (etapa 27): CREATE ce e doar in Martor,
/// ALTER ce difera, DROP ce e doar in Modificata (deploy-ul nu sterge nimic;
/// e doar un filtru, decizia din 2026-11-19).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operatie {
    Create,
    Alter,
    Drop,
}

impl Operatie {
    pub const TOATE: [Operatie; 3] = [Operatie::Create, Operatie::Alter, Operatie::Drop];

    pub fn eticheta(self) -> &'static str {
        match self {
            Operatie::Create => "CREATE",
            Operatie::Alter => "ALTER",
            Operatie::Drop => "DROP",
        }
    }
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
    /// Baza comparata in care s-a gasit obiectul. Cand referinta se compara cu
    /// mai multe baze, acelasi `schema.nume` apare o data pentru fiecare.
    pub baza: String,
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
    /// Cate obiecte din Modificata il folosesc; `None` cand nu s-a calculat.
    pub folosit_de: Option<usize>,
    /// Impactul estimat al deploy-ului; `None` la obiectele identice.
    pub impact: Option<Impact>,
    /// Ultima modificare a obiectului din baza comparata (`AAAA-LL-ZZ
    /// HH:MM:SS`); pentru „doar in Martor", a celui din martor. `None` cand
    /// sursa nu o stie (instantaneu vechi, cod lipit).
    pub modificat: Option<String>,
    /// Data crearii, din aceeasi parte ca `modificat` (etapa 27).
    pub creat: Option<String>,
}

impl Obiect {
    /// Ce ar face deploy-ul cu el (etapa 27); `None` la identic si criptat.
    pub fn operatie(&self) -> Option<Operatie> {
        match self.stare {
            Stare::DoarSursa => Some(Operatie::Create),
            Stare::Modificat => Some(Operatie::Alter),
            Stare::DoarBaza => Some(Operatie::Drop),
            Stare::Identic | Stare::Criptat => None,
        }
    }

    pub fn id(&self) -> String {
        format!("{}.{}", self.schema, self.nume)
    }

    /// Cheia care chiar e unica: acelasi obiect din doua baze sunt doua
    /// randuri. `id()` singur nu ajunge de cand se compara mai multe baze.
    pub fn cheie(&self) -> (String, String) {
        (self.baza.clone(), self.id())
    }

    /// Se poate deploia: are cod in sursa care se poate citi. Un obiect doar
    /// in baza n-are ce scrie, unul criptat n-are cod.
    pub fn se_poate_deploia(&self) -> bool {
        // tabelele doar se compara (etapa 28)
        matches!(self.stare, Stare::Modificat | Stare::DoarSursa)
            && !self.tip.trim().eq_ignore_ascii_case("U")
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
    /// Baza comparata.
    Baza,
    /// Similaritatea; implicit, cele mai diferite primele.
    Scor,
    DiferaIn,
    /// Revizuit / omis / nerevizuit.
    Revizuire,
    Deploy,
    /// Cate obiecte il folosesc; implicit, cele mai folosite primele.
    FolositDe,
    /// Impactul; implicit, cele mai grave primele.
    Impact,
    /// Ultima modificare; implicit, cele mai recente primele (etapa 25).
    Modificat,
}

impl Ordine {
    /// Sensul cu care incepe o coloana: liniile, cine-l foloseste si
    /// impactul, descrescator (ce conteaza mai mult, intai); restul,
    /// crescator.
    fn implicit_descrescator(self) -> bool {
        matches!(
            self,
            Ordine::Linii | Ordine::FolositDe | Ordine::Impact | Ordine::Modificat
        )
    }
}

#[derive(Debug, Clone)]
pub struct Filtru {
    pub stare: Option<Stare>,
    /// Numele bazei comparate; `None` = toate.
    pub baza: Option<String>,
    /// Codul tipului (`P`, `V`...).
    pub tip: Option<String>,
    pub cautare: String,
    pub revizuire: Option<FiltruRevizuire>,
    /// Intervalul de similaritate, in procente. Cat timp e 0-100, nu se
    /// filtreaza nimic; altfel, obiectele fara scor (doar intr-o parte,
    /// criptate) nu se mai vad.
    pub scor_min: f64,
    pub scor_max: f64,
    /// Doar obiectele cu impactul asta; `None` = toate.
    pub impact: Option<Impact>,
    /// Create, respectiv modificate in perioada, cu prima ei zi (etapa 27).
    pub creat: Option<(Perioada, String)>,
    pub modificat: Option<(Perioada, String)>,
    /// Doar ce ar face deploy-ul asa.
    pub operatie: Option<Operatie>,
}

impl Default for Filtru {
    fn default() -> Self {
        Filtru {
            stare: None,
            baza: None,
            tip: None,
            cautare: String::new(),
            revizuire: None,
            scor_min: 0.0,
            scor_max: 100.0,
            impact: None,
            creat: None,
            modificat: None,
            operatie: None,
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

    /// Cum porneste lista dupa o comparare noua (Mihai, 2026-11-25): doar
    /// obiectele modificate, cele mai diferite primele (similaritatea
    /// crescator). Restul filtrelor raman deschise; o recomparare isi pastreaza
    /// vederea (`preia_vederea`).
    pub fn vederea_implicita(&mut self) {
        self.filtru = Filtru {
            stare: Some(Stare::Modificat),
            ..Filtru::default()
        };
        self.ordine = Ordine::Scor;
        self.descrescator = false;
        self.recalculeaza();
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

    /// Selecteaza obiectul cu cheia data (tabul activat in editor). Daca
    /// filtrul il ascunde, filtrul se goleste (ordinea ramane): obiectul
    /// deschis trebuie sa fie mereu si selectia listei. `false` daca obiectul
    /// nu exista deloc.
    pub fn selecteaza_cheie(&mut self, c: &(String, String)) -> bool {
        let gaseste = |l: &Lista| l.vizibile.iter().position(|&i| &l.obiecte[i].cheie() == c);
        if let Some(p) = gaseste(self) {
            self.selectat = Some(p);
            return true;
        }
        if !self.obiecte.iter().any(|o| &o.cheie() == c) {
            return false;
        }
        self.filtru = Filtru::default();
        self.recalculeaza();
        match gaseste(self) {
            Some(p) => {
                self.selectat = Some(p);
                true
            }
            None => false,
        }
    }

    /// Filtrul, ordinea si obiectul selectat ale altei liste: dupa o
    /// recomparare lista arata ca inainte, doar cu datele noi (jurnalul din
    /// 2026-11-19: filtrele se pierdeau la Recompara). Selectia ramane doar
    /// daca obiectul mai exista si e vizibil.
    pub fn preia_vederea(&mut self, alta: &Lista) {
        // „in plan" ramane pe obiectele care inca se pot deploia: altfel o
        // recomparare golea planul facut pana atunci (F-A8; si v3.4 il pastra)
        let in_plan: std::collections::BTreeSet<(String, String)> =
            alta.in_plan().into_iter().collect();
        for o in self.obiecte.iter_mut() {
            if o.se_poate_deploia() && in_plan.contains(&o.cheie()) {
                o.in_plan = true;
            }
        }
        self.filtru = alta.filtru.clone();
        self.ordine = alta.ordine;
        self.descrescator = alta.descrescator;
        // „In plan" e alegerea omului, nu a compararii: o recomparare nu-l
        // pierde (C7 din `cercetare/audit-qa-2026-11-22.md`). Garda deploy-ului
        // se reface oricum, pe compararea noua. Obiectele care intre timp au
        // devenit identice sau au disparut din martor nu se pun inapoi: filtrul
        // e mai sus, pe `se_poate_deploia`.
        self.recalculeaza();
        if let Some(c) = alta.obiect_selectat().map(Obiect::cheie) {
            if let Some(p) = self
                .vizibile
                .iter()
                .position(|&i| self.obiecte[i].cheie() == c)
            {
                self.selectat = Some(p);
            }
        }
    }

    /// Cheile randurilor vizibile, in ordinea afisata (pentru `j` / `k`).
    pub fn chei_vizibile(&self) -> Vec<(String, String)> {
        self.vizibile
            .iter()
            .map(|&i| self.obiecte[i].cheie())
            .collect()
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

    pub fn filtreaza_baza(&mut self, baza: Option<String>) {
        self.filtru.baza = baza;
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

    /// Doar obiectele cu impactul `i` (`None` = toate).
    pub fn filtreaza_impact(&mut self, i: Option<Impact>) {
        self.filtru.impact = i;
        self.recalculeaza();
    }

    /// Doar obiectele create in perioada, de la `azi` (`AAAA-LL-ZZ...`).
    /// Fara data (instantaneu vechi) nu trec.
    pub fn filtreaza_creat(&mut self, p: Option<Perioada>, azi: &str) {
        self.filtru.creat = p.and_then(|p| Some((p, p.de_la(azi)?)));
        self.recalculeaza();
    }

    /// La fel, pe data ultimei modificari.
    pub fn filtreaza_modificat(&mut self, p: Option<Perioada>, azi: &str) {
        self.filtru.modificat = p.and_then(|p| Some((p, p.de_la(azi)?)));
        self.recalculeaza();
    }

    pub fn filtreaza_operatie(&mut self, o: Option<Operatie>) {
        self.filtru.operatie = o;
        self.recalculeaza();
    }

    /// Starea de deploy a obiectului `schema.nume` dintr-o anume baza.
    pub fn pune_deploy(&mut self, baza: &str, id: &str, d: Deploy) {
        for o in self
            .obiecte
            .iter_mut()
            .filter(|o| o.baza == baza && o.id() == id)
        {
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
    /// Starea de revizuire si de deploy a unui obiect, dupa cheie (dupa o
    /// unire cu ce a scris cli in sesiune).
    pub fn pune_stare_cheie(
        &mut self,
        cheie: &(String, String),
        revizuit: bool,
        omis: bool,
        deploy: Deploy,
    ) {
        for o in self.obiecte.iter_mut().filter(|o| &o.cheie() == cheie) {
            o.revizuit = revizuit && !omis;
            o.omis = omis;
            o.deploy = deploy;
            if omis {
                o.in_plan = false;
            }
        }
        self.recalculeaza();
    }

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

    /// Scoate din plan obiectul dat (baza si `schema.nume`), vizibil sau nu.
    pub fn scoate_din_plan(&mut self, baza: &str, id: &str) {
        for o in self
            .obiecte
            .iter_mut()
            .filter(|o| o.baza == baza && o.id() == id)
        {
            o.in_plan = false;
        }
    }

    /// Obiectele din plan, `(baza, schema.nume)`, in ordinea din lista
    /// completa.
    pub fn in_plan(&self) -> Vec<(String, String)> {
        self.obiecte
            .iter()
            .filter(|o| o.in_plan)
            .map(Obiect::cheie)
            .collect()
    }

    /// Pune detaliile calculate pentru un obiect vizibil.
    pub fn pune_detalii(&mut self, pozitie: usize, d: Detalii) {
        if let Some(&i) = self.vizibile.get(pozitie) {
            self.obiecte[i].detalii = Some(d);
        }
    }

    /// Detaliile obiectului cu cheia data, oriunde ar fi in lista (si filtrat).
    /// Pentru calculul de pe alt fir: rezultatele vin dupa cheie, nu dupa
    /// pozitie, fiindca intre timp filtrul se poate schimba.
    pub fn pune_detalii_cheie(&mut self, cheie: &(String, String), d: Detalii) {
        self.pune_detalii_multe(vec![(cheie.clone(), d)]);
    }

    /// Un lot de detalii venite de pe fir: cautate printr-un index, nu cu
    /// `find` pe toata lista pentru fiecare. Daca ordinea depinde de ele
    /// („Linii", „Difera in"), lista se resorteaza, altfel sortarea minte
    /// pana la urmatorul clic pe antet (F-A6).
    pub fn pune_detalii_multe(&mut self, lot: Vec<((String, String), Detalii)>) {
        if lot.is_empty() {
            return;
        }
        let index: std::collections::HashMap<(String, String), usize> = self
            .obiecte
            .iter()
            .enumerate()
            .map(|(i, o)| (o.cheie(), i))
            .collect();
        for (cheie, d) in lot {
            if let Some(&i) = index.get(&cheie) {
                self.obiecte[i].detalii = Some(d);
            }
        }
        // resortarea NU se face aici: costa cat lista si ar rula la fiecare
        // lot. O face `detalii_sosite`, o data pe tura de detalii (F-A6).
    }

    /// Au sosit detalii (de pe firul lor): daca lista e sortata dupa ele
    /// („Linii", „Difera in"), ordinea se reface, cu selectia pe acelasi
    /// obiect. Altfel randurile sosite tarziu ramaneau pe locul unui 0 si
    /// „cele mai schimbate intai" mintea (F-A6). Se cheama o data pe tura de
    /// detalii, nu pe fiecare: resortarea costa cat lista.
    pub fn detalii_sosite(&mut self) {
        if matches!(self.ordine, Ordine::Linii | Ordine::DiferaIn) {
            self.recalculeaza();
        }
    }

    /// Cheile obiectelor modificate fara detalii: intai cele vizibile, in
    /// ordinea afisata, apoi restul.
    pub fn chei_fara_detalii(&self) -> Vec<(String, String)> {
        let lipsa = |i: &usize| {
            let o = &self.obiecte[*i];
            o.detalii.is_none() && o.stare == Stare::Modificat
        };
        let mut v: Vec<usize> = self.vizibile.iter().copied().filter(lipsa).collect();
        let vazute: std::collections::BTreeSet<usize> = v.iter().copied().collect();
        v.extend((0..self.obiecte.len()).filter(|i| !vazute.contains(i) && lipsa(i)));
        v.into_iter().map(|i| self.obiecte[i].cheie()).collect()
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

    /// Bazele comparate, cu cate obiecte care difera are fiecare, in ordinea
    /// in care apar.
    pub fn contoare_baze(&self) -> Vec<(String, usize)> {
        let mut rez: Vec<(String, usize)> = Vec::new();
        for o in self.obiecte.iter().filter(|o| o.stare.difera()) {
            match rez.iter_mut().find(|(b, _)| *b == o.baza) {
                Some((_, n)) => *n += 1,
                None => rez.push((o.baza.clone(), 1)),
            }
        }
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

    /// Cate obiecte nerevizuite din cate se vad acum. Cuvintele le pune
    /// fereastra, din `Texte`: aici stau doar cifrele.
    pub fn rezumat_vizibile(&self) -> (usize, usize) {
        let nerevizuite = self
            .vizibile
            .iter()
            .filter(|&&i| self.obiecte[i].stare.difera() && !self.obiecte[i].revizuit)
            .count();
        (nerevizuite, self.vizibile.len())
    }

    fn recalculeaza(&mut self) {
        // obiectul selectat ramane selectat daca mai e vizibil: altfel
        // previzualizarea de dedesubt ar sari pe alt obiect la fiecare filtru
        let id_selectat = self.obiect_selectat().map(Obiect::cheie);
        let cautare = self.filtru.cautare.to_lowercase();
        let mut v: Vec<usize> = self
            .obiecte
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                self.filtru.stare.is_none_or(|s| o.stare == s)
                    && self.filtru.baza.as_deref().is_none_or(|b| o.baza == b)
                    && self.filtru.tip.as_deref().is_none_or(|t| o.tip == t)
                    && (cautare.is_empty()
                        || o.id().to_lowercase().contains(&cautare)
                        || o.baza.to_lowercase().contains(&cautare))
                    && self.filtru.revizuire.is_none_or(|f| f.potriveste(o))
                    && (!self.filtru.scor_activ()
                        || o.scor.is_some_and(|x| {
                            x >= self.filtru.scor_min && x <= self.filtru.scor_max
                        }))
                    && self.filtru.impact.is_none_or(|i| o.impact == Some(i))
                    && self.filtru.creat.as_ref().is_none_or(|(_, de_la)| {
                        o.creat.as_deref().is_some_and(|c| c >= de_la.as_str())
                    })
                    && self.filtru.modificat.as_ref().is_none_or(|(_, de_la)| {
                        o.modificat.as_deref().is_some_and(|c| c >= de_la.as_str())
                    })
                    && self
                        .filtru
                        .operatie
                        .is_none_or(|op| o.operatie() == Some(op))
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
                Ordine::Baza => o[a].baza.cmp(&o[b].baza),
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
                Ordine::FolositDe => o[a].folosit_de.cmp(&o[b].folosit_de),
                Ordine::Impact => o[a].impact.cmp(&o[b].impact),
                Ordine::Modificat => o[a].modificat.cmp(&o[b].modificat),
            };
            // numele (cu baza) departajeaza mereu crescator, ca ordinea sa fie
            // stabila; `id()` singur nu mai e unic peste mai multe baze
            let c = if self.descrescator { c.reverse() } else { c };
            c.then_with(|| o[a].cheie().cmp(&o[b].cheie()))
        });
        if ordine == Ordine::Nume && self.descrescator {
            v.reverse();
        }
        self.vizibile = v;
        self.selectat = match id_selectat {
            Some(id) => self
                .vizibile
                .iter()
                .position(|&i| self.obiecte[i].cheie() == id)
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
    /// 2026-11-25: lista porneste pe modificate, cea mai mica similaritate
    /// intai; identicele, doar-intr-o-parte si criptatele nu se vad.
    #[test]
    fn vederea_implicita_arata_modificatele_cele_mai_diferite_intai() {
        let cu_scor = |n: &str, stare: Stare, scor: Option<f64>| {
            let mut o = obiect(n, "P", stare);
            o.scor = scor;
            o
        };
        let mut l = Lista::noua(vec![
            cu_scor("a", Stare::Modificat, Some(99.0)),
            cu_scor("b", Stare::Identic, Some(100.0)),
            cu_scor("c", Stare::Modificat, Some(42.0)),
            cu_scor("d", Stare::DoarSursa, None),
            cu_scor("e", Stare::Modificat, Some(87.5)),
        ]);
        assert_eq!(l.vizibile().len(), 5, "fara vedere, totul");
        l.vederea_implicita();
        let nume: Vec<&str> = l
            .vizibile()
            .iter()
            .map(|&i| l.obiecte()[i].nume.as_str())
            .collect();
        assert_eq!(nume, ["c", "e", "a"]);
        assert_eq!(l.ordine(), Ordine::Scor);
        assert!(!l.descrescator());
        assert_eq!(l.filtru().stare, Some(Stare::Modificat));
    }

    use super::*;

    fn obiect(nume: &str, tip: &str, stare: Stare) -> Obiect {
        Obiect {
            schema: "dbo".into(),
            nume: nume.into(),
            baza: "Client1".into(),
            tip: tip.into(),
            stare,
            detalii: None,
            revizuit: false,
            in_plan: false,
            omis: false,
            scor: None,
            deploy: Default::default(),
            // valori diferite intre obiecte, ca sortarea sa aiba ce face
            folosit_de: Some(nume.len() % 4),
            impact: match stare {
                Stare::Modificat => Some(Impact::Ridicat),
                Stare::DoarSursa => Some(Impact::Mediu),
                Stare::Identic => None,
                _ => Some(Impact::Scazut),
            },
            modificat: None,
            creat: None,
        }
    }

    fn cheie(nume: &str) -> (String, String) {
        ("Client1".to_string(), format!("dbo.{nume}"))
    }

    /// F-A6: sortata pe „Linii", lista se reface cand vin detaliile.
    #[test]
    fn detaliile_venite_resorteaza_pe_linii() {
        let mut l = Lista::noua(vec![
            obiect("pA", "P", Stare::Modificat),
            obiect("pB", "P", Stare::Modificat),
        ]);
        l.sorteaza(Ordine::Linii);
        let cu = |n: usize| Detalii {
            linii_plus: n,
            ..Detalii::default()
        };
        let a = l.obiecte()[0].cheie();
        let b = l.obiecte()[1].cheie();
        l.pune_detalii_multe(vec![(a.clone(), cu(1)), (b.clone(), cu(50))]);
        assert_ne!(a, b);
        // resortarea vine o data pe tura, din `detalii_sosite`, nu la fiecare lot
        l.detalii_sosite();
        let primul = l.obiect(0).unwrap().cheie();
        assert_eq!(primul, b, "cele mai multe linii primele");
    }

    #[test]
    fn detaliile_se_pun_dupa_cheie_si_lipsesc_intai_cele_vizibile() {
        let mut l = lista_proba();
        l.filtreaza_tip(Some("V".into()));
        let lipsa = l.chei_fara_detalii();
        // doar modificatele: usp_Stoc si vw_Facturi; vw_Facturi e vizibil
        assert_eq!(lipsa, [cheie("vw_Facturi"), cheie("usp_Stoc")]);
        let d = Detalii {
            difera_in: "cod".into(),
            linii_plus: 2,
            linii_minus: 1,
        };
        // usp_Stoc e ascuns de filtru, dar primeste detaliile
        l.pune_detalii_cheie(&cheie("usp_Stoc"), d);
        assert_eq!(l.chei_fara_detalii(), [cheie("vw_Facturi")]);
    }

    #[test]
    fn recompararea_pastreaza_filtrul_ordinea_si_selectia() {
        let mut veche = lista_proba();
        veche.filtreaza_tip(Some("P".into()));
        veche.sorteaza(Ordine::Tip);
        assert!(veche.selecteaza_cheie(&cheie("usp_Import")));
        // datele noi: un obiect a disparut, unul nou a aparut
        let mut noua = Lista::noua(vec![
            obiect("usp_Stoc", "P", Stare::Modificat),
            obiect("usp_Import", "P", Stare::Modificat),
            obiect("usp_Nou", "P", Stare::DoarSursa),
            obiect("vw_Facturi", "V", Stare::Modificat),
        ]);
        noua.preia_vederea(&veche);
        assert_eq!(noua.filtru().tip.as_deref(), Some("P"));
        assert_eq!(noua.ordine(), Ordine::Tip);
        assert_eq!(noua.vizibile().len(), 3, "vw_Facturi ramane ascuns");
        assert_eq!(
            noua.obiect_selectat().map(Obiect::cheie),
            Some(cheie("usp_Import"))
        );
        // obiectul selectat nu mai exista: selectia nu se inventeaza
        let mut fara = Lista::noua(vec![obiect("usp_Stoc", "P", Stare::Modificat)]);
        fara.preia_vederea(&veche);
        assert_ne!(
            fara.obiect_selectat().map(Obiect::cheie),
            Some(cheie("usp_Import"))
        );
    }

    #[test]
    fn zilele_inainte_trec_peste_luna_an_si_februarie() {
        assert_eq!(zile_inainte("2026-11-22", 0).as_deref(), Some("2026-11-22"));
        assert_eq!(
            zile_inainte("2026-11-22 10:32:15", 7).as_deref(),
            Some("2026-11-15")
        );
        assert_eq!(zile_inainte("2026-03-05", 7).as_deref(), Some("2026-02-26"));
        assert_eq!(
            zile_inainte("2024-03-01", 1).as_deref(),
            Some("2024-02-29"),
            "an bisect"
        );
        assert_eq!(
            zile_inainte("2026-01-10", 30).as_deref(),
            Some("2025-12-11")
        );
        assert_eq!(zile_inainte("nu e data", 7), None);
    }

    #[test]
    fn filtrele_pe_data_si_operatie() {
        let cu = |nume: &str, stare: Stare, creat: Option<&str>, modificat: Option<&str>| Obiect {
            creat: creat.map(str::to_string),
            modificat: modificat.map(str::to_string),
            ..obiect(nume, "P", stare)
        };
        let mut l = Lista::noua(vec![
            cu(
                "pNou",
                Stare::DoarSursa,
                Some("2026-11-21 09:00:00"),
                Some("2026-11-22 08:00:00"),
            ),
            cu(
                "pVechi",
                Stare::Modificat,
                Some("2020-01-01 00:00:00"),
                Some("2026-11-01 00:00:00"),
            ),
            cu(
                "pExtra",
                Stare::DoarBaza,
                Some("2026-11-20 00:00:00"),
                Some("2026-11-20 00:00:00"),
            ),
            cu("pFaraDate", Stare::Modificat, None, None),
            cu(
                "pLaFel",
                Stare::Identic,
                Some("2026-11-22 00:00:00"),
                Some("2026-11-22 00:00:00"),
            ),
        ]);
        let azi = "2026-11-22 12:00:00";
        let nume = |l: &Lista| -> Vec<String> {
            let mut v: Vec<String> = (0..l.vizibile().len())
                .map(|p| l.obiect(p).unwrap().nume.clone())
                .collect();
            v.sort();
            v
        };
        l.filtreaza_creat(Some(Perioada::Zile7), azi);
        assert_eq!(nume(&l), ["pExtra", "pLaFel", "pNou"], "fara data nu trece");
        l.filtreaza_creat(None, azi);
        l.filtreaza_modificat(Some(Perioada::Azi), azi);
        assert_eq!(nume(&l), ["pLaFel", "pNou"]);
        l.filtreaza_modificat(Some(Perioada::Zile30), azi);
        assert_eq!(nume(&l), ["pExtra", "pLaFel", "pNou", "pVechi"]);
        l.filtreaza_modificat(None, azi);
        l.filtreaza_operatie(Some(Operatie::Alter));
        assert_eq!(nume(&l), ["pFaraDate", "pVechi"]);
        l.filtreaza_operatie(Some(Operatie::Drop));
        assert_eq!(nume(&l), ["pExtra"]);
        // se combina
        l.filtreaza_operatie(Some(Operatie::Create));
        l.filtreaza_creat(Some(Perioada::Azi), azi);
        assert!(nume(&l).is_empty(), "pNou e creat ieri");
        // si raman la recomparare
        let mut noua = Lista::noua(vec![cu(
            "pNou",
            Stare::DoarSursa,
            Some("2026-11-22 01:00:00"),
            None,
        )]);
        noua.preia_vederea(&l);
        assert_eq!(noua.filtru().operatie, Some(Operatie::Create));
        assert_eq!(nume(&noua), ["pNou"]);
        assert_eq!(Operatie::Drop.eticheta(), "DROP");
    }

    #[test]
    fn detaliile_sosite_tarziu_resorteaza_lista_pe_linii() {
        let mut l = Lista::noua(
            (0..80)
                .map(|i| obiect(&format!("p{i:02}"), "P", Stare::Modificat))
                .collect(),
        );
        l.sorteaza(Ordine::Linii);
        let d = |n: usize| Detalii {
            difera_in: String::new(),
            linii_plus: n,
            linii_minus: 0,
        };
        // detaliile primelor 60 au venit, pe al 70-lea abia acum
        for i in 0..60 {
            l.pune_detalii_cheie(&cheie(&format!("p{i:02}")), d(1));
        }
        l.detalii_sosite();
        l.pune_detalii_cheie(&cheie("p70"), d(500));
        assert_ne!(l.obiect(0).map(Obiect::id), Some("dbo.p70".to_string()));
        l.detalii_sosite();
        assert_eq!(
            l.obiect(0).map(Obiect::id),
            Some("dbo.p70".to_string()),
            "cel mai schimbat e primul, chiar daca a venit ultimul"
        );
    }

    #[test]
    fn recompararea_pastreaza_planul_pe_ce_se_mai_poate_deploia() {
        let mut veche = Lista::noua(vec![
            obiect("usp_Stoc", "P", Stare::Modificat),
            obiect("usp_Import", "P", Stare::Modificat),
            obiect("vw_Facturi", "V", Stare::Modificat),
        ]);
        for p in 0..3 {
            assert!(veche.comuta_in_plan(p));
        }
        // usp_Import a devenit identic, vw_Facturi a disparut din martor
        let mut noua = Lista::noua(vec![
            obiect("usp_Stoc", "P", Stare::Modificat),
            obiect("usp_Import", "P", Stare::Identic),
            obiect("vw_Facturi", "V", Stare::DoarBaza),
        ]);
        noua.preia_vederea(&veche);
        assert_eq!(noua.in_plan(), [cheie("usp_Stoc")]);
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
        assert_eq!(l.in_plan(), [cheie("pMod"), cheie("pNou")]);
        l.scoate_din_plan("Client1", "dbo.pMod");
        assert_eq!(l.in_plan(), [cheie("pNou")]);
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
        assert_eq!(l.rezumat_vizibile(), (2, 2));
        l.comuta_revizuit(0);
        assert_eq!(l.rezumat_vizibile(), (1, 2));
    }

    /// Lista pentru filtrele noi: scor, revizuire, deploy.
    fn lista_stari() -> Lista {
        let mut o1 = obiect("pMic", "P", Stare::Modificat);
        o1.scor = Some(20.0);
        let mut o2 = obiect("pMediu", "P", Stare::Modificat);
        o2.scor = Some(60.0);
        o2.revizuit = true;
        let mut o3 = obiect("pMare", "P", Stare::Modificat);
        o3.scor = Some(95.0);
        o3.omis = true;
        let mut o4 = obiect("pNou", "P", Stare::DoarSursa);
        o4.in_plan = true;
        o4.deploy = Deploy::Deployat;
        let mut o5 = obiect("pVechi", "P", Stare::DoarBaza);
        o5.deploy = Deploy::VersiuneNoua;
        Lista::noua(vec![o1, o2, o3, o4, o5])
    }

    fn nume_vizibile(l: &Lista) -> Vec<String> {
        (0..l.vizibile().len())
            .map(|p| l.obiect(p).unwrap().nume.clone())
            .collect()
    }

    #[test]
    fn filtrul_pe_stare_de_revizuire_separa_cele_sase_cazuri() {
        let mut l = lista_stari();
        let cu = |l: &mut Lista, f: FiltruRevizuire| {
            l.filtreaza_revizuire(Some(f));
            nume_vizibile(l)
        };
        assert_eq!(
            cu(&mut l, FiltruRevizuire::Nerevizuite),
            ["pMic", "pNou", "pVechi"],
            "omisul nu e nerevizuit: e analizat, doar lasat deoparte"
        );
        assert_eq!(cu(&mut l, FiltruRevizuire::Revizuite), ["pMediu"]);
        assert_eq!(cu(&mut l, FiltruRevizuire::Omise), ["pMare"]);
        assert_eq!(cu(&mut l, FiltruRevizuire::InPlan), ["pNou"]);
        assert_eq!(cu(&mut l, FiltruRevizuire::Deployate), ["pNou"]);
        assert_eq!(
            cu(&mut l, FiltruRevizuire::Nedeployate),
            ["pMare", "pMediu", "pMic", "pVechi"],
            "versiunea noua nu e deployata: s-a schimbat de atunci"
        );
        l.filtreaza_revizuire(None);
        assert_eq!(l.vizibile().len(), 5);
    }

    #[test]
    fn filtrul_pe_scor_taie_capetele_si_ascunde_obiectele_fara_scor() {
        let mut l = lista_stari();
        l.filtreaza_scor(50.0, 100.0);
        assert_eq!(
            nume_vizibile(&l),
            ["pMare", "pMediu"],
            "obiectele fara scor ies cand filtrul e activ"
        );
        // capetele sunt incluse
        l.filtreaza_scor(20.0, 20.0);
        assert_eq!(nume_vizibile(&l), ["pMic"]);
        // un interval scris invers se indreapta, nu goleste lista
        l.filtreaza_scor(100.0, 60.0);
        assert_eq!(nume_vizibile(&l), ["pMare", "pMediu"]);
        // in afara 0-100 se taie, iar 0-100 inseamna „fara filtru"
        l.filtreaza_scor(-30.0, 500.0);
        assert_eq!(l.vizibile().len(), 5, "si cele fara scor revin");
    }

    /// Cheia dupa care ar trebui sa iasa sortata coloana, ca text comparabil
    /// (numerele cu zerouri in fata, ca sa se compare ca numere).
    fn cheia(o: &Obiect, coloana: Ordine) -> String {
        let linii = o
            .detalii
            .as_ref()
            .map_or(0, |d| d.linii_plus + d.linii_minus);
        match coloana {
            Ordine::Nume => o.id(),
            Ordine::Tip => o.tip.clone(),
            Ordine::Baza => o.baza.clone(),
            Ordine::Linii => format!("{linii:06}"),
            // fara scor: la capat, ca „necunoscut"
            Ordine::Scor => format!("{:09.3}", o.scor.unwrap_or(101.0)),
            Ordine::DiferaIn => o
                .detalii
                .as_ref()
                .map_or_else(String::new, |d| d.difera_in.clone()),
            Ordine::Revizuire => u8::from(o.omis)
                .max(if o.revizuit { 2 } else { 0 })
                .to_string(),
            Ordine::Deploy => match o.deploy {
                Deploy::Nu => u8::from(o.in_plan),
                Deploy::VersiuneNoua => 2,
                Deploy::Deployat => 3,
            }
            .to_string(),
            Ordine::FolositDe => format!("{:06}", o.folosit_de.unwrap_or(0)),
            Ordine::Impact => format!("{:?}", o.impact.map(|i| i as u8)),
            Ordine::Modificat => o.modificat.clone().unwrap_or_default(),
        }
    }

    const COLOANE: [Ordine; 11] = [
        Ordine::Nume,
        Ordine::Tip,
        Ordine::Baza,
        Ordine::Linii,
        Ordine::Scor,
        Ordine::DiferaIn,
        Ordine::Revizuire,
        Ordine::Deploy,
        Ordine::FolositDe,
        Ordine::Impact,
        Ordine::Modificat,
    ];

    /// Filtrul pe impact se combina cu celelalte; contoarele de stare raman
    /// pe tot (ca la celelalte filtre).
    #[test]
    fn filtrul_pe_impact_se_combina_cu_starea() {
        let mut l = lista_stari();
        let toate = l.vizibile().len();
        let numara = |l: &Lista, i: Impact| {
            l.vizibile()
                .iter()
                .filter(|&&k| l.obiecte()[k].impact == Some(i))
                .count()
        };
        let ridicate = numara(&l, Impact::Ridicat);
        assert!(ridicate > 0 && ridicate < toate, "date de test cu de toate");
        l.filtreaza_impact(Some(Impact::Ridicat));
        assert_eq!(l.vizibile().len(), ridicate);
        assert!(l
            .vizibile()
            .iter()
            .all(|&k| l.obiecte()[k].impact == Some(Impact::Ridicat)));
        l.filtreaza_stare(Some(Stare::DoarSursa));
        assert!(l.vizibile().is_empty(), "nicio DoarSursa nu e Ridicata");
        l.filtreaza_stare(None);
        l.filtreaza_impact(None);
        assert_eq!(l.vizibile().len(), toate);
    }

    #[test]
    fn fiecare_coloana_sorteaza_in_ambele_sensuri() {
        let mut l = lista_stari();
        for p in 0..l.vizibile().len() {
            let n = l.obiect(p).unwrap().nume.clone();
            l.pune_detalii(
                p,
                Detalii {
                    difera_in: format!("in-{n}"),
                    linii_plus: n.len(),
                    linii_minus: 0,
                },
            );
        }
        for coloana in COLOANE {
            // prima apasare: sensul implicit al coloanei; a doua: invers
            l.sorteaza(coloana);
            let sens = l.descrescator();
            verifica_ordinea(&l, coloana);
            l.sorteaza(coloana);
            assert_ne!(
                sens,
                l.descrescator(),
                "{coloana:?}: a doua apasare schimba sensul"
            );
            verifica_ordinea(&l, coloana);
            // alta coloana si inapoi: sensul reporneste de la implicit
            l.sorteaza(Ordine::Nume);
            l.sorteaza(coloana);
            assert_eq!(
                l.descrescator(),
                matches!(
                    coloana,
                    Ordine::Linii | Ordine::FolositDe | Ordine::Impact | Ordine::Modificat
                ),
                "{coloana:?}: sensul implicit, nu cel de data trecuta"
            );
        }
    }

    /// Cheile merg in sensul cerut, iar la chei egale numele ramane crescator
    /// — altfel randurile ar sari intre ele la fiecare resortare.
    fn verifica_ordinea(l: &Lista, coloana: Ordine) {
        let desc = l.descrescator();
        let randuri: Vec<&Obiect> = (0..l.vizibile().len())
            .map(|p| l.obiect(p).unwrap())
            .collect();
        for pereche in randuri.windows(2) {
            let (a, b) = (pereche[0], pereche[1]);
            let (ka, kb) = (cheia(a, coloana), cheia(b, coloana));
            if ka == kb {
                assert!(
                    a.id() < b.id(),
                    "{coloana:?}: la chei egale numele departajeaza crescator"
                );
            } else if desc {
                assert!(ka > kb, "{coloana:?} descrescator: {ka} dupa {kb}");
            } else {
                assert!(ka < kb, "{coloana:?} crescator: {ka} inaintea {kb}");
            }
        }
    }

    #[test]
    fn sortarea_pe_scor_pune_la_capat_obiectele_fara_scor() {
        let mut l = lista_stari();
        l.sorteaza(Ordine::Scor);
        assert_eq!(
            nume_vizibile(&l),
            ["pMic", "pMediu", "pMare", "pNou", "pVechi"],
            "cele fara scor stau dupa 100%, nu inaintea celor de 20%"
        );
    }

    #[test]
    fn sortarea_pe_deploy_urca_deployatele_si_lasa_jos_ce_nu_e_in_plan() {
        let mut l = lista_stari();
        l.sorteaza(Ordine::Deploy);
        assert_eq!(
            nume_vizibile(&l),
            ["pMare", "pMediu", "pMic", "pVechi", "pNou"],
            "nimic < in plan < versiune noua < deployat"
        );
    }

    #[test]
    fn in_plan_intra_doar_vizibilele_care_au_rost() {
        let mut l = lista_stari();
        // pNou e deja in plan si deployat; pMare e omis; pVechi n-are cod
        assert_eq!(l.pune_vizibilele_in_plan(false), 2);
        assert_eq!(
            l.in_plan(),
            [cheie("pMic"), cheie("pMediu"), cheie("pNou")],
            "ordinea e cea din lista completa, nu cea afisata"
        );
        // a doua oara nu mai are ce adauga
        assert_eq!(l.pune_vizibilele_in_plan(false), 0);
    }

    #[test]
    fn in_plan_sare_deployatele_pana_cand_se_cere_altfel() {
        let mut l = lista_stari();
        l.scoate_din_plan("Client1", "dbo.pNou");
        assert_eq!(
            l.pune_vizibilele_in_plan(false),
            2,
            "pNou e deployat si neschimbat, deci se sare"
        );
        assert!(!l.in_plan().contains(&cheie("pNou")));
        assert_eq!(l.pune_vizibilele_in_plan(true), 1, "acum intra si el");
        assert!(l.in_plan().contains(&cheie("pNou")));
    }

    #[test]
    fn in_plan_se_uita_doar_la_ce_e_filtrat() {
        let mut l = lista_stari();
        l.scoate_din_plan("Client1", "dbo.pNou");
        l.filtreaza_scor(0.0, 50.0);
        assert_eq!(nume_vizibile(&l), ["pMic"]);
        assert_eq!(l.pune_vizibilele_in_plan(true), 1);
        assert_eq!(
            l.in_plan(),
            [cheie("pMic")],
            "ce nu se vede nu intra in plan"
        );
    }

    #[test]
    fn progresul_de_deploy_numara_doar_ce_difera_si_e_neschimbat() {
        let mut l = lista_stari();
        assert_eq!(
            l.progres_deploy(),
            (1, 5),
            "versiunea noua nu se pune la bifate"
        );
        l.pune_deploy("Client1", "dbo.pMic", Deploy::Deployat);
        assert_eq!(l.progres_deploy(), (2, 5));
        l.pune_deploy("Client1", "dbo.pMic", Deploy::VersiuneNoua);
        assert_eq!(l.progres_deploy(), (1, 5));
    }

    #[test]
    fn un_obiect_omis_iese_din_plan_si_nu_mai_e_revizuit() {
        let mut l = lista_stari();
        let p = (0..l.vizibile().len())
            .find(|&p| l.obiect(p).unwrap().nume == "pNou")
            .unwrap();
        l.pune_revizuirea(p, true, true);
        let o = l.obiect(p).unwrap();
        assert!(o.omis && !o.revizuit && !o.in_plan);
        assert!(l.in_plan().is_empty());
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
