# K-405 — Outlook POX autodiscover (ürün H web arayüzü, ADR-179)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürünün web arayüzü (ZeeMail) ayrı ve özel depodadır; sunucu adı, alan adı,
adres ve yerel yol taşınmaz (ADR-087).

| Exact ürün dosyası (ürün commit'i `effa18c0422fc8372c6826b689c1d11545609daf`, 26 Kasım 2026) | Git blob | SHA-256 |
|---|---|---|
| Sürtünme kaydı (`ZM-F047` satırı bu commit'te eklendi) | `4ac940de210242d51cf5aa0ba2890bbf9071fb3e` | `412b0d1047b554263c1dc77472435a17a7456c51da380c5a5fb4b4f560a1c45d` |
| Ayarlar rotaları (Thunderbird autoconfig ve Apple profili; Outlook POX buraya eklenecek) | `d82fcd0e7938e74cb6496d6ccae5c06c56aa69fe` | `0f3e609678ecfb44707575c9815c7108e4f610d7a592d812ddd5f2d35c17a60f` |

## Sürtünme (ZM-F047)

Klasik masaüstü Outlook hesap ayarlarını
`POST /autodiscover/autodiscover.xml` ile ister; gövde ~500 baytlık
`text/xml` belgesidir ve `<EMailAddress>…</EMailAddress>` taşır. Her POST
rotası `_csrf` istiyordu (`herkese açık` CSRF'yi kapatmaz), `--web-proxy`
profili `Origin` istiyordu ve gövde form olarak ayrıştırılıyordu; Outlook
ikisini de göndermediği için 403 alıp elle kuruluma düşüyordu. Thunderbird
autoconfig'i ([istemci-yapilandirma](../istemci-yapilandirma/README.md),
K-385), Apple profili ve SRV kayıtları zaten çalışıyordu. K-164 kataloğunda
sınıf "web adaptörü yüzeyi eksik" (ZM-F022, ZM-F033 ile aynı sınıf).

## Kesit ([ana.dil](ana.dil))

Rota `herkese açık makine uç noktası` taşır ve `isteğin ham gövdesi` ile
gövdeyi alır. Dil XML ayrıştırmaz: kesit `<EMailAddress>` ile böler ve yalnız
öğenin varlığını sınar; adres hiçbir yere yazılmaz (günlük/kişisel veri) ve
yanıt her alan adı için aynı durağan `application/xml` dosyasıdır (alan adı
sayımı yok; RFC-0046 tehdit modeli). Ürünün sürtünme kaydı yanıtı yalnız
kayıtlı alan için vermeyi öngörür; öyle yapılırsa tanınmayan alana da aynı
durum ve biçimde yanıt verilmesi önerilir. Değişiklik öncesi derleyici bu kaynağı
T049 ile reddediyordu (politika tanınmıyordu); şimdi ürün politikasıyla
`denetle` temizdir. Gerçek worker'da Origin'siz/CSRF'siz 200, Set-Cookie
yokluğu, 415 ve 413
`compiler/tests/web_cok_surec_testi.rs::makine_uc_noktasi_origin_csrf_cerezsiz_calisir_413_415_verir`
ile sınanır.

Açık: ürün taslağındaki GET `/autodiscover/autodiscover.xml` makine ucu
değildir (yalnız POST; GET zaten CSRF istemez, sıradan `herkese açık` rota
olarak yazılır). MS-OXDSCLI yanıtı kullanıcıyı anlatır; kullanıcıya göre üretilen
yanıt K-406 türlü metin yanıtıyla (`… metnini "application/xml" türünde
gönder`) yazılabilir; bu kesit durağan dosya yanıtıyla (`attachment`) kalır.
