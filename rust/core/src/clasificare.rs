//! In ce anume difera doua versiuni ale unui obiect: cod, sir, comentariu.
//!
//! **Se strica astfel:** un obiect care difera doar intr-un mesaj afisat
//! utilizatorului e trecut drept „difera in cod" (sau invers), si cine se uita
//! in lista decide gresit ce deployeaza. Exact cazul
//! `GenerateOrderByRecommendedOrder`: o majuscula intr-un sir, pe care v3.4 o
//! inghitea de tot.
//!
//! Granitele cod / sir / comentariu se iau **numai** din `lexer` (vezi
//! `CLAUDE.md` si capcana din `STARE.md`): o regula noua pe text, aici, ar
//! reintroduce clasa de bug-uri reparata la etapa 7.

use crate::diff::{diff_aliniat, Segment, TipDiferenta};
use crate::lexer::{segmente, Fel};
use crate::Limba;

/// Cate diferente sunt in fiecare fel de text. Se numara **bucatile
/// schimbate**, nu caracterele: „un sir" inseamna o singura bucata de sir
/// diferita, oriunde in obiect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Clasificare {
    pub cod: usize,
    pub siruri: usize,
    pub comentarii: usize,
}

impl Clasificare {
    pub fn nimic(&self) -> bool {
        self.cod == 0 && self.siruri == 0 && self.comentarii == 0
    }

    /// Doar sirurile difera — cazul care conteaza la revizuire: un mesaj
    /// schimbat nu are acelasi risc ca o conditie schimbata.
    pub fn doar_siruri(&self) -> bool {
        self.siruri > 0 && self.cod == 0 && self.comentarii == 0
    }

    pub fn doar_comentarii(&self) -> bool {
        self.comentarii > 0 && self.cod == 0 && self.siruri == 0
    }
}

/// Felul textului dintr-un interval: daca intervalul atinge mai multe feluri,
/// se raporteaza fiecare o data.
fn feluri_din_interval(lex: &[crate::lexer::Segment], start: usize, sfarsit: usize) -> [bool; 3] {
    // [cod, sir, comentariu]
    let mut f = [false, false, false];
    // segmentele sunt in ordinea pozitiei: sarim direct la primul care atinge
    // intervalul, in loc sa le parcurgem pe toate pentru fiecare bucata
    let de_la = lex.partition_point(|s| s.sfarsit <= start);
    for s in &lex[de_la..] {
        if s.start >= sfarsit {
            break;
        }
        match s.fel {
            Fel::Sir => f[1] = true,
            Fel::ComentariuLinie | Fel::ComentariuBloc => f[2] = true,
            // un identificator in paranteze sau ghilimele e tot cod: el
            // numeste ceva, nu e text afisat
            Fel::Cod | Fel::IdentificatorParanteze | Fel::IdentificatorGhilimele => f[0] = true,
        }
    }
    f
}

/// Bucatile schimbate dintr-o parte a unui bloc modificat, pe feluri.
fn numara_parte(text: &str, seg: &[Segment]) -> Clasificare {
    let lex = segmente(text);
    let mut c = Clasificare::default();
    let mut poz = 0usize;
    for s in seg {
        let lung = s.text.len();
        if s.schimbat.is_some() && lung > 0 {
            let f = feluri_din_interval(&lex, poz, poz + lung);
            c.cod += usize::from(f[0]);
            c.siruri += usize::from(f[1]);
            c.comentarii += usize::from(f[2]);
        }
        poz += lung;
    }
    c
}

/// Cele doua parti ale unui bloc modificat descriu **aceleasi** schimbari, din
/// doua unghiuri (ce era, ce e). Numarate amandoua, un singur sir schimbat ar
/// iesi „siruri". Se ia maximul pe fiecare fel: o parte poate avea mai multe
/// bucati decat cealalta (text sters fara inlocuitor).
fn imbina(a: Clasificare, b: Clasificare) -> Clasificare {
    Clasificare {
        cod: a.cod.max(b.cod),
        siruri: a.siruri.max(b.siruri),
        comentarii: a.comentarii.max(b.comentarii),
    }
}

