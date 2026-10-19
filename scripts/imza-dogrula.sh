#!/usr/bin/env bash
# K-317: indirilen resmî sürümün imzasını derleme gerektirmeden doğrular (OpenSSL 3+ Ed25519).
#   bash scripts/imza-dogrula.sh <sürüm-klasörü> [--anahtar-dosyasi docs/yayinci-anahtari-v1.txt]
# Klasörde SHA256SUMS ve SHA256SUMS.zee-imza bulunur. Önce imza (sabit yayıncı anahtarı, alan
# ayrımı "zee-surum-v1\0"), sonra SHA256SUMS içindeki her dosyanın özeti denetlenir.
set -euo pipefail
klasor="${1:?Kullanım: imza-dogrula.sh <sürüm-klasörü> [--anahtar-dosyasi DOSYA]}"; shift
betik_koku="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
anahtar_dosyasi="$betik_koku/../docs/yayinci-anahtari-v1.txt"
while [[ $# -gt 0 ]]; do
  case "$1" in --anahtar-dosyasi) anahtar_dosyasi="$2"; shift 2 ;; *) echo "bilinmeyen seçenek: $1" >&2; exit 2 ;; esac
done
command -v openssl >/dev/null || { echo "openssl gerekli" >&2; exit 2; }
toplam="$klasor/SHA256SUMS"; imza="$klasor/SHA256SUMS.zee-imza"
[[ -f "$toplam" && -f "$imza" ]] || { echo "SHA256SUMS ve SHA256SUMS.zee-imza gerekli" >&2; exit 1; }
alan() { sed -n "s/^[[:space:]]*\"$1\":[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" "$imza"; }
beklenen_kimlik="$(sed -n 's/^kimlik //p' "$anahtar_dosyasi")"
beklenen_acik="$(sed -n 's/^acik //p' "$anahtar_dosyasi")"
[[ "$(alan sema)" == "zee-surum-imza-v1" ]] || { echo "RET: imza şeması zee-surum-imza-v1 değil" >&2; exit 1; }
[[ "$(alan dosya)" == "SHA256SUMS" ]] || { echo "RET: imza başka dosyaya ait" >&2; exit 1; }
[[ "$(alan anahtar_kimligi)" == "$beklenen_kimlik" && "$(alan acik_anahtar)" == "$beklenen_acik" ]] || {
  echo "RET: imza resmî yayıncı anahtarıyla atılmamış ($(alan anahtar_kimligi))" >&2; exit 1; }
gecici="$(mktemp -d)"; trap 'rm -rf "$gecici"' EXIT
# Açık anahtar → DER (RFC 8410 SubjectPublicKeyInfo önek + 32 bayt), imza hex → ikili.
{ printf '302a300506032b6570032100'; printf '%s' "$beklenen_acik"; } | xxd -r -p > "$gecici/acik.der"
printf '%s' "$(alan ed25519)" | xxd -r -p > "$gecici/imza.bin"
{ printf 'zee-surum-v1\0'; cat "$toplam"; } > "$gecici/girdi"
openssl pkeyutl -verify -pubin -inkey "$gecici/acik.der" -keyform DER -rawin \
  -in "$gecici/girdi" -sigfile "$gecici/imza.bin" >/dev/null 2>&1 || { echo "RET: Ed25519 imzası geçersiz" >&2; exit 1; }
if command -v sha256sum >/dev/null; then (cd "$klasor" && sha256sum -c --strict --quiet SHA256SUMS)
else (cd "$klasor" && shasum -a 256 -c --strict --quiet SHA256SUMS); fi || { echo "RET: dosya özeti uyuşmuyor" >&2; exit 1; }
echo "İmza ve özetler geçerli: resmî yayıncı $beklenen_kimlik"
