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
`ci-taban-kimligi.py` yalnız K-214 eski master ve iki birleşmiş dal ucu için kaydedilmiş yeni
ata SHA'yı doğrular; diğer tabanlar aynen geçer, genel force istisnası yoktur.
`acik-depo` workflow'u push/PR kaynağını güvenilir master denetleyicisiyle
yalnız veri olarak tarar; PR kodu çalıştırılmaz, token salt-okunurdur.

Yerel kanıt: 770/770 faz, fmt/Clippy, katalog/tazelik/mimari/SCC/uyumluluk/
regresyon kapıları ve Gitleaks geçmiş+ağaç taraması geçti. Anonim tarihçe
kapısı yeni eşleşme bulmadı. Uzak ref eşitliği push sonrasında doğrulanır.

Kapı/kod/test/belge commit’i: `8706447a7337a54cad4bf4e8547c31d57fbfb2f0`. Kaynak beyanı takip commitindedir.
9 sentetik gizlilik/CI testi geçti; terim listesi veya eşleşme değeri loga yazılmadı.

İlk uzak 7eddb01 koşusu: gizlilik kapısı ve sır taraması geçti. CI komut
kimlik eşlemesindeki inline çağrı, iki mevcut mimari kablo assertion'ını
bozdu. Exact guard çağrısı çok satırlı adımda korunarak düzeltildi;
23 mimari vaka, 9 sentetik gizlilik/CI vakası ve 770/770 tam faz tekrar geçti.
Güvenlik eşiği/fixture gevşetilmedi; final uzak CI sonucu ayrıca izlenir.

[Eski yayımlanmış dal uçları](onceki-dal-uclari.md) ayrıca tarandı: üç
değiştirilmeyen dalda 261 dal/dosya/satır eşleşmesi (her dalda 87).
Bu önceki geçişler kaynak tabanındaki 84 konumdan ayrı dökümdür; yeni
hash istisnası eklenmedi. Tüm deponun geçmişten arındığı iddia edilmez.
Eski dallar kullanıcı kararıyla temizlenmeden ve GitHub cache/PR refs
teyidi gelmeden yeniden açılış hazır sayılmaz.

Windows ilk ddf8b88 fazında Python'ın genel `bash` seçimi Git Bash yerine
başka runtime'ı çağırdı. Sentetik CI hazırlık testi Windows'ta kurulu Git'in
Git Bash executable'ını ve ileri eğik çizgili yolları açıkça seçer; ret eşiği
ve gerçek kapı davranışı değişmez. Linux/macOS/tedarik aynı kaynakta geçti.

Windows Git executable'ı cmd/bin/mingw64 dizinlerinden bulunabilir. Git Bash
aynı kurulumun üst dizinlerinde aranır; System32/WSL seçilmez. Üç düzen ve
eksik Bash ret testi eklendi: 10 sentetik kapı/CI testi yerelde geçti.

## Makine dilimi kapanışı

**KAPALI:** anonim içerik ve Git geçmişi, 51 source kimliği eşlemesi,
source beyanları/regression sahipleri, PR #5/#7 metinleri ve CI kapısı
tamamlandı. Merbay ana kaynak; Enterprise aynı commit'in özel aynası ve
Actions kapalıdır. PR #6 kaynak dalının devraldığı tarihçe de anonimleştirildi.

Test/kod kaynağı `290fa52ec28c65ad5ce445fd663374dd5ddbcdd7` için
[altı uzak kontrolün tamamı geçti](ci-290fa52.json): Linux/macOS/Windows,
Rust tedarik, Gitleaks ve açık depo kapısı. Koşular 38417531795,
38417532301, 38417532272. Yerelde 770/770 faz ve 10 sentetik kapı
testi geçti. Önceki kablo/Bash hataları tarihsel kayıt olarak yukarıdadır.

**AÇIK:** önceki 84 ana-dal konumu ve üç eski dalda 261 dal/dosya/satır
eşleşmesi kullanıcı kararı bekler; bunlar anonimleşti diye sunulmaz.
GitHub PR refs/önbellek temizliği destek teyidi bekler (GB-046).
Depo gizlidir; yeniden açmak için ayrıca kullanıcı onayı alınır.
