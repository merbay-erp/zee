# RFC-0037 — Herkese açık rota için önbellek başlığı sözleşmesi

- **Durum:** kabul — 18 Kasım 2026, K-335'te gerçeklendi; normatif metin
  spec/12 §K-335, karar ADR-160.
- **Tarih:** 18 Kasım 2026
- **İş:** K-335 (kaynak: zee-cekirdek `docs/dil-talepleri/2026-10-20-web-eszamanlilik-ve-kopma.md` W3, exact `abaca3a`)
- **İlgili belgeler:** spec/12, ADR-059, ADR-064, ADR-158, GB-134
- **Freeze sınıfı:** `dogfood-change` — mevcut `herkese açık` politikasının
  tek contextual biçimi; yeni değer türü, bağımlılık veya `dil::api::v1`
  facade değişikliği yok. Yeni tanılar T056/T057.

## Sorun

Her yanıt `no-store` taşır; oturumsuz sayfaları önbelleğe almak isteyen ürün
kararı ön kapıda, rotadan uzakta veriyor ve yanılırsa oturum sızar. Ayrıntı
ADR-160 §Bağlam.

## Öneri

```zee
GET "/sitemap.xml" adresine istek geldiğinde
    herkese açık önbelleklenebilir 300 saniye
    …
```

N 1–86400. Yalnız GET/HEAD (T056); oturum/CSRF/çerez kullanımı geçişli
olarak T057. Çalışma zamanı Set-Cookie'siz, oturuma dokunmamış 200 GET/HEAD
yanıtında `Cache-Control: public, max-age=0, s-maxage=N`, aksi hâlde
`no-store`.

## Alternatifler

Önsözde ayrı önbellek cümlesi, `proje.dil` yol listesi, genel yanıt başlığı
yazma, tarayıcı `max-age` ve yalnız uyarı ADR-160 §Değerlendirilen
seçenekler'de gerekçesiyle reddedildi. Seçilen biçim izni erişim
politikasının kendisine bağlar: kimlikli rota yapısal olarak
önbelleklenebilir olamaz.

## Açık

`Vary`, ETag/koşullu istek, `stale-while-revalidate` ve önbellek temizleme bu
RFC'nin sözü değildir.
