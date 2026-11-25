//! Taburile editorului: un tab de previzualizare si cel mult `MAX_FIXATE`
//! taburi fixate, plus parcurgerea cu `j` / `k` care sare peste obiectele
//! fixate. Regulile sunt in `etape/16-lista-si-taburi.md`.
//!
//! **Se strica astfel:** un obiect apare in doua taburi, al noualea tab
//! inchide unul cu text final editat, `j` / `k` intra intr-un obiect deja
//! fixat sau sar din lista, ori `Ctrl+W` activeaza alt tab decat vecinul din
//! stanga. Toate se decid aici, fara Slint, si se testeaza ca atare.

/// Identitatea unui obiect: (baza, `schema.nume`), ca `model_lista::Obiect::cheie`.
pub type Cheie = (String, String);

/// Cate taburi fixate pot fi deschise deodata.
pub const MAX_FIXATE: usize = 8;

/// Ce tab e activ: previzualizarea sau al catelea fixat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activ {
    Previzualizare,
    Fixat(usize),
}

/// De ce nu s-a putut fixa un obiect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refuz {
    /// Sunt deja `MAX_FIXATE` taburi fixate si toate au text final editat:
    /// nu se inchide nimic singur.
    ToateEditate,
}

/// Rezultatul unui pas `j` / `k`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pas {
    /// Obiectul urmator, de incarcat in previzualizare.
    Obiect(Cheie),
    /// Nu mai e niciun obiect nefixat in directia asta: se ramane pe loc.
    Capat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Fixat {
    cheie: Cheie,
    /// Cand a fost activ ultima data (un numarator, nu un ceas).
    folosit: u64,
}

/// Taburile, cum se tin in sesiune: ordinea, previzualizarea si tabul
/// activ (`None` = previzualizarea).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SesiuneTaburi {
    pub fixate: Vec<Cheie>,
    pub previzualizare: Option<Cheie>,
    pub activ: Option<usize>,
}

/// Starea taburilor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Taburi {
    previzualizare: Option<Cheie>,
    fixate: Vec<Fixat>,
    activ: Activ,
    tic: u64,
}

impl Default for Taburi {
    fn default() -> Self {
        Taburi::noi()
    }
}

impl Taburi {
    pub fn noi() -> Self {
        Taburi {
            previzualizare: None,
            fixate: Vec::new(),
            activ: Activ::Previzualizare,
            tic: 0,
        }
    }

    pub fn activ(&self) -> Activ {
        self.activ
    }

    /// Obiectul din tabul activ.
    pub fn cheie_activa(&self) -> Option<&Cheie> {
        match self.activ {
            Activ::Previzualizare => self.previzualizare.as_ref(),
            Activ::Fixat(i) => self.fixate.get(i).map(|f| &f.cheie),
        }
    }

    pub fn previzualizare(&self) -> Option<&Cheie> {
        self.previzualizare.as_ref()
    }

    /// Obiectele fixate, in ordinea taburilor.
    pub fn fixate(&self) -> Vec<&Cheie> {
        self.fixate.iter().map(|f| &f.cheie).collect()
    }

    pub fn e_fixat(&self, c: &Cheie) -> bool {
        self.fixate.iter().any(|f| &f.cheie == c)
    }

    fn atinge(&mut self) -> u64 {
        self.tic += 1;
        self.tic
    }

    /// Clic simplu in lista, sau obiectul ales de `j` / `k`: un obiect deja
    /// fixat isi activeaza tabul; altfel intra in previzualizare, in locul
    /// celui de acolo.
    pub fn deschide(&mut self, c: Cheie) {
        if let Some(i) = self.fixate.iter().position(|f| f.cheie == c) {
            self.activeaza(Activ::Fixat(i));
            return;
        }
        self.previzualizare = Some(c);
        self.activ = Activ::Previzualizare;
    }

