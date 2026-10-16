# K-246: yönetilen native worker pilotu

[Host aracı](../scripts/worker-supervisor.py), mevcut güvenli web profiliyle
aynı proje kökünde iki `dil` sürecini yönetir. Compiler/grammar/Rust API
genişlemesi değildir; [ADR-106](../adr/106-worker-host-yasam-dongusu.md) ve
[production profili](web-production-profili.md) sınırları birlikte geçerlidir.

Python 3.9+ ve POSIX `waitid/WNOWAIT`, `O_NOFOLLOW`, `O_NONBLOCK` gerekir.
Bu primitive'ler yoksa araç çocuk başlatmadan reddeder. Windows yönetimi,
systemd/nginx/caddy şablon üretimi, temiz sunucuda15 dakika üretim kabulü ve
container/platform matrisi bu pilotla tamamlanmaz.

## Başlatma ve kimlik

Aşağıdaki `DERLEYICI_SHA` ve `UYGULAMA_SHA`, operator tarafından doğrulanmış
40 küçük hex kimliğiyle değiştirilir. İkili ve Python çalıştırıcısının
güvenilirliği host operatörünün sorumluluğudur; sürüm JSON'u yayıncı imzası
veya kaynak otoritesinin yerine geçmez.

```sh
python3 -I -B scripts/worker-supervisor.py \
  --dil /opt/zee/bin/dil --project /srv/uygulama \
  --origin https://site.example \
  --worker-port 18091 --worker-port 18092 \
  --expected-source DERLEYICI_SHA --application-revision UYGULAMA_SHA \
  --readiness-file /srv/uygulama/.zee/hazirlik.json \
  --health-path /hazir --health-body hazir
```

Yalnız iki farklı1–65535port, HTTPS origin, literal ASCII rota ve sınırlı
beklenen gövde kabul edilir. Genel URL, redirect, sorgu, serbest child argv
ve shell yoktur. Portlar önce bind/listen ile kontrol edilir; mevcut hizmet
bulunursa hiçbir child başlatılmaz ve o hizmetin PID'si keşfedilmez/adopt
edilmez. Kontrol soketleri child bind'ından önce kapanır; aradaki yarışın
atomik giderilmesi/socket activation P14 işi olarak açıktır.

`dil sürüm --json` en çok4096byte ve5saniye içinde okunur. Schema, beklenen
source SHA ve türler doğrulanır. Development veya dirty ikili yalnız açık
`--pilot` ile kabul edilir; gerçek dirty değeri logda korunur, clean/release
kabulü sayılmaz. Projede `zee.ayar` varsa compiler'ın exact
`zee-config-capability-1`/`zee-ayar-1`/`unknown_security=reject` beyanı da
gerekir. Mevcut Zee güvenlik ayarı ve CLI önceliği uygulanır; eski binary'nin
ayarları sessizce atlaması güvenli rollback sayılmaz.

## Uygulamanın hazırlık kapısı

İki worker'ın process'i yaşamalı, literal loopback sağlık isteği200 dönmeli,
gövde eşleşmeli ve `X-Zee-Build` query edilen ikiliyle birebir aynı olmalıdır.
Koşulsuz200rotası tek başına readiness değildir. Ayrıca uygulama helper'ının
proje içindeki gerçek, private-write olmayan regular dosyaya yazdığı kapı
gerekir:

```json
{"schema":"zee-worker-readiness-1","application_revision":"2222222222222222222222222222222222222222","ready":false}
```

Örnekteki revision synthetic yer tutucudur. Helper gerçek beklenen uygulama
revision'ını yazmalı ve yalnız gerçekten yaptığı kontroller başarılıysa
`ready:true` bildirmelidir. Yazı supervisor başlangıcından veya son HUP
restart neslinden sonra yapılmalı; gelecek tarihli dosya kabul edilmez.
Eski neslin hazır dosyası otomatik devralınmaz. Yerel dosya saatinin
tutarlılığı gerekir; aynı UID'nin timestamp/dosya değiştirmesine karşı OS
sandbox garantisi değildir.

