# 12 — Web güvenlik profili

Bu bölüm K-088/K-134/K-137/K-139/K-185/K-188 ile gelen normatif oturum, yetki, CSRF, ortak
oran sınırı ve HTTPS reverse-proxy sözleşmesidir. Uygulama eyleminin HTTP'den
ayrılması spec/11'de tanımlıdır; bu bölüm tarayıcı isteğinin o eyleme hangi
kapılardan geçerek ulaştığını tanımlar.

## 1. Rota önsözü

POST, PUT, PATCH ve DELETE rotalarının ilk cümlesi ZORUNLU olarak tam bir
erişim politikasıdır:

```text
herkese açık
oturum gerekli
"yönetici" yetkisi gerekli
```

`herkese açık`, CSRF denetimini kapatmaz; yalnız kullanıcı oturumu
aranmayacağını açıklar. Tek istisna kimliksiz makine ucudur
(`herkese açık makine uç noktası`, K-405 bölümü). Politika eksikliği T049'dur. Politika ilk ve tek
politika cümlesi değilse T050'dir.

Politikadan hemen sonra sıfır veya daha çok zorunlu alan bildirilebilir:

```text
"parola" alanı gerekli
"not" alanı gerekli
```

Alan yoksa veya yalnızca boşluksa rota gövdesi çalışmadan 400 döner. Alan
bildirimleri politika ile normal rota cümleleri arasında bulunur; başka yerde
T050'dir.

GET ve HEAD için politika yazılabilir. Yazılmazsa rota herkese açıktır;
spec/11'in salt-okuma kanıtı yine geçerlidir.
GET/HEAD rotası açık politikayı `herkese açık önbelleklenebilir 300 saniye`
biçiminde de yazabilir; anlamı ve sınırları K-335 bölümündedir.

## 2. Parola ve oturum açma

Parola doğrulama Argon2id v19 PHC veya sınırlı legacy bcrypt özeti kabul eder:

```text
verilen parola_özeti ile doğrulanıyorsa
    "Mustafa" kullanıcısını "yönetici" rolüyle oturuma al
```

Bozuk, desteklenmeyen algoritmalı veya eşleşmeyen özet `yanlış` sonucudur. PHC
özeti `dil parola-özeti` ile parolayı terminalde göstermeden üretilir;
otomasyon için açık `--stdin` seçeneği vardır.

K-210/ADR-083 ile PHC özeti dil içinde de üretilir: `parolanın argon2 özeti`
Argon2id v19 (kitaplık varsayılan parametreleri, CSPRNG tuz) döndürür;
`kriptografi` yetkinliği ister, web isteğinde giriş oran sınırına tabidir
(aşım yanıtı §5, K-365), playground'da C028'dir. İki adımlı doğrulama `yeni totp anahtarı` (160 bit
Base32, CSPRNG) ve `kod gizli anahtarıyla doğrulanıyorsa` (RFC 6238, 30 s adım,
6 hane, HMAC-SHA1, ±1 adım pencere, sabit zamanlı karşılaştırma) ile kurulur;
saat adaptörden gelir; web isteğinde ayrı TOTP giriş kovasına tabidir (aşım
yanıtı §5, K-368). Gizli anahtar sunucuda parola özeti gibi saklanır,
istemciye yalnız kayıt anında otpauth bağlantısı olarak verilir.

Başarılı `oturuma al` cümlesi:

1. varsa anonim veya eski oturumu iptal eder;
2. işletim sistemi CSPRNG kaynağından 256 bit yeni oturum kimliği ve
   ayrı bir 256 bit CSRF belirteci üretir;
3. depoda oturum kimliğinin yalnız SHA-256 özetini, kullanıcıyı, rolü,
   CSRF belirtecini ve son geçerlilik anını tutar;
4. oturumu oluşturma anından başlayan mutlak 30 dakika ile sınırlar; proje
   bildirimi bu ömrü ve kayar yenilemeyi K-330 bölümündeki sınırlarla seçebilir.

Bu döndürme session fixation saldırısına karşı normatif davranıştır.
Roller istemciden okunmaz; sunucu oturum kaydından denetlenir. Kimlik
gerektiren rotaya anonim istek 401, doğru kimlik fakat yanlış rol 403 döner.

`oturumu kapat` sunucu kaydını iptal eder ve oturum çerezini `Max-Age=0` ile
sonlandırır. Eski belirteç yeniden kullanılamaz. Süresi dolan kayıt istek
başında silinir. Geçerli erişim tahliye sırasını günceller ama son geçerlilik
anını ileri taşımaz; proje `oturum_kayar` açmadıkça ömür sliding değildir.

Oturum deposu en çok 4096 toplam oturum tutar. Yalnız kimliği doğrulanmış
kayıtlarla doluysa yeni giriş, var olan bir kullanıcıyı düşürmek yerine
fail-closed hata olur. Deneysel web bunu süreç içi adaptörle; `--web-proxy`
production profili proje kökündeki `.zee/web-durumu-v1.json` kalıcı ortak
adaptörüyle uygular. Kalıcı kayıt tokenın kendisini değil yalnız SHA-256
özetini tutar. Bilinmeyen biçim sürümü, bozuk/aşırı büyük içerik, symlink,
güvensiz dosya türü veya IO hatası 503'tür. Kalıcı adaptör durumu klasör
yazma kilidi altında okur, günceller ve yayımlar (K-353); eşzamanlı yazar
sayısı isteği reddettirmez, yalnız spec/08'in 5 sn kilit bekleme sınırı
aşılırsa istek 503 olur.

ZORUNLU: Rota içinde `csrf belirteci` ya da `oturuma al` yeni oturum kaydı için depoda yer ararken depo kaydı reddederse (yalnız kimlikli kayıtlarla dolu kapasite, depo IO hatası ya da kilit bekleme sınırı) bu kaynak durumudur, kullanıcı/program tanısı değildir: rota o cümlede kesilir, isteğin oturum/çerez mutation'ı ve istek transaction'ı rota hatasındaki gibi geri alınır ve yanıt istek commit'indeki depo reddiyle aynı sabit 503 `web güvenlik durumu güvenle kaydedilemedi` gövdesidir (C022 ya da 500 değildir). Neden yalnız operatör günlüğüne düşer, Zee kodu onu yakalamaz ve işçi sonraki isteği alır. Web isteği dışında aynı ret C022 kalır (K-370).

Her rota gövdesinde `oturum`, `Metin` değerli bir sözlük olarak ZORUNLU
bulunur. Geçerli kimlikli sunucu oturumunda tam olarak `kimlik`, `kullanıcı`,
`rol`, `oluşturulma_ms`, `son_erişim_ms` ve `son_geçerlilik_ms` anahtarlarını
taşır; anonim istekte boştur. `kimlik`, ham çerez belirteci değil depodaki 64
haneli küçük harfli SHA-256 özetidir ve yalnız iptal hedefi olarak TANIMLIDIR.
Bu değerlerin tek kaynağı
doğrulanmış sunucu oturum kaydıdır. İstek başlığı, form alanı veya tarayıcı
çerezi `oturum` sözlüğünü dolduramaz; adaptör iç taşıma alanlarını istemci
girdisinden ayırıp güvenilir kayıttan yeniden kurar. Ham oturum belirteci,
CSRF belirteci ve başka sırlar sözlüğe konmaz. `oturum` yalnız rota kapsamına
aittir; rota dışında kullanımı A001'dir.

```text
GET "/panel" adresine istek geldiğinde
    oturum gerekli
    kullanıcı oturumun "kullanıcı" değeri olsun
    rol oturumun "rol" değeri olsun
```

Kimliği doğrulanmış oturumları yönetmek için aşağıdaki iki `Sonuç` ifadesi
TANIMLIDIR:

```text
oturumlar kullanıcı ile web oturumlarını listelemeyi dene olsun
iptal kimlik ile web oturumunu iptal etmeyi dene olsun
```

Listeleme yalnız verilen kullanıcıya ait, süresi dolmamış kimlikli oturumları
oluşturma anı ve kimliğe göre deterministik sıralı `Metin` sözlükleri olarak
döndürmek ZORUNDADIR. Tekil iptal yalnız 64 haneli küçük harfli özet kimliği
kabul etmek, bulunmayan kayıtta başarılı `0`, silinen kayıtta başarılı `1`
döndürmek ve ham belirteç/CSRF açmamak ZORUNDADIR. Her iki işlem `web-oturumu`
yetkinliğine ve istek transaction'ına bağlıdır; ortak kalıcı depoda worker ve
restart sınırlarını aşan iptal aynı cevabın commit/rollback semantiğini izler.

