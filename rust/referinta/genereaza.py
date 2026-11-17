"""Ruleaza cazurile de test prin motorul PYTHON (motor.py) si scrie rezultatele.

Se ruleaza din radacina repo-ului:
    python3 -I rust/referinta/genereaza.py
"""
import json
import os
import sys

RADACINA = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, RADACINA)

import motor  # noqa: E402

AICI = os.path.dirname(os.path.abspath(__file__))

with open(os.path.join(AICI, "cazuri.json"), encoding="utf-8") as f:
    cazuri = json.load(f)

out = []
for c in cazuri:
    ref = motor.pregateste_afisare(c["ref"])
    cur = motor.pregateste_afisare(c["cur"])
    randuri = motor.diff_aliniat(ref, cur)
    out.append({
        "nume": c["nume"],
        "norm_ref": motor.normalizeaza(c["ref"]),
        "norm_cur": motor.normalizeaza(c["cur"]),
        "scor": motor.scor_similaritate(ref, cur),
        "blocuri_ref": list(motor._split_cached(ref)),
        "blocuri_cur": list(motor._split_cached(cur)),
        "rezumat_rapid": motor.rezumat_rapid(ref, cur),
        "rezumat_diff": motor.rezumat_diff(randuri),
        "tipuri": [r["tip"] for r in randuri],
    })

cale = os.path.join(AICI, "rezultate_python.json")
with open(cale, "w", encoding="utf-8") as f:
    json.dump(out, f, ensure_ascii=False, indent=1, sort_keys=True)
print(f"scris: {cale} ({len(out)} cazuri)")
