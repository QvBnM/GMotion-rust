#!/usr/bin/env python3
"""Instantaneu sintetic mare, pentru masuratorile de fluiditate.

Scrie un instantaneu cu bazele `Ref` (Before) si `C1` (After), cu N obiecte
in fiecare, in proportiile de pe baza lui Mihai (2026-11-25: 11.938 diferite
din ~28.000): ~30% modificate, ~25% doar in Before, ~10% Extra, restul
identice, proceduri de 40-90 de linii cu dependente intre ele. Determinist
(aceeasi samanta), ca cifrele inainte / dupa sa se compare.

    python3 -I unelte/genereaza-mare.py 12000 /tmp/mare.json
"""

import json
import random
import sys


def corp(i, rnd, varianta):
    linii = [f"CREATE PROCEDURE dbo.usp_{i:05d} (@Id int, @SiteId int, @Mod int = 0)", "AS", "BEGIN"]
    for k in range(rnd.randint(12, 30)):
        tinta = rnd.randint(0, i) if i else 0
        linii.append(f"    UPDATE t SET Valoare = Valoare + {k} FROM dbo.Tabel_{k % 40} t WHERE t.Id = @Id AND t.SiteId = @SiteId")
        if k % 3 == 0:
            linii.append(f"    EXEC dbo.usp_{tinta:05d} @Id = @Id, @SiteId = @SiteId")
        if k % 5 == 0:
            linii.append(f"    -- pasul {k}: recalculare pentru {i}")
    if varianta:
        poz = rnd.randint(3, len(linii) - 1)
        linii.insert(poz, f"    SELECT @Mod = {varianta} -- schimbare {varianta}")
        if varianta % 2:
            linii[poz + 1 if poz + 1 < len(linii) else poz] += " OPTION (RECOMPILE)"
    linii.append("END")
    return "\n".join(linii)


def main():
    n = int(sys.argv[1])
    cale = sys.argv[2]
    rnd = random.Random(20261125)
    ref, c1 = [], []
    for i in range(n):
        r = rnd.random()
        o = {"schema": "dbo", "nume": f"usp_{i:05d}", "tip": "P",
             "modificat": f"2026-{1 + i % 12:02d}-{1 + i % 28:02d} 10:00:00"}
        baza = corp(i, random.Random(i), 0)
        if r < 0.30:
            ref.append({**o, "definitie": baza})
            c1.append({**o, "definitie": corp(i, random.Random(i), 1 + i % 7)})
        elif r < 0.55:
            ref.append({**o, "definitie": baza})
        elif r < 0.65:
            c1.append({**o, "definitie": baza})
        else:
            ref.append({**o, "definitie": baza})
            c1.append({**o, "definitie": baza})
    inst = {"versiune": 1, "server": "SRV-MARE", "creat": "2026-11-25T12:00:00Z",
            "nota": f"sintetic, {n} obiecte (unelte/genereaza-mare.py)",
            "baze": {"Ref": ref, "C1": c1}}
    with open(cale, "w", encoding="utf-8", newline="\n") as f:
        json.dump(inst, f)


if __name__ == "__main__":
    main()
