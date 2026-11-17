"""Aplicatia Python v3.4 aduce obiectele cu pyodbc-ul ei; le compar cu instantaneul scris de Rust.

Se strica astfel: spune „aceleasi obiecte" cand driverele aduc texte diferite
(se compara campuri care ascund diferenta), sau masoara alt cod decat cel din
productie. De-aia se ruleaza `Api._comparatie` din `mostenire/app.py` neatins,
cu `pyodbc` real — aceeasi citire paralela, aceleasi 4 conexiuni — si se
compara, obiect cu obiect, hash-ul si textul de afisare calculate de
`motor.py` pe ce a adus pyodbc, cu aceleasi calculate pe ce a adus tiberius.

Raspunde la doua criterii din etapa 6: driverele aduc aceleasi obiecte, si cat
dureaza citirea in Python fata de Rust.

  set GMOTION_PAROLA (parola nu se da ca argument)
  py -3 -I referinta\\paritate_pyodbc.py instantaneu.json --server SRV\\INST --utilizator sa --referinta Ref

Fara --utilizator: autentificare integrata Windows. Cod de ieșire 0 daca totul
coincide, 1 daca nu, 2 la eroare de intrare.
"""

import argparse
import json
import os
import sys
import time
import types

AICI = os.path.dirname(os.path.abspath(__file__))
MOSTENIRE = os.path.join(os.path.dirname(AICI), "mostenire")
MAX_AFISATE = 20


def importa_app():
    sys.dont_write_bytecode = True  # nimic nou in mostenire/
    try:
        import pyodbc  # noqa: F401
    except ImportError:
        raise SystemExit("eroare: lipseste pyodbc. Ruleaza: py -3 -m pip install pyodbc")
    try:
        import webview  # noqa: F401
    except ImportError:
        # Doar fereastra are nevoie de el; comparatia nu.
        sys.modules["webview"] = types.ModuleType("webview")
    if MOSTENIRE not in sys.path:
        sys.path.insert(0, MOSTENIRE)
    import app
    import motor

    return app, motor


def compara_cu_instantaneu(motor, cache, inst):
    """Pentru fiecare baza: obiecte doar intr-o parte, si obiecte comune cu alt
    hash normalizat sau alt text de afisare."""
    rezultat = {}
    for baza, obiecte in inst["baze"].items():
        rs = {f"{o['schema']}.{o['nume']}": o for o in obiecte}
        py = cache.get(baza, {})
        doar_py = sorted(set(py) - set(rs))
        doar_rs = sorted(set(rs) - set(py))
        diferite = []
        for oid in sorted(set(py) & set(rs)):
            d = rs[oid]["definitie"]
            if motor.calc_hash(motor.normalizeaza(d)) != py[oid]["hash"]:
                diferite.append((oid, "hash normalizat"))
            elif motor.pregateste_afisare(d) != py[oid]["afisare"]:
                diferite.append((oid, "text de afisare"))
            elif (rs[oid]["tip"] or "").strip() != py[oid]["tip"]:
                diferite.append((oid, "tip"))
        rezultat[baza] = {
            "python": len(py),
            "rust": len(rs),
            "doar_python": doar_py,
            "doar_rust": doar_rs,
            "diferite": diferite,
        }
    return rezultat


def main(argv=None):
    for flux in (sys.stdout, sys.stderr):
        try:
            flux.reconfigure(encoding="utf-8", errors="replace")
        except (AttributeError, ValueError):
            pass
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("instantaneu")
    ap.add_argument("--server", required=True)
    ap.add_argument("--utilizator")
    ap.add_argument("--referinta", required=True)
    args = ap.parse_args(argv)

    with open(args.instantaneu, encoding="utf-8") as f:
        inst = json.load(f)
    if args.referinta not in inst["baze"]:
        print(f"eroare: referinta {args.referinta!r} nu e in instantaneu", file=sys.stderr)
        return 2
    if args.utilizator:
        auth, parola = "sql", os.environ.get("GMOTION_PAROLA", "")
        if not parola:
            print("eroare: lipseste GMOTION_PAROLA", file=sys.stderr)
            return 2
    else:
        auth, parola = "windows", None

    app, motor = importa_app()
    app.DEBUG_LIMIT = None
    api = app.Api()
    baze = [args.referinta] + sorted(b for b in inst["baze"] if b != args.referinta)
    t0 = time.perf_counter()
    try:
        api._comparatie(api._gen, args.server, auth, args.utilizator, parola, baze, args.referinta)
    except Exception as e:  # pyodbc arunca tipuri proprii; mesajul conteaza
        print(f"eroare: Python v3.4 nu a putut citi: {e}", file=sys.stderr)
        return 2
    secunde = time.perf_counter() - t0
    m = api._meta
    n_dif = sum(1 for x in api._lista if x["cat"] == "dif")

    print(f"Python v3.4 (pyodbc, 4 conexiuni): {secunde:.1f} s, citire + comparatie")
    print(f"  {m['total']} obiecte: {m['ident']} identice, {n_dif} diferite, "
          f"{m['lipsa']} lipsa, {m['extra']} extra")

    ok = True
    for baza, r in compara_cu_instantaneu(motor, api._cache, inst).items():
        print(f"\n{baza}: pyodbc {r['python']} obiecte, tiberius {r['rust']}")
        for eticheta, lista in (("doar in pyodbc", r["doar_python"]),
                                ("doar in tiberius", r["doar_rust"]),
                                ("text diferit", r["diferite"])):
            if lista:
                ok = False
                print(f"  {eticheta}: {len(lista)}")
                for x in lista[:MAX_AFISATE]:
                    print(f"    {x}")
                if len(lista) > MAX_AFISATE:
                    print(f"    ... si inca {len(lista) - MAX_AFISATE}")
    print("\nACELEASI OBIECTE" if ok else "\nDRIVERELE ADUC OBIECTE DIFERITE")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
