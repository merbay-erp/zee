# RFC-0041 — Web adaptöründe ikili dosya yanıtı, multipart yükleme ve SMTP ek türü kaydı

- **Durum:** kabul — 23 Kasım 2026, K-378 (`dogfood-change`, `urun-h`
  ZM-F022). Katman A **uygulandı** (spec/11 §4 “Dosya yanıtı”), katman B
  **uygulandı** (spec/26 “Multipart form yüklemesi”); katman C aynı işin
  sonraki dilimidir. Karar [ADR-163](../adr/163-ikili-yanit-multipart-ve-ek-turu-kaydi.md);
  açık soruların kapanışı aşağıda “K-378 kararları”nda.
- **Tarih:** 23 Kasım 2026
- **İş:** K-373 (taslak), K-378 (uygulama)
- **İlgili belgeler:** [spec/11](../spec/11-uygulama-eylemleri-ve-web-adaptoru.md),
  [spec/12](../spec/12-web-guvenlik-profili.md),
  [spec/21](../spec/21-deterministik-io-izi.md),
  [spec/23](../spec/23-yetkinlik-ve-outbound-guvenligi.md),
  [spec/24](../spec/24-kaynak-guvenlik-profili.md),
  [spec/26](../spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md),
  [spec/27](../spec/27-uyumluluk-ve-surumleme.md),
  [ADR-036](../adr/036-kanonik-web-proxy-origin-siniri.md),
  [ADR-037](../adr/037-byte-tabanli-http-istek-siniri.md),
  [RFC-0017](0017-web-oturum-ve-csrf.md),
  [RFC-0027](0027-binary-yukleme-ve-dosya-yasam-dongusu.md),
  [RFC-0034](0034-ekli-eposta.md),
  [RFC-0036](0036-dosya-imzasi-okuma.md),
  [RFC-0037](0037-rota-onbellek-basligi.md),
  [RFC-0039](0039-ag-sunucusu-yetkinligi.md),
  [RFC-0040](0040-webdav-caldav-carddav-adaptoru.md)
- **Freeze sınıfı adayı:** her katman için ayrı `dogfood-change`; bu taslak
  kendi başına hiçbir dil, CLI, tanı ya da API yüzeyi açmaz.

## Özet

E-posta ürünü (dogfood `urun-h`, ZeeMail; ürün sürtünme kaydı ZM-F022) web
arayüzünde ek **indirme**, tarayıcıdan JavaScript'siz ek **yükleme** ve PDF/
görsel dışındaki türlerde ek **gönderme** istiyor. Üç ihtiyaç da bugün
karşılanmıyor. Bu RFC üç eksiği e63a439 kaynağıyla tek tek doğrular ve onları
birbirinden bağımsız denetlenebilir üç katmana böler:

- **A — Dosyadan akışlı ikili yanıt:** rota, kayıtlı bir içerik türüyle,
  `Content-Disposition` başlığıyla ve bayt tavanıyla proje içindeki bir dosyayı
  yanıt gövdesi olarak akıtır. İçerik türü ve başlıklar
  [RFC-0040](0040-webdav-caldav-carddav-adaptoru.md) katman C'nin kapalı
  **yanıt türü/başlık kaydıyla tek model** olarak tanımlanır.
- **B — `multipart/form-data` yükleme:** HTML formu tek dosya alanı ve `_csrf`
  alanıyla gönderir; çalışma zamanı gövdeyi akışla ayrıştırır, dosya parçasını
  spec/26'nın `.parca` + SHA-256 geçici dosya modeline yazar ve CSRF'yi form
  alanından okur.
- **C — SMTP ek türü kaydı:** `zee-smtp-mime-1`'in PDF/PNG/JPEG/WebP
  allowlist'i **varsayılan olarak değişmeden** kalır; proje bildirimi
  kapalı bir aday kayıttan ek türleri açar.

İlke RFC-0039/0040 ile aynıdır: ürün mantığı (yetki, hangi dosyanın kime
verildiği, ek kaydı, antivirüs politikası) Zee kodudur; çalışma zamanı yalnız
HTTP/MIME zarfını, sınırları ve güvenli ayrıştırmayı sağlar. `--web-proxy`
(ZeeOS üretimi) ve `--deneysel-web` aynı yanıt ve yükleme sözleşmesini
kullanır; ayrım yalnız bugünkü Origin/HTTPS/çerez farklarıdır.

## Motivasyon

ZeeMail web arayüzü gelen iletinin ekini kullanıcıya indirtmek, yazılan
iletiye tarayıcıdan dosya eklemek ve bu eki SMTP ile göndermek zorundadır;
bunlar bir e-posta ürününün temel akışıdır. Ürün oturumu spec/11, 12, 23 ve
26'yı okuyarak üç eksik çıkardı (ZM-F022). Ürün ayrıca ekleri ileride
IMAP (RFC-0039) ve CalDAV eki (RFC-0040) yollarında da sunacağı için yanıt
yüzeyinin bu iki taslakla tek model olmasını istiyor.

### Bugünkü durum (e63a439 üzerinde doğrulandı)

Sonuç sütunu **yok / kısmi / var** biçimindedir.

