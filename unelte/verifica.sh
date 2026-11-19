#!/usr/bin/env bash
# Poarta. Nimic nu se commiteaza daca asta nu e verde.
#
# Ruleaza toate verificarile, apoi raporteaza. NU se opreste la prima eroare:
# vrei sa vezi tot ce e rupt intr-o singura rulare, nu sa reporneasca de zece
# ori. Verificarile care nu pot rula (lipseste `node`, lipseste tinta wasm) sunt
# raportate explicit ca SARIT — citeste iesirea, nu doar codul de retur.
#
#   unelte/verifica.sh            tot
#   unelte/verifica.sh rapid      fara wasm si fara conformitatea Python/JS
set -uo pipefail

RADACINA="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$RADACINA"
MOD="${1:-tot}"

ROSU=$'\033[31m'; VERDE=$'\033[32m'; GALBEN=$'\033[33m'; GRI=$'\033[90m'; ZERO=$'\033[0m'
if [ ! -t 1 ]; then ROSU=; VERDE=; GALBEN=; GRI=; ZERO=; fi

ESECURI=(); SARITE=()

pas()    { printf '%s  ok    %s%s\n'   "$VERDE"  "$1" "$ZERO"; }
cade()   { printf '%s  CADE  %s%s\n'   "$ROSU"   "$1" "$ZERO"; ESECURI+=("$1"); }
sare()   { printf '%s  sarit %s — %s%s\n' "$GALBEN" "$1" "$2" "$ZERO"; SARITE+=("$1: $2"); }
titlu()  { printf '\n%s== %s%s\n' "$GRI" "$1" "$ZERO"; }

# Ruleaza o comanda; la esec arata iesirea.
ruleaza() {
  local nume="$1"; shift
  local ies
  if ies="$("$@" 2>&1)"; then pas "$nume"; else cade "$nume"; printf '%s\n' "$ies" | sed 's/^/      /'; fi
}

# ---------------------------------------------------------------- 1. format
titlu "format"
if command -v cargo >/dev/null; then
  ruleaza "cargo fmt --check" cargo fmt --manifest-path rust/Cargo.toml --all -- --check
else
  sare "cargo fmt" "cargo nu e instalat"
fi

# ---------------------------------------------------------------- 2. clippy
titlu "clippy"
if command -v cargo >/dev/null; then
  ruleaza "clippy -D warnings" cargo clippy --manifest-path rust/Cargo.toml --all-targets --all-features -- -D warnings
else
  sare "clippy" "cargo nu e instalat"
fi

# Avertismentele compilatorului Slint (`.slint`) nu le vede clippy: apar
# doar cand build.rs al ferestrei ruleaza din nou, ca `warning: gmotion-
# fereastra@...`. Un fisier `.slint` atins il face sa ruleze; orice
# avertisment pica poarta (o proprietate depreciata a ajuns asa la Mihai,
# 2026-11-19, iar o bucla de layout „poate cadea la rulare").
avertismente_slint() {
  touch rust/fereastra/ui/fereastra.slint
  local ies
  ies="$(cargo build --manifest-path rust/Cargo.toml -p gmotion-fereastra 2>&1)" || { printf '%s\n' "$ies"; return 1; }
  if printf '%s\n' "$ies" | grep -q "^warning: gmotion-fereastra@"; then
    printf '%s\n' "$ies" | grep "^warning: gmotion-fereastra@" | head -20
    return 1
  fi
}
if command -v cargo >/dev/null; then
  ruleaza "fara avertismente Slint" avertismente_slint
else
  sare "avertismente Slint" "cargo nu e instalat"
fi

# ---------------------------------------------------------------- 3. teste
titlu "teste"
if command -v cargo >/dev/null; then
  ies="$(cargo test --manifest-path rust/Cargo.toml --all 2>&1)"
  if [ $? -eq 0 ]; then
    # Din rezumatele `test result:`, nu din randurile `test ... ok`: jurnalul
    # testelor se poate intercala pe acelasi rand si numaratoarea ar varia.
    pas "cargo test ($(printf '%s' "$ies" | sed -nE 's/^test result: ok\. ([0-9]+) passed.*/\1/p' | awk '{s+=$1} END {print s+0}') teste)"
  else
    cade "cargo test"; printf '%s\n' "$ies" | grep -E '^(test |error|---- )' | sed 's/^/      /' | head -40
  fi
else
  sare "cargo test" "cargo nu e instalat"
fi

