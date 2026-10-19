#!/usr/bin/env bash
# K-168/ADR-068 tekrar üretilebilir sürüm artefaktı.
# İki bağımsız temiz git klonunda aynı toolchain, --remap-path-prefix ve
# SOURCE_DATE_EPOCH ile `dil` + `dillsp` derlenir; SHA-256'lar eşit değilse
# durur. Çıktı klasörü ikilileri, SHA256SUMS, SPDX SBOM, SLSA provenance ve
# (anahtar verildiyse) zee-surum-imza-v1 imzasını taşır.
#   scripts/surum-artefakti.sh [--cikti KLASÖR] [--anahtar ANAHTAR] [--kirli-kabul]
set -euo pipefail

depo_koku="$(git rev-parse --show-toplevel)"
cd "$depo_koku"

cikti=""
anahtar=""
kirli_kabul=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --cikti) cikti="$2"; shift 2 ;;
        --anahtar) anahtar="$2"; shift 2 ;;
        --kirli-kabul) kirli_kabul=1; shift ;;
        *) echo "Kullanım: scripts/surum-artefakti.sh [--cikti KLASÖR] [--anahtar ANAHTAR] [--kirli-kabul]" >&2; exit 2 ;;
    esac
done

if [[ -n "$(git status --porcelain)" && "$kirli_kabul" -ne 1 ]]; then
    echo "Sürüm artefaktı temiz çalışma ağacı ister (yerel deneme için --kirli-kabul)." >&2
    exit 1
