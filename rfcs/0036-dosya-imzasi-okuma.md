# RFC-0036 — Yüklenen dosyanın ilk baytlarını onaltılık okuma

- **Durum:** kabul — 12 Kasım 2026, K-333'te gerçeklendi; normatif metin
  spec/26 §Beklenen dosya işlemleri, karar ADR-159.
- **Tarih:** 12 Kasım 2026
- **İş:** K-333 (kaynak: zee-cekirdek `docs/dil-talepleri/2026-11-12-dosya-imzasi-okuma.md`, exact `e8beb39`)
- **İlgili belgeler:** spec/21, spec/26, ADR-059, ADR-061, ADR-064, GB-130
- **Freeze sınıfı:** `dogfood-change` — tek contextual kalıp, yeni değer türü,
  tanı kodu, bağımlılık veya `dil::api::v1` facade değişikliği yok.

## Sorun

Oturumsuz CV yükleme rotası dosyanın bildirilen türde olduğunu kabul anında
denetleyemiyor: dilde ikili dosya baytı okunamaz. Ayrıntı ADR-159 §Bağlam.

## Öneri

```zee
imza yol dosyasının ilk 8 baytını onaltılık okumayı dene olsun
```

`Metin sonucu`; küçük harf onaltılık; 1 ≤ N ≤ 64, dosya kısaysa olan kadar;
`dosya-okuma`; yol kuralları `sha256 özetini almayı dene` ile aynı; arıza
C013; IO izi `dosya_ilk_baytlari`.

## Alternatifler

İçerik türü (MIME) döndüren ifade, genel ikili okuma ve sabit uzunluklu okuma
ADR-159 §Değerlendirilen seçenekler'de gerekçesiyle reddedildi. Seçilen biçim
dilin sözünü "sınırlı bayt okuma" ile sınırlar; tür tablosu üründe kalır.

## Açık

Antivirüs/polyglot tespiti, multipart ve içerik türü doğrulaması bu RFC'nin
sözü değildir (spec/26 §Açık sınır).
