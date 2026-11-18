//! Planul de deploy din fereastra: ce s-a pus „in plan", garda „baza n-a mai
//! fost atinsa" si scrierea scripturilor, cu revenirea salvata inainte.
//!
//! **Se strica astfel:** se scrie altceva decat ce s-a ales, se scrie fara
//! garda (sau dupa o garda veche), ori pasii afisati nu sunt in ordinea din
//! script. Toate trei se decid aici, fara Slint, cu aceleasi functii ca
//! `gmotion deploy --plan` (`diff-obiecte::plan`), nu cu o a doua
//! implementare.

use std::path::Path;

use gmotion_modul_diff_obiecte::{self as modul, Baza, ModImpartire, Plan, Raport};

/// Un pas, cum se arata: operatia, obiectul, tipul, in ordinea din script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasAfisat {
    /// `false` = obiectul nu exista in baza: deploy-ul il creeaza.
    pub exista_in_baza: bool,
    /// Baza in care se scrie. Cand planul acopera mai multe, randul trebuie
    /// sa spuna care — altfel „scoate din plan" ar scoate obiectul din toate.
    pub baza: String,
    pub obiect: String,
    pub tip: String,
}

/// Ce a spus garda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Garda {
    /// Inca nu s-a verificat; scrierea nu e permisa.
    Neverificata,
    /// Obiectele din plan sunt exact cum erau la comparare.
    Ok { la: String },
    /// Ceva s-a schimbat de la comparare; fiecare schimbare pe nume.
    Schimbata(Vec<String>),
    /// Recitirea n-a mers (conexiune, instantaneu lipsa).
    Eroare(String),
}

/// Starea ecranului de deploy, de la deschidere pana la scriere.
pub struct StareDeploy {
    pub plan: Plan,
    pub garda: Garda,
    pub impartire: ModImpartire,
    /// Rezultatul scrierii: raportul, sau de ce n-a mers.
    pub rezultat: Option<Result<Raport, String>>,
    /// Obiectele recitite la verificare (referinta, si fiecare baza tinta):
    /// din ele se scrie, nu din cele de la comparare.
    recitite: Option<(Baza, Vec<(String, Baza)>)>,
    /// Textele finale din editor, pe `(baza, schema.nume)`: ce se scrie in
    /// locul codului din sursa pentru obiectele editate.
    pub texte_finale: std::collections::BTreeMap<(String, String), String>,
}

impl StareDeploy {
    pub fn nou(plan: Plan) -> Self {
        StareDeploy {
            plan,
            garda: Garda::Neverificata,
            impartire: ModImpartire::default(),
            rezultat: None,
            recitite: None,
            texte_finale: Default::default(),
        }
    }

    /// Cu textele finale din editor (Aplica, scris de mana).
    pub fn cu_texte_finale(
        mut self,
        texte: std::collections::BTreeMap<(String, String), String>,
    ) -> Self {
        self.texte_finale = texte;
        self
    }

    /// Pasii, in ordinea in care ajung in script (dependentele intai), pe
    /// codul din `sursa`.
    pub fn pasi_in_ordine(&self, sursa: &Baza) -> Vec<PasAfisat> {
        let Ok(pe_baza) = self.plan.de_deployat(sursa) else {
            // un obiect a disparut din sursa: ordinea din plan, garda il va spune
            return self
                .plan
                .pasi
                .iter()
                .map(|p| PasAfisat {
                    exista_in_baza: p.tinta.is_some(),
                    baza: p.baza.clone(),
                    obiect: p.obiect.clone(),
                    tip: p.tip.clone(),
                })
                .collect();
        };
        let mut out = Vec::new();
        for (baza, obiecte) in pe_baza.iter() {
            for i in modul::deploy::ordine_dupa_dependente(obiecte) {
                let o = &obiecte[i];
                let exista = self
                    .plan
                    .pasi
                    .iter()
                    .find(|p| &p.baza == baza && p.obiect == o.obiect)
                    .is_some_and(|p| p.tinta.is_some());
                out.push(PasAfisat {
                    exista_in_baza: exista,
                    baza: baza.clone(),
                    obiect: o.obiect.clone(),
                    tip: o.tip.clone(),
                });
            }
        }
        out
    }

