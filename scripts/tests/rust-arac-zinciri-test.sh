#!/usr/bin/env bash
# K-191: gerçek giriş betikleri, farklı varsayılan ve negatif pin vakaları.
set -euo pipefail
depo_koku="$(cd "$(dirname "$0")/../.." && pwd)"
gecici="$(mktemp -d "${TMPDIR:-/tmp}/zee-toolchain-test.XXXXXX")"
trap 'rm -rf "$gecici"' EXIT
mkdir -p "$gecici/bin" "$gecici/depo/scripts" "$gecici/depo/compiler/fuzz"
cat > "$gecici/bin/rustc" <<'RUSTC'
#!/usr/bin/env bash
set -euo pipefail
surum="${RUSTUP_TOOLCHAIN:-9.9.9}"
case "$*" in
    --version) echo "rustc ${ZEE_TEST_YANLIS_SURUM:-$surum} (test 2026-10-12)" ;;
    -vV) echo 'host: aarch64-apple-darwin' ;;
    '--print sysroot') echo "/test/rust/$surum" ;;
    *) exit 1 ;;
esac
RUSTC
cat > "$gecici/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\t%s\t%s\n' "${RUSTUP_TOOLCHAIN:-yok}" "$*" "${RUSTFLAGS:-}" >> "$ZEE_TEST_KOMUTLAR"
case "$1" in
    build)
        mkdir -p "$CARGO_TARGET_DIR/release"
        for ad in dil dillsp; do printf 'test binary\n' > "$CARGO_TARGET_DIR/release/$ad"; done ;;
    run) echo 'test metadata' ;;
    vendor)
        for son; do :; done
        mkdir -p "$son/test-paket"
        printf 'test paket\n' > "$son/test-paket/kaynak"
        echo '# test vendor ayarı' ;;
    check) [[ "$CARGO_NET_OFFLINE" == true ]] ;;
    *) exit 1 ;;
esac
CARGO
chmod +x "$gecici/bin/rustc" "$gecici/bin/cargo"
export PATH="$gecici/bin:$PATH"
export ZEE_TEST_KOMUTLAR="$gecici/komutlar"
source "$depo_koku/scripts/rust-arac-zinciri.sh"
printf '[toolchain]\nchannel = "1.93.1"\n' > "$gecici/pin"
export RUSTUP_TOOLCHAIN=9.9.9
zee_rust_arac_zinciri "$gecici/pin"
[[ "$RUSTUP_TOOLCHAIN" == 1.93.1 ]]
for icerik in 'channel = "stable"' 'channel = "nightly"' $'channel = "1.93.1"\nchannel = "stable"' '' $'channel = "1.93.1"\nchannel = "1.97.1"'; do
    printf '%s\n' "$icerik" > "$gecici/kotu-pin"
    if (zee_rust_arac_zinciri "$gecici/kotu-pin") > /dev/null 2>&1; then
        echo 'Geçersiz pin kabul edildi' >&2; exit 1
    fi
done
if (export ZEE_TEST_YANLIS_SURUM=9.9.9; zee_rust_arac_zinciri "$gecici/pin") > /dev/null 2>&1; then
    echo 'Pin ile uyuşmayan rustc kabul edildi' >&2; exit 1
fi
cp "$depo_koku/scripts/"{rust-arac-zinciri,surum-artefakti,offline-vendor-denetle}.sh "$gecici/depo/scripts/"
cp "$gecici/pin" "$gecici/depo/compiler/rust-toolchain.toml"
printf 'test lock\n' > "$gecici/depo/compiler/Cargo.lock"
printf 'test fuzz lock\n' > "$gecici/depo/compiler/fuzz/Cargo.lock"
(
    cd "$gecici/depo"
    git init -q
    git add .
    git -c user.name=Zee -c user.email=zee@example.invalid -c commit.gpgsign=false commit -qm 'test source'
    # Kirli kabul bile commit içindeki pinle provenance üretmelidir.
    printf '[toolchain]\nchannel = "1.97.1"\n' > compiler/rust-toolchain.toml
    export RUSTUP_TOOLCHAIN=9.9.9
    bash scripts/surum-artefakti.sh --kirli-kabul --cikti "$gecici/cikti" > "$gecici/surum.log"
)
[[ "$(wc -l < "$ZEE_TEST_KOMUTLAR" | tr -d ' ')" == 6 ]]
awk -F '\t' '$1 != "1.93.1" { exit 1 }' "$ZEE_TEST_KOMUTLAR"
sabit_ara() { LC_ALL=C grep -F -- "$1" "$2" > /dev/null; }
sabit_ara '--rustc rustc 1.93.1 (test 2026-10-12)' "$ZEE_TEST_KOMUTLAR"
sabit_ara '--remap-path-prefix=/test/rust/1.93.1=/rust' "$ZEE_TEST_KOMUTLAR"
: > "$ZEE_TEST_KOMUTLAR"
(
    cd "$gecici/depo"
    export RUSTUP_TOOLCHAIN=9.9.9
    bash scripts/offline-vendor-denetle.sh > "$gecici/offline.log"
)
[[ "$(wc -l < "$ZEE_TEST_KOMUTLAR" | tr -d ' ')" == 4 ]]
awk -F '\t' '$1 != "1.97.1" { exit 1 }' "$ZEE_TEST_KOMUTLAR"
echo 'Rust araç zinciri: exact pin, negatif vakalar, HEAD provenance ve offline girişleri geçti.'
