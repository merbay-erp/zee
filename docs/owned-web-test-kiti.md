# Owned native web test kiti — K-271/P22

Bu dar Unix host aracı mevcut Zee CLI ile gerçek HTTP çalıştırır. Rust `ToplayanIo`
veya `api::v1` HTTP injection sözü değildir. Kaynak/branch coverage ölçmez; vaka ve
istek sayıları bu tür coverage yüzdesine çevrilmez. P22 bütünü ve P08 gerçek anonim
ürün korpusu/canlı kullanıcı/platform kabulü açıktır.

Operator güvenilir compiler ikilisini, exact SHA-256/Build source kimliğini,
güvenilir fixture kökünü ve PostgreSQL **16.15** ikili dizinini açıkça seçer:

```sh
python3 -I -B scripts/owned-web-tests.py \
  --fixture-root compiler/tests/fixtures/owned-web-kit \
  --suite compiler/tests/fixtures/owned-web-kit-suite.json \
  --suite-sha256 <suite-64-hex-sha256> \
  --dil /abs/trusted/dil --binary-sha256 <binary-64-hex-sha256> \
  --expected-source <exact-40-hex-source-sha> \
  --postgres-bin-dir /abs/trusted/pg16.15/bin \
  --output /abs/owned/new-output
```

Yeni çıktı dizininin mevcut canonical ebeveyni geçerli kullanıcıya ait olmalıdır;
root, mevcut çıktı, symlink ve fixture içine çıktı reddedilir. Varsayılan compiler
Build dirty=false ister. Yalnız kısa geliştirme adayı için `--candidate`, gerçek
dirty değerini saklayarak açıkça seçilebilir; final temiz kaynak kabulü değildir.
PG binary/version ve compiler/helper/source/fixture hashleri başta ve sonda
bağlanır. Eski dirty aday kanıtı yeni kaynak SHA'sına mal edilemez.

Suite dosyası açılmadan önce nofollow normal-dosya/64KiB sınırı doğrulanır;
aynı sınırlı snapshot baytları hem SHA-256 hem JSON decode için kullanılır.
Dosya identity/size/mtime başlangıç, açık FD ve bitiş arasında eşlenir. Compiler
ve PG executable hashleri en çok64MiB, runtime yardımcıları en çok1MiB olacak
şekilde streaming hesaplanır. Suite FIFO/symlink/dizin/oversize girişi hash okumadan reddedilir;
executable/runtime FIFO ve rol sınırını aşan dosyalar da açılmaz. Cancel flag ve180sn mutlak süre preflight okumalarından önce kurulur.

## Scenario biçimi

[`owned-web-kit-suite.json`](../compiler/tests/fixtures/owned-web-kit-suite.json)
çalışır tam örnektir. `schema=zee-owned-web-tests-1`, relative path→SHA-256
`inventory` ve `cases` zorunludur. Tekrarlı JSON anahtarı, unknown alan,
absolute/escape/symlink/unpinned dosya ve hash kayması yürütmeden önce reddedilir.
Inventory en çok128 dosya/2MiB; JSON64KiB;8 vaka; vaka başına64 adım,
proje taraması256 öğe/16 yol bileşeni sınırındadır. Fixture kaynakları güvenilirdir;
araç güvenilmeyen Zee/SQL kodu için sandbox değildir.

Her vaka `id`, `project`, `steps`; isteğe bağlı `database` ve
`seed=panel_synthetic_phc` taşır. Manifest executable/argv/shell/DSN/inline SQL
seçmez. Her proje yeni özel geçici kopyadır; asıl `.dil` kaynakları değiştirilmez.
DB fixture bildirimindeki tek bilinen loopback endpoint owned PG portuna bağlanır,
oluşan bildirim hash'i makbuza ayrıca eklenir. P23 site/panel kaynakları mevcut
başlangıçların kopyasıdır. Sentetik panel parolası rastgele üretilir, mevcut
`parola-özeti --stdin` ile PHC elde edilir; yalnız owned kopyada0600 dosyasıdır.
Normal dosya okuması credential launcher's nofollow/secret-redaction protokolü
sayılmaz; gerçek parola/PHC/DSN/özel ürün verisi bu kitin kapsamı değildir.