    /// Garda, pe obiectele recitite acum. Le tine minte: din ele se scrie.
    pub fn aplica_verificarea(&mut self, referinta: Baza, tinte: Vec<(String, Baza)>, la: &str) {
        let schimbari = self.plan.verifica(&referinta, &tinte);
        self.rezultat = None;
        if schimbari.is_empty() {
            self.garda = Garda::Ok { la: la.to_string() };
            self.recitite = Some((referinta, tinte));
        } else {
            self.garda = Garda::Schimbata(schimbari.iter().map(ToString::to_string).collect());
            self.recitite = None;
        }
    }

    /// Recitirea a esuat: scrierea ramane blocata.
    pub fn verificare_esuata(&mut self, motiv: String) {
        self.garda = Garda::Eroare(motiv);
        self.recitite = None;
    }

    /// Se poate scrie doar dupa o garda trecuta si inainte de o scriere reusita.
    pub fn poate_scrie(&self) -> bool {
        matches!(self.garda, Garda::Ok { .. })
            && self.recitite.is_some()
            && !self.plan.pasi.is_empty()
            && !matches!(self.rezultat, Some(Ok(_)))
    }

    /// Scrie scripturile si, inainte, revenirea, din obiectele recitite.
    pub fn scrie(&mut self, director: &Path, acum: &str) {
        let Some((referinta, altele)) = self.recitite.as_ref().filter(|_| self.poate_scrie())
        else {
            self.rezultat = Some(Err("verifica intai ca baza n-a mai fost atinsa".to_string()));
            return;
        };
        let altele = altele.clone();
        let r = self
            .plan
            .de_deployat_cu(referinta, &self.texte_finale)
            .and_then(|pe_baza| {
                let revenire = self.plan.revenire(referinta, &altele);
                modul::deploy::scrie_cu_revenire(
                    director,
                    &self.plan.referinta,
                    &pe_baza,
                    &revenire,
                    self.impartire,
                    self.plan.sarite,
                    acum,
                )
            })
            .map_err(|e| e.to_string());
        if let Err(e) = &r {
            tracing::warn!(target: "gmotion::app", eroare = %e, "deploy-ul nu s-a scris");
        }
        self.rezultat = Some(r);
    }

    /// Modul de impartire s-a schimbat: un rezultat vechi nu mai e al lui.
    pub fn alege_impartirea(&mut self, m: ModImpartire) {
        if m != self.impartire {
            self.impartire = m;
            if matches!(self.rezultat, Some(Ok(_))) {
                self.rezultat = None;
            }
        }
    }
}

/// Modul de impartire dupa pozitia din ecran (0 per baza, 1 unul, 2 per
/// obiect, 3 cate `cate`), si invers.
pub fn impartire_din(i: i32, cate: usize) -> ModImpartire {
    match i {
        1 => ModImpartire::Unul,
        2 => ModImpartire::PerObiect,
        3 => ModImpartire::CateN(cate.max(1)),
        _ => ModImpartire::PerBaza,
    }
}

pub fn pozitia_impartirii(m: ModImpartire) -> i32 {
    match m {
        ModImpartire::PerBaza => 0,
        ModImpartire::Unul => 1,
        ModImpartire::PerObiect => 2,
        ModImpartire::CateN(_) => 3,
    }
}

