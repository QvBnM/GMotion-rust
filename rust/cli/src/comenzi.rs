//! Implementarea comenzilor.
//!
//! **Se strica astfel:** o comanda face altceva decat spune, sau iese cu codul
//! 0 cand a eșuat. Al doilea e cel rau: o rulare programata ar raporta succes.
//!
//! Fiecare comanda e o functie care intoarce `Result<(), String>`. Mesajul de
//! eroare pastreaza codul original (`GM-DB-001`, ...), ca raportul
//! utilizatorului sa duca direct la cauza.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use gmotion_db::Instantaneu;
use gmotion_modul_diff_obiecte as modul;
use gmotion_platforma::Configurare;

use crate::afisare::{self, Format};
use crate::sursa::{lista, ArgumenteSursa};

/// Marcajul de timp pentru nume de fisiere si antete. Secundele de la epoca,
/// fara dependinta de un crate de date: nu avem nevoie de calendar, doar de
/// ceva monoton si lizibil.
fn acum() -> String {
    let s = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Conversie minima la data: suficient pentru un antet de script.
    let zile = s / 86_400;
    let ora = (s % 86_400) / 3600;
    let minut = (s % 3600) / 60;
    let (an, luna, zi) = din_zile(zile);
    format!("{an:04}-{luna:02}-{zi:02} {ora:02}:{minut:02}")
}

/// Zile de la 1970-01-01 -> (an, luna, zi). Algoritmul civil de la Howard
/// Hinnant, forma pentru ere de 400 de ani.
fn din_zile(zile: u64) -> (i64, u32, u32) {
    let z = zile as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

/// Marcaj compact pentru nume de fisiere: `20261117_1030`.
fn acum_compact() -> String {
    acum().replace(['-', ':'], "").replace(' ', "_")
}

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
                    ]
                })
                .collect();
            print!(
                "{}",
                afisare::tabel(&["SERVER", "IP", "PORT", "VERSIUNE"], &randuri)
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
    ieșire: &Path,
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
    inst.scrie(ieșire).map_err(|e| e.to_string())?;

    format.progres(&format!(
        "Scris {} ({} baze, {total} obiecte).\n\
         Reia-l cu: gmotion compara --sursa {} --referinta <BAZA>",
        ieșire.display(),
        inst.baze.len(),
        ieșire.display()
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
            "baza de referinta {referinta:?} nu exista in sursa. Disponibile: {}",
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
                format.progres(&format!("Citesc referinta {baza}..."));
            } else {
                format.progres(&format!("[{i}/{n_altele}] citesc {baza}..."));
            }
        })
        .map_err(|e| e.to_string())?
        .into_iter();
    let ref_obiecte = match citite.next() {
        Some((_, obiecte)) => modul::pregateste(&obiecte),
        None => return Err("referinta nu a putut fi citita".into()),
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
        let setari = modul::setari_diferite(a.setari, b.setari);
        rez.insert(
            rand.id,
            modul::detalii_obiect(&a.afisare, &b.afisare, &setari),
        );
    }
    rez
}

