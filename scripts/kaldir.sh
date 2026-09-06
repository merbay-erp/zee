#!/usr/bin/env bash
# K-169/ADR-071 kaldırma: yalnız kurulum manifestinde kayıtlı ve özeti
# değişmemiş dosyaları siler; yabancı ya da değişmiş dosyaya dokunmaz.
#   scripts/kaldir.sh [--veri ~/.local/share/zee] [--zorla]
set -euo pipefail

veri="${HOME}/.local/share/zee"
zorla=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --veri) veri="$2"; shift 2 ;;
        --zorla) zorla=1; shift ;;
        *) echo "Kullanım: scripts/kaldir.sh [--veri VERİ] [--zorla]" >&2; exit 2 ;;
    esac
done
manifest="$veri/kurulum-v1.tsv"
[[ -f "$manifest" ]] || { echo "Kurulum manifesti yok: $manifest (kurulu değil)." >&2; exit 1; }
[[ "$(head -n 1 "$manifest")" == "# zee-kurulum-1" ]] || { echo "Kurulum manifesti şeması bilinmiyor." >&2; exit 1; }

ozet_hesapla() {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'; else shasum -a 256 "$1" | awk '{print $1}'; fi
}

# Önce bütün dosyalar doğrulanır; tek biri değişmişse hiçbiri silinmez.
while IFS=$'\t' read -r dosya ozet _; do
    [[ -z "$dosya" || "$dosya" == \#* || ! -f "$dosya" ]] && continue
    bulunan="$(ozet_hesapla "$dosya")"
    if [[ "$bulunan" != "$ozet" && "$zorla" -ne 1 ]]; then
        echo "$dosya kurulumdan sonra değişmiş; hiçbir dosya silinmedi (--zorla ile silinir)." >&2
        exit 1
    fi
done < "$manifest"
silinen=0
while IFS=$'\t' read -r dosya _ _; do
    [[ -z "$dosya" || "$dosya" == \#* ]] && continue
    if [[ ! -f "$dosya" ]]; then
        echo "Zaten yok: $dosya"
        continue
    fi
    rm -f "$dosya"
    silinen=$((silinen + 1))
done < "$manifest"
rm -f "$manifest"
rmdir "$veri" 2>/dev/null || true
echo "Kaldırıldı ($silinen dosya); manifest silindi."
