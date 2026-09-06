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
platform="$(rustc -vV | awk '/^host:/ {print $2}')"
rustc_surumu="$(rustc --version)"
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
sysroot="$(rustc --print sysroot)"
gecici="$(mktemp -d "${TMPDIR:-/tmp}/zee-surum.XXXXXX")"
trap 'rm -rf "$gecici"' EXIT

echo "== Sürüm artefaktı: $sha ($platform, $rustc_surumu, SOURCE_DATE_EPOCH=$epoch)"
for klon in a b; do
    echo "-- Temiz klon $klon derleniyor"
    git clone -q --local "$depo_koku" "$gecici/$klon"
    git -C "$gecici/$klon" checkout -q "$sha"
    (
        cd "$gecici/$klon/compiler"
        RUSTFLAGS="--remap-path-prefix=$gecici/$klon=/zee --remap-path-prefix=$cargo_home=/cargo --remap-path-prefix=$sysroot=/rust" \
        CARGO_TARGET_DIR="$gecici/$klon/target" \
        cargo build --locked --release --bin dil --bin dillsp
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
    a="$(ozet_hesapla "$gecici/a/target/release/$ikili$uzanti")"
    b="$(ozet_hesapla "$gecici/b/target/release/$ikili$uzanti")"
    if [[ "$a" != "$b" ]]; then
        echo "TEKRAR ÜRETİLEMEDİ: $ikili iki temiz klonda farklı ($a / $b)" >&2
        exit 1
    fi
    echo "-- $ikili iki klonda eş: $a"
done

mkdir -p "$cikti"
cp "$gecici/a/target/release/dil$uzanti" "$gecici/a/target/release/dillsp$uzanti" "$cikti/"
arac=(cargo run -q --locked --bin surum_artefakti --)
(cd compiler && "${arac[@]}" ozet "$cikti" "dil$uzanti" "dillsp$uzanti") > "$cikti/SHA256SUMS"
cp "$cikti/SHA256SUMS" "$gecici/SHA256SUMS.b"
# İkinci klonun özeti bağımsız hesaplanır; provenance eşitliği kayda geçirir.
(cd compiler && "${arac[@]}" ozet "$gecici/b/target/release" "dil$uzanti" "dillsp$uzanti") > "$gecici/SHA256SUMS.b"
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
