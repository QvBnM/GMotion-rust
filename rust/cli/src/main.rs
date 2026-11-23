//! `gmotion` — binarul headless.
//!
//! **Se strica astfel:** argumentele nu se potrivesc cu ce face comanda, sau
//! programul iese cu 0 cand a esuat. Al doilea e cel rau: o rulare programata
//! ar raporta succes.
//!
//! Exista inaintea interfetei grafice, si ramane dupa ea, ca mod de depanare
//! permanent. Doua lucruri trebuie sa ramana adevarate:
//!
//! 1. **tot ce face aplicatia se poate actiona de aici**, deci orice bug e
//!    reproductibil fara interfata si fara mouse;
//! 2. **fiecare comanda merge si cu `--sursa <instantaneu>`**, nu doar cu un
//!    server, deci un bug de pe o baza de productie se reia pe orice masina.
//!
//! Modulele, fiecare cu categoria lui de bug: [`acreditari`] (parola ajunge
//! unde nu trebuie), [`sursa`] (o comanda nu merge pe instantaneu),
//! [`afisare`] (JSON care nu se parseaza), [`comenzi`] (comanda face altceva
//! decat spune).

/// `println!` care nu da panica pe o teava inchisa (`gmotion compara | head`):
/// `println!` panica, iar cu `panic = "abort"` comanda iesea cu 134 in loc de
/// 0 (auditul 2026-11-22). Toata iesirea standard trece pe aici.
macro_rules! scrie {
    () => {
        $crate::afisare::linie(format_args!(""))
    };
    ($($t:tt)*) => {
        $crate::afisare::linie(format_args!($($t)*))
    };
}

mod acreditari;
mod afisare;
mod comenzi;
mod sursa;

use std::path::PathBuf;

use afisare::Format;
use clap::{Parser, Subcommand, ValueEnum};
use gmotion_modul_diff_obiecte::ModImpartire;
use sursa::ArgumenteSursa;

#[derive(Parser)]
#[command(
    name = "gmotion",
    version,
    about = "Compara obiectele SQL ale mai multor baze modificate fata de un martor si genereaza scripturi de deploy.",
    long_about = "Compara obiectele SQL (proceduri, view-uri, functii, triggere) ale mai multor \
                  baze de date modificate fata de o baza martor, si genereaza scripturile de deploy.\n\n\
                  Orice comanda care citeste obiecte merge atat de pe un server (--server) cat \
                  si dintr-un instantaneu pe disc (--sursa). Asa se reia un bug de pe o baza \
                  reala pe orice masina:\n\n  \
                  gmotion instantaneu --server SRV01 --baze A,B -o bug.json\n  \
                  gmotion compara --sursa bug.json --referinta A\n\n\
                  Parola nu se da ca argument - ar ajunge in istoricul shell-ului si in lista \
                  de procese. Pune-o in GMOTION_PAROLA, sau foloseste --parola-fisier."
)]
struct Argumente {
    /// Scrie rezultatul ca JSON. Mesajele de progres merg atunci la iesirea de
    /// erori, ca iesirea standard sa ramana parsabila.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    comanda: Comanda,
}

#[derive(Clone, Copy, ValueEnum)]
enum ModArg {
    /// Tot intr-un singur fisier.
    Unul,
    /// Un fisier per baza (implicit).
    PerBaza,
    /// Un fisier per obiect.
    PerObiect,
    /// Cate N obiecte per fisier; N vine din --cate.
    CateN,
}

#[derive(Subcommand)]
enum Comanda {
    /// Ce vede GMotion: versiune, cai, configurare, ce e disponibil pe acest
    /// sistem. Prima comanda de rulat cand ceva nu merge.
    Diagnostic,

    /// Cauta instante SQL Server in reteaua locala (SQL Browser, UDP 1434).
    Servere,

    /// Listeaza bazele disponibile.
    Baze {
        #[command(flatten)]
        sursa: ArgumenteSursa,
    },

    /// Testeaza conexiunea la server, ca butonul din fereastra: editia,
    /// versiunea, de cand merge si cate baze are. Doar pe server.
    Testeaza {
        #[command(flatten)]
        sursa: ArgumenteSursa,
    },