| # | İddia | Sonuç | Kanıt |
|---|---|---|---|
| 1 | Rota yalnız metin/HTML yanıt verebiliyor; ikili/akışlı dosya yanıtı, izinli içerik türü, `Content-Disposition`, boyut sınırı yok | **Doğrulandı — yok.** Yanıt taslağı kapalı üç biçimdir: `Govde(String)`, `Durum(u16, String)`, `Yonlendirme(String)`; gövde her zaman UTF-8 `Metin`dir. İçerik türü gövdenin ilk karakterinden seçilir (`<` → `text/html`, aksi `text/plain`), `charset=utf-8` sabittir. `Content-Disposition` hiçbir web yanıtında yazılmaz; rota başlık yazamaz ("dinamik status/header builder … açmaz"). Dosya içeriğini yanıta akıtan bir yüzey yoktur; ikili bir dosyanın içeriği `Metin` olarak kurulamaz (UTF-8 dışı bayt, spec/24 heap zarfı). `X-Content-Type-Options: nosniff` **zaten** her yanıtta vardır (`Cache-Control: no-store` ile birlikte); eksik olan tür ve ek başlığıdır. spec/11 §7 K-163/F032 maddesi "Bu ürün kanıtı dynamic response API gerekçesi değildir" der; yani bugünkü durum bilinçli bir sınırdır. | spec/11 §4 (içerik türü kuralı; dinamik builder yok), §7; `compiler/src/main.rs` `enum WebYanitTaslagi`, `web_yanit_taslagini_yaz`, `guvenlik_basliklari`, `html_guvenlik_basliklari` |
| 2 | Tarayıcıdan JS'siz yükleme yok: spec/26 yalnız ham `application/octet-stream` + `X-Zee-CSRF` kabul ediyor; HTML formu ikisini gönderemez; multipart/form-data gerekiyor | **Doğrulandı — yok.** Binary yol yalnız tam `Content-Type: application/octet-stream` ile açılır; CSRF tek `X-Zee-CSRF` başlığından okunur ve K-332 ön kapısı bu başlık + oturum çerezi yoksa gövdeyi okumadan 403 verir. HTML formu `enctype` olarak yalnız `application/x-www-form-urlencoded`, `multipart/form-data`, `text/plain` gönderebilir ve özel başlık ekleyemez. `multipart/form-data` gövdesi octet-stream olmadığı için 64 KiB gövde tavanına düşer ve sığarsa `Content-Type`'tan bağımsız olarak URL-form olarak çözülür; `_csrf` parça başlıklarının içinde kaldığı için alan bulunamaz ve küçük multipart form **403** alır (ürün ölçümü, ZeeMail `POST /kisiler/ice-aktar`; yerelde `--deneysel-web` ile yeniden üretildi). 64 KiB üstü gövdede 413 yazılır ama bağlantı gövde okunmadan kapatıldığından istemci çoğunlukla **yanıt almaz** (aşağıdaki ek bulgu). Multipart ayrıştırıcı yoktur; spec/26 "Açık sınır" bölümü multipart'ı açıkça "bu bölümün sözü değil" sayar. | spec/26 (Binary HTTP gövdesi, K-332, Açık sınır); spec/11 §4–5; spec/12 §3; `compiler/src/main.rs` `binary_yukleme` seçimi, `tek_http_basligi(baslik, "X-Zee-CSRF")`, yükleme sonrası `_csrf=` sentezi; `compiler/src/yorumlayici.rs` `istek_govdesiyle` |
| 3 | `zee-smtp-mime-1` PDF/PNG/JPEG/WebP dışındaki ekleri yasaklıyor; docx/xlsx/pptx/zip/txt/csv/ics gerekiyor; kurum/ürün ayarıyla genişlemeli | **Doğrulandı — var (yasak), genişletme yolu yok.** spec/23 "PDF/PNG/JPEG/WebP dışında ek tipi YASAKTIR. Bu etiket whitelist'idir; içerik/antivirüs doğrulaması AÇIKTIR." Kodda `EkTuru` kapalı dört değerli enum'dur; tür **yalnız dosya adı uzantısından** seçilir (`pdf`, `png`, `jpg`/`jpeg`, `webp`), aksi "MIME ek türü PDF/PNG/JPEG/WebP olmalı" hatası → C029. Proje bildiriminde ek türü alanı yoktur (geçerli alanlar listesinde yalnız `e_posta_bağlantı_değişkeni`). Yasak yalnız çalışma zamanındadır: `dil denetle` `.docx` ekli çağrıyı temiz geçer. Tür reddi ağa çıkmadan (`ekleri_oku`, SMTP bağlantısından önce) verilir, ama bütün hatalar aynı maskeli C029 metnine (“MIME e-posta gönderilemedi; sır içeren ayrıntılar gizlendi…”) çevrildiği için kullanıcı düzeyinde **tür reddi, SMTP ayarı eksikliği ve ağ hatası ayırt edilemez** (ürün ölçümü: `.docx` ve SMTP sunucusu yokken `.pdf` aynı sonucu verir). İçerik imzası (magic bytes) denetlenmez; RFC-0036'nın `ilk N baytını onaltılık okuma` yüzeyi ürüne bu denetimi yaptırabilir ama MIME katmanına bağlı değildir. | spec/23 "Ekli SMTP e-posta (K-232)"; `compiler/src/eposta/mime.rs` `enum EkTuru`, `EkTuru::dosya_adindan`, `ekleri_oku`; `compiler/src/proje.rs` geçerli alan listesi |

Ek bulgular (ürün listesinde yoktu):

- SMTP ek adı `ek_adini_denetle` ile tek basename'e zorlanır ve
  `filename*=UTF-8''…` için `yuzde_kodla` yalnız `[A-Za-z0-9._-]`'yi düz
  bırakır. Bu, RFC 6266/8187 kaçışının **zaten çalışan** bir örneğidir; katman
  A'nın web `Content-Disposition` kodlaması aynı fonksiyonu paylaşmalıdır.
- Yanıt yazma soketi 10 sn ile sınırlıdır (spec/11 §5). 16 MiB'lık bir eki
  yavaş bağlantıda 10 sn'de yazmak her zaman mümkün değildir; katman A'nın
  yazma bütçesi ayrıca tanımlanmalıdır (Açık sorular). *K-372 sonrası:*
  sabit 10 sn yerine mutlak `10 sn + bayt / 128 KiB/s` bütçesi (16 MiB için
  138 sn) ve 10 sn ilerlemesizlik kesmesi geçerlidir; katman A ek hız
  gereksinimini bu bütçeyle karşılaştırır.
- **64 KiB üstü gövdede yanıtsız kapanma (önceden var olan, içerik türünden bağımsız).** `govde_zarfini_denetle` `Content-Length` sınırı aşınca 413'ü yazar ve bağlantıyı gövdeyi okumadan/boşaltmadan kapatır. Alım tamponunda okunmamış bayt varken kapanan TCP soketi RST gönderir; gövdenin tamamını hemen yollayan istemci 413'ü okuyamadan `Connection reset by peer` alır. Yerelde (`--deneysel-web`, curl 8.7.1, 70 KB gövde) multipart ve urlencoded gövdenin ikisi de `000` verdi; yalnız `Expect: 100-continue` gönderen istemci 413'ü gördü. ZeeMail'deki “urlencoded 413, multipart yanıtsız” farkı bu `Expect` davranışından gelir. Bu bir spec/11 §5 correctness boşluğudur (“413 döner” sözü istemcide gözlenemiyor) ve bu RFC'den bağımsız ayrı fix adayıdır: sınırlı süre/bayt boyunca gövdeyi boşaltıp atma (lingering close) ya da yazma yönünü önce kapatma. *K-374 ile kapandı:* yazma yönü kapatılır, artık en çok 2 sn / 16 MiB boşaltılır (spec/11 §5).
- `dil dene` hermetik web adaptörü de aynı üç biçimli yanıt modelini kullanır;
  yeni yüzey orada gerçek çalışma zamanıyla aynı kurallarla uygulanmalıdır.

Sonuç: bugün Zee ile yazılmış bir web e-posta istemcisi ek indiremez,
tarayıcıdan JS'siz ek yükleyemez ve PDF/görsel dışında ek gönderemez. Üç eksik
birbirinden bağımsızdır; hiçbiri yeni TCP yetkinliği gerektirmez.

## Amaç

1. Rotanın proje içindeki bir dosyayı, açık ve kayıtlı bir içerik türüyle,
   `inline` ya da `attachment` olarak, bayt tavanı altında ve belleğe tümüyle
   almadan yanıt olarak göndermesi.
2. Standart HTML formuyla (`<form method=post enctype=multipart/form-data>`)
   tek dosyanın ve `_csrf` alanının gönderilmesi; dosyanın spec/26 ile aynı
   güvenlik değişmezleriyle diske akması.