# ---------------------------------------------------------------- 4. wasm
# `core` trebuie sa mearga pe wasm. Daca nu merge, backendul si editorul live
# nu mai pot folosi acelasi motor — adica exact bug-ul din v3.4, reintrodus.
titlu "tinta wasm (core nu are I/O)"
if [ "$MOD" = rapid ]; then
  sare "wasm32" "mod rapid"
elif ! command -v cargo >/dev/null; then
  sare "wasm32" "cargo nu e instalat"
elif ! rustup target list --installed 2>/dev/null | grep -qx wasm32-unknown-unknown; then
  sare "wasm32" "tinta lipseste: rustup target add wasm32-unknown-unknown"
else
  ruleaza "core compileaza pe wasm32" cargo build --manifest-path rust/Cargo.toml -p gmotion-core --target wasm32-unknown-unknown
fi

# ---------------------------------------------------------------- 5. tinta Windows
# Tinta reala de rulare e Windows, dar dezvoltarea se face pe Linux. Codul
# `#[cfg(windows)]` — DPAPI, auth integrata, apeluri Win32 — nu e nici compilat
# nici vazut de clippy aici, deci s-ar strica fara sa se observe pana la build
# pe Windows. `cargo check` pe tinta nu cere linker, doar biblioteca standard.
titlu "tinta Windows (codul cfg(windows))"
if [ "$MOD" = rapid ]; then
  sare "windows-msvc" "mod rapid"
elif ! command -v cargo >/dev/null; then
  sare "windows-msvc" "cargo nu e instalat"
elif ! rustup target list --installed 2>/dev/null | grep -qx x86_64-pc-windows-msvc; then
  sare "windows-msvc" "tinta lipseste: rustup target add x86_64-pc-windows-msvc"
else
  ruleaza "cargo check pe windows-msvc" cargo check --manifest-path rust/Cargo.toml --all-targets --target x86_64-pc-windows-msvc
fi

# ---------------------------------------------------------------- 6. dependente
# Directia e: core <- db <- platforma <- module/* <- fereastra, cli <- app.
# Nimic nu depinde la dreapta, si niciun modul nu depinde de alt modul.
# `app` e deasupra tuturor: el compune ferestrele cu modulele.
titlu "directia dependentelor"
python3 - <<'PY'
import os, re, sys, glob

NIVEL = {"gmotion-core": 0, "gmotion-db": 1, "gmotion-platforma": 2,
         "gmotion-cli": 4, "gmotion-fereastra": 4, "gmotion-app": 5}
probleme = []

for cale in glob.glob("rust/**/Cargo.toml", recursive=True):
    txt = open(cale, encoding="utf-8").read()
    if "[workspace]" in txt and "[package]" not in txt:
        continue
    m = re.search(r'^name\s*=\s*"([^"]+)"', txt, re.M)
    if not m:
        continue
    nume = m.group(1)
    e_modul = os.sep + "module" + os.sep in cale
    nivel = 3 if e_modul else NIVEL.get(nume)
    if nivel is None:
        probleme.append(f"{nume}: crate necunoscut, adauga-l in unelte/verifica.sh si in ARHITECTURA.md")
        continue
    for dep in re.findall(r'^(gmotion-[a-z0-9-]+)\s*[=.]', txt, re.M):
        e_modul_dep = dep not in NIVEL
        nivel_dep = 3 if e_modul_dep else NIVEL[dep]
        if nivel_dep >= nivel:
            probleme.append(f"{nume} -> {dep}: dependinta la dreapta sau laterala")
        if e_modul and e_modul_dep:
            probleme.append(f"{nume} -> {dep}: un modul nu depinde de alt modul; mutǎ in platforma")

if probleme:
    print("\n".join(probleme)); sys.exit(1)
PY
if [ $? -eq 0 ]; then pas "directia dependentelor"; else cade "directia dependentelor"; fi

# ---------------------------------------------------------------- 6b. MSRV
# `rust-version` spunea 1.74, iar dependentele cereau de mult mai mult: `db` si
# `platforma` nici nu se construiau pe 1.74. O versiune minima falsa e o
# promisiune pe care o descopera abia cine incearca (F-B1 din auditul QA).
# Nu se instaleaza un toolchain vechi: se compara ce declaram cu ce cer
# dependentele, din `cargo metadata`.
titlu "versiunea minima de Rust (MSRV)"
if ! command -v cargo >/dev/null; then
  sare "MSRV" "cargo nu e instalat"
elif [ "$MOD" = rapid ]; then
  sare "MSRV" "mod rapid"
