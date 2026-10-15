# K-257 native request ve PostgreSQL metrikleri

Bu native profil P13/B5 ürün talebinin native sahibi tarafından üretilen sayaçlarını karşılar. Yeni endpoint, dil kalıbı veya desteklenen `dil::api::v1` export açmaz. `--web-metrik-fd N` yalnız native proxy çalıştırmada, kaynaktan önce verilir; supervisor `--runtime-metrics` ile opt-in olur. Varsayılan kapalı yolda süre ölçümü ve metrik IO yoktur. Credential launcher kompozisyonu K260/ADR119 tek `--native-metrics-fd` designated capability bridge ile sağlanır; canlı staging ve tüm P13/P25 kabulü ayrıca açıktır.

## Native sahiplik ve anlam

`zee-native-metrics-1` JSONL alanları tam olarak schema, kind, seq, dropped, v'dir. Tüm sayılar 0..9007199254740991 arasında doyumludur; kind 0 request dizisi22, kind1 pool dizisi8 taşır. Request sırası: accepted, completed, duration_us_sum, final_write_errors, no_response, actual_rate_rejections, ardından 200/303/400/401/403/404/405/408/413/426/429/431/500/503/504/other status sayaçlarıdır. Her kabul edilen bağlantı tek attempt'tir; readiness EOF bağlantısı da no_response olur. Duration kabulden akış kapanmasına kadardır. Status nihai seçilmiş attempted server status'tür: transaction başarısızlığından önceki taslak200 sayılmaz, sonuç503 sayılır. Başarısız write da attempted status'u korur; ayrı write_errors final write_all sonucunu taşır. EINTR retry ardından başarı hata sayılmaz. Tam write kernel kabulüdür; client receipt kanıtı değildir. Rate429 gerçek limit kararı sahibi tarafından artar; herhangi429 yanıtına bakılarak uydurulmaz.

Pool sırası actual connections, idle, busy, waiters, successful_checkout, checkout_timeout, cumulative_checkout_wait_us, cumulative_checked_in_lease_us'dur. busy=connections-idle gerçek r2d2 state() sınır örneğidir. Callback yalnız kısa sayaç lock'u alır: IO veya pool.state() çağırmaz. Snapshot checkout öncesi/sonrası, query/write dönüşü ve commit/rollback dönüşünde callback dışından alınır; süreklilik veya her an taze gauge vaat etmez. Henüz dönmeyen lease süresi checkin toplamında yoktur. Pool yenilenirse cumulative counter farkları korunur, gauge son havuzu gösterir. Host request/pool yaşını ayrı taşır; kayıp son snapshot daha eski değer bırakabilir.

## Transport sınırı

FD3..255 mevcut process'e açıkça devredilmiş FIFO, O_WRONLY, O_NONBLOCK ve current uid/root sahibi olmalıdır. stdio0..2 ile dev/inode eşliği reddedilir. SIGPIPE zaten ignored olmalıdır; bu özellik disposition değiştirmez. CLI kaynak/config okunmadan preflight yapar, sahiplik constructor'ı tekrar doğrular; başarılı constructor FD_CLOEXEC koyar ve descriptor'u yalnız kendi File sahibi kapatır. Küçük libc unsafe çağrıları fstat/fcntl/fpathconf/sigaction sorguları ve doğrulanmış from_raw_fd ile sınırlıdır. Özel kaynak, path, span, SQL, argüman, header/body, connection/event identity ve kullanıcı label'ı payload'a girmez.

Her native frame en çok512byte ve gerçek PIPE_BUF sınırı içindedir; tek nonblocking write, retry yoktur. Writer try_lock; EAGAIN/EINTR drop sayılır. Partial/zero write, EPIPE ve diğer hata kanalı kapatır, request'i durdurmaz. Bozuk/partial EOF/oversize stream host'ta sadece telemetry kanalını devreden çıkarır. Collector iki worker ve kind başına son snapshot ile sınırlıdır; restart toplamları doyumlu korunur. Supervisor üç küçük native_metrics event grubuyla yayımlar; mevcut health probe sayaçları native request/DB diye sunulmaz.

## Kanıt ve açık işler

Tarihsel ön-entegrasyon stage kaydı: `native_metrik_host_testi`15 model vakası ve binary35 vakası kaydetti; mevcut default worker25/25 actual native kampanyası da geçti. Yerel TLS PostgreSQL dört gerçek lease, beşinci waiter, release/recovery ve iki saniye timeout kanıtını üretir; native HTTP oracle200/429/404/426/DB200, tam pipe, kapalı pipe, malformed kaynak stdout/stderr alias erken ret ve iki gerçek worker rollup kanıtları dış manifesttedir. Stage dirty965 kaynak ve K253 aday bağımlılığı taşır; exact final kaynak commit kanıtı değildir. Final SHA soak1800s, public API/katman/SCC/uyumluluk/spec drift/faz/tazelik/ledger kapıları root entegrasyonundadır. Reproducer'ın gerçek B5 ihtiyacına eşliği insan review'unda doğrulanır. Yeni counter durumu core-freeze dogfood-change ve semantic-change olarak kaydedilmelidir; bugfix/regression provenance icat edilmez.

## K-257/K-260 güncel aday kompozisyonu

Dedicated native metrik8 actual vaka ve15 model vaka, credential capability7
model/subprocess+2 actual native vaka vardır; farklı kampanya sayıları Cargo
vaka sayısı gibi toplanmaz. Birleşmiş listener17/FD16 ayrıca metricflag iki
sırası, preclosed metric reader EPIPE, argv/config ve inherited listener sahipliğini
sınar. Yeni doğru makbuz çalışma ağacı/sabit aday ikili kimliği taşır; yukarıdaki
eski stage965/binary35 tarihsel kayıt temiz final kaynak kabulü değildir.
Finalclean/900sn/1800sn/ASAN RC, canlı ürün ve genel P13/P25 açıktır.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

S üzerinde common metrics8, credential capability7+native2 ve actual 900 saniye yük/1800 saniye soak geçti; exact kaynak, sayaç/RSS ve startup sınırları [son rapordadır](urunlesme-final-kabul-20261015.md). Genel delivery/audit veya production SLA eklenmez.
