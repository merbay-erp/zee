#!/usr/bin/env bash
# K-191: çağıranın varsayılanı yerine depoda kayıtlı exact Rust sürümü.
# Bu dosya source edilir; araç zinciri kurulmaz veya yükseltilmez.
zee_rust_arac_zinciri() {
    local ayar="$1"
    local surum gercek kanal_sayisi rustc_yolu
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
    # K-199: RUSTUP_TOOLCHAIN tek başına Cargo'nun RUSTC/config/wrapper
    # seçimini yönetmez. Derleyiciyi exact kurulu toolchain'den çöz.
    rustc_yolu="$(rustup which --toolchain "$surum" rustc)" || return 1
    case "$rustc_yolu" in
        [A-Za-z]:*) rustc_yolu="$(cygpath -m "$rustc_yolu")" || return 1 ;;
    esac
    [[ -n "$rustc_yolu" && -x "$rustc_yolu" ]] || {
        echo "Exact Rust derleyici yolu yürütülebilir değil: $rustc_yolu" >&2
        return 1
    }
    gercek="$("$rustc_yolu" --version)" || return 1
    if [[ "$gercek" != "rustc $surum "* ]]; then
        echo "Cargo Rust derleyicisi pine uymuyor: $gercek" >&2
        return 1
    fi
    export RUSTC="$rustc_yolu"
    # Boş değer Cargo config dosyası ve CARGO_BUILD_* wrapper'larını da ezer.
    export RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER=""
}
