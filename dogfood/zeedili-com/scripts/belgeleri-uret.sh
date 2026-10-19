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

# Çalışma bütçesine sığmayan belgeyi blok sınırlarından (kod bloğu dışındaki boş satırlar) parçalara bölüp
# her parçayı ayrı çalıştırmada dönüştürür; gövdeyi birleştirip tek sayfa üretir. Sığmazsa küçülterek dener.
parcali_uret() {
  local dosya="$1" boy ad sira ilk
  for boy in 40000 20000 10000; do
    rm -rf "$CIKTI/parca"; mkdir -p "$CIKTI/parca"
    python3 - "$CIKTI/md/$dosya" "$boy" <<'PY_BOL' || return 1
import sys
yol, boy = sys.argv[1], int(sys.argv[2])
satirlar = open(yol, encoding='utf-8').read().split('\n')
parcalar, simdiki, uzunluk, kodda = [], [], 0, False
for satir in satirlar:
    if satir.startswith('```'):
        kodda = not kodda
    simdiki.append(satir); uzunluk += len(satir.encode()) + 1
    if not kodda and satir.strip() == '' and uzunluk >= boy:
        parcalar.append(simdiki); simdiki, uzunluk = [], 0
if simdiki:
    parcalar.append(simdiki)
for i, p in enumerate(parcalar, 1):
    open(f'{yol}.{i}', 'w', encoding='utf-8').write('\n'.join(p) + '\n')
pass
PY_BOL
    sira=1; local tamam=1
    while [ -f "$CIKTI/md/$dosya.$sira" ]; do
      printf 'PARÇA:%s:%s\n' "$sira" "$dosya" | "$DIL_BIN" çalıştır "$SITE_KOKU/kaynak/belge_uret.dil" > /dev/null 2>"$CIKTI/hata.txt" || { tamam=0; break; }
      sira=$((sira + 1))
    done
    if [ $tamam = 1 ]; then
      # Belge adı zee'nin belgeAdi kuralından gelir: ilk parçanın dosya adı.
      ilk="$(ls "$CIKTI/parca/"*.1.html)"; ad="$(basename "$ilk" .1.html)"
      # Dosya yazımı her parçanın sonuna satır sonu ekler; birleştirirken yalnız o tek satır sonu atılır.
      python3 - "$CIKTI/parca" "$ad" <<'PY_BIRLESTIR'
import sys
from pathlib import Path
kok, ad = Path(sys.argv[1]), sys.argv[2]
parcalar, i = [], 1
while (kok / f'{ad}.{i}.html').exists():
    metin = (kok / f'{ad}.{i}.html').read_text(encoding='utf-8')
    parcalar.append(metin[:-1] if metin.endswith('\n') else metin)
    i += 1
(kok / f'{ad}.govde.html').write_text(''.join(parcalar), encoding='utf-8')
PY_BIRLESTIR
      printf 'BİRLEŞİK:%s\n' "$dosya" | "$DIL_BIN" çalıştır "$SITE_KOKU/kaynak/belge_uret.dil" > /dev/null || return 1
      rm -f "$CIKTI/md/$dosya".[0-9]*; rm -rf "$CIKTI/parca"
      return 0
    fi
    grep -q 'C024' "$CIKTI/hata.txt" || { cat "$CIKTI/hata.txt" >&2; return 1; }
    rm -f "$CIKTI/md/$dosya".[0-9]*
  done
  rm -rf "$CIKTI/parca"
  return 1
}

sayi=0
while IFS= read -r dosya; do
  # Çalışma bütçesine (güvenli profil) sığmayan çok büyük belge için kaynağa yönlendiren sayfa üretilir.
  if ! printf '%s\n' "$dosya" | "$DIL_BIN" çalıştır "$SITE_KOKU/kaynak/belge_uret.dil" > /dev/null 2>"$CIKTI/hata.txt"; then
    grep -q 'C024' "$CIKTI/hata.txt" || { cat "$CIKTI/hata.txt" >&2; exit 1; }
    if parcali_uret "$dosya"; then
      printf 'büyük belge (parçalı üretildi): %s\n' "$dosya"
    else
      printf 'KAYNAK:%s\n' "$dosya" | "$DIL_BIN" çalıştır "$SITE_KOKU/kaynak/belge_uret.dil" > /dev/null
      printf 'büyük belge (kaynağa yönlendirildi): %s\n' "$dosya"
    fi
  fi
  sayi=$((sayi + 1))
done < "$CIKTI/katalog.txt"
printf 'DİZİN\n' | "$DIL_BIN" çalıştır "$SITE_KOKU/kaynak/belge_uret.dil" > /dev/null

uretilen="$(find "$CIKTI/belge" -name '*.html' | wc -l | tr -d ' ')"
test "$uretilen" -eq $((sayi + 1))
rm -rf "$CIKTI/md" "$CIKTI/hata.txt"

# Yeni geliştirme notunun ham kaynağı da aynı yayında erişilebilir olsun.
# GitHub master'a henüz taşınmamış bir belge için ölü kaynak bağlantısı üretme.
for not_adi in zeedili-gelistirme-guncellemesi-20261017 surumler; do
  mkdir -p "$SITE_KOKU/static/updates"
  cp "$DEPO_KOKU/docs/$not_adi.md" "$SITE_KOKU/static/updates/$not_adi.md"
  python3 - "$CIKTI/belge/$not_adi.html" "$not_adi" <<'PY_SOURCE'
from pathlib import Path
import sys
p, ad = Path(sys.argv[1]), sys.argv[2]
s = p.read_text(encoding='utf-8')
eski = 'https://github.com/merbay-erp/zee/blob/master/docs/' + ad + '.md'
assert s.count(eski) == 1
p.write_text(s.replace(eski, '/assets/updates/' + ad + '.md'), encoding='utf-8')
PY_SOURCE
done

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
