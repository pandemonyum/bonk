#!/usr/bin/env bash
# Build web del gioco in dist/ (P1): wasm-release per wasm32, wasm-bindgen alla versione del
# Cargo.lock, wasm-opt se installato, poi pagina, asset e dati. Uso: ./scripts/build-web.sh
# Prova: python3 -m http.server 8080 -d dist  e apri http://localhost:8080 (mai file://).
set -euo pipefail
cd "$(dirname "$0")/.."

want=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"/\1/p')
have=$(wasm-bindgen --version | sed 's/wasm-bindgen //')
if [ "$have" != "$want" ]; then
  echo "wasm-bindgen $have, il Cargo.lock vuole $want: cargo install wasm-bindgen-cli --version $want --locked" >&2
  exit 1
fi

# versione di asset e dati (contenuto dei file): il gioco li cerca in cartelle con questo nome,
# così il browser non usa dalla cache quelli di una build vecchia (src/web.rs)
version=$(find assets game-data -type f | LC_ALL=C sort | while read -r f; do printf '%s' "/$f"; cat "$f"; done | sha256sum | cut -c1-10)
export PMS_WEB_VERSION="$version"
echo "==> cargo build (wasm-release, asset e dati $version)"
cargo build --profile wasm-release --target wasm32-unknown-unknown --bin pixel-monster-survivor
raw=target/wasm32-unknown-unknown/wasm-release/pixel-monster-survivor.wasm

# dist/ è generata: si ricrea da zero (è in .gitignore)
rm -rf dist
mkdir dist

echo "==> wasm-bindgen"
wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
  --out-dir dist --out-name game "$raw"

wasm=dist/game_bg.wasm
if command -v wasm-opt >/dev/null; then
  echo "==> wasm-opt -Os"
  wasm-opt -Os --strip-debug -o "$wasm.opt" "$wasm"
  mv -f "$wasm.opt" "$wasm"
else
  echo "(wasm-opt non installato: .wasm non ottimizzato oltre il profilo)"
fi

echo "==> pagina, asset e dati"
hash=$(sha256sum "$wasm" | cut -c1-12)
sed "s/__VERSION__/$hash/g" web/index.html > dist/index.html
cp -r assets "dist/assets-$version"
cp -r game-data "dist/game-data-$version"

mib() { awk -v b="$1" 'BEGIN { printf "%.1f MiB", b / 1048576 }'; }
echo "wasm: grezzo $(mib "$(stat -c%s "$raw")"), in dist $(mib "$(stat -c%s "$wasm")"), gzip $(mib "$(gzip -9c "$wasm" | wc -c)")"
biggest=$(find dist -type f -printf '%s %p\n' | sort -n | tail -1)
echo "dist: $(mib "$(du -sb dist | cut -f1)") in tutto, il file più grande ${biggest#* } ($(mib "${biggest%% *}"))"
if [ "${biggest%% *}" -gt $((25 * 1048576)) ]; then
  echo "un file di dist supera 25 MiB" >&2
  exit 1
fi
echo "OK: dist/ pronta (python3 -m http.server 8080 -d dist)"
