# ADR-163 — İkili dosya yanıtı, multipart yükleme ve SMTP ek türü kaydı

- **Durum:** kabul
- **Tarih:** 23 Kasım 2026
- **İlgili kayıt:** K-378, K-373, K-355 (`urun-h`, ZM-F022), RFC-0041,
  RFC-0040 katman C, RFC-0036, ADR-059 (core freeze), ADR-064 (dil yüzeyi),
  ADR-096 (ekli e-posta), ADR-159 (imza okuma), ADR-160 (rota önbelleği),
  spec/11, spec/26, spec/23, GB-154, GB-155, GB-156; ek: K-385 (ZM-F033),
  GB-160; ek: K-406 (ZM-F048), GB-177

## Bağlam

Etkin dogfood ürünü `urun-h` (ZeeMail; ürün sürtünme kaydı ZM-F022) web
arayüzünde ek indirme, JavaScript'siz tarayıcı yüklemesi ve PDF/görsel
dışındaki eklerle e-posta gönderimi istiyor. RFC-0041 (K-373) üç eksiği
kaynakla doğruladı: rota yalnız UTF-8 metin/HTML yanıt verebiliyor; spec/26
yalnız `application/octet-stream` + `X-Zee-CSRF` kabul ediyor ve HTML formu
ikisini de gönderemiyor; `zee-smtp-mime-1` dört tür dışını yasaklıyor ve
genişletme yolu yok. 23 Kasım 2026 merkez kararıyla ZeeMail yüzeyleri
dogfood-change istisnasıyla sırayla açılıyor; bu ADR adım 1'dir. Canlı 17 site
ve resmî sitenin davranışı değişmemelidir: her yeni yüzey isteğe bağlıdır.

## Karar

### Ortak içerik türü kaydı (RFC-0040 katman C ile tek model)

`dil::icerik_turu` tek, kapalı tablodur. Her satır kanonik kısa ad, küçük harf
MIME, uzantılar, metin bayrağı (`charset=utf-8` ve UTF-8 strict), isteğe bağlı
içerik imzası ve kullanılabileceği yüzeyleri (web dosya yanıtı, SMTP
varsayılanı, SMTP adayı) taşır. İlk satırlar: pdf, png, jpeg, webp, txt, csv,
ics, docx, xlsx, pptx, zip ve yalnız web için `application/octet-stream`.
HTML, SVG, XHTML, betik, yürütülebilir ve makro taşıyan türler tabloda yoktur;
yeni satır K-167 `bicim icerik-turu.<ad>` kaydı ve ayrı karar ister. RFC-0040
katman C'nin metin sınıfı (`application/xml`, `text/vcard` …) açıldığında bu
tabloya satır olarak girer; ayrı kayıt açılmaz. RFC-0041'in son açık sorusu
böyle kapanır: DAV türleri ortak alt RFC'ye değil, kendi işlerinde satır
olarak girer.

### A — Dosyadan akışlı ikili yanıt

1. **Yüzey tek contextual Sonuç kalıbı:**
   `sonuç <yol> dosyasını <tür> türünde <ad> adıyla ek olarak göndermeyi dene olsun`
   → `TamSayı sonucu` (bayt). Yeni AST/değer türü yoktur; intrinsic
   `web.dosya_yaniti` (`dosya-okuma`, etki `WebAdaptoru` → eylemde T044).
   Yeni kalıp kelimeleri `türünde`, `adıyla`. Hata C013 başarısız Sonuç'tur
   (dosya yaşam döngüsü ailesi); yeni tanı kodu yoktur.
2. **Yalnız `attachment`.** `inline` (PDF/görsel önizleme) bu adımda
   açılmadı: ürün ihtiyacı indirmedir ve `inline` sandbox CSP'nin PDF
   önizlemesiyle etkileşimi ölçülmeden açılmaz. `ETag`, `Range`/206 ve koşullu
   GET de açılmadı (RFC-0040 katman B başlık okuma listesine bağlı).
