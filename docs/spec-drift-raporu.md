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
| `spec/07-birimler.md` | 8 | 8 | 0 | 0 |
| `spec/08-kalici-dosya.md` | 5 | 4 | 1 | 0 |
| `spec/09-son-tarih-ve-iptal.md` | 1 | 1 | 0 | 0 |
| `spec/10-disari-acik-islem-sozlesmesi.md` | 4 | 4 | 0 | 0 |
| `spec/11-uygulama-eylemleri-ve-web-adaptoru.md` | 2 | 2 | 0 | 0 |
| `spec/12-web-guvenlik-profili.md` | 17 | 17 | 0 | 0 |
| `spec/13-surumlu-morfoloji-profili.md` | 8 | 8 | 0 | 0 |
| `spec/14-yapilandirilmis-eszamanlilik.md` | 2 | 2 | 0 | 0 |
| `spec/15-yapilandirilmis-hata-degeri.md` | 3 | 3 | 0 | 0 |
| `spec/16-keyfi-hassasiyetli-ondalik.md` | 2 | 2 | 0 | 0 |
| `spec/17-deger-semantigi-ve-gezme.md` | 5 | 5 | 0 | 0 |
| `spec/18-paket-yayini.md` | 13 | 12 | 1 | 0 |
| `spec/19-registry-metadata-guveni.md` | 10 | 10 | 0 | 0 |
| `spec/20-ifade-grameri.md` | 7 | 7 | 0 | 0 |
| `spec/21-deterministik-io-izi.md` | 5 | 5 | 0 | 0 |
| `spec/22-deterministik-io-profili.md` | 5 | 4 | 1 | 0 |
| `spec/23-yetkinlik-ve-outbound-guvenligi.md` | 21 | 21 | 0 | 0 |
| `spec/24-kaynak-guvenlik-profili.md` | 7 | 7 | 0 | 0 |
| `spec/25-postgresql-veri-erisimi.md` | 9 | 7 | 1 | 1 |
| `spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md` | 2 | 2 | 0 | 0 |
| `spec/27-uyumluluk-ve-surumleme.md` | 13 | 11 | 1 | 1 |
| `spec/28-gorunum-blogu.md` | 2 | 2 | 0 | 0 |
| **Toplam** | **199** | **192** | **5** | **2** |

## Kısmi kanıtlı maddeler

- `spec/08-kalici-dosya.md#532a9e61aa1fe70f` — - Dayanıklılık/kilit primitive'leri bulunmayan platform işlemi reddetmek… — Primitive'siz platform reddi yalnız desteklenen üç platformda derlenen koda bağlı; böyle bir platform CI matrisinde yok.
- `spec/18-paket-yayini.md#2af407978947e1d4` — Üretici dört dosyayı yazmadan önce bütün kümeyi tüketici doğrulayıcıyla … — Yazmadan önce tam doğrulama ve yarım küme reddi kanıtlı; süreç çökmesi ortasında yarım küme senaryosu enjekte edilmiyor (söz de verilmiyor).
- `spec/22-deterministik-io-profili.md#d68db0760744fd45` — Profilin bu gözlemlerinden birini kıran değişiklik yeni profil kimliği, … — Snapshot gözlem değişikliğini yakalar; yeni profil kimliği/RFC zorunluluğu süreç kuralıdır, makinece yalnız snapshot farkıyla görünür.
- `spec/25-postgresql-veri-erisimi.md#61d77038cc9e2321` — `sslmode=require`, `<BAĞLANTI_DEĞİŞKENİ>_TLS_CA_PEM` ortam değişkeninde … — K-222 izole gerçek PostgreSQL kapısı CA/hostname reddi ve TLS 1.2 olumlusunu dört sıfır-atlamalı vakada zorlar; bağımsız TLS 1.1 sunucusu downgrade reddi henüz sınanmadığı için protokol tabanı maddesi kısmi kalır.
- `spec/27-uyumluluk-ve-surumleme.md#15d606bdb76e4dc1` — - Kaldırılan biçim sessizce kabul edilmez: derleyici tanı üretir ve öner… — Henüz kaldırılmış bir kullanıcı yüzeyi yok; tanı+öneri kanıtı ilk DEP kaldırmasında regresyon korpusuna eklenecek.

## Açık maddeler

- `spec/25-postgresql-veri-erisimi.md#f94f82cb0245cf0a` — Managed-provider sertifika rotasyonu, birden çok worker'ın toplam bütçe … — K-169/K-173 release tatbikatı ve çok-worker toplam bütçe provası bekliyor; async driver, PostgreSQL dışı DB ve ORM vaat edilmiyor.
- `spec/27-uyumluluk-ve-surumleme.md#285d87941dc0c3ba` — 1.0 sonrasında dondurulmuş yüzeylerde kırıcı değişiklik yalnız yeni ana … — Edition alanının `proje.dil` sözdizimi ayrı RFC ister; 1.0 öncesi bağlayıcı yalnız RFC-0028 çerçevesidir.

## Sürüklenme

Yok: her normatif madde kayıtlı, her kayıt var olan bir maddeye ve var olan test işlevine bağlı.

Sürüklenme otomatik bir semantik hüküm değildir: kayıtsız madde kanıt ister, bayat kayıt ya silinir ya yeni kimliğe taşınır; seçicinin maddeyi gerçekten kanıtladığı kod incelemesinde değerlendirilir.
