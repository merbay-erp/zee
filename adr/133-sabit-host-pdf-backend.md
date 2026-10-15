# ADR-133 — Sabit owned host PDF backend (K-275)

- **Durum:** kabul; temiz S native Zee emitter+fixed hostPDF8 kabulü var; fullP11/ürün/platform açık
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** B3/P11, GB-091

## İhtiyaç ve minimal karar

Aktif zee-cekirdek ürününün cd5d12f0008620819e6167af5146e83503fd2b38 B3 talebi
PDF sayfa çıkarma/birleştirme ve metin+görsel+font üretimi ister. Talep dosyası
hash'i 694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7,
85–90 satırlarının LF-normalize dilim hash'i
6f55875cd6e8ba8d354088875f01895fd73fea5d156241b5266e9cfe2574f5bc'dir.
[Anonim teknik örnek](../golden/pdf-host/README.md) ve bitişik ürün köken
metadata'sı özel gövde/marka/müşteri verisi kopyalamaz. Mevcut gorsel.rs
JPEG/PNG/WebP küçültür; MIME hazır PDF dosyasını yalnız readonly taşır.

Önce yeni compiler intrinsic/grammar/dependency/FFI yerine açık host sınırı
seçilir. Mevcut Zee Metin sözlüğü mevcut JSON serializer ile sabit iş verisini
yazar; ayrı Python host sahibi sabit ReportLab Canvas ile sayfa üretir ve
pypdf ile owned temel sayfaları seçip ekler. Shell/HTML/JS/browser/URI motoru,
kullanıcı font/yol/SQL/callback veya outbound çağrı eklenmez. Resmî imzalı
plugin/FFI ya da Zee içinden backend çağrısı sunulmaz. Mevcut signed compiler
catalog yetkisi farklı plugin yetkisine sessizce genişletilmez.

## Kaynak, çıktı ve güven sınırı

Exact backend sürümleri ReportLab4.4.9, pypdf6.10.0, Pillow12.3.0'dır;
parser çağrısından önce doğrulanır. Eksik/farklı sürüm fixed hata verir;
kurulum/indirme/fallback yoktur. Job8192B, exact7key/Metin, tr/en, iki sabit
sayfa seçimi ve tek ekleme konumu; title80/product100/description200 karakter.
Kontrol/surrogate/BMP dışı karakter, fontta olmayan glyph ve sığmayan satır
reddedilir. Kodda üç asset SHA'sı sabittir; normal nofollow dosya, en çok1MiB
ve handle/path inode-size-mtime kontrolünden sonra immutable bytes okunur.
Her çıktı bounded buffer<=1MiB, en çok4sayfa; mevcut dizin/dosya ezilmez.

Parser'a yalnız hash'i önceden bilinen owned PDF/font/PNG gider. Byte cap
keyfi PDF decompression/recursive object saldırısını çözmez: bu profil hostile
PDF sandbox'ı değildir. Kullanıcı PDF'i, şifreli/imzalı PDF, AcroForm,
JS/URI/annotation/attachment, PDF/A/UA/tagging ve gerçek katalog ölçeği açık
kalır. Korunan sayfa content operators ve iki render piksel örneği eşleşir;
bütün PDF baytları, metadata/object identity veya dijital imza korunuyor denmez.

Yeni0700 output dizini/exclusive dosya yazımı vardır. Çok dosyalı atomik
publication yoktur; fsync/yazma hatasında yeni dizinde kısmi dosyalar kalabilir,
başarı receipt'i oluşmaz ve yeniden deneme başka fresh dizin ister. Hata
fixedJSON/exit2; başarı yalnız sabit dosya adı/boyut/hash verir. Ham body,
private path veya credential yazılmaz. Aynı UID/root hostile operatöre karşı
OS sandbox veya hard wall-time/RSS sözü verilmez. Dış kabul koşucusu retained
owned command deadline/group cleanup sınırını ayrıca korur.

## Font kökeni ve lisans ayrımı

