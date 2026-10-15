# Sınırlı SQL niyet dispatcher — K-258

[ADR-117](../adr/117-sinirli-sql-dispatcher.md), zee-cekirdek B1 talebinin
SQL-only host dilimini kabul eder. [K-254 ayrı lease sahibi](../adr/114-ayri-sql-kira-heartbeat.md)
uzun handler sırasında heartbeat'i mümkün kılar; bu dispatcher onun üzerinde
sabit niyet kaydı üretir. Yerel native11 kabulü vardır. Gerçek katalog scheduling,
calendar/daemon/cron, SMTP/PDF dış etkileri ve tam P09 benimsemesi açıktır.
Yeni grammar veya desteklenen Rust v1 facade export eklenmez.

## Girdi ve sabit işlem

[Host](../scripts/job-dispatcher.py) yalnız `reap`, `claim`, `begin`, `apply`,
`renew`, `fail` köprüsünü çağırır. [Anonim ürün dilimi](../dogfood/zee-cekirdek/is-kuyrugu-dispatcher/ana.dil)
sabit SQL niyet handler'ıdır. İş payload'ı veri olarak `zee_job_intents` tablosuna
aktarılır; SQL metni, callback, shell veya argv'ye dönüşmez. Aynı outer eylem
niyeti ve matching lease/generation tamamlamasını bağlar. Non-idempotent SMTP,
PDF veya başka dış side effect bu transaction'ın kapsadığı bir davranış değildir.

`--ticks`1..100 arasında, varsayılan20'dir. Host sonlu çalışma bittikten sonra
çıkar. İş yokken bounded poll ve reaper çağrısı yapması scheduler veya sürekli
service demek değildir. Lease30sn, toplam120sn/24 yenileme tavanı ve üç attempt/
deadletter sınırı [kuyruk sözleşmesinde](../adr/114-ayri-sql-kira-heartbeat.md) kalır.
V1 ve V2 lease workers aynı kuyruğa birlikte bağlanmaz.

## Kimlik, credential ve sahiplik

Compiler private byte snapshot'ı, expected binary SHA256 ve actual BuildJSON
source SHA/dirtyfalse doğrulanır. Proje `ana.dil`, `proje.dil` ve0001–0003 migration
byte hash'leri pinlenir; clean historical binary kullanılması daha yeni helper,
host veya ortak kaynak commit'inin temiz kabulünü kendiliğinden kanıtlamaz.

K245 private credential reader inline kullanılır: yalnız açık eşlenen env adları
çocuğa gider; ambient URL fallback, symlink/FIFO ve multiline credential reddedilir.
Bu pilot exact owned `localhost` PostgreSQL/TLSrequire profilidir. CA PEM bounded
ayrı public trust yapılandırmasıdır; credential mapping değildir. URL/credential
veya özel argüman diagnostic/log içine yazılmaz. Parser hatası fixed rejected
JSON/exit2'dir; literal argv ve usage stderr'e taşınmaz.

K246 [worker supervisor](worker-supervisor.md) exact Popen/session/grup sahibidir.
Leader WNOWAIT ile grup cleanup sonuna kadar reaped edilmez. Wrapper leader ölürken
native descendant stdout'u açık tutsa da admission/heartbeat devam ettirilmez;
TERM/KILL/group cleanup yalnız verilmiş owned handles'ı kapatır. PID/port
keşfiyle yabancı servis benimsenmez. JSON log writer PIPE_BUF sınırında tek
nonblocking kayıt yazar; dolu pipe kayıt düşürür ve drop sayar, SQL'i bloklamaz.

## Belirsizlik ve attempt sınırı

Bilinen SQL rollback sonucu exit0 sabit ret köprüsüyle fail/backoff yazımı yapabilir;
üç bilinen ret deadletter'da durur, dördüncü apply yoktur. COMMIT yanıtı kaybolması
C027, nonzero exit, timeout, malformed çıktı veya sinyal belirsizliği admission'ı
durdurur. Bu durumlarda otomatik replay ya da yeni fail/retry write yapılmaz.
SQL niyetinin ilerideki harici tüketicisi kendi idempotency/ack protokolünü ister.

## Kanonik kabul koşusu

```sh
python3 -I -B scripts/tests/job-dispatcher-native-test.py \
  --framework-root "$PWD" --dil <owned-clean-binary> \
  --postgres-bin-dir <owned-postgresql-bin-dir> --output <fresh-output>
```

[Koşucu](../scripts/tests/job-dispatcher-native-test.py) yalnız yeni owned loopback
PG/TLS cluster'ını başlatır; müşteri DB veya mevcut servisi benimsemez.11 gerçek
vaka: iki worker dört niyet/tek attempt,35sn actual handler bağımsız heartbeat,
TERM pause, üç bilinen ret/deadletter, COMMIT önce/sonra no replay, ambient/FIFO
credential ret, dolu log pipe altında SQL ilerlemesi, kayıp leader/native alt grup
cleanup ve argv canary maskelemesini kapsar. Drop/süre/renewal/attempt sayıları
bağımsız PG gözleminden gelir; test disabled/ignored başarı sayılmaz.

[Cargo kapısı](../compiler/tests/job_dispatcher_testi.rs) argüman maskesini normal,
owned PG kampanyasını explicit `ZEE_JOB_DISPATCHER_PG_BIN` ile dedicated yürütür.
Python11, Cargo/libtest toplamına11 yeni Rust vaka olarak eklenmez.

Final temiz ortak kaynak/native kabulü ve gerçek ürün/OS/systemd adoptionı
henüz bu aday sayılarından üretilmez. [Ürünleşme planı](urunlesme-plani.md),
[makine planı](urunlesme-plan-v1.tsv) ve [sürüm kapıları](v1-surum-kapilari.md)
P09'u kısmi tutar; dış etkiler ve sürdürülebilir operasyon ayrı açık iştir.

Güncel ek sınır: K263 owned-host cleanup dar düzeltmesi ac077e31 ile commitlendi; yukarıdaki yerel
aday makbuzları birleşik finalclean özellik kabulü değildir.
K262 dar source/argv düzeltmesi 7c56518c ve exact provenance4d8edeb2 ile kapalıdır; ortak özellik kabulü açıktır.

Kanonik güvenlik sırası: atomik K-263 owned-host cleanup düzeltmesi GB-077 ile `ac077e31e761e401eedc5140a8035fd02f2a19ba` kaynak commitinde tamamlandı; K-262 dar CLI bugfix `7c56518cbfc6f5c99ac56bfe7767b5bbdc106179` ve exact provenance `4d8edeb2a9f4453d2f28481ecd550bf8ba5f2032` ile kapandı. Ortak temiz özellik kabulü ayrıca beklenmektedir. Henüz commitlenmemiş özellik dilimlerinin kayıtları K-257/GB-078, K-258/GB-079, K-259/GB-080 ve K-260/GB-081’dir; ADR-116–119 numaraları korunur. Önceki immutable dış-stage makbuzlarının geçici GB etiketleri tarihsel kanıttır, kanonik yeni kayıt kimliği değildir. Finalclean/900sn host/1800sn soak/RC kabulü bu revizyonda ileri sürülmez.
