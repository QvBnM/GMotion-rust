//! Cum arata rezultatele: pentru om sau pentru masina.
//!
//! **Se strica astfel:** un tabel nealiniat sau un JSON care nu se poate
//! parsa. Al doilea conteaza mai mult decat pare: `--json` e cum se compara
//! doua rulari intre ele la etapa de paritate, si cum citesc eu un rezultat
//! fara sa cer o captura de ecran.
//!
//! Regula: pe `--json`, la ieșirea standard **nu iese nimic in afara de JSON**.
//! Mesajele de progres si avertismentele merg la ieșirea de erori.

use std::io::Write;

use gmotion_modul_diff_obiecte::{Comparare, Rand};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
}

impl Format {
    pub fn din(json: bool) -> Self {
        if json {
            Format::Json
        } else {
            Format::Text
        }
    }

    /// Unde merg mesajele de progres. Pe JSON, la erori: altfel ar strica
    /// ieșirea.
    pub fn progres(self, mesaj: &str) {
        let mesaj = fara_control(mesaj);
        match self {
            Format::Text => println!("{mesaj}"),
            Format::Json => eprintln!("{mesaj}"),
        }
    }
}

/// Caracterele de control (in afara de rand nou) devin `?`. Numele de baze,
/// obiecte si servere vin din baze sau de pe retea; un `ESC[2J` intr-un nume
/// rescria terminalul, iar un rand nou in mijlocul unei celule strica
/// tabelul. Gasit la audit.
pub fn fara_control(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() && c != '\n' { '?' } else { c })
        .collect()
}

/// Ca `fara_control`, dar si randul nou devine `?`: o celula de tabel e pe
/// un singur rand.
fn celula(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { '?' } else { c })
        .collect()
}

/// Un tabel simplu, aliniat pe lațimea conținutului.
///
/// Fara dependinta de un crate de tabele: zece linii de cod fac exact ce
/// trebuie, iar coloanele noastre sunt mereu text scurt.
pub fn tabel(cap: &[&str], randuri: &[Vec<String>]) -> String {
    let randuri: Vec<Vec<String>> = randuri
        .iter()
        .map(|r| r.iter().map(|c| celula(c)).collect())
        .collect();
    let randuri = &randuri;
    let mut lat: Vec<usize> = cap.iter().map(|c| c.chars().count()).collect();
    for r in randuri {
        for (i, c) in r.iter().enumerate() {
            if i < lat.len() {
                lat[i] = lat[i].max(c.chars().count());
            }
        }
    }
    let mut out = String::new();
    let linie = |out: &mut String, celule: &[String]| {
        let n = celule.len();
        for (i, c) in celule.iter().enumerate() {
            let umplutura = lat[i].saturating_sub(c.chars().count());
            out.push_str(c);
            if i + 1 < n {
                out.push_str(&" ".repeat(umplutura));
                out.push_str("  ");
            }
        }
        out.push('\n');
    };
    linie(
        &mut out,
        &cap.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
    );
    linie(
        &mut out,
        &lat.iter().map(|w| "-".repeat(*w)).collect::<Vec<_>>(),
    );
    for r in randuri {
        linie(&mut out, r);
    }
    out
}

fn scor_text(s: Option<f64>) -> String {
    s.map(|x| format!("{x:.2}")).unwrap_or_else(|| "-".into())
}

/// Lista de rezultate, pentru om.
pub fn comparare_text(c: &Comparare) -> String {
    let n = c.numaratoare;
    let mut out = format!(
        "Referinta: {}\nBaze comparate: {}\n\n\
         {} obiecte in referinta: {} identice, {} diferite, {} lipsa, {} doar in tinta\n",
        c.referinta,
        if c.baze.is_empty() {
            "(niciuna)".to_string()
        } else {
            c.baze.join(", ")
        },
        n.total_referinta,
        n.identice,
        n.diferite,
        n.lipsa,
        n.extra
    );
    if n.criptate > 0 {
        out.push_str(&format!(
            "{} criptate (WITH ENCRYPTION): nu se compara, nu se deployeaza\n",
            n.criptate
        ));
    }
    if c.randuri.is_empty() {
        out.push_str("\nNimic de raportat: bazele sunt identice cu referinta.\n");
        return out;
    }
    let randuri: Vec<Vec<String>> = c
        .randuri
        .iter()
        .map(|r| {
            vec![
                r.id.to_string(),
                r.baza.clone(),
                r.obiect.clone(),
                r.tip.clone(),
                r.categorie.eticheta().to_string(),
                scor_text(r.scor),
                r.rezumat.clone(),
            ]
        })
        .collect();
    out.push('\n');
    out.push_str(&tabel(
        &[
            "ID",
            "BAZA",
            "OBIECT",
            "TIP",
            "CATEGORIE",
            "SCOR",
            "DIFERENTE",
        ],
        &randuri,
    ));
    out
}

/// Forma serializata a unei comparari. Tip propriu, nu `Comparare` direct:
/// forma de pe sarma e un contract cu ce citeste JSON-ul, si nu trebuie sa se
/// schimbe de fiecare data cand se schimba un camp intern.
#[derive(serde::Serialize)]
pub struct CompararJson<'a> {
    pub referinta: &'a str,
    pub baze: &'a [String],
    pub total_referinta: usize,
    pub identice: usize,
    pub diferite: usize,
    pub lipsa: usize,
    pub extra: usize,
    pub criptate: usize,
    pub randuri: &'a [Rand],
}

