#!/usr/bin/env bash
# Amprenta API-ului public al unui crate. Serveste la o singura verificare:
# daca API-ul s-a schimbat, `CRATE.md` trebuie actualizat in acelasi commit.
#
# Cauza de eroare proprie: documentatia care rămâne in urma codului. E singurul
# fel de bug pe care nu-l prinde niciun test.
#
#   unelte/api-hash.sh rust/core
set -euo pipefail

crate="${1:?folosire: unelte/api-hash.sh <cale-crate>}"
src="$crate/src"

[ -d "$src" ] || { echo "nu exista $src" >&2; exit 1; }

# Liniile care declara ceva public. Elementele din `mod teste` nu sunt publice,
# deci nu intra. Normalizam spatiile ca reformatarea sa nu schimbe amprenta.
find "$src" -name '*.rs' -print0 \
  | sort -z \
  | xargs -0 grep -hE '^[[:space:]]*pub([[:space:]]|\()' \
  | sed -E 's/[[:space:]]+/ /g; s/^ //; s/ $//' \
  | sort \
  | sha256sum \
  | cut -c1-16