/// Felul unui bloc intreg aparut sau disparut: daca e numai comentariu, e o
/// diferenta de comentariu; altfel e cod (un bloc nou care contine un sir
/// ramane o schimbare de cod — s-a adaugat o instructiune, nu un mesaj).
fn clasifica_bloc(text: &str, c: &mut Clasificare) {
    let lex = segmente(text);
    let doar_comentariu = !lex.is_empty()
        && lex.iter().all(|s| {
            matches!(s.fel, Fel::ComentariuLinie | Fel::ComentariuBloc)
                || (s.fel == Fel::Cod && text[s.start..s.sfarsit].trim().is_empty())
        });
    if doar_comentariu {
        c.comentarii += 1;
    } else {
        c.cod += 1;
    }
}

/// In ce difera doua versiuni, pe textul pregatit pentru afisare
/// (`pregateste_afisare`), adica acelasi text pe care se face si diff-ul din
/// editor: un singur motor, aceleasi granite de bloc.
pub fn clasifica_diferente(text_referinta: &str, text_curent: &str) -> Clasificare {
    clasifica_randuri(&diff_aliniat(text_referinta, text_curent))
}

/// Acelasi lucru, pe randuri deja calculate: cine are nevoie si de diff, si de
/// clasificare (lista), nu plateste a doua aliniere.
pub fn clasifica_randuri(randuri: &[crate::diff::RandDiff]) -> Clasificare {
    let mut c = Clasificare::default();
    for r in randuri {
        match r.tip {
            TipDiferenta::Egal => {}
            TipDiferenta::Modificat => {
                let parte = |text: Option<&str>, seg: Option<&[Segment]>| match (text, seg) {
                    (Some(t), Some(s)) => numara_parte(t, s),
                    _ => Clasificare::default(),
                };
                let st = parte(r.stanga.as_deref(), r.seg_stanga.as_deref());
                let dr = parte(r.dreapta.as_deref(), r.seg_dreapta.as_deref());
                let b = imbina(st, dr);
                c.cod += b.cod;
                c.siruri += b.siruri;
                c.comentarii += b.comentarii;
            }
            // O mutare se numara **o data**, pe capatul din martor: cele doua
            // randuri descriu acelasi bloc, in doua locuri. Rama „cod":
            // ordinea instructiunilor s-a schimbat, chiar daca textul nu.
            TipDiferenta::Mutat if r.stanga.is_some() => {
                if let Some(t) = r.stanga.as_deref() {
                    clasifica_bloc(t, &mut c);
                }
            }
            TipDiferenta::Mutat => {}
            TipDiferenta::Sters => {
                if let Some(t) = r.stanga.as_deref() {
                    clasifica_bloc(t, &mut c);
                }
            }
            TipDiferenta::Adaugat => {
                if let Some(t) = r.dreapta.as_deref() {
                    clasifica_bloc(t, &mut c);
                }
            }
        }
    }
    c
}

/// Textul din coloana „Difera in", ca in machete: „cod, siruri", „un sir",
/// „comentariu", „QUOTED_IDENTIFIER", „—".
///
/// `setari` sunt numele setarilor de sesiune care difera (`QUOTED_IDENTIFIER`,
/// `ANSI_NULLS`): nu se vad in text, dar schimba comportamentul la executie.
pub fn eticheta(c: Clasificare, setari: &[&str]) -> String {
    eticheta_in(c, setari, Limba::Romana)
}

