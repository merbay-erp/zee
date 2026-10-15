# S3 ikinci dalga teknik kabulü — 15 Ekim 2026

Dar yerel teknik kabulün test kaynağı S3 =
`66e4149e53451111d153f47283c219079a8d1cab`, ağaç
`936eb74d6ce14b1bf9709f03756e0184d7428fa2` olarak sabittir. İzleyen E commit'i
yalnız belge ve kanıt arşividir; E için yeni RC/benchmark kaynağı iddiası kurulmaz.

| Dilim | Gözlenen S3 sonucu | Kalan sınır |
|---|---|---|
| K-280/K-287 JetBrains | IDEA 2025.3.1 / LSP4IJ 0.21.0, kanonik başlatıcı; gerçek tamamlama isteği RPC 2 / 111 aday, ols→olsun; S004→A012→[]; biçimlendirme isteği RPC 3; shutdown 4 yanıtı ve exit; iki normal Quit ile exit 0; özgün süreç sahipliğiyle kapanış doğrulandı | Popup ayrıca kanıtlanmadı; dört spell-check Typo sürer; Windows/yayın, artımlı HIR/type ve C1 özel quickfix ayrı |
| K-281 imzalı PDF | Gerçek Zee stdio istek/yanıt/continuation ve üç sayfa/font/görsel; dokuz vaka; sentetik özel anahtarlar kaldırıldı | Yerel amaç bağlı trusted host; gerçek katalog/canlı sağlayıcı, genel FFI ve hostile PDF sandbox değil |
| K-282/K-295 dispatcher | Linux gerçek SQL iki tick, canonical OnCalendar, TERM/unknown/no-replay ve adapter ACK: dört vaka | Açık pilot development kanalı, sentetik PG; durable unknown latch korundu; genel provider exactly-once/adoption değil |
| K-283 HTTP | Linux canonical user-service/PG drift-restore/HUP/restart/TERM/watcherlost: yedi vaka | Readiness false liveness200 olabilir; ingress reddi ve zero downtime garantisi değil |
| K-284 kapsam | Üç gerçek CLI/native kohortu; payda 14; dene 4/14 ve HTTP 6/14; çağrılmayan işlem sıfırda kaldı; origin/guard/FIFO/quiet retleri geçti | Üretici candidate etiketi korundu, temiz S3 ve kanonik ikili dışarıdan bağlandı; genel kapsama/ürün iddiası değil |
| K-285 SQL modülü | İki generic schema, parametreli CRUD, gerçek DDL down/archive/up, escaped HTML ve unknown no-replay: dokuz MacPG vaka | DB+FS atomik değil, pending/manual review ve ürün adoptionı ayrı |
| K-286 dene köprüsü | Birim testinden sonra pinli native suite; dört vaka/19HTTP, iki farklı fresh PG kimliği; altı admission negatif | Fresh DB bütün HTTP commitlerini rollback etmez |
| K-287 LSP | Kanonik stdio açık bağımlılık tanısı open/repair/break/close, UTF16 invalid-range atomik ret ve aggregate budget exact tek -32001; temiz kaynakta 1.800 saniyelik soak, 360 örnek | Cache yalnız token/yaprak AST; grafik/kilit/bağlama/tür taze; lint varsayılan kapalı; incremental HIR yok |
| K-289 TR–EN | Katalog/bileşen/layout/slot ile 10 HTTP ve 4 Zee vakası | Yeni çevir builtin ve gerçek ürün benimsenmesi yok |
| K-290 starter | İçerik/kimlik/medya/parametreli SQL/PNG: gerçek PG+HTTPS14 | İnsan rehber kabulü, üretim/platform ve adoption ayrı |
| K-291 devir | Mixed code revision dokuz vaka / 160 istek / dört dalga | Genel SLA veya DB şema migrationı değil |
| K-292/K-296 fault | İki native worker-loss/stale-gate; Build kimliği doğrulanmış liveness ve startup/readiness false/old gate bayt+mtime/cleanup | Kontrollü Python resolver native DNS iptali kanıtı değil |
| K-294/GB-110 | Kanonik CLI ile 26 native stdio vakası, sıfır control tanı baytı; exact regression/fixed_by kaynak382beed; nonempty dedicated native korpus replay | GB-110 exact S3 native/regression/RC kanıtıyla dar kapsamda kapalı |
| macOS teslim | Kanonik iki temiz klonda eş CLI/LSP, iki eş ZIP, restore/fsck bundle; install/uninstall/tamper sekiz vaka | Unsigned/local; release etiketi, gerçek imza veya public dağıtım yok |
| Linux teslim | Kanonik iki temiz klonda eş CLI/LSP, platform içinde bayt eş iki kaynak ZIP; Mac ile 1.822 dosya içerik eşliği, restore/fsck bundle ve kurulum8 | İlk PATH/unzip reddi korunur; corrected owned PATH dışında kaynak/araç/eşik değişmedi; fiziksel cihaz kabulü değil |
| Retained native | TLS8, göç1, parola11, queue21, metrics8, job11, SQL/SMTP11, readiness15+TTL5, web/PG4 ve PDF8;14 sahipli adım | 25 gerçek ignored vaka, dedicated owner ve release ölçümüyle ayrı eşlendi; alt senaryolar unique Cargo sayısına eklenmez |
| 1800s soak | 1.800,859579 saniye, 360 örnek; compiler 20.112→20.144 KiB, LSP 17.856→19.456 KiB; ikili/kaynak sabit | İki koşul birlikte >%10 ve >32768KiB aşılmadı; benchmark tabanı/genel SLA değil |
| 1/41 açık belge | 20.081 satır / 40 birim, 5 ısınma ve 20 ölçüm; p95 19,208/122,649 ms; max 20,393/125,044 ms | Fuzz/build bittikten sonra release gözlemi; p95 bağımsız hesaplandı; paylaşımlı makinede 200 ms katı eşik veya gerçek ürün SLA'sı değil |
| RC | Dört AddressSanitizer hedefi her biri 1.801 saniye; Miri iki grupta 3 test; canonical export/check ve aynı temiz S3 kapısı geçti | Tam Cargo 989 başarılı / 0 başarısız / 25 varsayılan dışı vaka; unsigned yerel teknik kanıt, release etiketi değil |

