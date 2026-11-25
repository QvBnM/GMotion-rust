//! Cat de **bune** sunt diferentele, nu doar cat de stabile.
//!
//! Restul testelor din `core` apara stabilitatea: conformitatea spune ca
//! rezultatul nu s-a schimbat fata de motorul de referinta, invariantele ca nu
//! se pierde cod, garda de cost ca nu devine lent. Niciunul nu spune ca
//! imperecherea e **corecta** — si fara un numar, pragurile mostenite din v3.4
//! (44, 40, 55, 8, 40, 70, 80) nu se pot atinge: n-ai cum sa arati ca o
//! schimbare e o imbunatatire.
//!
//! Aici se ia fiecare obiect real din `referinta/instantanee/`, se aplica o
//! mutatie **cunoscuta**, si se cere motorului sa raporteze exact mutatia. Rata
//! de recuperare are un plafon de jos: daca scade, testul cade si spune pe ce
//! obiecte.
//!
//! **Se strica astfel:** rata scade dupa o schimbare de prag sau de
//! segmentare, si nimeni nu observa pana cand cineva se uita la un diff si nu
//! recunoaste ce a facut.

use std::collections::BTreeMap;
use std::path::PathBuf;

use gmotion_core::{
    amprenta_comentarii, diff_aliniat, masoara, normalizeaza, pregateste_afisare, segmenteaza, Bloc,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Instantaneu {
    baze: BTreeMap<String, Vec<ObiectJson>>,
}

#[derive(Debug, Deserialize)]
struct ObiectJson {
    schema: String,
    nume: String,
    definitie: String,
}

fn obiecte() -> Vec<(String, String)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("rust/core are mereu doi stramosi")
        .join("referinta/instantanee");
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for fisier in ["testsimilitudine.json", "previzualizare.json"] {
        let cale = dir.join(fisier);
        let text = std::fs::read_to_string(&cale)
            .unwrap_or_else(|e| panic!("nu pot citi {}: {e}", cale.display()));
        let inst: Instantaneu = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("JSON invalid in {}: {e}", cale.display()));
        for (baza, obiecte) in inst.baze {
            for o in obiecte {
                let cheie = format!("{baza}/{}.{}", o.schema, o.nume);
                let afisare = pregateste_afisare(&o.definitie);
                if segmenteaza(&afisare, false).blocuri.len() >= 4 {
                    out.insert(cheie, afisare);
                }
            }
        }
    }
    out.into_iter().collect()
}

/// Un bloc pe care se poate umbla singur: nu e antetul, e contiguu in text, si
/// nu duce `BEGIN` sau `END` cu el (altfel mutatia strica structura si n-ar mai
/// fi o mutatie, ar fi alt obiect).
fn candidati(blocuri: &[Bloc]) -> Vec<usize> {
    blocuri
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, b)| {
            let n = normalizeaza(&b.text);
            let cuvant = |w: &str| {
                n.split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|t| t == w)
            };
            !cuvant("begin")
                && !cuvant("end")
                && !cuvant("else")
                && !cuvant("union")
                && !cuvant("cursor")
                && b.linii.windows(2).all(|w| w[1] == w[0] + 1)
        })
        .map(|(i, _)| i)
        .collect()
}

/// Liniile obiectului, fara liniile blocului `k` (si fara golul de dupa el).
fn fara_bloc(linii: &[&str], b: &Bloc) -> Vec<String> {
    let ultima = *b.linii.last().expect("bloc fara linii");
    let mut sari: Vec<usize> = b.linii.clone();
    let mut i = ultima + 1;
    while i < linii.len() && linii[i].trim().is_empty() {
        sari.push(i);
        i += 1;
    }
    linii
        .iter()
        .enumerate()
        .filter(|(i, _)| !sari.contains(i))
        .map(|(_, l)| (*l).to_string())
        .collect()
}

