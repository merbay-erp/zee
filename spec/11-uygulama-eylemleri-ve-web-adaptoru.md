# 11 — Uygulama eylemleri ve web adaptörü

Bu bölüm K-087/K-134 ile gelen normatif v1 sözleşmesidir. Amaç iş kuralını HTTP'den
ayırmak, aynı kuralı web/CLI/görev/test bağlamında yeniden kullanmak ve güvenli
HTTP yöntemlerinin durum değiştirmediğini derlemede kanıtlamaktır.

## 1. Eylem tanımı

Bir uygulama iş kuralı `eylem` başlığıyla tanımlanır:

```text
eylem notu kaydet
    notu Metin olarak al
    değer döndürmez
    "notlar.txt" dosyasına notu ekle
```

Eylem çağrısı işlem çağrısıyla aynı yüklem-sonlu ritmi kullanır:

```text
"CLI notu" ile notu kaydet
```

Eylem bir uygulama sınırı olduğu için bütün girdilerinin türü ve dönüş
sözleşmesi kaynakta açıkça yazılır. Parametre varsa her biri
`<ad> <Tür> olarak al` biçimindedir; ardından `<Tür> döndürür` ya da
`değer döndürmez` gelir. Çağrı-güdümlü imza çıkarımı eylemde yoktur (T043).

## 2. HTTP'den bağımsızlık

Eylem yanıt gönderemez, yönlendiremez, çerez yazıp silemez, sunucu/rota
tanımlayamaz. Bu etkileri doğrudan ya da çağırdığı başka bir işlem üzerinden
taşırsa T044 üretilir. Rota HTTP adaptörüdür; doğrulanmış girdiyi eyleme verir
ve eylem sonucunu protokole çevirir.

Transaction ile geri alınamayan ekran çıktısı, kullanıcı girdisi, donanım
eyleyicisi ve e-posta gönderimi (`ileti göndermeyi dene`, K-210) de eylem
içinde kullanılamaz (T048). Bunlar CLI, web, görev veya
donanım adaptöründe kalır. Ağdan salt okuma, zaman, rastgelelik, dosya okuma ve
transaction destekli dosya yazma eylem içinde kullanılabilir.

## 3. Kapalı etki çıkarımı

Derleyici her işlem/eylem için en az şu etkileri çağrı grafiği boyunca çıkarır:

- salt okuma;
- uygulama durumu yazma;
- web adaptörü etkisi;
- geri alınamayan dış etki.

Çıkarım geçişlidir: salt görünen bir işlem yazıcı bir işlem/eylem çağırıyorsa
yazıcı sayılır. Özyinelemeli çağrı grafiği sabit noktaya kadar çözülür; kaynak
sırası sonucu değiştirmez.

Rota dosya ya da donanım durumunu doğrudan değiştiremez. Durum değiştiren normal
bir `işlem`i çağırarak bu sınırı dolanamaz; değişiklik açık bir `eylem` üzerinden
yapılır (T046). Çerez, HTTP adaptörünün kendi durumudur ve yalnız güvenli olmayan
yöntem rotasında doğrudan yönetilebilir.

## 4. Yöntemli rota

Yeni rota başlığı yöntemi açıkça taşır:

```text
GET "/notlar" adresine istek geldiğinde
    sayfa yanıtını gönder

POST "/notlar" adresine istek geldiğinde
    not isteğin "not" değeri olsun
    not ile notu kaydet
    "/notlar" adresine yönlendir
```

Rota, varsayılan 200 yerine sabit bir HTTP durumu gönderebilir:

```text
GET "/hazir" adresine istek geldiğinde
    "hazır değil" yanıtını 503 durumuyla gönder
```

Durum yanıtının içerik türü 200 yanıtıyla aynı kuralla seçilir: gövde `<` ile
başlıyorsa `text/html`, aksi hâlde `text/plain` (K-210 eki, 9 Eylül 2026; panel
hata sayfaları için).

Durum yalnız kaynakta görünen `100..599` aralığında bir `TamSayı` literalidir.
Değişken, hesaplanmış ifade ve aralık dışı değer S037 ile reddedilir. Böylece
review ve etki analizi rota kaynak metninden exact durum kümesini görebilir.
V1 bu cümleyle dinamik status/header builder, ham response nesnesi veya eylem
içinden HTTP etkisi açmaz. Durumlu yanıt da yalnız rota gövdesindedir ve ilk
yanıtın istek transaction'ını kapatma kurallarına tabidir.

