# Zee – Rust – Go karşılaştırması (kanıt özeti aracı)

K-165/ADR-073'ün veri sayfası. Üç gerçekleme aynı dokuz kayıt defterinden
aynı [`docs/kanit-ozeti.md`](kanit-ozeti.md) sayfasını bayt bayt üretir
(`dogfood_karsilastirma_testi`); ölçüm `scripts/dogfood-karsilastirma.sh`
ile temiz ağaçta exact SHA'yla [`docs/dogfood-karsilastirma-v1.tsv`](dogfood-karsilastirma-v1.tsv)
tarihçesine yazılır.

<!-- ZEE-KARSILASTIRMA:BEGIN -->
Son ölçüm: `e4a583c29813` · 2026-09-06 · darwin-arm64 · rustc 1.93.1 (01f6ddf75 2026-02-11) · go go1.26.1 · 10 tur medyanı.

| Ölçü | Zee | Rust | Go |
|---|---:|---:|---:|
| Kaynak satırı (boş/yorum dışı, testler dahil) | 639 | 364 | 328 |
| Birim testi | 19 | 6 | 6 |
| Soğuk derleme süresi (ms; Zee yorumlayıcı: ürün derlemesi çalışma süresinde) | - | 3235 | 1902 |
| Çalışma süresi (ms; Zee: derle+yürüt) | 90 | 3 | 3 |
| Tepe RSS (KiB) | 12000 | 2240 | 4608 |
| Korpusa giren sürtünme vakası | 21 | — | — |
<!-- ZEE-KARSILASTIRMA:END -->

## Okuma kuralı

- Kaynak satırı ve test sayısı testte yeniden hesaplanır; süre ve RSS
  makineye bağlıdır, kapı değildir.
- Zee sütunundaki çalışma süresi derleme + yürütmedir (yorumlayıcı); Rust/Go
  önceden derlenmiş ikilidir. Zee için derleme süresi "-"dir: ürünün derlemesi
  çalışma süresine dahildir, `dil` ikilisinin cargo derlemesi ürünün ölçüsü
  değildir. Rust/Go derlemesi soğuktur (hedef klasörü/GOCACHE taze).
  Bu fark dilin bilinen bedelidir, gizlenmez.
- Sürtünme vakası yalnız Zee için vardır: K-164'te 16 sürtünme kaydedildi,
  11'i korpusa ret/çözüm çifti olarak girdi. Rust/Go gerçeklemeleri bu iş için
  sürtünme kaydı üretmedi.

## Zee'nin kaybettiği yerler (backlog girdisi)

- Kaynak satırı: Zee ~1,8× uzun (tablo). Ara ad zorunlulukları, sayaç
  döngüleri ve yerinde sayım kalıpları satır ekliyor; 19 birim testi de
  sayıma dahildir (Rust/Go 6'şar).
- Çalışma süresi ve tepe RSS: yorumlayıcı derle+yürüt, derlenmiş ikilinin
  ~30 katı; RSS 2,6–5×. V1 sözü değildir.
- Sıra ile öğe erişimi yok (F010) ve `\t` kaçışı yok (F011): TSV işleme
  Rust/Go'dan daha dolaylı yazılıyor.
- Morfoloji kökü seçimi (F006/F007/F008/F013) ad seçimini kısıtlıyor;
  `zee-tr-2` adayı.
- Çağrı argümanında birleştirme (F003) ve postfix zinciri (F014) ara ad
  istiyor; satır sayısını artırıyor.

## Komut

```bash
bash scripts/dogfood-karsilastirma.sh 10
```
