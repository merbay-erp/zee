# RFC-0047 — DAV istemci rotası: RFC-0040'ın ilk dilimi (A + dar B + dar C + E + F)

- **Durum:** taslak — 27 Kasım 2026. RFC-0040'ın ilk alt RFC'sidir. Core
  freeze (K-160A/ADR-059) ve V1 syntax freeze altında uygulanmaz; uygulama
  ayrıca `dogfood-change` kapısı ve proje sahibi kararı ister. V1 sözü
  değildir.
- **Tarih:** 27 Kasım 2026
- **İş:** K-364 (yeni K numarası açılmadı)
- **Üst RFC:** [RFC-0040](0040-webdav-caldav-carddav-adaptoru.md)
- **İlgili belgeler:** [spec/11](../spec/11-uygulama-eylemleri-ve-web-adaptoru.md),
  [spec/12](../spec/12-web-guvenlik-profili.md),
  [spec/21](../spec/21-deterministik-io-izi.md),
  [spec/23](../spec/23-yetkinlik-ve-outbound-guvenligi.md),
  [ADR-036](../adr/036-kanonik-web-proxy-origin-siniri.md),
  [ADR-037](../adr/037-byte-tabanli-http-istek-siniri.md),
  [ADR-157](../adr/157-istek-yerel-hata-siniri.md),
  [ADR-163](../adr/163-ikili-yanit-multipart-ve-ek-turu-kaydi.md) (K-406 eki),
  [ADR-179](../adr/179-makine-uc-noktasi.md),
  [RFC-0017](0017-web-oturum-ve-csrf.md),
  [RFC-0037](0037-rota-onbellek-basligi.md),
  [RFC-0046](0046-makine-uc-noktasi.md)
- **Ürün kanıtı:** [`dogfood/urun-h/dav-reproducer.md`](../dogfood/urun-h/dav-reproducer.md)
  ve `dogfood/urun-h/korpus/rfc0040-*.dil` (dokuz vaka)
- **Freeze sınıfı adayı:** `dogfood-change` (semantic-change); ürün `urun-h`.
  Bu taslak kendi başına hiçbir dil, CLI, tanı ya da API yüzeyi açmaz.

## Özet

RFC-0040, CalDAV/CardDAV için gereken web adaptörü eksiklerini yedi katmana
(A–G) böldü ve ilk alt RFC için "E+F ile A birlikte" sırasını önerdi. Ürün H
(ZeeMail) 26 Kasım 2026'da verdiği reproducer'larla bu sıranın tek başına
çalışan bir istemci vermediğini gösterdi: başlıksız PROPFIND `Depth: infinity`
sayılıp reddedilir, multistatus yanıtı 207 + `application/xml` ister, iki
yönlü eşitleme ETag ve `If-Match` ister. Bu alt RFC ürünün "en küçük çalışan
uç" önerisini tasarlar: tek bir açık rota sınıfı (`dav istemcisi gerekli`,
aday), bu sınıfa bağlı iki yeni yöntem (PROPFIND, REPORT) ve var olan PUT/
DELETE/GET/HEAD, kapalı üç başlıklık okuma listesi (`Depth`, `If-Match`,
`If-None-Match`), 207 + türlü yanıt birleşimi ve ETag yazımı, çerezsiz Basic +
HMAC'li uygulama parolası ve ayrı oran kovaları, bu sınıfa özgü CSRF/Origin
muafiyeti. XML ağacı (D) ve well-known keşif (G) sonraki alt RFC'dedir.

## Konum ve karar zinciri

- 21 Kasım 2026: RFC-0040 taslağı (K-364), katman başına dogfood kapısı.
- 26 Kasım 2026: proje sahibi "önce reproducer" dedi; ürün on iki vaka verdi,
  dokuzu RFC-0040 için `dogfood/urun-h/korpus/` altına girdi.
- 27 Kasım 2026: proje sahibi bu alt RFC'nin açılmasını onayladı. Onay
  **taslak** içindir; uygulama ayrı `dogfood-change` beyanı, ADR, spec
  maddeleri, olumlu/olumsuz test ve proje sahibi kararı ister.

## Kapsam

| Katman | Bu alt RFC'de | Sonraki alt RFC'ye |
|---|---|---|
| A — yöntem kaydı | PROPFIND, REPORT; PUT/DELETE/GET/HEAD'in DAV rotasında kullanımı; 405 + `Allow`, kayıt dışı yöntem 501 | OPTIONS'ın Zee rotası olarak yazılması, PROPPATCH, MKCOL, MKCALENDAR, MOVE/COPY |
| B — istek başlığı | Yalnız `Depth`, `If-Match`, `If-None-Match` | `Prefer`, `Brief`, `Destination`, `Overwrite`, `Content-Type` okuma |
| C — yanıt | Türlü yanıt + durum birleşimi (207 dahil), `text/vcard` web metin satırı, `ETag` | `Location`/`Content-Location`/`Preference-Applied`, dosyadan yanıta akıtma |
| D — XML gövde | Yalnız ham gövdenin `Metin` olarak alınması (K-405 ilkeli, DAV rotasına taşınır) | Sınırlı XML ağacı, XML kaçış yardımcısı, CRLF çevirisi kararı, büyük gövde tutamağı |
| E — kimlik | Çerezsiz Basic, uygulama parolası (128+ bit, HMAC + arama öneki, cihaz başına iptal), ayrı oran kovaları | Salt-okuma kapsamının ince ayarı, IMAP/SMTP AUTH ile ortak ilkelin RFC-0039 tarafı |
| F — CSRF/Origin | `dav istemcisi gerekli` sınıfına bağlı muafiyet; PUT ve DELETE dahil | — |
| G — well-known | — | `/.well-known/caldav`, `/.well-known/carddav`, 301/308 kararı |

### RFC-0040 sırasından sapmanın gerekçesi (uygulama ADR'sine yazılacak)

RFC-0040 "Core freeze gerekçesi" madde 4, sıra değişirse gerekçenin ADR'ye
yazılmasını ister. Bu alt RFC bir ADR değildir; uygulama ADR'si aşağıdaki
gerekçeyi taşımak ZORUNDA olacaktır:

1. **E+F+A tek başına çalışan istemci vermez.** Ürün reproducer'ı
   `rfc0040-b-depth-basligi` `Depth` okunamadığını gösterir. RFC-0040 B'ye
   göre başlıksız PROPFIND `infinity` sayılır ve 403 alır. Başlık
   okunamazsa rota `Depth: 0` ile `Depth: 1`i ayıramaz ve koleksiyon ile
   çocuklarının listesi aynı yanıtta karışır.
2. **Multistatus 207 + `application/xml` ister.** `rfc0040-c-207-xml` K-406
   türlü yanıtı ile durumlu yanıtın ayrı ayrı var olduğunu, birlikte
   yazılamadığını (S037) gösterir. 207'siz PROPFIND yanıtını istemciler
   reddeder.
