# Kanıt özeti

Bu sayfa `dogfood/kanit-ozeti` Zee ürünü tarafından depo kayıt defterlerinden üretilir (K-164/ADR-072) ve elle düzenlenmez. Yenilemek için: `cd compiler && cargo run --locked -- çalıştır ../dogfood/kanit-ozeti/kaynak/ana.dil`. CI aynı komutu koşar ve bu dosya bayatsa kırılır.

## Semantic regresyon korpusu

Kaynak: `regression/v2.tsv`

| Faz | Vaka |
| --- | --- |
| checker | 4 |
| concurrency | 2 |
| hir | 2 |
| morphology | 2 |
| parser | 5 |
| runtime | 4 |
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
| - | 11 |

Toplam 22 vaka; `-` tanısız çalışma/eşzamanlılık vakasıdır. Tam 40 karakterlik `fixed_by` commit'i taşıyan vaka: 22/22.

## Güvenlik bulguları

Kaynak: `docs/guvenlik-bulgulari-v1.tsv`

| Önem | Kapalı | Kabul | Açık | Toplam |
| --- | --- | --- | --- | --- |
| kritik | 1 | 0 | 0 | 1 |
| yuksek | 8 | 0 | 0 | 8 |
| orta | 7 | 3 | 1 | 11 |
| dusuk | 2 | 1 | 0 | 3 |

Kapı: açık kritik/yüksek bulgu 0 → **GEÇTİ** (ADR-066 sürekli kapı koşulu).
Toplam bulgu: 23.

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
| spec/12-web-guvenlik-profili.md | 6 | 0 | 0 | 6 | 100 |
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
| spec/25-postgresql-veri-erisimi.md | 4 | 2 | 1 | 7 | 57 |
| spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md | 2 | 0 | 0 | 2 | 100 |
| spec/27-uyumluluk-ve-surumleme.md | 7 | 2 | 1 | 10 | 70 |
| **Toplam** | 154 | 13 | 2 | 169 | 91 |

## Dogfood ürünleri ve korpusu

Kaynak: `docs/dogfood-projeleri-v1.tsv + dogfood/korpus-v1.tsv`

| Ürün (durum) | Kök |
| --- | --- |
| catli-itwise-admin (active) | dogfood/catli-itwise-admin |
| kanit-ozeti (active) | dogfood/kanit-ozeti |
| kelime-avi (active) | dogfood/kelime-avi |

| Ürün | denetle | calistir | proje | basarili | basarisiz | Toplam |
| --- | --- | --- | --- | --- | --- | --- |
| catli-itwise-admin | 6 | 4 | 0 | 7 | 3 | 10 |
| kanit-ozeti | 12 | 9 | 1 | 11 | 11 | 22 |
| kelime-avi | 2 | 2 | 1 | 3 | 2 | 5 |

Toplam korpus vakası: 37.

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
| gecti | 1 |

Son koşu: `051ee9a0296eaefb3f25bf3f8da124ddd48c7d49` (2026-09-06, macos-aarch64, 1800 sn): derleyici %1, dillsp %7 RSS büyümesi → gecti.

## Compiler değişiklik ve core freeze beyanları

Kaynak: `docs/compiler-degisiklik-beyanlari-v1.tsv + docs/core-freeze-beyanlari-v1.tsv`

| Değişiklik sınıfı | Beyan |
| --- | --- |
| maintenance | 17 |
| semantic-bugfix | 3 |
| semantic-change | 11 |

| Freeze sınıfı | Beyan |
| --- | --- |
| correctness | 8 |
| dogfood-change | 5 |
| maintenance | 14 |

Toplam: 31 compiler değişiklik beyanı, 27 core freeze beyanı.
