"""Paritate: lista de obiecte din Python v3.4 contra `gmotion compara`, pe acelasi instantaneu.

Se strica astfel: raporteaza paritate cand nu e (compara campuri greșite, sau
ruleaza alt cod Python decat cel din productie), ori raporteaza regresii care
vin din script, nu din Rust. De-aia Python-ul NU e reimplementat aici: se
ruleaza `Api._comparatie` din `mostenire/app.py` neatins, iar singurul lucru
inlocuit e `pyodbc`, cu un modul care serveste randurile din instantaneu.

Fiecare diferenta se clasifica:
  corectura  rezumatul difera, iar motorul JS din editor da exact ce da Rust-ul
             (portul urmeaza JS, vezi JURNAL.md "Portul Rust urmeaza semantica JS")
  regresie   orice altceva: rand lipsa intr-o parte, categorie, scor sau tip
             diferit, rezumat pe care nici JS nu-l confirma

Rulare, din radacina repo-ului:
  python3 -I referinta/paritate.py instantaneu.json --referinta Ref --rust rust.json
  python3 -I referinta/paritate.py instantaneu.json --referinta Ref --gmotion rust/target/release/gmotion
  python3 -I referinta/paritate.py --din-cazuri --gmotion rust/target/debug/gmotion
Optional: --csv export.csv (exportul aplicatiei Python pe aceleasi baze, ca sa
se vada ca pyodbc si tiberius aduc aceleasi obiecte), --json raport.json.

Cod de ieșire: 0 fara regresii, 1 cu regresii, 2 intrare invalida.
"""

import argparse
import csv
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import types

AICI = os.path.dirname(os.path.abspath(__file__))
RADACINA = os.path.dirname(AICI)
MOSTENIRE = os.path.join(RADACINA, "mostenire")

# Categoriile Python -> cele din `gmotion compara --json`.
CATEGORII = {"dif": "diferit", "lipsa": "lipsa", "extra": "extra"}


class EroareIntrare(Exception):
    pass


# ------------------------------------------------------------------ instantaneu


def citeste_instantaneu(cale):
    try:
        with open(cale, encoding="utf-8") as f:
            inst = json.load(f)
    except (OSError, ValueError) as e:
        raise EroareIntrare(f"nu pot citi instantaneul {cale}: {e}")
    if inst.get("versiune") != 1:
        raise EroareIntrare(f"{cale}: versiune de instantaneu {inst.get('versiune')!r}, astept 1")
    return inst


def instantaneu_din_cazuri():
    """Instantaneu sintetic: fiecare caz de conformitate e un obiect, `ref` in
    baza Referinta si `cur` in baza Tinta. Plus un obiect doar in referinta si
    unul doar in tinta, ca sa treaca si pe ramurile `lipsa` si `extra`."""
    with open(os.path.join(AICI, "cazuri.json"), encoding="utf-8") as f:
        cazuri = json.load(f)

    def ob(nume, definitie, tip="P"):
        return {"schema": "dbo", "nume": nume, "tip": tip, "definitie": definitie}

    ref = [ob(c["nume"], c["ref"]) for c in cazuri]
    cur = [ob(c["nume"], c["cur"]) for c in cazuri]
    ref.append(ob("doar_in_referinta", "CREATE VIEW dbo.doar_in_referinta AS SELECT 1 AS x", "V"))
    cur.append(ob("doar_in_tinta", "CREATE FUNCTION dbo.doar_in_tinta() RETURNS INT AS BEGIN RETURN 1 END", "FN"))
    return {
        "versiune": 1,
        "server": "sintetic",
        "creat": "-",
        "nota": "referinta/cazuri.json",
        "baze": {"Referinta": ref, "Tinta": cur},
    }


# ------------------------------------------------------------------ Python v3.4


