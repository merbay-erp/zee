#!/usr/bin/env bash
set -euo pipefail

depo_koku="$(git rev-parse --show-toplevel)"
cd "$depo_koku"

surum="$(git describe --tags --always --dirty 2>/dev/null)"
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
gecici="${cikti}.gecici-$$"
trap 'rm -f "$gecici"' EXIT

git archive --format=zip --prefix=zee/ --output="$gecici" HEAD

if unzip -Z1 "$gecici" | grep -Eq '(^|/)(__MACOSX|target|artifacts|\.git)(/|$)|\.(profraw|profdata)$'; then
    echo "Arşiv yasaklı build/metadata girdisi içeriyor." >&2
    exit 1
fi

mv "$gecici" "$cikti"
trap - EXIT
echo "Temiz kaynak arşivi: $cikti"