3. **Asıl kazanç iki yönlü eşitlemedir.** Salt okunur abonelik bugün K-406
   takvim yayınıyla (`GET … "text/calendar" türünde gönder`) vardır. Yeni
   değer, istemciden yazmadır: `rfc0040-f-put-makine` PUT'u, `rfc0040-c-
   etag-basligi` ETag'i, `rfc0040-b-depth-basligi` yorumu `If-Match`/
   `If-None-Match`'i ister. Bu yüzden F, PUT ve DELETE'i kapsar ve B/C'nin dar
   kısmı ilk dilime girer.
4. **D ve G'yi ertelemek güvenlik sınırını gevşetmez.** Ürün bugün makine
   uç noktasında XML'i metin işlemleriyle işliyor (ADR-179 AÇIK: "Dil XML
   ayrıştırmaz"). Ham gövde `Metin` olarak verildiğinde DTD işlenmez, XXE ve
   varlık genişlemesi yapısal olarak yoktur. XML ağacı yeni bir ayrıştırıcı
   crate'i (tedarik zinciri) ve geniş bir yüzey getirir; ayrı alt RFC'de
   ölçülmesi daha güvenlidir. G bugün GET/303 ile yazılabilir; istemcilerin
   çoğu elle verilen URL ile kurulur.

## Tasarım

Aşağıdaki sözdizimi **fikir taslağıdır**; kesin yüzey uygulama işinde
kullanılabilirlik kartıyla seçilir ve K-167 fixture'ına o zaman girer.

```text
PROPFIND "/dav/" önekli adrese istek geldiğinde
    dav istemcisi gerekli
    derinlik isteğin "Depth" başlığı olsun
    istek_govdesi isteğin ham gövdesi olsun
    xml koleksiyon_yaniti ile istekte "yol" ile derinlik ile oturum olsun
    xml metnini "application/xml" türünde 207 durumuyla gönder

PUT "/dav/takvimler/" önekli adrese istek geldiğinde
    dav istemcisi gerekli
    kosul isteğin "If-Match" başlığı olsun
    takvim isteğin ham gövdesi olsun
    # Zee: koşulu kaydın ETag'iyle karşılaştır, 412 ya da kaydet
    yanıta "ETag" başlığını yeni_etiket olarak ekle
    "" yanıtını 201 durumuyla gönder
