# 14 Ekim 2026 onaylı ürünleşme uygulama kaydı

[Son K-245–K-249 pilot kabulü](urunlesme-pilot-k245-k249.md) ayrı temiz
exact kaynakta faz/native/host/iki-klon kapılarını kaydeder.

[K-242/K-243 devam raporu](urunlesme-devam-20261014.md), yeni yerel
COMMIT fault kanıtını ve kaynak ZIP düzeltmesini ayrı exact kaynakta izler.
K-244 literal kaldırma düzeltmesi yerel PowerShell kabulündedir; fiziksel
Windows ve yeni uzak CI kanıtı ayrı açıktır.

Kullanıcı A1–A6, B1–B9, C1–C6, D1–D4, E ve ZEE-001–018'in birlikte
değerlendirildiği planı 14 Ekim 2026'da onayladı. Exact ürün talebi
`cd5d12f0008620819e6167af5146e83503fd2b38`, başlangıç compiler ağacı
`be5699c3913fe66594fb9d9509253fd9a0b7f5b9`tir. Dış statik inceleme Cargo,
canlı ürün veya uzak CI yeniden kabulü değildir.

Makine okunur [39 işlik eşleme](urunlesme-plan-v1.tsv) bütün talep kimliklerini
korur. `P-*` plan kimliğidir; K kaydı yalnız uygulanmış/onaylı atomik dilime
ayrılır. Açık/ertelenmiş/harici kanıt satırı tamamlanmış özellik değildir.

## Kullanıcı kararları ve gerçek dış sınırlar

- Lisans seçimi bekletildi; teknik işlere devam edilir. Kaynak-erişilebilir,
  açık kaynak veya ticari lisans kullanıcı adına seçilmez.
- [Sınırlı pilot destek](urun-destek-politikasi.md) seçildi; güvenlik ve
  uyumluluk kapılarıyla sürüm. LTS veya sabit takvim sözü verilmedi.
- Kullanıcı gerçek K-161/K-162 oturumu, temiz fiziksel Windows/minimum OS
  cihazı ve GB-046 destek yanıtı bulunmadığını bildirdi. Geliştirici Mac'i
  fiziksel fakat temiz değildir; temiz makine kabulü yerine kullanılamaz.
- [GitHub destek taslağını](incelemeler/k215/github-destek-talebi.md) kullanıcı
  gönderecek. Bu çalışma destek mesajı gönderimi veya public açma değildir.
- Mevcut resmî marka/site hattı sürer. Kaynak metni düzeltmesi canlı deploy
  edilmiş sayılmaz; dağıtım ve görünürlük ayrı işlem yetkisi ister.

## Sıra ve yeniden açılmayan kapsam

1. P01 kaynak/ZIP/CI eşliği; P31 TLS/NULL ve P33/P34 kalan güvenlik kanıtı.
2. P03 bilinmeyen seçenek; P02 deterministik build kimliği; P05 uyumluluk
   işletimi. A6(2) genel config, küçük tanı düzeltmesine yüklenmez.
3. P08 anonim ürün korpusu; P15 parola rehash ve P16 form kabulü; P19 yalnız
   mevcut maliyeti gerçek temiz projede ölçme. Cache ölçülmüş ihtiyaca bağlıdır.
4. P29 platform kanıtı/P06 dağıtım; P12 göç/P13 işletim/P25 credential ve
   P14 draining/yeniden başlatma.
5. P09 kalıcı iş/P10 MIME; P11 PDF; P18–P23 ürün kompozisyon ve tooling.
6. P30 yalnız insan/freeze/platform/ürün/güvenlik birleşik kapıları sonrası.

B-001/B-002 insan hattı, K-174 14 Ekim–11 Kasım penceresi, K-192 cihaz
hattı ve GB-046 destek teyidi teknik testle ikame edilmez. P17 S3, P26
sanitizer, P35 yeni ergonomi grameri ve P38 backend varsayılan olarak
ertelenmiştir. Tek koşuda çalışma isteği kayıtsız veya sahte kapanış değildir.

K-218/K-219/K-220 dil makine kabulü korunur. K-206 bounded toplama ve
GB-041 tek-frame sınırı kapalıdır; global RSS garantisi değildir. K-213
birleşmesi canlı parola/veritabanı göçü sayılmaz. Rust compiler playground'u
Zee→WASM codegen değildir.

