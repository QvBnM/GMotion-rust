//! Generatorul de scripturi de deploy.
//!
//! **Se strica astfel:** produce SQL gresit, si se observa **dupa** ce a rulat
//! pe o baza. E partea cu cel mai mare cost al unei greseli din tot modulul, de
//! aceea are cele mai multe teste si nicio scurtatura.
//!
//! Portat din `genereaza_deployer`, `_scrie_deployer` si
//! `_aplica_create_or_alter` din `mostenire/app.py`. Deciziile pastrate:
//! obiectele `extra` nu se pot deploia (nu exista cod in referinta de adus),
//! iar primul `CREATE`/`ALTER` devine `CREATE OR ALTER`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::eroare::{Eroare, Rezultat};

/// Cum se imparte deploy-ul in fisiere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModImpartire {
    /// Tot intr-un singur fisier, toate bazele.
    Unul,
    /// Un fisier per baza. Implicit.
    #[default]
    PerBaza,
    /// Un fisier per obiect.
    PerObiect,
    /// Cate `n` obiecte per fisier.
    CateN(usize),
}

/// Un obiect de scris, cu codul final deja decis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeDeployat {
    pub obiect: String,
    pub tip: String,
    pub cod: String,
    /// Setarile cu care a fost creat obiectul in referinta. `None`: necunoscute
    /// (instantaneu vechi) — se scrie `ON`/`ON`, ca SSMS.
    pub setari: Option<gmotion_db::Setari>,
}

/// Un pas din scriptul de revenire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasRevenire {
    /// Obiectul exista in tinta: se reface exact cum e acum (codul si
    /// setarile **din tinta**).
    Reface(DeDeployat),
    /// Obiectul nu exista in tinta, deci deploy-ul il creeaza: revenirea il
    /// sterge. `cod` e codul care se deployeaza — din el se afla ordinea in
    /// care se pot sterge.
    Sterge(DeDeployat),
}

/// Ce s-a scris.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raport {
    pub director: PathBuf,
    pub fisiere: Vec<PathBuf>,
    /// Scripturile de revenire, in `<director>/revenire/`. Goala daca s-a
    /// scris fara revenire.
    pub revenire: Vec<PathBuf>,
    pub obiecte: usize,
    /// Obiecte marcate dar nedeployabile: cele `extra`.
    pub sarite: usize,
    /// Ce poate pierde deploy-ul fara sa se vada in script (F-D8). Goala daca
    /// s-a scris fara revenire: atunci nu se stie ce e in tinta.
    pub avertismente: Vec<Avertisment>,
}

/// Ceva ce `CREATE OR ALTER` pierde in tinta si scriptul nu poate reface.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Avertisment {
    /// View existent cu `SCHEMABINDING`: poate avea indecsi, iar `CREATE OR
    /// ALTER VIEW` ii sterge. Indecsii nu sunt in definitie, deci nu se stie
    /// sigur; se spune pentru fiecare view care **poate** avea.
    ViewPosibilIndexat { baza: String, obiect: String },
    /// Trigger existent: ordinea data cu `sp_settriggerorder` se pierde la
    /// `ALTER TRIGGER`.
    OrdineTrigger { baza: String, obiect: String },
    /// Codul care se deployeaza nu se inchide (`BEGIN` fara `END`, paranteze,
    /// siruri; `gmotion_core::structura`): de regula un text final stricat de
    /// o plasare gresita. Scriptul ar cadea pe server (etapa 26).
    StructuraStricata {
        baza: String,
        obiect: String,
        linie: usize,
    },
}

/// Avertismentele pentru codul care se deployeaza: ce nu se inchide.
pub fn avertismente_structura(pe_baza: &BTreeMap<String, Vec<DeDeployat>>) -> Vec<Avertisment> {
    let mut rez = Vec::new();
    for (baza, obiecte) in pe_baza {
        for o in obiecte {
            if let Some(p) = gmotion_core::structura::verifica(&o.cod).first() {
                rez.push(Avertisment::StructuraStricata {
                    baza: baza.clone(),
                    obiect: o.obiect.clone(),
                    linie: p.linie,
                });
            }
        }
    }
    rez
}

/// Avertismentele pentru obiectele care exista deja in tinta, din pasii de
/// revenire (care tin codul **din tinta**).
pub fn avertismente(revenire: &BTreeMap<String, Vec<PasRevenire>>) -> Vec<Avertisment> {
    let mut rez = Vec::new();
    for (baza, pasi) in revenire {
        for p in pasi {
            let PasRevenire::Reface(o) = p else { continue };
            match o.tip.trim() {
                "V" if cuvinte_de_cod(&o.cod)
                    .iter()
                    .any(|c| c.text.eq_ignore_ascii_case("SCHEMABINDING")) =>
                {
                    rez.push(Avertisment::ViewPosibilIndexat {
                        baza: baza.clone(),
                        obiect: o.obiect.clone(),
                    })
                }
                "TR" => rez.push(Avertisment::OrdineTrigger {
                    baza: baza.clone(),
                    obiect: o.obiect.clone(),
                }),
                _ => {}
            }
        }
    }
    rez.sort();
    rez
}

impl std::fmt::Display for Avertisment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Avertisment::ViewPosibilIndexat { baza, obiect } => write!(
                f,
                "{baza}: {obiect} e legat cu SCHEMABINDING; daca are indecsi, deploy-ul ii sterge si trebuie refacuti de mana"
            ),
            Avertisment::OrdineTrigger { baza, obiect } => write!(
                f,
                "{baza}: {obiect} e un trigger existent; ordinea data cu sp_settriggerorder se pierde si trebuie refacuta de mana"
            ),
            Avertisment::StructuraStricata { baza, obiect, linie } => write!(
                f,
                "{baza}: codul lui {obiect} nu se inchide (linia {linie}); scriptul va cadea pe server, verifica textul final"
            ),
        }
    }
}

/// Numele scurt al tipului, pentru numele de fisier.
fn tip_nume(t: &str) -> &'static str {
    match t.trim() {
        "P" => "PROC",
        "V" => "VIEW",
        "FN" | "TF" | "IF" => "FUNC",
        "TR" => "TRIGGER",
        _ => "OBJ",
    }
}

/// Reduce un nume la ceva folosibil ca nume de fisier, pastrand punctul din
/// `schema.obiect`. Doua nume diferite pot da acelasi slug (`Baza Mea` /
/// `Baza_Mea`); `NumeFisiere` le face unice.
fn slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches(|c| c == '_' || c == '.').to_string()
}

/// Numele pe care Windows le trateaza ca dispozitive, cu orice extensie.
const NUME_REZERVATE_WINDOWS: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Da nume de fisier unice in cadrul unei rulari. Pana la audit, `Baza Mea` si
/// `Baza_Mea` scriau amandoua `Baza_Mea.sql`, a doua peste prima, iar raportul
/// spunea ca s-au scris doua fisiere.
#[derive(Default)]
struct NumeFisiere {
    folosite: std::collections::HashSet<String>,
}

/// Cat de lung poate fi numele unui fisier, fara extensie.
///
/// Pe Windows, calea intreaga nu poate trece de 260 de caractere, iar numele
/// per obiect ajungea la ~400 (`NNN_<baza>_<TIP>_<schema>.<nume>`): deploy-ul
/// cadea cu `GM-MOD-003` **dupa** ce revenirea si cateva fisiere erau deja
/// scrise (F-D6 din auditul QA). Ce se taie primeste un hash scurt, ca doua
/// nume lungi care incep la fel sa nu ajunga acelasi fisier.
const LUNGIME_MAXIMA_NUME: usize = 100;

/// Numele taiat la `LUNGIME_MAXIMA_NUME`, cu hash scurt cand s-a taiat.
fn plafoneaza(nume: &str) -> String {
    if nume.chars().count() <= LUNGIME_MAXIMA_NUME {
        return nume.to_string();
    }
    let h = gmotion_core::calc_hash(nume);
    let pastrat: String = nume.chars().take(LUNGIME_MAXIMA_NUME - 9).collect();
    format!("{pastrat}_{}", &h[..8])
}

impl NumeFisiere {
    /// `baza` e partea dinaintea extensiei, deja trecuta prin `slug`.
    fn unic(&mut self, baza: &str, extensie: &str) -> String {
        let mut baza = if baza.is_empty() {
            "fara_nume".to_string()
        } else {
            plafoneaza(baza)
        };
        let tulpina = baza.split('.').next().unwrap_or("").to_ascii_uppercase();
        if NUME_REZERVATE_WINDOWS.contains(&tulpina.as_str()) {
            baza = format!("_{baza}");
        }
        let mut nume = format!("{baza}.{extensie}");
        let mut k = 2;
        // Windows nu deosebeste majusculele in numele de fisier.
        while !self.folosite.insert(nume.to_lowercase()) {
            nume = format!("{baza}_{k}.{extensie}");
            k += 1;
        }
        nume
    }
}

/// Un cuvant din codul propriu-zis (nu din string sau comentariu), cu pozitia
/// lui in octeti in textul original.
struct Cuvant<'a> {
    text: &'a str,
    start: usize,
    sfarsit: usize,
}

/// Cuvintele din segmentele de cod, in ordine. Pozitiile sunt in textul
/// original: nimic nu se recalculeaza pe o copie cu litere mici (asa dadea
/// panica pe `K`, al carui minuscul are alta lungime in octeti).
fn cuvinte_de_cod(cod: &str) -> Vec<Cuvant<'_>> {
    let e_cuvant = |c: char| c.is_alphanumeric() || matches!(c, '_' | '@' | '#' | '$');
    let mut rez = Vec::new();
    for seg in gmotion_core::lexer::segmente(cod) {
        if seg.fel != gmotion_core::lexer::Fel::Cod {
            continue;
        }
        let bucata = &cod[seg.start..seg.sfarsit];
        let mut start: Option<usize> = None;
        for (i, c) in bucata
            .char_indices()
            .chain(std::iter::once((bucata.len(), ' ')))
        {
            match (start, e_cuvant(c)) {
                (None, true) => start = Some(i),
                (Some(s), false) => {
                    rez.push(Cuvant {
                        text: &bucata[s..i],
                        start: seg.start + s,
                        sfarsit: seg.start + i,
                    });
                    start = None;
                }
                _ => {}
            }
        }
    }
    rez
}

/// Numele calificat scris imediat dupa tipul obiectului: `[start, sfarsit)`.
///
/// Sare peste comentarii (`CREATE PROCEDURE /* vechi */ dbo.p`) si accepta
/// partile in paranteze, in ghilimele sau simple. `None` daca nu urmeaza un
/// nume — atunci codul nu se rescrie, si garda din `verifica_inainte` decide.
fn span_nume(cod: &str, de_la: usize) -> Option<(usize, usize)> {
    use gmotion_core::lexer::{segmente, Fel};
    let e_cuvant = |c: char| c.is_alphanumeric() || matches!(c, '_' | '@' | '#' | '$');
    let mut start: Option<usize> = None;
    let mut sfarsit = 0usize;
    let mut astept_parte = true;
    for seg in segmente(cod) {
        if seg.sfarsit <= de_la {
            continue;
        }
        match seg.fel {
            Fel::ComentariuLinie | Fel::ComentariuBloc => continue,
            Fel::Sir => break,
            Fel::IdentificatorParanteze | Fel::IdentificatorGhilimele => {
                if !astept_parte {
                    break;
                }
                start.get_or_insert(seg.start);
                sfarsit = seg.sfarsit;
                astept_parte = false;
            }
            Fel::Cod => {
                let de = seg.start.max(de_la);
                let bucata = &cod[de..seg.sfarsit];
                let mut i = 0usize;
                while let Some(c) = bucata[i..].chars().next() {
                    if c.is_whitespace() {
                        i += c.len_utf8();
                    } else if c == '.' {
                        if astept_parte {
                            return None;
                        }
                        astept_parte = true;
                        i += 1;
                    } else if e_cuvant(c) {
                        if !astept_parte {
                            return start.map(|s| (s, sfarsit));
                        }
                        let j = bucata[i..]
                            .char_indices()
                            .find(|(_, c)| !e_cuvant(*c))
                            .map_or(bucata.len(), |(k, _)| i + k);
                        start.get_or_insert(de + i);
                        sfarsit = de + j;
                        astept_parte = false;
                        i = j;
                    } else {
                        return start.map(|s| (s, sfarsit));
                    }
                }
            }
        }
    }
    if astept_parte {
        return None;
    }
    start.map(|s| (s, sfarsit))
}

/// Partile unui nume calificat, cu litere mici si fara paranteze.
fn parti_nume(text: &str) -> Vec<String> {
    use gmotion_core::lexer::{segmente, Fel};
    let mut parti = vec![String::new()];
    for seg in segmente(text) {
        let bucata = &text[seg.start..seg.sfarsit];
        match seg.fel {
            Fel::IdentificatorParanteze => parti
                .last_mut()
                .expect("mereu cel putin o parte")
                .push_str(&bucata[1..bucata.len() - 1].replace("]]", "]")),
            Fel::IdentificatorGhilimele => parti
                .last_mut()
                .expect("mereu cel putin o parte")
                .push_str(&bucata[1..bucata.len() - 1].replace("\"\"", "\"")),
            _ => {
                for c in bucata.chars() {
                    if c == '.' {
                        parti.push(String::new());
                    } else if !c.is_whitespace() {
                        parti.last_mut().expect("mereu cel putin o parte").push(c);
                    }
                }
            }
        }
    }
    parti.iter().map(|p| p.to_lowercase()).collect()
}

/// Numele pe care il creeaza efectiv codul, daca se poate citi: ultimele doua
/// parti ale numelui de dupa `CREATE OR ALTER <tip>`, ca `schema.nume`. O
/// singura parte iese fara schema — adica obiectul ar ajunge in schema
/// implicita a celui care ruleaza scriptul, nu neaparat cea din plan.
fn numele_creat(cod: &str) -> Option<String> {
    let (_, tip_sfarsit) = capul(cod)?;
    let (s, f) = span_nume(cod, tip_sfarsit)?;
    let parti = parti_nume(&cod[s..f]);
    match parti.as_slice() {
        [] => None,
        [nume] => Some(nume.clone()),
        [.., schema, nume] => Some(format!("{schema}.{nume}")),
    }
}

