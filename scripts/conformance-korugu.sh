#!/usr/bin/env bash
set -euo pipefail

taban="${1:-}"
if [[ -z "$taban" ]]; then
    echo "Kullanım: scripts/conformance-korugu.sh <taban-git-revizyonu>" >&2
    exit 2
fi
if [[ "$taban" =~ ^0+$ ]]; then
    echo "İlk depo gönderimi: korunacak önceki conformance artefaktı yok."
    exit 0
fi
if ! git cat-file -e "${taban}^{commit}" 2>/dev/null; then
    echo "Conformance koruğu taban revizyonunu bulamadı: $taban" >&2
    exit 2
fi

korunanlar="$(
    git ls-tree -r --name-only "$taban" -- compiler/tests/fixtures/ conformance/ |
        grep -E '^(compiler/tests/fixtures/morfoloji-zee-tr-[0-9]+\.sha256|conformance/.+\.json)$' || true
)"
while IFS= read -r dosya; do
    [[ -z "$dosya" ]] && continue
    if ! git diff --quiet "$taban" -- "$dosya"; then
        echo "IMMUTABLE CONFORMANCE ARTEFAKTI DEĞİŞTİ: $dosya" >&2
        echo "Yayımlanmış veriyi güncelleme; yeni profil/şema kimliği ve yeni dosya ekle." >&2
        exit 1
    fi
done <<< "$korunanlar"

echo "Yayımlanmış conformance artefaktları değişmedi."
