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
  # `touch` pe un fisier urmarit: nu-i schimba continutul, doar mtime-ul, ca
  # `build.rs` sa reruleze. Doua rulari in paralel pe acelasi repo isi
  # reconstruiesc deci reciproc `gmotion-fereastra` (D-13, auditul
  # 2026-11-23); nu se poate altfel fara sa ocolim cache-ul cargo.
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

# Compilarea pe wasm32 **nu** dovedeste „fara I/O": `std::fs`, `std::net` si
# `std::process` exista si acolo, ca stub-uri care cad abia la rulare, deci un
# `std::fs::read_to_string` in `core` trecea verde (auditul 2026-11-23). Lipsa
# lor se verifica pe text, in `rust/core/src`, unde e si regula (CLAUDE.md §3).
python3 - <<'PY_IO'
import glob, re, sys
INTERZIS = re.compile(r"\bstd\s*::\s*(fs|net|process|env|io)\b|\binclude_str!|\binclude_bytes!")
gasite = []
for f in sorted(glob.glob("rust/core/src/**/*.rs", recursive=True)):
    for nr, linie in enumerate(open(f, encoding="utf-8"), 1):
        if INTERZIS.search(linie.split("//")[0]):
            gasite.append(f"{f}:{nr}: {linie.strip()[:80]}")
if gasite:
    print("`core` nu are voie cu I/O (CLAUDE.md §3); pe wasm32 ar compila oricum:")
    print("\n".join(gasite))
    sys.exit(1)
PY_IO
if [ $? -eq 0 ]; then pas "core nu atinge I/O"; else cade "core nu atinge I/O"; fi

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
  # clippy, nu doar check, si cu -D warnings: codul `cfg(windows)` nu e
  # vazut de nimic altceva pe masina de dezvoltare (auditul 2026-11-23)
  ruleaza "clippy pe windows-msvc" cargo clippy --manifest-path rust/Cargo.toml --all-targets --target x86_64-pc-windows-msvc -- -D warnings
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
    # `gmotion-x = ...`, `[dependencies.gmotion-x]` si redenumirea cu
    # `package = "gmotion-x"` (auditul 2026-11-23: ultimele doua scapau)
    deps = set(re.findall(r'^(gmotion-[a-z0-9-]+)\s*[=.]', txt, re.M))
    deps |= set(re.findall(r'^\[[^]]*dependencies\.(gmotion-[a-z0-9-]+)\]', txt, re.M))
    deps |= set(re.findall(r'package\s*=\s*"(gmotion-[a-z0-9-]+)"', txt))
    for dep in sorted(deps):
        e_modul_dep = dep not in NIVEL
        nivel_dep = 3 if e_modul_dep else NIVEL[dep]
        if nivel_dep >= nivel:
            probleme.append(f"{nume} -> {dep}: dependinta la dreapta sau laterala")
        if e_modul and e_modul_dep:
            probleme.append(f"{nume} -> {dep}: un modul nu depinde de alt modul; muta in platforma")

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

# „Exact o etapa poate fi ACTIVA" (CLAUDE.md §0). Pana la auditul din
# 2026-11-23 se verifica doar „nu mai multe": zero etape active trecea verde,
# desi atunci nimeni nu stie ce se lucreaza.
if len(active) > 1:
    probleme.append("mai multe etape ACTIVA: " + ", ".join(active) + " — nu se poate sti unde suntem")
elif not active:
    probleme.append(
        "nicio etapa ACTIVA — deschide una noua din etape/SABLON.md "
        "si numeste-o in STARE.md (CLAUDE.md §0)"
    )

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
import unicodedata
# si fisierele noi, inca neadaugate: altfel trec pana la commit (auditul 2026-11-22)
fisiere = subprocess.run(["git", "ls-files", "--cached", "--others", "--exclude-standard"], capture_output=True, text=True).stdout.split("\n")
class _Diacritic:
    """Orice litera latina cu semn (a cu caciula, s cu virgula, dar si a cu
    caron U+01CE, e accentuat, I cu punct): o lista fixa scapa semnele
    asemanatoare, cum a scapat U+01CE din chiar acest fisier."""
    def search(self, l):
        for c in l:
            if ord(c) > 127 and c.isalpha():
                d = unicodedata.normalize("NFD", c)
                if len(d) > 1 and d[0].isascii() and unicodedata.combining(d[1]):
                    return True
        return False
r = _Diacritic()
gasite = []
for f in fisiere:
    if not f or f.startswith("mostenire/") or (f.startswith("referinta/") and f != "referinta/README.md"):
        continue
    if not re.search(r"\.(rs|slint|md|sh|bat|ps1|py|toml|html|js|css|sql|txt)$", f) or "LICENSE" in f:
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
        # `_` in nume, `pure callback` si alta indentare treceau neobservate,
        # iar potrivirea pe `\bon_x\(` gasea si un `on_x(` dintr-un comentariu
        # sau dintr-un nume mai lung: `compara_tot` se taia la `compara` si
        # gasea `on_compara` (auditul 2026-11-23). Se cere `.on_x(`.
        for c in re.finditer(r"^[ \t]+(?:pure\s+)?callback\s+([A-Za-z0-9_-]+)", corp, re.M):
            nume = "on_" + c.group(1).replace("-", "_")
            if not re.search(r"\." + nume + r"\s*\(", cod):
                lipsa.append(f"{f}: {m.group(1)}.{c.group(1)} n-are {nume}")
