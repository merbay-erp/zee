# RFC-0017 — Web oturumu, yetki, CSRF ve güvenilir proxy profili

- **Durum:** geçici kabul — K-088 güvenlik profili, K-134 istek transaction'ı,
  K-137 ortak durum/rate-limit, K-139 kanonik origin/loopback peer ve K-140
  byte HTTP framing sınırı saldırı regresyonlarına bağlandı
- **Tarih:** 1 Eylül 2026
- **Revizyon:** 2 Eylül 2026 — K-139/ADR-036 ve K-140/ADR-037
- **İlgili kararlar:** K-082, K-087, K-088, K-134, K-137, K-139, K-140,
  ADR-010, ADR-018, ADR-034, ADR-036, ADR-037; B-046, B-052, B-053, V1-P0-03
- **Normatif metin:** spec/12

## Problem

K-087 web rotasını uygulama eyleminden ayırdı ancak kullanıcının elle
ürettiği küçük tokenı çereze ve dosyaya yazmasını engellemiyordu. Düz
parola, session fixation, CSRF, yetkinin istemciden gelmesi ve sahte proxy
başlıkları production sözünü imkânsız kılıyordu.

Güvenlik doğru yazılması gereken bir örnek değil, atlanamayan dil/runtime
kapısı olmalıdır.

## Karar

1. Unsafe rota erişim politikasını ilk satırda açıklar; public olmak da
   sessiz varsayım değildir.
2. Tarayıcı unsafe yöntemleri sunucu oturumuna bağlı CSRF synchronizer token
   olmadan rota gövdesine ulaşamaz.
3. Oturum kimliği ve CSRF işletim sistemi CSPRNG'sinden ayrı 256 bit değerler
   olarak üretilir. Depoda oturum kimliğinin yalnız özeti tutulur.
4. Giriş eski/anonim oturumu siler; kimlik ve CSRF'yi birlikte döndürür.
   Çıkış kaydı iptal eder. Anonim form oturumu 10, giriş oturumu 30
   dakikadır.
5. Parola API'si yalnız Argon2id PHC doğrular; CLI aynı biçimde özet üretir.
6. Rol sunucu oturumunda tutulur. 401 kimlik, 403 yetki/CSRF ayrımı korunur.
7. Production sunucusu TLS yazmaz. Yalnız loopback'teki güvenilir HTTPS
   reverse proxy'ye bağlanır ve Host/proto/Origin üçlüsünü doğrular.
8. Production çerezi `__Host-`, Secure, HttpOnly, SameSite=Lax, Path=/ ve
   sınırlı Max-Age taşır.
9. Bir istekteki oturum/çerez mutation'ı ilk yanıtla birlikte tamponlanır.
   Yalnız rota başarıyla bitip socket yanıtı eksiksiz yazılırsa commit edilir;
   deadline, runtime/yazma hatası veya yanıtsız rota hepsini geri alır.
10. `--web-proxy` production profili oturum, revoke, expiry ve oran
    sayaçlarını proje kökündeki kalıcı ortak depoda atomik tutar. Deneysel web
    süreç içi adaptörü kullanabilir.
11. Kanonik istemci kimliği yalnız güvenilir loopback proxy'nin yeniden
    kurduğu tek-hop `Forwarded` başlığındaki IP'dir. İstemciden taşınmış proxy
    başlıkları ve `X-Forwarded-For` kimlik kaynağı değildir.
12. Production profili istemci+yöntem+sorgusuz path için 60 saniyede 100,
    CSRF için 60 saniyede 60 ve Argon2id parola doğrulaması için 300 saniyede
    5 denemelik ortak atomik pencere uygular. Aşım 429'dur; depo/kapasite
    güvenle yönetilemiyorsa 503'tür.
13. V1 sunucusu process başına tek worker'dır. Eşzamanlı kapasite, aynı
    depoyu paylaşan N ayrı süreç ve her süreç için `--web-worker-port` ile
    kurulur; process içi thread-pool sözü verilmez.
14. `--web-proxy`, Host, tek-hop `Forwarded host` ve unsafe Origin, outbound
    allowlist ile aynı kanonik `AgHedefi` DNS/IPv6/port parser'ını kullanır.
    Listener sabit `127.0.0.1`e bind eder ve kabul edilen socket peer'ini de
    loopback olarak doğrular.