    /// Activeaza un tab. Un indice care nu exista nu schimba nimic.
    pub fn activeaza(&mut self, a: Activ) {
        match a {
            Activ::Previzualizare if self.previzualizare.is_some() => self.activ = a,
            Activ::Fixat(i) if i < self.fixate.len() => {
                let t = self.atinge();
                self.fixate[i].folosit = t;
                self.activ = a;
            }
            _ => {}
        }
    }

    /// Fixeaza obiectul (dublu clic, „Deschide in tab nou", sau editarea
    /// textului final). Daca era in previzualizare, iese de acolo. La al
    /// noualea se inchide fixatul folosit cel mai de demult **si needitat**;
    /// `editat` spune care au text final. Intoarce obiectul inchis, daca a
    /// fost unul.
    pub fn fixeaza(
        &mut self,
        c: Cheie,
        editat: impl Fn(&Cheie) -> bool,
    ) -> Result<Option<Cheie>, Refuz> {
        if let Some(i) = self.fixate.iter().position(|f| f.cheie == c) {
            self.activeaza(Activ::Fixat(i));
            return Ok(None);
        }
        let mut inchis = None;
        if self.fixate.len() >= MAX_FIXATE {
            let Some(i) = self
                .fixate
                .iter()
                .enumerate()
                .filter(|(_, f)| !editat(&f.cheie))
                .min_by_key(|(_, f)| f.folosit)
                .map(|(i, _)| i)
            else {
                return Err(Refuz::ToateEditate);
            };
            inchis = Some(self.fixate.remove(i).cheie);
        }
        if self.previzualizare.as_ref() == Some(&c) {
            self.previzualizare = None;
        }
        let t = self.atinge();
        self.fixate.push(Fixat {
            cheie: c,
            folosit: t,
        });
        self.activ = Activ::Fixat(self.fixate.len() - 1);
        Ok(inchis)
    }

    /// Dublu clic pe tabul de previzualizare: obiectul din el se fixeaza.
    pub fn fixeaza_previzualizarea(
        &mut self,
        editat: impl Fn(&Cheie) -> bool,
    ) -> Result<Option<Cheie>, Refuz> {
        match self.previzualizare.clone() {
            Some(c) => self.fixeaza(c, editat),
            None => Ok(None),
        }
    }

    /// Inchide tabul activ (`Ctrl+W`); activ devine vecinul din stanga.
    pub fn inchide_activ(&mut self) {
        self.inchide(self.activ);
    }

    /// Inchide un tab (X-ul de pe el). Daca era activ, activ devine vecinul
    /// din stanga: fixatul dinainte, sau previzualizarea pentru primul.
    pub fn inchide(&mut self, a: Activ) {
        match a {
            Activ::Previzualizare => {
                self.previzualizare = None;
                if self.activ == Activ::Previzualizare && !self.fixate.is_empty() {
                    self.activ = Activ::Fixat(self.fixate.len() - 1);
                }
            }
            Activ::Fixat(i) if i < self.fixate.len() => {
                self.fixate.remove(i);
                self.activ = match self.activ {
                    Activ::Fixat(j) if j == i => {
                        if i > 0 {
                            Activ::Fixat(i - 1)
                        } else if self.previzualizare.is_some() || self.fixate.is_empty() {
                            Activ::Previzualizare
                        } else {
                            Activ::Fixat(0)
                        }
                    }
                    Activ::Fixat(j) if j > i => Activ::Fixat(j - 1),
                    alt => alt,
                };
            }
            _ => {}
        }
    }