/// `CREATE`/`ALTER <tip>` de rescris: tipul cu majuscule si unde se termina
/// cuvantul tipului. `None` daca nu exista niciunul in cod.
fn capul(cod: &str) -> Option<(Cap, usize)> {
    const TIPURI: [(&str, &str); 5] = [
        ("procedure", "PROCEDURE"),
        ("proc", "PROCEDURE"),
        ("view", "VIEW"),
        ("function", "FUNCTION"),
        ("trigger", "TRIGGER"),
    ];
    let cuv = cuvinte_de_cod(cod);
    for (i, c) in cuv.iter().enumerate() {
        let e_create = c.text.eq_ignore_ascii_case("create");
        if !e_create && !c.text.eq_ignore_ascii_case("alter") {
            continue;
        }
        let urm = |k: usize| cuv.get(i + k).map(|x| x.text);
        let deja = e_create
            && urm(1).is_some_and(|w| w.eq_ignore_ascii_case("or"))
            && urm(2).is_some_and(|w| w.eq_ignore_ascii_case("alter"));
        let pozitie_tip = if deja { i + 3 } else { i + 1 };
        let Some(tip) = cuv.get(pozitie_tip) else {
            continue;
        };
        let Some((_, mare)) = TIPURI
            .iter()
            .find(|(mic, _)| tip.text.eq_ignore_ascii_case(mic))
        else {
            if deja {
                return None;
            }
            continue;
        };
        return Some((
            Cap {
                start: c.start,
                tip: mare,
                deja,
            },
            tip.sfarsit,
        ));
    }
    None
}

struct Cap {
    start: usize,
    tip: &'static str,
    deja: bool,
}

/// Transforma primul `CREATE` sau `ALTER <tip>` in `CREATE OR ALTER`, si
/// rescrie numele din definitie la `obiect` (`schema.nume` din plan).
///
/// Un script care face `CREATE` pe un obiect existent esueaza; unul care face
/// `ALTER` pe un obiect inexistent, la fel. `CREATE OR ALTER` merge in ambele
/// cazuri, deci acelasi script poate fi rulat si pe o baza careia lipseste
/// obiectul si pe una care il are intr-o versiune veche.
///
/// Numele din `definition` nu e numele obiectului: dupa `sp_rename` sau
/// `ALTER SCHEMA TRANSFER`, SQL Server pastreaza textul original. Deploy-ul
/// crea atunci **alt** obiect decat cel din plan, iar revenirea stergea unul
/// pe care deploy-ul nu-l crease (F-D4 din auditul QA). Se rescrie la numele
/// din plan, care vine din `sys.objects`; o schema lipsa (`CREATE PROC p`) la
/// fel, altfel obiectul ar ajunge in schema implicita a celui care ruleaza.
///
/// Se uita **doar la cod**: un `CREATE` dintr-un comentariu (`-- vechi: CREATE
/// PROCEDURE ...`) sau dintr-un string nu se atinge. Pana la audit, regula
/// lovea primul `CREATE` din text, deci rescria comentariul si lasa `CREATE`-ul
/// adevarat, iar scriptul cadea pe orice baza care avea deja obiectul.
/// `CREATE TABLE` nu intra: `CREATE OR ALTER TABLE` nu exista in T-SQL.
pub fn aplica_create_or_alter(cod: &str, obiect: &str) -> String {
    let Some((cap, tip_sfarsit)) = capul(cod) else {
        return cod.to_string();
    };
    // numele intai: rescris de la coada, pozitiile capului raman valabile
    let mut rez = cod.to_string();
    if let Some((s, f)) = span_nume(cod, tip_sfarsit) {
        if !acelasi_nume(&parti_nume(&cod[s..f]), obiect) {
            rez = format!("{}{}{}", &cod[..s], nume_calificat(obiect), &cod[f..]);
        }
    }
    if !cap.deja {
        rez = format!(
            "{}CREATE OR ALTER {}{}",
            &rez[..cap.start],
            cap.tip,
            &rez[tip_sfarsit..]
        );
    }
    rez
}

/// `]` dublat: singurul caracter care poate iesi dintr-un `[identificator]`.
/// Pana la audit, o baza numita `X]; DROP DATABASE Prod; --` scria in script
/// `USE [X]; DROP DATABASE Prod; --];`.
fn paranteze(nume: &str) -> String {
    format!("[{}]", nume.replace(']', "]]"))
}

/// Un nume care ajunge in script (in `USE [..]` sau intr-un comentariu `--`)
/// nu are voie sa contina caractere de control: un rand nou incheie
/// comentariul, iar o linie `GO` aparuta asa taie batch-ul si face restul
/// executabil.
fn verifica_nume(nume: &str) -> Rezultat<()> {
    // un nume gol (plan `.json` scris de mana) dadea `CREATE OR ALTER
    // PROCEDURE []`, adica un script invalid la rulare in loc de un refuz
    // inainte de scriere, care e regula zonei (COSMETIC, auditul 2026-11-23)
    if nume.trim().is_empty() || nume.chars().any(char::is_control) {
        return Err(Eroare::NumeNesigur {
            nume: nume.to_string(),
        });
    }
    Ok(())
}

/// Cuvantul de dupa `CREATE` care corespunde tipului din `sys.objects`, daca
/// tipul e unul pe care il deployam.
fn tip_create(tip: &str) -> Option<&'static str> {
    match tip.trim().to_ascii_uppercase().as_str() {
        "P" | "PC" => Some("PROCEDURE"),
        "V" => Some("VIEW"),
        "FN" | "IF" | "TF" | "FS" | "FT" => Some("FUNCTION"),
        "TR" | "TA" => Some("TRIGGER"),
        _ => None,
    }
}

/// Codul unui obiect incepe (dupa comentarii) cu `CREATE [OR ALTER] <tip>`
/// sau `ALTER <tip>`, cu tipul din plan. Un text final scris de mana putea
/// fi `DELETE FROM dbo.Facturi`: se scria ca batch dupa `USE` si se rula
/// (B2 din `cercetare/audit-qa-2026-11-22.md`); o procedura care devenise
/// view se scria ca `CREATE OR ALTER VIEW` peste procedura (F-D3).
fn verifica_capul(o: &DeDeployat) -> Rezultat<()> {
    let cuv = cuvinte_de_cod(&o.cod);
    let asteptat = tip_create(&o.tip);
    let gasit: Vec<&str> = cuv.iter().take(4).map(|c| c.text).collect();
    let e = |w: Option<&&str>, x: &str| w.is_some_and(|w| w.eq_ignore_ascii_case(x));
    let pozitie_tip = if e(gasit.first(), "create") {
        if e(gasit.get(1), "or") && e(gasit.get(2), "alter") {
            Some(3)
        } else {
            Some(1)
        }
    } else if e(gasit.first(), "alter") {
        Some(1)
    } else {
        None
    };
    let tip_gasit = pozitie_tip.and_then(|k| gasit.get(k)).map(|t| {
        if t.eq_ignore_ascii_case("proc") {
            "PROCEDURE".to_string()
        } else {
            t.to_ascii_uppercase()
        }
    });
    // un tip pe care nu-l deployam (`plan.json` scris de mana cu `"tip":
    // "ZZ"`) nu trece: revenirea n-ar sti sa-l stearga (auditul 2026-11-23)
    let bun = match (asteptat, &tip_gasit) {
        (Some(a), Some(t)) => a == t,
        _ => false,
    };
    if bun {
        return Ok(());
    }
    Err(Eroare::CodFaraCreate {
        obiect: o.obiect.clone(),
        asteptat: asteptat.unwrap_or("<tip>").to_string(),
        gasit: if gasit.is_empty() {
            "nimic".to_string()
        } else {
            format!("`{}`", gasit.join(" "))
        },
    })
}

/// Un text final care n-ar trece de deploy, spus **cand se scrie**, nu abia la
/// deploy: gol, fara `CREATE <tip>` la inceput, sau cu o linie `GO`. Aceleasi
/// reguli ca verificarea dinaintea scrierii scripturilor.
pub fn verifica_text_final(obiect: &str, tip: &str, text: &str) -> Rezultat<()> {
    if text.trim().is_empty() {
        return Err(Eroare::TextFinalGol {
            obiect: obiect.to_string(),
        });
    }
    verifica_capul(&DeDeployat {
        obiect: obiect.to_string(),
        tip: tip.to_string(),
        cod: text.to_string(),
        setari: None,
    })?;
    let linii = gmotion_core::lexer::linii_go(text);
    if !linii.is_empty() {
        return Err(Eroare::GoInDefinitie {
            obiect: obiect.to_string(),
            linii: linii
                .iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        });
    }
    Ok(())
}

/// Tot ce poate face un script periculos se verifica **inainte** de a scrie
/// ceva: un deploy partial e mai rau decat niciunul.
fn verifica_inainte(referinta: &str, pe_baza: &BTreeMap<String, Vec<DeDeployat>>) -> Rezultat<()> {
    verifica_nume(referinta)?;
    for (baza, obiecte) in pe_baza {
        verifica_nume(baza)?;
        // un plan `.json` scris de mana poate repeta un pas: obiectul ar fi
        // scris de doua ori, iar revenirea ar avea doua DROP-uri pentru el
        // (F-D12 din auditul QA)
        let mut vazute: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for o in obiecte {
            if !vazute.insert(o.obiect.as_str()) {
                return Err(Eroare::ObiectRepetat {
                    obiect: o.obiect.clone(),
                    baza: baza.clone(),
                });
            }
        }
        for o in obiecte {
            verifica_nume(&o.obiect)?;
            if o.cod.trim().is_empty() {
                return Err(Eroare::ObiectCriptat {
                    obiect: o.obiect.clone(),
                });
            }
            // `dbo.p;1` e o procedura numerotata: `CREATE OR ALTER` nu le
            // accepta, deci scriptul ar cadea la rulare (F-D10)
            if o.obiect.contains(';') {
                return Err(Eroare::NumeNesigur {
                    nume: o.obiect.clone(),
                });
            }
            verifica_capul(o)?;
            // ultima garda pentru F-D4: `aplica_create_or_alter` rescrie
            // numele, dar un `DeDeployat` construit altfel (plan `.json` scris
            // de mana, alt apelant al modulului) poate avea in cod un nume pe
            // care nu l-a vazut nimeni. Un cod fara `CREATE <tip>` nu se
            // verifica: acolo nu se poate sti ce creeaza.
            if let Some(gasit) = numele_creat(&o.cod) {
                if gasit != o.obiect && parti_nume(&gasit) != parti_nume(&o.obiect) {
                    return Err(Eroare::NumeDinDefinitie {
                        obiect: o.obiect.clone(),
                        gasit,
                    });
                }
            }
            let linii = gmotion_core::lexer::linii_go(&o.cod);
            if !linii.is_empty() {
                return Err(Eroare::GoInDefinitie {
                    obiect: o.obiect.clone(),
                    linii: linii
                        .iter()
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                });
            }
        }
    }
    Ok(())
}

/// Garda de versiune, primul lucru din fiecare script.
///
/// `CREATE OR ALTER` exista de la SQL Server 2016 SP1 (13.0.4001). Pe un
/// server mai vechi fiecare batch cadea cu o eroare de sintaxa greu de inteles;
/// garda opreste executia inainte de orice `USE` si spune de ce. `SET NOEXEC
/// ON` face ca restul batch-urilor sa fie doar compilate, si in `sqlcmd` si in
/// SSMS (care altfel continua dupa o eroare). Azure SQL Database si Managed
/// Instance (`EngineEdition` 5 si 8) raporteaza versiunea 12.x, dar au
/// `CREATE OR ALTER`, deci sunt exceptate. Gasit la audit.
const GARDA_VERSIUNE: &str = "\
DECLARE @gmotion_versiune NVARCHAR(128) = CAST(SERVERPROPERTY('ProductVersion') AS NVARCHAR(128));
IF CAST(SERVERPROPERTY('EngineEdition') AS INT) NOT IN (5, 8)
   AND (CAST(PARSENAME(@gmotion_versiune, 4) AS INT) < 13
        OR (CAST(PARSENAME(@gmotion_versiune, 4) AS INT) = 13
            AND CAST(PARSENAME(@gmotion_versiune, 2) AS INT) < 4001))
BEGIN
    RAISERROR(N'GMotion: scriptul foloseste CREATE OR ALTER, care cere SQL Server 2016 SP1 (13.0.4001) sau mai nou; serverul are %s. Nu se executa nimic.', 16, 1, @gmotion_versiune);
    SET NOEXEC ON;
END
GO
";

/// La sfarsitul fiecarui fisier: sesiunea se intoarce la normal, ca o garda
/// declansata sa nu lase `NOEXEC` pornit in fereastra SSMS. Se redau si
/// `ANSI_NULLS` / `QUOTED_IDENTIFIER`: fiecare obiect si le pune pe ale lui,
/// iar in fereastra SSMS a omului ramaneau cele ale ultimului obiect
/// deployat, eventual `OFF` (COSMETIC, auditul 2026-11-23).
const SFARSIT_SCRIPT: &str =
    "SET NOEXEC OFF;\nGO\nSET ANSI_NULLS ON;\nSET QUOTED_IDENTIFIER ON;\nGO\n";

/// Liniile scrise de GMotion, cu `\r\n`.
///
/// Idempotenta: un `\r\n` deja acolo nu se dubleaza. **Nu se cheama pe un
/// text care contine codul unui obiect** — acolo capetele de linie sunt ale
/// utilizatorului si se pastreaza (vezi C5).
fn crlf(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / 16);
    let mut prec_r = false;
    for c in text.chars() {
        if c == '\n' && !prec_r {
            out.push('\r');
        }
        prec_r = c == '\r';
        out.push(c);
    }
    out
}

