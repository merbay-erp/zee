TÜRKÇE PROGRAMLAMA DİLİ
Uçtan Uca Master Proje Dokümanı
Dil • Derleyici • Runtime • Standart Kütüphane • Paket Ekosistemi • Tooling • Eğitim
“Türkçe düşün. Türkçe yaz. Makine kesin olarak anlasın.”
Sürüm: 1.0    |    Tarih: 31 Ağustos 2026    |    Çalışma adı: Belirlenecek    |    Geçici uzantı: .dil
Vizyon: İlkokuldaki bir çocuğun başlayabileceği, profesyonelin bırakmak zorunda kalmayacağı Türkçe genel amaçlı programlama dili.

# 1. Yönetici özeti
Bu proje İngilizce programlama dillerinin anahtar kelimelerini Türkçeye çeviren bir katman değil; Türkçenin doğal akışına göre tasarlanmış deterministik bir programlama dili ve onun eksiksiz ekosistemidir. Ekosistem; derleyici, runtime, standart kütüphane, paket yöneticisi ve registry, formatter, test sistemi, LSP, debugger, dokümantasyon üreticisi, web playground, editör eklentileri ve eğitim materyallerini kapsar.
Temel amaç İngilizce ve sembol bariyerini azaltırken profesyonel gücü korumaktır. Kullanıcı temel programlarda süslü parantez, noktalı virgül, &&, ||, != gibi işaretlerle başlamak zorunda kalmamalı; girinti ve kontrollü Türkçe kalıplarla kod yazmalıdır. Dil bir çocuk oyuncağı değil, çocukların da kullanabileceği ciddi bir genel amaçlı dil olacaktır.
Dil AI ile yorumlanmayacaktır. AI kod üretebilir, açıklayabilir veya hata çözümüne yardım edebilir; programın anlamını yalnız lexer, parser, type checker ve compiler belirler. Uzun vadeli sembolik hedef self-hosting’dir: derleyicinin önemli bölümünün kendi Türkçe kaynak kodundan kendisini derleyebilmesi.
# 2. Manifesto ve değişmez ilkeler
Türkçe-first: İngilizce anahtar kelime zorunluluğu yok.
Çeviri dili değil: if→eğer, function→fonksiyon makyajıyla yetinilmeyecek.
Deterministik: aynı geçerli kaynak tek AST ve tek semantik anlam üretir.
AI semantiğin parçası değildir; compiler hiçbir cümleyi tahmin etmez.
Noktalama minimum, belirsizlik sıfırdır; girinti blok yapısını belirler.
Statik tür güvenliği + yerel tür çıkarımı birlikte kullanılır.
Null-safety, kaynak güvenliği ve structured concurrency güvenli varsayımlardır.
Hata mesajları Türkçe, öğretici ve eyleme dönüktür.
Çocuk dostudur ama oyuncak değildir.
Cloud, LLM veya tek ticari sağlayıcı zorunlu değildir; offline toolchain mümkündür.
Tooling ürünün parçasıdır: formatter, LSP, test, paket, docs, debugger.
Çekirdek küçük ve kararlı; kütüphane katmanı daha hızlı evrilir.
Self-hosting uzun vadeli bağımsızlık hedefidir.
# 3. Hedef kullanıcılar ve başarı ölçütleri
8–14 yaş: dakikalar içinde ilk çalışan program; oyun, robot veya sensör projesi.
Lise/üniversite: algoritma, veri yapıları, dosya, ağ, test ve hata yönetimi.
Profesyonel: production servis, CLI, IoT, FFI, concurrency ve güçlü tooling.
Eğitimci: offline sınıf kurulumu, ders setleri, güvenli sandbox ve örnek projeler.
Ekosistem geliştiricisi: paket, kütüphane, araç ve platform adaptörü.
Başarı hedefleri: ilk kurulumdan Merhaba Dünya’ya 10 dakikadan kısa süre; başlangıç örneklerinin en az %90’ında gereksiz noktalama olmaması; kullanıcıya dönük compiler hata mesajlarının tamamının Türkçe olması; Windows/macOS/Linux resmi toolchain; x86-64 ve ARM64 Tier-1; ilk kararlı sürümde en az 100 örnek ve 20 eğitim projesi.
# 4. Kontrollü Türkçe ve dilbilim sınırı
Türkçe sondan eklemeli ve yüklem-sonlu bir dildir. Tasarım İngilizce fiil-önce API kalıplarını kopyalamak yerine nesne→eylem akışını kullanacaktır. Buna rağmen v1 tam serbest doğal dil olmayacaktır; kontrollü Türkçe kullanılacaktır. Böylece okunabilirlik korunurken parser deterministik kalır.
NFC Unicode normalizasyonu.
ç, ğ, ı, İ, ö, ş, ü tanımlayıcılarda doğal destek.
Büyük/küçük harf davranışı spesifikasyonda kesin tanımlanır.
Homoglyph/confusable tanımlayıcılar için compiler uyarısı.
İyelik ve hal eklerinin desteklenen biçimleri grammar’da açıkça tanımlanır.
Morfolojik serbestlik heuristic ile değil sürümlemeli grammar ile genişler.
Stage 0 bu sözü K-089/RFC-0018/spec-13 ile `zee-tr-1` profiline bağladı:
ek tablosu, iki katman sınırı ve kanonik çözüm↔üretim snapshot/property
korpusuyla sabittir; K-122 semantic SHA-256 kaydı ve Git-tarih koruğuyla aynı
kimlik altında değişikliği kapatır. Proje bildirimi ve kilit dosyası profil
kimliğini taşır. K-123 profil tablosu ile çözüm/üretim kararlarını kök JSON
Schema ve Rust'tan bağımsız conformance verisi olarak yayımlar.
# 5. Grammar discovery yöntemi
Grammar masa başında tek seferde dondurulmayacaktır. Önce 30 adet golden-source program yazılacak; syntax bu gerçek kullanım örneklerinden çıkarılacaktır. Her syntax değişikliği bu corpus üzerinde regression testine girecektir.
Merhaba Dünya
Hesap makinesi
Not ortalaması
Sayı tahmini
Dosya okuyucu
CSV analiz
HTTP istemcisi
Mini web sunucusu
PostgreSQL/SQLite örneği
CLI aracı
ESP32 LED/sensör
Basit 2D oyun
JSON API
Paralel görev
Paket oluşturma
Test yazma
# 6. Sözdizimi taslakları
"Dünyaya merhaba" yaz
isim "Ayşe" olsun
yaş 10 olsun

yaş 8 veya daha büyükse
    isim ile " programlamaya başlayabilir" yaz
değilse
    "Biraz daha oyun zamanı" yaz
10 kez tekrarla
    "Merhaba" yaz
1 den 100 e kadar her sayı için
    sayı çiftse
        sayıyı yaz
işlem ortalamayı hesapla
    sayıları al
    toplam 0 olsun

    her sayı için
        toplamı sayıyla artır

    sonucu toplamı sayıların adedine böl
    sonucu döndür
8080 kapısında sunucu başlat

GET "/durum" adresine istek geldiğinde
    "çalışıyor" yanıtını gönder

Stage 0'da gerçek soket yalnız `dil çalıştır --deneysel-web ...` açık izniyle
localhost eğitim/prototipi için kurulur (K-082). K-087 yöntemli rota, uygulama
eylemi, etki denetimi ve yerel transaction/savepoint sözleşmesini kapattı.
K-088 production oturum/çerez/CSRF ve loopback HTTPS reverse-proxy profilini
kapattı (RFC-0017, spec/12). K-137/ADR-034 kalıcı ortak oturum/rate-limit
deposunu, güvenilir proxy istemci kimliğini ve tek-worker/N süreç modelini
ekledi. K-185/ADR-076 gerçek Zee ERP ihtiyacıyla doğrulanmış kullanıcı ve rolü
yalnız rota kapsamındaki `oturum` sözlüğüne açtı; istemci girdisi ve oturum
sırları bu sınırın dışındadır. K-188/ADR-079 aynı gerçek ürünün güvenlik yaşam
döngüsü için kullanıcı oturumlarını sır taşımayan özet kimlikle listeleme ve
ortak depoda tekil iptal yüzeyini ekledi; ham bearer/CSRF açılmadı ve syntax
freeze yeniden başladı. Secret dağıtımı, çok-hostlu backend ve idempotency ayrı deployment/RFC
kapılarıdır.
# 7. Temel dil yüzeyi
Değer tanımı: “isim Ayşe olsun”.
Fonksiyon: “işlem”.
Sonuç: “döndür”.
Koşul: “… ise / değilse”.
Döngü: “tekrarla / her … için”.
Mantıksal: “doğru / yanlış”.
Null: mümkün olduğunca Seçenek türü; “boş” yalnız açık durumlarda.
Veri yapısı: “yapı”; OOP zorunlu paradigma olmayacak.
Modül: “modül” veya “birim”; kullanıcı testiyle karar.
Paket: “paket”; import için “kullan”.
Async: “eşzamanlı / bekle” ailesi.
# 8. Tür sistemi
Statik tür güvenliği ve yerel tür çıkarımı birlikte kullanılır. Yeni başlayan tür yazmak zorunda kalmaz; public API ve belirsiz durumlarda açık tür kullanılır.
Stage 0'da bu progressive disclosure K-083/K-086 ile kuruldu: `sayıyı al`
yerel başlangıçta çıkarımlı kalır; birim/paket yüzeyinde
`sayıyı Ondalık olarak al` + `Ondalık döndürür` (ya da `değer döndürmez`) tam
sözleşmedir. v1 public modeli bilinçli monomorfiktir; generic soyutlama ayrı
ADR/RFC işidir.
K-092 ile Ondalık tek keyfî hassasiyetli onluk türdür: tam işlemler kayıpsız,
yalnız sonsuz açılımlı bölüm 34 anlamlı haneye deterministik yuvarlanır
(RFC-0013/spec-16). Çocuk için `0,1+0,2=0,3` sözüyle profesyonel para/ölçüm
kapasitesi aynı türde birleşir.
K-093 ile bütün değerler derin kopyadır; paylaşılan gizli nesne kimliği yoktur.
Liste gezmesi değer-sonuç imleciyle alan yazmayı ve yeniden bağlamayı aynı
sıraya geri taşır. Gezilen kaynağın biçimi T053 ile sabittir
(RFC-0019/spec-17); insan zihinsel modeli usability kartlarını bekler.
TamSayı
Ondalık
Metin
Mantıksal
Liste<T>
Sözlük<K,V>
Küme<T>
Seçenek<T>
Sonuç<T,Hata>
Tarih
Saat
Süre
Para
Tehlikeli implicit conversion yok.
Exhaustive pattern matching hedeflenir.
Integer overflow davranışı debug/release arasında sürpriz yaratmayacak şekilde tanımlanır.
Generics ve trait/interface benzeri soyutlama ayrı ADR ile tasarlanır.
# 9. Bellek, kaynak ve hata modeli
v0.x için GC ve ARC prototipleri benchmark edilir. Ownership ancak öğrenilebilirliği bozmadan gerçek fayda sağlarsa değerlendirilir. Dosya, soket ve kilit gibi kaynaklar lexical scope ile otomatik kapanmalıdır.
Beklenen hatalar Sonuç<T,Hata> ile taşınır. Panic yalnız invariant ihlali gibi geri dönülemez durumlar içindir.

