#!/usr/bin/env python3
"""Instantaneul pentru modul de previzualizare al ferestrei.

Scrie `referinta/instantanee/previzualizare.json`: o referinta si trei baze
comparate, cu obiecte alese anume ca sa acopere tot ce stie motorul —
toate categoriile (identic, modificat, doar in sursa, doar in baza, criptat),
toate tipurile (P, V, FN, IF, TF, TR), obiecte mari, cod scris dezordonat
(majuscule amestecate, tab-uri, spatii la capat, comentarii imbricate,
string-uri cu cuvinte cheie, `GO` in definitie) si dependente intre obiecte.

Se ruleaza explicit; rezultatul se commiteaza, ca aplicatia sa porneasca in
previzualizare fara sa genereze nimic:

    python3 -I unelte/genereaza-previzualizare.py
"""

import json
import pathlib

REFERINTA = "CharismaERP_Referinta"
BAZE = ["CharismaERP_Client1", "CharismaERP_Client2_Prod", "CharismaERP_Client3_Test"]


def obiect(nume, tip, definitie, schema="dbo", setari=None):
    o = {"schema": schema, "nume": nume, "tip": tip, "definitie": definitie}
    if setari is not None:
        o["setari"] = setari
    return o


def proc_mare(nume, variatie):
    """Un obiect de cateva mii de linii: se vede ca lista nu se impiedica."""
    linii = [f"CREATE PROCEDURE dbo.{nume}", "AS", "BEGIN", "    SET NOCOUNT ON;"]
    for i in range(1, 1201):
        val = i * 2 if (variatie and i % 97 == 0) else i
        linii.append(f"    INSERT INTO #t (id, val) VALUES ({i}, {val});")
    linii += ["END"]
    return "\n".join(linii)


def dezordonat(variatie):
    """Acelasi cod, scris cum il scrie lumea: majuscule amestecate, tab-uri,
    spatii la capat de linie, comentarii imbricate, string-uri cu cuvinte
    cheie si un `GO` in interiorul definitiei."""
    coada = "AND l.sters = 0" if variatie else "AND l.sters = 0 AND l.vizibil = 1"
    return (
        "CREATE   proc dbo.uspRaportLunar   \n"
        "\t@luna INT,\n"
        "\t@an   int\n"
        "as\n"
        "BEGIN\n"
        "\t/* raportul lunar\n"
        "\t   /* nota: comentariu imbricat, ca in codul vechi */\n"
        "\t*/\n"
        "\tDECLARE @mesaj NVARCHAR(200) = N'nu stergeti: DROP TABLE nu e aici, e doar text';\n"
        "\tSELECT  l.id ,  l.valoare,   -- spatii la capat mai jos\n"
        "\t        l.data   \n"
        "\tFROM dbo.Linii AS l\n"
        f"\tWHERE l.luna = @luna AND l.an = @an {coada}\n"
        "\tORDER BY l.data DESC;\n"
        "\tPRINT 'GO nu inseamna sfarsit de lot cand e in string';\n"
        "END\n"
        "GO\n"
    )


def cursor_complet(variatie):
    """Un ciclu de cursor intreg, scris cum il scrie lumea: cuvinte cheie cu
    majuscule amestecate, indentare inconsecventa, linii lipite, `WHILE`
    imbricate si un `IF/ELSE`. Pe el se vede si butonul
    „Uneste ciclul cursor"."""
    prag = "100" if variatie else "250"
    return (
        "CREATE PROCEDURE dbo.uspReproceseazaFacturi\n"
        "AS\n"
        "begin\n"
        "\tSET NOCOUNT ON;\n"
        "\tDECLARE @id INT, @val DECIMAL(18,2), @nr int = 0;\n"
        "\tdeclare curFacturi CURSOR local fast_forward FOR\n"
        "\t\tSELECT f.id, f.valoare FROM dbo.Facturi AS f WHERE f.anulata = 0;\n"
        "\tOpen curFacturi;\n"
        "\tFETCH NEXT FROM curFacturi INTO @id, @val;\n"
        "\tWHile @@FETCH_STATUS = 0\n"
        "\tBEGIN\n"
        f"\t\tIF @val > {prag}\n"
        "\t\tBEGIN\n"
        "\t\t\tDECLARE @i INT = 0;\n"
        "\t\t\twhile @i < 3 begin\n"
        "\t\t\t\tUPDATE dbo.Facturi SET recalculat = 1 WHERE id = @id;   \n"
        "\t\t\t\tSET @i = @i + 1;\n"
        "\t\t\tend\n"
        "\t\tEND\n"
        "\t\tELSE\n"
        "\t\t\tPRINT 'sub prag: nu se face nimic, nici DELETE nici DROP';\n"
        "\t\tSET @nr = @nr + 1;\n"
        "\t\tFETCH next FROM curFacturi INTO @id, @val;\n"
        "\tEND\n"
        "\tclose curFacturi;\n"
        "\tDEALLOCATE curFacturi;\n"
        "\tSELECT @nr AS procesate;\n"
        "END\n"
    )


