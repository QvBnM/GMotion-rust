#!/usr/bin/env bash
# Regenereaza fixture-urile harness-ului de referinta.
#
# PAS EXPLICIT, niciodata automat. Un fixture regenerat ca sa treaca un test e
# exact mecanismul prin care au divergat cele doua motoare din v3.4. Daca un
# test de conformitate cade, intai presupui ca portul Rust e gresit.
#
# Se ruleaza doar cand:
#   - s-a adaugat un caz in referinta/cazuri.json, sau
#   - s-a schimbat mostenire/ui.py (si motor_js.mjs a fost RE-EXTRAS, nu rescris)
#
# Dupa rulare: noteaza motivul in JURNAL.md, si verifica diff-ul fixture-ului
# inainte de commit. Un fixture care se schimba fara motiv e un semnal de alarma.
set -euo pipefail

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

command -v node >/dev/null || { echo "node lipseste — fixture-ul de referinta nu poate fi regenerat" >&2; exit 1; }

echo "== fixture de referinta (motorul JS din mostenire/ui.py)"
node referinta/genereaza.mjs

echo "== rezultatele motorului Python (pentru raportul de divergente)"
python3 -I referinta/genereaza.py

echo "== raport de divergente Python vs JS"
python3 -I referinta/compara.py || true

cat <<'NOTA'

Mai departe:
  1. verifica `git diff referinta/rezultate_js.json` — se schimba doar ce te asteptai?
  2. noteaza in JURNAL.md de ce a fost regenerat
  3. cd rust && cargo test -p gmotion-core
NOTA
