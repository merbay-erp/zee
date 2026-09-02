#!/usr/bin/env bash
set -euo pipefail

depo_koku="$(git rev-parse --show-toplevel)"
manifest="$depo_koku/compiler/Cargo.toml"
kilit="$depo_koku/compiler/Cargo.lock"
fuzz_manifesti="$depo_koku/compiler/fuzz/Cargo.toml"
fuzz_kilidi="$depo_koku/compiler/fuzz/Cargo.lock"
gecici_klasor="$(mktemp -d "${TMPDIR:-/tmp}/zee-offline-vendor.XXXXXX")"
trap 'rm -rf "$gecici_klasor"' EXIT

sha256_dosya() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

vendor_manifesti() {
    local vendor_koku="$1"
    local cikti="$2"
    : > "$cikti"
    LC_ALL=C find "$vendor_koku" -type f -print | LC_ALL=C sort |
        while IFS= read -r dosya; do
            goreli="${dosya#"$vendor_koku"/}"
            printf '%s  %s\n' "$(sha256_dosya "$dosya")" "$goreli" >> "$cikti"
        done
}

kilit_once="$(sha256_dosya "$kilit")"
fuzz_kilidi_once="$(sha256_dosya "$fuzz_kilidi")"
for tur in bir iki; do
    vendor="$gecici_klasor/vendor-$tur"
    ayar="$gecici_klasor/config-$tur.toml"
    gunluk="$gecici_klasor/vendor-$tur.log"
    if ! cargo vendor --locked --versioned-dirs --manifest-path "$manifest" \
        --sync "$fuzz_manifesti" "$vendor" > "$ayar" 2> "$gunluk"; then
        cat "$gunluk" >&2
        exit 1
    fi
    vendor_manifesti "$vendor" "$gecici_klasor/manifest-$tur.sha256"
done

if ! cmp -s "$gecici_klasor/manifest-bir.sha256" \
    "$gecici_klasor/manifest-iki.sha256"; then
    echo "Aynı Cargo.lock iki farklı vendor içeriği üretti." >&2
    diff -u "$gecici_klasor/manifest-bir.sha256" \
        "$gecici_klasor/manifest-iki.sha256" >&2 || true
    exit 1
fi

mkdir "$gecici_klasor/cargo-home"
CARGO_HOME="$gecici_klasor/cargo-home" \
    CARGO_TARGET_DIR="$gecici_klasor/target" \
    CARGO_NET_OFFLINE=true \
    cargo check --quiet --locked --offline --all-targets --all-features \
        --manifest-path "$manifest" --config "$gecici_klasor/config-bir.toml"
CARGO_HOME="$gecici_klasor/cargo-home" \
    CARGO_TARGET_DIR="$gecici_klasor/fuzz-target" \
    CARGO_NET_OFFLINE=true \
    cargo check --quiet --locked --offline --bins \
        --manifest-path "$fuzz_manifesti" --config "$gecici_klasor/config-bir.toml"

kilit_sonra="$(sha256_dosya "$kilit")"
fuzz_kilidi_sonra="$(sha256_dosya "$fuzz_kilidi")"
if [[ "$kilit_once" != "$kilit_sonra" || \
    "$fuzz_kilidi_once" != "$fuzz_kilidi_sonra" ]]; then
    echo "Vendor/offline denetimi Cargo.lock dosyasını değiştirdi." >&2
    exit 1
fi

paket_sayisi="$(find "$gecici_klasor/vendor-bir" -mindepth 1 -maxdepth 1 \
    -type d | wc -l | tr -d ' ')"
manifest_ozeti="$(sha256_dosya "$gecici_klasor/manifest-bir.sha256")"
echo "Offline vendor doğrulandı: $paket_sayisi paket, manifest SHA-256 $manifest_ozeti"
