# ADR-141 — Purpose-bound imzalı PDF stdio host (K-281/P11)

- **Durum:** [S3](../docs/urunlesme-s3-kabul-20261015.md) (`66e4149e53451111d153f47283c219079a8d1cab`): amaç bağlı imzalı PDF stdio dokuz vaka ile dar yerel kabul; gerçek katalog/sağlayıcı, genel FFI, hostile PDF sandbox ve yayın açık.
- **Tarih:** 15 Ekim 2026
- **İlgili:** ADR-133/128, RFC-0012 taslak; GB-099

B3'ün PDF sayfa seçimi/birleştirme ve metin/görsel/font çıktısı K-275 fixed
backend ile vardır; Zee emitter'dan sonra ayrı operator komutu gerçek Zee
çağrısı değildir. Minimal çözüm yeni intrinsic/grammar yerine mevcut `diye sor`
ve `yanıt` üzerinden tek request/reply'dir. Program PDF receipt'ini aynı
çalışmada okuyup devam eder. Bu ayrı owned host protokolü, interpreter içi
native plugin loader, C FFI veya WASM host ABI genişlemesi değildir.
RFC-0012'nin FFI taslağı gerçeklenmiş sayılmaz; normatif dil spec'i değişmez.

PDF manifest/trust purpose'ı `zee-pdf-catalog-stdio-1` ile ayrı yetkidir.
Compiler release imzası/publisher listesi PDF kodu çalıştırmaya devredilmez.
Mevcut Ed25519 verifier ve selector doğrulama işlevi unchanged byte snapshot'tan
kullanılır; kaynak/asset/verifier/CLI private bytes ile source TOCTOU daraltılır.
Exact paket envanteri/hashleri doğrulanmadan backend yüklenmez/çalıştırılmaz.
Hostile sameUID/root veya imzalı kötü niyetli Python kodu sandboxlanmaz.
Operatör güvenilir publisher ve exact backend runtime'ını açıkça seçer.

[Host rehberi](../docs/pdf-plugin-host.md) exact şema/boyut/deadline/fresh output,
known receipt ve single invocation sözleşmesini taşır. Mevcut Supervisor yalnız
kendisinin çıkardığı liderleri WNOWAIT ile cleanup sonuna kadar tutar. Child
stdin'i yeni genel inherited-FD API'si değildir: sabit bootstrap yalnız host'un
yeni pipe read ucunu dup2 yapar ve private CLI'yı exec eder. Cancel flag ve
finally cleanup vardır; SIGKILL/powerloss altında geçmiş cleanup icat edilmez.

PDF üretimi geri alınmaz. Backend başladığında ret `unknown`; dosya receipt'i
known-produced fakat Zee devamı doğrulanmadığında `produced_client_unconfirmed`
olarak kalır. İkinci çağrı yoktur; unknown status/double invocation hata sonrası
output silinmez ve otomatik yeniden deneme yapılmaz. Fresh partial output ve
çok dosyalı fsync belirsizliği K-275 sınırıdır, false rollback/atomicity yoktur.

Stdlib9 ve explicit native9 iki ayrı Cargo vakasının sahibidir. Actual9 ephemeral
Ed25519+native Zee devamı ve gerçek PDF oracles içerir; normal stdlib9 paket
bağımlılığı/native compiler istemez. İlk bundled Python'daki eksik waitid
preflight adayı hatası cleanup kanıtı kaybetti: ilk failed receipt korunur,
yabancı PID adoption yapılmaz ve sonraki başarı geçmişi kapatmaz. Host artık
capability'yi ilk Popen'dan önce reddeder. İkinci runner stderr toplama ret kaydı
da ayrı korunur. Son kaynak koşusu ve root Cargo/faz/ledger/freshness kapıları
olmadan finalclean veya tam P11 tamamlandı denmez.

Production Rust/grammar/public API/dependency/katman/SCC değişmez; yeni compiler
bug regression kaydı yoktur. Yeni named host schemas/profil ve CLI yüzeyi
ADR-064 fixture'a; GB-099 bilinçli güven sınırı ledger'a aynı atomik dalgada
kaydedilir. RFC/spec norm/FNV, tanı kataloğu, benchmark ADR045/049 geçmişi
bilinçli değişmez. Gerçek ürün/adoption/P08/publisher/lisans/platform ayrıca açıktır.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.

## K-299 — Kabul betiğinin erken iptal temizliği

Üretim host zaten cooperative handler/finally kullanır; bu kararın üretim güven
sınırı değişmez. Native kabul betiğinde handler ilk owned komuttan önce kurulur,
setup/keygen finally kapsamına alınır. Fresh0700 dizinin held dirfd'si ve exact
iki isim yarıda kalmış keygen dosyalarını da temizler; tamamlanmış inode kimliği
korunur, private key baytları veya hashleri kanıta alınmaz. Yalnız başarısız host
çağrısının dış grace'i10 saniyedir; normal komut1 saniye kalır. Dış kampanya
sahibi nested cleanup için ayrıca yeterli pay ayırır.

İki yeni bounded stdlib vaka ile toplam9 geçti; bunlar gerçek Ed25519/PDF native9
kabulü değildir. GB-111 düşük/açık/K-299; yeni exact native9 ve ortak kapılar
beklenir. [Kayıt ve dosya/koşu hashleri](../docs/kabul-kosuculari-yasam-dongusu-20261016.md)
bu bekleyişi ve ayrı service-fault dilimini taşır. S3 kanıtı eski kaynağında kalır.
