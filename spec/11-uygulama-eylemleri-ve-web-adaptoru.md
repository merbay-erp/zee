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

### Dosya yanıtı — K-378

Normatif kaynak: [RFC-0041](../rfcs/0041-ikili-yanit-ve-ek-akisi.md) katman A,
[ADR-163](../adr/163-ikili-yanit-multipart-ve-ek-turu-kaydi.md). Rota proje
içindeki bir dosyayı ek (indirme) olarak gönderebilir:

```text
GET "/ek" adresine istek geldiğinde
    oturum gerekli
    sonuç yol dosyasını "application/pdf" türünde "fatura.pdf" adıyla ek olarak göndermeyi dene olsun
    sonuç başarısızsa
        "Ek bulunamadı." yanıtını 404 durumuyla gönder
```

- Kalıp `TamSayı sonucu` döndürür (gönderilecek bayt) ve `dosya-okuma` ile
  `ağ-sunucusu` yetkinliklerini ister; eylem içinde T044, rota dışında başarısız
  Sonuç'tur. Yol, tür ya da ad reddi yanıt başlığı yazılmadan **C013** başarısız
  Sonuç'tur; rota bunu yakalayıp kendi durum yanıtını verebilir. Dosya yanıtı
  isteğin ilk yanıtıdır: yanıtı zaten belirlenmiş istekte de başarısız Sonuç
  olur ve önceki yanıt değişmez. Bu kurallar TANIMLIDIR.
- İçerik türü yalnız kapalı `icerik_turu` kaydının dosya sınıfından exact,
  küçük harf, parametresiz yazımla seçilmek ZORUNDADIR: `application/pdf`,
  `image/png`, `image/jpeg`, `image/webp`, `text/plain`, `text/csv`,
  `text/calendar`, `application/zip`, üç OOXML türü (docx/xlsx/pptx),
  `application/xml`, `application/x-apple-aspen-config` ve
  `application/octet-stream`. `text/html`, SVG, XHTML ve betik türleri dosya
  sınıfında YASAKTIR; ürün bu dosyaları `application/octet-stream` ile verir.
  `text/*` türlerine çalışma zamanı `charset=utf-8` ekler.
- K-385 (ADR-163 eki): `application/xml` (e-posta istemcisi otomatik
  yapılandırma XML'i) ve `application/x-apple-aspen-config` (Apple
  yapılandırma profili) yalnız web dosya sınıfındadır; SMTP eki olarak
  seçilemez ve dosya gövdesi ayrıştırılmaz. `text/xml` takma adı ile
  `+xml` sonekli öteki türler kayıtta yoktur. Bu iki türe `charset`
  parametresi eklenmez; `Content-Disposition`, `nosniff` ve sandbox CSP
  öteki dosya türleriyle aynıdır. Bu kurallar TANIMLIDIR.
- Başlık değerlerini yalnız çalışma zamanı kurar: `Content-Disposition:
  attachment; filename="<ASCII geri düşüş>"; filename*=UTF-8''<RFC 8187>`.
  Boş, `.`/`..`, yol ayracı, NUL/CR/LF dahil denetim karakteri, çift tırnak,
  Unicode yön denetimi (U+061C, U+200E/F, U+202A–E, U+2066–9) taşıyan ya da 255
  baytı aşan ad YASAKTIR. Ad yoldan türetilmez. `inline` biçimi, `ETag`,
  `Range` ve koşullu GET bu sürümde AÇIK sınırdır.
- Dosya yanıtı her zaman `X-Content-Type-Options: nosniff`, `Cache-Control:
  no-store`, `Content-Security-Policy: default-src 'none'; sandbox` ve
  `Cross-Origin-Resource-Policy: same-origin` taşımak ZORUNDADIR; rota önbellek
  izni (K-335) dosya yanıtına uygulanmaz.
