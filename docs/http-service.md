# K-283 — Sürekli Linux HTTP user service

[`http-service.py`](../scripts/http-service.py), mevcut K-246 iki-worker supervisor
ve K-274 DB watcher'ını tek sahipli host altında birleştirir. Yeni compiler veya
web endpoint yoktur. [ADR-140](../adr/140-surekli-http-user-service.md).
Varsayılan check-only; `--render /owned/fresh-units` yalnız
`zee-http.service` üretir; `--run` açık Linux yürütmesidir. Kod systemctl, install,
enable, linger veya manager restart çağırmaz. Gerçek ürün/physical-server/Windows
ve sürekli production adoptionı ayrı kalır.

Config `schema=zee-http-service-1`, canonical/current-owner0600/single-link ve
16KiB'dir. Exact alanlar: `repo`, `source_sha`, `dil`, `binary_sha256`, `project`,
`project_files`, `probe_project`, `probe_files`, `contract`, `contract_sha256`,
`application_revision`, `origin`, `worker_ports`, `readiness_file`, `health_path`,
`health_body`, `state`, `watch_interval`, `probe_deadline`, `readiness_ttl`,
`credential_root`, `credential_env`, `probe_credential_root`,
`probe_credential_name`, `probe_ca`, `probe_ca_sha256`. Bunlara schema dahildir;
unknown/duplicate alan ve nonfinite sayı reddedilir. Hashler gerçek girdilerden
alınır; uydurma runnable config verilmez.

Repo exact temizS; native binary expected SHA ve BuildJSON dirtyfalse'dır.
`project_files`/`probe_files` relative dosya→SHA haritasıdır; her biri1–128 dosya,
proje.dil zorunlu, dosya262144bayt/toplam2MiB. Symlink/escape/FIFO/observedgrowth
retlidir. Bunlar açıkça bildirilen kaynak envanteridir, generic dependency resolver
veya hostile-code sandbox değildir. Declared kaynak, config, contract, CA, ikili
ve host kaynakları başlangıç/sonda hash,çalışma sırasında stat identity ile korunur.
Runtime `zee.ayar`, readiness, `.zee` ve application data source inventory'ye
konmaz; HUP yalnız mevcut güvenilir config sözleşmesini kullanır. Unlisted source
modülü için tam kaynak attestation iddiası kurulmaz.

State external/current-owner0700 parent altındadır; lifetime EX lock ve held
namespace denetimi bütün watcher/worker cleanup sonuna kadar sürer. İki distinct
worker portu açık seçilir, foreign socket/service/PID edinilmez. Watcher ayrı
owned Popen/session grubu, worker'lar mevcut Supervisor owner'ıdır. Watcher exit'i
worker kaybı gibi host'u kapatır; kalan tüm owner handles temizlenir.

## Credential ve sürekli readiness

Worker `credential_root` null ve `credential_env=[]` olabilir; explicit map varsa
mevcut launcher/capability/metrics yolu kullanılır. DB probe için ayrı
`probe_credential_root` ve bir `probe_credential_name`, yalnız
ZEE_READINESS_DATABASE_URL'a K-245 private reader ile eşlenir. Ambient mapped URL
ve CA fallback reddedilir. `probe_ca` owner0600/single-link normal public CA PEM,
16KiB ve explicit hash ile bound'dur; credential değeri/config/log alanı değildir.
Watcher env'i yalnız PATH/locale/TZ + designated DB URL ve public CA PEM'dir.
Başka ambient sır/dictionary/FD geçmez. Credential rotation için service restart
gerekir; HUP ile watcher'ın secret env'i sessizce değişmiş sayılmaz.

`db-hazirlik.py --continuous` yeni explicit opt-in'dir. Varsayılan bir kez ve
mevcut finite `--rounds`1–1000 davranışı korunur; continuous ile rounds!=1 retlidir.
Interval2–60s, probe deadline0.1–15s ve TTL1–3600s;
TTL >= interval + deadline +2 zorunludur. Migration/schema/timeout unready yazar
ve sonraki tura geçer; identity/input/output/gate/cancel terminaldir. TERM/INT
flag-only; bounded child deadline/cancellation ve final ready=false korunur.
SIGKILL finally çalıştırmaz; eski dosya TTL ile geçersizleşir. Yeni DB göçü,
DDL write, otomatik migration veya request transaction retry yapılmaz.

**Ready/unready trafik kontrolü değildir.** Supervisor'ın ayrı health ve DB state
raporu korunur; DB drift sırasında `/hazir`200 kalabilir. Reverse proxy'nin bu
bilgiyi kullanması ayrı application-owned routing entegrasyonudur. Type=simple
sadece main süreç başladı demektir; sd_notify/application-ready veya SLA değildir.

## HUP, stop ve restart

Rendered unit ExecReload=/bin/kill -HUP $MAINPID ile mevcut sıralı worker restart'ını
kullanır. Native config validation ve yeni generation freshness korunur; sıfır
kesinti, in-place reload veya rollback sözü yoktur. Unit Restart=on-failure;
preflight2 ve lock75 tekrar başlatılmaz; runtime arızası1 yeniden başlatılabilir.
Kaynak/pin değişince yeni preflight yine fail-closed'dur. Sırlar/SQL/body/rawchild
stream loglanmaz; mevcut bounded native request/pool metrics sahipliği değişmez.

Native stop-grace55s worker/watch gruplarına uygulanır; admitted HTTP isteği
olmayan identity-only query grupları ayrı1s cleanup budget kullanır; watcher ve iki worker sıralı
cleanup + terminal wait/query masrafı için unit TimeoutStopSec210s vardır.
KillMode=mixed/SendSIGKILL=yes son cgroup bütçesidir; kernel IO için genel hard
wall-time garantisi yoktur. Linux'ta retained-zombie probe grace süresini
kullanabilir; bu davranış performans iyileştirmesi diye gizlice değiştirilmez.

