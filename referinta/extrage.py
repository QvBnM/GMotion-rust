"""Extrage din un instantaneu mare doar obiectele alese, intr-un instantaneu mic.

Se strica astfel: fixture-ul extras nu mai reproduce ce s-a vazut pe baza
reala — lipseste perechea dintr-o baza, sau un obiect lipsa/extra devine
diferit. De-aia se pastreaza obiectul din **toate** bazele in care exista, iar
bazele raman toate in fisier, chiar goale.

Folosit la etapa 6: instantaneul real are 202 MB si cod care nu intra in repo;
in repo intra doar obiectele de test, ca fixture de regresie.

  python3 -I referinta/extrage.py instantaneu.json --filtru TESTSIMILITUDINE -o mic.json

`--filtru` se potriveste pe `schema.nume`, fara diferenta de majuscule; se poate
da de mai multe ori. Cod de ieșire 0, sau 2 daca nu s-a potrivit nimic.
"""

import argparse
import json
import sys


def extrage(inst, filtre):
    filtre = [f.lower() for f in filtre]
    potrivit = lambda o: any(f in f"{o['schema']}.{o['nume']}".lower() for f in filtre)
    return {
        **inst,
        "nota": (inst.get("nota", "") + f" | extras: {', '.join(filtre)}").strip(" |"),
        "baze": {b: [o for o in obs if potrivit(o)] for b, obs in inst["baze"].items()},
    }


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("instantaneu")
    ap.add_argument("--filtru", action="append", required=True)
    ap.add_argument("-o", "--iesire", required=True)
    args = ap.parse_args(argv)

    with open(args.instantaneu, encoding="utf-8") as f:
        inst = json.load(f)
    if inst.get("versiune") != 1:
        print(f"eroare: versiune de instantaneu {inst.get('versiune')!r}, astept 1", file=sys.stderr)
        return 2
    mic = extrage(inst, args.filtru)
    n = sum(len(v) for v in mic["baze"].values())
    if n == 0:
        print("eroare: niciun obiect nu se potriveste filtrului", file=sys.stderr)
        return 2
    # Aceeasi forma ca `Instantaneu::scrie` din Rust: indentat, ca sa fie diff-abil.
    with open(args.iesire, "w", encoding="utf-8", newline="\n") as f:
        json.dump(mic, f, ensure_ascii=False, indent=2)
        f.write("\n")
    for b, obs in mic["baze"].items():
        print(f"{b}: {len(obs)} obiecte")
    print(f"scris {args.iesire}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