- Yol spec/26 proje dosya sınırıyla çözülür (mutlak yol, `..`, kök dışına
  çıkan bağ reddi); son bileşen sembolik bağ, dizin, FIFO ya da normal olmayan
  dosya olamaz ve dosya readonly/`O_NOFOLLOW` açılır. Çağrı anındaki uzunluk
  32 MiB'ı (K-401; önce 16 MiB `yukleme_bayti`) aşarsa C013'tür; tam
  32 MiB kabul edilir. Gövde 16 KiB parçalarla akar,
  belleğe alınmaz ve heap zarfına girmez; yazım K-372 mutlak bütçesine
  (`10 sn + bayt / 128 KiB/s`, 10 sn ilerlemesizlik) tabidir. Akış sırasında
  dosya kısalır ya da son parçada uzunluğu değişirse bağlantı eksik
  `Content-Length` ile kesilir; bu başarı sayılmaz. HEAD rotası aynı başlıkları
  gövdesiz verir. Bu davranış ZORUNLUDUR.
- IO izi `dosya_yaniti_gonder` olayına yol, tür, ad ve bayt sonucunu yazar;
  dosya içeriği ize girmez. Hermetik adaptör (`dil dene`) aynı kayıt, ad ve
  tavan kuralını uygular.

### Türlü metin yanıtı — K-406

Normatif kaynak: [ADR-163](../adr/163-ikili-yanit-multipart-ve-ek-turu-kaydi.md)
K-406 eki. Rota bellekteki bir `Metin`i kayıtlı bir metin türüyle (ör. takvim
yayını) gönderebilir:

```text
GET "/takvim/yayin.ics" adresine istek geldiğinde
    herkese açık
    takvim metnini "text/calendar" türünde gönder
```

- ZORUNLU (K-406): `<ifade> metnini "<tür>" türünde gönder` cümlesinde tür
  kaynakta görünen tırnaklı Metin literalidir ve kapalı `icerik_turu`
  kaydının web metin satırlarından biriyle exact, küçük harf, parametresiz
  eşleşmek ZORUNDADIR: `text/plain`, `text/csv`, `text/calendar`,
  `text/vcard` (RFC-0047), `application/xml`. Değişken/hesaplanmış tür, boş
  tür, öteki kayıt satırları (ikili türler, `application/octet-stream`,
  `application/x-apple-aspen-config`), kayıt dışı türler ve `text/html`, SVG,
  XHTML, betik türleri derlemede önerili **S037**'dir. Durum eki olmayan
  biçimin durumu 200'dür; durumlu kardeşi aşağıdaki RFC-0047 bölümündedir.
- ZORUNLU (K-406): Yanıt dosya yanıtının başlık kümesini taşır:
  `Content-Type` (metin türlerine `charset=utf-8`, `application/xml`e
  parametre eklenmez), `X-Content-Type-Options: nosniff`, `Cache-Control:
  no-store`, `Content-Security-Policy: default-src 'none'; sandbox` ve
  `Cross-Origin-Resource-Policy: same-origin`. `Content-Disposition` yazılmaz
  (`attachment` biçimi AÇIK sınırdır); rota önbellek izni (K-335) uygulanmaz.
  Gövdenin `<` ile başlaması türü `text/html`e çevirmez. HEAD aynı başlıkları
  gövdesiz verir. Çerez taslağı öteki 200 yanıtlarındaki gibi yazılır.
- ZORUNLU (K-406): Gövde UTF-8 baytlarıyla olduğu gibi gider; CR, LF ve CRLF
  dönüştürülmez, satır sonu eklenmez ya da normalleştirilmez. RFC 5545/4180
  CRLF satır sonlarını ürün kurar; CR karakteri kaçışla yazılamadığından
  K-402 `13 kod noktası harfini almayı dene` yüzeyiyle ya da istek/dosya
  verisinden gelir. Gövde K-372 yanıt yazım bütçesine ve spec/24 metin
  sınırına tabidir.
- TANIMLI (K-406): Cümle öteki yanıt cümleleri gibi yalnız rota gövdesinde
  HTTP etkisi taşır, eylem içinde T044'tür, `ağ-sunucusu` yetkinliği ister ve
  K-403 uyarınca rotayı bitirir (`dil denetle --lint` sonraki cümleye S047).
  IO izi `turlu_yanit_gonder` olayına tür ve gövdeyi yazar; iz şeması ve
  hermetik adaptör (`dil dene`) aynı kapalı kümeyi uygular, kayıt dışı tür
  içeren iz okunmaz ve adaptöre doğrudan verilen kayıt dışı tür 500 durum
  yanıtına düşer (HTML koklamasına açılmaz).

