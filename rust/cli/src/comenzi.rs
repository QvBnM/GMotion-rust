//! Implementarea comenzilor.
//!
//! **Se strica astfel:** o comanda face altceva decat spune, sau iese cu codul
//! 0 cand a esuat. Al doilea e cel rau: o rulare programata ar raporta succes.
//!
//! Fiecare comanda e o functie care intoarce `Result<(), String>`. Mesajul de
//! eroare pastreaza codul original (`GM-DB-001`, ...), ca raportul
//! utilizatorului sa duca direct la cauza.

use std::path::{Path, PathBuf};

use gmotion_db::Instantaneu;
use gmotion_modul_diff_obiecte as modul;
use gmotion_platforma::Configurare;

use crate::afisare::{self, Format};
use crate::sursa::{lista, ArgumenteSursa};

use gmotion_platforma::timp::{acum, acum_compact};

pub fn diagnostic(format: Format) -> Result<(), String> {
    let cfg_cale = gmotion_platforma::cai::cale_configurare()
        .map(|c| c.display().to_string())
        .unwrap_or_else(|e| format!("(indisponibil: {e})"));
    let dir = gmotion_platforma::cai::director_date()
        .map(|c| c.display().to_string())
        .unwrap_or_else(|e| format!("(indisponibil: {e})"));
    let cfg = gmotion_platforma::cai::cale_configurare()
        .and_then(Configurare::citeste)
        .map_err(|e| e.to_string())?;

    #[derive(serde::Serialize)]
    struct D<'a> {
        versiune: &'a str,
        sistem: &'a str,
        director_date: String,
        configurare: String,
        protejare_criptata: bool,
        auth_integrata_disponibila: bool,
        tipuri_obiecte: Vec<&'a str>,
        configurare_activa: Configurare,
    }
    let d = D {
        versiune: env!("CARGO_PKG_VERSION"),
        sistem: std::env::consts::OS,
        director_date: dir,
        configurare: cfg_cale,
        protejare_criptata: gmotion_platforma::protejare::e_criptat(),
        auth_integrata_disponibila: gmotion_db::sqlserver::auth_integrata_disponibila(),
        tipuri_obiecte: gmotion_core::TIPURI_OBIECTE.to_vec(),
        configurare_activa: cfg,
    };

    match format {
        Format::Json => afisare::scrie_json(&d).map_err(|e| e.to_string()),
        Format::Text => {
            println!("GMotion {} pe {}", d.versiune, d.sistem);
            println!("Director de date   : {}", d.director_date);
            println!("Configurare        : {}", d.configurare);
            println!(
                "Sesiuni criptate   : {}",
                if d.protejare_criptata {
                    "da (DPAPI)"
                } else {
                    "NU - doar in dezvoltare, pe Windows se foloseste DPAPI"
                }
            );
            println!(
                "Auth integrata     : {}",
                if d.auth_integrata_disponibila {
                    "disponibila"
                } else {
                    "indisponibila pe acest sistem"
                }
            );
            println!("Tipuri comparate   : {}", d.tipuri_obiecte.join(", "));
            println!(
                "Timeout conectare  : {} s",
                d.configurare_activa.timeout_conectare_s
            );
            Ok(())
        }
    }
}

pub fn servere(format: Format) -> Result<(), String> {
    let cfg = gmotion_platforma::cai::cale_configurare()
        .and_then(Configurare::citeste)
        .map_err(|e| e.to_string())?;
    let timeout = std::time::Duration::from_millis(cfg.timeout_descoperire_ms);
    format.progres(&format!(
        "Caut instante SQL Server in retea ({} ms)...",
        cfg.timeout_descoperire_ms
    ));
    let gasite = gmotion_db::cauta_instante(timeout).map_err(|e| e.to_string())?;

    match format {
        Format::Json => afisare::scrie_json(&gasite).map_err(|e| e.to_string()),
        Format::Text => {
            if gasite.is_empty() {
                println!(
                    "Nicio instanta gasita.\n\n\
                     Nu inseamna neaparat ca nu exista. Cauze obisnuite:\n\
                     - serviciul SQL Browser e oprit pe server\n\
                     - UDP 1434 e blocat de firewall\n\
                     - serverul e in alt subnet (broadcast-ul nu traverseaza)\n\
                     - instanta are Hide Instance = Yes\n\n\
                     Poti da serverul direct: --server NUME"
                );
                return Ok(());
            }
            let randuri: Vec<Vec<String>> = gasite
                .iter()
                .map(|i| {
                    vec![
                        i.server.clone(),
                        i.ip.clone(),
                        i.port.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
                        i.versiune.clone().unwrap_or_else(|| "-".into()),
                        [(i.cluster, "CLUSTER"), (i.implicita, "IMPLICITA")]
                            .iter()
                            .filter(|(da, _)| *da)
                            .map(|(_, e)| *e)
                            .collect::<Vec<_>>()
                            .join(" "),
                    ]
                })
                .collect();
            print!(
                "{}",
                afisare::tabel(&["SERVER", "IP", "PORT", "VERSIUNE", "ETICHETE"], &randuri)
            );
            Ok(())
        }
    }
}

pub fn baze(args: &ArgumenteSursa, format: Format) -> Result<(), String> {
    let mut s = args.deschide()?;
    let b = s.baze().map_err(|e| e.to_string())?;
    match format {
        Format::Json => afisare::scrie_json(&b).map_err(|e| e.to_string()),
        Format::Text => {
            if b.is_empty() {
                println!("Nicio baza disponibila.");
            }
            for x in &b {
                println!("{x}");
            }
            Ok(())
        }
    }
}

pub fn instantaneu(
    args: &ArgumenteSursa,
    baze_cerute: Option<&str>,
    iesire: &Path,
    nota: &str,
    format: Format,
) -> Result<(), String> {
    let mut s = args.deschide()?;
    let lista_baze = match baze_cerute {
        Some(b) => lista(b),
        None => {
            format.progres("Nu s-au dat baze: iau toate bazele de pe server.");
            s.baze().map_err(|e| e.to_string())?
        }
    };
    if lista_baze.is_empty() {
        return Err("nicio baza de citit".into());
    }

    // Numele serverului, nu descrierea pentru jurnal (care vorbeste si despre
    // autentificare). Din alt instantaneu: descrierea lui, fara acreditari.
    let eticheta = args.eticheta_server().unwrap_or_else(|| s.descriere());
    let citite = s
        .obiecte_multe(&lista_baze, &mut |i, n, baza| {
            if !baza.is_empty() {
                format.progres(&format!("[{}/{n}] citesc {baza}...", i + 1));
            }
        })
        .map_err(|e| e.to_string())?;

    let mut inst = Instantaneu::nou(eticheta, acum());
    inst.nota = nota.to_string();
    let mut total = 0usize;
    for (baza, obiecte) in citite {
        total += obiecte.len();
        inst.baze.insert(baza, obiecte);
    }
    inst.scrie(iesire).map_err(|e| e.to_string())?;

    format.progres(&format!(
        "Scris {} ({} baze, {total} obiecte).\n\
         Reia-l cu: gmotion compara --sursa {} --referinta <BAZA>",
        iesire.display(),
        inst.baze.len(),
        iesire.display()
    ));
    Ok(())
}

