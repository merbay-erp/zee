# Web production profili

Zee V1 web runtime'ı process başına bilinçli olarak tek worker'dır. Bir worker
aynı anda bir bağlantıyı okur, bir Zee request'i çalıştırır ve yanıtı bitirir.
Production concurrency'si, aynı proje kökünü paylaşan birden çok süreç ve
önlerindeki HTTPS reverse proxy ile kurulur.

Örnek iki worker:

```sh
dil çalıştır --web-proxy https://panel.example --web-worker-port 18091 uygulama.dil
dil çalıştır --web-proxy https://panel.example --web-worker-port 18092 uygulama.dil
```

Kaynak içindeki `sunucu başlat` kapısı production kipinde worker portuyla
geçersiz kılınır; rota ve uygulama kaynağı iki süreçte aynıdır. Proxy yalnız
`127.0.0.1:18091` ve `127.0.0.1:18092` hedeflerine yük dağıtır. Runtime
loopback dışına açılmaz; listener `127.0.0.1`e bind eder ve kabul edilen socket
peer'inin loopback olduğunu ayrıca doğrular.

## Ortak durum

Production worker'ları proje kökündeki `.zee/web-durumu-v1.json` dosyasını
paylaşır. Burada tokenın kendisi değil SHA-256 özeti, CSRF, kullanıcı/rol,
mutlak expiry ve atomik oran pencereleri tutulur. Süreç restart'ı oturumu
silmez; bir worker'daki logout diğerlerinde hemen görünür. `.zee`/dosya Unix
izinleri 0700/0600'dür. Dosya başka bir klasöre kopyalanmamalı, elle
düzenlenmemeli ve web sunucusundan servis edilmemelidir. `.zee/` kaynak
kontrolünde ve temiz kaynak arşivinde yok sayılır; klasör silinirse bütün
oturumlar ve oran pencereleri bilinçli olarak sıfırlanır. Yedek alınacaksa
dosya izinleriyle birlikte özel runtime durumu olarak korunmalıdır.

Bu adaptör aynı makine veya güvenilir, kilit ve atomik replace semantiği aynı
olan ortak dosya sistemi içindir. Birden çok host için vaat edilen harici depo
adaptörü henüz yoktur.

## Proxy güven sözleşmesi

Reverse proxy:

1. internetten yalnız HTTPS kabul eder;
2. request'i bütünüyle buffer'lar ve kendi header/body/timeout sınırlarını
   uygular;
3. istemciden gelen `Forwarded`, `X-Forwarded-For` ve benzeri proxy
   başlıklarını siler;
4. backend'e tam olarak bir
   `Forwarded: for=<kanonik-IP>;proto=https;host=panel.example` başlığı yazar;
5. özgün `Host`u yapılandırılmış hosta, unsafe yöntemde `Origin`i tarayıcıdan
   gelen tek değere bırakır;
6. worker'lara yalnız loopback üzerinden erişir.

Runtime birden çok `Forwarded` başlığını, virgüllü proxy zincirini, IP olmayan
`for` değerini, yanlış proto/hostu ve yalnız `X-Forwarded-For` taşıyan isteği
reddeder. Rate-limit anahtarı ancak bu doğrulamadan sonra oluşan kanonik IP'dir.
Loopback dışı peer, doğru görünen başlıklar taşısa bile 403 ile reddedilir.

## Kanonik origin

`--web-proxy`, `Host`, `Forwarded host` ve unsafe `Origin` aynı
`AgHedefi` parser'ından geçer. Ayrı bir CLI origin parser'ı yoktur. Geçerli
örnekler:

```text
https://panel.example
https://panel.example:8443
https://[2001:db8::1]:8443
```

DNS adı büyük/küçük harfe duyarsızdır; `https://PANEL.EXAMPLE:443/` kanonik
olarak `https://panel.example` olur. IPv4/IPv6 standart metinsel yazımına
çevrilir. Farklı port farklı origin'dir. HTTP,
kullanıcı bilgisi, yol/sorgu/parça, geçersiz DNS etiketi, ayraçsız IPv6 ve
0 ya da 65535'i aşan port fail-closed reddedilir. Proxy `Forwarded host`
değerini yapılandırılmış kanonik otoriteyle kurmalıdır.

## Sabit oran ve süreler

| Kapı | Pencere | Eşik | Aşım |
|---|---:|---:|---|
| Aynı istemci + yöntem + sorgusuz path | 60 sn | 100 | 429 |
| CSRF doğrulaması | 60 sn | 60 | 429 |
| Argon2id parola doğrulaması | 300 sn | 5 | Argon2id çalışmadan 429 |

Oturum 30, anonim form oturumu 10 dakika mutlak ömürlüdür. Erişim ömrü
kaydırmaz. Toplam oturum 4096, anonim oturum 1024, canlı rate anahtarı 32768
ile sınırlıdır. Kalıcı depo okunamaz/bozuksa veya kapasite güvenle
yönetilemiyorsa istek fail-closed 503 olur.

Reverse proxy buffering Slowloris etkisini internet sınırında keser; backend
ayrıca bağlantı kabulünden başlayan 10 saniyelik mutlak request okuma ve 30
saniyelik Zee request çalışma deadline'ı taşır. Benchmark sonuçları process
başına tek worker gerçeğiyle yorumlanmalı; `-c 100` tek bir process'te 100
paralel uygulama yürütümü vaat etmez.

K-140'tan itibaren backend request-line, başlık ve gövdeyi ham byte parser'da
doğrular. Proxy backend'e yalnız CRLF satırlı, origin-form hedefli, tek
`Content-Length` taşıyan ve `Transfer-Encoding` taşımayan HTTP/1.0/1.1 isteği
göndermelidir. Chunked aktarım, obs-fold, pipelining, UTF-8 dışı/NUL gövde ve
duplicate CL desteklenmez. Her yanıt `Connection: close` ile bağlantıyı
sonlandırır.