    /// Scrie obiectele brute intr-un fisier, ca sa poata fi reluate offline.
    Instantaneu {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        /// Bazele de citit, separate prin virgula. Lipsa lor inseamna toate.
        #[arg(long, value_name = "A,B,C")]
        baze: Option<String>,

        /// Fisierul de iesire.
        #[arg(short = 'o', long = "iesire", alias = "iesire", value_name = "CALE")]
        iesire: PathBuf,

        /// Nota libera: ce bug se urmareste. Ajunge in fisier.
        #[arg(long, default_value = "")]
        nota: String,
    },

    /// Compara bazele modificate cu martorul si arata lista de diferente.
    Compara {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        /// Baza martor: cea fata de care se compara.
        #[arg(long, value_name = "BAZA")]
        referinta: String,

        /// Bazele modificate, de comparat cu martorul. Lipsa lor inseamna
        /// toate celelalte.
        #[arg(long, value_name = "A,B,C")]
        baze: Option<String>,

        /// In ce difera fiecare obiect (cod / siruri / comentariu / setari) si
        /// cate linii se misca. Cere alinierea fiecarui obiect diferit, deci
        /// dureaza mai mult decat compararea simpla.
        #[arg(long)]
        detalii: bool,
    },

    /// Ce ai marcat ca revizuit, pe o comparare. Bifa e legata de continutul
    /// obiectului: daca definitia s-a schimbat pe server, bifa cade singura.
    Revizuire {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        #[arg(long, value_name = "BAZA")]
        referinta: String,

        /// Bazele modificate, de comparat cu martorul. Lipsa lor inseamna
        /// toate celelalte.
        #[arg(long, value_name = "A,B,C")]
        baze: Option<String>,

        /// Marcheaza obiectele astea ca revizuite.
        #[arg(long, value_name = "schema.obiect,...")]
        marcheaza: Option<String>,

        /// Omite obiectele astea: analizate, dar lasate in afara deploy-ului.
        #[arg(long, value_name = "schema.obiect,...")]
        omite: Option<String>,

        /// Scoate bifa (revizuit sau omis) de pe obiectele astea.
        #[arg(long, value_name = "schema.obiect,...")]
        scoate: Option<String>,

        /// Sterge bifele (revizuit si omis) pentru martorul asta. Textele
        /// finale, notele si marcajele de deploy raman. Nu se combina cu
        /// celelalte actiuni.
        #[arg(long, conflicts_with_all = ["marcheaza", "omite", "scoate", "nota", "baze"])]
        goleste: bool,

        /// Obiectul caruia i se pune nota (pe toate bazele unde apare).
        #[arg(long, value_name = "schema.obiect", requires = "text")]
        nota: Option<String>,

        /// Textul notei; gol o sterge.
        #[arg(long, value_name = "TEXT", requires = "nota")]
        text: Option<String>,
    },

    /// Impactul estimat al deploy-ului fiecarui obiect care difera: cine il
    /// foloseste in baza modificata, cat de mare e schimbarea, si nivelul.
    Impact {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        #[arg(long, value_name = "BAZA")]
        referinta: String,

        /// Bazele modificate, de comparat cu martorul. Lipsa lor inseamna
        /// toate celelalte.
        #[arg(long, value_name = "A,B,C")]
        baze: Option<String>,

        /// Doar obiectul asta, cu lista celor care il folosesc.
        #[arg(long, value_name = "SCHEMA.NUME")]
        obiect: Option<String>,
    },

    /// Textul final al unui obiect: aplica diferente din martor, toate,
    /// reset, sau un text scris de mana. Fara actiune, arata diferentele
    /// ramase, numerotate. Se salveaza in sesiune si intra in deploy.
    Editare {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        #[arg(long, value_name = "BAZA")]
        referinta: String,

        /// Baza modificata.
        #[arg(long, value_name = "BAZA")]
        baza: String,

        /// Obiectul, ca `schema.nume`.
        #[arg(long, value_name = "SCHEMA.NUME")]
        obiect: String,

        /// Aplica diferentele cu numerele astea (de la 1, ca in editor).
        #[arg(long, value_name = "1,3", conflicts_with_all = ["aplica_toate", "reset", "din_fisier"])]
        aplica: Option<String>,

        /// Cu `--aplica N` (o singura diferenta): o lasa langa linia asta din
        /// textul curent al bazei modificate (de la 1), nu pe locul sugerat.
        /// Pe o linie dintr-un bloc il inlocuieste; altfel se insereaza
        /// inaintea ei. Ca tragerea in editor.
        #[arg(long, value_name = "LINIE", requires = "aplica")]
        la: Option<usize>,

        /// Textul final devine exact cel din martor.
        #[arg(long, conflicts_with_all = ["reset", "din_fisier"])]
        aplica_toate: bool,

        /// Inapoi la textul bazei modificate.
        #[arg(long, conflicts_with = "din_fisier")]
        reset: bool,

        /// Textul final, scris de mana, din fisierul asta.
        #[arg(long, value_name = "CALE")]
        din_fisier: Option<PathBuf>,

        /// Scrie textul final la iesirea standard, in loc de rezumat.
        #[arg(long)]
        arata_final: bool,

        /// Trimite si textul final (sau al bazei, daca nu e editat) la SQL
        /// Server, fara efecte (`SET PARSEONLY ON`, apoi `SET NOEXEC ON`), in
        /// baza modificata, si spune ce eroare da. Cere un server, nu un
        /// instantaneu.
        #[arg(long, conflicts_with = "arata_final")]
        pe_server: bool,
    },

