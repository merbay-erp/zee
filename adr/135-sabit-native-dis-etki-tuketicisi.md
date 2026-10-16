# ADR-135 — Sabit native dış etki tüketicisi (K-277/P09)

- **Durum:** kabul; temiz S owned loopback nativeeffect11 kabulü var; fullP09/ürün/platform açık
- **Tarih:** 15 Ekim 2026
- **İlgili:** ADR107/114/117, ADR096, GB-093

Kayıtlı B1/B2 ihtiyacı kalıcı katalog kuyruğundan gerçek e-posta etkisine geçmeyi
ister; SQL-only idempotent handler SMTP transaction'ı değildir. Aynı DB transaction
içinde SMTP'yi exactly-once yapma, C029'dan pre-DATA çıkarma veya timeout sonucunda
otomatik replay güvenli değildir. Yeni compiler/grammar yerine mevcut Zee SQL eylem
ve MIME kalıbıyla ayrı fixed finite owned-local profile sınanır. Native consumer
aktarımı production Rust/Python SMTP bypass değildir.

Kalıcı local pending guard claim'den önce; owner/token/generation30s lease ile
ayrı committed SQL effect-start latch SMTP'den öncedir. Sonraki native send live
hak/latch'i yeniden kontrol eder. Latch otomatik expiry/reaper ile açılmaz.
Known pre-effect SQL reddi, sıfır SMTP invocation ve matching lease/latch yokluğunda
ayrı acknowledged safe-fail ile en çok toplam3 ve1/2sn backoff yapabilir. Her
started-send/unknown transaction sonucu durur; ack başarılı olsa bile client
bilinmiyorsa manuel inceleme gerekir. Otomatik reset/reconcile/re-delivery yoktur.
Ayrı readonly SQL durum okuması dış SMTP sonucunun yokluğunu ispatlamaz.

Outer Supervisor issued group leader WNOWAIT ile korunur; native çocuklar aynı
grubu devralır. Host direct-child bounded capture/TERM/KILL/wait yapar; öldürülen
hostun grubu retained outer owner tarafından kapatılır. PID keşfi/adoption yoktur.
Credential/file/source/capture/deadline sınırları fixed profile'da tutulur. Local
SQL/SMTP fixtures explicit loopback plaintext'tir; production TLS/provider/onboarding
kanıtı değildir. Existing SQL-only dispatcher, canonical helpers ve compiler
kaynağı değişmez; hiçbir eski tabloya mixed worker verilmez.

Hazırlık8 ve actual11, gerçek SMTP connection/DATA sayısı ile bağımsız PG observer
state/ack'ini ayırır. COMMIT before/after actual tel kontrolü C027 belirsizliğini
üretir; sentetik PDF eki mevcut native MIME'dan geçer. Kaynak Build4d8/d53 dirty
adayı saklanır, finalclean kimliği uydurulmaz. Tam P09, B2 PDFgeneration/K275ürün
entegrasyonu, gerçek Gmail/M365/catalog/P08/calendar/platform kabulü açıktır.

İlk test-only COMMIT fixture bağlantı-sırası varsayımı başarısızdı ve korunmuştur;
son fixed-operation router gerçek acknowledged COMMIT'e fault uygular. Ad-hoc
probe teardown hatasında kaybolan original ownership yetkisi tarihsel UNVERIFIED
sınırdır; current-absence ile geçmiş cleanup kanıtı icat edilmez. Son kaynak spawn
öncesi ExitStack cleanup kaydı ve failed-receipt finally ile bu yolu daraltır.
Bağımsız peer sonucu ve root final acceptance olmadan entegrasyon READY değildir.

## Revision2: bilinen tamamlanmanın görünür yayını ve strict inventory

Bilinen `done`, `idle` veya `dead_letter` sonucu sonrasında guard unlink görünür
olup son directory fsync başarısızsa sonuç **exit4** ve
`known_completion_visible_durability_unconfirmed` olur; `completed_status` bilinen
sonucu taşır. Guard görünür olarak yoktur, dolayısıyla yeni admission engellendi
veya `paused_manual` olduğu iddia edilmez. Tekrar çağrı yeni işi alabilir; önceki
etkiyi tekrar gönderme izni verilmez. Powerloss eski guard'ı geri getirirse
konservatif ret mümkündür. Son result receipt publication fsync'i de başarısızsa
aynı exit4/sabit publication-unconfirmed metadata korunur. Bu, SMTP/SQL effect
outcome belirsizliğinden ayrı yerel dayanıklılık sınırıdır. Bilinmeyen
claim/prepare/send/ack guard'ı tutar ve exit3 kalır; guard kurulmadan identity/build
retleri exit2'dir. Başarılı fsync dahil normal completion exit0'dır.

