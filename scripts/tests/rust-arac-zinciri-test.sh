#!/usr/bin/env bash
# K-191: gerçek giriş betikleri, farklı varsayılan ve negatif pin vakaları.
set -euo pipefail
depo_koku="$(cd "$(dirname "$0")/../.." && pwd)"
gecici="$(mktemp -d "${TMPDIR:-/tmp}/zee toolchain-test.XXXXXX")"
trap 'rm -rf "$gecici"' EXIT
mkdir -p "$gecici/bin" "$gecici/depo/scripts" "$gecici/depo/compiler/fuzz"
cat > "$gecici/bin/rustc" <<'RUSTC'
#!/usr/bin/env bash
set -euo pipefail
surum="${RUSTUP_TOOLCHAIN:-9.9.9}"
case "$*" in
    --version) echo "rustc ${ZEE_TEST_YANLIS_SURUM:-$surum} (test 2026-10-12)" ;;
    -vV) echo "host: ${ZEE_TEST_PLATFORM:-aarch64-apple-darwin}" ;;
    '--print sysroot') echo "/test/rust/$surum" ;;
    *) exit 1 ;;
esac
RUSTC
cat > "$gecici/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
bayraklar="${CARGO_ENCODED_RUSTFLAGS:-${RUSTFLAGS:-}}"
bayraklar="${bayraklar//$'\x1f'/ }"
printf '%s\t%s\t%s\n' "${RUSTUP_TOOLCHAIN:-yok}" "$*" "$bayraklar" >> "$ZEE_TEST_KOMUTLAR"
case "$1" in
    build)
        IFS=$'\x1f' read -r -a ayrik <<< "$CARGO_ENCODED_RUSTFLAGS"
        [[ "${ayrik[0]}" == --remap-path-prefix=*"zee toolchain-test."*=/zee ]] || exit 1
        if [[ "$*" == *"--target x86_64-pc-windows-msvc"* ]]; then
            [[ "${#ayrik[@]}" == 7 ]] || exit 1
        else
            [[ "${#ayrik[@]}" == 3 ]] || exit 1
        fi
        dizin="$CARGO_TARGET_DIR/release"
        uzanti=""
        if [[ "$*" == *"--target x86_64-pc-windows-msvc"* ]]; then
            dizin="$CARGO_TARGET_DIR/x86_64-pc-windows-msvc/release"
            uzanti=.exe
            [[ "$bayraklar" == *"-C target-feature=+crt-static"* ]] || exit 1
            [[ "$bayraklar" == *"-C link-arg=/Brepro"* ]] || exit 1
        else
            [[ "$bayraklar" != *"crt-static"* ]] || exit 1
        fi
        mkdir -p "$dizin"
        for ad in dil dillsp; do
            printf 'test binary\n' > "$dizin/$ad$uzanti"
            if [[ "${ZEE_TEST_OZET_FARKI:-0}" == 1 ]]; then
                printf '%s\n' "$dizin" >> "$dizin/$ad$uzanti"
            fi
        done ;;
    run)
        if [[ "$*" == *"-- ozet "* ]]; then
            # Üreticinin ozet argümanı cwd=compiler altında da bulunmalı.
            while [[ "$1" != ozet ]]; do shift; done
            shift
            [[ -f "$1/$2" && -f "$1/$3" ]] || exit 1
        fi
        echo 'test metadata' ;;
    vendor)
        for son; do :; done
        mkdir -p "$son/test-paket"
        printf 'test paket\n' > "$son/test-paket/kaynak"
        echo '# test vendor ayarı' ;;
    check) [[ "$CARGO_NET_OFFLINE" == true ]] ;;
    *) exit 1 ;;
esac
CARGO
cat > "$gecici/bin/cygpath" <<'CYGPATH'
#!/usr/bin/env bash
[[ "$1" == -m || "$1" == -am ]] || exit 1
printf '%s\n' "$2"
CYGPATH
chmod +x "$gecici/bin/rustc" "$gecici/bin/cargo" "$gecici/bin/cygpath"
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
    TMPDIR="$gecici" bash scripts/surum-artefakti.sh --kirli-kabul --cikti "$gecici/cikti" > "$gecici/surum.log"
)
[[ "$(wc -l < "$ZEE_TEST_KOMUTLAR" | tr -d ' ')" == 6 ]]
awk -F '\t' '$1 != "1.93.1" { exit 1 }' "$ZEE_TEST_KOMUTLAR"
sabit_ara() { LC_ALL=C grep -F -- "$1" "$2" > /dev/null; }
sabit_ara '--rustc rustc 1.93.1 (test 2026-10-12)' "$ZEE_TEST_KOMUTLAR"
sabit_ara '--remap-path-prefix=/test/rust/1.93.1=/rust' "$ZEE_TEST_KOMUTLAR"
: > "$ZEE_TEST_KOMUTLAR"
(
    cd "$gecici/depo"
    export ZEE_TEST_PLATFORM=x86_64-pc-windows-msvc
    export CARGO_ENCODED_RUSTFLAGS=caller-bayragi
    TMPDIR="$gecici" bash scripts/surum-artefakti.sh --kirli-kabul --cikti goreli-windows > "$gecici/windows.log"
)
[[ -f "$gecici/depo/goreli-windows/dil.exe" && -f "$gecici/depo/goreli-windows/dillsp.exe" ]]
[[ "$(wc -l < "$ZEE_TEST_KOMUTLAR" | tr -d ' ')" == 6 ]]
sabit_ara 'build --locked --release --bin dil --bin dillsp --target x86_64-pc-windows-msvc' "$ZEE_TEST_KOMUTLAR"
sabit_ara '-C target-feature=+crt-static' "$ZEE_TEST_KOMUTLAR"
if awk -F '\t' '$2 ~ /^build / { print $3 }' "$ZEE_TEST_KOMUTLAR" | grep -F caller-bayragi; then
    echo 'Caller encoded flags release tarifini ezdi' >&2; exit 1
fi
: > "$ZEE_TEST_KOMUTLAR"
if (
    cd "$gecici/depo"
    export ZEE_TEST_OZET_FARKI=1
    export ZEE_SURUM_TANI_DIZINI="$gecici/tani"
    TMPDIR="$gecici" bash scripts/surum-artefakti.sh --kirli-kabul --cikti "$gecici/ret-surum"
) > "$gecici/ret.log" 2>&1; then
    echo 'Eş olmayan iki klon kabul edildi' >&2; exit 1
fi
sabit_ara 'TEKRAR ÜRETİLEMEDİ' "$gecici/ret.log"
[[ ! -e "$gecici/ret-surum" ]]
[[ -f "$gecici/tani/a/dil" && -f "$gecici/tani/b/dil" && -f "$gecici/tani/DURUM" ]]
: > "$ZEE_TEST_KOMUTLAR"
(
    cd "$gecici/depo"
    export RUSTUP_TOOLCHAIN=9.9.9
    bash scripts/offline-vendor-denetle.sh > "$gecici/offline.log"
)
[[ "$(wc -l < "$ZEE_TEST_KOMUTLAR" | tr -d ' ')" == 4 ]]
awk -F '\t' '$1 != "1.97.1" { exit 1 }' "$ZEE_TEST_KOMUTLAR"
echo 'Rust araç zinciri: exact pin, negatif vakalar, HEAD provenance ve offline girişleri geçti.'
