# K-378 — Ek akışı (ürün H web arayüzü, RFC-0041)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürünün web arayüzü (ZeeMail) ayrı ve özel depodadır; sunucu adı, alan adı,
adres ve yerel yol taşınmaz (ADR-087).

| Exact ürün dosyası (web arayüzü commit'i `0ee9d918bc5dc64516c14627310b18cd1a033cf8`, 23 Kasım 2026) | Git blob | SHA-256 |
|---|---|---|
| Sürtünme kaydı (`ZM-F022` satırı) | `cbd68d9e3c9b6ef0c840a9bd388839bd0ed8eafe` | `79cfe0fb6e759d5293a30b7b14d5097817577a6c51b0687782ec4ebb3f671d13` |
| Posta rotaları (`GET /posta/ek`: ikili ekte “İndirme yakında”) | `03489070cdc4c4a9afbc99f4199e160a4262fab8` | `8dd46219db4c50aad5aab4546e493e971ed1390b09c0e677571f907f2446e01e` |
| Kişi rotaları (`GET /kisiler/disa-aktar` text/plain + `<a download>`, `POST /kisiler/ice-aktar` yapıştırılan metin) | `eae54c1a995549ac5ee009959e6f55c659aa8650` | `888dbab8c39c51c1f198efc89a05695dcba9c3dac868ea610c0adf53e957f9da` |

## Sürtünme (ZM-F022)

Webmail'de PDF/görsel/Office eki indirilemiyor, dosya eklenemiyor ve
docx/xlsx/zip ekli ileti gönderilemiyordu. Ürün ikili ekte metin kartı
gösteriyor, vCard/CSV dışa aktarımını `text/plain` yanıt + `<a download>` ile,
içe aktarmayı dosya yerine yapıştırılan ≤ 64 KiB metinle yapıyordu. K-164
kataloğunda karşılığı “web adaptörü yüzeyi eksik”; aynı sınıf RFC-0040
katman C (dosyadan yanıt) ve RFC-0039 katman D'de tekrar ediyor.

## Katman A — dosya yanıtı ([ana.dil](ana.dil))

`GET /posta/ek` oturum sahibinin ekini `application/pdf` olarak,
`GET /kisiler/disa-aktar` vCard/CSV'yi kayıtlı metin türüyle `attachment`
yanıtı yapar. Değişiklik öncesi bu kaynak `S015` verir (kalıp yok); şimdi
`denetle` temizdir ve gerçek yanıt `dosya_yaniti_testi.rs` ile sınanır.
Ek yolunun kime ait olduğu ürün kararıdır (IDOR, GB-153).
