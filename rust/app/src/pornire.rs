//! De unde vin obiectele: ce alege utilizatorul in ecranul de conectare se
//! transforma aici in surse adevarate si intr-o comparare.
//!
//! **Se strica astfel:** parola ajunge intr-un mesaj de eroare sau intr-o
//! sesiune salvata, sau ecranul spune „conectat" cand nu e. De-aia construirea
//! sursei sta intr-un singur loc, si ce se arata vine din ce chiar s-a
//! intamplat (bazele citite), nu din ce s-a cerut.

use gmotion_core::Limba;
use gmotion_db::{Autentificare, Certificat, SursaFixture, SursaObiecte, SursaSqlServer, Tinta};
use gmotion_fereastra::model_conectare::{Conectare, Parte, StareConexiune};
use gmotion_modul_diff_obiecte as modul;

/// Deschide sursa unei parti, dupa ce s-a ales din ecran.
///
/// Parola se ia din `Acreditari` chiar aici si nu mai iese: nici in jurnal,
/// nici in starea afisata.
pub fn deschide(c: &Conectare, p: &Parte) -> Result<Box<dyn SursaObiecte>, String> {
    if let Some(cale) = &c.instantaneu {
        return SursaFixture::din_fisier(cale)
            .map(|s| Box::new(s) as Box<dyn SursaObiecte>)
            .map_err(|e| e.to_string());
    }
    let auth = if p.auth_windows {
        Autentificare::Integrata
    } else {
        Autentificare::Sql {
            utilizator: p.acreditari.utilizator.clone(),
            parola: p.acreditari.parola().to_string(),
        }
    };
    let (server, port) = imparte_portul(&p.server);
    SursaSqlServer::nou(
        Tinta {
            server,
            port,
            certificat: certificat(c),
        },
        auth,
    )
    .map(|s| Box::new(cu_timeout(c, s)) as Box<dyn SursaObiecte>)
    .map_err(|e| e.to_string())
}

/// `timeout_conectare_s` din configurare, daca e dat (F-B5).
fn cu_timeout(c: &Conectare, s: SursaSqlServer) -> SursaSqlServer {
    match c.timeout_conectare_s {
        Some(n) => s.cu_timeout_conectare(std::time::Duration::from_secs(n)),
        None => s,
    }
}

/// Ce alege ecranul ajunge in conexiune. Pana la 2026-11-18 aici era mereu
/// `Nevalidat`, desi ecranul putea spune „se verifica".
fn certificat(c: &Conectare) -> Certificat {
    if c.verifica_certificatul {
        Certificat::Sistem
    } else {
        Certificat::Nevalidat
    }
}

/// `srv.domeniu.local,1433` -> (`srv.domeniu.local`, 1433). Forma cu virgula e
/// cea pe care o scrie lumea (si cea din macheta), nu una inventata aici.
fn imparte_portul(server: &str) -> (String, Option<u16>) {
    match server.rsplit_once(',') {
        Some((s, p)) => match p.trim().parse::<u16>() {
            Ok(port) => (s.trim().to_string(), Some(port)),
            Err(_) => (server.trim().to_string(), None),
        },
        None => (server.trim().to_string(), None),
    }
}

/// Citeste bazele unei parti si intoarce starea de afisat.
///
/// Nu spune „conectat" decat daca chiar a citit ceva: asta e singurul lucru
/// care dovedeste conexiunea.
pub fn citeste_bazele(c: &Conectare, p: &Parte) -> (Vec<String>, StareConexiune) {
    let mut sursa = match deschide(c, p) {
        Ok(s) => s,
        Err(e) => return (Vec::new(), StareConexiune::Eroare(e)),
    };
    match sursa.baze() {
        Ok(baze) => {
            let l = c.limba;
            let stare = StareConexiune::Conectat(format!(
                "{} · {}",
                l.alege("conectat", "connected"),
                l.numar(baze.len(), ("baza", "baze"), ("database", "databases"))
            ));
            (baze, stare)
        }
        Err(e) => (Vec::new(), StareConexiune::Eroare(e.to_string())),
    }
}

