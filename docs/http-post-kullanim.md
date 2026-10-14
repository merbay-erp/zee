# HTTP POST ve Turnstile kullanımı (K-220)

Başlık ve gövde taşıyan HTTPS GET/POST, milisaniye cinsinden süre ve `Sonuç`
döndürür. [Karar ve sınırlar](../adr/091-http-post-baslik-ve-gizli-ortam.md),
[testli Turnstile helper’ı](../dogfood/zee-cekirdek/http-post/turnstile.dil).

## Proje izni

Anahtarın **adı** proje bildirimine, değeri sunucu ortamına girer:

```zee
proje "http-deneme" olsun
sürüm "0.1.0" olsun
giriş "ana.dil" olsun
yetkinlikler "ağ" listesi olsun
ağ_hedefleri "https://challenges.cloudflare.com" listesi olsun
http_gizli_değişkenleri "TURNSTILE_SECRET" listesi olsun
yerel_bağımlılıklar boş liste olsun
```

`turnstile.dil` helper’ını `ana.dil` ile aynı klasöre koyun. Dışa açık
işlemler parametre ve `Metin döndürür` sözleşmesini taşır. Aşağıdaki ana
kaynak, proje olarak `dil denetle .` ile doğrulanmıştır:

```zee
turnstile birimini kullan
jeton "YER_TUTUCU" olsun
ip "203.0.113.7" olsun
konak "site.example" olsun
durum jeton ve ip ve konak ile insan doğrula olsun
durum yaz
```

Handler içinde jetonu form alanından, IP’yi `istemci adresi` ifadesinden ve
beklenen hostname’i sitenin yapılandırmasından alın. `gecti` kayıt/bildirim
adımına izin verir; `gecersiz` reddedilir; `erisilemedi` durumunda mesaj
kaydedilmez ve ziyaretçiden sonra tekrar denemesi istenir. Doğrulamayı
veritabanı eyleminden önce yapın; HTTP POST geri alınamaz etki taşır.
Canlı rotanın entegrasyonu ve sunucu ikilisi yükseltmesi ürün tarafındadır.

## Doğrudan HTTP çağrısı

```zee
başlıklar boş sözlük olsun
başlıkların "Content-Type" değeri "application/json" olsun
gövde verinin json metni olsun
sonuç adres "POST" başlıklar gövde 5000 ile HTTP isteğini dene olsun
sonuç başarılıysa
    yanıt sonucun değeri olsun
    durum yanıtın "durum" değeri olsun
    metin yanıtın "gövde" değeri olsun
```

Burada `adres` Metin, `veri` mevcut sözlüktür. Başlıkların değerleri Metin
olmalıdır. Bearer kullanımında ortamdan başarılı `Sonuç` ile okunan jetonu
`Authorization` başlığına koyun ve o portalın exact HTTPS origin’ini açın.
HTTP 4xx/5xx taşıma yanıtı olarak gelir; durum metnini ayrıca değerlendirin.
Taşıma/izin/biçim/boyut hatası başarısız `Sonuç`tur.

HTTP JSON helper’ında değerler kanonik JSON metnidir: bool true `true`,
metin true `"true"` olarak ayrılır. Turnstile helper’ı gerçek bool ve beklenen
hostname’i birlikte denetler. Sır içeren HTTP/ortam işlemi iz kaydında
yürütülmez; bu programlarda ham sır veya yanıtı kendi günlüklerinize yazmayın.