DejaVu Sans Book2.37 exact757076B/SHA7da195a74c55bef988d0d48f9508bd5d849425c1770dba5d7bfc6ce9ed848954
asset'i değişmedi. İç nameID0 telif ve nameID13 tam Bitstream Vera+Arev metni
[assets](../golden/pdf-host/assets/LICENSE-DejaVu.txt) yanında özgün Unicode
metni UTF8 olarak saklanır; copyright ve bildirim hashleri bitişik
[FONT-PROVENANCE](../golden/pdf-host/FONT-PROVENANCE.json) içindedir.
Kurulu conda package hash/URL'si ve name-table çıkarım encoding zinciri korunur;
paketin BSD metadata etiketi bu tam özgün metnin yerine geçmez. Bu metadata
makineye özgü home yolu/runtimedep içermez. Proje/publisher lisansı kullanıcı
kararı olarak ertelenmiştir; bu üçüncü taraf notice'ı atlama gerekçesi değildir.
Diğer bağımlılıkların ürünle dağıtımı için genel lisans/dağıtım onayı verilmez.

## Test ve açık kabul

Normal Cargo wrapper yalnız stdlib/POSIX8 modelini yürütür; global veya Codex
bundled PDF bağımlılığı varsaymaz. Ayrı explicitignored wrapper operatörün
mutlak Python yolu ve exact üç backend dependency'siyle hostPDF8'i yürütür.
Eksik runtime pass/skip'e çevrilmez. İki wrapper'ın birincil fazı IO, ek kapsamı
yoktur: wrapper'lar Zee compiler çalıştırmaz. Golden .dil otomatik olarak
mevcut golden_testi'ne kaydedildi diye varsayılmaz.

Ayrı zorunlu final ortak kohort, temiz exact kaynak S'nin gerçek ikilisiyle
ana.dil'i çalıştırır, bounded JSON'u job.json beklentisine eşler, aynı S/build/
binary/helper/asset begin-end pinlerini korur; ardından explicit backend8'i
çalıştırır. Bu dış native pipeline lexer/parser/çözümleyici/tür/HIR/runtime/CLI
ve IO kullanır; Cargo wrapper'a gerçekleştirmediği fazlar yazılmaz. Adaydaki
kirli/tarihsel binary koşuları temiz S kabulüne dönüştürülmez. Root full fazı,
kanonik sayaçlar ve bağımsız source+asset-notice incelemesi ayrı gerekir.

P11 kısmi kalır: B3'ün 'katalog birleştirme Zee'den çağrılır' ürün kabulü,
resmî signed extension/host invocation seçimi, canlı katalog entegrasyonu,
P08 tam anonim korpus, fiziksel platform ve finalclean kampanya açık kalır.
Genel 900/1800/ASAN/Miri/kurulum/SMTP sağlayıcı kabulü ileri sürülmez.

## Bilinçli etki incelemesi

Yeni compiler/src/production Rust sahibi/kenar/dependency/grammar/tanı/API yok;
ADR046/SCC/facade/semantic/core source beyanı, compiler bug regression veya
normspec/FNV güncellemesi gerektiren compiler değişikliği yapılmadı. Backend
kararı ADR düzeyindedir; yeni RFC/spec normu önerilmez. Yeni host schema/profil
K167 envanterine exact kaynak marker ve giriş0.8.0-dev ile eklenir; mevcut
kayıtlar değiştirilmez. Golden tasarım örneği dogfood/P08 kabulü değildir.
ADR045/049 benchmark/soak tarihçesi yazılmaz. Rehber/backlog/plan/releases/
kapılar/günlük aynı atomik dilimde kısmi kabul sınırını taşır; gerçek sayaç ve
faz raporu yalnız root'un araçlarıyla yenilenir.

K-275 Git taşıma sınırı: base.pdf binary, özgün LICENSE-DejaVu.txt ise
-text ve yalnız bu dosyada blank-at-eol istisnasıyla tutulur. Lisansın
özgün sondaki boşlukları silinmez; index blob ve disk SHA-256 eşliği ayrıca
denetlenir. Bu dar metadata düzeltmesi runtime/fixture baytını değiştirmez.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

S common kohortunda gerçek cleanS Zee ikilisi golden ana.dil’den bounded job JSON üretti; fixedjob equality sonrasında exact ReportLab4.4.9/pypdf6.10.0/Pillow12.3.0 backend8 geçti. Font/notice/asset pinleri korundu; backend8 hostPDF kabulüdür, compiler wrapper’a çalıştırmadığı faz yazılmaz. Zee içinden signed plugin invocation/livecatalog/P08/physicalplatform/proje lisansı açık.

[Güncel dar kaynak kanıtı](../docs/urunlesme-final-kabul-20261015.md).