def tabel(nume, coloane):
    """Un tabel cum il construieste `gmotion_db::tabele::definitie_tabel`
    (etapa 28): o coloana pe linie, `[nume] tip [IDENTITY] NULL|NOT NULL`."""
    linii = [f"CREATE TABLE [dbo].[{nume}] ("]
    for k, (c, tip, null) in enumerate(coloane):
        virgula = "," if k + 1 < len(coloane) else ""
        linii.append(f"    [{c}] {tip} {'NULL' if null else 'NOT NULL'}{virgula}")
    linii.append(")")
    return obiect(nume, "U", "\n".join(linii))


TABELE_REFERINTA = {
    "Clienti": [("id", "int IDENTITY", False), ("nume", "nvarchar(100)", False), ("oras", "nvarchar(50)", True), ("activ", "bit", False)],
    "Facturi": [("id", "int IDENTITY", False), ("client", "int", False), ("valoare", "decimal(18,2)", False), ("anulata", "bit", False)],
    "Stoc": [("produs", "int", False), ("cantitate", "decimal(18,3)", False)],
    "Audit": [("id", "bigint IDENTITY", False), ("cand", "datetime2(3)", False), ("ce", "nvarchar(max)", True)],
}


def referinta():
    return [tabel(k, v) for k, v in TABELE_REFERINTA.items()] + [
        # identic in toate bazele
        obiect("uspListaClienti", "P", "CREATE PROCEDURE dbo.uspListaClienti\nAS\nSELECT id, nume FROM dbo.Clienti WHERE activ = 1;"),
        # modificat: diferenta mica, scor mare
        obiect("uspFactura", "P", "CREATE PROCEDURE dbo.uspFactura @id INT\nAS\nSELECT * FROM dbo.Facturi WHERE id = @id AND anulata = 0;"),
        # modificat: rescris, scor mic
        obiect("uspSoldClient", "P", "CREATE PROCEDURE dbo.uspSoldClient @client INT\nAS\nBEGIN\n    SELECT SUM(valoare) FROM dbo.Facturi WHERE client = @client;\nEND"),
        # cod dezordonat
        obiect("uspRaportLunar", "P", dezordonat(False)),
        # ciclu de cursor intreg, cu WHILE imbricat si IF/ELSE
        obiect("uspReproceseazaFacturi", "P", cursor_complet(False)),
        # obiect mare
        obiect("uspIncarcaTot", "P", proc_mare("uspIncarcaTot", False)),
        # doar in referinta (lipsa in baze)
        obiect("uspNouDinSprint", "P", "CREATE PROCEDURE dbo.uspNouDinSprint\nAS\nSELECT 1;"),
        # tipuri
        obiect("vwClientiActivi", "V", "CREATE VIEW dbo.vwClientiActivi\nAS\nSELECT id, nume FROM dbo.Clienti WHERE activ = 1;"),
        obiect("fnTVA", "FN", "CREATE FUNCTION dbo.fnTVA(@v DECIMAL(18,2))\nRETURNS DECIMAL(18,2)\nAS\nBEGIN\n    RETURN @v * 0.19;\nEND"),
        obiect("ifClientiDupaAn", "IF", "CREATE FUNCTION dbo.ifClientiDupaAn(@an INT)\nRETURNS TABLE\nAS\nRETURN (SELECT id FROM dbo.Clienti WHERE YEAR(creat) = @an);"),
        obiect("tfLiniiFactura", "TF", "CREATE FUNCTION dbo.tfLiniiFactura(@f INT)\nRETURNS @r TABLE (id INT)\nAS\nBEGIN\n    INSERT INTO @r SELECT id FROM dbo.Linii WHERE factura = @f;\n    RETURN;\nEND"),
        obiect("trgFacturiAudit", "TR", "CREATE TRIGGER dbo.trgFacturiAudit ON dbo.Facturi\nAFTER UPDATE\nAS\nINSERT INTO dbo.Audit (tabela, cand) VALUES ('Facturi', GETDATE());"),
        # dependente: uspRaport cheama fnTVA si vwClientiActivi
        obiect("uspRaportTVA", "P", "CREATE PROCEDURE dbo.uspRaportTVA\nAS\nSELECT c.nume, dbo.fnTVA(f.valoare) AS tva\nFROM dbo.vwClientiActivi AS c\nJOIN dbo.Facturi AS f ON f.client = c.id;"),
        # criptat in referinta
        obiect("uspSecret", "P", ""),
        # alta schema, si setari diferite
        obiect("uspVanzari", "P", "CREATE PROCEDURE vanzari.uspVanzari\nAS\nSELECT 1;", schema="vanzari",
               setari={"ansi_nulls": True, "quoted_identifier": True}),
    ]