3. SMTP ek türlerinin, varsayılanı değiştirmeden, proje bildirimiyle kapalı bir
   aday kayıttan genişletilmesi.
4. Yanıt türü/başlık kaydının RFC-0040 ile tek model olması: bir türü veya
   başlığı iki RFC ayrı ayrı tanımlamaz.

## Kapsam dışı

- Keyfi başlık adı yazan genel yanıt/başlık API'si (V1 kararı korunur).
- `Range`/`206 Partial Content`, `If-Range`, sürdürülebilir indirme; ilk adım
  tam gövdedir (Açık sorular).
- Birden çok dosya alanı, `multipart/mixed` iç içe parçalar, `<input multiple>`;
  ilk adım tek dosya alanıdır.
- Sıkıştırma (`Content-Encoding: gzip`) ve `Transfer-Encoding: chunked` yanıt;
  istek tarafında `Transfer-Encoding` YASAĞI (spec/11 §5) değişmez.
- Antivirüs/içerik tarayıcısı, makro ayıklama, arşiv açma: ürün ya da operatör
  katmanıdır; bu RFC yalnız sınırı ve kancayı tanımlar.
- Nesne deposu (S3 benzeri), CDN ve imzalı URL.
- Gelen iletideki ekleri MIME'den ayrıştırma (IMAP/SMTP alıcı tarafı):
  RFC-0039 MIME alt RFC'sinin işidir; bu RFC yalnız diske yazılmış ekin web'de
  sunulmasını ve giden SMTP ek türünü kapsar.
- `Baytlar` değer türü: RFC-0039 katman D ilkesiyle tutamak (yol + özet + bayt)
  modelinde kalınır.

## Tasarım ilkeleri

1. **Kapalı kayıt, açık builder yok.** İçerik türü, `Content-Disposition`
   biçimi ve yazılabilir yanıt başlıkları kapalı kayıtlardır; her öğe K-167
   dil yüzeyi fixture'ına girer. Rota keyfi başlık adı ya da ham değer yazamaz.
2. **Varsayılan gevşemez.** Bugünkü metin/HTML yanıt kuralı, octet-stream
   yükleme yolu, `X-Zee-CSRF` ön kapısı ve SMTP dört tür allowlist'i
   bildirimsiz projede bayt bayt aynı kalır.
3. **Fail-closed.** Kayıtsız tür, tavanı aşan dosya, bozuk multipart, eksik
   `_csrf`, yasak dosya adı rota gövdesi çalışmadan ya da yanıt başlığı
   yazılmadan sınıflandırılmış durumla reddedilir; kısmi kabul yoktur.
4. **Tek ilkel.** Diskteki dosya + SHA-256 + bayt tutamağı RFC-0039 katman D
   ve RFC-0040 katman D ile aynı ilkeldir; yükleme (B) üretir, yanıt (A) ve
   SMTP eki (C) tüketir.
5. **Deterministik kanıt.** Yanıta akıtılan dosya yolu, özeti, bayt sayısı ve
   seçilen tür; ayrıştırılan multipart alan adları ve dosya tutamağı spec/21–22
   IO izine girer. Dosya içeriği ize girmez; replay özet karşılaştırmasıyla
   doğrulanır.

## Katmanlar

Her katman ileride kendi alt RFC'si, ADR'si, spec bölümü, olumlu/olumsuz
testleri, tanı kodları ve güvenlik bulgusu kayıtlarıyla açılır. Sözdizimi
örnekleri **yalnız fikir taslağıdır**; bağlayıcı değildir ve freeze yüzeyine
yazılmaz.

### A — Dosyadan akışlı ikili yanıt

**Yanıt türü kaydı (RFC-0040 katman C ile tek model).** RFC-0040 katman C,
rotanın açık içerik türü seçebildiği kapalı bir kayıt önerir (`application/xml`,
`text/calendar`, `text/vcard`, `text/html`, `text/plain`). Bu RFC aynı kaydı
**iki sınıflı** tek tabloya dönüştürmeyi önerir:

| Sınıf | Gövde kaynağı | Örnek türler | Güvenlik başlığı |
|---|---|---|---|
| metin | `Metin` (bugünkü yol) | `text/plain`, `text/html`, `application/xml`, `text/calendar`, `text/vcard` | `text/html` → HTML CSP; diğerleri `nosniff` + kısıtlı CSP |
| dosya | proje içi dosya tutamağı (bu RFC) | `application/pdf`, `image/png`, `image/jpeg`, `image/webp`, `text/plain`, `text/csv`, `text/calendar`, `application/zip`, OOXML türleri, `application/octet-stream` | her zaman `nosniff` + `Content-Security-Policy: default-src 'none'; sandbox` + `Cross-Origin-Resource-Policy: same-origin` |

Her satır tür adı, izinli sınıf(lar), varsayılan `Content-Disposition`
(`inline` yalnız `application/pdf` ve dört görsel tür için izinli, diğerleri
için `attachment` ZORUNLU), `charset` kuralı (yalnız `text/*` için `utf-8`) ve
K-167 giriş sürümünü taşır. Kayıtta olmayan tür yazılamaz; tür yazılmayan
metin yanıtı bugünkü ilk karakter kuralıyla kalır (geriye uyum). `text/html`,
`image/svg+xml`, `application/xhtml+xml` ve herhangi bir `*script*` türü dosya
sınıfında **yoktur**: kullanıcı kaynaklı dosya hiçbir koşulda origin'de HTML/
betik olarak yorumlanamaz. Ürün bu dosyaları `application/octet-stream` +
`attachment` ile verir.

**Dosya kaynağı.** Fikir taslağı:

```text
GET "/ek" adresine istek geldiğinde
    oturum gerekli
    yol ekin "yol" değeri olsun
    yol dosyasını "application/pdf" türünde "fatura.pdf" adıyla ek olarak gönder
```

- Yol, spec/26 dosya işlemleriyle aynı kuralla proje köküne göre çözülür:
  mutlak yol, `..` bileşeni, sembolik bağ (leaf ve ara bileşen), dizin, FIFO
  ve normal olmayan dosya reddedilir. `dosya-okuma` yetkinliği ZORUNLUDUR.
- Açılış SMTP ek okuyucusundaki (spec/23 K-232) gibi readonly ve
  bloklamayan yapılır; açılan handle kimliği ve uzunluk `fstat` ile alınır,
  `Content-Length` bu uzunluktur. Akış sırasında dosya kısalır ya da uzarsa
  bağlantı kesilir (eksik `Content-Length` başarı sayılmaz, spec/11 §5).
- Bayt tavanı: varsayılan 16 MiB (spec/26 yükleme tavanı ile simetrik),
  `KaynakSinirlari` tek sahibinde; proje bildirimiyle üst tavan ayrı karar
  (Açık sorular). Tavan üstü dosya yanıt başlığı yazılmadan C013 benzeri
  beklenen hata `Sonuç`'u olur; rota bunu `… dene` biçimiyle yakalayıp 404/413
  sayfası verebilir.