## Başarısız koşuların korunması

S2 eski kaynak/fault oracle başarısızlığı yeni S3 kaynağına dönüştürülmez.
S3 Linux K-282 dört vaka geçti; ilk K-283, beklenen paused dispatcher testinin
bıraktığı kendi failed-unit bayrağı nedeniyle vaka0 öncesi manager önkoşulunda
durdu. Yalnız issued zee-dispatch.service, LoadState not-found/failed ve boş
cgroup doğrulanarak reset-failed edildi. SQL inflight latch SHA256
`9390eaecfc326878a771e4c18441303e130163e5ed1718353c95cb01103440c4` aynı kaldı;
SQL tekrarı ve yabancı unit temizliği yapılmadı. Yeni ayrı HTTP kohortu yedi
vakayı geçti. İlk failure makbuzu aynen korunur.

İlk macOS kurulum aracı params:null içeren eski LSP smoke girdisi yüzünden
reddedildi. JSON-RPC params nesne/dizi sözleşmesi korunarak yeni dış smoke
shutdown/exit'te params alanını atlar; iki exact yanıt, capabilities, shutdown
null sonucu, stderr0, exit0 ve ikili hash koşulları aynı kalır. Sekiz kabul yeni
çıktı kökünde geçti; ilk kısmi fixture ve failure kaydı silinmedi.

JetBrains trace ayarı ilk UI'da uygulanırken LSP4IJ settings görünümü EDT
hatası üretti. Vendor log korundu, hata raporu gönderilmedi. Trace açık ikinci
normal oturumda gerçek completion/formatting/diagnostic/shutdown/exit gözlendi.
Git'e aktarılan UI JSON, araç çıktılarından küratörlü gözlem ve exact trace
kesitleridir; tam ham konsol dışa aktarımı veya yeni screenshot dosyası değildir.
İlk UI kaydındaki trace-pending durumu tarihsel olarak korunur.

