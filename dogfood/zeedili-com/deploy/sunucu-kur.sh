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
# .zee çalışan işçilerin kalıcı web durumudur; işçiler orada atomik geçici
# dosya yazıp yeniden adlandırır. Canlı trafikte özyinelemeli chown/chmod bu
# dosyalarla yarışıp dağıtımı düşürür; .zee yalnız kendi dizin hakkıyla ele alınır.
find "$SITE_KOKU" -path "$SITE_KOKU/.zee" -prune -o -exec chown zeedili:zeedili {} +
find "$SITE_KOKU" -path "$SITE_KOKU/.zee" -prune -o -type d -exec chmod 0755 {} +
find "$SITE_KOKU" -path "$SITE_KOKU/.zee" -prune -o -type f -exec chmod 0644 {} +
chown zeedili:zeedili "$SITE_KOKU/.zee"
chmod 0700 "$SITE_KOKU/.zee"

/opt/zee/bin/dil denetle "$SITE_KOKU"
/opt/zee/bin/dil dene "$SITE_KOKU"

install -m 0644 "$KAYNAK_KOKU/deploy/zeedili.socket" /etc/systemd/system/zeedili.socket
install -m 0644 "$KAYNAK_KOKU/deploy/zeedili.service" /etc/systemd/system/zeedili.service

systemctl daemon-reload

# K-343: ExecStart'ı ezen bir drop-in (ör. sürümlü dil ikilisi) işçi grubu ve
# soket bayraklarını taşımazsa yönetici 8126'yı kendisi bağlamaya çalışır ve
# soketle çakışır. Hiçbir şey durdurulmadan önce reddedilir.
ETKIN_EXEC="$(systemctl show --property=ExecStart --value zeedili.service)"
for bayrak in --web-isci-sayisi --web-systemd-soketi "--web-worker-port 8126"; do
  if [[ "$ETKIN_EXEC" != *"$bayrak"* ]]; then
    echo "zeedili.service etkin ExecStart '$bayrak' taşımıyor (drop-in'i denetle: systemctl cat zeedili.service)." >&2
    exit 1
  fi
done

systemctl enable zeedili.socket zeedili.service
if ! systemctl is-active --quiet zeedili.socket; then
  # Tek seferlik geçiş: eski tek süreçli worker 8126'yı kendisi tutuyorsa soket
  # bağlanamaz. Yalnız bu ilk geçişte kısa bir kesinti olur.
  systemctl stop zeedili.service
  systemctl start zeedili.socket
fi
# Soket systemd'de kaldığı için bu yeniden başlatma bağlantı kaybetmez.
systemctl restart zeedili.service

HAZIR=0
for _ in {1..120}; do
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
systemctl is-active --quiet zeedili.socket
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
  echo "zeedili.com işçi grubu ve HTTPS Nginx yapılandırması hazır."
else
  echo "zeedili.com işçi grubu hazır; DNS ve Let's Encrypt sertifikası beklendiği için Nginx sitesi etkinleştirilmedi."
fi
