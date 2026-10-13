#!/usr/bin/env bash
set -euo pipefail

BETIK_KOKU="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SITE_KOKU="$(cd -- "$BETIK_KOKU/.." && pwd)"
SUNUCU="${ZEE_SUNUCU:?SSH bağlantı adını ZEE_SUNUCU ile belirt}"
UZAK_KOK="/tmp/zeedili-release"

"$BETIK_KOKU/dogrula.sh"
"$BETIK_KOKU/belgeleri-uret.sh"

ssh "$SUNUCU" "mkdir -p '$UZAK_KOK'"
rsync -az --delete --delete-excluded --exclude .zee "$SITE_KOKU/" "$SUNUCU:$UZAK_KOK/"
ssh -t "$SUNUCU" "sudo bash '$UZAK_KOK/deploy/sunucu-kur.sh' '$UZAK_KOK'"

if [[ -n "$(dig +short @"${ZEE_DNS_COZUCU:-1.1.1.1}" A zeedili.com | head -1)" ]]; then
  "$BETIK_KOKU/canli-dogrula.sh"
else
  echo "sunucu A worker kuruldu; zeedili.com henüz çözülmüyor."
fi