else
  cargo metadata --manifest-path rust/Cargo.toml --format-version 1 --all-features 2>/dev/null > "$RADACINA/.msrv.json"
  python3 - "$RADACINA/.msrv.json" <<'PY'
import json, sys

d = json.load(open(sys.argv[1], encoding="utf-8"))
pk = {p["id"]: p for p in d["packages"]}
noduri = {n["id"]: n for n in d["resolve"]["nodes"]}
nostru = [p for p in d["packages"] if p["name"].startswith("gmotion-")]

def ver(v):
    # „1.89" si „1.89.0" sunt aceeasi versiune
    p = [int(x) for x in v.split(".")]
    return (p + [0, 0, 0])[:3]

def toate_dependentele(id_):
    vazut, de_vazut = set(), [id_]
    while de_vazut:
        i = de_vazut.pop()
        if i in vazut:
            continue
        vazut.add(i)
        de_vazut.extend(dep["pkg"] for dep in noduri[i]["deps"])
    return vazut

probleme = []
for p in nostru:
    declarat = p.get("rust_version")
    if not declarat:
        probleme.append(p["name"] + ": fara rust-version")
        continue
    maxim = ("0.0.0", "")
    for i in toate_dependentele(p["id"]):
        rv = pk[i].get("rust_version")
        if rv and ver(rv) > ver(maxim[0]):
            maxim = (rv, pk[i]["name"])
    if ver(maxim[0]) > ver(declarat):
        probleme.append(
            "{}: rust-version = {}, dar {} cere {}".format(
                p["name"], declarat, maxim[1], maxim[0]
            )
        )

if probleme:
    print("\n".join(probleme))
    sys.exit(1)
PY
  if [ $? -eq 0 ]; then pas "MSRV acopera dependentele"; else cade "MSRV"; fi
  rm -f "$RADACINA/.msrv.json"
fi

# ---------------------------------------------------------------- 7. CRATE.md
# Cauza de eroare proprie: documentatia ramasa in urma codului. Singurul fel de
# bug pe care nu-l prinde niciun test.
titlu "CRATE.md sincronizat cu API-ul public"
for crate in rust/*/ rust/module/*/; do
  [ -f "$crate/Cargo.toml" ] || continue
  nume="$(basename "$crate")"
  if [ ! -f "$crate/CRATE.md" ]; then cade "$nume: lipseste CRATE.md"; continue; fi
  acum="$("$RADACINA/unelte/api-hash.sh" "$crate")"
  notat="$(grep -oE 'api-hash: [0-9a-f]+' "$crate/CRATE.md" | head -1 | cut -d' ' -f2)"
  if [ -z "$notat" ]; then
    cade "$nume: CRATE.md nu are linia <!-- api-hash: $acum -->"
  elif [ "$acum" != "$notat" ]; then
    cade "$nume: API-ul public s-a schimbat, CRATE.md nu. Actualizeaza-l si pune: <!-- api-hash: $acum -->"
  else
    pas "$nume/CRATE.md"
  fi
done

# ---------------------------------------------------------------- 8. stare
# Cauza de eroare proprie: sesiunea care nu stie de unde sa continue. Un
# asistent porneste de la zero la fiecare sesiune, iar utilizatorul poate intra
# de pe alt cont — daca STARE.md minte sau doua etape se cred active, se reia
# munca gresita. Verificat deci, nu doar cerut.
titlu "stare si etape"
python3 - <<'PY_STARE'
import glob, os, re, sys

probleme, avertismente = [], []
STARI = {"NEPORNITA", "ACTIVA", "INCHEIATA", "ABANDONATA"}

if not os.path.exists("STARE.md"):
    probleme.append("lipseste STARE.md — vezi sectiunea 0 din CLAUDE.md")

active = []
for cale in sorted(glob.glob("etape/*.md")):
    if os.path.basename(cale) == "SABLON.md":
        continue
    txt = open(cale, encoding="utf-8").read()
    m = re.search(r"^\*\*Stare:\*\* (\w+)", txt, re.M)
    if not m:
        probleme.append(f"{cale}: lipseste linia **Stare:** (una din {', '.join(sorted(STARI))})")
        continue
    if m.group(1) not in STARI:
        probleme.append(f"{cale}: stare necunoscuta {m.group(1)!r}")
    elif m.group(1) == "ACTIVA":
        active.append(cale)
    if m.group(1) == "INCHEIATA" and "(se completeaza la incheiere)" in txt:
        avertismente.append(f"{cale}: incheiata, dar 'Ce a iesit altfel decat planul' e necompletat")

