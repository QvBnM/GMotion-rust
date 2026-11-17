//! Modelul editorului de diferente: ce se deseneaza pe fiecare rand, in ce
//! culoare, cu ce numar de linie. Fara Slint, ca sa se poata testa.
//!
//! **Se strica astfel:** panourile nu se mai aliniaza (blocul modificat din
//! stanga nu e langa cel din dreapta), numerele de linie nu corespund textului
//! real, sau o bucata schimbata nu e marcata. Toate trei se vad imediat in
//! editor si duc la decizii gresite la deploy.
//!
//! Un singur motor: blocurile, imperecherea si bucatile schimbate vin din
//! `gmotion_core::diff_aliniat`. Aici doar se aseaza pe randuri.

use gmotion_core::lexer::{segmente, Fel};
use gmotion_core::{diff_aliniat, linii_start_blocuri, split_statements, TipDiferenta};

/// Culoarea de sintaxa a unei bucati.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sintaxa {
    Text,
    CuvantCheie,
    Variabila,
    Numar,
    Sir,
    Comentariu,
}

/// O bucata de text de aceeasi culoare, pe un rand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bucata {
    pub text: String,
    pub sintaxa: Sintaxa,
    /// Facea parte dintr-o diferenta in interiorul liniei.
    pub schimbat: bool,
}

/// Starea unui rand, pentru fundal si bara laterala.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stare {
    Egal,
    Modificat,
    Sters,
    Adaugat,
}

/// O jumatate de rand (stanga = referinta, dreapta = baza comparata).
/// `nr` e `None` pe randurile de umplutura, care tin panourile aliniate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parte {
    pub nr: Option<usize>,
    pub bucati: Vec<Bucata>,
}

impl Parte {
    fn umplutura() -> Self {
        Parte {
            nr: None,
            bucati: Vec::new(),
        }
    }