15. HTTP request-line, header ve gövde, metne çevrilmeden önce tek byte
    parser'da doğrulanır. Yalnız CRLF+origin-form+HTTP/1.0/1.1 ve exact tek
    `Content-Length` kabul edilir; TE, obs-fold, NUL, UTF-8 dışı ve fazla/eksik
    framing fail-closed reddedilir.

## Dil yüzeyi

```text
GET "/giris" adresine istek geldiğinde
    csrf csrf belirteci olsun
    # csrf, formdaki _csrf alanına yazılır

POST "/giris-yap" adresine istek geldiğinde
    herkese açık
    "parola" alanı gerekli
    verilen parola_özeti ile doğrulanıyorsa
        "Zeynep" kullanıcısını "yönetici" rolüyle oturuma al

POST "/kaydet" adresine istek geldiğinde
    "yönetici" yetkisi gerekli
    "not" alanı gerekli
    # iş kuralı eylem olarak çağrılır

POST "/cikis" adresine istek geldiğinde
    oturum gerekli
    oturumu kapat
```

## Reddedilen seçenekler

- **Uygulamanın elle token üretmesi:** CSPRNG, rotation, ömür ve revoke
  her uygulamada yeniden, eksik ve denetlenemez kurulurdu.
- **İmzasız JWT'yi varsayılan yapmak:** istemci tarafı rol/revoke karmaşası
  ve anahtar yönetimi getirir. Stage 0 için sunucu tarafı depo daha küçük ve
  daha açık bir sözdür.
- **Naif double-submit cookie:** CSRF değerini sunucu oturumuna bağlamadan
  cookie enjeksiyonu riskini büyütürür. Synchronizer token seçildi.
- **Runtime içinde TLS:** sertifika, cipher ve protokol yaşam döngüsünü
  bootstrap sunucusuna yükler. Dar loopback + doğrulanan reverse proxy sınırı
  seçildi.

## Kanıt

Conformance paketi CSPRNG farklılığını, Argon2id doğru/yanlış parolayı,
session rotation/revoke/süre dolumunu, rol ayrımını, eksik/sahte/geçerli
CSRF'yi, zorunlu alanı, güvenli çerez niteliklerini, CRLF enjeksiyonunu,
tekrarlı Host'u ve yanlış proto/Origin'i olumsuz testlerle sabitler.
K-134 kanıtı ayrıca timeout/runtime hatasında erken yanıtın, giriş çerezinin ve
sunucu oturumunun sızmadığını; gerçek TCP'de commit öncesi bayt çıkmadığını ve
socket yazma hatasının oturumu commit etmediğini doğrular.
K-137 kanıtı iki bağımsız CLI sürecinde login'in diğer worker'da görülmesini,
restart sonrası oturumun korunmasını, worker'lar arası logout/revoke'u ve
iki worker'a dağıtılan yanlış parola denemelerinin altıncıda Argon2id öncesi
429 olmasını doğrular. Ayrı persistent depo testleri bozuk şema/symlink'i,
Unix izinlerini, expiry'yi, koşullu rollback'i ve on eşzamanlı istemcide atomik
oran eşiğini korur.
K-139 kanıtı DNS harf farkı, varsayılan/farklı port, standart metinsel IPv6,
geçersiz şema/yol/kullanıcı/etiket/ayraç/port ve loopback/dış peer ayrımını;
mimari test ise ikinci origin parser'ının geri dönememesini doğrular.
K-140 kanıtı CRLF, request-target, alan adı/değeri, TE/CL, exact gövde ve
UTF-8 olumsuzlarını sabit regresyonla; ham byte uzayını ayrı libFuzzer
hedefiyle tarar.

## Bilinçli sınır

Secret dağıtımı, proxy kurulumu ve sertifika yenileme deployment
sorumluluğudur. V1 ortak depo garantisi aynı makineyi veya güvenilir kilit ve
atomik replace semantiği taşıyan ortak dosya sistemini paylaşan süreçlerle
sınırlıdır; çok-hostlu harici backend ayrıca tasarlanacaktır. Idempotency
anahtarı RFC-0015'in ayrı açık kapısıdır; V1-P0-03'te oturum/kimlik sözüymüş
gibi gösterilmez.
