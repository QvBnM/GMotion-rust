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
# Cere `node` si `playwright`. Daca lipsesc, poarta sare verificarea.
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
