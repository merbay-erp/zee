# Kanıt özeti

Bu sayfa `dogfood/kanit-ozeti` Zee ürünü tarafından depo kayıt defterlerinden üretilir (K-164/ADR-072) ve elle düzenlenmez. Yenilemek için: `cd compiler && cargo run --locked -- çalıştır ../dogfood/kanit-ozeti/kaynak/ana.dil`. CI aynı komutu koşar ve bu dosya bayatsa kırılır.

## Semantic regresyon korpusu

Kaynak: `regression/v2.tsv`

| Faz | Vaka |
| --- | --- |
| checker | 4 |
| concurrency | 2 |
| hir | 4 |
| morphology | 2 |
| parser | 5 |
| runtime | 6 |
| security | 3 |

| Tanı | Vaka |
| --- | --- |
| A001 | 2 |
| A002 | 1 |
| C003 | 1 |
| S004 | 2 |
| S007 | 1 |
| S030 | 1 |
| T017 | 1 |
| T054 | 2 |
| - | 15 |

Toplam 26 vaka; `-` tanısız çalışma/eşzamanlılık vakasıdır. Tam 40 karakterlik `fixed_by` commit'i taşıyan vaka: 26/26.

## Güvenlik bulguları

Kaynak: `docs/guvenlik-bulgulari-v1.tsv`

| Önem | Kapalı | Kabul | Açık | Toplam |
| --- | --- | --- | --- | --- |
| kritik | 1 | 0 | 0 | 1 |
| yuksek | 12 | 0 | 0 | 12 |
| orta | 16 | 5 | 2 | 23 |
| dusuk | 13 | 2 | 0 | 15 |

Kapı: açık kritik/yüksek bulgu 0 → **GEÇTİ** (ADR-066 sürekli kapı koşulu).
Toplam bulgu: 51.

## Spec maddeleri

Kaynak: `docs/spec-madde-kaniti-v1.tsv`

| Bölüm | Kanıtlı | Kısmi | Açık | Toplam | Kanıt % |
| --- | --- | --- | --- | --- | --- |
| spec/01-sozcukleme.md | 5 | 1 | 0 | 6 | 83 |
| spec/02-dizim.md | 6 | 1 | 0 | 7 | 85 |
| spec/03-adlar-ve-kapsam.md | 6 | 0 | 0 | 6 | 100 |
| spec/04-turler.md | 12 | 1 | 0 | 13 | 92 |
| spec/05-degerlendirme.md | 8 | 1 | 0 | 9 | 88 |
| spec/06-hata-modeli.md | 7 | 0 | 0 | 7 | 100 |
| spec/07-birimler.md | 7 | 0 | 0 | 7 | 100 |
| spec/08-kalici-dosya.md | 4 | 1 | 0 | 5 | 80 |
| spec/09-son-tarih-ve-iptal.md | 1 | 0 | 0 | 1 | 100 |
| spec/10-disari-acik-islem-sozlesmesi.md | 3 | 1 | 0 | 4 | 75 |
| spec/11-uygulama-eylemleri-ve-web-adaptoru.md | 2 | 0 | 0 | 2 | 100 |
| spec/12-web-guvenlik-profili.md | 14 | 0 | 0 | 14 | 100 |
| spec/13-surumlu-morfoloji-profili.md | 8 | 0 | 0 | 8 | 100 |
| spec/14-yapilandirilmis-eszamanlilik.md | 2 | 0 | 0 | 2 | 100 |
| spec/15-yapilandirilmis-hata-degeri.md | 3 | 0 | 0 | 3 | 100 |
| spec/16-keyfi-hassasiyetli-ondalik.md | 2 | 0 | 0 | 2 | 100 |
| spec/17-deger-semantigi-ve-gezme.md | 5 | 0 | 0 | 5 | 100 |
| spec/18-paket-yayini.md | 12 | 1 | 0 | 13 | 92 |
| spec/19-registry-metadata-guveni.md | 10 | 0 | 0 | 10 | 100 |
| spec/20-ifade-grameri.md | 7 | 0 | 0 | 7 | 100 |
| spec/21-deterministik-io-izi.md | 5 | 0 | 0 | 5 | 100 |
| spec/22-deterministik-io-profili.md | 4 | 1 | 0 | 5 | 80 |
| spec/23-yetkinlik-ve-outbound-guvenligi.md | 10 | 0 | 0 | 10 | 100 |
| spec/24-kaynak-guvenlik-profili.md | 6 | 1 | 0 | 7 | 85 |
| spec/25-postgresql-veri-erisimi.md | 5 | 2 | 1 | 8 | 62 |
| spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md | 2 | 0 | 0 | 2 | 100 |
| spec/27-uyumluluk-ve-surumleme.md | 7 | 2 | 1 | 10 | 70 |
| spec/28-gorunum-blogu.md | 2 | 0 | 0 | 2 | 100 |
| **Toplam** | 165 | 13 | 2 | 180 | 91 |

## Dogfood ürünleri ve korpusu

Kaynak: `docs/dogfood-projeleri-v1.tsv + dogfood/korpus-v1.tsv`

| Ürün (durum) | Kök |
| --- | --- |
| legacy-panel (active) | dogfood/legacy-panel |
| kanit-ozeti (active) | dogfood/kanit-ozeti |
| kelime-avi (active) | dogfood/kelime-avi |
| zee-erp (active) | dogfood/zee-erp |
| zee-cekirdek (active) | dogfood/zee-cekirdek |
| urun-e (active) | dogfood/urun-e |
| zeedili-com (active) | dogfood/zeedili-com |

| Ürün | denetle | calistir | proje | basarili | basarisiz | Toplam |
| --- | --- | --- | --- | --- | --- | --- |
| kanit-ozeti | 12 | 9 | 1 | 11 | 11 | 22 |
| kelime-avi | 2 | 2 | 1 | 3 | 2 | 5 |
| legacy-panel | 6 | 4 | 0 | 7 | 3 | 10 |
| urun-e | 1 | 0 | 0 | 1 | 0 | 1 |
| zeedili-com | 0 | 0 | 1 | 1 | 0 | 1 |
| zee-cekirdek | 2 | 1 | 0 | 3 | 0 | 3 |
| zee-erp | 2 | 0 | 0 | 2 | 0 | 2 |

Toplam korpus vakası: 44.

## Deprecation kayıtları

Kaynak: `docs/deprecation-kayitlari-v1.tsv`

| Durum | Kayıt |
| --- | --- |
| kaldirildi | 4 |

| Yüzey / sınıf | Kayıt |
| --- | --- |
| ic / kaldirma | 1 |
| tani / kaldirma | 3 |

Toplam kayıt: 4.

## Uzun soak tarihçesi

Kaynak: `docs/soak-gecmisi-v1.tsv`

| Sonuç | Koşu |
| --- | --- |
| gecti | 3 |

Son koşu: `4e351d34de861425c8a35b6ad1ddda6cd3e42707` (2026-10-13, macos-aarch64, 1800 sn): derleyici %0, dillsp %0 RSS büyümesi → gecti.

## Compiler değişiklik ve core freeze beyanları

Kaynak: `docs/compiler-degisiklik-beyanlari-v1.tsv + docs/core-freeze-beyanlari-v1.tsv`

| Değişiklik sınıfı | Beyan |
| --- | --- |
| maintenance | 38 |
| semantic-bugfix | 3 |
| semantic-change | 30 |

| Freeze sınıfı | Beyan |
| --- | --- |
| correctness | 11 |
| dogfood-change | 18 |
| maintenance | 35 |
| security | 3 |

Toplam: 71 compiler değişiklik beyanı, 67 core freeze beyanı.
