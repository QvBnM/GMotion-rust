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
    /// Pastila de pe primul rand al unui bloc care difera: ce e blocul si
    /// unde ajunge daca il aplici („DOAR IN MARTOR · 46% similar cu blocul
    /// 4"). Goala pe celelalte randuri.
    pub eticheta: String,
    /// Primul rand al locului unde ar veni blocurile din martor fara
    /// corespondent: partea din dreapta poarta textul „aici vin blocurile
    /// ...", in dreptul lor (nu pe un rand separat, mai jos; sesizat de
    /// Mihai: golul hasurat si randul-fantoma erau doua locuri pentru acelasi
    /// lucru). Textul e in `dreapta`, fara numar de linie.
    pub fantoma: bool,
    /// Partea din dreapta a randului e locul in care s-ar insera blocurile
    /// din martor (indexul grupului de inserare): se deseneaza curata, fara
    /// hasura, si se aprinde intreaga la tragere.
    pub inserare: Option<usize>,
    /// Ce se intampla cu un bloc tras si lasat pe randul asta, in panoul
    /// Modificata: inlocuieste blocul din dreapta al randului, sau se
    /// insereaza inaintea liniei lui (`gmotion_core::aplica_blocuri_la`).
    pub loc: Option<gmotion_core::Loc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelEditor {
    pub randuri: Vec<Rand>,
    /// Indexul randului de inceput al fiecarei diferente, in ordine: pentru
    /// navigarea „anterior / urmator" si pentru minimap.
    pub diferente: Vec<usize>,
    /// Pentru fiecare diferenta (acelasi index ca `diferente`), primul bloc
    /// al motorului. Tot grupul e in `grupuri`.
    pub blocuri: Vec<usize>,
    /// Pentru fiecare diferenta, blocurile motorului pe care le aplica:
    /// unul, sau instructiunile consecutive care merg in acelasi gol
    /// (`gmotion_core::grupuri_diferente`).
    pub grupuri: Vec<Vec<usize>>,
    /// Pentru fiecare diferenta, randul din model catre care se duce blocul:
    /// primul rand al blocului din dreapta pe care il inlocuieste, sau randul
    /// fantoma unde s-ar insera. Din el se deseneaza legatura din canal.
    pub tinte: Vec<Option<usize>>,
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

/// Randurile unui text scris de mana, colorate dupa sintaxa: ce se deseneaza
/// peste campul de editare cat scrii (etapa 25). Acelasi lexer ca in
/// diferente, deci culorile nu se schimba la salvare. Un text mai lung de
/// [`LINII_SINTAXA_LA_SCRIERE`] linii nu se coloreaza: lexarea la fiecare
/// tasta s-ar simti.
pub fn linii_sintaxa(text: &str) -> Vec<Vec<Bucata>> {
    if text.lines().count() > LINII_SINTAXA_LA_SCRIERE {
        return Vec::new();
    }
    linii_bloc(text, None)
}

/// Peste atatea linii, editarea de mana ramane text simplu.
pub const LINII_SINTAXA_LA_SCRIERE: usize = 5000;

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

    // Unde ajunge fiecare bloc care difera. Decizia e a motorului
    // (`gmotion_core::plasare`): in fereastra nu se ia o a doua parere, altfel
    // „Aplica" ar scrie in alt loc decat arata ecranul.
    let plasari = gmotion_core::plasari_aliniere(&a, text_tinta, uneste_cursor);
    // diferentele cum le vede omul: instructiunile consecutive care merg in
    // acelasi gol sunt una (decizia e a motorului, ca si plasarea)
    let grupuri = gmotion_core::grupuri_diferente(&a.diff, &plasari);
    // pentru fiecare bloc: grupul lui, si daca e primul din grup
    let mut grup_al_blocului: Vec<Option<(usize, bool)>> = vec![None; a.diff.len()];
    for (k, g) in grupuri.iter().enumerate() {
        for (j, &b) in g.iter().enumerate() {
            if let Some(x) = grup_al_blocului.get_mut(b) {
                *x = Some((k, j == 0));
            }
        }
    }

    // Numarul blocului pe fiecare parte, asa cum il numara omul care se uita:
    // „blocul 4 din Modificata", nu indicele din motor.
    let (mut nr_martor, mut nr_modificata) =
        (vec![0usize; a.diff.len()], vec![0usize; a.diff.len()]);
    // ordinalul blocului in tinta (ce numara si `plasare`) si drumul invers
    let mut ordinal_tinta = vec![usize::MAX; a.diff.len()];
    let mut bloc_al_ordinalului: Vec<usize> = Vec::new();
    let (mut ns, mut nd) = (0usize, 0usize);
    for (b, d) in a.diff.iter().enumerate() {
        if d.stanga.is_some() {
            ns += 1;
            nr_martor[b] = ns;
        }
        if d.dreapta.is_some() {
            nd += 1;
            nr_modificata[b] = nd;
            ordinal_tinta[b] = bloc_al_ordinalului.len();
            bloc_al_ordinalului.push(b);
        }
    }

    // Blocurile din martor fara corespondent, grupate dupa locul unde ar
    // ajunge: din ele iese randul-fantoma.
    let mut fantome: Vec<(usize, Vec<usize>)> = Vec::new();
    for p in plasari.iter().filter(|p| !p.inlocuieste) {
        match fantome.iter_mut().find(|(k, _)| *k == p.bloc_tinta) {
            Some((_, care)) => care.push(p.rand),
            None => fantome.push((p.bloc_tinta, vec![p.rand])),
        }
    }

    // Culorile se calculeaza pe blocul intreg, o data: un comentariu sau un
    // sir pe mai multe linii trebuie sa-si pastreze culoarea pe fiecare linie.
    let mut colorate: Vec<[Option<Vec<Vec<Bucata>>>; 2]> = vec![[None, None]; a.diff.len()];
    let mut randuri: Vec<Rand> = Vec::with_capacity(a.randuri.len());
    let mut diferente = Vec::new();
    let mut blocuri = Vec::new();
    // primul rand din model al fiecarui bloc, si randul fiecarei fantome
    let mut primul_rand = vec![usize::MAX; a.diff.len()];
    let mut rand_fantomei: Vec<(usize, usize)> = Vec::new();

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
        // un grup incepe o singura diferenta, pe primul lui bloc
        let inceput_bloc = r.primul_din_bloc
            && stare != Stare::Egal
            && r.bloc
                .and_then(|b| grup_al_blocului.get(b).copied().flatten())
                .is_none_or(|(_, primul)| primul);

        if inceput_bloc {
            diferente.push(randuri.len());
            blocuri.push(r.bloc.unwrap_or_default());
        }
        if let Some(b) = r.bloc {
            if primul_rand[b] == usize::MAX {
                primul_rand[b] = randuri.len();
            }
        }
        let eticheta = if inceput_bloc {
            let grup = r
                .bloc
                .and_then(|b| grup_al_blocului.get(b).copied().flatten())
                .and_then(|(k, _)| grupuri.get(k));
            match grup {
                Some(g) if g.len() > 1 => eticheta_grup(g, &plasari, &nr_martor),
                _ => eticheta_bloc(
                    r.bloc,
                    stare,
                    &plasari,
                    &nr_martor,
                    &nr_modificata,
                    &bloc_al_ordinalului,
                ),
            }
        } else {
            String::new()
        };
        let loc = match r.bloc {
            Some(b) if a.diff[b].dreapta.is_some() => gmotion_core::Loc::Inlocuieste(b),
            _ => gmotion_core::Loc::Insereaza(r.poz_dreapta),
        };
        randuri.push(Rand {
            stanga,
            dreapta,
            stare,
            inceput_bloc,
            bloc: r.bloc,
            marcaj: Marcaj::Niciunul,
            eticheta,
            fantoma: false,
            inserare: None,
            loc: Some(loc),
        });
    }

    // locul unde ar veni blocurile fara corespondent: chiar in dreptul lor,
    // in dreapta. Primul lor rand poarta textul; toate randurile lor sunt
    // „inserare" (curate, fara hasura)
    for (g, (ord, care)) in fantome.iter().enumerate() {
        let Some(primul) = randuri.iter().position(|r| r.bloc == Some(care[0])) else {
            continue;
        };
        rand_fantomei.push((*ord, primul));
        let text = text_fantoma(care, &nr_martor);
        let r = &mut randuri[primul];
        r.fantoma = true;
        r.dreapta = Parte {
            nr: None,
            bucati: vec![Bucata {
                text,
                sintaxa: Sintaxa::Comentariu,
                schimbat: false,
            }],
        };
        for r in randuri
            .iter_mut()
            .filter(|r| r.bloc.is_some_and(|b| care.contains(&b)))
        {
            r.inserare = Some(g);
        }
    }

    // tinta fiecarei diferente, in randuri de model: de acolo pleaca legatura
    // din canal
    let tinte = blocuri
        .iter()
        .map(|&b| {
            let p = plasari.iter().find(|p| p.rand == b)?;
            if p.inlocuieste {
                let tinta = bloc_al_ordinalului.get(p.bloc_tinta)?;
                (primul_rand[*tinta] != usize::MAX).then_some(primul_rand[*tinta])
            } else {
                rand_fantomei
                    .iter()
                    .find(|(ord, _)| *ord == p.bloc_tinta)
                    .map(|(_, rand)| *rand)
            }
        })
        .collect();

    ModelEditor {
        randuri,
        diferente,
        blocuri,
        grupuri,
        tinte,
    }
}

