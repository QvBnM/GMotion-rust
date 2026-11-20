//! Modelul editorului de diferente: ce se deseneaza pe fiecare rand, in ce
//! culoare, cu ce numar de linie. Fara Slint, ca sa se poata testa.
//!
//! **Se strica astfel:** panourile nu se mai aliniaza (blocul modificat din
//! stanga nu e langa cel din dreapta), numerele de linie nu corespund textului
//! real, sau o bucata schimbata nu e marcata. Toate trei se vad imediat in
//! editor si duc la decizii gresite la deploy.
//!
//! Un singur motor: blocurile, imperecherea, bucatile schimbate **si asezarea
//! pe linii** vin din `gmotion_core` (`aliniaza`), aceeasi aliniere pe care o
//! foloseste compunerea textului final. Aici doar se coloreaza.

use gmotion_core::lexer::{segmente, Fel};
use gmotion_core::{aliniaza, Limba, TipDiferenta};

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

/// De unde vine linia din baza, cand textul final nu mai e textul bazei.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Marcaj {
    /// Linia e cum era in baza.
    #[default]
    Niciunul,
    /// Adusa din sursa („Aplica").
    Adus,
    /// Scrisa de mana („Editeaza").
    ScrisDeTine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rand {
    pub stanga: Parte,
    pub dreapta: Parte,
    pub stare: Stare,
    /// Primul rand al unui bloc diferit: acolo se pune eticheta (MODIFICAT...).
    pub inceput_bloc: bool,
    /// Blocul motorului din care vine randul (indexul in diff), `None` pe
    /// liniile dintre blocuri. Prin el „Aplica" stie ce aplica.
    pub bloc: Option<usize>,
    /// Pentru linia din dreapta (baza): adusa din sursa, scrisa de tine, sau
    /// neatinsa. Se pune cu [`marcheaza_textul_final`].
    pub marcaj: Marcaj,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelEditor {
    pub randuri: Vec<Rand>,
    /// Indexul randului de inceput al fiecarei diferente, in ordine: pentru
    /// navigarea „anterior / urmator" si pentru minimap.
    pub diferente: Vec<usize>,
    /// Pentru fiecare diferenta (acelasi index ca `diferente`), blocul
    /// motorului: ce primeste `gmotion_core::compune` ca sa o aplice.
    pub blocuri: Vec<usize>,
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
    construieste_cu(text_ref, text_tinta, false)
}

/// Ca [`construieste`], cu „uneste ciclul cursorului" (v3.4): blocurile de
/// cursor se iau intregi, de la `DECLARE ... CURSOR` pana la `DEALLOCATE`.
/// Taierea blocurilor sta in `core`, nu aici: un al doilea loc care decide
/// granite ar fi exact bug-ul reparat la etapa 7.
pub fn construieste_cu(text_ref: &str, text_tinta: &str, uneste_cursor: bool) -> ModelEditor {
    let a = gmotion_core::aliniaza_cu(text_ref, text_tinta, uneste_cursor);
    // Culorile se calculeaza pe blocul intreg, o data: un comentariu sau un
    // sir pe mai multe linii trebuie sa-si pastreze culoarea pe fiecare linie.
    let mut colorate: Vec<[Option<Vec<Vec<Bucata>>>; 2]> = vec![[None, None]; a.diff.len()];
    let mut randuri: Vec<Rand> = Vec::with_capacity(a.randuri.len());
    let mut diferente = Vec::new();
    let mut blocuri = Vec::new();
    for r in &a.randuri {
        let stare = match r.tip {
            TipDiferenta::Egal => Stare::Egal,
            TipDiferenta::Modificat => Stare::Modificat,
            TipDiferenta::Sters => Stare::Sters,
            TipDiferenta::Adaugat => Stare::Adaugat,
        };
        let mut parte = |linie: &Option<gmotion_core::Linie>, dreapta: bool| -> Parte {
            let Some(l) = linie else {
                return Parte::umplutura();
            };
            let bucati = match r.bloc {
                Some(b) => {
                    let rd = &a.diff[b];
                    let cache = &mut colorate[b][usize::from(dreapta)];
                    let linii = cache.get_or_insert_with(|| {
                        let (text, seg) = if dreapta {
                            (rd.dreapta.as_deref(), rd.seg_dreapta.as_deref())
                        } else {
                            (rd.stanga.as_deref(), rd.seg_stanga.as_deref())
                        };
                        linii_bloc(
                            text.unwrap_or(""),
                            seg.map(schimbat_pe_caractere).as_deref(),
                        )
                    });
                    let idx = if dreapta {
                        r.linie_in_bloc_dreapta
                    } else {
                        r.linie_in_bloc_stanga
                    };
                    idx.and_then(|i| linii.get(i)).cloned().unwrap_or_default()
                }
                None => linii_bloc(&l.text, None)
                    .into_iter()
                    .next()
                    .unwrap_or_default(),
            };
            Parte {
                nr: Some(l.nr),
                bucati,
            }
        };
        let stanga = parte(&r.stanga, false);
        let dreapta = parte(&r.dreapta, true);
        let inceput_bloc = r.primul_din_bloc && stare != Stare::Egal;
        if inceput_bloc {
            diferente.push(randuri.len());
            blocuri.push(r.bloc.unwrap_or_default());
        }
        randuri.push(Rand {
            stanga,
            dreapta,
            stare,
            inceput_bloc,
            bloc: r.bloc,
            marcaj: Marcaj::Niciunul,
        });
    }
    ModelEditor {
        randuri,
        diferente,
        blocuri,
    }
}

/// Marcheaza liniile din dreapta care nu mai sunt cum erau in baza: modelul e
/// construit pe `(sursa, text_final)`, iar `text_baza` e baza neatinsa. O
/// linie schimbata care e acum egala cu sursa a fost **adusa**; una diferita
/// si de sursa a fost **scrisa de tine** (doar daca textul are si parti
/// scrise de mana — altfel tot adusa e, dintr-un bloc aplicat partial).
pub fn marcheaza_textul_final(
    model: &mut ModelEditor,
    text_baza: &str,
    text_final: &str,
    manual: bool,
) {
    if text_baza == text_final {
        return;
    }
    let a = aliniaza(text_baza, text_final);
    let schimbate: std::collections::BTreeSet<usize> = a
        .randuri
        .iter()
        .filter(|r| r.tip != TipDiferenta::Egal)
        .filter_map(|r| r.dreapta.as_ref().map(|l| l.nr))
        .collect();
    for r in &mut model.randuri {
        let Some(nr) = r.dreapta.nr else {
            continue;
        };
        if !schimbate.contains(&nr) {
            continue;
        }
        r.marcaj = if r.stare == Stare::Egal || !manual {
            Marcaj::Adus
        } else {
            Marcaj::ScrisDeTine
        };
    }
}

/// Un rand de afisat in editor: o linie din model, sau o pliere care ascunde
/// un sir de linii identice („pliaza identicul" din bara de unelte).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RandAfisat {
    Linie(usize),
    Pliere { randuri: usize, eticheta: String },
}

