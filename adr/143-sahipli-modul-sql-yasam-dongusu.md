# ADR-143: Sahipli modül SQL yaşam döngüsü

- **Durum:** [S3](../docs/urunlesme-s3-kabul-20261015.md) (`66e4149e53451111d153f47283c219079a8d1cab`): dokuz Mac PostgreSQL vakası ile dar yerel kabul; DB+FS atomikliği, pending/manual review ve ürün benimsenmesi açık.
- İş: K-285; güvenlik sınırı GB-101.
- İlgili: [ADR-131](131-yonetilen-modul-kompozisyonu.md), [rehber](../docs/module-sql-lifecycle.md).

K-273 içerik metadata'sını ve SQL archive byte'larını birleştiriyordu; gerçek
CRUD veya uygulanan up/down sağlamıyordu. P20'nin bu teknik boşluğu yeni compiler
özelliği gerektirmez: mevcut parametreli PostgreSQL eylemleri ve transaction'lı
göç komutu yeterlidir.

Yeni ayrı owned SQL manager, eski manager'ın exact private snapshot davranışını
korur. Sınırlandırılmış typed schema'dan sabit SQL üretilir; kullanıcı değerleri
parametredir. Modül registry/menü ve gerçek GET liste adaptörü aynı projection'da,
CRUD CLI aynı veritabanı tablolarında çalışır. Native GET HTML-typed olduğundan JSON metni html güvenlisi ile kaçışlanıp sabit pre içinde sunulur; raw script verisi elemente dönüşmez. Şema en çok8 modül/8 alan,
256 append-only göç ile sınırlıdır. SQL adı/data ayrımı test edilir.

Down veri yok etmez: gerçek SQL ALTER TABLE ile archive adı, up aynı relation'ı
geri getirir. Applied ledger ve eski source hash'leri saklanır. Keyfi migration,
destructive uninstall, genel schema evolution veya production DB adoption yoktur.

DB+filesystem atomicity vaadi yoktur. Kalıcı pending önce gelir; bilinmeyen sonuç
admission'ı kapatır. Göç recovery'si readonly exact ledger+schema gözlemine dayanır;
CRUD unknown için explicit operator gözlem kararı gerekir ve yazı replay edilmez.
Post-visible fsync hatası exit4 olarak ayrılır. Cooperative owned root/DB sınırı,
aynı UID saldırganına karşı sandbox veya exactly-once garantisi değildir.

Spec'teki çalışan dil/SQL semantiği değişmez; yeni sözleşme ayrı host aracı
rehberindedir. Yeni compiler tanısı olmadığı için hata kataloğu kodu eklenmez.
Production Rust modülü/kenarı yoktur; ADR-046/katman ve public facade değişmez.
Benchmark iş yükü veya kalıcı compiler/LSP durumu eklenmez. K-273 testleri ve
helper hash'i korunur. Gerçek ürün/adoption, yetkilendirme, diğer platformlar ve
nihai clean-source kabulü ayrı kapılardır.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.

## K-307 — ayrı test harness doğruluğu dilimi: açık

K-307 / GB-116 düşük ve açıktır. `scripts/tests/modul-sql-native.py` cleanup callback’inin exact AST modeli, fake cluster.close içinden fake TERM handler çağrıldığında yeni iptalin kaybolduğunu gösterdi: tek beklenen assertion failure, özgün owner exit1/reaped. Gerçek sinyal, OS/PG süreci veya sızıntı çalıştırılmadı; gerçek PG kaçağı kanıtı değildir.

Düzeltme kapsamı ayrı yapışkan iptal ledger’ı; cleanup için yalnız H iptal bayrağının geçici maskelenmesi ve her yolda geri yüklenmesi; korumalı kısmi ilk handler kurulumu; tüm handler geri yüklemeleri; ExitStack ve geçici dizin cleanup’ından sonra passed kararı; yayın sırasında iptal/deadline/failure kontrolü ve birincil hatanın korunmasıdır. Diagnostic veya bir handler restore hatası kalan geri yüklemeleri atlayamaz. Mevcut600sn kampanya kabul sınırı ve dokuz native SQL oracle değişmez.

T adayV4 inert13 ve T adayV5 birleşik23 yöntemi gerçek özgün owner makbuzlarıyla geçti; baseline10 ayrıca geçti. Eski10 ordinary metot korunup13 inert yöntem aynı `compiler/tests/modul_sql_testi.rs::modul_sql_stdlib_sozlesmesi` sahibi altında birleştirildi. Bu controlled sonuç kaynak veya fresh native kabulü değildir; Kaynak/test/belge uygulaması bu atomik dilimin parçasıdır. Ayrı ignored sahibi `modul_sql_native_crud_goc_yasam_dongusu` ve dokuz alt native vaka korunur; Python yöntem sayısı Cargo/native toplamına eklenmez. Kaynak+test+belge aynı atomik commit, gerekli katalog/tazelik/fmt/faz ve kaynak kapıları, exact temiz kaynak, fresh native9 ve son kapılar kapanış önkoşuludur.

Bu dar harness işi `TLS.LocalCluster` içindeki pg_ctl ile başlatılan detached PostgreSQL ve immediate-stop sahipliğini foreground özgün Popen/normal kapanışa dönüştürmez. Kalan beş PG tüketicisi, genel P20/adoption/platform, DB+FS atomikliği ve unknown no-replay sınırı açıktır. Üretim compiler175/bağımlılık/katman, grammar/API/ABI, normatif spec/RFC, kullanıcı tanısı/katalog kodu, sabit performans iş yükü veya compiler/LSP kalıcı durum değişmez; yeni özellik/RC/soak/IDE kabulü aktarılmaz.

Gerçek kontrollü sonuçlar ayrı makbuzlarla kayıtlıdır: baseline10, adayV4 inert13 ve adayV5 birleşik23 yöntemi geçti; her özgün owner observed_exit=exit=0/closed/reaped/errors[]/cancelled[]. Birleşik23, korunan10 ordinary ve eklenen13 inert yöntemdir; Cargo/native sayısı değildir. Bu kaynak/test/belge dilimi atomik uygulama kapsamındadır; kaynak kapıları, exact temiz kaynak ve fresh native9/son kapı kabulü bekleniyor.
