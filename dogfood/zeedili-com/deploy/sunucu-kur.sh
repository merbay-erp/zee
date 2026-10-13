#!/usr/bin/env bash
set -euo pipefail

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Bu betik root yetkisiyle çalıştırılmalı." >&2
  exit 1
fi

KAYNAK_KOKU="${1:-/tmp/zeedili-release}"
SITE_KOKU=/opt/zeedili
NGINX_HEDEF=/etc/nginx/sites-available/zeedili.com

test -x /opt/zee/bin/dil
test -f "$KAYNAK_KOKU/proje.dil"
test -f "$KAYNAK_KOKU/static/site.css"

if ! id zeedili >/dev/null 2>&1; then
  useradd --system --home-dir "$SITE_KOKU" --shell /usr/sbin/nologin zeedili
fi

install -d -o zeedili -g zeedili -m 0755 "$SITE_KOKU"
rsync -a --delete --exclude .zee --exclude deploy --exclude scripts "$KAYNAK_KOKU/" "$SITE_KOKU/"
install -d -o zeedili -g zeedili -m 0700 "$SITE_KOKU/.zee"
chown -R zeedili:zeedili "$SITE_KOKU"
find "$SITE_KOKU" -type d -exec chmod 0755 {} +
find "$SITE_KOKU" -type f -exec chmod 0644 {} +
chmod 0700 "$SITE_KOKU/.zee"

/opt/zee/bin/dil denetle "$SITE_KOKU"
/opt/zee/bin/dil dene "$SITE_KOKU"

install -m 0644 "$KAYNAK_KOKU/deploy/zeedili.service" /etc/systemd/system/zeedili.service

systemctl daemon-reload
systemctl enable zeedili.service
systemctl restart zeedili.service

HAZIR=0
for _ in {1..40}; do
  if curl --fail --silent --max-time 1 \
    -H 'Host: zeedili.com' \
    -H 'Forwarded: for=127.0.0.1;proto=https;host=zeedili.com' \
    http://127.0.0.1:8126/canli >/dev/null; then
    HAZIR=1
    break
  fi
  sleep 0.25
done
test "$HAZIR" -eq 1
systemctl is-active --quiet zeedili.service

ANA_SAYFA="$(curl --fail --silent --max-time 3 \
  -H 'Host: zeedili.com' \
  -H 'Forwarded: for=127.0.0.1;proto=https;host=zeedili.com' \
  http://127.0.0.1:8126/)"
JSON_LD="$(printf '%s' "$ANA_SAYFA" | sed -n "s/.*<script type='application\/ld+json'>\(.*\)<\/script>.*/\1/p")"
if [[ -z "$JSON_LD" ]]; then
  echo "Çalışan sayfadan JSON-LD içeriği çıkarılamadı." >&2
  exit 1
fi
JSON_LD_HASH="sha256-$(printf '%s' "$JSON_LD" | openssl dgst -sha256 -binary | openssl base64 -A)"

if [[ -f /etc/letsencrypt/live/zeedili.com/fullchain.pem && -f /etc/letsencrypt/live/zeedili.com/privkey.pem ]]; then
  sed "s|__JSON_LD_HASH__|'$JSON_LD_HASH'|g" "$KAYNAK_KOKU/deploy/nginx.zeedili.conf" > "$NGINX_HEDEF"
  ln -sfn "$NGINX_HEDEF" /etc/nginx/sites-enabled/zeedili.com
  nginx -t
  systemctl reload nginx
  echo "zeedili.com worker ve HTTPS Nginx yapılandırması hazır."
else
  echo "zeedili.com worker hazır; DNS ve Let's Encrypt sertifikası beklendiği için Nginx sitesi etkinleştirilmedi."
fi
