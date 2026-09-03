#!/usr/bin/env bash
set -euo pipefail

depo_koku="$(git rev-parse --show-toplevel)"
cd "$depo_koku"

taban="${1:-}"
if [[ -z "$taban" ]]; then
    echo "Kullanım: scripts/core-freeze-korugu.sh <taban-git-revizyonu>" >&2
    exit 2
fi
if [[ "$taban" =~ ^0+$ ]]; then
    taban=""
elif ! git cat-file -e "${taban}^{commit}" 2>/dev/null; then
    echo "CORE FREEZE koruğu taban revizyonunu bulamadı: $taban" >&2
    exit 2
fi

manifest="docs/core-freeze-beyanlari-v1.tsv"
semantic_manifest="docs/compiler-degisiklik-beyanlari-v1.tsv"
if [[ ! -f "$manifest" ]]; then
    echo "CORE FREEZE BEYAN MANİFESTİ EKSİK: $manifest" >&2
    exit 1
fi
if [[ "$(head -n 1 "$manifest")" != "# zee-core-freeze-beyanlari-1" ]]; then
    echo "CORE FREEZE BEYAN ŞEMASI BİLİNMİYOR" >&2
    exit 1
fi
if [[ ! -f "$semantic_manifest" ]]; then
    echo "COMPILER DEĞİŞİKLİK BEYAN MANİFESTİ EKSİK: $semantic_manifest" >&2
    exit 1
fi

enforcement_parent="$(awk -F '\t' '$1 == "# enforcement_parent" { print $2 }' "$manifest")"
if [[ ! "$enforcement_parent" =~ ^[0-9a-f]{40}$ ]] \
    || ! git cat-file -e "${enforcement_parent}^{commit}" 2>/dev/null; then
    echo "CORE FREEZE BAŞLANGICI GEÇERSİZ" >&2
    exit 1
fi

if [[ -n "$taban" ]] && git cat-file -e "${taban}:${manifest}" 2>/dev/null; then
    while IFS= read -r eski_satir; do
        [[ -z "$eski_satir" || "$eski_satir" == \#* ]] && continue
        eski_commit="${eski_satir%%$'\t'*}"
        if ! grep -Fqx "$eski_satir" "$manifest"; then
            echo "CORE FREEZE BEYANI YENİDEN YAZILDI: $eski_commit" >&2
            exit 1
        fi
    done < <(git show "${taban}:${manifest}")
fi

dosya_iste() {
    local commit="$1" alan="$2" deger="$3"
    if [[ "$deger" == "-" || "$deger" == /* || "$deger" == *".."* || ! -f "$deger" ]]; then
        echo "CORE FREEZE $alan KANITI GEÇERSİZ: $commit ($deger)" >&2
        exit 1
    fi
}

karar_iste() {
    local commit="$1" karar="$2"
    if [[ ! "$karar" =~ ^(spec|rfcs|adr)/[^/]+\.md$ || ! -f "$karar" ]]; then
        echo "CORE FREEZE ADR/SPEC KANITI GEÇERSİZ: $commit ($karar)" >&2
        exit 1
    fi
}

while IFS= read -r commit; do
    [[ -z "$commit" ]] && continue
    eslesmeler="$(awk -F '\t' -v commit="$commit" '$1 == commit { print }' "$manifest")"
    if [[ "$(printf '%s\n' "$eslesmeler" | awk 'NF { sayi++ } END { print sayi+0 }')" != "1" ]]; then
        echo "CORE FREEZE COMPILER COMMIT'İ TEKİL BEYAN TAŞIMIYOR: $commit" >&2
        exit 1
    fi
    IFS=$'\t' read -r _ sinif urun is reproducer etkilenen_proje minimalite karar <<< "$eslesmeler"
    if [[ -z "${karar:-}" || ${#minimalite} -lt 40 ]]; then
        echo "CORE FREEZE MİNİMALİTE GEREKÇESİ YETERSİZ: $commit" >&2
        exit 1
    fi

    semantic_satir="$(awk -F '\t' -v commit="$commit" '$1 == commit { print }' "$semantic_manifest")"
    if [[ "$(printf '%s\n' "$semantic_satir" | awk 'NF { sayi++ } END { print sayi+0 }')" != "1" ]]; then
        echo "CORE FREEZE SEMANTIC BEYANI BULAMADI: $commit" >&2
        exit 1
    fi
    IFS=$'\t' read -r _ semantic_sinif _ _ <<< "$semantic_satir"

    case "$sinif" in
        maintenance)
            if [[ "$semantic_sinif" != "maintenance" \
                || "$urun" != "-" || "$is" != "-" || "$reproducer" != "-" \
                || "$etkilenen_proje" != "-" || "$karar" != "-" ]]; then
                echo "CORE FREEZE MAINTENANCE BEYANI UYUŞMUYOR: $commit" >&2
                exit 1
            fi
            ;;
        bugfix)
            if [[ "$semantic_sinif" != "semantic-bugfix" || "$urun" != "-" \
                || ! "$is" =~ ^K-[0-9]+$ || "$etkilenen_proje" != "-" || "$karar" != "-" ]]; then
                echo "CORE FREEZE BUGFIX BEYANI UYUŞMUYOR: $commit" >&2
                exit 1
            fi
            dosya_iste "$commit" "REPRODUCER" "$reproducer"
            ;;
        security|correctness)
            if [[ ! "$semantic_sinif" =~ ^semantic-(bugfix|change)$ \
                || "$urun" != "-" || ! "$is" =~ ^K-[0-9]+$ || "$etkilenen_proje" != "-" ]]; then
                echo "CORE FREEZE $sinif BEYANI UYUŞMUYOR: $commit" >&2
                exit 1
            fi
            dosya_iste "$commit" "REPRODUCER" "$reproducer"
            karar_iste "$commit" "$karar"
            ;;
        dogfood-change)
            if [[ "$semantic_sinif" != "semantic-change" \
                || ! "$urun" =~ ^[a-z0-9][a-z0-9._-]*$ || ! "$is" =~ ^K-[0-9]+$ ]]; then
                echo "CORE FREEZE DOGFOOD BEYANI UYUŞMUYOR: $commit" >&2
                exit 1
            fi
            dosya_iste "$commit" "REPRODUCER" "$reproducer"
            dosya_iste "$commit" "ETKİLENEN PROJE" "$etkilenen_proje"
            karar_iste "$commit" "$karar"
            ;;
        *)
            echo "BİLİNMEYEN CORE FREEZE SINIFI: $commit ($sinif)" >&2
            exit 1
            ;;
    esac
done < <(git rev-list --reverse "${enforcement_parent}..HEAD" -- compiler/src)

while IFS=$'\t' read -r commit _ _ _ _ _ _ _; do
    [[ -z "$commit" || "$commit" == \#* ]] && continue
    if ! git merge-base --is-ancestor "$enforcement_parent" "$commit" \
        || ! git merge-base --is-ancestor "$commit" HEAD \
        || ! git diff-tree --no-commit-id --name-only -r "$commit" | grep -q '^compiler/src/'; then
        echo "SAHİPSİZ CORE FREEZE BEYANI: $commit" >&2
        exit 1
    fi
done < "$manifest"

echo "CORE FREEZE bütün compiler kaynak commit'lerinde yürütülüyor."
