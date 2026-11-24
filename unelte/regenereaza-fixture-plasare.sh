#!/usr/bin/env bash
# Regenereaza fixture-ul de paritate pentru stratul de plasare
# (`referinta/plasare_js.json`), ruland pagina v3.4 adevarata.
#
# PAS EXPLICIT, niciodata automat — vezi `unelte/regenereaza-fixture.sh`: un
# fixture regenerat ca sa treaca un test e exact mecanismul prin care au
# divergat cele doua motoare din v3.4.
#
# Plasarea nu se poate extrage mecanic din `ui.py`, ca motorul de diff: tine de
# starea paginii. Deci pagina se genereaza din `mostenire/ui.py` si se deschide
# intr-un browser fara cap, prin „comparare offline", care e complet locala.
#
# Cere `node` si `playwright`. **Poarta nu regenereaza nimic** si n-are niciun
# pas de plasare: fixture-ul `referinta/plasare_js.json` e commitat si citit de
# `rust/core/tests/plasare.rs`, iar regenerarea e numai de aici, explicit, cu
# motivul in `JURNAL.md` (regula 2). Pana la auditul 2026-11-23 comentariul
# asta si `STARE.md` spuneau ca poarta sare verificarea, ceea ce nu era
# adevarat: nu exista verificarea.
set -euo pipefail

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

command -v node >/dev/null || { echo "node lipseste" >&2; exit 1; }
node -e "require('playwright')" 2>/dev/null || { echo "playwright lipseste" >&2; exit 1; }

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "== pagina v3.4 din mostenire/ui.py"
python3 -B -I -c "
import sys; sys.path.insert(0, 'mostenire')
import ui
open('$tmp/v34.html', 'w', encoding='utf-8').write(ui.genereaza_pagina())
"

echo "== sugestiile de plasare, caz cu caz"
node referinta/genereaza_plasare.cjs "$tmp/v34.html" referinta/cazuri.json referinta/plasare_js.json

cat <<'NOTA'

Mai departe:
  1. verifica `git diff referinta/plasare_js.json`
  2. noteaza in JURNAL.md de ce a fost regenerat
  3. cd rust && cargo test -p gmotion-core --test plasare
NOTA
