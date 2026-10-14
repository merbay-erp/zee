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
    '--print deployment-target') echo "MACOSX_DEPLOYMENT_TARGET=${MACOSX_DEPLOYMENT_TARGET:-11.0}" ;;
    *) exit 1 ;;
esac
RUSTC
cat > "$gecici/bin/rustup" <<'RUSTUP'
#!/usr/bin/env bash
set -euo pipefail
[[ "$#" == 4 && "$1" == which && "$2" == --toolchain && "$4" == rustc ]] || exit 1
[[ "$3" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || exit 1
printf '%s\n' "$ZEE_TEST_RUSTC_PATH"
RUSTUP
cat > "$gecici/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
bayraklar="${CARGO_ENCODED_RUSTFLAGS:-${RUSTFLAGS:-}}"
bayraklar="${bayraklar//$'\x1f'/ }"
printf '%s\t%s\t%s\n' "${RUSTUP_TOOLCHAIN:-yok}" "$*" "$bayraklar" >> "$ZEE_TEST_KOMUTLAR"
[[ "$RUSTC" == "$ZEE_TEST_RUSTC_PATH" && -z "$RUSTC_WRAPPER" && -z "$RUSTC_WORKSPACE_WRAPPER" ]] || exit 1
case "$1" in
    build)
        IFS=$'\x1f' read -r -a ayrik <<< "$CARGO_ENCODED_RUSTFLAGS"
        [[ "${ayrik[0]}" == --remap-path-prefix=*"zee toolchain-test."*=/zee ]] || exit 1
        if [[ "$*" == *"--target x86_64-pc-windows-msvc"* ]]; then
            [[ "${#ayrik[@]}" == 9 ]] || exit 1
        else
            [[ "${#ayrik[@]}" == 5 ]] || exit 1
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
        if [[ "${ZEE_TEST_PLATFORM:-aarch64-apple-darwin}" == *-apple-darwin ]]; then
            [[ "$MACOSX_DEPLOYMENT_TARGET" == 11.0 ]] || exit 1
            mkdir -p "$dizin/build/ring-test/out"
            printf 'test native object\n' > "$dizin/build/ring-test/out/native.o"
        fi
        for ad in dil dillsp; do
            ikili_sha="$(git rev-parse HEAD)"
            kirli=false
            [[ "${ZEE_TEST_IKILI_KIMLIGI:-temiz}" == kirli ]] && kirli=true
            if [[ "${ZEE_TEST_IKILI_KIMLIGI:-temiz}" == yanlis-sha ]]; then
                ikili_sha=0000000000000000000000000000000000000000
            fi
            cat > "$dizin/$ad$uzanti" <<IKILI
#!/usr/bin/env bash
set -euo pipefail
[[ "\$*" == 'sürüm --json' ]] || exit 2
printf '%s\n' '{"schema":"zee-build-v1","version":"0.8.0-dev","source_sha":"$ikili_sha","source_date":"2026-10-14","channel":"development","dirty":$kirli}'
IKILI
            if [[ "${ZEE_TEST_IKILI_KIMLIGI:-temiz}" == bozuk-json ]]; then
                printf "printf '%%s\\n' '{'\\n" >> "$dizin/$ad$uzanti"
            fi
            chmod +x "$dizin/$ad$uzanti"
            if [[ "${ZEE_TEST_OZET_FARKI:-0}" == 1 ]]; then
                printf '# %s\n' "$dizin" >> "$dizin/$ad$uzanti"
            fi
        done
        if [[ "${ZEE_TEST_KAYNAK_MUTASYONU:-0}" == 1 ]]; then
            printf 'synthetic mutation\n' > ../generated-untracked.txt
        fi
        if [[ "${ZEE_TEST_HEAD_MUTASYONU:-0}" == 1 ]]; then
            git -c user.name=Zee -c user.email=zee@example.invalid -c commit.gpgsign=false commit -q --allow-empty -m 'synthetic clean HEAD mutation'
        fi
        [[ "$CC_ENABLE_DEBUG_OUTPUT" == 1 && "$*" == *"-vv"* ]] || exit 1
        printf '[ring 0.17.14] running: LC_ALL="C" "cc" "-c" "test.c"\n' >&2
        printf '[ring 0.17.14] running: ZERO_AR_DATE="1" "ar" "cq" "test.a"\n' >&2
        printf ' INFO rustc_codegen_ssa::back::link "cc" "-o" "%s/dil-abc123%s"\n' "$dizin" "$uzanti" >&2
        printf ' INFO rustc_codegen_ssa::back::link "cc" "-o" "%s/dillsp-abc123%s"\n' "$dizin" "$uzanti" >&2
        ;;
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
cat > "$gecici/bin/cc" <<'CC'
#!/usr/bin/env bash
echo 'test linker 1.0'
CC
cat > "$gecici/bin/otool" <<'OTOOL'
#!/usr/bin/env bash
printf 'Load command 0\n cmd LC_BUILD_VERSION\n platform 1\n minos 11.0\n sdk 27.0\n'
OTOOL
cat > "$gecici/bin/cygpath" <<'CYGPATH'
#!/usr/bin/env bash
[[ "$1" == -m || "$1" == -am ]] || exit 1
printf '%s\n' "$2"
CYGPATH
chmod +x "$gecici/bin/rustc" "$gecici/bin/rustup" "$gecici/bin/cargo" "$gecici/bin/cygpath" "$gecici/bin/cc" "$gecici/bin/otool"
export PATH="$gecici/bin:$PATH"
export ZEE_TEST_KOMUTLAR="$gecici/komutlar"
export ZEE_TEST_RUSTC_PATH="$gecici/bin/rustc"
export RUSTC=/caller/other/rustc RUSTC_WRAPPER=/caller/wrapper RUSTC_WORKSPACE_WRAPPER=/caller/workspace-wrapper
source "$depo_koku/scripts/rust-arac-zinciri.sh"
printf '[toolchain]\nchannel = "1.93.1"\n' > "$gecici/pin"
export RUSTUP_TOOLCHAIN=9.9.9
export MACOSX_DEPLOYMENT_TARGET=27.0
zee_rust_arac_zinciri "$gecici/pin"
[[ "$RUSTUP_TOOLCHAIN" == 1.93.1 ]]
[[ "$RUSTC" == "$ZEE_TEST_RUSTC_PATH" && -z "$RUSTC_WRAPPER" && -z "$RUSTC_WORKSPACE_WRAPPER" ]]
if (ZEE_TEST_RUSTC_PATH="$gecici/yok-rustc" zee_rust_arac_zinciri "$gecici/pin") >/dev/null 2>&1; then
    echo "Rustup eksik executable yolu kabul edildi" >&2; exit 1
fi
printf '#!/usr/bin/env bash\necho "rustc 1.97.1 (yanlis arac)"\n' > "$gecici/yanlis-rustc"
chmod +x "$gecici/yanlis-rustc"
if (ZEE_TEST_RUSTC_PATH="$gecici/yanlis-rustc" zee_rust_arac_zinciri "$gecici/pin") >/dev/null 2>&1; then
    echo "Rustup farkli compiler surumu kabul edildi" >&2; exit 1
fi
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
cp "$depo_koku/scripts/"{derleme-ortami-kaydet,native-deployment-denetle}.py "$gecici/depo/scripts/"
cp "$gecici/pin" "$gecici/depo/compiler/rust-toolchain.toml"
printf 'test lock\n' > "$gecici/depo/compiler/Cargo.lock"
printf 'test fuzz lock\n' > "$gecici/depo/compiler/fuzz/Cargo.lock"
printf 'target/\n' > "$gecici/depo/.gitignore"
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
sabit_ara 'build -vv --locked --release --bin dil --bin dillsp --target x86_64-pc-windows-msvc' "$ZEE_TEST_KOMUTLAR"
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
# Equal hashes do not prove a clean/exact identity. No rejected run publishes.
for kip in kirli yanlis-sha bozuk-json; do
    if (
        cd "$gecici/depo"
        export ZEE_TEST_IKILI_KIMLIGI="$kip"
        TMPDIR="$gecici" bash scripts/surum-artefakti.sh --kirli-kabul --cikti "$gecici/ret-$kip"
    ) > "$gecici/ret-$kip.log" 2>&1; then
        echo 'Bozuk built kimliği kabul edildi' >&2; exit 1
    fi
    sabit_ara 'exact kaynak/temiz build kimliği doğrulanamadı' "$gecici/ret-$kip.log"
    [[ ! -e "$gecici/ret-$kip" ]]
done
if (
    cd "$gecici/depo"
    export ZEE_TEST_KAYNAK_MUTASYONU=1
    TMPDIR="$gecici" bash scripts/surum-artefakti.sh --kirli-kabul --cikti "$gecici/ret-mutasyon"
) > "$gecici/ret-mutasyon.log" 2>&1; then
    echo 'Cargo kaynak mutasyonu kabul edildi' >&2; exit 1
fi
sabit_ara 'Sürüm klonu derleme boyunca temiz kalmalı' "$gecici/ret-mutasyon.log"
[[ ! -e "$gecici/ret-mutasyon" ]]
if (
    cd "$gecici/depo"
    export ZEE_TEST_HEAD_MUTASYONU=1
    TMPDIR="$gecici" bash scripts/surum-artefakti.sh --kirli-kabul --cikti "$gecici/ret-head"
) > "$gecici/ret-head.log" 2>&1; then
    echo 'Cargo temiz HEAD değişimi kabul edildi' >&2; exit 1
fi
sabit_ara "Sürüm klonu exact kaynak SHA'sında kalmalı" "$gecici/ret-head.log"
[[ ! -e "$gecici/ret-head" ]]
: > "$ZEE_TEST_KOMUTLAR"
(
    cd "$gecici/depo"
    export RUSTUP_TOOLCHAIN=9.9.9
    bash scripts/offline-vendor-denetle.sh > "$gecici/offline.log"
)
[[ "$(wc -l < "$ZEE_TEST_KOMUTLAR" | tr -d ' ')" == 4 ]]
awk -F '\t' '$1 != "1.97.1" { exit 1 }' "$ZEE_TEST_KOMUTLAR"
echo 'Rust araç zinciri: exact pin, negatif vakalar, HEAD provenance ve offline girişleri geçti.'