/// Instantaneele gasite pe disc, pentru „Deschide un instantaneu": in
/// directorul de date al aplicatiei, langa binar, in directorul curent si in
/// `referinta/instantanee` (in dezvoltare). Fara dialog nativ, deci fara o
/// dependinta noua — vezi `JURNAL.md`.
///
/// Se intorc doar fisiere `.json` care chiar exista, fara duplicate, cele mai
/// noi intai: cine tocmai a facut un instantaneu il gaseste primul.
/// Un `.json` e propus doar daca incepe ca un instantaneu: obiect cu cheia
/// `versiune` la inceput. Altfel apareau si `*.asteptari.json` (liste) sau
/// configurarea, iar alegerea lor dadea GM-DB-007 (Mihai, 2026-11-19).
fn pare_instantaneu(cale: &std::path::Path) -> bool {
    use std::io::Read;
    let mut inceput = [0u8; 512];
    let n = std::fs::File::open(cale)
        .and_then(|mut f| f.read(&mut inceput))
        .unwrap_or(0);
    let t = String::from_utf8_lossy(&inceput[..n]);
    let t = t.trim_start_matches('\u{feff}').trim_start();
    t.starts_with('{') && t.contains("\"versiune\"")
}

pub fn instantanee_gasite() -> Vec<std::path::PathBuf> {
    let mut locuri: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(d) = gmotion_platforma::cai::director_date() {
        locuri.push(d.join("instantanee"));
        locuri.push(d);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            locuri.push(d.to_path_buf());
        }
    }
    if let Ok(d) = std::env::current_dir() {
        locuri.push(d.clone());
        // in dezvoltare, instantaneele stau in `referinta/instantanee`, iar
        // comenzile se dau din radacina sau din `rust/<crate>`
        for sus in ["", "../", "../../", "../../../"] {
            locuri.push(d.join(format!("{sus}referinta/instantanee")));
        }
    }

    let mut gasite: Vec<(std::time::SystemTime, std::path::PathBuf)> = Vec::new();
    for loc in locuri {
        let Ok(intrari) = std::fs::read_dir(&loc) else {
            continue;
        };
        for e in intrari.flatten() {
            let cale = e.path();
            if !cale.is_file()
                || cale.extension().is_none_or(|x| x != "json")
                || !pare_instantaneu(&cale)
            {
                continue;
            }
            let cale = cale.canonicalize().map(fara_prefix_extins).unwrap_or(cale);
            if gasite.iter().any(|(_, c)| *c == cale) {
                continue;
            }
            let cand = e
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            gasite.push((cand, cale));
        }
    }
    gasite.sort_by_key(|g| std::cmp::Reverse(g.0));
    gasite.into_iter().map(|(_, c)| c).take(40).collect()
}

