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
kapattı (RFC-0017, spec/12). Çok süreçli ortak oturum deposu, rate limit,
secret dağıtımı ve idempotency ayrı deployment/RFC kapılarıdır.
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
Limitli uzak taşıma, kalıcı metadata/cache, offline hit/miss, exact manifest/
kilit ve CLI henüz tamamlanmamıştır; V1-P1-07 açık kalır.
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
Playground CPU/RAM/süre/IO limitleri.
Paket kurulumunda keyfi post-install varsayılan olarak yasak.
Secret değerlerin loglanmasını engelleyen tür/capability yaklaşımı ileri araştırma konusu.
# 19. FFI ve mevcut ekosistem
Yeni dil ilk günden bütün kütüphaneleri yeniden yazamaz. C ABI birincil köprü adayıdır. Rust/C/C++ ve platform API’leri için generator/adapters sonra gelir.
FFI unsafe boundary.
Harici İngilizce API’ler Türkçe wrapper ile sunulabilir.
ABI sürümleme politikası.
Native paketlerde OS/architecture metadata.
Ownership, lifetime ve crash sınırları açıkça belgelenir.
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
Unicode doğrulaması koydu. K-108/ADR-020 her semantic HIR ifadesinde kaynak
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
K-114/ADR-025 ile başlayan ve K-130'da 150 etkin + 3 ayrılmışa çıkan tanıların
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
kalan süreyi yeniden hesaplar ve dolmuş adaptörü başlatmaz. Web istek
yanıtı+oturum mutation transaction'ı K-134'e açıktır.
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
K-132 LSP outbound JSON'unu bounded üretip B-025'i kapattı. K-133 etki öncesi
deadline kapılarıyla B-026'yı kısmen kapattı. Sıradaki omurga K-134 web istek
yaşam döngüsü transaction'ıdır.
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
