# K-385 — İstemci yapılandırma dosyaları (ürün H web arayüzü, ADR-163 eki)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürünün web arayüzü (ZeeMail) ayrı ve özel depodadır; sunucu adı, alan adı,
adres ve yerel yol taşınmaz (ADR-087).

| Exact ürün dosyası (ürün commit'i `eb019cd0106cd7758f425f0875e3d6a6b201c31d`, 24 Kasım 2026) | Git blob | SHA-256 |
|---|---|---|
| Sürtünme kaydı (`ZM-F033` satırı) | `ba3d9b1f898e21c84cc7b6398f2fb35e840d76f3` | `3f593e4d5addf0cbc7f872191bf1225696080c6b66e67af28b53500981a4839b` |
| Ayarlar rotaları (`POST /ayarlar/apple-profili`: profil `application/octet-stream` ile iniyor) | `76f139af9e5cf2f851f43987cff87ef6c4497903` | `840172f51c3675451fffa547636b996552c9dc9ed99af912957957bd672be92d` |

## Sürtünme (ZM-F033)

K-378 dosya türü kaydında `application/xml` ve
`application/x-apple-aspen-config` yoktu. Thunderbird otomatik yapılandırma
XML'i (`/.well-known/autoconfig/mail/config-v1.1.xml`) sunulamıyordu (ürün bu rotayı tür yokluğu yüzünden hiç yazmadı; kesitteki iki autoconfig rotası ürünün hedeflediği biçimdir), Apple
profili `application/octet-stream` olarak iniyor ve iOS Safari kurulum
akışını başlatmıyordu (ürün geçici olarak Dosyalar'dan açtırıyor). K-164
kataloğunda karşılığı “web adaptörü yüzeyi eksik” (ZM-F022 ile aynı sınıf).

## Kesit ([ana.dil](ana.dil))

Autoconfig'in iki yolu `application/xml`, profil
`application/x-apple-aspen-config` ile `attachment` yanıtı yapar. Derleme
değişiklik öncesi de temizdi; sürtünme çalışma zamanındaydı (XML C013 →
404). Gerçek yanıt ve başlıklar
`dosya_yaniti_testi.rs::dosya_yaniti_autoconfig_xml_ve_apple_profili_kayitli_turle_gider`
ile sınanır. Outlook POX Autodiscover POST ister ve CSRF kuralı nedeniyle bu
kesitte yoktur; K-405 (ADR-179) ile ayrı kesitte:
[outlook-autodiscover](../outlook-autodiscover/README.md).
