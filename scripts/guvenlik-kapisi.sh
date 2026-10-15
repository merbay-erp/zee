#!/usr/bin/env bash
# K-170/ADR-066 birleşik güvenlik kapısı.
#   --surekli     her push: cargo-deny, fuzz derlemesi, bulgu kaydı ve tedarik kapısı
#   --surum-adayi ayrıca temiz ağaç, exact HEAD RC fuzz kanıtı, clippy ve tam test
set -euo pipefail

depo_koku="$(git rev-parse --show-toplevel)"
cd "$depo_koku"

kip="${1:---surekli}"
if [[ $# -gt 0 ]]; then shift; fi
rc_kanit=""
rc_kanit_sha=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --rc-kanit)
            [[ -z "$rc_kanit" && $# -ge 2 && -n "$2" ]] || exit 2
            rc_kanit="$2"; shift 2 ;;
        --rc-kanit-sha256)
            [[ -z "$rc_kanit_sha" && $# -ge 2 && -n "$2" ]] || exit 2
            rc_kanit_sha="$2"; shift 2 ;;
        *) echo "Bilinmeyen güvenlik kapısı seçeneği." >&2; exit 2 ;;
    esac
done
case "$kip" in --surekli|--surum-adayi) ;; *) exit 2 ;; esac
if [[ -n "$rc_kanit" || -n "$rc_kanit_sha" ]]; then
    [[ "$kip" == "--surum-adayi" && "$rc_kanit" == /* && "$rc_kanit_sha" =~ ^[0-9a-f]{64}$ ]] || exit 2
fi
# RC source must precede every gate/tool command, not only the evidence step.
if [[ "$kip" == "--surum-adayi" ]]; then
    [[ -z "$(git status --porcelain --untracked-files=all)" ]] || {
        echo "Sürüm adayı kapısı temiz çalışma ağacı ister." >&2; exit 1;
    }
    sha="$(git rev-parse HEAD)"
    kaynak_agaci="$(git rev-parse 'HEAD^{tree}')"
fi
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
    [[ "$(git rev-parse HEAD)" == "$sha" && "$(git rev-parse 'HEAD^{tree}')" == "$kaynak_agaci" ]] || {
        echo "Sürüm adayı kaynağı erken kapı adımlarında değişti." >&2; exit 1;
    }
    if [[ -n "$rc_kanit" ]]; then
        python3 -I -B scripts/rc-kaniti.py check --framework-root "$depo_koku" \
            --bundle "$rc_kanit" --sha256 "$rc_kanit_sha"
    else
        echo "-- Exact $sha için RC fuzz kanıtı (docs/fuzz-rc-gecmisi-v1.tsv)"
        for hedef in lexer_parser morfoloji http_istegi wasm_abi; do
            if ! awk -F '\t' -v sha="$sha" -v hedef="$hedef" \
                '$1 == sha && $9 == hedef && $16 == "gecti" && $11 >= 1800 { ok = 1 } END { exit ok ? 0 : 1 }' \
                docs/fuzz-rc-gecmisi-v1.tsv; then
                echo "RC fuzz kanıtı eksik ya da yetersiz: $hedef @ $sha (K-157/ADR-056)" >&2
                exit 1
            fi
        done
    fi
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
    # The same exact source and external raw bytes must survive all gate steps.
    [[ "$(git rev-parse HEAD)" == "$sha" && "$(git rev-parse 'HEAD^{tree}')" == "$kaynak_agaci" && -z "$(git status --porcelain --untracked-files=all)" ]] || {
        echo "Sürüm adayı kaynağı kapı sırasında değişti." >&2; exit 1;
    }
    if [[ -n "$rc_kanit" ]]; then
        python3 -I -B scripts/rc-kaniti.py check --framework-root "$depo_koku" \
            --bundle "$rc_kanit" --sha256 "$rc_kanit_sha"
    fi

fi

echo "Güvenlik kapısı geçti ($kip)."