3. **Başlığı çalışma zamanı kurar.** `Content-Disposition: attachment;
   filename="<ASCII geri düşüş>"; filename*=UTF-8''<RFC 8187>`; SMTP'nin
   mevcut `yuzde_kodla` kodlaması ortak `icerik_turu::rfc8187_kodla`'ya
   taşındı. Ad doğrulayıcısı SMTP'ninkinden sıkıdır: CR/LF dahil denetim,
   çift tırnak ve Unicode yön denetimi de reddedilir.
4. **Dosya yanıtı başlıkları sabittir:** `nosniff`, `no-store`,
   `Content-Security-Policy: default-src 'none'; sandbox`,
   `Cross-Origin-Resource-Policy: same-origin`. K-335 önbellek izni dosya
   yanıtına uygulanmaz.
5. **Yol ve tavan.** spec/26 proje dosya sınırı + SMTP ek okuyucusunun
   `dosyayi_guvenle_ac`'ı (readonly, `O_NOFOLLOW`, açılış öncesi/sonrası kimlik
   karşılaştırması). Tavan 16 MiB (`yukleme_bayti`, yüklemeyle simetrik);
   proje bildirimiyle yükseltme açılmadı. K-401 (ADR-167 eki, 25 Kasım
   2026) dosya yanıtı tavanını çözerek yazma hedef tavanıyla eşit 32 MiB'a
   çıkardı; yükleme 16 MiB kalır.
6. **Yazım bütçesi (RFC-0041 açık sorusu).** Ayrı formül açılmadı: K-372
   mutlak bütçesi `10 sn + bayt / 128 KiB/s` (16 MiB için ~138 sn) ve 10 sn
   ilerlemesizlik kesmesi başlık + gövdenin tamamına tek son tarih olarak
   uygulanır. Gövde 16 KiB parçalarla akar, belleğe ve heap zarfına girmez.
   Akış sırasında kısalan ya da son parçada uzunluğu değişen dosya bağlantıyı
   eksik `Content-Length` ile keser.
7. **İlk yanıt kazanır.** Dosya yanıtı isteğin yanıt taslağıdır; yanıtı
   belirlenmiş istekte başarısız Sonuç'tur. Taslak açık dosya tanıtıcısını
   taşır; istek geri alınırsa tanıtıcı kapanır, commit sonrası yazılır
   (K-327 kopma kuralı aynen).
8. **İz ve hermetik adaptör.** IO izi `dosya_yaniti_gonder(yol, tür, ad)` →
   bayt sonucu; içerik ize girmez. `ToplayanIo` aynı kayıt/ad/tavan kuralını
   uygular, gövdeyi metin dosyası olarak yazar ve (tür, disposition) çiftini
   kaydeder.

### B — `multipart/form-data` yükleme

1. **Açılış proje bildirimiyle:** `çok_parçalı_yükleme doğru olsun`
   (`web-oturumu` zorunlu, P015; değer mantıksal, P001). Rota politikası
   cümlesi (`dosya yüklemesi kabul edilir`) açılmadı: grammar değişikliği
   gerektirirdi ve spec/26 octet-stream yolu da rota başına değil süreç
   başına açıktır. Bildirimsiz projede multipart davranışı değişmez (415
   açılmadı; davranış değişikliği olurdu).
2. **Ayrıştırıcı** `web_cok_parcali` (CLI adaptörü; saf, birim testli):
   tek geçiş, belleğe alma yok, sınır arama tamponu `ayraç - 1` bayt
   (ayraçta CR yalnız başta olduğundan arama doğrusal). Sınırlar: ≤ 8 parça,
   tam bir dosya, başlık bloğu ≤ 8 KiB/≤ 8 satır, metin alanı ≤ 8 KiB UTF-8,
   alan adı ≤ 128 bayt, önsöz/sonsöz ≤ 1 KiB, zarf `Content-Length` ≤
   16 MiB + 64 KiB, dosya ≤ 16 MiB (413). Tanınan başlıklar yalnız
   `Content-Disposition` ve `Content-Type`; `Content-Transfer-Encoding`,
   katlama, bilinmeyen parametre, yinelenen ya da `yukleme_` önekli alan
   adı 400 (proxy/Zee ayrıştırma farkını ve alan zehirlemesini kapatır).
