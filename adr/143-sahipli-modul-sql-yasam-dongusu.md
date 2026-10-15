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