**Stage 0 gerçeklemesi:** K-091 ile `Hata`; kararlı kod, Türkçe mesaj,
`Seçenek<Hata>` neden zinciri ve Metin sözlüğü veri taşır. Eski düz mesaj
biçimi `GENEL` koduyla aynı çıktıyı korur; kod eşleme ve deterministik JSON
spec/15'te bağlayıcıdır.
HATA T104

"toplam" burada Sayı olarak kullanılamaz.

12 | toplam "Mustafa" olsun
13 | sonuç toplam ile 10 un toplamı olsun
             ^^^^^^

"toplam" bir Metin değeridir.
Bu işlem için Sayı gerekiyor.

Öneri:
"toplam" değişkenine verilen değeri kontrol et.
# 10. Eşzamanlılık
Structured concurrency hedeflenir. Parent scope iptal olduğunda child işler sahipsiz kalmaz. Timeout ve cancellation birinci sınıftır. Data race varsayılan olarak zorlaştırılır.
eşzamanlı olarak
    profil müşterinin profilini getir
    faturalar müşterinin faturalarını getir
    cihazlar müşterinin cihazlarını getir

hepsini bekle
sonucu döndür

**Stage 0 gerçeklemesi:** K-090 ile görevler deterministik tek-thread
scheduler'da `bekle` noktalarında dönüşümlü ilerler; `hepsini bekle` sonuçları
birleştirir. Sözcüksel sahiplik T051 ile sahipsiz görevi engeller, ilk hata
kardeşleri iptal eder. K-085 mutlak son tarihi bütün görev ağacına yayılır;
bekleme kalan süreye kırpılır ve iptalden sonraki yan etkiler çalışmaz.
K-124 `zee-esz-1` profiliyle çıktı/ortak IO sırasını, sanal süreyi, sonuç
bağlarını ve iptal gözlemlerini ikinci derleyici için bağımsız korpusa bağlar.
Çok çekirdek kullanımı v1 dil özelliği değildir; ancak aynı gözlemleri koruyan
bir iç optimizasyon olabilir. Bağlayıcı anlam: spec/14,
[conformance rehberi](eszamanlilik-conformance.md) ve
[v1 sürüm kapıları](v1-surum-kapilari.md).
# 11. Derleyici ve runtime mimarisi
Kaynak .dil
  ↓ Unicode normalizasyonu
Lexer
  ↓
Parser
  ↓
AST
  ↓
Ad çözümleme
  ↓
Tür kontrolü
  ↓
HIR
  ↓
MIR
  ↓
Optimizasyon
  ↓
Backend
  ├─ Interpreter / bytecode
  ├─ x86-64 native
  ├─ ARM64 native
  └─ WebAssembly
Bootstrap derleyici için Rust güçlü adaydır. Parser için handwritten recursive-descent + Pratt yaklaşımı değerlendirilir. İlk çalışan semantiği hızlı doğrulamak için interpreter/bytecode, ardından Cranelift ve LLVM karşılaştırması yapılır. Backend kararı benchmark, debug deneyimi, binary boyutu, compile süresi ve bakım maliyetiyle ADR üzerinden verilir.
K-194 kurulumun ikinci kayıt retlerinde manifestsiz ikili bırakmasını
staging/doğrulama/geri almayla düzeltti; GB-032 kaynak metadata manifest
enjeksiyonu exact SHA/yol doğrulamasıyla kapandı; gerçek Unix/Windows retleri ve üç platform
`38238657801` tatbikatı geçti, K-194 kapandı. Bu correctness dilimi dağıtım kanıtından önce kapanır.

K-191 derleme/bağımlılık envanteri ve exact Rust provenance tutarlılığını
kapattı. [Envanter](derleme-ve-bagimliliklar.md) mevcut yorumlayıcı ve Rust'tan
üretilen WASM'i gelecekteki Zee kod üretiminden ayırır. Sistem kitaplığı ve
linker/SDK provenance K-192; kaynak metadata kimliği aynı dilimde kanonik depoya bağlandı.
K-192 ortak kapısı kurulu ikili hash/SHA ve Rust'sız PATH program+LSP
provasını sistem kitaplığı raporuna bağlar; üç platform sonucu beklenir.
Ürün kanıtlı backend değerlendirmesi K-193
olarak açıktır. Native backend ve self-hosting gerçeklenmiş değildir.

# 12. Self-hosting stratejisi
Stage 0: Rust bootstrap compiler.
Stage 1: standart kütüphanenin parçaları yeni dilde.
Stage 2: parser/type checker bölümleri yeni dilde.
Stage 3: compiler kendi kaynak kodunu derler.
Stage 4: ardışık compiler çıktıları reproducible-build ile doğrulanır.
Stage 5: bootstrap trust için diverse double compiling araştırılır.
# 13. Standart kütüphane
Temel: Metin, Sayı, Liste, Sözlük, Küme, Seçenek, Sonuç.
Sistem: Dosya, Dizin, Yol, Süreç, Ortam.
Zaman: Tarih, Saat, Süre.
Ağ: TCP, UDP, DNS, HTTP istemci/sunucu.
Veri: JSON, CSV ve ortak DB sürücü sözleşmesi.
Güvenlik: hash, HMAC, güvenli rastgele; düşük seviyeli kripto ayrı modülde.
Test: assertion, fixture, snapshot ve property test.
IoT: GPIO, seri, I2C, SPI platform paketleri.
Grafik: 2D başlangıç paketi; çekirdeğe gömülmez.
# 14. Proje, build ve paket sistemi
dil yeni uzay-oyunum
dil ekle grafik
dil dene
dil çalıştır
dil derle
dil paketle
dil yayınla
Tek manifest + deterministik lock dosyası.
SemVer.
İmzalı metadata, checksum ve provenance.
Namespace politikası; typosquatting/dependency confusion koruması.
Yanked sürümler ve güvenlik duyuruları.
Offline cache ve okul mirror desteği.
Reproducible package build ve SBOM.
Registry API açık spesifikasyon.
Keyfi post-install script varsayılan olarak yasak veya capability ile sınırlı.

**Gerçekleme notu — 1 Eylül 2026 (K-076/K-078):** Proje ve ilk paket katmanı
çalışır:
geçerli zee sözdizimli `proje.dil` adı, `X.Y.Z` sürümü ve göreli giriş
dosyasını tanımlar; `dil çalıştır/denetle/dene <klasör>` ve `dil yeni`
bu sözleşmeyi kullanır. `yerel_bağımlılıklar` başka zee projelerini paket
olarak bağlar; `X paketini kullan` doğrudan bağımlılığı alır. `dil kilitle`
geçişli grafiği göreli yol, sürüm, kenar ve SHA-256 kaynak özetiyle
deterministik `proje.kilit` dosyasına sabitler. Registry, imza/provenance,
SBOM ve uzak sürüm çözümü bu aşamada henüz başlamamıştı. K-094'te deterministik
`.zep`, Ed25519 yayın bildirimi, SPDX 3.0.1 SBOM ve SLSA v1 provenance çalışan
ilk dağıtım çekirdeğine dönüştü. K-095 ağ dışı root sabitlemesi, eşik/çift
eşikli rotasyon, timestamp→snapshot→targets bağları, rollback/expiry ve exact
yayıncı/yanked/duyuru politikasını çalışan metadata doğrulayıcısına dönüştürdü.
K-135 limitli HTTPS/statik taşıma, kalıcı metadata/cache ve offline hit/miss'i
tamamladı. K-136 exact `ad@X.Y.Z` bildirimi, `proje.kilit` v3, proje-local
atomik kaynak kurulumu ve açık ağ kullanan CLI bağını tamamladı; V1-P1-07
kapandı. Normal derleme/LSP doğrulanmış cache'den sessiz ağ açmadan çalışır.
`dil ekle <yerel-yol> [proje]` (K-079) aday grafiği diske yazmadan çözer;
başarılıysa yorumu koruyan resmî biçimde bildirimi ve kilidi günceller,
yazma hatasında önceki iki dosyayı geri yüklemeyi dener.
`dil paketler [proje]` (K-080) doğrudan/geçişli grafiği sürüm, taşınabilir yol
ve SHA-256 özetle açıklar. `dil çıkar <paket> [proje]` yalnız doğrudan
bağımlılığı kaldırır; ana proje kaynağı paketi hâlâ kullanıyorsa P010 ile hiçbir
dosyaya dokunmadan durur. Başarı K-079'un iki dosyalı geri alma sözleşmesini
kullanır.

**Kalıcılık notu — 1 Eylül 2026 (K-084/RFC-0016):** Runtime'ın tek-dosya
`yaz/ekle` işlemi, biçimleyici ve paket/kilit yazma yolları atomik replace ve
süreçler arası işletim sistemi kilidi kullanır. Yarım dosya ve kayıp ekleme
V1-P0-04 düzeyinde kapatıldı; çok kaynaklı iş transaction'ı RFC-0015'tedir.
# 15. Geliştirici araçları
dil: resmi CLI.
dilfmt: tek resmi formatter.
dillsp: LSP; completion, rename, go-to-definition, diagnostics.
diltest: unit, integration, snapshot, property test.
dildoc: kaynak yorumlarından API dokümantasyonu.
dilpaket: registry istemcisi.
Debugger: DAP uyumu ve Türkçe stack trace.
Playground: mümkün olduğunca client-side WASM sandbox.
VS Code ilk resmi entegrasyon; JetBrains/Neovim sonra.
# 16. Eğitim ve kullanıcı deneyimi
Özel IDE ilk hedef değildir. Önce güçlü VS Code eklentisi ve web playground geliştirilir. Eğitim başarısı syntax ezberinden çok algoritmik düşünmeyi kolaylaştırmasıyla ölçülür.
Türkçe autocomplete, hover ve hata açıklaması.
Hata mesajında “neden?” bağlantısı.
Çocuk modu: dosya, ağ ve process erişimi sandbox.
Öğretmen modu: şablonlar ve izin profilleri.
İnternetsiz okul kurulumu.
Oyun, robot, matematik, hikâye ve web proje setleri.
Ekran okuyucu, klavye ve yüksek kontrast desteği.
# 17. IoT ve fiziksel dünya
Çocuklar için fiziksel sonuç güçlü motivasyondur. ESP32, micro:bit ve Arduino sınıfı cihazlar için güvenli eğitim paketleri planlanır.
kapı açıksa
    kırmızı ışığı yak
değilse
    yeşil ışığı yak