/// Ce se deseneaza pe ecranul de deploy, din starea lui.
pub fn ecran(
    st: &StareDeploy,
    sursa: &Baza,
    director: &str,
    cate: usize,
) -> gmotion_fereastra::model_deploy::EcranDeploy {
    use gmotion_fereastra::model_deploy::{EcranDeploy, GardaDeploy, PasDeploy, RezultatDeploy};
    let pasi = st.pasi_in_ordine(sursa);
    let mut baze: Vec<String> = Vec::new();
    for p in &pasi {
        if !baze.contains(&p.baza) {
            baze.push(p.baza.clone());
        }
    }
    EcranDeploy {
        referinta: st.plan.referinta.clone(),
        tinta: match baze.as_slice() {
            [] => String::new(),
            [una] => una.clone(),
            multe => format!("{} baze", multe.len()),
        },
        baze: baze.clone(),
        pasi: pasi
            .into_iter()
            .map(|p| PasDeploy {
                creeaza: !p.exista_in_baza,
                baza: p.baza,
                obiect: p.obiect,
                tip: gmotion_core::nume_tip(&p.tip).to_string(),
            })
            .collect(),
        sarite: st.plan.sarite,
        garda: match &st.garda {
            Garda::Neverificata => GardaDeploy::Neverificata,
            Garda::Ok { la } => GardaDeploy::Ok { la: la.clone() },
            Garda::Schimbata(v) => GardaDeploy::Schimbata(v.clone()),
            Garda::Eroare(e) => GardaDeploy::Eroare(e.clone()),
        },
        impartire: pozitia_impartirii(st.impartire),
        cate,
        poate_scrie: st.poate_scrie(),
        director: director.to_string(),
        rezultat: match &st.rezultat {
            None => RezultatDeploy::Nescris,
            Some(Ok(r)) => RezultatDeploy::Scris {
                director: r.director.display().to_string(),
                fisiere: r.fisiere.len(),
                revenire: r.revenire.len(),
            },
            Some(Err(e)) => RezultatDeploy::Eroare(e.clone()),
        },
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use gmotion_db::Obiect as ObiectDb;

    fn baza(obiecte: &[(&str, &str)]) -> Baza {
        let v: Vec<ObiectDb> = obiecte
            .iter()
            .map(|(nume, def)| ObiectDb {
                schema: "dbo".into(),
                nume: (*nume).into(),
                tip: if nume.starts_with("vw") { "V" } else { "P" }.into(),
                definitie: (*def).into(),
                setari: None,
            })
            .collect();
        modul::pregateste(&v)
    }

    /// Sursa: o procedura care foloseste un view nou, si o procedura
    /// modificata. Baza: doar procedura veche.
    fn date() -> (Baza, Baza) {
        let sursa = baza(&[
            (
                "pRaport",
                "CREATE PROC dbo.pRaport AS SELECT * FROM dbo.vwStoc",
            ),
            ("vwStoc", "CREATE VIEW dbo.vwStoc AS SELECT 1 AS x"),
            ("pVechi", "CREATE PROC dbo.pVechi AS SELECT 2"),
        ]);
        let tinta = baza(&[("pVechi", "CREATE PROC dbo.pVechi AS SELECT 1")]);
        (sursa, tinta)
    }

    fn plan_din(sursa: &Baza, tinta: &Baza, alese: &[&str]) -> Plan {
        let c = modul::compara("Dev", sursa, &[("Prod".into(), tinta.clone())], |_| {});
        Plan::din_comparare(
            &c,
            sursa,
            &[("Prod".into(), tinta.clone())],
            |r| alese.iter().any(|a| *a == r.obiect),
            "2026-11-18 14:00",
        )
    }

    fn dir(eticheta: &str) -> std::path::PathBuf {
        let d =
            std::env::temp_dir().join(format!("gmotion-deploy-{eticheta}-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        d
    }

    #[test]
    fn intra_doar_ce_s_a_ales_in_ordinea_dependentelor() {
        let (s, t) = date();
        let st = StareDeploy::nou(plan_din(&s, &t, &["dbo.pRaport", "dbo.vwStoc"]));
        let pasi = st.pasi_in_ordine(&s);
        let nume: Vec<&str> = pasi.iter().map(|p| p.obiect.as_str()).collect();
        // view-ul de care depinde procedura vine intai; pVechi n-a fost ales
        assert_eq!(nume, ["dbo.vwStoc", "dbo.pRaport"]);
        assert!(
            pasi.iter().all(|p| !p.exista_in_baza),
            "amandoua sunt noi in baza"
        );
    }

    #[test]
    fn fara_garda_nu_se_scrie_nimic() {
        let (s, t) = date();
        let mut st = StareDeploy::nou(plan_din(&s, &t, &["dbo.pVechi"]));
        assert!(!st.poate_scrie());
        let d = dir("fara-garda");
        st.scrie(&d, "2026-11-18 14:05");
        assert!(matches!(st.rezultat, Some(Err(_))));
        assert!(!d.exists(), "nimic scris inainte de garda");
    }

    #[test]
    fn baza_atinsa_opreste_scrierea_si_spune_ce_s_a_schimbat() {
        let (s, t) = date();
        let mut st = StareDeploy::nou(plan_din(&s, &t, &["dbo.pVechi"]));
        // intre comparare si deploy, cineva a schimbat pVechi in Prod
        let atinsa = baza(&[("pVechi", "CREATE PROC dbo.pVechi AS SELECT 99")]);
        st.aplica_verificarea(s.clone(), vec![("Prod".to_string(), atinsa)], "14:05");
        match &st.garda {
            Garda::Schimbata(v) => assert!(v[0].contains("dbo.pVechi"), "{v:?}"),
            g => panic!("garda trebuia sa opreasca: {g:?}"),
        }
        assert!(!st.poate_scrie());
    }

    #[test]
    fn dupa_garda_se_scriu_scripturile_si_revenirea() {
        let (s, t) = date();
        let mut st = StareDeploy::nou(plan_din(&s, &t, &["dbo.pVechi", "dbo.vwStoc"]));
        st.aplica_verificarea(s.clone(), vec![("Prod".to_string(), t.clone())], "14:05");
        assert_eq!(st.garda, Garda::Ok { la: "14:05".into() });
        assert!(st.poate_scrie());
        let d = dir("scrie");
        st.scrie(&d, "2026-11-18 14:05");
        let r = st
            .rezultat
            .as_ref()
            .unwrap()
            .as_ref()
            .expect("scrierea merge");
        assert_eq!(r.obiecte, 2);
        assert!(!r.fisiere.is_empty() && !r.revenire.is_empty());
        let script: String = r
            .fisiere
            .iter()
            .map(|f| std::fs::read_to_string(f).unwrap())
            .collect();
        assert!(script.contains("dbo.pVechi") && script.contains("dbo.vwStoc"));
        assert!(!script.contains("dbo.pRaport"), "doar ce s-a ales");
        // o a doua scriere peste aceeasi garda nu se face
        assert!(!st.poate_scrie());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn alta_impartire_permite_o_noua_scriere() {
        let (s, t) = date();
        let mut st = StareDeploy::nou(plan_din(&s, &t, &["dbo.pVechi"]));
        st.aplica_verificarea(s.clone(), vec![("Prod".to_string(), t.clone())], "14:05");
        let d = dir("impartire");
        st.scrie(&d, "2026-11-18 14:05");
        assert!(!st.poate_scrie());
        st.alege_impartirea(ModImpartire::Unul);
        assert!(st.poate_scrie(), "alt mod de impartire = alta scriere");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn ecranul_arata_pasii_si_starea_garzii() {
        let (s, t) = date();
        let mut st = StareDeploy::nou(plan_din(
            &s,
            &t,
            &["dbo.pRaport", "dbo.vwStoc", "dbo.pVechi"],
        ));
        let e = ecran(&st, &s, "C:\\deploy", 10);
        assert_eq!(e.pasi.len(), 3);
        assert_eq!(
            e.pasi.iter().filter(|p| p.creeaza).count(),
            2,
            "vwStoc si pRaport sunt noi"
        );
        assert!(!e.poate_scrie);
        assert_eq!(e.impartire, 0);
        st.aplica_verificarea(s.clone(), vec![("Prod".to_string(), t.clone())], "14:05");
        let e = ecran(&st, &s, "C:\\deploy", 10);
        assert!(e.poate_scrie);
        for i in 0..4 {
            assert_eq!(pozitia_impartirii(impartire_din(i, 10)), i);
        }
    }

    #[test]
    fn o_recitire_esuata_blocheaza_scrierea() {
        let (s, t) = date();
        let mut st = StareDeploy::nou(plan_din(&s, &t, &["dbo.pVechi"]));
        st.aplica_verificarea(s.clone(), vec![("Prod".to_string(), t.clone())], "14:05");
        st.verificare_esuata("serverul nu raspunde".into());
        assert!(!st.poate_scrie());
        assert_eq!(st.garda, Garda::Eroare("serverul nu raspunde".into()));
    }
}
