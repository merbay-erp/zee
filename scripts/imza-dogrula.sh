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
# Ed25519 için OpenSSL 3+ gerekir; macOS'un /usr/bin/openssl'i LibreSSL'dir ve desteklemez.
# Sıra: ZEE_OPENSSL, PATH'teki openssl, Homebrew openssl@3. Araç eksikliği imza reddi sayılmaz.
openssl_uygun() { [[ -x "$1" ]] && "$1" version 2>/dev/null | grep -Eq '^OpenSSL ([3-9]|[1-9][0-9])\.'; }
openssl_bin=""
for aday in "${ZEE_OPENSSL:-}" "$(command -v openssl || true)" \
  /opt/homebrew/opt/openssl@3/bin/openssl /usr/local/opt/openssl@3/bin/openssl; do
  [[ -n "$aday" ]] && openssl_uygun "$aday" && { openssl_bin="$aday"; break; }
done
[[ -n "$openssl_bin" ]] || {
  echo "OpenSSL 3+ gerekli (LibreSSL Ed25519 doğrulamaz). macOS: brew install openssl@3; ya da ZEE_OPENSSL=/yol/openssl" >&2
  exit 2; }
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
"$openssl_bin" pkeyutl -verify -pubin -inkey "$gecici/acik.der" -keyform DER -rawin \
  -in "$gecici/girdi" -sigfile "$gecici/imza.bin" >/dev/null 2>&1 || { echo "RET: Ed25519 imzası geçersiz" >&2; exit 1; }
if command -v sha256sum >/dev/null; then (cd "$klasor" && sha256sum -c --strict --quiet SHA256SUMS)
else (cd "$klasor" && shasum -a 256 -c --strict --quiet SHA256SUMS); fi || { echo "RET: dosya özeti uyuşmuyor" >&2; exit 1; }
echo "İmza ve özetler geçerli: resmî yayıncı $beklenen_kimlik"