- Akış 16 KiB parçalarla yapılır; gövde belleğe tümüyle alınmaz ve spec/24
  heap zarfına girmez. HEAD aynı başlıkları gövdesiz verir.
- İstek yaşam döngüsü (spec/11 §5): çerez/oturum commit'i yanıt soketi
  eksiksiz yazıldıktan sonradır; akış yarıda koparsa K-327 kuralı geçerlidir.
  Yanıt yazma bütçesi dosya yanıtı için bayt orantılı ayrı bir mutlak tavan
  ister (ör. taban 10 sn + MiB başına süre, üst sınır 120 sn); sayı ölçümle
  bağlanır.
- Önbellek: dosya yanıtı varsayılan `Cache-Control: no-store` kalır;
  RFC-0037/K-335 `herkese açık` önbellek politikası yalnız oturumsuz rotada ve
  ayrı karar olarak dosya sınıfına açılabilir.

**`Content-Disposition` ve dosya adı.**

- Biçim yalnız `inline` ya da `attachment`; parametre yalnız `filename*`
  (RFC 6266 §4.3 + RFC 8187, `UTF-8''` + yüzde kodlama) ve ASCII geri düşüşü
  `filename="…"`. Zee yalnız adı verir; başlık değerini çalışma zamanı kurar.
- Ad doğrulaması `ek_adini_denetle` ile aynıdır ve web için sıkılaştırılır:
  boş, `.`/`..`, `/`, `\`, NUL, denetim karakteri (CR/LF dahil), `"`,
  Unicode yön denetimi (U+202A–U+202E, U+2066–U+2069) ve 255 baytı aşan ad
  reddedilir (C022). Kodlama `yuzde_kodla` ile ortaktır; ASCII geri düşüş
  adındaki ASCII dışı ve `%`, `"`, `\`, `;` karakterlerini `_` yapar.
- Ad, yolun basename'inden türetilmez; rota açıkça verir. Böylece depodaki
  rastgele `.parca`/özet adı kullanıcıya sızmaz ve kullanıcı adı yol
  çözümüne girmez.

**Başlık kaydı (RFC-0040 katman C ile ortak).** Dosya yanıtının yazabildiği
başlıklar: `Content-Type` (kayıttan), `Content-Disposition` (yukarıdaki
biçim), `ETag` (çalışma zamanı, tutamağın SHA-256'sından `"sha256-<hex>"`;
RFC-0040'ın ETag satırıyla aynı sözdizimi kuralı), `Content-Length`
(çalışma zamanı). Rota başka başlık yazamaz. `Last-Modified` ve koşullu
GET (`If-None-Match` → 304) RFC-0040 katman B başlık okuma listesine bağlıdır
ve onunla birlikte açılır.

### B — `multipart/form-data` yükleme

**Seçim.** İstek başlığında tek `Content-Type: multipart/form-data;
boundary=…` varsa adaptör spec/26 binary yolunun **kardeşi** olan multipart
yoluna girer; diğer türler bugünkü yolda kalır. Yol yalnız proje bildirimiyle
açıkça etkinleştirilmiş rotalarda kullanılabilir (fikir: rota önsözünde
`dosya yüklemesi kabul edilir` politikası); diğer rotalarda multipart istek
bugünkü gibi davranır (64 KiB + form çözümü) — açık soru: 415 mi?

**Ayrıştırıcı sınırları (fail-closed).**

- Boundary: RFC 2046 §5.1.1, 1–70 karakter, izinli karakter kümesi; tırnaklı
  ya da tırnaksız tek `boundary` parametresi; başka parametre ya da yinelenen
  `boundary` 400.
- Toplam gövde: `Content-Length` ZORUNLU, multipart için tavan dosya tavanı
  (16 MiB) + 64 KiB zarf payı; aşım gövde okunmadan 413.
- Parça sayısı ≤ 8 (tek dosya + en çok 7 küçük metin alanı); dosya parçası
  tam olarak 1. Fazla parça, ikinci dosya ya da sıfır dosya 400.
- Parça başlık bloğu ≤ 8 KiB, parça başına başlık satırı ≤ 8; yalnız
  `Content-Disposition: form-data; name="…"[; filename="…"|filename*=…]`
  ve isteğe bağlı `Content-Type` tanınır. `Content-Transfer-Encoding`
  (RFC 7578 §4.7 kullanımdan kaldırdı) 400. Bilinmeyen başlık 400.
- Metin alanı değeri ≤ 8 KiB, UTF-8 strict; alan adları sorgu+form toplam 100
  alan sınırına (spec/11 §5) dahildir. Aynı alan adı iki kez 400.
- Dosya adı yalnız bilgi amaçlıdır: basename'e indirgenir, katman A'daki ad
  doğrulayıcısından geçer, geçmezse `yukleme_dosya_adi` boş gelir (red değil;
  tarayıcılar yolu da gönderebilir). Ad hiçbir zaman disk yoluna girmez.
- Parçanın beyan ettiği `Content-Type` güvenilmezdir; rotaya
  `yukleme_bildirilen_tur` olarak bilgi amaçlı verilir, yetki/tür kararı
  RFC-0036 imza okuması ve ürün tablosuyla verilir.
- Ayrıştırma tek geçişli akış durum makinesidir: boundary arama tamponu
  sınırlıdır (boundary uzunluğu + 4), gövde belleğe alınmaz. Epilog ve
  önsöz ≤ 1 KiB; kapanış boundary'si (`--…--`) olmadan biten gövde 400.
- 10 sn mutlak okuma bütçesi (spec/11 §5) değişmez; büyük dosya için
  bütçenin ayrıca orantılı açılması katman A ile aynı açık sorudur.

**spec/26 parça modeliyle birleşim.** Dosya parçasının gövdesi spec/26 ile
**aynı** yükleyiciye verilir: `.zee/yuklemeler` altında `0600` izinli,
runtime adlandırmalı `.parca` dosyası, aynı akışta SHA-256, istek sonunda
silme ve 15 dakikalık yaş süpürmesi. Rota bugünkü üç alana ek olarak şunları
görür:

- `yukleme_gecici_yolu`, `yukleme_sha256`, `yukleme_bayti` (değişmez);
- `yukleme_dosya_adi` (doğrulanmış basename ya da boş);
- `yukleme_bildirilen_tur` (bilgi; güvenilmez);
- diğer metin alanları bugünkü form alanları gibi `istek` sözlüğünde.

Böylece ürünün octet-stream rotası ile multipart rotası aynı taşıma/
özetleme/imza kodunu kullanır.

**CSRF'nin form alanından okunması.**

- Multipart yolunda belirteç yalnız `_csrf` adlı metin alanından okunur;
  `X-Zee-CSRF` başlığı bu yolda yok sayılır (iki kaynak birleşmez).
- Sıra ZORUNLUDUR: `_csrf` alanı **dosya parçasından önce** gelmek zorundadır
  (HTML formunda gizli alanı dosya alanının üstüne koymak yeterlidir;
  tarayıcılar alanları belge sırasıyla gönderir). Dosya parçasına ulaşıldığında
  `_csrf` yoksa ya da oturumla sabit zamanlı karşılaştırma başarısızsa, dosya
  gövdesi **diske hiç yazılmadan** 403 verilir. Bu, K-332'nin "gövde diske
  akmadan önce reddet" değişmezinin multipart karşılığıdır; tek fark, ön
  kapının başlık yerine ilk parçada çalışmasıdır.
- Oturum çerezi yoksa K-332 gibi gövde okunmadan 403.
- `--web-proxy` profilinde `Origin` zorunluluğu (spec/12 §5) aynen geçerlidir;
  tarayıcı form gönderiminde `Origin` taşıdığı için ürün etkisi yoktur.
- `SameSite=Lax` oturum çerezi çapraz site POST'ta gönderilmez; bu,
  belirteçten bağımsız ikinci savunmadır, yerine geçmez.

Fikir taslağı:

```text
POST "/ek-yukle" adresine istek geldiğinde
    oturum gerekli
    dosya yüklemesi kabul edilir
    gecici isteğin "yukleme_gecici_yolu" değeri olsun
    ...