def tabele_baza(i):
    """Tabelele unei baze: identice, cu o coloana lipsa, cu un tip schimbat,
    cu NULL schimbat, cu o coloana in plus; Audit lipseste dintr-una, iar o
    baza are un tabel in plus (etapa 28)."""
    t = {k: list(v) for k, v in TABELE_REFERINTA.items()}
    if i == 0:
        t["Clienti"] = [c for c in t["Clienti"] if c[0] != "oras"]  # coloana lipsa
        t["Facturi"][2] = ("valoare", "decimal(18,4)", False)  # tip schimbat
    if i == 1:
        t["Clienti"][1] = ("nume", "nvarchar(100)", True)  # NULL schimbat
        t["Stoc"] = t["Stoc"] + [("rezervat", "decimal(18,3)", True)]  # coloana in plus
        del t["Audit"]  # tabel doar in Martor
        t["ImportTemp"] = [("linie", "nvarchar(4000)", True)]  # tabel doar aici
    return [tabel(k, v) for k, v in t.items()]


def baza(i, nume):
    """Fiecare baza difera altfel fata de referinta."""
    o = tabele_baza(i)
    for x in referinta():
        if x["tip"] == "U":
            continue  # tabelele vin din `tabele_baza`
        if x["nume"] == "uspNouDinSprint":
            continue  # lipseste: exista doar in referinta
        if x["nume"] == "uspFactura":
            x = dict(x, definitie=x["definitie"].replace("anulata = 0", "anulata = 0 AND stearsa = 0"))
        if x["nume"] == "uspSoldClient" and i > 0:
            x = dict(x, definitie="CREATE PROCEDURE dbo.uspSoldClient @client INT\nAS\nBEGIN\n    DECLARE @s DECIMAL(18,2);\n    SELECT @s = SUM(f.valoare)\n    FROM dbo.Facturi AS f\n    WHERE f.client = @client AND f.anulata = 0;\n    SELECT ISNULL(@s, 0);\nEND")
        if x["nume"] == "uspRaportLunar":
            x = dict(x, definitie=dezordonat(True))
        if x["nume"] == "uspReproceseazaFacturi" and i != 1:
            x = dict(x, definitie=cursor_complet(True))
        if x["nume"] == "uspIncarcaTot" and i == 0:
            x = dict(x, definitie=proc_mare("uspIncarcaTot", True))
        if x["nume"] == "vwClientiActivi" and i == 1:
            x = dict(x, definitie="CREATE VIEW dbo.vwClientiActivi\nAS\nSELECT id, nume, oras FROM dbo.Clienti WHERE activ = 1;")
        if x["nume"] == "trgFacturiAudit" and i == 2:
            x = dict(x, definitie=x["definitie"].replace("AFTER UPDATE", "AFTER UPDATE, INSERT"))
        if x["nume"] == "uspVanzari" and i == 1:
            # aceeasi definitie, alte setari: diferenta „doar SET"
            x = dict(x, setari={"ansi_nulls": True, "quoted_identifier": False})
        o.append(x)
    # extra: exista doar in baza comparata
    o.append(obiect(f"uspLocal{i + 1}", "P", f"CREATE PROCEDURE dbo.uspLocal{i + 1}\nAS\nSELECT '{nume}';"))
    if i == 1:
        o.append(obiect("uspPatchUrgent", "P", "CREATE PROCEDURE dbo.uspPatchUrgent\nAS\nSELECT 'pus de mana in productie';"))
    return o


def main():
    radacina = pathlib.Path(__file__).resolve().parent.parent
    cale = radacina / "referinta" / "instantanee" / "previzualizare.json"
    date = {
        "versiune": 1,
        "server": "SRV-PREVIZUALIZARE",
        "creat": "2026-11-18T12:00:00Z",
        "nota": "instantaneu sintetic pentru `gmotion-ui --previzualizare`; generat de unelte/genereaza-previzualizare.py",
        "baze": {REFERINTA: referinta(), **{n: baza(i, n) for i, n in enumerate(BAZE)}},
    }
    # ultima modificare (etapa 25), deterministica: aceeasi la fiecare generare
    for b, (nume_baza, obiecte) in enumerate(date["baze"].items()):
        for k, o in enumerate(obiecte):
            luna, zi = 1 + (k * 7 + b) % 11, 1 + (k * 13 + b * 5) % 28
            ora, minut = 8 + (k * 3 + b) % 10, (k * 17 + b * 11) % 60
            o["modificat"] = f"2026-{luna:02}-{zi:02} {ora:02}:{minut:02}:00"
            # creat inainte de modificare: unele in 2026 (recente), restul in 2025
            o["creat"] = f"{2025 + (k % 3 == 0)}-{max(1, luna - 1):02}-{zi:02} 08:00:00"
    cale.write_text(json.dumps(date, indent=1, ensure_ascii=True) + "\n", encoding="utf-8")
    obiecte = sum(len(v) for v in date["baze"].values())
    print(f"scris {cale} ({len(date['baze'])} baze, {obiecte} obiecte)")


if __name__ == "__main__":
    main()
