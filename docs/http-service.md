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

## K-306 — Linux kabul koşucusu hazırlığı

K-306 / GB-115 açık. İki Linux native kabul koşucusunda canonical repo kökü, ilk çocuk sorgusundan önce flag-only sinyal handler kurulumu, yapışkan iptal sonucu ve kısmi setup/cleanup sahipliği düzeltildi; foreground PG özgün Popen adaptörü yalnız bu iki test tüketicisine bağlandı. Üretim servis davranışı, compiler/API/grammar ve mevcut dispatcher4/HTTP7 oracle’ları değiştirilmiyor.

Kontrollü Campaign sınıf modellerinde aynı 16 yöntem eski kaynakta 13 assertion failure/0 error, adayda 0 failure/0 error üretti; her iki ham günlük ve özgün dış owner exit0/reaped doğrulandı. Gerçek çocuk, OS sinyali, Linux, PostgreSQL veya systemd kabulü değildir. Hazırlık makbuzu `E/k306-campaign-models-1/PROOF.json` (`045a65ae1ce84b5e11d16a5da6620d786d8f3ad2700403b00ffcbb89c03260e6`); bağımsız inceleme `K306-CAMPAIGN-MODELS-INDEPENDENT-REVIEW.json` (`aa747b6998ae2c726b3cc713631aa4821e29f32e49dcaf7d5f8b54d053720bc8`).

Rust 1.93.1 için altı public araç zinciri arşivi (126,477,864 bayt) önce imzası doğrulanmış manifestin exact URL/SHA-256 kayıtlarıyla eşleşti; altı HTTP200/no-redirect günlüğü ve dış owner exit0/reaped doğrulandı. Arşiv açılması, installer yürütülmesi, VM başlatılması veya guest kurulumu bu kanıtın kapsamında değildir. İndirme makbuzu `E/k306-linux-rust-components-1/PROOF.json` (`1fd6170e75af757aa7f4722a034aeccd735da5046d5643fac9305271bbb65b59`); bağımsız inceleme `K306-RUST-COMPONENTS-INDEPENDENT-REVIEW.json` (`a09a2765e9a04227b72ce01f735229633d06c24a88bfb32c20b1ce17a3938df7`).

PG adaptör hazırlığı: Birleşik kaynakta Campaign16 ve foreground PG18 yöntemi/31 senaryo (29 davranış ve iki tüketici için iki statik oracle kontrolü) 0 failure/0 error/0 skipped ile geçti; özgün dış owner exit0/reaped. Bu kontrollü sınıf modelleri gerçek PostgreSQL/Linux/sinyal kabulü değildir. Kaynak birleşimi: Yedi kod dosyası hazır: iki canonical Rust wrapper, ortak Campaign/OwnedLocalCluster, iki native tüketici ve iki Python hazırlık dosyası. Her iki hazırlık mevcut dispatcher_service_hazirlik sahibine tek kez bağlandı; yeni Cargo sahibi yok. Kaynak/faz kapıları ve temiz kaynak commit’i henüz bekleniyor. Mevcut normal dispatcher22/HTTP18 ve watcher10 sahipleri korunur. İki yeni Python hazırlık dosyası mevcut dispatcher hazırlık sahibinden yürütülür; tam faz kabulü ve temiz kaynak kimliği henüz yoktur.

Gerçek Linux user manager, temiz exact kaynak/BuildJSON/ikili, fresh özel namespace ve özgün foreground PG handle kapanışıyla dispatcher4/HTTP7 ayrı doğrulanmalıdır. Başkasına ait PID, unit veya küme benimsenmez. Hazırlık alt vakaları Cargo toplamına katılmaz; genel P09/P13/P36 ve fiziksel minimum platform kabulü kısmi kalır. K-305 kapalıdır; önceki S3/RC/soak/IDE kanıtları yeni kaynağa aktarılmaz.

Bilinçli etki incelemesi: production Rust/bağımlılık/katman, grammar/public API/ABI, tanılar, normatif spec/RFC ve uyumluluk yüzeyi değişmiyor. Compiler/LSP kalıcı durum veya sabit performans iş yükü eklenmiyor; ADR045/049/046/069, performans/soak tarihçesi, compiler beyanı ve compiler regression kapsamı değişmez. Kaynak değişikliğiyle ilgili test/faz, katalog+tazelik, fmt/Clippy, koruklar ve sürekli güvenlik kapıları yine zorunludur. [Ortak kabul ve açık işler](urunlesme-k298-kabul-20261016.md).
