# RFC-0040 — Web adaptöründe WebDAV/CalDAV/CardDAV senkron istemci yüzeyi

- **Durum:** taslak — 21 Kasım 2026. Core freeze (K-160A/ADR-059) altında
  uygulanmaz; her katman ayrı alt RFC ve dogfood kanıt kapısıyla açılır.
- **Tarih:** 21 Kasım 2026
- **İş:** K-364
- **İlgili belgeler:** [spec/11](../spec/11-uygulama-eylemleri-ve-web-adaptoru.md),
  [spec/12](../spec/12-web-guvenlik-profili.md),
  [spec/21](../spec/21-deterministik-io-izi.md),
  [spec/23](../spec/23-yetkinlik-ve-outbound-guvenligi.md),
  [spec/24](../spec/24-kaynak-guvenlik-profili.md),
  [spec/26](../spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md),
  [spec/27](../spec/27-uyumluluk-ve-surumleme.md),
  [ADR-034](../adr/034-web-ortak-depo-ve-worker-modeli.md),
  [ADR-036](../adr/036-kanonik-web-proxy-origin-siniri.md),
  [ADR-037](../adr/037-byte-tabanli-http-istek-siniri.md),
  [ADR-157](../adr/157-istek-yerel-hata-siniri.md),
  [RFC-0017](0017-web-oturum-ve-csrf.md),
  [RFC-0027](0027-binary-yukleme-ve-dosya-yasam-dongusu.md),
  [RFC-0037](0037-rota-onbellek-basligi.md),
  [RFC-0039](0039-ag-sunucusu-yetkinligi.md)
- **Freeze sınıfı adayı:** her katman için ayrı `dogfood-change`; bu taslak
  kendi başına hiçbir dil, CLI, tanı ya da API yüzeyi açmaz.
- **Alt RFC'ler:** [RFC-0047](0047-dav-istemci-rotasi-ilk-dilim.md) — ilk
  dilim (A + dar B + dar C + E + F), taslak, 27 Kasım 2026; D ve G sonraki
  alt RFC'de.

## Özet

E-posta ürünü (dogfood `urun-h`) masaüstü ve telefonda standart istemcilerle
kullanılmak istiyor: posta IMAP/SMTP ile ([RFC-0039](0039-ag-sunucusu-yetkinligi.md)),
takvim **CalDAV** (RFC 4791) ve kişiler **CardDAV** (RFC 6352) ile; Linux,
macOS ve Windows istemcileri ile Android/iOS. Ürünün web tarafı takvim, kişi ve
görev verisini Zee + PostgreSQL ile zaten tutuyor. Bu RFC, aynı veriyi
WebDAV (RFC 4918) tabanlı senkron protokolleriyle Zee web adaptörü üzerinden
yayınlamayı engelleyen eksikleri kaynak kodla doğrular ve bunları yedi küçük,
birbirinden bağımsız denetlenebilir katmana böler: yöntem kaydı, istek başlığı
okuma yetkinliği, yanıt içerik türü/başlık kaydı, XML gövde sınırları ve akış,
çerezsiz Basic + uygulama parolası doğrulama modeli ve oran sınırı, CSRF
muafiyet kuralı, well-known keşif. İlke RFC-0039 ile aynıdır: **protokol
mantığı (PROPFIND yanıtının içeriği, senkron belirteci, iCalendar/vCard
doğrulaması) Zee kodudur**; çalışma zamanı yalnız HTTP zarfını, sınırları,
kimlik doğrulama ilkelini ve güvenli XML ayrıştırmayı sağlar. Ayrı bir
Rust/Go DAV sunucusu bu RFC'nin kapsamında değildir.

## Motivasyon

Ürün sahibinin 21 Kasım 2026 kararı: masaüstü/telefon için özel istemci
yazılmaz; işletim sistemlerinin ve yaygın istemcilerin konuştuğu standart
protokoller kullanılır. Takvim ve kişilerde bu standart CalDAV/CardDAV'dır.
Ürünün web arayüzü takvim/kişi/görev için hazırdır; eksik olan, aynı veriyi
senkron istemcisine sunacak HTTP yüzeyidir. Bu iki protokol HTTP üzerinde
çalıştığı için (RFC-0039'un tersine) yeni bir TCP dinleyicisi, bağlantı
görevi ya da sunucu TLS'i gerektirmez: mevcut `--web-proxy` profili (TLS
ters vekilde, Zee `127.0.0.1` arkasında) yapısal olarak yeterlidir. Engel,
web adaptörünün bilinçli olarak dar tutulmuş HTTP sözleşmesidir.

### Bugünkü durum (83d48cb üzerinde doğrulandı)

Ürün oturumu spec/11–12'yi okuyarak altı eksik çıkarmıştı; her biri kaynakla
ayrı ayrı doğrulandı (sonradan gelen iki ek sürtünme 7–8. satırlardadır). Sonuç sütunu **yok / kısmi / var** biçimindedir.

