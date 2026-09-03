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

Transaction ile geri alınamayan ekran çıktısı, kullanıcı girdisi ve donanım
eyleyicisi de eylem içinde kullanılamaz (T048). Bunlar CLI, web, görev veya
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

Bu sözleşme yorumlayıcı tarafından gözlenen hata/başarısız sonuç için çok
dosyalı geri almadır. Süreç ya da makine tam eylemin ortasında çökerse bütün
dosyaları tek bir kalıcı commit olarak yayınlama sözü vermez; çökme atomikliği
dosya başına spec/08'deki K-084 sözüdür. Veritabanı/dağıtık kaynak ACID'i ayrı
capability, uzun ömürlü kilitleme ve günlükleme kararı gerektirir.

## 7. Güvenlik sınırı

K-087 rota/eylem ayrımını, yöntem güvenliğini, kaynak limitini ve yerel
transaction sözleşmesini kurar. Kimlik doğrulama, yetkilendirme, CSRF,
güvenli oturum/çerez ve TLS/proxy güveni K-088 ile
[spec/12](12-web-guvenlik-profili.md)'de tanımlıdır. Idempotency anahtarı
ayrı açık kapıdır.
Bu nedenle gerçek TCP yüzeyi `--deneysel-web` açık seçimini korur; bu bölüm tek
başına “production web framework” sözü değildir.