### Durumlu türlü yanıt, 405 + `Allow` ve 501 — RFC-0047 dilim A

Normatif kaynak: [ADR-182](../adr/182-dav-istemci-rotasi-ilk-dilim.md)
(K-364, RFC-0047 A ve dar C). DAV multistatus 207 + `application/xml` ister;
kural her rota sınıfında geçerlidir:

```text
GET "/dav/" önekli adrese istek geldiğinde
    herkese açık
    xml metnini "application/xml" türünde 207 durumuyla gönder
```

- ZORUNLU (RFC-0047 C): `<ifade> metnini "<tür>" türünde <durum> durumuyla
  gönder` cümlesinde tür K-406 web metin satırlarından biri, durum kaynakta
  görünen bir `TamSayı` literali olmak ve kapalı kümeden seçilmek
  ZORUNDADIR: 200, 201, 207, 400, 403, 404, 409, 412, 415. Kayıt dışı
  durum (204, 3xx, 5xx dahil), değişken ya da tırnaklı durum derlemede
  önerili **S037**'dir. Başlık kümesi, gövdenin bayt bayt gitmesi, rota sonu
  (K-403), T044 ve yetkinlik kuralı durumsuz türlü yanıtla aynıdır; durum
  satırı `HTTP/1.1 <durum> <kanonik açıklama>` olur.
- ZORUNLU (RFC-0047 A): Yol bir ya da daha çok rotayla eşleşip istek
  yöntemi hiçbiriyle eşleşmezse yanıt 405'tir ve `Allow` başlığı o yolda
  tanımlı yöntemleri kanonik kayıt sırasıyla (`GET, HEAD, POST, PUT, PATCH,
  DELETE` alt kümesi), virgül ve tek boşlukla, tekrarsız taşır (RFC 9110
  §15.5.6). Kural her rota sınıfında (oturumlu, herkese açık, makine ucu,
  önbelleklenebilir) aynıdır. Yöntemsiz tarihsel rota `GET` sayılır. Yolda
  GET rotası varsa `Allow` HEAD'i de taşır. Hiçbir yol eşleşmezse yanıt
  404'tür ve `Allow` taşımaz.
- ZORUNLU (K-421, RFC 9110 §9.3.2): GET'e izin veren yol HEAD'i örtük olarak
  destekler. HEAD isteği için o yolda açık HEAD rotası yoksa ve GET rotası
  (yöntemsiz tarihsel, önekli ve dosya/türlü yanıtlı rotalar dahil) varsa,
  istek GET rotasıyla eşleşir. Rota GET ile aynı durum ve başlıkları üretir,
  gerçek adaptör gövdeyi yazmaz. Açık HEAD rotası önceliklidir. GET rotası
  olmayan yolda (ör. yalnız POST) HEAD 405 + `Allow` alır.
