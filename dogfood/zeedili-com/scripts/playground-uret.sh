#!/usr/bin/env bash
set -euo pipefail

BETIK_KOKU="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SITE_KOKU="$(cd -- "$BETIK_KOKU/.." && pwd)"
DEPO_KOKU="$(cd -- "$SITE_KOKU/../.." && pwd)"
HEDEF="$DEPO_KOKU/compiler/target/wasm32-unknown-unknown/release/dil.wasm"
CIKTI_KLASORU="$SITE_KOKU/static/playground"

cd "$DEPO_KOKU/compiler"
cargo rustc --locked --release --target wasm32-unknown-unknown --lib \
  --crate-type cdylib -- -C link-arg=-zstack-size=16777216

install -d -m 0755 "$CIKTI_KLASORU"
install -m 0644 "$HEDEF" "$CIKTI_KLASORU/dil.wasm"
(cd "$CIKTI_KLASORU" && shasum -a 256 dil.wasm > dil.wasm.sha256)
node "$DEPO_KOKU/scripts/wasm-abi-denetle.mjs" "$CIKTI_KLASORU/dil.wasm"

echo "zeedili.com playground WASM varlığı ve SHA-256 kaydı üretildi."
