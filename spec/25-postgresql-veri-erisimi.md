# 25 — PostgreSQL veri erişimi

Normatif kaynak: RFC-0026, ADR-060, ADR-062. Durum: **TANIMLI — K-163/F031 TLS ve sınırlı havuz profili**.

## Proje bildirimi

PostgreSQL kullanan proje `veritabanı` yetkinliğini ve şu üç alanı birlikte
bildirmek ZORUNDADIR:

```zee
veritabanı_hedefi "postgresql://127.0.0.1:5432/uygulama" olsun
veritabanı_bağlantı_değişkeni "UYGULAMA_DATABASE_URL" olsun
veritabanı_göçleri "göçler" olsun
```

Hedef kullanıcı/parola/query taşıyamaz; kanonik küçük ASCII DNS adı, IPv4,
`localhost` veya `::1` kabul edilir. Ortam değişkenindeki URL tek TCP hostla
exact hedefe eşleşmek ZORUNDADIR; `hostaddr` hostname doğrulamasını atlatamaz.
Loopback hedefte açık `sslmode=disable`, bütün hedeflerde açık
`sslmode=require` kullanılabilir; örtük `prefer` reddedilir. Migration yolu
relative ve proje içinde olmalıdır.

`sslmode=require`, `<BAĞLANTI_DEĞİŞKENİ>_TLS_CA_PEM` ortam değişkeninde PEM
kök sertifika ister. Sistem kökleri bu profilde kapalıdır; yalnız bildirilen
kök güvenilir sayılır. TLS en az 1.2 ve sürücü hostname doğrulaması ZORUNLUDUR.
CA boş veya 256 KiB'dan büyük olamaz.

## Bağlantı havuzu

Her Zee worker'ı en çok 4 PostgreSQL bağlantısı açar. Checkout bütçesi 2
saniye, boş bağlantı ömrü 30 saniye, azami bağlantı ömrü hedefi 300 saniyedir.
Aktif kira yarıda kesilmez; süresi dolan bağlantı bırakıldıktan sonra en geç
30 saniyelik bakım çevriminde emekliye ayrılır. Her checkout sağlık
kontrolünden geçer. Transaction tek kirayı BEGIN'den
COMMIT/ROLLBACK'e kadar tutar; transaction dışı sorgu kirayı işlem sonunda
havuza bırakır. Deployment toplam bağlantı bütçesi `worker sayısı × 4` olarak
ayrıca hesaplanmalıdır.

Havuz tükenmesi ve TLS/bağlantı kurma hatası C026'dır. Hata zinciri en fazla
dört neden ve 512 karakter taşır. Worker kapanırken boş bağlantılar kapanır;
aktif kira yarıda kesilmez ve son sahibi bıraktığında kapanır.

## Sorgular

- `"SQL" sorgusunu parametreler ile okumayı dene`, iki argümanlı typed
  intrinsic'tir. Parametreler `Metin listesi`, sonuç
  `Sonuç<List<Metin sözlüğü>>`dür.
- `"SQL" sorgusunu parametreler ile değiştirmeyi dene`, aynı bind kuralıyla
  `Sonuç<TamSayı>` döndürür ve yalnız eylem içinde geçerlidir.
- Parametreyi SQL metnine birleştirmek adaptör davranışı OLAMAZ. Her parametre
  extended-query protokolünde ayrı `TEXT` değeri olarak bind edilmelidir.
- Okuma sütunları non-null metin olmak ZORUNDADIR. Farklı PostgreSQL tipi
  `::text`, NULL olasılığı `COALESCE` ile sorguda açıkça çözülür.
- Veritabanı reddi `C025` Hata değeridir. PostgreSQL sağlıyorsa `sqlstate`,
  `kısıt` ve `tablo` Hata verisinde korunur.

Değişiklik application/state write etkisidir; GET/HEAD rotasında ve eylem
dışında reddedilir. İç içe eylem savepoint'tir. Aynı eylemde kalıcı dosya ile
PostgreSQL yazısı YASAKTIR.

Transaction dışında başlayan bir okuma, ilk sorgu bağlantı düzeyinde başarısız
olur ve sürücü client'ı kapalı olarak işaretlerse adaptör eski client'ı atıp
en fazla bir kez yeniden bağlanmalı ve aynı parametreli SELECT'i yeniden
çalıştırmalıdır. SQL reddi, açık transaction içindeki okuma, lock/stream/cursor
işi, ikinci başarısızlık, değişiklik ve COMMIT otomatik yeniden DENENEMEZ.
COMMIT sırasında bağlantı kaybı işlemin gerçekleşip gerçekleşmediğini
belirsiz bırakır. PostgreSQL adaptörü structured hata verisinde
`hata_sinifi=db.commit_unknown` taşır; transaction sınırı bunu kararlı C027'ye
çevirir. Mesaj belirsizliği, otomatik tekrar yapılmadığını ve uzlaştırma
gerektiğini açıkça bildirmelidir.