State inventory `Path.iterdir` eager allocation kullanmaz: streaming `os.scandir`
17'nci girdide reddeder ve iterator kapanır. Hazırlık8 içindeki durable alt-vakalar
permanent postclear fsync failure exit4/visible-guard/allowed-next-idle sınırını
kontrollü mock-native/real-filesystem modeliyle; gerçek17-entry retini ve
1000-entry kontrollü iteratorın yalnız17 gözlemle reddini sınar. Bunlar native SQL
veya SMTP kabulü değildir ve yeni native vaka olarak sayılmaz.

Fixture kanonik `compiler/tests/fixtures/native-effect-consumer` altındadır.
Native koşucu framework kökünü dolaşmaz; tam14 sabit source-role yolu, dosya128KiB
ve toplam2MiB sınırında hashlenir. Runtime/proje hashleri aynı kampanyanın sonunda
tekrar eşlenir. Revision1 frozen14/43 ve iki peer failure reproducer korunur;
R1/R2 bu ilk host feature kaynağından önce düzeltilen aday bulgularıdır.

## Revision3: yalnız pinlenen dört fixture dosyası

Native harness proje kurarken recursive discovery yapmaz. Yalnız `ana.dil`,
`catalog.pdf`, `gocler/0001_effect.sql`, `proje.dil` sırası sabittir; her biri
regular/nofollow/stable snapshot ve128KiB bound ile okunan aynı bytes üzerinden
yazılır. Ek SQL, `.dil` ve FIFO dosyası ne açılır ne kopyalanır. Hazırlık8 içindeki
actual filesystem alt-vakası extraSQL/source/FIFO varlığında yalnız bu dört
dosyanın tam beklenen bytes ile kurulduğunu sınar; SQL/SMTP kabulü değildir.
Revision2 kaynak/kanıt/peer paketleri değişmez. R3 yalnız acceptance harness ve
alt-vakadır; runtime host/compiler davranışı değişmez ve yeni native case sayılmaz.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

S common nativeSMTP/PGfault11 actual fixed sentetik loopback kabulü geçti;14canonical GitS source-role, source/binary begin-end ve readonly PG/SMTPcount oracle ayrı korundu. Known preeffectSQL sıfır-send sınırlı retry; C029/DATA/ackCOMMIT belirsizliği manualpause/no replay. Tarihsel ad-hoc authoritylost UNVERIFIED kaydı currentabsence veya yeni ownedcleanup ile geriye dönük kapanmaz. FullP09/provider/realcatalog/PDFgenerationintegration açık.

[Güncel dar kaynak kanıtı](../docs/urunlesme-final-kabul-20261015.md).

## K-311 / GB-120 — test harness lifecycle: açık

K-311 / GB-120 düşük ve açık: effect kabul harness’inin OperationProxy.close yolunda tek socket-close hatası kalan özgün thread ve COMMIT sahibi cleanup çağrılarını atlayabiliyordu. Child-free karşı örnekte1 beklenen failure/exit1/reaped doğrulandı; bu gerçek OS/PG/SMTP sızıntısı veya diskte PASS gözlemi değildir.

Düzeltme resource-free constructor/start öncesi registration, partial allocation retained ownership, kalan cleanup denemeleri/ilk hata kimliği, yapışkan iptal/handler restore ve failed-only publication sağlar. Work180 ve tek cleanup90 mutlak bütçeleri uzatılmaz; H/WNOWAIT özgün sahipliği, foreign kaynaklara dokunmama, normal foreground PG SIGINT0/reaped/noFallback/readiness/SHOW identity ve private-root tutma sınırları korunur. Bu normal PG yollarının gerçekten kabulü gelecekteki exact-source native kampanyasının konusudur.

PG profili explicit plaintext loopback, TLS kapalı/startup196608; TLS6e5 shared helper’a çevrilmez. SQL/SMTP latch, C029/DATA/ackCOMMIT belirsizliği manualpause/no-replay; native11 gözlemi/60need ve14 sabit source-role değişmez. İç helper ayrıca fixedpin ile tüketilir. Production175/compiler/src/H/ürün API/ABI/grammar/performance değişmez. ZEE_EFFECT_NATIVE_OUT yalnız internal ignored native test için fresh absolute output seçimidir; canonical parent/NotFound kontrolü ve default hedef yolu korunur, ürün CLI kapasitesi değildir.