/// Pe Windows, `canonicalize` intoarce `\\?\C:\...`: se vedea asa in
/// lista de instantanee si in jurnal. Prefixul se scoate; o cale de retea
/// (`\\?\UNC\srv\x`) redevine `\\srv\x`.
fn fara_prefix_extins(cale: std::path::PathBuf) -> std::path::PathBuf {
    let s = cale.to_string_lossy();
    let curata = if let Some(r) = s.strip_prefix(r"\\?\UNC\") {
        Some(format!(r"\\{r}"))
    } else {
        s.strip_prefix(r"\\?\").map(str::to_string)
    };
    curata.map_or(cale, std::path::PathBuf::from)
}

/// Verifica o conexiune si intoarce ce a raspuns serverul: editia, sistemul,
/// de cat timp merge si cate baze are. Pe instantaneu nu exista server, deci
/// se spune de unde s-au luat obiectele.
///
/// Nu arunca parola in mesaj, ca peste tot aici.
pub fn testeaza(c: &Conectare, p: &Parte) -> Result<String, String> {
    if let Some(cale) = &c.instantaneu {
        let mut s = SursaFixture::din_fisier(cale).map_err(|e| e.to_string())?;
        let baze = s.baze().map_err(|e| e.to_string())?;
        let l = c.limba;
        return Ok(format!(
            "{} · {} · {}",
            l.alege("instantaneu", "snapshot"),
            s.descriere(),
            l.numar(baze.len(), ("baza", "baze"), ("database", "databases"))
        ));
    }
    if let Some(ce) = p.ce_lipseste() {
        return Err(gmotion_fereastra::model_conectare::completeaza(ce, c.limba));
    }
    let (server, port) = imparte_portul(&p.server);
    let auth = if p.auth_windows {
        Autentificare::Integrata
    } else {
        Autentificare::Sql {
            utilizator: p.acreditari.utilizator.clone(),
            parola: p.acreditari.parola().to_string(),
        }
    };
    let mut s = SursaSqlServer::nou(
        Tinta {
            server,
            port,
            certificat: certificat(c),
        },
        auth,
    )
    .map(|s| cu_timeout(c, s))
    .map_err(|e| e.to_string())?;
    // `master` exista pe orice server si nu cere drepturi pe bazele
    // utilizatorului: altfel „Testeaza" ar cadea acolo unde conectarea merge
    let info = s.verifica("master").map_err(|e| e.to_string())?;
    Ok(descrie_serverul(&info, c.limba))
}

/// Propozitia din ecran, din ce a raspuns serverul.
fn descrie_serverul(i: &gmotion_db::InfoServer, l: Limba) -> String {
    let mut parti = vec![i.editie.clone(), i.sistem.clone()];
    if let Some(s) = i.pornit_de_secunde {
        parti.push(format!(
            "{} {}",
            l.alege("pornit de", "up for"),
            durata_scurta(s, l)
        ));
    }
    if let Some(n) = i.nr_baze {
        parti.push(l.numar(
            usize::try_from(n).unwrap_or(0),
            ("baza", "baze"),
            ("database", "databases"),
        ));
    }
    parti.retain(|p| !p.trim().is_empty());
    parti.join(" · ")
}

/// „3 zile", „5 ore", „12 minute": destul pentru ecran, fara dependinte.
/// Singularul si pluralul se scriu amandoua — „1 zile" se vede.
fn durata_scurta(secunde: i64, l: Limba) -> String {
    let s = secunde.max(0);
    let (n, ro, en) = match s {
        0..=119 => (s, ("secunda", "secunde"), ("second", "seconds")),
        120..=7199 => (s / 60, ("minut", "minute"), ("minute", "minutes")),
        7200..=86_399 => (s / 3600, ("ora", "ore"), ("hour", "hours")),
        _ => (s / 86_400, ("zi", "zile"), ("day", "days")),
    };
    l.numar(usize::try_from(n).unwrap_or(0), ro, en)
}

/// Compararea offline din v3.4: doua bucati de cod lipite in ecran, fara
/// server si fara instantaneu. Obiectul se numeste ca in machete.
pub fn compara_offline(sursa: &str, tinta: &str, l: Limba) -> Result<Comparata, String> {
    if sursa.trim().is_empty() || tinta.trim().is_empty() {
        return Err(l
            .alege(
                "lipeste cod in ambele panouri",
                "paste code into both panels",
            )
            .to_string());
    }
    let obiect = |text: &str| gmotion_db::Obiect {
        schema: "dbo".into(),
        nume: "cod_lipit".into(),
        tip: "P".into(),
        definitie: text.to_string(),
        setari: None,
    };
    let s = modul::pregateste(&[obiect(sursa)]);
    let t = modul::pregateste(&[obiect(tinta)]);
    let tinte = vec![("Lipit".to_string(), t)];
    let comparare = modul::compara("Lipit", &s, &tinte, |_| {});
    Ok(Comparata {
        randuri: comparare.randuri,
        sursa: s,
        tinte,
        server: l
            .alege("comparare offline", "offline comparison")
            .to_string(),
        referinta: l.alege("Lipit (martor)", "Pasted (witness)").to_string(),
    })
}

/// Rezultatul unei comparari pornite din ecranul de conectare.
pub struct Comparata {
    pub randuri: Vec<modul::Rand>,
    pub sursa: modul::Baza,
    /// Bazele comparate, in ordinea bifata. Una singura e cazul obisnuit.
    pub tinte: Vec<(String, modul::Baza)>,
    pub server: String,
    /// Baza de referinta **cu care a rulat**, nu ce scrie in formular acum.
    ///
    /// Compararea ruleaza pe alt fir, iar formularul primea tastatura cat
    /// rula: o tasta in campul bazei si rezultatul se incarca sub referinta
    /// "", sesiunea se salveaza sub cheia "", directorul de deploy devine
    /// „_<data>" (F-A3 din auditul QA).
    pub referinta: String,
}

impl Comparata {
    /// Prima tinta: ecranele care arata o singura baza (antetul, deploy-ul
    /// pe o baza) au nevoie de un nume de aratat.
    pub fn prima_tinta(&self) -> &str {
        self.tinte.first().map_or("", |(n, _)| n.as_str())
    }
}

/// Obiectele citite de pe server (sau din instantaneu), **fara comparare**.
///
/// Garda de deploy are nevoie doar de atat: obiectele recitite, ca sa le
/// compare amprentele cu cele din plan. Pana la auditul QA chema `compara`,
/// adica refacea toata compararea doar ca sa reciteasca — minute, pe o baza
/// mare (P4).
pub struct Citit {
    pub sursa: modul::Baza,
    pub tinte: Vec<(String, modul::Baza)>,
    pub server: String,
    pub referinta: String,
}

/// Citeste sursa si fiecare baza bifata. `progres` primeste pasii pe rand
/// (ce baza se citeste, a cata din cate) si intoarce `false` daca
/// utilizatorul a apasat „Anuleaza".
pub fn citeste(c: &Conectare, mut progres: impl FnMut(Pas) -> bool) -> Result<Citit, String> {
    let baze = c.baza.baze_alese();
    if baze.is_empty() {
        return Err(c
            .limba
            .alege("nicio baza de comparat", "no database to compare")
            .to_string());
    }
    let mut sursa_db = deschide(c, &c.sursa)?;
    // identitatea, nu descrierea: intra in cheia sesiunii si in antet, deci
    // trebuie sa insemne „acelasi server" si maine, fara felul autentificarii
    let server = sursa_db.identitate();
    if !progres(Pas {
        baza: c.sursa.baza.clone(),
        facut: 0,
        din: baze.len() + 1,
    }) {
        return Err(ANULAT.to_string());
    }
    let obiecte_sursa = sursa_db.obiecte(&c.sursa.baza).map_err(|e| e.to_string())?;
    let sursa = modul::pregateste(&obiecte_sursa);

    let mut tinte: Vec<(String, modul::Baza)> = Vec::new();
    for (i, nume) in baze.iter().enumerate() {
        if !progres(Pas {
            baza: nume.clone(),
            facut: i + 1,
            din: baze.len() + 1,
        }) {
            return Err(ANULAT.to_string());
        }
        // pe instantaneu toate bazele vin din acelasi fisier, deci nu se
        // deschide o a doua conexiune pentru fiecare
        let obiecte = if c.instantaneu.is_some() {
            sursa_db.obiecte(nume).map_err(|e| e.to_string())?
        } else {
            let mut db = deschide(c, &c.baza)?;
            db.obiecte(nume).map_err(|e| e.to_string())?
        };
        tinte.push((nume.clone(), modul::pregateste(&obiecte)));
    }

    Ok(Citit {
        sursa,
        tinte,
        server,
        referinta: c.sursa.baza.clone(),
    })
}

/// Citeste si compara. Vezi [`citeste`] pentru partea de citire.
pub fn compara_cu(c: &Conectare, progres: impl FnMut(Pas) -> bool) -> Result<Comparata, String> {
    compara_cu_anulare(c, progres, || true)
}

/// Ca [`compara_cu`], dar si compararea propriu-zisa (nu doar citirea) se
/// opreste cand `continua()` spune `false`, cel mult dupa un obiect (P8).
/// Oprita, intoarce [`ANULAT`].
pub fn compara_cu_anulare(
    c: &Conectare,
    progres: impl FnMut(Pas) -> bool,
    continua: impl Fn() -> bool,
) -> Result<Comparata, String> {
    let x = citeste(c, progres)?;
    let comparare = modul::compara_cu_oprire(&x.referinta, &x.sursa, &x.tinte, |_| {}, &continua)
        .ok_or_else(|| ANULAT.to_string())?;
    Ok(Comparata {
        randuri: comparare.randuri,
        sursa: x.sursa,
        tinte: x.tinte,
        server: x.server,
        referinta: x.referinta,
    })
}

/// Recitirea pentru garda de deploy: fara comparare, fara progres.
pub fn reciteste(c: &Conectare) -> Result<Citit, String> {
    citeste(c, |_| true)
}

/// Unde a ajuns compararea: ce baza se citeste acum, a cata din cate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pas {
    pub baza: String,
    pub facut: usize,
    pub din: usize,
}