/// Scrie un script pe disc: **cu BOM UTF-8**.
///
/// `sqlcmd` fara `-f 65001` citeste un fisier fara BOM in pagina de cod OEM,
/// deci un literal `N'...'` cu caractere non-ASCII ajungea mojibake **in codul
/// deployat**; SSMS intreaba de capetele de linie amestecate. BOM-ul rezolva
/// amandoua (F-D2 din auditul QA).
fn scrie_script(cale: &Path, continut: &str) -> Rezultat<()> {
    let mut octeti = Vec::with_capacity(continut.len() + 3);
    octeti.extend_from_slice("\u{feff}".as_bytes());
    octeti.extend_from_slice(continut.as_bytes());
    std::fs::write(cale, octeti).map_err(|e| Eroare::ScriereDeploy {
        cale: cale.display().to_string(),
        detaliu: e.to_string(),
    })
}

/// `USE [baza]` plus verificarea ca am ajuns acolo.
///
/// Daca `USE` cade (baza offline, redenumita, fara drept), eroarea 911
/// opreste doar batch-ul lui: SSMS si `sqlcmd` fara `-b` ruleaza apoi toate
/// `CREATE OR ALTER` pe baza **curenta** — master, referinta, sau, cu „un
/// singur fisier", baza anterioara. Deci se verifica explicit (F-D1 din
/// auditul QA).
fn du_te_in_baza(baza: &str) -> String {
    // numele intra ca **argument** al lui RAISERROR, nu in mesaj: un `%` din
    // numele bazei ar fi altfel citit ca specificator de format
    let nume = baza.replace('\'', "''");
    format!(
        "USE {};\nGO\n\
         IF DB_NAME() <> N'{nume}'\n\
         BEGIN\n    \
             RAISERROR(N'GMotion: nu s-a putut intra in baza %s; nu se executa nimic.', 16, 1, N'{nume}');\n    \
             SET NOEXEC ON;\n\
         END\nGO\n\n",
        paranteze(baza)
    )
}

/// Avertismentul pus in capul fiecarui fisier cand deploy-ul e impartit.
const AVERTISMENT_IMPARTIT: &str = "\
-- ATENTIE: deploy impartit in mai multe fisiere. Oprirea dupa un obiect cazut
-- tine doar pana la capatul acestui fisier. Ruleaza-le in ordinea numelor si
-- cu oprire la eroare (sqlcmd -b), altfel un obiect cazut la mijloc lasa
-- restul sa se execute.
";

/// Antetul unui script. `mai_multe_fisiere`: deploy-ul e impartit, deci
/// oprirea dintre obiecte (`SET NOEXEC ON`) **nu trece de marginea
/// fisierului** — `SET NOEXEC OFF` de la capat o stinge. Se spune in script,
/// fiindca omul alege ordinea in care le ruleaza (A-7 din auditul
/// 2026-11-23); altfel un obiect cazut la mijloc lasa restul sa ruleze.
fn antet(baza: &str, referinta: &str, acum: &str, mai_multe_fisiere: bool) -> String {
    let avertisment = if mai_multe_fisiere {
        AVERTISMENT_IMPARTIT
    } else {
        ""
    };
    crlf(&format!(
        "-- DEPLOYER {baza} | referinta: {referinta} | {acum}\n{avertisment}{GARDA_VERSIUNE}{}",
        du_te_in_baza(baza)
    ))
}

/// Un obiect in script: setarile de sesiune intr-un batch propriu, apoi codul.
///
/// `SET ANSI_NULLS` si `SET QUOTED_IDENTIFIER` se aplica la **compilarea**
/// obiectului si raman legate de el: `sqlcmd` porneste cu `QUOTED_IDENTIFIER
/// OFF`, deci un deploy fara ele facea din `"coloana"` un string si strica
/// procedurile folosite cu indecsi filtrati, view-uri indexate sau XML. Se
/// refac exact cum erau in referinta. Gasit la audit; v3.4 nu le scria.
fn bloc(o: &DeDeployat) -> String {
    let on = |b: bool| if b { "ON" } else { "OFF" };
    let (ansi, qi, nota) = match o.setari {
        Some(s) => (s.ansi_nulls, s.quoted_identifier, ""),
        None => (
            true,
            true,
            " -- setari necunoscute (instantaneu vechi): ON, ca SSMS",
        ),
    };
    // codul obiectului se lipeste **neatins**: capetele lui de linie sunt ale
    // utilizatorului, inclusiv in interiorul sirurilor
    let mut text = crlf(&format!(
        "-- ═══ {} ═══\nSET ANSI_NULLS {};\nSET QUOTED_IDENTIFIER {};{}\nGO\n",
        o.obiect,
        on(ansi),
        on(qi),
        nota,
    ));
    text.push_str(&o.cod);
    text.push_str(&crlf("\nGO\n"));
    text.push_str(&opreste_daca_a_cazut(&o.obiect));
    text
}

/// Dupa fiecare obiect: daca batch-ul lui a cazut, restul scriptului nu se
/// mai executa (F-D7 din auditul QA).
///
/// Fara asta, SSMS si `sqlcmd` fara `-b` trec mai departe, iar un obiect care
/// cade lasa un deploy partial: obiectele de dupa el ajung in baza, cele care
/// depindeau de el cad si ele. Tiparul e cel folosit de uneltele comerciale
/// de comparare; `@@ERROR` se citeste in primul batch de dupa `GO`.
/// **Neprobat inca pe SQL Server** (proba e in `STARE.md`). Numele intra ca
/// argument al lui `RAISERROR`, ca un `%` din el sa nu fie format.
fn opreste_daca_a_cazut(obiect: &str) -> String {
    crlf(&format!(
        "IF @@ERROR <> 0\n\
         BEGIN\n    \
             RAISERROR(N'GMotion: %s a cazut; restul scriptului nu se mai executa.', 16, 1, N'{}');\n    \
             SET NOEXEC ON;\n\
         END\nGO\n\n",
        obiect.replace('\'', "''")
    ))
}

/// Numele (cu puncte) la care face referire codul: `dbo.f`, `[dbo].[v]`, `p`.
/// Doar din cod — un nume dintr-un comentariu sau string nu e o dependenta.
/// Fiecare nume e lista partilor, cu litere mici si fara paranteze.
fn nume_referite(cod: &str) -> Vec<Vec<String>> {
    use gmotion_core::lexer::{segmente, Fel};
    let e_cuvant = |c: char| c.is_alphanumeric() || matches!(c, '_' | '@' | '#' | '$');
    let mut nume: Vec<Vec<String>> = Vec::new();
    let mut parti: Vec<String> = Vec::new();
    let mut cuvant = String::new();
    let inchide_cuvant = |cuvant: &mut String, parti: &mut Vec<String>| {
        if !cuvant.is_empty() {
            parti.push(std::mem::take(cuvant));
        }
    };
    let inchide_nume = |parti: &mut Vec<String>, nume: &mut Vec<Vec<String>>| {
        if !parti.is_empty() {
            nume.push(std::mem::take(parti));
        }
    };
    for seg in segmente(cod) {
        let text = &cod[seg.start..seg.sfarsit];
        match seg.fel {
            Fel::Cod => {
                for c in text.chars() {
                    if e_cuvant(c) {
                        cuvant.extend(c.to_lowercase());
                    } else if c == '.' {
                        inchide_cuvant(&mut cuvant, &mut parti);
                    } else {
                        inchide_cuvant(&mut cuvant, &mut parti);
                        inchide_nume(&mut parti, &mut nume);
                    }
                }
            }
            Fel::IdentificatorParanteze => {
                inchide_cuvant(&mut cuvant, &mut parti);
                let interior = text
                    .strip_prefix('[')
                    .and_then(|s| s.strip_suffix(']'))
                    .unwrap_or(text)
                    .replace("]]", "]");
                parti.push(interior.to_lowercase());
            }
            // `"dbo"."f"`: tot un nume (cu QUOTED_IDENTIFIER ON, implicitul);
            // inainte era separator, deci dependenta se pierdea din ordinea
            // de deploy (F-D11)
            Fel::IdentificatorGhilimele => {
                inchide_cuvant(&mut cuvant, &mut parti);
                let interior = text
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .unwrap_or(text)
                    .replace("\"\"", "\"");
                parti.push(interior.to_lowercase());
            }
            _ => {
                inchide_cuvant(&mut cuvant, &mut parti);
                inchide_nume(&mut parti, &mut nume);
            }
        }
    }
    inchide_cuvant(&mut cuvant, &mut parti);
    inchide_nume(&mut parti, &mut nume);
    nume
}

/// Pentru fiecare obiect `(schema.nume, cod)`, celelalte obiecte din lista
/// pe care le foloseste codul lui (indici, fara repetitii, fara el insusi).
///
/// Singurul loc unde un nume din cod se leaga de un obiect: ordinea de deploy
/// si impactul estimat il folosesc pe acesta. Numele cu schema se cauta exact;
/// cel scurt, doar daca e unic in lista (un nume scurt ambiguu nu se leaga).
pub(crate) fn referinte(obiecte: &[(&str, &str)]) -> Vec<Vec<usize>> {
    use std::collections::HashMap;
    let mut dupa_nume_complet: HashMap<String, usize> = HashMap::new();
    let mut dupa_nume_scurt: HashMap<String, Option<usize>> = HashMap::new();
    for (i, (nume, _)) in obiecte.iter().enumerate() {
        let complet = nume.to_lowercase();
        let scurt = complet.rsplit('.').next().unwrap_or(&complet).to_string();
        dupa_nume_complet.insert(complet, i);
        dupa_nume_scurt
            .entry(scurt)
            .and_modify(|v| *v = None) // nume scurt ambiguu: nu se foloseste
            .or_insert(Some(i));
    }
    obiecte
        .iter()
        .enumerate()
        .map(|(i, (_, cod))| {
            let mut deps: Vec<usize> = Vec::new();
            for parti in nume_referite(cod) {
                let gasit = if parti.len() >= 2 {
                    dupa_nume_complet
                        .get(&format!(
                            "{}.{}",
                            parti[parti.len() - 2],
                            parti[parti.len() - 1]
                        ))
                        .copied()
                } else {
                    dupa_nume_scurt.get(&parti[0]).copied().flatten()
                };
                if let Some(j) = gasit.filter(|&j| j != i) {
                    if !deps.contains(&j) {
                        deps.push(j);
                    }
                }
            }
            deps
        })
        .collect()
}

/// Ordinea in care se scriu obiectele unei baze: fiecare dupa obiectele din
/// acelasi deploy pe care le foloseste.
///
/// Un view sau o functie care foloseste un obiect inca inexistent nu se
/// compileaza; v3.4 scria obiectele alfabetic, deci un view `aView` peste o
/// functie `zFunc` cadea la primul deploy pe o baza unde lipseau amandoua.
/// Dependentele se iau din cod (prin lexer), comparate cu numele obiectelor
/// din deploy — nu din `sys.sql_expression_dependencies`, ca sa mearga si din
/// instantanee. La egalitate: functii, view-uri, proceduri, triggere, apoi
/// ordinea primita. Un ciclu nu pierde obiecte: cele ramase se adauga la
/// final, in aceeasi ordine, cu avertisment.
pub fn ordine_dupa_dependente(obiecte: &[DeDeployat]) -> Vec<usize> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    let rang = |tip: &str| match tip.trim() {
        "FN" | "IF" | "TF" => 0u8,
        "V" => 1,
        "P" => 2,
        "TR" => 3,
        _ => 2,
    };
    let n = obiecte.len();
    let perechi: Vec<(&str, &str)> = obiecte
        .iter()
        .map(|o| (o.obiect.as_str(), o.cod.as_str()))
        .collect();
    let mut urmasi: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut intrari = vec![0usize; n];
    for (i, deps) in referinte(&perechi).into_iter().enumerate() {
        for j in deps {
            urmasi[j].push(i);
            intrari[i] += 1;
        }
    }

    let mut gata: BinaryHeap<Reverse<(u8, usize)>> = (0..n)
        .filter(|&i| intrari[i] == 0)
        .map(|i| Reverse((rang(&obiecte[i].tip), i)))
        .collect();
    let mut ordine = Vec::with_capacity(n);
    while let Some(Reverse((_, i))) = gata.pop() {
        ordine.push(i);
        for &k in &urmasi[i] {
            intrari[k] -= 1;
            if intrari[k] == 0 {
                gata.push(Reverse((rang(&obiecte[k].tip), k)));
            }
        }
    }
    if ordine.len() < n {
        let mut pus = vec![false; n];
        for &i in &ordine {
            pus[i] = true;
        }
        let mut ramase: Vec<usize> = (0..n).filter(|&i| !pus[i]).collect();
        ramase.sort_by_key(|&i| (rang(&obiecte[i].tip), i));
        tracing::warn!(
            target: "gmotion::modul",
            obiecte = ?ramase.iter().map(|&i| obiecte[i].obiect.as_str()).collect::<Vec<_>>(),
            "dependente circulare: obiectele raman in ordinea primita"
        );
        ordine.extend(ramase);
    }
    ordine
}

