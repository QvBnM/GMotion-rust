#!/usr/bin/env bash
# Porneste `gmotion-ui`, pe Linux / WSL. Acelasi lucru ca `porneste.bat`.
# Fara ecran (X11 / Wayland) fereastra nu se poate deschide; pentru o privire
# fara ecran, fa o captura: vezi cercetare/machete/capturi/README.md.
set -euo pipefail
cd "$(dirname "$0")/.."

if [ ! -f referinta/instantanee/previzualizare.json ]; then
  echo "Generez instantaneul de previzualizare..."
  python3 -I unelte/genereaza-previzualizare.py
fi

echo "Compilez (prima data dureaza cateva minute)..."
cargo build --release --manifest-path rust/Cargo.toml -p gmotion-app

# fara `--server` si fara `--sursa`, pornim in previzualizare
argumente=()
previzualizare=1
for a in "$@"; do
  case "$a" in
    --server) previzualizare=0 ;;
    --sursa) previzualizare=0; argumente+=("$a") ;;
    *) argumente+=("$a") ;;
  esac
done
[ "$previzualizare" = 1 ] && argumente=(--previzualizare "${argumente[@]+"${argumente[@]}"}")

echo
echo "Pornesc gmotion-ui ${argumente[*]+${argumente[*]}}"
echo "  F1 ajutor · F11 ecran complet · Ctrl+F cauta · Enter deschide obiectul"
echo
exec rust/target/release/gmotion-ui ${argumente[@]+"${argumente[@]}"}
