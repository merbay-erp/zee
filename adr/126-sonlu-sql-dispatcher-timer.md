# ADR-126 — Sonlu SQL dispatcher user timer ve belirsizlik kilidi (K-268/P09)

- **Durum:** kabul, dar owned host hazırlığı; gerçek Linux/systemd/native kabulü açık.
Güvenlik kaydı: GB-084 bilinçli admission/unknown-outcome sınırı.

54a8d6dc0efb060ae5644b37546da40ad03ee035 K258 mevcut --ticks1..100 SQL-only
hostunu sağlar. Yeni grammar, compiler/Rust export veya dış etki handler'ı
istenmiyor. Dar iç ürünleşme işi; finite çağrıyı user oneshot/timer ile
zamanlamak, unknown COMMIT sonrası otomatik admission'ı engellemek ve mevcut
owned grup temizliğiyle son bütçeyi bağlamaktır. Cron/calendar catchup, genel
daemon, provider retry veya tam P09 adoptionı bu kararın çıktısı değildir.

`scripts/finite_timer.py` tek named config ve fixed --ticks1 kullanır. Kaynak/
BuildJSON/binary/project/owned localhost TLS/credential sözleşmesi korunur.
Owner0700 state, nofollow0600 single-link file ve nonblocking exclusive flock
aynı isimli servis dışındaki manuel girişleri de serialleştirir. fsync edilmiş
inflight kaydı Popen'dan önce gelir; yalnız exit0+owned cleanup+exact postpins+
completion metadata fsync sonrası silinir. Belirsizlik/crash/TERM/exit3 yeni işi
kilitler. Resume, latch SHA ve aynı kaynak/binary'ye bağlı explicit operator
reconciliation makbuzu ister; makbuz DB sonucu veya imzalı trust kanıtı değildir.
Unknown sonucu doğrulamadan resume yapılmaz. Hiçbir receipt key'i silinecek
pathname veya yürütülecek komut vermez.

Reviewed509106d7 içinden dokuz exact tanım lifecycle_core.py modülüne
çıkarılır; LINEAGE.json her tanımı ve iki dosya hashini bağlar. Release/archive/
bundle kodu veya tomllib taşınmaz. WNOWAIT/session/grup
TERM/KILL/reap/capture sınırı değiştirilmez. Child raw stdout/stderr/argv
saklanmaz; finite capture sonrası yalnız metadata+hash kaydı tutulur. User
unit mixed kill mode main TERM ile helper iptalini başlatır, son cgroup KILL
systemd bütçesindedir. Mac üzerindeki kısa helper testleri gerçek Linux cgroup
ve systemd kabulü değildir. Source/host service kurulmadı veya başlatılmadı.

SMTP/PDF effect consumer, send sonucu belirsizken quarantine, provider dedupe/
ack sözleşmesi ve durable dosya publication ayrı gerçek kod işleridir. Mevcut
SQL transaction dıştaki e-posta teslimini kapsamaz. Kapsam ve test/reconciliation
protokolü [host rehberinde](../docs/sonlu-dispatcher-timer.md) kayıtlıdır.
Backlog/faz/güvenlik/doküman entegrasyonu aynı değişikliktedir; gerçek Linux
native acceptance ayrıca açıktır.

V2 state parent exact owner0700, held parent/state/tmp dirFD ve inode/namespace
denetimi kullanır. Çalıştırma yalnız Linux/proc profilidir; cwd/TMPDIR yalnız
current wrapper PID ve kendi held FDlerinden türetilir, pathname fallback yoktur.
Böylece rename/Popen aralığı yeni bir pathname'e yazım yönlendirmez; namespace
değişimi postcheckte pause'u korur. Python model testleri gerçek Linux/proc
ve native kabulü sayılmaz. Snapshot bounded loop/pre/post/path identity ile
okunur; observed swap/mutation reddi testlidir. Eksik fcntl veya bozuk helper
digest'i quiet exit2 verir. Başka UIDye karşı private parent sınırı vardır;
aynı UID operatorün activations arasında bütün state'i silmesi güven sınırı
dışıdır. Soak tetikleyen compiler/LSP kalıcı durum değişikliği yapılmaz.

V3 inceleme düzeltmesi: nativeSQL0+owned cleanup+postpins+durable completion
makbuzundan sonra başarılı inflight unlink ve başarısız son directory fsync,
exit3/Paused değildir. Sabit zee-sql-timer-clear-publication-1 çıkışı exit4 ile
completion-visible-durability-unconfirmed bildirir. Sonraki activation görünür
kayıt yokluğunda yeni tick alabilir; güç kaybında eski latch dönmesi muhafazakâr
pause üretir. Reconcile için aynı görünürlük sınırı ayrı
reconciliation-visible-durability-unconfirmed'dır. SQLunknown her zaman
clear öncesi kalır ve latch+exit3 admission'ı kapatır. Best-effort restore veya
IO arızasında dayanıklılık garantisi uydurulmaz. Reconcile yalnız exact schema,
matching SHA alanları ve positive bounded int created_ns kaydını kabul eder;
bozuk/ek/eksik/duplicate/nonfinite/type yanlış kayıtları silmeden reddeder.
