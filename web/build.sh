#!/usr/bin/env bash
# Builds the web viewer into dist/, ready to be served as static assets.
#
# Needs: the wasm32-unknown-unknown target, wasm-bindgen-cli matching the wasm-bindgen
# version in Cargo.lock, and optionally wasm-opt (binaryen) for a faster binary.
set -euo pipefail

cd "$(dirname "$0")/.."

OUT=dist
WASM=target/wasm32-unknown-unknown/wasm-release/mandelbrot_web.wasm
# Cloudflare Workers static assets are capped at 25 MiB per file
MAX_BYTES=$((25 * 1024 * 1024))

cargo build -p mandelbrot-web --profile wasm-release --target wasm32-unknown-unknown

rm -rf "$OUT"
wasm-bindgen --out-dir "$OUT" --target web --no-typescript "$WASM"

if command -v wasm-opt >/dev/null; then
    # Only the features rustc emits for wasm32-unknown-unknown. `--all-features` lets
    # wasm-opt use proposals browsers don't ship yet, which makes the module fail to load.
    wasm-opt -O3 \
        --enable-bulk-memory --enable-bulk-memory-opt \
        --enable-reference-types --enable-call-indirect-overlong \
        --enable-multivalue --enable-mutable-globals \
        --enable-nontrapping-float-to-int --enable-sign-ext \
        "$OUT/mandelbrot_web_bg.wasm" -o "$OUT/mandelbrot_web_bg.wasm"
else
    echo "warning: wasm-opt not found, skipping optimization" >&2
fi

cp web/static/* "$OUT/"

size=$(wc -c < "$OUT/mandelbrot_web_bg.wasm")
echo "mandelbrot_web_bg.wasm: $((size / 1024)) KiB"
if [ "$size" -gt "$MAX_BYTES" ]; then
    echo "error: wasm is over Cloudflare's 25 MiB asset limit" >&2
    exit 1
fi
