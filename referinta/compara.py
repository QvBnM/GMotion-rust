"""Compara rezultatele celor doua motoare (Python vs JS) si raporteaza divergentele.

    python3 -I referinta/compara.py
"""
import json
import os

AICI = os.path.dirname(os.path.abspath(__file__))

with open(os.path.join(AICI, "rezultate_python.json"), encoding="utf-8") as f:
    py = {c["nume"]: c for c in json.load(f)}
with open(os.path.join(AICI, "rezultate_js.json"), encoding="utf-8") as f:
    js = {c["nume"]: c for c in json.load(f)}

CAMPURI = ["norm_ref", "norm_cur", "scor", "blocuri_ref", "blocuri_cur",
           "rezumat_rapid", "rezumat_diff", "tipuri"]

divergente = {c: 0 for c in CAMPURI}
cazuri_cu_probleme = []

for nume in py:
    dif = [c for c in CAMPURI if py[nume][c] != js[nume][c]]
    for c in dif:
        divergente[c] += 1
    if dif:
        cazuri_cu_probleme.append((nume, dif))

total = len(py)
print(f"Cazuri testate: {total}")
print(f"Cazuri cu cel putin o divergenta: {len(cazuri_cu_probleme)}/{total}\n")

print("Divergente pe camp:")
for c in CAMPURI:
    n = divergente[c]
    marcaj = "  OK " if n == 0 else f"  !! "
    print(f"{marcaj}{c:<16} {n}/{total}")

if cazuri_cu_probleme:
    print("\nDetaliu (primele 8 cazuri):")
    for nume, dif in cazuri_cu_probleme[:8]:
        print(f"\n--- {nume}: difera la {', '.join(dif)}")
        for c in dif:
            if c in ("scor", "rezumat_rapid", "rezumat_diff"):
                print(f"    python: {py[nume][c]!r}")
                print(f"    js    : {js[nume][c]!r}")
            elif c in ("blocuri_ref", "blocuri_cur"):
                print(f"    python: {len(py[nume][c])} blocuri")
                print(f"    js    : {len(js[nume][c])} blocuri")
            elif c == "tipuri":
                print(f"    python: {py[nume][c]}")
                print(f"    js    : {js[nume][c]}")