/// Rezultatul unei comparari, plus obiectele pregatite — de care au nevoie
/// `deploy` si `export` ca sa nu reciteasca.
struct Rulare {
    comparare: modul::Comparare,
    referinta: modul::Baza,
    /// Bazele comparate, pastrate pentru detaliile cerute cu `--detalii`.
    altele: Vec<(String, modul::Baza)>,
}

fn ruleaza_comparare(
    args: &ArgumenteSursa,
    referinta: &str,
    baze_cerute: Option<&str>,
    format: Format,
) -> Result<Rulare, String> {
    let mut s = args.deschide()?;
    let disponibile = s.baze().map_err(|e| e.to_string())?;
    if !disponibile.iter().any(|b| b == referinta) {
        return Err(format!(
            "baza martor {referinta:?} nu exista in sursa data. Disponibile: {}",
            disponibile.join(", ")
        ));
    }
    let altele_nume: Vec<String> = match baze_cerute {
        Some(b) => lista(b),
        None => disponibile
            .iter()
            .filter(|b| *b != referinta)
            .cloned()
            .collect(),
    };
    let altele_nume: Vec<String> = altele_nume.into_iter().filter(|b| b != referinta).collect();

    // Cerintele modulului se verifica inainte de a citi ceva: altfel am citi
    // degeaba, posibil minute.
    use gmotion_platforma::Modul;
    let m = modul::ModulDiffObiecte;
    m.cerinte()
        .verifica(m.nume(), referinta, &altele_nume)
        .map_err(|e| e.to_string())?;

    // Referinta si celelalte baze dintr-o data: de pe un server se citesc in
    // paralel (`SursaSqlServer::obiecte_multe`), ca in v3.4.
    let mut toate_nume = Vec::with_capacity(altele_nume.len() + 1);
    toate_nume.push(referinta.to_string());
    toate_nume.extend(altele_nume.iter().cloned());
    let n_altele = altele_nume.len();
    let mut citite = s
        .obiecte_multe(&toate_nume, &mut |i, _, baza| {
            if baza.is_empty() {
                return;
            }
            if i == 0 {
                format.progres(&format!("Citesc martorul {baza}..."));
            } else {
                format.progres(&format!("[{i}/{n_altele}] citesc {baza}..."));
            }
        })
        .map_err(|e| e.to_string())?
        .into_iter();
    let ref_obiecte = match citite.next() {
        Some((_, obiecte)) => modul::pregateste(&obiecte),
        None => return Err("martorul nu a putut fi citit".into()),
    };
    let altele: Vec<(String, modul::Baza)> = citite
        .map(|(baza, obiecte)| (baza, modul::pregateste(&obiecte)))
        .collect();

    let comparare = modul::compara(referinta, &ref_obiecte, &altele, |p| {
        if p.gata {
            format.progres(&p.mesaj);
        }
    });
    Ok(Rulare {
        comparare,
        referinta: ref_obiecte,
        altele,
    })
}

pub fn compara(
    args: &ArgumenteSursa,
    referinta: &str,
    baze_cerute: Option<&str>,
    detalii: bool,
    format: Format,
) -> Result<(), String> {
    let r = ruleaza_comparare(args, referinta, baze_cerute, format)?;
    let detalii = if detalii {
        format.progres("Calculez in ce difera fiecare obiect...");
        detalii_randuri(&r)
    } else {
        afisare::Detalii::new()
    };
    match format {
        Format::Json => afisare::scrie_json(&afisare::comparare_json(&r.comparare, &detalii))
            .map_err(|e| e.to_string()),
        Format::Text => {
            print!("{}", afisare::comparare_text(&r.comparare, &detalii));
            Ok(())
        }
    }
}

/// In ce difera fiecare obiect **diferit** si cate linii se misca. Numai
/// pentru `Diferit`: la `Lipsa` / `Extra` / `Criptat` nu exista cu ce compara.
fn detalii_randuri(r: &Rulare) -> afisare::Detalii {
    let mut rez = afisare::Detalii::new();
    for rand in &r.comparare.randuri {
        if rand.categorie != modul::Categorie::Diferit {
            continue;
        }
        let (Some(a), Some(b)) = (
            r.referinta.get(&rand.obiect),
            r.altele
                .iter()
                .find(|(nume, _)| *nume == rand.baza)
                .and_then(|(_, baza)| baza.get(&rand.obiect)),
        ) else {
            continue;
        };
        let setari = modul::diferente_ascunse(a, b);
        rez.insert(
            rand.id,
            modul::detalii_obiect(&a.afisare, &b.afisare, &setari),
        );
    }
    rez
}

/// Ce ai marcat ca revizuit, pe o comparare: arata, marcheaza, scoate, sterge.
///
/// Compararea se reface de fiecare data, fiindca bifa e legata de amprenta
/// continutului: fara definitiile de acum nu se poate spune daca mai e
/// valabila. Asa se vede si cate bife au cazut fiindca obiectul s-a schimbat
/// pe server — informatia pentru care exista comanda.
/// Sesiunea salvata a referintei, sau una noua. Aceeasi pentru `revizuire`
/// si `editare`: progresul si textele finale stau impreuna, ca in fereastra.
fn sesiune_sau_noua(
    sesiuni: &gmotion_platforma::Sesiuni,
    args: &ArgumenteSursa,
    referinta: &str,
) -> Result<gmotion_platforma::Sesiune, String> {
    let identitate = args.identitate()?;
    Ok(sesiuni
        .incarca(&identitate, referinta)
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| {
            gmotion_platforma::Sesiune::noua(
                identitate.clone(),
                if args.sursa.is_some() {
                    "instantaneu"
                } else if args.auth.utilizator.is_some() {
                    "SQL"
                } else {
                    "Windows"
                },
                referinta,
                acum(),
            )
        }))
}

