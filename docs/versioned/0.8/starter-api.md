# Zee 0.8 starter API ve güven sınırı

Bu [başlangıcın](icerik-paneli.md) sözleşmesi ayrı Python owned host + mevcut
Zee HTTP/SQL uygulamasıdır; yeni supported Rust facade/API veya compiler CLI
komutu değildir. Host yalnız loopback HTTPS dinler; exact Host/Origin,
header16KiB/32alan, form16KiB/raw1MiB, response1MiB ve tek request10s sınırı
vardır. Persistent connection, chunked, Expect, Upgrade ve forwarded kullanıcı
başlıkları kabul edilmez. Deadline connect/read/send/EOF boyunca aynıdır.

| Yöntem/yol | Sözleşme |
|---|---|
| GET / | en çok50 içerik başlığı; HTML escaped |
| GET /icerik?id=... | ASCII kimlik, escaped başlık/body, ready medya |
| GET /giris, POST /giris-yap | native session/PHC/CSRF; yanlış parola rol açmaz |
| GET /admin | yönetici rolü, edit/create/upload/delete formları |
| POST /ekle, /guncelle | form id/title/body/_csrf; gerçek parametreli SQL |
| POST /sil | id/_csrf; medya referansı varken409 |
| POST /medya-yukle?id=... | raw octet-stream + X-Zee-CSRF, typed image pipeline |
| POST /medya-sil | source/png/_csrf; exact ready pair, dosya + ayrı SQL finalization |
| POST /cikis | native session revoke/CSRF |
| GET /media/<lower64>.png | nofollow/tek-link/owner/private, SHA eş, ≤1MiB RGBA8 PNG ≤320px |

`%` encoded path, dot/traversal, backslash, duplicate query/header/form ve
bilinmeyen route/method reddedilir. Query kimlikleri frontend'de ASCII sınırına
bağlıdır; data SQL parametresidir. Mutasyon hata latch'i ve operator review'u
[rehberde](icerik-paneli.md) tanımlıdır. Auth header/cookie/body günlüklenmez.
Host source/config pin'i bütünlüğü gösterir; operator güvenini imzalı publisher
veya hostile same-UID sandbox kanıtına dönüştürmez.