    /// Arata diferentele unui singur obiect, linie cu linie.
    Diferente {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        #[arg(long, value_name = "BAZA")]
        referinta: String,

        /// Baza modificata in care se uita.
        #[arg(long, value_name = "BAZA")]
        baza: String,

        /// Obiectul, ca `schema.nume`.
        #[arg(long, value_name = "SCHEMA.NUME")]
        obiect: String,
    },

    /// Genereaza scripturile de deploy cu codul din martor.
    Deploy {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        #[arg(long, value_name = "BAZA")]
        referinta: String,

        #[arg(long, value_name = "A,B,C")]
        baze: Option<String>,

        /// Directorul de iesire. Lipsa lui genereaza un nume cu data.
        #[arg(short = 'd', long, value_name = "CALE")]
        director: Option<PathBuf>,

        /// Cum se impart fisierele.
        #[arg(long, value_enum, default_value = "per-baza")]
        mod_fisiere: ModArg,

        /// Cate obiecte per fisier, pentru --mod-fisiere cate-n (implicit 10).
        /// Dat cu alt mod, sau 0, e eroare: inainte se ignora, respectiv
        /// devenea 1 fara sa spuna (F-C7).
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        cate: Option<u64>,

        /// Doar obiectele astea, ca `schema.nume`, separate prin virgula.
        #[arg(long, value_name = "A,B", conflicts_with = "plan")]
        doar: Option<String>,

        /// Planul scris de `gmotion plan`. Inainte de a scrie ceva, obiectele
        /// din plan se recitesc si se compara cu amprentele de atunci; daca
        /// s-a schimbat ceva, deploy-ul se opreste si spune ce (GM-MOD-012).
        #[arg(long, value_name = "PLAN.JSON", conflicts_with = "baze")]
        plan: Option<PathBuf>,
    },

    /// Scrie planul de deploy: ce intra, cu amprentele de acum ale fiecarui
    /// obiect, ca deploy-ul de mai tarziu sa poata verifica ca baza modificata n-a mai
    /// fost atinsa intre timp.
    Plan {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        #[arg(long, value_name = "BAZA")]
        referinta: String,

        #[arg(long, value_name = "A,B,C")]
        baze: Option<String>,

        /// Doar obiectele astea, ca `schema.nume`, separate prin virgula.
        #[arg(long, value_name = "A,B")]
        doar: Option<String>,

        /// Fisierul planului. Lipsa lui genereaza un nume cu data.
        #[arg(short = 'o', long = "iesire", alias = "iesire", value_name = "CALE")]
        iesire: Option<PathBuf>,
    },

    /// Exporta lista de diferente ca CSV, pentru Excel.
    Export {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        #[arg(long, value_name = "BAZA")]
        referinta: String,

        #[arg(long, value_name = "A,B,C")]
        baze: Option<String>,

        /// Fisierul de iesire. Lipsa lui genereaza un nume cu data.
        #[arg(short = 'o', long = "iesire", alias = "iesire", value_name = "CALE")]
        iesire: Option<PathBuf>,
    },
}