#[allow(clippy::too_many_arguments)]
pub fn revizuire(
    args: &ArgumenteSursa,
    referinta: &str,
    baze_cerute: Option<&str>,
    schimbari: SchimbariRevizuire,
    goleste: bool,
    format: Format,
) -> Result<(), String> {
    let SchimbariRevizuire {
        marcheaza,
        omite,
        scoate,
        nota,
    } = schimbari;
    let sesiuni =
        gmotion_platforma::Sesiuni::pentru_modul(modul::NUME).map_err(|e| e.to_string())?;
    if goleste {
        let sters = sesiuni
            .sterge(&args.identitate()?, referinta)
            .map_err(|e| e.to_string())?;
        format.progres(if sters {
            "Progresul salvat a fost sters."
        } else {
            "Nu exista progres salvat pentru martorul asta."
        });
        return Ok(());
    }

    let r = ruleaza_comparare(args, referinta, baze_cerute, format)?;
    let mut sesiune = sesiune_sau_noua(&sesiuni, args, referinta)?;
    let mut revizuire = modul::Revizuire::din_sesiune(&sesiune);
    let (valabile, cazute) = revizuire.valabile(&r.comparare.randuri);

    let mut schimbat = false;
    let mut necunoscute: Vec<String> = Vec::new();
    if let Some(l) = marcheaza {
        for obiect in lista(l) {
            // pe toate bazele comparate, ca `--omite` si `--scoate`: cu
            // `.find` se bifa doar prima, tacut (F-C1 din auditul QA)
            let mut gasit = false;
            for rand in r
                .comparare
                .randuri
                .iter()
                .filter(|x| x.obiect.eq_ignore_ascii_case(&obiect))
            {
                revizuire.marcheaza(rand);
                gasit = true;
            }
            schimbat |= gasit;
            if !gasit {
                necunoscute.push(obiect);
            }
        }
    }
    if let Some(l) = omite {
        for obiect in lista(l) {
            let mut gasit = false;
            for rand in r
                .comparare
                .randuri
                .iter()
                .filter(|x| x.obiect.eq_ignore_ascii_case(&obiect))
            {
                revizuire.omite(rand);
                gasit = true;
            }
            schimbat |= gasit;
            if !gasit {
                necunoscute.push(obiect);
            }
        }
    }
    if let Some(l) = scoate {
        for obiect in lista(l) {
            // pe toate bazele, nu doar pe prima: acelasi obiect poate fi
            // bifat separat pe fiecare baza comparata (deci fara `any`, care
            // s-ar opri la prima)
            let mut gasit = false;
            for x in r
                .comparare
                .randuri
                .iter()
                .filter(|x| x.obiect.eq_ignore_ascii_case(&obiect))
            {
                gasit |= revizuire.scoate(&x.baza, &x.obiect);
                gasit |= revizuire.scoate_omis(&x.baza, &x.obiect);
            }
            schimbat |= gasit;
            if !gasit {
                necunoscute.push(obiect);
            }
        }
    }
    if let Some((obiect, text)) = nota {
        let mut gasit = false;
        for x in r
            .comparare
            .randuri
            .iter()
            .filter(|x| x.obiect.eq_ignore_ascii_case(obiect))
        {
            schimbat |= revizuire.pune_nota(&x.baza, &x.obiect, text);
            gasit = true;
        }
        if !gasit {
            necunoscute.push(obiect.to_string());
        }
    }
    if !necunoscute.is_empty() {
        return Err(format!(
            "obiecte care nu sunt in comparare (sau nu erau bifate): {}",
            necunoscute.join(", ")
        ));
    }
    if schimbat {
        revizuire.curata(&r.comparare.randuri, &r.comparare.baze);
        revizuire.in_sesiune(&mut sesiune);
        sesiune.salvata = acum();
        sesiune.adauga_baze(&r.comparare.baze);
        sesiuni.salveaza(&sesiune).map_err(|e| e.to_string())?;
    }

    let (valabile_acum, _) = revizuire.valabile(&r.comparare.randuri);
    let total = r.comparare.randuri.len();
    match format {
        Format::Json => {
            // numai bifele inca valabile: una cazuta nu mai spune nimic
            // despre ce e acum pe server
            let obiecte: Vec<&str> = r
                .comparare
                .randuri
                .iter()
                .filter(|x| revizuire.e_revizuit(x))
                .map(|x| x.obiect.as_str())
                .collect();
            let omise: Vec<&str> = r
                .comparare
                .randuri
                .iter()
                .filter(|x| revizuire.e_omis(x))
                .map(|x| x.obiect.as_str())
                .collect();
            let note: Vec<serde_json::Value> = revizuire
                .note()
                .into_iter()
                .map(|(b, o, t)| serde_json::json!({ "baza": b, "obiect": o, "text": t }))
                .collect();
            afisare::scrie_json(&serde_json::json!({
                "referinta": referinta,
                "note": note,
                "omise": omise,
                "revizuite": valabile_acum,
                "total": total,
                "cazute": cazute,
                "obiecte": obiecte,
            }))
            .map_err(|e| e.to_string())
        }
        Format::Text => {
            println!("Revizuite: {valabile_acum} din {total} obiecte care difera.");
            if cazute > 0 {
                println!(
                    "{cazute} bife au cazut: obiectele s-au schimbat de cand au fost revizuite."
                );
            }
            if valabile_acum > 0 {
                println!();
                for x in r
                    .comparare
                    .randuri
                    .iter()
                    .filter(|x| revizuire.e_revizuit(x))
                {
                    println!("  {}  {}", x.baza, x.obiect);
                }
            }
            let omise: Vec<&modul::Rand> = r
                .comparare
                .randuri
                .iter()
                .filter(|x| revizuire.e_omis(x))
                .collect();
            if !omise.is_empty() {
                println!("\nOmise (nu intra in deploy): {}", omise.len());
                for x in omise {
                    println!("  {}  {}", x.baza, x.obiect);
                }
            }
            let note = revizuire.note();
            if !note.is_empty() {
                println!("\nNote: {}", note.len());
                for (b, o, t) in note {
                    println!(
                        "  {}  {}  {}",
                        b,
                        o,
                        afisare::fara_control(&t.replace('\n', " / "))
                    );
                }
            }
            let _ = valabile;
            Ok(())
        }
    }
}

/// Ce schimba `revizuire` in progresul salvat.
pub struct SchimbariRevizuire<'a> {
    pub marcheaza: Option<&'a str>,
    pub omite: Option<&'a str>,
    pub scoate: Option<&'a str>,
    /// `(schema.obiect, text)`; textul gol sterge nota.
    pub nota: Option<(&'a str, &'a str)>,
}