pub fn export(
    args: &ArgumenteSursa,
    referinta: &str,
    baze_cerute: Option<&str>,
    ieșire: Option<PathBuf>,
    format: Format,
) -> Result<(), String> {
    let r = ruleaza_comparare(args, referinta, baze_cerute, format)?;
    if r.comparare.randuri.is_empty() {
        return Err("nimic de exportat: bazele sunt identice cu referinta".into());
    }
    let cale =
        ieșire.unwrap_or_else(|| PathBuf::from(format!("gmotion_export_{}.csv", acum_compact())));
    let n = modul::scrie_csv(&cale, &r.comparare.randuri, |_| modul::Stare::default())
        .map_err(|e| e.to_string())?;
    format.progres(&format!("Scris {} ({n} randuri).", cale.display()));
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
    format: Format,
) -> Result<(), String> {
    let r = ruleaza_comparare(args, referinta, baze_cerute, format)?;

    let filtru = doar.map(lista);
    let mut pe_baza: BTreeMap<String, Vec<modul::DeDeployat>> = BTreeMap::new();
    let mut sarite = 0usize;

    for rand in &r.comparare.randuri {
        if let Some(f) = &filtru {
            if !f.iter().any(|x| x == &rand.obiect) {
                continue;
            }
        }
        if rand.categorie == modul::Categorie::Extra {
            // exista doar in baza comparata: nu exista cod in referinta de adus
            sarite += 1;
            continue;
        }
        if rand.categorie == modul::Categorie::Criptat {
            // criptat intr-una din parti: nu exista cod de citit, si un obiect
            // criptat al furnizorului nu se suprascrie
            sarite += 1;
            continue;
        }
        let Some(o) = r.referinta.get(&rand.obiect) else {
            // nu se poate intampla: randurile non-extra vin din referinta
            continue;
        };
        if o.definitie.is_empty() {
            sarite += 1; // criptat in referinta, lipsa in tinta
            continue;
        }
        pe_baza
            .entry(rand.baza.clone())
            .or_default()
            .push(modul::DeDeployat {
                obiect: rand.obiect.clone(),
                tip: rand.tip.clone(),
                cod: modul::aplica_create_or_alter(&o.definitie),
                setari: o.setari,
            });
    }

    let director = director.unwrap_or_else(|| {
        PathBuf::from(format!(
            "deploy_{}_{}",
            referinta.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
            acum_compact()
        ))
    });

    let raport = modul::deploy::scrie(
        &director,
        referinta,
        &pe_baza,
        mod_impartire,
        sarite,
        &acum(),
    )
    .map_err(|e| e.to_string())?;

    format.progres(&format!(
        "Scris {} fisiere in {} ({} obiecte{}).",
        raport.fisiere.len(),
        raport.director.display(),
        raport.obiecte,
        if sarite > 0 {
            format!(", {sarite} sarite - exista doar in baza comparata")
        } else {
            String::new()
        }
    ));
    if format == Format::Json {
        #[derive(serde::Serialize)]
        struct R {
            director: String,
            fisiere: Vec<String>,
            obiecte: usize,
            sarite: usize,
        }
        return afisare::scrie_json(&R {
            director: raport.director.display().to_string(),
            fisiere: raport
                .fisiere
                .iter()
                .map(|f| f.display().to_string())
                .collect(),
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

    match format {
        Format::Json => {
            #[derive(serde::Serialize)]
            struct D<'a> {
                obiect: &'a str,
                referinta: &'a str,
                baza: &'a str,
                rezumat: &'a str,
                randuri: &'a [modul::Diferenta],
            }
            afisare::scrie_json(&D {
                obiect,
                referinta,
                baza,
                rezumat: &rezumat,
                randuri: &d,
            })
            .map_err(|e| e.to_string())
        }
        Format::Text => {
            use std::io::IsTerminal;
            let culori = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
            let (linii, locuri) = text_diferente(&d, &text_ref, &text_tinta, culori);
            println!(
                "{}",
                afisare::fara_control(&format!("{obiect}: {referinta} -> {baza}"))
            );
            println!("{rezumat}");
            if !locuri.is_empty() {
                let lista: Vec<&str> = locuri.iter().take(20).map(String::as_str).collect();
                let rest = locuri.len().saturating_sub(20);
                println!(
                    "Diferente la liniile (referinta/tinta): {}{}",
                    lista.join(", "),
                    if rest > 0 {
                        format!(" si inca {rest}")
                    } else {
                        String::new()
                    }
                );
            }
            println!();
            for linie in linii {
                println!("{linie}");
            }
            Ok(())
        }
    }
}

/// Textul lui `gmotion diferente`: tot obiectul, ca in editorul din v3.4, cu
/// numerele de linie din referinta si din tinta, plus locurile diferentelor
/// (pentru rezumatul de sus).
///
/// Un bloc modificat se arata cu **ambele** versiuni: `-` referinta, `+` baza
/// comparata. Bucatile schimbate din linie se evidentiaza: pe terminal cu
/// culoare inversata (rosu / verde), redirectat in fisier cu `[-...-]` /
/// `{+...+}`, ca `git diff --word-diff`. Pana la a doua rulare pe Windows se
/// arata doar versiunea noua cu `~`, deci nu se vedea ce s-a schimbat.
fn text_diferente(
    d: &[modul::Diferenta],
    text_ref: &str,
    text_tinta: &str,
    culori: bool,
) -> (Vec<String>, Vec<String>) {
    // Linia reala de start a fiecarui bloc: segmentarea arunca liniile goale si
    // blocurile care sunt doar comentariu, deci numararea pe blocuri ar da
    // numere gresite si ar ascunde liniile acelea. Golurile se completeaza din
    // textul original, ca context.
    let start_ref = gmotion_core::linii_start_blocuri(
        text_ref,
        &gmotion_core::split_statements(text_ref, false),
    );
    let start_tinta = gmotion_core::linii_start_blocuri(
        text_tinta,
        &gmotion_core::split_statements(text_tinta, false),
    );
    let original_ref: Vec<&str> = text_ref.lines().collect();
    let original_tinta: Vec<&str> = text_tinta.lines().collect();
    const ROSU: &str = "\x1b[31m";
    const VERDE: &str = "\x1b[32m";
    const INVERS: &str = "\x1b[7m";
    const FARA_INVERS: &str = "\x1b[27m";
    const NORMAL: &str = "\x1b[0m";
    // Caracterele de control din cod (venit din baze) nu ajung pe terminal;
    // culorile se adauga abia dupa curatare.
    let curat = |s: &str| afisare::fara_control(s);
    let evidentiat =
        |bucati: &Option<Vec<modul::Bucata>>, simplu: &Option<String>, semn: char| -> String {
            let (deschis, inchis) = match (culori, semn) {
                (true, _) => (INVERS, FARA_INVERS),
                (false, '-') => ("[-", "-]"),
                (false, _) => ("{+", "+}"),
            };
            match bucati {
                Some(b) => b
                    .iter()
                    .map(|x| {
                        let text = curat(&x.text);
                        if x.marcaj.is_some() {
                            // marcajul se inchide la capat de linie si se redeschide
                            // pe urmatoarea, ca fiecare linie sa fie completa
                            text.split('\n')
                                .map(|bucata| format!("{deschis}{bucata}{inchis}"))
                                .collect::<Vec<_>>()
                                .join("\n")
                        } else {
                            text
                        }
                    })
                    .collect(),
                None => curat(simplu.as_deref().unwrap_or("")),
            }
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

    // Numarul de linie (1-based) al liniei `k` din blocul `b`; daca blocul nu
    // are start cunoscut, continua dupa ultima linie scrisa.
    let mut bloc_ref = 0usize;
    let mut bloc_tinta = 0usize;
    let (mut ultim_ref, mut ultim_tinta) = (0usize, 0usize);
    let start = |starturi: &[usize], bloc: &mut usize, ultim: usize| -> usize {
        let s = starturi.get(*bloc).map_or(ultim, |&s| s.max(ultim));
        *bloc += 1;
        s
    };
    // Liniile originale dintre doua blocuri (goale sau doar comentariu), ca
    // context: intervale 0-based, capatul din dreapta exclusiv.
    let completeaza = |linii: &mut Vec<String>, r: (usize, usize), t: (usize, usize)| {
        let nr = r.1.saturating_sub(r.0).max(t.1.saturating_sub(t.0));
        for i in 0..nr {
            let lr = (r.0 + i < r.1).then_some(r.0 + i);
            let lt = (t.0 + i < t.1).then_some(t.0 + i);
            let text = lt
                .and_then(|k| original_tinta.get(k))
                .or_else(|| lr.and_then(|k| original_ref.get(k)))
                .copied()
                .unwrap_or("");
            linii.push(rand(
                lr.map(|k| k + 1),
                lt.map(|k| k + 1),
                ' ',
                &curat(text),
            ));
        }
    };
    let mut linii = Vec::new();
    let mut locuri = Vec::new();
    for x in d {
        let are_ref = x.stanga.is_some();
        let are_tinta = x.dreapta.is_some();
        let s_ref = if are_ref {
            start(&start_ref, &mut bloc_ref, ultim_ref)
        } else {
            ultim_ref
        };
        let s_tinta = if are_tinta {
            start(&start_tinta, &mut bloc_tinta, ultim_tinta)
        } else {
            ultim_tinta
        };
        completeaza(&mut linii, (ultim_ref, s_ref), (ultim_tinta, s_tinta));
        match x.fel {
            modul::Fel::Egal => {
                let s: Vec<String> = x
                    .stanga
                    .as_deref()
                    .unwrap_or("")
                    .lines()
                    .map(curat)
                    .collect();
                let dr: Vec<String> = x
                    .dreapta
                    .as_deref()
                    .unwrap_or("")
                    .lines()
                    .map(curat)
                    .collect();
                for i in 0..s.len().max(dr.len()) {
                    let r = (i < s.len()).then_some(s_ref + i + 1);
                    let t = (i < dr.len()).then_some(s_tinta + i + 1);
                    let text = dr.get(i).or(s.get(i)).map(String::as_str).unwrap_or("");
                    linii.push(rand(r, t, ' ', text));
                }
                ultim_ref = s_ref + s.len();
                ultim_tinta = s_tinta + dr.len();
            }
            modul::Fel::Sters => {
                locuri.push(format!("{}/-", s_ref + 1));
                let text = curat(x.stanga.as_deref().unwrap_or(""));
                for (i, l) in text.lines().enumerate() {
                    linii.push(rand(Some(s_ref + i + 1), None, '-', l));
                }
                ultim_ref = s_ref + text.lines().count();
            }
            modul::Fel::Adaugat => {
                locuri.push(format!("-/{}", s_tinta + 1));
                let text = curat(x.dreapta.as_deref().unwrap_or(""));
                for (i, l) in text.lines().enumerate() {
                    linii.push(rand(None, Some(s_tinta + i + 1), '+', l));
                }
                ultim_tinta = s_tinta + text.lines().count();
            }
            modul::Fel::Modificat => {
                locuri.push(format!("{}/{}", s_ref + 1, s_tinta + 1));
                let vechi = evidentiat(&x.bucati_stanga, &x.stanga, '-');
                for (i, l) in vechi.lines().enumerate() {
                    linii.push(rand(Some(s_ref + i + 1), None, '-', l));
                }
                let nou = evidentiat(&x.bucati_dreapta, &x.dreapta, '+');
                for (i, l) in nou.lines().enumerate() {
                    linii.push(rand(None, Some(s_tinta + i + 1), '+', l));
                }
                ultim_ref = s_ref + vechi.lines().count();
                ultim_tinta = s_tinta + nou.lines().count();
            }
        }
    }
    completeaza(
        &mut linii,
        (ultim_ref, original_ref.len()),
        (ultim_tinta, original_tinta.len()),
    );
    (linii, locuri)
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn randul_modificat_arata_ambele_versiuni_si_ce_s_a_schimbat() {
        let a = "SELECT 1\n\nSELECT dbo.ResStr('nu s-au gasit detalii') + x";
        let b = "SELECT 1\n\nEXEC p\n\nSELECT dbo.ResStr('Nu s-au gasit detalii') + x";
        let (d, _) = modul::diferente_obiect(a, b);
        let (linii, locuri) = text_diferente(&d, a, b, false);
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
        let (cu_culori, _) = text_diferente(&d, a, b, true);
        assert!(
            cu_culori.iter().any(|l| l.contains("\u{1b}[7mNu")),
            "{cu_culori:#?}"
        );
    }

    #[test]
    fn conversia_de_zile_e_corecta_pe_repere_cunoscute() {
        assert_eq!(din_zile(0), (1970, 1, 1));
        assert_eq!(din_zile(1), (1970, 1, 2));
        assert_eq!(din_zile(365), (1971, 1, 1));
        // 2000 a fost an bisect (divizibil cu 400)
        assert_eq!(din_zile(10_957), (2000, 1, 1));
        assert_eq!(din_zile(11_016), (2000, 2, 29));
        assert_eq!(din_zile(20_819), (2027, 1, 1));
    }

    #[test]
    fn marcajul_de_timp_are_forma_asteptata() {
        let a = acum();
        assert_eq!(a.len(), 16, "{a}");
        assert_eq!(a.as_bytes()[4], b'-');
        assert_eq!(a.as_bytes()[10], b' ');
        assert_eq!(a.as_bytes()[13], b':');
        let c = acum_compact();
        assert_eq!(c.len(), 13, "{c}");
        assert!(c.contains('_'));
        assert!(!c.contains('-') && !c.contains(':') && !c.contains(' '));
    }
}
