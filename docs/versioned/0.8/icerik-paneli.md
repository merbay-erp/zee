# Zee 0.8 — içerik, yönetim ve medya başlangıcı

K-290 / ADR-148 ayrı bir local owned starter'dır. `dil yeni site` mevcut genel
proje başlangıcını üretir; ona yeni bir site/compiler komutu eklenmedi. Tam
bu başlangıç için depo kökünden creator'a **yeni boş hedef adı** verilir:

```sh
python3 -I -B scripts/icerik-paneli-olustur.py --framework-root "$PWD" --root /absolute/private/site --dil /absolute/verified/dil --db-target postgresql://localhost:5432/zee_starter
```

Hedefin parent dizini kendi UID'ne ait0700 olmalı. Araç mevcut projeyi ezmez;
başarısız fresh kopyayı silmez. Mevcut `dil yeni` komutuyla bizzat çıkardığı fresh proje üzerine exact declared dosyaları yerleştirir; mevcut CLI kilitle/denetle/dene komutlarını gerçekten çalıştırır. Var olan `dil yeni site` klasörünü dönüştürmez; creator için farklı fresh hedef seçilir. DSN veya parola üretmez,
veritabanına bağlanmaz ve otomatik migration/install/download yapmaz. Yeni
native temiz-source kabulü tamamlanana kadar bu kaynak adaydır.

## Yerel operator hazırlığı

1. Kendine ait yeni PostgreSQL veritabanı ve gerçek TLS CA hazırla.
   `ZEE_STARTER_DATABASE_URL` ile eşleşen target'ın DSN'sini,
   `ZEE_STARTER_DATABASE_URL_TLS_CA_PEM` ile CA PEM'ini sağlayan mevcut güvenilir
   credential launcher/yerel environment kullanılır. Müşteri veritabanı bu
   başlangıç deneyi için kullanılmaz.
2. `dil göçür /absolute/private/site` explicit çalıştırılır. Mevcut transaction
   ve `_zee_gocleri` kaynak hash'leri korunur; CREATE çakışması yabancı tabloyu
   benimseme veya drop yetkisi değildir.
3. Site kökünde `(set -euo pipefail; set -C; umask 077; /absolute/verified/dil parola-özeti | tr -d '\n' > panel-phc)`
   Bash içinde kullanılır. Parola
   terminalden gizli alınır. PHC dosyası0600 ve tek link olmalı; eksik/bozuk dosya
   host başlamasını reddeder. Normal Zee dosya okuması secret IO redaction
   protokolü değildir; IO trace'e veya source kontrolüne alma.
4. Yerel TLS için yeni private dizinde, explicit operatör cert/key kullanılır.
   Örneğin bir günlük **yalnız deneme** self-signed sertifika:

```bash
(set -euo pipefail; umask 077; mkdir tls-local
 openssl req -x509 -newkey rsa:2048 -nodes -days 1 -subj /CN=localhost -addext subjectAltName=IP:127.0.0.1 -keyout tls-local/key.pem -out tls-local/cert.pem
 chmod 600 tls-local/key.pem tls-local/cert.pem)
```

Bu local sertifikaya istemcinin bilinçli güvenmesi gerekir; production PKI,
sertifika kurulumu veya public HTTPS kabulü değildir. Private key basılmaz.
Doğrulanmış binary'nin exact BuildJSON kaynak SHA'sı ve SHA-256'sını kullan:

```sh
python3 -I -B /absolute/private/site/yonetim/starter-content-host.py --root /absolute/private/site --dil /absolute/verified/dil --expected-source FULL40SHA --binary-sha256 FULL64SHA256 --front-port 9443 --tls-cert /absolute/tls-local/cert.pem --tls-key /absolute/tls-local/key.pem
```

Tarayıcı adresi `https://127.0.0.1:9443`. Native worker portu verilmez; host kendi
loopback listening socket'ini yaratıp mevcut inherited-listener/ready/activate
protokolüyle yalnız çıkardığı çocuğa verir. Ayrı typed host binary PNG yanıtını
sunar; Zee'ye native binary response özelliği eklenmiş değildir. Browser kabulü
ile programatik native HTTP/TLS kanıtı ayrı raporlanır.

## İş akışı ve arıza

`/giris` → `/admin`: yönetici, plaintext içerik create/update/delete ve 1MiB'a
kadar raw image upload yapar. PNG/JPEG/WebP mevcut image adaptörüyle en çok320px
RGBA PNG olur. Native role/session/CSRF ve fixed script CSP hash'i korunur.
Herkese açık liste/detay kullanıcı verisini HTML'den kaçışlar; medya aynı-origin
immutable `<sha256>.png` olarak strict host allowlist'inden gelir.

DB ve dosya sistemi ortak atomik transaction değildir. `preparing → ready`,
`ready → deleting → kaldırılmış` adımları ayrı SQL/dosya eylemleridir. Dosya
publish no-clobber'dır; hazırlanıyor/siliniyor ve orphan durumları bilinçli
saklanır. Aynı kaynak tekrar yüklenince var olan ready veri ezilmez. Çok-worker
saga veya automatic GC vaat edilmez; bir owned worker/tek front profili vardır.

Unsafe içerik/medya proxy isteğinden **önce** durable `starter-pending.json`
yazılır. Timeout/503/422/409 gibi belirsiz veya yarım işte sonraki mutasyonlar
kapalı, GET gözlemleri açık kalır. Host otomatik yeniden göndermez. Kaynak
isteğin body/cookie/CSRF'si latch'e yazılmaz; yalnız SHA-256/nonce vardır.

```sh
python3 -I -B yonetim/starter-content-reconcile.py --root /absolute/private/site --dil /absolute/verified/dil --expected-source FULL40SHA --binary-sha256 FULL64SHA256 --output /absolute/private/observation.json
```

Gözlem private dosyaya gerçek content/media satırlarını yazar; dışarı paylaşma.
Review `zee-content-starter-review-1`, exact observation SHA,
`decision=accept-observed-state-no-replay`, `media_actions` listesi ister.
Liste boşken yalnız incelenmiş mevcut durum kabul edilir. Açık medya kararı:
`mark-ready` (owned mevcut PNG hash doğrulaması), `discard-preparing` (yalnız
hazırlık metadata'sı; orphan dosyayı silmez), `complete-deleting` (PNG gerçekten
yoksa metadata sonlandırma). Her karar exact source_hash/png_hash taşır ve
bütün roller mutation öncesi doğrulanır. `--review /private/review.json` yeni
readonly gözlemi aynı SHA'ya bağlar; eski business request yeniden oynatılmaz.
Bu assertion insan incelemesinin veya exactly-once'ın otomatik kanıtı değildir.

Post-visible unlink/fsync arızası sabit exit4'tür; dosya yokken sahte paused
raporu verilmez. SIGTERM/INT sahipli child group cleanup'a ulaşır; SIGKILL/
powerloss/host kaybı aynı garantiyi vermez. Kaybolmuş PID üzerinden adoption yoktur.

## Açık sınırlar

Bu kaynak candidate teknik kabulü, gerçek novice30min ölçümü değildir. Gerçek
ürün/P08/adoption, physical Windows/minOS, remote CI, production deployment,
media AV/hostile same-UID sandbox, arbitrary HTML importer ve multi-worker
reconciliation ayrı açıktır. Önceki temiz S sonuçları yeni kaynak kabulü olmaz.
Eski site/panel şablonları ve RFC-0012 genel FFI taslağı aynı kalır.
