# Kanıt özeti

Bu sayfa `dogfood/kanit-ozeti` Zee ürünü tarafından depo kayıt defterlerinden üretilir (K-164/ADR-072) ve elle düzenlenmez. Yenilemek için: `cd compiler && cargo run --locked -- çalıştır ../dogfood/kanit-ozeti/kaynak/ana.dil`. CI aynı komutu koşar ve bu dosya bayatsa kırılır.

## Semantic regresyon korpusu

Kaynak: `regression/v2.tsv`

| Faz | Vaka |
| --- | --- |
| checker | 6 |
| cli | 9 |
| concurrency | 2 |
| hir | 5 |
| lsp | 2 |
| morphology | 2 |
| parser | 5 |
| runtime | 17 |
| security | 30 |

| Tanı | Vaka |
| --- | --- |
| A001 | 2 |
| A002 | 1 |
| C003 | 1 |
| S004 | 2 |
| S007 | 1 |
| S030 | 1 |
| T017 | 1 |
| T036 | 1 |
| T054 | 2 |
| - | 66 |

Toplam 78 vaka; `-` tanısız çalışma/eşzamanlılık vakasıdır. Tam 40 karakterlik `fixed_by` commit'i taşıyan vaka: 78/78.

## Güvenlik bulguları

Kaynak: `docs/guvenlik-bulgulari-v1.tsv`

| Önem | Kapalı | Kabul | Açık | Toplam |
| --- | --- | --- | --- | --- |
| kritik | 1 | 0 | 0 | 1 |
| yuksek | 19 | 0 | 0 | 19 |
| orta | 52 | 16 | 2 | 70 |
| dusuk | 46 | 30 | 1 | 77 |

Kapı: açık kritik/yüksek bulgu 0 → **GEÇTİ** (ADR-066 sürekli kapı koşulu).
Toplam bulgu: 167.

## Spec maddeleri

Kaynak: `docs/spec-madde-kaniti-v1.tsv`

| Bölüm | Kanıtlı | Kısmi | Açık | Toplam | Kanıt % |
| --- | --- | --- | --- | --- | --- |
| spec/01-sozcukleme.md | 8 | 0 | 0 | 8 | 100 |
| spec/02-dizim.md | 7 | 0 | 0 | 7 | 100 |
| spec/03-adlar-ve-kapsam.md | 6 | 0 | 0 | 6 | 100 |
| spec/04-turler.md | 15 | 0 | 0 | 15 | 100 |
| spec/05-degerlendirme.md | 9 | 0 | 0 | 9 | 100 |
| spec/06-hata-modeli.md | 7 | 0 | 0 | 7 | 100 |
| spec/07-birimler.md | 13 | 1 | 1 | 15 | 86 |
| spec/08-kalici-dosya.md | 7 | 1 | 0 | 8 | 87 |
| spec/09-son-tarih-ve-iptal.md | 1 | 0 | 0 | 1 | 100 |
| spec/10-disari-acik-islem-sozlesmesi.md | 4 | 0 | 0 | 4 | 100 |
| spec/11-uygulama-eylemleri-ve-web-adaptoru.md | 26 | 0 | 0 | 26 | 100 |
| spec/12-web-guvenlik-profili.md | 70 | 0 | 6 | 76 | 92 |
| spec/13-surumlu-morfoloji-profili.md | 8 | 0 | 1 | 9 | 88 |
| spec/14-yapilandirilmis-eszamanlilik.md | 2 | 0 | 0 | 2 | 100 |
| spec/15-yapilandirilmis-hata-degeri.md | 3 | 0 | 0 | 3 | 100 |
| spec/16-keyfi-hassasiyetli-ondalik.md | 2 | 0 | 0 | 2 | 100 |
| spec/17-deger-semantigi-ve-gezme.md | 11 | 0 | 0 | 11 | 100 |
| spec/18-paket-yayini.md | 13 | 1 | 0 | 14 | 92 |
| spec/19-registry-metadata-guveni.md | 10 | 0 | 0 | 10 | 100 |
| spec/20-ifade-grameri.md | 9 | 0 | 0 | 9 | 100 |
| spec/21-deterministik-io-izi.md | 6 | 0 | 0 | 6 | 100 |
| spec/22-deterministik-io-profili.md | 4 | 1 | 0 | 5 | 80 |
| spec/23-yetkinlik-ve-outbound-guvenligi.md | 49 | 1 | 2 | 52 | 94 |
| spec/24-kaynak-guvenlik-profili.md | 10 | 0 | 0 | 10 | 100 |
| spec/25-postgresql-veri-erisimi.md | 11 | 0 | 1 | 12 | 91 |
| spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md | 18 | 0 | 1 | 19 | 94 |
| spec/27-uyumluluk-ve-surumleme.md | 11 | 1 | 1 | 13 | 84 |
| spec/28-gorunum-blogu.md | 2 | 0 | 0 | 2 | 100 |
| spec/29-test-kapsami.md | 5 | 0 | 0 | 5 | 100 |
| spec/30-editor-analizi-ve-lint.md | 12 | 0 | 1 | 13 | 92 |
| spec/31-dns-ve-posta-kimlik-dogrulama.md | 19 | 0 | 1 | 20 | 95 |
| **Toplam** | 378 | 6 | 15 | 399 | 94 |

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
| urun-g (active) | dogfood/urun-g |
| urun-h (active) | dogfood/urun-h |

| Ürün | denetle | calistir | proje | basarili | basarisiz | Toplam |
| --- | --- | --- | --- | --- | --- | --- |
| kanit-ozeti | 12 | 10 | 1 | 12 | 11 | 23 |
| kelime-avi | 2 | 2 | 1 | 3 | 2 | 5 |
| legacy-panel | 6 | 4 | 0 | 7 | 3 | 10 |
| urun-e | 1 | 0 | 0 | 1 | 0 | 1 |
| urun-g | 1 | 0 | 0 | 1 | 0 | 1 |
| urun-h | 13 | 4 | 0 | 16 | 1 | 17 |
| zeedili-com | 0 | 0 | 1 | 1 | 0 | 1 |
| zee-cekirdek | 16 | 1 | 5 | 22 | 0 | 22 |
| zee-erp | 2 | 0 | 0 | 2 | 0 | 2 |

Toplam korpus vakası: 82.

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
| gecti | 12 |

Son koşu: `41135379556c24d147a6115645a09fb5909019b8` (2026-11-24, macos-aarch64, 1800 sn): derleyici %0, dillsp %0 RSS büyümesi → gecti.

## Compiler değişiklik ve core freeze beyanları

Kaynak: `docs/compiler-degisiklik-beyanlari-v1.tsv + docs/core-freeze-beyanlari-v1.tsv`

| Değişiklik sınıfı | Beyan |
| --- | --- |
| maintenance | 109 |
| semantic-bugfix | 42 |
| semantic-change | 68 |

| Freeze sınıfı | Beyan |
| --- | --- |
| bugfix | 2 |
| correctness | 42 |
| dogfood-change | 46 |
| maintenance | 106 |
| security | 19 |

Toplam: 219 compiler değişiklik beyanı, 215 core freeze beyanı.