Hazırlık18 gerçek owned Python süreçleriyle watcher ölümü/all-owned cleanup ve
foreign survival, cancellation, lock/paths/config, finalfalse ve signal restoration
sınar. Mevcut watcher10 da tekrar çalıştırılır. Bunlar native HTTP/PG/systemd
kabulü değildir. Ayrı koşucu actual rendered unit, native HTTP+PG drift/restore,
quota100→101 HUP, owned unit crash/restart, inflight TERM response ve actual own
Popen watcher kaybını ayrı gözlemlerle ister. İlk hazırlıkta yürütülmemiş olan
koşucu, tarihsel [S3 kaynağında](urunlesme-s3-kabul-20261015.md) yedi Linux
vakasını geçti. Bu, sonraki kaynakların yeniden kabulü değildir; her yeni
kabul exact temiz kaynak/ikili ve sahipli Linux koşusuna bağlanır.

Revision2: optional `allow_development` gerçek boolean, yoksa false. True yalnız
mevcut worker --pilot seçeneğini açar; identity query ayrıca dirty=false ve exact
source SHA ister, binary hash kontrolü korunur. Native anonymous kampanya bunu
açık true kaydeder; production trust veya dirty binary kabulü değildir. Frozenv1
development-channel precondition eksikliği ve eski hashleri dış kanıtta korunur.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## K-306 — Linux kabul koşucusu: dar kapsamda kapalı

İki Rust wrapper’ın canonical repo kökü, ilk çocuk öncesi handler, yapışkan iptal/başarısızlık ve kısmi cleanup düzeltildi. Foreground PostgreSQL özgün Popen adaptörü yalnız dispatcher ve HTTP kabul tüketicilerine bağlıdır. Üretim davranışı ve mevcut4/7 oracle değişmedi.

Kaynak doğrulaması, ikinci denemenin geçen 12 komutu ile yalnız EOF boşluk düzeltmesinden sonra yeniden geçen beş komuta dayanır; 15 kapının bütünü yeniden çalışmış gösterilmez. Temiz `7d5e4b0` kaynak commit’i ve exact ağaç kayıtlıdır. Kontrollü hazırlık Campaign16 ve PG18 yöntem/31 senaryoda geçti. Dış sahiplik hazırlığında core17, CLI9 ve runner5 modeli geçti; bu sayılar Cargo/native vaka toplamına eklenmez.

Fresh yeni kaynakta iki mevcut Cargo sahibi üzerinden dispatcher4 ve HTTP7 geçti; 102 komutluk bootstrap48/build18/dispatcher18/HTTP18 zinciri ve özgün owner kapanışı doğrulandı. Yalnız bu exact kaynakta2/25 mevcut ignoredCargo sahibi ve dispatcher4+HTTP7native geçti;tamfaz998/0/25 ayrı;kalan23ignoredCargo-vakasıbuAttempt8kampanyasındaçalışmadı. Native11 iç vaka Cargo toplamına eklenmez; C12 kontrollü modelleri yeni Rust/native toplamına katılmaz. Tarihsel RC/soak/IDE, fiziksel platform ve ürün kabulü bu kaynağa aktarılmaz. Foreground PostgreSQL normal SIGINT/exit0/reaped; iki kabul rolünün sahipli unit/path temizliği, manager running son durumu, guest/host bütün özgün sahiplerin reap sonucu ve VM normal exit0 kapanışı doğrulandı; fallback kullanılmadı, host runtime kaldırıldı. Gerçek source15/clean commit ve attempt8 source/native/owner/raw zinciri bağımsız review ile issues=[] olarak doğrulandı; frozen kabulün exact11 öncül kayıt ve public artifact seti eşleşti.

Tarihsel S3 ile K-298–K-305 sonuçları kendi kaynaklarında kalır; farklı SHA sonuçları toplanmaz. P09/P13/P36 kısmi, P37 sürekli; ürün/sağlayıcı benimsenmesi, genel exactly-once ve fiziksel platform U2 açık kalır. C1/P33 reddedilmiş kapsamı yeniden denenmez. Yeni kaynak tam15 kapısı geçti (1136,782 sn); gerçek tam faz 998/1023 geçti, 0 başarısız, 25 ignored. C12 12 tarihsel + 5 yeniden komut birleşimi bu yeni tam15 sonuçtan ayrıdır. Temiz kaynak commit’i 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa, tree 5585785367353af22dd35b539b50ce4233dbbac2; kaynakgate4 tested overlay ve sourcecommit3/özgün owner makbuzları eşleşti, kaynak341 pini ve production175 sınırı korundu. K-306/GB-115 dar kapanışı yalnız bu exact belge kümesinin son11 kapısı ve tested-overlay temiz kayıt commit’i makbuzlarıyla geçerlidir: E/k306-final-gates-1/PROOF.json, E/k306-final-gates-controller-1.json, E/k306-final-commit-1.json ve E/k306-final-commit-controller-1.json. Bu metin kendi başına kapı veya commit kanıtı değildir. Ayrıntı ve başarısız deneme geçmişi [ortak kabul raporu](urunlesme-k298-kabul-20261016.md).

Bilinçli etki incelemesi: production Rust/bağımlılık/katman, grammar/API/ABI, tanı, normatif spec/RFC ve uyumluluk yüzeyi değişmez. Compiler/LSP kalıcı durum veya sabit performans iş yükü eklenmedi; ADR045/049/046/069, performans/soak tarihçesi ve compiler regression/beyanları etkilenmez.
