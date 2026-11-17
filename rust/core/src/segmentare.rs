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
static RE_CUVINTE_STIVA: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(case|begin|end)\b").unwrap());
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

/// Starea de parcurgere care traverseaza liniile: suntem intr-un literal sau
/// intr-un comentariu bloc deschis pe o linie anterioara?
#[derive(Debug, Default, Clone, Copy)]
struct StareScanare {
    in_literal: bool,
    in_comentariu_bloc: bool,
}

/// Linia cu literalele si comentariile inlocuite cu spatii, plus variatia de
/// adancime a parantezelor.
///
/// Fara asta, un `SELECT` din interiorul unui literal
/// (`'INSERT INTO fake SELECT 1'`) ar porni un bloc nou.
fn scaneaza_linie(linie: &str, stare: &mut StareScanare) -> (String, i32) {
    let ch: Vec<char> = linie.chars().collect();
    let n = ch.len();
    let mut out = String::with_capacity(n);
    let mut delta: i32 = 0;
    let mut i = 0usize;

    let gaseste = |de_la: usize, model: &[char]| -> Option<usize> {
        if model.len() > n {
            return None;
        }
        (de_la..=n.saturating_sub(model.len())).find(|&p| ch[p..p + model.len()] == *model)
    };

    while i < n {
        if stare.in_comentariu_bloc {
            match gaseste(i, &['*', '/']) {
                None => {
                    for _ in i..n {
                        out.push(' ');
                    }
                    break;
                }
                Some(sfarsit) => {
                    for _ in i..sfarsit + 2 {
                        out.push(' ');
                    }
                    i = sfarsit + 2;
                    stare.in_comentariu_bloc = false;
                    continue;
                }
            }
        }
        if stare.in_literal {
            match gaseste(i, &['\'']) {
                None => {
                    for _ in i..n {
                        out.push(' ');
                    }
                    break;
                }
                Some(qi) => {
                    if ch.get(qi + 1) == Some(&'\'') {
                        // apostrof escapat in interiorul literalului
                        out.push_str("  ");
                        i = qi + 2;
                        continue;
                    }
                    stare.in_literal = false;
                    for _ in i..qi + 1 {
                        out.push(' ');
                    }
                    i = qi + 1;
                    continue;
                }
            }
        }
        let c = ch[i];
        if c == '-' && ch.get(i + 1) == Some(&'-') {
            for _ in i..n {
                out.push(' ');
            }
            break;
        }
        if c == '/' && ch.get(i + 1) == Some(&'*') {
            stare.in_comentariu_bloc = true;
            out.push_str("  ");
            i += 2;
            continue;
        }
        if c == '\'' {
            stare.in_literal = true;
            out.push(' ');
            i += 1;
            continue;
        }
        if c == '(' {
            delta += 1;
        } else if c == ')' {
            delta -= 1;
        }
        out.push(c);
        i += 1;
    }

    (out, delta)
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
fn adancime_begin_end(text: &str) -> i32 {
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
/// inghitim tot obiectul cand `BEGIN` aparține unui construct mult mai larg.
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

/// Uneste `DECLARE ... CURSOR` ... `DEALLOCATE <nume>` intr-un singur bloc.
fn uneste_cicluri_cursor(blocuri: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(blocuri.len());
    let mut i = 0usize;
    while i < blocuri.len() {
        if let Some(nume) = nume_cursor_din_bloc(&blocuri[i]) {
            let re_dealloc =
                Regex::new(&format!(r"(?i)\bdeallocate\s+{}\b", regex::escape(&nume))).unwrap();
            if let Some(sfarsit) =
                (i + 1..blocuri.len()).find(|&j| re_dealloc.is_match(&norm(&blocuri[j])))
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

/// Imparte codul in blocuri logice.
///
/// `uneste_cursor` corespunde comutatorului din interfata ("Unește ciclul
/// cursor"); implicit e oprit.
pub fn split_statements(cod: &str, uneste_cursor: bool) -> Vec<String> {
    if cod.is_empty() {
        return Vec::new();
    }

    let mut blocuri: Vec<String> = Vec::new();
    let mut curent: Vec<&str> = Vec::new();
    let mut in_antet = true;
    let mut linie_anterioara = String::new();
    let mut stare = StareScanare::default();
    let mut adancime_paranteze: i32 = 0;
    let mut stiva_case: Vec<String> = Vec::new();

    for linie in cod.split('\n') {
        let s = linie.trim();

        // starea se evalueaza INAINTE de scanarea liniei curente
        let in_case = stiva_case.iter().any(|x| x == "case");
        let in_construct =
            adancime_paranteze > 0 || stare.in_literal || stare.in_comentariu_bloc || in_case;

        let (curata, delta) = scaneaza_linie(linie, &mut stare);
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
            actualizeaza_stiva(&curata, &mut stiva_case);
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
        actualizeaza_stiva(&curata, &mut stiva_case);
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
/// normalizata, si un prefix de MD5 pentru restul.
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
    let h = crate::md5_hex(&n);
    format!("{}:{}:{}", tip, prefix, &h[..8])
}

/// Linia (0-based) pe care incepe fiecare bloc in textul original.
///
/// Nu se poate obtine prin insumarea numarului de linii al blocurilor:
/// segmentarea arunca liniile goale folosite ca separator, deci suma
/// subestimeaza pozitia reala cu o linie pentru fiecare gol intalnit. Se caută
/// in schimb blocul ca subsir literal in textul original.
pub fn linii_start_blocuri(cod: &str, blocuri: &[String]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::with_capacity(blocuri.len());
    let mut poz = 0usize;
    for bloc in blocuri {
        match cod.get(poz..).and_then(|rest| rest.find(bloc.as_str())) {
            None => out.push(out.last().copied().unwrap_or(0)),
            Some(relativ) => {
                let idx = poz + relativ;
                out.push(cod[..idx].matches('\n').count());
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
}
