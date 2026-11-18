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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ordine {
    /// Alfabetic pe `schema.nume`.
    #[default]
    Nume,
    /// Cele mai multe linii mutate, primele („Linii ↓" din macheta).
    LiniiDesc,
}

#[derive(Debug, Clone, Default)]
pub struct Filtru {
    pub stare: Option<Stare>,
    /// Codul tipului (`P`, `V`...).
    pub tip: Option<String>,
    pub cautare: String,
}

/// Lista, cu filtrul si selectia ei.
#[derive(Debug, Clone)]
pub struct Lista {
    obiecte: Vec<Obiect>,
    filtru: Filtru,
    ordine: Ordine,
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

    pub fn sorteaza(&mut self, ordine: Ordine) {
        self.ordine = ordine;
        self.recalculeaza();
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
            })
            .map(|(i, _)| i)
            .collect();
        match self.ordine {
            Ordine::Nume => v.sort_by_key(|&i| self.obiecte[i].id()),
            Ordine::LiniiDesc => {
                let linii = |i: usize| -> usize {
                    self.obiecte[i]
                        .detalii
                        .as_ref()
                        .map_or(0, |d| d.linii_plus + d.linii_minus)
                };
                v.sort_by(|&a, &b| {
                    linii(b)
                        .cmp(&linii(a))
                        .then_with(|| self.obiecte[a].id().cmp(&self.obiecte[b].id()))
                });
            }
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
        l.sorteaza(Ordine::LiniiDesc);
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
