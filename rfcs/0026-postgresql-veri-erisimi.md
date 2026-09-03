# RFC-0026 — PostgreSQL veri erişimi

- **Durum:** **geçici kabul** (K-163 dogfood dilimi)
- **Tarih:** 3 Eylül 2026
- **İlgili kayıtlar:** K-163, B-073, ADR-060, spec/25
- **Gerçekleme:** `postgresql.rs`, PostgreSQL intrinsic'leri ve `dil göçür`

## Amaç

Çatlı/ITWISE Admin'in gerçek veri kalıcılığı ihtiyacını, SQL metni ile kullanıcı
değerini ayıran en küçük güvenli native yüzeyle karşılamak. Bu karar genel ORM,
uzak production TLS profili veya bütün veritabanlarını destekleme sözü değildir.

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

## İlk profil sınırı

Proje bildirimi yalnız sır içermeyen loopback hedefi, bağlantı URL'sini taşıyan
ortam değişkeni adı ve proje içi migration klasörü taşır. URL hedefle exact
eşleşmeli ve `sslmode=disable` olmalıdır. Bu yalnız yerel K-163 kanıt profilidir;
production TLS/uzak host ayrı dogfood ve güvenlik kararı ister.

## Bağlantı kaybı ve tekrar sınırı

Gerçek K-163 backend-sonlandırma provası, daha önce kurulmuş senkron client'ın
stale kaldığını gösterdi. Bu nedenle yalnız transaction dışındaki salt-okuma
yolu, sürücü client'ı kapalı işaretlediğinde eski client'ı düşürür; tek yeniden
bağlantıdan sonra aynı parametreli SELECT'i bir kez daha çalıştırır. SQL hatası,
transaction içi okuma, ikinci başarısızlık, yazı ve COMMIT aynı mekanizmayı
kullanmaz.

Özellikle COMMIT cevabı kaybolduğunda veritabanı işlemi uygulamış olabilir.
Otomatik tekrar çift yan etki doğurabileceğinden sonuç “belirsiz” olarak C025'e
taşınır ve kullanıcı yeniden denemeye yönlendirilmez. Pool, cursor/stream ve
kilitli okuma bu ilk senkron profilin parçası değildir.

## Kabul kanıtı

Parser/tür/etki, parametre ayrılığı, structured hata, IO trace/replay, hedef
doğrulama ve migration reddi testlenir. K-163 ürününde gerçek PostgreSQL ile
apply→skip, injection-benzeri parametre, rollback ve 23505 zinciri ayrıca
kaydedilmeden production yeterliliği iddia edilmez. Sürümlü recovery fixture'ı
read/write/commit tekrar politikasını korur; gerçek ürün provası stale read'in
process restart olmadan iyileştiğini ve write kaybında tekrar yapılmadığını
gösterir. COMMIT-sonucu-belirsiz dalı politika ve mesaj düzeyinde kanıtlıdır;
zamanlaması güvenilir biçimde enjekte edilmiş gerçek COMMIT kaybı henüz yoktur.
