# Owned socket devri reproducer

K-259 / ADR-118, zee-cekirdek gerçek B6 talebinin anonim repo içi dilimidir.
External request commit `cd5d12f0008620819e6167af5146e83503fd2b38`, request SHA256
`694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7`,
etkilenen genel `cekirdek/kaynak/ana.dil` SHA256
`f0a65a0b1272a717728db4050fd450fd85900434ed6d5309bdcb9bbf9f138619`.
Özel ürün gövdesi, müşteri verisi ve hizmet yapılandırması taşınmaz.

Owned host tek loopback TCP LISTEN socket'ini yaratır. Native worker onun exact
fd duplicate'ini alır; bind/proxy relay/reuseport ile devir taklit edilmez. İlk
argüman host'un yalnız bu generation için açtığı private receipt yoludur.
Receipt anonim uygulama init'ini bildirir; DB/göç hazırlığı sözü değildir.
Native nonce/PID/build ready pipe ve app receipt doğrulanıp ACTIVATE+EOF gelmeden
accept başlamaz. `/slow` üç saniyelik kabul edilmiş istektir; inherited profile'da
TERM sonrası200/completed tamamlanır. Direct bind davranışı ayrı K253 sınırıdır.

`scripts/native-listener-host.py` sınırlı host API'sidir. Kabul koşucusu:
`python3 -I -B scripts/tests/native-listener-test.py --framework-root . --dil <owned-binary> --output <fresh-output>`.
Birleşmiş metrics/listener profili için ayrıca `--metrics-composition` kullanılır.
FD negatifleri `scripts/tests/native-listener-preflight-test.py` ile aynı üç
argümanla çalışır. Test kendi anonim session/quota kaynaklarını geçici dizinde
oluşturur.1000 istek concurrency1/dört restart ve ayrı sentetik loopback istemci
kimlikleriyle sınırlıdır; kota yükseltilmez/sıfırlanmaz ve istek replay edilmez.
Genel sıfır hata/SLA, farklı uygulama revision'larının atomik dağıtımı ve canlı
ürün benimsemesi açık kalır. JSON ortak durum korunur; process-memory quota
inherited overlap'ta kabul edilmez. K260 credential launcher socket aktarımı
ayrı açık kompozisyondur; bu reproducer doğrudan native worker kullanır.

Korpusun ürün kök politikası da `dosya-yazma` taşır: anonim B6 reproducer yalnız owned host tarafından verilen private generation receipt yoluna yazar. Alt proje bildirimi zaten bu yetkiyi taşıyordu; ürün kök aynası eksikliği ilk tam fazda T054 ile yakalandı ve aynı ilk özellik diliminde eşlendi. Bu kayıt gerçek ürün deposunu veya genel dış yol güven sınırını değiştirmez.
