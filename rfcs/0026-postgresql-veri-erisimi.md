# RFC-0026 — PostgreSQL veri erişimi

- **Durum:** **geçici kabul** (K-163 dogfood dilimi)
- **Tarih:** 3 Eylül 2026
- **İlgili kayıtlar:** K-163, B-073, ADR-060, spec/25
- **Gerçekleme:** `postgresql.rs`, `postgresql/havuz.rs`, PostgreSQL intrinsic'leri ve `dil göçür`

## Amaç

Çatlı/ITWISE Admin'in gerçek veri kalıcılığı ihtiyacını, SQL metni ile kullanıcı
değerini ayıran en küçük güvenli native yüzeyle karşılamak. F031 bu yüzeye
pinned-CA TLS ve sınırlandırılmış senkron havuz ekler. Bu karar genel ORM veya
bütün veritabanlarını destekleme sözü değildir.

## Dil yüzeyi

```zee
sonuç "SELECT slug::text FROM sayfalar WHERE slug = $1" sorgusunu yollar listesi ile okumayı dene
sonuç "INSERT INTO sayfalar(slug) VALUES ($1)" sorgusunu yollar listesi ile değiştirmeyi dene
```

Parametre listesi `Metin listesi`dir ve SQL'e birleştirilmez. Okuma
`Sonuç<List<Metin sözlüğü>>`, değişiklik `Sonuç<TamSayı>` verir. Hata değeri
`C025` kodunu, varsa `sqlstate`, `kısıt` ve `tablo` verisini taşır. Böylece
UNIQUE ihlali olan `23505` metin eşleştirmeyle kaybolmaz.

## Transaction ve migration

Değişiklik yalnız `eylem` transaction'ında geçerlidir. İç içe eylem savepoint
kullanır. Aynı eylemde kalıcı dosya ve PostgreSQL yazısı yasaktır; dağıtık
atomiklik sözü verilmez.

`dil göçür [proje]`, `NNNN_aciklama.sql` dosyalarını tek transaction ve
advisory lock içinde uygular. `_zee_gocleri` sürüm, ad ve SHA-256 kaydını
tutar. Aynı dosyanın ikinci koşusu idempotenttir; uygulanmış dosyanın silinmesi
veya değiştirilmesi fail-closed hatadır.

## Hedef, TLS ve havuz profili

Proje bildirimi sır içermeyen kanonik DNS/IP hedefi, bağlantı URL'sini taşıyan
ortam değişkeni adı ve proje içi migration klasörü taşır. URL tek TCP hostla
hedefe exact eşleşir; `hostaddr` reddedilir. Loopback geliştirmede açık
`sslmode=disable`, production profilde açık `sslmode=require` kullanılır;
`prefer` kabul edilmez. Require profili yalnız bağlantı değişkenine bağlı
`_TLS_CA_PEM` içindeki kökü güvenilir sayar, sistem köklerini kapatır, en az
TLS 1.2 ve hostname doğrulamasını zorlar.

Her worker'ın havuzu 4 bağlantı, 2 saniye checkout, 30 saniye idle ve 300
saniye azami lifetime hedefine sahiptir. Aktif lease kesilmez; süresi dolan
bağlantı bırakıldıktan sonra en geç 30 saniyelik bakım çevriminde emekliye
ayrılır. Checkout sağlık kontrollüdür;
transaction tek kirayı sınır boyunca tutar. Bu değerler bir dağıtım geneli
koordinatör değildir: toplam bütçe worker sayısıyla çarpılarak işletilir.

## Bağlantı kaybı ve tekrar sınırı

Gerçek K-163 backend-sonlandırma provası, daha önce kurulmuş senkron client'ın
stale kaldığını gösterdi. Bu nedenle yalnız transaction dışındaki salt-okuma
yolu, sürücü client'ı kapalı işaretlediğinde eski client'ı düşürür; tek yeniden
bağlantıdan sonra aynı parametreli SELECT'i bir kez daha çalıştırır. SQL hatası,
transaction içi okuma, ikinci başarısızlık, yazı ve COMMIT aynı mekanizmayı
kullanmaz.

Özellikle COMMIT cevabı kaybolduğunda veritabanı işlemi uygulamış olabilir.
Otomatik tekrar çift yan etki doğurabileceğinden adaptör
`hata_sinifi=db.commit_unknown` üretir; transaction sınırı bunu C027'ye taşır
ve kullanıcı yeniden denemeye yönlendirilmez. Havuz transaction dışı güvenli
read recovery'nin yeni bağlantı edinme sahibidir; cursor/stream ve kilitli
okuma bu ilk senkron profilin parçası değildir.

Transaction sınırındaki bağlantı hatası mevcut client'ı ve o bağlantıya ait
savepoint durumunu geçersiz kılar. Web adaptörü C021'i istek düzeyinde 503'e
dönüştürür ve dinleme döngüsünü sürdürür. Sonraki bağımsız istek yeni bağlantı
kurabilir; başarısız write hiçbir koşulda bu recovery tarafından tekrar
çalıştırılmaz.

## Kabul kanıtı

Parser/tür/etki, parametre ayrılığı, structured hata, IO trace/replay, hedef
doğrulama ve migration reddi testlenir. K-163 ürününde gerçek PostgreSQL ile
apply→skip, injection-benzeri parametre, rollback ve 23505 zinciri ayrıca
kaydedilmeden production yeterliliği iddia edilmez. Sürümlü recovery fixture'ı
read/write/commit tekrar politikasını korur; gerçek ürün provası stale read'in
process restart olmadan iyileştiğini ve write kaybında tekrar yapılmadığını
gösterir. COMMIT-sonucu-belirsiz dalı wire-level proxy ile iki uçta kanıtlanır:
COMMIT iletilmeden bağlantı kesildiğinde kayıt yoktur; COMMIT PostgreSQL'de
tamamlanıp ReadyForQuery yanıtı yutulduğunda kayıt vardır. İki durumda da HTTP
503/C027, sıfır otomatik retry ve yaşayan worker gözlenir. Sonraki bağlantıda
iş anahtarı okunur; aynı UNIQUE anahtarlı bilinçli girişim ikinci kayıt
oluşturamaz.

DB dışı binary medya yaşam döngüsü F030/RFC-0027/spec-26'da ayrı saga olarak
kanıtlandı; bu PostgreSQL RFC'si dağıtık atomiklik sözü kazanmadı.

F031 gerçek PostgreSQL 16.11 TLS cluster'ında doğru CA+hostname'i kabul etti;
yanlış CA ile DNS-only sertifikaya IP erişimini reddetti. Dört farklı backend
PID'de havuz doldu, beşinci checkout zaman aşımına uğradı; öldürülen stale PID
atılıp yenisi alındı. Idle bağlantı temizliği, 300 saniye + bakım çevrimindeki lifetime dönüşü
ve worker sonrası sıfır bağlantı ayrıca ölçüldü. Managed provider rotasyonu ve
çok-worker toplam bütçe provası hâlâ release işletim kanıtıdır.
