# ADR-113 — Native Unix web worker için sınırlı SIGTERM drain

- **Durum:** kabul, dar Unix graceful shutdown
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** K-253/P13/B5, semantic-change + dogfood-change

## Bağlam ve karar

[B5 anonim provenance](../dogfood/zee-cekirdek/hizmet/README.md) gerçek native
listener sahibi üründe SIGTERM nazik kapanışı ister. ADR-106 önceki pilot
sınırı inflight kesintisini açık tutmuştur; yeni kabul bir compiler bugfix
olarak sınıflanmaz. Grammar, yeni CLI seçeneği, language intrinsic veya
public API açılmaz. Mevcut libc bağımlılığı kullanılır.

Private Unix CLI guard'ı SIGTERM disposition'ını süreç boyunca tek sahipli
kurar, eski sigaction'ı saklar ve bırakırken geri yükler. Signal handler'ı
yalnız lock-free AtomicBool store yapar; allocation/lock/log/IO/interpreter
state yoktur. Rust atomic guarantee'si [resmî std belgesindedir](https://doc.rust-lang.org/std/sync/atomic/index.html);
POSIX sigaction/SA_RESTART sözleşmesi [resmî POSIX sayfasındadır](https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigaction.html).

Listener nonblocking olur; mevcut istek yokken POLLIN/100ms poll yeni
socket'le hemen uyanır. TERM bayrağı accept öncesi ve accept yarışı sonrası
kontrol edilir. Admit edilen current read/handler/transaction/response
mevcut kaynak ve transaction kurallarıyla tamamlanır; sonra yeni istek
alınmadan listener kapanır. Interrupted TCP read aynı mutlak deadline'la
tekrarlanır, bütçe sıfırlanmaz. Kabul edilmiş akış açıkça blocking moda alınır.

## Bütçe, güvenlik ve state sınırı

Signal kurulum/restore syscall'ları küçük gerekçeli unsafe kapsar. İkinci
processwide guard reddedilir; restore OS hatası sahipliği boşaltmaz ve
sabit hata bildirir. Kapanmış sinyal state'i yeni guard kurulurken sıfırlanır.
CLI web adaptörü dışında global handler kurulmaz; Rust v1 facade değişmez.

Mevcut okuma10s/handler30s/write10s bütçeleri genel hard wall-clock garantisi
değildir. Supervisor default grace5s kalır; üst sınır60s ve explicit55s
örneği bounded current işi bitirme fırsatı verir. Dış grace sonunda KILL
escalation ve cleanup korunur; kesilemeyen kernel/disk/DB IO için tam drain
sözü verilmez. SIGINT hâlâ önceki native disposition'ıdır; supervisor'ın
SIGINT'i owned worker'a TERM göndermesi aynı dar drain hattını kullanır.

Windows/non-Unix signal drain, bütün backlog, socket activation/overlap,
yeni traffic gate, P14 zero downtime, request/rate-reject/DB pool metrics
ve in-process runtime config reload bu değişiklikte yoktur. Explicit memory
kota restart'la sıfırlanır; HUP config reread sıralı restart kanıtı ayrı tutulur.

## Kanıt ve kapılar

Önceki clean c67 native handler normalde3.010s200 verir, entry sonrası
TERM0.001s/0byte/exit-15 olur. Aday dirty965native: current handler200+exit0,
TERM sonrası yeni handler yok, idle exit bounded. Bunlar stage kanıtıdır;
final exact temiz kaynak kabulü ayrı alınır. Guard conflict/old handler,
1–128mask karşılaştırması, invalid poll fd ve readiness/idle poll testleri
vardır; mevcut bin testleri, fmt ve clippy yürütülür.

Dogfood korpusu/registered ürün kökü, spec11 normatif sınırı+exact testkanıtı,
faz owner, katman/SCC ve güvenlik defteri aynı toplu değişiklikte güncellenir.
Source commit ardından semantic-change/dogfood-change beyanı exactSHA ile
kaydedilir; bug/regression fixed_by kaydı icat edilmez. Yeni long-lived
signal state nedeniyle final exact kaynakta soak1800s zorunludur. RC/Miri
kampanyası bu stage adayına geçmiş kabul olarak taşınmaz.
