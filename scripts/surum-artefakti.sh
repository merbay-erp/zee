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
trap 'rm -rf "$gecici"' EXIT
# --kirli-kabul dahi HEAD'i üretir; yerel değiştirilmiş pin kullanılmaz.
git show "$sha:compiler/rust-toolchain.toml" > "$gecici/rust-toolchain.toml"
source "$depo_koku/scripts/rust-arac-zinciri.sh"
zee_rust_arac_zinciri "$gecici/rust-toolchain.toml"
platform="$(rustc -vV | awk '/^host:/ {print $2}')"
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
for klon in a b; do
    echo "-- Temiz klon $klon derleniyor"
    git clone -q --local --no-hardlinks "$depo_koku" "$gecici/$klon"
    git -C "$gecici/$klon" checkout -q "$sha"
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
            rust_bayraklari+=(-C target-feature=+crt-static -C link-arg=/Brepro)
        fi
        printf -v kodlanmis_bayraklar '%s\x1f' "${rust_bayraklari[@]}"
        kodlanmis_bayraklar="${kodlanmis_bayraklar%$'\x1f'}"
        # Boşluklu yollar tek argümandır; caller encoded flags tarifi ezemez.
        RUSTFLAGS="" CARGO_ENCODED_RUSTFLAGS="$kodlanmis_bayraklar" \
        CARGO_TARGET_DIR="$hedef_yolu" \
        cargo build "${build_argumanlari[@]}"
    )
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

mkdir -p "$cikti"
cp "$gecici/a/target/$release_dizini/dil$uzanti" "$gecici/a/target/$release_dizini/dillsp$uzanti" "$cikti/"
arac=(cargo run -q --locked --bin surum_artefakti --)
(cd compiler && "${arac[@]}" ozet "$cikti" "dil$uzanti" "dillsp$uzanti") > "$cikti/SHA256SUMS"
cp "$cikti/SHA256SUMS" "$gecici/SHA256SUMS.b"
# İkinci klonun özeti bağımsız hesaplanır; provenance eşitliği kayda geçirir.
(cd compiler && "${arac[@]}" ozet "$gecici/b/target/$release_dizini" "dil$uzanti" "dillsp$uzanti") > "$gecici/SHA256SUMS.b"
(cd compiler && "${arac[@]}" sbom --sha "$sha" --kilit Cargo.lock --ozet "$cikti/SHA256SUMS" --epoch "$epoch") > "$cikti/sbom.spdx.json"
(cd compiler && "${arac[@]}" provenance --sha "$sha" --ozet "$cikti/SHA256SUMS" --ikinci-ozet "$gecici/SHA256SUMS.b" \
    --epoch "$epoch" --rustc "$rustc_surumu" --platform "$platform") > "$cikti/provenance.intoto.json"
if [[ -n "$anahtar" ]]; then
    (cd compiler && "${arac[@]}" imzala --anahtar "$anahtar" "$cikti/SHA256SUMS") > "$cikti/SHA256SUMS.zee-imza"
    (cd compiler && "${arac[@]}" dogrula --dosya "$cikti/SHA256SUMS" --imza "$cikti/SHA256SUMS.zee-imza")
fi
printf '%s\n' "$sha" > "$cikti/GIT_SHA"
echo "Sürüm artefaktı hazır: $cikti"
cat "$cikti/SHA256SUMS"
