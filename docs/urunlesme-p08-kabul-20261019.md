# P08 — anonim beş site ve çekirdek ürün korpusu kabulü (19 Ekim 2026)

**K-315 · P08/A5.** Beş gerçek, üretimde çalışan Zee sitesi ve paylaşılan çekirdek
kökü anonim tam paket olarak hazırlandı ve exact derleyiciyle iki platformda
ölçüldü. Paketler depoya kopyalanmaz ([dogfood rehberi](dogfood-korpusu.md)); özgün
ürüne köken bağı (ürün deposu, commit, paket kökü) özel ürün kaydında tutulur.

## Paketler

| Anonim kimlik | Tam paket (yerel) | Dosya | paket_sha256 (MANIFEST.sha256 özeti) |
|---|---|---|---|
| site-a | `zee-urun-korpusu/p08-20261019/site-a` | 174 | `3e7a90e8d0b0ddf2b650d4fa581fd1e8a2dfd29e5897f92e141f40d4b30524e6` |
| site-b | `zee-urun-korpusu/p08-20261019/site-b` | 265 | `ba1fe5ad64c5f6a399c9f447af1bab0f911a8ccf98b3359adf4b19e69c5d7749` |
| site-c | `zee-urun-korpusu/p08-20261019/site-c` | 241 | `2ec808c00e6a74abaab70d044e41caf3505d90096fa8e15a021713d03d11578d` |
| site-d | `zee-urun-korpusu/p08-20261019/site-d` | 305 | `6d1631b50599098035a887a50d34cc4d742680f01bbedf661d8dca36eb771315` |
| site-e | `zee-urun-korpusu/p08-20261019/site-e` | 203 | `0b494e347e76fca18ed457e8b145b98292c4805960d214f875c50df870a8f598` |
| cekirdek | `zee-urun-korpusu/p08-20261019/cekirdek` | 151 | `4829e0529c7e5a484727232b2dc63d7cdd5fe355e76d4db61b9a8ed308559272` |

Arşiv `p08-20261019.tgz` sha256 `f2dd94c9c69038846fae094a816cbb50534283860993c363306a01ff69cb901f`.
Her paket, ürünün üretimde dağıtılmış exact commit'indeki üretilmiş tam ağacıdır
(kaynak, göçler, statik dosyalar, dağıtım şablonları, betikler, testler).

## Anonimleştirme

Marka ve kod öneki, alan adı, e-posta, gerçek telefon (testlerin sentetik
numaraları korunur), yer adı, sunucu adı/yolu/IP ve görseller (aynı adla yer
tutucu; yazı tipleri korunur) tutarlı biçimde değiştirildi. Doğrulama:

- [Açık depo kapısı](acik-depo-politikasi-v1.json) sözlüğü: altı pakette **0** eşleşme.
- Gitleaks 8.30.1 `dir`: altı pakette **0** bulgu.
- Davranış eşliği: özgün ve anonim ağaçta `denetle` exit 0 ve `dene` sayıları aynı.

## Derleyici pini

Kaynak `2746606f2fdb4f0acdbbc5a0d6fbb29e44b7e057` (üretimde çalışan sürüm).
macOS arm64 ikili sha256 `aca48ae77f79f080b55356933378f40c819fecdd6af4ea7c1524dc37c42ed194`;
Linux x86_64 üretim ikilisi sha256 `3c3b724a9110c1f852b013f01964cc9699f0f6e6553564baa72451d218e15d18`.

## Sonuç (üçer koşu; süre medyan, RSS azami)

| Paket | denetle | dene N/N | macOS süre denetle/dene | macOS RSS KiB | Linux süre denetle/dene | Linux RSS KiB |
|---|---|---|---|---|---|---|
| site-a | exit 0 | 50/50 | 0.59 / 0.29 sn | 54768 | 2.01 / 0.95 sn | 42888 |
| site-b | exit 0 | 65/65 | 0.70 / 0.36 sn | 56800 | 2.46 / 1.25 sn | 47468 |
| site-c | exit 0 | 64/64 | 0.80 / 0.40 sn | 62816 | 2.73 / 1.29 sn | 49212 |
| site-d | exit 0 | 71/71 | 0.75 / 0.38 sn | 63520 | 2.58 / 1.18 sn | 51064 |
| site-e | exit 0 | 53/53 | 0.62 / 0.31 sn | 57168 | 2.08 / 1.00 sn | 44864 |
| cekirdek | exit 0 | 40/40 | 0.43 / 0.22 sn | 49232 | 1.37 / 0.65 sn | 35288 |

Toplam **343/343** test, iki platformda aynı. macOS Apple Silicon geliştirici makinesi;
Linux üretim sunucusu (Ubuntu, çekirdek 6.8). Süreler gözlemseldir, SLA değildir.

## Paketlerde gerçek kullanım (exact kaynakta sayım)

| Dilim | Kanıt (beş site) | Not |
|---|---|---|
| P10 e-posta | Her sitede `ekli ileti göndermeyi dene` (K-232: düz+HTML, Reply-To, cid gömülü görsel) form bildirimlerinde | Düz kalıp kullanılmıyor; katalog PDF eki zee dışı servisle gönderiliyor |
| P12 göç | 24–32 göç dosyası/paket, üretim dağıtımında `dil göçür` | Üretimde göç Python sarmalayıcıyla tetikleniyor |
| P13/P14 web | 80–88 rota/paket, üretimde `--web-proxy` worker | Sistem servis yöneticisi altında |
| P15 parola | Argon2id panel kimliği, 2FA | Canlı giriş |
| P16 form | 44–50 POST rotası; CSRF + form belirteci + tuzak alan | Turnstile kullanılmıyor |
| P20 modül SQL | 3–6 modül göçü/paket (1xxx/2xxx) | Modül mağazası |
| P21 TR–EN | 39–72 `/en/` rota/bağlantı | İki dilli siteler |
| P22 test | 42–73 `test` bloğu/paket | Ürün testleri `dene` ile koşar |
| P25 sır | Panel SMTP parolası `gizli_anahtar_değişkeni` ile şifreli | Sağlayıcı sırrı panelde |

Bu tablo kullanım kanıtıdır; ilgili maddelerin sağlayıcı, platform, insan ve
SLA sınırlarını kapatmaz.
