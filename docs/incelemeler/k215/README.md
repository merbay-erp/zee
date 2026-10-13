# K-215 — Önceki ürün geçişleri ve bütün yayımlanmış geçmiş

K-214 sonrasında kullanıcı önceki geçişlerin de temizlenmesini onayladı.
Yayımlanmış ana dal, üç eski dal, iki birleşmiş dal ve site dalı ile etiketler
bağımsız yedekli klonda `git filter-repo` ile anonimleştirilir. Aktif yerel
çalışma ağaçları sıfırlanmaz. Merge ebeveynleri, K/vaka kimlikleri, gerçek
harici ürün SHA'ları ve üretim davranışı korunur.

Eski panel kanıtı `dogfood/legacy-panel` kimliğine taşındı. Gerçek ürün adı,
özel kaynak deposu adı ve veritabanı/ortam değişkeni örnekleri anonimdir.
Ürün gereksinimi ve exact harici provenance özel ürün deposunda kalır.
Eski maskeli konum listeleri tarihsel envanterdir; istisna listesi değildir.

## İstisnasız kapı

`history_scope=all-reachable`: ilk commit, silinmiş dosyalar ve commit mesajları
dahil seçilen dalın bütün ataları taranır. `legacy_fingerprints` boş olmak
zorundadır. Eski müşteri kaydını istisna olarak geri ekleme ve eski taban
öncesinde saklama olumlu/olumsuz hermetik testlerle reddedilir. Terim/yol
sözlüğü hash biçimindedir; eşleşme değerleri ve dosya adı loglanmaz.
Yayımlanacak her dal/etiket ayrıca izole klonda bu aynı politikayla taranır.
GitHub'ın yönetilen PR refs ve cache nesneleri kullanıcı Git push'u ile silinmez.

## Yerel PostgreSQL yaması

Compiler `third_party/postgres-protocol`, fuzz `../third_party/postgres-protocol`
path yamasını kullanır. Cargo/lock Git self kaynağı ve CI global URI yönlendirmesi
kaldırılır; deny politikası Git kaynağına istisna vermez. Paket sürümü ve
kanonik ağaç SHA-256'sı değişmez. Yeni SBOM, kayıtlı paket için hem exact build
commit ağacını hem Cargo'nun derlediği disk ağacını aynı kayıtlı SHA-256 ile
doğrular; bilinmeyen paket/kaynak, bozuk checksum, eksik commit veya değişen
bayt reddedilir. SBOM URL'si actual build commit'ine bağlanır; registry checksum
uydurulmaz. `patches-v2` kaynak SHA'sı anonim kaynak soy ağacını taşır.

## SHA ve tarihi kanıt

[Eşleme](sha-esleme-v1.tsv), K-214 sonrası yayımlanmış kaynaktan K-215 anonim
kaynağa giden tam SHA zinciridir. K-214 eşlemesinin özgün sol sütunu korunur,
sağ sütunu yeni kaynak graph'ına bağlanır. Compiler/core-freeze beyanları ve
regression sahipleri yenilenir. CI yalnız kayıtlı eski master kimliğini
exact yeni atasına normalleştirir; rastgele force push kapıları atlayamaz.

Benchmark/soak/fuzz/platform/ikili/provenance kayıtları gerçek ölçüldükleri
özgün SHA'yı korur. Bu işlem onları yeni SHA'da yeniden koşulmuş kanıt yapmaz;
eski artefaktların hash/imzası yeniden üretilmiş gibi gösterilmez. Yeni sürüm
yalnız iki temiz klonlu sürüm betiği ve yeniden doğrulanmış SBOM ile çıkar.

## Korunan özel ürün hattı ve dış teyit

Ürün talebinin 17:15 eki uyarınca Enterprise `ci/site-dil` ve
`17b79c951b9eaeaad406da7a08804f5f1962ce09` / özgün `45b66f3` soy ağacı
korunur; bu özel dal force push edilmez/silinmez. Enterprise master yalnız
ana kaynağın aynı commit'teki aynasıdır; Actions kapalı kalır.

