# ADR-107 — Mevcut PostgreSQL eylemleriyle kalıcı iş kuyruğu pilotu

- **Durum:** kabul, dar SQL prototipi
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-247/P09, GB-070

## Bağlam ve karar

Exact `zee-cekirdek` B1 talebi kalıcı lease/idempotency, üç deneme,
exponential backoff, dead-letter, crash recovery ve concurrency sınırı
ister. Mevcut parametreli PostgreSQL yazma/okuma ve `eylem` transaction
yüzeyi bu dar prototip için yeterlidir. Yeni grammar, compiler komutu,
intrinsic, desteklenen Rust API, dependency veya cache eklenmez.

Anonim [ürün helper'ı](../dogfood/zee-cekirdek/is-kuyrugu/kaynak/kuyruk.dil)
ve migration, business-key/onay-revizyonu için tek immutable payload'lı
iş kaydı, ready/leased/retry_wait/done/dead_letter durumları, lease token,
generation, expiry ve en fazla üç başlamış attempt taşır. Bu profil tek
sabit idempotent SQL handler içindir; genel job-kind/dispatch tasarımı
ve gerçek ürün Python betiklerinin kaldırılması tamamlanmaz.

Claim writer CTE içinde `FOR UPDATE SKIP LOCKED` kullanır; concurrency
limit/single-owner kontrolü ayrı singleton writer gate'inde transaction
boyunca seri tutulur. Sonuç, aynı eylemde saf SELECT ile okunur. Read API
üzerinden DML RETURNING ile effect sınıfı atlanmaz. Begin-attempt commit'i
handler'dan önce onaylanır. Sabit SQL etki ve fenced tamamlanma aynı
eylemde atomiktir; başarısız fenced geçiş bütün etkiyi geri alır. Geçersiz
eski nesil etkileyemez. İlk iki ret 1/2 saniye backoff, üçüncü dead-letter;
lease recovery yalnız idempotent SQL profilindedir.

## Belirsizlik ve güvenlik sınırı

C027 çocuk CLI'ını durdurur. Ayrı süreç stable business-key/job/token/nesil
ile durable sonucu okuyup uzlaştırır; belirsiz tel yazımı otomatik
tekrarlanmaz. COMMIT fault fixture ve bağımsız PG gözlemcisi aynı sınırı
native sınar. `last_outcome` ayrıntılı credential/provider hatası taşımaz.

GB-070 kabul edilen sınırdır: iş payload'ı SQL, shell, callback veya kod
olarak çalıştırılmaz. Test counter handler'ı yalnız geçici fixture'dır;
izlenen ürün adoption'ı değildir. SMTP/dosya/HTTP dış etkisi için aynı
transaction garantisi çıkarılmaz. C029 teslim belirsizliğini üç otomatik
SMTP retry'a çevirmek yasaktır; canlı provider ve dış-etki uzlaştırması
P09/P10 kapsamında açık kalır. Genel exactly-once veya takvim scheduler
sözü yoktur.

## Kanıt ve etki incelemesi

11 actual native vaka owned PG16.15'te iki worker/limit, stale token,
expiry, üçüncü başarı/dead-letter, gerçek lock barrier+SIGKILL ve
enqueue/handler COMMIT önce/sonra fault+durable observer'ı yürütür.
Normal Cargo 16 strict validator vakasını atlamasız çalıştırır; native
kampanya ayrıca temiz exact kaynakla kaydedilir. Ayrıntılı çağrı/kabul
sınırı [ürün README'sindedir](../dogfood/zee-cekirdek/is-kuyrugu/README.md).

RFC/spec/tanı/ABI/API ve production graph değişmez; performans iş yükü,
compiler state ve RC fuzz/soak kabulü bu üründen türetilmez. Dogfood kayıt
defteri/K-164 karşılaştırması, günlük/backlog, ADR/kanıt/faz/güvenlik haritası,
README/ürünleşme kapıları ve kanıt özeti birlikte yenilenir.