/// Impactul estimat al fiecarui obiect care difera (`modul::impact`), cel mai
/// mare primul. Cu `obiect`, doar el, plus cine il foloseste (in baza
/// comparata) si ce foloseste el (in Martor; la Extra, in baza comparata) —
/// aceleasi liste ca panoul de detalii din lista.
pub fn impact(
    args: &ArgumenteSursa,
    referinta: &str,
    baze_cerute: Option<&str>,
    obiect: Option<&str>,
    format: Format,
) -> Result<(), String> {
    let r = ruleaza_comparare(args, referinta, baze_cerute, format)?;
    let dep: std::collections::BTreeMap<&str, modul::Dependente> = r
        .altele
        .iter()
        .map(|(nume, baza)| (nume.as_str(), modul::impact::dependente(baza)))
        .collect();
    let dep_martor = if obiect.is_some() {
        modul::impact::dependente(&r.referinta)
    } else {
        modul::Dependente::default()
    };
    let foloseste = |x: &modul::Rand| -> Vec<String> {
        let de_unde = if x.categorie == modul::Categorie::Extra {
            dep.get(x.baza.as_str())
        } else {
            Some(&dep_martor)
        };
        de_unde
            .and_then(|d| d.foloseste.get(&x.obiect))
            .cloned()
            .unwrap_or_default()
    };
    let mut randuri: Vec<(&modul::Rand, modul::Impact, Vec<String>)> = r
        .comparare
        .randuri
        .iter()
        .filter(|x| obiect.is_none_or(|o| x.obiect.eq_ignore_ascii_case(o)))
        .map(|x| {
            let cine = dep
                .get(x.baza.as_str())
                .and_then(|d| d.folosit_de.get(&x.obiect))
                .cloned()
                .unwrap_or_default();
            (x, modul::impact::estimeaza(x, cine.len()), cine)
        })
        .collect();
    if let Some(o) = obiect {
        if randuri.is_empty() {
            return Err(format!("obiectul {o:?} nu difera in nicio baza comparata"));
        }
    }
    randuri.sort_by(|a, b| {
        b.1.nivel
            .cmp(&a.1.nivel)
            .then(b.1.folosit_de.cmp(&a.1.folosit_de))
            .then(a.0.obiect.cmp(&b.0.obiect))
    });
    match format {
        Format::Json => {
            let v: Vec<serde_json::Value> = randuri
                .iter()
                .map(|(x, i, cine)| {
                    let mut v = serde_json::json!({
                        "baza": x.baza,
                        "obiect": x.obiect,
                        "nivel": i.nivel,
                        "folosit_de": i.folosit_de,
                        "motive": i.motive,
                    });
                    if obiect.is_some() {
                        v["cine_il_foloseste"] = serde_json::json!(cine);
                        v["foloseste"] = serde_json::json!(foloseste(x));
                    }
                    v
                })
                .collect();
            afisare::scrie_json(&serde_json::json!({ "referinta": referinta, "obiecte": v }))
                .map_err(|e| e.to_string())
        }
        Format::Text => {
            for (x, i, cine) in &randuri {
                println!(
                    "{:<8} {}  {}  {}",
                    i.nivel.eticheta(),
                    afisare::fara_control(&x.baza),
                    afisare::fara_control(&x.obiect),
                    i.motive.join(", ")
                );
                if obiect.is_some() {
                    for c in cine {
                        println!("           folosit de {}", afisare::fara_control(c));
                    }
                    for c in foloseste(x) {
                        println!("           foloseste  {}", afisare::fara_control(&c));
                    }
                }
            }
            if randuri.is_empty() {
                println!("Niciun obiect nu difera.");
            }
            Ok(())
        }
    }
}

/// Alinierea si diferentele asa cum le numara editorul: pe grupuri.
fn grupuri_editare(sursa: &str, curent: &str) -> (gmotion_core::Aliniere, Vec<Vec<usize>>) {
    let a = gmotion_core::aliniaza(sursa, curent);
    let plasari = gmotion_core::plasari_aliniere(&a, curent, false);
    let grupuri = gmotion_core::grupuri_diferente(&a.diff, &plasari);
    (a, grupuri)
}

/// Ce face `editare` cu textul final.
pub enum ActiuneEditare {
    /// Doar arata diferentele ramase, numerotate.
    Arata,
    /// Aplica diferentele cu numerele astea (de la 1, ca in editor); cu o
    /// linie (de la 1), o singura diferenta lasata acolo, nu pe locul sugerat.
    Aplica(Vec<usize>, Option<usize>),
    AplicaToate,
    Reset,
    /// Textul final, scris de mana, dintr-un fisier.
    DinFisier(PathBuf),
}

