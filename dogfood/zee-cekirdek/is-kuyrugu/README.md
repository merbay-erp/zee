# K-247/K-254 — Kalıcı SQL iş kuyruğu ve ayrı kira sahibi

Bu kesit `zee-cekirdek` ürününün B1 talebinden gelir: katalog onayından sonra
kalıcı iş, sınırlı eşzamanlılık, yeniden deneme, ölü mektup ve süreç çökmesinden
kurtarma. Mevcut ZeeSQL ve `eylem` ile uygulama protokolünü sınar. Yeni grammar,
CLI komutu veya compiler özelliği eklemez. Gerçek katalog/e-posta işçisi ürün
tarafında uygulanmamıştır; Python servislerinin kaldırılması kabulü açıktır.

Harici kaynak exact commit'i `cd5d12f0008620819e6167af5146e83503fd2b38`:

| Kaynak | SHA-256 |
| --- | --- |
| `docs/dil-talepleri/2026-10-14-zee-urunlesme.md` | `694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7` |
| `moduller/pdf-katalog/kaynak/rota_sm_katalog.dil` | `b8721f8044ee976c3f1770bfe43d4788e0e962ecd6196be7845367c54d29fb59` |
| `moduller/pdf-katalog/kaynak/sm_katalog.dil` | `c7d29f4dc1d89422bac9bb8ec719281c47578609ad867f37f63ac5df45496d69` |
| `moduller/pdf-katalog/gocler/2900_katalog.sql` | `f69790799982fd0608ae0818b8e2d2f0d40b5a1dc570771fccdeb6cf536f368b` |
| `moduller/pdf-katalog/modul.py` | `f100cc31d6b714cac088cdd829ee281e19ec9e456f01303578c880e4933c17c2` |

Bu dizin anonim uygulama yardımcısıdır. Özel ürün kaynağı, alıcı adresi,
kimlik bilgisi veya gerçek bağlantı hedefi içermez. Talepteki mevcut ürün
kullanımı ile bu teknik prototipin kabulü ayrı tutulur. K-164 sürtünme kataloğu
karşılaştırıldı: F003/F014 için ara değişken kullanımı mevcut çözümle uyumludur;
tek satırlı SQL sonucunda `ilki` kullanıldığı için F010 sıra erişimi talebi
burada yeni bir sürtünme üretmez. Yeni dil yüzeyi istenmez.

## Uygulama protokolü

[İlk göç](gocler/0001_kuyruk.sql) iş durumunu PostgreSQL'de tutar;
[0002](gocler/0002_kira_sahibi.sql) lease yetkisini ayrı satıra taşır.
0001 immutable kalır; 0002 eski süresi dolmuş kirayı uzatmaz. Karma v1/v2
worker kullanımına izin verilmez.
[Yardımcı](kaynak/kuyruk.dil) şu sabit işlemleri sunar:

1. `isKuyrugaEkle`: iş anahtarı ve onay revizyonu UNIQUE'dir. Aynı içerikle
   yinelenen ekleme aynı kimliği döndürür; aynı anahtar/revizyonda farklı
   içerik reddedilir. Parametreler bağlı TEXT değerleridir; iş içeriği SQL,
   program, callback veya komut adı olarak çalıştırılmaz.
2. `isKirala`: sahip ve tek kullanımlık jeton alır; PostgreSQL saatiyle
   30 saniyelik kira ve artan nesil verir. Ortak kapasite satırı kilidi,
   `FOR UPDATE SKIP LOCKED` ve iş satırı geçişi aynı `eylem` içindedir.
   Örnek göçte en çok iki süresi dolmamış kira ve sahip başına bir kira vardır.
   Kira sonucunun okuması yalnız süresi dolmamış `reserved` satırı seçer;
   reaper öncesi eski kayıtla jeton yanlışlıkla tekrar kullanılsa bile eski
   iş kimliği döndürülmez. Çağıran yine her kirada yeni jeton üretir.