HTTP adımı `method=GET|POST`, salt path, named private `jar` ve
`expect.status` taşır. Form değerleri sabit metin veya önceden gerçek yanıttan
alınmış `{"csrf":"login"}`/sentetik `{"secret":"panel_password"}` referansıdır.
`capture_csrf` HTML'deki `_csrf` inputlarından tek benzersiz64hex değeri alır;
aynı belirtecin çok formda görünmesi kabul edilir, farklı değerler reddedilir.
CSRF üretilmez veya framework'e enjekte edilmez. Ayrı jar ile yanlış belirteç ve
hiç belirteç vermeme gerçek403 testleridir. Redirect otomatik izlenmez. Cookie
ve CSRF bellekte özeldir, makbuza veya loga yazılmaz.

`expect.rotated` gerçek Set-Cookie yeniliğini; `expect.snapshot` pinlenmiş
public gövde dosyasına byte eşliğini sınar. Snapshot otomatik güncellenmez;
tarih/cookie/CSRF için geniş regex temizleme yoktur. İlk dar biçim dinamik/gizli
HTML snapshot'ı vaat etmez. HTTP header16KiB, gövde64KiB, form8KiB sınırındadır;
aynı3sn mutlak monotonic süre connect/send/receive/EOF/final parse boyunca geçer.
Content-Length/Connection-close/framing ve exact `X-Zee-Build` doğrulanır.

## PostgreSQL izolasyonu ve rollback sınırı

Her DB vakası yeni initdb dizini ve foreground PG leader/process group alır.
Küme kimliği gerçek `pg_control_system()` sonucudur; farklı vakalarda farklı
olmalıdır. Mevcut sunucu/DSN/pg_ctl PID dosyası edinilmez. Yalnız127.0.0.1 ve
özel Unix socket kullanılır. `sslmode=disable` var olan explicit loopback geliştirme
profilidir; P31 TLS/production olumlu kontrol kanıtı değildir. Yerel test trust
authentication kötü niyetli aynı-host aktörlerine karşı servis izolasyonu sözü
vermez; fixture ve operator güven sınırı açıktır.

`database.bootstrap` ve `database.observers` yalnız inventory'deki güvenilir
fixture dosyalarıdır. Psql backslash meta reddedilir; fixture owner/app/observer
superuser/CREATEDB/CREATEROLE değildir. App DML, observer SELECT-only ve default
transaction_read_only=on kullanır. `observe` yalnız fixed named SELECT seçer;
statement/lock/connect timeoutları vardır. Gözlemci ayrı rol ve bağlantıdan okur.

Bir HTTP isteğinin COMMIT'i sonraki istekte kalıcıdır. Örnek failed SQL eylemi
Sonuç/Hata döndürüp kendi transaction/savepoint yazısını geri alır; rota açıkça503
üretir. Bağımsız gözlemci eski committed=1, failedwrite=0 görür. Sonraki yeni
kümenin baseline0 olması **vaka başına fresh database izolasyonudur**; committed
HTTP isteklerini kapsayan dış rollback değildir. Dosya ve cookie izolasyonu da
SQL rollback olarak gösterilmez.

## Sahiplik ve kanıt

Mevcut Supervisor yalnız kendi Popen leader'larını WNOWAIT ile grup cleanup bitene
kadar tutar; foreign PID adoption/kill yoktur. Worker-port bind seçimi yarışında
koşu başarısız olabilir; generic app receipt üretilmez ve native Host app-ready
sözleşmesi gevşetilmez. Başarılı ready HTTP cevabı production application/DB SLA
olarak sunulmaz. Global180sn watchdog, bounded command capture ve cancel flag
vardır. Signal handler async exception atmaz; kayıtlı owned çocuklar finally ile
TERM/KILL/reap edilir. Cleanup hatası passed makbuz bırakamaz.

`proof.json` status/case/step/fixed failure type+reason, kaynak/ikili/hash kimliği,
owned cleanup ve DB sayısal gözlemlerini taşır. Raw body, stderr, parola/PHC,
Cookie/CSRF ve DSN yazılmaz. Yanlış snapshot, yetkisiz bootstrap ve gerçek SIGTERM
kısa negatifleri ayrı kanıttır.20 hazırlık testinin TCP/process yardımcıları native
Zee kabulü veya source coverage sayılmaz. İki Cargo vaka önerilmiştir: normal
hazırlık20 ve explicit `ZEE_OWNED_WEB_KIT_PG_BIN` isteyen ignored gerçek4 kampanya.
Windows'ta native vaka manuel zorlanırsa dürüstçe reddeder; fiziksel Windows/minOS
kabulü ileri sürülmez. [ADR-129](../adr/129-owned-native-web-test-kiti.md)/GB-087
aynı atomik entegrasyonda bağlanmalıdır.
