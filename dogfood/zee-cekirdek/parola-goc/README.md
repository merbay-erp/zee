# K-223 — İlk girişte legacy parola göçü

Gerçek ürün gereksinimi özel `zee-cekirdek` deposundaki
`docs/dil-talepleri/2026-10-14-zee-urunlesme.md` B7 maddesidir; exact talep
commit'i `cd5d12f0008620819e6167af5146e83503fd2b38`.
Etkilenen gerçek dosya `cekirdek/kaynak/rota_kimlik.dil` içindeki
`POST /giris-yap`; kimlik bağlama eylemi `cekirdek/kaynak/kimlik.dil`dedir.
Müşteri, sunucu, alan adı, hesap, parola veya gerçek özet kopyalanmamıştır.

[Minimal helper](giriste-yukselt.dil), mevcut K-208 doğrulaması, K-210
Argon2id üretimi ve K-163 eylem/parametreli SQL sözleşmesini kullanır. Yeni
compiler özelliği, grammar veya kriptografi API'si açmaz. Tablo ve hesaplar
yalnız anonim fixture'dır; üründeki schema/göç değildir.

Helper, doğru parola ve yalnız desteklenen `$2a$/$2b$/$2y$` önekinde yeni
Argon2id PHC özeti üretir. Güncelleme eski özetle karşılaştırmalı olduğundan
iki eşzamanlı girişin veya araya giren parola değişiminin yeni özeti
ezmesi engellenir. Etkilenen satır tam bir değilse hata döner. Modern özette
yeniden üretim ve UPDATE yoktur.

Başarılı Sonuç içindeki özet oturum bağlamanın beklenen özeti olmalıdır.
Göç ve oturum bağlama aynı dış `eylem` içinde yapılmalıdır; bağlama SQL'i
hesabın güncel özetiyle exact eşlik arar. Helper modern girişi tek başına
oturuma bağlamaz. Göçten sonra eski özetle bağlama denenmez. Hata Sonuç'u
dış eylemden de hata olarak döndürülür; düz Metin hata etiketi transaction'ı
geri almaz. İkinci faktör ve native giriş oran sınırı korunur.

[Kabul testleri](../../../compiler/tests/zee_cekirdek_urun_kabulu_testi.rs)
gerçek Zee helper'ını çalıştırır: üç bcrypt öneki, ilk/ikinci giriş, yanlış
ve bozuk parola, eski snapshot yarışları, SQL hatası, iç içe eylemde oturum
bağlama hatası, özet üretim hatası ve COMMIT belirsizliği. SQL/CAS modeli
hermetiktir; gerçek PostgreSQL eşzamanlılık veya canlı ürün kabulü değildir.
COMMIT belirsizliğinde C027 ve otomatik tekrar yoktur; sonucu ürün yeni
bağımsız okumayla uzlaştırır.

Hermetik `Commit::Red` açıkça backend-known ret/C021 modelidir; ham native
PostgreSQL ErrorResponse bu bilgiye sahip bir sonuç değildir. Native
adaptör bütün dış COMMIT hatalarında konservatif C027'yi korur. Tek bir
`ERROR/23514` fixture'ında bağımsız gözlemcinin rollback görmesi bütün
provider/extension durumlarına genellenmez.

K-242 / [ADR-103](../../../adr/103-native-commit-belirsizligi-kaniti.md)
ile [native koşucu](../../../scripts/parola-goc-ci.py) güncel
`zee-parola-goc-ci-2` şemasında yedi TLS ve dört owned loopback plaintext
vaka ister. Başarılı COMMIT kontrolü stdout'a ve bir sentetik SMTP
bildirimine ulaşır. COMMIT iletilmeden kesinti, uygulanmış COMMIT sonrası
ack kaybı ve iletilen deferred `ERROR/23514` yolları aynı C027, boş stdout
ve sıfır SMTP üretir. Her ek vakada bir UPDATE attempt, bir INSERT attempt
ve tek COMMIT; bağımsız DB bağlantısıyla eski/sıfır veya modern/tek
oturum/audit sonucu doğrulanır. [Proxy ve kabul sınırı](../../../docs/postgresql-commit-belirsizligi.md)
exact COMMIT completion + hata yokluğu + idle, bounded startup/frame,
absolute deadline ve owned cleanup ister.

Bu genişletmenin final temiz `316a313` kabulü 11/11, sıfır atlamayla geçti;
eski yedi vakalık kayıt tarihsel kalır. Plaintext yalnız owned loopback
test profiline aittir; public plaintext desteği, TLS fault interception,
canlı ürün login/göçü veya RC/uzak CI kabulü değildir.

Maliyet bu değişiklikte özelleştirilmez. Üretim, mevcut Argon2id varsayılanını
korur; bcrypt doğrulama maliyeti 4–14 ve parola 72 UTF-8 byte sınırındadır.
Hermetik deterministik tuz üretim güvenlik tuzu değildir. Production tuzu
mevcut OS rastgeleliğinden gelir. Canlı yükseltme, migration, rate-limit
kapasitesi ve ürün giriş duman testi ürün sahibinde açık kalır.