## Kaynak ve CI kanıtı

[Başlangıç eşleme JSON'u](incelemeler/urunlesme-20261014/source-ci.json)
1036 Git archive kaynağını ZIP ve dosya manifestiyle birebir karşılaştırır.
ZIP SHA-256 `1555749c7783e7b34a72b61b85985e83d1770494d0769a45ef4b768750664ede`;
ZIP comment exact başlangıç HEAD'idir. Kaynak eşliği Git archive'a aittir;
rapor hazırlanırken uygulama dalındaki çalışma ağacı kirli olabilir.

Gitsrv özel aynanın `master`ı exact be5699c olarak görüldü; #30 commit
durumu başarıdır. Job ayrıntısı 404, artefakt indirme kullanıcı tarafından
reddedilmiştir. #29 kaynak SHA'sı 3d7ff93, compiler kaynak commit'i
488dc7'dir; #29'dan be5699c'ye compiler tree aynı, helper/belge farkı vardır.
#29 veya visible #30 status yeni uygulama commit'lerinin CI kabulü değildir.
GitHub ana kaynak 404 ve Enterprise ayna SSO erişim sınırı ayrı açıktır;
silinme veya kaynak değiştirme sonucu çıkarılmaz. Aynaya push yapılmaz.

## Atomik kapanış

Her kaynak işi ilgili spec/RFC/ADR, olumlu/olumsuz test, tanı/regression,
faz/mimari/SCC/API/uyumluluk, tazelik ve güvenlik kayıtlarıyla birlikte ilerler.
Compiler source commit'inin exact semantic/core-freeze beyanı ve bugfix
`fixed_by` kaydı takip eden provenance commit'ine aittir. Kaynak şimdi-belge
sonra borcu bırakılmaz. [AGENTS.md](../AGENTS.md) ve [runbook](surum-runbook.md)
kapıları bağlayıcıdır. Cache/kalıcı durum ekleyen iş exact temiz kaynakta
1800 saniye soak; release exact ASan/Miri/imza/kurulum kabulü ister.

Normatif spec maddeleri exact test işlevine bağlanır; tasarım boşluğu test
eklenerek kapatılmaz. Tamamlandı durumu yalnız gerçek kabulden sonra yazılır;
yerel kabul, uzak CI, canlı ürün ve release birbirinin yerine geçmez.

## Yerel teknik kapanış kayıtları

[Temiz native checkpoint](incelemeler/urunlesme-20261014/native-7127c7d/README.md)
TLS 4/4, göç 1/1 ve rehash 7/7 gerçek PostgreSQL kabulünü taşır.
[K-226 iki-klon](incelemeler/urunlesme-20261014/build-99f31d7/README.md)
exact kaynakta eş hash üretmiştir; imzasız çıktı yayın kabulü değildir.

Mevcut primary GitHub HTTPS/connector ve mevcut SSH transport üzerinden
read-only kaynak okuma sonuçları 404/repository-not-found; Enterprise ayna
SSO sınırı 403'tür. Primary master yetkisi/SHA'sı bilinmiyor. Yerel onaylı
uygulama dalı uzak master değişikliği veya yeni exact CI kabulü sayılmaz.

Kullanıcı P08/A5 anonim beş tam site ve çekirdek korpusunun henüz hazır
olmadığını teyit etti; kapı açık tutulur ve mevcut teknik dilimler sürer.
Anonim küçük fixture sayısı tam ürün N/N kabulü değildir.

K-236/GB-060, P10 SMTP diliminde incelemeyle bulunan AUTH echo sızıntısını
sabit native/runtime C029 maskesine kapatır. Gerçek loopback AUTH535,
explicit URL hatası ve CLI testleri ile [SMTP regression kipi](semantic-regresyon-korpusu.md)
kullanılır. Kaynak SHA ve exact fixed_by takip provenance kaydında yazılır;
P09 kuyruk, canlı provider/ürün ve K-237 yanıt bütçesi ayrı kalır.

K-237 P10/P34 inherited SMTP güvenlik dilimidir: [ADR-100](../adr/100-smtp-yanit-okuma-siniri.md)/GB-061 mevcut 16 KiB bütçeyi LF/EOF öncesinde zorlar; actualhelper/native/old-body negatif kanıtı vardır. K-236 AUTH gizlilik kapanışından ayrıdır. Global RSS/full audit, queue/provider/ürün, yeni exact CI ve insan/platform/syntax kapıları açık kalır; yeni benchmark tabanı/cache/state yoktur.

## K-239 inceleme gizliliği ve tarihsel kanıt

İnceleme raporuna eklenen üç özel ayna bağlantısı rol/hash kaydına
dönüştürüldü. Yalnız bu izole iş serisi yeniden kuruldu;
[exact SHA eşlemesi](incelemeler/urunlesme-20261014/sanitized-sha-map.tsv)
regression/freeze ve sentetik TLS kökenleriyle tutarlıdır. Eski native ve
iki-klon raporları özgün kaynaklarını adlandırır; eşlenmiş yeni SHA kabulü
değildir. Güvenlik yaması sonrası temiz kabul ayrıca kaydedilir.

## Son yerel teknik kabul ve tarihsel sınırlar

[14 Ekim teslim raporu](urunlesme-sonucu-20261014.md) exact
`2e970e47d59e2408eff836cdacef9bd0fe0b3028` kaynağında faz 887/893
(0 başarısız, 6 dedicated atlama, 138.698 sn), ayrı TLS/bütçe 5/5, göç 1/1,
parola CAS 7/7 ve sürekli güvenlik/sır kapısının kabulünü kaydeder.
[K-241 / ADR-102](../adr/102-surum-klonu-ve-build-kimligi.md) sonrası
[kanonik iki-klon kanıtı](incelemeler/urunlesme-20261014/final-2e970e4/build-proof.json)
önce/sonra temiz kaynak ve gerçek a/b/çıktı ikililerinde exact SHA ile
`dirty: false` kimliği, eş `dil`/`dillsp` hashlerini doğrular. P02 yerel
kabul; P06 kısmi, P37 sürekli kalır. Yeni Linux/Windows, imza ve RC kabulü
bu yerel macOS koşusundan çıkarılmaz.

a2b8557 [kimlik tekrar kontrolü](incelemeler/urunlesme-20261014/final-a2b8557/build-identity-recheck.json)
`dirty: true` ile temiz ikili kimliğini reddeder; eski faz/native/hash
kanıtları tarihsel kalır. e9145844 yalnız checksum JSON biçimi düzeltmesiyle
yerini 2e970e4'e bıraktı; yeni sır istisnası eklenmedi ve eski raporlar
yeniden etiketlenmedi. Ayrı koşularda örtüşen vakalar yeni toplam gibi
birleştirilmez. Takip belge commit'i testleri kendi HEAD'ine taşımaz;
uzak CI/platform/ürün/RC kabulü değildir. 39 plan maddesinin açık kapasitesi
ve kullanıcıca bekletilen kararlar korunur.

K-245/P25 credential host pilotu (ADR-105/GB-068) mevcut env izinlerini
koruyan POSIX dosya köprüsüdür;14 actual vaka native allowlist/SMTP dahil
yürütülür. Temiz c67a69e host14 kabulü tamamlandı; Windows/systemd/canlı
ürün ve file-native sır inheritance sınırı tam P25 olarak kapanmaz.

K-246/P13 (ADR-106/GB-069) worker host identity+ayrı readiness ve sahipli
cleanup pilotudur; public launcher cleanup bug'ı temiz c67a69e üzerinde 25-vaka kabulüyle kapandı.
Tam request drain/reload/metrics ve P14 socket devir açıktır.
K-247/P09 (ADR-107/GB-070) dar idempotent SQL queue helper'ıdır;11 native
PG crash/COMMIT observer ve16 validator temiz c67a69e kabulündedir. Canlı ürün adoptionı, SMTPretry ve full P08 açık kalır.

K-249/P21 (ADR-109/GB-071) tek anonim TR–EN article fragmenti, plain
Metin slotları ve sabit sözlüktür. Dört kaynak testi, yedi native CLI
snapshot/XSS/ret senaryosu mevcut grammar/API ile temiz c67a69e kabulündedir. Genel layout/i18n ve gerçek ürün/P08 kabulü açıktır.

K-245–K-249 [temiz pilot kabulü](urunlesme-pilot-k245-k249.md)
`c67a69e6ed9c24c4a540a55eeaaa986a348a2de3` üzerinde 895/901 faz,
0 başarısız/6 dedicated atlama; ayrıca 28 PostgreSQL, credential14,
supervisor25 ve iki Python sürümünde TR–EN7+4 ile kanıtlıdır. İki temiz
klondan Mac ARM64 ikilileri eş ve dirty=false; yeni sır istisnası yoktur.
Lisans/insan/fiziksel/tam ürün/uzak CI/RC kapıları ve genel P09/P13/P14/
P21/P25 kapsamı açık kalır. Takip belge commit'i test kaynağı değildir.

## K-250 — PostgreSQL statement fault kanıtı

[ADR-111](../adr/111-postgresql-statement-fault-kaniti.md): gerçek 40P01 ve
40001, no-retry sayaçları, outer rollback onayı ve bağımsız durable observer
iki native vakayla sınanır. Dedicated TLS kapısı yedi vakadır; COMMIT
belirsizliği ADR-103/C027 olarak kalır. Çalışma ağacı native7/strict9 geçti;
temiz exact ortak kabul ayrıca gereklidir; provider/full fault matrisi açık.

## K-251 — DB ve göç hazırlık probu

[ADR-112](../adr/112-db-goc-hazirlik-probu.md), mevcut CLI ile uygulama
sahipli tek seferlik DB+migration gate üretir. Native13/strict10 yerel aday
kabulü vardır; compiler özelliği yoktur. DB/worker nesli değişince yeniden
prob gerekir; continuous/TTL, request metrics/drain/config reload ve P14
socket devri açık kalır. Gerçek ürün/platform adoptionı ve exact ortak temiz
kabul ayrıca gereklidir.

## K-253 — Native Unix graceful shutdown

[ADR-113](../adr/113-native-web-sigterm-drain.md), gerçek B5 ürün ihtiyacını
anonim native listener reproducer'ına bağlar. Yeni grammar/CLI/API yoktur;
SIGTERM mevcut admitted handler'ı bitirir, yeni admission durur ve idle
listener kapanır. Supervisor default5s korunur, max60s/explicit55s fırsatı
vardır; dış grace sonunda KILL/cleanup sürer. Native4/guard3 ve eski host25
stage kanıtıdır; exact kaynak/beyan ve final soak1800s kabulü ayrıca gerekir.
Bu semantic-change/dogfood-change'dir; bug fixed_by kaydı icat edilmez.
Metrics, Windows signal drain, tüm backlog/P14 socket/overlap ve adoption açıktır.

## K-255 — P19 mevcut LSP ölçümü

**Yerel ölçüm dilimi kapalı.** Temiz021bff4 release25örnek/2ısınma
LSP2k–20k p95 7.490/18.227/53.784/177.321ms;
[actual raw proof](incelemeler/urunlesme-20261014/k255-lsp-olcum/measurement.json)
ve immutable tarihçe eklendi. Komşu yerel işler varken gözlemseldir, hard eşik
ve yeni baseline yoktur. Gerçek ürün200ms/cache/lint/JetBrains ve P19bütünü açık.

## K-254 — Ayrı SQL kira sahibi heartbeat

[ADR-114](../adr/114-ayri-sql-kira-heartbeat.md): mevcut SQL-only job satırı
handler'da kilitliyken ayrı lease satırı yenilenir. Lease30s/toplam120s/24renew;
immutable0002 geçişi, old-generation fencing ve mixedv1/v2 ret sınırı vardır.
Dedicated21 native+24validator stage kabulündedir; final exactclean ayrı.
Dispatcher/cron/SMTP non-idempotent etkiler ve gerçek katalog adoptionı açık.

## K-256 — TLS1.1 capable-positive kontrol

[ADR-115](../adr/115-tls11-capable-positive-kontrol.md): owned verifiedTLS1.1
positive connection+server gözlemi sonrasında native ret, dedicated8/strict14.
Yeni compiler özelliği/tanı yoktur; legacy cipher policy test contextiyle
sınırlıdır. Provider/full matrix/Windows/uzakCI ve final exactclean kabul ayrı.


## 15 Ekim 2026 temiz teknik kontrol noktası

[K-250–K-256 kontrol noktası](urunlesme-kontrol-noktasi-20261015.md)
exact temiz d42f147 üzerinde faz904/913 (0 başarısız/9 dedicated atlama),
ayrı PostgreSQL41, DB hazırlık13, native drain4, credential14, worker25 ve
iki Python sürümünde TR–EN7+4 kabulünü kaydeder. K-252 exact regression ve
beyan zinciriyle kapalıdır. K-253 mevcut admitted handler'ı SIGTERM sonrası
bitirir; final1800s soak hâlâ gereklidir. K-254 uzun handler heartbeat ve
K-256 verifiedTLS1.1 capable-positive ret kanıtı artık çalışma ağacı
adayı değil, bu exact kaynakta yerel kabuldür. Metrics/socket/dispatcher,
gerçek ürün, insan/platform ve RC/release kapıları açık kalır. Kaynak
kimliği daha sonraki belge veya özellik commit'lerine taşınmaz.

## K-257–K-260 — Yerel aday kapasitesi, ürün kabulünden ayrı

Native request/PG pool metrikleri (dedicated8), SQL-only finite dispatcher
(native11), inherited TCP listener ve nesil hazırlığı (native17/quietFD16),
credential→tek metric FD köprüsü (7+2) aynı ürünleşme dalgasının dar kapasiteleridir.
ADR-116–119 sınırlı kabul durumundadır. Listener17 içinde gerçek argv oracle,
bozuk `zee.ayar`, metric seçenek sırası ve kapanmış metric reader kompozisyonu
vardır; FD16 erken stdio/control/metric/config aliases'i reddeder. GB-078–081
sınırları kaynak+test+kararla birlikte değerlendirilir.

Bu dalga yerel çalışma ağacı/adayı kabulüdür. Listener17/FD16, metrik8 dedicated,
credential7 model/subprocess+2 native ve dispatcher11 sayıları ayrı kampanyalara
aittir; Cargo/libtest toplamına eklenmez. Listener birleşim kanıtı pinlenmiş
`2caa26e55e50bf35b4220e4f6d16ac7273f337159243b6e97c8f2443581adbf8` ikilisinin
actual BuildJSON'unda `17b29da1b8321fe6dcc4d34bd2027c4c969bf406` tabanı ve
`dirty=true` taşır. Bu kimlik temiz kaynak kabulü değildir. Final exact temiz
ortak faz/native, 900 sn host tatbikatı, 1800 sn soak, dört hedef explicit ASAN
RC/Miri, imza/kurulum ve fiziksel platform kanıtı henüz bu yeni kaynağa alınmadı.
Önceki d42f147 temiz kabulü kendi tarihsel kaynağında kalır.

Native listener host tek kendi loopback TCP LISTEN socket'ini tutar; proxy,
SO_REUSEPORT veya yabancı PID/port benimsemesi değildir. Native ready channel
eski worker HTTP200'ünden ayrı, fresh generation receipt de ortak eski dosyadan
ayrıdır.1000 istek concurrency1/dört owned rollover ve ayrı sentetik loopback
istemcileriyle sınır içi kabul gördü; quota raise/reset/retry yapılmadı. Bu genel
zero5xx/refusal SLA değildir. Kalıcı `.zee/web-durumu-v1.json` kota ve revoke
paylaşımını korur; süreç belleği kota overlap profilinde reddedilir.

P09 gerçek catalog/SMTP effect adoptionı; P13 continuous/TTL/config reload ve
uzun işletim; P14 farklı kod/resource revision dağıtımı/live/platform; P25
file-native secret ve listener credential kompozisyonu açıktır.39 plan satırı,
insan/harici CI/fiziksel/RC kapıları ve frozen syntax takvimi kapanmış gösterilmez.
[Dispatcher](job-dispatcher.md) ve [listener](native-listener.md) rehberleri bu
kapsamın kullanım ve ret sınırlarını verir. K262 ayrı correctness dalgasıdır.

Güncel ek sınır: K263 owned-host cleanup dar düzeltmesi ac077e31 ile commitlendi; yukarıdaki yerel
aday makbuzları birleşik finalclean özellik kabulü değildir.
K262 dar source/argv düzeltmesi 7c56518c ve exact provenance4d8edeb2 ile kapalıdır; ortak özellik kabulü açıktır.

Kanonik güvenlik sırası: atomik K-263 owned-host cleanup düzeltmesi GB-077 ile `ac077e31e761e401eedc5140a8035fd02f2a19ba` kaynak commitinde tamamlandı; K-262 dar CLI bugfix `7c56518cbfc6f5c99ac56bfe7767b5bbdc106179` ve exact provenance `4d8edeb2a9f4453d2f28481ecd550bf8ba5f2032` ile kapandı. Ortak temiz özellik kabulü ayrıca beklenmektedir. Kaynak `54a8d6dc0efb060ae5644b37546da40ad03ee035` ile commitlenen özellik dilimlerinin kayıtları K-257/GB-078, K-258/GB-079, K-259/GB-080 ve K-260/GB-081’dir; ADR-116–119 numaraları korunur. Önceki immutable dış-stage makbuzlarının geçici GB etiketleri tarihsel kanıttır, kanonik yeni kayıt kimliği değildir. Finalclean/900sn host/1800sn soak/RC kabulü bu revizyonda ileri sürülmez.

## K-271–K-274 dar current-source kapasitesi

[Makine kaydı](urunlesme-plan-v1.tsv) P22/P21/P20/P13 satırlarına sırasıyla
K-271/272/273/274 ekler, `kismi` durumunu korur. Native fixture kiti, sabit typed
TR/EN golden, yeni owned modül scaffold'u ve explicit finite TTL/DBwatch/config
HUP uygulanmıştır. Bunlar genel HTTP injection/source coverage, generic i18n,
DB migration/CRUD veya sürekli production service değildir. Önceki K-268–270
exact62282aec kaynak kabulü ve source/archive ayrımı korunur; yeni ortak Cargo,
clean final/900/1800/RC/platform ve P08/gerçek ürün benimsenmesi ayrıca beklenir.

## P11 teknik PDF ilerlemesi — K-275

K-275/ADR-133/GB-091 sabit owned PDF host profili: mevcut Zee JSON örneği, exact ReportLab4.4.9/pypdf6.10.0/Pillow12.3.0, stdlib8 normal ve hostbackend8 explicitignored ayrımı. Sabit kaynak/font/notice, bounded job/output ve nofollow roller korunur. Finalclean S native emitter+backend8 ortak kohortu ayrı zorunludur; tarihsel dirty kanıt temiz S sayılmaz. P11/B3 Zee içinden invocation/resmi signedplugin, ürün/adoption/P08/platform ve proje lisansı açıktır; full900/1800/RC/soak/kurulum kabulü ileri sürülmez. Bu iç fixture/backend kararı yalnız harici erişim bahanesiyle bekletilmez; gerçek B3 invocation/adoption ayrı açık kalır.

### K-276–278: Linux taşıyıcı hazırlığı, native dış etki ve editör şablonu

K-276/[ADR-134](../adr/134-linux-ozel-paket-tasiyicisi.md) özel inert Linux taşıyıcı hazırlığıdır: normal wrapper
12 preflight +9 carrier model/negatifi yürütür. ELF header doğrulaması Linux
ABI/çalıştırma kanıtı değildir. İki temiz klon kanonik teslimi, final kaynak S
üzerinde gerçek .deb build/install/remove/user-prefix activation/container
kabulü ve yayıncı/lisans/imza/apt repository ayrı açıktır. Namespace/image
manifest önkoşulu tamamlanmış Zee container kabulüne dönüştürülmez.

K-277/[ADR-135](../adr/135-sabit-native-dis-etki-tuketicisi.md), mevcut SQL-only dispatcher'dan ayrı fixed owned
loopback native SQL→MIME SMTP→ack profilidir. Effect-start latch bilinen committed
olmadan SMTP başlatılmaz; owner/token/generation/lease korunur. Yalnız kanıtlı
pre-effect SQL reddi, sıfır send invocation ile toplam3/1–2sn backoff yapabilir.
C029/DATA/ackCOMMIT belirsizliği guard+exit3/manual incelemedir; SMTP otomatik
replay/exactly-once sözü yoktur. Bilinen completion sonrası görünür guard unlink
fakat final fsync hatası ayrı exit4'tür; olmayan guard yeni admission'ı engelliyor
denmez. Normal hazırlık8 ve explicit ignored native11 ayrı kapılardır. Kaynak
harness yalnız dört pinli fixture dosyasını kopyalar; ek SQL/.dil/FIFO alınmaz.
Tarihsel dirty aday kanıtı finalclean değildir. Gerçek B1/B2 katalog, sağlayıcı
Gmail/M365, K-275 PDF üretimiyle ürün entegrasyonu ve tam P09/P08 açıktır.

K-278/[ADR-136](../adr/136-jetbrains-yerel-lsp-sablonu.md) mevcut K-270 launcher'ını kullanan POSIX LSP4IJ veri
şablonudur; TextMate mevcut VS Code bundle'ından gelir. Npm bağımlılığı olmayan
elle yazılmış VS Code istemcisi için owned `dillsp` mutlak yolu açıkça seçilir;
IDE macro/tokenizer/kurulum/etkileşim/Windows kabulü henüz açık kalır. Şablon
installer/download/yeni grammar/capability/cache veya gerçek IDE kabulü eklemez.
[PDF/SMTP'den ayrı editör rehberi](../editors/jetbrains/README.md) bu sınırı korur.

### K-279: gerçek önceki küçük sürüm korpusu ve açık göç

[ADR-137](../adr/137-onceki-kucuk-surum-korpusu.md) gerçek v0.7.0 exact
`d08a947713ddc38d2163360c17d07fbffcb1213d` kaynağındaki33 golden girdisini
değişmeyen blob/SHA ile korur.32 directcheck eşliği ve15 saf runtime çıktısı
gerçek eski/yeni CLI'da sınandı; birim örneği K-086 publictype sözleşmesi için
yalnız birimde iki annotation satırıyla göç ister. Ham33 kaynak gizlice
yenilenmez. Normal iki Rust vakası49 modernCLI altkontrol+5 erken bootstrap
reddini yürütür;54 libtest veya gerçek anonim ürün/P08 kanıtı sayılmaz.

Eski CLI config capability JSON sunmadığı için mevcut BuildJSON/selector
reddi korunur; yeni `zee.ayar` için otomatik legacy rollback açılmaz. Native
çalışma altkümesi effect-free'dir; ağ/web/dosya/saat/rastgele/async/hardware
runtime örnekleri çalıştırılmaz. Sabit33 kaynakta provenance olarak duran bu örnekler
compiler bugfix/dogfood veya minOS/platform kabulü değildir.

Linux'ta e76 retained leader cleanup ölçümü komut başına yaklaşık3.001s'dir.
52 sınırlı komutun yaklaşık156s toplam bakım yükü için yalnız test harness
aggregate bütçesi120→240s seçildi; her native çağrı8s, owned cleanup/deadline/
capture, kaynak/output oracles ve production SLA/perf eşikleri değişmedi.
Bağımsız Mac49+5 replay ayrı; gerçek tam Linux ve temiz finalS kabulü ayrıca
gerekir. Tam P04/P05, ürün/kanal/platform/adoption açık kalır. [Rehber](onceki-kucuk-surum-korpusu.md).

K-271–K-279 commit öncesi birleşik kapısı (15 Ekim 2026): tam faz matrisi
**943/959 geçti, 0 başarısız, 16 explicit ignored**; faz süreleri toplamı
429.100 saniyedir. Koşu boyunca 1.477 kaynak/belge/fixture dosyası aynı hash'te
kaldı. Fmt, tüm hedeflerde Clippy, işlev64, spec214, katalog/tazelik,
katman/SCC/public API/uyumluluk/tedarik, gerçek Zee–Rust–Go çıktı eşliği,
sürekli güvenlik, semantic/core-freeze ve kanonik sır taraması geçti.
Üç yeni Rust test sarmalayıcısındaki sabit platform assertion'ı açık Unix
koşullu gövde ve desteklenmeyen platform panic'iyle kanonik biçime taşındı;
Unix vakaları ve üretim compiler davranışı değişmedi. İlk Clippy retleri
ayrı dış kayıtta korunur. Normal fazdaki16 dedicated atlama temiz kaynak
kampanyasında ayrıca yürütülmelidir; bu sonuç final temiz kaynak/native,
900 saniye işletim, 1800 saniye soak, ASan/Miri, paket/kurulum veya gerçek
ürün/platform/IDE/insan kabulü değildir. Yeni compiler/src, grammar, spec
normu, public Rust API veya production bağımlılık değişikliği yoktur.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

[15 Ekim temiz S raporu](urunlesme-final-kabul-20261015.md) bu plandaki yerel teknik kanıtları günceller. P01/P02 teslimleri, P06 dar Linux/macOS kurulumları, P09 finite native dış etki/timer, P11 fixed host backend, P13 uzun owned koşu ve P19 kısmi IDE etkileşimleri gerçek kanıt taşır. Tam ürün ve süreç kapsamları ayrıca açık kalır; tablo dar kabulü genel tamamlanma olarak genişletmez.