```

### 1. Rota sınıfı: `dav istemcisi gerekli` (F)

Muafiyet bir ayrıcalık değil tehdit modelinin farkıdır: CSRF, tarayıcının
ortam kimlik bilgisini (çerez) kendiliğinden eklemesine dayanır. Bu sınıfta
kimlik yalnız istekte açıkça taşınan Basic + uygulama parolasından gelir.

- **Politika.** Rota önsözünün ilk ve tek politika cümlesi `dav istemcisi
  gerekli` (aday) olabilir; T049/T050 kuralı aynen geçerlidir.
- **Yöntemler.** Cümle yalnız GET, HEAD, PROPFIND, REPORT, PUT ve DELETE
  rotalarında yazılabilir. POST ve PATCH ile birleşimi derleme tanısıdır
  (aday). POST yasağının nedeni: HTML formu ve CORS önuçuşu gerektirmeyen
  `fetch` ile tarayıcının çapraz kökenden sahte üretebildiği durum
  değiştiren tek yöntem POST'tur; DAV POST kullanmaz.
- **Yol.** Makine uç noktasının (T060) tersine önekli yol (`"/dav/" önekli
  adrese`) serbesttir. DAV kaynak yolları değişkendir ve yol parametresi
  sözdizimi yoktur (RFC-0040 açık soru). Rota `istek` sözlüğünün `"yol"`
  alanını Zee'de böler.
- **Çerez ve oturum.** İstek `Cookie` başlığı okunmaz. Rota `oturum`u
  çerezden asla doldurmaz; `oturum` yalnız E'nin istek yerel kimlik
  kararıdır ve istek bitince atılır. Yanıt hiçbir koşulda `Set-Cookie`
  taşımaz; kayar yenileme yapılmaz. `çerezler` boş sözlüktür.
- **CSRF ve Origin.** `_csrf` alanı aranmaz; `--web-proxy` profilinin durum
  değiştiren yöntemde `Origin` zorunluluğu bu sınıfta kalkar. **Ama**
  istekte `Origin` başlığı varsa ve yapılandırılan origin ile kanonik olarak
  eşleşmiyorsa yanıt rota gövdesi çalışmadan 403'tür. `Sec-Fetch-Site:
  cross-site` için de aynı ret adaydır (Açık sorular).
- **CORS yok.** Sınıf hiçbir `Access-Control-*` başlığı üretmez. PROPFIND,
  REPORT, PUT ve DELETE "basit olmayan" yöntemlerdir; tarayıcı çapraz
  kökenden göndermeden önce OPTIONS önuçuşu yapar, yanıtta CORS izni
  olmadığı için asıl isteği hiç göndermez. GET/HEAD salt okumadır (T045) ve
  yanıtı CORS'suz çapraz kökende okunamaz.
- **Host/Forwarded/HTTPS.** spec/12 §5 denetimleri aynen uygulanır
  (ADR-036). Basic yalnız `--web-proxy` (HTTPS) profilinde kabul edilir;
  `--deneysel-web` düz HTTP'de DAV rotası 403 ve operatör günlüğü verir
  (geliştirme istisnası Açık sorular). Hermetik `dil dene` adaptörü aynı
  kuralları uygular.
- **Statik kapı.** Rota gövdesi ya da geçişli olarak çağırdığı işlemler
  `csrf belirteci`, `oturuma al`, `oturumu kapat`, çerez yazma/silme, web
  oturumu listeleme/iptal ve `çerezler` okumasını kullanamaz. Bu T061'in
  aynı ilkesidir; mevcut kodu genişletmek mi yeni kod mu açmak alt karardır
  (aday).
- **Makine uç noktasıyla ilişki.** `herkese açık makine uç noktası`
  (ADR-179) değişmez: kimliksiz, yalnız POST, öneksiz. Bu sınıf onun
  "kimlikli" biçimi değildir. ADR-179'un değişmezi ("CSRF'siz rota tarayıcı
  kimliğine ulaşamaz") burada "CSRF'siz rota yalnız açıkça taşınan, ortamdan
  eklenemeyen kimliği görür" biçiminde korunur.

### 2. Yöntem kaydı (A)

- `HttpYontemi` kapalı kaydı `PROPFIND` ve `REPORT` ile genişler. İkisi
  **salt okuma** sınıfındadır: T045 kanıtı GET/HEAD ile aynı biçimde
  uygulanır (yazıcı eylem, çerez, dosya yazımı yok). Ürün yazımı PUT/DELETE
  rotalarında yapar.
- PROPFIND ve REPORT yalnız `dav istemcisi gerekli` rotasında yazılabilir;
  başka politikayla ya da politikasız birleşim derleme tanısıdır (aday). Bu,
  `rfc0040-a-propfind`/`rfc0040-a-report` vakalarındaki `herkese açık makine
  uç noktası` + PROPFIND birleşimini kapsar.
- T047 (aynı yöntem + yol iki kez) yeni yöntemlere de uygulanır. Önekli
  (K-055) ve tam yollu iki DAV rotası aynı isteğe eşleşebildiğinde hangisinin
  seçileceği bugün normatif olarak yazılı değildir; uygulama spec maddesi
  bunu (öneri: tam yol önce, sonra en uzun önek) ZORUNLU kural olarak bağlar.
- **405 + `Allow`.** Yol eşleşip yöntem eşleşmezse 405 yanıtı, o yol için
  tanımlı yöntemlerin kanonik sırasıyla `Allow` başlığını taşır (RFC 9110
  §15.5.6). Bu kural yalnız DAV sınıfı için değil, bütün rotalar için
  düzeltmedir; bugün 405 `Allow` taşımaz. Kapsamı (yalnız DAV mı, her yol mu)
  uygulama ADR'sinin kararıdır, öneri her yol.
- **501.** Kapalı kayıtta hiç olmayan yöntem (örn. `LOCK`, `MKCALENDAR`,
  `PROPPATCH`, `MOVE`) yoldan bağımsız 501'dir. Bugünkü davranış (405 ya da
  404) RFC 9110 §15.6.2 ile uyumsuzdur.
- **OPTIONS.** İlk dilimde Zee rotası olarak yazılamaz. Bazı istemciler
  yetenek keşfi için OPTIONS gönderir ve `DAV` başlığına bakar. Öneri:
  çalışma zamanı, en az bir DAV sınıfı rotanın öneki altındaki yolda OPTIONS'ı
  kimlik istemeden 200 + `Allow` + `DAV` ile kendisi yanıtlar; CORS başlığı
  yazmaz. Kesin karar istemci ölçümüne bağlıdır (Açık sorular).

### 3. İstek başlığı okuma (B, dar liste)

- İfade (aday): `isteğin "<ad>" başlığı`. Ad, kaynakta görünen tırnaklı
  literal olmak ve kapalı listeyle exact (büyük/küçük harf duyarsız
  karşılaştırma, kanonik yazım kayıtta) eşleşmek ZORUNDA olacaktır:
  `Depth`, `If-Match`, `If-None-Match`. Değişken/hesaplanmış ad ve liste
  dışı ad derleme tanısıdır (aday).
- **Güven başlıkları hiçbir zaman listeye girmez:** `Authorization`,
  `Cookie`, `Forwarded`, `X-Forwarded-For`, `Host`, `Origin`, `X-Zee-CSRF`.
  Bunları okuma girişimi ayrı, önerili bir tanıdır (aday): "kimlik
  `dav istemcisi gerekli` ile çalışma zamanından gelir". Bu,
  `rfc0040-e-basic-kimlik` vakasının kalıcı cevabıdır.
- İfade yalnız `dav istemcisi gerekli` rotasının gövdesinde okunur; başka
  rotada, işlem/eylem gövdesinde ya da üst düzeyde derleme tanısıdır (aday;
  T062'nin ilkesi). İlk dilimde yüzey dar tutulur; genel rotalara açılması
  ayrı ürün kanıtı ister.
- Dönüş `Seçenek<Metin>`: başlık yoksa boş. Çalışma zamanı değeri rota
  çalışmadan doğrular; geçersiz değer 400'dür ve rota çalışmaz:
  - Tekrarlanan başlık (iki `Depth`, iki `If-Match` satırı) 400.
  - Değer yalnız görünür ASCII ve boşluk; ≤ 1 KiB.
  - `Depth` ∈ {`0`, `1`, `infinity`}, küçük harfe normalleştirilir.
    **PROPFIND'de başlık yoksa ya da `infinity` ise** yanıt rota çalışmadan
    403'tür ve gövdesi `DAV:propfind-finite-depth` ön koşul öğesini taşıyan
    sabit `application/xml` belgesidir (RFC 4918 §9.1). Böylece PROPFIND
    rotasının gördüğü değer her zaman `0` ya da `1`dir. REPORT'ta başlık
    yoksa değer `0` sayılır (RFC 3253 §3.6).
  - `If-Match`/`If-None-Match`: `*` ya da virgülle ayrılmış en çok 16
    entity-tag (`"…"` ya da `W/"…"`, RFC 9110 §8.8.3). Değer tek boşluklu
    `, ` ayracıyla normalleştirilmiş Metin olarak verilir.
- **Koşul değerlendirmesi Zee'dedir.** Çalışma zamanı `If-Match` ile kaydın
  ETag'ini karşılaştırmaz; kayıt ürün verisidir. Ürün uyuşmazlıkta 412,
  `If-None-Match: *` ile var olan kaynağa PUT'ta 412 yazar.

### 4. Ham gövde (D'den ayrılan dar parça)

- K-405 `isteğin ham gövdesi` ifadesi `dav istemcisi gerekli` rotasında da
  okunabilir. T062 sınırı "yalnız makine ucu" yerine "yalnız makine ucu ve
  DAV sınıfı" olur; sıradan oturumlu rotada ham gövde yine T062'dir
  (`rfc0040-f-oturumlu-ham-govde` kalıcı olarak reddedilir).
- Gövde form olarak çözülmez; `istek` yalnız `yol`, `yöntem` ve sorgu
  alanlarını taşır.
- Yöntem başına tür kuralı (çalışma zamanı, rota çalışmadan):
  - PROPFIND: gövde isteğe bağlıdır (yoksa boş Metin; RFC 4918 `allprop`);
    varsa `application/xml` ya da `text/xml`.
  - REPORT: gövde zorunlu; `application/xml` ya da `text/xml`.
  - PUT: gövde zorunlu; `text/calendar` ya da `text/vcard`.
  - GET, HEAD, DELETE: gövde yasak; `Content-Length` > 0 ya da gövde 400.
  - `charset` parametresi varsa yalnız `utf-8`. Eksik ya da başka tür 415.
- **Tavan.** Varsayılan 64 KiB (spec/11 §5 istek gövde tavanı), makine
  ucunun 8 KiB'ı değil: `calendar-multiget` REPORT'u yüzlerce `href`, tek
  etkinlik uzun açıklama taşıyabilir. Aşım gövde okunmadan 413 ve K-374
  boşaltması. Katı UTF-8; geçersiz bayt 400. Daha yüksek tavan (K-391
  `geniş_gövde_rotaları` genişlemesi) Açık sorulardadır.
- **CRLF.** Gövde baytları olduğu gibi verilir; CR, LF ve CRLF dönüştürülmez.
  Bu, K-406 yanıt kuralıyla (spec/11 "CR, LF ve CRLF dönüştürülmez")
  simetriktir. RFC-0040 D'nin "türlü yanıtta LF→CRLF" (i) önerisi K-406 ile
  çeliştiği için D alt RFC'sinde yeniden değerlendirilir; bu dilim yeni
  satır sonu kuralı koymaz.

### 5. Yanıt (C, dar kısım)

- **Türlü yanıt + durum.** K-406 cümlesinin durumlu kardeşi (aday):
  `<ifade> metnini "<tür>" türünde <durum> durumuyla gönder`. Durum kaynakta
  görünen literaldir (S037 ilkesi) ve kapalı kümeden olmak ZORUNDA
  olacaktır: 200, 201, 207, 400, 403, 404, 409, 412, 415. Bu birleşim yalnız
  DAV sınıfında değil, her rotada açılabilir: tür ve durum ikisi de kapalı
  literaldir, yeni yetenek getirmez. Öneri her rota; bu,
  `rfc0040-c-207-xml` vakasını olduğu gibi başarılıya çevirir.
- **Tür kaydı.** K-406 web metin satırlarına `text/vcard` eklenir (CardDAV,
  RFC 6352). Başlık kümesi K-406 ile aynıdır (`nosniff`, `no-store`,
  `default-src 'none'; sandbox` CSP, CORP). RFC-0037 önbellek politikası DAV
  sınıfında yasaktır: içerik kimliğe bağlıdır.
- **Durum açıklamaları.** Durum satırı tablosu `207 Multi-Status`,
  `412 Precondition Failed`, `501 Not Implemented` açıklamalarını alır (bugün
  207 durum satırında `Error` açıklamasıyla gider; RFC-0040 tablo satırı 3).
- **Gövdesiz durumlar.** PUT/DELETE başarısında 201/204 bugünkü durumlu
  yanıtla yazılır; 204 yanıtının gövde ve `Content-Type` taşımaması kabul
  testinde doğrulanır.
- **ETag.** Cümle (aday, ürünün yazımı): `yanıta "ETag" başlığını <ifade>
  olarak ekle`. Başlık adı yalnız `ETag` literali olabilir; başka ad derleme
  tanısıdır (aday) ve keyfi başlık yazımı yine yoktur (V1 "dinamik header
  builder yok"). Cümle yalnız DAV sınıfında ve yanıt cümlesinden önce
  yazılır; aynı rotada ikinci ETag cümlesi sonuncusunu geçersiz kılmaz,
  derleme tanısıdır (aday).
  - Değer çalışma zamanında doğrulanır: güçlü entity-tag, `"` + etagc* +
    `"`, etagc = `%x21 / %x23-7E` (obs-text yok), ≤ 128 bayt. `W/` zayıf
    etiketi yazılamaz (`If-Match` güçlü karşılaştırma ister).
  - Geçersiz değer başlığa hiç girmez: istek-yerel hata sınırıyla (ADR-157)
    500 ve operatör günlüğü; tanı ailesi spec/12 §4 C022 ile aynıdır (aday:
    mevcut C022'nin genişlemesi ya da yeni C kodu).
  - ETag 200 (GET/HEAD türlü yanıt) ve 201/204 (PUT) yanıtlarında yazılır;
    207 yanıtında `getetag` gövdenin parçasıdır.
- **Çalışma zamanının yazdığı başlıklar** (Zee yazamaz): `Allow` (405 ve
  OPTIONS), `DAV` (OPTIONS ve DAV sınıfı yanıtlar; değer Açık sorular),
  `WWW-Authenticate` (401), `Retry-After` (429).

### 6. Kimlik: çerezsiz Basic ve uygulama parolası (E)

**Uygulama parolası biçimi.** Kullanıcı web arayüzünde (oturum + CSRF +
mevcut Argon2id girişi) cihaz başına uygulama parolası üretir. Değer:

- sürüm etiketi + **arama öneki** (40 bit, Base32, gizli değil) + **gizli
  kısım** (160 bit CSPRNG, Base32). Gizli kısım tek başına 128 biti aşar;
  önek entropi hesabına katılmaz. Örnek biçim adayı `zua1-` + 8 + 32 Base32
  karakter, gösterimde dörtlü gruplar.
- Yalnız üretim anında bir kez gösterilir; depoda tutulmaz.

**Üretim yüzeyi (aday).** `yeni uygulama parolası` ifadesi `kriptografi`
yetkinliği ister, yalnız oturumlu (CSRF'li) web rotasında ya da CLI'da
çalışır ve `{görünen, önek, özet}` sözlüğü döndürür. `özet`
`HMAC-SHA256(anahtar, alan_ayracı ‖ önek ‖ gizli)` değeridir ve anahtar
kimliğini taşır (aday biçim `zuah1:<anahtar-kimliği>:<base64url>`).
Üretim Argon2id kullanmaz, giriş kovasına dokunmaz (ZM-F020/K-365 sorunu bu
akışta doğmaz). `görünen` IO izinde maskelidir.

**Doğrulama (öneri: Zee arar, çalışma zamanı karşılaştırır).** Rota
gövdesinden önce, sırasıyla:

1. Tek `Authorization` başlığı; şema `Basic` (büyük/küçük harf duyarsız);
   katı base64 (dolgu zorunlu, boşluk yok); çözülmüş değer ≤ 512 bayt, katı
   UTF-8, tam bir `:`; kullanıcı adı boş değil ve ≤ 254 bayt. Aksi 401.
2. Başarısız kimlik kovası doluysa 429 + `Retry-After`; 3–5. adımlar
   çalışmaz.
3. Parola uygulama parolası biçiminde değilse (hesap parolası dahil)
   başarısızdır: arama yapılmaz, sahte HMAC hesabıyla süre eşitlenir, sayaç
   artar, 401. **Hesap parolası DAV'da yapısal olarak kabul edilmez** ve
   Argon2id hiç çalışmaz.
4. Projede bildirilen **kimlik çözücü işlem** (aday proje alanı
   `dav_kimlik_çözücüsü`) `(kullanıcı adı, önek)` ile çağrılır ve
   `Seçenek` olarak `{özet, kullanıcı, rol, kapsam, kimlik}` döndürür.
   İşlem salt okumadır (T045 kanıtı; PostgreSQL okuma serbest, yazma yok)
   ve isteğin son tarihi altında çalışır. Önbellek yoktur: iptal bir sonraki
   istekte etkilidir.
5. Çalışma zamanı HMAC'i hesaplar ve kayıttaki özetle **sabit zamanlı**
   karşılaştırır; kayıttaki `kullanıcı` Basic kullanıcı adıyla bayt bayt
   eşleşmek ZORUNDA olacaktır. Kayıt yoksa aynı maliyette sahte karşılaştırma
   yapılır.
6. Başarıda `oturum` istek yerel olarak `kullanıcı`, `rol`,
   `kimlik_türü = "uygulama_parolası"`, `uygulama_parolası_kimliği` ile
   dolar; `"<rol>" yetkisi` kararı rota içinde bu sözlükten verilir. Kapsam
   `salt_okuma` ise PUT/DELETE 403'tür.

Ham `Authorization` değeri, gizli kısım ve HMAC anahtarı hiçbir zaman Zee
değeri olmaz. Zee yalnız kullanıcı adını, öneki (gizli değil) ve depodaki
özeti görür. Son kullanım anının yazımı ürünün işidir (PUT/DELETE
rotasında ya da ayrı süreçte); çözücü salt okuma olduğundan oraya yazılmaz.

**Ret biçimi.** Başlık yok, bozuk, kullanıcı yok ya da parola yanlış:
hepsi aynı sabit gövdeli 401 +
`WWW-Authenticate: Basic realm="<kanonik origin>", charset="UTF-8"`;
süre kullanıcı varlığından bağımsızdır. Doğru kimlik, yanlış kapsam 403.
401 yalnız DAV sınıfı rotalarda `WWW-Authenticate` taşır.

**HMAC anahtarı.** Öneri ayrı credential (aday proje alanı
`uygulama_parolası_anahtar_değişkeni`), K-339 modeliyle ortamdan ya da
systemd credential'dan; şifreleme anahtarı (`gizli_anahtar_değişkeni`) ile
MAC anahtarının ayrılması anahtar ayrımı ilkesidir. Anahtar döndürmede eski
anahtar kimlikleri okuma için tutulur (Açık sorular).

### 7. Oran kovaları

| Kova | Anahtar | Pencere / eşik (aday) | Aşım | Not |
|---|---|---:|---|---|
| Başarısız DAV kimliği | kanonik istemci + kullanıcı adı | 60 sn / 10 | 429 + `Retry-After` | Başarı sayacı sıfırlamaz |
| Başarısız DAV kimliği, istemci geneli | kanonik istemci | 60 sn / 30 | 429 | Çok kullanıcı adına yayılan doldurma |
| Başarılı DAV isteği | uygulama parolası kimliği | 60 sn / 600 | 429 | İlk eşitleme yükü; sayı ölçümle bağlanır |
| Uç nokta (spec/12 §5) | istemci + yöntem + yol | 60 sn / 100 | 429 | Değişmez |
| Argon2id parola | — | 300 sn / 5 | — | DAV'da hiç tetiklenmez |

İstemci kimliği çalışma zamanının ADR-036 ile çözdüğü kanonik adrestir ve
Zee'ye verilmez. Kullanıcı adı başına (istemciden bağımsız) küresel kilit
bilinçli olarak yoktur: 160 bitlik gizli kısma karşı dağıtık çevrimiçi
deneme anlamsızdır, böyle bir kilit ise saldırganın meşru kullanıcıyı
kilitlemesine (DoS) izin verir.

### 8. IO izi ve hermetik adaptör

- Yeni olay adayları: okunan başlık (ad + doğrulanmış değer), DAV kimlik
  kararı (kullanıcı adı, önek, sonuç sınıfı, kova kararı), ETag yazımı.
- `Authorization` değeri, gizli kısım, HMAC anahtarı ve `yeni uygulama
  parolası`nın `görünen` alanı ize hiç girmez; masking testi kabul
  ölçütüdür.
- `dil dene` hermetik web adaptörü aynı sınıf kurallarını (yöntem, tür,
  tavan, Depth, Origin, kova) gerçek çalışma zamanıyla aynı biçimde uygular;
  aynı izle aynı yanıt.

## Tehdit modeli

| Tehdit | Senaryo | Karşılık |
|---|---|---|
| Kimlik doldurma (credential stuffing) | Sızmış hesap parolası listesiyle Basic denemesi | Hesap parolası biçim kapısında reddedilir (adım 3), Argon2id çalışmaz; başarısız kovalar istemci + kullanıcı adı ve istemci geneli; uygulama parolası yalnız bu sunucuya özgüdür |
| Çevrimiçi kaba kuvvet | Hızlı HMAC doğrulamasının denemeyi ucuzlatması | 160 bit gizli kısım; kovalar; GB kaydı uygulamada |
| Çevrimdışı deneme | Depo (PostgreSQL) sızıntısı | Yalnız HMAC özeti; anahtar ayrı credential'da, depoda değil |
| Kullanıcı adı numaralandırma | 401 gövdesi ya da süresi farkı | Tek sabit 401; kayıt yoksa sahte karşılaştırma; sabit zamanlı eşitlik |
| CSRF | Saldırgan sayfası tarayıcının Basic önbelleğiyle DAV'a yazar | PUT/DELETE/PROPFIND/REPORT basit olmayan yöntem → önuçuş, CORS izni yok; POST/PATCH DAV sınıfında derleme tanısı; yabancı `Origin` 403; çerez okunmaz, `Set-Cookie` yok; statik kapı (T061 ilkesi) |
| Muafiyetin sızması | Tarayıcı rotası CSRF'siz kalır | Muafiyet yalnız açık `dav istemcisi gerekli` cümlesinde; başka rota bugünkü CSRF/Origin kurallarında |
| Başlık enjeksiyonu | ETag değerine CR/LF ile ek başlık | ETag kapalı etagc kümesi (CR/LF/denetim yok), ≤ 128 bayt; geçersiz değer başlığa girmez, 500 |
| Yanıt bölme | Başlık değeriyle ikinci yanıt kurmak | Keyfi başlık adı yok; tek yazılabilir başlık `ETag`; durum ve tür literal |
| İstek kaçakçılığı | DAV gövdesiyle smuggling | ADR-037 bayt parser'ı, tek `Content-Length`, `Transfer-Encoding` reddi değişmez |
| Kimlik sızması (Zee'ye) | `Authorization` okunup günlüğe, ize ya da veritabanına yazılır | Güven başlıkları okuma listesinde yok, okuma girişimi derleme tanısı; ham değer Zee değeri olmaz |
| Kimlik sızması (iz/günlük) | Uygulama parolası IO izine düşer | Maskeleme; `görünen` yalnız üretim yanıtında |
| Derinlik DoS | `Depth: infinity` ile tüm ağaç dökümü | PROPFIND'de varsayılan ve `infinity` 403 `propfind-finite-depth` |
| Büyük gövde | Bellek/heap tükenmesi | 64 KiB tavan, `Content-Length` ön denetimi, 413, K-372 bütçeleri |
| XXE / varlık genişlemesi | XML gövdesiyle dosya okuma | Dil XML ayrıştırmaz (bu dilim); DTD işlenmez. D alt RFC'si ayrıştırıcıyla birlikte DOCTYPE reddini getirir |
| Ağ seviyesi düz metin | Basic HTTP üzerinden | Yalnız `--web-proxy` (HTTPS); düz HTTP'de DAV rotası 403 |

Uygulama işinde her bilinçli sınır `docs/guvenlik-bulgulari-v1.tsv` GB
satırı alır. Bu taslak kod değiştirmediği için GB satırı açmaz.

## Reproducer → madde eşlemesi

| Vaka | Bugünkü tanı | Karşılayan madde | Uygulama sonrası beklenti |
|---|---|---|---|
| `rfc0040-a-propfind` | S036 | §2 PROPFIND; §1 sınıf | Ret kalır: PROPFIND + `herkese açık makine uç noktası` birleşimi yeni yöntem-politika tanısı (aday). Çözüm çifti `dav istemcisi gerekli` ile başarılı |
| `rfc0040-a-report` | S036 | §2 REPORT; §1 | `rfc0040-a-propfind` ile aynı |
| `rfc0040-b-depth-basligi` | S015 | §3 `Depth` | Ret kalır: başlık okuma POST + makine ucunda (DAV sınıfı dışında) tanı (aday). Çözüm çifti PROPFIND DAV rotasında başarılı |
| `rfc0040-c-207-xml` | S037 | §5 türlü yanıt + durum | **Başarılıya döner** (öneri her rota; makine ucunda 207 + `application/xml` geçerli) |
| `rfc0040-c-etag-basligi` | S015 | §5 ETag | Ret kalır: ETag cümlesi yalnız DAV sınıfında; oturumlu GET'te tanı (aday). Seçilen cümle ürün yazımından farklıysa tanı değişir. Çözüm çifti DAV rotasında başarılı |
| `rfc0040-e-basic-kimlik` | S015 | §3 güven başlığı yasağı; §6 | Kalıcı ret, önerili yeni tanı (aday): "kimliği `dav istemcisi gerekli` sağlar". Çözüm çifti başarılı |
| `rfc0040-f-kimlikli-makine` | T050 | §1 | Kalıcı ret (T050, bilinçli sınır). Çözüm çifti başarılı |
| `rfc0040-f-oturumlu-ham-govde` | T062 | §4 | Kalıcı ret (T062: oturumlu rotada ham gövde yok). Çözüm çifti başarılı |
| `rfc0040-f-put-makine` | T060 | §1 PUT; §4 | Kalıcı ret (T060: makine ucu yalnız POST). Çözüm çifti PUT DAV rotasında başarılı |

## K-167 yüzey kaydı adayları

Hiçbiri bu taslakla `dil-yuzeyi-v1.tsv`'ye girmez; uygulama commit'inde
`aktif` satırı ve giriş sürümüyle girer.

| Yüzey sınıfı | Aday |
|---|---|
| Rota yöntemi | `PROPFIND`, `REPORT` |
| Politika cümlesi | `dav istemcisi gerekli` |
| İfade | `isteğin "<ad>" başlığı` |
| Başlık adı kaydı (okuma) | `Depth`, `If-Match`, `If-None-Match` |
| Yanıt cümlesi | `<ifade> metnini "<tür>" türünde <durum> durumuyla gönder` |
| Yanıt başlığı cümlesi | `yanıta "ETag" başlığını <ifade> olarak ekle` (ya da kullanılabilirlik kartında seçilen eşdeğeri) |
| İçerik türü kaydı (yanıt) | `text/vcard` (web metin satırı) |
| İçerik türü kaydı (istek, DAV sınıfı) | `application/xml`, `text/xml`, `text/calendar`, `text/vcard` |
| İfade (üretim) | `yeni uygulama parolası` |
| Proje alanı | `dav_kimlik_çözücüsü`, `uygulama_parolası_anahtar_değişkeni` |
| `oturum` anahtarı | `kimlik_türü`, `uygulama_parolası_kimliği` |
| IO izi olayı | başlık okuma, DAV kimlik kararı, ETag yazımı (adlar uygulamada) |

### Tanı kodu adayları (numara verilmez)

- DAV sınıfı politikası POST/PATCH rotasında (T ailesi adayı; T060 ile
  aynı aile).
- PROPFIND/REPORT `dav istemcisi gerekli` dışında (T ailesi adayı).
- `isteğin "<ad>" başlığı` kayıt dışı ya da hesaplanmış adla (S ailesi
  adayı).
- Güven başlığı (`Authorization`, `Cookie`, `Origin`…) okuma girişimi,
  önerili (S ailesi adayı; ayrı kod çünkü öneri farklıdır).
- Başlık okuma DAV sınıfı dışında (T062 ilkesi; genişleme ya da yeni T).
- DAV rotasında oturum/CSRF/çerez kullanımı (T061 genişlemesi adayı).
- Türlü yanıtta kapalı küme dışı durum (S037 genişlemesi adayı).
- ETag cümlesi DAV dışında, başka başlık adıyla ya da iki kez (S/T adayı).
- Geçersiz ETag değeri (çalışma zamanı, C022 ailesi adayı).
- Kimlik çözücü işlem yazıcı eylem içeriyor (T045 ilkesi).
- `dav istemcisi gerekli` rota var ama `dav_kimlik_çözücüsü` ya da anahtar
  değişkeni bildirilmemiş (P ailesi adayı).

Her yeni tanı `docs/hata-katalogu.md`, `katalog_testi` ve
`tani_kalitesi` kapısıyla birlikte açılır.

## Kabul ölçütleri

Bu alt RFC şunlar kanıtlandığında "geçici kabul"e aday olur:

1. Ürün H'nin Zee ile yazılmış en küçük CalDAV koleksiyonu ve CardDAV adres
   defteri `--web-proxy` arkasında, elle verilen URL ile (well-known yok)
   Thunderbird ve DAVx5'te hesap kurar; etkinlik/kişi oluşturur, düzenler,
   siler; iki cihaz eşitlenir. macOS/iOS ölçümü yapılır, OPTIONS ihtiyacı
   kayda geçer.
2. Olumsuz testler: çerezli ama Basic'siz DAV isteği (401, çerez yok
   sayılır), hesap parolasıyla Basic (401, Argon2id çalışmadı), iptal
   edilmiş uygulama parolası (bir sonraki istekte 401), yanlış kullanıcı
   adı + doğru önek, yabancı `Origin` 403, POST + DAV politikası derleme
   tanısı, başlıksız ve `infinity` PROPFIND 403, iki `Depth` 400, kayıt dışı
   yöntem 501, tanımsız yöntem 405 + `Allow`, CR/LF'li ETag 500 ve başlığa
   girmemesi, 64 KiB üstü gövde 413, yanlış tür 415, başarısız kova 429.
3. 401 süresinin kullanıcı varlığından bağımsızlığı ölçülür (dağılım
   karşılaştırması, eşik uygulama ADR'sinde).
4. İlk eşitleme (≥ 1000 etkinlik, ≥ 1000 kişi) normal istemciyle 429
   almaz; başarılı DAV kovası eşiği bu ölçümle bağlanır, spec/24 bütçesi
   aşılmaz.
5. IO izi replay aynı yanıtları verir; `Authorization`, gizli kısım ve
   `görünen` hiçbir iz/günlük satırında yoktur.
6. Zincir: `dogfood-change` beyanı, ADR (sıra sapması gerekçesi dahil),
   spec/11–12 maddeleri + `spec-madde-kaniti`, K-167 satırları, GB satırları,
   tanı kalitesi raporu, korpus güncellemesi aynı değişiklikte.

## Korpus etkisi

Taslağın kendisi golden, regression, korpus ve dil yüzeyi fixture'ını
değiştirmez. Uygulama işinde:

- `rfc0040-c-207-xml` `basarisiz` → `basarili` olur (yalnız beklenti
  sütunu; dosya değişmez).
- Öteki sekiz vaka ret kalır. `rfc0040-a-*`, `rfc0040-b-depth-basligi`,
  `rfc0040-c-etag-basligi` ve `rfc0040-e-basic-kimlik`in beklenen tanısı
  değişir (S036/S015 → yeni aday tanılar); `rfc0040-f-*` üçü T050/T060/T062
  ile aynı kalır. Vaka kimlikleri ve yolları silinmez/yeniden kullanılmaz.
- Her ret vakası için ürün kaynağından gelen bir **çözüm vakası**
  (`dav istemcisi gerekli` ile yazılmış, `basarili`) eklenir; çözüm
  vakaları da ürün provenance'ıyla birebir alınır, merkez tarafından
  uydurulmaz.

## Kapsam dışı: ZM-F027 ve ZM-F050

ZM-F027 (web rotasında istemci IP'si, `zm-f027-istemci-ip`) DAV'a özgü
değildir: giriş denetim kaydı, IP başına web giriş kilidi ve "yeni konumdan
giriş" uyarısı için bütün web rotalarına ait bir yüzeydir. Doğru çözüm form
alanı ya da istek sözlüğü anahtarı olamaz (değeri saldırgan belirler);
ADR-036 güvenilir vekil modeliyle çözülmüş kanonik adresin (çalışma zamanı
bugün oran sınırında zaten kullanıyor) salt okunur, iz kaydı ve gizlilik
kuralı belli bir ifadeyle verilmesini ister. Bu alt RFC bu adresi
kovalarda kullanır ama Zee'ye vermez. ZM-F027 ürün kaydı K-355 altında,
ayrı bir web adaptörü RFC'sinde ele alınır (yeni K numarası ürün
önceliklendirmesiyle açılır). ZM-F050 (17. alan için DKIM anahtarı ve tek
credential'ın birden çok alana bağlanması, P015) HTTP ile ilgisizdir;
tavan ve tekillik K-397 Dilim 2/ADR-174 kararıdır ve ancak çok kiracılı
barındırma ihtiyacı kanıtlanırsa o hatta (RFC-0044 B3) yeniden açılır.

## Alternatifler

- **RFC-0040'ın E+F+A sırası:** istemci kurulamaz (bkz. sapma gerekçesi);
  ikinci alt RFC beklenmeden ürün değer alamaz. Reddedildi.
- **Makine uç noktasını genişletmek (PUT, önek, kimlik):** ADR-179'un
  "CSRF'siz rota kimliksizdir" değişmezini ve T060/T061 statik akıl
  yürütmesini bozar. Ayrı açık sınıf daha denetlenebilir. Reddedildi.
- **Çalışma zamanına ait uygulama parolası deposu:** kimlik rota öncesi
  karar verilir, geri çağrı gerekmez; ama ürünün PostgreSQL'indeki ikinci
  kimlik deposu doğar, IMAP/SMTP AUTH (ürünün Zee kodu) aynı kaydı
  paylaşamaz, listeleme/iptal için ek yüzey gerekir. Açık soru olarak
  kaldı; öneri Zee arar, çalışma zamanı karşılaştırır.
- **Zee `Authorization`'ı okur ve HMAC'i kendisi hesaplar:** gizli değer
  Zee değeri olur (günlük/iz/veritabanı sızıntısı), oran kovası ürüne kalır.
  Reddedildi.
- **Argon2id'li uygulama parolası + daha yüksek kova:** her istekte
  yüzlerce milisaniyelik KDF, ilk eşitlemede yüzlerce istek; CPU DoS
  çarpanı. Reddedildi.
- **Yalnız SHA-256 özet:** 160 bitlik gizli kısım için çevrimdışı deneme
  yine olanaksızdır; HMAC aynı maliyetle zayıf üretim ya da depo sızıntısına
  karşı anahtar bağlılığı ekler. Öneri HMAC; açık soru RFC-0040'tan devralındı.
- **Genel başlık okuma/yazma API'si:** V1 "dinamik header builder yok"
  kararını bozar. Reddedildi.
- **D'yi (XML ağacı) bu dilime almak:** yeni ayrıştırıcı crate'i ve
  geniş yüzey; ürün bugün XML'i metinle işleyebiliyor. Ertelendi.

## Açık sorular

1. Kimlik çözücü: proje alanıyla bildirilen işlem geri çağrısı mı, rota
   önsözünde ikinci cümle mi, çalışma zamanı deposu mu?
2. HMAC anahtarı ayrı credential mı, `gizli_anahtar_değişkeni`'nden alan
   ayraçlı türetme mi; anahtar döndürmede eski kimliklerin süresi.
3. Uygulama parolası biçimi (önek uzunluğu, alfabe, sürüm etiketi) ve
   ürünün bugünkü Argon2id'li IMAP uygulama parolalarının göçü (RFC-0039
   ile ortak ilkel).
4. Otomatik OPTIONS yanıtı ve `DAV` başlığının değeri: sınıf `1`/`3`
   iddiası PROPPATCH/MKCOL/COPY/MOVE olmadan dürüst mü; yalnız
   `calendar-access, addressbook` mı?
5. Başlık okuma ve ETag yazımı ilk dilimde yalnız DAV sınıfında mı, yoksa
   bütün rotalarda mı?
6. `If-Match`/`If-None-Match` değeri `Metin` mi, `Liste<Metin>` mi?
7. 64 KiB gövde tavanı yeterli mi; DAV sınıfı K-391 `geniş_gövde_rotaları`
   kapsamına alınmalı mı?
8. DAV öneki ayrı origin (`dav.<alan>`) olarak önerilmeli mi, zorunlu mu?
   Aynı origin'de webmail XSS'i tarayıcının Basic önbelleğiyle DAV'a
   ulaşabilir.
9. `--deneysel-web` (K-400 loopback) geliştirme profilinde Basic kabulü.
10. `Sec-Fetch-Site: cross-site` reddi gerekli mi, `Origin` reddi yeterli mi?
11. Salt-okuma kapsamı ilk dilimde mi, sonraki dilimde mi?
12. Başarılı DAV kovasının eşiği ve anahtarı (uygulama parolası mı,
    kullanıcı mı); sayı ilk eşitleme ölçümüyle bağlanır.
13. 405 + `Allow` ve 501 düzeltmesi bütün rotalara mı (öneri), yalnız DAV
    sınıfına mı?

### Ürün görüşü (urun-h, 27 Kasım 2026)

Etkin dogfood ürünü `urun-h` (ZeeMail; ürün kodu 27 Kasım ana dalı ve temiz
Ubuntu 24.04 uçtan uca kurulum denemesi) açık soruların tamamına görüş
verdi. Bu görüşler karar değildir; uygulama ADR'si bunları kabul ya da
gerekçeli ret ile kapatır.

- **Soru 1 — kimlik çözücü:** proje alanıyla bildirilen işlem (aday
  `dav_kimlik_çözücüsü`); çalışma zamanı deposu ve rota önsözünde ikinci
  cümle istenmez. Uygulama parolaları ürünün kendi tablosunda tek doğruluk
  kaynağıdır (IMAP, gönderim ve web iptal aynı tabloyu kullanır); ayrı depo
  gecikmeli iptal doğurur. Hesap kapatma IMAP'te açık oturumu bir sonraki
  komutta düşürüyor; DAV'da da aynı anda düşmesi için "önbellek yok" kuralı
  korunmalı. Ürün isteği: kullanıcı adı karşılaştırması bayt eşitliği değil,
  ASCII küçük harfe katlanmış olmalı ya da çözücünün döndürdüğü kanonik ad
  esas alınmalı; istemciler kullanıcının yazdığı büyük harfi gönderir ve bayt
  eşitliği gerçek kullanıcıda yanlış 401 üretir.
- **Soru 2 — HMAC anahtarı:** ayrı credential; `gizli_anahtar`'dan türetme
  istenmez (o anahtarın döndürülmesi bugün MFA'ları sıfırlatıyor, uygulama
  parolalarını aynı olaya bağlamak bütün cihazları düşürür). Kurulum root
  0600 dosya + `LoadCredential` modeliyle sınandı. Döndürmede eski anahtar
  kimliği süreyle değil, ona bağlı en az bir kayıt kaldıkça tutulur; ürün
  "eski anahtarla üretilmiş N parola" sorgusunu ve yenileme çağrısını verir.
  Zaman aşımıyla silme sessizce çalışmayan cihaz bırakır.
- **Soru 3 — biçim ve göç:** aday biçim ürünün bugünkü gösterimiyle
  (Base32 dörtlü gruplar) uyumlu, kabul. Gizli değer saklanmadığından zorla
  göç yoktur: (a) yeni parolalar yalnız yeni biçimde ve IMAP/gönderim de aynı
  ilkelle doğrular (RFC-0039 ortak); (b) eski Argon2id kayıtları yalnız
  IMAP/gönderimde geçerli kalır, DAV'da kabul edilmez, arayüz yenilemeyi
  ister; (c) kapanış ürün kararıdır. Önekli arama IMAP girişindeki çoklu
  Argon2id denemesini ve oran kovasına dokunmayı da bitirir.
- **Soru 4 — `DAV` başlığı:** PROPPATCH/MKCOL/COPY/MOVE yokken sınıf 1/3
  iddia edilmemeli; yalnız `calendar-access, addressbook`. Ürün iOS ve
  Thunderbird ile ölçüme hazır.
- **Soru 7 — gövde tavanı:** 64 KiB CardDAV PUT için yetmez (gömülü base64
  fotoğraflı kişi kartı tipik 50–200 KB; 413 eşitlemeyi kullanıcıya görünmeden
  takar). Öneri: `text/vcard`/`text/calendar` PUT 1 MiB (ürünün mevcut geniş
  gövde sınırıyla aynı), REPORT/PROPFIND 256 KiB; tavanın yöntem ya da tür
  başına bildirilebilmesi tercih edilir. DAV sınıfı K-391
  `geniş_gövde_rotaları` kapsamına alınmalı.
- **Soru 8 — ayrı origin:** üretim profilinde `dav.<alan>` **zorunlu**
  olmalı; çerezli webmail ile aynı origin'de tarayıcının Basic önbelleği
  üzerinden XSS'in DAV'a ulaşması gerçek risk. Ürünün ters vekili Host izin
  listesi kullandığından ek maliyet düşüktür.

- **Soru 5 — başlık okuma/ETag kapsamı:** ilk dilimde yalnız DAV sınıfında;
  webmail'in ihtiyacı yok, dar yüzey denetimi kolaylaştırır.
- **Soru 6 — `If-Match`/`If-None-Match` türü:** `Liste<Metin>`; RFC 9110
  virgülle ayrılmış liste ve `*` izin verir, `Metin` ürünü hataya açık
  ayrıştırmaya zorlar. `*` ayrı bir değer olarak tanınmalı
  (`If-None-Match: *` oluşturma korumasıdır; iOS PUT'ta kullanır).
- **Soru 9 — `--deneysel-web`:** Basic yalnız loopback'te kabul edilmeli;
  aksi hâlde DAV geliştirmede uçtan uca sınanamaz (ürün testleri bu
  profilde koşuyor).
- **Soru 10 — `Sec-Fetch-Site`:** `cross-site` reddi de olmalı; tarayıcı
  Basic'i önbellekten kendisi ekler, `Origin` tek başına yetmez. DAV
  istemcileri `Sec-Fetch-*` göndermediğinden yanlış ret riski yoktur.
- **Soru 11 — salt okuma:** ilk dilimde olmalı; ürünün temsilci düzeyleriyle
  (okur/düzenler) doğrudan eşleşir, PUT/DELETE 403 maliyeti küçüktür.
- **Soru 12 — başarılı DAV kovası:** ilk uygulama diliminde ürünle birlikte
  ölçülecek (iOS/Thunderbird ilk eşitleme).
- **Soru 13 — 405 + `Allow` ve 501:** bütün rotalarda; tanımsız yönteme
  verilen yanıtın tutarlılığı güvenlik ve istemci davranışı için iyidir.

Ürün tarafında iCalendar üretimi ve vCard 4.0 içe/dışa aktarma hazırdır.
Ürün görüşü olmayan açık soru kalmadı; 12 ölçüm bekler.

## Dört soru süzgeci

1. **Türkçe doğal mı?** — `dav istemcisi gerekli`, `oturum gerekli`
   ritmini izler; `isteğin "Depth" başlığı`, `isteğin ham gövdesi` ile aynı
   iyelik kalıbıdır. Yöntem ve başlık adları protokol sabitidir.
2. **Deterministik mi?** — Başlık değeri, kimlik kararı ve ETag IO izine
   girer; gizliler maskelidir; hermetik adaptör aynı kuralları uygular.
3. **Öğrenilebilir mi?** — Beş kavram: sınıf cümlesi, iki yöntem, başlık
   okuma, türlü + durumlu yanıt, ETag. Protokol mantığı Zee kodudur.
4. **Profesyonel ölçekte savunulabilir mi?** — Uygulama parolası Apple,
   Google, Fastmail ve Nextcloud'un modelidir; CSRF muafiyeti tarayıcı
   önuçuş modeline dayanır; güven başlıkları yapısal olarak Zee'ye kapalıdır.

## İlgili kayıt

K-364 (backlog ve karar günlüğü). Uygulama yok; bu belge RFC-0040'ın ilk
dilimini, sıra sapmasının ürün kanıtını, tehdit modelini ve kabul
ölçütlerini önden bağlar.