Dosya≤64KiB, tekil JSON alanları, exact üç alan, boolean ready ve revision
eşliği ister. Symlink/FIFO/dizin, bozuk/derin JSON, fazla alan, okunamama veya
dosyanın okuma sırasında değişmesi hazır olmamaktır. Dosya ebeveyni kanonik
proje içinde ve diğer kullanıcıların yazmasına kapalı olmalıdır. Supervisor
bu dosyaya `ready:true` yazmaz; helper sahipliği uygulamadadır.

DB/göç kontrolünü supervisor yapmaz. Native test helper'ı başlangıç
başarısını gerçekten Zee'den dosyaya yazar; bu DB/göç hazır kanıtı değildir.
Gerçek üründe DB/göç bozulduğunda helper kapıyı kapatmalı ve bağımlılığın
gerçek olumlu/olumsuz kabulü ayrıca yapılmalıdır. Kapı kapanınca worker'lar
öldürülmez; aggregate state unready olur. Liveness/sağlık probe'u ile
uygulama kapısının sonucu ayrı alanlarda görünür.

## Log, ölçüm ve sinyaller

stdout yalnız `zee-worker-supervisor-1` JSON satırlarıdır. `build`,
`worker_started`, `worker_exit`, `state`, `metrics`, `rejected` olayları sabit
alanlar taşır. Child stdout/stderr `DEVNULL`'dır; program/tanı satırı, body,
cookie, credential, env değeri veya origin loga taşınmaz. Keyfi program
çıktısına regex ile tüm sırları maskeleme sözü verilmez.

Log pipe'ı nonblocking'dir; satırlar pipe'ın atomik yazım sınırını aşmaz.
Backpressure/disk hatasında satır atılabilir ve `log_drops` artar; bu audit
log teslim garantisi değildir. Metrikler probe girişimi/başarısızlığı,
monoton toplam probe süresi, restart, managed worker ve SIGKILL escalation
sayısıdır. `forced_kills` başarılı SIGKILL çağrısını sayar; öldürülen istek
veya tamamlanmış drain sayısı değildir. Request-ID/süre/durum logları,
request/rate-reject/DB-pool Prometheus metrikleri ve management HTTP
endpoint'i bu pilotta yoktur.

Probe aralığı2–60saniye, probe timeout .1–5, startup1–120, TERM grace .1–30
saniyeyle sınırlıdır. Startup/restart da bu probe aralığını ve kalan mutlak
startup bütçesini kullanır; hazır olmama durumunda kota yakmak için sık
probe yapılmaz. İki worker varsayılan ortak kota penceresini korur. Seri
worker meşru bir istekle meşgulken probe gecikebilir; timeout anında otomatik
kill/restart yapılmaz.

SIGTERM/SIGINT hazır durumunu kapatır, yalnız yeni oluşturulmuş child
process group'larına TERM, gerekirse KILL gönderir ve doğrudan child
liderlerini wait eder. Lider çıkışı `WNOWAIT` ile gözlenir; lider PID'si
grup temizliği bitmeden reap edilmez. Böylece credential launcher çökerken
native child'ın temizliği yapılabilir; port/ad üzerinden yabancı süreç
aranmaz. Kernelde kesilemeyen IO veya aynı UID'nin hostile süreç/dosya
değişikliği için genel sandbox/hard wall-clock garantisi yoktur.

SIGHUP aynı portlarda sıralı child restart'tır. Yeni child sağlık ve taze
uygulama kapısını geçmelidir; hata kapanır. In-process ayar reload'u,
transactional rollback, socket devri, sıfır kesinti ve tüm inflight
isteklerin tamamlanması sözü yoktur. Devam eden istek kesilebilir.
`.zee/web-durumu-v1.json` silinmez; oturum/revoke/kota için mevcut profil
sınırları ve P14 yük/overlap kabulü geçerlidir.

## Credential bridge ve test

İsteğe bağlı `--credential-root /srv/kimlikler --credential-env APP_KEY=anahtar`
aynı grup içinde [credential launcher](credential-baslat.md) çağırır. Exact
dosya/izin/nofollow/boyut ve mevcut Zee yetkinlik kontrolleri korunur.
Mapped ambient env varsa ret vardır; supervisor başka ambient credential
forward etmez. Native kabul synthetic anahtarla gerçek şifreleme/çözme,
stdout'un kapatılması ve launcher lideri çökünce native child temizliğini
kapsar; canlı DB/SMTP/rotasyon veya ürün staging kabulü değildir.