3. **CSRF dosyadan önce.** Multipart isteğinde oturum gövdeden önce yüklenir
   (`web_istegini_baslat` yalnız bu yolda öne alınır); `_csrf` alanı dosya
   parçasından önce gelmeli ve oturumla sabit zamanlı eşleşmeli, aksi hâlde
   dosya baytı diske yazılmadan 403 (başarısız deneme CSRF oran sınırına
   sayılır). Oturum çerezi yoksa gövde okunmaz. `X-Zee-CSRF` bu yolda yok
   sayılır. Rota denetimi belirteci ayrıca karşılaştırır.
4. **Tek ilkel.** Dosya spec/26 `AkanYukleme`'ye akar; rota `yukleme_*`
   alanlarına ek olarak `yukleme_dosya_adi` (doğrulanmış basename ya da boş)
   ve `yukleme_bildirilen_tur` (güvenilmez) görür. Red yanıtından önce kısmi
   `.parca` silinir.
5. **Okuma bütçesi değişmez:** 10 sn mutlak istek son tarihi; orantılı okuma
   bütçesi ayrı karar (yavaş bağlantıda büyük yükleme 408).

### C — SMTP ek türü kaydı

1. **Bildirim:** `e_posta_ek_türleri "pdf", "docx" listesi olsun` kümeyi tam
   belirler (varsayılanlar örtük eklenmez; kurum PDF'yi de kapatabilir).
   Yalnız `icerik_turu` SMTP adayları (11 tür); makro/eski Office/
   yürütülebilir/betik/HTML/SVG kayıtta yoktur, bildirimle açılamaz. Hata
   yeni tanı **P019**; `ağ` yoksa P015. Alan yoksa dört varsayılan.
2. **Profil etiketi değişmez** (`zee-smtp-mime-1`): tel biçimi aynı, yalnız
   tür kümesi bildirimle genişler; ayrı `zee-smtp-mime-2` iki profil kanıtı
   doğururdu.
3. **İçerik denetimi:** OOXML/zip için `PK\x03\x04`, metin türleri için
   NUL'suz UTF-8; `text/*` `charset=utf-8`. Varsayılan dört türün içerik
   denetimi eklenmedi (bugünkü davranış bayt bayt aynı). Bu yalnız uzantı
   sahteciliğini daraltır.
4. **Ölçülebilir yerel ret:** ekler SMTP ayarı çözülmeden ve ağdan önce
   okunur; tür/imza/UTF-8/boyut/ad/erişim reddi C029 içinde sabit
   `MIME e-posta gönderilemedi: ek reddedildi (<neden>)` metniyle görünür.
   Neden yalnız sabit metinlerdir (yol, sunucu, kimlik yok). SMTP/URL/kimlik
   hataları K-236 maskesinde kalır. `dil denetle` uyarısı açılmadı.
5. **Çağrı başına dar liste açılmadı:** ürün kurum politikasını gönderimden
   önce Zee kodunda uygular; proje tavanı bildirimde tek yerde kalır.
6. **Antivirüs:** seçenek (a) — ürün/operatör; dile tarayıcı protokolü girmez.

## Sonuçlar

- ZeeMail ek, vCard ve CSV indirmesini rotadan, yetki kararı Zee kodunda
  kalarak verebilir. Bildirimsiz/kalıpsız projenin yanıtı bayt bayt aynıdır.
- Kullanıcı kaynaklı dosya hiçbir koşulda origin'de belge/betik olarak
  yorumlanmaz (tür kaydı + sandbox CSP + `attachment` + `nosniff`).
- IDOR (hangi dosyanın kime verildiği) ürün sorumluluğudur; dil yetki kararı
  vermez (GB-154 bilinçli sınır).
- ZeeMail docx/xlsx/pptx/zip/txt/csv/ics ekli ileti gönderebilir; bildirimsiz
  projelerde dört tür kuralı aynıdır; yerel ek reddi artık ağ hatasından
  ayırt edilir (GB-156).
- ZeeMail JS'siz formla ek ve kişi dosyası yükleyebilir; bildirimsiz
  projelerin multipart davranışı aynıdır. Sınırlar ve CSRF sırası GB-155.

## Ek — K-385: istemci yapılandırma türleri (24 Kasım 2026)

**Bağlam.** `urun-h` (ZeeMail) sürtünmesi ZM-F033: webmail Thunderbird
otomatik yapılandırma XML'ini (`/.well-known/autoconfig/mail/config-v1.1.xml`,
`autoconfig.<alan>/mail/config-v1.1.xml`) ve Apple yapılandırma profilini
(`.mobileconfig`) sunmak istiyor. Kayıtta ikisinin türü yoktu: XML rotası
C013 ile 404'e düşüyor, profil `application/octet-stream` olarak iniyor ve
iOS Safari profil kurulum akışını başlatmıyordu. İçerik sır taşımaz.