Desteklenen yöntemler `GET`, `HEAD`, `POST`, `PUT`, `PATCH`, `DELETE`dir.
Yöntemsiz tarihsel rota yazımı geriye uyum için yalnız GET'e bağlanır; yeni kod
yöntemi açıkça yazar. `istek` sözlüğündeki `"yöntem"` alanı gözlem amacıyla
korunur ancak yönlendirme/yetki kararı değildir.

GET ve HEAD salt okumadır. Doğrudan çerez/dosya/donanım yazması veya çağrı
grafiğinde ulaşılabilen yazıcı eylem T045 ile derlemede reddedilir. Aynı
yöntem+yol çifti iki kez tanımlanamaz (T047). HEAD gerçek TCP adaptöründe GET
ile aynı başlık hesabını yapar ama yanıt gövdesi göndermez.

Yol eşleşip yöntem eşleşmezse 405, yol eşleşmezse 404 döner. PUT/PATCH/DELETE
form gövdesi POST ile aynı sınırlı alan çözümlemesini kullanır.

Sorgu ve form alan adları/değerleri strict URL-form kodlamasıyla çözülür. `+`
boşluktur; `%` işaretini tam iki ASCII hexadecimal hane izler. Eksik/kuralsız
yüzde kaçışı veya çözülmüş baytların geçersiz UTF-8 oluşturması 400'dür.
Kayıplı UTF-8 dönüşümü ve kısmi kabul YASAKTIR; rota gövdesi bu red sonrasında
çalıştırılmaz.

## 5. İstek kaynak sınırları ve son tarih

Her istek varsayılan olarak:

- en çok 64 KiB gövde;
- sorgu ve gövde toplamında en çok 100 alan;
- bağlantı kabulünden başlık ve gövdenin tamamına kadar 10 saniye mutlak okuma;
- K-085 işbirlikli iptal modelinde 30 saniye son tarih

taşır. Gövde/alan sınırı aşımı 413, eksik başlık/gövdeyi yavaşça taşıyan socket
okuma süresi aşımı 408, uygulama son tarihi aşımı 504'tür. Gerçek TCP adaptörü
bildirilen `Content-Length` sınırı aşınca gövdeyi uygulamaya vermeden 413 döner.
Okuma süresi her parçada kalan mutlak bütçeye ayarlanır; bayt damlatmak süreyi
yenilemez. Yanıt yazma socket'i de 10 saniye ile sınırlıdır. Bu limitleri
büyüten kaynak sözdizimi v1'de yoktur.

K-140/ADR-037 uyarınca gerçek TCP isteği metne çevrilmeden önce byte tabanlı
parser'dan geçmek ZORUNDADIR. Yalnız CRLF satır sonu, RFC token yöntemi, `/`
ile başlayan origin-form hedef ve `HTTP/1.0`/`HTTP/1.1` kabul edilir. Bare-LF,
bare-CR, obs-fold, NUL, denetim karakteri, UTF-8 dışı başlık/gövde,
absolute/authority/asterisk-form ve fragment 400'dür. `Transfer-Encoding`
YASAKTIR; tek `Content-Length` yalnız ASCII rakam taşır ve gövde exact aynı
uzunlukta olmalıdır. Pipelining desteklenmez; aynı okumada gövde sonrasındaki
ek bayt reddedilir ve yanıt `Connection: close` taşır.

Rota seçildikten sonra istek tek yaşam döngüsü taşır. İlk yanıt/yönlendirme,
Set-Cookie değişiklikleri ve sunucu tarafı oturum mutation'ları gövde başarıyla
bitene kadar görünmezdir. Başarıda yanıt socket'e eksiksiz yazıldıktan sonra
birlikte commit edilir. Deadline, çalışma hatası, yanıtsız rota veya socket
yazma hatasında session/cookie/yanıt birlikte geri alınır. TCP'nin yazılmış
kısmı fiziksel olarak geri alınamaz; bağlantı kapatılır ve eksik
`Content-Length` başarı sayılmaz.

## 6. Eylem transaction'ı ve iç içe savepoint

Her eylem çağrısı bir transaction/savepoint başlatır:

- olağan değer veya başarılı `Sonuç` dönerse tamamlanır;
- çalışma hatasında geri alınır ve hata yayılır;
- başarısız `Sonuç` dönerse geri alınır, başarısız değer çağırana verilir;
- iç içe eylem kendi savepoint'ini taşır. İç eylemin başarısızlığı, dış
  eylemin daha önceki yazılarını silmez.

