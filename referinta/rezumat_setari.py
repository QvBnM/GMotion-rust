"""Cate obiecte din instantaneu au fost create cu ANSI_NULLS / QUOTED_IDENTIFIER OFF.

Deploy-ul le reface exact; rezumatul arata cat de des conteaza (v3.4 nu le
scria, deci obiectele acestea isi schimbau comportamentul la fiecare deploy).

  py -3 -I referinta\\rezumat_setari.py instantaneu.json
"""
import json
import sys

with open(sys.argv[1], encoding="utf-8") as f:
    inst = json.load(f)
for baza, obiecte in inst["baze"].items():
    fara = sum(1 for o in obiecte if "setari" not in o)
    an = [o for o in obiecte if o.get("setari") and not o["setari"]["ansi_nulls"]]
    qi = [o for o in obiecte if o.get("setari") and not o["setari"]["quoted_identifier"]]
    print(f"{baza}: {len(obiecte)} obiecte, fara setari {fara}, "
          f"ANSI_NULLS OFF {len(an)}, QUOTED_IDENTIFIER OFF {len(qi)}")
    for o in (an + qi)[:5]:
        print(f"  ex: {o['schema']}.{o['nume']}")