3. `isDenemesiniBaslat`: doğru kimlik/jeton/nesil ile denemeyi önceden ayırır.
   Sınır toplam üç denemedir; ilk denemeye ek üç retry değildir. Çökme, handler
   başlamadan önce ayrılmış denemeyi de tüketebilir; sayaç gerçek teslim sayısı
   sayılmaz.
4. SQL handler'ın dış `eylem`i `isHakkiniKilitle`, uygulamaya özel idempotent
   SQL etkisi ve `isiTamamla` çağrılarını birlikte tutar. Hak, jeton, nesil,
   süre ve durumla koşullu güncellenir. Etki ve `done` aynı transaction'da
   commit olur; hata sonucu tüm dış transaction'ı geri alır. Yardımcı tek
   başına `isiTamamla` çağrısının uygulama etkisi ürettiğini iddia etmez.
5. Açık SQL deneme reddinden sonra `isDenemesiniReddet` ilk iki reddi
   `retry_wait` durumuna alır: beklemeler 1 ve 2 saniyedir. Üçüncü ret
   `dead_letter` olur; dördüncü deneme verilmez. Süresi dolmuş kirayı
   `suresiDolanIsleriBirak` açar; üç deneme tüketilmişse ölü mektuba taşır.
6. `isDurumunuOku` anahtar/revizyon üzerinden salt okunur uzlaştırmadır.
   İş yazısı veya handler tekrar çalıştırmaz.

Uygulama handler'ı açıkça idempotent ve aynı PostgreSQL transaction'ına bağlı
olmalıdır. Göç ve yardımcı fiziksel süreç singleton'ı veya süresi dolan eski
süreçle yeni sürecin aynı anda dış IO yapmamasını garanti etmez. Eski neslin
SQL yazısını koşullu güncelleme ve transaction kilidi sınırlar. Süre aşımı,
heartbeat, süreç supervisor'ı, yetkili panel ve gerçek ürün handler'ı ayrıca
tasarlanmalıdır; kiralama sınırı bu kabiliyetlerin yerine geçmez.

## C027 ve SMTP sınırı

COMMIT sonucu belirsizse mevcut compiler C027 verir, başarı devamı ve otomatik
yazı tekrarı yoktur. Uygulama yeni bağlantıda anahtar ve durum okur. `done`
gözlemi SQL etkisiyle aynı commit'i gösterir; `leased` okuması tekrar izni
değildir. Devam eden transaction veya kayıp yanıt olasılığı değerlendirilir;
SQL kira kurtarma ancak süre/nesil/row-lock protokolü üzerinden yapılır.
Bir okumada işin bulunmaması evrensel rollback kanıtı değildir. Yeni ekleme
kararı açık bir uygulama kararıdır; aynı UNIQUE anahtar korunur. Native
fixture önce kendi backend'inin sona erdiğini gözler, sonra bu iki sonucu
bağımsız bağlantıyla karşılaştırır.

SMTP/PDF/dosya/HTTP bu SQL handler profilinin dışında kalır. İleride queued
MIME için hazırlanmış → gönderim başladı → kesin teslim veya belirsiz duraklama
protokolü gerekir. DATA/COMMIT sonrası kayıp yanıt tekrar teslim izni vermez;
C029 transport aşamasını güvenilir biçimde açmaz. Bu prototip SMTP göndermez,
üç SMTP denemesini kabul etmez ve M365/Gmail canlı provider kabulü sağlamaz.

## İzole kanıt

[Native koşucu](../../../scripts/is-kuyrugu-ci.py) yeni sahipli PostgreSQL kurar;
mevcut DB URL'si kabul etmez. TLS normal işlemlerde açıktır. Dört COMMIT fault
vakası aynı sahipli cluster'a loopback plaintext proxy ile gider; bu istisna
genel istemci profili değildir. Docker yayın portu yalnız `127.0.0.1`'dir;
container içi guest bağlantısının HBA `samenet` kapsamı yalnız sahipli test
cluster'ına aittir. Yerel motorun plaintext kapsamı `127.0.0.1/32`'dir.

