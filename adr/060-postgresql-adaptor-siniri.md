# ADR-060 — PostgreSQL adaptör sahipliği ve güven sınırı

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-163, B-073, RFC-0026

## Karar

`veritabani_modeli` yalnız davranışsız hedef/bildirim/hata veri sözleşmesidir.
Proje ayrıştırıcısı ve runtime bu modeli tüketir; `postgresql` adaptörü
runtime'a ve proje davranış katmanına bağımlı olmaz. CLI bildirimi adaptöre bağlar. Böylece transport,
semantic yürütme ve ürün bildirimi ayrı sahiplikte kalır.

SQL ile değerler extended-query protokolünde ayrı taşınır ve bütün değerler
açık `TEXT` parametresidir. Okuma yalnız non-null `TEXT/VARCHAR/BPCHAR/NAME`
sütun kabul eder; ürün sorgusu diğer tipleri `::text`, NULL'ı `COALESCE` ile
açıklaştırır. Sorgu, parametre, satır, sütun, sonuç ve migration boyutları
merkezî `KaynakSinirlari.veritabani()` profilinden gelir.

PostgreSQL değişikliği yalnız eylem transaction'ı içinde yapılır. Dosya ve DB
yazısını tek eylemde birleştirmek yasaktır. Migration'lar advisory lock ve tek
transaction altında SHA-256 geçmişine bağlanır; migration içinde transaction
komutu fail-closed reddedilir.

Bağlantı recovery sahipliği adaptördedir fakat yalnız transaction dışı
idempotent okuma için geçerlidir. Kapalı sürücü client'ı atılır, tek kez yeni
client açılır ve aynı SELECT tekrar edilir. Transaction içi okuma, SQL reddi,
ikinci başarısızlık, yazı ve COMMIT fail-closed kalır. COMMIT bağlantı kaybı
“sonuç belirsiz”dir; adaptör otomatik tekrar yapamaz. Bu sınır
`postgresql-recovery-v1.tsv` karar matrisiyle korunur.

K-163/F028 ile belirsizlik string eşleşmesi değildir. PostgreSQL adaptörü
`db.commit_unknown` structured sınıfını, yorumlayıcı `CommitSonucuBelirsiz`
transaction sınıfını ve kullanıcı yüzeyi C027'yi taşır. Sürümlü IO izi sınıfı
exact korur; v1'in eski genel hata satırları geriye dönük okunur.

Gerçek IO sahibi transaction başlangıç/tamamlama/geri alma hatasında client'ı
yeniden kullanmaz. Yerel eylem bookkeeping'i temizlenir; web yorumlayıcısı
C021'i 503 isteğine dönüştürüp worker döngüsünü sürdürür. Bu yalnız süreç
survival ve sonraki bağımsız bağlantıdır; başarısız write'ın tekrarına izin
veren bir recovery değildir. K-163 F027 exact ürün kanıtı `00a659a` commit'idir.

K-163/F029 ürün protokolü DB metadata ile fiziksel dosyayı ayrı eylemlerde
tutar; ikisini tek transaction gibi göstermeye çalışmaz. Bu protokol gerçek
dizin çakışmasıyla C013 ürettiğinde web worker'ının sonlanması request-local
failure sınırını ihlal etti. Web adaptörü bu nedenle C013'ü ayrıntı sızdırmayan
503'e çevirir, istek taslağını geri alır ve worker'ı yaşatır; dosya yazımını
otomatik tekrar etmez. CLI tanısı değişmez. İlk saha reproducer'ı kayıtlı
`itwise-admin` ürünündeki `b3cee9a` commit'indedir.
Final PostgreSQL 16.11, worker-survival, restart sonrası uzlaştırma ve temizlik
kanıtı aynı ürünün `29d30c8` commit'indedir.

Bağlantı URL'si kaynakta bulunamaz. F031 öncesi adaptör yalnız exact loopback
hedefi ve `sslmode=disable` kabul ediyordu. DNS hedefi, pinned-CA TLS ve bounded
pool sahipliği ADR-062 ile ayrı davranış katmanına alınmıştır; saf
`veritabani_modeli` bu transport kararlarını çağırmaz.

`postgres-protocol 0.6.x` geçişli olarak `base64 0.22` kullanırken mevcut HTTP
istemcisi `base64 0.23` kullanır. `cargo deny` bu tek, gerekçeli sürüm
çoğulluğunu geçici olarak kabul eder; başka çoğulluklar yine reddedilir. İki
zincir ortak sürüme geçtiğinde istisna kaldırılır.

PostgreSQL okuma/değiştirme, sonuç ve structured hata dahil sürümlü IO izine
girer; replay dış veritabanına bağlanmaz.

K-163 ürün commit'i `43d04fc` stale-client arızasını buldu. Ardından gerçek
PostgreSQL 16.11 backend sonlandırma koşusunda tek read reconnect aynı süreçte
başarılı oldu; write olumsuzunda tekrar ve satır oluşmadı. Exact saha kanıtı
ürün deposundaki `78cce11` commit'indedir. F027 write bağlantı kaybında worker
survival'ı; F028 ise COMMIT öncesi kesinti ve uygulanmış COMMIT sonrası kayıp
yanıtı aynı C027 ile ayırmadan raporlamayı kapatır. Proxy deneyinde iki DB
durumu sırasıyla 0 ve 1 satırdır; ikisinde de retry yoktur. Production
TLS/pool ADR-062 ile kapanır; multi-process toplam bütçe deployment
sorumluluğudur. Binary medya, fiziksel
silme ve orphan tarama F030/RFC-0027/ADR-061 ile ayrı sahiplikte kanıtlandı;
PostgreSQL katmanı dağıtık transaction sözü kazanmadı.
