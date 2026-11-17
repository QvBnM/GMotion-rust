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
/// Detaliile unui rand („difera in", `+N −M`), pe id de rand. Se calculeaza
/// doar la cerere (`--detalii`): alinierea fiecarui obiect e scumpa.
pub type Detalii = std::collections::BTreeMap<usize, gmotion_modul_diff_obiecte::Detalii>;

pub fn comparare_text(c: &Comparare, detalii: &Detalii) -> String {
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
            let mut v = vec![
                r.id.to_string(),
                r.baza.clone(),
                r.obiect.clone(),
                r.tip.clone(),
                r.categorie.eticheta().to_string(),
                scor_text(r.scor),
                r.rezumat.clone(),
            ];
            if !detalii.is_empty() {
                let d = detalii.get(&r.id);
                v.push(d.map_or_else(|| "—".to_string(), |d| d.difera_in.clone()));
                v.push(d.map_or_else(String::new, |d| {
                    format!("+{} -{}", d.linii_plus, d.linii_minus)
                }));
            }
            v
        })
        .collect();
    let mut antet = vec![
        "ID",
        "BAZA",
        "OBIECT",
        "TIP",
        "CATEGORIE",
        "SCOR",
        "DIFERENTE",
    ];
    if !detalii.is_empty() {
        antet.push("DIFERA IN");
        antet.push("LINII");
    }
    out.push('\n');
    out.push_str(&tabel(&antet, &randuri));
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
    /// Doar cu `--detalii`: in ce difera fiecare obiect si cate linii se misca.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detalii: Option<Vec<RandDetaliat<'a>>>,
}

/// Detaliile unui rand, in JSON, legate de rand prin `id`.
#[derive(serde::Serialize)]
pub struct RandDetaliat<'a> {
    pub id: usize,
    pub obiect: &'a str,
    pub baza: &'a str,
    pub difera_in: &'a str,
    pub linii_plus: usize,
    pub linii_minus: usize,
}

pub fn comparare_json<'a>(c: &'a Comparare, detalii: &'a Detalii) -> CompararJson<'a> {
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
        detalii: (!detalii.is_empty()).then(|| {
            c.randuri
                .iter()
                .filter_map(|r| {
                    detalii.get(&r.id).map(|d| RandDetaliat {
                        id: r.id,
                        obiect: &r.obiect,
                        baza: &r.baza,
                        difera_in: &d.difera_in,
                        linii_plus: d.linii_plus,
                        linii_minus: d.linii_minus,
                    })
                })
                .collect()
        }),
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
        let t = comparare_text(&comparare_exemplu(), &Detalii::new());
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
        let t = comparare_text(&c, &Detalii::new());
        assert!(t.contains("Nimic de raportat"), "{t}");
        assert!(
            !t.contains("OBIECT"),
            "fara cap de tabel cand n-are randuri"
        );
    }

    #[test]
    fn jsonul_se_parseaza_si_are_campurile_asteptate() {
        let c = comparare_exemplu();
        let text = serde_json::to_string(&comparare_json(&c, &Detalii::new())).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["referinta"], "Ref");
        assert_eq!(v["identice"], 3);
        assert_eq!(v["randuri"].as_array().unwrap().len(), 2);
        assert_eq!(v["randuri"][0]["obiect"], "dbo.pLung");
        assert_eq!(v["randuri"][0]["categorie"], "diferit");
        assert_eq!(v["randuri"][1]["scor"], serde_json::Value::Null);
    }

    fn detalii_exemplu() -> Detalii {
        Detalii::from([(
            0,
            gmotion_modul_diff_obiecte::Detalii {
                difera_in: "un șir".into(),
                linii_plus: 2,
                linii_minus: 1,
            },
        )])
    }

    #[test]
    fn cu_detalii_tabelul_capata_doua_coloane_in_plus() {
        let c = comparare_exemplu();
        let fara = comparare_text(&c, &Detalii::new());
        assert!(!fara.contains("DIFERA IN"));
        let cu = comparare_text(&c, &detalii_exemplu());
        assert!(cu.contains("DIFERA IN") && cu.contains("LINII"), "{cu}");
        assert!(cu.contains("un șir") && cu.contains("+2 -1"), "{cu}");
        // randul fara detalii (lipsa din baza) nu ramane cu coloana goala
        assert!(cu.contains("—"), "{cu}");
    }

    #[test]
    fn jsonul_cu_detalii_le_leaga_de_rand_prin_id() {
        let c = comparare_exemplu();
        let fara: serde_json::Value = serde_json::from_str(
            &serde_json::to_string(&comparare_json(&c, &Detalii::new())).unwrap(),
        )
        .unwrap();
        assert!(
            fara.get("detalii").is_none(),
            "fara --detalii, campul lipseste"
        );
        let cu: serde_json::Value = serde_json::from_str(
            &serde_json::to_string(&comparare_json(&c, &detalii_exemplu())).unwrap(),
        )
        .unwrap();
        let d = cu["detalii"].as_array().unwrap();
        assert_eq!(d.len(), 1);
        assert_eq!(d[0]["id"], 0);
        assert_eq!(d[0]["obiect"], "dbo.pLung");
        assert_eq!(d[0]["difera_in"], "un șir");
        assert_eq!(d[0]["linii_plus"], 2);
    }
}