/// Textul final al unui obiect: ce face in editor „Aplica", „Aplica toate",
/// „Reset" si „Editeaza", aici fara fereastra (CLAUDE.md §3). Se salveaza in
/// aceeasi sesiune ca revizuirea, deci fereastra il vede, si invers.
#[allow(clippy::too_many_arguments)]
pub fn editare(
    args: &ArgumenteSursa,
    referinta: &str,
    baza: &str,
    obiect: &str,
    actiune: ActiuneEditare,
    arata_final: bool,
    format: Format,
) -> Result<(), String> {
    // cu `--arata-final`, iesirea standard e **fisierul** cerut de om: progresul
    // merge pe erori, ca la `--json`. Altfel textul redirectat incepea cu
    // „Citesc referinta..." (F-C3 din auditul QA).
    let progres = if arata_final { Format::Json } else { format };
    let r = ruleaza_comparare(args, referinta, Some(baza), progres)?;
    let rand = r
        .comparare
        .randuri
        .iter()
        .find(|x| x.baza == baza && x.obiect.eq_ignore_ascii_case(obiect))
        .ok_or_else(|| format!("{obiect} nu difera intre {referinta} si {baza}"))?
        .clone();
    let text_sursa = r
        .referinta
        .get(&rand.obiect)
        .map(|o| o.afisare.clone())
        .ok_or_else(|| format!("{obiect} nu exista in {referinta}: nu e nimic de adus"))?;
    let text_baza = r
        .altele
        .iter()
        .find(|(b, _)| b == baza)
        .and_then(|(_, o)| o.get(&rand.obiect))
        .map(|o| o.afisare.clone())
        .unwrap_or_default();

    let sesiuni =
        gmotion_platforma::Sesiuni::pentru_modul(modul::NUME).map_err(|e| e.to_string())?;
    let mut sesiune = sesiune_sau_noua(&sesiuni, args, referinta)?;
    let mut editari = modul::Editari::din_sesiune(&sesiune);

    let schimbat = match actiune {
        ActiuneEditare::Arata => false,
        ActiuneEditare::Aplica(numere, la) => {
            let curent = editari.curent(&rand, &text_baza).to_string();
            let (a, grupuri) = grupuri_editare(&text_sursa, &curent);
            let mut blocuri = Vec::new();
            for n in numere {
                let g = n
                    .checked_sub(1)
                    .and_then(|i| grupuri.get(i))
                    .ok_or_else(|| {
                        format!(
                            "nu exista diferenta {n}: obiectul are {} diferente ramase",
                            grupuri.len()
                        )
                    })?;
                blocuri.extend_from_slice(g);
            }
            match la {
                None => {
                    editari.aplica_blocuri(&rand, &text_sursa, &text_baza, false, &blocuri);
                }
                Some(linie) => {
                    let total = curent.lines().count();
                    if linie == 0 || linie > total + 1 {
                        return Err(format!(
                            "--la {linie}: textul curent are {total} linii (1..={})",
                            total + 1
                        ));
                    }
                    let loc = gmotion_core::loc_la_linie(&a, linie - 1);
                    editari.aplica_blocuri_la(&rand, &text_sursa, &text_baza, false, &blocuri, loc);
                }
            }
            true
        }
        ActiuneEditare::AplicaToate => {
            editari.aplica_toate(&rand, &text_sursa);
            true
        }
        ActiuneEditare::Reset => editari.reset(&rand),
        ActiuneEditare::DinFisier(cale) => {
            let text = std::fs::read_to_string(&cale)
                .map_err(|e| format!("nu pot citi {}: {e}", cale.display()))?;
            let text = gmotion_core::pregateste_afisare(&text);
            // un text care n-ar trece de deploy se refuza acum, nu abia la
            // deploy (B2, F-C6 din auditul 2026-11-22)
            modul::verifica_text_final(&rand.obiect, &rand.tip, &text)
                .map_err(|e| e.to_string())?;
            editari.pune(&rand, text, true);
            true
        }
    };
    if schimbat {
        editari.curata(&r.comparare.randuri, &r.comparare.baze);
        editari.in_sesiune(&mut sesiune);
        sesiune.salvata = acum();
        sesiune.adauga_baze(&r.comparare.baze);
        sesiuni.salveaza(&sesiune).map_err(|e| e.to_string())?;
    }

    let curent = editari.curent(&rand, &text_baza).to_string();
    // numerotate ca in editor: instructiunile consecutive care merg in acelasi
    // gol sunt o singura diferenta (`core::grupuri_diferente`)
    let (a, grupuri) = grupuri_editare(&text_sursa, &curent);
    let diferente: Vec<usize> = grupuri.iter().map(|g| g[0]).collect();
    let manual = editari.text_final(&rand).is_some_and(|t| t.manual);
    let editat = editari.text_final(&rand).is_some();
    if format == Format::Json {
        let lista_dif: Vec<serde_json::Value> = diferente
            .iter()
            .enumerate()
            .map(|(i, &b)| {
                let d = &a.diff[b];
                serde_json::json!({
                    "nr": i + 1,
                    "tip": d.tip.eticheta(),
                    "sursa": d.stanga,
                    "baza": d.dreapta,
                    "instructiuni": grupuri[i].len(),
                })
            })
            .collect();
        return afisare::scrie_json(&serde_json::json!({
            "obiect": rand.obiect,
            "baza": baza,
            "editat": editat,
            "manual": manual,
            "diferente": lista_dif,
            "text_final": if editat { Some(curent.as_str()) } else { None },
        }))
        .map_err(|e| e.to_string());
    }
    if arata_final {
        println!("{curent}");
        return Ok(());
    }
    println!(
        "{} in {baza}: {}{}",
        rand.obiect,
        if editat {
            "text final salvat"
        } else {
            "needitat (textul bazei modificate)"
        },
        if manual { ", scris si de mana" } else { "" }
    );
    if diferente.is_empty() {
        println!("Nicio diferenta ramasa fata de {referinta}.");
    }
    for (i, &b) in diferente.iter().enumerate() {
        let d = &a.diff[b];
        let prima = |t: &Option<String>| {
            t.as_deref()
                .and_then(|t| t.lines().next())
                .unwrap_or("—")
                .chars()
                .take(70)
                .collect::<String>()
        };
        println!(
            "  {:>3}. {:<9} martor:     {}\n       {:<9} modificata: {}",
            i + 1,
            d.tip.eticheta(),
            prima(&d.stanga),
            "",
            prima(&d.dreapta)
        );
    }
    Ok(())
}

pub fn export(
    args: &ArgumenteSursa,
    referinta: &str,
    baze_cerute: Option<&str>,
    iesire: Option<PathBuf>,
    format: Format,
) -> Result<(), String> {
    let r = ruleaza_comparare(args, referinta, baze_cerute, format)?;
    if r.comparare.randuri.is_empty() {
        return Err("nimic de exportat: bazele sunt identice cu martorul".into());
    }
    let cale =
        iesire.unwrap_or_else(|| PathBuf::from(format!("gmotion_export_{}.csv", acum_compact())));
    // starea de revizuire vine din sesiune: altfel „Rezolvat" si „Omis" ies
    // mereu NU, iar exportul minte despre munca facuta (F-C2 din auditul QA)
    let (revizuire, _) = salvate(args, referinta)?;
    let n = modul::scrie_csv(&cale, &r.comparare.randuri, |i| {
        r.comparare
            .randuri
            .get(i)
            .map_or_else(modul::Stare::default, |rand| modul::Stare {
                rezolvat: revizuire.e_revizuit(rand),
                omis: revizuire.e_omis(rand),
            })
    })
    .map_err(|e| e.to_string())?;
    format.progres(&format!("Scris {} ({n} randuri).", cale.display()));
    Ok(())
}

/// Planul din compararea de acum: randurile alese (toate, sau `--doar`).
fn plan_din(args: &ArgumenteSursa, r: &Rulare, doar: Option<&str>) -> Result<modul::Plan, String> {
    let filtru = doar.map(lista);
    // ce s-a omis (in fereastra sau cu `revizuire --omite`) nu intra in deploy,
    // ca in v3.4; cu `--doar` se poate cere totusi explicit
    let (omise, editari) = salvate(args, &r.comparare.referinta)?;
    // amprenta textelor finale intra in plan: `deploy --plan` le verifica
    let texte = editari.pentru_deploy(&r.comparare.randuri);
    Ok(modul::Plan::din_comparare(
        &r.comparare,
        &r.referinta,
        &r.altele,
        |rand| match &filtru {
            Some(f) => f.iter().any(|x| x == &rand.obiect),
            None => !omise.e_omis(rand),
        },
        &acum(),
    )
    .cu_texte_finale(&texte))
}