fn main() -> std::process::ExitCode {
    gmotion_platforma::jurnal::porneste();
    let a = Argumente::parse();
    let format = Format::din(a.json);

    let rezultat = match &a.comanda {
        Comanda::Diagnostic => comenzi::diagnostic(format),
        Comanda::Servere => comenzi::servere(format),
        Comanda::Baze { sursa } => comenzi::baze(sursa, format),
        Comanda::Testeaza { sursa } => comenzi::testeaza(sursa, format),
        Comanda::Instantaneu {
            sursa,
            baze,
            iesire,
            nota,
        } => comenzi::instantaneu(sursa, baze.as_deref(), iesire, nota, format),
        Comanda::Compara {
            sursa,
            referinta,
            baze,
            detalii,
        } => comenzi::compara(sursa, referinta, baze.as_deref(), *detalii, format),
        Comanda::Revizuire {
            sursa,
            referinta,
            baze,
            marcheaza,
            omite,
            scoate,
            goleste,
            nota,
            text,
        } => comenzi::revizuire(
            sursa,
            referinta,
            baze.as_deref(),
            comenzi::SchimbariRevizuire {
                marcheaza: marcheaza.as_deref(),
                omite: omite.as_deref(),
                scoate: scoate.as_deref(),
                nota: nota.as_deref().zip(text.as_deref()),
            },
            *goleste,
            format,
        ),
        Comanda::Impact {
            sursa,
            referinta,
            baze,
            obiect,
        } => comenzi::impact(sursa, referinta, baze.as_deref(), obiect.as_deref(), format),
        Comanda::Editare {
            sursa,
            referinta,
            baza,
            obiect,
            aplica,
            la,
            aplica_toate,
            reset,
            din_fisier,
            arata_final,
            pe_server,
        } => (|| {
            let actiune = if let Some(a) = aplica {
                let numere = sursa::lista(a)
                    .iter()
                    .map(|n| {
                        n.trim()
                            .parse::<usize>()
                            .map_err(|_| format!("[GM-CLI-001] {n:?} nu e un numar de diferenta"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                // `--aplica ""` (o variabila goala intr-un script) salva tacut un
                // text final egal cu Modificata, care pleca apoi in deploy in
                // locul Martorului (auditul 2026-11-23)
                if numere.is_empty() {
                    return Err(
                        "[GM-CLI-001] --aplica cere cel putin un numar de diferenta".to_string()
                    );
                }
                if la.is_some() && numere.len() != 1 {
                    return Err(
                        "[GM-CLI-001] --la merge cu o singura diferenta (--aplica N)".to_string(),
                    );
                }
                comenzi::ActiuneEditare::Aplica(numere, *la)
            } else if *aplica_toate {
                comenzi::ActiuneEditare::AplicaToate
            } else if *reset {
                comenzi::ActiuneEditare::Reset
            } else if let Some(f) = din_fisier {
                comenzi::ActiuneEditare::DinFisier(f.clone())
            } else {
                comenzi::ActiuneEditare::Arata
            };
            comenzi::editare(
                sursa,
                referinta,
                baza,
                obiect,
                actiune,
                *arata_final,
                *pe_server,
                format,
            )
        })(),
        Comanda::Diferente {
            sursa,
            referinta,
            baza,
            obiect,
        } => comenzi::diferente(sursa, referinta, baza, obiect, format),
        Comanda::Deploy {
            sursa,
            referinta,
            baze,
            director,
            mod_fisiere,
            cate,
            doar,
            plan,
        } => (|| {
            if cate.is_some() && !matches!(mod_fisiere, ModArg::CateN) {
                return Err("[GM-CLI-001] --cate merge doar cu --mod-fisiere cate-n".to_string());
            }
            comenzi::deploy(
                sursa,
                referinta,
                baze.as_deref(),
                director.clone(),
                match mod_fisiere {
                    ModArg::Unul => ModImpartire::Unul,
                    ModArg::PerBaza => ModImpartire::PerBaza,
                    ModArg::PerObiect => ModImpartire::PerObiect,
                    ModArg::CateN => ModImpartire::CateN(
                        usize::try_from(cate.unwrap_or(10)).unwrap_or(usize::MAX),
                    ),
                },
                doar.as_deref(),
                plan.as_deref(),
                format,
            )
        })(),
        Comanda::Plan {
            sursa,
            referinta,
            baze,
            doar,
            iesire,
        } => comenzi::plan(
            sursa,
            referinta,
            baze.as_deref(),
            doar.as_deref(),
            iesire.clone(),
            format,
        ),
        Comanda::Export {
            sursa,
            referinta,
            baze,
            iesire,
        } => comenzi::export(sursa, referinta, baze.as_deref(), iesire.clone(), format),
    };

    match rezultat {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(mesaj) => {
            // Mereu la iesirea de erori, si mereu cod nenul: altfel o rulare
            // programata ar raporta succes.
            eprintln!("Eroare: {mesaj}");
            std::process::ExitCode::FAILURE
        }
    }
}
