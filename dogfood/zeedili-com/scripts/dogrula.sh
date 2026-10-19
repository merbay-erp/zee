#!/usr/bin/env bash
set -euo pipefail

BETIK_KOKU="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SITE_KOKU="$(cd -- "$BETIK_KOKU/.." && pwd)"
DEPO_KOKU="$(cd -- "$SITE_KOKU/../.." && pwd)"
DIL_BIN="${DIL_BIN:-$DEPO_KOKU/compiler/target/release/dil}"

for betik in "$SITE_KOKU"/scripts/*.sh "$SITE_KOKU"/deploy/*.sh; do
  bash -n "$betik"
done
python3 "$BETIK_KOKU/surum-notlarini-denetle.py" --denetle

"$DIL_BIN" denetle "$SITE_KOKU"
"$DIL_BIN" dene "$SITE_KOKU"
node --check "$SITE_KOKU/static/site.js"
node --check "$SITE_KOKU/static/oturum/oturum.js"
node --check "$SITE_KOKU/static/oturum/programlar.js"
python3 "$BETIK_KOKU/oturum-programlari-uret.py" --denetle
node --check "$SITE_KOKU/static/playground.js"
node --check "$SITE_KOKU/static/zee-motor.js"
node --check "$BETIK_KOKU/yerel-onkapi.mjs"
xmllint --noout "$SITE_KOKU/static/sitemap.xml"
python3 -m json.tool "$SITE_KOKU/static/manifest.webmanifest" >/dev/null

for dosya in \
  favicon.ico favicon.svg og-cover.svg og-cover.png manifest.webmanifest \
  robots.txt sitemap.xml site.css site.js playground.css playground.js \
  zee-motor.js fonts/fraunces-latin.woff2 fonts/figtree-latin.woff2 \
  fonts/jetbrains-mono-latin.woff2 brand/yildiz.svg brand/cini-bordur-ince.svg brand/nav-bordur.svg brand/yildiz-altin.svg brand/lale.svg \
  brand/ayrac.svg brand/karo.svg brand/cini-bordur.svg brand/yildiz-desen-acik.svg brand/yildiz-desen-koyu.svg \
  playground/dil.wasm playground/dil.wasm.sha256 \
  brand/zee-mark-koyu.svg brand/zee-mark-acik.svg brand/zee-z-acik.svg \
  icons/favicon-16.png icons/favicon-32.png icons/apple-touch-icon.png \
  icons/app-192.png icons/app-512.png; do
  test -s "$SITE_KOKU/static/$dosya"
done

grep -Fq '#2b2f36' "$SITE_KOKU/static/brand/zee-mark-koyu.svg"
grep -Fq '#d9b877' "$SITE_KOKU/static/brand/zee-mark-koyu.svg"
grep -Fq '/assets/brand/zee-mark-koyu.svg?v=k190a1' "$SITE_KOKU/kaynak/gorunum.dil"
grep -Fq '/assets/site.css?v=k192a2' "$SITE_KOKU/kaynak/gorunum.dil"
grep -Fq '/assets/zee-motor.js?v=k190a1' "$SITE_KOKU/kaynak/gorunum.dil"
grep -Fq '/assets/playground.js?v=k192a1' "$SITE_KOKU/kaynak/gorunum.dil"
grep -Fq "/assets/playground/dil.wasm?v=k188p2" "$SITE_KOKU/static/zee-motor.js"
if grep -Fq 'blob/main/' "$SITE_KOKU"/kaynak/*.dil "$SITE_KOKU"/static/*.js; then
  echo "GitHub bağlantıları varsayılan dal master'a gitmeli (blob/main 404)." >&2
  exit 1
fi
grep -Fq "'wasm-unsafe-eval'" "$SITE_KOKU/deploy/nginx.zeedili.conf"
grep -Fq "'wasm-unsafe-eval'" "$SITE_KOKU/scripts/yerel-onkapi.mjs"

(cd "$SITE_KOKU/static/playground" && shasum -a 256 -c dil.wasm.sha256)
node "$DEPO_KOKU/scripts/wasm-abi-denetle.mjs" "$SITE_KOKU/static/playground/dil.wasm"

echo "zeedili.com kaynak, test ve statik doğrulamaları geçti."