    /// Pasul `j` (`pas` = 1) sau `k` (-1) in `lista`, ordinea curenta a
    /// listei (filtru + sortare): de la obiectul activ, urmatorul care **nu e
    /// fixat**. Daca obiectul activ nu e in lista, se porneste de la capatul
    /// ei (inceputul pentru `j`, sfarsitul pentru `k`).
    pub fn urmatorul(&self, lista: &[Cheie], pas: i32) -> Pas {
        let n = lista.len() as i64;
        let start = self
            .cheie_activa()
            .and_then(|c| lista.iter().position(|x| x == c))
            .map(|p| p as i64);
        let pas = if pas >= 0 { 1 } else { -1 };
        let mut p = match start {
            Some(p) => p + pas,
            None if pas > 0 => 0,
            None => n - 1,
        };
        while (0..n).contains(&p) {
            let c = &lista[p as usize];
            if !self.e_fixat(c) {
                return Pas::Obiect(c.clone());
            }
            p += pas;
        }
        Pas::Capat
    }

    /// Dupa o recomparare: taburile obiectelor care nu mai exista se inchid.
    pub fn pastreaza(&mut self, exista: impl Fn(&Cheie) -> bool) {
        let activ = self.cheie_activa().cloned();
        if self.previzualizare.as_ref().is_some_and(|c| !exista(c)) {
            self.previzualizare = None;
        }
        self.fixate.retain(|f| exista(&f.cheie));
        self.activ = match activ.and_then(|c| self.fixate.iter().position(|f| f.cheie == c)) {
            Some(i) => Activ::Fixat(i),
            None if self.previzualizare.is_some() || self.fixate.is_empty() => {
                Activ::Previzualizare
            }
            None => Activ::Fixat(self.fixate.len() - 1),
        };
    }

    /// Fixatele, in ordine, pentru sesiune.
    pub fn de_salvat(&self) -> Vec<Cheie> {
        self.fixate.iter().map(|f| f.cheie.clone()).collect()
    }

    /// Tot ce trebuie ca taburile sa revina la fel (etapa 33): fixatele in
    /// ordine, obiectul din previzualizare si tabul activ.
    pub fn de_salvat_complet(&self) -> SesiuneTaburi {
        SesiuneTaburi {
            fixate: self.de_salvat(),
            previzualizare: self.previzualizare.clone(),
            activ: match self.activ {
                Activ::Previzualizare => None,
                Activ::Fixat(i) => Some(i),
            },
        }
    }

    /// Inversul lui [`Taburi::de_salvat_complet`]. Un tab activ care nu mai
    /// exista (sesiune stricata, obiect disparut) cade pe primul fixat.
    pub fn din_sesiune_complet(s: SesiuneTaburi) -> Self {
        let mut t = Taburi::din_sesiune(s.fixate);
        if let Some(p) = s.previzualizare.filter(|p| !t.e_fixat(p)) {
            t.previzualizare = Some(p);
        }
        match s.activ {
            Some(i) if i < t.fixate.len() => t.activeaza(Activ::Fixat(i)),
            None if t.previzualizare.is_some() => t.activ = Activ::Previzualizare,
            _ => {}
        }
        t
    }

    /// Taburile fixate din sesiune; cele peste `MAX_FIXATE` se ignora.
    pub fn din_sesiune(fixate: Vec<Cheie>) -> Self {
        let mut t = Taburi::noi();
        for c in fixate.into_iter().take(MAX_FIXATE) {
            if !t.e_fixat(&c) {
                let _ = t.fixeaza(c, |_| true);
            }
        }
        if !t.fixate.is_empty() {
            t.activ = Activ::Fixat(0);
        }
        t
    }
}

/// Numele taiat **pe mijloc**, ca sa incapa in `maxim` caractere: finalul
/// deosebeste obiectele (`TES…_LiniiGoale`), deci se pastreaza. Se numara
/// caractere, nu octeti: un nume cu diacritice nu se rupe in mijlocul unei
/// litere.
pub fn taie_pe_mijloc(nume: &str, maxim: usize) -> String {
    let c: Vec<char> = nume.chars().collect();
    if c.len() <= maxim {
        return nume.to_string();
    }
    if maxim == 0 {
        return String::new();
    }
    if maxim == 1 {
        return "\u{2026}".to_string();
    }
    let ramas = maxim - 1;
    let fata = ramas / 3;
    let spate = ramas - fata;
    let mut s: String = c[..fata].iter().collect();
    s.push('\u{2026}');
    s.extend(&c[c.len() - spate..]);
    s
}

