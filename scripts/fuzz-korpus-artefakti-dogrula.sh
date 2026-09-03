#!/usr/bin/env bash
set -euo pipefail

hedef="${1:-}"
artefakt="${2:-}"
if [[ -z "$hedef" || -z "$artefakt" ]]; then
    echo "Kullanım: scripts/fuzz-korpus-artefakti-dogrula.sh <hedef> <artefakt-klasoru>" >&2
    exit 2
fi
case "$hedef" in
    lexer_parser|morfoloji|http_istegi|wasm_abi) ;;
    *)
        echo "Bilinmeyen fuzz hedefi: $hedef" >&2
        exit 2
        ;;
esac

manifest="$artefakt/manifest.tsv"
korpus="$artefakt/corpus"
if [[ ! -f "$manifest" || ! -d "$korpus" ]]; then
    echo "Fuzz korpus artefaktı manifest.tsv ve corpus/ taşımalı: $artefakt" >&2
    exit 1
fi
if [[ "$(head -n 1 "$manifest")" != "# zee-fuzz-corpus-artifact-1" ]]; then
    echo "Bilinmeyen fuzz korpus artefakt şeması" >&2
    exit 1
fi
manifest_hedefi="$(awk -F '\t' '$1 == "# target" { print $2 }' "$manifest")"
if [[ "$manifest_hedefi" != "$hedef" ]]; then
    echo "Fuzz hedefi eşleşmiyor: beklenen=$hedef manifest=$manifest_hedefi" >&2
    exit 1
fi
manifest_korpusu="$(awk -F '\t' '$1 == "# corpus" { print $2 }' "$manifest")"
kaynak_commit="$(awk -F '\t' '$1 == "# source_commit" { print $2 }' "$manifest")"
run_id="$(awk -F '\t' '$1 == "# run_id" { print $2 }' "$manifest")"
run_attempt="$(awk -F '\t' '$1 == "# run_attempt" { print $2 }' "$manifest")"
toolchain="$(awk -F '\t' '$1 == "# toolchain" { print $2 }' "$manifest")"
cargo_fuzz="$(awk -F '\t' '$1 == "# cargo_fuzz" { print $2 }' "$manifest")"
if [[ "$manifest_korpusu" != "$hedef" \
    || ! "$kaynak_commit" =~ ^[0-9a-f]{40}$ \
    || ! "$run_id" =~ ^[0-9]+$ || ! "$run_attempt" =~ ^[0-9]+$ \
    || "$toolchain" != "nightly-2026-08-31" || "$cargo_fuzz" != "0.13.2" ]]; then
    echo "Fuzz korpus provenance metadata'sı eksik veya geçersiz" >&2
    exit 1
fi

sha256_dosya() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d ' ' -f 1
    else
        shasum -a 256 "$1" | cut -d ' ' -f 1
    fi
}

gecici="$(mktemp -d)"
trap 'rm -rf "$gecici"' EXIT
manifest_yollari="$gecici/manifest-yollari"
agac_yollari="$gecici/agac-yollari"
: > "$manifest_yollari"
manifest_sayisi=0
while IFS=$'\t' read -r beklenen goreli fazladan; do
    [[ -z "$beklenen" || "$beklenen" == \#* ]] && continue
    if [[ -n "${fazladan:-}" || ! "$beklenen" =~ ^[0-9a-f]{64}$ \
        || -z "$goreli" || "$goreli" == /* || "$goreli" == *..* \
        || "$goreli" == *\\* ]]; then
        echo "Geçersiz fuzz seed manifest satırı: $beklenen $goreli" >&2
        exit 1
    fi
    dosya="$korpus/$goreli"
    if [[ ! -f "$dosya" ]]; then
        echo "Manifestteki fuzz seed'i yok: $goreli" >&2
        exit 1
    fi
    gercek="$(sha256_dosya "$dosya")"
    if [[ "$gercek" != "$beklenen" ]]; then
        echo "Fuzz seed özeti uyuşmuyor: $goreli" >&2
        exit 1
    fi
    printf '%s\n' "$goreli" >> "$manifest_yollari"
    manifest_sayisi=$((manifest_sayisi + 1))
done < "$manifest"

LC_ALL=C sort "$manifest_yollari" -o "$manifest_yollari"
find "$korpus" -type f | sed "s|^$korpus/||" | LC_ALL=C sort > "$agac_yollari"
if ! cmp -s "$manifest_yollari" "$agac_yollari"; then
    echo "Fuzz korpusu ile manifest yolları birebir değil" >&2
    exit 1
fi

echo "$hedef fuzz korpus artefaktı doğrulandı: $manifest_sayisi seed."
