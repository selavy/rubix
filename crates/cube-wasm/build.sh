#!/usr/bin/env bash
# Build the wasm module and generate JS bindings for the browser.
#
# Produces crates/cube-wasm/pkg/ containing cube_wasm.js + cube_wasm_bg.wasm.
# Pass a wasm-bindgen `--target` as $1 (default: web). Common values:
#   web     — ES module for browsers / bundlers (the frontend uses this)
#   nodejs  — CommonJS for Node (used by smoke_test.cjs)
set -euo pipefail

TARGET="${1:-web}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

cargo build -p cube-wasm --target wasm32-unknown-unknown --release
wasm-bindgen \
  "target/wasm32-unknown-unknown/release/cube_wasm.wasm" \
  --out-dir "crates/cube-wasm/pkg" \
  --target "$TARGET"

echo "Generated crates/cube-wasm/pkg (--target $TARGET)"
