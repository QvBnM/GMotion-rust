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

impl NumeFisiere {
    /// `baza` e partea dinaintea extensiei, deja trecuta prin `slug`.
    fn unic(&mut self, baza: &str, extensie: &str) -> String {
        let mut baza = if baza.is_empty() {
            "fara_nume".to_string()
        } else {
            baza.to_string()
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

/// Transforma primul `CREATE` sau `ALTER <tip>` in `CREATE OR ALTER`.
///
/// Un script care face `CREATE` pe un obiect existent esueaza; unul care face
/// `ALTER` pe un obiect inexistent, la fel. `CREATE OR ALTER` merge in ambele
/// cazuri, deci acelasi script poate fi rulat si pe o baza careia lipseste
/// obiectul si pe una care il are intr-o versiune veche.
///
/// Se uita **doar la cod**: un `CREATE` dintr-un comentariu (`-- vechi: CREATE
/// PROCEDURE ...`) sau dintr-un string nu se atinge. Pana la audit, regula
/// lovea primul `CREATE` din text, deci rescria comentariul si lasa `CREATE`-ul
/// adevarat, iar scriptul cadea pe orice baza care avea deja obiectul.
/// `CREATE TABLE` nu intra: `CREATE OR ALTER TABLE` nu exista in T-SQL.
pub fn aplica_create_or_alter(cod: &str) -> String {
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
        if e_create
            && urm(1).is_some_and(|w| w.eq_ignore_ascii_case("or"))
            && urm(2).is_some_and(|w| w.eq_ignore_ascii_case("alter"))
        {
            return cod.to_string(); // deja CREATE OR ALTER
        }
        let Some(tip) = cuv.get(i + 1) else {
            continue;
        };
        if let Some((_, mare)) = TIPURI
            .iter()
            .find(|(mic, _)| tip.text.eq_ignore_ascii_case(mic))
        {
            return format!(
                "{}CREATE OR ALTER {}{}",
                &cod[..c.start],
                mare,
                &cod[tip.sfarsit..]
            );
        }
    }
    cod.to_string()
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
    if nume.chars().any(char::is_control) {
        return Err(Eroare::NumeNesigur {
            nume: nume.to_string(),
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
        for o in obiecte {
            verifica_nume(&o.obiect)?;
            if o.cod.trim().is_empty() {
                return Err(Eroare::ObiectCriptat {
                    obiect: o.obiect.clone(),
                });
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
/// declansata sa nu lase `NOEXEC` pornit in fereastra SSMS.
const SFARSIT_SCRIPT: &str = "SET NOEXEC OFF;\nGO\n";

fn antet(baza: &str, referinta: &str, acum: &str) -> String {
    format!(
        "-- DEPLOYER {baza} | referinta: {referinta} | {acum}\n{GARDA_VERSIUNE}USE {};\nGO\n\n",
        paranteze(baza)
    )
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
    format!(
        "-- ═══ {} ═══\nSET ANSI_NULLS {};\nSET QUOTED_IDENTIFIER {};{}\nGO\n{}\nGO\n\n",
        o.obiect,
        on(ansi),
        on(qi),
        nota,
        o.cod
    )
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
    scrie_deploy(director, referinta, pe_baza, mod_impartire, sarite, acum)
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

    let dir_revenire = director.join("revenire");
    std::fs::create_dir_all(&dir_revenire).map_err(|e| Eroare::ScriereDeploy {
        cale: dir_revenire.display().to_string(),
        detaliu: e.to_string(),
    })?;
    let mut nume_fisiere = NumeFisiere::default();
    let mut scrise = Vec::new();
    for (baza, pasi) in revenire {
        let cale = dir_revenire.join(nume_fisiere.unic(&slug(baza), "sql"));
        std::fs::write(&cale, text_revenire(baza, referinta, pasi, acum)).map_err(|e| {
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
    Ok(raport)
}

/// `[schema].[nume]` din `schema.nume`. Se taie la primul punct: o schema cu
/// punct in nume ar iesi gresit, dar `Baza` nu pastreaza partile separat.
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
            // Nu se ghiceste: scriptul se opreste vizibil la rulare.
            return format!(
                "-- ═══ {} ═══\nRAISERROR(N'GMotion: nu stiu sa sterg un obiect de tip {}; sterge-l de mana.', 16, 1);\nGO\n\n",
                o.obiect,
                alt.replace('\'', "''")
            );
        }
    };
    format!(
        "-- ═══ {} ═══ (creat de deploy)\nDROP {fel} IF EXISTS {};\nGO\n\n",
        o.obiect,
        nume_calificat(&o.obiect)
    )
}

/// Textul scriptului de revenire pentru o baza.
///
/// Intai se refac obiectele care existau (in ordinea dependentelor codului
/// vechi), abia apoi se sterg cele noi, in ordinea **inversa** dependentelor
/// codului nou: un view nou `WITH SCHEMABINDING` pe o functie noua ar bloca
/// stergerea functiei pana cand view-ul nu e intai refacut sau sters.
fn text_revenire(baza: &str, referinta: &str, pasi: &[PasRevenire], acum: &str) -> String {
    let mut reface = Vec::new();
    let mut sterse = Vec::new();
    for p in pasi {
        match p {
            PasRevenire::Reface(o) => reface.push(o.clone()),
            PasRevenire::Sterge(o) => sterse.push(o.clone()),
        }
    }
    let mut text = format!(
        "-- REVENIRE {baza} | anuleaza deploy-ul din {referinta} | salvat {acum}\n\
         -- Ruleaza-l doar ca sa anulezi deploy-ul: reface obiectele cum erau\n\
         -- inainte si sterge ce a creat deploy-ul.\n\
         {GARDA_VERSIUNE}USE {};\nGO\n\n",
        paranteze(baza)
    );
    for i in ordine_dupa_dependente(&reface) {
        text.push_str(&bloc(&reface[i]));
    }
    for i in ordine_dupa_dependente(&sterse).into_iter().rev() {
        text.push_str(&sterge(&sterse[i]));
    }
    text.push_str(SFARSIT_SCRIPT);
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
        let continut = format!("{continut}{SFARSIT_SCRIPT}");
        std::fs::write(&cale, continut).map_err(|e| Eroare::ScriereDeploy {
            cale: cale.display().to_string(),
            detaliu: e.to_string(),
        })?;
        fisiere.push(cale);
        Ok(())
    };

    match mod_impartire {
        ModImpartire::PerObiect => {
            for (baza, obiecte) in pe_baza {
                for (i, o) in obiecte.iter().enumerate() {
                    let nume = format!(
                        "{:03}_{}_{}_{}",
                        i + 1,
                        slug(baza),
                        tip_nume(&o.tip),
                        slug(&o.obiect)
                    );
                    scrie_fisier(nume, format!("{}{}", antet(baza, referinta, acum), bloc(o)))?;
                }
            }
        }
        ModImpartire::CateN(n) => {
            let n = n.max(1);
            for (baza, obiecte) in pe_baza {
                for (parte, grup) in obiecte.chunks(n).enumerate() {
                    let primul = parte * n + 1;
                    let nume = format!(
                        "{}_part{:02}_obj{}-{}",
                        slug(baza),
                        parte + 1,
                        primul,
                        primul + grup.len() - 1
                    );
                    let mut text = antet(baza, referinta, acum);
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
                text.push_str(&antet(baza, referinta, acum));
                for o in obiecte {
                    text.push_str(&bloc(o));
                }
                text.push('\n');
            }
            scrie_fisier("deploy_complet".to_string(), text)?;
        }
        ModImpartire::PerBaza => {
            for (baza, obiecte) in pe_baza {
                let mut text = antet(baza, referinta, acum);
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
    })
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn create_devine_create_or_alter() {
        for (intrare, asteptat) in [
            (
                "CREATE PROCEDURE dbo.p AS SELECT 1",
                "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "CREATE PROC dbo.p AS SELECT 1",
                "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "ALTER PROCEDURE dbo.p AS SELECT 1",
                "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "create view dbo.v as select 1",
                "CREATE OR ALTER VIEW dbo.v as select 1",
            ),
            (
                "ALTER FUNCTION dbo.f() RETURNS INT AS BEGIN RETURN 1 END",
                "CREATE OR ALTER FUNCTION dbo.f() RETURNS INT AS BEGIN RETURN 1 END",
            ),
            (
                "CREATE TRIGGER t ON tab AFTER INSERT AS SELECT 1",
                "CREATE OR ALTER TRIGGER t ON tab AFTER INSERT AS SELECT 1",
            ),
        ] {
            assert_eq!(aplica_create_or_alter(intrare), asteptat, "pe {intrare:?}");
        }
    }

    #[test]
    fn spatii_si_comentarii_inainte_nu_incurca() {
        let cod = "-- comentariu\n/* bloc */\n\n   CREATE   PROCEDURE  dbo.p AS SELECT 1";
        let rez = aplica_create_or_alter(cod);
        assert!(
            rez.starts_with("-- comentariu"),
            "antetul trebuie pastrat: {rez}"
        );
        assert!(rez.contains("CREATE OR ALTER PROCEDURE"));
        assert!(!rez.contains("CREATE   PROCEDURE"));
    }

    #[test]
    fn deja_create_or_alter_nu_se_atinge() {
        for cod in [
            "CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            "create or alter view dbo.v as select 1",
            "CREATE\n  OR\n  ALTER PROCEDURE dbo.p AS SELECT 1",
        ] {
            assert_eq!(aplica_create_or_alter(cod), cod, "pe {cod:?}");
        }
    }

    #[test]
    fn doar_prima_aparitie_se_schimba() {
        let cod = "CREATE PROCEDURE dbo.p AS\n-- CREATE PROCEDURE dbo.altceva\nEXEC('CREATE VIEW v AS SELECT 1')";
        let rez = aplica_create_or_alter(cod);
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
            assert_eq!(aplica_create_or_alter(cod), cod, "pe {cod:?}");
        }
    }

    #[test]
    fn create_table_nu_se_atinge() {
        // Nu e un tip pe care-l comparam, si `CREATE OR ALTER TABLE` nu exista.
        let cod = "CREATE TABLE t (a INT)";
        assert_eq!(aplica_create_or_alter(cod), cod);
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
        for (intrare, asteptat) in [
            (
                "-- vechi: CREATE PROCEDURE dbo.pCom_old\nCREATE PROCEDURE dbo.pCom AS SELECT 1",
                "-- vechi: CREATE PROCEDURE dbo.pCom_old\nCREATE OR ALTER PROCEDURE dbo.pCom AS SELECT 1",
            ),
            (
                "/* exemplu: ALTER VIEW dbo.v */ CREATE PROCEDURE dbo.p AS SELECT 1",
                "/* exemplu: ALTER VIEW dbo.v */ CREATE OR ALTER PROCEDURE dbo.p AS SELECT 1",
            ),
            (
                "/* use CREATE OR ALTER next time */\nCREATE PROCEDURE dbo.pStr AS SELECT 1",
                "/* use CREATE OR ALTER next time */\nCREATE OR ALTER PROCEDURE dbo.pStr AS SELECT 1",
            ),
            (
                "CREATE PROC /* x */ dbo.p AS SELECT 'ALTER VIEW v'",
                "CREATE OR ALTER PROCEDURE /* x */ dbo.p AS SELECT 'ALTER VIEW v'",
            ),
        ] {
            assert_eq!(aplica_create_or_alter(intrare), asteptat, "{intrare:?}");
        }
    }

    #[test]
    fn caracterele_care_isi_schimba_lungimea_la_minuscule_nu_strica_nimic() {
        // `K` (semnul Kelvin) si `İ`: minusculele au alta lungime in octeti.
        // Inainte: panica, respectiv `CCREATE OR ALTER PROCEDUREdbo...`.
        assert_eq!(
            aplica_create_or_alter("-- 5\u{212A}\nCREATE PROCEDURE dbo.p AS SELECT 1"),
            "-- 5\u{212A}\nCREATE OR ALTER PROCEDURE dbo.p AS SELECT 1"
        );
        assert_eq!(
            aplica_create_or_alter("-- İstanbul\nCREATE PROCEDURE dbo.pTurc AS SELECT 1"),
            "-- İstanbul\nCREATE OR ALTER PROCEDURE dbo.pTurc AS SELECT 1"
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
            t.contains("SET ANSI_NULLS OFF;\nSET QUOTED_IDENTIFIER ON;\nGO\nCREATE OR ALTER PROC dbo.pVechi"),
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
            t.contains("SET ANSI_NULLS ON;\nSET QUOTED_IDENTIFIER ON; -- setari necunoscute"),
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
            assert!(t.ends_with("SET NOEXEC OFF;\nGO\n"), "{t}");
        }
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
        assert!(t.starts_with("-- DEPLOYER Client1 | referinta: Ref | 2026-11-17 10:00\n"));
        assert!(t.contains("USE [Client1];\nGO\n"));
        assert_eq!(t.matches("-- ═══").count(), 3);
        assert_eq!(
            t.matches("\nGO\n").count(),
            9,
            "garda, USE, doua per obiect (setarile, apoi codul), sfarsitul"
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
        let d = dir("gol");
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
                cod: "SELECT 1".into(),
                setari: None,
            }],
        );
        let r = scrie(&d, "Ref", &m, ModImpartire::PerBaza, 0, "a").unwrap();
        assert_eq!(r.fisiere.len(), 1);
        assert_eq!(r.fisiere[0].parent().unwrap(), d.as_path());
        std::fs::remove_dir_all(&d).ok();
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
        assert!(sql.contains("SELECT \"vechi\""), "codul din tinta:\n{sql}");
        assert!(
            sql.contains("SET QUOTED_IDENTIFIER OFF;"),
            "setarile tintei, nu ale referintei:\n{sql}"
        );
        assert!(sql.contains("DROP VIEW IF EXISTS [dbo].[vNou];"));
        assert!(sql.contains("DROP FUNCTION IF EXISTS [dbo].[fNou];"));
        assert!(sql.trim_end().ends_with("SET NOEXEC OFF;\nGO"));
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn revenirea_reface_intai_apoi_sterge_dependentii_inainte() {
        let (_, rev) = cu_revenire();
        let sql = text_revenire("Client1", "Ref", &rev["Client1"], "t");
        let poz = |s: &str| {
            sql.find(s)
                .unwrap_or_else(|| panic!("lipseste {s}:\n{sql}"))
        };
        assert!(
            poz("SELECT \"vechi\"") < poz("DROP VIEW"),
            "refacerea inainte de stergeri"
        );
        assert!(
            poz("DROP VIEW") < poz("DROP FUNCTION"),
            "view-ul cu SCHEMABINDING blocheaza stergerea functiei:\n{sql}"
        );
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
}
