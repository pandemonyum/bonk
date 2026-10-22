#!/usr/bin/env sh
# Gate qualità: formattazione, clippy senza warning, test, validazione asset.
# Uso: ./scripts/check.sh        Esce con codice != 0 al primo controllo fallito.
set -eu
cd "$(dirname "$0")/.."

PYTHON=$(command -v python3 || command -v python)

echo "==> fmt"    && cargo fmt --all -- --check
echo "==> clippy" && cargo clippy --all-targets --locked -- -D warnings
echo "==> test"   && cargo test --locked
echo "==> assets" && "$PYTHON" scripts/validate-assets.py
echo "OK: tutti i controlli passati"
