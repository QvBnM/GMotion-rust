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
    })
}

pub fn compara(
    args: &ArgumenteSursa,
    referinta: &str,
    baze_cerute: Option<&str>,
    format: Format,
) -> Result<(), String> {
    let r = ruleaza_comparare(args, referinta, baze_cerute, format)?;
    match format {
        Format::Json => {
            afisare::scrie_json(&afisare::comparare_json(&r.comparare)).map_err(|e| e.to_string())
        }
        Format::Text => {
            print!("{}", afisare::comparare_text(&r.comparare));
            Ok(())
        }
    }
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
    let (d, rezumat) = modul::diferente_obiect(
        &gmotion_core::pregateste_afisare(&r.definitie),
        &gmotion_core::pregateste_afisare(&c.definitie),
    );

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
            println!("{obiect}: {referinta} -> {baza}");
            println!("{rezumat}\n");
            for rand in &d {
                let semn = match rand.fel {
                    modul::Fel::Egal => " ",
                    modul::Fel::Modificat => "~",
                    modul::Fel::Sters => "-",
                    modul::Fel::Adaugat => "+",
                };
                let text = rand
                    .dreapta
                    .as_deref()
                    .or(rand.stanga.as_deref())
                    .unwrap_or("");
                for linie in text.lines() {
                    println!("{semn} {linie}");
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod teste {
    use super::*;

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