```

HTML tarafı (ürün):

```text
<form method="post" action="/ek-yukle" enctype="multipart/form-data">
  <input type="hidden" name="_csrf" value="…">
  <input type="file" name="dosya">
</form>
```

### C — SMTP ek türü kaydı

**Varsayılan değişmez.** Bildirimsiz projede `zee-smtp-mime-1` dört türü
(PDF/PNG/JPEG/WebP) kabul etmeye devam eder; spec/23 cümlesi bu kip için
aynen geçerlidir. Profil etiketi değişmez; genişletilmiş kip ayrı etiket alır
(öneri: `zee-smtp-mime-1+ek-kaydi` ya da `zee-smtp-mime-2`, Açık sorular).

**Kapalı aday kayıt.** Dil, çalışma zamanında sabit bir aday tablo taşır;
her satır uzantı(lar), MIME türü, risk sınıfı ve isteğe bağlı imza beklentisi
içerir. İlk aday küme:

| Uzantı | MIME türü | Risk sınıfı | Not |
|---|---|---|---|
| `txt` | `text/plain; charset=utf-8` | düşük | UTF-8 strict |
| `csv` | `text/csv; charset=utf-8` | orta | formül enjeksiyonu alıcı tarafında; Zee kaçmaz, ürün sorumluluğu |
| `ics` | `text/calendar; charset=utf-8; method=…` | düşük | CRLF kuralı RFC-0040 katman D(i) ile ortak |
| `docx` | `application/vnd.openxmlformats-officedocument.wordprocessingml.document` | yüksek | makro içermez ama dış ilişki/OLE taşıyabilir |
| `xlsx` | `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` | yüksek | aynı |
| `pptx` | `application/vnd.openxmlformats-officedocument.presentationml.presentation` | yüksek | aynı |
| `zip` | `application/zip` | yüksek | iç içe arşiv/bomba; içerik açılmaz |

Makro taşıyan türler (`docm`, `xlsm`, `pptm`, eski `doc`/`xls`/`ppt`),
yürütülebilir ve betik türleri (`exe`, `msi`, `bat`, `cmd`, `ps1`, `js`, `vbs`,
`jar`, `scr`, `lnk`, `iso`, `html`, `svg`) aday tabloda **yoktur**; bildirimle
de açılamaz. Yeni aday satırı K-167 yüzey kaydı ve ayrı karar ister.

**Proje/kurum ayarı.** Proje bildirimine kapalı bir alan (fikir:
`e_posta_ek_türleri: ["pdf", "png", "jpeg", "webp", "docx", "xlsx", "csv"]`).
Kurallar:

- Yalnız aday tablodaki uzantılar yazılabilir; bilinmeyen değer proje
  yüklemesinde tanıyla reddedilir (S/T sınıfı, alt RFC'de).
- Alan yoksa dört varsayılan geçerlidir; alan varsa liste **tam kümedir**
  (varsayılanlar örtük eklenmez — kurum PDF'yi de kapatabilir).
- Kurum ayarı: ZeeOS çok kiracılı dağıtımda kiracı başına liste ürün
  verisidir; dilin işi yalnız **proje tavanını** uygulamaktır: ürün çağrı
  başına daha dar bir küme verebilir (fikir: ekli ileti çağrısına isteğe bağlı
  izinli tür listesi), proje tavanını aşamaz. Böylece kurum politikası Zee
  kodunda, güvenlik tavanı bildirimde kalır.
- Tür seçimi uzantıdan yapılır (bugünkü gibi); ek olarak aday satırın imza
  beklentisi varsa (OOXML/zip için `504b0304`, PDF için `25504446`) çalışma
  zamanı ilk baytları doğrular, uyuşmazlık C029'dur. Metin türleri UTF-8
  strict denetlenir. Bu denetim içerik güvenliği değildir; yalnız uzantı
  sahteciliğini (ör. `.pdf` adlı yürütülebilir) daraltır.
- Boyut sınırları (`KaynakSinirlari.eposta()`: 8 ek, tek ek 8 MiB, toplam
  16 MiB) değişmez.
- **Ölçülebilir ret.** Tür, imza, boyut ve ad reddi bugün de ağdan önce
  verilir ama ağ hatasıyla aynı maskeli C029 metnine düşer; bu yüzden ne
  ürün ne de reproducer “ek türü reddedildi”yi kanıtlayabilir. Katman C,
  ağa çıkmadan verilen yerel ek reddini sır içermeyen ayrı bir sınıfla
  ayırmak ZORUNDADIR (ör. aynı C029 kodu altında sabit “ek türü izinli
  değil” / “ek boyutu sınırı aşıyor” metinleri ya da `Hata` değerinde ayrı
  bir neden alanı). SMTP yanıtı, URL ve kimlik bilgisi maskesi (K-236)
  değişmez; yalnız yerel, sır taşımayan ret nedenleri görünür olur. Mümkünse
  sabit bir ek ifadesi (literal liste) için aynı ret `dil denetle`de uyarı
  olarak da verilir.

**İçerik/antivirüs sınırı.** Allowlist etiket ve imza düzeyindedir; içerik
güvenliği sözü vermez (spec/23 "AÇIKTIR" korunur). Alt RFC iki seçenekten
birini seçer: (a) ürün, göndermeden önce operatörün tarayıcısını çağırır
(ör. ClamAV soketi RFC-0039 bağlantı katmanı ya da yerel süreç) ve sonucu
kendi kaydına yazar; (b) dil, proje bildiriminde isteğe bağlı bir tarayıcı
kancası tanımlar (fail-closed: tarayıcı yoksa ya da zaman aşımı → gönderim
yok). Öneri (a): antivirüs politikası ürün/operatör kararıdır, dil çekirdeğine
tarayıcı protokolü girmez.

## Güvenlik etkisi

| Sınıf | Risk | Bu taslaktaki karşılık |
|---|---|---|
| MIME karışıklığı | Tarayıcının kullanıcı dosyasını HTML/betik olarak koklaması (stored XSS) | `nosniff` her yanıtta (bugün de); dosya sınıfında `text/html`/SVG/XHTML/betik türü yok; dosya yanıtında `CSP: default-src 'none'; sandbox` ve `CORP: same-origin`; `inline` yalnız PDF + dört görsel; diğerleri `attachment` ZORUNLU |
| MIME karışıklığı | Bildirilen `Content-Type`'a güvenip yanlış tür sunmak | Multipart parçası türü yalnız bilgi; tür kararı ürün tablosu + RFC-0036 imza okuması; yanıt türü rotanın açık seçimi |
| Başlık enjeksiyonu | `Content-Disposition` dosya adında CR/LF, `"`, `;` ile yanıt bölme ya da parametre ekleme | Başlık değerini çalışma zamanı kurar; ad doğrulayıcısı CR/LF/denetim/`"`/yön karakterini reddeder (C022); `filename*` yalnız `[A-Za-z0-9._-]` düz, kalan yüzde kodlu (bugünkü `yuzde_kodla`); ASCII geri düşüşte riskli karakter `_` |
| Dosya adı | Unicode yön karakteriyle `fatura‮fdp.exe` biçimli ad sahteciliği | U+202A–E, U+2066–9 reddi; ad yoldan değil rotadan |
| Yol geçişi | Yanıt yolunda `..`, mutlak yol, bağ ile proje dışı dosya sızdırma | spec/26 yol kuralı + leaf/ara symlink reddi + normal dosya şartı + `dosya-okuma` yetkinliği; multipart dosya adı hiçbir zaman yol olmaz |
| Yol geçişi | Yetkisiz kullanıcının başka kullanıcının ekine erişmesi (IDOR) | Ürün sorumluluğu (yetki Zee kodu); yanıt yüzeyi yetki kararı vermez; ürün kılavuzu ek kimliğini tahmin edilemez tutar |
| Multipart bomba | Çok parça, dev başlık, boundary arama tamponu, yavaş gövde | Parça ≤ 8, tek dosya, başlık bloğu ≤ 8 KiB, alan ≤ 8 KiB, sınırlı tampon, `Content-Length` ZORUNLU + 413, 10 sn mutlak okuma bütçesi, tek geçiş |
| Multipart karışıklık | Proxy ile Zee'nin gövdeyi farklı ayrıştırması (boundary/`Content-Transfer-Encoding` farkı) | Strict tek `boundary`, `Content-Transfer-Encoding` 400, bilinmeyen başlık 400, kapanış boundary'si ZORUNLU |
| CSRF | Form yüklemesinde belirteç sonradan okunursa dosyanın diske yazılması | `_csrf` dosya parçasından önce ZORUNLU; doğrulanmadan dosya baytı diske akmaz; oturum çerezi yoksa gövde okunmaz (K-332 karşılığı); `X-Zee-CSRF` ile birleşmez; `Origin` kuralı aynen |
| Disk | Tamamlanmış ama kullanılmayan yüklemenin birikmesi | spec/26 istek sonu silme + 15 dk yaş süpürmesi, aynı yükleyici |
| Zip bombası | `application/zip`/OOXML içinde üstel açılım | Zee arşiv açmaz (yanıtta da SMTP'de de); boyut tavanları sıkıştırılmış bayta uygulanır; alıcı tarafındaki açılım ürün/antivirüs sınırı |
| Makro içerik | Office dosyasıyla zararlı makro iletmek | Makro taşıyan türler aday tabloda yok, bildirimle açılamaz; OOXML imza denetimi uzantı sahteciliğini daraltır; içerik taraması AÇIK ve ürün/operatör katmanı (seçenek a) |
| Antivirüs | Allowlist'in güvenlik garantisi sanılması | spec/23 "içerik/antivirüs doğrulaması AÇIKTIR" korunur; belge, tanı metni ve sürüm notu bunu açıkça söyler |
| Kaynak | Büyük dosya yanıtında yazma bütçesi ve heap | 16 KiB akış, heap zarfına girmez; yazma bütçesi bayt orantılı ve üst sınırlı; K-327 kopma kuralı |
| İz | Dosya içeriğinin IO izine düşmesi | İzde yalnız yol, özet, bayt, tür, ad; içerik yok |

Her bulgu ve bilinçli sınır, uygulama işinde `docs/guvenlik-bulgulari-v1.tsv`
GB satırı alır. Bu taslak kod değiştirmediği için yeni GB satırı açmaz.

## Core freeze gerekçesi ve kanıt kapısı

Core freeze altında yeni compiler özelliği varsayılan olarak reddedilir. Bu
RFC'nin hiçbir katmanı şu zincir kurulmadan uygulanmaz:

1. **Ürün kaydı.** Kullanan ürün `urun-h`dir; K-355 ile
   `docs/dogfood-projeleri-v1.tsv` içinde etkin, exact harici köken ve depo
   içi kanıt kökü [`dogfood/urun-h`](../dogfood/urun-h/README.md) ile kayıtlı.
   Bu adım karşılanmıştır.
2. **Ürün sürtünmesi.** ZM-F022 ürünün kendi sürtünme kaydındadır (depo
   dışı). Üç ihtiyaç e-posta ürününün temel akışıdır ve bu RFC'de kaynakla
   doğrulanmıştır; ama **depo içi reproducer henüz yok**. Her katman
   açılmadan önce ürün, eksik yetkinliğe çarpan gerçek Zee kaynağını
   `dogfood/urun-h/korpus/` altına exact beklentiyle koyar ve K-164
   kataloğuyla karşılaştırır. Ürünün 23 Kasım 2026 ölçümü: (A) ZeeMail
   dışa aktarımı `text/plain` gidiyor, vCard/CSV `<a download>` ile
   geçiştiriliyor, ek indirme “yakında”; `%PDF` gövdeli `Metin` yanıt
   `text/plain` gelir. (B) Küçük multipart form 403, 64 KiB üstü yanıtsız
   kapanma. (C) `.docx` eki C029 — ama aynı maske ağ hatasında da çıktığı
   için `calistir basarisiz C029` beklentisi türü **kanıtlamaz**; bu vaka ya
   ağ hedefi olmayan derleyici test fikstürü olarak ya da katman C'nin
   ölçülebilir ret metniyle yazılmalıdır. (B) Zee `proje` testinden HTTP
   isteği atılamadığı için adaptör entegrasyon testi ya da kayıtlı curl
   ölçümü olarak tutulur.
3. **Tekrar sinyali.** Dosyadan yanıt ihtiyacı RFC-0040 katman C ("dosyadan
   yanıta akıtma … ayrı ikinci adım") ve RFC-0039 katman D'de de geçer; aynı
   ürün içinde üç protokol yolunda tekrar eden sınıf K-164 kimliğiyle anılır.
   İkinci bir üründe tekrar (ör. site çekirdeğinin dosya indirmesi) backlog
   tasarım maddesini güçlendirir.
4. **Katman başına zincir.** Alt RFC → ADR → spec maddesi +
   `spec-madde-kaniti` satırı → olumlu/olumsuz test → reproducer → K-167
   yüzey kaydı (içerik türü satırları, başlık, politika cümlesi, proje alanı,
   tanı kodu) → güvenlik bulgusu kaydı aynı değişiklikte.
5. **Sıra önerisi.** A (indirme; okuma yönlü, en düşük risk) ile RFC-0040
   katman C'nin ortak tür/başlık kaydı birlikte ilk alt RFC'dir. Ardından C
   (SMTP; proje alanı + aday tablo, grammar gerektirmez), en son B
   (multipart; yeni ayrıştırıcı, en geniş saldırı yüzeyi). Sıra değişirse
   gerekçe ADR'ye yazılır.
6. **Grammar.** Yeni yanıt cümlesi ve politika cümlesi mevcut kalıpların
   uzantısıdır; K-167 yüzey kaydı ve RFC-0028 deprecation kuralları geçerlidir.
   V1 syntax freeze penceresi içinde açılmaz.

V1 kapı listesine (`docs/v1-surum-kapilari.md`) bu RFC eklenmez: ikili yanıt,
multipart ve genişletilmiş ek türleri V1 sözü değildir.

## RFC-0039 ve RFC-0040 ile ilişki

- **RFC-0040 katman C (yanıt içerik türü/başlık kaydı).** Bu RFC, RFC-0040'ın
  önerdiği kaydı ayrı bir kayıtla çoğaltmaz; tek tabloyu "metin" ve "dosya"
  sınıflarıyla genişletir. `ETag` sözdizimi, başlık değer doğrulayıcıları
  (C022) ve `no-store` varsayılanı iki RFC'de ortaktır. Hangisi önce kabul
  edilirse kaydı o tanımlar, öteki ona satır ekler.
- **RFC-0040 katman C son madde / katman D (b).** RFC-0040 "dosyadan yanıta
  akıtma (büyük ek, `GET` ile .ics) … ayrı ikinci adım" ve büyük iCalendar
  gövdesinin `0600` geçici dosyaya akmasını önerir. Bu RFC'nin katman A'sı o
  ikinci adımın kendisidir; katman B'nin yükleyicisi RFC-0040 D(b) ile aynı
  spec/26 ilkelidir.
- **RFC-0040 katman D(i) CRLF.** `.ics` SMTP eki ve `text/calendar` dosya
  yanıtı CRLF kuralını RFC-0040 ile paylaşır; çalışma zamanı satır sonunu
  bir kez tanımlar.
- **RFC-0039 katman D (tutamak).** Yol + SHA-256 + bayt tutamağı RFC-0039'un
  "bayt verisi diskte kalır, Zee tutamak görür" ilkesidir. Bu RFC yeni bir
  `Baytlar` türü önermez; IMAP `FETCH BODY[]` ve web indirme aynı tutamağı
  tüketir.
- **RFC-0039 MIME alt RFC'si.** Gelen iletiden eki ayrıştırma ve SMTP alıcı
  tarafı orada kalır; bu RFC'nin katman C'si yalnız **giden**
  `zee-smtp-mime-1` ek türü kaydıdır. Aday tablo iki yön için ortak
  tutulmalıdır (alınan ekin web'de hangi türle sunulacağı katman A
  kaydından).
- **Ortak ürün.** Üçü de `urun-h` sürtünmesinden büyür; reproducer'lar aynı
  korpus kökünde ayrı Fnnn kimlikleriyle tutulur.

## Dört soru süzgeci

1. **Türkçe doğal mı?** — `… dosyasını "…" türünde "…" adıyla ek olarak
   gönder` mevcut `… yanıtını gönder` ritmini izler; politika cümlesi
   `dosya yüklemesi kabul edilir` mevcut `oturum gerekli` ile aynı önsöz
   kalıbıdır. Kesin yüzey alt RFC'de kullanılabilirlik kartıyla seçilir.
2. **Deterministik mi?** — Yanıt ve yükleme izde yol/özet/bayt/tür ile yer
   alır; replay içerik yerine özeti karşılaştırır.
3. **Öğrenilebilir mi?** — Üç yeni kavram: dosyadan yanıt, form yüklemesi
   politikası, proje ek türü listesi. Hepsi mevcut spec/26 tutamak alanlarını
   yeniden kullanır.
4. **Profesyonel ölçekte savunulabilir mi?** — `nosniff` + sandbox CSP +
   `attachment` varsayılanı, RFC 6266/8187 ad kodlaması, sınırlı tek geçişli
   multipart ayrıştırıcı, gövde diske akmadan CSRF, makro/yürütülebilir türün
   yapısal yasağı ve açıkça ürüne bırakılmış antivirüs sınırı yerleşik web
   çatılarının (Django `FileResponse`, Rails `send_file`, Go
   `http.ServeContent` + `multipart.Reader` sınırları) güvenlik pratikleriyle
   uyumludur.

## Kabul ölçütleri

Bu taslak ancak şunlar kanıtlandığında "geçici kabul"e aday olur:

1. ZeeMail web arayüzü `--web-proxy` arkasında 16 MiB'a kadar PDF, PNG, docx
   ve zip ekini Firefox, Chromium ve Safari'de indirir; Türkçe/Unicode dosya
   adı üç tarayıcıda da doğru görünür.
2. JS kapalı tarayıcıda form ile tek dosya yüklenir; `_csrf` eksik/yanlış,
   `_csrf` dosyadan sonra, iki dosya, 9 parça, dev başlık, kapanışsız
   boundary, `Content-Transfer-Encoding`, tavan üstü gövde vakalarında dosya
   diske hiç yazılmaz (olumsuz testler `.zee/yuklemeler` boşluğunu doğrular).
3. Bildirimsiz projede `.docx` eki bugünkü gibi C029 verir; bildirimli
   projede kabul edilir; `.docm`/`.exe` bildirimle de reddedilir; imzası
   uyuşmayan `.pdf` reddedilir.
4. Olumsuz testler: CR/LF/`"`/yön karakterli dosya adı, `..`/symlink yanıt
   yolu, `text/html` dosya yanıtı denemesi, yanıt sırasında dosya
   kısalması, yavaş istemcide yazma bütçesi.
