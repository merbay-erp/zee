# RFC-0015 — Uygulama eylemleri ve web güvenlik sınırı

- **Durum:** taslak — sözdizimi sözü değildir
- **Tarih:** 1 Eylül 2026
- **İlgili kararlar:** ADR-010, K-081; v1 kapıları P0-02/P0-03/P0-04
- **Gerçekleme:** yalnız eğitim amaçlı rota/istek/çerez prototipi; production
  eylem modeli yok

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

## Aday yüzey — araştırma notu

Aşağıdaki biçim yalnız tasarım yönünü gösterir; usability ve parser deneyi
olmadan kabul edilmiş syntax değildir:

```text
eylem notu kaydet
    girdi NotGirdisi olarak al
    yetkiyi NotYazma olarak iste
    atomik olarak
        notlara girdinin metnini ekle
    kaydedileni döndür

POST "/notlar" adresine istek geldiğinde
    formu NotGirdisi olarak doğrula
    isteği notu kaydet eylemine ver
```

Eylem HTTP bilmez. Aynı eylem daha sonra CLI, zamanlanmış iş, kuyruk veya
test adaptöründen çağrılabilir. Form bir iş kuralı değil, eyleme adaptördür.

## Capability ve etki modeli

Derleyici bir eylemin etkilerini en az şu sınıflarda görmelidir:

- `salt-okuma`
- `durum-yazma`
- `ağ`
- `kimlik/yetki`
- `transaction`

GET adaptörü yalnız salt-okuma eylemini bağlayabilir. Bu ilk sürümde tam bir
effect type sistemi olmak zorunda değildir; işlem metadata'sı ve kapalı bir
etki kümesi yeterlidir. Etki çıkarımı public eylem sınırında açık sözleşmeye
dönüşür.

## Aşamalı gerçekleme

1. **Korkuluk:** mevcut TCP/web yüzeyi açık `--deneysel-web` opt-in'i olmadan
   gerçek soket açmaz; örnekler production olmadığını söyler ve bütün durum
   değişiklikleri POST kontrolü taşır.
2. **Protokol sınırı:** yöntemli route, gövde/başlık sınırı, güvenli çerez
   seçenekleri, istek deadline'ı ve kontrollü reverse-proxy güveni.
3. **Eylem:** typed girdi doğrulama, authz ve form/API/CLI adaptörlerinden
   bağımsız çağrı.
4. **Durum:** atomik dosya değiştirme + veri tabanı transaction capability'si;
   idempotency anahtarı ve rollback testleri.
5. **Üretim profili:** TLS sonlandırma sözleşmesi, secret yönetimi, rate limit,
   güvenlik başlıkları, gözlemlenebilirlik ve saldırı conformance paketi.

## Kabul kapıları

- GET ile dosya/DB/çerez durum değişimi derlemede reddedilir.
- Yanlış yöntem 405, fazla gövde 413, doğrulama 400, kimlik 401 ve yetki 403
  olarak ayırt edilir; uygulama bunları ham metinle tahmin etmez.
- CSRF, session fixation, zayıf token, çift gönderim ve yarım yazma için
  olumsuz testler vardır.
- Eylemin aynı saf iş mantığı web ve CLI adaptöründen çağrılır.
- Atomik durum kapısı V1-P0-04 ile birlikte kapanır.

Bu kapılar tamamlanana kadar zee “TCP üzerinde eğitim/prototip web yüzeyi”
sağlar; “production web framework” sözü vermez.
