# ADR-141 — Purpose-bound imzalı PDF stdio host (K-281/P11)

- **Durum:** doğrulama önerisi; mevcut dil ile gerçek signed request/reply, genel FFI değil
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

Model7 ve explicit native9 iki ayrı Cargo vakasının sahibidir. Actual9 ephemeral
Ed25519+native Zee devamı ve gerçek PDF oracles içerir; normal model7 paket
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