USB/seri yükleme aracı.
Kart tanım paketleri.
Donanım olmadan simulator.
Offline kullanım.
Elektriksel güvenlik ve sınıf laboratuvarı dokümanları.
# 18. Güvenlik modeli
Paket metadata/provenance imzalanır.
Registry publish passkey/2FA destekler.
Build scriptler sandbox/capability modeliyle sınırlandırılır.
Unicode confusable kontrolü.
Unsafe/FFI açık ve görünür sınırdır.
Compiler fuzzing ve parser property/differential testleri.
SBOM, checksum ve security advisory.
K-141/ADR-038 bootstrap Rust grafiğini sabit cargo-deny ile güncel RustSec,
lisans, ban ve kaynak kökeni kapısına; iki lock'u SHA-256 doğrulamalı gerçek
offline vendor derlemesine bağlar.
Playground kaynak/soru ön-tahsis RAM limiti K-143/ADR-040 ile çalışır;
daha geniş CPU/süre/IO profil genişletmeleri ayrıca izlenir.
Paket kurulumunda keyfi post-install varsayılan olarak yasak.
Secret değerlerin loglanmasını engelleyen tür/capability yaklaşımı ileri araştırma konusu.
# 19. FFI ve mevcut ekosistem
Yeni dil ilk günden bütün kütüphaneleri yeniden yazamaz. C ABI birincil köprü adayıdır. Rust/C/C++ ve platform API’leri için generator/adapters sonra gelir.
FFI unsafe boundary.
Harici İngilizce API’ler Türkçe wrapper ile sunulabilir.
ABI sürümleme politikası.
Native paketlerde OS/architecture metadata.
Ownership, lifetime ve crash sınırları açıkça belgelenir.
K-142/ADR-039 tarayıcı bootstrap köprüsünü genel FFI'dan ayıran `dil ABI v2`
olarak sürümler. Host yalnız kayıtlı başlangıç pointer'ı+exact boy verir;
strict UTF-8, sorgulanabilir sonuç sahipliği ve yanlış/çift bırakma davranışı
native, gerçek wasm32 hostu ve ayrı libFuzzer hedefiyle kanıtlanır.
K-143/ADR-040 bu köprüyü `dil ABI v3`e taşır. Host 8 MiB kaynak ile
1 MiB/4.096 satır soru limitini modülden okur; Rust sahipli kopya/satır
tablosundan, tarayıcı UTF-8 byte dizisi ve WASM tahsisinden önce reddeder.
K-125/ADR-029 ile Ondalık'ın C `float`/`double` ya da binary32/binary64'e
örtük eşlenmesi yasaktır. Gelecekteki köprü görünür `kayıplı` işareti,
`Sonuç` ve açık IEEE 754 semantiği ister; FFI Faz 4/5'e kadar dil yüzeyi
değildir.
# 20. Platform hedefleri
Tier 1: Windows x86-64, macOS ARM64/x86-64, Linux x86-64/ARM64.
Tier 2: WebAssembly.
Tier 3: ESP32/microcontroller subset veya özel backend araştırması.
CI tüm Tier-1 hedeflerde conformance çalıştırır.
Platform davranışı stdlib adapter’larında izole edilir.
# 21. Test ve doğrulama
Lexer/parser golden tests.
AST snapshot tests.
Compile-pass/compile-fail corpus.
Property-based testing.
Fuzzing.
Differential backend tests.
Runtime memory/resource tests.
Resmi conformance suite.
Reproducible-build checks.
Package/registry supply-chain tests.
Bootstrap dependency graph için günlük advisory ve boş-cache offline vendor
kanıtı K-141/ADR-038 ile çalışır.
Playground C ABI'si kayıt dışı pointer/uzunluk ve yaşam döngüsü sıralarıyla
K-142/ADR-039 ve K-143/ADR-040 kaynak+soru kipli `wasm_abi` hedefinde gecelik
fuzz edilir.
LSP protocol tests.
Cross-platform integration.
Performance regression benchmarks.
Çocuk, öğretmen ve profesyonel usability testleri.
# 22. Performans hedefleri
İlk hedef en hızlı dil olmak değildir. Önce correctness, tanılama kalitesi ve öğrenilebilirlik gelir. Performans sürekli ölçülür ve regression bütçeleri tanımlanır.
Küçük projede anlık typecheck.
Incremental build.
Compiler memory sınırları.
Native binary startup.
Runtime throughput/latency benchmark seti.
LSP p95 yanıt süresi.
Paket çözümleme süresi.
Benchmark sonuçlarının sürümler arası arşivlenmesi.
# 23. Resmi dil spesifikasyonu
Lexical ve Unicode kuralları.
EBNF grammar.
Ad çözümleme ve scope.
Tür sistemi ve dönüşümler.
Evaluation order.
Numeric overflow.
Memory/resource semantics.
Concurrency/cancellation.
Error model.
Module/package resolution.
FFI/ABI.
Standard library stability.
Deprecation ve compatibility politikası.
Breaking change sessizce yapılamaz. Büyük dil evrimleri için edition benzeri
model değerlendirilebilir. Morfolojik aday kümesini değiştiren ilk kırıcı
sınır şimdiden tanımlıdır: `zee-tr-1` yerinde değişmez; yeni profil kimliği
ve ana sürüm/edition kararı gerekir. K-122 çalışan semantic kayıt ile
yayımlanmış fixture'ın değiştirilmesini ayrı test+CI kapılarında reddeder
(K-089/K-122, RFC-0018).
# 24. Repository ve bileşen yapısı
turkce-dil/
  compiler/
    lexer/
    parser/
    ast/
    types/
    hir/
    mir/
    backend/
  runtime/
  stdlib/
  cli/
  formatter/
  lsp/
  debugger/
  package-manager/
  registry/
  playground/
  editors/vscode/
  examples/
  education/
  spec/
  rfcs/
  adr/
  tests/
    conformance/
    compile-pass/
    compile-fail/
    fuzz/
    golden/
  docs/
# 25. Yönetişim, RFC ve ADR
Dil değişikliği RFC ister.
Güvenlik/mimari sınır değişikliği ADR ister.
Core team ve maintainer rolleri.
Code of Conduct.
Security policy ve özel bildirim kanalı.
Sürüm takvimi ve deprecation süresi.
Trademark/isim politikası.
Paket registry moderasyon politikası.
Eğitimci ve kullanıcı geri bildirim mekanizması.
Kurucu vizyon korunırken bus-factor tek kişi olmamalıdır.
# 26. Lisans ve sürdürülebilirlik
Çekirdek dilin çocukların ve eğitim kurumlarının önüne ücret duvarı koymaması temel ilkedir. Lisans seçimi hukuk incelemesiyle yapılır; Apache-2.0/MIT ve copyleft seçenekleri topluluk ve ticari kullanım etkileriyle karşılaştırılır.
Compiler, stdlib, CLI ve temel tooling ücretsiz/açık kaynak hedefi.
Registry açık protokollü; alternatif mirror/registry mümkün.
Sürdürülebilirlik bağış, sponsor, eğitim, destek veya kurumsal hizmetlerden gelebilir; dilin kendisi kilitlenmez.
Marka kullanımı ile kod lisansı ayrılır.
# 27. CI/CD ve release engineering
Her PR: format, lint, unit, compile-pass/fail, conformance subset.
Nightly: full conformance, fuzz corpus, cross-platform, sanitizer, benchmark.
Release candidate: reproducible build, SBOM, provenance, imza, security scan.
Binary dağıtımları checksum ve signature ile.
Nightly/beta/stable kanalları.
Rollback ve yanked release prosedürü.
Compiler bootstrap zinciri ayrıca doğrulanır.
K-141'den itibaren hiçbir CI Cargo komutu kilitsiz fallback yapamaz; compiler
ve fuzz lock'ları aynı vendor manifestini ve offline derlemeyi üretmelidir.
# 28. Gizlilik ve telemetri
Compiler ve araçlar varsayılan olarak kullanıcı kaynak kodunu veya kişisel veriyi toplamaz. Telemetri varsa opt-in, açık şemalı ve kapatılabilir olur.
Crash raporlarında kaynak kodu varsayılan olarak gönderilmez.
Paket indirme istatistiği minimum veriyle tutulur.
Eğitim/UX araştırması ayrı açık rıza ile.
Telemetry schema public belgelenir.
# 29. Dokümantasyon mimarisi
5 dakikada başla.
Dil turu.
Resmi referans.
Compiler hata kataloğu.
Standart kütüphane API.
Paket geliştirme rehberi.
FFI rehberi.
Öğretmen rehberi.
Çocuk proje kitaplığı.
RFC/ADR arşivi.
Migration ve edition rehberleri.
# 30. Ürün yüzeyi ve web
Ana site: manifesto, indir, öğren, doküman, paketler, playground.
Paket registry araması.
Paylaşılabilir playground örnekleri.
Sürüm ve güvenlik duyuruları.
Topluluk katkı rehberi.
Türkçe birincil; yabancı katkıcılar için İngilizce dokümantasyon eklenebilir, dil yüzeyi Türkçe kalır.
# 31. Yol haritası
Faz 0 — Felsefe: manifesto, 30 golden program, isim araştırması, grammar ilkeleri.
Faz 1 — Interpreter: lexer, parser, AST, temel türler, koşul/döngü/işlem, Türkçe diagnostics.
Faz 2 — Dil çekirdeği: modül, koleksiyon, Seçenek/Sonuç, formatter, test runner.
Faz 3 — Tooling: LSP, VS Code, docs, package manifest ve local package manager.
Faz 4 — Native: HIR/MIR, backend, x86-64/ARM64, debug info.
Faz 5 — Ekosistem: registry, imza/provenance, stdlib ağ/veri/sistem modülleri.
Faz 6 — Eğitim: playground, dersler, IoT başlangıç paketi, öğretmen pilotu.
Faz 7 — Self-hosting: compiler parçalarını yeni dile taşıma.
Faz 8 — 1.0: spesifikasyon freeze, compatibility, security audit, conformance, stable release.
# 32. İlk 90 günlük uygulanabilir plan
Hafta 1–2: manifesto + 30 golden program + 10 anti-example; syntax karar günlüğü.
Hafta 3–4: lexer + Unicode kuralları + indentation tokenları + parser skeleton.
Hafta 5–6: AST + isim çözümleme + TamSayı/Metin/Mantıksal + değişken/koşul/döngü.
Hafta 7–8: işlem/return + Liste + Seçenek/Sonuç ilk taslağı + interpreter.
Hafta 9: Türkçe diagnostic framework + compile-fail corpus.
Hafta 10: resmi formatter + CLI: çalıştır/denetle/biçimle.
Hafta 11: VS Code syntax/LSP minimum.
Hafta 12: v0.1 demo; 10 öğrenci + 5 profesyonel usability oturumu; grammar revizyonu.
# 33. v0.1 kabul kriterleri
Merhaba Dünya, hesap makinesi, not ortalaması ve sayı tahmini çalışır.
Türkçe tanımlayıcılar sorunsuz.
Girinti blokları deterministik.
Temel type errors Türkçe ve kaynak konumlu.
Formatter idempotent.
Windows/macOS/Linux üzerinde interpreter/CLI çalışır.
Golden corpus CI’da.
Kaynak kodda İngilizce keyword yazmak gerekmez.
# 34. v1.0 tamamlanma tanımı
Resmi grammar/type/runtime spesifikasyonu yayımlanmış.
Tier-1 platformlarda conformance yeşil.
Native x86-64/ARM64 toolchain.
Kararlı paket formatı ve registry.
Formatter, LSP, debugger, test ve docs production kalitesinde.
Stdlib temel/sistem/ağ/veri alanlarını kapsar.
Supply-chain imza/provenance/SBOM hattı.
Security audit ve fuzzing operasyonu.
Compatibility/deprecation politikası.
En az bir gerçek eğitim pilotu ve birkaç gerçek profesyonel proje.
Self-hosting tamamen bitmemiş olsa bile compiler’ın anlamlı parçaları kendi dilinde.
# 35. Risk kaydı
Doğal Türkçe ile deterministik grammar gerilimi — controlled Turkish + golden corpus.
Morfoloji kapsamı patlatabilir — v1 varyantlarını sınırlı tut.
Çocuk dostuluğu dili güçsüzleştirebilir — progressive disclosure.
Tek kişi bus-factor — RFC, doküman, test, topluluk.
Tooling yükü compiler’dan büyük olabilir — LSP/DAP standartlarını kullan.
Kütüphane eksikliği benimsemeyi öldürebilir — C ABI/FFI köprüsü.
Registry supply-chain riski — imza, provenance, namespace, 2FA.
Native backend çok erken zaman yiyebilir — önce interpreter ile semantiği doğrula.
İsim/marka çakışması — isim seçmeden domain/trademark/repo taraması.
Türkçe ile sınırlı algılanma — bilinçli hedef; katkıcı dokümanı çok dilli olabilir.
# 36. Açık tasarım kararları
Dil adı ve dosya uzantısı.
GC mi ARC mi?
Cranelift mi LLVM mi, ikisi birden mi?
Case sensitivity davranışı.
modül mü birim mi?
Generic syntax tamamen sembolsüz nasıl olacak?
Pattern matching Türkçe yüzeyi.
Async/await yerine nihai kelime ailesi.
Trait/interface kavramının Türkçe adı.
Macro/metaprogramming olacak mı?
Reflection seviyesi.
Package namespace modeli.
Microcontroller desteği ana dil mi subset mi?
# 37. İlk RFC listesi
RFC-0001 — Dil Manifestosu ve Tasarım İlkeleri
RFC-0002 — Lexical ve Unicode Kuralları
RFC-0003 — Girinti ve Blok Modeli
RFC-0004 — Değer Tanımı ve Scope
RFC-0005 — Koşullar ve Mantıksal İfadeler
RFC-0006 — İşlemler ve Parametreler
RFC-0007 — Temel Tür Sistemi
RFC-0008 — Seçenek ve Sonuç
RFC-0009 — Modül ve Paket Modeli
RFC-0010 — Hata ve Tanılama Standardı
RFC-0011 — Structured Concurrency
RFC-0012 — FFI ve Unsafe Sınırı
# 38. İlk ADR listesi
ADR-001 — Bootstrap implementasyon dili
ADR-002 — Parser stratejisi
ADR-003 — İlk execution modeli: interpreter/bytecode
ADR-004 — Bellek yönetimi prototip kararı
ADR-005 — Native backend seçimi
ADR-006 — Paket registry trust modeli
ADR-007 — Telemetri ve gizlilik
ADR-008 — Self-hosting aşamaları
ADR-009 — Dilin adı
ADR-010 — Normatif otorite ve değişiklik bütünlüğü
ADR-011 — Core AST intrinsic/yetkinlik sınırı
ADR-012 — Derleyici fiziksel faz modülleri
ADR-013 — Checker semantik katmanları ve tek sahiplik
ADR-014 — Yapı, işlem ve sembol semantic kimlikleri
ADR-015 — Derleyici fazlarını Rust türleriyle görünür kılma
ADR-016 — Typed HIR çekirdeği ve aşamalı runtime geçişi
ADR-017 — Native ağ I/O kaynak sınırları
ADR-018 — Sınırlı web oturum deposu ve mutlak ömür
ADR-019 — LSP çerçeve ve JSON girdi sınırları
ADR-020 — HIR düğümlerinde zorunlu kaynak aralığı
ADR-021 — Production panic yüzeyi ve fail-closed hata politikası
ADR-022 — Lexer/parser fuzz korpusu ve sürekli mutation politikası
ADR-023 — AST/HIR invariant doğrulama politikası
ADR-024 — Parser hata kurtarma politikası
ADR-025 — Tanı kimliği kararlılık politikası
ADR-026 — Deterministik IO izi mimarisi
ADR-027 — Sürümlü deterministik IO profili
ADR-028 — Kanonik `.zep` yol ve Unicode güvenlik profili
ADR-029 — Ondalık ile binary float arasında örtük köprü yoktur
ADR-030 — Bütün AST ifadelerinde kesin kaynak aralığı
ADR-031 — Merkezî yetkinlik politikası ve native outbound istemci
ADR-032 — Atomik replace metadata koruma politikası
ADR-033 — Merkezî ve değişmez kaynak bütçesi
ADR-034 — Kalıcı ortak web deposu ve tek-worker süreç modeli
ADR-035 — Protokol-kesin LSP JSON-RPC sınırı
ADR-036 — Kanonik web proxy origin sınırı
ADR-037 — Byte tabanlı HTTP/1.x istek sınırı
ADR-038 — Rust tedarik zinciri ve offline vendor kapısı
ADR-039 — Hasım hosta karşı sürümlü WASM C ABI
ADR-040 — Playground girdi ön-tahsis bütçesi
# 39. Ekip ve rol modeli
Dil mimarı: semantik ve uzun vadeli vizyon.
Compiler: parser, types, IR, backend.
Runtime/stdlib: memory, IO, network, platform.
Tooling: LSP, formatter, debugger, editor.
Package/security: registry, signing, provenance.
Education: pedagojik içerik ve okul pilotları.
Language/UX: Türkçe dilbilim, okunabilirlik ve usability.
Release: CI, conformance, reproducible builds.
Başlangıçta bir kişi birden çok rolü taşıyabilir. Rol ayrımı, ileride katkı geldiğinde sorumluluk sınırını net tutmak içindir.