| # | İddia | Sonuç | Kanıt |
|---|---|---|---|
| 1 | Rota yöntemleri yalnız GET/HEAD/POST/PUT/PATCH/DELETE; OPTIONS, PROPFIND, PROPPATCH, REPORT, MKCALENDAR, MKCOL, MOVE/COPY yok | **Doğrulandı — yok.** Kapalı küme hem normatif hem kodda. Bayt parser'ı (`istek_satirini_ayristir`) herhangi bir RFC token yöntemini kabul eder, ama rota eşleşmesi kapalı enum'dur: aynı yolda PROPFIND/OPTIONS **405**, yol yoksa 404 döner; 405 yanıtı `Allow` başlığı taşımaz. Bilinmeyen yöntem için 501 yoktur. | spec/11 §4 "Desteklenen yöntemler `GET`, `HEAD`, `POST`, `PUT`, `PATCH`, `DELETE`dir"; `compiler/src/agac.rs` `HttpYontemi::ayristir`; `compiler/src/http_istegi.rs` `istek_satirini_ayristir`; `compiler/src/yorumlayici/web_istek.rs` (`yol_eslesti` → 405) |
| 2 | Zee kodu istek başlığı okuyamaz (Authorization, Depth, If-Match, If-None-Match, Content-Type); kimlik yalnız çerez oturumu | **Doğrulandı — yok.** Rota ortamına yalnız `istek` (`yol`, `yöntem` + sorgu/form alanları), `çerezler` ve `oturum` sözlükleri konur. Başlıklar yalnız çalışma zamanının kendi kapılarında okunur (`Cookie`, `Host`, `Forwarded`, `Origin`, `Content-Type`, `X-Zee-CSRF`). `Authorization` hiçbir sunucu yolunda okunmaz (yalnız giden HTTP istemcisinde yazılır). spec/12 §2 açıkça "İstek başlığı … `oturum` sözlüğünü dolduramaz" der. | `compiler/src/yorumlayici/web_istek.rs` (rota ortamı); `compiler/src/yorumlayici.rs` `istek_govdesiyle`; `compiler/src/main.rs` `govde_zarfini_denetle`, `yukleme_on_kapisi`; spec/12 §2 |
| 3 | 207 Multi-Status, `application/xml`, `text/calendar`, `text/vcard` ve ETag/DAV/Allow/WWW-Authenticate başlıkları yok | **Doğrulandı — kısmi.** Sabit durum literali 100–599 olduğundan `207`, `401`, `412` **yazılabilir**; ama içerik türü yalnız gövdenin ilk karakterinden seçilir (`<` → `text/html`, aksi `text/plain`). Bir XML gövdesi (`<?xml …` ya da `<d:multistatus`) bugün **`text/html` + HTML CSP** ile gider; durum satırı açıklaması 207 için `Error` olur. ETag, DAV, Allow, WWW-Authenticate, Location (303 dışı), Content-Location yazılamaz; V1 "dinamik status/header builder" açmaz. | spec/11 §4 (içerik türü kuralı ve "V1 bu cümleyle dinamik status/header builder … açmaz"); `compiler/src/main.rs` `web_yanit_taslagini_yaz`, `http_durum_aciklamasi`, `guvenlik_basliklari` |
| 4 | XML gövde ve 64 KiB tavanı aşan .ics/.vcf; RFC-0039 katman D bayt akışıyla ilişki | **Doğrulandı — yok.** `application/octet-stream` dışındaki her gövde 64 KiB ile sınırlıdır ve `Content-Type`'tan bağımsız olarak **URL-form** olarak çözülür: XML/iCalendar gövdesi ham metin olarak rotaya ulaşmaz; `%` içeren gövde 400, diğerleri anlamsız alanlara bölünür. Ham büyük gövde yalnız spec/26 binary yükleme yoluyla diske akar (16 MiB, CSRF başlığı + oturum çerezi ister). XML ayrıştırıcı yoktur. | spec/11 §4–5; spec/26; `compiler/src/kaynak_sinirlari.rs` (`istek_govde_bayti: 64 KiB`, `yukleme_bayti: 16 MiB`); `compiler/src/main.rs` `govde_zarfini_denetle`; `compiler/src/yorumlayici.rs` `istek_govdesiyle` |
| 5 | Çerezsiz istek başına Basic; Argon2 oran sınırı (300 sn/5) senkron istemcilerle çatışır; uygulama parolası için hızlı güvenli model ve cookie'siz Basic için CSRF muafiyeti gerekir | **Doğrulandı — yok, üç ayrı engel.** (a) Basic hiç okunmaz (#2). (b) Parola doğrulaması istemci kimliği başına 300 sn'de 5 ile sınırlı; her istekte Argon2id doğrulayan bir senkron istemci (bir PROPFIND + REPORT + N GET turu) altıncı istekte 429 alır. (c) Güvenli olmayan her yöntem `_csrf` form alanını ister (`herkese açık` bile kapatmaz) ve `--web-proxy` profilinde tek `Origin` başlığı ZORUNLUDUR; DAV istemcileri ikisini de göndermez → 403. | spec/12 §1, §3, §5 (oran tablosu, Origin maddesi); `compiler/src/yorumlayici/web_istek.rs` (`rota_guvenligini_denetle`) |
| 6 | `/.well-known/caldav` ve `/.well-known/carddav` yönlendirmesi, DNS SRV notu | **Doğrulandı — kısmi.** `GET "/.well-known/caldav" adresine istek geldiğinde` + `"/dav/" adresine yönlendir` bugün yazılabilir (yerel yol, 303). Ama istemciler keşfi çoğunlukla PROPFIND ile yapar → 405 (#1); 303 dışında (301/307/308) yönlendirme kodu yoktur. DNS SRV/TXT (RFC 6764) dağıtım katmanıdır, dil yüzeyi değildir. | spec/11 §4; spec/12 §4 (yerel yönlendirme); `compiler/src/main.rs` (`303 See Other`) |

Ürün oturumunun sonradan ilettiği iki sürtünme (ZeeMail sürtünme kaydı
ZM-F020, ZM-F004) de aynı tabanda doğrulandı:

| # | İddia | Sonuç | Kanıt |
|---|---|---|---|
| 7 | (ZM-F020) Argon2 oran sınırı `parolanın argon2 özeti` **üretimini** de sayıyor; aşımda C028 yakalanamıyor, istek 500 dönüyor; uygulama parolası üretimi ve parola değişikliği giriş bütçesini harcıyor | **Doğrulandı — var (tasarım boşluğu).** Üretim ve doğrulama aynı `Giris` kovasını aynı `"parola"` anahtarıyla, istemci kimliği başına artırır (300 sn/5). Doğrulama aşımda 429 yazıp `yanlış` döner; üretim aşımda `Err` → **C028** tanısı olur. C028 `Sonuç` değil tanıdır, `… dene` ailesiyle yakalanamaz; spec/11 §5 kuralıyla 503 listesinde olmadığı için genel **500** "istek işlenemedi" gövdesine çevrilir. Böylece bir kullanıcının 5 dakika içinde 1 giriş + 4 uygulama parolası üretmesi beşinci üretimde 500 verir. | `compiler/src/main.rs` `parola_ozeti_uret` / `parola_dogrula` (aynı `RateLimitTuru::Giris`, `"parola"`); `compiler/src/yorumlayici/kimlik_iletisim.rs` (`PAROLA_OZETI` → C028); `compiler/src/yorumlayici/eylem_hatasi.rs` `web_mesaji` (yalnız C013/C021/C027); spec/12 §2 ("web isteğinde giriş oran sınırına tabidir"), §5 tablo |
| 8 | (ZM-F004) iCalendar/vCard CRLF satır sonu ister; Zee metninde `\r` kaçışı yok | **Doğrulandı — yok.** Kaçış kümesi `\"`, `\\`, `\n`; `\r` S040'tır. RFC 5545 §3.1 ve RFC 6350 §3.2 içerik satırını CRLF ile sınırlar; bugün Zee'de CRLF'li `Metin` literalle kurulamaz. Aynı eksik K-356 tasarım maddesinde üç ürün tekrarıyla kayıtlıdır. | spec/01; `docs/hata-katalogu.md` S040; K-356 |

**ZM-F020 spec uyumu (ayrı rapor).** Davranış bugünkü spec metniyle
çelişmez: spec/12 üretimi giriş oran sınırına bağlar ama aşım yanıtını
yazmaz; spec/11 §5 C013/C021/C027/C023/C024 dışındaki bütün C tanılarını
500'e çevirir. Bu yüzden bu RFC onu correctness hatası olarak değil, iki
**spec boşluğu** olarak kaydeder: (a) aynı kovanın doğrulamada 429,
üretimde 500 vermesi tutarsız; geçici bir koşulun (oran) kalıcı sunucu
hatası gibi görünmesi istemciyi yanlış yönlendirir; (b) üretim, kimliği
zaten doğrulanmış oturumda bile giriş deneme bütçesini tüketir. Düzeltme
yönü (merkezin ayrı işine aday, bu taslakta kod yok): üretim aşımı en azından
doğrulamayla aynı 429'a eşlenmeli ve/veya `parolanın argon2 özeti`nin
`Sonuç` döndüren `… üretmeyi dene` biçimi eklenmeli; kimlikli oturumda
üretim ayrı bir kovaya (ör. kullanıcı başına) alınmalı. Katman E'deki
uygulama parolası Argon2 kullanmadığı için bu RFC'nin kendi akışı
kovaya hiç dokunmaz; ama hesap parolası değişikliği ve web girişi aynı
kovayı paylaşmaya devam eder.

Ek bulgu (ürün listesinde yoktu): DAV kaynak yolları
(`/dav/takvimler/<kullanıcı>/<takvim>/<uid>.ics`) için yol parametresi yoktur;
yalnız tam yol ve `"…" önekli adrese istek geldiğinde` (K-055) vardır. Önekli
rota ve `istek`in `"yol"` alanıyla Zee'de elle bölme mümkündür; bu RFC yeni yol
deseni sözdizimi önermez (Açık sorular).

Sonuç: bugün Zee web adaptörüyle CalDAV/CardDAV sunucusu yazılamaz. Eksik tek
bir özellik değil, yedi bağımsız katmandır; ancak hiçbiri yeni TCP
yetkinliği gerektirmez.

## Tasarım ilkeleri

1. **Protokol Zee'dedir.** PROPFIND/REPORT yanıt içeriği, `getetag`/`getctag`,
   `sync-token` ve `sync-collection`, `calendar-query` filtreleri, iCalendar/
   vCard alan doğrulaması ve yetkilendirme Zee kodudur. "CalDAV sunucusu
   çalıştır" biçiminde bir host intrinsic'i reddedilir (RFC-0039 ilke 1,
   ADR-011 alan adaptörü sınırı).
