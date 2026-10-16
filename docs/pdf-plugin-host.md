# İmzalı PDF stdio host protokolü — K-281

Bu araç mevcut Zee programının PDF isteği gönderip aynı çalışmada sonucu almasını
sağlar. `golden/pdf-plugin/ana.dil`, typed job sözlüğünü JSON'a çevirir ve mevcut
`diye sor` ile tek satır yazar; mevcut `yanıt` değerindeki receipt'i denetleyerek
`PDF_PLUGIN_DONE` ile devam eder. Operator tarafından ayrı çalıştırılan eski
K-275 JSON emitter/backend zincirinin yerine gerçek request/reply vardır.

Bu **owned host protokolüdür**; genel native FFI, interpreter plugin loader veya
RFC-0012'nin gerçeklenmesi değildir. WASM ABI3 yalnız playground sözleşmesidir.
Yeni compiler/grammar, public Rust API veya compiler bağımlılığı eklenmez.
[ADR-141](../adr/141-imzali-pdf-stdio-host.md) güven sınırını kaydeder.

## Paket ve güven

Operatör local/private paketi ve explicit trust dosyasının SHA256'sını seçer.
Paket yalnız `plugin.json`, `plugin.json.zee-imza`, `backend.py`, `assets/` içerir.
Asset'ler K-275'in base.pdf/font.ttf/product.png ve iki özgün font notice'ıdır;
manifestte exact altı dosyanın SHA256'sı bulunur. Her dosya nofollow/regular,
tek-link ve en çok1MiB; manifest16KiB, imza64KiB, trust16KiB'dır. Root/assets
dizinleri mevcut kullanıcıya aittir ve group/world writable değildir. Streaming
exact inventory ilk fazla/yinelenen girdide durur; recursive discovery yoktur.

`plugin.json` exact `schema,purpose,libraries,files` alanlarını taşır. Schema
`zee-pdf-plugin-1`, purpose `zee-pdf-catalog-stdio-1`; backend paket sürümleri
ReportLab4.4.9/pypdf6.10.0/Pillow12.3.0'dır. Trust exact
`schema,purpose,publisher,manifest_sha256,python_sha256` alanlarını içerir;
`schema=zee-pdf-plugin-trust-1` ve aynı PDF purpose gerekir. Compiler release
catalog veya onun publisher kaydı otomatik PDF yetkisi değildir.

Mevcut `surum_artefakti imzala/dogrula` Ed25519 biçimi ve selector'ın exact
`verify_signed_bytes` işlevi yeniden kullanılır. İmza domain'i `plugin.json`
adına; trust ise manifest hash'i ve ayrı publisher kimliğine bağlıdır. Verifier
ikilisi explicit SHA256 ile okunup private snapshot'tan yürür; helper'ın süreç
çağrısı mevcut Supervisor retained-owner ve bounded capture yoluna bağlanır.
İmzalı kod güvenilir yerel Python kodudur: imza zararsızlık veya OS sandboxı
kanıtı değildir. Üretim anahtarı, publisher onayı, public dağıtım veya key store
kurulumu bu profilden çıkmaz. Testler yalnız ephemeral anahtar üretip siler.

## Çağrı

```sh
python3 -I -B scripts/pdf-plugin-host.py \
  --framework-root /absolute/zee \
  --package /private/signed-pdf-package \
  --trust /private/pdf-trust.json --trust-sha256 EXACT_TRUST_SHA256 \
  --dil /absolute/dil --dil-sha256 EXACT_BINARY_SHA256 \
  --verifier /absolute/surum_artefakti --verifier-sha256 EXACT_VERIFIER_SHA256 \
  --python /absolute/exact-backend-python \
  --expected-source EXACT_SOURCE_SHA \
  --source /absolute/zee/golden/pdf-plugin/ana.dil \
  --output /private/fresh-pdf-output
```

Output parent mevcut kullanıcıya ait private0700 dizindir; output önceden var
olamaz. Paket hazırlama örneği `scripts/tests/pdf-plugin-native.py` içindeki
explicit altı dosya/manifest/signature/trust zinciridir; bu test üretim güven
anahtarı dağıtmaz. Mevcut backend ve asset'ler kopyalanır; job metadata executable,
argv, path, shell, URI veya font seçemez.

Host sistem Python'unda POSIX waitid/WNOWAIT gerekir; eksikse **spawn öncesi** ret
verir. PDF paketlerini taşıyan ayrı `--python` executable hash'i trust'a bağlıdır;
-I/-B ve temiz ortamla çağrılır, exact dependency sürümleri backend tarafından
PDF işlenmeden denetlenir. Bundled PDF runtime'ının süreç-sahipliği API'sini
sağladığı varsayılmaz. İndirme/kurulum/PATH değişikliği yoktur.

## Framing, çıktı ve belirsizlik

