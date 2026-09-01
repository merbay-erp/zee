# zee dil spesifikasyonu (normatif çekirdek)

Bu klasör geçerli dilin **normatif** tanımıdır (master plan bölüm 23,
ADR-010): bir cümlenin bugün geçerli olup olmadığı ve ne anlama geldiği
konusunda ayrıntılı son söz buradadır. RFC değişikliğin gerekçesi/yetkisidir;
spec ve conformance testi aynı değişiklikte güncellenmeden yürürlükteki anlamı
değiştirmez. Golden korpus ile regression testleri bu tanımın yürütülebilir
kanıtıdır — spec ile test çelişirse sürüm engellenir.

## Sözleşme dili

- **ZORUNLU** — uyulmazsa program geçersizdir; derleyici tanı üretir.
- **YASAK** — dilde yeri yoktur; derleyici tanı üretir.
- **TANIMLI** — davranış budur, başka gerçekleme de aynı davranışı vermek
  zorundadır (determinizm sözü, RFC-0001).
- **AÇIK** — henüz karara bağlanmadı; bağlandığında RFC + sürüm notu ister.

Kırıcı değişiklik sessizce yapılamaz: davranış değişikliği RFC'den geçer ve
[docs/surumler.md](../docs/surumler.md)'de duyurulur.

## Bölümler

| Bölüm | Kapsam | Normatif kaynaklar |
|---|---|---|
| [01 — Sözcükleme](01-sozcukleme.md) | alfabe, tokenlar, kaçışlar, sayılar | RFC-0002, RFC-0013 |
| [02 — Dizim](02-dizim.md) | satır/blok yapısı, yüklem-sonlu dağıtım | RFC-0003, RFC-0006 |
| [03 — Adlar ve kapsam](03-adlar-ve-kapsam.md) | morfolojik çözüm, blok kapsamı | RFC-0004, RFC-0018, K-089 |
| [04 — Türler](04-turler.md) | tür envanteri, birleşim, daraltma | RFC-0007, RFC-0008, RFC-0013 |
| [05 — Değerlendirme](05-degerlendirme.md) | yürütme sırası, taşma, determinizm | RFC-0001 §7, ADR-003 |
| [06 — Hata modeli](06-hata-modeli.md) | tanı sözleşmesi, Seçenek/Sonuç, test | RFC-0008, RFC-0010 |
| [07 — Birimler](07-birimler.md) | birim çözümü ve kapsülleme | RFC-0009 §2 |
| [08 — Kalıcı dosya IO](08-kalici-dosya.md) | atomik yazma, süreçler arası yarış ve dayanıklılık | RFC-0016 |
| [09 — Son tarih ve iptal](09-son-tarih-ve-iptal.md) | `içinde/yetişmezse`, işbirlikli iptal ve iç içe deadline | RFC-0011 |
| [10 — Dışa açık işlem sözleşmesi](10-disari-acik-islem-sozlesmesi.md) | public imza, dönüş kanıtı ve v1 kaynak ABI'si | RFC-0006, RFC-0007, RFC-0009 |
| [11 — Uygulama eylemleri ve web adaptörü](11-uygulama-eylemleri-ve-web-adaptoru.md) | açık eylem imzası, etki çıkarımı, yöntemli rota, savepoint/geri alma | RFC-0015 |
| [12 — Web güvenlik profili](12-web-guvenlik-profili.md) | erişim politikası, Argon2id, sunucu oturumu/rol, CSRF, güvenli çerez ve HTTPS proxy sınırı | RFC-0017 |
| [13 — Sürümlü morfoloji profili](13-surumlu-morfoloji-profili.md) | `zee-tr-1`, ek tablosu, çözüm↔üretim, proje/paket sabitlemesi | RFC-0018 |
| [14 — Yapılandırılmış eşzamanlılık](14-yapilandirilmis-eszamanlilik.md) | görev grubu, deterministik scheduler, sahiplik, hata/iptal yayılımı | RFC-0011, K-090 |
| [15 — Yapılandırılmış Hata değeri](15-yapilandirilmis-hata-degeri.md) | kod, mesaj, neden zinciri, veri, eşleme ve geriye uyum | RFC-0008, K-091 |
| [16 — Keyfî hassasiyetli Ondalık](16-keyfi-hassasiyetli-ondalik.md) | keyfî katsayı/ölçek, exact işlemler ve 34 haneli sonsuz bölüm bağlamı | RFC-0013, K-092 |
| [17 — Değer semantiği ve gezme](17-deger-semantigi-ve-gezme.md) | derin değer kopyası, liste değer-sonuç imleci ve T053 kaynak sabitliği | RFC-0019, K-093 |
| [18 — Tekrar üretilebilir paket yayını](18-paket-yayini.md) | `.zep`, Ed25519 yayın imzası, SPDX SBOM ve SLSA provenance | RFC-0020, ADR-006, K-094 |

## Faza bağlı — henüz spec dışı

Çok çekirdekli paralellik ve yarış/akış/dinamik görev yüzeyleri (RFC-0011),
ikili FFI/ABI (RFC-0012, Faz 4/5), standart kütüphane kararlılık
politikası, çok süreçli web durumu/idempotency (RFC-0015/0017), uzak registry
istemcisi ve TUF POUF (RFC-0020 §6) ile genel deprecation/edition modeli. Morfoloji
profili için kırıcı sürüm sınırı spec/13'te şimdiden tanımlıdır.
Bu başlıklar karara bağlandıkça buraya bölüm olarak eklenir.
