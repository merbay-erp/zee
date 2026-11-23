# Spec maddesi ↔ test kanıtı drift raporu

<!-- `cd compiler && cargo run --locked --bin spec_drift -- --rapor-yaz` üretir. Elle değiştirme. -->

Bu rapor K-171/ADR-065 kapısının güncel görünümüdür. Madde; `spec/` altında ZORUNLU/ZORUNDA/YASAK/TANIMLI/AÇIK işaretçisi taşıyan paragraf, liste öğesi ya da başlıktır. Kimlik dosya adı + sadeleştirilmiş metnin parmak izidir; metin değişince kimlik değişir ve kanıt yeniden incelenir.

| Bölüm | Madde | Kanıtlı | Kısmi | Açık |
|---|---:|---:|---:|---:|
| `spec/01-sozcukleme.md` | 6 | 6 | 0 | 0 |
| `spec/02-dizim.md` | 7 | 7 | 0 | 0 |
| `spec/03-adlar-ve-kapsam.md` | 6 | 6 | 0 | 0 |
| `spec/04-turler.md` | 13 | 13 | 0 | 0 |
| `spec/05-degerlendirme.md` | 9 | 9 | 0 | 0 |
| `spec/06-hata-modeli.md` | 7 | 7 | 0 | 0 |
| `spec/07-birimler.md` | 10 | 10 | 0 | 0 |
| `spec/08-kalici-dosya.md` | 7 | 6 | 1 | 0 |
| `spec/09-son-tarih-ve-iptal.md` | 1 | 1 | 0 | 0 |
| `spec/10-disari-acik-islem-sozlesmesi.md` | 4 | 4 | 0 | 0 |
| `spec/11-uygulama-eylemleri-ve-web-adaptoru.md` | 17 | 17 | 0 | 0 |
| `spec/12-web-guvenlik-profili.md` | 43 | 40 | 0 | 3 |
| `spec/13-surumlu-morfoloji-profili.md` | 9 | 8 | 0 | 1 |
| `spec/14-yapilandirilmis-eszamanlilik.md` | 2 | 2 | 0 | 0 |
| `spec/15-yapilandirilmis-hata-degeri.md` | 3 | 3 | 0 | 0 |
| `spec/16-keyfi-hassasiyetli-ondalik.md` | 2 | 2 | 0 | 0 |
| `spec/17-deger-semantigi-ve-gezme.md` | 5 | 5 | 0 | 0 |
| `spec/18-paket-yayini.md` | 14 | 13 | 1 | 0 |
| `spec/19-registry-metadata-guveni.md` | 10 | 10 | 0 | 0 |
| `spec/20-ifade-grameri.md` | 7 | 7 | 0 | 0 |
| `spec/21-deterministik-io-izi.md` | 5 | 5 | 0 | 0 |
| `spec/22-deterministik-io-profili.md` | 5 | 4 | 1 | 0 |
| `spec/23-yetkinlik-ve-outbound-guvenligi.md` | 36 | 35 | 0 | 1 |
| `spec/24-kaynak-guvenlik-profili.md` | 9 | 9 | 0 | 0 |
| `spec/25-postgresql-veri-erisimi.md` | 10 | 8 | 0 | 2 |
| `spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md` | 6 | 6 | 0 | 0 |
| `spec/27-uyumluluk-ve-surumleme.md` | 13 | 11 | 1 | 1 |
| `spec/28-gorunum-blogu.md` | 2 | 2 | 0 | 0 |
| `spec/29-test-kapsami.md` | 5 | 5 | 0 | 0 |
| `spec/30-editor-analizi-ve-lint.md` | 13 | 12 | 0 | 1 |
| **Toplam** | **286** | **273** | **4** | **9** |

## Kısmi kanıtlı maddeler

- `spec/08-kalici-dosya.md#532a9e61aa1fe70f` — - Dayanıklılık/kilit primitive'leri bulunmayan platform işlemi reddetmek… — Primitive'siz platform reddi yalnız desteklenen üç platformda derlenen koda bağlı; böyle bir platform CI matrisinde yok.
- `spec/18-paket-yayini.md#2af407978947e1d4` — Üretici dört dosyayı yazmadan önce bütün kümeyi tüketici doğrulayıcıyla … — Yazmadan önce tam doğrulama ve yarım küme reddi kanıtlı; süreç çökmesi ortasında yarım küme senaryosu enjekte edilmiyor (söz de verilmiyor).
- `spec/22-deterministik-io-profili.md#d68db0760744fd45` — Profilin bu gözlemlerinden birini kıran değişiklik yeni profil kimliği, … — Snapshot gözlem değişikliğini yakalar; yeni profil kimliği/RFC zorunluluğu süreç kuralıdır, makinece yalnız snapshot farkıyla görünür.
- `spec/27-uyumluluk-ve-surumleme.md#15d606bdb76e4dc1` — - Kaldırılan biçim sessizce kabul edilmez: derleyici tanı üretir ve öner… — Henüz kaldırılmış bir kullanıcı yüzeyi yok; tanı+öneri kanıtı ilk DEP kaldırmasında regresyon korpusuna eklenecek.

