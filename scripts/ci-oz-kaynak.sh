#!/usr/bin/env bash
# K-214: özel depoda self Git dependency, checkout'taki aynı exact nesneden gelir.
set -euo pipefail
[[ "${CI:-}" == true && -n "${GITHUB_ENV:-}" ]] || {
    echo 'Bu hazırlık yalnız geçici CI ortamında çalışır.' >&2
    exit 2
}
depo_koku="$(git rev-parse --show-toplevel)"
rev="$(awk -F '\t' '!/^#/ && $1 == "postgres-protocol" {print $8}' "$depo_koku/compiler/third_party/patches-v2.tsv")"
# Lock'un rev'i değiştirilmez. Eksik/sığ checkout anonim ağ isteğine düşemez.
git -C "$depo_koku" cat-file -e "${rev}^{commit}"
if [[ "$depo_koku" == [A-Za-z]:/* ]]; then
    yerel_uri="file:///$depo_koku"
else
    yerel_uri="file://$depo_koku"
fi
git config --global --replace-all "url.${yerel_uri}.insteadOf" 'https://github.com/merbay-erp/zee'
printf 'CARGO_NET_GIT_FETCH_WITH_CLI=true\n' >> "$GITHUB_ENV"
echo 'CI self kaynak nesnesi doğrulandı; Cargo URI ve rev pini korunuyor.'
