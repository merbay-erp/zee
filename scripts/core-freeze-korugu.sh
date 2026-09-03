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
dogfood_manifest="docs/dogfood-projeleri-v1.tsv"
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
if [[ ! -f "$dogfood_manifest" ]] \
    || [[ "$(head -n 1 "$dogfood_manifest")" != "# zee-dogfood-projeleri-1" ]]; then
    echo "DOGFOOD ÜRÜN KAYDI EKSİK YA DA ŞEMASI BİLİNMİYOR: $dogfood_manifest" >&2
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

if [[ -n "$taban" ]] && git cat-file -e "${taban}:${dogfood_manifest}" 2>/dev/null; then
    while IFS= read -r eski_satir; do
        [[ -z "$eski_satir" || "$eski_satir" == \#* ]] && continue
        if ! grep -Fqx "$eski_satir" "$dogfood_manifest"; then
            echo "DOGFOOD ÜRÜN KAYDI YENİDEN YAZILDI: ${eski_satir%%$'\t'*}" >&2
            exit 1
        fi
    done < <(git show "${taban}:${dogfood_manifest}")
fi

if ! awk -F '\t' '
    /^#/ || NF == 0 { next }
    NF != 4 || slug[$1]++ || kok[$2]++ { exit 1 }
' "$dogfood_manifest"; then
    echo "DOGFOOD ÜRÜN KAYDI TEKİL DEĞİL" >&2
    exit 1
fi
while IFS=$'\t' read -r slug kok koken_commit durum fazladan; do
    [[ -z "$slug" || "$slug" == \#* ]] && continue
    if [[ -n "${fazladan:-}" || ! "$slug" =~ ^[a-z0-9][a-z0-9._-]*$ \
        || "$kok" == /* || "$kok" == *".."* \
        || "$kok" == "." || ! -d "$kok" \
        || ! "$koken_commit" =~ ^[0-9a-f]{40}$ || ! "$durum" =~ ^(active|retired)$ ]]; then
        echo "DOGFOOD ÜRÜN KAYDI GEÇERSİZ: $slug" >&2
        exit 1
    fi
done < "$dogfood_manifest"

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

dogfood_butunlugunu_iste() {
    local commit="$1" urun="$2" is="$3" etkilenen="$4" karar="$5"
    local urun_satiri kok durum
    urun_satiri="$(awk -F '\t' -v urun="$urun" '$1 == urun { print }' "$dogfood_manifest")"
    if [[ "$(printf '%s\n' "$urun_satiri" | awk 'NF { sayi++ } END { print sayi+0 }')" != "1" ]]; then
        echo "CORE FREEZE DOGFOOD ÜRÜNÜ KAYITLI DEĞİL: $commit ($urun)" >&2
        exit 1
    fi
    IFS=$'\t' read -r _ kok _ durum <<< "$urun_satiri"
    if [[ "$durum" != "active" ]]; then
        echo "CORE FREEZE DOGFOOD ÜRÜNÜ ETKİN DEĞİL: $commit ($urun)" >&2
        exit 1
    fi
    if ! grep -Eq "(^|[^[:alnum:]-])${is}([^0-9]|$)" \
        docs/oncelikli-backlog.md kararlar/gunluk.md; then
        echo "CORE FREEZE DOGFOOD İŞİ KAYITLI DEĞİL: $commit ($is)" >&2
        exit 1
    fi
    case "$etkilenen" in
        "$kok"/*.dil) ;;
        *)
            echo "CORE FREEZE ETKİLENEN PROJE KAYITLI ÜRÜN KÖKÜNDE DEĞİL: $commit ($etkilenen)" >&2
            exit 1
            ;;
    esac
    if ! grep -Fq "$is" "$karar" && ! grep -Fq "$urun" "$karar"; then
        echo "CORE FREEZE KARARI DOGFOOD İŞİNE/ÜRÜNÜNE BAĞLI DEĞİL: $commit ($karar)" >&2
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
            dogfood_butunlugunu_iste "$commit" "$urun" "$is" "$etkilenen_proje" "$karar"
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