def _pyodbc_fals(inst):
    """Un `pyodbc` care raspunde la interogarea din `aduce_obiecte` cu obiectele
    bazei numite in sirul de conectare. Filtrul pe tip e cel din interogare."""
    mod = types.ModuleType("pyodbc")

    class Cursor:
        def __init__(self, randuri):
            self._randuri = randuri

        def execute(self, _sql):
            return self

        def fetchall(self):
            return self._randuri

    class Conexiune:
        def __init__(self, baza):
            self._baza = baza

        def __enter__(self):
            return self

        def __exit__(self, *_):
            return False

        def cursor(self):
            import motor  # cel din mostenire/, pus in sys.path inainte

            obiecte = inst["baze"].get(self._baza)
            if obiecte is None:
                raise RuntimeError(f"baza {self._baza!r} nu e in instantaneu")
            return Cursor([
                (o["schema"], o["nume"], o["tip"], o["definitie"])
                for o in obiecte
                if o["tip"].strip() in motor.TIPURI_OBIECTE
            ])

    def connect(cs, timeout=None):
        m = re.search(r"DATABASE=([^;]*);", cs)
        if not m:
            raise RuntimeError(f"sir de conectare fara DATABASE: {cs!r}")
        return Conexiune(m.group(1))

    mod.drivers = lambda: ["ODBC Driver 18 for SQL Server"]
    mod.connect = connect
    return mod


def lista_python(inst, referinta):
    """Ruleaza `Api._comparatie` din mostenire/app.py. Intoarce (meta, lista)."""
    sys.dont_write_bytecode = True  # nimic nou in mostenire/
    sys.modules["pyodbc"] = _pyodbc_fals(inst)
    sys.modules["webview"] = types.ModuleType("webview")
    if MOSTENIRE not in sys.path:
        sys.path.insert(0, MOSTENIRE)
    import app  # noqa: E402  (dupa inlocuirea modulelor)

    app.DEBUG_LIMIT = None
    api = app.Api()
    baze = [referinta] + sorted(b for b in inst["baze"] if b != referinta)
    api._comparatie(api._gen, inst.get("server", ""), "sql", "u", "p", baze, referinta)
    if api._prog.get("eroare"):
        raise EroareIntrare(f"Python a esuat: {api._prog['eroare']}")
    return api._meta, api._lista


# ------------------------------------------------------------------ Rust


def lista_rust(args, cale_inst, referinta):
    if args.rust:
        try:
            with open(args.rust, encoding="utf-8") as f:
                return json.load(f)
        except (OSError, ValueError) as e:
            raise EroareIntrare(f"nu pot citi {args.rust}: {e}")
    cmd = [args.gmotion, "compara", "--sursa", cale_inst, "--referinta", referinta, "--json"]
    p = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8")
    if p.returncode != 0:
        raise EroareIntrare(f"gmotion a iesit cu {p.returncode}:\n{p.stderr.strip()}")
    try:
        return json.loads(p.stdout)
    except ValueError as e:
        raise EroareIntrare(f"gmotion --json nu a scos JSON valid: {e}")


# ------------------------------------------------------------------ arbitru JS


def rezumate_js(perechi):
    """Rezumatul motorului JS pentru fiecare pereche (ref, cur), sau None daca
    node lipseste — caz in care nimic nu poate fi clasificat `corectura`."""
    if not perechi:
        return []
    node = shutil.which("node")
    if not node:
        return None
    p = subprocess.run(
        [node, os.path.join(AICI, "rezumat_js.mjs")],
        input=json.dumps(perechi), capture_output=True, text=True, encoding="utf-8",
    )
    if p.returncode != 0:
        raise EroareIntrare(f"motorul JS a esuat:\n{p.stderr.strip()}")
    return json.loads(p.stdout)


# ------------------------------------------------------------------ comparatie