```sh
python3 -I -B scripts/tests/worker-supervisor-test.py --dil compiler/target/debug/dil
```

Kampanya 25 gerçek test, 0 ignored ister. Native ikili başlangıçta private byte
snapshot'a alınır; kimlik/hash başlangıç ve sonda eş olmalıdır. Gerçek iki
worker, kapı kaybı sonrası yeni state olayı, foreign port, child/launcher
çökmesi, sır/uzun çıktı, log backpressure, geç HTTP probe deadline,
TERM/KILL, aynı-port restart ve yerel başlatıcının owned cleanup'ı yürür.
Node varsa gerçek public yerel frontend çalışır; yoksa synthetic frontend
kanıtı vardır ve `native_node_fixture:false` açık kalır. Sonuç dirty/source
değerini aynen bildirir; temiz exact commit kabulü ayrı tekrar koşusudur.

B5'in tam işletim/temiz-host kabulü ve P14 socket/drain/overlap garantileri
açık kalır; bu araç `dil hizmet örnek` komutu eklemez.

## K-301 — Başlamış isteğin kesilmesi için test kanıtı

**Dar test doğruluğu tamamlandı.** `test_real_inflight_request_can_be_interrupted`
artık TCP yazımı ve kısa uyku yerine handler girişini doğrular; tek `recv`
parçası yerine tüm sınırlı yanıtı inceler. Yalnız bu teste özel V3 fixture'ı
`web-oturumu` ile `GET "/csrf"` kullanır. Yardımcı HTTP200 yanıtını, 64 haneli
hex belirteci ve oturum çerezini doğrular; `Origin`, çerez ve `_csrf` form
alanıyla `POST "/yavas"` gönderir. `herkese açık` CSRF'yi kapatmaz; normal
kontrol akışı kullanılır. Handler, `eylem girişi kaydet` üzerinden
`inflight.entered` dosyasına `entered` ve bir LF baytı yazar. Exact işaret
iki saniye içinde görülmeden supervisor kapanışı başlatılmaz. Kapanıştan
sonra yanıt bir saniyelik mutlak süre ve 16 KiB sınırında EOF/reset sonuna
kadar biriktirilir; `bitti` bütün yanıtta aranır. Timeout başarı sayılmaz.

İlk GET fixture'ının dosya yazımı T045 ile reddedildi; worker1 readiness
öncesinde başarısız oldu. `k301-fixture-diagnostic-1.log` ve özgün başarısız
controller korunur. V2 POST/eylem fixture'ının `denetle` kontrolü geçti;
worker2 yalnız inflight vakada başarısız oldu (25 test/1 hata). Assertion
ayrıntısı kayıtta yoktur; işaret yokluğu veya HTTP403 gözlemi iddia edilmez.
POST isteğinde oturum çerezi ve `_csrf` eksikliği kaynak/spec incelemesinin
bulgusudur. Her iki başarısız koşu kendi sonucu ile korunur.

V3'ün üçüncü gerçek worker koşusu geçti: mevcut tek Cargo sahibi 1/0/0
(78,28 saniye), Rust wrapper'ın doğruladığı Python sonucu 25/0/0;
sekiz native ve iki credential vakası dahil. Özgün controller exit0 ve
reap ile kapandı (79,284 saniye); 11 giriş dosyası pini korundu. V3 için
ayrıca 10 kontrollü senaryo/20 eski-yeni yöntem karşılaştırması geçti.
Bu karşılaştırmada HTTP/CSRF yardımcısı mock'tur; gerçek yardımcının kanıtı
ayrı worker3 kabulüdür. Varsayılan fixture ve diğer modül AST eşliği,
değişmeyen 25 Python test metodu doğrulandı; test eklenmedi/taşınmadı ve
yeni Cargo sahibi yoktur. K-300'ün 998/0/25 tam fazı K-301 sonucu değildir.