pub fn comparare_json(c: &Comparare) -> CompararJson<'_> {
    CompararJson {
        referinta: &c.referinta,
        baze: &c.baze,
        total_referinta: c.numaratoare.total_referinta,
        identice: c.numaratoare.identice,
        diferite: c.numaratoare.diferite,
        lipsa: c.numaratoare.lipsa,
        extra: c.numaratoare.extra,
        criptate: c.numaratoare.criptate,
        randuri: &c.randuri,
    }
}

/// Scrie un JSON la ieșirea standard. `Result` ca sa nu dea panica pe o
/// conducta inchisa (`| head`).
pub fn scrie_json<T: serde::Serialize>(v: &T) -> std::io::Result<()> {
    let text = serde_json::to_string_pretty(v)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let mut o = std::io::stdout().lock();
    o.write_all(text.as_bytes())?;
    o.write_all(b"\n")
}

#[cfg(test)]
mod teste {
    use super::*;
    use gmotion_modul_diff_obiecte::{Categorie, Numaratoare};

    fn comparare_exemplu() -> Comparare {
        Comparare {
            referinta: "Ref".into(),
            baze: vec!["Client1".into()],
            randuri: vec![
                Rand {
                    id: 0,
                    obiect: "dbo.pLung".into(),
                    baza: "Client1".into(),
                    scor: Some(87.5),
                    rezumat: "3 linii modificate".into(),
                    tip: "P".into(),
                    categorie: Categorie::Diferit,
                    hash_referinta: Some("a".into()),
                    hash_curent: Some("b".into()),
                },
                Rand {
                    id: 1,
                    obiect: "dbo.v".into(),
                    baza: "Client1".into(),
                    scor: None,
                    rezumat: "lipseste complet".into(),
                    tip: "V".into(),
                    categorie: Categorie::Lipsa,
                    hash_referinta: Some("a".into()),
                    hash_curent: None,
                },
            ],
            numaratoare: Numaratoare {
                total_referinta: 5,
                identice: 3,
                diferite: 1,
                lipsa: 1,
                extra: 0,
                criptate: 0,
            },
        }
    }

    #[test]
    fn caracterele_de_control_nu_ajung_in_terminal() {
        let t = tabel(&["N"], &[vec!["a\u{1b}[2Jb\nc".to_string()]]);
        assert!(!t.contains('\u{1b}') && t.lines().count() == 3, "{t:?}");
        assert_eq!(fara_control("x\u{1b}y\nz"), "x?y\nz");
    }

    #[test]
    fn tabelul_se_alinieaza_pe_cel_mai_lung() {
        let t = tabel(
            &["A", "B"],
            &[
                vec!["scurt".into(), "x".into()],
                vec!["mult-mai-lung".into(), "y".into()],
            ],
        );
        let linii: Vec<&str> = t.lines().collect();
        // linia de separatori are lațimea coloanei celei mai lungi
        assert!(linii[1].starts_with(&"-".repeat("mult-mai-lung".len())));
        // prima coloana e umpluta pana la aceeasi lațime
        assert!(linii[2].starts_with("scurt        "));
    }

    #[test]
    fn tabelul_nu_lasa_spatii_la_capatul_randului() {
        let t = tabel(&["A", "B"], &[vec!["x".into(), "y".into()]]);
        for l in t.lines() {
            assert_eq!(l, l.trim_end(), "spatii la capat: {l:?}");
        }
    }

    #[test]
    fn tabelul_numara_caractere_nu_octeti() {
        let t = tabel(&["A"], &[vec!["ăîâșț".into()]]);
        let linii: Vec<&str> = t.lines().collect();
        assert_eq!(
            linii[1].chars().count(),
            5,
            "separatorul trebuie sa aiba 5 caractere, nu 10 octeti: {:?}",
            linii[1]
        );
    }

    #[test]
    fn textul_arata_numaratoarea_si_toate_randurile() {
        let t = comparare_text(&comparare_exemplu());
        assert!(t.contains("Referinta: Ref"));
        assert!(t.contains("5 obiecte in referinta: 3 identice, 1 diferite, 1 lipsa"));
        assert!(t.contains("dbo.pLung"));
        assert!(t.contains("87.50"));
        assert!(t.contains("dbo.v"));
        assert!(t.contains(" - "), "scorul lipsa se arata ca -");
    }

    #[test]
    fn baze_identice_spun_asta_in_loc_de_tabel_gol() {
        let mut c = comparare_exemplu();
        c.randuri.clear();
        let t = comparare_text(&c);
        assert!(t.contains("Nimic de raportat"), "{t}");
        assert!(
            !t.contains("OBIECT"),
            "fara cap de tabel cand n-are randuri"
        );
    }

    #[test]
    fn jsonul_se_parseaza_si_are_campurile_asteptate() {
        let c = comparare_exemplu();
        let text = serde_json::to_string(&comparare_json(&c)).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["referinta"], "Ref");
        assert_eq!(v["identice"], 3);
        assert_eq!(v["randuri"].as_array().unwrap().len(), 2);
        assert_eq!(v["randuri"][0]["obiect"], "dbo.pLung");
        assert_eq!(v["randuri"][0]["categorie"], "diferit");
        assert_eq!(v["randuri"][1]["scor"], serde_json::Value::Null);
    }
}