5. Yerel ek reddi (tür/imza/boyut/ad) ağ hatasından ayrı ve SMTP bağlantısı
   açılmadan gözlenir; reproducer bunu ağ hedefi olmadan (ör.
   `compiler/tests/fixtures/native-effect-consumer` kalıbında) kanıtlar.
6. 64 KiB (ve multipart için katman B tavanı) üstündeki gövdede istemci,
   `Expect: 100-continue` göndermese de 413'ü okuyabilir; bağlantı RST ile
   kopmaz (ayrı fix adayı kapanmış olmalı).
7. Aynı oturum IO izinden replay edilir ve aynı yanıt başlıklarını ve
   gövde özetini üretir; `dil dene` hermetik adaptör aynı sonucu verir.

## Alternatifler

- **`Metin` yanıtta base64/data URL:** tarayıcı indirmesi için uygun değil,
  boyutu üçte bir şişirir, heap zarfına girer; reddedildi.
- **Ters vekilin `X-Accel-Redirect`/`X-Sendfile` ile dosya sunması:** yetki
  kararı Zee'de kalır ama dosya sunumu ve başlık güvenliği vekile geçer;
  `--deneysel-web` ve `dil dene`'de karşılığı yoktur, iki adaptör ayrışır.
  Operatör optimizasyonu olarak ileride açık; ilk model değildir.