**Birincil kaynak ölçümü (24 Kasım 2026).**

- Thunderbird (comm-central `FetchHTTP.sys.mjs`): yanıt `Content-Type`
  değeri `text/xml`, `application/xml` ya da `text/rdf` ile başlıyorsa XML
  olarak ayrıştırılır, aksi hâlde metin döner ve `readFromXML` yapılandırmayı
  reddeder. İstek arka plan XHR'ıdır (GET); `Content-Disposition` hiç
  okunmaz, CSP/sandbox belge olmayan yanıta uygulanmaz. IETF taslağı
  (draft-ietf-mailmaint-autoconfig-07 §3) `text/xml` yazar; istemci ikisini
  eşit kabul eder.
- RFC 7303: `text/xml` `application/xml`'in takma adıdır; `application/xml`
  karışıklığı önlemek için ÖNERİLİR. Kodlamayı BOM, sonra `charset`
  parametresi belirler; UTF-8 BOM'suz ÖNERİLİR.
- Apple (Over-the-Air Profile Delivery): Safari'nin yanıtı yapılandırma
  profili olarak işlemesi için tür `application/x-apple-aspen-config`'tir;
  dosya `.mobileconfig` uzantısını korur. `Content-Disposition` hakkında
  Apple birincil kaynağı yoktur.
- Outlook POX Autodiscover (MS-OXDSCLI §2.1, §3.2.5): istek `POST`'tur,
  yanıt istekteki `EMailAddress` kullanıcısını anlatır; GET yalnız 302
  yönlendirmesi içindir. Statik dosya yanıtı bu istemciye uymaz.

**Karar.**

1. Kayda iki satır eklenir: `xml` → `application/xml`, `mobileconfig` →
   `application/x-apple-aspen-config`. İkisi de **yalnız web dosya
   sınıfındadır** (`smtp_aday` yok); gövde ayrıştırılmaz, imza denetimi
   yoktur. K-167 `bicim icerik-turu.xml` ve `bicim icerik-turu.mobileconfig`.
2. **`text/xml` değil `application/xml`.** Kapalı kayıtta tek yazım
   tutulur (exact eşleşme, takma ad yok). RFC 7303 önerisine uyar ve
   kaydın `text/*` satırlarına eklediği `charset=utf-8`, XML'in kendi kodlama
   bildirimini geçersiz kılardı. Thunderbird iki yazımı eşit kabul eder.
   `+xml` sonekli öteki türler (XSLT, RSS, XHTML, SVG) açılmaz.
3. **`charset` eklenmez.** XML kodlamasını BOM/bildirim taşır; imzalı profil
   ikili CMS'tir.
