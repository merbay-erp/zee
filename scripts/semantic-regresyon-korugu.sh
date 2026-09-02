#!/usr/bin/env bash
set -euo pipefail

depo_koku="$(git rev-parse --show-toplevel)"
cd "$depo_koku"

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
guncel_manifest="regression/v2.tsv"
if ! git cat-file -e "${taban}:regression/v2.tsv" 2>/dev/null \
    && ! git cat-file -e "${taban}:regression/v1.tsv" 2>/dev/null; then
    echo "Taban revizyonda semantic regresyon manifesti yok; ilk korpus ekleniyor."
    exit 0
fi
if [[ ! -f "$guncel_manifest" ]]; then
    echo "SEMANTIC REGRESYON MANİFESTİ SİLİNDİ: $guncel_manifest" >&2
    exit 1
fi

if git cat-file -e "${taban}:regression/v2.tsv" 2>/dev/null; then
    taban_manifest="regression/v2.tsv"
    taban_sema="v2"
else
    taban_manifest="regression/v1.tsv"
    taban_sema="v1"
fi

while IFS=$'\t' read -r vaka bug ucuncu dorduncu besinci altinci yedinci sekizinci dokuzuncu onuncu onbirinci onikinci; do
    [[ -z "$vaka" || "$vaka" == \#* ]] && continue
    if [[ "$taban_sema" == "v2" ]]; then
        fixed_by="$ucuncu"
        introduced_by="$dorduncu"
        guaranteed_since="$besinci"
        dosya="$onikinci"
    else
        fixed_by=""
        introduced_by=""
        guaranteed_since=""
        dosya="$dokuzuncu"
    fi
    eslesmeler="$(awk -F '\t' -v vaka="$vaka" '$1 == vaka { print }' "$guncel_manifest")"
    if [[ -z "$eslesmeler" ]]; then
        echo "KALICI SEMANTIC REGRESYON VAKASI SİLİNDİ: $vaka ($bug)" >&2
        exit 1
    fi
    if [[ "$(printf '%s\n' "$eslesmeler" | wc -l | tr -d ' ')" != "1" ]]; then
        echo "SEMANTIC REGRESYON VAKASI YİNELENDİ: $vaka" >&2
        exit 1
    fi
    IFS=$'\t' read -r _ guncel_bug guncel_fixed_by guncel_introduced_by guncel_guaranteed_since _ _ _ _ _ _ guncel_dosya <<< "$eslesmeler"
    if [[ "$guncel_bug" != "$bug" || "$guncel_dosya" != "$dosya" ]]; then
        echo "SEMANTIC REGRESYON KİMLİĞİ YENİDEN KULLANILDI: $vaka" >&2
        echo "Eski: $bug -> $dosya" >&2
        echo "Yeni: $guncel_bug -> $guncel_dosya" >&2
        exit 1
    fi
    introduced_gecisi_gecerli="false"
    if [[ "$introduced_by" == "-" \
        && "$guncel_introduced_by" =~ ^[0-9a-f]{40}$ ]] \
        && git merge-base --is-ancestor "$guncel_introduced_by" "$guncel_fixed_by"; then
        introduced_gecisi_gecerli="true"
    fi
    if [[ "$taban_sema" == "v2" ]] \
        && { [[ "$guncel_fixed_by" != "$fixed_by" ]] \
            || [[ "$guncel_guaranteed_since" != "$guaranteed_since" ]] \
            || { [[ "$guncel_introduced_by" != "$introduced_by" ]] \
                && [[ "$introduced_gecisi_gecerli" != "true" ]]; }; }; then
        echo "SEMANTIC REGRESYON PROVENANCE'I YENİDEN YAZILDI: $vaka" >&2
        exit 1
    fi
done < <(git show "${taban}:${taban_manifest}")

while IFS=$'\t' read -r commit baslik; do
    if [[ "$baslik" =~ (^|[[:space:]:])(fix|bugfix|bug|duzelt|düzelt)([[:space:]:]|$) ]]; then
        if ! awk -F '\t' -v commit="$commit" '$1 !~ /^#/ && $3 == commit { bulundu=1 } END { exit !bulundu }' "$guncel_manifest"; then
            echo "COMPILER BUG DÜZELTMESİ REGRESYON PROVENANCE'I TAŞIMIYOR: $commit $baslik" >&2
            echo "Önce düzeltme commit'i, sonra fixed_by alanıyla minimal regression/v2.tsv kaydı ekleyin." >&2
            exit 1
        fi
    fi
done < <(git log --format='%H%x09%s' "${taban}..HEAD" -- compiler/src)

echo "Yayımlanmış semantic regresyon kimliği/provenance'ı korunuyor; compiler bug fix'leri fixture sahibidir."