- **Statik dosya dizini:** yetkisiz erişim (IDOR) ve kullanıcı dosyasının
  aynı origin'de sunulması riski; e-posta eki kullanıcıya özel veridir;
  reddedildi.
- **Multipart yerine JS ile octet-stream:** ürünün "JS'siz çalışır" ilkesi
  ve erişilebilirlik gereksinimiyle çelişir; octet-stream yolu korunur ama
  tek yol olamaz.
- **CSRF'yi multipart'ta da yalnız başlıktan okumak:** HTML formu başlık
  gönderemez; ihtiyacın kendisini reddeder.
- **Dosya parçasını önce diske yazıp sonra `_csrf`'i doğrulamak:** K-332'nin
  "gövde diske akmadan reddet" değişmezini bozar ve kimliksiz disk
  doldurmaya açar; reddedildi.
- **SMTP allowlist'ini tamamen kaldırıp ürüne bırakmak:** `zee-smtp-mime-1`
  güvenli varsayılanını ve makro/yürütülebilir yapısal yasağını kaybeder;
  reddedildi.
- **Serbest MIME türü listesi (proje istediği türü yazar):** kapalı aday
  tablo ve yasak tür listesi olmadan yürütülebilir türler açılabilir;
  reddedildi.

## Korpus etkisi

