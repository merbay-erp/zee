# RFC-0015 — Uygulama eylemleri ve web güvenlik sınırı

- **Durum:** geçici kabul — K-087 eylem sınırı ve K-088 güvenlik profili gerçeklendi
- **Tarih:** 1 Eylül 2026
- **İlgili kararlar:** ADR-010, K-081, K-087, K-088, RFC-0017; v1 kapıları P0-02/P0-03/P0-04
- **Gerçekleme:** açık imzalı eylem, yöntemli rota, geçişli etki denetimi,
  istek limiti/son tarihi, yerel dosya savepoint'i, production oturum/CSRF ve
  TLS/proxy profili var; idempotency ve dağıtık transaction açık

## Problem

`"/kaydet" adresine istek geldiğinde` bugün çalışan bir rota gövdesidir.
HTTP yöntemi, doğrulama, yetki, CSRF, idempotency, transaction ve timeout bu
gövdenin dil sözleşmesinde görünmez. Aynı iş kuralı web formuna gömülür;
CLI/job/queue/API tarafından yeniden kullanılması kullanıcı disiplinine kalır.

Bu yüzey eğitim ve localhost prototipi için değerlidir ama production web
uygulaması anlamına gelmez. Güvenli varsayımlar gereği iki katman ayrılacaktır:

- **adaptör:** HTTP isteğini güvenli, sınırlı girdiye dönüştürür;
- **eylem:** iş kuralını, yetkiyi ve kalıcı durum sınırını taşır.

## Değişmez güvenlik kuralları

1. GET/HEAD güvenlidir: durum değiştiren capability kullanamaz.
2. Durum değiştiren HTTP çağrısı yöntemi açıkça POST/PUT/PATCH/DELETE'dir;
   yöntem bir sözlük anahtarını elle kontrol etmeye bırakılmaz.
3. İstek gövdesi, alan sayısı ve süre üst sınırı varsayılanla gelir; büyütmek
   açık karardır.
4. Kimlik doğrulama ile yetkilendirme ayrıdır. Bir kullanıcının tanınması,
   eylemi yapmaya yettiği anlamına gelmez.
5. Tarayıcıdan durum değiştiren eylem CSRF politikasız yayınlanamaz.
6. Eylem ya tamamlanır ya geri alınır. Dosya, veri tabanı ve kuyruk aynı
   transaction API'sini birebir vermek zorunda değildir ama yarım başarı
   görünür ve yönetilebilir olmalıdır.
7. Tekrar gelebilen eylem idempotency politikasını açıklar; ağ tekrarı sessiz
   çift kayıt üretmez.
8. Oturum tokenı CSPRNG'dir; düz parola ve küçük rastgele sayı production
   API'sinin parçası olamaz. Çerez varsayılanı HttpOnly + SameSite=Lax;
   HTTPS ortamında Secure'dur ve ömür açıkça sınırlıdır.
9. Web adaptörü ham hata ayrıntısını istemciye sızdırmaz; yapılandırılmış Hata
   güvenli HTTP sonucuna eşlenir, tam neden gözlemlenebilirlik katmanında kalır.

## Kabul edilen çekirdek yüzey (K-087)

```text
eylem notu kaydet
    notu Metin olarak al
    değer döndürmez
    "notlar.txt" dosyasına notu ekle

POST "/notlar" adresine istek geldiğinde
    not isteğin "not" değeri olsun
    not ile notu kaydet
    "/notlar" adresine yönlendir
```

Eylem HTTP bilmez. Aynı eylem daha sonra CLI, zamanlanmış iş, kuyruk veya
test adaptöründen çağrılabilir. Form bir iş kuralı değil, eyleme adaptördür.

## Capability ve etki modeli

Derleyici bir eylemin etkilerini şu kapalı sınıflarda çağrı grafiği boyunca
görür:

- `salt-okuma`
- `durum-yazma`
- `web-adaptörü`
- `geri-alınamaz dış etki`