2. **Kapalı kayıt, açık builder yok.** Yöntem, içerik türü ve yanıt başlığı
   **kapalı kayıtlardır**; her öğe K-167 dil yüzeyi fixture'ına girer. V1'in
   "dinamik status/header builder yok" kararı korunur: rota keyfi başlık adı
   yazamaz, yalnız kayıtlı başlığa doğrulanmış değer verir.
3. **Mevcut güvenlik varsayılanı gevşemez.** Tarayıcı rotaları için oturum,
   CSRF, Origin ve oran sınırı bugünkü gibi kalır. DAV istemci profili ayrı,
   açık bildirilen bir rota sınıfıdır; muafiyet örtük değildir ve yalnız o
   sınıfa uygulanır.
4. **Fail-closed.** Bilinmeyen yöntem, kayıtsız içerik türü, tavanı aşan
   gövde, DTD içeren XML ya da bozuk Basic başlığı rota gövdesi çalışmadan
   sınıflandırılmış durumla reddedilir; kısmi kabul ve sessiz normalleştirme
   yoktur (spec/11 §4 strict çözüm ilkesinin karşılığı).
5. **Grammar en son çaredir.** Önce mevcut kalıplar (yöntemli rota başlığı,
   rota önsözü politika cümlesi, durumlu yanıt, `… dene` ailesi) uzatılır.
6. **Deterministik kanıt.** Okunan başlıklar, Basic kimlik kararı ve XML
   ayrıştırma sonucu spec/21–22 IO izine girer; parola ve uygulama parolası
   maskelidir. `dil dene` hermetik web adaptörü yeni yüzeyi gerçek
   çalışma zamanıyla aynı kurallarla uygular (K-330 modelindeki gibi).

## Katmanlar

Her katman ileride kendi alt RFC'si, ADR'si, spec bölümü, olumlu/olumsuz
testleri, tanı kodları ve güvenlik bulgusu kayıtlarıyla açılır. Aşağıdaki
sözdizimi örnekleri **yalnız fikir taslağıdır**; bağlayıcı değildir ve
freeze yüzeyine yazılmaz.

### A — Yöntem kaydı

- `HttpYontemi` kapalı kaydı şu yöntemlerle genişler: `OPTIONS`,
  `PROPFIND`, `PROPPATCH`, `REPORT`, `MKCOL`, `MKCALENDAR`; `MOVE` ve `COPY`
  isteğe bağlı ikinci adımdır (istemcilerin çoğu kullanmaz; `Destination`
  başlığı ve üzerine yazma semantiği ayrı risk taşır). `LOCK`/`UNLOCK`
  (WebDAV sınıf 2) kapsam dışıdır; `DAV:` başlığı yalnız `1, 3,
  calendar-access, addressbook` ilan eder.
- **Güvenlik sınıfı.** `OPTIONS`, `PROPFIND` ve `REPORT` **salt okuma**
  sınıfındadır: spec/11 §4 T045 kanıtı (yazıcı eylem/çerez/dosya yasağı)
  GET/HEAD ile aynı biçimde uygulanır. `PROPPATCH`, `MKCOL`, `MKCALENDAR`,
  `MOVE`, `COPY` durum değiştiren sınıftadır; spec/12 §1 politika önsözü
  ZORUNLU kalır.
- **Gövdeli salt okuma.** PROPFIND ve REPORT gövde taşır (XML). Bugün
  `govde_kullanilir` yalnız durum değiştiren yöntemlerde açıktır; bu iki
  yöntem gövdeyi form olarak değil yalnız Katman D XML yolundan alır.
- **405 ve Allow.** Yol eşleşip yöntem eşleşmezse 405 yanıtı, o yol için
  tanımlı yöntemlerin kanonik sırasıyla `Allow` başlığını taşır (RFC 9110
  §15.5.6 ZORUNLU). Kaydın tamamen dışındaki yöntem 501'dir.
- **Otomatik OPTIONS.** Rota tanımlamayan yolda OPTIONS'ı çalışma zamanının
  `Allow` + `DAV` ile yanıtlaması mı, yoksa Zee'nin açıkça yazması mı — açık
  soru. Öneri: yalnız DAV istemci profilli rota önekleri için otomatik.
- T047 (aynı yöntem+yol iki kez) yeni yöntemler için de geçerlidir.

Fikir taslağı:

```text
PROPFIND "/dav/" önekli adrese istek geldiğinde
    dav istemcisi gerekli
    ...
```

### B — İstek başlığı okuma yetkinliği

- Rota, **kapalı bir başlık listesinden** tek değer okur; keyfi başlık adı
  verilemez. İlk liste: `Depth`, `If-Match`, `If-None-Match`, `Content-Type`,
  `Prefer` (RFC 8144 `return=minimal`), `Brief` (eski istemciler),
  `Destination` ve `Overwrite` (yalnız MOVE/COPY açılırsa), `User-Agent`
  (yalnız günlük/tanılama; yetki kararı olamaz).
- `Authorization`, `Cookie`, `Forwarded`, `Host`, `Origin` ve `X-Zee-CSRF`
  **listede yoktur**: bunlar çalışma zamanı güven kapılarının girdisidir ve
  Zee'ye ham değer olarak sızmaz. Kimlik Katman E'nin doğruladığı sonuç
  olarak `oturum` sözlüğüne gelir.