GitHub destek taslağı [K-214 dizininde](../k214/github-destek-talebi.md)
genişletilir. PR refs/cache silinmesi destek teyidi bekler (GB-046).
Depo gizli kalır; yeniden açılış ayrı kullanıcı onayı gerektirir.

## Etki incelemesi

Spec/RFC metinlerinde yalnız ürün kimliği anonimleşir; grammar/tanı/ABI/API
değişmez. Spec drift kanıt/raporu ve türetilen dogfood özet/sayılar yenilenir.
ADR-046/katman fixture/SCC graph'ı incelenir: sürüm mühendisliği sahibi aynı,
üretim adaptörüne yeni kenar yoktur. ADR-045/049 ve performans tarihçesi
incelenir; iş yükü/performans davranışı veya ölçüm satırı eklenmez. Action
pinleri ve release dosya kümesi değişmez. ADR-068 SBOM source sözleşmesi
güncellenir; tedarik/offline vendor, faz ve ikili eşitlik kontrolleri yapılır.

Makine doğrulaması ve uzak ref sonuçları takip kayıtlarında yazılır;
tamamlanmamış dış teyit kapanmış gösterilmez.

K-204 dört incelenmiş sır taraması fingerprintinin yalnız kaynak SHA alanı
yeni soy ağacına eşlendi. Satır SHA-256, dosya, tanı ve satır numarası aynıdır;
yeni/geniş istisna eklenmedi. Gerçek yeni anahtar ret testi korunur.

Gizlilik tarayıcısı uzun derleme kayıtlarının bütün ara hash kümelerini
önbelleğe almaz; yalnız bulunan politika eşleşmeleri saklanır ve politika
cache anahtarına dahildir. Uzun kayıt içinde gömülü ad ret kapsamı korunur;
farklı politikalar arasında sonuç paylaşılmaz (13 sentetik test).

Kısa kimlik `word` kuralıyla sözcük sınırında/ayırıcılarla denetlenir;
uzun terimler gömülü adları da bulur. Ürün test dosyası/seçicileri
`urun_e_yetkinlik_testi` olarak anonimleştirilir; vaka ve faz sahipliği korunur.
Kısa rastgele harf dizisinin daha uzun sözcük içindeki çakışması ret değildir;
Rust noktalama işaretleri ayırıcı sayılmaz (13 sentetik test).

Yerel tedarik: cargo-deny/compiler+fuzz, stable fuzz derlemesi ve birleşik
güvenlik kapısı geçti. 186 paket iki vendor üretiminde byte byte aynı;
boş CARGO_HOME ile compiler/fuzz offline derlendi. Vendor manifest SHA-256:
`723f334f8a3b7d259cf2f3f69f97da2fb8f3cdc17f6fc03baec6ed0e87385efd`.

İlk yerel faz matrisi: **770/770 geçti**, 0 kalan, 0 atlanan. Katalog,
belge tazeliği, spec kanıtları, katman/SCC, uyumluluk, dogfood/regression
ve Rust/Go byte eşdeğerliği bu matriste geçti. Clippy/fmt ve Gitleaks
tam geçmiş+çalışma ağacı temiz; 13 sentetik gizlilik testi geçti.

Kod/test/belge kaynağı: `8ac1e2ca770b8e9e98c5f096dd6f5b1024dd3f6f`. Exact compiler/core-freeze
beyanları bu kaynağı izleyen committe kaydedilir.

Son [bütün ref taraması](tarama.json): anonim tarihçe tabanında 18 ref,
406 commit, 5.165 path/blob çifti ve **0 eşleşme**. Müşteri eski satır
istisnası yoktur. Sonraki K-215 politika/kod/beyan dosyaları aynı
politikayla çalışma ağacında ayrıca denetlenir. Uzak CI kapısı tanımlıdır;
aşağıdaki hesap engeli nedeniyle bu yeni kaynakta henüz çalışmamıştır.

## Path yama ikili tekrarlanabilirliği