IO adaptörü transaction desteğini açıkça vermiyorsa eylem fail-closed biçimde
C021 ile başlamaz. Hermetik test adaptörü dosya tablosunu, gerçek yerel adaptör
ise dokunulan her dosyanın eylem başındaki içeriğini yedekler. Geri alma da
K-084 kilitli atomik replace çekirdeğini kullanır. Hedef, eylemin en son
bıraktığı içerikten sonra başka bir yazarca değişmişse bu yazarın verisi
ezilmez; geri alma C021 ile görünür biçimde başarısız olur.

Bir web isteğinde eylem transaction'ının başlaması, tamamlanması veya geri
alınması C021 üretirse istek yanıtı/çerez/oturum taslağı geri alınmalı, istemciye
503 verilmelidir. Worker/process bu istek yüzünden SONLANMAMALI ve sonraki
bağımsız isteği kabul etmelidir. Bu recovery eylem gövdesini veya write'ı
otomatik tekrar EDEMEZ. Transaction adaptörü bozuk bağlantıyı yeniden
kullanıma bırakmamalı ve yerel savepoint bookkeeping'ini istekten sızdırmamalıdır.

İstek içindeki atomik dosya yazımı C013 üretirse de aynı istek-yalıtımı
geçerlidir: taslak geri alınır, istemciye ayrıntı sızdırmayan “dosya kaydı
tamamlanamadı; otomatik tekrar yok” gövdeli 503 verilir ve worker sonraki
isteği kabul eder. CLI C013 tanısını doğrudan korur. Web adaptörü başarısız
dosya yazımını otomatik tekrarlayamaz; ürün, dosya dışındaki metadata veya yan
etkileri kendi açık uzlaştırma protokolüyle ele almalıdır.

En dış COMMIT'in sonucu bağlantı kaybı yüzünden doğrulanamıyorsa genel C021
yerine C027 üretilmelidir. Web adaptörü C027'yi “işlem sonucu belirsiz;
otomatik tekrar yok; uzlaştırma gerekli” gövdeli 503'e çevirir ve worker'ı
yaşatır; CLI C027'yi doğrudan yayar. COMMIT'in PostgreSQL'e ulaşmamış veya
ulaşıp uygulanmış olması bu istemci sözleşmesini DEĞİŞTİREMEZ. Yeni bağlantıda
iş anahtarıyla okuma/uzlaştırma yapılmadan aynı write tekrarlanamaz.

Bu sözleşme yorumlayıcı tarafından gözlenen hata/başarısız sonuç için çok
dosyalı geri almadır. Süreç ya da makine tam eylemin ortasında çökerse bütün
dosyaları tek bir kalıcı commit olarak yayınlama sözü vermez; çökme atomikliği
dosya başına spec/08'deki K-084 sözüdür. K-163 PostgreSQL adaptörü aynı eylem
sınırını `BEGIN`/savepoint ile uygular; tek eylemde dosya ve DB yazısı dağıtık
atomiklik vaat etmemek için yasaktır. Ayrıntı spec/25'tedir.

## 7. Güvenlik sınırı

K-087 rota/eylem ayrımını, yöntem güvenliğini, kaynak limitini ve yerel
transaction sözleşmesini kurar. Kimlik doğrulama, yetkilendirme, CSRF,
güvenli oturum/çerez ve TLS/proxy güveni K-088 ile
[spec/12](12-web-guvenlik-profili.md)'de tanımlıdır. Idempotency anahtarı
ayrı açık kapıdır.

K-163/F030 native binary isteği ve uygulama kontrollü dosya saga'sı spec/26'da
tanımlıdır. `application/octet-stream` gövde rota çalışmadan önce temp dosyaya
akar; rota yalnız göreli yol/hash/boyut metadata'sını görür. Publish ve silme
ayrı dosya eylemleridir; PostgreSQL durum geçişiyle aynı eylemde birleşemez.
Bu nedenle gerçek TCP yüzeyi `--deneysel-web` açık seçimini korur; bu bölüm tek
başına “production web framework” sözü değildir.

K-163/F032 gerçek ürün readiness ihtiyacı, yukarıdaki sabit durum yüzeyini
açmıştır. legacy panel'da DB-bağımsız liveness 200 kalırken PostgreSQL readiness
başarısızlığı 503, bulunamayan/taslak içerik 404 olur. Bu ürün kanıtı dynamic
response API gerekçesi değildir; yeni durum kodları aynı literal sözleşmeyle
yazılır.

## Native Unix SIGTERM yaşam döngüsü — K-253

