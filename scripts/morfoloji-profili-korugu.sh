#!/usr/bin/env bash
set -euo pipefail

taban="${1:-}"
if [[ -z "$taban" ]]; then
    echo "Kullanım: scripts/morfoloji-profili-korugu.sh <taban-git-revizyonu>" >&2
    exit 2
fi
if [[ "$taban" =~ ^0+$ ]]; then
    echo "İlk depo gönderimi: korunacak önceki morfoloji fixture'ı yok."
    exit 0
fi
if ! git cat-file -e "${taban}^{commit}" 2>/dev/null; then
    echo "Morfoloji koruğu taban revizyonunu bulamadı: $taban" >&2
    exit 2
fi

korunanlar="$(
    git ls-tree -r --name-only "$taban" -- compiler/tests/fixtures/ |
        grep -E '^compiler/tests/fixtures/morfoloji-zee-tr-[0-9]+\.sha256$' || true
)"
while IFS= read -r dosya; do
    [[ -z "$dosya" ]] && continue
    if ! git diff --quiet "$taban" -- "$dosya"; then
        echo "IMMUTABLE MORFOLOJİ PROFİLİ DEĞİŞTİ: $dosya" >&2
        echo "Eski fixture'ı güncelleme; yeni zee-tr-N kimliği ve yeni fixture ekle." >&2
        exit 1
    fi
done <<< "$korunanlar"

echo "Yayımlanmış morfoloji profil fixture'ları değişmedi."