İlk temiz `b70f426cd29bd971a5aea68405549a273a06a9d8` artefakt denemesi
macOS dil ikilisinde eş olmayan SHA-256 nedeniyle reddedildi; artefakt
yayımlanmadı. Tanı PostgreSQL panic konumlarında Cargo kanonik
`/private/var/.../a` ve `/b` kaynak yollarını gösterdi. Mantıksal `/var`
prefix remap bu yolları kapsayamıyordu. Geçici klon kökü `pwd -P` ile
kanonikleştirilir; Unix sembolik yol testi gerçek betik adımını çalıştırır.
İki bağımsız temiz klon hash eşitliği yeniden doğrulanmadan kapı kapanmaz.

Kanonik yol düzeltmesi sonrası **771/771 faz testi** geçti; yeni Unix
sembolik yol vakası birincil tedarik fazındadır. Clippy/fmt, katalog ve
belge tazeliği temiz. Temiz commit ikili tekrar üretimi ayrıca kaydedilir.

Temiz `b8376349b3645061e7b83338b5f9a28d12f8a8d9` kaynağında [iki klon kanıtı](ikili-esitligi.json)
**GEÇTİ**: dil ve dillsp SHA-256 eş; SBOM, native deployment ve gerçek
build-environment doğrulaması başarılı. Önceki fark denemeleri yukarıda
tarihsel negatif kanıttır; başarısız ikililer dağıtıma çıkarılmadı.

Merbay atomik [ref geçişi](ref-esleme-v1.tsv) doğrulandı: 7 dal ve 11
etiket işlemi, exact-SHA lease ile yapıldı. Baslangıç/doğum etiketleri
değişmedi; v0.1–v0.7 anonim source soy ağacına bağlandı. İlk geçiş
masterı `39f6ca26e20805feb8833ea0fb1a61d7a7249df0`; bunu izleyen belgeler
normal fast-forward commitlerdir. Geçiş sırasında Actions geçici kapatılıp
**enabled=true** özgün izinleriyle geri getirildi. Ref/PR olayları yine
başlatılamayan koşular oluşturdu; bu duraklama bildirimleri engellediği
iddiasını taşımaz. Depo gizli kalır.

## Uzak CI hesap engeli — açık kanıt

`1f61e51ca56c324a914c8f796940d8b982aee3a3` master kaynağındaki üç workflow'un
altı işi, hesap ödemesi/harcama limiti engeli nedeniyle koşucu başlamadan
başarısız oldu. [GitHub API kanıtı](ci-hesap-engeli.json) her iş için
`steps_count=0` ve aynı hesap tanısını kaydeder. Bu sonuç test başarısızlığı
veya uzak test geçişi değildir; Linux/Windows ve uzak güvenlik/gizlilik
kanıtı **AÇIK** kalır. Yerel 771/771 sonucu ve iki temiz macOS klonu kanıtı
ayrı, gerçek sonuçlardır.

Ödeme/limit ayarı kullanıcı tarafından düzeltildikten sonra güncel master'da
`dil`, `tedarik` ve `acik-depo` yeniden çalıştırılmalı ve exact SHA sonuçları
kaydedilmelidir. Ayar veya harcama yetkisi bu iş kapsamında değiştirilmez.
Son belge/dış teyit commit'i `[skip ci]` taşır: yalnız zaten bilinen hesap
engelinde yeni push koşuları üretmemek içindir; kontrol yeşili veya kalıcı
workflow istisnası değildir. Enterprise Actions kapalı kalır.

[Özel ayna geçiş teyidi](ayna-teyidi.json): Enterprise master aynı
`1f61e51ca56c324a914c8f796940d8b982aee3a3` geçiş kaynağına eşitlendi;
iki birleşmiş dal ana depodaki `39f6ca26e20805feb8833ea0fb1a61d7a7249df0`
ile aynı. Actions önce/sonra kapalı, `ci/site-dil` pini ve özgün ebeveyni
değişmedi. Bu teyidi ekleyen normal belge commit'i de iki master'a
fast-forward olarak gönderilir; diğer özel ürün dalları sıfırlanmaz.