/// Eticheta unei plieri, ca in macheta: „22 de linii identice, 24-45".
fn eticheta_pliere(n: usize, de_la: Option<usize>, pana_la: Option<usize>, l: Limba) -> String {
    let linii = l.numar(
        n,
        ("linie identica", "linii identice"),
        ("identical line", "identical lines"),
    );
    match (de_la, pana_la) {
        (Some(a), Some(b)) if a != b => format!("{linii}, {a}\u{2013}{b}"),
        (Some(a), Some(_)) => format!("{linii}, {a}"),
        _ => linii,
    }
}

/// Randurile de afisat, cu sirurile lungi de linii identice pliate.
///
/// `context` = cate linii identice se lasa vizibile lipit de o diferenta.
/// Se plieaza doar daca ramane ceva de ascuns (minim 3 randuri), altfel
/// plierea ar ocupa mai mult decat liniile pe care le ascunde.
pub fn plieaza(model: &ModelEditor, context: usize, l: Limba) -> Vec<RandAfisat> {
    const MINIM_ASCUNS: usize = 3;
    let n = model.randuri.len();
    let mut vizibil = vec![false; n];
    for (i, r) in model.randuri.iter().enumerate() {
        if r.stare == Stare::Egal {
            continue;
        }
        let de_la = i.saturating_sub(context);
        let pana_la = (i + context + 1).min(n);
        vizibil[de_la..pana_la].fill(true);
    }
    let mut rez = Vec::new();
    let mut i = 0;
    while i < n {
        if vizibil[i] {
            rez.push(RandAfisat::Linie(i));
            i += 1;
            continue;
        }
        let j = (i..n).find(|&k| vizibil[k]).unwrap_or(n);
        if j - i < MINIM_ASCUNS {
            rez.extend((i..j).map(RandAfisat::Linie));
        } else {
            // numerele de linie reale ale sursei; daca partea aceea e
            // umplutura (nu are numere), se cad pe cele ale bazei
            let nr = |k: usize| model.randuri[k].stanga.nr.or(model.randuri[k].dreapta.nr);
            let de_la = (i..j).find_map(nr);
            let pana_la = (i..j).rev().find_map(nr);
            rez.push(RandAfisat::Pliere {
                randuri: j - i,
                eticheta: eticheta_pliere(j - i, de_la, pana_la, l),
            });
        }
        i = j;
    }
    rez
}