/// Dupa un deploy scris: obiectele din plan se tin minte ca deployate, cu
/// amprenta de acum. Fereastra arata coloana „Deploy" din aceeasi stare, deci
/// un deploy facut din `cli` se vede si acolo.
///
/// Nu opreste deploy-ul daca sesiunea nu se poate salva: scripturile sunt deja
/// pe disc, iar pierderea e doar a marcajului.
fn marcheaza_deployatele(
    args: &ArgumenteSursa,
    referinta: &str,
    plan: &modul::Plan,
    randuri: &[modul::Rand],
) {
    let Ok(sesiuni) = gmotion_platforma::Sesiuni::pentru_modul(modul::NUME) else {
        return;
    };
    let Ok(mut sesiune) = sesiune_sau_noua(&sesiuni, args, referinta) else {
        return;
    };
    let mut revizuire = modul::Revizuire::din_sesiune(&sesiune);
    let mut marcate = 0;
    for pas in &plan.pasi {
        if let Some(rand) = randuri
            .iter()
            .find(|r| r.baza == pas.baza && r.obiect == pas.obiect)
        {
            revizuire.marcheaza_deployat(rand);
            marcate += 1;
        }
    }
    if marcate == 0 {
        return;
    }
    revizuire.in_sesiune(&mut sesiune);
    sesiune.salvata = acum();
    if let Err(e) = sesiuni.salveaza(&sesiune) {
        tracing::warn!(target: "gmotion::cli", eroare = %e, "marcajul de deploy nu s-a salvat");
    }
}

/// Revizuirea (cu omisele) si textele finale salvate pentru referinta.
///
/// Lipsa sesiunii da stari goale: deploy-ul merge si fara ea, cu codul din
/// referinta. O sesiune care **exista** dar nu se poate citi opreste comanda:
/// altfel obiectele omise ar ajunge in deploy si textele finale s-ar pierde,
/// fara niciun semn (B1 din `cercetare/audit-qa-2026-11-22.md`).
fn salvate(
    args: &ArgumenteSursa,
    referinta: &str,
) -> Result<(modul::Revizuire, modul::Editari), String> {
    let sesiuni =
        gmotion_platforma::Sesiuni::pentru_modul(modul::NUME).map_err(|e| e.to_string())?;
    let id = args.identitate()?;
    Ok(sesiuni
        .incarca(&id, referinta)
        .map_err(|e| e.to_string())?
        .map(|s| {
            (
                modul::Revizuire::din_sesiune(&s),
                modul::Editari::din_sesiune(&s),
            )
        })
        .unwrap_or_default())
}