#[cfg(test)]
mod teste {
    use super::*;

    fn k(n: &str) -> Cheie {
        ("B".into(), format!("dbo.{n}"))
    }

    fn lista(n: &[&str]) -> Vec<Cheie> {
        n.iter().map(|x| k(x)).collect()
    }

    /// Etapa 33, A: „Open in new tab" pe alt obiect nu atinge tabul deschis
    /// (nici previzualizarea, nici fixatele); pe un obiect deja deschis nu
    /// face al doilea tab, ci il activeaza pe al lui.
    #[test]
    fn tab_nou_nu_pierde_tabul_deschis_si_nu_dubleaza() {
        let mut t = Taburi::noi();
        t.deschide(k("a"));
        t.fixeaza(k("b"), |_| false).unwrap();
        assert_eq!(t.previzualizare(), Some(&k("a")), "a ramane deschis");
        assert_eq!(t.fixate(), vec![&k("b")]);
        assert_eq!(t.activ(), Activ::Fixat(0));
        t.fixeaza(k("c"), |_| false).unwrap();
        // b din nou: se activeaza tabul lui, nu apare al doilea
        t.fixeaza(k("b"), |_| false).unwrap();
        assert_eq!(t.fixate(), vec![&k("b"), &k("c")]);
        assert_eq!(t.activ(), Activ::Fixat(0));
        // a (din previzualizare) intr-un tab nou: se muta, nu se dubleaza
        t.fixeaza(k("a"), |_| false).unwrap();
        assert_eq!(t.previzualizare(), None);
        assert_eq!(t.fixate(), vec![&k("b"), &k("c"), &k("a")]);
    }

    /// Etapa 33, A: tabul activ, ordinea si previzualizarea revin din sesiune;
    /// o sesiune veche (doar fixatele) sau stricata nu cade.
    #[test]
    fn sesiunea_tine_ordinea_tabul_activ_si_previzualizarea() {
        let mut t = Taburi::noi();
        t.fixeaza(k("x"), |_| false).unwrap();
        t.fixeaza(k("y"), |_| false).unwrap();
        t.fixeaza(k("z"), |_| false).unwrap();
        t.deschide(k("p"));
        t.activeaza(Activ::Fixat(1));
        let s = t.de_salvat_complet();
        let r = Taburi::din_sesiune_complet(s.clone());
        assert_eq!(r.fixate(), vec![&k("x"), &k("y"), &k("z")]);
        assert_eq!(r.previzualizare(), Some(&k("p")));
        assert_eq!(r.activ(), Activ::Fixat(1));
        assert_eq!(r.cheie_activa(), Some(&k("y")));
        // activa previzualizarea
        t.activeaza(Activ::Previzualizare);
        let r = Taburi::din_sesiune_complet(t.de_salvat_complet());
        assert_eq!(r.cheie_activa(), Some(&k("p")));
        // sesiune veche: doar fixatele, activ primul
        let r = Taburi::din_sesiune_complet(SesiuneTaburi {
            fixate: lista(&["x", "y"]),
            ..Default::default()
        });
        assert_eq!(r.activ(), Activ::Fixat(0));
        // indice in afara: ramane pe primul; previzualizarea fixata deja se ignora
        let r = Taburi::din_sesiune_complet(SesiuneTaburi {
            fixate: lista(&["x"]),
            previzualizare: Some(k("x")),
            activ: Some(7),
        });
        assert_eq!(r.fixate(), vec![&k("x")]);
        assert_eq!(r.previzualizare(), None);
        assert_eq!(r.activ(), Activ::Fixat(0));
    }