    /// Textul randului, fara culori.
    pub fn text(&self) -> String {
        self.bucati.iter().map(|b| b.text.as_str()).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rand {
    pub stanga: Parte,
    pub dreapta: Parte,
    pub stare: Stare,
    /// Primul rand al unui bloc diferit: acolo se pune eticheta (MODIFICAT...).
    pub inceput_bloc: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelEditor {
    pub randuri: Vec<Rand>,
    /// Indexul randului de inceput al fiecarei diferente, in ordine: pentru
    /// navigarea „anterior / urmator" si pentru minimap.
    pub diferente: Vec<usize>,
}

const CUVINTE_CHEIE: &[&str] = &[
    "add",
    "all",
    "alter",
    "and",
    "any",
    "apply",
    "as",
    "asc",
    "begin",
    "between",
    "bigint",
    "bit",
    "break",
    "by",
    "case",
    "cast",
    "catch",
    "char",
    "close",
    "coalesce",
    "collate",
    "commit",
    "continue",
    "convert",
    "create",
    "cross",
    "cursor",
    "date",
    "datetime",
    "datetime2",
    "deallocate",
    "decimal",
    "declare",
    "default",
    "delete",
    "desc",
    "distinct",
    "drop",
    "else",
    "end",
    "exec",
    "execute",
    "exists",
    "fetch",
    "float",
    "for",
    "from",
    "full",
    "function",
    "goto",
    "group",
    "having",
    "if",
    "in",
    "index",
    "inner",
    "insert",
    "int",
    "into",
    "is",
    "isnull",
    "join",
    "left",
    "like",
    "matched",
    "merge",
    "money",
    "nchar",
    "next",
    "nocount",
    "not",
    "null",
    "numeric",
    "nvarchar",
    "of",
    "off",
    "on",
    "open",
    "option",
    "or",
    "order",
    "outer",
    "output",
    "over",
    "partition",
    "print",
    "proc",
    "procedure",
    "raiserror",
    "return",
    "returns",
    "right",
    "rollback",
    "save",
    "select",
    "set",
    "smallint",
    "table",
    "then",
    "throw",
    "tinyint",
    "top",
    "tran",
    "transaction",
    "trigger",
    "try",
    "union",
    "update",
    "using",
    "values",
    "varchar",
    "view",
    "when",
    "where",
    "while",
    "with",
];

fn sintaxa_cuvant(c: &str) -> Sintaxa {
    if c.starts_with('@') {
        Sintaxa::Variabila
    } else if c.chars().next().is_some_and(|x| x.is_ascii_digit()) {
        Sintaxa::Numar
    } else if CUVINTE_CHEIE.contains(&c.to_ascii_lowercase().as_str()) {
        Sintaxa::CuvantCheie
    } else {
        Sintaxa::Text
    }
}

/// Culoarea fiecarui caracter dintr-un text (un bloc intreg, ca string-urile
/// si comentariile pe mai multe linii sa fie colorate corect).
fn sintaxa_pe_caractere(text: &str) -> Vec<Sintaxa> {
    let mut rez = Vec::with_capacity(text.len());
    for seg in segmente(text) {
        let bucata = &text[seg.start..seg.sfarsit];
        match seg.fel {
            Fel::Sir => rez.extend(bucata.chars().map(|_| Sintaxa::Sir)),
            Fel::ComentariuLinie | Fel::ComentariuBloc => {
                rez.extend(bucata.chars().map(|_| Sintaxa::Comentariu))
            }
            Fel::IdentificatorParanteze | Fel::IdentificatorGhilimele => {
                rez.extend(bucata.chars().map(|_| Sintaxa::Text))
            }
            Fel::Cod => {
                let e_cuvant = |c: char| c.is_alphanumeric() || matches!(c, '_' | '@' | '#' | '$');
                let car: Vec<char> = bucata.chars().collect();
                let mut i = 0;
                while i < car.len() {
                    if e_cuvant(car[i]) {
                        let j = (i..car.len())
                            .find(|&k| !e_cuvant(car[k]))
                            .unwrap_or(car.len());
                        let cuvant: String = car[i..j].iter().collect();
                        let s = sintaxa_cuvant(&cuvant);
                        rez.extend(std::iter::repeat_n(s, j - i));
                        i = j;
                    } else {
                        rez.push(Sintaxa::Text);
                        i += 1;
                    }
                }
            }
        }
    }
    rez
}

/// Imparte un bloc in randuri de bucati, combinand sintaxa cu marcajul de
/// schimbare (pe caractere). `schimbat` poate lipsi (fara diferente in linie).
fn linii_bloc(text: &str, schimbat: Option<&[bool]>) -> Vec<Vec<Bucata>> {
    let sint = sintaxa_pe_caractere(text);
    let mut linii: Vec<Vec<Bucata>> = vec![Vec::new()];
    for (i, c) in text.chars().enumerate() {
        if c == '\n' {
            linii.push(Vec::new());
            continue;
        }
        let s = sint.get(i).copied().unwrap_or(Sintaxa::Text);
        let sch = schimbat.and_then(|v| v.get(i).copied()).unwrap_or(false);
        let linie = linii.last_mut().expect("are mereu un rand");
        match linie.last_mut() {
            Some(b) if b.sintaxa == s && b.schimbat == sch => b.text.push(c),
            _ => linie.push(Bucata {
                text: c.to_string(),
                sintaxa: s,
                schimbat: sch,
            }),
        }
    }
    linii
}

/// Marcajul de schimbare pe caractere, din segmentele date de motor.
fn schimbat_pe_caractere(segmente: &[gmotion_core::Segment]) -> Vec<bool> {
    let mut v = Vec::new();
    for s in segmente {
        let sch = s.schimbat.is_some();
        v.extend(s.text.chars().map(|_| sch));
    }
    v
}

/// Construieste modelul pentru doua texte deja pregatite pentru afisare
/// (`gmotion_core::pregateste_afisare`).
pub fn construieste(text_ref: &str, text_tinta: &str) -> ModelEditor {
    let randuri_diff = diff_aliniat(text_ref, text_tinta);
    let start_ref = linii_start_blocuri(text_ref, &split_statements(text_ref, false));
    let start_tinta = linii_start_blocuri(text_tinta, &split_statements(text_tinta, false));
    let original_ref: Vec<&str> = text_ref.lines().collect();
    let original_tinta: Vec<&str> = text_tinta.lines().collect();

    let mut randuri: Vec<Rand> = Vec::new();
    let mut diferente: Vec<usize> = Vec::new();
    let (mut bloc_ref, mut bloc_tinta) = (0usize, 0usize);
    let (mut ultim_ref, mut ultim_tinta) = (0usize, 0usize);

    // Liniile originale dintre blocuri (goale, sau blocuri doar-comentariu pe
    // care segmentarea le arunca): context egal, ca sa se vada tot obiectul.
    let completeaza = |randuri: &mut Vec<Rand>, r: (usize, usize), t: (usize, usize)| {
        let n = r.1.saturating_sub(r.0).max(t.1.saturating_sub(t.0));
        for i in 0..n {
            let parte = |k: usize, original: &[&str]| Parte {
                nr: Some(k + 1),
                bucati: linii_bloc(original.get(k).copied().unwrap_or(""), None)
                    .into_iter()
                    .next()
                    .unwrap_or_default(),
            };
            let lr = (r.0 + i < r.1).then_some(r.0 + i);
            let lt = (t.0 + i < t.1).then_some(t.0 + i);
            randuri.push(Rand {
                stanga: lr.map_or_else(Parte::umplutura, |k| parte(k, &original_ref)),
                dreapta: lt.map_or_else(Parte::umplutura, |k| parte(k, &original_tinta)),
                stare: Stare::Egal,
                inceput_bloc: false,
            });
        }
    };

    for rd in &randuri_diff {
        let s_ref = rd.stanga.as_ref().map(|_| {
            let s = start_ref
                .get(bloc_ref)
                .map_or(ultim_ref, |&s| s.max(ultim_ref));
            bloc_ref += 1;
            s
        });
        let s_tinta = rd.dreapta.as_ref().map(|_| {
            let s = start_tinta
                .get(bloc_tinta)
                .map_or(ultim_tinta, |&s| s.max(ultim_tinta));
            bloc_tinta += 1;
            s
        });
        completeaza(
            &mut randuri,
            (ultim_ref, s_ref.unwrap_or(ultim_ref)),
            (ultim_tinta, s_tinta.unwrap_or(ultim_tinta)),
        );
        let stare = match rd.tip {
            TipDiferenta::Egal => Stare::Egal,
            TipDiferenta::Modificat => Stare::Modificat,
            TipDiferenta::Sters => Stare::Sters,
            TipDiferenta::Adaugat => Stare::Adaugat,
        };
        let st = rd.stanga.as_deref().map(|t| {
            linii_bloc(
                t,
                rd.seg_stanga
                    .as_deref()
                    .map(schimbat_pe_caractere)
                    .as_deref(),
            )
        });
        let dr = rd.dreapta.as_deref().map(|t| {
            linii_bloc(
                t,
                rd.seg_dreapta
                    .as_deref()
                    .map(schimbat_pe_caractere)
                    .as_deref(),
            )
        });
        let n_st = st.as_ref().map_or(0, Vec::len);
        let n_dr = dr.as_ref().map_or(0, Vec::len);
        if stare != Stare::Egal {
            diferente.push(randuri.len());
        }
        // Blocul ocupa aceeasi inaltime in ambele panouri (ca in v3.4): partea
        // mai scurta primeste randuri de umplutura.
        for i in 0..n_st.max(n_dr) {
            let parte = |linii: &Option<Vec<Vec<Bucata>>>, start: Option<usize>| match (
                linii.as_ref().and_then(|l| l.get(i)),
                start,
            ) {
                (Some(b), Some(s)) => Parte {
                    nr: Some(s + i + 1),
                    bucati: b.clone(),
                },
                _ => Parte::umplutura(),
            };
            randuri.push(Rand {
                stanga: parte(&st, s_ref),
                dreapta: parte(&dr, s_tinta),
                stare,
                inceput_bloc: i == 0 && stare != Stare::Egal,
            });
        }
        if let Some(s) = s_ref {
            ultim_ref = s + n_st;
        }
        if let Some(s) = s_tinta {
            ultim_tinta = s + n_dr;
        }
    }
    completeaza(
        &mut randuri,
        (ultim_ref, original_ref.len()),
        (ultim_tinta, original_tinta.len()),
    );
    ModelEditor { randuri, diferente }
}

#[cfg(test)]
mod teste {
    use super::*;

    fn text_st(m: &ModelEditor) -> Vec<String> {
        m.randuri.iter().map(|r| r.stanga.text()).collect()
    }

    #[test]
    fn obiect_identic_da_doar_randuri_egale_cu_toate_liniile() {
        let t = "CREATE PROC p AS\n\nSELECT 1\n\n-- gata";
        let m = construieste(t, t);
        assert!(m.diferente.is_empty());
        assert_eq!(
            text_st(&m),
            ["CREATE PROC p AS", "", "SELECT 1", "", "-- gata"]
        );
        let nr: Vec<Option<usize>> = m.randuri.iter().map(|r| r.stanga.nr).collect();
        assert_eq!(nr, [Some(1), Some(2), Some(3), Some(4), Some(5)]);
        assert!(m.randuri.iter().all(|r| r.stanga == r.dreapta));
    }

    #[test]
    fn blocul_modificat_ocupa_aceeasi_inaltime_in_ambele_panouri() {
        let a = "SELECT 1\n\nUPDATE t\nSET a = 1";
        let b = "SELECT 1\n\nUPDATE t\nSET a = 2,\n    b = 3";
        let m = construieste(a, b);
        assert_eq!(m.diferente.len(), 1);
        let i = m.diferente[0];
        let bloc: Vec<&Rand> = m.randuri[i..]
            .iter()
            .take_while(|r| r.stare != Stare::Egal)
            .collect();
        assert_eq!(
            bloc.len(),
            3,
            "2 linii in stanga, 3 in dreapta -> 3 randuri"
        );
        assert!(bloc[0].inceput_bloc);
        assert_eq!(bloc[2].stanga.nr, None, "umplutura in stanga");
        assert_eq!(bloc[2].dreapta.nr, Some(5));
        assert_eq!(
            bloc[0].stanga.nr,
            Some(3),
            "numerele de linie sunt cele reale, cu golul numarat"
        );
    }

    #[test]
    fn bucatile_schimbate_si_sintaxa_se_combina() {
        let m = construieste("SELECT 'Tip1' FROM t", "SELECT 'Tip2' FROM t");
        let r = &m.randuri[m.diferente[0]];
        let sir_schimbat = r
            .dreapta
            .bucati
            .iter()
            .any(|b| b.sintaxa == Sintaxa::Sir && b.schimbat);
        assert!(sir_schimbat, "{:?}", r.dreapta.bucati);
        let select = &r.stanga.bucati[0];
        assert_eq!(
            (select.text.as_str(), select.sintaxa, select.schimbat),
            ("SELECT", Sintaxa::CuvantCheie, false)
        );
    }

    #[test]
    fn sintaxa_variabile_numere_comentarii_string_pe_mai_multe_linii() {
        let l = linii_bloc("SET @a = 10 -- x\nPRINT 'a\nb'", None);
        let fel = |linie: usize, text: &str| {
            l[linie]
                .iter()
                .find(|b| b.text.contains(text))
                .map(|b| b.sintaxa)
        };
        assert_eq!(fel(0, "@a"), Some(Sintaxa::Variabila));
        assert_eq!(fel(0, "10"), Some(Sintaxa::Numar));
        assert_eq!(fel(0, "-- x"), Some(Sintaxa::Comentariu));
        assert_eq!(fel(1, "PRINT"), Some(Sintaxa::CuvantCheie));
        assert_eq!(
            fel(2, "b'"),
            Some(Sintaxa::Sir),
            "string-ul continua pe randul urmator"
        );
    }

    #[test]
    fn blocuri_adaugate_si_sterse_au_umplutura_in_partea_opusa() {
        let m = construieste("SELECT 1\n\nSELECT 2", "SELECT 1\n\nEXEC p\n\nSELECT 2");
        let adaugat = m
            .randuri
            .iter()
            .find(|r| r.stare == Stare::Adaugat)
            .expect("adaugat");
        assert_eq!(adaugat.stanga.nr, None);
        assert_eq!(adaugat.dreapta.text(), "EXEC p");
        // tot textul din dreapta apare, o singura data, in ordine
        let dreapta: Vec<String> = m
            .randuri
            .iter()
            .filter(|r| r.dreapta.nr.is_some())
            .map(|r| r.dreapta.text())
            .collect();
        assert_eq!(dreapta, ["SELECT 1", "", "EXEC p", "", "SELECT 2"]);
    }
}
