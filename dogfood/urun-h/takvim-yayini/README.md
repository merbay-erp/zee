# K-406 — Takvim yayını (ürün H web arayüzü, ADR-163 eki)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürünün web arayüzü (ZeeMail) ayrı ve özel depodadır; sunucu adı, alan adı,
adres ve yerel yol taşınmaz (ADR-087).

| Exact ürün dosyası (ürün commit'i `b75db4c7a460652f24c5d96d60817593587fbc70`, 26 Kasım 2026) | Git blob | SHA-256 |
|---|---|---|
| Sürtünme kaydı (`ZM-F048` satırı) | `5175dbf4bca8ba6b946290468703b4a753288cb6` | `e03452732b790d2ba93edcdd67ace64eb01beeb01891dbf6953ccf43a551d1c9` |
| Takvim rotaları (`GET /takvim/yayin.ics`: takvim `metin yanıtını gönder` ile `text/plain` iniyor) | `ee1ec87a17c0f06dcd79cee9a75c11d716a28810` | `a2eff1ca5fb3f6b333440936d16cf979ed45fe207b877a2522c69a411f22f830` |

## Sürtünme (ZM-F048)

Ürün takvimleri gizli belirteçli `GET /takvim/yayin.ics?k=<belirteç>`
adresinde yayımlar (Outlook "takvim yayımla", Google "gizli iCal adresi").
Takvim bellekte kurulan bir `Metin`dir. Bellekteki metin yanıtı yalnız
`text/html` ya da `text/plain` olabiliyordu; `text/calendar` yalnız dosya
yanıtındaydı ve GET rotası dosya yazamaz (T045). Ürün takvimi `text/plain`
ve LF ile veriyordu. K-164 kataloğunda karşılığı “web adaptörü yüzeyi eksik”
(ZM-F022/ZM-F033 ile aynı sınıf).

## Kesit ([ana.dil](ana.dil))

Rota takvimi `takvim metnini "text/calendar" türünde gönder` ile verir;
yanıt `text/calendar; charset=utf-8`, `nosniff`, `no-store`, sandbox CSP
taşır ve `Content-Disposition` yazmaz. Değişiklik öncesi bu cümle S037 idi.
Gerçek yanıt, başlıklar ve gövde baytları
`dosya_yaniti_testi.rs::turlu_metin_yaniti_kayitli_tur_guvenlik_basliklari_ve_ham_crlf_govdeyle_gider`
ile sınanır.

RFC 5545 CRLF satır sonu için yeni sözdizimi açılmadı: gövde bayt bayt
gider ve kesit CR'yi K-402 yüzeyiyle (`13 kod noktası harfini almayı dene`)
üretip her içerik satırını CRLF ile bitirir (ürün sürtünmesi ZM-F004,
ADR-163 eki §4).
