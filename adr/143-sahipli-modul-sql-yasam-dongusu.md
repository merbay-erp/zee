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

## K-307 — foreground PostgreSQL takip2: açık

K-307 / GB-116 düşük ve açıktır. İlk sticky cleanup iptali/yayın dilimindeki exact AST fakeTERM/fakecluster.close karşı örneği tek beklenen failure/exit1/reaped verdi; gerçek OS/PG sızıntısı gözlemi değildi. Baseline10, V4inert13 ve V5birleşik23 controlled sonuçları kendi makbuzlarıyla korunur; ilk kaynak15 kabulü yalnız kendi tam koşu/owner ve temiz commit makbuzlarıyla değerlendirilir.

Bu foreground takip2 anlatımı yalnız reviewed beş kaynak dosyasının aynı atomik kaynak/test/belge diliminde uygulanması altında geçerlidir. Yeni test-only `scripts/tests/native-pg-owner.py` ve preflight dosyası, modül SQL native harness’i ile ordinary/Rust wrapper aynı dilimdedir. SQL harness’i `TLS.LocalCluster` yerine `OwnedLocalCluster` foreground sahibi ve `SqlPgCampaign` adapter’ını kullanır. Yeni PostgreSQL özgün Popen nesnesi korunur; readiness çocuklarının özgün sahipliği ve data-directory/version kimliği doğrulanır. Normal kapanış için aynı özgün PostgreSQL sahibine SIGINT, observed_exit=exit=0/reap ve fallback_used=false gerekir. Belirsiz sahip veya private root kimlik değişiminde private kök silinmez; retained durum sonradan benimsenmez. Bu normal kapanış gerçek PG üzerinde henüz ölçülmüş değildir; helper’ın kabul gereğidir.

İlk yapışkan ledger, cleanup sırasında H maskesi, garantili restore, korumalı handler kurulumu, ExitStack sonrası passed ve publication iptal/deadline/failure denetimleri korunur. SqlPgCampaign cleanup90 sınırı kabul600 sınırını uzatmaz; dokuz SQL native oracle, pinler, pending/unknown no-replay, SQL/DB+FS ve üretim semantiği değişmez. Native Rust wrapper ayrıca foreground PG kapanış/readiness sonucunu ister; bu kontrolün varlığı fresh native PASS değildir. `ZEE_MODULE_SQL_NATIVE_OUT` yalnız internal ignored testin taze mutlak çıktı yolu seçimidir; belirtilmediğinde mevcut0700 geçici dizin varsayılanı korunur. Ürün CLI/API kapasitesi açılmaz.

Gerçek kontrollü hazırlık: aynı helper/preflight baytları üzerinde standalone foreground23 ve takip2 birleşik ordinary24 geçti; özgün owner observed_exit=exit=0/closed/reaped/errors[]/cancelled[]. Standalone23 önceki Tproposal1 yolunda çalıştı; actual hedef/preflight hashleri Tproposal2 ile birebirdir. Birleşik24, önceki23 yöntemi koruyup yeni foreground preflight23’ü içten çalıştıran bir ordinary yöntem ekler; bunlar47 veya yeni Cargo/native toplamı değildir. Mevcut ordinary Cargo sahibi ve ignored native sahibi/dokuz vaka korunur.

SQL foreground bağlantısı bu takip dilimindedir; diğer dört PG tüketicisinin foreground dönüşümü bu dilime dahil değildir ve açık kalır. SQL’in kendi fresh native9 kabulü de bekler; beş PG tüketicisinin tamamı kapalı veya tüm sahiplik kabulü geçti denmez. Genel P20 kısmi, ürün/adoption/platform ve C1/K288/P33/K293 retleri korunur. Önce ilk kaynak15/temiz kaynak commit’i gerçek makbuzlarıyla kapanmalı; sonra takip2 beş kaynak+test+12 belge atomik uygulanıp kendi kaynak kapıları/temiz exact commit’i, ayrı fresh native9 ve son kapılar sağlanmalıdır. Sonraki kaynak SHA/tree henüz bilinmiyor; burada üretilmez.

Etki incelemesi: production Rust175, bağımlılık/katman, grammar/API/ABI, normatif spec/RFC, kullanıcı tanısı/katalog kodu, sabit performans iş yükü ve compiler/LSP kalıcı durum değişmez. Test-only helper/adapter ve native output/proof kontrolleri kullanıcı ürün kapasitesi açmaz; RC/soak/IDE veya geçmiş SHA sonuçları aktarılmaz.
