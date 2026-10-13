#!/usr/bin/env bash
set -euo pipefail

# Depo Markdown belgelerini Zee üreticisiyle site sayfalarına çevirir.
# Çıktı: uretilen/belge/<ad>.html ve uretilen/belge/index.html (git dışı).

BETIK_KOKU="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SITE_KOKU="$(cd -- "$BETIK_KOKU/.." && pwd)"
DEPO_KOKU="$(cd -- "$SITE_KOKU/../.." && pwd)"
DIL_BIN="${DIL_BIN:-$DEPO_KOKU/compiler/target/release/dil}"
CIKTI="$SITE_KOKU/uretilen"

rm -rf "$CIKTI"
mkdir -p "$CIKTI/belge" "$CIKTI/md"

: > "$CIKTI/katalog.txt"
cd "$DEPO_KOKU"
for dosya in docs/*.md spec/*.md rfcs/*.md adr/*.md editors/README.md; do
  test -f "$dosya" || continue
  mkdir -p "$CIKTI/md/$(dirname "$dosya")"
  cp "$DEPO_KOKU/$dosya" "$CIKTI/md/$dosya"
  printf '%s\n' "$dosya" >> "$CIKTI/katalog.txt"
done

sayi=0
while IFS= read -r dosya; do
  printf '%s\n' "$dosya" | "$DIL_BIN" çalıştır "$SITE_KOKU/kaynak/belge_uret.dil" > /dev/null
  sayi=$((sayi + 1))
done < "$CIKTI/katalog.txt"
printf 'DİZİN\n' | "$DIL_BIN" çalıştır "$SITE_KOKU/kaynak/belge_uret.dil" > /dev/null

uretilen="$(find "$CIKTI/belge" -name '*.html' | wc -l | tr -d ' ')"
test "$uretilen" -eq $((sayi + 1))
rm -rf "$CIKTI/md"

# Üretilen sayfalardaki bütün /belge/ bağlantıları ve çapaları var olmalı.
python3 - "$CIKTI/belge" <<'PY'
import glob, html, os, re, sys
kok = sys.argv[1]
kimlikler = {}
for dosya in glob.glob(os.path.join(kok, '*.html')):
    ad = os.path.basename(dosya)[:-5]
    kimlikler[ad] = set(re.findall(r" id='([^']+)'", open(dosya, encoding='utf-8').read()))
kirik = []
for dosya in glob.glob(os.path.join(kok, '*.html')):
    for adres in re.findall(r"href='(/belge/[^']*)'", open(dosya, encoding='utf-8').read()):
        m = re.match(r'^/belge/([a-z0-9-]*)/?(?:#(.*))?$', adres)
        hedef = (m.group(1) or 'index') if m else None
        if not m or hedef not in kimlikler or (m.group(2) and html.unescape(m.group(2)) not in kimlikler[hedef]):
            kirik.append(f"{os.path.basename(dosya)} → {adres}")
if kirik:
    print('Kırık belge bağlantıları:', *kirik[:20], sep='\n  ', file=sys.stderr)
    sys.exit(1)
PY
echo "$sayi belge ve dizin sayfası üretildi; iç bağlantılar sağlam."
