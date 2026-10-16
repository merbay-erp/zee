# ADR-139 — K-282 sürekli dispatcher ve application-owned adapter

- **Durum:** tarihsel [S3](../docs/urunlesme-s3-kabul-20261015.md) (`66e4149e53451111d153f47283c219079a8d1cab`) kaynağında dört Linux native vakasıyla dar yerel kabul; genel provider/exactly-once, ürün benimsenmesi ve diğer platformlar açık. Bu kanıt sonraki kaynaklarda yeniden koşulmuş sayılmaz. P09/B1'in genel daemon ve uygulama-owned effect consumer
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
SQL/calendar/adapter native koşucusu aynı dilimdedir. İlk hazırlık aşamasında
native kabul henüz üretilmemişti; sonraki tarihsel S3 kaynağında dört vaka geçti.
Eski finite/SMTP veya S3 kanıtı başka bir kaynakta yürütülmüş sayılmaz. GB-097 kaydı,
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

## K-306 — kabul koşucusu doğruluğu

K-306 / GB-115 dar kapsamda kapalı. Canonical Rust kökü, ilk çocuk öncesi handler, yapışkan iptal/başarısızlık ve kısmi cleanup; yalnız iki kabul tüketicisi için foreground PostgreSQL özgün Popen sahipliği düzeltildi. Üretim servis davranışı ve mevcut dispatcher4/HTTP7 oracle değişmedi.

Kaynak doğrulaması, ikinci denemenin geçen 12 komutu ile yalnız EOF boşluk düzeltmesinden sonra yeniden geçen beş komuta dayanır; 15 kapının bütünü yeniden çalışmış gösterilmez. Temiz `7d5e4b0` kaynak commit’i ve exact ağaç kayıtlıdır. Fresh yeni kaynakta iki mevcut Cargo sahibi üzerinden dispatcher4 ve HTTP7 geçti; 102 komutluk bootstrap48/build18/dispatcher18/HTTP18 zinciri ve özgün owner kapanışı doğrulandı. Foreground PostgreSQL normal SIGINT/exit0/reaped; iki kabul rolünün sahipli unit/path temizliği, manager running son durumu, guest/host bütün özgün sahiplerin reap sonucu ve VM normal exit0 kapanışı doğrulandı; fallback kullanılmadı, host runtime kaldırıldı. Yeni kaynak tam15 kapısı geçti (1136,782 sn); gerçek tam faz 998/1023 geçti, 0 başarısız, 25 ignored. C12 12 tarihsel + 5 yeniden komut birleşimi bu yeni tam15 sonuçtan ayrıdır. Temiz kaynak commit’i 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa, tree 5585785367353af22dd35b539b50ce4233dbbac2; kaynakgate4 tested overlay ve sourcecommit3/özgün owner makbuzları eşleşti, kaynak341 pini ve production175 sınırı korundu. K-306/GB-115 dar kapanışı yalnız bu exact belge kümesinin son11 kapısı ve tested-overlay temiz kayıt commit’i makbuzlarıyla geçerlidir: E/k306-final-gates-1/PROOF.json, E/k306-final-gates-controller-1.json, E/k306-final-commit-1.json ve E/k306-final-commit-controller-1.json. Bu metin kendi başına kapı veya commit kanıtı değildir. Tarihsel S3 sonuçları korunur; ürün/provider/platform kapsamı genişlemez. [Güncel exact kaynak ve kanıt](../docs/urunlesme-k298-kabul-20261016.md).
