# ADR-091 — Ürün HTTP POST, başlık ve gizli ortam sınırı (K-220)

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **Ürün:** zee-cekirdek
- **Karar zinciri:** RFC-0033, spec/23, GB-053

## Gerçek ihtiyaç ve minimal yüzey

[Exact harici talep ve dosya kanıtı](../dogfood/zee-cekirdek/http-post/README.md)
2d570601608592decc3a94d4ad061bdde1d84874 ürün commit’indendir.
POST /iletisim-gonder insan doğrulamasını yazamıyordu; Bearer portal POST’u
kabuk/Python’da kalıyordu. Eski GET cümlesi değişmez. Yeni genel HTTP çağrısı
mevcut Intrinsic AST, Sonuç ve Metin sözlüğü türlerini kullanır; yeni temel
veri türü ya da desteklenen Rust facade export’u eklenmez.

```zee
sonuç adres "POST" başlıklar gövde 5000 ile HTTP isteğini dene olsun
```

Adres/yöntem/gövde Metin, başlıklar Metin sözlüğü, süre TamSayı milisaniyedir.
GET ve POST; HTTPS; 1..30000 ms; GET gövdesi boş. Başarı yanıt sözlüğünde
`durum` (ondalık HTTP durum metni) ve `gövde` (UTF-8) taşır. HTTP 4xx/5xx ve
3xx transport yanıtıdır; uygulama durumu inceler. Ağ, süre, boyut ve geçersiz
istek hatası C018 kodlu başarısız Sonuç’tur; adaptör ayrıntısı/URL/başlık/gövde
hata mesajına girmez. Üst program deadline’ı mevcut C020 davranışını korur;
alt HTTP süresi üst son tarihin kalanından uzun olamaz. Eşzamanlı görev
adaptörden önce kardeşlere bir tur verir; senkron taşıma içinde yield yoktur.
Geri alınamaz çağrı eylem transaction’ında T048 ile reddedilir; yeni GET için
de bu konservatif sınır uygulanır, eski GET okuması aynı kalır.

## Kaynak ve yetki sınırı

Mevcut exact origin allowlist, DNS sonrası bütün IP’lerin denetimi, global
bağlantı bütçesi, rustls doğrulaması, kapalı redirect ve ortam proxy’sinin
kullanılmaması korunur. En fazla 64 başlık / toplam 64 KiB, istek ve yanıt
gövdesi en fazla 8 MiB - 64 KiB. Başlık adı HTTP token’ı, değeri printable
ASCII olmalı; case-insensitive tekrar ve Host/CL/TE/Connection/Proxy-Connection/
Proxy-Authorization/Upgrade/Expect kullanıcı girdisi reddedilir. Kullanıcı
başlıkları HeaderValue sensitive olarak işaretlenir; değeri Debug’a girmez.
Native CLI logger veya ham wire/body dökümü açmaz.

`"TURNSTILE_SECRET" gizli ortam değerini okumayı dene` Metin Sonucu’dur.
Yalnız ana projenin `http_gizli_değişkenleri` listesi yetki verir; bağımlı
proje veya standalone dosya kendiliğinden ortam erişimi kazanmaz. En fazla
16 tekil ad, ad başına 64 ASCII A-Z/0-9/_ karakteri ve değer başına 64 KiB;
`ağ` yetkinliği gerekir. Ortam değeri kaynağa, proje bildirimine veya
adaptör hatasına yazılmaz. Kendi programının açık `yaz`/dosya eylemiyle
paylaştığı değerler otomatik sır taint sistemi değildir.

## İz ve JSON

Sır taşıyabilen yeni HTTP çağrısı ve gizli ortam okuması iz kaydında
**yürütülmez**, export kapanır; replay adaptörü de başarısız Sonuç döndürür.
Başarı taklidi, gizli argüman/yanıt kaydı veya yeni şema içinde sır saklama
yoktur. Eski GET iz formatı/semantiği değişmez.

