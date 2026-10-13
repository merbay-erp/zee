#!/usr/bin/env bash
set -euo pipefail

# Public zeedili.com'u Cloudflare üzerinden uçtan uca doğrular.
COZUCU="${ZEE_DNS_COZUCU:-1.1.1.1}"
IP="$(dig +short @"$COZUCU" A zeedili.com | head -1)"
test -n "$IP"
R=(--resolve "zeedili.com:443:$IP" --resolve "www.zeedili.com:443:$IP" --max-time 15 --silent)

for yol in / /playground/ /dil/ /ogren/ /ornekler/ /belgeler/ /manifesto/ \
  /yol-haritasi/ /topluluk/ /indir/ /canli /hazir /robots.txt /sitemap.xml \
  /belge/ /belge/dil-turu/ /belge/spec/ /belge/adr-081-resmi-site-ve-uretim-siniri/; do
  kod="$(curl "${R[@]}" -o /dev/null -w '%{http_code}' "https://zeedili.com$yol")"
  [[ "$kod" == 200 ]] || { echo "$yol → $kod" >&2; exit 1; }
done
[[ "$(curl "${R[@]}" -o /dev/null -w '%{http_code}' https://zeedili.com/playground)" == 308 ]]
[[ "$(curl "${R[@]}" -o /dev/null -w '%{http_code}' https://zeedili.com/belge/dil-turu)" == 308 ]]
[[ "$(curl "${R[@]}" -o /dev/null -w '%{redirect_url}' https://www.zeedili.com/)" == https://zeedili.com/ ]]
wasm_basligi="$(curl "${R[@]}" -I "https://zeedili.com/assets/playground/dil.wasm?v=k188p2")"
grep -qi '^content-type: application/wasm' <<<"$wasm_basligi"
ana_basligi="$(curl "${R[@]}" -I https://zeedili.com/)"
grep -qi "^content-security-policy:.*'wasm-unsafe-eval'" <<<"$ana_basligi"
grep -Fq "data-defter-kutusu" <<<"$(curl "${R[@]}" https://zeedili.com/)"
dersler="$(curl "${R[@]}" https://zeedili.com/playground/ | grep -o '<template data-ders' | wc -l | tr -d ' ')"
[[ "$dersler" -ge 15 ]] || { echo "Ders sayısı: $dersler" >&2; exit 1; }
yazi_basligi="$(curl "${R[@]}" -I https://zeedili.com/assets/fonts/fraunces-latin.woff2)"
grep -qi '^content-type: font/woff2' <<<"$yazi_basligi"
echo "zeedili.com public HTTPS yayını doğrulandı ($IP)."
