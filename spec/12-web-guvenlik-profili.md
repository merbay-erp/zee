# 12 — Web güvenlik profili

Bu bölüm K-088 ile gelen normatif oturum, yetki, CSRF ve HTTPS reverse-proxy
sözleşmesidir. Uygulama eyleminin HTTP'den ayrılması spec/11'de tanımlıdır;
bu bölüm tarayıcı isteğinin o eyleme hangi kapılardan geçerek ulaştığını
tanımlar.

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
4. oturumu 30 dakika ile sınırlar.

Bu döndürme session fixation saldırısına karşı normatif davranıştır.
Roller istemciden okunmaz; sunucu oturum kaydından denetlenir. Kimlik
gerektiren rotaya anonim istek 401, doğru kimlik fakat yanlış rol 403 döner.

`oturumu kapat` sunucu kaydını iptal eder ve oturum çerezini `Max-Age=0` ile
sonlandırır. Eski belirteç yeniden kullanılamaz. Süresi dolan kayıt istek
başında silinir.

## 3. CSRF

Form gösteren GET rotası `csrf belirteci` ifadesini kullanır:

```text
csrf csrf belirteci olsun
form "<input type=hidden name=_csrf value=\"" ile csrf ile "\">" olsun
```

Geçerli oturum yoksa ifade 10 dakikalık anonim form oturumu oluşturur.
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
dil çalıştır --web-proxy https://panel.example uygulama.dil
```

Runtime yalnız `127.0.0.1` üzerinde düz HTTP dinler. TLS'yi aynı makinedeki
güvenilir reverse proxy sonlandırır. Runtime her istekte:

- tek `Host` başlığının yapılandırılan host ile eşleşmesini;
- tek `X-Forwarded-Proto` değerinin `https` olmasını;
- durum değiştiren yöntemde tek `Origin` değerinin yapılandırılan origin
  ile birebir eşleşmesini ZORUNLU tutar.

Eksik/tekrarlı Host 400, HTTPS olmayan proxy zinciri 426, eksik veya yanlış
unsafe Origin 403'tür. Dinleyicinin loopback dışına açılması bu güven
sözleşmesini bozar ve YASAKTIR.

Yanıtlar `no-store`, `nosniff`, `DENY`, `no-referrer`, kısıtlı CSP taşır;
HTTPS profilinde HSTS de eklenir. 16 KiB başlık, 64 KiB gövde, tek
`Content-Length` ve `Transfer-Encoding` reddi request-smuggling/yığın
korkuluklarıdır. Başlık ile gövdenin tamamı bağlantı kabulünden başlayan
10 saniyelik mutlak okuma bütçesini aşarsa 408 döner; yanıt yazımı da 10 saniye
socket zaman aşımı taşır. Spec/11'in 100 alan ve 30 saniye uygulama sınırı
ayrıca geçerlidir.

## 6. Platform sınırı

Native CLI CSPRNG ve Argon2id capability'sini taşır. WASM playground
production oturumu veya parola özeti üretmez; parola doğrulaması başarısız
olur. TLS sertifikası, secret dağıtımı, kaba-kuvvet/rate-limit ve çok süreçli
paylaşılan oturum deposu deployment katmanının sorumluluğudur; bu profil
bunları varmış gibi göstermez.

Normatif gerekçe: RFC-0017. Rota/eylem ayrımı: RFC-0015 ve spec/11.
