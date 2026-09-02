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
| [03 — Adlar ve kapsam](03-adlar-ve-kapsam.md) | morfolojik çözüm, blok kapsamı | RFC-0004, RFC-0018, K-089/K-120 |
| [04 — Türler](04-turler.md) | tür envanteri, birleşim, daraltma ve sıra-bağımsız yerel çağrı çıkarımı | RFC-0006/0007/0008/0013, K-121 |
| [05 — Değerlendirme](05-degerlendirme.md) | yürütme sırası, taşma, determinizm | RFC-0001 §7, ADR-003 |
| [06 — Hata modeli](06-hata-modeli.md) | tanı sözleşmesi, Seçenek/Sonuç, test | RFC-0008, RFC-0010 |
| [07 — Birimler](07-birimler.md) | birim, proje, yerel/exact registry paket çözümü ve kilit v3 | RFC-0009 §2–4, K-136 |
| [08 — Kalıcı dosya IO](08-kalici-dosya.md) | atomik yazma, süreçler arası yarış, platform metadata'sı ve dayanıklılık | RFC-0016, ADR-032, K-128 |
| [09 — Son tarih ve iptal](09-son-tarih-ve-iptal.md) | `içinde/yetişmezse`, işbirlikli iptal ve iç içe deadline | RFC-0011 |
| [10 — Dışa açık işlem sözleşmesi](10-disari-acik-islem-sozlesmesi.md) | public imza, yerel çıkarım, dönüş kanıtı ve v1 kaynak ABI'si | RFC-0006, RFC-0007, RFC-0009, K-121 |
| [11 — Uygulama eylemleri ve web adaptörü](11-uygulama-eylemleri-ve-web-adaptoru.md) | açık eylem imzası, etki çıkarımı, yöntemli rota, savepoint/geri alma | RFC-0015 |
| [12 — Web güvenlik profili](12-web-guvenlik-profili.md) | erişim politikası, Argon2id, ortak oturum/rate-limit deposu, CSRF, güvenli çerez ve kanonik HTTPS proxy sınırı | RFC-0017, ADR-034/036, K-137/K-139 |
| [13 — Sürümlü morfoloji profili](13-surumlu-morfoloji-profili.md) | `zee-tr-1`, immutable parmak izi, bağımsız conformance, çözüm↔üretim ve proje/paket sabitlemesi | RFC-0018, K-120/K-122/K-123 |
| [14 — Yapılandırılmış eşzamanlılık](14-yapilandirilmis-eszamanlilik.md) | `zee-esz-1`, kaynak sıralı gözlemler, görev grubu, sahiplik ve hata/iptal yayılımı | RFC-0011, K-090/K-124 |
| [15 — Yapılandırılmış Hata değeri](15-yapilandirilmis-hata-degeri.md) | kod, mesaj, neden zinciri, veri, eşleme ve geriye uyum | RFC-0008, K-091 |
| [16 — Keyfî hassasiyetli Ondalık](16-keyfi-hassasiyetli-ondalik.md) | keyfî katsayı/ölçek, exact işlemler, 34 haneli bölüm ve örtük binary float yasağı | RFC-0013, ADR-029, K-092/K-125 |
| [17 — Değer semantiği ve gezme](17-deger-semantigi-ve-gezme.md) | derin değer kopyası, liste değer-sonuç imleci ve T053 kaynak sabitliği | RFC-0019, K-093 |
| [18 — Tekrar üretilebilir paket yayını](18-paket-yayini.md) | NFC/Unicode güvenlik profilli kanonik `.zep`, Ed25519 imzası, SPDX SBOM ve SLSA provenance | RFC-0020, ADR-006/028, K-094/K-117 |
| [19 — Registry metadata güven zinciri](19-registry-metadata-guveni.md) | eşik root/rotasyon, rol bağları, rollback/expiry, doğrulanmış cache ve exact proje/kilit/CLI | RFC-0020, ADR-006, K-095/K-135/K-136 |
| [20 — İfade grameri](20-ifade-grameri.md) | primary→postfix→çağrı→aritmetik→birleştirme→karşılaştırma→boolean katmanları, tam tüketim ve formatter eşdeğerliği | RFC-0021, ADR-002, K-097/K-119 |
| [21 — Deterministik IO izi](21-deterministik-io-izi.md) | sürümlü kanonik olay biçimi, bütçeli kayıt ve dış etkisiz replay | RFC-0022, ADR-026, K-115 |
| [22 — Deterministik IO profili](22-deterministik-io-profili.md) | `zee-io-1` tohum, rastgele dizi, sanal saat ve hermetik adaptör sözleşmesi | RFC-0023, ADR-027, K-116 |
| [23 — Yetkinlik ve outbound ağ güvenliği](23-yetkinlik-ve-outbound-guvenligi.md) | proje/paket izinleri, compile/runtime kapısı, ortak inbound/outbound origin, DNS/HTTPS ve dosya kökü | RFC-0024, ADR-031/036, K-127/K-139 |
| [24 — Kaynak güvenlik profili](24-kaynak-guvenlik-profili.md) | kaynak/token, runtime adım/çıktı/değer heap'i, domain limit sahipliği, bağlantı, dosya ve protokol-kesin LSP sınırları | RFC-0025, ADR-033/035, K-129/K-130/K-131/K-132/K-138 |

## Faza bağlı — henüz spec dışı

Çok çekirdekli paralellik ve yarış/akış/dinamik görev yüzeyleri (RFC-0011),
ikili FFI/ABI (RFC-0012, Faz 4/5), standart kütüphane kararlılık
politikası, çok-hostlu web durumu/idempotency (RFC-0015/0017) ile genel
deprecation/edition modeli. Morfoloji
profili için kırıcı sürüm sınırı spec/13'te şimdiden tanımlıdır.
Bu başlıklar karara bağlandıkça buraya bölüm olarak eklenir.
