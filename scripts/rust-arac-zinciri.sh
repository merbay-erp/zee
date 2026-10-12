#!/usr/bin/env bash
# K-191: çağıranın varsayılanı yerine depoda kayıtlı exact Rust sürümü.
# Bu dosya source edilir; araç zinciri kurulmaz veya yükseltilmez.
zee_rust_arac_zinciri() {
    local ayar="$1"
    local surum gercek kanal_sayisi
    surum="$(sed -nE 's/^[[:space:]]*channel[[:space:]]*=[[:space:]]*"([0-9]+\.[0-9]+\.[0-9]+)"[[:space:]]*(#.*)?$/\1/p' "$ayar")" || return 1
    kanal_sayisi="$(sed -nE '/^[[:space:]]*channel[[:space:]]*=/p' "$ayar" | wc -l | tr -d ' ')" || return 1
    if [[ "$kanal_sayisi" != 1 || ! "$surum" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Rust araç zinciri tek bir exact X.Y.Z channel kaydı ister: $ayar" >&2
        return 1
    fi
    export RUSTUP_TOOLCHAIN="$surum"
    gercek="$(rustc --version)" || return 1
    if [[ "$gercek" != "rustc $surum "* ]]; then
        echo "Rust araç zinciri uyuşmuyor: beklenen $surum, bulunan $gercek" >&2
        return 1
    fi
}
