# ADR-140 — K-283 sürekli HTTP user service

- **Durum:** [S3](../docs/urunlesme-s3-kabul-20261015.md) (`66e4149e53451111d153f47283c219079a8d1cab`): Linux native yedi vaka ile dar yerel kabul; üretim/ürün benimsenmesi ve diğer platform kabulü açık.
P13/B5'in sürekli service ihtiyacı mevcut Supervisor+DB watcher ile karşılanır;
compiler grammar/API/endpoint veya request transaction semantiği değiştirilmez.

Yeni owner explicit Linux user-service config'inden yalnız mevcut typed host
profillerini kurar. Ayrı private DB credential mapping ve pinlenmiş public CA,
watcher'a dar env verir; ambient secret forwarding yoktur. Watcher için açık
--continuous seçeneği finite defaultu korur. Migration/schema/timeout tekrar
probe edilebilir; diğer terminal hatalar/finalfalse/TTL/cancel sınırını korur.
Watcher exit'i host'u kapatır; yalnız verilmiş Popen/WNOWAIT grupları temizlenir.

Ready state proxy admission garantisi değildir; liveness200/DBunready ayrımı
kabul koşucusunda özellikle ölçülür. Type=simple notification veya SLA değildir.
HUP mevcut sıralı restart ve config doğrulamasıdır; zero downtime/in-place reload
uydurulmaz. Native55s grace ve üç sahipli cleanup için outer210s cgroup stop
budget'i açıkça sözleşmelidir. Runtime1 restart edilebilir, config2/lock75 değildir.

[Rehber](../docs/http-service.md), hazırlık18, korunmuş watcher10 ve dedicated
Linux native koşucusu aynı dilimdedir. İlk hazırlık aşamasında native/systemd
kabul henüz koşulmamıştı; sonraki tarihsel S3 kaynağında yedi vaka geçti.
Eski S900/PG15/TTL5 veya S3 kanıtı başka bir kaynak için kabul sayılamaz. GB-098, faz/surface ve
ortak plan/backlog kayıtları aynı kaynak diliminde güncellendi. Compiler/LSP cache veya
persistent buffer eklenmez; yeni host işi core soak yerine mevcut bütün kaynak
ve gerçek native işletim kapılarıyla doğrulanır. Ürün/adoption/platform açıktır.

Revision2: optional `allow_development` gerçek boolean, yoksa false. True yalnız
mevcut worker --pilot seçeneğini açar; identity query ayrıca dirty=false ve exact
source SHA ister, binary hash kontrolü korunur. Native anonymous kampanya bunu
açık true kaydeder; production trust veya dirty binary kabulü değildir. Frozenv1
development-channel precondition eksikliği ve eski hashleri dış kanıtta korunur.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.

## K-306 — kabul koşucusu doğruluğu

K-306 / GB-115 dar kapsamda kapalı. Canonical Rust kökü, ilk çocuk öncesi handler, yapışkan iptal/başarısızlık ve kısmi cleanup; yalnız iki kabul tüketicisi için foreground PostgreSQL özgün Popen sahipliği düzeltildi. Üretim servis davranışı ve mevcut dispatcher4/HTTP7 oracle değişmedi.

Kaynak doğrulaması, ikinci denemenin geçen 12 komutu ile yalnız EOF boşluk düzeltmesinden sonra yeniden geçen beş komuta dayanır; 15 kapının bütünü yeniden çalışmış gösterilmez. Temiz `7d5e4b0` kaynak commit’i ve exact ağaç kayıtlıdır. Fresh yeni kaynakta iki mevcut Cargo sahibi üzerinden dispatcher4 ve HTTP7 geçti; 102 komutluk bootstrap48/build18/dispatcher18/HTTP18 zinciri ve özgün owner kapanışı doğrulandı. Foreground PostgreSQL normal SIGINT/exit0/reaped; iki kabul rolünün sahipli unit/path temizliği, manager running son durumu, guest/host bütün özgün sahiplerin reap sonucu ve VM normal exit0 kapanışı doğrulandı; fallback kullanılmadı, host runtime kaldırıldı. Yeni kaynak tam15 kapısı geçti (1136,782 sn); gerçek tam faz 998/1023 geçti, 0 başarısız, 25 ignored. C12 12 tarihsel + 5 yeniden komut birleşimi bu yeni tam15 sonuçtan ayrıdır. Temiz kaynak commit’i 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa, tree 5585785367353af22dd35b539b50ce4233dbbac2; kaynakgate4 tested overlay ve sourcecommit3/özgün owner makbuzları eşleşti, kaynak341 pini ve production175 sınırı korundu. K-306/GB-115 dar kapanışı yalnız bu exact belge kümesinin son11 kapısı ve tested-overlay temiz kayıt commit’i makbuzlarıyla geçerlidir: E/k306-final-gates-1/PROOF.json, E/k306-final-gates-controller-1.json, E/k306-final-commit-1.json ve E/k306-final-commit-controller-1.json. Bu metin kendi başına kapı veya commit kanıtı değildir. Tarihsel S3 sonuçları korunur; ürün/provider/platform kapsamı genişlemez. [Güncel exact kaynak ve kanıt](../docs/urunlesme-k298-kabul-20261016.md).
