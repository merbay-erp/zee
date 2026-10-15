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