# 40. V1 öncesi öncelikli mühendislik backlog'u

Hızlı büyüyen bootstrap derleyicinin omurgasını yeni özelliklerden önce
sağlamlaştıran bağlayıcı uygulama sırası
[docs/oncelikli-backlog.md](oncelikli-backlog.md) içindedir. İlk kapılar K-016
çağrı usability kararı, K-093 gezme usability sonucu, genellenebilir expression
grammar mimarisi ve core AST→capability/intrinsic ayrımıdır. HIR/SymbolId/faz
zincirinin SymbolId temeli K-101, açık faz tipleri K-102 ile tamamlandı; K-103
typed HIR tür/bağ çekirdeğini kurdu, K-104 standart runtime'ı bu bağlara
geçirdi. 1 Eylül güvenlik incelemesi source span'in önüne sınırsız ağ/oturum/
LSP girdisini aldı: K-105/ADR-017 native HTTP istemcisine varsayılan 30 saniye
ve 8 MiB yanıt, yerel sunucuya 10 saniye mutlak istek okuma sınırı koydu.
K-106/ADR-018 process içi oturum deposunu 4096 toplam/1024 anonim kayıtla
sınırlayıp anonim LRU ve kaymayan mutlak ömrü bağladı. K-107/ADR-019 LSP
çerçevesine 8 KiB/8 MiB, JSON'a 128 derinlik/100 bin düğüm bütçesi ve sıkı
Unicode doğrulaması koydu. K-138/ADR-035 bu JSON sınırını RFC 8259 sayı durum
makinesi, kayıpsız sayısal kimlik, duplicate alan reddi, standart
`-32700/-32600/-32601/-32602` ayrımı ve notification sessizliğiyle
protokol-kesin yaptı. K-108/ADR-020 her semantic HIR ifadesinde kaynak
aralığını yapısal zorunluluk yaptı; kesin token konumu yoksa uydurma sütun
yerine kaynak satırı zarfı taşınır. K-126/ADR-030 bütün parser AST yaprak ve
bileşiklerini kesin `AstKaynakAraligi` zarflarına aldı; checker bu aralığı
HIR'a birebir aktarır, tanı ve LSP üretimde tüketir. K-109/ADR-021 production'daki 46 doğrudan
panic noktasını tanı/sonuca çevirdi ve dört crate kökünde kalıcı Clippy deny
kapısı kurdu. K-110/ADR-022 sekiz saldırı tohumu, Zee mutation sözlüğü,
deterministik UTF-8 regresyonları ve korpusu büyüten gecelik libFuzzer hattıyla
lexer/parser panic-free sınırını kapattı. K-111 `zee-tr-1` üret→çöz uzayını
4.096 köke genişletip bütün-aday A002 ve NFC/NFD sınırını stable+gecelik fuzz
kanıtına bağladı. K-112/ADR-023 parser AST'si ile bağlı typed-HIR arasındaki
tamlık, kimlik ve imkânsız-durum değişmezlerini debug/test faz kapısına
bağladı; özellik→alan dönüşümündeki klon kaynaklı yetim HIR kaydını bulup
düzeltti. K-113/ADR-024 LSP odaklı parser recovery'yi cümle sonu+dengeli
girinti senkronizasyonu, kaynak sırası ve 20 tanılık ortak bütçeyle kapattı.
K-114/ADR-025 ile başlayan ve K-136'da P017 ile 152 etkin + 3 ayrılmışa çıkan tanıların
kod↔anlam bağı sürümlü
fixture'a sabitledi. K-115/RFC-0022/ADR-026 bütün runtime IO çağrılarını
kanonik, sürümlü ve dış etkisiz replay edilebilen bir iz protokolüne bağladı;
`dil iz kaydet/oynat` kullanıcı kapısı ve gizlilik sınırları birlikte geldi.
K-116/RFC-0023/ADR-027 tohumdan rastgele diziye geçişi, tam i64 aralık
eşlemesini, sanal saat ilerlemesini ve hermetik adaptör gözlemlerini
`zee-io-1` profiline sabitledi. K-117/ADR-028 paket yolunu NFC'ye kanonikledi;
Türkçe Unicode `.zep` fixture'ını üç Tier-1 işletim sisteminde aynı teste ve
80 saldırı vakasını kalıcı korpusa bağladı. K-118 71 RFC/ADR/spec belgesini
yürütülebilir test dosyalarına bağlayan haritayı ve README canlı sayı
üreticisini tazelik kapısına aldı. K-119 formatter'ın `SatirSonu` ve girinti
yapısı dahil tam parser-token izini 33 golden programın dağınık-boşluk
varyantında eşitleyip iki tarafı ayrıştırarak parse-equivalence kapısını
kurdu. K-120 LSP definition/rename'i `SymbolId`/`IslemId`/`YapiId` typed-HIR
dizinlerine geçirip hatalı belgede metin tahminini ve dış tanım için eksik
tek-dosya rename'i kapattı. K-121 yerel işlem parametre kısıtlarını gövde/HIR
üretiminden önce birleştirip çağrı sırası etkisini kapattı. K-122 `zee-tr-1`
davranışını semantic SHA-256 kayıt ve Git-geçmişli CI koruğuyla immutable
yaptı. K-123 bunu kök `conformance/` alanında sürümlü JSON Schema, 27
çözüm/karar ve 21 üretim vakasıyla derleyiciden bağımsız bir tüketici
sözleşmesine dönüştürüp B-009'u kapattı. K-124 aynı alanı `zee-esz-1` ve 10
kaynak+gözlem vakasıyla scheduler'a genişletip B-011'i kapattı.
K-125/ADR-029 açık Ondalık↔binary float FFI sınırını örtük eşleme yasağı ve
zorunlu `kayıplı`+`Sonuç` kapısı olarak kapattı. K-126/ADR-030 bütün AST
ifadelerinde kesin source span'i HIR, tanı ve LSP'ye bağlayıp B-050'yi
kapattı. K-127/RFC-0024/ADR-031 sekiz dış dünya yetkinliğini proje/paket,
compile ve runtime boyunca tek fail-closed politikaya bağladı; native outbound
rustls HTTPS, exact origin, DNS sonrası IP/özel-kullanım-geçiş öneki, sıfır
redirect/proxy ve kaynak bütçeleri taşır. B-023/B-049 kapandı. K-128/ADR-032
atomik replace'i Tier-1 owner/group, ACL/xattr ve Windows security metadata
koruması; symlink/desteksiz platform fail-closed kararıyla bağlayıp B-048'i
kapattı. K-129/RFC-0025/ADR-033 tek değişmez `KaynakSinirlari` profilini
kaynak/token/proje toplamı, runtime adım/çıktı/koleksiyon/görev, sınırlı dosya
okuma ve LSP toplam bellek/outbound yüzeyine bağladı. K-130 bunu 16 MiB
bütçeli metin üretimi, yaklaşık 64 MiB saklanan değer zarfı, görev ortamı
klonları ve 64 süreç-geneli ağ bağlantısına genişletti. K-131 HTTP/ağ,
oturum, IO izi, LSP, paket/registry, tanı ve kalıcı dosya sabitlerinin sayısal
sahipliğini aynı profilin domain görünümlerinde topladı. K-132 LSP outbound
JSON'unu append sırasında 8 MiB ile sınırlayıp rename/diagnostics ara gövde
tahsislerini kaldırdı; B-025/V1-P0-31 kapandı.
K-133 B-026'nın ilk diliminde bütün dış etki sınırlarını tam çağrı öncesi
deadline denetimine bağladı; HTTP görevi scheduler'a sıra verdikten sonra
kalan süreyi yeniden hesaplar ve dolmuş adaptörü başlatmaz. K-134 ilk web
yanıtı, çerez ve session mutation'ını request transaction'ında birleştirdi;
timeout/runtime/socket hatası veya yanıtsız rota rollback eder. B-026 kapandı.
P0 maddeleri kapanmadan yeni dil özelliği
öne alınmaz; yarım güvenlik/correctness dilimi önce atomik olarak tamamlanır.

