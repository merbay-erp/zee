# ADR-096 — Ekli e-posta ve MIME sınırı (K-232)

- **Durum:** kabul — native teknik dilim; canlı sağlayıcı/ürün kabulü açık
- **Tarih:** 14 Ekim 2026
- **Ürün:** zee-cekirdek
- **Karar zinciri:** RFC-0034, spec/23, spec/24, GB-058

## İhtiyaç ve minimal yüzey

Gerçek B2/P10 katalog PDF ihtiyacı `cd5d12f0008620819e6167af5146e83503fd2b38`
harici talebinden gelir. [Ürün README’si](../dogfood/zee-cekirdek/mime/README.md)
exact gerçek `rota_sm_katalog.dil` yolu/hash’i ve anonim
[reproducer](../dogfood/zee-cekirdek/mime/katalog-talep.dil) taşır.
Tek yedi atomik argümanlı intrinsic mevcut AST, Metin listesi ve Sonuç
türünü kullanır. HTTP provider helper yerel binary/Base64 dosya yüzeyi veya
public dosya URL’si varsayımı ister; seçilen dilim mevcut SMTP ayarına bağlanır.
Genel FFI, kuyruk/cron ve binary türü eklenmez. Rust `api::v1` export’u değişmez.

İlk beş argüman alıcı, konu, düz metin, HTML, yanıt adresi; altıncı ek yolu
Metin listesi, yedinci SMTP bağlantı Metin’idir. Boş metin HTML/Reply-To’yu,
boş liste ekleri kaldırır; boş bağlantı mevcut proje ortam ayarını seçer.
Bağlantı URL’si ortam ya da explicit değer yolundan gelir, AgHedefi exact
origin ve çözülmüş bütün IP kontrolleri her ikisinde uygulanır. Ekler yalnız
DosyaOkuma grant ve proje dosya sınırından geçen readonly handle ile okunur.
Native host mutlak/`..`/kök dışı symlink’i reddeder. PolitikaliIo aynı grant ve
lexical sınırı iç adaptörden önce uygular; fiziksel sınır host’un sorumluluğudur.
Host açmadan önce symlink olmayan normal dosyayı denetler; Unix readonly
O_NONBLOCK|O_NOFOLLOW|O_CLOEXEC ile writer-yok FIFO’yu bloklamadan reddeder.
Açılan handle tekrar normal dosya ve dev/inode kimliğiyle denetlenir;
Windows reparse-point açma ve iki handle’ın disk/file-id karşılaştırmasını
kullanır. Böylece leaf değiştirme yarışı kabul edilmez. Proje kökü mevcut
GB-016 gibi düşman yerel süreçlere karşı tam OS sandbox değildir.
Win32 helper Rust 1.93.1 ile std-only Windows metadata hedefinde derlenir;
bu makinede tam Windows Cargo build’i ring/Windows SDK `assert.h` eksikliğiyle
engellidir. Fiziksel Windows/minimum-OS kabulü açık kalır.
DosyaYazma kapalı kalabilir; hiçbir dosya içeriği/izin/mtime yazılmaz.

## MIME ve kaynak sınırı

`zee-smtp-mime-1` kontrollü profile göre HTML varsa alternative; ekler varsa
dış mixed zarfı kurar. Metin satırları CRLF’ye normalize edilir; binary ek
baytları değişmez. Her gövde parçası 76 sütun Base64’dür. Multipart ayraçları
OS CSPRNG kimliğinden gelir; `-` Base64 alfabesinde olmadığı için gövdeyle
çakışmaz. Unicode başlıklar mevcut UTF-8 karakter sınırından ayrılan en çok
45 bayt RFC2047 sözcüklerini CRLF+whitespace ile katlar; encoded-word ≤75’tir.
ASCII display-name yeni MIME zarfında quote/escape edilir. Dosya adı yalnız
tek basename, kontrolsüz/boş olmayan en çok 255 UTF-8 bayt; RFC2231 filename*
yüzde kodlamasıyla yazılır. En kötü 765 karakterlik parametre 998 SMTP satır
sınırına sığar; 78 sütun önerisi filename* satırında garanti edilmez.
CRLF/header/filename injection reddedilir. SMTP veri noktaları dot-stuff edilir.

Envelope ASCII, tek `@`, ≤254 bayt ve local-part ≤64 bayttır; SMTPUTF8 ve
quoted/multiple-address envelope bu profilin dışındadır. PDF/PNG/JPEG/WebP
etiketi uzantıdan seçilir, enum dışında MIME tipi girilemez. Bu whitelist
dosya içeriği doğrulaması, antivirüs veya PDF üretimi değildir.

