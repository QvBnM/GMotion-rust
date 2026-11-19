#!/usr/bin/env bash
# Porneste `gmotion-ui`, pe Linux / WSL. Acelasi lucru ca `porneste.bat`:
# implicit ecranul de conectare; `--previzualizare` pentru instantaneul
# sintetic. Fara ecran (X11 / Wayland) fereastra nu se poate deschide; pentru
# o privire fara ecran, fa o captura: vezi cercetare/machete/capturi/README.md.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "Compilez (prima data dureaza cateva minute)..."
cargo build --release --manifest-path rust/Cargo.toml -p gmotion-app

echo
echo "Pornesc gmotion-ui $*"
echo "  F1 ajutor · F11 ecran complet · Ctrl+F cauta · Enter deschide obiectul"
echo
exec rust/target/release/gmotion-ui "$@"
