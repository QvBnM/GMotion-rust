//! Impartirea codului unui obiect SQL in blocuri logice.
//!
//! Aici se afla toata divergenta istorica dintre `mostenire/motor.py` si portul JS din
//! `mostenire/ui.py`: portul JS a primit urmarirea string-urilor, a comentariilor bloc, a
//! adancimii parantezelor si a stivei `CASE`/`BEGIN`/`END`, plus splitarea pe
//! `CREATE` si `DECLARE` - niciuna adusa inapoi in Python. Varianta de aici
//! urmeaza semantica JS, care e cea corecta.
//!
//! Vezi `referinta/README.md` pentru divergentele masurate.
//!
//! **Se strica astfel:** blocuri taiate in mijlocul unui literal, al unui
//! comentariu bloc, al unei paranteze sau al unui `CASE`; un `UNION` sau un
//! ciclu de cursor rupt in doua; numar de blocuri diferit de ce arata editorul.
//! E zona cu cel mai greu istoric: **toata** divergenta masurata intre
//! `mostenire/motor.py` si portul JS vine de aici, si de aici se propaga in
//! tipurile de diferenta. De-aia are si fuzz pe invariante, in
//! `tests/invariante.rs`.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::normalizare::norm;

/// Instructiunile pe care se taie un bloc nou. `CREATE` si `DECLARE` sunt
/// incluse - lipsa lor in `mostenire/motor.py` e una din cauzele divergentei.
static RE_INCEPUT_INSTRUCTIUNE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"(?i)^\s*(?:",
        r"SELECT\b|INSERT\b|UPDATE\b|DELETE\b|MERGE\b|SET\s+@|EXEC(?:UTE)?\b|",
        r"IF\b|ELSE\b|WHILE\b|RETURN\b|PRINT\b|RAISERROR\b|THROW\b|",
        r"OPEN\b|CLOSE\b|FETCH\b|DEALLOCATE\b|",
        r"TRUNCATE\b|DROP\b|GRANT\b|DENY\b|REVOKE\b|UNION\b|",
        r"CREATE\b|DECLARE\b",
        r")"
    ))
    .unwrap()
});

