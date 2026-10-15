# K-277 — Sabit native SMTP etkisi ve ayrı SQL ack (P09 dar prototip)

Bu dış aday mevcut SQL-only `job-dispatcher.py` veya tablosunu değiştirmez.
Ayrı fixed consumer/SQL şeması yalnız sentetik, owned loopback fixture içindir.
İş verisi recipient, URL, executable, callback, SQL veya shell seçmez. Native Zee
SQL eylemleri lease/latch/ack'i, native Zee ekli ileti kalıbı SMTP/MIME'i yürütür.
Python SMTP göndermez; fixture sunucusu yalnız kendi loopback portunda alır.

## Güvenlik sırası ve belirsizlik

EX kilitli owned0700 state altında fsync edilmiş `pending.json`, claim'den önce
oluşturulur. Süreç ölürse veya sonuç belirsizse kalır ve aynı profilin yeniden
başlamasını kapatır. Bu dosya hostile sameUID/root'a karşı kimlik doğrulama değildir.
Ayrı SQL claim owner/token/artan generation ve server-clock30s lease verir.
Toplam en fazla3 deneme vardır. `effect_started` ve UNIQUE `effect_latches`
satırı ayrı `eylem` transaction'ında **bilinen COMMIT ack'i ile**, native send
komutu başlatılmadan önce kalıcı olur. Bu state expiry/reaper ile yeniden açılmaz;
eski idempotent SQL-only reaper bu ayrı tablolara bağlanmaz. Native send yolu da
live lease/owner/token/generation/latch'i okur; yetkisiz/eski sahibi SMTP bağlantısı
kurmadan reddeder. Bu kontrol, aynı hak sahibi tarafından kötü niyetli direct
binary bypass veya evrensel fiziksel singleton güvencesi değildir.

C029 transport aşamasını açmaz. Başlatılmış native send komutunun her başarısızlığı,
DATA sonrasındaki kayıp250, timeout, TERM/SIGKILL ve SMTP başarıdan sonra belirsiz
SQL ack **manual pause** olur. Hiçbiri otomatik SMTP replay izni değildir.
Yalnız native SQL preflight açıkça `preflight` döndürüp, matching lease ve latch
**yokluğuyla** ayrı safe-fail transaction'ı kabul ettiğinde SMTP çağrılmadan tekrar
vardır: toplam3 deneme,1/2sn bekleme, üçüncüde dead_letter. Sayaç3 retry eklemek
anlamına gelmez. Aşaması bilinmeyen C029 safe-fail'e gönderilmez.

SMTP başarılıysa matching live lease/token/generation ve started latch ile ayrı
SQL done ack'i gerekir. SMTP ile PostgreSQL arasında ortak transaction yoktur.
SMTP sunucusu DATA'yı kabul etmişken istemci250'yi görmeyebilir; veya SQL COMMIT
olmuşken ack kaybolabilir. **Exactly-once teslim sözü yoktur.** Salt okunur durum
incelemesi dış dünyada teslim olmadığını ispatlayamaz. Bu prototip otomatik reset,
operator assertion ile replay, receipt forge veya yeniden teslim komutu sağlamaz.
Kayıp hak sahibi için manuel araştırma/P09 operasyon tasarımı açık kalır.

## Süreç, kaynak ve sır sınırı

Consumer yalnız canonical dış Supervisor tarafından verilen yeni owned
session/group leader olarak çalışır (kendi PID=pgrp). Native çocuklar bu outer
grubu devralır; yeni ayrı session'a kaçmaz. Host yalnız kendi direct child'ını
bounded TERM/KILL/wait ile kapatır; parent Supervisor WNOWAIT leader'ı grup cleanup
bitene kadar tutar. Host SIGKILL olursa cleanup, bu retained outer owner'ın işidir.
Bu script shell'in ortak grubunu veya yabancı PID'yi edinmez ve service discovery
uygulamaz. Tek başına cron/systemd/live-service kabulü değildir.

