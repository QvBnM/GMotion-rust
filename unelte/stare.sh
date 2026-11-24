#!/usr/bin/env bash
# Faptele mecanice de pus in STARE.md. Rost: actualizarea sa fie ieftina —
# un document scump de intretinut moare in doua saptamani.
#
# Nu scrie nimic. Afiseaza; antetul se copiaza in STARE.md.
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

head_scurt="$(git rev-parse --short HEAD 2>/dev/null || echo '?')"
branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo '?')"
azi="$(date +%F)"

activa="$(grep -lE '^\*\*Stare:\*\* ACTIVA' etape/*.md 2>/dev/null | grep -v SABLON || true)"
if [ -z "$activa" ]; then
  # „exact o etapa ACTIVA" (CLAUDE.md §0): zero e la fel de rau ca doua, si
  # poarta cade pe asta. Se spune aici, nu se trece ca stare normala.
  activa="(NICIUNA — poarta cade; deschide una din etape/SABLON.md)"
fi

# Din rezumatele `test result:`, nu din randurile `test ... ok`: jurnalul
# testelor se poate intercala pe acelasi rand, iar `grep -c` fara potriviri
# iese cu 1, deci `|| echo '?'` scria si `0` si `?` (D-21, auditul
# 2026-11-23). Aceeasi metoda ca in `unelte/verifica.sh`.
teste="$(cd rust && cargo test --all 2>/dev/null \
  | sed -nE 's/^test result: ok\. ([0-9]+) passed.*/\1/p' \
  | awk '{s+=$1} END {print s+0}')"
[ -z "$teste" ] && teste='?'

printf '\nAntetul pentru STARE.md:\n\n'
printf '**Actualizat:** %s · commit `%s` · branch `%s`\n' "$azi" "$head_scurt" "$branch"
printf '**Poarta:** <ruleaza unelte/verifica.sh si scrie verde/rosie> (%s teste)\n' "$teste"

printf '\nEtapa activa: %s\n' "$activa"

if [ -f STARE.md ]; then
  ultim="$(git log -1 --format=%h -- STARE.md 2>/dev/null)"
  if [ -n "$ultim" ]; then
    n="$(git rev-list --count "$ultim..HEAD" 2>/dev/null || echo 0)"
    printf 'STARE.md: ultima actualizare in %s, acum %s commit-uri in urma\n' "$ultim" "$n"
  else
    printf 'STARE.md: inca necommitat\n'
  fi
fi

printf '\nCommit-uri de cand a fost actualizat STARE.md:\n'
git log --oneline "$(git log -1 --format=%H -- STARE.md 2>/dev/null)..HEAD" 2>/dev/null | sed 's/^/  /' || true
printf '\n'
