#!/usr/bin/env sh
# Ripulisce le copie vecchie del nostro crate in target/ (eseguibili e simboli di gioco e test).
# Cargo ne crea una nuova a ogni cambio di dipendenze, feature o profilo e non cancella le
# precedenti. Bevy e le altre dipendenze restano: niente ricompilazione lunga.
# Uso: ./scripts/clean-stale.sh          Pulizia totale (ricompila tutto, ~20 min): cargo clean
set -eu
cd "$(dirname "$0")/.."

size() { du -sh target 2>/dev/null | cut -f1 || echo 0; }

before=$(size)
cargo clean -p pixel-monster-survivor
echo "target/: $before -> $(size)"
