# RFC-0033 — Ürün HTTP POST ve başlıklar

- **Durum:** **geçici kabul** (K-220)
- **Tarih:** 14 Ekim 2026
- **İlgili:** ADR-091, spec/23, GB-053

Gerçek Turnstile ve Bearer portal ihtiyacına üç minimal kalıp eklenir:

```zee
sonuç adres "POST" başlıklar gövde 5000 ile HTTP isteğini dene olsun
sır "TURNSTILE_SECRET" gizli ortam değerini okumayı dene olsun
okuma metin HTTP JSON sözlüğünü okumayı dene olsun
```

Birinci ve üçüncü Metin sözlüğü Sonucu; ikinci Metin Sonucu döndürür.
Sonuç değeri yalnız `başarılıysa` dalında okunur. Başlıklara ve JSON
istek gövdesine mevcut sözlük/`json metni` kalıpları yeterlidir; `{}`/`[]`
Zee söz dizimi değildir. Genel form encoder eklenmez; önceden kodlanmış
form gövdesi kayıpsız taşınır, gerçek Turnstile JSON’u da kabul eder.
[Resmî API](https://developers.cloudflare.com/turnstile/get-started/server-side-validation/).

Eski GET ve AST/value türleri, Rust facade, Cargo graph ve iz formatı
korunur. Yeni istekler native HTTPS, explicit origin, sınırlı başlık/gövde/
süre ve açık proje ortam adıyla çalışır. Sır taşıyan işlemler kayıt/replay’de
kapalıdır. JSON değerleri kanonik JSON metni olarak tip ayrımını korur.
Ayrıntı ve kanıt [ADR-091](../adr/091-http-post-baslik-ve-gizli-ortam.md).

K-220 / K-174: exact gerçek ürün talebiyle üç contextual kalıp eklendi; syntax gözlem penceresi 14.10.2026–11.11.2026 olarak bilinçli yeniden başladı. Core freeze ve insan/RC kapıları sürer; V1 tamamlandı olarak gösterilmez.