/// `eticheta`, in limba data.
pub fn eticheta_in(c: Clasificare, setari: &[&str], l: Limba) -> String {
    let mut parti: Vec<String> = Vec::new();
    if c.cod > 0 {
        parti.push(l.alege("cod", "code").to_string());
    }
    if c.siruri > 0 {
        // singular doar cand sirul e singura diferenta din tot obiectul
        let singur = c.siruri == 1 && c.cod == 0 && c.comentarii == 0 && setari.is_empty();
        parti.push(
            if singur {
                l.alege("un sir", "a string")
            } else {
                l.alege("siruri", "strings")
            }
            .to_string(),
        );
    }
    if c.comentarii > 0 {
        parti.push(
            if c.comentarii == 1 {
                l.alege("comentariu", "comment")
            } else {
                l.alege("comentarii", "comments")
            }
            .to_string(),
        );
    }
    parti.extend(setari.iter().map(|s| match *s {
        "hinturi" => l.alege("hinturi", "hints").to_string(),
        // comentariile pot veni si din `clasificare` (un bloc numai
        // comentariu) si din `diferente_ascunse` (amprenta comentariilor):
        // textul e acelasi, deci nu se repeta
        "comentarii" if c.comentarii > 0 => String::new(),
        "comentarii" => l.alege("comentarii", "comments").to_string(),
        alt => alt.to_string(),
    }));
    parti.retain(|p| !p.is_empty());
    if parti.is_empty() {
        "—".to_string()
    } else {
        parti.join(", ")
    }
}

/// O propozitie despre ce difera, pentru previzualizarea din lista:
/// „un singur caracter, intr-un sir: Nu → nu" cand chiar atat e, altfel
/// „difera in cod, siruri".
///
/// Rostul ei e sa raspunda la intrebarea pusa inainte de a deschide editorul:
/// merita deschis? Un mesaj cu o majuscula schimbata, nu; o conditie, da.
pub fn propozitie(text_referinta: &str, text_curent: &str, setari: &[&str]) -> String {
    propozitie_in(text_referinta, text_curent, setari, Limba::Romana)
}