/// Textul locului unde ar veni blocurile din martor care n-au corespondent.
fn text_fantoma(care: &[usize], nr_martor: &[usize]) -> String {
    let numere: Vec<usize> = care
        .iter()
        .map(|&b| nr_martor.get(b).copied().unwrap_or(0))
        .collect();
    match numere.as_slice() {
        [n] => format!("aici vine blocul {n} din Martor, daca il aplici"),
        [a, .., z] if numere.windows(2).all(|w| w[1] == w[0] + 1) => {
            format!("aici vin blocurile {a}-{z} din Martor, daca le aplici")
        }
        _ => format!(
            "aici vin blocurile {} din Martor, daca le aplici",
            numere
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// Pastila unui grup de instructiuni consecutive doar in Martor, care merg
/// toate in acelasi loc: o singura eticheta pe tot grupul.
fn eticheta_grup(
    grup: &[usize],
    plasari: &[gmotion_core::plasare::Plasare],
    nr_martor: &[usize],
) -> String {
    let nr = |b: usize| nr_martor.get(b).copied().unwrap_or(0);
    let (prim, ultim) = (grup[0], grup[grup.len() - 1]);
    let unde = if plasari.iter().any(|p| p.rand == prim) {
        "se insereaza impreuna"
    } else {
        "fara corespondent"
    };
    format!(
        "blocurile {}-{} · DOAR IN MARTOR → {unde}",
        nr(prim),
        nr(ultim)
    )
}

/// Pastila de pe primul rand al unui bloc: ce e si unde ajunge.
fn eticheta_bloc(
    bloc: Option<usize>,
    stare: Stare,
    plasari: &[gmotion_core::plasare::Plasare],
    nr_martor: &[usize],
    nr_modificata: &[usize],
    bloc_al_ordinalului: &[usize],
) -> String {
    let Some(b) = bloc else {
        return String::new();
    };
    let nume = match stare {
        Stare::Modificat => format!(
            "bloc {} · MODIFICAT",
            nr_martor.get(b).copied().unwrap_or(0)
        ),
        Stare::Sters => format!(
            "bloc {} · DOAR IN MARTOR",
            nr_martor.get(b).copied().unwrap_or(0)
        ),
        Stare::Adaugat => {
            return format!(
                "bloc {} · DOAR IN MODIFICATA",
                nr_modificata.get(b).copied().unwrap_or(0)
            )
        }
        Stare::Egal => return String::new(),
    };
    let Some(p) = plasari.iter().find(|p| p.rand == b) else {
        return nume;
    };
    let tinta = bloc_al_ordinalului
        .get(p.bloc_tinta)
        .and_then(|&t| nr_modificata.get(t).copied())
        .unwrap_or(0);
    if p.inlocuieste {
        format!("{nume} → inlocuieste blocul {tinta} din Modificata")
    } else if p.scor >= 40.0 && tinta > 0 {
        format!(
            "{nume} · {:.0}% similar cu blocul {tinta} → se insereaza, nu inlocuieste",
            p.scor
        )
    } else {
        format!("{nume} → fara corespondent: se insereaza")
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

/// O linie din previzualizarea la hover („cum arata Modificata daca aplici").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiniePrevizualizare {
    /// Numarul liniei in textul rezultat; `None` pe randul „…".
    pub nr: Option<usize>,
    pub text: String,
    /// Linia vine din blocul aplicat (se evidentiaza); altfel e context.
    pub adusa: bool,
}

/// Previzualizarea aplicarii unui bloc: liniile din `rezultat` care nu mai
/// sunt cum erau in `curent`, cu `context` linii neschimbate in jur, cel mult
/// `max_aduse` evidentiate (restul se rezuma intr-un rand „…").
///
/// `rezultat` vine din `gmotion_core::aplica_blocuri`, aceeasi functie ca la
/// „Aplica": previzualizarea nu are voie sa arate alt text decat cel care se
/// scrie. Aici doar se taie bucata de aratat (prefixul si sufixul comun pe
/// linii sunt neschimbate).
pub fn previzualizare(
    curent: &str,
    rezultat: &str,
    context: usize,
    max_aduse: usize,
) -> Vec<LiniePrevizualizare> {
    let c: Vec<&str> = curent.split('\n').collect();
    let r: Vec<&str> = rezultat.split('\n').collect();
    let prefix = c.iter().zip(&r).take_while(|(a, b)| a == b).count();
    let max_sufix = c.len().min(r.len()) - prefix;
    let sufix = c
        .iter()
        .rev()
        .zip(r.iter().rev())
        .take(max_sufix)
        .take_while(|(a, b)| a == b)
        .count();
    let (inceput, sfarsit) = (prefix, r.len() - sufix); // aduse: [inceput, sfarsit)
    let linie = |i: usize, adusa: bool| LiniePrevizualizare {
        nr: Some(i + 1),
        text: r[i].to_string(),
        adusa,
    };
    let mut out: Vec<LiniePrevizualizare> = (inceput.saturating_sub(context)..inceput)
        .map(|i| linie(i, false))
        .collect();
    let aduse = sfarsit - inceput;
    out.extend((inceput..inceput + aduse.min(max_aduse)).map(|i| linie(i, true)));
    if aduse > max_aduse {
        out.push(LiniePrevizualizare {
            nr: None,
            text: format!("… +{}", aduse - max_aduse),
            adusa: true,
        });
    }
    out.extend((sfarsit..(sfarsit + context).min(r.len())).map(|i| linie(i, false)));
    out
}

/// Pentru fiecare rand, a cata diferenta e (indexul in `diferente`), sau
/// `None` pe randurile de context. Bara din jgheab e mai lata pe diferenta
/// curenta, deci randul trebuie sa stie de care diferenta face parte.
pub fn diferenta_fiecarui_rand(model: &ModelEditor) -> Vec<Option<usize>> {
    // a cata diferenta e fiecare bloc al motorului (un grup are mai multe)
    let mut a_blocului: std::collections::HashMap<usize, usize> = Default::default();
    for (k, g) in model.grupuri.iter().enumerate() {
        for &b in g {
            a_blocului.insert(b, k);
        }
    }
    model
        .randuri
        .iter()
        .map(|r| {
            if r.stare == Stare::Egal {
                return None;
            }
            r.bloc.and_then(|b| a_blocului.get(&b).copied())
        })
        .collect()
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
    fn sintaxa_la_scriere_are_un_rand_pe_linie_si_aceleasi_caractere() {
        let text = "SELECT @a,\t'x\ny' -- c\n\nEND";
        let linii = linii_sintaxa(text);
        assert_eq!(linii.len(), 4, "si linia goala are randul ei");
        let refacut: Vec<String> = linii
            .iter()
            .map(|l| l.iter().map(|b| b.text.as_str()).collect())
            .collect();
        assert_eq!(
            refacut.join("\n"),
            text,
            "suprapunerea are exact textul campului"
        );
        assert_eq!(linii[0][0].sintaxa, Sintaxa::CuvantCheie);
        assert!(
            linii[1].iter().any(|b| b.sintaxa == Sintaxa::Sir),
            "sirul continua pe linia 2"
        );
        let lung = "SELECT 1\n".repeat(LINII_SINTAXA_LA_SCRIERE + 1);
        assert!(linii_sintaxa(&lung).is_empty(), "prea lung: text simplu");
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

#[cfg(test)]
mod teste_previzualizare {
    use super::*;

    #[test]
    fn arata_liniile_aduse_cu_context_in_jur() {
        let curent = "a\nb\nc\nd\ne\nf";
        let rezultat = "a\nb\nc\nNOU1\nNOU2\nd\ne\nf";
        let p = previzualizare(curent, rezultat, 2, 10);
        let texte: Vec<(&str, bool)> = p.iter().map(|l| (l.text.as_str(), l.adusa)).collect();
        assert_eq!(
            texte,
            vec![
                ("b", false),
                ("c", false),
                ("NOU1", true),
                ("NOU2", true),
                ("d", false),
                ("e", false)
            ]
        );
        assert_eq!(p[2].nr, Some(4), "numarul liniei din rezultat");
    }

    /// Un bloc inlocuit: linia veche nu apare, cea noua e evidentiata.
    #[test]
    fn la_inlocuire_se_vede_doar_textul_nou() {
        let p = previzualizare("x\nvechi\ny", "x\nnou\ny", 1, 10);
        let texte: Vec<&str> = p.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texte, vec!["x", "nou", "y"]);
        assert!(p[1].adusa && !p[0].adusa && !p[2].adusa);
    }

    #[test]
    fn un_bloc_lung_se_rezuma() {
        let rezultat = (0..30)
            .map(|i| format!("l{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let p = previzualizare("", &rezultat, 2, 5);
        assert_eq!(p.iter().filter(|l| l.adusa && l.nr.is_some()).count(), 5);
        assert_eq!(p.last().map(|l| l.text.as_str()), Some("… +25"));
    }

    /// Aceleasi linii pe ambele parti (prefixul si sufixul se suprapun):
    /// fara panica, fara nimic evidentiat.
    #[test]
    fn fara_schimbare_nu_evidentiaza_nimic() {
        let p = previzualizare("a\na\na", "a\na\na", 2, 5);
        assert!(p.iter().all(|l| !l.adusa), "{p:?}");
    }
}

#[cfg(test)]
mod teste_plasare {
    use super::*;

    const MARTOR: &str = concat!(
        "CREATE PROCEDURE p\n @DocumentId int\nAS\n",
        "\tDECLARE @t int\n",
        "\tEXEC uspAudit_Log @Actiune = 'GetTemplates'\n",
        "\tSELECT 'RCA.doc' AS Template"
    );
    const MODIFICATA: &str = concat!(
        "CREATE PROCEDURE p(\n @DocumentId INT,\n @UserId INT)\nAS\nBEGIN\n",
        "\tDECLARE @t INT\n\n",
        "\tSELECT 'RCA.doc' AS Template,\n\t\tCAST(0 AS BIT) AS Nou\nEND"
    );

    fn model() -> ModelEditor {
        construieste(
            &gmotion_core::pregateste_afisare(MARTOR),
            &gmotion_core::pregateste_afisare(MODIFICATA),
        )
    }

    /// Eticheta spune si ce e blocul, si unde ajunge: fara partea a doua,
    /// „Aplica" ramane o sageata despre care nu stii ce face.
    #[test]
    fn eticheta_spune_unde_ajunge_blocul() {
        let m = model();
        let etichete: Vec<&str> = m
            .randuri
            .iter()
            .filter(|r| !r.eticheta.is_empty())
            .map(|r| r.eticheta.as_str())
            .collect();
        assert!(
            etichete
                .iter()
                .any(|e| e.contains("MODIFICAT") && e.contains("inlocuieste blocul")),
            "{etichete:?}"
        );
        assert!(
            etichete
                .iter()
                .any(|e| e.contains("DOAR IN MARTOR") && e.contains("se insereaza")),
            "{etichete:?}"
        );
        // etichetele stau doar pe primul rand al blocului
        assert!(m
            .randuri
            .iter()
            .all(|r| r.eticheta.is_empty() || r.inceput_bloc));
    }

    /// Locul unde ar veni blocul fara corespondent e in dreptul lui, in
    /// dreapta (etapa 23: nu un rand separat, mai jos). Fara el, „se
    /// insereaza" e o promisiune fara loc.
    #[test]
    fn exista_rand_fantoma_pentru_blocul_fara_corespondent() {
        let m = model();
        let fantome: Vec<&Rand> = m.randuri.iter().filter(|r| r.fantoma).collect();
        assert_eq!(fantome.len(), 1, "{:?}", fantome);
        let text = fantome[0].dreapta.text();
        assert!(text.contains("din Martor"), "{text:?}");
        // in dreptul blocului: linia lui din Martor la stanga, textul la dreapta
        assert!(fantome[0].stanga.nr.is_some() && fantome[0].dreapta.nr.is_none());
        assert!(fantome[0].inserare.is_some());
        assert!(
            m.randuri
                .iter()
                .filter(|r| r.inserare.is_some())
                .all(|r| r.dreapta.nr.is_none()),
            "locul de inserare n-are linii din Modificata"
        );
    }

    /// Fiecare diferenta stie catre ce rand se duce: de acolo se deseneaza
    /// legatura din canal.
    #[test]
    fn fiecare_diferenta_are_o_tinta_in_model() {
        let m = model();
        assert_eq!(m.tinte.len(), m.diferente.len());
        for (k, tinta) in m.tinte.iter().enumerate() {
            let Some(t) = tinta else { continue };
            assert!(*t < m.randuri.len(), "tinta {t} iese din model");
            let r = &m.randuri[*t];
            assert!(
                r.fantoma || r.dreapta.nr.is_some(),
                "diferenta {k} tinteste un rand fara nimic in dreapta: {r:?}"
            );
        }
        // blocul fara corespondent tinteste fantoma
        let fantoma = m.randuri.iter().position(|r| r.fantoma).expect("fantoma");
        assert!(m.tinte.contains(&Some(fantoma)), "{:?}", m.tinte);
    }
}
