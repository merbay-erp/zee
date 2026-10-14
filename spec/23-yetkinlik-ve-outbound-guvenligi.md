# 23 — Yetkinlik ve outbound ağ güvenliği

Normatif kaynak: RFC-0024, ADR-031. Gerçekleme kaydı: K-127.

## Yetkinlik kümesi (TANIMLI)

Kararlı proje yazımları şunlardır:

| Yazım | Açtığı sınır |
|---|---|
| `dosya-okuma` | Proje kökündeki dosyayı okuma/listeleme/SHA-256 hesaplama |
| `dosya-yazma` | Proje kökündeki dosyayı atomik yazma/ekleme/taşıma/silme |
| `ağ` | Bildirilmiş origin'e outbound HTTP(S) |
| `yerel-ağ` | Private/loopback hedef ve explicit düz HTTP |
| `ağ-sunucusu` | Native loopback web sunucusu/adaptörü |
| `donanım` | Sensör ve ışık adaptörü |
| `web-oturumu` | Oturum, rol, çerez ve CSRF adaptörü |
| `kriptografi` | Parola/kriptografi adaptörü; K-210 eki 3 ile gizli değer şifreleme (`gizli_anahtar_değişkeni` ortam değişkeni, `zee1:` XChaCha20-Poly1305) |
| `veritabanı` | Bildirilmiş exact loopback PostgreSQL adaptörü |

`proje.dil` içindeki `yetkinlikler` ve `ağ_hedefleri` yoksa boş listedir.
Değerler tekil olmak ZORUNDADIR. `yerel-ağ` → `ağ`, `web-oturumu` →
`ağ-sunucusu` bağı ZORUNLUDUR. `ağ` ancak boş olmayan `ağ_hedefleri` ile ve
tersi birlikte bulunabilir. İhlal P015'tir.

`veritabanı`, bildirimsiz tek dosya geliştirici profilinde örtük açılmaz.
Proje hedefi, bağlantı sırrı ve migration kökü spec/25'teki üçlü bildirime
bağlıdır; outbound `ağ` allowlist'i yerine geçmez.

## Origin ve IP kuralları (TANIMLI)

Bildirim hedefi yalnız `http://host[:port]` veya `https://host[:port]`
origin'idir; yol, sorgu ve parça YASAKTIR. İstek URL'sinin yolu/sorgusu olabilir
ama şema+küçük-harfli ASCII DNS/IP+etkin port exact bildirime eşleşmek
ZORUNDADIR. Kullanıcı bilgisi, kontrol karakteri, ters bölü, sıfır port,
ayraçsız IPv6 ve başka şema YASAKTIR. K-210 ile `smtps://` (465, implicit TLS) ve
`smtp://` (587, STARTTLS zorunlu; düz metin yalnız loopback) şemaları aynı
exact origin kuralıyla kabul edilir; bildirimsiz geliştirici ağında yalnız
şifreli şemalar (https, smtps) kendiliğinden izinlidir. SMTP kimlik bilgisi
bildirimde değil `e_posta_bağlantı_değişkeni` ortam değişkenindedir.

Public hedef HTTPS olmak ZORUNDADIR. Düz HTTP bildirimi için `yerel-ağ`
ZORUNLUDUR. DNS çözümünden sonra bağlantıda kullanılabilecek bütün IP'ler
denetlenir: public unicast kabul edilir; loopback, RFC1918, CGNAT ve IPv6 ULA
yalnız `yerel-ağ` ile kabul edilir. Link-local/metadata ile IANA public olmayan
özel-kullanım veya IPv4-geçiş önekleri her profilde YASAKTIR. Yasak ve izinli
IP'yi birlikte döndüren DNS cevabı reddedilir.

Redirect izlenmez; ortam proxy'si kullanılmaz. Native istemci global ve DNS
çözümünde varsayılan 30 saniye son tarih, en fazla 64 KiB response header ve
8 MiB toplam zarf içinde body uygular. HTTPS sertifika/TLS doğrulaması rustls
backend'indedir. Bu sınırların ihlali runtime'da C018'e dönüşür.

K-139/ADR-036 ile aynı `AgHedefi` origin ayrıştırıcısı production web
proxy'nin `--web-proxy`, `Host`, `Forwarded host` ve unsafe `Origin`
sınırlarında da ZORUNLUDUR. Outbound ve inbound güven yüzeyleri ikinci bir
DNS/IPv6/port parser'ı taşıyamaz; web'e özgü HTTPS ve loopback değişmezleri
spec/12'de tanımlıdır.

## Compile ve runtime uygulaması (TANIMLI)

