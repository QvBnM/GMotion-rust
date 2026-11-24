#!/usr/bin/env bash
# Amprenta API-ului public al unui crate. Serveste la o singura verificare:
# daca API-ul s-a schimbat, `CRATE.md` trebuie actualizat in acelasi commit.
#
# Cauza de eroare proprie: documentatia care ramane in urma codului. E singurul
# fel de bug pe care nu-l prinde niciun test.
#
#   unelte/api-hash.sh rust/core
#
# Ce intra (auditul 2026-11-23: varianta pe linii `pub` scapa trei feluri de
# schimbari):
# - fiecare declaratie `pub`, cu semnatura intreaga chiar daca e pe mai multe
#   linii (pana la `{` sau `;`);
# - variantele unui `pub enum` si metodele unui `pub trait`;
# - pentru un binar cu `enum Comanda` (cli), subcomenzile si argumentele.
# Comentariile nu intra si spatiile se normalizeaza. **`mod teste` intra**: un
# `pub fn` dintr-un `#[cfg(test)] mod teste` schimba hash-ul, fiindca filtrul e
# pe linii care incep cu `pub`, nu pe module. Verificat la auditul 2026-11-23,
# cand comentariul de aici spunea contrariul. Practic: cand adaugi un ajutor de
# test `pub`, `CRATE.md` cere si el hash nou.
set -euo pipefail

crate="${1:?folosire: unelte/api-hash.sh <cale-crate>}"
src="$crate/src"

[ -d "$src" ] || { echo "nu exista $src" >&2; exit 1; }

python3 -I - "$src" <<'PY' | sha256sum | cut -c1-16
import os, re, sys
src = sys.argv[1]
linii_out = []

def fara_comentarii(t):
    t = re.sub(r"/\*.*?\*/", " ", t, flags=re.S)
    return "\n".join(re.sub(r"//.*$", "", l) for l in t.split("\n"))

def bloc(text, start):
    """Corpul dintre acolada de la `start` si perechea ei."""
    i = text.index("{", start)
    d = 0
    for j in range(i, len(text)):
        if text[j] == "{":
            d += 1
        elif text[j] == "}":
            d -= 1
            if d == 0:
                return text[i + 1:j]
    return text[i + 1:]

def la_adancime_zero(corp):
    """Bucatile de la nivelul de sus al unui corp, despartite de `,` sau `;`/`}`."""
    out, d, cur = [], 0, ""
    for c in corp:
        if c in "{([<":
            d += 1
        elif c in "})]>":
            d -= 1
        if d == 0 and c in ",;":
            out.append(cur); cur = ""
            continue
        cur += c
        if d == 0 and c == "}":
            out.append(cur); cur = ""
    out.append(cur)
    return [re.sub(r"\s+", " ", x).strip() for x in out if x.strip()]

for radacina, _, fisiere in os.walk(src):
    for f in sorted(fisiere):
        if not f.endswith(".rs"):
            continue
        text = open(os.path.join(radacina, f), encoding="utf-8").read()
        m = re.search(r"^\s*#\[cfg\(test\)\]", text, re.M)
        if m:
            text = text[: m.start()]
        text = fara_comentarii(text)
        for m in re.finditer(r"^[ \t]*pub(\s|\()", text, re.M):
            rest = text[m.start():]
            k = min([p for p in (rest.find("{"), rest.find(";")) if p >= 0] or [len(rest)])
            sig = re.sub(r"\s+", " ", rest[:k]).strip()
            linii_out.append(sig)
            if re.match(r"pub(\(\w+\))?\s+(enum|trait)\b", sig) and rest[k:k + 1] == "{":
                corp = bloc(text, m.start() + k)
                for p in la_adancime_zero(corp):
                    if re.match(r"pub(\(\w+\))?\s+(enum)\b", sig) or p.startswith("fn ") or " fn " in p:
                        linii_out.append(sig + " :: " + re.sub(r"\{.*$", "", p).strip())
        if f == "main.rs":
            m = re.search(r"^enum Comanda \{", text, re.M)
            if m:
                for p in la_adancime_zero(bloc(text, m.start())):
                    linii_out.append("Comanda :: " + p)
for l in sorted(linii_out):
    print(l)
PY