- Değer yalnız ASCII görünür karakter ve boşluk taşıyabilir; çoklu
  başlık ya da birleştirme gerektiren alan (ör. iki `If-Match`) rota gövdesi
  çalışmadan 400'dür. Her başlığın sözdizimi çalışma zamanınca doğrulanır:
  `Depth` ∈ {`0`, `1`, `infinity`}, `If-Match`/`If-None-Match` `*` ya da
  tırnaklı ETag listesi. **`Depth: infinity` PROPFIND varsayılan olarak 403**
  (`DAV:propfind-finite-depth`, RFC 4918 §9.1); başlık yoksa PROPFIND için
  `infinity` sayıldığından yine 403'tür — istemciler buna göre `1` gönderir.
- Yüzey `Seçenek` döndürür (başlık yok → boş). Fikir: `isteğin "Depth"
  başlığı`. Kayıt K-167 yüzeyidir; listeye ekleme ayrı kayıt ister.
- IO izine okunan başlık adı + değer girer; maskelenmesi gereken başlık
  listede olmadığı için iz sırrı taşımaz.

### C — Yanıt içerik türü ve başlık kaydı

- **İçerik türü kaydı.** Gövdeli yanıt bugün ilk karakterden tür tahmin
  eder; bu tahmin XML'i `text/html` yapar. Yeni kayıt rota tarafında açık
  türü seçtirir: `application/xml`, `text/calendar`, `text/vcard`
  (`charset=utf-8` sabit), mevcut `text/html` ve `text/plain`. Tür yazılmazsa
  bugünkü kural değişmeden kalır (geriye uyum). Kayıtsız tür yazılamaz.
  `text/calendar`/`text/vcard` yanıtı HTML CSP almaz; `nosniff`,
  `no-store` (aşağıya bak) ve diğer güvenlik başlıkları korunur.
- **Durum.** `207 Multi-Status` ve DAV'ın kullandığı `412`, `415`, `501`, `507`
  durum açıklama tablosuna girer (bugün 207 `Error` açıklaması alır);
  durum hâlâ kaynakta görünen literaldir (S037 korunur).
- **Başlık kaydı.** Rota yalnız şu başlıklara değer verebilir:
  `ETag` (tırnaklı, ASCII, ≤ 128 bayt), `DAV` (çalışma zamanının sabit
  ilanı; Zee yazmaz), `Allow` (çalışma zamanı; Katman A), `Location` (201
  ve 3xx için yerel yol; mevcut yerel yönlendirme doğrulayıcısıyla),
  `Content-Location`, `Preference-Applied`. `WWW-Authenticate` yalnız
  çalışma zamanınca, Katman E reddinde yazılır. Değerler HTTP başlığına
  girmeden spec/12 §4 doğrulayıcılarıyla CR/LF/denetim karakteri bakımından
  denetlenir (C022).
- **Önbellek.** DAV yanıtları `no-store` kalır; koşullu istekler ETag ile
  uygulamada çözülür. RFC-0037/K-335 önbellek politikası DAV rotalarında
  yasaktır (kimliğe bağlı içerik).
- **Gövde boyutu.** Yanıt gövdesi `Metin` olarak kurulduğundan spec/24 adım
  ve heap zarfı geçerlidir. Büyük takvim için `sync-collection` ve
  sayfalama (RFC 6578 `DAV:number-of-matches-within-limits`) Zee
  sorumluluğudur; dosyadan yanıta akıtma (büyük ek, `GET` ile .ics) RFC-0039
  katman D tutamak modelinin HTTP karşılığıdır ve ayrı ikinci adımdır.

Fikir taslağı:

```text
    yanit 207 durumuyla "application/xml" türünde gönder
    "\"" ile etiket ile "\"" ETag başlığıyla
```

### D — XML gövde sınırları ve akış

- **Gövde kaynağı.** PROPFIND, PROPPATCH, REPORT, MKCOL, MKCALENDAR
  gövdesi `application/xml` ya da `text/xml`; PUT gövdesi `text/calendar`
  ya da `text/vcard`. Bu türlerde gövde **form çözümüne girmez**; rota onu
  ayrı bir kaynaktan alır. Diğer türler bugünkü form/binary yollarında kalır;
  tür ile yöntem uyuşmazlığı 415'tir.
- **Tavanlar.** XML gövde: varsayılan 64 KiB (bugünkü gövde tavanı), proje
  bildirimiyle üst tavan 1 MiB'a kadar. iCalendar/vCard: varsayılan 1 MiB,
  üst tavan 16 MiB (spec/26 yükleme tavanı). Sınırın üstü gövde okunmadan
  413'tür (mevcut `Content-Length` ön denetimi).
- **Güvenli XML ayrıştırma.** XML, çalışma zamanında doğrulanmış bir
  crate ile (aday: `quick-xml`; seçim alt RFC'de RustSec/`cargo-deny` ile)
  ayrıştırılır ve Zee'ye **sınırlı bir ağaç değeri** olarak (ad alanı URI +
  yerel ad, öznitelikler, metin) `Sözlük`/`Liste` iç içe biçiminde verilir.
  Değişmezler:
  - `<!DOCTYPE` içeren her belge reddedilir (DTD yok → harici varlık, XXE,
    parametre varlığı, "milyar gülüş" genişlemesi yapısal olarak imkânsız);
    yalnız önceden tanımlı beş varlık ve sayısal karakter başvurusu çözülür;
  - işleme talimatı (`<?xml` bildirimi dışında) ve dış kaynak çözümü yok;
  - derinlik ≤ 32, öğe sayısı ≤ 10 000, öznitelik/öğe başına ≤ 32, ad ≤ 256
    bayt, metin düğümü ≤ gövde tavanı; aşım 413 değil 400 + sınıflandırılmış
    neden (gövde boyutu sınırın altındadır);
  - yalnız UTF-8; BOM kabul, başka kodlama bildirimi reddedilir;
  - ad alanı öneki çözülmüş olarak verilir (`DAV:` + `propfind`), önek
    adı uygulamaya güvenilir girdi değildir.
- **iCalendar/vCard.** Ayrıştırma Zee kodudur (satır katlama açma, `\r\n`
  zorunluluğu). Gövde 1 MiB'ı aşabileceği ve yorumlayıcıda satır satır
  işlemek pahalı olduğu için iki yol vardır: (a) küçük gövde `Metin`;
  (b) büyük gövde spec/26 gibi `0600` geçici dosyaya akar, SHA-256 aynı
  akışta, Zee yol + özet + bayt görür ve satır okuyucuyla işler. (b),
  RFC-0039 katman D tutamak modeliyle **aynı ilkeldir**; ikisi ayrı ayrı
  yazılmaz.
