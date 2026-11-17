"""Genereaza corpusul sintetic pentru bench (corpus_bench.json, negitat).

Proceduri T-SQL plauzibile, cu `WITH (NOLOCK)`, tabele temporare, `IF ... BEGIN`,
atribuiri in variabile - adica exact formele pe care segmentarea trebuie sa le
trateze. Fiecare pereche are o varianta modificata ușor, ca sa existe diferente
reale de aliniat.

    python3 -I rust/referinta/genereaza_corpus.py
"""
import json
import os
import random

TABELE = ["Clienti", "Facturi", "Stoc", "Produse", "Comenzi", "Gestiuni", "Parteneri"]
NR_PERECHI = 400


def procedura(seed: int, varianta: bool = False) -> str:
    random.seed(seed)
    linii = [
        f"CREATE PROCEDURE dbo.p_{seed} @id INT, @data DATETIME",
        "AS",
        "BEGIN",
        "    DECLARE @x INT, @y VARCHAR(50)",
    ]
    for i in range(random.randint(8, 16)):
        t = random.choice(TABELE)
        match random.randint(0, 4):
            case 0:
                linii += [
                    "",
                    "    SELECT a.id, a.nume, b.cantitate",
                    f"    FROM {t} a WITH (NOLOCK)",
                    "    JOIN Stoc b ON b.id = a.id",
                    "    WHERE a.data >= @data AND a.id = @id",
                ]
            case 1:
                linii += [
                    "",
                    f"    UPDATE {t}",
                    f"    SET cantitate = cantitate - {random.randint(1, 9)}",
                    "    WHERE id = @id",
                ]
            case 2:
                linii += ["", f"    INSERT INTO #tmp_{i} (a, b)", f"    SELECT a, b FROM {t}"]
            case 3:
                linii += [
                    "",
                    f"    IF @x = {random.randint(1, 5)}",
                    "    BEGIN",
                    "        EXEC dbo.p_ajutor @id",
                    "    END",
                ]
            case _:
                linii += ["", f"    SELECT @x = COUNT(*) FROM {t} WHERE id = @id"]
    linii += ["", "END"]

    cod = "\n".join(linii)
    if varianta:
        cod = cod.replace("cantitate - 3", "cantitate - 7")
        cod = cod.replace("@id = @id", "@id = @id AND a.activ = 1")
        cod = cod.replace("COUNT(*)", "COUNT(1)", 1)
    return cod


perechi = [{"ref": procedura(i), "cur": procedura(i, True)} for i in range(NR_PERECHI)]

cale = os.path.join(os.path.dirname(os.path.abspath(__file__)), "corpus_bench.json")
with open(cale, "w", encoding="utf-8") as f:
    json.dump(perechi, f, ensure_ascii=False)

linii_total = sum(p["ref"].count("\n") + 1 for p in perechi)
print(
    f"scris: {cale}\n"
    f"{len(perechi)} perechi, {linii_total} linii "
    f"({linii_total // len(perechi)} linii/obiect)"
)
