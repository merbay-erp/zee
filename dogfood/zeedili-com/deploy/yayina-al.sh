#!/usr/bin/env bash
set -euo pipefail

# zeedili.com public HTTPS kapısı (sunucu A).
# sunucu A'ün 80/443 portları yalnız Cloudflare'e açıktır; DNS kayıtları Cloudflare
# proxy'li (turuncu bulut) olmalıdır. Bu yüzden HTTP-01 değil, hesabın mevcut
# certbot Cloudflare kimliğiyle DNS-01 kullanılır; yenileme de aynı yoldan olur.

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Bu betik root yetkisiyle çalıştırılmalı." >&2
  exit 1
fi

KAYNAK_KOKU="${1:-/tmp/zeedili-release}"
CF_KIMLIGI="${ZEE_CF_KIMLIGI:?Özel Cloudflare kimlik dosyasını ZEE_CF_KIMLIGI ile belirt}"

for ad in zeedili.com www.zeedili.com; do
  if [[ -z "$(getent ahostsv4 "$ad" | awk 'NR==1{print $1}')" ]]; then
    echo "DNS hazır değil: $ad çözülmüyor." >&2
    exit 1
  fi
done
test -f "$CF_KIMLIGI"

certbot certonly --dns-cloudflare \
  --dns-cloudflare-credentials "$CF_KIMLIGI" \
  --dns-cloudflare-propagation-seconds 30 \
  --cert-name zeedili.com \
  --domains zeedili.com,www.zeedili.com \
  --non-interactive --agree-tos --keep-until-expiring

bash "$KAYNAK_KOKU/deploy/sunucu-kur.sh" "$KAYNAK_KOKU"
curl --fail --silent --show-error --max-time 10 \
  --resolve "zeedili.com:443:${ZEE_ORIGIN_IP:?Özel origin adresini ZEE_ORIGIN_IP ile belirt}" \
  https://zeedili.com/canli >/dev/null
echo "zeedili.com origin HTTPS yayını ve sertifikası doğrulandı."