- **CRLF (ZM-F004).** iCalendar/vCard yanıtı CRLF satır sonu ister;
  Zee'de `\r` kaçışı yoktur. Üç seçenek karşılaştırıldı:

  | Seçenek | Ne yapar | Artı | Eksi / risk |
  |---|---|---|---|
  | (i) Yanıt yüzeyi çevirir | `text/calendar`/`text/vcard` türlü yanıtta çalışma zamanı her çıplak `\n`'yi `\r\n`'ye çevirir; gövdede çıplak `\r` varsa yanıt C022 ile reddedilir | Dil değişikliği yok; Zee kodu CR'yi hiç görmez ve üretemez, CR'li satır enjeksiyonu yapısal olarak kapalı; tür kaydına (Katman C) bağlı tek kural | Yalnız bu iki tür için geçerli; Zee'de satır uzunluğu (75 oktet katlama) hesabı CR'siz yapılır; istek tarafında gelen CRLF'nin LF'ye normalleştirilmesi ayrıca tanımlanmalı |
  | (ii) Dile `\r` kaçışı (K-356) | Sözcükleyici `\r`'yi kabul eder | Genel çözüm; RFC-0039 SMTP/IMAP ve başka biçimler de yararlanır | Sözcükleyici/grammar değişikliği, syntax freeze sonrası; her yanıt/başlık/dosya yolunda CR'nin artık Zee değeri olabilmesi, CRLF enjeksiyon savunmasını her çıkışın kendi doğrulayıcısına yükler |
  | (iii) Kayıtlı yardımcı | `metnin crlf biçimi` gibi tek intrinsic LF→CRLF çevirir | Grammar değişmez | Zee'ye CR'li `Metin` sızar (ii ile aynı çıkış riski), yalnız daha dar; unutulursa istemci uyumsuzluğu sessiz kalır |

  **Öneri: (i).** Satır sonu bir tel biçimi ayrıntısıdır ve RFC-0039
  katman A'daki "CRLF'yi çalışma zamanı ekler, Zee metni CR/LF içeremez"
  ilkesiyle aynıdır; iki RFC aynı kuralı paylaşır. Gelen `text/calendar`/
  `text/vcard` gövdesinde yalnız CRLF kabul edilir, çıplak CR/LF 400'dür
  (strict ilke) ve rotaya LF'ye normalleştirilmiş `Metin` olarak verilir.
  (ii) K-356 kendi gerekçesiyle ilerleyebilir; ama DAV yüzeyi ona bağlanmaz.
  Enjeksiyon açısından (i) en güçlüsüdür: başlıklar zaten C022 ile CR/LF
  reddeder, gövdede CR üretilemez, katlama/satır sınırı çalışma zamanının
  tek kuralıdır.
- **Çıktı.** XML yanıtı Zee'de metin birleştirmeyle kurulabilir, ama kaçış
  hatası (enjeksiyon) riskini azaltmak için en az bir `XML metni kaçışla`
  yardımcısı (`& < > " '`) gerekir; tam XML yazıcı ağacı ikinci adımdır.

### E — Çerezsiz Basic ve uygulama parolası doğrulama modeli

Senkron istemciler çerez tutmaz (ya da tutsa da güvenilmez) ve her isteği
`Authorization: Basic` ile gönderir. Hesap parolasını her istekte Argon2id ile
doğrulamak hem CPU açısından savunulamaz hem de spec/12 §5 parola oran
sınırıyla çakışır. Model:

- **Uygulama parolası.** Kullanıcı web arayüzünde (oturum + CSRF + mevcut
  Argon2id giriş) cihaz başına bir uygulama parolası üretir. Değer
  CSPRNG'den en az 128 bit (öneri: 160 bit, Base32, gruplanmış gösterim),
  yalnız üretim anında bir kez gösterilir. Hesap parolası DAV'da **kabul
  edilmez**; Basic yalnız uygulama parolasını doğrular. İki adımlı doğrulama
  açık hesapta da çalışır (uygulama parolası ikinci etken sayılmaz, ayrı
  kimliktir ve tek tek iptal edilebilir).
- **Hızlı ama güvenli doğrulama.** Uygulama parolası yüksek entropili
  olduğundan yavaş KDF gerekmez. Depoda yalnız
  `HMAC-SHA256(sunucu_anahtarı, parola)` (anahtar mevcut
  `gizli_anahtar_değişkeni`/systemd credential modelinden; K-339) ve kısa,
  gizli olmayan bir arama öneki tutulur; karşılaştırma sabit zamanlıdır.
  Ham değer ve anahtar Zee değeri olmaz. Alternatif (yalnız SHA-256) açık
  soru olarak not edilir; HMAC, depo sızıntısında çevrimdışı denemeyi
  anahtara bağlar.
- **Kimin yazdığı.** Uygulama parolası kayıtları ürün verisidir (PostgreSQL),
  ama doğrulama çalışma zamanı ilkelidir: Zee, `uygulama parolası doğrulama`
  kaydını (kullanıcı, önek, HMAC, kapsam, son kullanım) bir sorgu
  sözleşmesiyle sağlar; Basic başlığını çalışma zamanı çözer, Zee yalnız
  sonucu (`oturum` sözlüğü: `kullanıcı`, `rol`, `kimlik_turu=uygulama`,
  `uygulama_parolasi_kimligi`) görür. Kesin sınır (çalışma zamanı mı ürün
  sorgusu mu) alt RFC'nin ana kararıdır; Açık sorular.
- **Kapsam.** Uygulama parolası yalnız DAV istemci profilli rotalarda
  geçerlidir; tarayıcı rotasında Basic yok sayılır (oturum çerezi gerekir).
  İsteğe bağlı salt-okuma kapsamı (yalnız PROPFIND/REPORT/GET).
- **Oran sınırı.** Üç ayrı kapı:
  1. **Başarısız Basic** (istemci kimliği + kullanıcı adı başına):
     60 sn'de 10, aşımda 429 + `Retry-After`; başarısız deneme sayacı
     başarılı doğrulamayla sıfırlanmaz (deneme bütçesi tükenir).
  2. **Başarılı istek** spec/12 §5 uç nokta kapısına (100/60 sn) tabidir;
     ilk eşitlemede yüzlerce GET üreten istemciler için DAV öneki başına
     ayrı ve daha yüksek eşik (ör. 600/60 sn) — sayı ölçüm sonrası bağlanır.
  3. Argon2id parola kapısı (5/300 sn) **değişmez** ve DAV'da hiç
     tetiklenmez, çünkü hesap parolası DAV'da doğrulanmaz.
- **Ret biçimi.** Başlık yok ya da bozuk → 401 +
  `WWW-Authenticate: Basic realm="<kanonik origin>", charset="UTF-8"`;
  doğru kimlik yanlış kapsam → 403. Basic yalnız `--web-proxy` (HTTPS)
  profilinde kabul edilir; `--deneysel-web` düz HTTP'de reddedilir.