/// Mesajul cu care se incheie o comparare oprita de utilizator. Se recunoaste
/// dupa text in fereastra, ca sa nu se arate ca eroare.
pub const ANULAT: &str = "comparare anulata";

/// Compararea fara progres si fara anulare.
pub fn compara(c: &Conectare) -> Result<Comparata, String> {
    compara_cu(c, |_| true)
}

#[cfg(test)]
mod teste {
    use super::*;
    use gmotion_fereastra::model_conectare::Acreditari;

    #[test]
    fn caile_de_instantaneu_nu_mai_au_prefixul_extins() {
        use std::path::PathBuf;
        assert_eq!(
            fara_prefix_extins(PathBuf::from(r"\\?\C:\GMotion\a.json")),
            PathBuf::from(r"C:\GMotion\a.json")
        );
        assert_eq!(
            fara_prefix_extins(PathBuf::from(r"\\?\UNC\srv\x\a.json")),
            PathBuf::from(r"\\srv\x\a.json")
        );
        assert_eq!(
            fara_prefix_extins(PathBuf::from("/home/a.json")),
            PathBuf::from("/home/a.json")
        );
    }

    #[test]
    fn alegerea_certificatului_ajunge_in_conexiune() {
        let mut c = Conectare::noua();
        assert_eq!(certificat(&c), Certificat::Nevalidat);
        c.verifica_certificatul = true;
        assert_eq!(certificat(&c), Certificat::Sistem);
    }

