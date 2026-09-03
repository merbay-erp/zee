# ADR-062 — PostgreSQL TLS ve sınırlı havuz sahipliği

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-163/F031, RFC-0026, ADR-060, spec/25

## Bağlam

İlk K-163 adaptörü tek loopback client ile correctness ve COMMIT
belirsizliğini kanıtladı. Gerçek ürünün production sınırı; uzak hedefte
sertifika doğrulaması, sabit bağlantı bütçesi, havuz tükenmesi, stale bağlantı
atımı ve kontrollü kapanışı aynı retry sözleşmesini bozmadan taşımayı gerektirdi.

## Karar

`postgresql/havuz.rs` transport ve lease sahibidir. `veritabani_modeli` yalnız
secretsiz hedef/veri sözleşmesi taşır; TLS veya pool davranışını çağırmaz.
Runtime transaction başladığında tek lease alır ve sınır bitene kadar tutar.
Transaction dışı işler lease'i işlem sonunda bırakır.

Production TLS profili açık `sslmode=require` ister. Kök yalnız
`<BAĞLANTI_DEĞİŞKENİ>_TLS_CA_PEM` içinden gelir; sistem kökleri kapalıdır,
hostname doğrulaması ve en az TLS 1.2 zorunludur. `hostaddr` ve örtük
`sslmode=prefer` fail-closed reddedilir. `sslmode=disable` yalnız loopback'te
yerel geliştirme uyumluluğudur.

Her worker en çok 4 bağlantı, 2 saniye checkout, 30 saniye idle ve 300 saniye
azami lifetime hedefi taşır. Aktif lease kesilmez; süresi dolan bağlantı
bırakıldıktan sonra en geç 30 saniyelik bakım çevriminde emekliye ayrılır. Her
checkout doğrulanır. Havuz düşen/stale bağlantıyı
atar; salt read'in mevcut tek-retry sözleşmesi yeni lease ile devam edebilir.
Write, transaction içi read ve COMMIT otomatik retry kazanmaz. COMMIT
belirsizliği C027 kalır.

Havuz kapanırken aktif lease zorla kesilmez; son kullanıcı bıraktığında
bağlantı kapanır. Connection error ayrıntısı dört neden/512 karakterle
sınırlandırılır. Çok-worker toplamı ortak global pool değildir; deployment
bağlantı bütçesini `worker sayısı × 4` üzerinden sınırlar.

## Kanıt ve açık sınır

Hermetik test 4/5 exhaustion, stale replacement ve son lease'e kadar yaşamayı
sahte manager ile zorlar. Açık saha testi gerçek PostgreSQL 16.11 cluster'ında
pinned CA, hostname reddi, farklı backend PID'leri, stale recovery ve opsiyonel
300 saniye + bakım çevrimi lifetime soak'ını doğrular. K-163 ürün runbook'u gerçek worker idle
cleanup ve shutdown sonrası sıfır bağlantıyı kaydeder.

Managed-provider CA rotasyonu, proxy/load-balancer davranışı ve birden çok
worker'ın birlikte max_connections provası bu ADR'nin kanıtladığı kapsam
değildir; release/deployment tatbikatında ayrıca ölçülür.
Exact K-163/F031 ürün saha kaydı `977cd2a` commit'indedir.