    #[test]
    fn clic_simplu_inlocuieste_previzualizarea_iar_un_fixat_isi_activeaza_tabul() {
        let mut t = Taburi::noi();
        t.deschide(k("a"));
        t.deschide(k("b"));
        assert_eq!(
            t.previzualizare(),
            Some(&k("b")),
            "previzualizarea se inlocuieste"
        );
        t.fixeaza(k("c"), |_| false).unwrap();
        t.deschide(k("c"));
        assert_eq!(t.activ(), Activ::Fixat(0));
        assert_eq!(
            t.previzualizare(),
            Some(&k("b")),
            "c nu intra si in previzualizare"
        );
        assert_eq!(t.fixate().len(), 1, "un obiect sta intr-un singur tab");
    }

    #[test]
    fn fixarea_previzualizarii_o_scoate_de_acolo() {
        let mut t = Taburi::noi();
        t.deschide(k("a"));
        t.fixeaza_previzualizarea(|_| false).unwrap();
        assert_eq!(t.previzualizare(), None);
        assert_eq!(t.fixate(), [&k("a")]);
        assert_eq!(t.cheie_activa(), Some(&k("a")));
    }

    #[test]
    fn al_noualea_inchide_cel_mai_vechi_needitat() {
        let mut t = Taburi::noi();
        for i in 0..MAX_FIXATE {
            t.fixeaza(k(&format!("p{i}")), |_| false).unwrap();
        }
        // p0 e cel mai vechi, dar editat; p1 e urmatorul, dar folosit acum
        t.activeaza(Activ::Fixat(1));
        let editat = |c: &Cheie| c == &k("p0");
        let inchis = t.fixeaza(k("nou"), editat).unwrap();
        assert_eq!(
            inchis,
            Some(k("p2")),
            "cel mai de demult folosit si needitat"
        );
        assert_eq!(t.fixate().len(), MAX_FIXATE);
        assert_eq!(t.cheie_activa(), Some(&k("nou")));
    }

    #[test]
    fn al_noualea_cu_toate_editate_se_refuza_fara_sa_inchida_nimic() {
        let mut t = Taburi::noi();
        for i in 0..MAX_FIXATE {
            t.fixeaza(k(&format!("p{i}")), |_| false).unwrap();
        }
        let inainte = t.clone();
        assert_eq!(t.fixeaza(k("nou"), |_| true), Err(Refuz::ToateEditate));
        assert_eq!(t, inainte, "nimic schimbat");
    }

    #[test]
    fn j_si_k_sar_peste_obiectele_fixate() {
        let l = lista(&["a", "b", "c", "d", "e"]);
        let mut t = Taburi::noi();
        t.fixeaza(k("b"), |_| false).unwrap();
        t.fixeaza(k("c"), |_| false).unwrap();
        t.deschide(k("a"));
        assert_eq!(
            t.urmatorul(&l, 1),
            Pas::Obiect(k("d")),
            "b si c sunt fixate"
        );
        t.deschide(k("e"));
        assert_eq!(t.urmatorul(&l, -1), Pas::Obiect(k("d")));
        // de la un fixat, tot peste fixate
        t.deschide(k("b"));
        assert_eq!(t.urmatorul(&l, 1), Pas::Obiect(k("d")));
        assert_eq!(t.urmatorul(&l, -1), Pas::Obiect(k("a")));
    }

    #[test]
    fn j_cu_toate_urmatoarele_fixate_ramane_pe_loc() {
        let l = lista(&["a", "b", "c"]);
        let mut t = Taburi::noi();
        t.fixeaza(k("b"), |_| false).unwrap();
        t.fixeaza(k("c"), |_| false).unwrap();
        t.deschide(k("a"));
        assert_eq!(t.urmatorul(&l, 1), Pas::Capat);
        assert_eq!(t.urmatorul(&l, -1), Pas::Capat, "a e primul");
    }

