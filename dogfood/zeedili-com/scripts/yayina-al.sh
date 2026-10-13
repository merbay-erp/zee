#!/usr/bin/env bash
set -euo pipefail

BETIK_KOKU="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SITE_KOKU="$(cd -- "$BETIK_KOKU/.." && pwd)"
SUNUCU="${ZEE_SUNUCU:?SSH bağlantı adını ZEE_SUNUCU ile belirt}"
UZAK_KOK=/tmp/zeedili-release
# Yerel/kurumsal çözücü negatif yanıtı önbellekte tutabilir; herkese açık çözücü sorulur.
COZUCU="${ZEE_DNS_COZUCU:-1.1.1.1}"

"$BETIK_KOKU/dogrula.sh"
"$BETIK_KOKU/belgeleri-uret.sh"

for ad in zeedili.com www.zeedili.com; do
  if [[ -z "$(dig +short @"$COZUCU" A "$ad" | head -1)" ]]; then
    echo "$ad henüz çözülmüyor (Cloudflare'de proxy'li A kaydı gerekir)." >&2
    exit 1
  fi
done

ssh "$SUNUCU" "mkdir -p '$UZAK_KOK'"
rsync -az --delete --delete-excluded --exclude .zee "$SITE_KOKU/" "$SUNUCU:$UZAK_KOK/"
ssh -t "$SUNUCU" "sudo bash '$UZAK_KOK/deploy/yayina-al.sh' '$UZAK_KOK'"

"$BETIK_KOKU/canli-dogrula.sh"