def compara(inst, referinta, meta_py, lista_py, rust):
    definitii = {
        (b, f"{o['schema']}.{o['nume']}"): o["definitie"]
        for b, obs in inst["baze"].items()
        for o in obs
    }
    py = {(x["obiect"], x["baza"]): x for x in lista_py}
    rs = {(x["obiect"], x["baza"]): x for x in rust["randuri"]}
    if len(py) != len(lista_py) or len(rs) != len(rust["randuri"]):
        raise EroareIntrare("perechi (obiect, baza) duplicate intr-una din liste")

    regresii, corecturi, de_arbitrat = [], [], []

    def regresie(obiect, baza, camp, v_py, v_rs):
        regresii.append({"obiect": obiect, "baza": baza, "camp": camp, "python": v_py, "rust": v_rs})

    for cheie in sorted(set(py) | set(rs)):
        obiect, baza = cheie
        a, b = py.get(cheie), rs.get(cheie)
        if a is None:
            regresie(obiect, baza, "rand", None, b["categorie"])
            continue
        if b is None:
            regresie(obiect, baza, "rand", CATEGORII[a["cat"]], None)
            continue
        if CATEGORII[a["cat"]] != b["categorie"]:
            regresie(obiect, baza, "categorie", CATEGORII[a["cat"]], b["categorie"])
            continue
        if (a["tip"] or "").strip() != b["tip"]:
            regresie(obiect, baza, "tip", a["tip"], b["tip"])
        sa, sb = a["scor"], b["scor"]
        if (sa is None) != (sb is None) or (sa is not None and abs(sa - sb) > 0.005):
            regresie(obiect, baza, "scor", sa, sb)
        if a["rezumat"] == b["rezumat"]:
            continue
        if b["categorie"] == "diferit":
            de_arbitrat.append((obiect, baza, a["rezumat"], b["rezumat"]))
        else:
            # `lipsa` / `extra` au un text fix, iar JS nu are ce compara.
            regresie(obiect, baza, "rezumat", a["rezumat"], b["rezumat"])

    perechi = [[definitii[(referinta, o)], definitii[(bz, o)]] for o, bz, _, _ in de_arbitrat]
    js = rezumate_js(perechi)
    for i, (obiect, baza, r_py, r_rs) in enumerate(de_arbitrat):
        r_js = None if js is None else js[i]
        intrare = {"obiect": obiect, "baza": baza, "camp": "rezumat",
                   "python": r_py, "rust": r_rs, "js": r_js}
        (corecturi if r_js == r_rs else regresii).append(intrare)

    numaratoare = {
        "identice": (meta_py["ident"], rust["identice"]),
        "lipsa": (meta_py["lipsa"], rust["lipsa"]),
        "extra": (meta_py["extra"], rust["extra"]),
        "total_referinta": (meta_py["total"], rust["total_referinta"]),
    }
    for camp, (v_py, v_rs) in numaratoare.items():
        if v_py != v_rs:
            regresie("*", "*", f"numar {camp}", v_py, v_rs)

    return {
        "randuri_python": len(py),
        "randuri_rust": len(rs),
        "numaratoare": {k: {"python": a, "rust": b} for k, (a, b) in numaratoare.items()},
        "regresii": regresii,
        "corecturi": corecturi,
        "js_disponibil": js is not None,
    }


def compara_csv(cale, lista_py):
    """Exportul aplicatiei Python de pe Windows contra Python-ului rulat aici pe
    instantaneu. O diferenta inseamna ca driverele au adus obiecte diferite (sau
    ca exportul a fost facut cu un filtru activ in interfata)."""
    try:
        with open(cale, encoding="utf-8-sig", newline="") as f:
            randuri = list(csv.DictReader(f, delimiter=";"))
    except OSError as e:
        raise EroareIntrare(f"nu pot citi {cale}: {e}")
    if randuri and "Obiect" not in randuri[0]:
        raise EroareIntrare(f"{cale}: nu are antetul exportului din aplicatia Python")
    din_csv = {(r["Obiect"], r["Baza"]): (r["Categorie"], r["Similaritate %"]) for r in randuri}
    aici = {(x["obiect"], x["baza"]): (x["cat"], "" if x["scor"] is None else str(x["scor"]))
            for x in lista_py}
    diferente = []
    for cheie in sorted(set(din_csv) | set(aici)):
        if din_csv.get(cheie) != aici.get(cheie):
            diferente.append({"obiect": cheie[0], "baza": cheie[1],
                              "csv": din_csv.get(cheie), "instantaneu": aici.get(cheie)})
    return diferente


