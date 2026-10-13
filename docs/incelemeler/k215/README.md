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

Kısa marka kimliği için hashli `word` kuralı kelime sınırında ve ayırıcılarla
taranır; uzun terim kuralları gömülü adları da reddeder. Kısa rastgele harf
dizisinin daha uzun sözcük içindeki çakışması ret değildir (13 sentetik test).

Kısa kimlik `word` kuralıyla sözcük sınırında/ayırıcılarla denetlenir;
uzun terimler gömülü adları da bulur. Ürün test dosyası/seçicileri
`urun_e_yetkinlik_testi` olarak anonimleştirilir; vaka ve faz sahipliği korunur.

Yerel tedarik: cargo-deny/compiler+fuzz, stable fuzz derlemesi ve birleşik
güvenlik kapısı geçti. 186 paket iki vendor üretiminde byte byte aynı;
boş CARGO_HOME ile compiler/fuzz offline derlendi. Vendor manifest SHA-256:
`723f334f8a3b7d259cf2f3f69f97da2fb8f3cdc17f6fc03baec6ed0e87385efd`.

Son yerel faz matrisi: **770/770 geçti**, 0 kalan, 0 atlanan. Katalog,
belge tazeliği, spec kanıtları, katman/SCC, uyumluluk, dogfood/regression
ve Rust/Go byte eşdeğerliği bu matriste geçti. Clippy/fmt ve Gitleaks
tam geçmiş+çalışma ağacı temiz; 13 sentetik gizlilik testi geçti.

Kod/test/belge kaynağı: `8ac1e2ca770b8e9e98c5f096dd6f5b1024dd3f6f`. Exact compiler/core-freeze
beyanları bu kaynağı izleyen committe kaydedilir.

Son [bütün ref taraması](tarama.json): anonim tarihçe tabanında 18 ref,
406 commit, 5.165 path/blob çifti ve **0 eşleşme**. Müşteri eski satır
istisnası yoktur. Sonraki K-215 politika/kod/beyan dosyaları aynı
politikayla çalışma ağacı ve uzak CI kapısında ayrıca denetlenir.