if len(active) > 1:
    probleme.append("mai multe etape ACTIVA: " + ", ".join(active) + " — nu se poate sti unde suntem")

# STARE.md trebuie sa numeasca etapa activa, altfel trimite in alta parte.
if os.path.exists("STARE.md") and active:
    stare = open("STARE.md", encoding="utf-8").read()
    if os.path.basename(active[0]) not in stare:
        probleme.append(f"STARE.md nu numeste etapa activa ({active[0]})")

for a in avertismente:
    print("avertisment: " + a)
if probleme:
    print("\n".join(probleme))
    sys.exit(1)
PY_STARE
if [ $? -eq 0 ]; then pas "stare si etape"; else cade "stare si etape"; fi

# Nu cade pe asta: ar invita la actualizari formale, de bifa. Doar impinge.
if git rev-parse --git-dir >/dev/null 2>&1 && [ -f STARE.md ]; then
  ultim="$(git log -1 --format=%H -- STARE.md 2>/dev/null || true)"
  if [ -n "$ultim" ]; then
    n="$(git rev-list --count "$ultim..HEAD" 2>/dev/null || echo 0)"
    if [ "$n" -ge 3 ]; then
      printf '%s  nota  STARE.md nu a fost atins de %s commit-uri — ruleaza unelte/stare.sh%s\n' "$GALBEN" "$n" "$ZERO"
    fi
  fi
fi

# ---------------------------------------------------------------- 9. mostenire
# ---------------------------------------------------------------------------
# Fara diacritice nicaieri (Mihai, 2026-11-18): textele vor trece in engleza,
# iar pana atunci se scriu fara diacritice — in interfata, in mesaje, in
# comentarii si in documente. Exceptie: datele de test care verifica
# intentionat nume cu diacritice ale utilizatorului, marcate pe linie cu
# `diacritice-permise`. mostenire/ si fixture-urile din referinta/ nu se ating.
titlu "fara diacritice"
python3 - <<'PY_DIACRITICE'
import re, subprocess, sys
fisiere = subprocess.run(["git", "ls-files"], capture_output=True, text=True).stdout.split("\n")
r = re.compile("[\u0103\u00e2\u00ee\u0219\u015f\u021b\u0163\u0102\u00c2\u00ce\u0218\u015e\u021a\u0162]")
gasite = []
for f in fisiere:
    if not f or f.startswith("mostenire/") or (f.startswith("referinta/") and f != "referinta/README.md"):
        continue
    if not re.search(r"\.(rs|slint|md|sh|bat|ps1|py|toml)$", f) or "LICENSE" in f:
        continue
    try:
        linii = open(f, encoding="utf-8").read().split("\n")
    except (OSError, UnicodeDecodeError):
        continue
    for i, l in enumerate(linii, 1):
        if r.search(l) and "diacritice-permise" not in l:
            gasite.append(f"{f}:{i}: {l.strip()[:70]}")
if gasite:
    print("\n".join(gasite[:20]))
    if len(gasite) > 20:
        print(f"... inca {len(gasite) - 20}")
    sys.exit(1)
PY_DIACRITICE
if [ $? -eq 0 ]; then pas "fara diacritice"; else cade "fara diacritice"; fi

# ---------------------------------------------------------------------------
# Niciun buton fara actiune (etapa 14): fiecare `callback` declarat la nivelul
# unei ferestre exportate din rust/fereastra/ui trebuie sa aiba un handler
# `on_<nume>` in codul Rust care o foloseste. Asa au trecut neobservate 7
# butoane moarte prin etapele 8-11: arata perfect in captura, nu fac nimic.
titlu "fiecare buton are actiune"
python3 - <<'PY_BUTOANE'
import glob, re, sys
cod = ""
for f in glob.glob("rust/app/src/**/*.rs", recursive=True) + glob.glob("rust/fereastra/src/**/*.rs", recursive=True):
    cod += open(f, encoding="utf-8").read()
lipsa = []
for f in sorted(glob.glob("rust/fereastra/ui/*.slint")):
    text = open(f, encoding="utf-8").read()
    for m in re.finditer(r"^export component (\w+) inherits Window \{", text, re.M):
        corp = text[m.end():]
        # pana la acolada care inchide componenta (prima linie "}" fara indentare)
        sfarsit = re.search(r"^\}", corp, re.M)
        corp = corp[: sfarsit.start() if sfarsit else len(corp)]
        for c in re.finditer(r"^    callback ([a-z0-9-]+)", corp, re.M):
            nume = "on_" + c.group(1).replace("-", "_")
            if not re.search(r"\b" + nume + r"\(", cod):
                lipsa.append(f"{f}: {m.group(1)}.{c.group(1)} n-are {nume}")