TANIMLI (K-404): Parola değişimi ya da sıfırlaması sonrası kullanıcının öteki
oturumlarını kapatmak ayrı bir dil ilkeli değil, bu üç yüzeyin bileşimidir:
rota `oturumun "kimlik"` değerini saklar, aynı kullanıcı için
`web oturumlarını listelemeyi dene` sonucunu gezer, kimliği geçerli
oturumunkine eşit olmayan her kaydı `web oturumunu iptal etmeyi dene` ile
iptal eder; geçerli oturum korunur ya da aynı istekte `oturuma al` ile yeni
belirteç ve CSRF'e döndürülür (eski belirteç geçersizdir). Bileşim tek istek
transaction'ıdır: başarıda iptaller ve döndürme birlikte commit edilir, rota
hatasında hiçbiri uygulanmaz; başka kullanıcının oturumu etkilenmez. Oturumsuz
"parolamı unuttum" akışı aynı listeyi koşulsuz iptal eder. Listeleme yalnız
commit edilmiş oturumları görür; değişimle yarışan ve eski parolayla doğrulanmış
eşzamanlı girişin oturumunu yakalamak uygulamanın görevidir
([web-production-profili.md](../docs/web-production-profili.md) "Parola
değişiminde öteki oturumları kapatma").

## 3. CSRF

Form gösteren GET rotası `csrf belirteci` ifadesini kullanır:

```text
csrf csrf belirteci olsun
form "<input type=hidden name=_csrf value=\"" ile csrf ile "\">" olsun
```

Geçerli oturum yoksa ifade oluşturma anından başlayan mutlak 10 dakikalık
anonim form oturumu oluşturur. Depoda aynı anda en çok 1024 anonim oturum
bulunur. Anonim veya toplam kota dolduğunda en uzun süredir kullanılmayan
anonim kayıt tahliye edilir; aynı erişim anında oluşturma sırası belirleyicidir.
Belirteç sunucu oturumuna bağlı synchronizer token'dır. Her POST, PUT,
PATCH ve DELETE isteğinde `_csrf` alanı otomatik denetlenir. Oturum, alan veya
eşleşme yoksa rota gövdesi çalışmadan 403 döner.

Başarılı giriş hem oturum kimliğini hem CSRF belirtecini yeniler. Giriş
öncesi anonim CSRF değeri kimliği doğrulanmış oturumda geçerli değildir.

## 4. Çerez profili

`--web-proxy` profilinde yönetilen oturum çerezi:

```text
__Host-zee-oturum=<belirteç>; Path=/; Max-Age=<sınır>;
HttpOnly; SameSite=Lax; Secure
```

`__Host-` öneki nedeniyle Domain niteliği YASAK, Path `/` ve Secure
ZORUNLUDUR. Deneysel düz-HTTP localhost kipinde tarayıcının Secure çerezi
reddetmemesi için ad `zee-oturum`dur; bu kip production profili değildir.

Ham `çerezine yaz`, `çerezini sil` ve `adresine yönlendir` değerleri HTTP
başlığına girmeden doğrulanır. Kontrol karakteri, CR/LF, geçersiz çerez
adı/değeri ve yerel olmayan yönlendirme C022 ile reddedilir.

## 5. HTTPS reverse-proxy sınırı

Production profili şu komutla açılır:

```text
dil çalıştır --web-proxy https://panel.example --web-worker-port 18091 uygulama.dil
```

Runtime yalnız `127.0.0.1` üzerinde düz HTTP dinler. TLS'yi aynı makinedeki
güvenilir reverse proxy sonlandırır. Kabul edilen socket peer'i ayrıca
loopback olmak ZORUNDADIR; aksi durumda başlıklar güvenilir sayılmadan 403
döner. Runtime her istekte:

- tek `Host` başlığının yapılandırılan origin otoritesiyle eşleşmesini;
- tam bir `Forwarded: for=<IP>;proto=https;host=<host>` başlığının tek header
  ve tek hop olmasını;
- `for` değerinin `IpAddr` ile kanoniklenen yalın IPv4/IPv6 adresi olmasını;
- `proto=https` ve `host` değerinin yapılandırılan origin ile eşleşmesini;
- durum değiştiren yöntemde tek `Origin` değerinin yapılandırılan origin
  ile kanonik eşleşmesini ZORUNLU tutar.

`--web-proxy`, `Host`, `Forwarded host` ve `Origin` tek `AgHedefi` origin
ayrıştırıcısını kullanır. Production origin'i HTTPS olmalı; yol, sorgu, parça
ve kullanıcı bilgisi taşımamalı; host geçerli ASCII DNS adı veya köşeli
ayraçlı IPv6, port 1–65535 olmalıdır. Şema yazımı `https://` olmak zorundadır;
DNS adı küçük harfe çevrilir ve açık `:443` varsayılan HTTPS portuyla aynı
kimliktir. IP adresi standart metinsel yazımına çevrilir. Tek kapanış `/`
yalnız origin yazımında kabul edilip kanonik çıktıda kaldırılır. Farklı portlar
aynı origin değildir.

Proxy istemciden gelen `Forwarded`, `X-Forwarded-For` ve benzeri başlıkları
silip doğruladığı bağlantıdan tek kanonik `Forwarded` başlığını kendisi
kurmalıdır. Runtime `X-Forwarded-For`ı istemci kimliği saymaz. Eksik/tekrarlı
Host, tekrarlı `Forwarded`, virgüllü zincir, yinelenen parametre, IP olmayan
`for` ve host uyuşmazlığı 400; eksik `Forwarded` veya HTTPS olmayan proxy
zinciri 426; eksik ya da yanlış unsafe Origin ve loopback dışı peer 403'tür.
Dinleyicinin loopback dışına açılması bu güven sözleşmesini bozar ve
YASAKTIR.

Yanıtlar `no-store` (tek dar istisnası aşağıdaki K-335 bölümündedir), `nosniff`, `DENY`, `no-referrer`, kısıtlı CSP taşır;
HTTPS profilinde HSTS de eklenir. 16 KiB başlık, 64 KiB gövde, byte tabanlı
yalnız-CRLF parser, tek `Content-Length`, origin-form hedef ve
`Transfer-Encoding` reddi request-smuggling/yığın korkuluklarıdır. Başlık ile
gövdenin tamamı bağlantı kabulünden başlayan
10 saniyelik mutlak okuma bütçesini aşarsa 408 döner. Spec/11'in 100 alan ve
30 saniye uygulama sınırı ayrıca geçerlidir.

ZORUNLU: Yanıt yazımı (durum satırı, başlıklar ve gövde; ham 4xx/5xx
yanıtları dahil) yazım başlangıcından itibaren mutlak bir bütçe içinde biter:
`10 sn + yanıt baytı / 128 KiB/s` (boş yanıt 10 sn, 2 MiB 26 sn, 16 MiB metin
üst sınırı 138 sn). Ayrıca 10 saniye boyunca tek bayt ilerlemeyen yazım da
kesilir. Kısmi ilerleme bütçeyi yenilemez; okuyucunun her pencerede biraz
okuması işçiyi bütçeden uzun tutamaz. Aşımda bağlantı kapanır, yanıt teslim
edilemedi olarak sayılır (metrikte yazım hatası, günlükte `yanıt yazım
bütçesi doldu`), isteğin oturum/çerez mutation'ı aşağıdaki yazım hatası
kuralıyla geri alınır ve işçi sonraki isteği alır (K-372).

Doğrulanmış kanonik istemci kimliğiyle varsayılan ortak kalıcı depoda (K-218 explicit bellek seçiminde süreç başına depoda, K-219 uç nokta eşiği ayarı dışında) şu pencereler
ZORUNLUDUR:

| Kapı | Pencere | Eşik | Aşım |
|---|---:|---:|---|
| istemci + yöntem + sorgusuz path | 60 saniye | 100 | 429 |
| CSRF doğrulaması | 60 saniye | 60 | 429 |
| Argon2id/bcrypt parola doğrulaması | 300 saniye | 5 | Parola doğrulaması çalışmadan 429 |
| TOTP kod doğrulaması | 300 saniye | 5 | Kod doğrulaması çalışmadan 429 |

ZORUNLU: `parolanın argon2 özeti` web isteğinde parola doğrulamasıyla aynı giriş kovasını (aynı istemci ve kapsam, 300 saniye/5) artırır. Kova aşılmışsa özet üretilmez, rota o cümlede kesilir, isteğin bekleyen oturum/çerez mutation'ı ve istek transaction'ı rota hatasındaki gibi geri alınır ve yanıt parola doğrulamasıyla aynı sabit 429 `çok fazla giriş denemesi` gövdesidir; oran deposuna erişilemezse aynı biçimde 503 `giriş oran sınırı deposuna erişilemedi` döner. Bu ret kullanıcı tanısı değildir (C028 ya da 500 değildir), Zee kodu onu yakalamaz ve işçi sonraki isteği alır. Web isteği dışında (CLI `dil çalıştır` akışı, açılış bloğu) giriş kovası yoktur; yetkinlik kapalılığı ve adaptör üretim hatası C028 kalır (K-365).

ZORUNLU: `kod gizli anahtarıyla doğrulanıyorsa` web isteğinde parola kovasından ayrı TOTP giriş kovasını (aynı istemci, 300 saniye/5) artırır. Kova aşılmışsa kod denetlenmez; doğru kod da yanlış kod da aynı sabit 429 `çok fazla giriş denemesi` yanıtını alır, oran deposuna erişilemezse 503 `giriş oran sınırı deposuna erişilemedi` döner. Yanıt parola doğrulamasındaki gibi isteğin ilk yanıtıdır; rotanın sonradan gönderdiği yanıt bu durumu değiştirmez ve işçi sonraki isteği alır. Web isteği dışında TOTP kovası yoktur, doğru kod `doğru` kalır (K-368).

AÇIK: Parola ve TOTP doğrulaması aşımda rotaya `yanlış` döndürüp rotayı sürdürür; rota bu dalda kendi hesap kilidi sayacını artırabilir ve aşımı yanlış koddan ayıramaz. Aşımın Zee koduna ayrı bir değerle görünmesi ya da rotanın kesilmesi grammar/IO izi genişlemesidir, core freeze altında açık kalır.

Özet üretimine ayrı kova ya da yakalanabilir `Sonuç` biçimi vermek grammar/API genişlemesidir; V1 core freeze altında K-364 CalDAV/CardDAV taslağının açık sorusudur.

Sayaç artışı ile pencere sonu tek atomik transaction'dır. En çok 32768 canlı
oran anahtarı tutulur. Süresi dolan anahtarlar temizlenir; tablo yalnız canlı
kayıtlarla doluysa varsayılan doluluk seçiminde istek fail-closed 503 olur.
K-219 explicit en-eski seçimi yalnız uç nokta kayıtlarına tahliye uygular;
giriş/CSRF canlı kayıtları korunur.

Oturum/çerez mutation'ı gönderilmemiş ilk yanıtla aynı request transaction'ına
aittir. Mutation socket yanıtından önce ortak depoya atomik commit edilir.
Runtime hata veya 30 saniyelik deadline'da erken başarı yanıtını atar,
oturum mutation'ını bırakır ve 504'ü temiz olarak gönderir. Yanıtsız rota
session kaydı bırakamaz. Gerçek socket yazımı başarısızsa yalnız bu isteğin
eklediği/sildiği ve hâlâ beklenen değeri taşıyan kayıtlar atomik geri alınır;
başka sürecin ilgisiz değişikliği ezilmez. Yarım HTTP gövdesi bağlantı
kapanışıyla geçersiz kalır.

## 5a. Geliştirme ve üretim web sözleşmesi (K-400)

Zee'nin iki gerçek TCP web kipi vardır ve aralarındaki sınır makinece
denetlenir. Geliştirme kipi `dil çalıştır --deneysel-web` (ya da `zee.ayar`
`web-modu: deneysel`) yalnız aynı makinedeki tarayıcı için prototiptir;
üretim kipi `--web-proxy` ailesidir ve filonun tek yayın yoludur.

ZORUNLU (K-400): Deneysel kip dinleyiciyi yalnız `127.0.0.1` üzerinde açar;
bind adresini değiştiren CLI bayrağı, ortam değişkeni ya da proje alanı
yoktur. Kabul edilen bağlantının eşi loopback değilse başlıklar okunmadan
403 `deneysel web yalnız loopback bağlantısı kabul eder` döner.

ZORUNLU (K-400): Deneysel kip her istekte tek `Host` başlığı ister; eksik ya
da yinelenen Host 400'dür. Host otoritesinin adı `localhost`, yalnız ASCII
harf/rakam/tire etiketli `*.localhost` (RFC 6761), noktalı ondalık
`127.0.0.0/8` adresi ya da `[::1]` değilse (kapı 1–65535 serbest) istek
rota, dosya ya da oturum yüzeyine ulaşmadan 421 `Misdirected Request` ile
reddedilir; sabit gövde üretim bayraklarını önerir. İlk retle süreç başına
bir kez, Host değerini yazmayan stderr uyarısı verilir. Böylece DNS
rebinding (saldırgan alanının 127.0.0.1'e çözülmesi) ve public Host'u
koruyan bir tünel/proxy arkasından yayın deneysel kipe ulaşamaz.

TANIMLI (K-400): Deneysel kipte `Forwarded`, `X-Forwarded-For` ve benzeri
başlıklar reddedilmez ama hiçbir zaman kimlik sayılmaz: `istemci adresi` ve
oran kimliği soket IP'sidir. Oturum çerezi `zee-oturum` adıyla Secure'sız
yazılır, HSTS eklenmez; CSP, `nosniff`, gövde/başlık sınırları ve oran
pencereleri üretimle aynıdır. Deneysel kip üretim güvenlik sözleşmesi
değildir ve açılışta bunu stderr'e yazar.

YASAK (K-400): Deneysel kip üretim dinleyici yollarını alamaz. Adlı
dinleyici bildirimi olmayan projede `LISTEN_FDS` ya da `LISTEN_PID`
tanımlıysa (systemd soket devri) kaynak yürütülmeden çıkış 2 ile reddedilir
ve ileti üretim bayraklarını önerir; sessizce 127.0.0.1'e dönülmez.
`--web-worker-port`, `--web-oran-deposu`, `--web-isci-sayisi`,
`--web-systemd-soketi` ve `--web-proxy` deneysel kiple birleşmez (çıkış 2).

ZORUNLU (K-400): İnternete açık her Zee web süreci üretim kipindedir:
`dil çalıştır --web-proxy https://<origin> --web-worker-port <kapı>`, çok
süreçte `--web-isci-sayisi N` ve kesintisiz yeniden başlatma için
`--web-systemd-soketi` (§7). Worker yalnız loopback'te düz HTTP dinler, TLS'yi
aynı makinedeki reverse proxy sonlandırır ve §5'in Host/Forwarded/Origin/eş
kuralları, `__Host-` Secure çerezi (§4), HSTS, kaynak değişmezli CSP (K-379),
16 KiB başlık ve 64 KiB varsayılan gövde ile rota bazlı gövde bütçeleri
(K-391) ve oran pencereleri birlikte geçerlidir.

AÇIK (K-400): Host'u `127.0.0.1` ya da `localhost` olarak yeniden yazan
yerel bir proxy/tünel deneysel kipi dışarı açabilir; çalışma zamanı bunu
gerçek yerel ön kapıdan (ör. resmî sitenin yerel önizlemesi) ayırt edemez.
Bu biçim operatör kuralıyla yasaktır (`docs/web-production-profili.md`);
filo denetimi üretim süreçlerinde `--deneysel-web` olmadığını doğrular.

## 6. Platform sınırı

Native CLI CSPRNG, Argon2id ve sınırlı legacy bcrypt doğrulama capability'sini taşır. WASM playground
production oturumu veya parola özeti üretmez; parola doğrulaması başarısız
olur. TLS sertifikası ve secret dağıtımı deployment katmanının
sorumluluğudur.

V1 HTTP server modeli process başına tek worker/thread'dir: bir bağlantıyı
okur, bir Zee isteğini yürütür ve yanıtı bitirmeden yenisini kabul etmez.
Production concurrency'si aynı proje kökünü ve `.zee/web-durumu-v1.json`
dosyasını paylaşan N ayrı süreçle kurulur: Unix'te tek komutluk resmî çok işçi
profiliyle (§7, `--web-isci-sayisi N`) ya da her süreç kaynak portunu
`--web-worker-port N` ile farklı loopback porta geçirerek. Thread-pool veya aynı
process içinde paralel Zee request yürütümü vaat edilmez. Kalıcı adaptör aynı
makine veya güvenilir kilit+atomik replace semantiği sunan ortak dosya
sistemiyle sınırlıdır; çok-hostlu harici backend henüz yoktur. Sticky session
ortak revoke ve oran sınırının yerine geçmez. Operasyon ayrıntıları
`docs/web-production-profili.md` içindedir.

Normatif gerekçe: RFC-0017, ADR-034, ADR-036, ADR-037, ADR-076, ADR-079 ve ADR-176 (K-400). Rota/eylem ayrımı:
RFC-0015 ve spec/11. Ortak origin tipi: ADR-031 ve spec/23.

## Legacy bcrypt ve kanonik istemci adresi (K-208/K-209)

TANIMLI: `parola ozet ile doğrulanıyorsa` native ortamda `$2a$`, `$2b$`
ve `$2y$` önekli 60 ASCII bayt bcrypt özetini, maliyet 4–14 ve en fazla
72 UTF-8 bayt parola ile doğrular. Büyük maliyet/uzun parola/bozuk biçim
crypto işinden önce `yanlış` olur; sessiz kırpma yapılmaz. Digest
karşılaştırması sabit zamanlıdır; toplam çalışma süresinin eşitliği sözü
yoktur. Yeni bcrypt üretimi YASAKTIR; Argon2id tek üretim yolu kalır.

TANIMLI: `istemci adresi` aktif native handler'da oran sınırıyla aynı
trusted kimliğin kanonik Metin değeridir; güvenli proxy'de doğrulanmış
tek-hop Forwarded, doğrudan deneysel web'de soket IP'sidir. X-Forwarded-For,
form ve sorgu alanları kimlik kaynağı OLAMAZ. IPv6 parantezsizdir; request
cleanup sonrası ve HTTP isteği dışındaki varsayılan adaptörde boş metindir.
Ağ-sunucusu yetkinliği ve mevcut T044 eylem/web ayrımı korunur.

Ürün başarılı girişte eski biçimi SQL LIKE veya mevcut Metin prefix
yüklemiyle ayırıp Argon2'ye taşır; yeni algoritma sorgu yüklemi açılmaz.
IP kişisel veri olarak ele alınır; KVKK bağlamında saklama süresini
uygulama kendi politikasıyla belirler. IO trace IP içerebilir, özel veridir.

## K-218 — Bellek oran backend'i

TANIMLI: `--web-oran-deposu bellek` yalnız `--web-proxy` ile, tek değer ve tek kez kabul edilir; eksik, geçersiz, tekrar veya deneysel/kapalı kipte kullanım hata ile çıkar. Seçilmezse mevcut ortak dosya backend'i kullanılır.

TANIMLI: Bellek backend'i aynı süreç ve clone'ları arasında atomiktir; ayrı worker ve restart ayrı kota oluşturur. Kullanıcı ortak saldırı bütçesini trusted proxy'de sağlamalıdır. Oturum/CSRF/revoke dosya paylaşımı sürer; cookie'siz anonim istek oturum dosyasına dokunmaz, cookie'li okuma ve oturum commit'i kalıcı taze durumla fail-closed çalışır.

TANIMLI: Bellek backend’inin varsayılan Endpoint/CSRF/giriş eşikleri ve 32.768 canlı anahtar sınırı korunur; varsayılan doluluk seçiminde aktif kayıt tahliye edilmez. Dolu depoda yeni anahtar 503 ile reddedilir, mevcut anahtar kotası sürer; süre dolumu kayıt ve son-tarih indeksini birlikte temizler. İndeks canlı anahtar başına bir giriş taşır.

## K-219 — Bellek oran ayarları

TANIMLI: `--web-oran-siniri` 1..1000000 uç nokta isteği/60 saniye, `--web-oran-kapasitesi` 1..32768 canlı anahtar, `--web-oran-doluluk` `reddet` veya `en-eski` kabul eder. Bu tekil ayarlar yalnız `--web-proxy` ve `--web-oran-deposu bellek` ile kullanılabilir; eksik, tekrar, sayı taşması veya sınır dışı değer hata ile çıkar. Varsayılanlar 100, 32768 ve `reddet`tir; giriş 5/300 sn ve CSRF 60/60 sn değişmez.

TANIMLI: Explicit `en-eski` seçiminde yeni anahtar için kapasite gerektiğinde yalnız en küçük pencere sonuna sahip uç nokta kaydı (eşitlikte hash sırası) tahliye edilir; erişim pencereyi uzatmaz. Tahliye edilmiş uç nokta yeniden gelince kota sıfırlanabilir; kullanıcı ortak proxy saldırı bütçesini sağlamalıdır. Canlı giriş/CSRF kayıtları tahliye edilmez; tüm kayıtlar korunmuşsa yeni anahtar 503 alır, mevcut sayaç kotası sürer.

ZORUNLU: Bellek kayıt/süre/tahliye indeksleri aynı süreçte clone’lar arasında atomik tutulur ve seçilmiş kapasiteyi aşmaz. Süre dolumu ve tahliye ilgili tüm indeksleri temizler; mevcut anahtar artışı kapasite tahliyesi başlatmaz. Aynı IP/uç nokta seçilmiş eşiği aşınca 429 döner ve iç oran kararı kalan pencere süresini taşır; dosya backend’i ve oturum/revoke paylaşımı değişmez.

## K-330 — Proje başına oturum ömrü ve kayar yenileme

TANIMLI: Ana `proje.dil` isteğe bağlı `oturum_ömrü_saniye` (sabit Sayı, 300–2592000) ve `oturum_kayar` (`doğru`/`yanlış`) alanlarını kabul eder; ikisi de `web-oturumu` yetkinliği ister. Sınır dışı ömür ve yetkinliksiz kullanım P015, yanlış tür P001 ile reddedilir. Alan yoksa kimlikli oturum ömrü 1800 sn, kayar kapalıdır. Seçilen ömür ilk oturum çerezinin `Max-Age` değeridir.

ZORUNLU: Kayar açıkken her kimlikli istek, istek başındaki kalıcı depo güncellemesinde son geçerliliği `min(şimdi + ömür, oluşturma + 2592000 sn)` yapar, var olan geçerliliği kısaltmaz ve yanıt aynı belirteçle kalan süreyi `Max-Age` olarak taşıyan oturum çerezini yeniden yazar. Oluşturmadan sayılan 30 günlük mutlak tavan hiçbir erişimle aşılamaz. Yeniden başlatma ve ayrı worker'lar aynı kalıcı kaydı görür.

YASAK: Kayar yenileme anonim (giriş öncesi) oturumu uzatamaz ve yeni belirteç üretemez. Aynı istekte giriş, `oturumu kapat` ya da kendi oturumunun kimlikle iptali varsa yenileme çerezi yazılmaz; çıkış sunucu kaydını commit ile hemen siler ve yanıt yalnız `Max-Age=0` silme çerezini taşır.

ZORUNLU: `dil dene` bir proje klasörüyle koşulduğunda her testin hermetik web adaptörü aynı `oturum_ömrü_saniye`/`oturum_kayar` ayarını gerçek runtime ile aynı kurallarla uygular: kimlikli oturum seçilen ömürde biter, kayar kipte kimlikli istek ömrü yeniden başlatır ve aynı belirteçle yenilenmiş `Max-Age` çerezi yazar, mutlak kipte erişim ömrü uzatmaz. Proje bildirimi olmadan koşulan dosya ve gömme `programi_dene` varsayılan 1800 sn mutlak ömürde kalır.

## 7. Çok işçi profili ve açılış birincilliği (K-336/K-337)

TANIMLI: `dil çalıştır --web-proxy <origin> --web-worker-port P --web-isci-sayisi N [--web-saglik-port S] <kaynak>` (1 ≤ N ≤ 64) bir yönetici süreç açar. Yönetici `127.0.0.1:P` dinleyicisini bir kez bağlar ve aynı açık dinleyiciyi N işçi sürecine devreder; bağlantıyı boştaki işçi `accept` ile alır. Yönetici Zee kaynağını yürütmez; her işçi kaynağı baştan yürüten §5 tek süreç çalışma zamanıdır. SO_REUSEPORT ya da proxy aktarımı kullanılmaz.

ZORUNLU: Oturum, CSRF, revoke ve varsayılan ortak oran deposu davranışı N işçide tek süreçle aynıdır: işçiler aynı proje kökünü ve `.zee/web-durumu-v1.json` durumunu paylaşır, bir işçide açılan oturum ve verilen CSRF belirteci ötekinde geçerli, çıkış her işçide 401, giriş deneme sınırı grup genelinde tek sayaçtır. Explicit `--web-oran-deposu bellek` seçimi işçi başınadır ve etkin oran sınırını N katına çıkarır; yönetici açılışta bunu uyarır.

TANIMLI: `birincil süreç` argümansız Mantıksal ifadedir (`ağ-sunucusu` yetkinliği). Tek süreçte ve sunucu yetkinliği olmayan politikada `doğru`; çok işçi grubunda yalnız grubun ilk açılan birincil işçisinde `doğru`, ikincil ve yeniden başlatılan her işçide `yanlış`. Değer süreç ömrü boyunca değişmez; IO izinde `birincil_surec` olayı `0`/`1` sonucuyla kaydedilir ve oynatılır.

ZORUNLU: Yönetici ikincil işçileri birincil işçi `sunucu başlat`a ulaşmadan başlatmaz. Böylece `birincil süreç` doğruyken koşan açılış bloğu grubun ilk açılışında hiçbir işçi istek kabul etmeden ve yalnız bir kez koşar; yeniden başlatılan işçi ikincil olduğundan çalışan işçilerin o an yüklediği geçici dosyaları açılış taramasına kaptırmaz. Birincil `sunucu başlat`a ulaşmadan çıkarsa grup başlatılmaz ve yönetici onun sıfır olmayan çıkış koduyla (yoksa 1) biter.

ZORUNLU: `--web-worker-port` ile açılan her süreç (grup işçisi ya da elle kurulan ayrı portlu süreç) `birincil süreç`i ilk değerlendirdiğinde dosya sınırı kökündeki `.zee/birincil-acilis.kilit` dışlayıcı kilidini alır, tutuluyorsa bırakılana dek bekler. Kilit altında `.zee/birincil-canli.kilit` üzerinde başka katılımcının paylaşımlı kilidi yoksa ve grup rolü ikincil değilse değer `doğru`dur; süreç ardından canlı kilidi ömrü boyunca paylaşımlı tutar. Birincil açılış kilidini `sunucu başlat`a dek tutar, ikincil hemen bırakır. Böylece aynı kökte canlı başka katılımcı varken açılan ya da yeniden başlayan süreç `yanlış` görür ve birincilin açılış bloğu bitmeden başka katılımcı karar vermez. Kilit kurulamazsa (kilit yolu normal dosya değil, kök yazılamaz) süreç stderr'e yazar ve `yanlış` görür (K-341).

TANIMLI: Beklenmedik biçimde çıkan işçi aynı yuvada ikincil olarak 250 ms'den 5 sn'ye katlanan gecikmeyle yeniden başlatılır; 10 sn dolmadan art arda 10 kez çıkan yuva grubu durdurur (çıkış 1). SIGTERM ya da SIGINT alan yönetici işçilere SIGTERM iletir; işçi kabul ettiği isteği tamamlar ve yeni bağlantı almaz; 40 sn içinde bitmeyen işçi sonlandırılır. Yönetici kaybolursa (SIGKILL) işçiler en çok bir boşta yoklama aralığında (100 ms) yeni istek kabul etmeyi bırakıp çıkar.

ZORUNLU: Grup işçisinin dinleyicisi yönetici (ya da systemd) ve kardeş işçilerle paylaşılır; işçi SIGTERM ile kendi kopyasını kapatsa da kuyruktaki bağlantılar grupta kalır. Bu yüzden işçinin başarıyla kabul ettiği bağlantı, kabul ile kapanış denetimi arasına SIGTERM düşse de düşürülmez: istek mevcut bütçelerle yanıtlanır, sonraki kabul denetimi yeni bağlantı almayı durdurur. Dinleyicinin tek sahibi olan tek süreç (doğrudan bind) bu yarışta kabul edilen akışı dinleyiciyle birlikte kapatır (K-253; K-360).

TANIMLI: `--web-saglik-port S` yalnız `127.0.0.1:S` üzerinde `GET /saglik` ve `HEAD /saglik` yanıtlar: birincil hazır ve N işçi süreci yaşıyorsa 200, aksi hâlde 503; gövde `durum` (`basliyor`/`hazir`/`eksik`/`kapaniyor`), `isci`, `canli`, `yeniden_baslatma` ve `surecler` (işçi PID'leri) alanlı JSON'dur. Başka yol 404'tür. Sağlık yanıtı uygulama hazırlığı (DB/göç) kanıtı değildir.

ZORUNLU: Yönetici sağlık bağlantısını işçilere miras bırakmaz: bağlantının kabulü ve FD_CLOEXEC işaretlenmesi işçi başlatmayla aynı anda yapılmaz. Yanıt yazıldıktan sonra bağlantı iki yönde kapatılır; istemci yanıtı her zaman EOF ile alır, başka bir süreçte kopya kalsa da bekletilmez (K-345).

TANIMLI: Proje kökündeki `zee.ayar` (K-234) `guvenlik` dalı `isci-sayisi` (tam sayı 1..64) ve `saglik-port` (tam sayı 1..65535) alanlarıyla aynı grubu kaynak öncesi `--web-isci-sayisi`/`--web-saglik-port` bayraklarına eşler. `isci-sayisi` aynı config'te `web-modu` `proxy` ve `worker-port` ister, `saglik-port` `isci-sayisi` ister; eksik bağımlılık, tür ya da sınır hatası çıkış 2'dir ve CLI'daki worker portu config'i geçerli kılamaz. Kaynak öncesi açık bayrak aynı alanın config değerini ezer. İşçi süreci config'i yeniden okusa da yönetici olmaz; işçi alanları işçide yok sayılır (K-340).

YASAK: `--web-isci-sayisi` `--deneysel-web` ile, worker portu verilmemiş proxy ile, `--web-saglik-port` da `--web-isci-sayisi` olmadan kullanılamaz; 1..64 dışı değer ve `dil çalıştır` dışındaki komut çıkış 2 ile reddedilir. Native listener devri (`--web-listener-fd` …) ve `--web-metrik-fd` çok işçi profiliyle birleşmez.

TANIMLI: `--web-systemd-soketi` (ya da `zee.ayar` `guvenlik.systemd-soketi: true`) verilen işçi grubu yöneticisi ortak dinleyiciyi bağlamaz; systemd soket etkinleştirmesiyle devralır: `LISTEN_PID` yöneticinin PID'i, `LISTEN_FDS` `1` ve soket fd 3 olmalıdır. Soket devralınan dinleyici doğrulayıcısıyla `127.0.0.1:<worker-port>` üzerinde dinleyen TCP olarak sınanır; `LISTEN_PID`, `LISTEN_FDS` ve `LISTEN_FDNAMES` her durumda hemen silinir. Devir yoksa ya da geçersizse çıkış 2'dir ve ileti `.socket` düzeltme önerisi taşır; bind'e geri dönülmez. `LISTEN_FDS` kendiliğinden algılanmaz. Bayrak yalnız `--web-isci-sayisi` ile geçerlidir (K-343).

ZORUNLU: systemd soketiyle çalışan grubun yöneticisi SIGTERM ile boşaltılıp yeniden başlatılırken soket systemd'de kaldığından aralıkta gelen bağlantılar reddedilmez; soket kuyruğunda bekleyip yeni grubun işçileri tarafından yanıtlanır (K-343).

AÇIK: Windows'ta paylaşılan dinleyici devri yoktur; `--web-isci-sayisi` Windows'ta çıkış 2 ile reddedilir ve eşzamanlılık ayrı `--web-worker-port` süreçleri ile önlerindeki proxy dengelemesiyle kurulur. Bağlantıların işçilere dağılımı çekirdeğe bağlıdır (macOS'ta eşit değildir); ayrı portlu süreçlerde birincillik yalnız `birincil süreç`i değerlendiren süreçler arasında ve yerel dosya kilidinin (Unix `flock`, Windows `LockFileEx`) geçerli olduğu tek makinede sağlanır, ağ dosya sistemi ve çok hostlu kurulum kapsam dışıdır.

## 8. Adlı dinleyici devri ve hizmet bildirimi (K-376)

RFC-0039 Aşama A1; karar [ADR-162](../adr/162-adli-dinleyici-devri-ve-hizmet-bildirimi.md), işletim [docs/adli-dinleyici-devri.md](../docs/adli-dinleyici-devri.md).

TANIMLI: `proje.dil` `dinleyiciler` alanı Metin listesidir; her öğe `<ad> <adres>:<kapı>` biçimindedir. Ad `^[a-z][a-z0-9_]{0,31}$` ve systemd `FileDescriptorName=` değeridir; adres IPv4 ya da köşeli parantezli IPv6'dır, kapı 1..65535. Ad ve adres:kapı tekildir, en çok 64 öğe. Çift yığın aile başına ayrı adla bildirilir. Biçim, ad, kapı 0, IPv6 kapsam kimliği, IPv4 eşlemeli IPv6, yinelenen ad/adres ve 64 üstü sayı P018 ile reddedilir. Alan yoksa ya da boş listeyse davranış değişmez.

ZORUNLU: Boş olmayan `dinleyiciler` bildirimli proje `dil çalıştır` ile açıldığında kaynak derlenip yürütülmeden önce systemd soket devri doğrulanır: `LISTEN_PID` sürecin kendi PID'i, `LISTEN_FDS` bildirim sayısı (1..64), `LISTEN_FDNAMES` `LISTEN_FDS` kadar geçerli ve yinelenmeyen ad taşır ve gelen ad kümesi bildirim kümesine eşittir. fd `3+i`'nin adı `LISTEN_FDNAMES`'in i'nci adıdır; eşleme yalnız adla yapılır, gelen sıra bildirim sırasıyla karşılaştırılmaz.

ZORUNLU: Her devralınan fd dinleyen TCP akış soketidir ve yerel adresi adının bildirimindeki aile, adres ve kapıya tam eşittir; IPv6 soketinde `IPV6_V6ONLY` açıktır. Doğrulanan soketler `FD_CLOEXEC` ve engellemeyen kipe alınır ve süreç ömrü boyunca açık tutulur; `LISTEN_PID`, `LISTEN_FDS` ve `LISTEN_FDNAMES` doğrulamadan önce silinir.

YASAK: Devir yoksa ya da herhangi bir koşul tutmazsa (eksik/fazla/yinelenen/geçersiz ad, sayı/ad uyuşmazlığı, başka PID, yanlış tür/aile/adres/kapı, V6ONLY kapalı IPv6) süreç bind'e geri dönemez ve kaynak yürütülemez: çıkış 2'dir, ileti ilgili adı ve `.socket` düzeltme önerisini taşır. Çok işçi grubu işçisi, `--güvenli` çocuk modu ve Unix dışı platform adlı devri çıkış 2 ile reddeder.

AÇIK: Adlı dinleyicilerden bağlantı kabulü yalnız §9'daki bağlantı bloğuyla yapılır; bloğu olmayan adlı dinleyici tutulur ama kabul edilmez. Çok işçi grubuna adla aktarım ve HTTP dinleyicisinin adlı devirden alınması açıktır; HTTP dinleyicisi bugünkü `--web-worker-port`/K-343 yolundan açılır.

TANIMLI: `NOTIFY_SOCKET` her `dil` sürecinin başında okunur ve ortamdan silinir; mutlak yol ve Linux soyut ad (`@`) desteklenir. Değişken yoksa hiçbir bildirim gönderilmez. Gönderim hatası günlüklenir, süreci durdurmaz.

ZORUNLU: `NOTIFY_SOCKET` verilmişse tek süreç web çalışma zamanı `sunucu başlat` dinleyiciyi açtıktan sonra, çok işçi yöneticisi birincil işçi açılışı bitip işçiler başlatıldıktan sonra bir kez `READY=1` gönderir; SIGTERM ile yeni kabul durduğunda (yöneticide kapanışın başında) bir kez `STOPPING=1` gönderir. İşçi süreçleri bildirim göndermez (`NotifyAccess=main`).

## 9. Adlı dinleyici bağlantı yüzeyi (K-381, K-384, K-388, K-396)

RFC-0039 Aşama A2a, A2b (sunucu TLS'i), A2c (blok okuma, K-384) ve A2d (satırlı base64 aktarımı, K-396); karar [ADR-165](../adr/165-ag-dinleyici-baglanti-yuzeyi.md) ve A2d için [ADR-172](../adr/172-baglantiya-satirli-base64-aktarimi.md), işletim [docs/adli-dinleyici-devri.md](../docs/adli-dinleyici-devri.md).

TANIMLI: Üst düzey `"<ad>" dinleyicisinde bağlantı geldiğinde` bloğu, §8 ile devralınan `<ad>` dinleyicisinden kabul edilen her bağlantı için taze ortamda koşar; blok yalnız `bağlantı` adlı Metin sözlüğünü görür (`dinleyici`, `uzak_adres`, `uzak_kapı`, `yerel_kapı`). Bağlantı kalıpları `<ad> bağlantıdan satır okumayı dene olsun` (`Sonuç<Metin>`), `bağlantıya <Metin> satırını yaz`, `<ad> bağlantıya <Metin> satırını yazmayı dene olsun` (`Sonuç<TamSayı>`, CRLF dahil bayt) ve `bağlantıyı kapat`'tır ve o anki bağlantı görevinin bağlantısına uygulanır. Bildirimde olmayan dinleyici adı, aynı ada ikinci blok, aynı programda `sunucu başlat` ile bağlantı bloğu ve bağlantı görevi dışında bağlantı kalıbı C030'dur.

ZORUNLU: Okunan satır yalnız CRLF ile biter ve CRLF'siz verilir. Çıplak LF, çıplak CR ve karışık diziler satır sonu sayılmaz ve normalleştirilmez: bozuk satır tümüyle tüketilir ve okuma `tür=satır_sonu` döner. CRLF dahil `bağlantı_satır_sınırı`nı aşan satır kırpılıp kabul edilmez: `tür=satır_uzun` döner, kalanı sonraki LF'ye kadar atılır ve gelen tampon büyümez. Geçersiz UTF-8, NUL ya da TAB dışı denetim karakteri taşıyan satır `tür=kodlama`dır.

ZORUNLU: Bir okuma çağrısı tam satırı en çok `bağlantı_boşta_süresi` bekler; süre çağrının başından mutlaktır ve aralıklı bayt gelişi onu uzatmaz (`tür=zaman_aşımı`). Etkin Zee son tarihi (`içinde`) okuma ve yazma beklemesinde de denetlenir; dolarsa bekleme noktasında Ç001 iptali olur ve yalnız o son tarihin `yetişmezse` kolu çalışır. Bağlantı görevinin tamamı `bağlantı_ömrü` mutlak son tarihine bağlıdır.

ZORUNLU: Yazılan satıra CRLF'yi çalışma zamanı ekler; CR, LF, NUL ya da TAB dışı denetim karakteri içeren ya da CRLF dahil satır sınırını aşan Metin yazılmaz (`tür=yazılamadı`). Giden tampon bağlantı başına 64 KiB'tır; dolunca yazma görevi askıya alınır (geri basınç) ve tampon `bağlantı_boşta_süresi` içinde boşalmazsa yazma `tür=yazılamadı` olur. `bağlantıya … satırını yaz` cümlesinin hatası yalnız o bağlantı görevini C030 ile bitirir.

ZORUNLU: Açık bağlantı sayısı `bağlantı_sınırı`na ya da aynı uzak IP'den açık bağlantı sayısı `uzak_adres_bağlantı_sınırı`na ulaştığında yeni bağlantı kabul edilir, bildirilmişse `bağlantı_dolu_yanıtı` satırı tek engellemeyen yazımla gönderilir ve bağlantı kapatılır; Zee görevi kurulmaz. Ret sayısı günlüğe en çok beş saniyede bir yazılır.

ZORUNLU: Bağlantı görevleri tek iş parçacıklı bağlantı scheduler'ında ilerler; bekleyen okuma/yazma ve `bekle` scheduler noktasıdır, bir bağlantının beklemesi diğerini durdurmaz. Görev hatası (C023/C024 dahil) yalnız kendi bağlantısını kapatır ve kardeşleri iptal etmez (yalıtılmış bağlantı kapsamı; spec/14 §3'ten bilinçli ayrılış); neden yalnız operatör günlüğüne yazılır. Her görev kendi çalışma bütçesiyle başlar ve her başarılı satır okuması bütçeyi yeniler. Görev içindeki `programı bitir` bütün bağlantıları kapatıp süreci o kodla bitirir.

ZORUNLU (K-388): Bağlantı görevinin protokol birimi sınırı, girdi tüketen ya da süresini dolduran her okuma dönüşüdür: satır okuması, süreli satır okuması, bayt okuması, nokta bloğu ve bayt bloğu çağrısının başarıyla ya da `zaman_aşımı`, `satır_uzun`, `satır_sonu`, `kodlama` türüyle, nokta bloğunda ayrıca `boyut_aşımı` türüyle dönmesi. Her sınırda görevin adım, çıktı ve heap bütçesi (spec/24) web isteğindeki gibi taze kurulur. Anında dönen `kapandı`, `kapanıyor`, `geçersiz` ve `tls`, hiç okumadan dönen bayt bloğu ya da bayt okuması `boyut_aşımı`, yazma, parça yazımı, dosya aktarımı, TLS başlatma ve `bekle` sınır değildir: bunları yok sayan döngü aynı bütçede kalır ve adım (C023) ya da heap (C024) sınırıyla yalnız kendi bağlantısını kapatır. Tek birim içinde spec/24 sınırları aynen geçerlidir. Yenileme `bağlantı_ömrü`, `bağlantı_boşta_süresi`, blok süre bütçesi, `bağlantı_sınırı` ve `uzak_adres_bağlantı_sınırı`nı uzatmaz.

ZORUNLU: Bağlantı scheduler'ı başlayınca `NOTIFY_SOCKET` varsa `READY=1` gönderilir. SIGTERM'de yeni kabul durur, `STOPPING=1` gönderilir, bekleyen ve sonraki okumalar `tür=kapanıyor` döner, yazma sürer; görevler en çok 10 saniyede bitmezse iptal edilip bağlantıları kapatılır ve süreç 0 ile çıkar.

ZORUNLU: Bağlantı bloğu `ağ-dinleyici` yetkinliği ister (spec/23); bildirimli projede yetkinlik yoksa T054, `--güvenli` çocuk modunda C030'dur. Bağlantı olay beklemesi ve her bağlantı işlemi (oku, yaz, kapat, engelleyici bekleme, günlük) IO izine girer; aynı izle yeniden oynatma aynı sırayı ve çıktıyı üretir.

TANIMLI: `proje.dil` bağlantı ayarları isteğe bağlı sabit değerlerdir: `bağlantı_sınırı` 1–4096 (varsayılan 64), `uzak_adres_bağlantı_sınırı` 1–`bağlantı_sınırı` (varsayılan 8, genel sınırdan büyükse ona çekilir), `bağlantı_satır_sınırı` 64–65536 bayt (varsayılan 1000), `bağlantı_boşta_süresi` 1–3600 saniye (varsayılan 300), `bağlantı_ömrü` 1–86400 saniye (varsayılan 3600), `bağlantı_dolu_yanıtı` en çok 512 baytlık yazdırılabilir ASCII, `bağlantı_blok_sınırı` 1024–1073741824 bayt (varsayılan 33554432; A2c). Aralık dışı ya da sabit olmayan değer, kuralı bozan ret satırı, genel sınırı aşan uzak adres sınırı ve `dinleyiciler` bildirimi olmadan bağlantı ayarı P020'dir.

TANIMLI (A2b): `proje.dil` `tls_sertifika_değişkeni` ve `tls_anahtar_değişkeni` PEM sertifika zincirini ve özel anahtarı taşıyan iki ayrı credential/ortam adıdır (A-Z, 0-9, _; değer değil ad); K-339 modeliyle önce `CREDENTIALS_DIRECTORY` dosyasından, yoksa ortamdan okunur. `tls_dinleyicileri` implicit TLS dinleyicilerinin (`dinleyiciler`deki) adlarıdır ve kimlik ister. Eksik, aynı ya da kurala uymayan ad, kimliksiz, bildirilmemiş ya da yinelenen TLS dinleyicisi P021'dir. Kimlik okunamaz, PEM çözülemez ya da anahtar sertifikayla uyuşmazsa süreç kaynak yürütülmeden çıkış 2 verir; özel anahtar baytı hiçbir zaman Zee değeri, IO izi, günlük ya da tanı metni olmaz.

ZORUNLU (A2b): Sunucu TLS politikası sabittir ve bildirimle gevşetilemez: `rustls` + `ring`, yalnız TLS 1.3 ve TLS 1.2; TLS 1.3'te AES-GCM ve CHACHA20-POLY1305, TLS 1.2'de yalnız ECDHE + AES-GCM/CHACHA20-POLY1305 takımları; yeniden müzakere, sıkıştırma ve istemci sertifikası yok. Implicit TLS dinleyicisinde el sıkışma `bağlantı_boşta_süresi` içinde bitmeden Zee görevi kurulmaz; başarısız el sıkışma görev kurmadan kapanır ve bağlantı sınırlarına sayılır. `bağlantı` sözlüğü `tls` = `evet`/`hayır` taşır.

ZORUNLU (A2b): `<ad> bağlantıda TLS başlatmayı dene olsun` (`Sonuç<Metin>`: anlaşılan sürüm ve şifre takımı) yükseltme anında Zee'nin tüketmediği gelen düz metin tamponu boş değilse yükseltme yapmaz, tamponu atar, bağlantıyı kapatır ve `tür=tls` döner (STARTTLS komut enjeksiyonu, CVE-2011-0411 sınıfı); önceki düz metinden hiçbir bayt TLS akışına taşınmaz ve satır okuyucu sıfırlanır. Düz giden tampon yükseltmeden önce boşaltılır. TLS bağlantısında ikinci yükseltme, kimliksiz süreçte yükseltme `tür=tls`'dir (bağlantı açık kalır); süresinde bitmeyen ya da başarısız el sıkışma bağlantıyı kapatır. Protokol durumunun sıfırlanması (EHLO tekrarı, AUTH ilanı) Zee'nin işidir.

TANIMLI (A2c): Blok kalıpları o anki bağlantıya uygulanır: `<ad> bağlantıdan nokta bloğunu dosyaya almayı dene olsun` (SMTP `DATA` gövdesi) ve `<ad> bağlantıdan <n> baytı dosyaya almayı dene olsun` (IMAP literal'i) `Sonuç<Sözlük>` verir; sözlük `yol` (göreli IO köküne göre `.zee/baglanti-bloklari/<64 hex>.parca`), `sha256` (küçük harf 64 hane) ve `bayt` (onluk) Metin alanlarını taşır. `<ad> bağlantıdan <n> bayt okumayı dene olsun` tam `n` baytı bellekte `Sonuç<Metin>` olarak verir. `bağlantıya <Metin> parçasını yaz` ve `<ad> bağlantıya <Metin> parçasını yazmayı dene olsun` (`Sonuç<TamSayı>`) satır sonu eklemeden yazar. `<ad> bağlantıya <yol> dosyasını aktarmayı dene olsun` dosyayı bağlantıya akıtır (`Sonuç<TamSayı>`: bayt) ve ayrıca `dosya-okuma` yetkinliği ister; yol proje dosya sınırıyla çözülür. `<ad> bağlantıdan satır okumayı <n> saniye içinde dene olsun` süresi kısaltılmış satır okumasıdır (`Sonuç<Metin>`).

ZORUNLU (A2c): Nokta bloğu yalnız birebir `CRLF . CRLF` dizisiyle biter; komut satırının CRLF'si bloğun ilk satır başıdır, boş ileti `.` CRLF'dir. Satır başındaki ilk nokta atılır (nokta doldurma çözülür), satır sonları CRLF olarak korunur, sonlandırıcı içeriğe girmez ve sonlandırıcıdan sonraki baytlar tüketilmez (sonraki komuttur). Bloktaki çıplak LF ya da çıplak CR normalleştirilmez ve satır başı sayılmaz: blok `tür=satır_sonu` ile reddedilir, kalan akış gerçek sonlandırıcıya kadar tüketilip atılır; böylece `<LF>.<LF>`, `<LF>.<CR><LF>`, `<CR>.<CR>` ve benzeri dizilerin ardındaki baytlar hiçbir zaman komut olarak okunmaz (SMTP smuggling). Çözülmüş içerik `bağlantı_blok_sınırı`nı aşarsa blok `tür=boyut_aşımı` ile reddedilir ve aynı biçimde sonlandırıcıya kadar tüketilir. Her iki retten sonra oturum eşzamanlı sürer.

ZORUNLU (A2c): Bayt bloğu tam `n` bayttır; sonraki baytlar tüketilmez. `n` `bağlantı_blok_sınırı`nı, bellekteki bayt okumasında `bağlantı_satır_sınırı`nı aşarsa hiçbir bayt okunmadan `tür=boyut_aşımı`, negatif `n` ya da 1 saniyeden kısa okuma süresi `tür=geçersiz` döner. Bellekteki bayt okuması geçerli UTF-8 ister ve NUL kabul etmez (`tür=kodlama`; baytlar tüketilmiştir); CR ve LF içerikte kalabilir.

ZORUNLU (A2c): Blok içeriği belleğe alınmadan `0600` izinli, benzersiz `.parca` dosyasına yazılır ve SHA-256 aynı akışta hesaplanır; başarı dosya `fsync` edildikten sonra döner. Yarım dosya her hatada (`satır_sonu`, `boyut_aşımı`, `zaman_aşımı`, `kapandı`, `kapanıyor`, yazma hatası) silinir. Tamamlanan dosya bağlantıya aittir: aynı bağlantıdaki sonraki blokta ya da bağlantı bitince, Zee taşımadıysa silinir; bağlantı başına en çok bir taşınmamış dosya durur. Süreç ölümünden kalan dosyalar çalışma zamanınca en sık 5 dakikada bir, yalnız runtime adlandırmasına uyan ve son değişikliği `bağlantı_ömrü` + 15 dakikadan eski normal dosyalar olarak süpürülür; web yüklemesi dizini (spec/26) ayrıdır.

ZORUNLU (A2c): Blok okuması, bayt okuması ve dosya aktarımı çağrının başından mutlak `bağlantı_boşta_süresi + bayt / 65536 bayt/sn` bütçesine bağlıdır (nokta bloğunda bayt o ana dek alınan, literal ve aktarımda bildirilen/dosya baytıdır); bu hızın altındaki damla damla trafik bütçeyi uzatamaz. Blok ya da literal yarıda kalırsa (`zaman_aşımı`, `kapandı`, `kapanıyor`, yazma hatası) bağlantının okuma yönü kapanır: sonraki okumalar `tür=kapandı` döner, yazma ve kapatma çalışır (Zee `421` yazabilir). Aktarım süresinde bitmez ya da dosya aktarım sırasında kısalırsa bağlantı kırık sayılır ve `tür=yazılamadı` döner; `bağlantı_blok_sınırı`nı aşan dosya baytı gönderilmeden `tür=boyut_aşımı`, açılamayan, normal olmayan ya da sınır dışı dosya `tür=dosya`'dır. Tek çağrı bir turda en çok 256 KiB işleyip scheduler'a döner.

ZORUNLU (A2c): Parça yazımı satır kuralına uyar: CR, LF, NUL ya da TAB dışı denetim karakteri içeren ya da `bağlantı_satır_sınırı`nı aşan Metin yazılmaz (`tür=yazılamadı`); satır sonunu yalnız `… satırını yaz` üretir, dolayısıyla Zee metninde CR kaçışı gerekmez ve CRLF dışı satır sonu yazılamaz. Aktarım giden tamponu 64 KiB geri basınçla doldurur; dosya baytı Zee değeri ya da IO izi içeriği olmaz.

ZORUNLU (A2c): Okuma zaman aşımı (`tür=zaman_aşımı`) bağlantıyı kapatmaz ve gelen tampondaki yarım satırı atmaz; sonraki okuma onu tamamlar. Süreli okumanın süresi `bağlantı_boşta_süresi`ni aşamaz (aşan değer ona indirilir) ve çağrının başından mutlaktır; IDLE gibi uzun beklemeler Zee'nin kısa süreli okuma döngüsüyle yazılır.

TANIMLI (A2c): `bağlantı` Sözlüğü sözcüksel kapsamlıdır: yalnız bağlantı bloğunun kendi gövdesinde tanımlıdır ve bloğun çağırdığı işleme geçmez; işlemde kullanılması A001'dir ve tanı alanın parametre olarak geçirilmesini önerir. Bağlantı kalıpları (okuma, yazma, blok, TLS) işlemde de o anki bağlantıya uygulanır. Adı yerleşik `yaz` ya da `gönder` fiiliyle biten işlem cümle biçiminde çağrılamaz (satır yerleşik cümle olarak ayrıştırılır); bu durumda çıkan A001 ve S037 tanıları çakışan işlemi adıyla anar ve başka ad önerir, ifade konumundaki çağrı değişmez.

TANIMLI (A2d): `<ad> <yol> dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun` dosyanın tamamını, `<ad> <yol> dosyasının base64 satırlarının <konum> baytından <adet> baytını bağlantıya aktarmayı dene olsun` ise kodlanmış çıktının bir aralığını o anki bağlantıya akıtır; ikisi de `Sonuç<TamSayı>` (yazılan bayt) verir. Yol Metin, `konum` ve `adet` TamSayıdır (aksi T034). Kaynak yol dosya okumasının kurallarına uyar: proje dosyası `dosya-okuma`, `veri:` yolu `veri-okuma` (spec/23, RFC-0045) yetkinliği ister ve aynı merkezî yol sınırından geçer; `ağ-dinleyici` ayrıca gerekir.

ZORUNLU (A2d): Çıktı dosya baytlarının RFC 4648 §4 alfabesiyle dolgulu base64 kodlamasıdır ve RFC 2045 §6.8 satırlarına bölünür: son satır dışındaki her satır tam 76 karakterdir, son satır 1–76 karakterdir ve her satır, son satır dahil, CRLF ile biter; boş dosya sıfır bayt üretir. `b` ham bayt için kodlanmış karakter sayısı e = 4·⌈b/3⌉ ve yazılan toplam bayt T = e + 2·⌈e/76⌉'dır; tam aktarım başarıda tam olarak T döndürür. `b` dosyanın açıldığı andaki boyudur.

ZORUNLU (A2d): Aralık, T baytlık kodlanmış çıktı akışı (CRLF baytları dahil, sıfır tabanlı) üzerindedir; ham dosya konumu değildir. Yazılan baytlar akışın [min(K, T), min(K + N, T)) dilimidir ve dönen sayı min(N, max(0, T − K))'dir (K = `konum`, N = `adet`). K ≥ T ya da N = 0 hiçbir bayt yazmadan başarıyla 0 döner (IMAP `BODY[n]<K.N>` kırpması). Negatif `konum` ya da `adet` dosya açılmadan `tür=geçersiz` C030 ret Sonuç'udur.

ZORUNLU (A2d): Aktarım akışlıdır: kodlanmış çıktı Zee `Metin`i olmaz, değer zarfına sayılmaz ve bellekte aynı anda en çok bir ham parçanın (57·256 bayt) kodlaması durur; aralık, satır başına hizalı ham konuma atlanarak başlar. A2c dosya aktarımının kuralları aynen geçerlidir: 64 KiB giden tampon geri basıncı, tur başına en çok 256 KiB, çağrının başından mutlak `bağlantı_boşta_süresi + yazılan bayt / 65536 bayt/sn` bütçesi, `bağlantı_blok_sınırı`nın ham dosya boyuna uygulanması (`tür=boyut_aşımı`, bayt gönderilmez), yok/normal olmayan/sınır dışı dosyada `tür=dosya`, karşı tarafın kapanmasında, süre aşımında ya da dosya kısaldığında bağlantının kırık sayılması ve `tür=yazılamadı`. IO izi yalnız yolu ve aralığı taşır, içerik taşımaz.

ZORUNLU (A2d): Kaynak dosya tavanı 32 MiB (33 554 432 bayt) ham dosya boyudur ve K-395 çözerek yazma hedef tavanıyla hizalıdır; tavandaki dosya kabul edilir (T = 45 916 594 bayt), bir bayt fazlası her iki biçimde de dosya açıldıktan hemen sonra, hiçbir bayt yazılmadan `tür=boyut_aşımı` C030 ret Sonuç'udur. Tavan `bağlantı_blok_sınırı`ndan bağımsızdır: blok sınırı daha küçükse o, daha büyükse 32 MiB uygulanır.

AÇIK: Bağlantı görevi içindeki PostgreSQL, dosya ve HTTP çağrıları ile blok dosyasına yazım senkron dilimdir ve kardeş bağlantıları o süre bekletir. Bağlantılar arası uyandırma (yeni ileti gelince IDLE oturumunu uyandırma) yoktur; Zee kısa süreli okumayla yoklar. Unix dışı platformda bağlantı bloğu C030'dur. Binlerce boşta oturumun bellek ölçümü yapılmamıştır. Ham dosya baytlarının kısmi aktarımı (IMAP `BODY[]<o.n>` ham baytı) yoktur; base64 kodlu kısmi getirme A2d aralığıyla yapılır (K-396). Birim sınırındaki bütçe yenilemesi önceki birimlerde bağlanıp yaşayan değerleri yeni bütçeye yeniden saymaz; oturum boyunca biriken canlı değer bağlantı başına ayrıca sınırlanmaz (her değer spec/24 tekil metin ve koleksiyon sınırına bağlıdır). Birim içinde atılan geçici değer zarftan iade edilmez; MB ölçeğinde gövdeyi tek birimde Zee metni olarak işlemek C024'e yürür (dosya tutamağıyla aralıklı işleme RFC-0043 taslağıdır).

## K-335 — Önbelleklenebilir herkese açık rota

TANIMLI: Rota önsözünün politika cümlesi `herkese açık önbelleklenebilir N saniye` biçiminde yazılabilir; N 1–86400 arası sabit TamSayıdır. Bu cümle erişim bakımından `herkese açık` ile aynıdır ve önsözün ilk ve tek politika cümlesi kuralına (T050) uyar. Başka sayı, birim ya da eksik parça S043'tür.

YASAK: Önbelleklenebilir politika GET ve HEAD dışındaki rotada T056 ile reddedilir. Rota gövdesi `oturum` ya da `çerezler` sözlüğünü okuyamaz; gövde ya da gövdenin (geçişli olarak) çağırdığı işlemler `csrf belirteci`, çerez yazma/silme, `oturuma al`, `oturumu kapat` ve web oturumu listeleme/iptal ifadelerini kullanamaz. İhlal T057'dir ve ilk dokunuşun satırını gösterir.

ZORUNLU: Çalışma zamanı önbelleklenebilir rotaya boş `oturum` ve `çerezler` sözlükleri verir. Yanıt yalnız şu koşulların hepsi sağlanınca `Cache-Control: public, max-age=0, s-maxage=N` taşır: istek yöntemi GET ya da HEAD, rota gövdeyle 200 yanıtı gönderdi, yanıt hiçbir `Set-Cookie` başlığı taşımıyor ve istek sırasında oturum yüzeyine dokunulmadı. Diğer her durumda (hata/durum yanıtı, yönlendirme, kayar oturum yenilemesi dahil herhangi bir Set-Cookie) yanıt `no-store` kalır. Diğer güvenlik başlıkları değişmez; `max-age=0` tarayıcı önbelleğini kapalı tutar, ömür yalnız paylaşılan önbelleğe verilir.

AÇIK: `Vary`, ETag/koşullu istek, `stale-while-revalidate` ve önbellek temizleme sözü verilmez. Sorgu dizgisi önbellek anahtarının parçasıdır ve ön kapı tarafından korunmalıdır. Normatif gerekçe: RFC-0037, ADR-160.

## K-405 — Makine uç noktası

TANIMLI: Rota önsözünün politika cümlesi `herkese açık makine uç noktası` biçiminde yazılabilir. Cümle erişim bakımından `herkese açık`tır ve ilk ve tek politika cümlesi kuralına (T050) uyar. `makine uç noktası` ile biten başka her yazım (oturumlu, yetkili, önbelleklenebilir ya da tek başına) S050'dir. `isteğin ham gövdesi` ifadesi yalnız bu rotanın kendi gövdesinde okunur ve isteğin gövdesini `Metin` olarak verir.

YASAK: Makine uç noktası POST dışı rotada, önekli yolda ya da sabit metin olmayan yolda T060 ile reddedilir. Rota gövdesi `oturum` ya da `çerezler` sözlüğünü okuyamaz; gövde ya da gövdenin (geçişli olarak) çağırdığı işlemler `csrf belirteci`, çerez yazma/silme, `oturuma al`, `oturumu kapat` ve web oturumu listeleme/iptal ifadelerini kullanamaz (T061, ilk dokunuşun satırı). `isteğin ham gövdesi` sıradan rotada, işlem/eylem gövdesinde ya da üst düzeyde T062'dir.

ZORUNLU: Makine uç noktasına giden istek CSRF ve `Origin` denetiminden geçmez; Host, `Forwarded` ve HTTPS proxy denetimi aynen uygulanır. İstek `Cookie` başlığı okunmaz, oturum anonim başlar, kayar yenileme yapılmaz ve yanıt hiçbir koşulda `Set-Cookie` taşımaz; rotaya boş `oturum` ve `çerezler` sözlükleri verilir. Uç nokta oran sınırı rotanın kendi `POST <yol>` kovasında uygulanır.

ZORUNLU: Makine isteğinin tek `Content-Type` başlığı `text/xml`, `application/xml` ya da `application/json` medya türü taşımalı, `charset` parametresi varsa `utf-8` olmalıdır; başlık yoksa ya da tür başkaysa yanıt 415'tir. Gövde 8 KiB (8192 bayt) ile sınırlıdır; aşım gövde okunmadan 413, geçersiz UTF-8 400'dür. Gövde form olarak ayrıştırılmaz: `istek` sözlüğü yalnız `yol`, `yöntem` ve sorgu alanlarını taşır. CSP ve diğer güvenlik başlıkları (`Cache-Control: no-store` dahil) değişmez.

AÇIK: Dil XML ya da JSON ayrıştırmaz; ürün gövdeyi metin işlemleriyle işler. E-posta adresinin günlüğe yazılmaması ve yanıtın alan adına göre değişmemesi (sayım) ürünün sorumluluğundadır. Kullanıcıya göre üretilen yanıt K-406 türlü metin yanıtıyla (`application/xml`) yazılabilir; adresi XML'e kaçışlamak ürünün işidir. Büyük/küçük harfe duyarsız yol, PUT/PATCH ve CORS başlıkları bu sözleşmenin parçası değildir. Normatif gerekçe ve tehdit modeli: RFC-0046, ADR-179.

## K-379 — Satır içi betik CSP özeti

TANIMLI: Gerçek web adaptörünün `text/html` 200 yanıtında CSP `script-src` yönergesi `'self'` ile, gövdedeki öznitelik taşımayan her satır içi `<script>…</script>` bloğundan (gövde ilk `</script>` kapanışında biter) yalnız güvenilir olanların `'sha256-…'` özetlerini taşır. Güvenilir betik kümesi program derlendikten sonra ve ilk istekten önce bir kez kurulur: ana kaynak ile derlemenin yüklediği her birimin (yerel, paket ve gömülü) `test` blokları (derinlik 0'da satır başı `test` kelimesi ve girintili gövdesi) dışındaki her dize değişmezinin, kaçışları çözülmüş değerinde aynı kuralla tam bulunan betik gövdeleridir.

ZORUNLU: Gövdesi güvenilir kümede olmayan satır içi betik (istek alanı, dosya, veritabanı ya da başka bir çalışma zamanı değerinden gelen, birden çok değişmezin birleştirilmesiyle kurulan, yorumdaki ya da yalnız bir `test` bloğundaki değişmezde duran betik) CSP özeti almaz ve tarayıcıda çalışmaz; çalışma zamanı ilk böyle betikte süreç başına bir kez, betik içeriğini yazmadan stderr uyarısı verir. Küme kurulmadan (derleme bitmeden) hiçbir satır içi betik izinli değildir; `'unsafe-inline'` hiçbir koşulda eklenmez.

AÇIK: Kaynak değişmezindeki bir betiği gövdeye yansıtan istemci o betiği yeniden çalıştırabilir (özet CSP'nin bilinen sınırı); bu, yalnız geliştiricinin yazdığı betiklerdir. Öznitelikli `<script …>` blokları özetlenmez: `src` betikleri `'self'` ile, veri blokları (`type="application/ld+json"`) yürütülmediği için etkilenmez. Hata/durum yanıtları satır içi betik izni taşımaz.
