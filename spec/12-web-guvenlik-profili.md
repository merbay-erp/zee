# 12 — Web güvenlik profili

Bu bölüm K-088/K-134/K-137/K-139 ile gelen normatif oturum, yetki, CSRF, ortak
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
aranmayacağını açıklar. Politika eksikliği T049'dur. Politika ilk ve tek
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

## 2. Parola ve oturum açma

Parola doğrulama yalnız Argon2id v19 PHC özeti kabul eder:

```text
verilen parola_özeti ile doğrulanıyorsa
    "Mustafa" kullanıcısını "yönetici" rolüyle oturuma al
```

Bozuk, başka algoritmalı veya eşleşmeyen özet `yanlış` sonucudur. PHC
özeti `dil parola-özeti` ile parolayı terminalde göstermeden üretilir;
otomasyon için açık `--stdin` seçeneği vardır.

Başarılı `oturuma al` cümlesi:

1. varsa anonim veya eski oturumu iptal eder;
2. işletim sistemi CSPRNG kaynağından 256 bit yeni oturum kimliği ve
   ayrı bir 256 bit CSRF belirteci üretir;
3. depoda oturum kimliğinin yalnız SHA-256 özetini, kullanıcıyı, rolü,
   CSRF belirtecini ve son geçerlilik anını tutar;
4. oturumu oluşturma anından başlayan mutlak 30 dakika ile sınırlar.

Bu döndürme session fixation saldırısına karşı normatif davranıştır.
Roller istemciden okunmaz; sunucu oturum kaydından denetlenir. Kimlik
gerektiren rotaya anonim istek 401, doğru kimlik fakat yanlış rol 403 döner.

`oturumu kapat` sunucu kaydını iptal eder ve oturum çerezini `Max-Age=0` ile
sonlandırır. Eski belirteç yeniden kullanılamaz. Süresi dolan kayıt istek
başında silinir. Geçerli erişim tahliye sırasını günceller ama son geçerlilik
anını ileri taşımaz; ömür sliding değildir.

Oturum deposu en çok 4096 toplam oturum tutar. Yalnız kimliği doğrulanmış
kayıtlarla doluysa yeni giriş, var olan bir kullanıcıyı düşürmek yerine
fail-closed hata olur. Deneysel web bunu süreç içi adaptörle; `--web-proxy`
production profili proje kökündeki `.zee/web-durumu-v1.json` kalıcı ortak
adaptörüyle uygular. Kalıcı kayıt tokenın kendisini değil yalnız SHA-256
özetini tutar. Bilinmeyen biçim sürümü, bozuk/aşırı büyük içerik, symlink,
güvensiz dosya türü veya IO hatası 503'tür.

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

Yanıtlar `no-store`, `nosniff`, `DENY`, `no-referrer`, kısıtlı CSP taşır;
HTTPS profilinde HSTS de eklenir. 16 KiB başlık, 64 KiB gövde, tek
`Content-Length` ve `Transfer-Encoding` reddi request-smuggling/yığın
korkuluklarıdır. Başlık ile gövdenin tamamı bağlantı kabulünden başlayan
10 saniyelik mutlak okuma bütçesini aşarsa 408 döner; yanıt yazımı da 10 saniye
socket zaman aşımı taşır. Spec/11'in 100 alan ve 30 saniye uygulama sınırı
ayrıca geçerlidir.

Doğrulanmış kanonik istemci kimliğiyle ortak kalıcı depoda şu sabit pencereler
ZORUNLUDUR:

| Kapı | Pencere | Eşik | Aşım |
|---|---:|---:|---|
| istemci + yöntem + sorgusuz path | 60 saniye | 100 | 429 |
| CSRF doğrulaması | 60 saniye | 60 | 429 |
| Argon2id parola doğrulaması | 300 saniye | 5 | Argon2id çalışmadan 429 |

Sayaç artışı ile pencere sonu tek atomik transaction'dır. En çok 32768 canlı
oran anahtarı tutulur. Süresi dolan anahtarlar temizlenir; tablo yalnız canlı
kayıtlarla doluysa etkin bir sayacı tahliye ederek eşiği delmek yerine istek
fail-closed 503 olur.

Oturum/çerez mutation'ı gönderilmemiş ilk yanıtla aynı request transaction'ına
aittir. Mutation socket yanıtından önce ortak depoya atomik commit edilir.
Runtime hata veya 30 saniyelik deadline'da erken başarı yanıtını atar,
oturum mutation'ını bırakır ve 504'ü temiz olarak gönderir. Yanıtsız rota
session kaydı bırakamaz. Gerçek socket yazımı başarısızsa yalnız bu isteğin
eklediği/sildiği ve hâlâ beklenen değeri taşıyan kayıtlar atomik geri alınır;
başka sürecin ilgisiz değişikliği ezilmez. Yarım HTTP gövdesi bağlantı
kapanışıyla geçersiz kalır.

## 6. Platform sınırı

Native CLI CSPRNG ve Argon2id capability'sini taşır. WASM playground
production oturumu veya parola özeti üretmez; parola doğrulaması başarısız
olur. TLS sertifikası ve secret dağıtımı deployment katmanının
sorumluluğudur.

V1 HTTP server modeli process başına tek worker/thread'dir: bir bağlantıyı
okur, bir Zee isteğini yürütür ve yanıtı bitirmeden yenisini kabul etmez.
Production concurrency'si aynı proje kökünü ve `.zee/web-durumu-v1.json`
dosyasını paylaşan N ayrı süreçle kurulur; her süreç kaynak portunu
`--web-worker-port N` ile farklı loopback porta geçirir. Thread-pool veya aynı
process içinde paralel Zee request yürütümü vaat edilmez. Kalıcı adaptör aynı
makine veya güvenilir kilit+atomik replace semantiği sunan ortak dosya
sistemiyle sınırlıdır; çok-hostlu harici backend henüz yoktur. Sticky session
ortak revoke ve oran sınırının yerine geçmez. Operasyon ayrıntıları
`docs/web-production-profili.md` içindedir.

Normatif gerekçe: RFC-0017, ADR-034 ve ADR-036. Rota/eylem ayrımı: RFC-0015
ve spec/11. Ortak origin tipi: ADR-031 ve spec/23.
