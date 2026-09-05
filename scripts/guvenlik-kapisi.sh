#!/usr/bin/env bash
# K-170/ADR-066 birleşik güvenlik kapısı.
#   --surekli     her push: cargo-deny, fuzz derlemesi, bulgu kaydı ve tedarik kapısı
#   --surum-adayi ayrıca temiz ağaç, exact HEAD RC fuzz kanıtı, clippy ve tam test
set -euo pipefail

depo_koku="$(git rev-parse --show-toplevel)"
cd "$depo_koku"

kip="${1:---surekli}"
case "$kip" in
    --surekli | --surum-adayi) ;;
    *)
        echo "Kullanım: scripts/guvenlik-kapisi.sh [--surekli|--surum-adayi]" >&2
        exit 2
        ;;
esac
echo "== Güvenlik kapısı ($kip)"

if ! command -v cargo-deny >/dev/null 2>&1; then
    echo "cargo-deny 0.20.2 gerekli: cargo install cargo-deny --version 0.20.2 --locked" >&2
    exit 1
fi
deny_surumu="$(cargo deny --version | awk '{print $2}')"
if [[ "$deny_surumu" != "0.20.2" ]]; then
    echo "cargo-deny sürümü 0.20.2 olmalı: $deny_surumu" >&2
    exit 1
fi

echo "-- RustSec/lisans/ban/kaynak (compiler + fuzz)"
(
    cd compiler
    cargo deny --locked check -D warnings
    cargo deny --manifest-path fuzz/Cargo.toml --config deny.toml --locked check -D warnings
)

echo "-- Fuzz hedefleri stable araç zincirinde derlenir"
(cd compiler && cargo check --manifest-path fuzz/Cargo.toml --locked --bins)

echo "-- Bulgu kaydı, açık kritik/yüksek sıfır ve tedarik kapısı"
(cd compiler && cargo test --locked --test guvenlik_kapisi_testi --test tedarik_kapisi_testi)

if [[ "$kip" == "--surum-adayi" ]]; then
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "Sürüm adayı kapısı temiz çalışma ağacı ister." >&2
        exit 1
    fi
    sha="$(git rev-parse HEAD)"
    echo "-- Exact $sha için RC fuzz kanıtı (docs/fuzz-rc-gecmisi-v1.tsv)"
    for hedef in lexer_parser morfoloji http_istegi wasm_abi; do
        if ! awk -F '\t' -v sha="$sha" -v hedef="$hedef" \
            '$1 == sha && $9 == hedef && $16 == "gecti" && $11 >= 1800 { ok = 1 } END { exit ok ? 0 : 1 }' \
            docs/fuzz-rc-gecmisi-v1.tsv; then
            echo "RC fuzz kanıtı eksik ya da yetersiz: $hedef @ $sha (K-157/ADR-056)" >&2
            exit 1
        fi
    done
    echo "-- Clippy ve tam test paketi"
    (
        cd compiler
        cargo clippy --locked --all-targets -- -D warnings
        cargo test --locked --no-fail-fast
    )
    echo "-- Yayımlanmış kanıt korukları"
    taban="$(git rev-parse HEAD~1)"
    bash scripts/conformance-korugu.sh "$taban"
    bash scripts/semantic-regresyon-korugu.sh "$taban"
    bash scripts/core-freeze-korugu.sh "$taban"
fi

echo "Güvenlik kapısı geçti ($kip)."