/// Scrie planul de deploy, cu amprentele de acum, ca sa fie verificat de
/// `deploy --plan` inainte de a scrie ceva.
pub fn plan(
    args: &ArgumenteSursa,
    referinta: &str,
    baze_cerute: Option<&str>,
    doar: Option<&str>,
    iesire: Option<PathBuf>,
    format: Format,
) -> Result<(), String> {
    let r = ruleaza_comparare(args, referinta, baze_cerute, format)?;
    let p = plan_din(args, &r, doar)?;
    if p.pasi.is_empty() {
        return Err(modul::Eroare::NimicDeDeployat.to_string());
    }
    let cale = iesire.unwrap_or_else(|| PathBuf::from(format!("plan_{}.json", acum_compact())));
    let text = serde_json::to_string_pretty(&p).map_err(|e| e.to_string())?;
    std::fs::write(&cale, text).map_err(|e| format!("nu pot scrie {}: {e}", cale.display()))?;
    format.progres(&format!(
        "Scris {} ({} pasi{}).\n\
         Deploy-ul, cu verificarea ca baza n-a mai fost atinsa:\n  \
         gmotion deploy <SURSA> --referinta {referinta} --plan {}",
        cale.display(),
        p.pasi.len(),
        if p.sarite > 0 {
            format!(", {} sarite", p.sarite)
        } else {
            String::new()
        },
        cale.display()
    ));
    if format == Format::Json {
        return afisare::scrie_json(&p).map_err(|e| e.to_string());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn deploy(
    args: &ArgumenteSursa,
    referinta: &str,
    baze_cerute: Option<&str>,
    director: Option<PathBuf>,
    mod_impartire: modul::ModImpartire,
    doar: Option<&str>,
    cale_plan: Option<&Path>,
    format: Format,
) -> Result<(), String> {
    // Cu `--plan`: bazele sunt cele din plan, iar ce se citeste acum e
    // verificat contra amprentelor de atunci. Fara: planul se face acum, din
    // ce tocmai s-a citit, deci garda e trivial adevarata.
    let (plan, r) = match cale_plan {
        Some(c) => {
            let text = std::fs::read_to_string(c)
                .map_err(|e| format!("nu pot citi planul {}: {e}", c.display()))?;
            let plan: modul::Plan = serde_json::from_str(&text)
                .map_err(|e| format!("planul {} nu e valid: {e}", c.display()))?;
            if plan.referinta != referinta {
                return Err(format!(
                    "planul e facut cu martorul {:?}, nu {referinta:?}",
                    plan.referinta
                ));
            }
            let mut baze: Vec<&str> = plan.pasi.iter().map(|p| p.baza.as_str()).collect();
            baze.sort_unstable();
            baze.dedup();
            if baze.is_empty() {
                return Err(modul::Eroare::NimicDeDeployat.to_string());
            }
            let r = ruleaza_comparare(args, referinta, Some(&baze.join(",")), format)?;
            plan.verifica_sau_opreste(&r.referinta, &r.altele)
                .map_err(|e| e.to_string())?;
            format.progres(&format!(
                "Baza modificata n-a mai fost atinsa: cele {} obiecte din plan (facut la {}) sunt exact cum erau.",
                plan.pasi.len(),
                plan.facut_la
            ));
            (plan, r)
        }
        None => {
            let r = ruleaza_comparare(args, referinta, baze_cerute, format)?;
            (plan_din(args, &r, doar)?, r)
        }
    };

    // textele finale facute in editor (sau cu `gmotion editare`), daca sunt
    // inca valabile pentru obiectele de acum
    let texte_finale = salvate(args, referinta)?
        .1
        .pentru_deploy(&r.comparare.randuri);
    // cu `--plan`, textele finale trebuie sa fie cele de la plan: un text
    // schimbat dupa plan ar pleca in baza fara sa-l fi vazut cine a aprobat
    // planul (F-D5)
    if cale_plan.is_some() {
        let s = plan.verifica_texte(&texte_finale);
        if !s.is_empty() {
            return Err(modul::Eroare::BazaAtinsa {
                schimbari: s
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; "),
            }
            .to_string());
        }
    }
    let cu_text_final = plan
        .pasi
        .iter()
        .filter(|p| texte_finale.contains_key(&(p.baza.clone(), p.obiect.clone())))
        .count();
    if cu_text_final > 0 {
        format.progres(&format!(
            "{cu_text_final} obiecte se deployeaza cu textul final din editor, nu cu codul din martor."
        ));
    }
    let pe_baza = plan
        .de_deployat_cu(&r.referinta, &texte_finale)
        .map_err(|e| e.to_string())?;
    let revenire = plan.revenire(&r.referinta, &r.altele);
    let sarite = plan.sarite;

    let director = director.unwrap_or_else(|| {
        PathBuf::from(format!(
            "deploy_{}_{}",
            referinta.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
            acum_compact()
        ))
    });

    let raport = modul::deploy::scrie_cu_revenire(
        &director,
        referinta,
        &pe_baza,
        &revenire,
        mod_impartire,
        sarite,
        &acum(),
    )
    .map_err(|e| e.to_string())?;

    marcheaza_deployatele(args, referinta, &plan, &r.comparare.randuri);

    format.progres(&format!(
        "Scris {} fisiere in {} ({} obiecte{}).\n\
         Revenirea, salvata inainte: {} fisiere in {}.",
        raport.fisiere.len(),
        raport.director.display(),
        raport.obiecte,
        if sarite > 0 {
            format!(", {sarite} sarite - exista doar in baza modificata sau sunt criptate")
        } else {
            String::new()
        },
        raport.revenire.len(),
        raport.director.join("revenire").display(),
    ));
    if format == Format::Json {
        #[derive(serde::Serialize)]
        struct R {
            director: String,
            fisiere: Vec<String>,
            revenire: Vec<String>,
            obiecte: usize,
            sarite: usize,
        }
        let siruri = |v: &[PathBuf]| v.iter().map(|f| f.display().to_string()).collect();
        return afisare::scrie_json(&R {
            director: raport.director.display().to_string(),
            fisiere: siruri(&raport.fisiere),
            revenire: siruri(&raport.revenire),
            obiecte: raport.obiecte,
            sarite: raport.sarite,
        })
        .map_err(|e| e.to_string());
    }
    Ok(())
}

pub fn diferente(
    args: &ArgumenteSursa,
    referinta: &str,
    baza: &str,
    obiect: &str,
    format: Format,
) -> Result<(), String> {
    let mut s = args.deschide()?;
    let ref_ob = s.obiecte(referinta).map_err(|e| e.to_string())?;
    let cur_ob = s.obiecte(baza).map_err(|e| e.to_string())?;
    let gaseste = |v: &[gmotion_db::Obiect]| v.iter().find(|o| o.id() == obiect).cloned();

    let (Some(r), Some(c)) = (gaseste(&ref_ob), gaseste(&cur_ob)) else {
        return Err(format!(
            "obiectul {obiect:?} nu exista in ambele baze ({referinta} si {baza})"
        ));
    };
    let text_ref = gmotion_core::pregateste_afisare(&r.definitie);
    let text_tinta = gmotion_core::pregateste_afisare(&c.definitie);
    let (d, rezumat) = modul::diferente_obiect(&text_ref, &text_tinta);
    let plasari = modul::plasari_obiect(&text_ref, &text_tinta);

    match format {
        Format::Json => {
            #[derive(serde::Serialize)]
            struct D<'a> {
                obiect: &'a str,
                referinta: &'a str,
                baza: &'a str,
                rezumat: &'a str,
                randuri: &'a [modul::Diferenta],
                plasare: &'a [modul::Plasare],
            }
            afisare::scrie_json(&D {
                obiect,
                referinta,
                baza,
                rezumat: &rezumat,
                randuri: &d,
                plasare: &plasari,
            })
            .map_err(|e| e.to_string())
        }
        Format::Text => {
            use std::io::IsTerminal;
            let culori = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
            let aliniere = gmotion_core::aliniaza(&text_ref, &text_tinta);
            let (linii, locuri) = text_diferente(&aliniere, culori);
            println!(
                "{}",
                afisare::fara_control(&format!("{obiect}: {referinta} -> {baza}"))
            );
            println!("{rezumat}");
            if !locuri.is_empty() {
                let lista: Vec<&str> = locuri.iter().take(20).map(String::as_str).collect();
                let rest = locuri.len().saturating_sub(20);
                println!(
                    "Diferente la liniile (martor/modificata): {}{}",
                    lista.join(", "),
                    if rest > 0 {
                        format!(" si inca {rest}")
                    } else {
                        String::new()
                    }
                );
            }
            if !plasari.is_empty() {
                println!();
                println!("Plasarea blocurilor din martor, adica ce face Aplica:");
                for p in &plasari {
                    let unde = if p.inlocuieste {
                        format!(
                            "inlocuieste blocul {} (linia {}), potrivire {:.2}%",
                            p.bloc_tinta + 1,
                            p.linie_tinta,
                            p.scor
                        )
                    } else if p.linie_tinta > 0 {
                        format!(
                            "fara corespondent: se insereaza inaintea liniei {}",
                            p.linie_tinta
                        )
                    } else {
                        "fara corespondent: se adauga la sfarsit".to_string()
                    };
                    let impreuna = if p.randuri_mutate.len() > 1 {
                        format!(" (cu {} blocuri)", p.randuri_mutate.len())
                    } else {
                        String::new()
                    };
                    println!("  linia {:>5} -> {unde}{impreuna}", p.linie_martor);
                }
            }
            println!();
            for linie in linii {
                println!("{linie}");
            }
            Ok(())
        }
    }
}

/// Textul lui `gmotion diferente`: tot obiectul, ca in editor, cu numerele de
/// linie din martor si din versiunea modificata, plus locurile diferentelor
/// (pentru rezumatul de sus).
///
/// Asezarea pe linii vine din `gmotion_core::aliniaza`, nu se recalculeaza
/// aici: pana la etapa 20, functia asta isi facea propria socoteala cu
/// `linii_start_blocuri` si umplea singura golurile dintre blocuri — a doua
/// implementare a aceleiasi logici, adica exact felul in care au divergat
/// motoarele din v3.4. Acum liniile din interiorul unui bloc modificat sunt
/// imperecheate intre ele, nu pe pozitie.
///
/// Un rand modificat se arata cu **ambele** versiuni: `-` martorul, `+`
/// versiunea modificata. Bucatile schimbate se evidentiaza: pe terminal cu
/// culoare inversata, redirectat in fisier cu `[-...-]` / `{+...+}`, ca
/// `git diff --word-diff`.
fn text_diferente(a: &gmotion_core::Aliniere, culori: bool) -> (Vec<String>, Vec<String>) {
    const ROSU: &str = "\x1b[31m";
    const VERDE: &str = "\x1b[32m";
    const INVERS: &str = "\x1b[7m";
    const FARA_INVERS: &str = "\x1b[27m";
    const NORMAL: &str = "\x1b[0m";
    // Caracterele de control din cod (venit din baze) nu ajung pe terminal;
    // culorile se adauga abia dupa curatare.
    let curat = |s: &str| afisare::fara_control(s);

    // marcajele pe linii, calculate o data per bloc si parte
    /// Marcajele unui bloc, pe linii: `[stanga, dreapta]`, calculate la cerere.
    type MarcajeBloc = [Option<Vec<Vec<(usize, usize)>>>; 2];
    let mut marcaje: Vec<MarcajeBloc> = vec![[None, None]; a.diff.len()];

    let evidentiat = |text: &str, m: &[(usize, usize)], semn: char| -> String {
        let (deschis, inchis) = match (culori, semn) {
            (true, _) => (INVERS, FARA_INVERS),
            (false, '-') => ("[-", "-]"),
            (false, _) => ("{+", "+}"),
        };
        if m.is_empty() {
            return curat(text);
        }
        let caractere: Vec<char> = text.chars().collect();
        let mut out = String::new();
        let mut poz = 0usize;
        for &(x, y) in m {
            let (x, y) = (x.min(caractere.len()), y.min(caractere.len()));
            if poz < x {
                out.push_str(&curat(&caractere[poz..x].iter().collect::<String>()));
            }
            out.push_str(deschis);
            out.push_str(&curat(&caractere[x..y].iter().collect::<String>()));
            out.push_str(inchis);
            poz = y;
        }
        if poz < caractere.len() {
            out.push_str(&curat(&caractere[poz..].iter().collect::<String>()));
        }
        out
    };

    let rand = |r: Option<usize>, t: Option<usize>, semn: char, text: &str| -> String {
        let nr = |n: Option<usize>| n.map_or("     ".to_string(), |n| format!("{n:>5}"));
        let culoare = match (culori, semn) {
            (true, '-') => ROSU,
            (true, '+') => VERDE,
            _ => "",
        };
        let sfarsit = if culoare.is_empty() { "" } else { NORMAL };
        format!("{} {} {culoare}{semn} {text}{sfarsit}", nr(r), nr(t))
    };

    // locurile diferentelor: prima linie a blocului, pe fiecare parte
    let mut locuri = Vec::new();
    for (b, d) in a.diff.iter().enumerate() {
        if d.tip == gmotion_core::TipDiferenta::Egal {
            continue;
        }
        let ale_lui = a.randuri.iter().filter(|r| r.bloc == Some(b));
        let (mut st, mut dr) = (None, None);
        for r in ale_lui {
            st = st.or(r.stanga.as_ref().map(|l| l.nr));
            dr = dr.or(r.dreapta.as_ref().map(|l| l.nr));
        }
        let text = |n: Option<usize>| n.map_or("-".to_string(), |n| n.to_string());
        locuri.push(format!("{}/{}", text(st), text(dr)));
    }

    let mut linii = Vec::new();
    for r in &a.randuri {
        let mut marcaj = |parte: usize, indice: Option<usize>| -> Vec<(usize, usize)> {
            // o linie ignorata la comparare (goala sau numai comentariu) n-are
            // ce sa aiba marcat: normalizarea oricum n-o vede, deci marcajul ar
            // spune „asta s-a schimbat" despre ceva ce motorul nu numara
            let (Some(b), Some(i)) = (r.bloc, indice.filter(|_| !r.ignorata)) else {
                return Vec::new();
            };
            let cache = &mut marcaje[b][parte];
            let m = cache.get_or_insert_with(|| {
                let seg = if parte == 0 {
                    a.diff[b].seg_stanga.as_deref()
                } else {
                    a.diff[b].seg_dreapta.as_deref()
                };
                seg.map(gmotion_core::diff::marcaje_pe_linii)
                    .unwrap_or_default()
            });
            m.get(i).cloned().unwrap_or_default()
        };

        let m_st = marcaj(0, r.linie_in_bloc_stanga);
        let m_dr = marcaj(1, r.linie_in_bloc_dreapta);
        // doua linii imperecheate fara nimic marcat sunt aceeasi linie: se
        // arata o data, ca si context, chiar daca blocul din jur e modificat
        let nimic_schimbat =
            r.stanga.is_some() && r.dreapta.is_some() && m_st.is_empty() && m_dr.is_empty();
        if r.tip == gmotion_core::TipDiferenta::Egal || nimic_schimbat {
            let text = r
                .dreapta
                .as_ref()
                .or(r.stanga.as_ref())
                .map(|l| l.text.as_str())
                .unwrap_or("");
            linii.push(rand(
                r.stanga.as_ref().map(|l| l.nr),
                r.dreapta.as_ref().map(|l| l.nr),
                ' ',
                &curat(text),
            ));
            continue;
        }
        if let Some(l) = r.stanga.as_ref() {
            linii.push(rand(
                Some(l.nr),
                None,
                '-',
                &evidentiat(&l.text, &m_st, '-'),
            ));
        }
        if let Some(l) = r.dreapta.as_ref() {
            linii.push(rand(
                None,
                Some(l.nr),
                '+',
                &evidentiat(&l.text, &m_dr, '+'),
            ));
        }
    }

    (linii, locuri)
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn randul_modificat_arata_ambele_versiuni_si_ce_s_a_schimbat() {
        let a = "SELECT 1\n\nSELECT dbo.ResStr('nu s-au gasit detalii') + x";
        let b = "SELECT 1\n\nEXEC p\n\nSELECT dbo.ResStr('Nu s-au gasit detalii') + x";
        let aliniere = gmotion_core::aliniaza(a, b);
        let (linii, locuri) = text_diferente(&aliniere, false);
        let vechi = linii
            .iter()
            .find(|l| l.contains("- SELECT dbo"))
            .expect("versiunea veche");
        let noua = linii
            .iter()
            .find(|l| l.contains("+ SELECT dbo"))
            .expect("versiunea noua");
        assert!(vechi.contains("[-nu-]"), "{vechi}");
        assert!(noua.contains("{+Nu+}"), "{noua}");
        // numerele de linie: referinta are 3 linii, tinta 5
        assert!(vechi.trim_start().starts_with('3'), "{vechi}");
        assert!(noua.trim_start().starts_with('5'), "{noua}");
        assert!(linii.iter().any(|l| l.contains("+ EXEC p")));
        assert_eq!(locuri, ["-/3", "3/5"], "{linii:#?}");
        assert!(
            !linii.iter().any(|l| l.contains('\u{1b}')),
            "fara culori cand nu se cer"
        );
        // tot obiectul, inclusiv linia goala dintre blocuri, fiecare o data
        assert_eq!(linii.iter().filter(|l| l.contains("SELECT 1")).count(), 1);
        // linia goala 2 din referinta corespunde liniei goale 4 din tinta; linia
        // goala 2 din tinta e inaintea lui EXEC p, adaugat
        assert!(linii.iter().any(|l| l.trim() == "2     4"), "{linii:#?}");
        assert!(linii.iter().any(|l| l.trim() == "2"), "{linii:#?}");
        assert_eq!(linii.len(), 6, "{linii:#?}");
        let (cu_culori, _) = text_diferente(&aliniere, true);
        assert!(
            cu_culori.iter().any(|l| l.contains("\u{1b}[7mNu")),
            "{cu_culori:#?}"
        );
    }
}
