//! Tabelele, ca obiecte comparabile: o definitie construita din coloane.
//!
//! **Se strica astfel:** doua tabele la fel ies „diferite" (un tip formatat
//! altfel: `nvarchar(100)` din `max_length` 200) sau doua diferite ies la fel
//! (o lungime sau un `NULL` uitat din definitie). Un tabel devine un
//! `Obiect` de tip `U` cu un `CREATE TABLE` sintetic (etapa 28), o coloana pe linie, ca
//! sa-l compare acelasi motor ca pe proceduri.

use crate::model::Obiect;

/// Tipul `sys.objects.type` al tabelelor utilizatorului.
pub const TIP_TABEL: &str = "U";

/// Coloanele tabelelor utilizatorului, in ordinea din tabel, cu ce trebuie
/// ca sa se scrie tipul: lungime, precizie, scala, NULL, IDENTITY. Plus
/// datele tabelului (ultima modificare, creare), ca la celelalte obiecte.
pub const INTEROGARE_TABELE: &str = "SELECT s.name, t.name, c.name, ty.name, \
     CAST(c.max_length AS INT), CAST(c.precision AS INT), CAST(c.scale AS INT), \
     c.is_nullable, c.is_identity, \
     CONVERT(nvarchar(19), t.modify_date, 120), CONVERT(nvarchar(19), t.create_date, 120) \
     FROM sys.tables t \
     JOIN sys.schemas s ON s.schema_id = t.schema_id \
     JOIN sys.columns c ON c.object_id = t.object_id \
     JOIN sys.types ty ON ty.user_type_id = c.user_type_id \
     WHERE t.is_ms_shipped = 0 \
     ORDER BY s.name, t.name, c.column_id";

/// O coloana, cum vine din `sys.columns` / `sys.types`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coloana {
    pub nume: String,
    /// `sys.types.name`: `varchar`, `decimal`...
    pub tip: String,
    /// `max_length`, in octeti; -1 = `max`.
    pub lungime: i32,
    pub precizie: i32,
    pub scala: i32,
    pub null: bool,
    pub identitate: bool,
}

/// Tipul coloanei cum s-ar scrie intr-un `CREATE TABLE`: `varchar(50)`,
/// `nvarchar(max)`, `decimal(18,2)`, `datetime2(3)`, `int`.
pub fn tip_coloana(c: &Coloana) -> String {
    let t = c.tip.to_ascii_lowercase();
    let lungime = |imparte: i32| {
        if c.lungime < 0 {
            "max".to_string()
        } else {
            (c.lungime / imparte).to_string()
        }
    };
    match t.as_str() {
        "varchar" | "char" | "varbinary" | "binary" => format!("{t}({})", lungime(1)),
        // lungimea e in octeti: doi pe caracter
        "nvarchar" | "nchar" => format!("{t}({})", lungime(2)),
        "decimal" | "numeric" => format!("{t}({},{})", c.precizie, c.scala),
        "datetime2" | "time" | "datetimeoffset" => format!("{t}({})", c.scala),
        // `float(53)` e implicitul; altfel se spune
        "float" if c.precizie != 53 => format!("float({})", c.precizie),
        _ => t,
    }
}

/// Definitia sintetica a unui tabel: `CREATE TABLE [schema].[nume] (`, o
/// coloana pe linie, `)`. O linie pe coloana, ca o coloana lipsa sau
/// schimbata sa fie exact o linie diferita in editor.
pub fn definitie_tabel(schema: &str, nume: &str, coloane: &[Coloana]) -> String {
    let paranteze = |s: &str| format!("[{}]", s.replace(']', "]]"));
    let mut linii = vec![format!(
        "CREATE TABLE {}.{} (",
        paranteze(schema),
        paranteze(nume)
    )];
    let n = coloane.len();
    for (i, c) in coloane.iter().enumerate() {
        linii.push(format!(
            "    {} {}{} {}{}",
            paranteze(&c.nume),
            tip_coloana(c),
            if c.identitate { " IDENTITY" } else { "" },
            if c.null { "NULL" } else { "NOT NULL" },
            if i + 1 < n { "," } else { "" }
        ));
    }
    linii.push(")".to_string());
    linii.join("\n")
}