/// Eticheta romaneasca (cum o tin modelele, calculata o data) in limba data.
/// Partile necunoscute (numele setarilor) raman cum sunt. Testul
/// `traducerea_etichetei_da_aceeasi_eticheta` o tine egala cu `eticheta_in`.
pub fn traduce_eticheta(ro: &str, l: Limba) -> String {
    if l == Limba::Romana {
        return ro.to_string();
    }
    ro.split(", ")
        .map(|p| match p {
            "cod" => "code",
            "un sir" => "a string",
            "siruri" => "strings",
            "comentariu" => "comment",
            "comentarii" => "comments",
            "hinturi" => "hints",
            alt => alt,
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `propozitie`, in limba data.
pub fn propozitie_in(text_referinta: &str, text_curent: &str, setari: &[&str], l: Limba) -> String {
    let randuri = diff_aliniat(text_referinta, text_curent);
    let c = clasifica_randuri(&randuri);
    let general = || {
        if c.nimic() && setari.is_empty() {
            l.alege("fara diferente", "no differences").to_string()
        } else {
            format!(
                "{} {}",
                l.alege("difera in", "differs in"),
                eticheta_in(c, setari, l)
            )
        }
    };
    if !setari.is_empty() {
        return general();
    }
    // o singura bucata schimbata, pe fiecare parte: atunci se poate spune
    // exact ce s-a schimbat
    let mut schimbate: Vec<(String, String)> = Vec::new();
    for r in &randuri {
        if r.tip != TipDiferenta::Modificat {
            if r.tip != TipDiferenta::Egal {
                return general();
            }
            continue;
        }
        let bucati = |seg: &Option<Vec<Segment>>| -> Vec<String> {
            seg.as_ref().map_or_else(Vec::new, |v| {
                v.iter()
                    .filter(|s| s.schimbat.is_some())
                    .map(|s| s.text.clone())
                    .collect()
            })
        };
        let (st, dr) = (bucati(&r.seg_stanga), bucati(&r.seg_dreapta));
        if st.len() > 1 || dr.len() > 1 || schimbate.len() == 1 {
            return general();
        }
        if let (Some(a), Some(b)) = (st.first(), dr.first()) {
            schimbate.push((a.clone(), b.clone()));
        } else {
            return general();
        }
    }
    let [(a, b)] = schimbate.as_slice() else {
        return general();
    };
    const LUNGIME_MAXIMA: usize = 24;
    if a.chars().count() > LUNGIME_MAXIMA || b.chars().count() > LUNGIME_MAXIMA {
        return general();
    }
    let unde = if c.doar_siruri() {
        l.alege("intr-un sir", "in a string")
    } else if c.doar_comentarii() {
        l.alege("intr-un comentariu", "in a comment")
    } else {
        l.alege("in cod", "in code")
    };
    // „un singur caracter" se refera la cat s-a schimbat, nu la cat se arata:
    // in machete scrie „un singur caracter, intr-un sir: Nu → nu", unde se
    // arata cuvantul din jur (asa il recunoaste omul in cod).
    let un_caracter = a.chars().count() == b.chars().count()
        && a.chars().zip(b.chars()).filter(|(x, y)| x != y).count() == 1;
    let cat = if un_caracter {
        l.alege("un singur caracter", "a single character")
    } else {
        l.alege("o singura bucata", "a single piece")
    };
    format!("{cat}, {unde}: {a} → {b}")
}

#[cfg(test)]
mod teste_b11 {
    use super::*;

    /// `clasifica_randuri` si `doar_comentarii` sunt API public si nu aveau
    /// test propriu (B-11 din auditul 2026-11-23). `clasifica` merge pe
    /// acelasi drum, deci se compara cu el.
    #[test]
    fn clasifica_randuri_da_acelasi_lucru_ca_clasifica() {
        let a = "CREATE PROC p AS\n-- o nota\nSELECT 'mesaj', col FROM t";
        let b = "CREATE PROC p AS\n-- alta nota\nSELECT 'mesaj', col FROM t";
        let randuri = crate::diff::diff_aliniat(a, b);
        assert_eq!(clasifica_randuri(&randuri), clasifica_diferente(a, b));
    }

    #[test]
    fn doar_comentarii_cere_sa_nu_fie_nimic_altceva() {
        let c = Clasificare {
            cod: 0,
            siruri: 0,
            comentarii: 2,
        };
        assert!(c.doar_comentarii());
        assert!(!c.doar_siruri());
        assert!(!Clasificare {
            cod: 1,
            siruri: 0,
            comentarii: 2,
        }
        .doar_comentarii());
        assert!(!Clasificare {
            cod: 0,
            siruri: 1,
            comentarii: 2,
        }
        .doar_comentarii());
        // nicio diferenta: nu e „doar comentarii"
        assert!(!Clasificare::default().doar_comentarii());
    }
}

#[cfg(test)]
mod teste {
    use super::*;
    use crate::pregateste_afisare;

    fn clasifica(a: &str, b: &str) -> Clasificare {
        clasifica_diferente(&pregateste_afisare(a), &pregateste_afisare(b))
    }

    #[test]
    fn o_majuscula_intr_un_sir_e_diferenta_de_sir_nu_de_cod() {
        // cazul GenerateOrderByRecommendedOrder, cel pe care v3.4 il ascundea
        let a = "CREATE PROC p AS\nBEGIN\nRAISERROR('nu s-au gasit date', 16, 1);\nEND";
        let b = "CREATE PROC p AS\nBEGIN\nRAISERROR('Nu s-au gasit date', 16, 1);\nEND";
        let c = clasifica(a, b);
        assert!(c.doar_siruri(), "{c:?}");
        assert_eq!(eticheta(c, &[]), "un sir");
    }

    #[test]
    fn un_comentariu_schimbat_intr_un_bloc_modificat_se_vede() {
        let a = "UPDATE t SET a = 1 -- vechi";
        let b = "UPDATE t SET a = 2 -- nou";
        let c = clasifica(a, b);
        assert_eq!((c.cod, c.siruri, c.comentarii), (1, 0, 1), "{c:?}");
        assert_eq!(eticheta(c, &[]), "cod, comentariu");
    }

    /// Asa se comporta motorul azi, si e **deliberat** (decizia lui Mihai,
    /// 2026-11-17): normalizarea sterge comentariile, ca in v3.4, deci un
    /// obiect care difera doar intr-un comentariu iese „identic" si nu ajunge
    /// in lista. Clasificarea nu-l poate vedea nici ea, fiindca segmentarea nu
    /// da blocul respectiv mai departe.
    ///
    /// Testul sta aici ca sa cada daca cineva schimba regula fara sa stie ce
    /// schimba: atunci se schimba si ce intra in lista, si asta vrea decizie,
    /// fixture-uri si o intrare in `JURNAL.md`.
    #[test]
    fn comentariul_singur_ramane_zgomot_filtrat() {
        let c = clasifica("UPDATE t SET a = 1 -- vechi", "UPDATE t SET a = 1 -- nou");
        assert!(c.nimic(), "{c:?}");
        let c = clasifica("SELECT 1\n\nSELECT 2", "SELECT 1\n\n-- nota\n\nSELECT 2");
        assert!(c.nimic(), "{c:?}");
    }

    #[test]
    fn o_conditie_schimbata_e_diferenta_de_cod() {
        let a = "SELECT * FROM t WHERE a < 10";
        let b = "SELECT * FROM t WHERE a <= 10";
        let c = clasifica(a, b);
        assert_eq!((c.cod > 0, c.siruri, c.comentarii), (true, 0, 0), "{c:?}");
        assert_eq!(eticheta(c, &[]), "cod");
    }

    #[test]
    fn cod_si_sir_in_acelasi_obiect_dau_amandoua() {
        let a = "SELECT * FROM t WHERE a < 10\n\nPRINT 'gata'";
        let b = "SELECT * FROM t WHERE a <= 10\n\nPRINT 'terminat'";
        let c = clasifica(a, b);
        assert!(c.cod > 0 && c.siruri > 0, "{c:?}");
        assert_eq!(eticheta(c, &[]), "cod, siruri");
    }

    #[test]
    fn un_bloc_care_incepe_cu_comentariu_si_are_cod_e_cod() {
        let a = "SELECT 1\n\nSELECT 2";
        let b = "SELECT 1\n\n-- de scos dupa migrare\nEXEC dbo.p;\n\nSELECT 2";
        let c = clasifica(a, b);
        assert_eq!((c.cod, c.comentarii), (1, 0), "{c:?}");
    }

    #[test]
    fn un_bloc_de_cod_adaugat_e_cod_chiar_daca_are_siruri() {
        let a = "SELECT 1\n\nSELECT 2";
        let b = "SELECT 1\n\nPRINT 'ceva'\n\nSELECT 2";
        let c = clasifica(a, b);
        assert_eq!((c.cod, c.siruri, c.comentarii), (1, 0, 0), "{c:?}");
    }

    #[test]
    fn obiecte_identice_nu_difera_in_nimic() {
        let t = "CREATE PROC p AS\nSELECT 'x' -- nota";
        let c = clasifica(t, t);
        assert!(c.nimic(), "{c:?}");
        assert_eq!(eticheta(c, &[]), "—");
    }

    #[test]
    fn setarile_apar_in_eticheta_chiar_daca_textul_e_identic() {
        let c = Clasificare::default();
        assert_eq!(eticheta(c, &["QUOTED_IDENTIFIER"]), "QUOTED_IDENTIFIER");
        assert_eq!(
            eticheta(c, &["QUOTED_IDENTIFIER", "ANSI_NULLS"]),
            "QUOTED_IDENTIFIER, ANSI_NULLS"
        );
        // cu setari, sirul nu mai e singura diferenta, deci forma lunga
        let s = Clasificare {
            siruri: 1,
            ..Default::default()
        };
        assert_eq!(eticheta(s, &["ANSI_NULLS"]), "siruri, ANSI_NULLS");
    }

    #[test]
    fn un_identificator_in_paranteze_e_cod_nu_sir() {
        let a = "SELECT [Coloana Veche] FROM t";
        let b = "SELECT [Coloana Noua] FROM t";
        let c = clasifica(a, b);
        assert_eq!((c.siruri, c.comentarii), (0, 0), "{c:?}");
        assert!(c.cod > 0);
    }

    #[test]
    fn un_sir_pe_mai_multe_linii_ramane_sir() {
        let a = "PRINT 'linia unu\nlinia doi'";
        let b = "PRINT 'linia unu\nlinia trei'";
        let c = clasifica(a, b);
        assert!(c.doar_siruri(), "{c:?}");
    }

    #[test]
    fn propozitia_spune_exact_ce_s_a_schimbat_cand_e_un_singur_lucru() {
        // cazul din machete: „un singur caracter, intr-un sir: Nu → nu"
        let a = pregateste_afisare("RAISERROR('Nu s-au gasit date', 16, 1);");
        let b = pregateste_afisare("RAISERROR('nu s-au gasit date', 16, 1);");
        assert_eq!(
            propozitie(&a, &b, &[]),
            "un singur caracter, intr-un sir: Nu → nu"
        );
    }

    #[test]
    fn propozitia_cade_pe_forma_scurta_cand_sunt_mai_multe_diferente() {
        let a = pregateste_afisare("SELECT 1 FROM t\n\nPRINT 'a'");
        let b = pregateste_afisare("SELECT 2 FROM t\n\nPRINT 'b'");
        let p = propozitie(&a, &b, &[]);
        assert!(p.starts_with("difera in "), "{p}");
    }

    #[test]
    fn propozitia_nu_insira_un_bloc_intreg() {
        let a = pregateste_afisare("SELECT 1");
        let b = pregateste_afisare(
            "SELECT 1\n\nUPDATE dbo.Tabel SET Coloana = 'o valoare destul de lunga' WHERE Id = 1",
        );
        let p = propozitie(&a, &b, &[]);
        assert!(p.starts_with("difera in "), "{p}");
    }

    #[test]
    fn propozitia_pomeneste_setarile_cand_doar_ele_difera() {
        let t = pregateste_afisare("CREATE PROC dbo.p AS\nSELECT 1");
        assert_eq!(
            propozitie(&t, &t, &["QUOTED_IDENTIFIER"]),
            "difera in QUOTED_IDENTIFIER"
        );
        assert_eq!(propozitie(&t, &t, &[]), "fara diferente");
    }

    #[test]
    fn eticheta_si_propozitia_urmeaza_limba() {
        let c = Clasificare {
            cod: 1,
            siruri: 2,
            comentarii: 1,
        };
        assert_eq!(eticheta(c, &[]), "cod, siruri, comentariu");
        assert_eq!(
            eticheta_in(c, &["ANSI_NULLS"], Limba::Engleza),
            "code, strings, comment, ANSI_NULLS"
        );
        assert_eq!(
            propozitie_in("SELECT 'Nu'", "SELECT 'nu'", &[], Limba::Engleza),
            "a single character, in a string: Nu \u{2192} nu"
        );
        assert_eq!(
            propozitie_in("SELECT 1", "SELECT 1", &[], Limba::Engleza),
            "no differences"
        );
    }

    #[test]
    fn traducerea_etichetei_da_aceeasi_eticheta() {
        for cod in 0..2 {
            for siruri in 0..3 {
                for comentarii in 0..3 {
                    let c = Clasificare {
                        cod,
                        siruri,
                        comentarii,
                    };
                    for setari in [&[][..], &["QUOTED_IDENTIFIER"][..], &["hinturi"][..]] {
                        assert_eq!(
                            traduce_eticheta(&eticheta(c, setari), Limba::Engleza),
                            eticheta_in(c, setari, Limba::Engleza),
                            "{c:?} {setari:?}"
                        );
                    }
                }
            }
        }
    }
}