static RE_CORP_CONTROL: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^(if|else|while)\b").unwrap());
// Caracterul dinainte nu are voie sa fie parte de identificator: `@case`,
// `#case` si `a.case` sunt nume de variabila / coloana, nu cuvinte cheie.
// `[Case]` si `"Case"` nici nu ajung aici: linia curatata le inlocuieste cu
// spatii (P3 din auditul QA).
static RE_CUVINTE_STIVA: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(?:^|[^A-Za-z0-9_@#.])(case|begin|end)\b").unwrap());
static RE_UNION_LA_FINAL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bunion(?:\s+all)?\s*$").unwrap());
static RE_CURSOR: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bcursor\b").unwrap());
static RE_FOR_LA_FINAL: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bfor\s*$").unwrap());
static RE_INCEPE_INSERT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^insert\b").unwrap());
static RE_PUNCT_VIRGULA_FINAL: Lazy<Regex> = Lazy::new(|| Regex::new(r";\s*$").unwrap());
static RE_SELECT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bselect\b").unwrap());
static RE_BEGIN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bbegin\b").unwrap());
static RE_TRAN_DUPA_BEGIN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^\s*tran(?:saction)?\b").unwrap());
static RE_END: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bend\b").unwrap());
static RE_INCEPE_IF_WHILE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^(if|while)\b").unwrap());
static RE_DECLARE_CURSOR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bdeclare\s+([0-9A-Za-z_]+)\s+cursor\b").unwrap());

/// Textul cu literalele, identificatorii cu paranteze / ghilimele si
/// comentariile inlocuite cu spatii, octet cu octet (deci liniile raman la
/// aceleasi pozitii).
///
/// Granitele vin din [`crate::lexer`], singurul loc unde se decide ce e cod.
/// Pana la auditul QA exista aici un **al doilea** lexer, care nu stia de
/// `[...]`, `"..."` si `@`: `SELECT [Case]` deschidea un `CASE` care nu se
/// mai inchidea, si tot obiectul devenea un singur bloc (16 s pe 5.000 de
/// linii, si un diff fara nicio valoare). Un apostrof intr-un `[O'Brien]`
/// pornea un literal peste jumatate de obiect.
fn text_curatat(cod: &str) -> String {
    use crate::lexer::{segmente, Fel};
    let mut out = cod.as_bytes().to_vec();
    for s in segmente(cod) {
        if s.fel == Fel::Cod {
            continue;
        }
        for b in &mut out[s.start..s.sfarsit] {
            if *b != b'\n' {
                *b = b' ';
            }
        }
    }
    // s-au inlocuit doar octeti non-ASCII cu spatii ASCII, deci textul ramane
    // UTF-8 valid
    String::from_utf8(out).unwrap_or_else(|_| cod.to_string())
}

/// `CASE`/`BEGIN` deschid, `END` inchide. Stiva spune daca suntem in interiorul
/// unui `CASE`, caz in care liniile goale nu mai despart blocuri.
fn actualizeaza_stiva(linie_curata: &str, stiva: &mut Vec<String>) {
    for c in RE_CUVINTE_STIVA.captures_iter(linie_curata) {
        let kw = c[1].to_lowercase();
        if kw == "case" || kw == "begin" {
            stiva.push(kw);
        } else if !stiva.is_empty() {
            stiva.pop();
        }
    }
}

/// Blocul curent e inca neterminat, deci urmatoarea instructiune nu porneste un
/// bloc nou: un `DECLARE ... CURSOR FOR` asteapta corpul, iar un
/// `INSERT INTO ... ` fara `VALUES` asteapta `SELECT`-ul propriu.
fn bloc_este_continuat(curent: &str) -> bool {
    let n = norm(curent);
    if n.is_empty() {
        return false;
    }
    if RE_CURSOR.is_match(&n) && RE_FOR_LA_FINAL.is_match(&n) {
        return true;
    }
    if RE_INCEPE_INSERT.is_match(&n)
        && !n.contains("values")
        && !RE_PUNCT_VIRGULA_FINAL.is_match(&n)
    {
        // Odata ce SELECT-ul propriu al acestui INSERT a aparut, urmatoarea
        // instructiune e una noua. Fara conditia asta, INSERT-ul ar inghiti tot
        // restul obiectului, fiindca in acest dialect nu apar VALUES sau `;`.
        return !RE_SELECT.is_match(&n);
    }
    false
}

/// `BEGIN`-uri minus `END`-uri, ignorand `BEGIN TRAN`/`BEGIN TRANSACTION`
/// (deschid o tranzactie, nu un bloc de instructiuni).
pub(crate) fn adancime_begin_end(text: &str) -> i32 {
    let n = norm(text);
    let mut deschise = 0i32;
    for m in RE_BEGIN.find_iter(&n) {
        if !RE_TRAN_DUPA_BEGIN.is_match(&n[m.end()..]) {
            deschise += 1;
        }
    }
    let inchise = RE_END.find_iter(&n).count() as i32;
    deschise - inchise
}

fn incepe_cu_if_while(text: &str) -> bool {
    RE_INCEPE_IF_WHILE.is_match(text.trim_start())
}

/// Plafon de siguranta: peste atatea blocuri renuntam la grupare, ca sa nu
/// inghitim tot obiectul cand `BEGIN` apartine unui construct mult mai larg.
const MAX_BLOCURI_COMPUSE: usize = 6;

/// Leaga un `IF`/`WHILE ... BEGIN` neinchis de blocurile urmatoare, pana se
/// echilibreaza `BEGIN`/`END`.
fn uneste_blocuri_compuse(blocuri: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(blocuri.len());
    let mut i = 0usize;
    while i < blocuri.len() {
        let d = adancime_begin_end(&blocuri[i]);
        if d > 0 && incepe_cu_if_while(&blocuri[i]) {
            let mut adancime = d;
            let mut j = i + 1;
            while j < blocuri.len() && adancime > 0 && (j - i) < MAX_BLOCURI_COMPUSE {
                adancime += adancime_begin_end(&blocuri[j]);
                j += 1;
            }
            if adancime <= 0 {
                out.push(blocuri[i..j].join("\n"));
                i = j;
                continue;
            }
            // nu s-a echilibrat in limita de siguranta - nu fortam
        }
        out.push(blocuri[i].clone());
        i += 1;
    }
    out
}

fn nume_cursor_din_bloc(bloc: &str) -> Option<String> {
    RE_DECLARE_CURSOR
        .captures(bloc)
        .map(|c| c[1].to_lowercase())
}

/// `text` (normalizat, deci cu litere mici) contine `deallocate <nume>` ca
/// cuvinte intregi. Cautare directa, nu regex: numele vine din codul
/// utilizatorului, iar un regex compilat din el dadea panica pe un nume de
/// 1.000.000 de caractere (`CompiledTooBig`), gasit la audit.
fn contine_deallocate(text: &str, nume: &str) -> bool {
    let e_cuvant = |c: char| c.is_alphanumeric() || c == '_';
    let mut cautat_de_la = 0usize;
    while let Some(rel) = text[cautat_de_la..].find("deallocate") {
        let poz = cautat_de_la + rel;
        cautat_de_la = poz + "deallocate".len();
        if text[..poz].chars().next_back().is_some_and(e_cuvant) {
            continue;
        }
        let dupa = &text[cautat_de_la..];
        let fara_spatii = dupa.trim_start();
        if fara_spatii.len() == dupa.len() {
            continue; // `deallocate` lipit de urmatorul cuvant
        }
        if let Some(rest) = fara_spatii.strip_prefix(nume) {
            if !rest.chars().next().is_some_and(e_cuvant) {
                return true;
            }
        }
    }
    false
}

/// Uneste `DECLARE ... CURSOR` ... `DEALLOCATE <nume>` intr-un singur bloc.
fn uneste_cicluri_cursor(blocuri: Vec<String>) -> Vec<String> {
    // Forma normalizata a fiecarui bloc, calculata o data: inainte se
    // recalcula pentru fiecare cursor, patratic in numarul de blocuri.
    let norme: Vec<String> = blocuri.iter().map(|b| norm(b)).collect();
    let mut out: Vec<String> = Vec::with_capacity(blocuri.len());
    let mut i = 0usize;
    while i < blocuri.len() {
        if let Some(nume) = nume_cursor_din_bloc(&blocuri[i]) {
            if let Some(sfarsit) =
                (i + 1..blocuri.len()).find(|&j| contine_deallocate(&norme[j], &nume))
            {
                out.push(blocuri[i..=sfarsit].join("\n"));
                i = sfarsit + 1;
                continue;
            }
        }
        out.push(blocuri[i].clone());
        i += 1;
    }
    out
}

/// Codul are un cursor pe care l-ar putea uni `split_statements(.., true)`:
/// exista un `DECLARE <nume> CURSOR`. Fereastra arata butonul „Uneste ciclul
/// cursor" doar atunci — pe restul obiectelor n-ar face nimic.
pub fn are_cursor(cod: &str) -> bool {
    split_statements(cod, false)
        .iter()
        .any(|b| nume_cursor_din_bloc(b).is_some())
}

/// Imparte codul in blocuri logice.
///
/// `uneste_cursor` corespunde comutatorului din interfata ("Uneste ciclul
/// cursor"); implicit e oprit.
pub fn split_statements(cod: &str, uneste_cursor: bool) -> Vec<String> {
    if cod.is_empty() {
        return Vec::new();
    }

    let mut blocuri: Vec<String> = Vec::new();
    let mut curent: Vec<&str> = Vec::new();
    let mut in_antet = true;
    let mut linie_anterioara = String::new();
    let mut adancime_paranteze: i32 = 0;
    let mut stiva_case: Vec<String> = Vec::new();
    // liniile curatate, din lexer: aceleasi pozitii, fara siruri si comentarii
    let curatat = text_curatat(cod);
    let mut curate = curatat.split('\n');
    // segmentele, ca sa se stie daca o linie **incepe** in interiorul unui sir
    // sau al unui comentariu bloc deschis mai sus (un comentariu `--` se
    // inchide la capat de linie, deci nu se pune)
    let seg: Vec<crate::lexer::Segment> = crate::lexer::segmente(cod)
        .into_iter()
        .filter(|x| {
            !matches!(
                x.fel,
                crate::lexer::Fel::Cod | crate::lexer::Fel::ComentariuLinie
            )
        })
        .collect();
    let deschis_la = |poz: usize| {
        let i = seg.partition_point(|x| x.start <= poz);
        i > 0 && poz < seg[i - 1].sfarsit && poz > seg[i - 1].start
    };
    let mut poz_linie = 0usize;

    for linie in cod.split('\n') {
        let s = linie.trim();
        let curata = curate.next().unwrap_or("");
        let in_text_deschis = deschis_la(poz_linie);
        poz_linie += linie.len() + 1;

        // starea se evalueaza INAINTE de scanarea liniei curente
        let in_case = stiva_case.iter().any(|x| x == "case");
        let in_construct = adancime_paranteze > 0 || in_text_deschis || in_case;

        let delta = curata.chars().fold(0i32, |d, c| match c {
            '(' => d + 1,
            ')' => d - 1,
            _ => d,
        });
        adancime_paranteze += delta;
        if adancime_paranteze < 0 {
            adancime_paranteze = 0;
        }

        if s.is_empty() {
            if in_construct {
                curent.push(linie);
            } else if !in_antet && !curent.is_empty() {
                blocuri.push(curent.join("\n"));
                curent.clear();
            } else if in_antet && !curent.is_empty() {
                curent.push(linie);
            }
            linie_anterioara.clear();
            actualizeaza_stiva(curata, &mut stiva_case);
            continue;
        }

        // `bloc_este_continuat` normalizeaza tot blocul acumulat, deci evaluarea
        // lui la fiecare linie face segmentarea patratica in dimensiunea
        // blocului. E o functie pura si conteaza doar in conditia de mai jos,
        // asa ca o evaluam ultima, dupa ce celelalte doua au trecut.
        let porneste_instructiune = RE_INCEPUT_INSTRUCTIUNE.is_match(s)
            && !RE_CORP_CONTROL.is_match(&linie_anterioara)
            && !in_construct
            && !bloc_este_continuat(&curent.join("\n"));

        if porneste_instructiune {
            in_antet = false;
            if !curent.is_empty() {
                blocuri.push(curent.join("\n"));
                curent.clear();
            }
            curent.push(linie);
        } else {
            curent.push(linie);
        }

        linie_anterioara = s.to_string();
        actualizeaza_stiva(curata, &mut stiva_case);
    }
    if !curent.is_empty() {
        blocuri.push(curent.join("\n"));
    }

    if uneste_cursor {
        blocuri = uneste_blocuri_compuse(blocuri);
        blocuri = uneste_cicluri_cursor(blocuri);
    }

    // `UNION` / `UNION ALL` la finalul unui bloc se lipeste de blocul urmator.
    // Forma normalizata a fiecarui bloc se calculeaza o singura data: e nevoie
    // de ea atat la testul de UNION, cat si la filtrarea finala.
    let normalizate: Vec<String> = blocuri.iter().map(|b| norm(b)).collect();
    let mut rezultat: Vec<String> = Vec::with_capacity(blocuri.len());
    let mut i = 0usize;
    while i < blocuri.len() {
        if RE_UNION_LA_FINAL.is_match(&normalizate[i]) && i + 1 < blocuri.len() {
            // blocul unit se renormalizeaza; e singurul caz in care forma
            // precalculata nu mai corespunde
            let unit = format!("{}\n{}", blocuri[i], blocuri[i + 1]);
            if !norm(&unit).is_empty() {
                rezultat.push(unit);
            }
            i += 2;
        } else {
            if !normalizate[i].is_empty() {
                rezultat.push(blocuri[i].clone());
            }
            i += 1;
        }
    }

    rezultat
}

/// Cheia de identificare a unui bloc, folosita la alinierea cu
/// `SequenceMatcher`: tipul instructiunii, primele 40 de caractere din forma
/// normalizata, si 128 de biti din SHA-256 pe toata forma normalizata.
///
/// Doua blocuri cu aceeasi cheie se raporteaza `egal`, fara alta comparatie,
/// deci cheia trebuie sa insemne „acelasi text normalizat". Pana la auditul
/// din 2026-11-17 erau 32 de biti de MD5 (ca in JS): o coliziune s-a gasit in
/// 0,1 s, iar editorul arata „egal" la o modificare reala.
pub fn cheie_bloc(bloc: &str) -> String {
    const CUVINTE: [&str; 14] = [
        "select", "insert", "update", "delete", "set", "exec", "if", "while", "return", "create",
        "begin", "end", "declare", "union",
    ];
    let n = norm(bloc);
    let tip = CUVINTE
        .iter()
        .find(|kw| n.starts_with(**kw))
        .copied()
        .unwrap_or("other");
    let prefix: String = n.chars().take(40).collect();
    let h = crate::calc_hash(&n);
    format!("{}:{}:{}", tip, prefix, &h[..32])
}

/// Linia (0-based) pe care incepe fiecare bloc in textul original.
///
/// Nu se poate obtine prin insumarea numarului de linii al blocurilor:
/// segmentarea arunca liniile goale folosite ca separator, deci suma
/// subestimeaza pozitia reala cu o linie pentru fiecare gol intalnit. Se cauta
/// in schimb blocul ca subsir literal in textul original.
///
/// O potrivire conteaza doar daca incepe la inceputul unei linii (dupa
/// spatii) si **nu** in interiorul unui comentariu sau al unui literal inceput
/// inaintea ei — granitele vin din [`crate::lexer`], ca peste tot. Pana la
/// 2026-11-18 blocul `SELECT 1` se aseza pe linia `SELECT 1 din comentariu`
/// dintr-un `/* ... */` (gasit de fuzz-ul compunerii, seed 61): editorul il
/// desena acolo, iar „Aplica" ar fi inlocuit linii gresite.
pub fn linii_start_blocuri(cod: &str, blocuri: &[String]) -> Vec<usize> {
    use crate::lexer::{segmente, Fel};
    // segmentele sunt in ordinea pozitiei, deci se cauta binar; cautarea
    // liniara pe fiecare candidat facea functia patratica (P2 din auditul QA)
    let seg: Vec<_> = segmente(cod)
        .into_iter()
        .filter(|s| s.fel != Fel::Cod)
        .collect();
    let in_interior = |poz: usize| {
        let i = seg.partition_point(|s| s.start <= poz);
        i > 0 && poz < seg[i - 1].sfarsit && poz > seg[i - 1].start
    };
    let la_inceput_de_linie = |poz: usize| {
        cod[..poz]
            .rsplit('\n')
            .next()
            .is_none_or(|inainte| inainte.chars().all(|c| c == ' ' || c == '\t'))
    };
    let mut out: Vec<usize> = Vec::with_capacity(blocuri.len());
    let mut poz = 0usize;
    // numaratoarea liniilor merge din aproape in aproape: `cod[..idx]` la
    // fiecare bloc insemna inca o parcurgere a tot textului (P2)
    let (mut linii_pana, mut numarat_pana) = (0usize, 0usize);
    for bloc in blocuri {
        let gasit = if bloc.is_empty() {
            None
        } else {
            cod.get(poz..).and_then(|rest| {
                rest.match_indices(bloc.as_str())
                    .map(|(i, _)| poz + i)
                    .find(|&idx| la_inceput_de_linie(idx) && !in_interior(idx))
            })
        };
        match gasit {
            None => out.push(out.last().copied().unwrap_or(0)),
            Some(idx) => {
                if idx >= numarat_pana {
                    linii_pana += cod[numarat_pana..idx].matches('\n').count();
                } else {
                    linii_pana = cod[..idx].matches('\n').count();
                }
                numarat_pana = idx;
                out.push(linii_pana);
                poz = idx + bloc.len();
            }
        }
    }
    out
}

#[cfg(test)]
mod teste {
    use super::*;

    #[test]
    fn deallocate_se_recunoaste_ca_in_regex_ul_vechi() {
        assert!(contine_deallocate("close c deallocate c", "c"));
        assert!(contine_deallocate("deallocate   c;", "c"));
        assert!(!contine_deallocate("deallocate cc", "c"));
        assert!(!contine_deallocate("xdeallocate c", "c"));
        assert!(!contine_deallocate("deallocatec", "c"));
        assert!(contine_deallocate("x deallocatey; deallocate c", "c"));
    }

    /// Regresie din audit: panica `CompiledTooBig` cu `uneste_cursor = true`.
    #[test]
    fn nume_de_cursor_urias_nu_da_panica() {
        let t = format!(
            "DECLARE {} CURSOR FOR SELECT 1\n\nSELECT 2",
            "c".repeat(1_000_000)
        );
        assert_eq!(split_statements(&t, true).len(), 2);
    }

    #[test]
    fn blocul_nu_se_aseaza_in_interiorul_unui_comentariu() {
        let cod = "SELECT 2\n/* vechi\nSELECT 1 din comentariu\n*/\nSELECT 1";
        let blocuri = vec!["SELECT 2".to_string(), "SELECT 1".to_string()];
        assert_eq!(linii_start_blocuri(cod, &blocuri), vec![0, 4]);
    }

    #[test]
    fn nici_in_interiorul_unui_sir_sau_in_mijlocul_liniei() {
        let cod = "PRINT 'x\nEXEC p'\nSELECT a, EXEC_p\nEXEC p";
        let blocuri = vec!["EXEC p".to_string()];
        assert_eq!(linii_start_blocuri(cod, &blocuri), vec![3]);
    }

    #[test]
    fn antetul_sta_lipit_de_begin() {
        let b = split_statements("CREATE PROCEDURE p AS\nBEGIN\nSELECT 1\nEND", false);
        assert_eq!(b, vec!["CREATE PROCEDURE p AS\nBEGIN", "SELECT 1\nEND"]);
    }

    #[test]
    fn literalul_nu_porneste_bloc_nou() {
        let b = split_statements("SELECT 'INSERT INTO fake SELECT 1' AS s\nFROM t", false);
        assert_eq!(
            b.len(),
            1,
            "cuvintele din literal nu trebuie sa taie blocul"
        );
    }

    #[test]
    fn comentariul_bloc_nu_porneste_bloc_nou() {
        let b = split_statements(
            "/* comentariu\n   SELECT 1\n   pe linii */\nSELECT a FROM t",
            false,
        );
        assert_eq!(
            b.len(),
            1,
            "SELECT-ul din comentariu nu trebuie sa taie blocul"
        );
    }

    #[test]
    fn linia_goala_din_case_nu_desparte() {
        let cod = "SELECT\n  CASE\n    WHEN a = 1 THEN 'x'\n\n    WHEN a = 2 THEN 'y'\n  END AS v\nFROM t";
        assert_eq!(split_statements(cod, false).len(), 1);
    }

    #[test]
    fn union_se_lipeste_de_urmatorul() {
        // `UNION ALL` porneste propriul bloc, apoi e lipit de blocul URMATOR -
        // nu de cel precedent. Rezultatul sunt doua blocuri, nu unul:
        // primul SELECT separat, si `UNION ALL` + al doilea SELECT impreuna.
        let b = split_statements("SELECT a FROM t1\nUNION ALL\nSELECT a FROM t2", false);
        assert_eq!(b, vec!["SELECT a FROM t1", "UNION ALL\nSELECT a FROM t2"]);
    }

    #[test]
    fn corpul_unui_if_ramane_cu_if() {
        let b = split_statements("IF @x = 1\n    SELECT 1\nELSE\n    SELECT 2", false);
        assert_eq!(b.len(), 2, "IF+corp, apoi ELSE+corp");
    }

    #[test]
    fn begin_tran_nu_conteaza_ca_bloc() {
        assert_eq!(adancime_begin_end("BEGIN TRAN"), 0);
        assert_eq!(adancime_begin_end("BEGIN TRANSACTION"), 0);
        assert_eq!(adancime_begin_end("BEGIN"), 1);
        assert_eq!(adancime_begin_end("BEGIN\nEND"), 0);
    }

    #[test]
    fn linii_start_sare_peste_goluri() {
        let cod = "SELECT 1\n\nSELECT 2";
        let b = split_statements(cod, false);
        assert_eq!(linii_start_blocuri(cod, &b), vec![0, 2]);
    }

    #[test]
    fn cod_gol_da_zero_blocuri() {
        assert!(split_statements("", false).is_empty());
        assert!(split_statements("\n\n   \n", false).is_empty());
    }

    #[test]
    fn are_cursor_vede_doar_cursoarele_adevarate() {
        assert!(!are_cursor("SELECT 1"));
        assert!(!are_cursor(""));
        // „cursor" ca nume de coloana sau in comentariu nu e un cursor
        assert!(!are_cursor("SELECT cursor FROM t"));
        assert!(!are_cursor("-- declare c cursor\nSELECT 1"));
        assert!(are_cursor(
            "DECLARE c CURSOR FOR SELECT 1\nOPEN c\nCLOSE c\nDEALLOCATE c"
        ));
        // litere mari sau mici, tot aia
        assert!(are_cursor("declare   Ciclu   cursor for select 1"));
    }

    /// P3 din `cercetare/audit-qa-2026-11-18.md`: segmentarea avea un **al
    /// doilea** lexer, care nu stia de `[...]`, `"..."` si `@`. Un `[Case]`
    /// deschidea un `CASE` care nu se mai inchidea niciodata, deci tot
    /// obiectul devenea un singur bloc: 16 s in `diff_aliniat` pe 5.000 de
    /// linii, si un diff care spunea doar „totul modificat". Un apostrof
    /// intr-un `[O'Brien]` pornea un literal peste jumatate de obiect.
    #[test]
    fn un_case_in_paranteze_sau_dupa_arond_nu_inghite_obiectul() {
        let ca_blocuri = |cuvant: &str| {
            let cod: String = (0..20)
                .map(|i| format!("SELECT {cuvant} AS c{i}\n\n"))
                .collect();
            split_statements(&cod, false).len()
        };
        for cuvant in [
            "[Case]",
            "@case",
            "#case",
            "t.[End]",
            "[O'Brien]",
            "\"Can't connect\"",
            "\"Case\"",
        ] {
            assert_eq!(
                ca_blocuri(cuvant),
                20,
                "{cuvant} nu e un cuvant de stiva si nu deschide niciun literal"
            );
        }
        // iar un CASE adevarat tine blocul intreg, ca pana acum
        let cod = "SELECT CASE WHEN a = 1\n\nTHEN 2 ELSE 3 END AS x\n\nSELECT 2";
        assert_eq!(
            split_statements(cod, false).len(),
            2,
            "liniile goale dintr-un CASE nu despart blocuri"
        );
    }
}
