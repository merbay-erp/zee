#!/usr/bin/env bash
# K-169/ADR-071 kurulum: sürüm artefaktındaki dil/dillsp ikililerini SHA256SUMS
# doğrulayarak hedef klasöre kopyalar ve kurulum manifestini yazar.
#   scripts/kur.sh --artefakt KLASÖR [--hedef ~/.local/bin] [--veri ~/.local/share/zee]
set -euo pipefail

artefakt=""
hedef="${HOME}/.local/bin"
veri="${HOME}/.local/share/zee"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --artefakt) artefakt="$2"; shift 2 ;;
        --hedef) hedef="$2"; shift 2 ;;
        --veri) veri="$2"; shift 2 ;;
        *) echo "Kullanım: scripts/kur.sh --artefakt KLASÖR [--hedef BIN] [--veri VERİ]" >&2; exit 2 ;;
    esac
done
[[ -n "$artefakt" && -d "$artefakt" ]] || { echo "Sürüm artefakt klasörü gerekli (--artefakt)." >&2; exit 2; }
[[ -f "$artefakt/SHA256SUMS" ]] || { echo "Artefaktta SHA256SUMS yok; doğrulanamayan ikili kurulmaz." >&2; exit 1; }

ozet_hesapla() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'; else shasum -a 256 "$1" | awk '{print $1}'; fi
}

manifest="$veri/kurulum-v1.tsv"
if [[ -f "$manifest" ]]; then
    echo "Önce var olan kurulumu kaldır: scripts/kaldir.sh --veri $veri" >&2
    exit 1
fi
mkdir -p "$hedef" "$veri"
gecici_manifest="$(mktemp "${TMPDIR:-/tmp}/zee-kurulum.XXXXXX")"
printf '# zee-kurulum-1\n# dosya\tsha256\tkaynak_git_sha\n' > "$gecici_manifest"
git_sha="$( [[ -f "$artefakt/GIT_SHA" ]] && cat "$artefakt/GIT_SHA" || echo '-' )"
kurulan=0
while IFS= read -r satir; do
    [[ -z "$satir" ]] && continue
    beklenen="${satir%%  *}"
    ad="${satir##*  }"
    case "$ad" in dil | dil.exe | dillsp | dillsp.exe) ;; *) continue ;; esac
    kaynak="$artefakt/$ad"
    [[ -f "$kaynak" ]] || { echo "Artefaktta $ad yok." >&2; rm -f "$gecici_manifest"; exit 1; }
    bulunan="$(ozet_hesapla "$kaynak")"
    if [[ "$bulunan" != "$beklenen" ]]; then
        echo "SHA-256 uyuşmuyor: $ad (beklenen $beklenen, bulunan $bulunan); kurulum iptal." >&2
        rm -f "$gecici_manifest"; exit 1
    fi
    if [[ -e "$hedef/$ad" ]]; then
        echo "$hedef/$ad zaten var; kaldırmadan üzerine yazılmaz." >&2
        rm -f "$gecici_manifest"; exit 1
    fi
    install -m 0755 "$kaynak" "$hedef/$ad"
    printf '%s\t%s\t%s\n' "$hedef/$ad" "$bulunan" "$git_sha" >> "$gecici_manifest"
    kurulan=$((kurulan + 1))
done < "$artefakt/SHA256SUMS"
if [[ "$kurulan" -eq 0 ]]; then
    echo "SHA256SUMS içinde dil/dillsp girdisi yok." >&2
    rm -f "$gecici_manifest"; exit 1
fi
mv "$gecici_manifest" "$manifest"
echo "Kuruldu ($kurulan ikili) → $hedef; manifest: $manifest"
case ":$PATH:" in *":$hedef:"*) ;; *) echo "Not: $hedef PATH içinde değil; kabuğuna ekle." ;; esac
