# Zee – Rust – Go karşılaştırması (kanıt özeti aracı)

K-165/ADR-073'ün veri sayfası. Üç gerçekleme aynı dokuz kayıt defterinden
aynı [`docs/kanit-ozeti.md`](kanit-ozeti.md) sayfasını bayt bayt üretir
(`dogfood_karsilastirma_testi`); ölçüm `scripts/dogfood-karsilastirma.sh`
ile temiz ağaçta exact SHA'yla [`docs/dogfood-karsilastirma-v1.tsv`](dogfood-karsilastirma-v1.tsv)
tarihçesine yazılır.

<!-- ZEE-KARSILASTIRMA:BEGIN -->
Son ölçüm: `bc651d823a4b` · 2026-09-09 · linux-aarch64 · rustc 1.93.1 (01f6ddf75 2026-02-11) · go go1.26.1 · 10 tur medyanı.

| Ölçü | Zee | Rust | Go |
|---|---:|---:|---:|
| Kaynak satırı (boş/yorum dışı, testler dahil) | 639 | 364 | 328 |
| Birim testi | 19 | 6 | 6 |
| Soğuk derleme süresi (ms; Zee yorumlayıcı: ürün derlemesi çalışma süresinde) | - | 3522 | 3519 |
| Çalışma süresi (ms; Zee: derle+yürüt) | 114 | 5 | 7 |
| Tepe RSS (KiB) | 9436 | 9308 | 9436 |
| Korpusa giren sürtünme vakası | 21 | — | — |
<!-- ZEE-KARSILASTIRMA:END -->

## Platform satırları

Tarihçe aynı üç gerçeklemeyi farklı platformlarda taşır: `e4a583c` macOS arm64
(Zee 90 ms, Rust 3 ms, Go 3 ms; RSS 12000/2240/4608 KiB) ve `bc651d8` Linux
aarch64 (Zee 114 ms, Rust 5 ms, Go 7 ms; `rust:1.93.1-bookworm` konteyneri,
K-179 ağacı). Oran iki platformda da aynı sınıftadır: yorumlayıcı derle+yürüt,
derlenmiş ikilinin ~20–30 katı. Linux `ru_maxrss` süreç başlangıç tabanını
(~9 MiB) içerdiğinden üç sütun birbirine yakın çıkar; RSS farkı macOS
satırında okunur, Linux satırı yalnız süre için anlamlıdır.

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
