#!/usr/bin/env python3
"""Harta codului: fiecare fisier .rs / .slint din rust/, cu rolul lui (prima
linie din antet) si elementele de nivel inalt, cu numarul de linie.

Rostul: sa ajungi direct la locul cautat (`sed -n 120,180p fisier`) in loc sa
citesti fisiere intregi. Fisierul generat (HARTA-COD.md) nu intra in git: se
regenereaza la pornirea sesiunii, deci nu poate fi invechit.

    python3 -I unelte/harta-cod.py            # scrie HARTA-COD.md
    python3 -I unelte/harta-cod.py --stdout   # doar afiseaza
"""
import os
import re
import sys

RADACINA = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RUST = os.path.join(RADACINA, "rust")

RS = [
    (re.compile(r"^(pub(\([a-z]+\))? )?(async )?(const )?fn ([A-Za-z0-9_]+)"), "fn"),
    (re.compile(r"^(pub(\([a-z]+\))? )?struct ([A-Za-z0-9_]+)"), "struct"),
    (re.compile(r"^(pub(\([a-z]+\))? )?enum ([A-Za-z0-9_]+)"), "enum"),
    (re.compile(r"^(pub(\([a-z]+\))? )?trait ([A-Za-z0-9_]+)"), "trait"),
    (re.compile(r"^(pub(\([a-z]+\))? )?type ([A-Za-z0-9_]+)"), "type"),
    (re.compile(r"^(pub(\([a-z]+\))? )?(const|static) ([A-Z0-9_]+)"), "const"),
    (re.compile(r"^impl(<[^>]*>)? (.+?) ?\{"), "impl"),
    (re.compile(r"^(pub )?mod ([a-z0-9_]+)"), "mod"),
    (re.compile(r"^macro_rules! ([a-z0-9_]+)"), "macro"),
]
SLINT = [
    (re.compile(r"^(export )?component ([A-Za-z0-9_-]+)"), "component"),
    (re.compile(r"^(export )?struct ([A-Za-z0-9_-]+)"), "struct"),
    (re.compile(r"^(export )?global ([A-Za-z0-9_-]+)"), "global"),
    (re.compile(r"^\s+callback ([a-z0-9-]+)"), "callback"),
]


def nume(m):
    g = [x for x in m.groups() if x]
    return g[-1].strip()


def rol(linii, slint):
    """Prima propozitie din antetul fisierului."""
    out = []
    pref = "//" if slint else "//!"
    for l in linii[:12]:
        s = l.strip()
        if not s.startswith(pref):
            if out:
                break
            continue
        t = s[len(pref):].strip()
        if not t:
            if out:
                break
            continue
        out.append(t)
        if t.endswith("."):
            break
    # taiere pe cuvant, nu pe caracter: pana la auditul 2026-11-23 rolul se
    # rupea in mijlocul unui cuvant („...iar optional ")
    rol = " ".join(out)
    if len(rol) <= 160:
        return rol
    scurt = rol[:160]
    taiat = scurt.rsplit(" ", 1)[0] if " " in scurt else scurt
    return taiat.rstrip(" ,;:") + "…"


def harta_fisier(cale):
    with open(cale, encoding="utf-8") as f:
        linii = f.read().split("\n")
    slint = cale.endswith(".slint")
    elemente = []
    in_teste = False
    teste = 0
    for i, l in enumerate(linii, 1):
        if not slint:
            if re.match(r"^#\[cfg\(test\)\]", l):
                in_teste = True
                elemente.append((i, "teste", ""))
                continue
            if in_teste:
                if re.match(r"^\s+fn ", l):
                    teste += 1
                continue
            # elementele din impl, indentate cu 4: metodele
            m = re.match(r"^    (pub(\([a-z]+\))? )?(async )?(const )?fn ([A-Za-z0-9_]+)", l)
            if m:
                elemente.append((i, "  fn", m.group(5)))
                continue
            for rx, fel in RS:
                m = rx.match(l)
                if m:
                    elemente.append((i, fel, nume(m)))
                    break
        else:
            for rx, fel in SLINT:
                m = rx.match(l)
                if m:
                    elemente.append((i, fel, nume(m)))
                    break
    return rol(linii, slint), elemente, teste, len(linii)


def fisiere():
    for d, subd, fs in os.walk(RUST):
        subd[:] = sorted(x for x in subd if x not in ("target",))
        for f in sorted(fs):
            if f.endswith(".rs") or f.endswith(".slint"):
                yield os.path.join(d, f)


def main():
    out = [
        "# Harta codului (generata)",
        "",
        "Generata de `unelte/harta-cod.py`; nu se editeaza de mana si nu e in git.",
        "Format: `L<linie> <fel> <nume>`; `  fn` = metoda dintr-un `impl`.",
        "Citeste doar intervalul: `sed -n '<start>,<urmatorul-1>p' <fisier>`.",
        "",
    ]
    for cale in fisiere():
        rel = os.path.relpath(cale, RADACINA)
        r, el, teste, n = harta_fisier(cale)
        out.append(f"## {rel} ({n} linii)")
        if r:
            out.append(r)
        for i, fel, nm in el:
            if fel == "teste":
                out.append(f"L{i} teste ({teste} fn)")
            else:
                out.append(f"L{i} {fel} {nm}")
        out.append("")
    text = "\n".join(out)
    if "--stdout" in sys.argv:
        sys.stdout.write(text)
    else:
        with open(os.path.join(RADACINA, "HARTA-COD.md"), "w", encoding="utf-8") as f:
            f.write(text)
        print(f"HARTA-COD.md: {len(out)} linii")


if __name__ == "__main__":
    main()
