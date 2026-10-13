# K-214 — Açık depo temizliği

13 Ekim 2026 kullanıcı talebi: PR #5/#7 ile gelen müşteri/alan adı/sunucu
bilgisi anonimleştirilir, bugünkü Git geçmişi ve exact-SHA beyanları yenilenir.
Eski taban `12c59218dc84e8e9d4610274c722985ce6a99dae`; eski master
`45b66f3574e0be020aeffc95580dd8db822e1379`. Merge topolojisi korunur.

Anonim Site A–F/ürün E etiketleri kullanılır. Kaynak ürün, özgün talep,
üretim yolları ve müşteri eşlemesi özel ürün tarafında kalır. KOKEN dosyaları
exact harici ürün SHA'sını ve hermetik reproducer ilişkisini korur.

[SHA eşlemesi](sha-esleme-v1.tsv) kaynak beyanları, regression sahipleri ve
normal belge referanslarının yeni kimliklerini taşır. Ölçüm/provenance/
SBOM/ikili hash geçmişi özgün ölçülen SHA'yı korur; bu kayıtlar yeniden
koşulmuş test veya yeni sürüm ikilisi olarak sunulmaz. SMTP vektörleri
sentetik `.invalid` alan adına geçer; derleyici davranışı aynı kalır.

[Önceki geçişler](onceki-gecisler.md) 84 maskeli konumdur; bu eski kayıtların
kaldırılması kullanıcıya ayrı karar olarak bırakılmıştır. Hash politikası
129 exact eski fingerprint taşır. Başka dosyaya kopyalama, satır değiştirme
ve ek tekrar reddedilir. Politika terimlerin plaintext listesini taşımaz.

Yerel kapı: `python3 scripts/acik-depo-denetle.py`.
Sentetik olumlu/olumsuz test: `python3 scripts/tests/acik-depo-test.py`.
Tam Git ağaçları, dosya adları, commit mesajları ve çalışma ağacı taranır;
sonradan silinen içerik de kapsanır. Hata çıktısı yalnız dosya hash'i/satırdır.

PR metinleri ve normal yayımlanmış dal referansları ayrıca anonimleştirilir.
Enterprise özel ayna, Actions kapalı kalır. GitHub sunucu tarafındaki PR
refs/önbellek silinmesi [destek talebi](github-destek-talebi.md) ile ürün
 tarafınca iletilecektir. GB-046 bu dış teyit gelene kadar orta/açık kalır.
Depo gizlidir; yeniden açılış kullanıcı onayı ister.

Spec/RFC/dil yüzeyi/tanı kataloğu incelendi: dil davranışı ve tanılar değişmez,
yeni grammar veya public API yoktur. ADR-045/049 ve performans geçmişi yeni
ölçümle değiştirilmez. ADR-046/katman fixture/SCC graph'ı aynı kalır;
production farkı sentetik test metni/yorum ve kanonik Rust biçimidir.
Release betiği, Cargo pinleri, SBOM sözleşmesi ve action pinleri değişmez.
Faz matrisi, canlı sayılar ve dogfood kanıt özeti üreticiyle yenilenir.

Doğrulama sonuçları ve uzak ref eşitliği aynı kayıt dalgasında eklenir.

Özel depo CI'ında self Git bağımlılığı aynı exact checkout nesnesinden gelir;
Cargo URI/rev değişmez, credential saklanmaz. Tam checkout zorunludur.
`ci-taban-kimligi.py` yalnız eski master 45b66f3 için kaydedilmiş yeni
ata SHA'yı doğrular; diğer tabanlar aynen geçer, genel force istisnası yoktur.
`acik-depo` workflow'u push/PR kaynağını güvenilir master denetleyicisiyle
yalnız veri olarak tarar; PR kodu çalıştırılmaz, token salt-okunurdur.

Yerel kanıt: 770/770 faz, fmt/Clippy, katalog/tazelik/mimari/SCC/uyumluluk/
regresyon kapıları ve Gitleaks geçmiş+ağaç taraması geçti. Anonim tarihçe
kapısı yeni eşleşme bulmadı. Uzak ref eşitliği push sonrasında doğrulanır.

Kapı/kod/test/belge commit’i: `8706447a7337a54cad4bf4e8547c31d57fbfb2f0`. Kaynak beyanı takip commitindedir.
9 sentetik gizlilik/CI testi geçti; terim listesi veya eşleşme değeri loga yazılmadı.