if lipsa:
    print("\n".join(lipsa))
    sys.exit(1)
PY_BUTOANE
if [ $? -eq 0 ]; then pas "fiecare buton are actiune"; else cade "fiecare buton are actiune"; fi

titlu "mostenire/ nemodificat"
if git rev-parse --git-dir >/dev/null 2>&1; then
  # Doar CONTINUT modificat (filtrul M). O cale nou adaugata nu e o modificare:
  # altfel commit-ul care a mutat Python-ul in mostenire/ ar cadea pe sine.
  if git diff --name-only --diff-filter=M HEAD -- mostenire/ 2>/dev/null | grep -q .; then
    cade "mostenire/ e modificat — vezi regula 2 din CLAUDE.md"
    git diff --name-only --diff-filter=M HEAD -- mostenire/ | sed 's/^/      /'
  else
    pas "mostenire/ nemodificat"
  fi
else
  sare "mostenire/" "nu e repo git"
fi

# ---------------------------------------------------------------- 10. conformitate Python/JS
# Nu valideaza Rust-ul (asta o face cargo test), ci verifica faptul ca harness-ul
# de referinta inca ruleaza — plasa de siguranta pentru tot ce urmeaza.
titlu "harness de referinta"
if [ "$MOD" = rapid ]; then
  sare "harness" "mod rapid"
else
  if command -v python3 >/dev/null; then
    ruleaza "motorul Python ruleaza cazurile" python3 -I referinta/genereaza.py
  else
    sare "harness Python" "python3 nu e instalat"
  fi
  if command -v node >/dev/null; then
    ruleaza "motorul JS ruleaza cazurile" node referinta/genereaza.mjs
  else
    sare "harness JS" "node nu e instalat"
  fi
fi

# ---------------------------------------------------------------- 11. paritate
# Python v3.4 (mostenire/app.py, neatins) si binarul gmotion pe acelasi
# instantaneu sintetic. node e arbitrul pentru rezumatele care difera; fara el,
# orice corectura ar arata ca regresie, deci se sare explicit.
titlu "paritate Python v3.4 / gmotion"
if [ "$MOD" = rapid ]; then
  sare "paritate" "mod rapid"
elif ! command -v python3 >/dev/null || ! command -v node >/dev/null; then
  sare "paritate" "cere python3 si node"
elif ! cargo build -q --manifest-path rust/Cargo.toml -p gmotion-cli 2>/dev/null; then
  cade "paritate: binarul gmotion nu se construieste"
else
  # --strict: si diferentele „de verificat" cad. Diferentele deliberate fata
  # de v3.4 stau in <fixture>.asteptari.json, fiecare cu motivul ei, si
  # fiecare trebuie sa apara (o asteptare care nu mai apare cade si ea).
  ruleaza "paritate pe instantaneul din cazuri" \
    python3 -I referinta/paritate.py --din-cazuri --strict --gmotion rust/target/debug/gmotion
  # Fixture-uri extrase de pe baze reale (referinta/extrage.py). Fiecare cu
  # baza lui de referinta: instantaneul nu o retine.
  while read -r fisier referinta; do
    asteptari=()
    [ -f "${fisier%.json}.asteptari.json" ] && asteptari=(--asteptari "${fisier%.json}.asteptari.json")
    ruleaza "paritate pe $(basename "$fisier")" \
      python3 -I referinta/paritate.py "$fisier" --referinta "$referinta" --strict "${asteptari[@]}" --gmotion rust/target/debug/gmotion
  done <<'FIXTURI'
referinta/instantanee/testsimilitudine.json CharismaERP_GetsBet
FIXTURI
fi

# ---------------------------------------------------------------- raport
printf '\n'
if [ ${#SARITE[@]} -gt 0 ]; then
  printf '%ssarit %d verificari:%s\n' "$GALBEN" "${#SARITE[@]}" "$ZERO"
  for s in "${SARITE[@]}"; do printf '  - %s\n' "$s"; done
fi
if [ ${#ESECURI[@]} -gt 0 ]; then
  printf '\n%sPOARTA ROSIE — %d probleme:%s\n' "$ROSU" "${#ESECURI[@]}" "$ZERO"
  for e in "${ESECURI[@]}"; do printf '  - %s\n' "$e"; done
  exit 1
fi
printf '%sPOARTA VERDE%s\n' "$VERDE" "$ZERO"