/// Pentru fiecare rand, a cata diferenta e (indexul in `diferente`), sau
/// `None` pe randurile de context. Bara din jgheab e mai lata pe diferenta
/// curenta, deci randul trebuie sa stie de care diferenta face parte.
pub fn diferenta_fiecarui_rand(model: &ModelEditor) -> Vec<Option<usize>> {
    let mut rez = vec![None; model.randuri.len()];
    for (k, &start) in model.diferente.iter().enumerate() {
        for (i, r) in model.randuri.iter().enumerate().skip(start) {
            if r.stare == Stare::Egal || (i > start && r.inceput_bloc) {
                break;
            }
            rez[i] = Some(k);
        }
    }
    rez
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

    #[test]
    fn liniile_identice_lungi_se_plieaza_cu_context_in_jur() {
        // 40 de linii identice, cu o diferenta la mijloc
        let mut a: Vec<String> = (1..=40).map(|i| format!("SELECT {i}")).collect();
        let mut b = a.clone();
        b[20] = "SELECT 999".to_string();
        a.remove(0);
        b.remove(0);
        let m = construieste(&a.join("\n\n"), &b.join("\n\n"));
        let afisat = plieaza(&m, 3, Limba::Romana);
        let plieri: Vec<&RandAfisat> = afisat
            .iter()
            .filter(|r| matches!(r, RandAfisat::Pliere { .. }))
            .collect();
        assert_eq!(plieri.len(), 2, "o pliere inainte si una dupa: {afisat:?}");
        // randurile pliate plus cele afisate acopera exact modelul, o data
        let acoperit: usize = afisat
            .iter()
            .map(|r| match r {
                RandAfisat::Linie(_) => 1,
                RandAfisat::Pliere { randuri, .. } => *randuri,
            })
            .sum();
        assert_eq!(acoperit, m.randuri.len());
        // diferenta si contextul ei au ramas vizibile
        let i = m.diferente[0];
        for k in i.saturating_sub(3)..=(i + 3).min(m.randuri.len() - 1) {
            assert!(
                afisat.contains(&RandAfisat::Linie(k)),
                "randul {k} trebuia vizibil"
            );
        }
    }

    #[test]
    fn un_obiect_cu_diferente_dese_nu_se_plieaza_degeaba() {
        let a = "SELECT 1\n\nSELECT 2\n\nSELECT 3";
        let b = "SELECT 9\n\nSELECT 2\n\nSELECT 8";
        let m = construieste(a, b);
        let afisat = plieaza(&m, 3, Limba::Romana);
        assert!(
            afisat.iter().all(|r| matches!(r, RandAfisat::Linie(_))),
            "nimic de ascuns, deci nicio pliere: {afisat:?}"
        );
        assert_eq!(afisat.len(), m.randuri.len());
    }

    #[test]
    fn eticheta_plierii_e_in_romana_cu_intervalul_de_linii() {
        assert_eq!(
            eticheta_pliere(1, Some(5), Some(5), Limba::Romana),
            "1 linie identica, 5"
        );
        assert_eq!(
            eticheta_pliere(7, Some(10), Some(16), Limba::Romana),
            "7 linii identice, 10\u{2013}16"
        );
        assert_eq!(
            eticheta_pliere(22, Some(24), Some(45), Limba::Romana),
            "22 de linii identice, 24\u{2013}45"
        );
        assert_eq!(
            eticheta_pliere(4, None, None, Limba::Romana),
            "4 linii identice"
        );
        assert_eq!(
            eticheta_pliere(22, Some(24), Some(45), Limba::Engleza),
            "22 identical lines, 24\u{2013}45"
        );
    }

    #[test]
    fn liniile_aduse_si_cele_scrise_de_tine_se_marcheaza() {
        let sursa = "SELECT 1\n\nUPDATE t SET a = 1";
        let baza = "SELECT 2\n\nUPDATE t SET a = 2";
        // adus: primul bloc ia textul sursei
        let final_ = "SELECT 1\n\nUPDATE t SET a = 2";
        let mut m = construieste(sursa, final_);
        marcheaza_textul_final(&mut m, baza, final_, false);
        let marcaj = |m: &ModelEditor, t: &str| {
            m.randuri
                .iter()
                .find(|r| r.dreapta.text() == t)
                .map(|r| r.marcaj)
        };
        assert_eq!(marcaj(&m, "SELECT 1"), Some(Marcaj::Adus));
        assert_eq!(marcaj(&m, "UPDATE t SET a = 2"), Some(Marcaj::Niciunul));
        // scris de mana: o linie care nu e nici in sursa, nici in baza
        let manual = "SELECT 1\n\nUPDATE t SET a = 3";
        let mut m = construieste(sursa, manual);
        marcheaza_textul_final(&mut m, baza, manual, true);
        assert_eq!(marcaj(&m, "UPDATE t SET a = 3"), Some(Marcaj::ScrisDeTine));
        assert_eq!(marcaj(&m, "SELECT 1"), Some(Marcaj::Adus));
    }

    #[test]
    fn fiecare_rand_stie_a_cata_diferenta_e() {
        let a = "SELECT 1\n\nUPDATE t\nSET a = 1\n\nSELECT 3";
        let b = "SELECT 1\n\nUPDATE t\nSET a = 2\n\nSELECT 9";
        let m = construieste(a, b);
        assert_eq!(m.diferente.len(), 2);
        let d = diferenta_fiecarui_rand(&m);
        assert_eq!(d.len(), m.randuri.len());
        assert_eq!(d[m.diferente[0]], Some(0));
        assert_eq!(d[m.diferente[1]], Some(1));
        for (i, r) in m.randuri.iter().enumerate() {
            assert_eq!(
                d[i].is_some(),
                r.stare != Stare::Egal,
                "randul {i} ({:?})",
                r.stare
            );
        }
        // blocul modificat are doua linii: ambele apartin diferentei 0
        let bloc: Vec<Option<usize>> = (m.diferente[0]..)
            .take_while(|&i| m.randuri[i].stare != Stare::Egal)
            .map(|i| d[i])
            .collect();
        assert!(
            bloc.len() >= 2 && bloc.iter().all(|&x| x == Some(0)),
            "{bloc:?}"
        );
    }
}
