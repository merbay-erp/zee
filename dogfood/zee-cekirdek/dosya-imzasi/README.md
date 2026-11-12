# K-333 — Yüklenen CV'nin kabul anında içerik imzası

Bu anonim kesit aktif `zee-cekirdek` ürününün (bkz. [KOKEN](../KOKEN.md))
oturumsuz kariyer başvurusu talebine bağlıdır. Ürün deposu özeldir; müşteri,
alan adı, sunucu yolu ve gerçek veri taşınmaz (ADR-087).

Exact harici ürün commit'i: `e8beb39e4fe9486331a9605307ff56468df473bc`
(12 Kasım 2026, zee-cekirdek 1.18.0 seçimli başvuru formu).

| Exact ürün dosyası | Git blob | SHA-256 |
|---|---|---|
| `secimli/basvuru-formu/kaynak/rota_basvuru.dil` (etkilenen `POST /basvuru-yukle`) | `d5e970d76a47b73ef1506533c8cbfe5561692610` | `de091d2e6386beccea39464dc3ef2c2dfa4b6c8d6b471038ee415ddbc7b8af68` |
| `docs/dil-talepleri/2026-11-12-dosya-imzasi-okuma.md` (talep) | `85aa3539395cf04607f61c6ad6ac337ce322340f` | `c2cff711b32e46e25d96e54d87aa7567dae2ece6a003d2b8de7264e3673560c1` |
| `secimli/basvuru-formu/test/basvuru_e2e.sh` (kabul ölçütü: "imzası PDF olmayan .pdf" adımı) | `1544342cc4353c7ca21b3366e6241a333a978161` | `05af072a67be0dae9c1e7ba836664c703a99383cbe7247abe1307fcfeb15caf0` |

## Sürtünme

Rota uzantı, boyut ve adı denetler; dosyanın bildirilen türde olduğunu
(PDF `%PDF-`, OLE `D0 CF 11 E0 A1 B1 1A E1`, ZIP `50 4B 03 04`) kabul anında
denetleyemez. spec/26 yüklenen dosya için yalnız taşı/sil/listele/sha256
verir; `dosyasını okumayı dene` UTF-8 metin bekler. Ürün imzayı yalnız
indirmede ayrı bir Node varlık sunucusunda denetliyor; imzası tutmayan dosya
diske ve veritabanına giriyor. K-164 kataloğunda karşılığı yok: yeni dar sınıf
"ikili içerik imzası okunamıyor" (K-232'nin "yerel ikili dosya okunamıyor"
sürtünmesiyle aynı aileden; ikinci ürün dosyası).

## Minimal değişiklik

[Reproducer](ana.dil) geçici yükleme yolunun ilk 4/5/8 baytını
`yol dosyasının ilk N baytını onaltılık okumayı dene` ile okur, ürünün imza
tablosuyla karşılaştırır; uyuşmazsa 415 döner ve geçici dosyayı bir eylem
içinde siler. [proje.dil](proje.dil) `dosya-okuma` yetkinliğini açar.
Dil yalnız sınırlı (≤ 64 bayt) okuma sözü verir; tür tablosu üründe kalır.
Karar [ADR-159](../../../adr/159-dosya-imzasi-okuma.md),
[RFC-0036](../../../rfcs/0036-dosya-imzasi-okuma.md),
[spec/26](../../../spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md).

`dil denetle .` bu klasörde hermetik geçer; gerçek ikili dosya ve sınır
kanıtı `compiler/tests/proje_testi.rs::dosya_imzasi_gercek_ikili_dosyada_sinirli_ve_proje_kokunde_okunur`.

## Açık

Ürün rotasının yeni ifadeye bağlanması, `basvuru_e2e.sh` adımının 415
beklentisine dönmesi ve filo dağıtımı ürün tarafındadır. Polyglot/antivirüs
denetimi kapsam dışıdır (GB-130).