Taslağın kendisi golden, regression, dogfood korpusu ve dil yüzeyi
fixture'ını değiştirmez. Her katman açıldığında `urun-h` reproducer'ları
ret/çözüm çiftine dönüşür.

## K-378 kararları (açık soruların kapanışı)

23 Kasım 2026 merkez kararıyla (ZeeMail yüzeyleri sırayla, adım 1) RFC kabul
edildi; alt RFC açılmadı, karar ADR-163'te. Yanıtlar:

- **Yazma/okuma bütçesi:** dosya yanıtı ayrı formül almaz; K-372 mutlak
  bütçesi `10 sn + bayt / 128 KiB/s` başlık + gövdeye tek son tarih olarak
  uygulanır (16 MiB ≈ 138 sn). Okuma tarafında spec/11 §5'in 10 sn mutlak
  istek bütçesi multipart yüklemede de değişmez; orantılı okuma bütçesi
  ayrı karar ve K-işi adayıdır (yavaş bağlantıda büyük yükleme 408 alır).
- **Dosya yanıtı tavanı:** 16 MiB sabit (`yukleme_bayti`); proje bildirimiyle
  yükseltme açılmadı.
- **`Range`/206:** ilk sürümde yok; `inline`, `ETag` ve koşullu GET de yok.
  Yalnız `attachment` açıldı.
- **Politikasız rotaya multipart:** multipart yolu rota politikası değil
  proje bildirimiyle (`çok_parçalı_yükleme doğru olsun`) açılır; bildirimsiz
  projede multipart istek bugünkü gibi davranır (64 KiB + form çözümü, çoğu
  zaman 403). 415 davranış değişikliği olduğundan açılmadı (katman B dilimi).
- **`_csrf` sırası:** ZORUNLU; dosya parçasından önce gelmeyen ya da
  oturumla eşleşmeyen belirteçte dosya baytı diske yazılmadan 403.
- **SMTP profil etiketi:** `zee-smtp-mime-1` değişmez; tel biçimi aynıdır,
  genişletme proje bildirimidir (`e_posta_ek_türleri`).
- **Çağrı başına dar tür listesi:** dil yüzeyi açılmadı; ürün göndermeden
  önce kendi denetimini yapar, proje tavanı bildirimde tek yerde kalır.
- **Antivirüs:** seçenek (a) — ürün/operatör; dil çekirdeğine tarayıcı
  protokolü girmez.
- **CORP/sandbox CSP ve PDF önizleme:** yalnız `attachment` açıldığından
  önizleme etkisi bu sürümde yok; `inline` açılırken ölçülür.
- **Ortak kayıt ve DAV türleri:** `dil::icerik_turu` tek tablodur; DAV
  türleri RFC-0040 işinde satır olarak eklenir.

## Açık sorular (taslak dönemi, tarihsel)

- Dosya yanıtı ve multipart yükleme için yazma/okuma bütçesi: bayt orantılı
  formül mü, proje bildirimli sabit üst sınır mı? 10 sn spec/11 §5 sözü bu
  iki yol için nasıl istisnalanır?
- Dosya yanıtı bayt tavanı: 16 MiB sabit mi, proje bildirimiyle (ör. 64 MiB)
  yükseltilebilir mi? SMTP tek ek 8 MiB sınırıyla ilişki.
- `Range`/206 ilk sürümde gerekli mi (mobil tarayıcıda büyük PDF önizleme)?
- Multipart politikası olmayan rotaya gelen multipart istek: bugünkü gibi mi
  (64 KiB + form çözümü), yoksa açık 415 mi? İkincisi davranış değişikliği
  olduğundan K-167/deprecation değerlendirmesi ister.
- `_csrf` sırası ZORUNLU mu, yoksa dosya parçası kısa süreli belleğe/geçici
  dosyaya alınıp sonradan doğrulanabilir mi? Taslak ZORUNLU sırayı önerir.
- Genişletilmiş SMTP kipinin profil etiketi: `zee-smtp-mime-1` + bildirim mi,
  yeni `zee-smtp-mime-2` mi?
- Ürün çağrı başına dar tür listesi (kurum politikası) yeni bir imza argümanı
  mı ister, yoksa ürün göndermeden önce kendi denetimini mi yapar? İkincisi
  dil yüzeyi açmaz ama tavanı tek yerde tutmaz.
- Antivirüs için seçenek (a) ürün/operatör mü, (b) bildirimli kanca mı?
- Dosya yanıtında `Cross-Origin-Resource-Policy` ve sandbox CSP'nin PDF
  önizlemesini tarayıcılarda bozup bozmadığı ölçülmeli.
- Ortak tür/başlık kaydı RFC-0040'tan önce kabul edilirse DAV türleri (`207`,
  `application/xml`) aynı alt RFC'ye mi girer, yoksa yalnız satır olarak mı
  bekler?