İlk stdout satırı en çok16384B JSON'dur: exact `schema,job`, request schema
`zee-pdf-plugin-request-1`, `job` UTF8 JSON Metin'dir ve en çok8192B'dır. Sonrası
operator seçtiği backend'in mevcut job/asset/content sınırlarıdır. Host üç PDF'yi
üretir, `zee-owned-pdf-receipt-1/status=produced` ve exact üç dosyanın
ad/boyut/hash eşliğini yeniden okuyarak doğrular. Receipt en çok4096B ve tek
satır olarak Zee stdin'ine verilir. Zee yalnız `PDF_PLUGIN_DONE\n` ile sonlanırsa
host `zee-pdf-plugin-result-1/status=completed` verir. İkinci invocation, bozuk
framing, unknown status veya başka epilogue başarısızdır; ikinci backend çağrısı
yapılmaz. PDF üretildi ama istemci devamı doğrulanmadıysa sonuç
`produced_client_unconfirmed` olur, otomatik retry yoktur.

Host request/reply için tek60sn monotonik bütçe, verifier için en çok15sn,
backend için20sn, BuildJSON sorgusu için10sn ve bounded capture kullanır.
Read/EOF/frame dönüşlerinde son tarih tekrar denetlenir. Preflight bounded dosya
hashleme ve platform import/CPU çalışması hard toplam wall-time garantisi değildir.
SIGTERM/INT yalnız iptal bayrağı kurar; sahipli cleanup finally'de çalışır.
Supervisor API'nin held WNOWAIT lideri grup TERM/KILL temizliğinden önce reap
edilmez. PID keşfi/adoption yoktur. Host SIGKILL/powerloss altında finally garantisi
verilmez; dış operator belirsizliği yeniden oynatma izni sayamaz.

Hata fixed JSON/exit2 ve `not_started`, `unknown` veya
`produced_client_unconfirmed` effect sınıfıdır; raw body/private path/anahtar
basılmaz. Backend başladıktan sonraki ret, dosya/efekt yokluğu kanıtı değildir.
Fresh outputta kısmi veya tamam PDF kalabilir; publication çok dosyalı atomik
transaction değildir ve host böyle bir dizini hata diye silmez.

## Kanıt ve açıklar

Stdlib9 normal test; ayrı explicit ignored actual9 kampanyası gerçek Ed25519,
Zee request/receipt devamı, PDF sayfa/metin/font/görsel ve purpose/key/signature/
request schema/size/runtime/double invocation/unknown status negatiflerini yürütür.
Son iki negatifte signed test backend counter'ı tam1 olur; bunlar keyfi plugin
kodunun sandboxlandığı iddiası değildir. Normal stdlib vakaları compiler çalıştırmaz; iki lifecycle vakası inert key
dosyaları ve sonlu owned Python çocukları kullanır. Native wrapper gerçek
CLI/lexer/parser/çözümleyici/tür/HIR/runtime ve IO kullanır.
İki libtest ile alt dokuz senaryo unique test toplamına eklenmez.

K-275 fixed trusted PDF/asset sınırı korunur. Keyfi saldırgan PDF, form/imzalı
PDF koruma, genel extension ABI, gerçek ürün/catalog entegrasyonu, publisher
kararı, P08 ve fiziksel/diğer platform kabulü açık kalır. Kaynak değişmiş yeni
K-281 için önceki clean-S kampanyaları yeni finalclean kabulü değildir.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## K-299 — Kabul koşucusunun iptal/kapanış gereği

Native kabul betiği ilk owned komuttan önce handler kurar; keygen dahil setup
finally kapsamındadır. Fresh held directory'nin exact iki key adı tamamlanmamış
keygen sırasında da temizlenir; private key baytı/hash'i kaydedilmez. Üretim host
zaten cooperative'dir. Başarısız host komutunun dış kapanış payı10 saniyeye çıkar;
normal komutun1 saniyelik değeri değişmez. Onu başlatan özgün dış owner nested
cleanup için en az45 saniye TERM payı ayırır; mevcut direct watchdog330 saniye
kullanır. Kısa grace'li ara wrapper eklenmez; shutdown payı iş timeout'undan ayrıdır.

Stdlib9 geçti; kaynak `8178920e03fed782a817d1864b8280fcb80812a3` üzerindeki ayrı native sonuç:
temiz C2 üzerinde 9/9 iç vaka ve bağımsız PDF içerik/sayfa/font/görsel incelemesi geçti. C2 öncesi gate4: 13 adım geçti; tam faz 998/1023 geçti, 0 başarısız, 25 atlanan Temizlik hatası/iptal
passed sayılmaz. [GB-111 ve actual kabul kaydı](kabul-kosuculari-yasam-dongusu-20261016.md)
durumu kapalı; toplu commit öncesi son belge kapısı gerekli olarak, exact source/tree ve özgün owner kapanışına
bağlar. Eski S3 veya K-298 kabulü yeni kaynak sonucu değildir.