## Açık işler

UTF-16 eşitleme kabulü, BMP dışı karakterlerde hover/rename ve tanı sütunu
sınırlarını kapatmaz. Artımlı HIR/tür analizi ve taşıma/UI/lint gecikme kabulü
ayrı açık işlerdir.

K-288/C1 özel beş tanı+hızlı düzeltme zinciri ve K-293/P33 koordineli reproducer
araç incelemesi retleri nedeniyle açıktır. Bu iki iş dış girdiye indirgenmez
ve bu kabul tarafından kapanmış gösterilmez. Opt-in lint C1 kabulü değildir.

P08 beş anonim gerçek site ve exact ürün provenance'ı; gerçek katılımcı,
10 çocuk/öğrenci ve 5 profesyonel çalışması; fiziksel Windows/minOS; syntax takvimi
14 Ekim–11 Kasım; lisans/destek/publisher ve canlı sağlayıcı kararları açık kalır.
Bu yerel teknik kanıt tam V1, signed release, ürün adoptionı veya üretim SLA'sı
değildir.

## Belge etkisi

Bu final arşiv compiler/src, grammar, tanı, public facade, uyumluluk yüzeyi,
production modül/kenar, bağımlılık veya normatif spec maddesi değiştirmez.
Spec/RFC/ADR indeksleri, hata kataloğu ve katman/SCC fixture'ları bu nedenle
yeni davranış kaydı istemez. İlgili ADR'lere S3 kanıt bağlantısı eklenir;
backlog/plan/master/v1/release/README ve öğretici rehberler kabul ile açık
sınırı birlikte gösterir. Fuzz ve soak tarihçesine exact S3 actual satırları
eklenir. ADR045/049 ve performans-gecmisi-v2.tsv etkisi incelenir: paylaşımlı
makine gözlemi yeni sabit hard-gate tabanı olarak kaydedilmez.

Platformlar arası kaynak ZIP karşılaştırmasında 1.822 dosyanın bütün ad ve byte
içerikleri aynı exact Git snapshot'ına eşleşti. ZIP byte hashleri farklıdır:
1.822 girdinin DOS saat alanı Mac yerel saatinde 12:36:38, Linux UTC saatinde 09:36:38
kaydedildi. Her platform kendi iki arşivinde byte eşliğini geçti; platformlar
arası ZIP byte eşliği ileri sürülmez. İlk kurulum dış koşucusunun bu fazladan
eşlik assert'i kurulum başlamadan reddedildi ve ayrı saklandı; yeni koşu
kanonik entry/manifest/Git içerik eşliğini açıkça doğrular.

İlk aynı-S3 birleşik RC kapısı boş disk alanı 10 GiB altına düşünce durdu. Export/check başarılıydı; tam güvenlik kapısının sonucu bilinmiyor. İlk makbuz korundu. İkinci koşuda iki Cargo grubu başarısız oldu; sonraki seçili koşuda listener geçti, worker grubunda farklı bir hazırlık vakası başarısız oldu. Bu kayıtlar korundu. Diğer oturumdaki temizlik tamamlandıktan sonra seçili tekrar ve seri tam v3 koşusu ayrı kanıtlardır. Son kabul v3 sonucuna dayanır; testler, zaman sınırları ve kaynak değişmedi, tam koşunun test paralelliği bire indirildi. Zamanlama hassasiyetinin kök nedeni kanıtlanmış sayılmaz.

## Kanıt bağlantısı

[Seçili arşiv ve ham kapsam](incelemeler/urunlesme-20261014/final-66e4149/README.md), [exact S3 RC kapısı](incelemeler/urunlesme-20261014/final-66e4149/rc/GATE-RECEIPT.json), [25 ignored owner eşliği](incelemeler/urunlesme-20261014/final-66e4149/ignored/OWNER-REVIEW.json), [JetBrains protokol gözlemi](incelemeler/urunlesme-20261014/final-66e4149/jetbrains/SECOND-UI-OBSERVATIONS.json). Ham dış makbuzlarla derived temsillerin farkı arşiv manifestinde açıktır; hash kimlik doğrulama değildir.