4. **Yalnız `attachment`; `inline` açılmadı.** Ölçülen istemcilerin hiçbiri
   `Content-Disposition` okumaz (Thunderbird XHR'ı) ya da `inline` istemez
   (Apple tür ve uzantıya bakar). `inline` XML'i tarayıcıda belge olarak
   oluşturur; sandbox CSP betiği durdursa da gereksiz yüzeydir.
5. **Outlook POX Autodiscover açık sınırdır.** POST rotası `_csrf` ister
   (spec/12 §3, `herkese açık` kapatmaz) ve `--web-proxy` profilinde
   `Origin` ZORUNLUDUR; Outlook ikisini de göndermez → 403. Aynı engel
   RFC-0040 #5(c) CalDAV/CardDAV için doğrulanmıştır; dar CSRF muafiyeti o
   işte tek model olarak karara bağlanır. Ürün bugün yalnız `autodiscover`
   kaydını DNS/vekil katmanında karşılayabilir.

**Güvenlik değerlendirmesi (GB-160).** XML sunucuda ayrıştırılmaz: XXE/DTD
riski Zee tarafında yoktur. Tarayıcıda `attachment` belge oluşturmaz;
oluşsa bile `default-src 'none'; sandbox` betik ve alt kaynak yüklemesini,
`nosniff` tür tahminini engeller. Profilin içeriği (gömülü parola, kök
sertifika, imza) ürün sorumluluğudur; kurulum iOS'ta kullanıcı onayı ister.
Ölçümde görülen açık: HTML standardı yanıt CSP'sindeki `sandbox`'ı
gezinme sandbox bayraklarına katar ve `allow-downloads` olmadan
`attachment` indirmesini engeller; Chromium ve WebKit bugün bu denetimi
uygulamaz (bayraklar çerçeveden alınır). Değişiklik bütün dosya yanıtlarını
etkilediği için bu ekte yapılmadı; `sandbox allow-downloads` önerisi ayrı
iştir. iOS cihazda profil kurulumunun insan ölçümü de açıktır.

**Sonuç.** ZeeMail autoconfig XML'ini ve Apple profilini kayıtlı türle
sunar; bildirimsiz/kalıpsız projeler ve mevcut on iki tür bayt bayt aynıdır.

## Ek — K-406: türlü metin yanıtı (26 Kasım 2026)

**Bağlam.** `urun-h` (ZeeMail) sürtünmesi ZM-F048: ürün takvimleri
`GET /takvim/yayin.ics?k=<belirteç>` adresinde yayımlıyor (Outlook "takvim
yayımla", Google "gizli iCal adresi" abonelikleri). Takvim gövdesi bellekte
kurulan bir `Metin`dir. Bellekteki metin yanıtı yalnız `text/html` (gövde `<`
ile başlıyorsa) ya da `text/plain` olabiliyordu; `text/calendar` yalnız dosya
yanıtında vardı ve GET rotası dosya yazamaz (T045, doğru kural). Ürün takvimi
`text/plain` ve LF satır sonlarıyla veriyor; katı abonelik istemcileri bunu
reddedebilir. Merkez 26 Kasım 2026 09:05'te küçük yüzey olarak onayladı.

**Birincil kaynak.** RFC 5545 §3.1: içerik satırları CRLF ile ayrılır; §8.1
medya türü `text/calendar`, `charset` varsayılanı UTF-8. RFC 4180 §2 CSV
kayıtları için CRLF yazar. RFC 7303: `application/xml`. Abonelik
istemcilerinin `text/plain`/LF takvime gerçek davranışı (Outlook, Google)
bu ekte ölçülmedi; insan ölçümü açıktır.

**Karar.**

1. Yanıt cümlesine tek kardeş biçim: `<ifade> metnini "<tür>" türünde gönder`
   (`Cumle::YanitGonder` + `tur`; yeni AST türü yok). Yeni kalıp kelimesi
   yalnız `metnini`; `türünde` K-378 dosya yanıtının kelimesidir.
2. **Tür aynı kapalı kayıttan, daha dar alt kümeyle.** `WEB_METIN_TURLERI` =
   `txt`, `csv`, `ics`, `xml` — UTF-8 metin taşıyan ve tarayıcıda betik
   yürütmeyen dosya sınıfı satırları. İkili türler, `octet-stream` ve Apple
   profili (imzalı hâli ikili CMS) dışarıdadır; HTML/SVG/XHTML/betik kayıtta
   hiç yoktur. Tür kaynakta görünen literaldir (durum kodu gibi review'da
   exact görünür); aksi önerili S037 — yeni tanı kodu açılmaz. K-167
   `bicim web-metin-yaniti-turleri`.
3. **Başlıklar dosya yanıtıyla aynı**, yalnız `Content-Disposition` yok:
   `nosniff`, `no-store`, `default-src 'none'; sandbox`, CORP same-origin;
   `charset` kuralı `baslik_degeri` ile ortak. Rota önbellek izni (K-335)
   uygulanmaz: belirteçli takvim adresi paylaşılan önbelleğe girmemelidir.
   `attachment` biçimi abonelik istemcilerinde gerekmediği için açılmadı.
4. **CRLF için yeni sözdizimi yok.** Gövde bayt bayt gider; satır sonu
   dönüştürülmez. Seçenekler: (a) `CRLF satır sonlarıyla` niteleyicisi — üç
   yeni kalıp kelimesi ve normalleştirme semantiği (çıplak CR/LF ne olur?);
   (b) türe göre örtük CRLF normalleştirmesi — kaynakta görünmeyen dönüşüm,
   ürünün kurduğu baytı değiştirir; (c) K-402 (RFC-0043 Dilim 2) `13'ün kod
   noktası harfini almayı dene` ile CR'yi ürün üretir. (c) en küçük yüzeydir
   ve K-402 bunu açıkça amaçlar; seçildi. K-402 birleşmeden önce de istek ve
   dosya verisinden gelen CR olduğu gibi geçer.
5. Cümle K-403 kesin yanıtıdır, eylem içinde T044'tür; IO izi
   `turlu_yanit_gonder` (tür, gövde) olayı, şema ve hermetik adaptör aynı
   kapalı kümeyi uygular. Desteklemeyen adaptörün varsayılanı türü düşürüp
   `text/html` koklamasına açılmaz: fail-closed 500 durum yanıtı.

**Güvenlik değerlendirmesi (GB-177).** Risk: kullanıcı verisi taşıyan
bellek metninin tarayıcıda belge/betik olarak yorumlanması (XSS), tür
başlığına enjeksiyon, gizli belirteçli adresin paylaşılan önbelleğe girmesi.
Tür derlemede kapalı kümeden literaldir, çalışma zamanı başlık değerini
kayıttan kurar (Zee metni başlığa girmez); `application/xml` gövdesi
`<script>` taşısa da `nosniff` + sandbox CSP betiği durdurur ve gövde `<` ile
başlasa `text/html` seçilmez; `no-store` önbellek iznine rağmen korunur.
Belirtecin gizliliği, iptali ve oran sınırı ürün sorumluluğudur.

**Sonuç.** ZeeMail takvimi `text/calendar; charset=utf-8` ile sunar; mevcut
yanıt biçimleri ve dosya yanıtı bayt bayt aynıdır.

## Değerlendirilen seçenekler

- **Genel başlık/yanıt builder'ı:** V1 kararıyla reddedildi.
- **`Metin` yanıtta base64/data URL ya da `X-Accel-Redirect`:** RFC-0041
  Alternatifler'deki gerekçeyle reddedildi.
- **Ayrı dosya yanıtı yazma bütçesi:** K-372 formülü 16 MiB'ı zaten
  kapsıyor; ikinci formül iki ayar ve iki kanıt yolu yaratırdı.
- **Hata kodu C022 (web güvenlik adaptörü):** ad/tür reddi dosya yaşam
  döngüsü Sonuç ailesiyle (C013) aynı yakalama kalıbını kullanır; ürün tek
  `başarısızsa` dalıyla 404/415 verir.