Native komutlar tek mutlak5sn SQL/25sn SMTP ve256bayt capture sınırındadır;
SMTP adaptörünün mevcut20sn bütçesi ve C029 maskelemesi korunur. Kampanya90sn,
iptal flag-only'dir. Credential reader mevcut `credential-baslat.py`/capability
sınırını kullanır: yalnız designated `ZEE_EFFECT_DB_URL` ve
`ZEE_EFFECT_SMTP_URL`, tek satırlı owned nofollow dosyalar; ambient fallback yoktur.
Çocuk FD'leri default close_fds ile kapalıdır. Sabit fixture şifresi sentetiktir;
public/trusted profile kaynağında URL/parola değil credential dosyası bulunur.

Kaynak dosyası başına128KiB; binary128MiB observed streaming/nofollow regular
hash; source/manifest/SQL/PDF exact pin; schema/sabit iki loopback port dışındaki
handler/proje değişikliği reddedilir. Kaynak private kopyaya alınır; orijinal
fixture değiştirilmez. Makbuz yalnız sabit type/stage, kaynak/Build hashleri,
sayısal command counter ve direct-child cleanup gerçeğini taşır; ham C029,
SQL/SMTP body, URL, DSN, PHC, AUTH veya credential değeri yazılmaz.
`native_send_commands` native send entrypoint child sayısıdır; SMTP bağlantısı ve
DATA kabulü fixture'ın bağımsız sayaçlarıdır, birbirinin yerine konmaz.

## Kısa kanıt ve açık iş

Hazırlık8: gerçek FIFO/oversize/fixed-source/origin/ambient/credential retleri,
pre-child audit; masked unknown CLI; pending guard; actual bounded child output ve
timeout; FD/env aktarılmaması; cleanup arızasında retained tüm owned handle'ların
kapanması. Bunlar native uygulama kabulü diye gösterilmez.

Ayrı actual11: native MIME/PDF bytes +durable SQL ack; DATA kabulü/250 kaybı;
pre-DATA native ret yine no retry; üç known SQL preflight/deadletter ve zero send;
committed latch sonrasında TERM/SIGKILL ve outer-group cleanup; stale owner
zero-SMTP/fresh generation; prepare/ack COMMIT before+after dört fault.
Bağımsız PG observer farklı SELECT-only/default read-only roldedir. Fault-only
plaintext PG router sabit application_name ile mevcut CommitProxy'a yönlendirir;
COMMIT tel sonucu ve kalıcı state ayrı gözlenir. Bu TLS/production proxy değildir.
PG16.15 ve SMTP yalnız newly owned127.0.0.1 fixture'larıdır; retained foreign
sentinel hiçbir consumer grubuna dahil edilmez.

Kullanılan immutable d53 binary, source4d8edeb2/dirty=true tarihsel adayıdır.
Finalclean/Cargo/host900/soak1800/RC, canlı Gmail/M365/SMTP hesapları, gerçek katalog
rotası veya kullanıcı kabulü/P08/calendar/platform kanıtı değildir. Sentetik
`catalog.pdf` yalnız mevcut MIME byte taşımasını sınar; PDF rendering K275 ve
ürün dosya üretimi ayrı iştir. B1/B2'nin kayıtlı anonim provenance'ı korunur;
bu teknik prototip gerçek ürünün Python hizmetlerini kaldırmış sayılmaz.

Tarihsel ad-hoc fault probe exec99594'te proxy.close hatası sonraki teardown'ı
atladı; orijinal Popen yetkisi kaybolduğu için cleanup **UNVERIFIED** kaydı
korunur. Root'un daha sonraki exact-executable current-absence gözlemi varsa
ayrı kanıttır; geçmişte cleanup yapıldığı biçiminde sunulmaz. Yeni fixture
ExitStack ile cleanup'ı spawn öncesi kaydeder, cleanup arızasında kalan sahipleri
kapatır ve başarısız receipt'i finally ile saklar. Son11 kampanyanın closure'ı
kendi retained handle'larıyla doğrulandı; tarihsel probe'a mal edilmez.

[ADR-135](../adr/135-sabit-native-dis-etki-tuketicisi.md) ve GB-093 entegrasyon
öncesi taslaktır; hiçbir ortak kaynak/ledger bu dış stage tarafından değiştirilmedi.

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
