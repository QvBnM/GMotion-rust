"""Proba gărzii de versiune din scripturile de deploy, pe un SQL Server real.

Se strica astfel: garda e SQL scris in Rust si nerulat niciodata pe un server
(mediul de dezvoltare n-are SQL Server). O greseala de sintaxa ar face ca
FIECARE script de deploy sa cada la prima linie, sau garda sa nu opreasca
nimic pe un server vechi.

Ce face, doar pe `master`, doar citire (fara USE, fara CREATE):
  1. extrage garda din primul fisier de deploy dat;
  2. o ruleaza ca atare: pe un server >= 13.0.4001 trebuie sa treaca;
  3. o ruleaza cu versiunea inlocuita cu '12.0.6024.0' (SQL Server 2014) si
     editia 3: trebuie sa ridice eroarea „GMotion: ...".

  set GMOTION_PAROLA (parola nu se da ca argument)
  py -3 -I referinta\\proba_garda.py deploy\\fisier.sql --server SRV\\INST --utilizator sa

Cod de iesire 0 daca ambele ramuri se comporta corect.
"""

import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from paritate_pyodbc import importa_app  # noqa: E402


def garda_din(cale):
    with open(cale, encoding="utf-8") as f:
        text = f.read()
    start = text.index("DECLARE @gmotion_versiune")
    sfarsit = text.index("\nGO\n", start)
    return text[start:sfarsit]


def main(argv=None):
    for flux in (sys.stdout, sys.stderr):
        try:
            flux.reconfigure(encoding="utf-8", errors="replace")
        except (AttributeError, ValueError):
            pass
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("fisier_deploy")
    ap.add_argument("--server", required=True)
    ap.add_argument("--utilizator")
    args = ap.parse_args(argv)

    garda = garda_din(args.fisier_deploy)
    app, _ = importa_app()
    import pyodbc

    auth = "sql" if args.utilizator else "windows"
    cs = app.build_cs(args.server, "master", auth, args.utilizator, os.environ.get("GMOTION_PAROLA"))
    ok = True
    with pyodbc.connect(cs, timeout=30, autocommit=True) as conn:
        cur = conn.cursor()
        versiune = cur.execute("SELECT CAST(SERVERPROPERTY('ProductVersion') AS NVARCHAR(128))").fetchone()[0]
        print(f"server: {versiune}")
        try:
            cur.execute(garda)
            while cur.nextset():
                pass
            print("1. garda pe serverul real: trece (corect pentru >= 13.0.4001)")
        except pyodbc.Error as e:
            ok = False
            print(f"1. GRESIT: garda a cazut pe serverul real: {e}")
        cur.execute("SET NOEXEC OFF")

        veche = garda.replace(
            "CAST(SERVERPROPERTY('ProductVersion') AS NVARCHAR(128))", "N'12.0.6024.0'"
        ).replace("CAST(SERVERPROPERTY('EngineEdition') AS INT)", "3")
        try:
            cur.execute(veche)
            while cur.nextset():
                pass
            ok = False
            print("2. GRESIT: pe versiunea 12.0 garda nu a oprit nimic")
        except pyodbc.Error as e:
            mesaj = str(e)
            if "GMotion" in mesaj:
                print("2. garda pe versiunea simulata 12.0: opreste, cu mesajul GMotion (corect)")
            else:
                ok = False
                print(f"2. GRESIT: alta eroare decat cea asteptata: {mesaj}")
        cur.execute("SET NOEXEC OFF")
    print("\nGARDA CORECTA" if ok else "\nGARDA GRESITA")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