    #[test]
    fn portul_se_desparte_de_server_ca_in_machete() {
        assert_eq!(
            imparte_portul("srv-prod.firma.local,1433"),
            ("srv-prod.firma.local".to_string(), Some(1433))
        );
        assert_eq!(
            imparte_portul("SRV-DEV\\SQL2019"),
            ("SRV-DEV\\SQL2019".to_string(), None)
        );
        // o virgula care nu e port ramane in numele serverului
        assert_eq!(
            imparte_portul("srv,instanta"),
            ("srv,instanta".to_string(), None)
        );
        assert_eq!(
            imparte_portul("  srv , 1433 "),
            ("srv".to_string(), Some(1433))
        );
    }

    fn instantaneu(cale: &std::path::Path) {
        std::fs::write(
            cale,
            r#"{"versiune":1,"server":"SRV","creat":"2026-11-17T10:00:00Z","nota":"","baze":{
                "A":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 1"}],
                "B":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 2"}]}}"#,
        )
        .unwrap();
    }

    #[test]
    fn dintr_un_instantaneu_se_citesc_bazele_si_se_compara() {
        let d = std::env::temp_dir().join(format!("gmotion-pornire-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let cale = d.join("inst.json");
        instantaneu(&cale);

        let mut c = Conectare::noua();
        c.instantaneu = Some(cale.to_string_lossy().to_string());
        let (baze, stare) = citeste_bazele(&c, &c.sursa);
        assert_eq!(baze, ["A", "B"]);
        assert!(stare.e_conectat(), "{stare:?}");
        assert_eq!(stare.text(), "conectat · 2 baze");

        c.sursa.baza = "A".into();
        c.baza.baza = "B".into();
        let r = compara(&c).expect("compararea merge");
        assert_eq!(r.randuri.len(), 1);
        assert_eq!(r.randuri[0].obiect, "dbo.p");
        assert_eq!(r.prima_tinta(), "B");
        std::fs::remove_dir_all(&d).ok();
    }

    /// F-A3 din `cercetare/audit-qa-2026-11-18.md`: compararea ruleaza pe alt
    /// fir, iar formularul primeste tastatura cat ruleaza (e desenat blocat,
    /// dar nu e). Rezultatul se folosea de formularul **de acum**: o tasta in
    /// campul bazei si rezultatul se incarca sub referinta "", sesiunea se
    /// salveaza sub cheia "", iar directorul de deploy devine „_<data>".
    #[test]
    fn rezultatul_spune_cu_ce_referinta_a_rulat() {
        let d = std::env::temp_dir().join(format!("gmotion-fa3-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let cale = d.join("inst.json");
        instantaneu(&cale);
        let mut c = Conectare::noua();
        c.instantaneu = Some(cale.to_string_lossy().to_string());
        c.sursa.baza = "A".into();
        c.baza.baza = "B".into();

        let r = compara(&c).expect("compararea merge");
        assert_eq!(r.referinta, "A");
        // formularul se goleste dupa ce compararea a pornit (o tasta in camp,
        // sau Deconecteaza): rezultatul tot despre „A" vorbeste
        c.sursa.baza = String::new();
        assert_eq!(
            r.referinta, "A",
            "rezultatul nu are voie sa depinda de formularul de acum"
        );
        assert!(!r.referinta.is_empty());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn o_referinta_se_compara_cu_mai_multe_baze_deodata() {
        let d = std::env::temp_dir().join(format!("gmotion-multi-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let cale = d.join("inst.json");
        std::fs::write(
            &cale,
            r#"{"versiune":1,"server":"SRV","creat":"2026-11-17T10:00:00Z","nota":"","baze":{
                "Ref":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 1"}],
                "C1":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 2"}],
                "C2":[{"schema":"dbo","nume":"p","tip":"P","definitie":"CREATE PROC dbo.p AS\nSELECT 3"}],
                "C3":[]}}"#,
        )
        .unwrap();
        let mut c = Conectare::noua();
        c.instantaneu = Some(cale.to_string_lossy().to_string());
        c.sursa.baza = "Ref".into();
        c.baza.comuta_baza("C1");
        c.baza.comuta_baza("C2");

        let pasi = std::cell::RefCell::new(Vec::new());
        let r = compara_cu(&c, |p| {
            pasi.borrow_mut().push(p.baza.clone());
            true
        })
        .expect("compararea merge");
        assert_eq!(
            *pasi.borrow(),
            ["Ref", "C1", "C2"],
            "progresul spune ce baza se citeste, referinta intai"
        );
        assert_eq!(r.tinte.len(), 2);
        assert_eq!(r.prima_tinta(), "C1");
        // acelasi obiect apare o data pentru fiecare baza
        let baze: Vec<&str> = r.randuri.iter().map(|x| x.baza.as_str()).collect();
        assert_eq!(baze, ["C1", "C2"]);

        // „Anuleaza" opreste fara sa citeasca restul
        let citite = std::cell::Cell::new(0);
        let e = compara_cu(&c, |_| {
            citite.set(citite.get() + 1);
            citite.get() < 2
        })
        .map(|_| ())
        .expect_err("anulata");
        assert_eq!(e, ANULAT);
        assert_eq!(citite.get(), 2, "s-a oprit la a doua intrebare");

        // si dupa citire, in mijlocul compararii (P8)
        let intrebari = std::cell::Cell::new(0);
        let e = compara_cu_anulare(
            &c,
            |_| true,
            || {
                intrebari.set(intrebari.get() + 1);
                false
            },
        )
        .map(|_| ())
        .expect_err("anulata in comparare");
        assert_eq!(e, ANULAT);
        assert_eq!(intrebari.get(), 1, "la primul obiect");

        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn instantaneele_gasite_sunt_json_uri_reale_fara_duplicate() {
        // directorul curent al testelor e crate-ul, deci `referinta/` se vede
        // prin „..": exact una din caile cautate
        let gasite = instantanee_gasite();
        assert!(
            gasite.iter().all(|c| c.is_file()),
            "s-a propus ceva ce nu e fisier: {gasite:?}"
        );
        assert!(
            gasite
                .iter()
                .all(|c| c.extension().is_some_and(|x| x == "json")),
            "doar .json: {gasite:?}"
        );
        let mut unice = gasite.clone();
        unice.sort();
        unice.dedup();
        assert_eq!(unice.len(), gasite.len(), "fara duplicate: {gasite:?}");
        assert!(gasite.len() <= 40, "lista nu creste la nesfarsit");
        // fisierele de asteptari (liste JSON) nu sunt instantanee (GM-DB-007)
        assert!(
            !gasite
                .iter()
                .any(|c| c.to_string_lossy().ends_with(".asteptari.json")),
            "{gasite:?}"
        );
        // in repo exista cel putin instantaneul de previzualizare
        assert!(
            gasite.iter().any(|c| c.ends_with("previzualizare.json")),
            "instantaneul de previzualizare ar trebui gasit: {gasite:?}"
        );
    }

    #[test]
    fn compararea_offline_lucreaza_pe_doua_bucati_de_cod() {
        let e = compara_offline("", "SELECT 1", Limba::Romana)
            .map(|_| ())
            .expect_err("gol");
        assert_eq!(e, "lipeste cod in ambele panouri");
        let r = compara_offline(
            "CREATE PROC dbo.p AS\nSELECT 1",
            "CREATE PROC dbo.p AS\nSELECT 2",
            Limba::Romana,
        )
        .expect("merge");
        assert_eq!(r.randuri.len(), 1);
        assert_eq!(r.randuri[0].obiect, "dbo.cod_lipit");
        assert_eq!(r.prima_tinta(), "Lipit");
        // cod identic: nu apare niciun rand, ca la comparare normala
        let r = compara_offline("SELECT 1", "SELECT 1", Limba::Romana).expect("merge");
        assert!(r.randuri.is_empty());
    }

    #[test]
    fn durata_se_scrie_in_unitatea_potrivita() {
        assert_eq!(durata_scurta(-5, Limba::Romana), "0 secunde");
        assert_eq!(durata_scurta(1, Limba::Romana), "1 secunda");
        assert_eq!(durata_scurta(119, Limba::Romana), "119 secunde");
        assert_eq!(durata_scurta(120, Limba::Romana), "2 minute");
        assert_eq!(durata_scurta(3600, Limba::Romana), "60 de minute");
        assert_eq!(durata_scurta(7200, Limba::Romana), "2 ore");
        assert_eq!(durata_scurta(86_399, Limba::Romana), "23 de ore");
        assert_eq!(durata_scurta(86_400, Limba::Romana), "1 zi");
        assert_eq!(durata_scurta(172_800, Limba::Romana), "2 zile");
    }

    #[test]
    fn descrierea_serverului_sare_campurile_lipsa() {
        use gmotion_db::InfoServer;
        let i = InfoServer {
            editie: "SQL Server 2019 Standard".into(),
            sistem: "Windows Server 2019".into(),
            pornit_de_secunde: Some(90_000),
            nr_baze: Some(42),
        };
        assert_eq!(
            descrie_serverul(&i, Limba::Romana),
            "SQL Server 2019 Standard · Windows Server 2019 · pornit de 1 zi · 42 de baze"
        );
        let gol = InfoServer {
            editie: "SQL Server".into(),
            sistem: String::new(),
            pornit_de_secunde: None,
            nr_baze: None,
        };
        assert_eq!(descrie_serverul(&gol, Limba::Romana), "SQL Server");
    }

    #[test]
    fn testarea_pe_instantaneu_spune_de_unde_vin_obiectele() {
        let d = std::env::temp_dir().join(format!("gmotion-test-con-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let cale = d.join("inst.json");
        instantaneu(&cale);
        let mut c = Conectare::noua();
        c.instantaneu = Some(cale.to_string_lossy().to_string());
        let t = testeaza(&c, &c.sursa).expect("instantaneul se citeste");
        assert!(t.starts_with("instantaneu · "), "{t}");
        assert!(t.ends_with("2 baze"), "{t}");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn testarea_fara_server_spune_ce_lipseste_nu_incearca_sa_se_conecteze() {
        let c = Conectare::noua();
        assert_eq!(
            testeaza(&c, &c.sursa).expect_err("lipseste serverul"),
            "completeaza serverul"
        );
    }

    #[test]
    fn fara_nicio_baza_bifata_nu_e_nimic_de_comparat() {
        let mut c = Conectare::noua();
        c.instantaneu = Some("/nu/conteaza.json".into());
        c.sursa.baza = "Ref".into();
        assert_eq!(
            compara(&c).map(|_| ()).expect_err("nimic de comparat"),
            "nicio baza de comparat"
        );
    }

    #[test]
    fn un_instantaneu_inexistent_da_eroare_nu_conectat() {
        let mut c = Conectare::noua();
        c.instantaneu = Some("/nu/exista/inst.json".into());
        let (baze, stare) = citeste_bazele(&c, &c.sursa);
        assert!(baze.is_empty());
        assert!(!stare.e_conectat(), "{stare:?}");
        assert!(stare.text().contains("GM-DB-007"), "{}", stare.text());
    }

    #[test]
    fn parola_nu_apare_in_eroarea_de_conectare() {
        let mut c = Conectare::noua();
        c.sursa.server = "server.care.nu.exista.invalid".into();
        c.sursa.auth_windows = false;
        c.sursa.acreditari = Acreditari::noi("deploy_ro", "parola-secreta");
        let (_, stare) = citeste_bazele(&c, &c.sursa);
        assert!(!stare.e_conectat());
        assert!(
            !stare.text().contains("parola-secreta"),
            "parola in mesajul de eroare: {}",
            stare.text()
        );
    }
}