# ------------------------------------------------------------------ raport


def scrie_raport(r, csv_dif):
    print(f"randuri: Python {r['randuri_python']}, Rust {r['randuri_rust']}")
    for camp, v in r["numaratoare"].items():
        semn = "  " if v["python"] == v["rust"] else "!!"
        print(f" {semn} {camp}: Python {v['python']}, Rust {v['rust']}")
    if not r["js_disponibil"] and any(x["camp"] == "rezumat" for x in r["regresii"]):
        print("ATENTIE: node lipseste, rezumatele diferite nu au putut fi arbitrate")
    print(f"\ncorecturi (rezumat diferit, JS confirma Rust-ul): {len(r['corecturi'])}")
    for x in r["corecturi"]:
        print(f"  {x['obiect']} @ {x['baza']}: Python {x['python']!r} -> Rust {x['rust']!r}")
    print(f"\nregresii: {len(r['regresii'])}")
    for x in r["regresii"]:
        extra = f", JS {x['js']!r}" if "js" in x else ""
        print(f"  {x['obiect']} @ {x['baza']} [{x['camp']}]: Python {x['python']!r}, Rust {x['rust']!r}{extra}")
    if csv_dif is not None:
        print(f"\nexport CSV contra instantaneu: {len(csv_dif)} diferente")
        for x in csv_dif[:50]:
            print(f"  {x['obiect']} @ {x['baza']}: CSV {x['csv']}, instantaneu {x['instantaneu']}")
        if len(csv_dif) > 50:
            print(f"  ... si inca {len(csv_dif) - 50}")
    ok = not r["regresii"] and not csv_dif
    print("\nPARITATE" if ok else "\nFARA PARITATE")
    return ok


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("instantaneu", nargs="?")
    ap.add_argument("--din-cazuri", action="store_true",
                    help="instantaneu sintetic din referinta/cazuri.json")
    ap.add_argument("--referinta")
    sursa = ap.add_mutually_exclusive_group(required=True)
    sursa.add_argument("--rust", help="ieșirea lui `gmotion compara --json`")
    sursa.add_argument("--gmotion", help="binarul gmotion, rulat pe instantaneu")
    ap.add_argument("--csv", help="exportul CSV al aplicatiei Python, pe aceleasi baze")
    ap.add_argument("--json", help="scrie raportul si ca JSON")
    args = ap.parse_args(argv)

    try:
        with tempfile.TemporaryDirectory() as tmp:
            if args.din_cazuri:
                if args.rust:
                    raise EroareIntrare("--din-cazuri cere --gmotion: rezultatul Rust se produce aici")
                inst = instantaneu_din_cazuri()
                referinta = args.referinta or "Referinta"
                cale = os.path.join(tmp, "instantaneu.json")
                with open(cale, "w", encoding="utf-8") as f:
                    json.dump(inst, f)
            else:
                if not args.instantaneu or not args.referinta:
                    raise EroareIntrare("trebuie dat instantaneul si --referinta (sau --din-cazuri)")
                cale, referinta = args.instantaneu, args.referinta
                inst = citeste_instantaneu(cale)
            if referinta not in inst["baze"]:
                raise EroareIntrare(f"referinta {referinta!r} nu e in instantaneu: {sorted(inst['baze'])}")

            meta_py, lista_py = lista_python(inst, referinta)
            rust = lista_rust(args, cale, referinta)
            if rust.get("referinta") != referinta:
                raise EroareIntrare(f"rezultatul Rust are referinta {rust.get('referinta')!r}, nu {referinta!r}")
            raport = compara(inst, referinta, meta_py, lista_py, rust)
            csv_dif = compara_csv(args.csv, lista_py) if args.csv else None
    except EroareIntrare as e:
        print(f"eroare: {e}", file=sys.stderr)
        return 2

    ok = scrie_raport(raport, csv_dif)
    if args.json:
        with open(args.json, "w", encoding="utf-8") as f:
            json.dump({**raport, "csv": csv_dif}, f, ensure_ascii=False, indent=1)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