- **Yaşam döngüsü.** İptal anında etkilidir (ortak kalıcı depo; worker ve
  restart sınırı spec/12 §7 ile aynı); son kullanım anı ve istemci etiketi
  (User-Agent'tan değil kullanıcının verdiği addan) ürün arayüzünde görünür.
  Hesap parolası değişince tüm uygulama parolalarının iptali ürün
  politikasıdır (öneri: varsayılan iptal).

### F — CSRF ve Origin muafiyet kuralı

Muafiyet bir *ayrıcalık* değil, *tehdit modelinin farkı*dır: CSRF tarayıcının
ortam kimlik bilgisini (çerez) kendiliğinden eklemesine dayanır. Basic ile
gelen ve çerez kimliği kullanmayan istek tarayıcı tarafından sahte üretilse
bile kimlik bilgisini saldırgan bilmek zorundadır. Kural:

- Rota önsözüne yeni bir politika cümlesi: fikir `dav istemcisi gerekli`
  (T049/T050 ilk-ve-tek politika kuralına uyar). Bu cümle:
  - kimliği **yalnız** Katman E Basic + uygulama parolasından alır;
    istekte oturum çerezi varsa yok sayılır ve rota `oturum`u çerezden
    asla doldurmaz;
  - `_csrf` alanını ve `--web-proxy` `Origin` zorunluluğunu bu rota için
    kaldırır;
  - **ama** istekte `Origin` başlığı varsa ve yapılandırılan origin değilse
    403 verir (tarayıcıdan gelen çapraz köken isteği hiçbir durumda geçmez);
  - CORS yanıtı üretmez (`Access-Control-*` yok), dolayısıyla tarayıcı
    JavaScript'i yanıtı okuyamaz;
  - yalnız Katman A yöntemleri + GET/HEAD/PUT/DELETE ile birleşebilir; POST
    ile birleşemez (form gönderimiyle sahte üretilebilen tek yöntem budur;
    DAV POST kullanmaz).
- Bu cümleyi taşıyan rota `csrf belirteci`, `oturuma al`, `oturumu kapat`,
  çerez yazma/silme ve web oturumu listeleme/iptal kullanamaz (T057
  benzeri statik kapı; geçişli).
- Tarayıcı Basic kimlik önbelleği riski: tarayıcı aynı origin için Basic
  bilgisini hatırlarsa ortam kimliği gibi davranır. Bu yüzden DAV öneki
  ayrı bir alt alan adında (ör. `dav.<alan>`) önerilir ve 401 yanıtı yalnız
  DAV önekinde `WWW-Authenticate` taşır; aynı origin zorunlu kalırsa yukarıdaki
  `Origin` reddi + POST yasağı + `SameSite` bağımsızlığı birlikte ele alınır
  — açık soru.

### G — Well-known keşif

- `/.well-known/caldav` ve `/.well-known/carddav` (RFC 6764 §5) için GET,
  HEAD ve PROPFIND, DAV bağlam köküne yönlendirme ister. Bugünkü yerel
  yönlendirme (303) GET için yeterlidir; PROPFIND için Katman A gerekir ve
  yönlendirme kodu seçimi (301/307/308) durumlu yönlendirme olarak açılır.
  Öneri: çalışma zamanı bu iki yolu, projede bildirilen DAV köküne
  (`dav_kökü: "/dav/"` gibi proje alanı) **kendisi** 301 ile yönlendirir;
  böylece her ürün aynı keşif hatasını tekrar etmez.
- Keşfin geri kalanı (`DAV:current-user-principal`,
  `calendar-home-set`, `addressbook-home-set`) Zee'de PROPFIND yanıtıdır.
- **DNS SRV/TXT** (`_caldavs._tcp`, `_carddavs._tcp`, `TXT path=`) dağıtım
  katmanıdır; dil yüzeyi açılmaz, `docs/web-production-profili.md`'ye
  operatör notu olarak girer. Apple istemcileri ve DAVx5 SRV'yi, Thunderbird
  well-known'u kullanır; ikisi birlikte önerilir.
- Ters vekil `/.well-known/` yolunu Zee'ye geçirmek zorundadır; bazı
  barındırma panelleri bu yolu ACME için ayırır — operatör notu.

## Güvenlik etkisi

| Sınıf | Risk | Bu taslaktaki karşılık |
|---|---|---|
| Kimlik | Basic her istekte parola taşır; sızan parola kalıcı erişim | Hesap parolası DAV'da kabul edilmez; cihaz başına 128+ bit uygulama parolası, HMAC ile saklanır, tek tek iptal edilir, yalnız HTTPS proxy profili, başarısız deneme oran kapısı |
| Kimlik | Kullanıcı adı numaralandırma | 401 gövdesi ve süresi kullanıcı varlığından bağımsız; sabit zamanlı karşılaştırma; başarısız kapı kullanıcı adı + istemci başına |
| Kimlik | Hızlı doğrulamanın çevrimiçi kaba kuvvete açılması | 128+ bit entropi çevrimiçi denemeyi anlamsızlaştırır; yine de 10/60 sn kapısı ve GB kaydı |
| CSRF | Muafiyetin tarayıcı rotalarına sızması | Muafiyet yalnız açık `dav istemcisi gerekli` rotası; çerez kimliği o rotada yok sayılır; yabancı `Origin` 403; POST ile birleşemez; statik T057 benzeri kapı |
| XXE / XML bomba | Harici varlık, dosya okuma, SSRF, üstel genişleme | DOCTYPE reddi (DTD hiç işlenmez), dış kaynak çözümü yok, derinlik/öğe/ad/öznitelik tavanları, yalnız UTF-8 |
| Büyük gövde | Bellek/heap tükenmesi, yavaş gövde | Tür başına tavan, `Content-Length` ön denetimi ve 413, 10 sn mutlak okuma bütçesi (spec/11 §5) değişmez, büyük iCalendar diske akar (spec/26 modeli) |
| Başlık enjeksiyonu | ETag/Location değerinde CR/LF ile yanıt bölme | Kapalı başlık kaydı; değer doğrulayıcıları (spec/12 §4 C022) ve ETag/Location sözdizimi denetimi; keyfi başlık adı yok |
| CRLF | iCalendar/vCard gövdesinde CR ile satır/özellik enjeksiyonu | Zee CR üretemez; çıkışta yalnız LF→CRLF çevirisi, çıplak CR C022; girişte çıplak CR/LF 400 |
| Oran / kullanılabilirlik | Parola özeti üretiminin giriş kovasını tüketip 500 vermesi (ZM-F020) | DAV akışı Argon2 kullanmaz; üretim aşımının 429/`Sonuç` olması ve ayrı kova ayrı fix adayı |
| Başlık okuma | Güven başlıklarının (Authorization, Forwarded…) Zee'ye sızması | Okuma listesi kapalı ve güven başlıklarını içermez |
| Yol | `Depth: infinity` ile tüm ağaç dökümü (DoS) | Varsayılan 403 `propfind-finite-depth` |
| Yol | MOVE/COPY `Destination` ile kök dışı yazma | İkinci adım; açılırsa yalnız aynı origin + DAV öneki içi, `..` ve yüzde kodlu ayraç reddi |
| İz | Uygulama parolasının IO izine düşmesi | `Authorization` maskeli; iz yalnız kimlik kararı + uygulama parolası kimliğini taşır |

Her bulgu ve bilinçli sınır, uygulama işinde `docs/guvenlik-bulgulari-v1.tsv`
GB satırı alır. Bu taslak kod değiştirmediği için yeni GB satırı açmaz.

## Core freeze gerekçesi ve kanıt kapısı

Core freeze altında yeni compiler özelliği varsayılan olarak reddedilir. Bu
RFC'nin hiçbir katmanı şu zincir kurulmadan uygulanmaz:

1. **Ürün kaydı.** Kullanan ürün `urun-h`dir; K-355 ile
   `docs/dogfood-projeleri-v1.tsv` içinde etkin, exact harici köken ve depo
   içi kanıt kökü [`dogfood/urun-h`](../dogfood/urun-h/README.md) ile kayıtlı.
   Bu adım karşılanmıştır.
2. **Gerçek Zee kodu.** Ürünün web tarafındaki takvim/kişi/görev kodu Zee
   + PostgreSQL ile çalışıyor. DAV sürtünmesinin depo içi reproducer'ları
   26 Kasım 2026'da korpusa girdi (ürün sahibi kararı “önce reproducer”):
   A, B, C, E ve F katmanları için dokuz tek dosyalı vaka
   `dogfood/urun-h/korpus/rfc0040-*.dil` altında ürün politikasıyla S036,
   S015, S037, T050, T060 ve T062 verir; köken ve K-164 karşılaştırması
   [`dogfood/urun-h/dav-reproducer.md`](../dogfood/urun-h/dav-reproducer.md).
   D (XML gövdesi) ve G (well-known) için ayrı vaka yoktur: D'nin gövde
   sürtünmesi `rfc0040-f-oturumlu-ham-govde` (T062) ile birlikte görünür, G
   bugün GET/303 ile yazılabilir. Alt RFC kararı bu reproducer'lardan sonra
   verilir ve büyük olasılıkla V1 sonrasıdır; reproducer'ların varlığı hiçbir
   katmanı açmaz.
3. **Katman başına zincir.** Alt RFC → ADR → spec maddesi +
   `spec-madde-kaniti` satırı → olumlu/olumsuz test → reproducer → K-167
   yüzey kaydı (yöntem, başlık adı, içerik türü, politika cümlesi, tanı
   kodu) → güvenlik bulgusu kaydı aynı değişiklikte.
4. **Sıra önerisi.** E+F (kimlik ve muafiyet; tasarım riski en yüksek) ile
   A (yöntem) birlikte ilk alt RFC'dir, çünkü tek başına açılan bir yöntem
   kaydı kimliksiz kullanılamaz. Ardından B+C (başlık okuma/yazma), D (XML),
   G (well-known). Sıra değişirse gerekçe ADR'ye yazılır. **27 Kasım 2026:**
   ilk alt RFC [RFC-0047](0047-dav-istemci-rotasi-ilk-dilim.md) ürünün sıra
   notuna göre A + dar B + dar C + E + F kesitini seçti; sapmanın ürün
   kanıtına dayalı gerekçesi orada yazılıdır ve uygulama ADR'sine taşınır.
5. **Grammar.** Yeni politika cümlesi ve yöntem adları mevcut rota başlığı
   ve önsöz kalıplarının uzantısıdır; yine de K-167 yüzey kaydı ve
   RFC-0028 deprecation kuralları geçerlidir. V1 syntax freeze penceresi
   içinde açılmaz.

V1 kapı listesine (`docs/v1-surum-kapilari.md`) bu RFC eklenmez: DAV
yüzeyi V1 sözü değildir.

## RFC-0039 ile ilişki

- **Ayrı hat.** RFC-0039 HTTP dışı TCP sunucusu (SMTP/IMAP) içindir: yeni
  dinleyici, bağlantı görevi, engellemeyen IO ve sunucu TLS'i ister. Bu RFC
  mevcut HTTP adaptörünün içinde kalır; RFC-0039'un A, B, C, E katmanlarına
  **bağımlı değildir** ve onlardan önce açılabilir.
- **Ortak ilkel.** Büyük iCalendar/vCard gövdesi ve dosyadan yanıta akıtma,
  RFC-0039 katman D'nin tutamak (dosya + SHA-256 + bayt sayısı) modelini
  kullanır; aynı ilkel iki kez tasarlanmaz. Hangi RFC önce kabul edilirse
  ilkeli o tanımlar, öteki ona bağlanır.
- **Ortak kimlik.** IMAP/SMTP AUTH (RFC-0039) ve DAV Basic aynı uygulama
  parolası modelini kullanmalıdır: posta istemcisi ve takvim istemcisi aynı
  cihazda aynı uygulama parolasıyla kurulabilir. Katman E'nin doğrulama
  ilkeli bu yüzden HTTP'ye özel değil, protokol bağımsız tanımlanır.
- **Ortak ürün.** İkisi de `urun-h` sürtünmesinden büyür; reproducer'lar
  aynı korpus kökünde, ayrı Fnnn kimlikleriyle tutulur.

## Dört soru süzgeci

1. **Türkçe doğal mı?** — Yöntem adları protokol sabitidir ve bugünkü `GET`
   /`POST` gibi büyük harfle yazılır. Politika cümlesi (`dav istemcisi
   gerekli`) mevcut `oturum gerekli` ritmini izler. Kesin yüzey alt RFC'de
   kullanılabilirlik kartıyla seçilir.
2. **Deterministik mi?** — Başlık okuma, kimlik kararı ve XML ağacı IO
   izine girer; aynı izle aynı yanıt. Uygulama parolası maskelidir.
3. **Öğrenilebilir mi?** — DAV protokolünün kendisi karmaşıktır; dil
   yüzeyi yalnız "yöntem + başlık oku + XML ağacı + türlü yanıt + ETag" beş
   kavramı ekler. Ürün tarafında örnek bir minimal CalDAV koleksiyonu golden
   değil dogfood dokümanı olarak verilir.
4. **Profesyonel ölçekte savunulabilir mi?** — XXE/bomba yapısal olarak
   kapalı, uygulama parolası endüstri pratiği (Apple, Google, Fastmail,
   Nextcloud aynı modeli kullanır), CSRF muafiyeti tehdit modeline dayalı ve
   dar, kabul ölçütleri gerçek istemcilerle.

## Kabul ölçütleri

Bu taslak ancak şunlar kanıtlandığında "geçici kabul"e aday olur:

1. Zee ile yazılmış en küçük CalDAV + CardDAV sunucusu `--web-proxy`
   arkasında şu istemcilerle hesap kurar, takvim/kişi oluşturur, düzenler,
   siler ve iki cihaz arasında eşitler: Thunderbird (Linux/Windows/macOS),
   macOS Takvim/Kişiler, iOS, DAVx5 (Android), GNOME Evolution. Windows'ta
   yerleşik Outlook CalDAV/CardDAV desteklemediği için kapsam açıkça
   Thunderbird ile sınırlıdır.
2. RFC 4791/6352 uyumluluk sınaması (ör. CalDAVTester ya da eşdeğeri)
   çalıştırılır; geçmeyen kalemler bilinçli sınır olarak listelenir.
3. Olumsuz testler: DOCTYPE/harici varlık/derin iç içe XML, tavan üstü gövde,
   `Depth: infinity`, CR/LF'li ETag, yabancı `Origin`, çerezli ama Basic'siz
   DAV isteği, hesap parolasıyla Basic, iptal edilmiş uygulama parolası,
   başarısız Basic oran kapısı.
4. İlk eşitleme yükü (≥ 1000 olay, ≥ 1000 kişi) ölçülür; uç nokta oran
   eşiği ve yanıt süresi sayıya bağlanır, spec/24 bütçesi aşılmaz.
5. Aynı oturum IO izinden replay edilir ve aynı çıktıyı üretir.

## Alternatifler

- **Ayrı DAV sunucusu (Radicale, Baïkal, Rust/Go) + PostgreSQL paylaşımı:**
  ürünün "her şey Zee ile" kararıyla çelişir; iki veri modeli ve iki kimlik
  deposu doğurur; dilin kendi web yetkinliğini kanıtlamaz.
- **Ters vekilde Basic → çerez çevirisi:** kimlik kararını Zee dışına taşır,
  vekili güven sınırının parçası yapar ve uygulama parolası iptalini
  doğrulayamaz.
- **Genel başlık okuma/yazma API'si (keyfi ad):** V1 "dinamik header
  builder yok" kararını bozar; güven başlıklarının sızmasını tek tek
  engellemek gerekir. Kapalı kayıt daha dar ve denetlenebilir.
- **Hesap parolasıyla Basic + daha gevşek Argon2 oranı:** her istekte
  Argon2id CPU maliyeti, parolanın cihazlarda saklanması ve oran sınırının
  zayıflaması; reddedildi.
- **OAuth 2 / bearer token:** CalDAV/CardDAV istemcilerinin çoğu (özellikle
  Apple ve Thunderbird'ün genel sağlayıcı yolu) yalnız Basic/Digest destekler;
  ilk adım için uygulanabilir değil. İleride ek yöntem olarak açık.
- **Digest kimlik doğrulama:** sunucuda geri çevrilebilir parola türevi
  saklamayı gerektirir ve TLS altında Basic'e güvenlik katmaz; reddedildi.
- **Gövdeyi her zaman binary yükleme yoluyla diske akıtıp Zee'de XML
  ayrıştırmak:** yorumlayıcıda XML ayrıştırıcısı yazmak hem pahalı hem
  XXE/bomba korumasını ürün koduna bırakır; reddedildi.

## Korpus etkisi

Taslağın kendisi golden, regression ve dil yüzeyi fixture'ını değiştirmez.
Dogfood korpusunda bugün dokuz `rfc0040-*` ret vakası vardır (K-364); her
katman açıldığında ilgili `urun-h` reproducer'ı ret/çözüm çiftine dönüşür.

## Açık sorular

- Uygulama parolası doğrulaması çalışma zamanı ilkeli mi (kayıt deposu
  çalışma zamanının ortak kalıcı deposu), yoksa Zee'nin PostgreSQL sorgusuyla
  sağladığı ve çalışma zamanının yalnız HMAC karşılaştırması yaptığı bir
  geri çağrı mı? İkincisi Basic kararını rota çalışmadan önce vermeyi
  zorlaştırır.
- HMAC anahtarı mevcut `gizli_anahtar_değişkeni` mi, ayrı credential mı;
  anahtar döndürmede eski parolaların geçişi.
- Otomatik OPTIONS yanıtı yalnız DAV önekinde mi, her yolda mı?
- DAV öneki ayrı alt alan adı (ayrı origin) olarak mı zorunlu, yoksa aynı
  origin + `Origin` reddi + POST yasağı yeterli mi (tarayıcı Basic önbelleği)?
- Yol parametresi: önekli rota + Zee'de elle bölme yeterli mi, yoksa
  `"/dav/takvimler/{kullanıcı}/"` benzeri desen ayrı RFC mi? (Grammar eşiği
  yüksek; şimdilik elle bölme.)
- XML yanıt yazımı için yalnız kaçış yardımcısı mı, yoksa ağaçtan XML yazıcı
  mı?
- MOVE/COPY ve `If` başlığı (kilit belirteçleri olmadan) ilk sürümde gerekli
  mi? Apple istemcilerinin davranışı ölçülmeli.
- Senkron belirteci (`sync-token`) ve `getctag` için ürün PostgreSQL'inde
  değişiklik günlüğü modeli — ürün kararı, dil kapsamı dışında; ama büyük
  `sync-collection` yanıtının spec/24 heap zarfına sığıp sığmadığı ölçüm
  konusudur.
- ZM-F020: `parolanın argon2 özeti` aşımı 429'a mı eşlenmeli, `Sonuç` döndüren
  ayrı biçim mi açılmalı, yoksa ikisi birden mi; kimlikli oturumda üretim
  kovasının anahtarı (kullanıcı mı istemci mi)?
- CRLF çevirisinin (i) yalnız DAV türlerinde mi, yoksa RFC-0039 satır
  yazıcısıyla ortak tek çalışma zamanı kuralı olarak mı tanımlanacağı.
- Ürünün sıra notu (26 Kasım 2026, reproducer'larla birlikte): E+F+A tek
  başına çalışan istemci vermez — başlıksız PROPFIND `Depth: infinity`
  sayılıp 403 alır, multistatus 207 + `application/xml` ister. En küçük
  çalışan uç A + B (`Depth`, `If-Match`, `If-None-Match`) + C (207 + XML
  türü, ETag) + E + F'dir. Salt okunur abonelik bugün takvim yayınıyla
  (K-406) var; asıl kazanç iki yönlü eşitleme olduğundan F, PUT ve DELETE'i
  kapsamalıdır. E'de istek başına Argon2id doğrulaması spec/12 §5 oran
  sınırına takılır (bir PROPFIND + REPORT + N GET turu altıncı istekte 429);
  bu HMAC modelini (E) zorunlu kılar. Sıra önerisi (madde 4) bu nota göre alt
  RFC'de yeniden değerlendirilir. → [RFC-0047](0047-dav-istemci-rotasi-ilk-dilim.md)
  bu kesiti tasarlar (taslak, 27 Kasım 2026).
- İstemci IP'si (ZM-F027, `zm-f027-istemci-ip`): DAV dışı ama aynı web
  adaptörü sürtünmesi. Çözüm istek sözlüğü anahtarı ya da form alanı
  olamaz (değeri saldırgan belirler); ADR-036 güvenilir vekil modeline göre
  çözülmüş ayrı bir yüzey gerekir. Bu RFC'nin katmanı değildir.
- Zamanlama/davet (CalDAV scheduling, RFC 6638: `schedule-inbox`,
  iTIP e-postası) bu RFC kapsamı dışında; posta tarafıyla (RFC-0039)
  birleştiği yerde ayrı RFC.

## İlgili kayıt

K-364 (backlog ve karar günlüğü). Uygulama yok; bu belge yalnız altı eksiğin ve iki ek
sürtünmenin (ZM-F020, ZM-F004) doğrulamasını, yedi katmanı, güvenlik etkisini ve kabul ölçütlerini önden
bağlar.
