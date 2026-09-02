#!/usr/bin/env bash
set -euo pipefail

depo_koku="$(git rev-parse --show-toplevel)"
cd "$depo_koku"

surum="$(git describe --tags --always HEAD 2>/dev/null)"
cikti="${1:-dist/zee-kaynak-${surum}.zip}"

if [[ "$cikti" != /* ]]; then
    cikti="$depo_koku/$cikti"
fi

case "$cikti" in
    "$depo_koku"/*) ;;
    *)
        echo "Arşiv yalnız depo içindeki açık bir çıktı yoluna yazılabilir." >&2
        exit 2
        ;;
esac

mkdir -p "$(dirname "$cikti")"
gecici_klasor="$(mktemp -d "${TMPDIR:-/tmp}/zee-kaynak-arsivi.XXXXXX")"
gecici="${cikti}.gecici-$$"
gecici_ozet="${cikti}.sha256.gecici-$$"
manifest="$gecici_klasor/KAYNAK-SHA256.txt"
trap 'rm -rf "$gecici_klasor"; rm -f "$gecici" "$gecici_ozet"' EXIT

sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum | awk '{print $1}'
    else
        shasum -a 256 | awk '{print $1}'
    fi
}

while IFS= read -r dosya; do
    if [[ "$dosya" == *$'\n'* ]]; then
        echo "Satır sonu taşıyan Git yolu kaynak manifestinde desteklenmiyor." >&2
        exit 1
    fi
    ozet="$(git show "HEAD:$dosya" | sha256)"
    printf '%s  %s\n' "$ozet" "$dosya" >> "$manifest"
done < <(git ls-tree -r --name-only HEAD)

commit_zamani="$(git show -s --format=%ct HEAD)"
git archive --format=zip --mtime="@$commit_zamani" --prefix=zee/ \
    --add-file="$manifest" --output="$gecici" HEAD

if unzip -Z1 "$gecici" | grep -Eq '(^|/)(__MACOSX|target|artifacts|\.git)(/|$)|\.(profraw|profdata)$'; then
    echo "Arşiv yasaklı build/metadata girdisi içeriyor." >&2
    exit 1
fi
if ! unzip -Z1 "$gecici" | grep -qx 'zee/KAYNAK-SHA256.txt'; then
    echo "Arşiv kaynak SHA-256 manifestini taşımıyor." >&2
    exit 1
fi

arsiv_ozeti="$(sha256 < "$gecici")"
printf '%s  %s\n' "$arsiv_ozeti" "$(basename "$cikti")" > "$gecici_ozet"
mv "$gecici" "$cikti"
mv "$gecici_ozet" "${cikti}.sha256"
rm -rf "$gecici_klasor"
trap - EXIT
echo "Temiz kaynak arşivi: $cikti"
echo "Arşiv özeti: ${cikti}.sha256"
