//! `gmotion` — binarul headless.
//!
//! **Se strica astfel:** argumentele nu se potrivesc cu ce face comanda, sau
//! programul iese cu 0 cand a eșuat. Al doilea e cel rau: o rulare programata
//! ar raporta succes.
//!
//! Exista inaintea interfetei grafice, si rămâne dupa ea, ca mod de depanare
//! permanent. Doua lucruri trebuie sa rămână adevarate:
//!
//! 1. **tot ce face aplicatia se poate acționa de aici**, deci orice bug e
//!    reproductibil fara interfata si fara mouse;
//! 2. **fiecare comanda merge si cu `--sursa <instantaneu>`**, nu doar cu un
//!    server, deci un bug de pe o baza de productie se reia pe orice masina.
//!
//! Modulele, fiecare cu categoria lui de bug: [`acreditari`] (parola ajunge
//! unde nu trebuie), [`sursa`] (o comanda nu merge pe instantaneu),
//! [`afisare`] (JSON care nu se parseaza), [`comenzi`] (comanda face altceva
//! decat spune).

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
    about = "Compara obiectele SQL ale mai multor baze fata de o referinta si genereaza scripturi de deploy.",
    long_about = "Compara obiectele SQL (proceduri, view-uri, functii, triggere) ale mai multor \
                  baze de date fata de o baza de referinta, si genereaza scripturile de deploy.\n\n\
                  Orice comanda care citeste obiecte merge atat de pe un server (--server) cat \
                  si dintr-un instantaneu pe disc (--sursa). Asa se reia un bug de pe o baza \
                  reala pe orice masina:\n\n  \
                  gmotion instantaneu --server SRV01 --baze A,B -o bug.json\n  \
                  gmotion compara --sursa bug.json --referinta A\n\n\
                  Parola nu se da ca argument - ar ajunge in istoricul shell-ului si in lista \
                  de procese. Pune-o in GMOTION_PAROLA, sau foloseste --parola-fisier."
)]
struct Argumente {
    /// Scrie rezultatul ca JSON. Mesajele de progres merg atunci la ieșirea de
    /// erori, ca ieșirea standard sa rămână parsabila.
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

    /// Caută instante SQL Server in reteaua locala (SQL Browser, UDP 1434).
    Servere,

    /// Listeaza bazele disponibile.
    Baze {
        #[command(flatten)]
        sursa: ArgumenteSursa,
    },

    /// Scrie obiectele brute intr-un fisier, ca sa poată fi reluate offline.
    Instantaneu {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        /// Bazele de citit, separate prin virgula. Lipsa lor inseamna toate.
        #[arg(long, value_name = "A,B,C")]
        baze: Option<String>,

        /// Fisierul de ieșire.
        #[arg(short = 'o', long = "ieșire", alias = "iesire", value_name = "CALE")]
        ieșire: PathBuf,

        /// Nota libera: ce bug se urmarește. Ajunge in fisier.
        #[arg(long, default_value = "")]
        nota: String,
    },

    /// Compara bazele cu referinta si arata lista de diferente.
    Compara {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        /// Baza de referinta.
        #[arg(long, value_name = "BAZA")]
        referinta: String,

        /// Bazele de comparat. Lipsa lor inseamna toate celelalte.
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

        /// Bazele de comparat. Lipsa lor inseamna toate celelalte.
        #[arg(long, value_name = "A,B,C")]
        baze: Option<String>,

        /// Marcheaza obiectele astea ca revizuite.
        #[arg(long, value_name = "schema.obiect,...")]
        marcheaza: Option<String>,

        /// Scoate bifa de pe obiectele astea.
        #[arg(long, value_name = "schema.obiect,...")]
        scoate: Option<String>,

        /// Sterge tot progresul salvat pentru referinta asta.
        #[arg(long)]
        goleste: bool,
    },

    /// Arata diferentele unui singur obiect, linie cu linie.
    Diferente {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        #[arg(long, value_name = "BAZA")]
        referinta: String,

        /// Baza in care se uita.
        #[arg(long, value_name = "BAZA")]
        baza: String,

        /// Obiectul, ca `schema.nume`.
        #[arg(long, value_name = "SCHEMA.NUME")]
        obiect: String,
    },

    /// Genereaza scripturile de deploy cu codul din referinta.
    Deploy {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        #[arg(long, value_name = "BAZA")]
        referinta: String,

        #[arg(long, value_name = "A,B,C")]
        baze: Option<String>,

        /// Directorul de ieșire. Lipsa lui genereaza un nume cu data.
        #[arg(short = 'd', long, value_name = "CALE")]
        director: Option<PathBuf>,

        /// Cum se impart fisierele.
        #[arg(long, value_enum, default_value = "per-baza")]
        mod_fisiere: ModArg,

        /// Cate obiecte per fisier, pentru --mod-fisiere cate-n.
        #[arg(long, default_value = "10")]
        cate: usize,

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
    /// obiect, ca deploy-ul de mai tarziu sa poata verifica ca baza n-a mai
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
        #[arg(short = 'o', long = "ieșire", alias = "iesire", value_name = "CALE")]
        ieșire: Option<PathBuf>,
    },

    /// Exporta lista de diferente ca CSV, pentru Excel.
    Export {
        #[command(flatten)]
        sursa: ArgumenteSursa,

        #[arg(long, value_name = "BAZA")]
        referinta: String,

        #[arg(long, value_name = "A,B,C")]
        baze: Option<String>,

        /// Fisierul de ieșire. Lipsa lui genereaza un nume cu data.
        #[arg(short = 'o', long = "ieșire", alias = "iesire", value_name = "CALE")]
        ieșire: Option<PathBuf>,
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
        Comanda::Instantaneu {
            sursa,
            baze,
            ieșire,
            nota,
        } => comenzi::instantaneu(sursa, baze.as_deref(), ieșire, nota, format),
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
            scoate,
            goleste,
        } => comenzi::revizuire(
            sursa,
            referinta,
            baze.as_deref(),
            marcheaza.as_deref(),
            scoate.as_deref(),
            *goleste,
            format,
        ),
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
        } => comenzi::deploy(
            sursa,
            referinta,
            baze.as_deref(),
            director.clone(),
            match mod_fisiere {
                ModArg::Unul => ModImpartire::Unul,
                ModArg::PerBaza => ModImpartire::PerBaza,
                ModArg::PerObiect => ModImpartire::PerObiect,
                ModArg::CateN => ModImpartire::CateN(*cate),
            },
            doar.as_deref(),
            plan.as_deref(),
            format,
        ),
        Comanda::Plan {
            sursa,
            referinta,
            baze,
            doar,
            ieșire,
        } => comenzi::plan(
            sursa,
            referinta,
            baze.as_deref(),
            doar.as_deref(),
            ieșire.clone(),
            format,
        ),
        Comanda::Export {
            sursa,
            referinta,
            baze,
            ieșire,
        } => comenzi::export(sursa, referinta, baze.as_deref(), ieșire.clone(), format),
    };

    match rezultat {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(mesaj) => {
            // Mereu la ieșirea de erori, si mereu cod nenul: altfel o rulare
            // programata ar raporta succes.
            eprintln!("Eroare: {mesaj}");
            std::process::ExitCode::FAILURE
        }
    }
}