if lipsa:
    print("\n".join(lipsa))
    sys.exit(1)
PY_BUTOANE
if [ $? -eq 0 ]; then pas "fiecare buton are actiune"; else cade "fiecare buton are actiune"; fi

titlu "niciun text pentru om scris direct in .slint"
# CLAUDE.md, sectiunea 6: textele trec prin `fereastra::texte` (romana si
# engleza). Se cauta un sir cu litere dat lui `text`, `t`, `gol` sau `title`;
# marca si cuvintele SQL se marcheaza pe aceeasi linie cu `text-permis`.
python3 - <<'PY_TEXTE'
import glob, re, sys
LITERAL = re.compile(r'"(?:[^"\\]|\\.)*"')
# numele tastelor (`Ctrl J K`, `Shift F7`) nu se traduc
TASTE = re.compile(r'^(?:(?:ctrl|shift|alt|esc|enter|tab|f\d{1,2}|[a-z])\b\s*)+$', re.I)
gasite = []
for f in sorted(glob.glob("rust/fereastra/ui/*.slint")):
    for nr, linie in enumerate(open(f, encoding="utf-8"), 1):
        if "text-permis" in linie or linie.lstrip().startswith(("//", "import")):
            continue
        if not re.search(r"\b(?:text|t|gol|title)\s*:", linie):
            continue
        for m in LITERAL.finditer(linie):
            continut = m.group(0)[1:-1]
            if re.search(r"[A-Za-z]{2,}", continut) and not TASTE.match(continut.strip()):
                gasite.append(f"{f}:{nr}: {continut}")
if gasite:
    print("\n".join(gasite))
    sys.exit(1)
PY_TEXTE
if [ $? -eq 0 ]; then pas "niciun text pentru om scris direct in .slint"; else cade "text scris direct in .slint: pune-l in fereastra::texte"; fi

titlu "mostenire/ nemodificat"
# Arborele lui mostenire/ e fixat aici: o stergere, o modificare deja commitata
# sau un fisier nou schimba hash-ul. Varianta pe `git diff --diff-filter=M`
# nu vedea nici stergerile, nici ce s-a commitat deja (auditul 2026-11-23).
# Se schimba doar odata cu regula 2 din CLAUDE.md, explicit.
ARBORE_MOSTENIRE=f7f1455dd4054a2bb310c1a971fe027f8c63dcc8
if git rev-parse --git-dir >/dev/null 2>&1; then
  actual="$(git rev-parse HEAD:mostenire 2>/dev/null || echo lipsa)"
  local_schimbat="$(git status --porcelain -- mostenire/ 2>/dev/null)"
  if [ "$actual" != "$ARBORE_MOSTENIRE" ]; then
    cade "mostenire/ s-a schimbat in commit-uri (arbore $actual) — vezi regula 2 din CLAUDE.md"
  elif [ -n "$local_schimbat" ]; then
    cade "mostenire/ e modificat local — vezi regula 2 din CLAUDE.md"
    printf '%s\n' "$local_schimbat" | sed 's/^/      /'
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
# Cele doua `genereaza.*` **scriu** fixture-urile. Pana la auditul din
# 2026-11-23 poarta le rula direct, deci isi rescria singura asteptarea pe
# care regula 2 din CLAUDE.md o declara intangibila: o schimbare in motorul de
# referinta dadea rosu o singura data, iar a doua rulare era verde pe fixture-ul
# nou — exact mecanismul prin care au divergat cele doua motoare din v3.4, pus
# in poarta. Acum fixture-ul se pune la loc si se **compara**: daca regenerarea
# l-ar schimba, poarta cade si spune sa se treaca prin
# `unelte/regenereaza-fixture.sh`, cu motivul in `JURNAL.md`.
harness() {
  local generator="$1" fixture="$2"; shift 2
  local copie; copie="$(mktemp)"
  cp "$fixture" "$copie"
  local ies rc
  ies="$("$@" 2>&1)"; rc=$?
  if [ $rc -ne 0 ]; then
    cp "$copie" "$fixture"; rm -f "$copie"
    printf '%s\n' "$ies"; return 1
  fi
  if ! cmp -s "$copie" "$fixture"; then
    diff -u "$copie" "$fixture" | head -20
    cp "$copie" "$fixture"; rm -f "$copie"
    printf 'regenerarea ar schimba %s. Daca e intentionat, ruleaza\n' "$fixture"
    printf 'unelte/regenereaza-fixture.sh ca pas propriu si noteaza motivul in JURNAL.md\n'
    return 1
  fi
  rm -f "$copie"
}
if [ "$MOD" = rapid ]; then
  sare "harness" "mod rapid"
else
  if command -v python3 >/dev/null; then
    ruleaza "motorul Python da acelasi fixture" \
      harness py referinta/rezultate_python.json python3 -I referinta/genereaza.py
  else
    sare "harness Python" "python3 nu e instalat"
  fi
  if command -v node >/dev/null; then
    ruleaza "motorul JS da acelasi fixture" \
      harness js referinta/rezultate_js.json node referinta/genereaza.mjs
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