/// Directorul de iesire trebuie sa nu existe sau sa fie gol. Pana la audit,
/// fisierele unei rulari vechi ramaneau langa cele noi, iar rulate in ordine
/// alfabetica aplicau o versiune veche dupa una noua.
fn pregateste_director(director: &Path) -> Rezultat<()> {
    let eroare = |e: std::io::Error| Eroare::ScriereDeploy {
        cale: director.display().to_string(),
        detaliu: e.to_string(),
    };
    match std::fs::read_dir(director) {
        Ok(mut intrari) => {
            if intrari.next().is_some() {
                return Err(Eroare::DirectorNegol {
                    cale: director.display().to_string(),
                });
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(director).map_err(eroare)
        }
        Err(e) => Err(eroare(e)),
    }
}

/// Scrie scripturile. `acum` e dat din afara, nu citit din ceas: altfel
/// rezultatul n-ar fi testabil.
pub fn scrie(
    director: &Path,
    referinta: &str,
    pe_baza: &BTreeMap<String, Vec<DeDeployat>>,
    mod_impartire: ModImpartire,
    sarite: usize,
    acum: &str,
) -> Rezultat<Raport> {
    if pe_baza.is_empty() {
        return Err(Eroare::NimicDeDeployat);
    }
    verifica_inainte(referinta, pe_baza)?;
    pregateste_director(director)?;
    in_director_temporar(director, |lucru| {
        scrie_deploy(lucru, referinta, pe_baza, mod_impartire, sarite, acum)
    })
}

/// Scrie intr-un `<director>.tmp` si il muta la final, cu tot ce are in el.
///
/// O eroare la al optulea fisier lasa in urma sapte fisiere si o revenire,
/// iar reluarea cade cu `GM-MOD-010` („directorul nu e gol") — adica omul
/// ramane cu un deploy pe jumatate si fara drum inapoi (F-D6 din auditul QA).
/// Asa, ori apare directorul intreg, ori nu apare deloc.
fn in_director_temporar(
    director: &Path,
    f: impl FnOnce(&Path) -> Rezultat<Raport>,
) -> Rezultat<Raport> {
    let eroare = |cale: &Path, e: std::io::Error| Eroare::ScriereDeploy {
        cale: cale.display().to_string(),
        detaliu: e.to_string(),
    };
    // numele directorului de lucru se **adauga** la cel cerut si e unic pe
    // proces; `create_dir` (nu `_all`) refuza unul existent. Inainte era
    // `with_extension("tmp")`, sters recursiv fara intrebare: `-d proiect`
    // stergea `proiect.tmp`, iar `-d deploy_2026.11.23` stergea
    // `deploy_2026.11.tmp` (N2 din auditul 2026-11-23)
    static CONTOR: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = CONTOR.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut nume = director.file_name().unwrap_or_default().to_os_string();
    nume.push(format!(".gmotion-lucru-{}-{n}", std::process::id()));
    let lucru = director.with_file_name(nume);
    if let Some(p) = lucru.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(p).map_err(|e| eroare(p, e))?;
    }
    std::fs::create_dir(&lucru).map_err(|e| eroare(&lucru, e))?;
    let raport = match f(&lucru) {
        Ok(r) => r,
        Err(e) => {
            std::fs::remove_dir_all(&lucru).ok();
            return Err(e);
        }
    };
    // `pregateste_director` l-a creat gol (sau era gol): pe Windows
    // redenumirea peste un director existent cade, deci se scoate intai
    std::fs::remove_dir(director).ok();
    if let Err(e) = std::fs::rename(&lucru, director) {
        std::fs::remove_dir_all(&lucru).ok();
        return Err(eroare(director, e));
    }
    // caile din raport au fost scrise in directorul de lucru
    let muta = |c: &Path| -> std::path::PathBuf {
        c.strip_prefix(&lucru)
            .map(|rest| director.join(rest))
            .unwrap_or_else(|_| c.to_path_buf())
    };
    Ok(Raport {
        director: director.to_path_buf(),
        fisiere: raport.fisiere.iter().map(|c| muta(c)).collect(),
        revenire: raport.revenire.iter().map(|c| muta(c)).collect(),
        ..raport
    })
}

/// Ca [`scrie`], dar **intai** scriptul de revenire, in
/// `<director>/revenire/<baza>.sql`, apoi deploy-ul.
///
/// Totul se verifica inainte de primul fisier: daca revenirea nu se poate
/// scrie (o definitie actuala cu `GO`, un nume nesigur), nu se scrie nici
/// deploy-ul — un deploy fara cale de intoarcere e exact ce trebuie evitat.
/// Revenirea sta intr-un subdirector ca sa nu fie rulata din greseala odata
/// cu `*.sql` din director.
#[allow(clippy::too_many_arguments)]
pub fn scrie_cu_revenire(
    director: &Path,
    referinta: &str,
    pe_baza: &BTreeMap<String, Vec<DeDeployat>>,
    revenire: &BTreeMap<String, Vec<PasRevenire>>,
    mod_impartire: ModImpartire,
    sarite: usize,
    acum: &str,
) -> Rezultat<Raport> {
    if pe_baza.is_empty() {
        return Err(Eroare::NimicDeDeployat);
    }
    verifica_inainte(referinta, pe_baza)?;
    let de_refacut: BTreeMap<String, Vec<DeDeployat>> = revenire
        .iter()
        .map(|(baza, pasi)| {
            let v = pasi
                .iter()
                .filter_map(|p| match p {
                    PasRevenire::Reface(o) => Some(o.clone()),
                    PasRevenire::Sterge(_) => None,
                })
                .collect();
            (baza.clone(), v)
        })
        .collect();
    verifica_inainte(referinta, &de_refacut)?;
    for pasi in revenire.values() {
        for p in pasi {
            if let PasRevenire::Sterge(o) = p {
                verifica_nume(&o.obiect)?;
            }
        }
    }
    pregateste_director(director)?;
    in_director_temporar(director, |lucru| {
        scrie_totul(
            lucru,
            referinta,
            pe_baza,
            revenire,
            mod_impartire,
            sarite,
            acum,
        )
    })
}

/// Revenirea si deploy-ul, in directorul dat (de lucru).
#[allow(clippy::too_many_arguments)]
fn scrie_totul(
    director: &Path,
    referinta: &str,
    pe_baza: &BTreeMap<String, Vec<DeDeployat>>,
    revenire: &BTreeMap<String, Vec<PasRevenire>>,
    mod_impartire: ModImpartire,
    sarite: usize,
    acum: &str,
) -> Rezultat<Raport> {
    let dir_revenire = director.join("revenire");
    std::fs::create_dir_all(&dir_revenire).map_err(|e| Eroare::ScriereDeploy {
        cale: dir_revenire.display().to_string(),
        detaliu: e.to_string(),
    })?;
    let mut nume_fisiere = NumeFisiere::default();
    let mut scrise = Vec::new();
    for (baza, pasi) in revenire {
        let cale = dir_revenire.join(nume_fisiere.unic(&slug(baza), "sql"));
        scrie_script(&cale, &text_revenire(baza, referinta, pasi, acum)).map_err(|e| {
            Eroare::ScriereDeploy {
                cale: cale.display().to_string(),
                detaliu: e.to_string(),
            }
        })?;
        scrise.push(cale);
    }
    tracing::info!(
        target: "gmotion::modul",
        director = %dir_revenire.display(), fisiere = scrise.len(),
        "scripturi de revenire scrise"
    );

    let mut raport = scrie_deploy(director, referinta, pe_baza, mod_impartire, sarite, acum)?;
    raport.revenire = scrise;
    raport.avertismente = avertismente(revenire);
    raport.avertismente.extend(avertismente_structura(pe_baza));
    Ok(raport)
}

/// `[schema].[nume]` din `schema.nume`. Se taie la primul punct: o schema cu
/// punct in nume ar iesi gresit, dar `Baza` nu pastreaza partile separat.
/// Numele din definitie (pe parti, din paranteze) e `obiect` (`schema.nume`)?
/// O schema cu punct (`[my.s].[x]`, deci `my.s.x`) nu se mai desparte gresit
/// la primul punct: era rescrisa in `[my].[s.x]`, alt obiect (auditul
/// 2026-11-23).
fn acelasi_nume(parti_def: &[String], obiect: &str) -> bool {
    if parti_def == parti_nume(obiect).as_slice() {
        return true;
    }
    match parti_def {
        [.., schema, nume] => format!("{schema}.{nume}") == obiect.to_lowercase(),
        _ => false,
    }
}

/// Numele calificat pentru `DROP`: din definitie, cand ea numeste chiar
/// obiectul (pastreaza o schema cu punct intreaga), altfel din `schema.nume`.
fn nume_calificat_din_cod(cod: &str, obiect: &str) -> String {
    // un singur punct: `schema.nume` e neambiguu, ca inainte
    if obiect.matches('.').count() <= 1 {
        return nume_calificat(obiect);
    }
    // altfel numele exact cum e scris in definitie, daca o numeste chiar pe ea
    capul(cod)
        .and_then(|(_, t)| span_nume(cod, t))
        .filter(|&(s, f)| acelasi_nume(&parti_nume(&cod[s..f]), obiect))
        .map_or_else(
            || nume_calificat(obiect),
            |(s, f)| cod[s..f].trim().to_string(),
        )
}

fn nume_calificat(obiect: &str) -> String {
    match obiect.split_once('.') {
        Some((schema, nume)) => format!("{}.{}", paranteze(schema), paranteze(nume)),
        None => paranteze(obiect),
    }
}

/// `DROP ... IF EXISTS` pentru un obiect creat de deploy.
fn sterge(o: &DeDeployat) -> String {
    let fel = match o.tip.trim() {
        "P" => "PROCEDURE",
        "V" => "VIEW",
        "FN" | "TF" | "IF" => "FUNCTION",
        "TR" => "TRIGGER",
        alt => {
            // Nu se ghiceste. `RAISERROR` de severitate 16 **nu** opreste
            // nici SSMS, nici `sqlcmd` fara `-b`, deci pana la auditul
            // 2026-11-23 revenirea trecea peste obiect si refacea restul, iar
            // omul credea ca a reusit. Se pune si `SET NOEXEC ON`, ca la un
            // obiect cazut.
            return crlf(&format!(
                "-- ═══ {} ═══\nRAISERROR(N'GMotion: nu stiu sa sterg un obiect de tip %s; sterge-l de mana. Revenirea se opreste aici.', 16, 1, N'{}');\nSET NOEXEC ON;\nGO\n\n",
                o.obiect,
                alt.replace('\'', "''")
            ));
        }
    };
    let mut text = crlf(&format!(
        "-- ═══ {} ═══ (creat de deploy)\nDROP {fel} IF EXISTS {};\nGO\n",
        o.obiect,
        nume_calificat_din_cod(&o.cod, &o.obiect)
    ));
    text.push_str(&opreste_daca_a_cazut(&o.obiect));
    text
}

/// Textul scriptului de revenire pentru o baza.
///
/// Intai se sterg obiectele noi, in ordinea **inversa** dependentelor codului
/// nou, abia apoi se refac cele care existau (in ordinea dependentelor codului
/// vechi). Un view nou `WITH SCHEMABINDING` peste o functie care exista ar
/// bloca refacerea functiei cat timp view-ul e acolo (F-D8); codul vechi al
/// unui obiect existent nu poate depinde de unul creat abia de deploy, deci
/// stergerea intai nu rupe nimic.
fn text_revenire(baza: &str, referinta: &str, pasi: &[PasRevenire], acum: &str) -> String {
    let mut reface = Vec::new();
    let mut sterse = Vec::new();
    for p in pasi {
        match p {
            PasRevenire::Reface(o) => reface.push(o.clone()),
            PasRevenire::Sterge(o) => sterse.push(o.clone()),
        }
    }
    let mut text = crlf(&format!(
        "-- REVENIRE {baza} | anuleaza deploy-ul din {referinta} | salvat {acum}\n\
         -- Ruleaza-l doar ca sa anulezi deploy-ul: reface obiectele cum erau\n\
         -- inainte si sterge ce a creat deploy-ul.\n\
         {GARDA_VERSIUNE}{}",
        du_te_in_baza(baza)
    ));
    for i in ordine_dupa_dependente(&sterse).into_iter().rev() {
        text.push_str(&sterge(&sterse[i]));
    }
    for i in ordine_dupa_dependente(&reface) {
        text.push_str(&bloc(&reface[i]));
    }
    text.push_str(&crlf(SFARSIT_SCRIPT));
    text
}