## Açık maddeler

- `spec/12-web-guvenlik-profili.md#f0003f30d3695cf6` — AÇIK: Parola ve TOTP doğrulaması aşımda rotaya `yanlış` döndürüp rotayı … — K-368 kapsamı dışı: aşımı Zee koduna ayrı değerle göstermek ya da rotayı kesmek GirdiCikti bool sözleşmesini ve IO izi şemasını değiştirir; ZM-F021 hesap kilidi sayacı etkisi bu madde altında izlenir.
- `spec/12-web-guvenlik-profili.md#5d930da7687a2f1e` — AÇIK: Windows'ta paylaşılan dinleyici devri yoktur; `--web-isci-sayisi` … — Windows paylaşılan dinleyici devri, dengeli dağılım, ağ dosya sistemi ve çok hostlu birincillik açık (K-340 `zee.ayar` alanını, K-341 tek makinede elle kurulumda tek birincili bu maddeden çıkardı); Windows'taki çıkış 2 reddi web_cok_surec_testi::isci_grubu_gecersiz_birlesimleri_ve_platform_siniri_reddedilir cfg(windows) vakasıyla Windows CI'da koşar.
- `spec/12-web-guvenlik-profili.md#d0c68c44c7c98c4b` — AÇIK: `Vary`, ETag/koşullu istek, `stale-while-revalidate` ve önbellek t… — K-335 kapsamı dışı; Vary/ETag/purge ve sorgu anahtarı ön kapının sorumluluğundadır.
- `spec/13-surumlu-morfoloji-profili.md#8a414db8b5ca5b82` — AÇIK: `zee-tr-1` `â`, `î`, `û` harflerini ünlü saymaz. Bu harfleri taşıy… — K-367: düzeltme uyumluluk kaydını değiştirir (53.248 üretimin 10.306'sı, 2.865 çözüm kümesi); zee-tr-1 içinde yapılamaz, zee-tr-2 adayı (RFC + yeni kayıt + kilit geçişi + edition). Çözücü doğru yazımı kabul eder.
- `spec/23-yetkinlik-ve-outbound-guvenligi.md#15b3c0c87713d1ae` — AÇIK: Gerçek systemd servis kurulumu ile canlı kabul, çalışan süreçte ro… — K-339 kapsamı dışı; canlı systemd servis kabulü ve rotasyon ürün/işletim tarafında.
- `spec/25-postgresql-veri-erisimi.md#c395a90f9132ccfa` — AÇIK: PostgreSQL okuma ve değişikliği dış `içinde` ya da web isteğinin 3… — K-371 ölçümü yerel PostgreSQL 16 ile: 1 sn son tarihte `pg_sleep(4)` 4,1 sn, web `pg_sleep(33)` 504'ü 33. saniyede; iptalin transaction/C027 etkisi tasarım ister, RFC-0042 taslağı ve GB-149.
- `spec/25-postgresql-veri-erisimi.md#f94f82cb0245cf0a` — Managed-provider sertifika rotasyonu, birden çok worker'ın toplam bütçe … — K-169/K-173 release tatbikatı ve çok-worker toplam bütçe provası bekliyor; async driver, PostgreSQL dışı DB ve ORM vaat edilmiyor.
- `spec/27-uyumluluk-ve-surumleme.md#285d87941dc0c3ba` — 1.0 sonrasında dondurulmuş yüzeylerde kırıcı değişiklik yalnız yeni ana … — Edition alanının `proje.dil` sözdizimi ayrı RFC ister; 1.0 öncesi bağlayıcı yalnız RFC-0028 çerçevesidir.
- `spec/30-editor-analizi-ve-lint.md#38480ce75d9537ab` — Kapsam ve performans (AÇIK) — K-287: artımlı HIR ve genel gerçek ürün gecikmesi açık; tarihsel S3 66e4149e53451111d153f47283c219079a8d1cab kaynağındaki kabul korunur. S4 4b7dd76ad5e10bdf1b16bded82b3ea20ad55a9e2 kaynağında ayrı 1800sn RSS ve lint açık/kapalı 1/41 belge stdio gözlemi vardır; 41 belge lint açık p95 223,698ms ile 200ms hedefi açıktır, UI/ürün SLA'sı değildir.

## Sürüklenme

Yok: her normatif madde kayıtlı, her kayıt var olan bir maddeye ve var olan test işlevine bağlı.

Sürüklenme otomatik bir semantik hüküm değildir: kayıtsız madde kanıt ister, bayat kayıt ya silinir ya yeni kimliğe taşınır; seçicinin maddeyi gerçekten kanıtladığı kod incelemesinde değerlendirilir.