TANIMLI: Native Unix CLI web adaptörü SIGTERM alındığında yeni handler kabulünü durdurur; önceden kabul edilmiş tek isteğin okuma, handler, transaction ve yanıtı mevcut bütçelerle tamamlanır. Idle listener en çok 100 ms poll aralığında kapanış isteğini görür; incoming bağlantı poll beklemesini hemen uyandırır. Bu davranış dış stop-grace bütçesiyle sınırlıdır; KILL, kesilemeyen IO, bütün backlog drain, Windows signal drain ve socket devri garantisi yoktur. Önceki süreç SIGTERM disposition sahibi guard bırakıldığında geri yüklenir.

## Native metadata profili — K-257

TANIMLI: --web-metrik-fd opt-in native proxy profili zee-native-metrics-1 şemasında request22 ve pool8 doyumlu sayısal metadata dizileri üretir. Accepted/completed ve kabulden akış kapanışına duration toplamı, nihai attempted server status, final write sonucu ve gerçek rate kararı ayrı sayılır; client receipt veya audit garantisi yoktur. Varsayılan kapalı yolda süre ölçümü ve metrik IO yapılmaz.

ZORUNLU: Metrik FD kaynak/config tanısından önce stdio dev/inode alias, FIFO, O_WRONLY, O_NONBLOCK, current uid/root, SIGPIPE_IGN ve PIPE_BUF açısından doğrulanır; ownership constructor tekrar denetler. Başarıda tek File sahibi FD_CLOEXEC koyar; yanlış descriptor üzerinden özel kaynak, argüman veya gövde yazılmaz.

ZORUNLU: Her metrik frame en çok512byte ve actual PIPE_BUF içindedir; tek nonblocking write retry yapmaz. EAGAIN/EINTR drop, partial/zero/EPIPE ve diğer hata kanal disable üretir; hizmet isteğini durdurmaz. Host iki worker ve kind başına sınırlı snapshot taşır, bozuk stream payload olarak yayımlanmaz.

YASAK: PostgreSQL HandleEvent metrik callback içinde IO veya pool.state() çağrısı yapılmaz. Actual connections/idle/busy ve waiter snapshot checkout ve query/commit/rollback dönüş sınırlarında callback dışında alınır; sürekli taze gauge değildir. Checkin süre toplamı henüz dönmeyen lease süresini kapsamaz.

TANIMLI: Credential launcher --native-metrics-fd ile yalnız tek designated write-only metrik FIFO capability aktarır. Native --web-metrik-fd kaynak öncesi aynı descriptor rolünü belirtir; launcher credential/source okumadan doğrular ve spawn sonrası kendi kopyasını kapatır. Ambient FD ve varsayılan noinherit korunur; bu profil Windows/systemd veya tam P13/P25 ürün kabulü değildir.

## Sahipli Unix listener ve nesil hazırlığı — K-259

TANIMLI: Unix native --web-listener-fd, --web-ready-fd, --web-activate-fd ve --web-generation birlikte sahipli inherited listener profilidir. Listener exact loopback IPv4 TCP LISTEN port ve nonblocking rol taşır; native yalnız kendi duplicate sahibidir, fallback bind yapmaz. Kaynak/config tanısından önce quiet preflight stdio ve roller arası aliası reddeder; Darwin pipe peer karşılaştırması yalnız kendi PID descriptor metadata sınırındadır.

ZORUNLU: Ready kaydı zee-native-listener-ready-1 şemasında exact native PID, fresh generation, build kimliği ve port taşır. Host ayrı generation-owned application receipt doğrular; exact ACTIVATE generation ve EOF gelmeden accept başlamaz. Activation okuması nonblocking ve süre sınırlıdır; EOF veya hata admission açmaz. Host leader rezervasyonunu süreç grubu cleanup bitene dek tutar.

TANIMLI: Inherited listener profilinde successful accept admission noktasıdır; accept sonrasındaki SIGTERM yarışı kabul edilen isteğin mevcut bütçelerle tamamlanmasını engellemez. Varsayılan direct bind profili K-253 kapanış sınırını korur. Ortak canonical atomic JSON quota/session/revoke durumu overlap ve restart boyunca kullanılır; süreç belleği kota deposu bu profilde reddedilir.

TANIMLI: Inherited listener ile optional native metrik capability aynı kaynak öncesi çağrıda kompoze edilir. Metrik readerının önceden kapanması hizmeti durdurmaz; listener/ready/activate kontrol rolleri canlı peer ve exact rol ister. Mevcut credential launcher listener kontrol descriptorlarını taşımaz. Pilot aynı frozen uygulama revisionıyla sınırlıdır; genel deployment atomicity ve ürün adoptionı AÇIK kalır.