Ürün success+hostname kontrolünü yanıtı dosyaya yazarak yapmamalıdır.
`metin HTTP JSON sözlüğünü okumayı dene`, native HTTP profilinde 1 MiB,
128 üst alan, serde derinlik sınırı ve tekil üst anahtarlarla bir JSON nesnesi
okur. Alan değeri **kanonik JSON metni**dir: bool true `true`, metin true
`"true"`; hostname beklenen metnin `json metni` ile karşılaştırılır.
Bu mevcut Metin sözlüğü Sonucu’na sığar; genel heterojen JSON tipi eklenmez.
İç içe değerler kanonik JSON olarak taşınır; iç nesnelerde serde’nin tekrar
anahtar davranışı korunur, üst düzey tekrar reddedilir. WASM yeni native
HTTP/ortam/JSON adaptörünü açmaz; desteklenmeyen işlem başarısız Sonuç’tur.

## Kabul ve etki incelemesi

Gerçek Zee helper’ı geçerli/başarısız/yanlış hostname/tip taklidi/eksik alan/
bozuk JSON/503/taşıma hatasını ayırır. CLI izin listesi, iz export kapatma,
T034/T048 olumsuzları, gerçek sahte TLS sunucusu, sertifika reddi, timeout,
form gövdesi, Bearer ve 200/403/302 kanıtı aynı dilimdedir. Production CA
trust store değişmez; sentetik PKCS12 yalnız testtir. Canlı ürün rotası,
sunucu yükseltmesi, kayıt ve e-posta entegrasyonu ürün tarafında kalır.

ADR-046: parser, runtime, proje ve ağ adaptörü mevcut sahiplerindedir; yeni
sahip veya ters temel→adaptör kenarı yoktur. Dependency/SCC fixture etkisi
kapıyla doğrulanır. Cargo graph, toolchain, sürüm artefaktı/SBOM sözleşmesi
ve api::v1 değişmez. ADR-045/049 sabit benchmark korpusu ve eski tarihçe
incelendi/değişmez. HTTP Agent/JSON değerleri çağrı süresince yaşar; yeni
istemci cache’i, kalıcı durum veya büyüyen uzun ömürlü tampon yoktur.
İzin listesi sınırlı değişmez proje metadata’sıdır. ADR-069 etkisi bu sınırla
incelendi; bu iş K-219 soak yükünü değiştirmez. Kabul [K-220](../docs/incelemeler/k220/README.md).

K-220 / K-174: exact gerçek ürün talebiyle üç contextual kalıp eklendi; syntax gözlem penceresi 14.10.2026–11.11.2026 olarak bilinçli yeniden başladı. Core freeze ve insan/RC kapıları sürer; V1 tamamlandı olarak gösterilmez.

HTTP parser ve proje bildirim handler’ları ayrı alt modüldedir; politika/iz trait yöntemleri mevcut kimlik/iletişim örüntüsü gibi alan makrosunda toplanır. Yeni modüller bilinçli satır bütçesiyle mimari kapıya kaydedildi; kök handler büyümesi veya temel→adaptör ters kenarı yoktur.

## Gitsrv sentetik kimlik taraması (GB-054)

Gitsrv #28 Linux kalite/faz kapısını geçti, fakat directory sır taraması test server.p12 kimliğini pkcs12-file olarak reddetti. Kimlik yeni üretilmiş ve test parolası herkese açıktır. [Exact kayıt](../docs/sentetik-tls-fixture-v1.tsv) yalnız tek sabit fixture yolu/kuralı, immutable kaynak commit’i ve SHA-256 taşır. Tarayıcı öncesi immutable Git blob’u, kaynak dosyası ve kopya eş hash olmalıdır; başka PKCS12 veya değişmiş içerik istisna alamaz. Default kurallar, redact, ignore-gitleaks-allow ve tarihçe taraması korunur. Olumlu/olumsuz Python testleri Gitleaks kurulumu sonrasında koşar. Native TLS trust ve compiler kaynakları değişmedi.
