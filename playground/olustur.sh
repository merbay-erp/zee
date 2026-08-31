#!/bin/sh
# zee playground üretici: derleyiciyi wasm'a derler, tek dosyalık
# zee-playground.html üretir (çift tıkla açılır; sunucu gerekmez).
set -e
KOK="$(cd "$(dirname "$0")/.." && pwd)"
cd "$KOK/compiler"
# cdylib yalnız wasm hedefinde (K-040: Cargo.toml'da bildirmek Windows'ta
# PDB çakışması uyarısı üretiyordu). Yığın 16 MB: C019 sınırı (1000) bol sığar.
cargo rustc --release --target wasm32-unknown-unknown --lib --crate-type cdylib \
    -- -C link-arg=-zstack-size=16777216
SURUM="$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)"
WASM="$KOK/compiler/target/wasm32-unknown-unknown/release/dil.wasm"
B64="$(base64 < "$WASM" | tr -d '\n')"
python3 - "$KOK" "$SURUM" "$B64" <<'PY'
import sys
kok, surum, b64 = sys.argv[1], sys.argv[2], sys.argv[3]
sablon = open(f"{kok}/playground/sablon.html", encoding="utf-8").read()
cikti = sablon.replace("__WASM_B64__", b64).replace("__SURUM__", surum)
open(f"{kok}/playground/zee-playground.html", "w", encoding="utf-8").write(cikti)
print(f"üretildi: playground/zee-playground.html ({len(cikti)//1024} KB)")
PY