Sayısal kaynak politikasının tek sahibi temel `KaynakSinirlari.eposta()`dir:
8 ek; ek başına 8 MiB; toplam 16 MiB; düz/HTML gövdenin her biri 512 KiB;
kodlu ileti 24 MiB; konu 512 bayt, ad 255, tarih 78 ve kimlik 48 bayt.
Tam ileti kurulduktan sonra ayrıca fiziksel satır ≤998 denetlenir.
Metadata kontrolüne ek olarak gerçek readonly read `take(sınır+1)` ile
sınırlıdır; büyümeyi/başarılı truncate’ı kabul etmez, ek adedi açmadan önce
reddedilir. Kodlama tahsisleri bu sınırlı girdiye bağlıdır, çağrı sonunda bırakılır.
Read bytecount ilk metadata uzunluğuyla eş olmalı; okuma sonrası uzunluk ve
mtime aynı kalmalıdır. Metadata zamanı restore eden düşman yazara/NFS’e karşı
atomik/coherent filesystem snapshot garantisi açık kalır.
SMTP global 64 bağlantı iznini RAII ile alır/bırakır.

## Taşıma, hata ve iz

Native MIME hazırlığı+SMTP için en çok 20 s ve varsa dış `içinde` son tarihinin
kalanı uygulanır. Hazırlıkta harcanan süre bütçeden düşer; her TCP read/write
ve TLS handshake kalan tek monotonic bütçeyi kullanır. DNS çözümü ve düzenli
dosyanın bloklayan OS read’i zorla iptal edilemez; bu işlemlerden sonra süre
yeniden denetlenir, dolan süreyle SMTP etkisi başlatılmaz. Bu nedenle sistem
DNS/filesystem’i için sert wall-time garantisi açık kalır.
SMTPS veya STARTTLS normal OS trust/hostname doğrulaması ve TLS≥1.2 kullanır.
Public plaintext fallback yoktur; STARTTLS’siz düz metin yalnız loopback’tir.
Yeni güven store/ortam proxy/redirect seçeneği açılmaz.

Başarı TamSayı 1; boyut, izin, başlık, TLS, transport ve ortam hatası C029
başarısız Sonuç’tur. Kullanıcı/URL/ek yolu/içerik/sunucu echo’su hata mesajına
girmez. SMTP DATA kabul yanıtı kaybolursa hata sonucu gönderilmedi kanıtı
değildir; otomatik retry yoktur. Bu geri alınamaz işlem eylemde T048 ile
reddedilir. Kayıt adaptörü inner çağrıyı yürütmez ve export’u kapatır;
replay de açıkça reddeder. Yeni serialized iz olayı/şeması yoktur.
WASM/playground trait default unsupported ile başarısız Sonuç döndürür;
bağımsız WASM target derlemesi native TLS/file modülünü yüklemez.

## Kabul ve etki incelemesi

Yeni `smtp_mime_testi` gerçek anonim reproducer’ı çalıştırır: typed lowering,
yanlış türler, T048/C029, network/read/path guards, maskelenmiş hata, iz
yürütme/export/replay, playground, outer deadline ve native SMTP binary PDF,
alternative/Reply-To kabulü. Codec testleri Unicode/max75/max998, injection,
limit, readonly load, dotstuffing, tek DATA, STARTTLS re-EHLO, trusted/untrusted
SMTPS ve yavaş TLS/greeting deadline kanıtıdır. Bağımsız Python email parser
Unicode konu/display-name/dosya adı ile exact binary PDF roundtrip’ini doğrular.
Sentetik CA/PKCS12 fixture yalnız test connector’ında kullanılır; leaf expiry
15.11.2027 ve exact fixture kayıt/tarama incelemesi aynı dalganın kapısıdır.

ADR-046 temel kaynak_sinirlari sahibi korunur; yeni native eposta/mime child
eposta sahibidir. `eposta→kaynak_sinirlari` tek yönlü bağımlılık bilinçlidir;
temel→adaptör veya yeni owner SCC açılmaz. Parser/runtime/LSP/native host mevcut
sahiplerindedir. Kaynak graph/SCC ve faz/yüzey/GB/spec kanıt kayıtları aynı
değişiklikte kapılardan geçer. Cargo graph/toolchain/SBOM/Rust facade değişmez.
ADR-045/049 sabit benchmark korpusu incelendi, bu additive SMTP kabulü tarihçe
satırı üretmez. Yeni cache/kalıcı state/uzun ömürlü tampon yoktur; ADR-069
soak yükü değişmez. Eski plain encoder/transport/deadline/failure semantiği
K-232’de korunur; bulunan legacy AUTH echo açığı ayrı güvenlik işidir.

P09 kuyruk/outbox/retry/cron, M365/Gmail canlı SMTP doğrulaması, özel ürün rota
değişikliği ve staging kabulü açık kalır. K-174 syntax gözlem penceresi
14.10.2026–11.11.2026 olarak bilinçli yeniden hesaplanır; V1 kapısı kapanmaz.
