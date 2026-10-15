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
Linux native koşucusu aynı dilimdedir. Yeni native/systemd kabul henüz koşulmadı;
eski S900/PG15/TTL5 kanıtı yeni source için kabul sayılamaz. GB-098, faz/surface ve
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
