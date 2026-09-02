# RFC-0017 — Web oturumu, yetki, CSRF ve güvenilir proxy profili

- **Durum:** geçici kabul — K-088 güvenlik profili ve K-134 istek transaction'ı
  saldırı/cancellation regresyonlarına bağlandı
- **Tarih:** 1 Eylül 2026
- **İlgili kararlar:** K-082, K-087, K-088, K-134, ADR-010; V1-P0-03
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

## Bilinçli sınır

Rate limit, secret dağıtımı, ortak harici oturum deposu, proxy kurulumu ve
sertifika yenileme deployment sorumluluğudur. Idempotency anahtarı RFC-0015'in
ayrı açık kapısıdır; V1-P0-03'te oturum/kimlik sözüymüş gibi gösterilmez.