Koşucu geçici projeye sabit, UNIQUE iş kimliğiyle idempotent SQL sayaç handler'ı
ekler. Sayaç test aracıdır; gerçek ürünün katalog veya SMTP handler'ı değildir.
Sentetik ret ayrı test tablosuyla kontrol edilir; onaylanmış payload değişmez.
Gerçek iş yazıları `değiştirmeyi dene`, gözlemler salt `okumayı dene` kullanır.
Yazı `RETURNING` ile okuma kapısından geçirilmez.

On bir vaka: aynı onay/aynı iş ve içerik değişimi reddi; sırasız iki worker ve
kira sınırı; kira devri/eski nesil/tek etki; üçüncü denemede kurtarma; üç ret
sonrası ölü mektup; reaper öncesi aynı jetonun eski kaydı seçmemesi;
COMMIT öncesi ve sonrası gerçek SIGKILL; handler COMMIT'inin
iletilmemesi veya uygulandıktan sonra yanıtının kaybı; aynı iki fault'un enqueue
anahtarıyla uzlaştırılması. PostgreSQL lock beklemesi gerçek barrier'dır;
uyku ya da canned sonuç kanıt yerine kullanılmaz. COMMIT etiketini ve ReadyIdle
yanıtını proxy; kalıcı etkiyi bağımsız bağlantı gözler.

```sh
python3 -I scripts/tests/is-kuyrugu-ci-test.py
python3 -I scripts/is-kuyrugu-ci.py --output /tmp/is-kuyrugu-ci.json
```

İkinci komut varsayılan immutable PostgreSQL 16.15 Docker image'ını kullanır.
Sahipli `initdb` araçları için `--engine local --postgres-bin-dir <bin-dizini>`
seçilir; gerçek gözlenen sürüm rapora yazılır. Binary BuildJSON exact HEAD ve
`dirty=false` gerektirir. Çalışma ağacı temiz değilse sonuç yalnız aday kanıtıdır;
son kabul temiz exact kaynakta yeniden çalıştırılır. On bir vaka ve bütün
gözlem alanları atlamasız gerekir. Normal Python/Cargo kapısı 16 skor kartı
negatifini çalıştırır; native DB kampanyası ayrı çalıştırılır. Raw rapor kaynak
dizininin dışında korunur; benchmark geçmişi veya canlı ürün kabulü değildir.

Karar ve ortak kayıtlar ADR-107 ile root iş dalgasında birlikte tutulur.
Mevcut normatif SQL/eylem/C027 davranışı değişmez; yeni tanı, public API veya
dependency yoktur. Performans ölçümü gözlemseldir, eşik veya tarihçe güncellemesi
yoktur; ADR-045/049 ve compiler mimari/soak sınırları bu prototiple genişlemez.

## K-254 protokol v2 — heartbeat yetki sahibi

Jobs satırını SQL handler transaction'ı kilitler; heartbeat ayrı
`zee_job_leases` satırını günceller, jobs durumunu yalnız MVCC okur.
Kira30s, server-clock toplam tavan120s ve en çok24 yenilemedir. Doğru
owner/token/generation ve süresi dolmamış kira olmadan yenileme yoktur.
Reaper `FOR UPDATE OF job SKIP LOCKED` kullanır; kilitli handler'ı beklemez.
Eski job expiry alanı v2 yetkisi değildir. Completion statement anındaki
kira kontrolüdür; COMMIT anındaki saat veya physical worker concurrency
sözü vermez. Expired eski kayda tekrar kullanılan token hayat vermez.

Dedicated native kapı11eski+10yeni=21vaka ve24 validator yürütür. Actual35s
kilitli handler sırasında7 heartbeat commit, ilk30s expiry sonrası geçerli
kira ve tek durable effect/done görüldü. Doğal expiry/reaper/SIGKILL/backendgone,
eski nesil retleri ve heartbeat C027 before/after renewal0/1 observer
aynı kapıdadır. [ADR-114](../../../adr/114-ayri-sql-kira-heartbeat.md) ayrıntıyı
ve sınırları açıklar. Bunlar stage sonuçlarıdır; final exact temiz kabul
ayrıdır. Operasyonel dispatcher/cron, non-idempotent dış etki, SMTP teslim
retry ve gerçek katalog Python kaldırma kabulü açık kalır.
