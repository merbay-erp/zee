# PostgreSQL TLS ve havuz production profili

Bu runbook K-163/F031, RFC-0026, ADR-062 ve spec/25'in işletim karşılığıdır.

## Bildirim ve sırlar

`proje.dil` yalnız secretsiz exact hedefi taşır:

```zee
veritabanı_hedefi "postgresql://db.example.internal:5432/uygulama" olsun
veritabanı_bağlantı_değişkeni "UYGULAMA_DATABASE_URL" olsun
veritabanı_göçleri "göçler" olsun
```

Sırlar ve CA süreç ortamından gelir:

```text
UYGULAMA_DATABASE_URL=postgresql://kullanici:...@db.example.internal:5432/uygulama?sslmode=require
UYGULAMA_DATABASE_URL_TLS_CA_PEM=-----BEGIN CERTIFICATE-----...
```

CA değişkenine leaf değil sağlayıcının doğrulanacak kök/ara CA sertifikası
konur. URL hostu sertifikanın DNS/IP SAN'ıyla eşleşmelidir. `hostaddr`,
`sslmode=prefer`, uzak hedefte `disable`, boş veya 256 KiB üstü CA reddedilir.
Sistem kökleri bilinçli olarak kullanılmaz.

## Kapasite hesabı

Bir worker için sabit profil:

- maksimum 4 bağlantı;
- checkout zaman aşımı 2 saniye;
- idle timeout 30 saniye;
- max lifetime hedefi 300 saniye; aktif kira bırakıldıktan sonra en geç 30
  saniyelik bakım çevriminde yenileme;
- her checkout'ta sağlık kontrolü.

Toplam üst sınır `worker sayısı × 4` bağlantıdır. PostgreSQL
`max_connections`, migration/işletim bağlantıları ve güvenlik payı düşüldükten
sonra bu toplamı taşımalıdır. Pool exhaustion C026'dır; uygulama bunu sınırsız
bekleme veya yeni thread açma ile gizlemez.

## Recovery ve kapanış

Stale lease checkout sağlık kontrolünde atılır. Transaction dışı idempotent
SELECT yalnız bir kez yeni lease ile tekrar edilebilir. Transaction içi read,
write ve COMMIT otomatik tekrar edilmez; COMMIT kaybı C027 sonuç-belirsizdir.

Kontrollü shutdown boş bağlantıları kapatır. Aktif kira zorla kesilmez; son
sahibi bıraktığında kapanır. Deployment drain süresi uygulamanın en uzun izinli
transaction süresini taşımalıdır.

## Sürekli izole TLS doğrulaması

K-222/GB-022 koşucusu gerçek PostgreSQL'de doğru CA/hostname/TLS 1.2,
yanlış CA/hostname ve NULL/non-text sonuç reddini kendi geçici CA/key ve
kümesiyle dört atlamasız vakada zorlar. Sürekli Linux CI bu kapıyı ayrıca
çalıştırır; default faz koşusunda dört gerçek-fixture vakası `ignored`
olarak dürüstçe raporlanır. [Koşucu, exact kimlik ve sınır](postgresql-tls-ci.md).
Bu yeni kapı production cluster ve uzun havuz soak kanıtı değildir.

## Saha doğrulaması

Gerçek TLS cluster hazırken opt-in test şu üç değişkenle çalıştırılır:

```text
ZEE_POSTGRES_SAHA_HEDEFI=postgresql://db.example.internal:5432/uygulama
ZEE_POSTGRES_SAHA_URL=postgresql://kullanici@db.example.internal:5432/uygulama?sslmode=require
ZEE_POSTGRES_SAHA_URL_TLS_CA_PEM=<PEM içerik>
```

`postgresql::havuz::testler::saha_profili_gercek_tls_exhaustion_ve_stale_recoveryyi_olcer`
testi dört ayrı backend PID, beşinci checkout reddi ve stale replacement'ı
doğrular. `ZEE_POSTGRES_SAHA_UZUN_SOAK=1` ayrıca 300 saniyelik active-lease
lifetime + bakım çevrimi davranışını ölçer. Bu opt-in test hermetik CI yerine gerçek saha
kanıtıdır; secrets ve CA repoya yazılmaz.

K-205/K-206 ek incelemede temiz 9e80b69, izole PG 16.15/TLS localhost
ortamında uzun native havuz testi 336,840 sn ve exit 0 verdi. Dört TLS 1.2
backend, exhaustion/stale recovery ve active-lease lifetime ölçüldü.
[Ham kayıt ve reproducer](incelemeler/k206/README.md). Bu üretim cluster
kanıtı veya bağımsız negatif hostname/CA/TLS downgrade testi değildir.