fi
sha="$(git rev-parse HEAD)"
epoch="$(git log -1 --format=%ct "$sha")"
export SOURCE_DATE_EPOCH="$epoch"
cikti="${cikti:-$depo_koku/hedef/surum/$sha}"
# Metadata araçları compiler/ içinde koşar; caller'ın göreli yolu depo
# köküne sabitlenir. Native Windows drive yolu zaten mutlaktır.
case "$cikti" in
    /*|[A-Za-z]:*) ;;
    *) cikti="$depo_koku/$cikti" ;;
esac
gecici="$(mktemp -d "${TMPDIR:-/tmp}/zee-surum.XXXXXX")"
# Cargo canonicalizes absolute path-package sources. macOS /var is a symlink;
# remapping its logical spelling would leave /private/var embedded in binaries.
gecici="$(cd "$gecici" && pwd -P)"
trap 'rm -rf "$gecici"' EXIT
# --kirli-kabul dahi HEAD'i üretir; yerel değiştirilmiş pin kullanılmaz.
git show "$sha:compiler/rust-toolchain.toml" > "$gecici/rust-toolchain.toml"
source "$depo_koku/scripts/rust-arac-zinciri.sh"
zee_rust_arac_zinciri "$gecici/rust-toolchain.toml"
platform="$(rustc -vV | awk '/^host:/ {print $2}')"
native_hedef=none
if [[ "$platform" == *-apple-darwin ]]; then
    # CC/ring SDK sürümünü minimum OS sanamaz; exact Rust varsayılanı ortak.
    deployment="$(env -u MACOSX_DEPLOYMENT_TARGET rustc --print deployment-target)"
    [[ "$deployment" =~ ^MACOSX_DEPLOYMENT_TARGET=[0-9]+\.[0-9]+(\.[0-9]+)?$ ]] || {
        echo "Rust native deployment hedefi okunamadı." >&2; exit 1
    }
    native_hedef="${deployment#*=}"
    export MACOSX_DEPLOYMENT_TARGET="$native_hedef"
fi
# Windows dağıtımı CRT'yi ikiliye bağlar. Explicit target yalnız hedef
# crate'lerine bayrak uygular; host proc-macro/build-script profili değişmez.
build_argumanlari=(--locked --release --bin dil --bin dillsp)
release_dizini=release
if [[ "$platform" == *-pc-windows-msvc ]]; then
    build_argumanlari+=(--target "$platform")
    release_dizini="$platform/release"
fi
rustc_surumu="$(rustc --version)"
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
sysroot="$(rustc --print sysroot)"
if [[ "$platform" == *-pc-windows-msvc ]]; then
    # MSYS shell yolları native rustc'nin gördüğü yollarla aynı olmalıdır.
    cikti="$(cygpath -am "$cikti")"
    cargo_home="$(cygpath -m "$cargo_home")"
    sysroot="$(cygpath -m "$sysroot")"
fi

echo "== Sürüm artefaktı: $sha ($platform, $rustc_surumu, SOURCE_DATE_EPOCH=$epoch)"
python_komutu=python
command -v python3 >/dev/null 2>&1 && python_komutu=python3

klon_temizligini_denetle() {
    local klon_durumu klon_sha
    klon_sha="$(git -C "$1" rev-parse HEAD)" || {
        echo "Sürüm klonunun kaynak kimliği okunamadı." >&2; exit 1
    }
    if [[ "$klon_sha" != "$sha" ]]; then
        echo "Sürüm klonu exact kaynak SHA'sında kalmalı." >&2; exit 1
    fi
    klon_durumu="$(git -C "$1" status --porcelain --untracked-files=all -- .)" || {
        echo "Sürüm klonunun kaynak durumu okunamadı." >&2; exit 1
    }
    if [[ -n "$klon_durumu" ]]; then
        echo "Sürüm klonu derleme boyunca temiz kalmalı." >&2; exit 1
    fi
}

ikili_kimligini_denetle() {
    "$python_komutu" -I - "$1" "$sha" <<'PY'
import json
import subprocess
import sys

try:
    result = subprocess.run([sys.argv[1], "sürüm", "--json"], capture_output=True, timeout=10)
    if result.returncode != 0 or result.stderr or len(result.stdout) > 65536:
        raise ValueError
    identity = json.loads(result.stdout)
    if (not isinstance(identity, dict) or identity.get("schema") != "zee-build-v1"
            or identity.get("source_sha") != sys.argv[2] or identity.get("dirty") is not False):
        raise ValueError
except (OSError, ValueError, RecursionError, subprocess.SubprocessError):
    print("Sürüm ikilisinin exact kaynak/temiz build kimliği doğrulanamadı.", file=sys.stderr)
    sys.exit(1)
PY
}

for klon in a b; do
    echo "-- Temiz klon $klon derleniyor"
    # K-317: Windows 260 karakter yol sınırı (kanıt arşivinde 229+ karakterlik yollar) — uzun yol açık.
    git clone -q --local --no-hardlinks -c core.longpaths=true "$depo_koku" "$gecici/$klon"
    git -C "$gecici/$klon" checkout -q "$sha"
    klon_temizligini_denetle "$gecici/$klon"
    mkdir -p "$gecici/evidence/$klon"
    klon_yolu="$gecici/$klon"
    hedef_yolu="$gecici/$klon/target"
    if [[ "$platform" == *-pc-windows-msvc ]]; then
        klon_yolu="$(cygpath -m "$klon_yolu")"
        hedef_yolu="$(cygpath -m "$hedef_yolu")"
    fi
    (
        cd "$gecici/$klon/compiler"
        rust_bayraklari=("--remap-path-prefix=$klon_yolu=/zee" "--remap-path-prefix=$cargo_home=/cargo" "--remap-path-prefix=$sysroot=/rust")
        if [[ "$platform" == *-pc-windows-msvc ]]; then
            rust_bayraklari+=(-C target-feature=+crt-static -C link-arg=/Brepro -C link-arg=/VERBOSE:LIB)
        else
            rust_bayraklari+=(-C link-arg=-v)
        fi
        printf -v kodlanmis_bayraklar '%s\x1f' "${rust_bayraklari[@]}"
        kodlanmis_bayraklar="${kodlanmis_bayraklar%$'\x1f'}"
        # Boşluklu yollar tek argümandır; caller encoded flags tarifi ezemez.
        if ! RUSTFLAGS="" CARGO_ENCODED_RUSTFLAGS="$kodlanmis_bayraklar" \
            CC_ENABLE_DEBUG_OUTPUT=1 \
            RUSTC_LOG=rustc_codegen_ssa::back::link=info \
            CARGO_TARGET_DIR="$hedef_yolu" \
            cargo build -vv "${build_argumanlari[@]}" > "$gecici/evidence/$klon/build.log" 2>&1; then
            cat "$gecici/evidence/$klon/build.log" >&2
            exit 1
        fi
    )
    klon_temizligini_denetle "$gecici/$klon"
    ikili_uzantisi=""
    [[ "$platform" == *windows* ]] && ikili_uzantisi=.exe
    ikili_kimligini_denetle "$gecici/$klon/target/$release_dizini/dil$ikili_uzantisi"
    klon_temizligini_denetle "$gecici/$klon"
    if [[ "$native_hedef" != none ]]; then
        "$python_komutu" "$depo_koku/scripts/native-deployment-denetle.py" --root "$gecici/$klon/target" --target "$native_hedef" --out "$gecici/evidence/$klon/native-deployment.json"
    fi
done

ozet_hesapla() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

uzanti=""
[[ "$platform" == *windows* ]] && uzanti=".exe"
for ikili in dil dillsp; do
    a="$(ozet_hesapla "$gecici/a/target/$release_dizini/$ikili$uzanti")"
    b="$(ozet_hesapla "$gecici/b/target/$release_dizini/$ikili$uzanti")"
    if [[ "$a" != "$b" ]]; then
        echo "TEKRAR ÜRETİLEMEDİ: $ikili iki temiz klonda farklı ($a / $b)" >&2
        # Yalnız CI tanısı: eş olmayan byte'lar sürüm kümesine girmez.
        if [[ -n "${ZEE_SURUM_TANI_DIZINI:-}" ]]; then
            mkdir -p "$ZEE_SURUM_TANI_DIZINI/a" "$ZEE_SURUM_TANI_DIZINI/b"
            for tani_klonu in a b; do
                for tani_ikilisi in dil dillsp; do
                    cp "$gecici/$tani_klonu/target/$release_dizini/$tani_ikilisi$uzanti" "$ZEE_SURUM_TANI_DIZINI/$tani_klonu/"
                done
            done
            printf '%s\n' "$sha" > "$ZEE_SURUM_TANI_DIZINI/GIT_SHA"
            printf 'YAYINLANAMAZ: iki temiz klon hashleri eş değil.\n' > "$ZEE_SURUM_TANI_DIZINI/DURUM"
        fi
        exit 1
    fi
    echo "-- $ikili iki klonda eş: $a"
done

if ! "$python_komutu" "$depo_koku/scripts/derleme-ortami-kaydet.py" --a "$gecici/evidence/a/build.log" --b "$gecici/evidence/b/build.log" --sha "$sha" --out "$gecici/build-environment.json" --native-target "$native_hedef"; then
    if [[ -n "${ZEE_SURUM_TANI_DIZINI:-}" ]]; then
        mkdir -p "$ZEE_SURUM_TANI_DIZINI"
        cp "$gecici/evidence/a/build.log" "$ZEE_SURUM_TANI_DIZINI/a-build.log"
        cp "$gecici/evidence/b/build.log" "$ZEE_SURUM_TANI_DIZINI/b-build.log"
        printf '%s\n' "$sha" > "$ZEE_SURUM_TANI_DIZINI/GIT_SHA"
        printf 'YAYINLANAMAZ: linker ortam kanıtı eksik.\n' > "$ZEE_SURUM_TANI_DIZINI/DURUM"
    fi
    exit 1
fi
mkdir -p "$cikti"
cp "$gecici/a/target/$release_dizini/dil$uzanti" "$gecici/a/target/$release_dizini/dillsp$uzanti" "$cikti/"
arac=(cargo run -q --locked --bin surum_artefakti --)
(cd compiler && "${arac[@]}" ozet "$cikti" "dil$uzanti" "dillsp$uzanti") > "$cikti/SHA256SUMS"
cp "$cikti/SHA256SUMS" "$gecici/SHA256SUMS.b"
# İkinci klonun özeti bağımsız hesaplanır; provenance eşitliği kayda geçirir.
(cd compiler && "${arac[@]}" ozet "$gecici/b/target/$release_dizini" "dil$uzanti" "dillsp$uzanti") > "$gecici/SHA256SUMS.b"
(cd compiler && "${arac[@]}" sbom --sha "$sha" --kilit Cargo.lock --ozet "$cikti/SHA256SUMS" --epoch "$epoch") > "$cikti/sbom.spdx.json"
(cd compiler && "${arac[@]}" provenance --sha "$sha" --ozet "$cikti/SHA256SUMS" --ikinci-ozet "$gecici/SHA256SUMS.b" \
    --epoch "$epoch" --rustc "$rustc_surumu" --platform "$platform" --build-environment "$gecici/build-environment.json") > "$cikti/provenance.intoto.json"
if [[ -n "$anahtar" ]]; then
    (cd compiler && "${arac[@]}" imzala --anahtar "$anahtar" "$cikti/SHA256SUMS") > "$cikti/SHA256SUMS.zee-imza"
    (cd compiler && "${arac[@]}" dogrula --dosya "$cikti/SHA256SUMS" --imza "$cikti/SHA256SUMS.zee-imza")
fi
printf '%s\n' "$sha" > "$cikti/GIT_SHA"
echo "Sürüm artefaktı hazır: $cikti"
cat "$cikti/SHA256SUMS"
