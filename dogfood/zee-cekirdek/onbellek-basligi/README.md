# K-335 — Herkese açık yanıt için rota düzeyinde önbellek başlığı

Bu anonim kesit aktif `zee-cekirdek` ürününün (bkz. [KOKEN](../KOKEN.md))
web çalışma zamanı talebinin W3 maddesine bağlıdır. Ürün deposu özeldir;
müşteri, alan adı, sunucu yolu ve gerçek veri taşınmaz (ADR-087).

Exact harici ürün commit'i: `abaca3aba0e90f57d26435c4befffa903346c425`
(18 Kasım 2026). Talep belgesi 20 Ekim 2026'da `ca3870d` ile girdi.

| Exact ürün dosyası | Git blob | SHA-256 |
|---|---|---|
| `cekirdek/kaynak/rota_site.dil` (etkilenen `GET /robots.txt`, `GET /sitemap.xml`) | `c31879d8b2da079e0a68b2068ad4fd3578bc08cc` | `2663956679c55ee5f4c1a8fb6f87529c93afc75a28ef57fa626848ce35306608` |
| `cekirdek/deploy/nginx-onbellek-konum.conf` (bugünkü ön kapı eşlemesi) | `bfb1a7594cb70eee1ea51ac63f4410d46cf978f7` | `91ad9febfeb3753d75ce5d5c20d22213be23ded3f5d2b4de85ecce9324b0113f` |
| `docs/dil-talepleri/2026-10-20-web-eszamanlilik-ve-kopma.md` (talep, W3) | `7dc54f0d49c763e7ade3a2990e08b05871b1e7cb` | `f500dd0452366a7174f58661104317dcac992f16c338fd5b62f91ec614af8dfa` |

## Sürtünme

Çalışma zamanı her yanıta sabit `Cache-Control: no-store` yazar (spec/12 §5).
Ürün oturumsuz site sayfalarını paylaşılan önbellekte tutmak için ön kapıda
`proxy_hide_header Cache-Control` + istek yolu eşlemesi kullanıyor; "bu yanıt
oturumdan bağımsız" kararı uygulamadan uzakta, yol listesinde duruyor. Eşleme
yanılırsa (Set-Cookie'li ya da oturuma bağlı yanıtı işaretlerse) oturum başka
kullanıcıya sızar. K-164 kataloğunda karşılığı yok: yeni dar sınıf "yanıtın
önbelleklenebilirliği kaynakta söylenemiyor".

## Minimal değişiklik

[Reproducer](ana.dil) iki site rotasını `herkese açık önbelleklenebilir 300
saniye` ile açar; oturuma bağlı giriş formu sıradan rota kalır.
Dil yalnız şunu söz verir: GET/HEAD, oturum/CSRF/çerez yüzeyine dokunmayan
rota, Set-Cookie'siz 200 yanıtında `Cache-Control: public, max-age=0,
s-maxage=N`; diğer her durumda `no-store`. Karar
[ADR-160](../../../adr/160-rota-onbellek-basligi.md),
[RFC-0037](../../../rfcs/0037-rota-onbellek-basligi.md),
[spec/12 §7](../../../spec/12-web-guvenlik-profili.md).

`dil denetle .` bu klasörde hermetik geçer; gerçek worker kanıtı
`compiler/tests/web_cok_surec_testi.rs::onbelleklenebilir_rota_set_cookie_uretince_asla_public_olmaz`.

## Açık

Ürün rotalarının yeni politikaya geçmesi ve ön kapı eşlemesinin sadeleşmesi
ürün tarafındadır. `Vary`, ETag/koşullu istek ve önbellek temizleme (purge)
bu işin sözü değildir.
