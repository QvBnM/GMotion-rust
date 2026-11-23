#!/usr/bin/env bash
# Care e ultima versiune a proiectului: branch-ul de pe origin cu cel mai nou
# commit, si unde e branch-ul curent fata de el.
#
# Cauza de eroare proprie: doua sesiuni au lucrat in paralel pe branch-uri
# pornite din acelasi punct, iar una a lucrat (si a auditat) pe o versiune
# veche fara sa stie (2026-11-23). Primul pas la pornire, inainte de orice.
#
#   unelte/ultima-versiune.sh
set -uo pipefail
cd "$(dirname "$0")/.."

if ! git fetch --prune --quiet origin 2>/dev/null; then
  echo "ATENTIE: git fetch a esuat; lista de mai jos poate fi veche."
fi

curent="$(git rev-parse --abbrev-ref HEAD)"
echo "Branch-uri pe origin, cel mai nou primul:"
git for-each-ref --sort=-committerdate \
  --format='  %(committerdate:short) %(committerdate:relative)  %(refname:short)  %(subject)' \
  refs/remotes/origin | grep -v 'origin/HEAD' | head -8

cel_nou="$(git for-each-ref --sort=-committerdate --format='%(refname:short)' refs/remotes/origin \
  | grep -v '^origin/HEAD$' | head -1)"
echo
echo "Ultima versiune: ${cel_nou#origin/}"
echo "Branch curent:   $curent"

if [ "origin/$curent" = "$cel_nou" ] || [ "$curent" = "${cel_nou#origin/}" ]; then
  in_urma="$(git rev-list --count "HEAD..$cel_nou")"
  if [ "$in_urma" -gt 0 ]; then
    echo "ATENTIE: esti cu $in_urma commit-uri in urma: git pull origin $curent"
  else
    echo "Esti pe branch-ul cel mai nou dupa data."
  fi
else
  lipsesc="$(git rev-list --count "HEAD..$cel_nou")"
  in_plus="$(git rev-list --count "$cel_nou..HEAD")"
  echo "ATENTIE: NU lucrezi pe ultima versiune."
  echo "  ${cel_nou#origin/} are $lipsesc commit-uri care nu sunt aici;"
  echo "  aici sunt $in_plus commit-uri care nu sunt acolo."
  echo "  Spune-i utilizatorului inainte de orice lucru: de unde se porneste,"
  echo "  si daca branch-urile trebuie unite."
fi

# alte branch-uri cu munca neadusa in ultima versiune (lucru in paralel):
# atunci nu exista o singura „ultima versiune", ci doua jumatati de munca
paralel=0
for b in $(git for-each-ref --format='%(refname:short)' refs/remotes/origin | grep -v 'origin/HEAD'); do
  [ "$b" = "$cel_nou" ] && continue
  n="$(git rev-list --count "$cel_nou..$b")"
  baza="$(git merge-base "$cel_nou" "$b" 2>/dev/null)"
  # branch-urile fara istoric comun (fork-uri vechi) nu sunt lucru paralel
  if [ -n "$baza" ] && [ "$n" -gt 0 ]; then
    echo "LUCRU PARALEL: ${b#origin/} are $n commit-uri care nu sunt in ${cel_nou#origin/}."
    paralel=1
  fi
done
if [ "$paralel" = 1 ]; then
  echo
  echo "ATENTIE: munca e impartita pe mai multe branch-uri neunite; cel mai nou"
  echo "dupa data NU le contine pe toate. Spune-i utilizatorului si cere-i sa"
  echo "aleaga: se unesc intai, sau se lucreaza pe unul anume. Nu porni de pe"
  echo "unul la intamplare."
  exit 2
fi
