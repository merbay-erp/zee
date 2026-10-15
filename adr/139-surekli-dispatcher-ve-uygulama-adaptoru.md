# ADR-139 — K-282 sürekli dispatcher ve application-owned adapter

- **Durum:** uygulanmış aday; hazırlık testleri ve source review, yeni exact-clean Linux
kampanyasından ayrıdır. P09/B1'in genel daemon ve uygulama-owned effect consumer
sürtünmesi mevcut host araçları üzerinde karşılanır; compiler/grammar/API açılmaz.

K-268 SQL-only tick daemon içinde yeniden kullanılır; bilinen completion sonrası
monotonic aralık ve ayrı explicit OnCalendar usertimer eklenir. Lifetime service
kilidi bekleme aralarında da tektir. Unknown sonuçlar için kalıcı latch/restart
ret ve exit3 korunur. Bilinen completion sonrası görünür clear/fsync belirsizliği
exit4'tür; gizli rollback veya otomatik unknown retry yoktur.

Genel dış etkiyi sabit SMTP fixture'ı gibi gösterme yerine, ayrı opt-in pinlenmiş
application executable/argv/data/cwd ve strict bounded ack sözleşmesi kullanılır.
Adapter güvenilir uygulama kodudur, sandbox değildir. Host app ack'ini teslim,
SQL transaction veya exactly-once diye yeniden yorumlamaz. Provider protokolü ve
credential yetkisi uygulamaya aittir; host ambient sır veya arbitrary FD aktarmaz.
Durable guard Popen öncesindedir; nofollow/bounded snapshot, held-directory/proc
cwd, WNOWAIT cleanup ve fixed metadata sınırı korunur. Unknown guard için bu
sürüm otomatik/operator-asserted replay veya arbitrary deletion komutu sağlamaz.

[Rehber](../docs/dispatcher-service.md), host hazırlık22 ve ayrı rendered-unit/
SQL/calendar/adapter native koşucusu aynı dilimdedir. Yeni native kabul henüz
üretilmemiştir; eski finite/SMTP kanıtı yeni source'a mal edilmez. GB-097 kaydı,
backlog, plan, faz ve uyumluluk kayıtları aynı kaynak diliminde güncellendi.
Kalıcı benchmark veya compiler/LSP persistent buffer/cache değişmediği için bu
host işi tek başına core soak tetiklemez; bütün kaynak kapıları yine korunur.

## K-295 — daemon unit'in explicit yürütme düzeltmesi

S2 `e35b617095e6a9d00915794d6adee90842238a5f` gerçek Linux koşusunda
daemon ExecStart yalnız --config taşıdığı için check-only kipinde exit0 dönmüş,
hiç SQL tick başlamamıştır (0/4; cleanup_errors boş, owned_units_released true).
Üretilen daemon artık explicit --run, OnCalendar hizmeti yalnız --once taşır.
Doğrudan CLI varsayılan preflight ve render'ın servis başlatmama sınırı değişmez.
Eski unit dosyaları yeniden --render ile üretilmelidir. Exact S2 kaynağına karşı
yeni regresyon kırmızı, düzeltmeyle hazırlık22 yeşildir; S3 temiz Linux kabulünde dört vaka geçti. Host correctness düzeltmesidir; compiler bug/grammar/tanı/API değişmez.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.