Ana program, bütün testler ve çağrılmayan işlemler dahil her gövdenin dış dünya
ihtiyacı yürütmeden önce çıkarılır. Eksik yetkinlik veya sabit outbound URL'nin
allowlist dışı origin'i kesin AST ifade aralığında T054'tür. Dinamik URL runtime
origin denetiminden geçmek ZORUNDADIR.

Proje çalıştırma ve denetleme manifest politikasını kullanır. `--güvenli`
yalnız proje-kökü dosya okuma/yazmayı açar. Bildirimsiz tek dosya geriye uyumlu
geliştirici profilidir; dosya ve adaptörler açık, outbound yalnız public
HTTPS, `yerel-ağ` kapalıdır. Her runtime IO çağrısı policy sarmalayıcısından;
native dosya/ağ/web adaptörü ayrıca aynı kapıdan geçer.

Proje dosya yolu göreli ve kök içinde olmalıdır. `..`, mutlak yol ve
canonical hedef/ebeveynin sembolik bağla kök dışına çıkması reddedilir. Bu
sözleşme eşzamanlı düşman yerel süreçlere karşı OS sandbox'ı değildir.

## Paket yetkisi (TANIMLI)

Her paket kendi `proje.dil` ihtiyacını bildirir. Paket yetkinlik ve origin
kümeleri ana uygulama politikasının alt kümesi olmak ZORUNDADIR; aksi P015'tir.
Uygulama izni süreçteki ortak üst sınırdır. Paket başına ayrı filesystem,
tenant veya süreç izolasyonu bu sürümde AÇIKTIR ve vaat edilmez.

## K-220 — Native başlıklı HTTP istek profili

TANIMLI: `adres yöntem başlıklar gövde süre ile HTTP isteğini dene`, native HTTPS GET/POST için Metin sözlüğü Sonucu döndürür. Adres/yöntem/gövde Metin, başlıklar Metin sözlüğü (boş sözlük kabul), süre TamSayı milisaniye ve 1..30000 aralığındadır. GET gövdesi boş olmalı; başarı sözlüğü `durum` HTTP durum metni ve `gövde` UTF-8 metnidir. 3xx/4xx/5xx yanıtları başarı taşımasıdır; ağ/süre/boyut/istek hatası C018 kodlu başarısız Sonuç olur. Eski GET kalıbı değişmez; yeni çağrı eylem transaction’ında T048 ile reddedilir.

ZORUNLU: Yeni istek exact origin ve DNS sonrası IP denetimi, global bağlantı bütçesi, TLS doğrulaması, kapalı redirect ve kapalı ortam proxy’sini korur. En fazla 64 başlık/toplam 64 KiB, gövde en fazla 8 MiB - 64 KiB kabul edilir; GET gövdesi, tekrar başlık, printable ASCII dışı değer, geçersiz HTTP adı ve Host/Content-Length/Transfer-Encoding/Connection/Proxy-Connection/Proxy-Authorization/Upgrade/Expect kullanıcı başlığı reddedilir. Başlık değerleri sensitive işaretlidir; adaptör hata ayrıntısı, URL, başlık ve gövde hata mesajına girmez. İstek süresi üst deadline’ın kalanına sınırlandırılır; mevcut üst deadline C020 davranışı sürer.

TANIMLI: `ad gizli ortam değerini okumayı dene` yalnız ana projenin `http_gizli_değişkenleri` listesinde kayıtlı adları Metin Sonucu olarak okur. Liste ağ yetkinliği, en fazla 16 tekil ad, ad başına 64 A-Z/0-9/_ karakteri ve değer başına 64 KiB ister. Standalone dosya veya bağımlı proje kendiliğinden izin vermez. Yeni HTTP ve gizli ortam işlemleri IO iz kaydında yürütülmez, iz export’u kapanır; replay desteklenmeyen işlem olarak kapalı başarısızlık döndürür.

TANIMLI: `metin HTTP JSON sözlüğünü okumayı dene` native HTTP profilinde bir JSON nesnesini Metin sözlüğü Sonucu yapar; alan değerleri kanonik JSON metnidir (bool true `true`, metin true `"true"`). Girdi en fazla 1 MiB, en fazla 128 üst alan ve serde derinlik sınırıyla işlenir; üst düzey tekrar anahtar ve trailing veri reddedilir. İç nesnelerde serde tekrar anahtar davranışı korunur. Yeni native HTTP/ortam/JSON adaptörü WASM’da açılmaz; desteklenmeyen işlem başarısız Sonuç’tur.

HTTP kabulündeki sentetik TLS kimliği yalnız localhost test Agent’ında kullanılır. Kaynak ve scanner kopyası immutable fixture blob’u ile aynı SHA-256 değerindeyken tek pkcs12-file directory fingerprint’i verilir; bu üretim TLS güven deposu veya sır kabul politikası değildir (ADR-091/GB-054).