GET/HEAD adaptörü yalnız salt okumaya ulaşabilir. Yazma, normal işlem içine
saklanarak sınır dolanılamaz. Eylem HTTP yanıtı/çerezi ve geri alınamayan
ekran/girdi/donanım etkisi taşıyamaz. Ayrıntılı normatif sözleşme spec/11'dedir.

## Aşamalı gerçekleme

1. **Korkuluk (K-082 — gerçeklendi):** mevcut TCP/web yüzeyi açık
   `--deneysel-web` opt-in'i olmadan gerçek soket açmaz; örnekler production
   olmadığını söyler ve bütün durum değişiklikleri POST kontrolü taşır.
2. **Protokol sınırı (K-087 çekirdeği):** yöntemli route, 64 KiB/100 alan
   sınırı, 30 saniye istek son tarihi, 404/405/413/504 ayrımı gerçeklendi.
   Güvenli çerez ve kontrollü reverse-proxy güveni K-088/RFC-0017 ile kapandı.
   K-176 sorgu/form yüzde kodlamasını tam iki hexadecimal hane ve exact UTF-8
   zorunluluğuyla fail-closed 400 sınırına taşır.
3. **Eylem (K-087):** tam tür sözleşmeli ve form/API/CLI/görev bağlamından
   bağımsız çağrı; HTTP etkisi derlemede yasak.
4. **Durum (K-084/K-087):** tek-dosya atomik değiştirme ve süreç kilidinin
   üstünde, çalışma hatası/başarısız Sonuç için iç içe çok-dosyalı savepoint
   geri alması gerçeklendi. Süreç çökmesinde çok-dosyalı tek commit, veri
   tabanı/dağıtık transaction ve idempotency anahtarı açık.
5. **Üretim güvenlik profili (K-088):** Argon2id, CSPRNG sunucu oturumu,
   rotation/revoke/ömür, rol, synchronizer CSRF, `__Host-` çerez, güvenlik
   başlıkları ve loopback HTTPS reverse-proxy sözleşmesi gerçeklendi.
   Secret dağıtımı, rate limit ve gözlemlenebilirlik deployment sorumluluğudur.

## Kabul kapıları

- **K-087 kapalı:** GET/HEAD ile doğrudan veya dolaylı dosya/çerez/donanım
  durum değişimi derlemede reddedilir (T045).
- **K-087/K-088 kapalı:** yanlış yöntem 405, fazla gövde/alan 413,
  bilinmeyen yol 404 ve son tarih 504'tür. Alan doğrulama 400, kimlik 401,
  yetki/CSRF 403'tür.
- **K-176 kapalı:** eksik/kuralsız `%xx` ve yüzde çözümü sonrası geçersiz
  UTF-8 rota çalışmadan 400'dür; exact uygulama commit'i semantic regression
  fixture'ı ve değişiklik beyanıyla kaynaklanmıştır.
- Yarım yazma K-087 olumsuzlarıyla; CSRF, session fixation, zayıf token,
  süre/iptal, başlık enjeksiyonu ve sahte proxy K-088 olumsuzlarıyla kanıtlıdır.
- **K-087 kapalı:** eylemin aynı iş mantığı web ve CLI bağlamından çağrılır;
  çalışma hatası ve başarısız Sonuç savepoint'i geri alır.
- Tek-dosya atomik durum V1-P0-04/K-084 ile kapandı; eylemin çok-kaynaklı
  transaction/idempotency kapısı bu RFC'de açık kalır.

K-088 ile başlayan production oturum/CSRF/proxy profili K-137/ADR-034'te
süreçler arası ortak kalıcı durum, oran sınırı ve tek-worker/N süreç modeline
genişledi. Secret dağıtımı, çok-hostlu harici backend ve idempotency ayrı
deployment/RFC kapılarıdır; dil bunları varmış gibi göstermez.
