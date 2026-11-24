# K-395 — Büyük ek çıkarma (ürün H posta motoru, RFC-0043 Dilim 2 kısmi)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürün deposu özeldir; sunucu adı, alan adı, adres ve yerel yol taşınmaz
(ADR-087).

| Exact ürün dosyası (ürün commit'i `@@URUN_COMMIT@@`, 24 Kasım 2026) | Git blob | SHA-256 |
|---|---|---|
| Sürtünme kaydı (`ZM-F039` satırı) | `@@URUN_BLOB@@` | `@@URUN_SHA256@@` |

## Sürtünme (ZM-F039)

Ürünün SMTP alıcısı 25 MB'lık iletiyi A2c ile dosyaya alıyor; iletinin
~15–18 MB'lık tek base64 eki çıkarılamıyordu:

- Satır satır okuma ~4,7 MB'ta C024'e çarpıyor (çalışma değeri zarfı
  kümülatif; ayrı iş K-393).
- K-389'un 1 MiB pencereli çözümü çalışıyor (20 pencere × 1 048 554 bayt,
  15,3 MB, 0,56 sn), ama ek gövdesinin **bayt** başlangıcı bulunamıyor:
  pencereler `Metin` olduğundan harf konumu bayt konumu değil, dilde bayt
  arama yok.
- `çözerek … eklemeyi dene` hedef tavanı 16 MiB; 18 MB'lık ek sığmıyor.

K-164 kataloğunda sınıf "yerleşik yüzey eksik" (ZM-F034 ile aynı aile,
aynı üründe tekrar).

## Kesit ([ana.dil](ana.dil))

Bağlantı bloğu DATA'yı A2c ile dosyaya alır; `eki çıkar` ürünün MIME
ayrıştırıcısının yaptığı gibi `Content-Transfer-Encoding: base64` başlığını,
ardından gelen boş satırı (`"\n"` araması) ve kapanış sınırını **bayt
konumu** olarak arar; ek gövdesi tek çağrıda
`… baytından … baytını base64 çözerek … dosyasına eklemeyi dene` ile
akışla çözülür, tutamağın `sha256` alanı yanıt satırına girer. Değişiklik
öncesi bu kaynak `S015` verir (kalıp yok); şimdi ürün politikasıyla
`denetle` temizdir. 18 000 000 baytlık ekin bayt-kesin çıkarılması, 32 MiB
sınırı, bozuk base64'ün kaynak konumu ve iz replay'i
`compiler/tests/kodlanmis_dosya_testi.rs` ile gerçek `dil` sürecinde
sınanır.

Açık: dosyadan dosyaya quoted-printable ve RFC-0043 Dilim 2'nin kalanı;
C024 kümülatif zarfı K-393'tedir.