/// Liniile blocurilor `a` si `b` schimbate intre ele (amandoua contigue).
fn inversate(linii: &[&str], a: &Bloc, b: &Bloc) -> Vec<String> {
    let (ra, rb) = (
        a.linie()..=*a.linii.last().unwrap(),
        b.linie()..=*b.linii.last().unwrap(),
    );
    let mut out: Vec<String> = Vec::with_capacity(linii.len());
    let mut i = 0usize;
    while i < linii.len() {
        if i == *ra.start() {
            out.extend(rb.clone().map(|k| linii[k].to_string()));
            i = *ra.end() + 1;
        } else if i == *rb.start() {
            out.extend(ra.clone().map(|k| linii[k].to_string()));
            i = *rb.end() + 1;
        } else {
            out.push(linii[i].to_string());
            i += 1;
        }
    }
    out
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Numar {
    modificate: usize,
    adaugate: usize,
    sterse: usize,
    mutate: usize,
}

fn numara(referinta: &str, tinta: &str) -> Numar {
    let m = masoara(&diff_aliniat(referinta, tinta));
    Numar {
        modificate: m.modificate,
        adaugate: m.adaugate,
        sterse: m.sterse,
        mutate: m.mutate,
    }
}

/// Plafonul de jos al ratei de recuperare, in procente, pe fiecare mutatie.
///
/// Nu sunt 100 si nu se pretinde ca ar trebui: segmentarea e euristica, si pe
/// cateva obiecte reale o stergere muta granita blocului vecin, deci motorul
/// raporteaza doua diferente in loc de una. Rostul plafonului e sa prinda
/// **scaderea**. Valorile masurate la deschiderea etapei 30 sunt in
/// `etape/30-motor-de-comparare.md`.
const PLAFOANE: &[(&str, f64)] = &[
    ("bloc_sters", 90.0),
    ("bloc_adaugat", 90.0),
    ("blocuri_inversate", 85.0),
    ("literal_schimbat", 90.0),
    ("reindentare", 100.0),
    ("comentariu_adaugat", 100.0),
];

/// Invarianta 4e pe obiectele **reale**: fiecare bloc e exact liniile lui din
/// text. Pe fuzz o verifica `invariante.rs`; aici conteaza ca o verifica si cele
/// 109 obiecte din instantanee, intre care cele 8 pe care cautarea de dinainte
/// le asezau pe linia gresita (B-1).
#[test]
fn fiecare_bloc_sta_pe_liniile_lui() {
    let obiecte = obiecte();
    let mut blocuri = 0usize;
    for (nume, text) in &obiecte {
        let linii: Vec<&str> = text.split('\n').collect();
        for uneste_cursor in [false, true] {
            for (i, b) in segmenteaza(text, uneste_cursor).blocuri.iter().enumerate() {
                let din_linii: Vec<&str> = b.linii.iter().map(|&k| linii[k]).collect();
                assert_eq!(
                    b.text,
                    din_linii.join("\n"),
                    "{nume}, uneste_cursor={uneste_cursor}, blocul {i}: liniile {:?}",
                    b.linii
                );
                blocuri += 1;
            }
        }
    }
    assert!(blocuri > 1000, "prea putine blocuri verificate: {blocuri}");
}

#[test]
fn motorul_recupereaza_mutatiile_cunoscute() {
    let obiecte = obiecte();
    assert!(
        obiecte.len() >= 20,
        "prea putine obiecte reale: {}",
        obiecte.len()
    );

    let mut reusite: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let mut rateuri: Vec<String> = Vec::new();
    let mut socoteste = |mutatie: &'static str, nume: &str, ok: bool, detaliu: String| {
        let e = reusite.entry(mutatie).or_insert((0, 0));
        e.1 += 1;
        if ok {
            e.0 += 1;
        } else {
            rateuri.push(format!("  {mutatie} / {nume}: {detaliu}"));
        }
    };

    for (nume, text) in &obiecte {
        let linii: Vec<&str> = text.split('\n').collect();
        let blocuri = segmenteaza(text, false).blocuri;
        let cand = candidati(&blocuri);
        if cand.is_empty() {
            continue;
        }
        let k = cand[cand.len() / 2];

        // 1. un bloc sters din tinta: exact „1 sterse"
        let tinta = fara_bloc(&linii, &blocuri[k]).join("\n");
        let n = numara(text, &tinta);
        socoteste(
            "bloc_sters",
            nume,
            n == Numar {
                sterse: 1,
                ..Default::default()
            },
            format!("{n:?}"),
        );

        // 2. un bloc nou in tinta: exact „1 adaugate"
        let mut noi: Vec<String> = linii.iter().map(|l| (*l).to_string()).collect();
        let loc = blocuri[k].linie();
        noi.insert(loc, String::new());
        noi.insert(loc, "SELECT 1 AS gmotion_proba_de_calitate;".to_string());
        let n = numara(text, &noi.join("\n"));
        socoteste(
            "bloc_adaugat",
            nume,
            n == Numar {
                adaugate: 1,
                ..Default::default()
            },
            format!("{n:?}"),
        );

        // 3. doua blocuri inversate: numai mutari, niciun „modificat",
        // „sters" sau „adaugat". Pot fi raportate 1 sau 2 mutari, dupa care
        // bloc ramane ancorat de `SequenceMatcher` — amandoua descriu corect ce
        // s-a intamplat, deci nu se cere un numar anume.
        if cand.len() >= 2 {
            let (a, b) = (cand[0], cand[cand.len() - 1]);
            if a != b {
                let tinta = inversate(&linii, &blocuri[a], &blocuri[b]).join("\n");
                let n = numara(text, &tinta);
                socoteste(
                    "blocuri_inversate",
                    nume,
                    n.mutate >= 1 && n.modificate == 0 && n.sterse == 0 && n.adaugate == 0,
                    format!("{n:?}"),
                );
            }
        }

        // 4. un literal schimbat intr-un bloc: exact „1 modificate"
        if let Some(poz) = blocuri[k].text.find(" = 0") {
            let mut t = blocuri[k].text.clone();
            t.replace_range(poz..poz + 4, " = 4242");
            let tinta = text.replacen(&blocuri[k].text, &t, 1);
            if tinta != *text {
                let n = numara(text, &tinta);
                socoteste(
                    "literal_schimbat",
                    nume,
                    n == Numar {
                        modificate: 1,
                        ..Default::default()
                    },
                    format!("{n:?}"),
                );
            }
        }

        // 5. reindentare: normalizarea o inghite, deci ZERO diferente
        let reindentat: Vec<String> = linii
            .iter()
            .map(|l| {
                if l.trim().is_empty() {
                    (*l).to_string()
                } else {
                    format!("    {l}")
                }
            })
            .collect();
        let n = numara(text, &reindentat.join("\n"));
        socoteste("reindentare", nume, n == Numar::default(), format!("{n:?}"));

        // 6. un comentariu nou: zero diferente de cod, dar amprenta se schimba
        let mut cu_comentariu: Vec<String> = linii.iter().map(|l| (*l).to_string()).collect();
        cu_comentariu.insert(blocuri[k].linie(), "-- proba de calitate".to_string());
        let tinta = cu_comentariu.join("\n");
        let n = numara(text, &tinta);
        let amprenta_schimbata = amprenta_comentarii(text) != amprenta_comentarii(&tinta);
        socoteste(
            "comentariu_adaugat",
            nume,
            n == Numar::default() && amprenta_schimbata,
            format!("{n:?}, amprenta schimbata: {amprenta_schimbata}"),
        );
    }

    let mut probleme: Vec<String> = Vec::new();
    let mut raport: Vec<String> = Vec::new();
    for (mutatie, plafon) in PLAFOANE {
        let (ok, tot) = reusite.get(mutatie).copied().unwrap_or((0, 0));
        assert!(tot > 0, "mutatia {mutatie} n-a fost incercata niciodata");
        let rata = 100.0 * ok as f64 / tot as f64;
        raport.push(format!("{mutatie}: {rata:.1}% ({ok}/{tot})"));
        if rata < *plafon {
            probleme.push(format!(
                "{mutatie}: rata {rata:.1}% ({ok}/{tot}) sub plafonul de {plafon:.0}%"
            ));
        }
    }
    println!("rate de recuperare: {}", raport.join(", "));
    if !probleme.is_empty() {
        panic!(
            "{}\n\nrateuri:\n{}",
            probleme.join("\n"),
            rateuri.join("\n")
        );
    }
}