- ZORUNLU (RFC-0047 A): İstek yöntemi kapalı yöntem kaydında
  (`HttpYontemi::KAYIT`; dilim C'den beri `PROPFIND` ve `REPORT` dahil)
  yoksa (ör. `LOCK`, `MKCALENDAR`, `PROPPATCH`, `MOVE`; DAV öneki dışında
  `OPTIONS`) yanıt yoldan bağımsız 501'dir
  (RFC 9110 §15.6.2); rota eşleştirmesi, `_csrf` denetimi ve rota gövdesi
  hiç çalışmaz, `Allow` yazılmaz. Yöntem karşılaştırması bugünkü gibi büyük
  harfe çevrilmiş yazımla yapılır.
- ZORUNLU (RFC-0047 C): Gerçek adaptörün durum satırı 200 `OK`, 201
  `Created`, 204 `No Content`, 207 `Multi-Status`, 409 `Conflict`, 412
  `Precondition Failed`, 415 `Unsupported Media Type` ve 501 `Not
  Implemented` açıklamalarını taşır (öncekiler değişmez; tabloda olmayan
  kod `Error` kalır). 204 yanıtı gövde, `Content-Type` ve `Content-Length`
  taşımaz (RFC 9110 §6.4.1, §8.6); programın verdiği gövde yazılmaz.
- TANIMLI (RFC-0047): IO izi durumlu türlü yanıtı `turlu_durumlu_yanit_gonder`
  olayına (tür, durum, gövde) yazar; iz şeması ve hermetik adaptör (`dil
  dene`) aynı kapalı kümeleri uygular, adaptöre doğrudan verilen kayıt dışı
  tür ya da durum 500 durum yanıtına düşer. 405 olayı `durum_yaniti_gonder`
  (405, gövde) olarak kalır: `Allow` program ve yoldan deterministik
  türediği için ize girmez, eski izler aynı biçimde yeniden oynar.

### DAV istemci rotası — RFC-0047 dilim C

Normatif kaynak: [ADR-182](../adr/182-dav-istemci-rotasi-ilk-dilim.md)
dilim C eki; güvenlik sözleşmesi spec/12 §RFC-0047 dilim C.

```text
PROPFIND "/dav/" önekli adrese istek geldiğinde
    dav istemcisi gerekli
    kim oturumun "kullanıcı" değeri olsun
    xml metnini "application/xml" türünde 207 durumuyla gönder
```

- ZORUNLU (RFC-0047 F): `dav istemcisi gerekli` rota önsözünün ilk ve tek
  politika cümlesidir (T049/T050) ve yalnız GET, HEAD, PROPFIND, REPORT,
  PUT ve DELETE rotalarında, sabit (tam ya da önekli) Metin yolla yazılır.
  POST/PATCH rotası ya da sabit olmayan yol derlemede önerili **T065**'tir.
  `PROPFIND` ve `REPORT` yalnız bu politikayla yazılır; başka politika,
  politikasız ya da makine ucu birleşimi **T064**'tür. İki yöntem salt okuma
  sınıfındadır (T045). Politika `ağ-sunucusu` ve `kriptografi` yetkinliği
  ister.
- ZORUNLU (RFC-0047 F): DAV rotası gövdesi ya da geçişli çağırdığı
  işlemler çerez yazma/silme, `oturuma al`, `oturumu kapat`, `csrf
  belirteci` ve web oturumu listeleme/iptal yüzeyini kullanamaz (**T066**,
  ilk dokunuşun satırı) ve `yeni uygulama parolası` üretemez (T063).
  Çalışma zamanı rotaya boş `çerezler` sözlüğü verir; `oturum` yalnız
  kimlik kararından dolar: `kullanıcı` (çözücünün kanonik adı), `rol`,
  `kimlik_türü` = `uygulama_parolası`, `uygulama_parolası_kimliği`,
  `kapsam`. `_csrf` aranmaz.
- ZORUNLU (RFC-0047 E): Proje kipinde DAV rotası olan program
  `dav_kimlik_çözücüsü` bildirir. Çözücü iki parametreli, değer döndüren
  bir `işlem`dir (eylem değil) ve geçişli olarak durum, web ya da geri
  alınamaz etki taşımaz; aksi ve bildirimsiz/tanımsız çözücü önerili
  **T067**'dir (`dil denetle`, `dil çalıştır`, `dil dene`). Çözücü her
  istekte (önbelleksiz) ASCII küçük harfe katlanmış kullanıcı adı ve
  uygulama parolası önekiyle çağrılır; `özet`, `kullanıcı`, `rol`,
  `kapsam`, `kimlik` Metin alanlı sözlük döndürür, alanı eksik sonuç
  "kayıt yok" sayılır.
- ZORUNLU (RFC-0047 A, soru 4): DAV rotasının yolu altındaki `OPTIONS`
  isteği kimlik ve rota çalışmadan 200 ile yanıtlanır: `Allow` o yolun
  yöntemleri (kanonik sırayla) ve `OPTIONS`, `DAV: calendar-access,
  addressbook`; CORS başlığı yazılmaz. DAV yolu olmayan yerde `OPTIONS`
  501'dir.
- AÇIK (dilim D): DAV istek gövdesi bu dilimde okunur ve atılır (64 KiB
  tavanı); `isteğin ham gövdesi` DAV rotasında hâlâ T062'dir. Başlık okuma
  (`Depth`, `If-Match`, `If-None-Match`) ve `ETag` yazımı dilim D'dedir.
- TANIMLI (RFC-0047 §8): IO izi kimlik kararını `dav_kimligi` (kullanıcı,
  önek, çözücü adı), `dav_ozet_dogrula` (kayıttaki özet → sonuç) ve
  `dav_kovasi` (kova, anahtar → karar) olaylarıyla kaydeder; 401, 429 ve
  OPTIONS yanıtları `durum_yaniti_gonder` biçimindedir. `Authorization`
  değeri, gizli kısım ve HMAC anahtarı ize girmez. Hermetik adaptör
  (`ToplayanIo.dav_basic`, `dene` anahtarı) aynı kararı verir.

Desteklenen yöntemler `GET`, `HEAD`, `POST`, `PUT`, `PATCH`, `DELETE`; DAV
rotasında ayrıca `PROPFIND` ve `REPORT`tir.
Yöntemsiz tarihsel rota yazımı geriye uyum için yalnız GET'e bağlanır; yeni kod
yöntemi açıkça yazar. `istek` sözlüğündeki `"yöntem"` alanı gözlem amacıyla
korunur ancak yönlendirme/yetki kararı değildir.

ZORUNLU (RFC-0047 E, ADR-182 dilim B): `yeni uygulama parolası` ifadesi bir
rota gövdesinde ya da rotanın geçişli çağırdığı bir işlemde yalnız rota önsözü
`oturum gerekli` ya da `"<rol>" yetkisi gerekli` olan ve yöntemi POST, PUT,
PATCH ya da DELETE olan (CSRF korumalı) rotada kullanılabilir. Herkese açık
rota (makine uç noktası ve önbelleklenebilir biçimler dahil) ile GET/HEAD
rotası derlemede önerili **T063** alır; rota dışı kod (CLI, `dil çalıştır`)
serbesttir.

GET ve HEAD salt okumadır. Doğrudan çerez/dosya/donanım yazması veya çağrı
grafiğinde ulaşılabilen yazıcı eylem T045 ile derlemede reddedilir. Aynı
yöntem+yol çifti iki kez tanımlanamaz (T047). HEAD gerçek TCP adaptöründe GET
ile aynı başlık hesabını yapar ama yanıt gövdesi göndermez; açık HEAD rotası
olmayan yolda GET rotasıyla örtük eşleşir (K-421).

Yol eşleşip yöntem eşleşmezse 405 + `Allow`, yol eşleşmezse 404, kayıt dışı
yöntem yoldan bağımsız 501 döner (RFC-0047 dilim A). PUT/PATCH/DELETE
form gövdesi POST ile aynı sınırlı alan çözümlemesini kullanır. Makine uç noktasının
(spec/12 §K-405) gövdesi form olarak çözülmez; rota onu `isteğin ham gövdesi`
ile en çok 8 KiB katı UTF-8 `Metin` olarak okur.

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
yenilemez. Yanıt yazımı da mutlak bütçeyle sınırlıdır (K-372): yazım
başladığı andan en çok `10 sn + yanıt baytı / 128 KiB/s` sürer ve 10 saniye
boyunca tek bayt ilerlemeyen yazım kesilir; kısmi ilerleme bütçeyi yenilemez.
Ham red yanıtından sonraki istek artığı boşaltması (aşağıda, K-374) bu
bütçeden sonra en çok 2 saniye ve 16 MiB ekler. Bu limitleri büyüten kaynak
sözdizimi v1'de yoktur.

ZORUNLU (K-391 Dilim 2, ADR-168): `geniş_gövde_rotaları` ve
`geniş_gövde_sınırı` birlikte yazan projede POST/PUT/PATCH/DELETE isteğinin
sorgusuz yolu listedeki bir ögeyle (`/` ile biten öge önek, diğeri tam yol)
eşleşirse urlencoded gövde sınırı ve multipart metin bütçesi
`geniş_gövde_sınırı` bayttır. Eşleşme başlıktan, gövde okunmadan yapılır;
`Content-Length` zarfı bu bütçeyle denetlenir. Sınır 65 537 ile 1 MiB
arasındadır, liste 1–32 benzersiz geçerli yol taşır; ikisinden biri eksik ya
da aralık dışıysa P022'dir. Eşleşmeyen yol, GET/HEAD, alan sayısı (100) ve
10 saniyelik urlencoded okuma süresi değişmez. Hermetik adaptörde rota bazlı
bütçe AÇIKTIR.

ZORUNLU (K-391, [ADR-168](../adr/168-istek-boyut-sinirlari-ve-govde-asimi.md)):
`proje.dil` içinde `gövde_aşımı_rotası "/yol" olsun` yazan projede gerçek TCP
adaptörü, POST/PUT/PATCH/DELETE isteğini gövde boyutu yüzünden reddettiğinde
(bildirilen `Content-Length` zarfı aşar; multipart dosya ya da metin bütçesi
akış sırasında aşılır) ham 413 yerine aynı bağlantıda o yolu `GET` isteği
olarak, gövdesiz ve isteğin çerez/oturumuyla yönlendirir. Rota
`govde_asimi_yolu` (sorgusuz özgün yol), `govde_asimi_siniri` ve
`govde_asimi_bayti` sorgu alanlarını görür; bu alanlar güvenilmezdir. Rotanın
200 metin/HTML yanıtı `413 Payload Too Large` durumu ve `Cache-Control:
no-store` ile yazılır; rota önbellek izni uygulanmaz. Yönlendirme, dosya, 404
ya da başka durum düz 413 iletisine döner. Yanıttan sonra yukarıdaki K-374
boşaltması uygulanır. Özgün rotanın gövdesiz ya da kısmi gövdeyle
çalıştırılması YASAKTIR (CSRF belirteci okunmamış gövdededir). GET ve HEAD
isteğinin aşımı, alan sayısı (100) aşımı ve bildirimsiz projede 413 ham
kalır. Yol `/` ile başlar, en çok 256 bayttır, sorgu/parça/yüzde/boşluk ya da
denetim karakteri taşımaz; aksi P022'dir. Hermetik `dil dene` adaptöründe bayt
düzeyi zarf olmadığından aşım rotası AÇIKTIR.

ZORUNLU: Gerçek TCP adaptörü ham red yanıtı (413, 431, 400, 408, 403, 429,
503 ve benzeri; çoğu gövde ya da başlığın kalanı okunmadan verilir) yazdıktan
sonra bağlantıyı doğrudan kapatmaz: önce yalnız yazma yönünü kapatır (yanıt
ve FIN istemciye gider), ardından istemcinin gönderdiklerini en çok 2 saniye
ve en çok 16 MiB (yükleme sınırı) boyunca okuyup atar, EOF'ta, okuma
hatasında ya da bütçe dolunca kapatır. Okunmamış istek baytıyla kapanış
çekirdeğin RST göndermesine ve istemcinin yanıtı hiç görememesine yol açar;
bu YASAKTIR. Boşaltma bütçesi mutlaktır: bayt damlatan ya da bağlantıyı
kapatmayan istemci işçiyi yanıt yazımından sonra 2 saniyeden uzun tutamaz
(K-374).

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

ZORUNLU (K-403): Rota gövdesi koşarken `yanıtını gönder` (sabit durumlu
biçimi dahil) ya da `yönlendir` yanıt taslağını başarıyla belirlediği anda
rota gövdesi SONLANIR: aynı bloktaki, çevreleyen `ise`/`göre`/döngü/
`içinde` bloklarındaki ve çağrı yığınındaki işlemlerin sonraki cümleleri
koşmak YASAKTIR. Bu olağan bitiştir; yanıt, çerez ve oturum taslağı başarı
yolundaki gibi commit edilir, tanı ya da IO izi olayı üretilmez. Böylece
`yetkili değilse` → `"/giris" adresine yönlendir` biçimindeki koruma
fail-closed çalışır; korumadan sonraki iş (çerez/oturum/veritabanı
mutation'ı, eylem, çıktı, giden istek) koşmaz. Yanıttan önce oluşan tanı,
reddedilen yönlendirme (C022) ve yanıtsız rota bu maddeden önceki gibi
işlenir. Rota dışında (üst düzey program, `test` bloğu) yanıt cümlesi akışı
bitirmez. Dosya yanıtı (`… ek olarak göndermeyi dene`) bir `Sonuç`
ifadesidir ve rotayı bitirmez; çağıran başarı/başarısızlık dalını kendisi
seçer, başarıdan sonraki yanıt cümlesi rotayı bitirir ama yanıtı değiştirmez. İç uygulama
`Ç004` nöbetçisini kullanır; nöbetçi rota yürütücüsü dışına sızmaz.
`dil denetle --lint` rota gövdesinde kesin yanıttan sonraki ulaşılamaz
cümleyi önerili S047 ile gösterir.

K-324/ADR-157 uyarınca rota yürütmesi bir tanıyla biterse istek
taslağı geri alınmak ve worker sonraki bağımsız isteği kabul etmek ZORUNDADIR;
rota hatası süreci SONLANDIRAMAZ. İstemciye tanı kodu, mesajı, satırı ya da
değer içeriği SIZDIRILAMAZ. Yanıt tanı kodunun saf işlevidir: Ç001 504; C013,
C021, C027 bu bölümdeki 503 metinleri; C023/C024 spec/24'teki 503 metni; diğer
bütün C/D tanıları “istek işlenemedi; sunucu hatası” gövdeli 500'dür. Ret
kümesi yalnız iç tutarlılıktır: C000 ve iç Ç nöbetçileri süreci tanıyla
bitirir. Kurtarma rota gövdesini, eylemi ya da yazımı otomatik TEKRARLAYAMAZ.
Gövde bittikten sonra yanıt commit'inin (`istek_islemini_tamamla`) yalnız
iç tutarlılık hatası (bekleyen bağlantının yokluğu) ve socket kopmasından
sonra web oturum commit'inin geri alınamaması C022 ile süreci bitirir; socket
yazım hatası bu istisnaya GİREMEZ (aşağıdaki K-327 maddesi). `programı N ile
bitir` rotada sunucuyu kapatmaya devam eder; geçersiz N (C020) 500'dür.

K-327 uyarınca “worker bu istek yüzünden SONLANMAMALI” ilkesi yanıtın socket'e
yazımını da kapsar: istemci ya da önündeki proxy yanıt yazılırken bağlantıyı
keserse (EPIPE, ECONNRESET, K-372 yazım bütçesinin ya da ilerlemesizlik
süresinin dolması ve diğer socket
yazım hataları) hata istek kapsamlı kalmak ve worker sonraki bağımsız isteği
kabul etmek ZORUNDADIR. Yanıt taslağının çerez ve oturum commit'i geri alınır
(istemci yanıtı almadı); PostgreSQL istek transaction'ı bu noktada zaten
tamamlanmıştır ve geri alınmaz. Bağlantı izni ve socket aynı istekte iade
edilir. Operatör stderr'de tek satırlık `zee-web istemci yanıt sırasında
koptu: durum=… yöntem=… yol="…" hata="…"` kaydını görür (yol sorgusuz,
kaçışlı, 4 KiB sınırlı) ve native metrikte kopma `final_write_errors`
(`yanit_yazilamadi`) sayacına düşer. Kopma tanı değildir; IO izine GİREMEZ ve
yazım otomatik TEKRARLANAMAZ.

İstek düzeyinde yanıta çevrilen her tanı, yanıt gönderilmeden önce yalnız
operatöre görünen kanala verilmek ZORUNDADIR (spec/06'daki sessiz devam yasağı).
Gerçek CLI adaptörü tek satırlık `zee-web istek-hatası durum=… kod=… satır=…
yöntem=… yol="…" mesaj="…"` kaydını stderr'e yazar; yol sorgu/parça dizgesi
olmadan yazılır, yol ve mesajdaki tırnak, ters bölü ve denetim karakterleri
kaçışlanır, satır 4 KiB ile sınırlıdır. Bu kayıt programa görünmez ve
deterministik IO izine GİREMEZ.

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

ZORUNLU (K-346): Gerçek adaptör eylem yedeğini ve eylemin hedefte bıraktığı
içeriği, yazım ya da silmeyle aynı süreçler-arası kilit penceresinde yakalar.
Kilit dışında okunan yedek, başka sürecin araya giren başarılı eklemesini geri
almada silebilir. Aynı savepoint'in iki dokunuşu arasında hedef başka yazarca
değişmişse geri alma eylem başı içeriğini geri koyamaz; C021 ile başarısız
olur ve araya giren yazarın verisi kalır.

ZORUNLU (K-347): Eylem içindeki atomik taşıma kaynak ve hedef klasörlerinin
yazma kilitlerini birlikte tutar (aynı klasörde tek kilit, ayrı klasörlerde
kanonik klasör yolu sırasıyla) ve taşınan içeriği adlandırmayla aynı kilit
penceresinde yakalar; savepoint kaynağa daha önce dokunduysa ve kaynak o
dokunuştan sonra başka yazarca değiştiyse kayıt çatışmalıdır. Geri alma bir
savepoint'in herhangi bir kaydı çatışmalıysa hiçbir yolu değiştirmeden C021
verir; aksi hâlde önce içerik geri koymalarını, sonra silmeleri yapar, böylece
kısmi geri alma taşınan içeriği kaybettirmez.

Bir web isteğinde eylem transaction'ının başlaması, tamamlanması veya geri
alınması C021 üretirse istek yanıtı/çerez/oturum taslağı geri alınmalı, istemciye
503 verilmelidir. Worker/process bu istek yüzünden SONLANMAMALI ve sonraki
bağımsız isteği kabul etmelidir; bu ilke 503 yanıtının socket'e yazımını da
kapsar (§5 K-327 maddesi). Bu recovery eylem gövdesini veya write'ı
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

TANIMLI: Credential launcher --native-metrics-fd tek metrik profiliyle designated write-only metrik FIFO capability aktarır. Native --web-metrik-fd kaynak öncesi aynı descriptor rolünü belirtir; launcher credential/source okumadan doğrular ve spawn sonrası kendi kopyasını kapatır. Ambient FD ve varsayılan noinherit korunur; bu profil Windows/systemd veya tam P13/P25 ürün kabulü değildir.

## Sahipli Unix listener ve nesil hazırlığı — K-259

TANIMLI: Unix native --web-listener-fd, --web-ready-fd, --web-activate-fd ve --web-generation birlikte sahipli inherited listener profilidir. Listener exact loopback IPv4 TCP LISTEN port ve nonblocking rol taşır; native yalnız kendi duplicate sahibidir, fallback bind yapmaz. Kaynak/config tanısından önce quiet preflight stdio ve roller arası aliası reddeder; Darwin pipe peer karşılaştırması yalnız kendi PID descriptor metadata sınırındadır.

ZORUNLU: Ready kaydı zee-native-listener-ready-1 şemasında exact native PID, fresh generation, build kimliği ve port taşır. Host ayrı generation-owned application receipt doğrular; exact ACTIVATE generation ve EOF gelmeden accept başlamaz. Activation okuması nonblocking ve süre sınırlıdır; EOF veya hata admission açmaz. Host leader rezervasyonunu süreç grubu cleanup bitene dek tutar.

TANIMLI: Inherited listener profilinde successful accept admission noktasıdır; accept sonrasındaki SIGTERM yarışı kabul edilen isteğin mevcut bütçelerle tamamlanmasını engellemez. Varsayılan direct bind profili K-253 kapanış sınırını korur. Ortak canonical atomic JSON quota/session/revoke durumu overlap ve restart boyunca kullanılır; süreç belleği kota deposu bu profilde reddedilir.

TANIMLI: Inherited listener ile optional native metrik capability aynı kaynak öncesi çağrıda kompoze edilir. Metrik readerının önceden kapanması hizmeti durdurmaz; listener/ready/activate kontrol rolleri canlı peer ve exact rol ister. Typed credential listener profili listener/ready/activate/startup FD, generation, worker-port ve binary hash alanlarını birlikte kaynak öncesi exact bağlar; arbitrary FD listesi vermez. Borrowed rol/alias/socket/port denetimi secret okumadan ve spawn öncesi tekrarlanır. Yalnız launcher actual native child PID/generation/binary hash kaydını ayrı startup FIFOya tek sınırlı yazımla bırakır; startup FD native childa devredilmez. Host bu kaydı native ready ile eşler ve yalnız kendi launcher Popen/held process group yetkisini tutar; native PID edinmez. Varsayılan noinherit ve secret-masked stdio korunur. Pilot aynı frozen uygulama revisionıyla sınırlıdır; genel deployment atomicity ve ürün adoptionı AÇIK kalır.