Eylem sınırındaki PostgreSQL bağlantı hatası client'ı geçersiz kılmalı;
transaction/savepoint durumu sonraki isteğe taşınmamalıdır. Web adaptörü C021'i
503'e çevirip worker'ı ayakta tutar. Aynı write kendiliğinden tekrar edilmez;
sonraki bağımsız read veya write yeni bağlantıyla başlayabilir. CLI/web dışı
çağrıda C021 normal çalışma tanısı olarak yayılmayı sürdürür.

COMMIT sonucu belirsiz olan dar sınıf C021 DEĞİLDİR: web'de C027 anlamlı 503,
CLI'da C027 tanısıdır. İstemci “commit olmadı” varsayamaz. Uygulama kararlı bir
iş anahtarıyla sonucu okumalı; kayıt varsa başarılı sonucu benimsemeli, yoksa
ürün politikasına göre yeni bir girişim başlatmalıdır. UNIQUE iş anahtarı aynı
yan etkinin iki kez oluşmasını engelleyebilir; bu genel otomatik retry izni
değildir.

## Migration

`dil göçür [proje]`, yalnız `NNNN_aciklama.sql` düzenli dosyalarını sürüm
sırasıyla tek transaction içinde uygular. Sürüm pozitif ve tekil, açıklama
küçük ASCII/sayı/alt çizgi olmalıdır. Kendi `BEGIN`, `COMMIT`, `ROLLBACK` veya
`SAVEPOINT` komutunu taşıyan migration reddedilir.

Uygulayıcı `_zee_gocleri` tablosunda sürüm, ad ve içerik SHA-256'sını tutar ve
transaction-scoped advisory lock alır. Aynı ağaç ikinci koşuda atlanır.
Uygulanmış dosyanın kaybolması veya aynı sürümün ad/içerik değiştirmesi C026 ile
fail-closed durur.

## Kaynak ve iz sınırı

Sorgu 64 KiB; parametre sayısı 100; tek parametre 64 KiB; sonuç 10.000 satır,
100 sütun ve 16 MiB; migration tek dosya 1 MiB, toplam 8 MiB sınırını AŞAMAZ.
Bu değerlerin tek sahibi `KaynakSinirlari.veritabani()`dir.

Okuma, değişiklik, değerler ve structured hata `zee-io-izi\t1` içine kanonik
sırada yazılır. Replay aynı SQL/parametreleri exact eşleştirir ve dış
veritabanına bağlanamaz.

Yeniden bağlanma karar matrisi
`compiler/tests/fixtures/postgresql-recovery-v1.tsv` ile sürümlüdür. K-163
gerçek PostgreSQL 16.11 kanıtında canlı backend sonlandırıldıktan sonraki ilk
transaction-dışı okuma, süreç yeniden başlamadan yeni backend'e bağlanmış ve
aynı sentinel satırını görünür kılmıştır. Yazı olumsuzunda backend kaybından
sonra otomatik tekrar veya satır oluşmamış; F027 worker survival'ı kapatmıştır.
F028 wire-level proxy'si COMMIT'i iletmeden kesilen durumda 0, COMMIT
ReadyForQuery yanıtı yutulan durumda 1 satır üretmiş; iki durumda da aynı
C027/503 görünmüş, worker yaşamış ve otomatik retry olmamıştır.

F031 saha profili pinned deney CA'sıyla gerçek hostname/CA reddini, dört
bağlantılık exhaustion sınırını, stale backend atımını, idle cleanup'ı,
300 saniye + bakım çevrimi sonunda lease dönüşünde lifetime yenilemeyi ve kontrollü worker
kapanışını kanıtlamıştır. Hermetik CI aynı havuz invariants'ını sahte manager
üzerinden zorlar; gerçek TLS koşusu açık ortam değişkenli saha testidir.

## Açık sınır

Managed-provider sertifika rotasyonu, birden çok worker'ın toplam bütçe provası,
async driver, PostgreSQL dışı veritabanı, genel ORM/schema DSL ve production
migration işletimi AÇIKTIR.
