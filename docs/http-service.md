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
Popen watcher kaybını ayrı gözlemlerle ister. Yeni koşucu henüz yürütülmemiştir;
önce root review ve exact clean kaynak/ikili, sonra owned Linux kabulü gerekir.

Revision2: optional `allow_development` gerçek boolean, yoksa false. True yalnız
mevcut worker --pilot seçeneğini açar; identity query ayrıca dirty=false ve exact
source SHA ister, binary hash kontrolü korunur. Native anonymous kampanya bunu
açık true kaydeder; production trust veya dirty binary kabulü değildir. Frozenv1
development-channel precondition eksikliği ve eski hashleri dış kanıtta korunur.