Child-free1/0.003sn expectedFAIL/exit1/reaped; actualpeer812690ff68322169e9fa96770f397a503418f33c5100b803a432e62459cfdcb9. Yalnız stop.set/owned-sockets.close gözlendi; sonraki beş cleanup eylemi atlandı ve ilk hata korundu. Tarihsel ad-hoc UNVERIFIED kaydı geriye dönük kapanmaz.

Eski baseline8 attempt2:8/0.947sn PASS; peerab299b4506236e838d97de88a87271bdae42b005e17c8a1cd2b4430fda371b1f. İlk precommand driver KeyError ayrı başarısız hazırlıktır. Ordinary9 fixture2:9/1.101sn PASS/özgün owner0/reaped/1.1660061669535935sn; peerc3acfa3ac563a1ad8f91bac982bc29ad3805bdeb59e6fa0605792e509da8ec9d. Fixture1 setup/missing-script hatası testlerden önceydi; source kusuru veya testFAIL değildir.

Lifecycle27:27/0.081sn PASS, peer7cadf3e0bfaaf1a2c08c52d36bb906cd47bd2c4cacd0ad81b01b8049ae2b1d85. Aynı lifecycle27+PG18’i seçen preflight45:45/0.153sn PASS/özgün owner0/reaped/0.27195924997795373sn; peerbcf933e37f25472a49d419ab9415a93e0e89fbe0ce796361a1a17765a216876a. Ordinary9 nested45’i yeniden toplamlatmaz;45/27/18 native veya yeni Cargo sahibi değildir.

Standalone PG18:18/0.051sn PASS, peera6268fd4a1e08e23859d97f8eddf3f02c646f55e638c12da9fcd26f4f37cfcf4. Fake command/readiness/SIGINT/FD/uncertainty modeli; gerçek PG/native yürütümü değildir.

Kaynak/test/belge aynı atomik dilimdedir. Son beş kaynak pininin tam envanteri:

| Kaynak/test dosyası | SHA-256 | Bayt |
| --- | --- | ---: |
| `scripts/tests/effect-dispatcher-native-test.py` | `c4a91293f32d8f7463a922f15928c70cd4e058d3e0ca7a922966774f8c1bcf1f` | 46884 |
| `scripts/tests/effect-dispatcher-test.py` | `3a95add8341e83b934874cab497f64589fc2c2ca99debaa2fb4c9fdcfd7cd0f5` | 11846 |
| `scripts/tests/effect-pg-owner.py` | `37e72f521260f5dc039840bd413efa405164d0a86ea5601e4c5abfd820e0611e` | 21768 |
| `scripts/tests/effect-pg-owner-preflight-test.py` | `922d58f149c5bce69d9ef66c6523a61689b0c1c7ba842ae84becbbb826f51cfa` | 41082 |
| `compiler/tests/native_effect_consumer_testi.rs` | `8e8d3d23016dfefe1d108617602ede6fc920ec4d2d4601a5c0c16dbcf0024fa9` | 4482 |

Rust8e8d output tasarımı45fed bağımsız incelemesine bağlıdır; rootaudit5a788 format/check0-empty ve byteexact sonucunu kaydeder, ayrı formatter raw/owner makbuzu sunulmadığından bu sınırlı root attestation’dır. Prospective source union358/175 için k311-source-pins-DRAFT-1 manifesti türetildi; bu taslak pin envanteri source15/temiz kaynak/native/final kabulü değildir ve frozen değildir.14role içindeki docs/native-effect-consumer.md ürün komutları/semantik değişmediği için kaynak-pinli olarak korunur.

Tam kaynak15/temiz kaynak ve bağımsız sourcepeer; aynı SHA’da özgün dış owner ile native11/60need/normalPG/readiness/retained tüm kaynak cleanup/public proof; ardından exact belge overlay’inde final11 ve temiz kapanış kaydı zorunludur. Native/final kabulü açık; K311/GB120 düşük-açık kalır. K306–K310 dar kapanışları kendi exact kaynaklarında, ilk başarısız denemeler ve tarihsel UNVERIFIED sınırlar yerinde kalır. Effect şu an tek açık PG tüketicisidir; ancak dar effect kapanışı sonrasında bu beş-harness PG kuyruğu tamamlanır.39 statü korunur; P09/P13/P20/P23/P36 kısmi, P37 sürekli, GB046/109 açık. Genel P09/ürün/provider/harici proje/insan/platform, RC/soak/IDE ve C1/K288-P33/K293 ret sınırları değişmez.
