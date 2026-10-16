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

## K-306 — kabul koşucusu hazırlığı

K-306 / GB-115 açık. İki Linux native kabul koşucusunda canonical repo kökü, ilk çocuk sorgusundan önce flag-only sinyal handler kurulumu, yapışkan iptal sonucu ve kısmi setup/cleanup sahipliği düzeltildi; foreground PG özgün Popen adaptörü yalnız bu iki test tüketicisine bağlandı. Üretim servis davranışı, compiler/API/grammar ve mevcut dispatcher4/HTTP7 oracle’ları değiştirilmiyor.

Kontrollü Campaign sınıf modellerinde aynı 16 yöntem eski kaynakta 13 assertion failure/0 error, adayda 0 failure/0 error üretti; her iki ham günlük ve özgün dış owner exit0/reaped doğrulandı. Gerçek çocuk, OS sinyali, Linux, PostgreSQL veya systemd kabulü değildir. Birleşik kaynakta Campaign16 ve foreground PG18 yöntemi/31 kontrollü senaryo (29 davranış/iki statik kontrol) geçti; yeni Cargo sahibi eklenmedi. Tam kaynak/faz kapıları, yeni temiz exact kaynakta gerçek Linux4+7 ve son belge/güvenlik kapıları geçmeden GB-115 kapanmaz. Tarihsel S3 kabulü kendi kaynağında kalır; ürün/provider/platform kapsamı genişlemez. [Güncel hazırlık sınırı](../docs/http-service.md).