/// Tabelele, din randurile coloanelor (sortate pe tabel, apoi pe coloana):
/// un `Obiect` de tip `U` pe tabel.
pub fn tabele_din_coloane(
    randuri: impl IntoIterator<Item = (String, String, Coloana, Option<String>, Option<String>)>,
) -> Vec<Obiect> {
    let mut out: Vec<Obiect> = Vec::new();
    let mut coloane: Vec<Coloana> = Vec::new();
    let mut curent: Option<(String, String, Option<String>, Option<String>)> = None;
    let inchide = |cur: Option<(String, String, Option<String>, Option<String>)>,
                   col: &mut Vec<Coloana>,
                   out: &mut Vec<Obiect>| {
        if let Some((schema, nume, modificat, creat)) = cur {
            out.push(Obiect {
                definitie: definitie_tabel(&schema, &nume, col),
                schema,
                nume,
                tip: TIP_TABEL.to_string(),
                setari: None,
                modificat,
                creat,
            });
        }
        col.clear();
    };
    for (schema, nume, c, modificat, creat) in randuri {
        let acelasi = curent
            .as_ref()
            .is_some_and(|(s, n, _, _)| *s == schema && *n == nume);
        if !acelasi {
            inchide(curent.take(), &mut coloane, &mut out);
            curent = Some((schema, nume, modificat, creat));
        }
        coloane.push(c);
    }
    inchide(curent.take(), &mut coloane, &mut out);
    out
}

#[cfg(test)]
mod teste {
    use super::*;

    fn col(nume: &str, tip: &str, lungime: i32, precizie: i32, scala: i32, null: bool) -> Coloana {
        Coloana {
            nume: nume.into(),
            tip: tip.into(),
            lungime,
            precizie,
            scala,
            null,
            identitate: false,
        }
    }

    #[test]
    fn tipurile_se_scriu_ca_intr_un_create_table() {
        assert_eq!(
            tip_coloana(&col("a", "varchar", 50, 0, 0, true)),
            "varchar(50)"
        );
        assert_eq!(
            tip_coloana(&col("a", "nvarchar", 200, 0, 0, true)),
            "nvarchar(100)"
        );
        assert_eq!(
            tip_coloana(&col("a", "nvarchar", -1, 0, 0, true)),
            "nvarchar(max)"
        );
        assert_eq!(
            tip_coloana(&col("a", "decimal", 9, 18, 2, true)),
            "decimal(18,2)"
        );
        assert_eq!(
            tip_coloana(&col("a", "datetime2", 8, 27, 3, true)),
            "datetime2(3)"
        );
        assert_eq!(tip_coloana(&col("a", "INT", 4, 10, 0, true)), "int");
        assert_eq!(tip_coloana(&col("a", "float", 8, 53, 0, true)), "float");
        assert_eq!(tip_coloana(&col("a", "float", 4, 24, 0, true)), "float(24)");
    }

    #[test]
    fn definitia_are_o_coloana_pe_linie() {
        let mut id = col("Id", "int", 4, 10, 0, false);
        id.identitate = true;
        let d = definitie_tabel(
            "dbo",
            "Clienti",
            &[id, col("Nume", "nvarchar", 200, 0, 0, true)],
        );
        assert_eq!(
            d,
            "CREATE TABLE [dbo].[Clienti] (\n    [Id] int IDENTITY NOT NULL,\n    [Nume] nvarchar(100) NULL\n)"
        );
    }

    #[test]
    fn coloanele_se_strang_pe_tabele() {
        let r = |t: &str, c: &str| {
            (
                "dbo".to_string(),
                t.to_string(),
                col(c, "int", 4, 10, 0, false),
                Some("2026-11-22 10:00:00".to_string()),
                None,
            )
        };
        let t = tabele_din_coloane(vec![r("A", "x"), r("A", "y"), r("B", "z")]);
        assert_eq!(t.len(), 2);
        assert_eq!((t[0].nume.as_str(), t[0].tip.as_str()), ("A", "U"));
        assert_eq!(t[0].definitie.lines().count(), 4);
        assert_eq!(t[1].definitie.lines().count(), 3);
        assert_eq!(t[0].modificat.as_deref(), Some("2026-11-22 10:00:00"));
        assert!(tabele_din_coloane(Vec::new()).is_empty());
    }
}