Üç saniyelik handler karşısında explicit `--stop-grace .2`, başlamış
isteğin kısa host bütçesi sonunda kesilebildiğini sınar. Genel drain,
native DNS, tam fault matrisi veya P34 kapanışı değildir. Üretim
host/worker/compiler davranışı ve varsayılan fixture değişmez.

Etki incelemesi: spec/RFC normları, kullanıcı tanıları, compiler/katman/bağımlılık
grafiği, kalıcı durum ve performans iş yükü değişmez; yeni compiler beyanı,
semantic regression, GB veya ADR numarası açılmaz. Son 11 belge/güvenlik
kapısının geçmesi kayıt commit'inin önkoşuludur.
[Kabul raporu](urunlesme-k298-kabul-20261016.md) kaynak ve sonuçları ayırır.

## K-253 güncel Unix drain sınırı

Native Unix worker SIGTERM sonrasında mevcut admitted isteği tamamlar,
yeni handler almaz ve idle listener bounded poll ile kapanır. Host
`--stop-grace` default5s kalır, kabul aralığının üstü60s'dir; explicit55s
mevcut read10/handler30/write10 bütçelerine fırsat verir. Süre sonunda KILL
ve sahipli cleanup korunur. Kernel/DB IO veya bütün backlog için hard
wall-time/drain garantisi değildir. Windows/non-Unix, P14 socket devri ve
zero downtime açık; [ADR-113](../adr/113-native-web-sigterm-drain.md).

## K-263 sahipli cleanup geçişi

Dar host correctness adayı [ADR-122](../adr/122-supervisor-sahipli-eperm-gecisi.md), gerçek Darwin killpg0 EPERM/WNOWAITNone geçişini tek stop_grace budget içinde aynı heldleader üzerinde yeniden gözler. Deadline ilkprobe öncesinde başlar ve resetlenmez. Kalıcı ambiguouspermission ownership retlidir; Popen.returncode-set veya ECHILD signal öncesi reddedilir. Leader terminal olsa da group_alive True ise descendant grubu temizlenmeden wait/reap yapılmaz. TERM/KILL permissionretry aynı yetkidedir; SIGKILL grace sonunda ve başarılı dispatch sonrası mevcut2s wait korunur. Genel Unix/setuid containment veya foreignPID adoption sözü yoktur.

Ayrı8 Python lifecycle vakası ve Cargo wrapper1 mevcut25 nativeworker kampanyasına eklenir; kampanya adı/sayısı değişmez. Kaynak/finalfullphase kabulü root entegrasyonuna bağlıdır.

## K-274 opt-in süreli DB hazırlığı

`--readiness-ttl-seconds` yoksa eski readiness davranışı kalır. Explicit1–3600s
TTL, nesil tazeliği/source/build/health denetimlerine ek wall-age ve ilk gözlemden
monotonic deadline uygular. Aynı mtime tekrar gözlenmesi veya clockrollback süreyi
yenilemez; eski revision yeni kaydın yerine geçmez. Ready dosyası yetkili host
yazarının girdisidir, imza veya kaynağın güvenilirliğinin yerine geçmez.

[Finite DB watcher](db-hazirlik.md) explicit turlar arasında ready=true tutabilir;
interval+probe bütçesi TTL ile uyumlu seçilmelidir. SIGKILL dosyayı temizleyemez:
TTL dolunca admission kapanır, o anda probe grubu kaldıysa outer owned owner
cleanup yapar. Anında failure/zero-error SLA veya PID adoption yoktur. ActualTTL/
configHUP5 ve dedicatedPG15 tarihsel kısa kanıttır; root currentclean ve900/1800/RC
ayrıca gerekir. [ADR-132](../adr/132-sureli-hazirlik-ve-kontrollu-config-yenileme.md).

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

S common worker25, readiness PG15/TTL5, actual 900 saniye owned HTTP yükü ve 1800 saniye soak kabulü [kaynak raporuna](urunlesme-final-kabul-20261015.md) bağlandı. Linux finite SQL user-timer ayrı kabul edildi; sürekli production systemd HTTP hizmeti, temiz fiziksel sunucu ve Windows/ürün kabulü açık kalır.