    #[test]
    fn filtrul_care_scoate_obiectul_activ_porneste_de_la_capat() {
        let mut t = Taburi::noi();
        t.deschide(k("x"));
        let l = lista(&["a", "b", "c"]);
        assert_eq!(t.urmatorul(&l, 1), Pas::Obiect(k("a")));
        assert_eq!(t.urmatorul(&l, -1), Pas::Obiect(k("c")));
        assert_eq!(t.urmatorul(&[], 1), Pas::Capat);
    }

    #[test]
    fn ctrl_w_activeaza_vecinul_din_stanga() {
        let mut t = Taburi::noi();
        t.deschide(k("p"));
        for n in ["a", "b", "c"] {
            t.fixeaza(k(n), |_| false).unwrap();
        }
        t.activeaza(Activ::Fixat(1));
        t.inchide_activ();
        assert_eq!(t.cheie_activa(), Some(&k("a")), "b inchis, a in stanga");
        t.inchide_activ();
        assert_eq!(
            t.activ(),
            Activ::Previzualizare,
            "la stanga primului: previzualizarea"
        );
        t.inchide_activ();
        assert_eq!(
            t.cheie_activa(),
            Some(&k("c")),
            "fara previzualizare, ultimul fixat"
        );
        // inchis un tab din dreapta celui activ: activul ramane acelasi obiect
        t.fixeaza(k("d"), |_| false).unwrap();
        t.activeaza(Activ::Fixat(0));
        t.inchide(Activ::Fixat(1));
        assert_eq!(t.cheie_activa(), Some(&k("c")));
        // inchis un tab din stanga: indicele activului scade
        t.fixeaza(k("e"), |_| false).unwrap();
        t.inchide(Activ::Fixat(0));
        assert_eq!(t.cheie_activa(), Some(&k("e")));
    }

    #[test]
    fn dupa_recomparare_dispar_taburile_obiectelor_disparute() {
        let mut t = Taburi::noi();
        t.deschide(k("p"));
        t.fixeaza(k("a"), |_| false).unwrap();
        t.fixeaza(k("b"), |_| false).unwrap();
        t.pastreaza(|c| c != &k("b") && c != &k("p"));
        assert_eq!(t.fixate(), [&k("a")]);
        assert_eq!(t.previzualizare(), None);
        assert_eq!(t.cheie_activa(), Some(&k("a")));
    }

    #[test]
    fn sesiunea_reface_fixatele_in_ordine() {
        let mut t = Taburi::noi();
        for n in ["a", "b"] {
            t.fixeaza(k(n), |_| false).unwrap();
        }
        let s = t.de_salvat();
        let r = Taburi::din_sesiune(s.clone());
        assert_eq!(r.de_salvat(), s);
        assert_eq!(r.cheie_activa(), Some(&k("a")));
        let multe: Vec<Cheie> = (0..12).map(|i| k(&format!("x{i}"))).collect();
        assert_eq!(Taburi::din_sesiune(multe).fixate().len(), MAX_FIXATE);
    }

    #[test]
    fn numele_se_taie_pe_mijloc_pe_caractere() {
        assert_eq!(taie_pe_mijloc("scurt", 10), "scurt");
        assert_eq!(
            taie_pe_mijloc("TESTSIMILITUDINE_CaseA_LiniiGoale", 16),
            "TESTS\u{2026}LiniiGoale"
        );
        let t = taie_pe_mijloc("TESTSIMILITUDINE_CaseA_LiniiGoale", 16);
        assert_eq!(t.chars().count(), 16);
        // nu se rupe o litera cu diacritice
        let d = taie_pe_mijloc("ăăăăăăăăăăăăăăăăăăăă", 7); // diacritice-permise
        assert_eq!(d.chars().count(), 7);
        assert_eq!(taie_pe_mijloc("abc", 1), "\u{2026}");
        assert_eq!(taie_pe_mijloc("abc", 0), "");
    }
}