K-016'nın makine hazırlığı K-096 ile
[karar paketine](k016-cagri-karar-paketi.md) bağlandı: önce serbest üretim,
sonra sıra dengeli kör A/B/C kartları; 10 çocuk + 5 profesyonel alt grup
eşikleri ve yedi teknik bağlam. A çalışan hipotezdir, insan verisi değildir.
V1 yalnız bir genel çağrı grammar'ı taşır; B'nin güçlü çıkması ikinci yüzey
eklemek yerine K-097 ile dondurulan RFC-0021 expression grammar mimarisini
yeniden açar.

B-003, K-097 ile tamamlandı: RFC-0021/spec-20/ADR-002 primary → erişim/postfix
→ çağrı → aritmetik → birleştirme → karşılaştırma → boolean katmanlarını,
tam bölge tüketimini ve her yeni ifade yüzeyinin çakışma+conformance kapısını
bağlar. Parser'daki fiziksel fonksiyon/modül ayrımı bu davranışı değiştirmeden
B-005/K-099'da tamamlandı. B-004, K-098/ADR-011 ile tamamlandı: HTTP, sensör, CSRF
ve parola kaynak yüzeyleri tek `Intrinsic { kimlik, argumanlar }` AST düğümüne
indirilir; tür imzası, gereken yetkinlik ve statik etki merkezi kayıttadır.
Kaynak semantiği değişmedi. B-005, K-099/ADR-012 ile parser cümle/ifade,
checker cümle/ifade/çağrı, runtime cümle/ifade fazlarına ayrıldı ve kaynak-mimari
bütçe testi yeniden birleşmeyi durdurdu. B-006, K-100/ADR-013 ile checker
kökünü orkestrasyona indirdi; tür, bağlam, sembol, akış, çağrı, sözleşme,
etki/yetkinlik ve dönüş kurallarını tek sahipli katmanlara ayırdı. Sıradaki
omurga B-010, K-101/ADR-014 ile tamamlandı: `YapiId`, `IslemId` ve `SymbolId`
kaynak adı, semantic identity ve fiziksel depolamayı ayırır; AST checker
sonrası açık bağları taşır. Kaynak, token, parsed AST, bağlanmamış ve
bağlanmış programı ayrı türlere taşıyan B-018, K-102/ADR-015 ile tamamlandı;
standart runtime yalnız bağlı giriş kullanır. B-019, K-103/K-104/ADR-016 ile
checker türleri ve ID bağlarını zorunlu HIR'a taşıdı; runtime ve `dene` bu
bağları tek semantic karar kaynağı yapar. Güvenlik incelemesiyle K-105 ağ
kaynak sınırlarını, K-106 process içi oturum kotasını ve K-107 LSP girdi
sınırını kapattı. K-108 HIR source span değişmezini, K-109 production panic
audit'ini, K-110 lexer/parser fuzz hattını, K-111 morfoloji property/fuzz
kapısını, K-112 AST/HIR invariant doğrulayıcıyı ve K-113 parser hata kurtarma
kapısını kapattı. K-114 tanı kodlarının sürümler arası semantic kimliğini ve
mezar taşlarını fixture'a bağladı. K-115 bütün `GirdiCikti` protokolünü
byte-kanonik trace/replay ile kapattı. K-116 rastgelelik, saat ve fake-IO
semantiğini `zee-io-1` profiliyle sürümledi. K-117 NFC-kanonik `.zep` yolunu,
üç platformlu sabit fixture'ı ve 80 vakalık saldırı korpusunu tamamladı.
K-118 spec↔code kanıt haritasını ve canlı depo sayıları üreticisini tamamladı.
K-119 formatter parse-equivalence kanıtıyla B-042'yi, K-120 semantic LSP
bağıyla B-041'i, K-121 iki fazlı yerel çağrı çıkarımıyla B-007'yi
tamamladı. K-122 immutable `zee-tr-1` kapısıyla B-008'i tamamladı. K-123 kök
JSON Schema ve veri korpusuyla compiler'dan bağımsız morfoloji conformance
sözünü bağlayıp B-009'u kapattı. K-124 `zee-esz-1` gözlem profiliyle
scheduler'ın bağımsız uyumluluk sözünü bağlayıp B-011'i kapattı.
K-125/ADR-029 örtük Ondalık↔binary float köprüsünü yasaklayıp B-012'yi
kapattı. K-126/ADR-030 bütün AST ifadelerini kesin kaynaklandırıp B-050'yi
kapattı. K-127/RFC-0024/ADR-031 proje/paket capability ve outbound SSRF/HTTPS
sınırını kapattı. K-128/ADR-032 atomik replace metadata sözleşmesiyle B-048'i
kapattı. K-129/RFC-0025/ADR-033 B-025'in ilk ortak `KaynakSinirlari` profilini
kurdu; K-130 değer/metin heap'i, görev klonları ve bağlantı izinlerini ekledi.
K-131 eski domain kaynak sabitlerinin sayısal sahipliğini aynı tipe taşıdı.
K-132 LSP outbound JSON'unu bounded üretip B-025'i kapattı. K-133/K-134 etki
öncesi deadline ve web request transaction katmanlarıyla B-026'yı kapattı.
K-135 B-029'un HTTPS taşıma, doğrulanmış içerik-adresli cache, CAS korumalı
kalıcı rollback ve offline dilimini kapattı. K-136 exact manifest, kilit v3,
salt-okunur kaynak kurulumu ve CLI entegrasyonuyla B-029/V1-P1-07'yi kapattı.
K-137 kalıcı ortak web oturumu/revoke/expiry ve atomik rate-limit deposunu,
tek-hop `Forwarded` kimliğini ve N ayrı tek-worker süreç modelini kurarak
B-046'yı kapattı. K-185/ADR-076 doğrulanmış oturum kullanıcı/rolünü rota içi
salt-okunur bağlama taşıyarak Zee ERP'nin tenant bağlamını istemci girdisinden
kurma riskini GB-026 ile kapattı. K-138/ADR-035 RFC 8259 sayı/duplicate alan doğrulamasını,
kayıpsız JSON-RPC kimliğini ve standart parse/request/method/params hata
ayrımını kurarak B-051'i kapattı. K-139/ADR-036 CLI'a özel origin parser'ını
kaldırıp web proxy'yi ortak `AgHedefi` DNS/IPv6/port kimliğine ve açık
loopback bind+peer değişmezine bağlayarak B-052'yi kapattı. K-140/ADR-037
request-line/header/gövde framing'ini yalnız CRLF, origin-form, tek kurallı CL
ve exact UTF-8 kabul eden byte parser'a taşıdı; ayrı ham-byte fuzz hedefiyle
B-053'ü kapattı. K-141/ADR-038 compiler ve fuzz bağımlılıklarını sabit
`cargo-deny` RustSec/lisans/ban/kaynak politikasına, kilitli CI'a ve iki
üretimli SHA-256 manifestli boş-cache offline vendor derlemesine bağlayarak
B-054'ü kapattı. K-142/ADR-039 WASM köprüsünü kayıtlı exact pointer/uzunluk,
strict UTF-8, açık sonuç ömrü, native+gerçek wasm32 host testi ve ayrı hasım
çağrı fuzzer'ına bağlayarak B-055'i kapattı. K-143/ADR-040 ABI v3 limit
dışa aktarımlarıyla kaynak/soru girdisini sahipli kopya, satır tablosu ve
tarayıcı UTF-8 tahsisinden önce sınırlayarak B-056'yı kapattı. K-144/ADR-041,
48 kritik üretim işlevini incelenmiş Clippy tabanına, kademeli büyüme payına ve
bayat olmayan makine raporuna bağlayarak B-035'i kapattı. K-145/ADR-042,
53 dosyadaki eski biçim borcunu tek mekanik dilimde temizleyip sabit
`rustfmt` bileşenli üç platformlu `cargo fmt --all -- --check` kapısıyla
B-037'yi kapattı. K-146/ADR-043 gerçek Cargo/libtest envanterini 21 birincil
faza tam sahipletip her Tier-1 platformunda pass/fail/ignored, süre,
regression, fuzz ve conformance bağlarını ayrı raporlayarak B-038'i kapattı.
K-147/ADR-044 yedi semantic fazdaki 17 tarihsel bug kaynağını K-kimliği,
kesin tanı spanı, program exit'i ve stdout ile birebir manifestledi; faz
matrisine ayrı 22. `Semantic regression` dilimini ekleyerek B-039'u kapattı.
K-148/ADR-045 dokuz parse/checker/HIR/runtime/yürütme/LSP/bellek yüzeyini
iki ısınma+25 turluk ham dağılım, p50/p95, JSON/Markdown ve sürümlü TSV
tarihçesine bağladı. Shared CI sonucu yalnız summary+90 günlük artefakttır;
hard eşik ancak sabit adanmış runner'da açıkça etkinleşir. B-040 kapandı.
K-149/ADR-046 bütün production Rust ağacını bugün 38 sorumluluk sahibine, exact
doğrudan kenar tabanına ve temel→adaptör katman yönüne bağladı. Morfoloji→
paket SHA-256 ve tedarik→runtime takvim ters bağımlılıkları `guvenlik` ve
`zaman` temel sahiplerine taşındı; B-057/V1-P0-32 kapandı. K-150/ADR-047
tanı↔kaynak bütçesi ve checker↔HIR SCC'lerini iki bağımsız model sahibiyle
kırdı. Son paket/registry/tedarik SCC'si önce tarih sonlu izinle görünür borç
yapıldı; K-160/ADR-059 saf paket modeli, resolver, registry taşıması,
verification ve yayın orkestrasyonu sahiplerine ayırarak bu borcu kapattı.
Production SCC ve izin sayısı sıfırdır; B-058/B-070 ve V1-P0-33 kapalıdır. Üçüncü dış
incelemenin K-151/ADR-048 adımı bütün workflow action'larını immutable SHA,
birebir pin kaydı ve kontrollü Dependabot PR'ına bağladı; B-059 kapandı.
K-152/ADR-049 performans tarihçesini tam Git SHA, ayrı milestone, temiz Git
ağacı, OS/CPU/RAM/Rust/release profili ve ölçüm başına gerçek örnekleme
semantiğine yükseltti. RSS'in 25 tur değil tek süreç-tepe görüntüsü olduğu
makine-okunur; eski K-148 sayıları exact üretici commit'ine bağlandı ve B-060
kapandı. K-153/ADR-050 in-process engine initialize ile gerçek dillsp process
spawn→stdio→capabilities maliyetini ayırdı; Tier-1 ikili testi ve CI ölçümü
hazırdır. Exact temiz `a2693d6…` uygulama commit'indeki 25 örnek process için
p50 1,557 ms/p95 1,997 ms tabanını verdi; B-061/K-153 kapandı. Bağlayıcı
makine sırasında K-154/ADR-051, 2k/5k/10k/20k tam-metin `didChange` eğrisini
ve p95 250/500/1000 ms ilk-aşım raporunu ekledi. Bugünkü invalidation sınırı
tam belge→tam lexer/parser→resolver/checker→typed-HIR yeniden kurulumudur;
incremental cache ve bu dilimde optimizasyon yoktur. Exact temiz 25 örnekli
`59580cd…` uygulama tabanında 2k/5k/10k/20k p95 sırasıyla 167,281 ms /
1.081,746 ms / 4.669,372 ms / 20.159,636 ms'dir; 250/500/1000 ms çizgilerinin
üçü de ilk kez 5k'da aşılır. B-062/K-154 kapandı. K-155/ADR-052 semantic
regresyon manifestini exact `fixed_by`, kanıtlıysa `introduced_by` ve garanti
sürümüyle v2'ye taşıdı; v1→v2 soy ağacını ve fixture'sız compiler bug fix'ini
fail-closed korur. K-155A/ADR-053 başlık regex'ini kaldırarak sabit başlangıçtan
sonraki bütün compiler kaynak commit'lerini explicit semantic sınıf, kanıt ve
gerekçeye bağladı; bugfix exact fixture ister. K-176/ADR-054 de strict form
çözümünü exact `32247c7…` uygulama commit'ine bağlı web fixture'ı ve semantic
beyanla kapattı. K-156/ADR-055 her hedefin coverage korpusunu cache dışı,
SHA-256 ve run/commit provenance'lı 90 günlük artefakta bağladı; review'lü
küçültme akışı kalıcı repo seed'ini korur. K-157/ADR-056 dört hedefi ayrı RC
işinde explicit AddressSanitizer'a bağladı; ilk exact taban 30'ar dakikada
toplam 136.789.564 yürütme, sıfır crash/timeout/bulgu ve 3/3 seçili Miri testi
üretti. K-158/ADR-057 tekil birincil sahipliği korurken kritik test
seçicilerinin gerçekten yürüttüğü ek fazları bildirip çapraz kanıtı mimari
aşağı akıştan ayırdı. B-063/B-064/B-065/B-066/B-067/B-068 kapandı; sırada
K-159/ADR-058 desteklenen Rust gömme sözünü exact `dil::api::v1` facade'ına
taşıdı; kök legacy modüller açıkça internal sınıflandı ve yeni ihracat allowlist
kapısına bağlandı. K-160/ADR-059 paket modelini davranıştan, resolver'ı taşıma
ve imzadan, registry'yi verification'dan ve yayın orkestrasyonunu güven
mekanizmasından ayırdı. B-069/B-070 kapandı ve CORE FREEZE başladı. Yeni
compiler özelliği gerçek dogfood ihtiyacı kanıtlanmadıkça reddedilir; sırada
K-161/K-162 insan kanıtı ve K-163 ilk gerçek Zee ürünü hattı vardır; K-163
F032 ile kapanmıştır.
K-160A bu sözü `core-freeze-beyanlari-v1.tsv` ve CI koruğuyla executable yaptı:
semantic feature yalnız dogfood/security/correctness sınıfında; dogfood ise
ürün, K-işi, reproducer, etkilenen proje, minimalite ve normatif kararın
tamamıyla geçer. B-071/V1-P0-35 kapandı. Eski `tedarik` cephesi K-167/ADR-064
uyumluluk politikasıyla DEP-004 olarak kaldırıldı; B-072 kapandı.
B-073 de ilk dogfood-kaynaklı compiler değişikliğinden önce kapandı. Append-only
ürün kaydı, exact harici ürün provenance'ı, etkin repo-içi kanıt kökü, gerçek
K-işi, kök-altı Zee dosyası ve karar belgesi referansı artık freeze koruğunda
zorunludur. Reproducer'ın ürün ihtiyacına semantik uygunluğu bilinçli olarak
insan review'unda kalır; CI kolay sahte metadata'yı daraltır ama insan hükmünü
taklit etmez.
K-163 bu sınırlar korunarak `legacy-panel` deposunun temiz
`codex/k-163-legacy-panel` dalında başladı. `43171b9 → cef5ae3` ürün hattı
495 satır Zee ile duyuru listeleme, Argon2id yönetici girişi, rol, CSRF CRUD,
JSON site ayarları ve slug doğrulamalı taslak/yayında sayfa yaşam döngüsünü
birleştirir. HTTP/eylem, görünüm ve sayfa deposu üç sorumluluk birimidir.
`dil denetle`, 7/7 hermetik test ve gerçek TCP içerik+ayar+sayfa provası
geçmiştir. İlk lifecycle provası son kayıt silinince boş CSV şemasının
kaybolduğunu buldu; mevcut dilde kanonik başlık yazımıyla kapatılıp hermetik
teste alındı. Dosya depoları geçici ürün adaptörüdür; PostgreSQL, migration,
medya ve template sürtünmeleri ölçülmeden compiler ihtiyacı sayılmaz. K-163'ü
500–1500 satırlık gerçek uygulama ve süreli ergonomi günlüğü kapanışa taşır.
K-163 dördüncü dilimde ilk kanıtlı core genişlemesini açtı. RFC-0026/ADR-060/
spec-25 ile PostgreSQL secretsiz exact loopback bildirimi, parameterized TEXT
bind, structured C025/23505, eylem transaction/savepoint'i ve değişmez
migration geçmişi tamamlandı. legacy panel kaynağı dosya slug ön-kontrolünden gerçek
PRIMARY KEY'e geçti; PostgreSQL 16.11'de apply→skip→hash reddi, injection
benzeri değer, UNIQUE dalı ve outer rollback gerçek koşuda geçti. Production
TLS, pool ve multi-process yarışı ayrı açık dogfood kanıtıdır.
Beşinci dilimin `43d04fc` ürün commit'i 509 Zee satırında ilk dayanıklılık
kırığını ölçtü. Bağlantı daha önce hiç kurulamamışsa kapasite geri geldiğinde
restart olmadan iyileşir; kurulmuş client backend sonlandırması sonrası yeniden
bağlanmıyordu. Ardındaki correctness değişikliği yalnız transaction dışı salt
okumaya kapalı-client invalidation, tek reconnect ve tek SELECT tekrarı verdi.
Gerçek PostgreSQL 16.11 provasında öldürülen backend'den sonraki ilk istek yeni
backend'e bağlandı ve sentinel'ı süreç restartı olmadan gördü; süreler, PID'ler
ve temizlik `78cce11` ürün commit'indedir. Write olumsuzu
otomatik tekrar ve satır üretmedi fakat C021 ile uygulamayı sonlandırdı;
COMMIT-kaybı F028'e kadar gerçek enjeksiyonla kanıtlanmamıştı. F027,
transaction sınırı C021'ini 503'e çevirip client'ı
geçersiz kılarak worker'ı sonraki bağımsız read/write için yaşattı; aynı write
tekrar edilmedi ve savepoint durumu sızmadı. Exact saha kaydı `00a659a` ürün
commit'indedir. F028 tipli `db.commit_unknown` → C027 hattını ekledi; wire
proxy COMMIT öncesi kesintide 0, uygulanmış COMMIT'in ReadyForQuery yanıtı
yutulduğunda 1 satır üretirken iki durumda da aynı 503, sıfır retry ve worker
survival gözlendi. Yeni bağlantıdaki iş anahtarı sorgusu ve UNIQUE, bilinçli
aynı-anahtar girişimini tek kayıtta tuttu. Gerçek pool ile medya+DB uzlaştırma
K-163'ün açık production kanıtıdır; exact F028 ürün kaydı `8a12848`
commit'indedir. 492 LOC tabanı;
modül/test/dokunulan dosya, bugfix süresi, sürtünme, workaround ve LSP p95 için
aynı yöntemli 1000+ LOC skor kartına bağlandı.
F029 ürün commit'i `b3cee9a`, 690 Zee satırı ve 4 modülde DB metadata ile
fiziksel dosyayı ayrı eylemlere bölen `hazırlanıyor → hazır/hatalı` protokolünü
kurdu. PostgreSQL `hazır` geçişi reddedildiğinde kalan dosya+metadata sonraki
uzlaştırmada başarıyla tamamlandı. Gerçek dizin çakışmalı dosya yazımı ise
metadata'yı `hazırlanıyor` bırakırken C013 ile worker'ı sonlandırdı. Bu bulgu
web'de request-local C013 → 503, sıfır retry ve worker survival correctness
sözünü açtı. Binary hash/upload ve fiziksel silme/orphan tarama F030'da;
production TLS/pool ise F031'de kapanmıştır.
Zee `395b2e9` değişikliğinin ardından exact final ürün kanıtı `29d30c8` oldu:
C013 503/no-retry verirken worker sağlık isteğinde 200 kaldı; eksik dosya
metadata'sı `hatalı`ya, finalize reddiyle kalan kayıt process restartından
sonra `hazır`a uzlaştırıldı. Aynı kodlu tekrar UNIQUE ile reddedildi ve dosya
hash'i değişmedi. Böylece F029 metin-payload yaşam döngüsü kapandı.
F030, ürünün gerçek binary ihtiyacıyla RFC-0027/ADR-061/spec-26'yı açtı.
Native octet-stream 16 KiB parçalarla temp dosyaya ve SHA-256'ya akar;
no-clobber publish, idempotent silme, listeleme ve hash beklenen hataları
`Sonuç` olarak taşır. legacy panel 970 Zee LOC/4 modül/10 testte normal yaşam
döngüsünü ve temp, DB-hazırlama, rename, tombstone kesmelerinin restart
yakınsamasını PG16.11 üzerinde kanıtladı. F030 kapalıdır. F031/ADR-062 pinned
CA+hostname, yanlış CA/IP reddi, 4 bağlantılık exhaustion, stale replacement,
idle/lifetime ve shutdown ölçümüyle production TLS/pool sınırını da gerçek
PG16.11 üzerinde kapattı. Organik 1000+ LOC bakım kontrolü F032'de kapanmıştır;
managed-provider rotasyonu ve çok-worker toplam bütçe release
tatbikatında açık kalır.
Exact ürün saha kaydı `977cd2a` commit'indedir.
F032/ADR-063 ürün kontrollü yanıtı en küçük yüzeyle açtı: durum yalnız
100..599 literalidir; dinamik response/header builder yoktur. legacy panel'nın
DB-bağımsız `/canli` rotası backend kapalıyken 200 kalırken `/hazir` 503 oldu,
PostgreSQL dönüşünde aynı worker 200'e iyileşti. F014'ün yayımlanmamış/yok
içerik için 200 workaround'u 404 ile emekli edildi. Ürün organik 1037 Zee
LOC/5 modül/12 teste ulaştı; gerçek workspace LSP full-change p95 41,416 ms,
10 dogfood commit'inde dosya medyanı 5,5'tir. Exact final ürün kanıtı
`bb8e1ac`tır. Böylece K-163/V1-P0-36 kapanır; managed-provider/çok-worker
tatbikatı K-169/K-173'e aktarılır; içerik/malware politikası K-170/ADR-066
ile kabul edilen sınır (GB-019) olarak kayıtlıdır.
B-001 çağrı sözdizimi ile B-002 gezme zihinsel modeli gerçek insan
kanıtı gelmeden kapatılmaz veya yeni syntax kararıyla atlanmaz.
# 40. Proje felsefesinin korunması
Bu projenin başarısı yalnız compiler’ın çalışması değildir. Başarı; Türkçe konuşan bir çocuğun yabancı syntax bariyerine takılmadan algoritmik düşünceyle tanışması, aynı dilin yıllar sonra onu terk etmeye zorlamaması ve ekosistemin tek bir şirketin kapalı ürünü haline gelmemesidir.
Her yeni özellik şu dört sorudan geçmelidir: Türkçe doğal mı? Deterministik mi? Öğrenilebilir mi? Profesyonel ölçekte savunulabilir mi? Dördünden biri hayırsa özellik yeniden tasarlanır.
# 41. İlk gerçek milestone
İlk kutlanacak milestone web sitesi, logo veya paket registry değildir. Tek bir dosya:
merhaba.dil
ve tek satır:
"Dünyaya merhaba" yaz
Bu dosya kendi lexer/parser/type checker zincirimizden geçip çalıştığında proje doğmuş sayılır. İkinci sembolik milestone, compiler’ın kendi dilinde yazılmış bir parçasını derlemesidir. Son büyük sembolik milestone ise kendi compiler’ının kendisini derlemesidir.
# 42. Son söz
Amaç yalnız yeni bir syntax üretmek değil; Türkçe konuşan insanların bilgisayara kendi dillerinin düşünme ritmiyle kesin talimat verebildiği, öğrenme ile profesyonel üretim arasındaki duvarı azaltan kalıcı bir teknik eser bırakmaktır.
Projenin temel cümlesi değişmez: Çocukların kullanabileceği bir dil yapacağız; çocuk dili yapmayacağız.