/// Scrierea propriu-zisa a scripturilor de deploy, dupa verificari.
fn scrie_deploy(
    director: &Path,
    referinta: &str,
    pe_baza: &BTreeMap<String, Vec<DeDeployat>>,
    mod_impartire: ModImpartire,
    sarite: usize,
    acum: &str,
) -> Rezultat<Raport> {
    let ordonat: BTreeMap<String, Vec<DeDeployat>> = pe_baza
        .iter()
        .map(|(baza, obiecte)| {
            let ordine = ordine_dupa_dependente(obiecte);
            (
                baza.clone(),
                ordine.into_iter().map(|i| obiecte[i].clone()).collect(),
            )
        })
        .collect();
    let pe_baza = &ordonat;

    let mut fisiere = Vec::new();
    let mut nume_fisiere = NumeFisiere::default();
    let mut scrie_fisier = |baza_nume: String, continut: String| -> Rezultat<()> {
        let nume = nume_fisiere.unic(&baza_nume, "sql");
        let cale = director.join(&nume);
        let continut = format!("{continut}{}", crlf(SFARSIT_SCRIPT));
        scrie_script(&cale, &continut)?;
        fisiere.push(cale);
        Ok(())
    };

    match mod_impartire {
        ModImpartire::PerObiect => {
            for (baza, obiecte) in pe_baza {
                // cate cifre cere ultimul numar: peste 999 de obiecte,
                // `1000_` venea alfabetic inaintea lui `101_` (F-D9)
                let latime = obiecte.len().to_string().len().max(3);
                for (i, o) in obiecte.iter().enumerate() {
                    let nume = format!(
                        "{:0latime$}_{}_{}_{}",
                        i + 1,
                        slug(baza),
                        tip_nume(&o.tip),
                        slug(&o.obiect)
                    );
                    scrie_fisier(
                        nume,
                        format!("{}{}", antet(baza, referinta, acum, true), bloc(o)),
                    )?;
                }
            }
        }
        ModImpartire::CateN(n) => {
            let n = n.max(1);
            for (baza, obiecte) in pe_baza {
                let latime = obiecte.len().div_ceil(n).to_string().len().max(2);
                for (parte, grup) in obiecte.chunks(n).enumerate() {
                    let primul = parte * n + 1;
                    let nume = format!(
                        "{}_part{:0latime$}_obj{}-{}",
                        slug(baza),
                        parte + 1,
                        primul,
                        primul + grup.len() - 1
                    );
                    let mut text = antet(baza, referinta, acum, true);
                    for o in grup {
                        text.push_str(&bloc(o));
                    }
                    scrie_fisier(nume, text)?;
                }
            }
        }
        ModImpartire::Unul => {
            let mut text = String::new();
            for (baza, obiecte) in pe_baza {
                text.push_str(&antet(baza, referinta, acum, false));
                for o in obiecte {
                    text.push_str(&bloc(o));
                }
                text.push_str("\r\n");
            }
            scrie_fisier("deploy_complet".to_string(), text)?;
        }
        ModImpartire::PerBaza => {
            for (baza, obiecte) in pe_baza {
                let mut text = antet(baza, referinta, acum, pe_baza.len() > 1);
                for o in obiecte {
                    text.push_str(&bloc(o));
                }
                scrie_fisier(slug(baza), text)?;
            }
        }
    }

    let obiecte = pe_baza.values().map(Vec::len).sum();
    tracing::info!(
        target: "gmotion::modul",
        director = %director.display(), fisiere = fisiere.len(), obiecte,
        "scripturi de deploy scrise"
    );
    Ok(Raport {
        director: director.to_path_buf(),
        fisiere,
        revenire: Vec::new(),
        obiecte,
        sarite,
        // avertismentele de structura se calculeaza **numai** din ce se
        // deployeaza, deci exista si aici: lipseau doar fiindca `scrie` nu
        // vede tinta (A-11 din auditul 2026-11-23). Cele care chiar cer tinta
        // (view-uri legate, ordinea triggerelor) raman doar la
        // `scrie_cu_revenire`.
        avertismente: avertismente_structura(pe_baza),
    })
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn create_devine_create_or_alter() {
        for (intrare, obiect, asteptat) in [
            (
                "CREATE PROCEDURE dbo.p AS SELECT 1",
                "dbo.p",
                "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "CREATE PROC dbo.p AS SELECT 1",
                "dbo.p",
                "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "ALTER PROCEDURE dbo.p AS SELECT 1",
                "dbo.p",
                "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "create view dbo.v as select 1",
                "dbo.v",
                "CREATE OR ALTER VIEW dbo.v as select 1",
            ),
            (
                "ALTER FUNCTION dbo.f() RETURNS INT AS BEGIN RETURN 1 END",
                "dbo.f",
                "CREATE OR ALTER FUNCTION dbo.f() RETURNS INT AS BEGIN RETURN 1 END",
            ),
            (
                "CREATE TRIGGER dbo.t ON tab AFTER INSERT AS SELECT 1",
                "dbo.t",
                "CREATE OR ALTER TRIGGER dbo.t ON tab AFTER INSERT AS SELECT 1",
            ),
        ] {
            assert_eq!(
                aplica_create_or_alter(intrare, obiect),
                asteptat,
                "pe {intrare:?}"
            );
        }
    }

    #[test]
    fn spatii_si_comentarii_inainte_nu_incurca() {
        let cod = "-- comentariu\n/* bloc */\n\n   CREATE   PROCEDURE  dbo.p AS SELECT 1";
        let rez = aplica_create_or_alter(cod, "dbo.p");
        assert!(
            rez.starts_with("-- comentariu"),
            "antetul trebuie pastrat: {rez}"
        );
        assert!(rez.contains("CREATE OR ALTER PROCEDURE"));
        assert!(!rez.contains("CREATE   PROCEDURE"));
    }

    #[test]
    fn deja_create_or_alter_nu_se_atinge() {
        for (cod, obiect) in [
            ("CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1", "dbo.p"),
            ("create or alter view dbo.v as select 1", "dbo.v"),
            ("CREATE\n  OR\n  ALTER PROCEDURE dbo.p AS SELECT 1", "dbo.p"),
        ] {
            assert_eq!(aplica_create_or_alter(cod, obiect), cod, "pe {cod:?}");
        }
    }

    #[test]
    fn doar_prima_aparitie_se_schimba() {
        let cod = "CREATE PROCEDURE dbo.p AS\n-- CREATE PROCEDURE dbo.altceva\nEXEC('CREATE VIEW v AS SELECT 1')";
        let rez = aplica_create_or_alter(cod, "dbo.p");
        assert_eq!(rez.matches("CREATE OR ALTER").count(), 1);
        assert!(
            rez.contains("-- CREATE PROCEDURE dbo.altceva"),
            "comentariul nu se atinge"
        );
        assert!(
            rez.contains("EXEC('CREATE VIEW"),
            "SQL-ul dinamic nu se atinge"
        );
    }

    #[test]
    fn cod_fara_create_sau_alter_ramane_neatins() {
        for cod in [
            "SELECT 1",
            "",
            "-- doar un comentariu",
            "CREATED PROCEDURE x",
        ] {
            assert_eq!(aplica_create_or_alter(cod, "dbo.x"), cod, "pe {cod:?}");
        }
    }

    #[test]
    fn create_table_nu_se_atinge() {
        // Nu e un tip pe care-l comparam, si `CREATE OR ALTER TABLE` nu exista.
        let cod = "CREATE TABLE t (a INT)";
        assert_eq!(aplica_create_or_alter(cod, "dbo.t"), cod);
    }

    /// F-D4 din `cercetare/audit-qa-2026-11-18.md`: dupa `sp_rename` sau
    /// `ALTER SCHEMA TRANSFER`, `definition` pastreaza numele vechi. Scriptul
    /// crea atunci alt obiect decat cel din plan, iar revenirea stergea unul
    /// pe care deploy-ul nu-l crease.
    #[test]
    fn numele_din_definitie_se_rescrie_la_cel_din_plan() {
        for (cod, obiect, asteptat) in [
            // redenumit
            (
                "CREATE PROCEDURE dbo.pVechi AS SELECT 1",
                "dbo.pNou",
                "CREATE OR ALTER PROCEDURE [dbo].[pNou] AS SELECT 1",
            ),
            // mutat in alta schema
            (
                "CREATE OR ALTER VIEW [dbo].[v] AS SELECT 1",
                "raport.v",
                "CREATE OR ALTER VIEW [raport].[v] AS SELECT 1",
            ),
            // fara schema: ar ajunge in schema implicita a celui care ruleaza
            (
                "CREATE PROC p AS SELECT 1",
                "dbo.p",
                "CREATE OR ALTER PROCEDURE [dbo].[p] AS SELECT 1",
            ),
            // comentariu intre tip si nume
            (
                "CREATE PROC /* x */ dbo.pVechi AS SELECT 1",
                "dbo.pNou",
                "CREATE OR ALTER PROCEDURE /* x */ [dbo].[pNou] AS SELECT 1",
            ),
            // acelasi nume scris altfel nu se atinge: colatia e
            // case-insensitive, iar parantezele nu schimba obiectul
            (
                "CREATE OR ALTER PROCEDURE [DBO].[P] AS SELECT 1",
                "dbo.p",
                "CREATE OR ALTER PROCEDURE [DBO].[P] AS SELECT 1",
            ),
        ] {
            assert_eq!(aplica_create_or_alter(cod, obiect), asteptat, "{cod:?}");
        }
    }

    #[test]
    fn numele_se_citeste_si_cand_e_in_ghilimele_sau_cu_baza_in_fata() {
        assert_eq!(
            numele_creat("CREATE PROCEDURE \"dbo\".\"p\" AS SELECT 1").as_deref(),
            Some("dbo.p")
        );
        assert_eq!(
            numele_creat("CREATE PROCEDURE Baza.dbo.p AS SELECT 1").as_deref(),
            Some("dbo.p")
        );
        assert_eq!(
            numele_creat("CREATE PROCEDURE p AS SELECT 1").as_deref(),
            Some("p")
        );
        // fara `CREATE <tip>` nu se poate sti ce creeaza codul
        assert_eq!(numele_creat("SELECT 1"), None);
    }

    #[test]
    fn tip_si_slug() {
        assert_eq!(tip_nume("P"), "PROC");
        assert_eq!(tip_nume(" V "), "VIEW");
        assert_eq!(tip_nume("TF"), "FUNC");
        assert_eq!(tip_nume("X"), "OBJ");
        assert_eq!(slug("dbo.pClienti"), "dbo.pClienti");
        assert_eq!(slug("Baza Mea"), "Baza_Mea");
        assert_eq!(slug("a//b"), "a_b");
        // Punctele se pastreaza in interior, pentru `schema.obiect`; la
        // capete se taie, ca un nume de fisier sa nu inceapa cu `..`.
        assert_eq!(slug("../etc"), "etc");
        for nume in ["../../etc/passwd", "..", "C:\\Windows\\x", "./.."] {
            let s = slug(nume);
            assert!(
                !s.contains('/') && !s.contains('\\') && !s.contains(':'),
                "{nume:?} -> {s:?} ar putea forma o cale"
            );
        }
    }

    fn dir(et: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("gmotion-dep-{et}-{}", std::process::id()));
        std::fs::remove_dir_all(&d).ok();
        d
    }

    fn exemplu() -> BTreeMap<String, Vec<DeDeployat>> {
        let mut m = BTreeMap::new();
        m.insert(
            "Client1".to_string(),
            vec![
                DeDeployat {
                    obiect: "dbo.pA".into(),
                    tip: "P".into(),
                    cod: "CREATE OR ALTER PROCEDURE dbo.pA AS SELECT 1".into(),
                    setari: None,
                },
                DeDeployat {
                    obiect: "dbo.vB".into(),
                    tip: "V".into(),
                    cod: "CREATE OR ALTER VIEW dbo.vB AS SELECT 2".into(),
                    setari: None,
                },
                DeDeployat {
                    obiect: "dbo.pC".into(),
                    tip: "P".into(),
                    cod: "CREATE OR ALTER PROCEDURE dbo.pC AS SELECT 3".into(),
                    setari: None,
                },
            ],
        );
        m.insert(
            "Client2".to_string(),
            vec![DeDeployat {
                obiect: "dbo.pA".into(),
                tip: "P".into(),
                cod: "CREATE OR ALTER PROCEDURE dbo.pA AS SELECT 1".into(),
                setari: None,
            }],
        );
        m
    }

    fn o(obiect: &str, cod: &str) -> DeDeployat {
        DeDeployat {
            obiect: obiect.into(),
            tip: "P".into(),
            cod: cod.into(),
            setari: None,
        }
    }

    fn o_baza(baza: &str, obiect: DeDeployat) -> BTreeMap<String, Vec<DeDeployat>> {
        BTreeMap::from([(baza.to_string(), vec![obiect])])
    }

    // ---- regresii din auditul de la f561bf9 ----

    #[test]
    fn create_din_comentariu_sau_string_nu_se_rescrie() {
        for (intrare, obiect, asteptat) in [
            (
                "-- vechi: CREATE PROCEDURE dbo.pCom_old\nCREATE PROCEDURE dbo.pCom AS SELECT 1",
                "dbo.pCom",
                "-- vechi: CREATE PROCEDURE dbo.pCom_old\nCREATE OR ALTER PROCEDURE dbo.pCom AS SELECT 1",
            ),
            (
                "/* exemplu: ALTER VIEW dbo.v */ CREATE PROCEDURE dbo.p AS SELECT 1",
                "dbo.p",
                "/* exemplu: ALTER VIEW dbo.v */ CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "/* use CREATE OR ALTER next time */\nCREATE PROCEDURE dbo.pStr AS SELECT 1",
                "dbo.pStr",
                "/* use CREATE OR ALTER next time */\nCREATE OR ALTER PROCEDURE dbo.pStr AS SELECT 1",
            ),
            (
                "CREATE PROC /* x */ dbo.p AS SELECT 'ALTER VIEW v'",
                "dbo.p",
                "CREATE OR ALTER PROCEDURE /* x */ dbo.p AS SELECT 'ALTER VIEW v'",
            ),
        ] {
            assert_eq!(
                aplica_create_or_alter(intrare, obiect),
                asteptat,
                "{intrare:?}"
            );
        }
    }

    #[test]
    fn caracterele_care_isi_schimba_lungimea_la_minuscule_nu_strica_nimic() {
        // `K` (semnul Kelvin) si `İ`: minusculele au alta lungime in octeti. (diacritice-permise)
        // Inainte: panica, respectiv `CCREATE OR ALTER PROCEDUREdbo...`.
        assert_eq!(
            aplica_create_or_alter("-- 5\u{212A}\nCREATE PROCEDURE dbo.p AS SELECT 1", "dbo.p"),
            "-- 5\u{212A}\nCREATE OR ALTER PROCEDURE dbo.p AS SELECT 1"
        );
        assert_eq!(
            aplica_create_or_alter(
                "-- İstanbul\nCREATE PROCEDURE dbo.pTurc AS SELECT 1", // diacritice-permise: date de test
                "dbo.pTurc"
            ),
            "-- İstanbul\nCREATE OR ALTER PROCEDURE dbo.pTurc AS SELECT 1" // diacritice-permise: date de test
        );
    }

    #[test]
    fn numele_bazei_nu_poate_iesi_din_use() {
        let d = dir("use");
        let baza = "X]; DROP DATABASE Prod; --";
        let r = scrie(
            &d,
            "Ref",
            &o_baza(baza, o("dbo.p", "CREATE OR ALTER PROC dbo.p AS SELECT 1")),
            ModImpartire::PerBaza,
            0,
            "t",
        )
        .unwrap();
        let text = std::fs::read_to_string(&r.fisiere[0]).unwrap();
        assert!(
            text.contains("USE [X]]; DROP DATABASE Prod; --];"),
            "{text}"
        );
        assert!(
            !text.lines().any(|l| l.trim_start().starts_with("DROP")),
            "{text}"
        );
    }

    #[test]
    fn nume_cu_caractere_de_control_sunt_refuzate_inainte_de_orice_scriere() {
        let cod = "CREATE OR ALTER PROC dbo.p AS SELECT 1";
        for (baza, obiect) in [
            ("B\nGO\nDROP DATABASE Prod\nGO\n--", "dbo.p"),
            ("B", "dbo.q\nGO\nDROP TABLE dbo.Facturi\nGO\n--"),
            ("B\r", "dbo.p"),
        ] {
            let d = dir("control");
            let e = scrie(
                &d,
                "Ref",
                &o_baza(baza, o(obiect, cod)),
                ModImpartire::Unul,
                0,
                "t",
            )
            .unwrap_err();
            assert_eq!(e.cod(), "GM-MOD-008", "{baza:?} / {obiect:?}");
            assert!(!d.exists(), "nimic nu se scrie cand se refuza");
        }
    }

    #[test]
    fn go_in_definitie_e_refuzat() {
        let d = dir("go");
        let cod =
            "CREATE OR ALTER PROC dbo.pGo AS\nSELECT '\nGO\nDROP TABLE dbo.Clienti\nGO\n' AS x";
        let e = scrie(
            &d,
            "Ref",
            &o_baza("B", o("dbo.pGo", cod)),
            ModImpartire::Unul,
            0,
            "t",
        )
        .unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-009");
        assert!(e.to_string().contains("3, 5"), "{e}");
        assert!(!d.exists());
    }

    /// C7, partea vazuta de utilizator: `GO -- gata` trecea de garda, iar
    /// scriptul taia procedura in doua — `SELECT 43` rula ca batch separat pe
    /// baza tinta.
    #[test]
    fn go_urmat_de_comentariu_e_refuzat() {
        for cod in [
            "CREATE OR ALTER PROC dbo.pGo AS\nSELECT 42\nGO -- gata\nSELECT 43",
            "CREATE OR ALTER PROC dbo.pGo AS\nSELECT 42\nGO /* gata */\nSELECT 43",
        ] {
            let d = dir("go-comentariu");
            let e = scrie(
                &d,
                "Ref",
                &o_baza("B", o("dbo.pGo", cod)),
                ModImpartire::Unul,
                0,
                "t",
            )
            .unwrap_err();
            assert_eq!(e.cod(), "GM-MOD-009", "{cod}");
            assert!(e.to_string().contains('3'), "{e}");
            assert!(!d.exists(), "nimic nu se scrie cand se refuza");
        }
    }

    #[test]
    fn obiect_fara_cod_e_refuzat() {
        let d = dir("gol");
        let e = scrie(
            &d,
            "Ref",
            &o_baza("B", o("dbo.pCriptat", "")),
            ModImpartire::Unul,
            0,
            "t",
        )
        .unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-011");
    }

    #[test]
    fn nume_de_baze_care_dau_acelasi_fisier_nu_se_suprascriu() {
        let d = dir("coliziune");
        let cod = "CREATE OR ALTER PROC dbo.p AS SELECT 1";
        let mut m = BTreeMap::new();
        let baze = ["Baza Mea", "Baza_Mea", "baza_mea", "CON", "Bază", "Bazâ"]; // diacritice-permise: date de test
        for b in baze {
            m.insert(b.to_string(), vec![o("dbo.p", cod)]);
        }
        let r = scrie(&d, "Ref", &m, ModImpartire::PerBaza, 0, "t").unwrap();
        assert_eq!(r.fisiere.len(), 6);
        let mut nume: Vec<String> = r
            .fisiere
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_lowercase())
            .collect();
        nume.sort();
        nume.dedup();
        assert_eq!(nume.len(), 6, "{nume:?}");
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 6, "pe disc tot 6");
        assert!(r.fisiere.iter().all(|f| {
            let s = f.file_name().unwrap().to_string_lossy().to_uppercase();
            !s.starts_with("CON.")
        }));
    }

    /// Garda pentru F-D4 cand codul nu vine prin `aplica_create_or_alter`:
    /// un plan `.json` scris de mana, sau alt apelant al modulului.
    #[test]
    fn un_cod_care_creeaza_alt_obiect_e_refuzat_inainte_de_orice_scriere() {
        let d = dir("alt-obiect");
        let pe_baza = o_baza(
            "Baza",
            o("dbo.pNou", "CREATE OR ALTER PROC dbo.pVechi AS SELECT 1"),
        );
        let e = scrie(&d, "Ref", &pe_baza, ModImpartire::PerBaza, 0, "t")
            .expect_err("codul creeaza alt obiect");
        assert_eq!(e.cod(), "GM-MOD-014", "{e}");
        assert!(!d.exists(), "nu se scrie nimic");
    }

    #[test]
    fn director_ne_gol_e_refuzat() {
        let d = dir("negol");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("003_vechi.sql"), "-- vechi").unwrap();
        let e = scrie(&d, "Ref", &exemplu(), ModImpartire::PerObiect, 0, "t").unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-010");
        // director existent dar gol: e in regula
        let d2 = dir("gol-existent");
        std::fs::create_dir_all(&d2).unwrap();
        assert!(scrie(&d2, "Ref", &exemplu(), ModImpartire::PerBaza, 0, "t").is_ok());
    }

    #[test]
    fn setarile_obiectului_preced_codul_intr_un_batch_propriu() {
        let d = dir("setari");
        let mut ob = o("dbo.pVechi", "CREATE OR ALTER PROC dbo.pVechi AS SELECT 1");
        ob.setari = Some(gmotion_db::Setari {
            ansi_nulls: false,
            quoted_identifier: true,
        });
        let r = scrie(&d, "Ref", &o_baza("B", ob), ModImpartire::Unul, 0, "t").unwrap();
        let t = std::fs::read_to_string(&r.fisiere[0]).unwrap();
        assert!(
            t.contains("SET ANSI_NULLS OFF;\r\nSET QUOTED_IDENTIFIER ON;\r\nGO\r\nCREATE OR ALTER PROC dbo.pVechi"),
            "{t}"
        );
        // necunoscute: ON / ON, cu explicatie
        let d2 = dir("setari-necunoscute");
        let r = scrie(
            &d2,
            "Ref",
            &o_baza("B", o("dbo.p", "CREATE OR ALTER PROC dbo.p AS SELECT 1")),
            ModImpartire::Unul,
            0,
            "t",
        )
        .unwrap();
        let t = std::fs::read_to_string(&r.fisiere[0]).unwrap();
        assert!(
            t.contains("SET ANSI_NULLS ON;\r\nSET QUOTED_IDENTIFIER ON; -- setari necunoscute"),
            "{t}"
        );
    }

    fn o_tip(obiect: &str, tip: &str, cod: &str) -> DeDeployat {
        DeDeployat {
            tip: tip.into(),
            ..o(obiect, cod)
        }
    }

    fn ordonat(obiecte: &[DeDeployat]) -> Vec<String> {
        ordine_dupa_dependente(obiecte)
            .into_iter()
            .map(|i| obiecte[i].obiect.clone())
            .collect()
    }

    #[test]
    fn obiectele_folosite_se_scriu_inainte() {
        let obiecte = [
            o_tip(
                "dbo.aProc",
                "P",
                "CREATE PROC dbo.aProc AS SELECT * FROM dbo.mView",
            ),
            o_tip(
                "dbo.mView",
                "V",
                "CREATE VIEW dbo.mView AS SELECT dbo.zFunc(1) AS x",
            ),
            o_tip(
                "dbo.zFunc",
                "FN",
                "CREATE FUNCTION dbo.zFunc(@a INT) RETURNS INT AS BEGIN RETURN @a END",
            ),
        ];
        assert_eq!(ordonat(&obiecte), ["dbo.zFunc", "dbo.mView", "dbo.aProc"]);
    }

    #[test]
    fn referintele_cu_paranteze_sau_fara_schema_conteaza_cele_din_comentarii_nu() {
        let obiecte = [
            o_tip("dbo.p1", "P", "CREATE PROC dbo.p1 AS EXEC [dbo].[p2]"),
            o_tip("dbo.p2", "P", "CREATE PROC dbo.p2 AS EXEC p3"),
            o_tip(
                "dbo.p3",
                "P",
                "CREATE PROC dbo.p3 AS SELECT 1 -- apoi EXEC dbo.p1\nPRINT 'dbo.p2'",
            ),
        ];
        assert_eq!(ordonat(&obiecte), ["dbo.p3", "dbo.p2", "dbo.p1"]);
    }

    /// F-D11: `"dbo"."f"` e o referinta ca `[dbo].[f]`.
    #[test]
    fn referintele_cu_ghilimele_conteaza() {
        let obiecte = [
            o_tip("dbo.a", "P", "CREATE PROC dbo.a AS SELECT \"dbo\".\"f\"()"),
            o_tip(
                "dbo.f",
                "FN",
                "CREATE FUNCTION dbo.f() RETURNS INT AS BEGIN RETURN 1 END",
            ),
        ];
        assert_eq!(ordonat(&obiecte), ["dbo.f", "dbo.a"]);
    }

    /// F-D9: peste 999 de obiecte, numerele au patru cifre, deci ordinea
    /// alfabetica a fisierelor ramane ordinea din script.
    #[test]
    fn numerotarea_tine_ordinea_alfabetica_peste_999() {
        let d = dir("o-mie");
        let obiecte: Vec<DeDeployat> = (0..1001)
            .map(|i| DeDeployat {
                obiect: format!("dbo.p{i}"),
                tip: "P".into(),
                cod: format!("CREATE PROCEDURE dbo.p{i} AS SELECT {i}"),
                setari: None,
            })
            .collect();
        let m = BTreeMap::from([("Client".to_string(), obiecte)]);
        let r = scrie(&d, "Ref", &m, ModImpartire::PerObiect, 0, "a").unwrap();
        let mut nume: Vec<String> = r
            .fisiere
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        let in_ordine = nume.clone();
        nume.sort();
        assert_eq!(nume, in_ordine);
        assert!(in_ordine[0].starts_with("0001_"), "{}", in_ordine[0]);
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn ciclurile_nu_pierd_obiecte_si_independentele_isi_pastreaza_ordinea() {
        let obiecte = [
            o_tip("dbo.a", "P", "CREATE PROC dbo.a AS EXEC dbo.b"),
            o_tip("dbo.b", "P", "CREATE PROC dbo.b AS EXEC dbo.a"),
            o_tip("dbo.c", "P", "CREATE PROC dbo.c AS SELECT 1"),
            o_tip("dbo.d", "P", "CREATE PROC dbo.d AS SELECT 2"),
        ];
        let r = ordonat(&obiecte);
        assert_eq!(r, ["dbo.c", "dbo.d", "dbo.a", "dbo.b"]);
    }

    #[test]
    fn deploy_ul_scrie_in_ordinea_dependentelor() {
        let d = dir("ordine");
        let mut m = BTreeMap::new();
        m.insert(
            "B".to_string(),
            vec![
                o_tip(
                    "dbo.aView",
                    "V",
                    "CREATE OR ALTER VIEW dbo.aView AS SELECT dbo.zFunc() AS x",
                ),
                o_tip(
                    "dbo.zFunc",
                    "FN",
                    "CREATE OR ALTER FUNCTION dbo.zFunc() RETURNS INT AS BEGIN RETURN 1 END",
                ),
            ],
        );
        let r = scrie(&d, "Ref", &m, ModImpartire::PerObiect, 0, "t").unwrap();
        let nume: Vec<String> = r
            .fisiere
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(
            nume,
            ["001_B_FUNC_dbo.zFunc.sql", "002_B_VIEW_dbo.aView.sql"]
        );
    }

    #[test]
    fn fiecare_fisier_incepe_cu_garda_de_versiune_si_o_inchide() {
        let d = dir("garda");
        let r = scrie(&d, "Ref", &exemplu(), ModImpartire::PerObiect, 0, "t").unwrap();
        for f in &r.fisiere {
            let t = std::fs::read_to_string(f).unwrap();
            let garda = t.find("SET NOEXEC ON").expect("garda lipseste");
            let poz_use = t.find("USE [").unwrap();
            // mesajul garzii contine si el „CREATE OR ALTER"; codul incepe cu tipul
            let primul_create = t
                .find("CREATE OR ALTER PROC")
                .or_else(|| t.find("CREATE OR ALTER VIEW"))
                .unwrap();
            assert!(garda < poz_use && poz_use < primul_create, "{t}");
            assert!(t.contains("< 4001"), "{t}");
            // la capat si setarile de sesiune se redau, ca SSMS sa nu ramana
            // cu cele ale ultimului obiect deployat
            assert!(
                t.ends_with("SET NOEXEC OFF;\r\nGO\r\nSET ANSI_NULLS ON;\r\nSET QUOTED_IDENTIFIER ON;\r\nGO\r\n"),
                "{t}"
            );
        }
    }

    /// F-D1 din `cercetare/audit-qa-2026-11-18.md`: daca `USE [baza]` cade
    /// (baza offline, redenumita, fara drept), doar batch-ul ala se opreste.
    /// SSMS si `sqlcmd` fara `-b` ruleaza apoi toate `CREATE OR ALTER` pe
    /// baza curenta — master, referinta, sau, cu „un singur fisier", baza
    /// anterioara. Dupa `USE` trebuie sa urmeze o verificare care opreste tot.
    #[test]
    fn dupa_use_se_verifica_in_ce_baza_suntem() {
        let d = dir("use-verificat");
        let r = scrie(&d, "Ref", &exemplu(), ModImpartire::Unul, 0, "t").unwrap();
        let t = std::fs::read_to_string(&r.fisiere[0]).unwrap();
        for baza in ["Client1", "Client2"] {
            let poz_use = t.find(&format!("USE [{baza}];")).expect("USE lipseste");
            let verificare = t[poz_use..]
                .find("DB_NAME()")
                .expect("verificarea bazei lipseste");
            let primul_create = t[poz_use..]
                .find("CREATE OR ALTER")
                .expect("obiectele bazei");
            assert!(
                verificare < primul_create,
                "verificarea vine inainte de orice obiect:\n{}",
                &t[poz_use..]
            );
            assert!(
                t[poz_use + verificare..].contains(&format!("N'{baza}'")),
                "se verifica numele bazei"
            );
        }
        std::fs::remove_dir_all(&d).ok();
    }

    /// F-D2 din `cercetare/audit-qa-2026-11-18.md`: scripturile erau UTF-8
    /// **fara BOM**, cu capete de linie amestecate. `sqlcmd` fara
    /// `-f 65001` citeste un fisier fara BOM in pagina de cod OEM, deci un
    /// literal `N'...'` cu caractere non-ASCII ajunge mojibake in codul
    /// deployat; SSMS intreaba de capetele de linie.
    #[test]
    fn scripturile_au_bom_si_liniile_gmotion_sunt_crlf() {
        let d = dir("bom");
        let cod = "CREATE OR ALTER PROC dbo.pMesaj AS\nSELECT N'cheie   \nvaloare'";
        let (mut deploy, rev) = cu_revenire();
        deploy.insert(
            "Client1".to_string(),
            vec![dd("dbo.pMesaj", "P", cod, true)],
        );
        let r = scrie_cu_revenire(&d, "Ref", &deploy, &rev, ModImpartire::PerBaza, 0, "t").unwrap();
        for f in r.fisiere.iter().chain(r.revenire.iter()) {
            let octeti = std::fs::read(f).unwrap();
            assert_eq!(
                &octeti[..3],
                &[0xEF, 0xBB, 0xBF],
                "BOM UTF-8 la inceputul lui {}",
                f.display()
            );
            let t = String::from_utf8(octeti).unwrap();
            // fiecare linie scrisa de GMotion se incheie cu \r\n
            for linie in ["GO", "SET NOEXEC OFF;"] {
                assert!(
                    t.contains(&format!("{linie}\r\n")),
                    "linia {linie} cu CRLF in {}:\n{t:?}",
                    f.display()
                );
            }
        }
        // codul obiectului ramane octet cu octet, cu LF-urile lui
        let t = std::fs::read_to_string(d.join("Client1.sql")).unwrap();
        assert!(
            t.contains("SELECT N'cheie   \nvaloare'"),
            "codul utilizatorului neatins:\n{t:?}"
        );
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn mod_per_baza() {
        let d = dir("per-baza");
        let r = scrie(
            &d,
            "Ref",
            &exemplu(),
            ModImpartire::PerBaza,
            0,
            "2026-11-17 10:00",
        )
        .unwrap();
        assert_eq!(r.fisiere.len(), 2);
        assert_eq!(r.obiecte, 4);
        let t = std::fs::read_to_string(d.join("Client1.sql")).unwrap();
        assert!(
            t.starts_with("\u{feff}-- DEPLOYER Client1 | referinta: Ref | 2026-11-17 10:00\r\n"),
            "{t:?}"
        );
        assert!(t.contains("USE [Client1];\r\nGO\r\n"));
        assert_eq!(t.matches("-- ═══").count(), 3);
        assert_eq!(
            t.matches("\r\nGO\r\n").count(),
            14,
            "garda, USE, verificarea bazei, trei per obiect (setarile, codul, oprirea), \
             si doua la sfarsit (NOEXEC OFF, setarile redate)"
        );
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn mod_unul_singur() {
        let d = dir("unul");
        let r = scrie(&d, "Ref", &exemplu(), ModImpartire::Unul, 0, "a").unwrap();
        assert_eq!(r.fisiere.len(), 1);
        let t = std::fs::read_to_string(d.join("deploy_complet.sql")).unwrap();
        // fiecare baza isi aduce propriul USE: altfel al doilea set ar rula pe
        // baza gresita
        assert_eq!(t.matches("USE [").count(), 2);
        assert!(t.find("USE [Client1]").unwrap() < t.find("USE [Client2]").unwrap());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn mod_per_obiect() {
        let d = dir("per-obiect");
        let r = scrie(&d, "Ref", &exemplu(), ModImpartire::PerObiect, 0, "a").unwrap();
        assert_eq!(r.fisiere.len(), 4);
        // fara dependente intre ele: view-urile inaintea procedurilor
        assert!(d.join("001_Client1_VIEW_dbo.vB.sql").exists());
        assert!(d.join("002_Client1_PROC_dbo.pA.sql").exists());
        assert!(d.join("003_Client1_PROC_dbo.pC.sql").exists());
        assert!(d.join("001_Client2_PROC_dbo.pA.sql").exists());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn mod_cate_n() {
        let d = dir("cate-n");
        let r = scrie(&d, "Ref", &exemplu(), ModImpartire::CateN(2), 0, "a").unwrap();
        // Client1 are 3 obiecte -> doua fisiere; Client2 are 1 -> unul
        assert_eq!(r.fisiere.len(), 3);
        assert!(d.join("Client1_part01_obj1-2.sql").exists());
        assert!(d.join("Client1_part02_obj3-3.sql").exists());
        assert!(d.join("Client2_part01_obj1-1.sql").exists());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn cate_n_cu_zero_nu_imparte_la_zero() {
        let d = dir("cate-zero");
        let r = scrie(&d, "Ref", &exemplu(), ModImpartire::CateN(0), 0, "a").unwrap();
        assert_eq!(r.fisiere.len(), 4, "0 se trateaza ca 1");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn nimic_de_deployat_e_eroare_nu_un_director_gol() {
        let d = dir("gol-nimic-de-deployat");
        let e = scrie(&d, "Ref", &BTreeMap::new(), ModImpartire::PerBaza, 2, "a").unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-002");
        assert!(!d.exists(), "nu se creeaza directorul degeaba");
    }

    #[test]
    fn numele_de_baza_cu_caractere_dificile_nu_scapa_din_director() {
        let d = dir("nume-dificil");
        let mut m = BTreeMap::new();
        m.insert(
            "../evadare".to_string(),
            vec![DeDeployat {
                obiect: "dbo.p".into(),
                tip: "P".into(),
                cod: "CREATE PROCEDURE dbo.p AS SELECT 1".into(),
                setari: None,
            }],
        );
        let r = scrie(&d, "Ref", &m, ModImpartire::PerBaza, 0, "a").unwrap();
        assert_eq!(r.fisiere.len(), 1);
        assert_eq!(r.fisiere[0].parent().unwrap(), d.as_path());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn textul_final_se_verifica_la_scriere() {
        let v = |t: &str| verifica_text_final("dbo.p", "P", t).map_err(|e| e.cod());
        assert_eq!(v(""), Err("GM-MOD-016"));
        assert_eq!(v("DROP TABLE dbo.Clienti"), Err("GM-MOD-015"));
        assert_eq!(
            v("CREATE PROCEDURE dbo.p AS SELECT 1\nGO"),
            Err("GM-MOD-009")
        );
        assert_eq!(v("CREATE PROCEDURE dbo.p AS SELECT 1"), Ok(()));
    }

    /// B2 si F-D3 din `cercetare/audit-qa-2026-11-22.md`: un cod care nu
    /// creeaza obiectul din plan (SQL oarecare, alt tip) nu se scrie.
    #[test]
    fn codul_care_nu_creeaza_obiectul_din_plan_e_refuzat() {
        let pas = |tip: &str, cod: &str| DeDeployat {
            obiect: "dbo.p".into(),
            tip: tip.into(),
            cod: cod.into(),
            setari: None,
        };
        for (tip, cod) in [
            ("P", "DELETE FROM dbo.Facturi"),
            ("P", "SELECT 1"),
            ("P", "-- CREATE PROCEDURE dbo.p\nDROP TABLE dbo.Clienti"),
            ("P", "CREATE VIEW dbo.p AS SELECT 1"),
            ("V", "CREATE PROCEDURE dbo.p AS SELECT 1"),
            ("FN", "CREATE PROC dbo.p AS SELECT 1"),
        ] {
            let e = verifica_capul(&pas(tip, cod)).unwrap_err();
            assert_eq!(e.cod(), "GM-MOD-015", "{tip} / {cod}");
        }
        for (tip, cod) in [
            ("P", "/* antet */\n-- x\nCREATE PROCEDURE dbo.p AS SELECT 1"),
            ("P", "create proc dbo.p as select 1"),
            ("P", "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1"),
            ("V", "ALTER VIEW dbo.p AS SELECT 1"),
            (
                "IF",
                "CREATE FUNCTION dbo.p() RETURNS TABLE AS RETURN SELECT 1",
            ),
            ("TR", "CREATE TRIGGER dbo.p ON t AFTER INSERT AS SELECT 1"),
        ] {
            assert!(verifica_capul(&pas(tip, cod)).is_ok(), "{tip} / {cod}");
        }
        let d = dir("fara-create");
        let mut m = BTreeMap::new();
        m.insert(
            "Client".to_string(),
            vec![pas("P", "DELETE FROM dbo.Facturi")],
        );
        let e = scrie(&d, "Ref", &m, ModImpartire::PerBaza, 0, "a").unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-015");
        assert!(!d.exists(), "nimic scris");
    }

    fn dd(obiect: &str, tip: &str, cod: &str, qi: bool) -> DeDeployat {
        DeDeployat {
            obiect: obiect.into(),
            tip: tip.into(),
            cod: cod.into(),
            setari: Some(gmotion_db::Setari {
                ansi_nulls: true,
                quoted_identifier: qi,
            }),
        }
    }

    /// Deploy pe Client1: pA modificat (exista), fNou + vNou noi; vNou
    /// foloseste fNou cu SCHEMABINDING.
    fn cu_revenire() -> (
        BTreeMap<String, Vec<DeDeployat>>,
        BTreeMap<String, Vec<PasRevenire>>,
    ) {
        let f = "CREATE OR ALTER FUNCTION dbo.fNou() RETURNS INT AS BEGIN RETURN 1 END";
        let v = "CREATE OR ALTER VIEW dbo.vNou WITH SCHEMABINDING AS SELECT dbo.fNou() AS x";
        let mut deploy = BTreeMap::new();
        deploy.insert(
            "Client1".to_string(),
            vec![
                dd(
                    "dbo.pA",
                    "P",
                    "CREATE OR ALTER PROCEDURE dbo.pA AS SELECT 1",
                    true,
                ),
                dd("dbo.vNou", "V", v, true),
                dd("dbo.fNou", "FN", f, true),
            ],
        );
        let mut rev = BTreeMap::new();
        rev.insert(
            "Client1".to_string(),
            vec![
                PasRevenire::Reface(dd(
                    "dbo.pA",
                    "P",
                    "CREATE OR ALTER PROCEDURE dbo.pA AS SELECT \"vechi\"",
                    false,
                )),
                PasRevenire::Sterge(dd("dbo.vNou", "V", v, true)),
                PasRevenire::Sterge(dd("dbo.fNou", "FN", f, true)),
            ],
        );
        (deploy, rev)
    }

    #[test]
    fn revenirea_se_scrie_separat_cu_codul_si_setarile_tintei() {
        let d = dir("rev");
        let (deploy, rev) = cu_revenire();
        let r = scrie_cu_revenire(&d, "Ref", &deploy, &rev, ModImpartire::PerBaza, 0, "t").unwrap();
        assert_eq!(r.revenire, vec![d.join("revenire").join("Client1.sql")]);
        assert_eq!(
            r.fisiere,
            vec![d.join("Client1.sql")],
            "revenirea nu sta langa deploy"
        );
        let sql = std::fs::read_to_string(&r.revenire[0]).unwrap();
        assert!(sql.contains("USE [Client1];"));
        // si revenirea scrie cod peste o baza: tot are nevoie de verificarea
        // ca `USE` a reusit (F-D1)
        assert!(
            sql.contains("DB_NAME() <> N'Client1'"),
            "revenirea verifica baza:\n{sql}"
        );
        assert!(sql.contains("SELECT \"vechi\""), "codul din tinta:\n{sql}");
        assert!(
            sql.contains("SET QUOTED_IDENTIFIER OFF;"),
            "setarile tintei, nu ale referintei:\n{sql}"
        );
        assert!(sql.contains("DROP VIEW IF EXISTS [dbo].[vNou];"), "{sql}");
        assert!(sql.contains("DROP FUNCTION IF EXISTS [dbo].[fNou];"));
        assert!(sql.contains("SET NOEXEC OFF;\r\nGO"), "{sql}");
        assert!(
            sql.trim_end().ends_with("SET QUOTED_IDENTIFIER ON;\r\nGO"),
            "{sql}"
        );
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn revenirea_sterge_intai_dependentii_apoi_reface() {
        let (_, rev) = cu_revenire();
        let sql = text_revenire("Client1", "Ref", &rev["Client1"], "t");
        let poz = |s: &str| {
            sql.find(s)
                .unwrap_or_else(|| panic!("lipseste {s}:\n{sql}"))
        };
        assert!(
            poz("DROP FUNCTION") < poz("SELECT \"vechi\""),
            "stergerile inainte de refaceri (F-D8)"
        );
        assert!(
            poz("DROP VIEW") < poz("DROP FUNCTION"),
            "view-ul cu SCHEMABINDING blocheaza stergerea functiei:\n{sql}"
        );
    }

    #[test]
    fn un_view_nou_legat_de_o_functie_existenta_se_sterge_inainte_de_refacerea_ei() {
        // F-D8: deploy-ul a modificat dbo.f si a creat dbo.v WITH SCHEMABINDING
        // peste ea; cat timp v exista, ALTER pe f cade
        let v = "CREATE OR ALTER VIEW dbo.v WITH SCHEMABINDING AS SELECT dbo.f() AS x";
        let pasi = vec![
            PasRevenire::Reface(dd(
                "dbo.f",
                "FN",
                "CREATE OR ALTER FUNCTION dbo.f() RETURNS INT AS BEGIN RETURN 0 END",
                true,
            )),
            PasRevenire::Sterge(dd("dbo.v", "V", v, true)),
        ];
        let sql = text_revenire("Client1", "Ref", &pasi, "t");
        let drop = sql.find("DROP VIEW IF EXISTS [dbo].[v]").expect("drop");
        let reface = sql.find("RETURN 0").expect("reface");
        assert!(drop < reface, "{sql}");
    }

    #[test]
    fn avertismente_pentru_view_uri_legate_si_triggere_existente() {
        let mut rev = BTreeMap::new();
        rev.insert(
            "C1".to_string(),
            vec![
                PasRevenire::Reface(dd(
                    "dbo.vLegat",
                    "V",
                    "CREATE VIEW dbo.vLegat WITH SCHEMABINDING AS SELECT a FROM dbo.t",
                    true,
                )),
                PasRevenire::Reface(dd(
                    "dbo.vSimplu",
                    "V",
                    "CREATE VIEW dbo.vSimplu AS SELECT 'SCHEMABINDING' AS a -- SCHEMABINDING",
                    true,
                )),
                PasRevenire::Reface(dd(
                    "dbo.tr",
                    "TR",
                    "CREATE TRIGGER dbo.tr ON t AFTER INSERT AS SELECT 1",
                    true,
                )),
                // un trigger nou nu are ordine de pierdut
                PasRevenire::Sterge(dd(
                    "dbo.trNou",
                    "TR",
                    "CREATE TRIGGER dbo.trNou ON t AFTER INSERT AS SELECT 1",
                    true,
                )),
            ],
        );
        assert_eq!(
            avertismente(&rev),
            vec![
                Avertisment::ViewPosibilIndexat {
                    baza: "C1".into(),
                    obiect: "dbo.vLegat".into()
                },
                Avertisment::OrdineTrigger {
                    baza: "C1".into(),
                    obiect: "dbo.tr".into()
                },
            ]
        );
        assert!(avertismente(&rev)[0].to_string().contains("indecsi"));
    }

    #[test]
    fn codul_care_nu_se_inchide_primeste_avertisment() {
        let mut pe_baza = BTreeMap::new();
        pe_baza.insert(
            "C1".to_string(),
            vec![
                dd(
                    "dbo.pBun",
                    "P",
                    "CREATE OR ALTER PROC dbo.pBun AS\nBEGIN\nSELECT 1\nEND",
                    true,
                ),
                dd(
                    "dbo.pRau",
                    "P",
                    "CREATE OR ALTER PROC dbo.pRau AS\nBEGIN\nSELECT 1",
                    true,
                ),
            ],
        );
        assert_eq!(
            avertismente_structura(&pe_baza),
            [Avertisment::StructuraStricata {
                baza: "C1".into(),
                obiect: "dbo.pRau".into(),
                linie: 2
            }]
        );
        assert!(avertismente_structura(&pe_baza)[0]
            .to_string()
            .contains("linia 2"));
    }

    #[test]
    fn raportul_cu_revenire_poarta_avertismentele() {
        let d = dir("avertismente");
        let mut deploy = BTreeMap::new();
        deploy.insert(
            "Client1".to_string(),
            vec![dd(
                "dbo.tr",
                "TR",
                "CREATE OR ALTER TRIGGER dbo.tr ON dbo.t AFTER INSERT AS SELECT 2",
                true,
            )],
        );
        let mut rev = BTreeMap::new();
        rev.insert(
            "Client1".to_string(),
            vec![PasRevenire::Reface(dd(
                "dbo.tr",
                "TR",
                "CREATE TRIGGER dbo.tr ON dbo.t AFTER INSERT AS SELECT 1",
                true,
            ))],
        );
        let r = scrie_cu_revenire(&d, "Ref", &deploy, &rev, ModImpartire::PerBaza, 0, "t").unwrap();
        assert_eq!(r.avertismente.len(), 1);
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn dupa_fiecare_obiect_scriptul_se_opreste_daca_a_cazut() {
        // F-D7: un obiect care cade nu lasa restul sa ruleze
        let o = dd(
            "dbo.p%s",
            "P",
            "CREATE OR ALTER PROCEDURE dbo.[p%s] AS SELECT 1",
            true,
        );
        let text = bloc(&o);
        let cod = text.find("SELECT 1").expect("cod");
        let garda = text.find("IF @@ERROR <> 0").expect("garda");
        assert!(cod < garda, "garda dupa cod:\n{text}");
        assert!(
            text[cod..garda].contains("\r\nGO\r\n"),
            "garda in batch-ul de dupa obiect:\n{text}"
        );
        assert!(text[garda..].contains("SET NOEXEC ON;"), "{text}");
        assert!(text.trim_end().ends_with("GO"), "{text}");
        // numele e argument, nu format
        assert!(text.contains("16, 1, N'dbo.p%s');"), "{text}");
        assert_eq!(text.matches("IF @@ERROR").count(), 1);

        let s = sterge(&o);
        assert!(
            s.find("DROP PROCEDURE").expect("drop") < s.find("IF @@ERROR <> 0").expect("garda"),
            "{s}"
        );
    }

    #[test]
    fn scriptul_intreg_are_o_garda_per_obiect_si_reda_sesiunea_la_final() {
        let d = dir("garda-per-obiect");
        scrie(&d, "Ref", &exemplu(), ModImpartire::PerBaza, 0, "t").unwrap();
        let t = std::fs::read_to_string(d.join("Client1.sql")).unwrap();
        assert_eq!(t.matches("IF @@ERROR <> 0").count(), 3, "{t}");
        let ultima = t.rfind("IF @@ERROR <> 0").unwrap();
        assert!(t[ultima..].contains("SET NOEXEC OFF;"), "{t}");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn revenire_imposibila_nu_scrie_nici_deploy_ul() {
        let d = dir("rev-go");
        let (deploy, mut rev) = cu_revenire();
        rev.get_mut("Client1").unwrap()[0] = PasRevenire::Reface(dd(
            "dbo.pA",
            "P",
            "CREATE PROCEDURE dbo.pA AS\nGO\nSELECT 1",
            true,
        ));
        let e =
            scrie_cu_revenire(&d, "Ref", &deploy, &rev, ModImpartire::PerBaza, 0, "t").unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-009");
        assert!(!d.exists(), "nu trebuia scris nimic");
    }

    #[test]
    fn stergerea_pune_numele_intre_paranteze() {
        let s = sterge(&dd("dbo.p]; DROP DATABASE x; --", "P", "", true));
        assert!(
            s.contains("DROP PROCEDURE IF EXISTS [dbo].[p]]; DROP DATABASE x; --];"),
            "{s}"
        );
        let s = sterge(&dd("dbo.t", "TR", "", true));
        assert!(s.contains("DROP TRIGGER IF EXISTS [dbo].[t];"));
        let s = sterge(&dd("dbo.x", "U", "", true));
        assert!(
            s.contains("RAISERROR") && !s.contains("DROP"),
            "tip necunoscut: {s}"
        );
    }

    /// F-D12 si F-D10 din `cercetare/audit-qa-2026-11-18.md`: un plan `.json`
    /// scris de mana putea repeta un pas (obiectul scris de doua ori, cu doua
    /// DROP-uri in revenire), iar o procedura numerotata (`dbo.p;1`) ajungea
    /// intr-un `CREATE OR ALTER`, care nu le accepta: scriptul cadea la
    /// rulare, dupa ce alte obiecte fusesera deja scrise.
    #[test]
    fn pasii_repetati_si_procedurile_numerotate_se_refuza() {
        let d = dir("repetat");
        let mut pe_baza: BTreeMap<String, Vec<DeDeployat>> = BTreeMap::new();
        pe_baza.insert(
            "B".to_string(),
            vec![
                o("dbo.p", "CREATE OR ALTER PROC dbo.p AS SELECT 1"),
                o("dbo.p", "CREATE OR ALTER PROC dbo.p AS SELECT 2"),
            ],
        );
        let e = scrie(&d, "Ref", &pe_baza, ModImpartire::Unul, 0, "t").unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-013", "{e}");
        assert!(!d.exists(), "nimic nu se scrie cand se refuza");

        let d = dir("numerotat");
        let e = scrie(
            &d,
            "Ref",
            &o_baza(
                "B",
                o("dbo.p;1", "CREATE OR ALTER PROC dbo.p;1 AS SELECT 1"),
            ),
            ModImpartire::Unul,
            0,
            "t",
        )
        .unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-008", "{e}");
        assert!(!d.exists());
    }

    /// F-D6 din `cercetare/audit-qa-2026-11-18.md`: numele per obiect
    /// (`NNN_<baza>_<TIP>_<schema>.<nume>`) ajungea la ~400 de caractere, iar
    /// pe Windows calea intreaga nu poate trece de 260: deploy-ul cadea
    /// **dupa** ce revenirea si cateva fisiere erau deja scrise, iar reluarea
    /// cadea cu `GM-MOD-010`.
    /// O schema cu punct nu se rescrie in alt obiect, nici la deploy, nici in
    /// `DROP`-ul din revenire.
    #[test]
    fn schema_cu_punct_ramane_intreaga() {
        let cod = "CREATE PROCEDURE [my.s].[x] AS SELECT 1";
        let rez = aplica_create_or_alter(cod, "my.s.x");
        assert!(rez.contains("[my.s].[x]"), "{rez}");
        assert!(!rez.contains("[my].[s.x]"), "{rez}");
        let o = DeDeployat {
            obiect: "my.s.x".into(),
            tip: "P".into(),
            cod: cod.into(),
            setari: None,
        };
        assert!(verifica_capul(&o).is_ok());
        assert!(
            sterge(&o).contains("DROP PROCEDURE IF EXISTS [my.s].[x]"),
            "{}",
            sterge(&o)
        );
        let mare = "CREATE PROCEDURE [My.S].[X] AS SELECT 1";
        assert!(aplica_create_or_alter(mare, "My.S.X").contains("[My.S].[X]"));
        // un tip necunoscut nu trece
        let zz = DeDeployat {
            tip: "ZZ".into(),
            ..o.clone()
        };
        assert_eq!(verifica_capul(&zz).unwrap_err().cod(), "GM-MOD-015");
        // un nume chiar diferit e rescris ca inainte (F-D4)
        assert!(
            aplica_create_or_alter("CREATE PROCEDURE dbo.vechi AS SELECT 1", "dbo.nou")
                .contains("[dbo].[nou]")
        );
    }

    /// N2: un director strain cu nume asemanator nu se atinge.
    #[test]
    fn directorul_de_lucru_nu_sterge_nimic_strain() {
        let baza = dir("strain");
        std::fs::create_dir_all(&baza).unwrap();
        for (cerut, strain) in [
            ("proiect", "proiect.tmp"),
            ("deploy_2026.11.23", "deploy_2026.11.tmp"),
        ] {
            let s = baza.join(strain);
            std::fs::create_dir_all(&s).unwrap();
            std::fs::write(s.join("important.sql"), "munca").unwrap();
            let mut m = BTreeMap::new();
            m.insert(
                "Client".to_string(),
                vec![DeDeployat {
                    obiect: "dbo.p".into(),
                    tip: "P".into(),
                    cod: "CREATE PROCEDURE dbo.p AS SELECT 1".into(),
                    setari: None,
                }],
            );
            scrie(&baza.join(cerut), "Ref", &m, ModImpartire::PerBaza, 0, "a").unwrap();
            assert_eq!(
                std::fs::read_to_string(s.join("important.sql")).unwrap(),
                "munca"
            );
        }
        let ramase: Vec<String> = std::fs::read_dir(&baza)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains("gmotion-lucru"))
            .collect();
        assert!(ramase.is_empty(), "director de lucru ramas: {ramase:?}");
        std::fs::remove_dir_all(&baza).ok();
    }

    #[test]
    fn numele_lungi_se_plafoneaza_si_raman_distincte() {
        let d = dir("nume-lungi");
        let lung = |sufix: &str| format!("dbo.{}{sufix}", "N".repeat(220));
        let mut pe_baza: BTreeMap<String, Vec<DeDeployat>> = BTreeMap::new();
        pe_baza.insert(
            "BazaCuNumeFoarteLungPentruProba".to_string(),
            vec![
                o(
                    &lung("_unu"),
                    &format!("CREATE OR ALTER PROC {} AS SELECT 1", lung("_unu")),
                ),
                o(
                    &lung("_doi"),
                    &format!("CREATE OR ALTER PROC {} AS SELECT 2", lung("_doi")),
                ),
            ],
        );
        let r = scrie(&d, "Ref", &pe_baza, ModImpartire::PerObiect, 0, "t").unwrap();
        assert_eq!(r.fisiere.len(), 2);
        let nume: Vec<String> = r
            .fisiere
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        for n in &nume {
            assert!(n.chars().count() <= 110, "{n} ({})", n.chars().count());
        }
        assert_ne!(nume[0], nume[1], "doua obiecte, doua fisiere");
        std::fs::remove_dir_all(&d).ok();
    }

    /// F-D6, partea a doua: ori apare directorul intreg, ori nu apare deloc.
    /// Scrierea se face intr-un `<director>.tmp`, mutat la final.
    #[test]
    fn un_deploy_esuat_nu_lasa_director_pe_jumatate() {
        let d = dir("tmp-mutat");
        let (deploy, rev) = cu_revenire();
        let r = scrie_cu_revenire(&d, "Ref", &deploy, &rev, ModImpartire::PerBaza, 0, "t").unwrap();
        assert!(d.join("revenire").is_dir(), "revenirea e in director");
        assert!(
            !d.with_extension("tmp").exists(),
            "directorul de lucru nu ramane in urma"
        );
        assert!(
            r.fisiere.iter().all(|f| f.starts_with(&d)),
            "caile din raport sunt cele finale: {:?}",
            r.fisiere
        );
        assert!(r.revenire.iter().all(|f| f.starts_with(&d)));
        std::fs::remove_dir_all(&d).ok();

        // un `<director>.tmp` ramas de la o rulare oprita brusc nu
        // blocheaza urmatoarea si nu-si amesteca fisierele
        let d3 = dir("tmp-vechi");
        std::fs::create_dir_all(d3.with_extension("tmp")).unwrap();
        std::fs::write(d3.with_extension("tmp").join("vechi.sql"), b"-- vechi").unwrap();
        let r =
            scrie_cu_revenire(&d3, "Ref", &deploy, &rev, ModImpartire::PerBaza, 0, "t").unwrap();
        assert!(
            !d3.join("vechi.sql").exists(),
            "fisierul vechi nu intra in deploy"
        );
        assert!(!r.fisiere.is_empty());
        std::fs::remove_dir_all(&d3).ok();

        // mutarea finala cade (in locul directorului e un fisier): nu ramane
        // nici directorul de lucru
        let d2 = dir("tmp-cade");
        std::fs::remove_dir_all(&d2).ok();
        std::fs::write(&d2, b"sunt un fisier").unwrap();
        let e = scrie_cu_revenire(&d2, "Ref", &deploy, &rev, ModImpartire::PerBaza, 0, "t")
            .unwrap_err();
        assert_eq!(e.cod(), "GM-MOD-003", "{e}");
        assert!(
            !d2.with_extension("tmp").exists(),
            "directorul de lucru s-a sters"
        );
        std::fs::remove_file(&d2).ok();
    }
}
