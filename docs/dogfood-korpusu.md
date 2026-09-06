# Dogfood korpusu rehberi

K-172/ADR-067'nin işletim sözleşmesi. Manifest
[`dogfood/korpus-v1.tsv`](../dogfood/korpus-v1.tsv), test
`compiler/tests/dogfood_korpusu_testi.rs`.

## Satır sözleşmesi

`vaka	urun	kaynak_is	kip	beklenti	tani	cikti	dosya	not`

- `vaka` küçük kebab-case ve tekil; `urun` etkin dogfood ürün slug'ı.
- `kaynak_is`: `K-NNN` ya da `K-NNN/FNNN`; K-işi ve dilim günlükte gerçek.
- `kip`: `denetle` (ürün politikasıyla derleme+yetkinlik), `calistir`
  (hermetik IO ile yürütme) ya da `proje` (K-164/ADR-072: depo içi ürünün
  giriş dosyası birimleriyle diskten yüklenir, ürün politikasıyla derlenir ve
  bütün `test` blokları koşar; giriş klasörü altındaki her `.dil` o vakaya
  aittir, beklenti `basarili`, çıktı `-`).
- `beklenti`: `basarili` (tanı `-`; `calistir` için `|` ayraçlı exact çıktı)
  ya da `basarisiz` (dört karakterli tanı kodu; çıktı `-`).
- `dosya`: etkin ürün kökü altında `.dil`; kanonik biçimde ve sürtünmeyi
  anlatan `# ` yorumuyla başlar. `not` en az 20 karakter. Sözcükleme
  düzeyinde reddedilen sürtünme (örn. S040 kaçış) biçimlenemez; yalnız
  beklenen tanısı aynı kodsa kabul edilir.

## Ne girer, ne girmez

- Girer: gerçek ürün sürtünmesinin reddedilen biçimi, ürünün mevcut dilde
  uyguladığı çözüm, dogfood dilimi gereksinim aynası.
- Girmez: dil tasarımı örneği (golden), düzeltilmiş compiler bug'ı
  (`regression/`), dış ürün kaynağının kopyası. Depo içi ürünün kendi
  kaynağı tek `proje` vakasıyla kapsanır; sürtünme çiftleri `korpus/`
  altındadır.

## Komut

```bash
cd compiler
cargo test --locked --test dogfood_korpusu_testi
```

`dogfood/` altına manifest dışı `.dil` eklemek kapıyı kırar.