K-195 Windows dağıtımında statik CRT ve iki temiz klon üretimi uygular; exact `c7e3d04` kaynağında üç platform eş-hash/DLL/program/LSP kanıtı geçti. Native backend ve self-hosting tamamlanmış değildir.

K-195 ilk Windows çift-klon koşusu eş-hash kapısında kaldı; artefakt
üretilmedi. Yeni MSVC linker tarifi `/Brepro` ve provenance
`targetRustflags` alanını birlikte günceller; exact tekrar kanıtı gerekir.
Eş olmayan byte'lar ayrı `yayinlanamaz-surum-farki` tanısında saklanır,
sürüm/kurulum kümesine girmez. Benchmark tarihçesine yeni satır yazılmadı.

K-195 kapanma: `c7e3d04adf27905ade8b622d24bd76762123d2dc` kaynağındaki
[38244549913 tatbikatı](https://github.com/itwise-bilisim/zee/actions/runs/38244549913)
üç platformda geçti. Windows iki temiz klonda eş hash, dinamik CRT importu
yok ve kur/program/LSP/kaldır başarılı. İndirilen byte'lar metadata ile
ayrıca eşleştirildi; ham rapor/provenance kalıcı fixture'dadır. İlk Windows
reddi korunur. K-192 temiz Windows/macOS, minimum OS/linker-SDK kanıtı
ayrı açık kalır; native Zee backend ve self-hosting tamamlanmış değildir.

K-196 linker/SDK kayıt dilimi exact üç platform kanıtıyla kapandı: release iki klonun gerçek link
komutunu, program path/hash/version ve SDK referanslarını provenance'a
bağlar. Unix `-v`, MSVC `/VERBOSE:LIB` tarifte görünür; Python 3 build kanıtı
aracıdır, runtime bağımlılığı değildir. Eksik kanıt yayımlanmadan reddedilir.
Exact b153064 kaynağı/run 38252942210 üç platformda geçti; minimum OS/temiz Windows-macOS ve
bütün native C araçlarının kaydı tamamlanmış gösterilmez. Performans
baseline'ı/tarihçe satırı üretilmedi; ADR-045/049 kuralları korunur.

K-192 ek kanıt: exact `e0f5363` Linux ikilileri AlmaLinux 9.8 minimal
userland/glibc 2.34 üzerinde ağsız, read-only, ayrıcalıksız program ve LSP
provasını geçti; Rust paketleri/PATH araçları yok. [Ham rapor](temiz-almalinux-runtime-kaniti-2026-10-12.json).
Bu seçili amd64 emülasyon provası tüm dil IO davranışları, minimum kernel
ve temiz Windows/macOS garantisi değildir; K-192 kısmen açık kalır.

K-197 temiz Windows runtime provası exact kanıtıyla kapandı; exact release kaynağı/hash, immutable Nano Server image,
ContainerUser/network none/read-only bind ile program ve LSP denenir.
Nano Server program/LSP sonucu geçti; masaüstü/minimum OS ve bütün IO
desteği ayrı açıktır. Release/prova kaynak SHA'ları ayrı kaydedilir.
ADR-068/GB-034; spec/RFC/tanı/API/production Rust katmanları ve benchmark
tarihçesi değişmez.

K-197 kapanma: release c7e3d04, prova 27c0fa9/run 38253777857/job
114826470199. [Ham rapor](temiz-windows-runtime-kaniti-2026-10-12.json),
[defter](temiz-windows-platform-kaniti-v1.tsv); Nano Server 10.0.26100.33438
program/LSP geçti. Kurulum/kaldırma, desktop/minimum OS ve bütün IO ayrı.

K-196 kapanma: b153064/run 38252942210; iki klon eş hash ve gerçek
kur/program/LSP/kaldır üç platformda geçti. GCC 13.3.0, Apple Clang 21.0.0
+ SDK 26.5, MSVC FileVersion 14.51.36260.0 + SDK 10.0.26100.0 dört
link kaydında doğrulandı. Ham runtime/provenance ve artifact hash'leri
[platform defterinde](derleme-platform-kaniti-v1.tsv) korunur. K-192 minimum OS, bütün native C
araçları ve fiziksel temiz makine kanıtı ayrıca açıktır.

K-198 temiz Server Core kurulum/kaldırma dilimi exact kanıtla kapandı; exact source/
byte/script hash'leri, Türkçe boşluklu ContainerUser dizini, manifest
path/hash/source, program/LSP ve kaldırma yokluk kanıtı istenir. Gerçek
Windows koşusu geçti; desktop/minimum OS/fiziksel macOS ve bütün IO
tamamlanmış gösterilmez. ADR-068/GB-035. Spec/RFC/tanı/API, production
Rust katmanları, release dosya kümesi ve benchmark tarihçesi değişmez.

K-198 kapanma: release b153064/prova 33176cf/run 38255788989/job
114833388588; stock Server Core 10.0.26100.33438'de Rust kurulmadan
Türkçe dizine kur/program/LSP/kaldır geçti. [Ham rapor](temiz-windows-kurulum-kaniti-2026-10-12.json)
ve [kimlik/hash defteri](temiz-windows-platform-kaniti-v1.tsv) kalıcıdır.
ContainerUser/network none/read-only bind; host kernel, fiziksel/desktop
minimum OS ve tüm IO kapsamı ayrı açıktır.

K-199 Cargo compiler/wrapper pin düzeltmesi üç platform exact kanıtıyla kapandı: helper
exact kurulu rustc executable path/version'ını Cargo RUSTC'ye bağlar ve
wrapper/config seçimini boş değerle kapatır. Önce helper 1.93.1 bildirip
Cargo RUSTC ile 1.97.1 çağırabiliyordu. Resmî release/offline girişi
ambient compiler/wrapper seçimini korumaz; custom geliştirme wrapper
kendi Cargo çağrısıyla kullanılır. Gerçek küçük Cargo poisoned ENV/config
provası exact 22ea2b8/run 38259341995 üç platformda geçti. ADR-068/GB-036; dil spec/RFC/
tanı/API/production Rust kenarı ve release dosya kümesi değişmez.

K-199 kapanma: 22ea2b8/run 38259341995; Linux/macOS/Windows gerçek
Cargo/build-script/ikili Rust 1.93.1, empty wrappers ve temiz source
kanıtı [kalıcı defterde](rustc-pin-platform-kaniti-v1.tsv). Helper/prova/pin
hash'leri kaynakla ayrıca eşleşir. Tam Zee release ve bütün Cargo
profil/linker/native C ayarları bu dar probe'un kapsamı değildir.

K-200 macOS C/Rust deployment uyum kapısı kapalı: final minos 11.0
bildiren eski derlemede C nesnesi 27.0 idi. Resmî release exact Rust
default deployment'ını C/Rust'a ortak export eder; iki klon native .o
başlık/hash kanıtı provenance'a girer. Yerel sonrası 24/24 nesne 11.0
ve program/LSP geçti. Exact kaynak `44fba404c79762aff1698a07978c79ea069a59df`,
run 38263264133 üç platform iki-klon/kurulum/program/LSP/kaldırma kapısını
geçti; macOS a/b klonlarında ayrı ayrı 24 nesnenin minos değeri 11.0. Gerçek
macOS 11/minimum OS/tüm native alt süreç desteği değildir. ADR-068/GB-037;
spec/RFC/tanı/API/production Rust katmanı ve release dosya kümesi değişmez.

K-201 native C/archiver driver çağrı kaydı hazırdır; iki klonda compile ve
archive zorunlu, paket/argüman/executable path/hash ve sürüm sorgusu sonucu
provenance'a eklenir. Exact kaynak 25b6882/run 38349601833 üç platform release kanıtı geçti.
pkg-config/vcpkg, driver iç alt süreçleri ve minimum OS K-192'de ayrıca
açık kalır. Ayrıntı [derleme rehberinde](derleme-ve-bagimliliklar.md).

K-202: ana GitHub/CI deposu `merbay-erp/zee`; `itwise-bilisim/zee` yalnız
ayna ve repository Actions kapalıdır. SBOM/provenance yeni ana kaynak
URI kullanır; eski CI/fixture kanıtları tarihsel bağlantılarıyla korunur.

K-204: sır taramasının dört kamuya açık belge yanlış pozitifine yalnız
immutable fingerprint/kaynak satırı hash doğrulaması uygulandı; yeni
anahtar exit 1/REDACTED testi, default kurallar ve iki tarama korunur.
K-202 beyan sonrası türetilen kanıt özeti yenilendi.

13 Ekim [dış inceleme eşlemesi](dis-inceleme-2026-10-13.md) K-205 olarak
izlenir. PostgreSQL ön-limit sonuç toplama K-206, ad çözümleme/LSP maliyeti
K-207 dilimidir; gerçek kullanıcı kanıtı ve core freeze sınırı sürer.

K-207 başarılı ad çözümlemesini hash üyeliğine taşıdı; full ad listesi
yalnız A001 önerisinde üretilir, A002 ve exact-ad kararı değişmez. Oracle
26.624 karar, golden/morfoloji/LSP ve mimari kapıları geçti; exact önce/
sonra performans kanıtı açık. K-206 PG sonucu bellek deneyi GB-041 olarak
orta/açık kaydedildi; 16 MiB sonuç bütün driver belleği tavanı değildir.

K-207 ilk exact ölçümü: 29be85b → 94de420 aynı M4/Rust/release/25+2
koşulunda 20k p95 22.169,906 → 2.961,638 ms. Raw sample ve tarihçe
[ölçümlerde](olcumler.md). Kapsam maliyeti ve 1 sn hedefi açık;
incremental parser/cache veya genel platform SLA eklenmedi.


K-207 kapsam dilimi kapalı: temiz exact 9e80b69 release/25 örnek/2 ısınma
20k LSP p95 165,137 ms; aynı exact kaynakta 1800 sn/264.343 döngü soak
geçti. Derleyici RSS medyanı 8.144→8.160 KiB, LSP 19.088→17.568 KiB.
Ham örnek/çevre ve tarihçe [ölçümlerde](olcumler.md). 728/728 faz,
26.624 ad oracle ve 4.096 kapsam işlemi oracle karşılaştırması geçti.
K-207 KISMEN; full didChange ve incremental/cache kapasitesi açıktır.

K-206 native okuma akış toplamasına geçti: sonuç sınırından sonra çıktı
biriktirilmez; borrowed metin kopyalanmadan denetlenir. Hata önceliği ve
mevcut read retry/transaction/API korunur. Tek wire satırı/driver RSS
bütçesi GB-041'de açık; exact native kabul ve RSS kaydı aşağıdaki kapanışta saklıdır.

K-206 akış toplama dilimi kapalı: source f2ba654, temiz beyan/ölçüm
6463a7f; 731/731 faz ve gerçek native oracle/transaction kabulü geçti.
64×1 MiB sonuçta native süreç RSS 128,77→23,81 MiB, tek 64 MiB satırda
136,77→72,72 MiB; üç vakada sonraki sorgu başarılıdır. Bir örnek/sıfır
ısınma gözlemi, genel RSS garantisi değildir. [Ham kanıt](incelemeler/k206/README.md).
**K-206 KISMEN:** tek wire satırı/driver sınırı ve GB-041 açık kalır.

K-206 tedarik hazırlığı: upstream protokol Rust src byte'ları değişmeden
local patch kimliği compiler/fuzz lock ve doğrulanmış SPDX kaynak/tree
hash ile kaydedildi. GB-042 kapalı; GB-041/erken wire sınırı açık.
Offline vendor ve iki-clone release kabulü kapanış öncesi beklenir.

K-206 path patch iki-clone kabulünde reddedildi; yalnız kaynak kimliği
exact ana repo Git SHA pinine çevrildi. Protokol davranışı değişmedi.
SBOM yerel + Git blob tree hash eşitliği ve exact lock source ister;
v2 kayıt etkin, v1 tarihsel. Yeniden offline/release kanıtı beklenir.

K-206 dependency kimliği dilimi kapalı: source 9dae3e6, temiz beyan/
ikili üretimi ab8e97e. 735/735 faz, git pin/blob/tree SBOM, offline vendor
163 paket ve iki temiz klonda eş dil/dillsp SHA-256 geçti. Yerel macOS
kabulüdür; [ham kanıt](incelemeler/k206/source-identity/README.md).
Path kaynaklı başarısız iki-clone koşusu da saklanır. Protokol src byte'ları
upstream ile aynıdır; GB-041/32 MiB erken wire guard hâlâ açık kalır.

K-206 erken header kabul tasarımı: iki ayrı backend parser'ı kapsayan
typed 32 MiB aday yama bağımsız geçici crate'te 34/34 testle doğrulandı.
Aktif dependency pin değişmedi; üretimde mesaj sınırı etkin değildir.
Transaction durum temizliği, read retry engeli ve exact immutable kaynak
entegrasyonu açık; GB-041 kapatılmadı. [Kabul tasarımı](incelemeler/k206/frame-guard/README.md).

K-206 aday protokol kabulü clean c68af0f üzerinde kapandı: iki parser'da
34/34 bağımsız lib test, native 5 vaka, typed io cause ve kopan transaction
yazısının kalıcı olmaması doğrulandı. Ham JSON/log [kayıttadır](incelemeler/k206/frame-guard/README.md).
Bu yalnız adayın kabulüdür; aktif pin, Zee session/retry/RLS cleanup
entegrasyonu ve GB-041 açık kalır. Compiler fazı 735/735 geçti.

K-206 wire entegrasyonu: compiler/fuzz protokolü exact 1ffdb584 .2 kaynağına
pinlidir. İki parser'da 32 MiB length reddi, typed C025 verisi, read retry
engeli ve geçersiz transaction/request-local cleanup uygulanır. Gönderilmiş
COMMIT sonrası bağlantı kaybı C027 olarak korunur. Native loopback kabulü
okuma, büyük Notice üreten write, nested/RLS ve deferred COMMIT'i kapsar.
GB-041 bu kaynakta uygulandı; exact temiz kaynak, vendor/iki-clone ve
sürekli kapı kabulü henüz beklenir. Global RSS sözü verilmez.

K-206 wire security dilimi kapandı: source 7836b73, clean beyan/kanıt
a4e1045; native 13 vaka, 736/736 faz, offline vendor 163 ve iki-clone
release geçti. GB-041 kapalı; global RSS garantisi ve sürekli geniş
fault-proxy matrisi değildir. Exact ham kanıt
[buradadır](incelemeler/k206/frame-guard/README.md).

## K-208/K-209 — Zee çekirdek legacy giriş ve audit IP

Gerçek ürün talebi ve exact köken dogfood/zee-cekirdek altında korunur.
K-208 bcrypt $2a/$2b/$2y yalnız doğrulama (maliyet 4–14, 72 UTF-8 byte
sınırı), K-209 trusted `istemci adresi` Metin ifadesidir. Node vektör,
real worker proxy/socket/IPv6/spoofed XFF, CLI boş değer ve replay testleri
mevcuttur. RFC-0030 geçici kabul/ADR-082; yeni özet Argon2id kalır.
Ürün upgrade/migration/audit saklama işini kendi tarafında yapar.
K-174 syntax gözlem penceresi gerçek K-209 grammar değişikliği nedeniyle
2026-10-13–2026-11-10 olarak yeniden başladı; B-001/B-002 açık.
Tam faz, source beyanı ve vendor/release kapanışı beklenir.

K-208/K-209 kapandı: feature source 2c355ef, clean beyan/ikili kanıt
daaa3ea. 742/742 faz, Node crypto ve gerçek HTTP/CLI/trace kabulü,
167 paket offline vendor ve iki-clone binary eşitliği geçti.
[Ham kayıt](incelemeler/k208/README.md) yerel macOS kanıtıdır; V1
usability/RC/signed release ve sunucu dağıtımı değildir.

### ürün E özelliklerinin ana kaynak hattı — K-213

[ADR-086](../adr/086-site-hatlarinin-birlestirilmesi.md) mevcut ürün E
parola üretimi/TOTP/SMTP, rota birimi, görünüm, görsel ve gizli değer
yüzeylerini K-208/K-209 bcrypt/IP ile aynı hatta toplar. Ürün kabulü
dosya kontrolüyle değil bütün proje `dil dene .` ile alınır. Ham ürün E
şablonu normal katalog üretimini ister. Exact source/CI kapanışı beklenir;
canlı dağıtım ve otomatik parola göçü bu işte yapılmaz.

K-213 clean source kabulü: source e09493e, beyan/ikili/test 0531432;
769/769 faz, beş bütün proje 242/242, 187 paket offline vendor ve
macOS arm64 iki-clone eşliği geçti. Exact [kayıt](incelemeler/k213/README.md).
İlk Linux CI'ın test Git kimliği bağımlılığı düzeltildi; son uzak CI
kapanışı beklenir. Canlı dağıtım ve otomatik parola göçü açık kalır.

K-213 compiler gerçekleme/test/belge dilimi kapandı. Eski ürün E
sunucu hattı ile master aynı kaynak geçmişinde; ürün CI/sunucu pininin
bu kaynağa geçirilmesi ve canlı giriş/IP kabulü ürün tarafının işidir.
Otomatik bcrypt→Argon2 ürün göçü henüz uygulanmadı.

### K-214: Anonim açık depo kanıtı

Müşteri ve altyapı kimliği anonimleştirildi; yeni içerik hash politikasıyla
CI içinde denetlenir. Eski geçişler maskeli dökümle ayrı kullanıcı kararını
bekler. Tarihsel ölçümler eski kaynak kimliklerini korur; yeniden açılış ve
GitHub PR önbelleği temizliği ayrıca onay/teyit ister. [Kayıt](incelemeler/k214/README.md).

### K-215: Önceki ürün geçişleri ve bağımsız yerel yama

K-214'te ayrı bırakılan geçişler kullanıcı onayıyla anonim kaynak soy ağacına
taşınır; eski fingerprint istisnaları kaldırılır ve ilk commit dahil tüm ata
geçmişi denetlenir. Compiler/fuzz PostgreSQL yaması path bağımlılığıdır;
derleme self Git deposu veya credential gerektirmez. SBOM gerçek build
commit'ini ve yerel paket baytlarını kayıtlı SHA-256 ile doğrular.
Tarihi ölçümler yeni kaynakta koşulmuş sayılmaz. GitHub PR refs/cache teyidi
ve yeniden açılış onayı açıktır; depo gizli kalır.
