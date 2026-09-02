#!/usr/bin/env bash
set -euo pipefail

taban="${1:-}"
if [[ -z "$taban" ]]; then
    echo "Kullanım: scripts/semantic-regresyon-korugu.sh <taban-git-revizyonu>" >&2
    exit 2
fi
if [[ "$taban" =~ ^0+$ ]]; then
    echo "İlk depo gönderimi: korunacak önceki semantic regresyon vakası yok."
    exit 0
fi
if ! git cat-file -e "${taban}^{commit}" 2>/dev/null; then
    echo "Semantic regresyon koruğu taban revizyonunu bulamadı: $taban" >&2
    exit 2
fi
if ! git cat-file -e "${taban}:regression/v1.tsv" 2>/dev/null; then
    echo "Taban revizyonda semantic regresyon manifesti yok; ilk korpus ekleniyor."
    exit 0
fi
if [[ ! -f regression/v1.tsv ]]; then
    echo "SEMANTIC REGRESYON MANİFESTİ SİLİNDİ: regression/v1.tsv" >&2
    exit 1
fi

while IFS=$'\t' read -r vaka bug _ _ _ _ _ _ dosya; do
    [[ -z "$vaka" || "$vaka" == \#* ]] && continue
    eslesmeler="$(awk -F '\t' -v vaka="$vaka" '$1 == vaka { print }' regression/v1.tsv)"
    if [[ -z "$eslesmeler" ]]; then
        echo "KALICI SEMANTIC REGRESYON VAKASI SİLİNDİ: $vaka ($bug)" >&2
        exit 1
    fi
    if [[ "$(printf '%s\n' "$eslesmeler" | wc -l | tr -d ' ')" != "1" ]]; then
        echo "SEMANTIC REGRESYON VAKASI YİNELENDİ: $vaka" >&2
        exit 1
    fi
    IFS=$'\t' read -r _ guncel_bug _ _ _ _ _ _ guncel_dosya <<< "$eslesmeler"
    if [[ "$guncel_bug" != "$bug" || "$guncel_dosya" != "$dosya" ]]; then
        echo "SEMANTIC REGRESYON KİMLİĞİ YENİDEN KULLANILDI: $vaka" >&2
        echo "Eski: $bug -> $dosya" >&2
        echo "Yeni: $guncel_bug -> $guncel_dosya" >&2
        exit 1
    fi
done < <(git show "${taban}:regression/v1.tsv")

echo "Yayımlanmış semantic regresyon vaka kimlikleri ve yolları korunuyor."
