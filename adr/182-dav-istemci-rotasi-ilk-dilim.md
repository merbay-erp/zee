# ADR-182 — DAV istemci rotası: RFC-0047 uygulaması, dilim sırası ve açık soruların kapanışı

- **Durum:** kabul — dört dilim (A, B, C, D) uygulandı (28 Kasım 2026)
- **Tarih:** 28 Kasım 2026
- **İlgili kayıt:** K-364, [RFC-0047](../rfcs/0047-dav-istemci-rotasi-ilk-dilim.md),
  [RFC-0040](../rfcs/0040-webdav-caldav-carddav-adaptoru.md), ADR-059 (core
  freeze), ADR-064 (dil yüzeyi), ADR-067 (dogfood korpusu), ADR-163 K-406 eki
  (türlü metin yanıtı), ADR-178 (rota sonu), ADR-179 (makine uç noktası),
  ADR-036 (kanonik proxy origin), spec/11 §4, spec/12 §RFC-0047, GB-185, GB-186, GB-187, GB-188

## Bağlam

Ürün H (ZeeMail, `urun-h`) CalDAV/CardDAV sunmak istiyor; 26 Kasım 2026'da
on iki reproducer verdi, dokuzu RFC-0040 içindir
([`dogfood/urun-h/dav-reproducer.md`](../dogfood/urun-h/dav-reproducer.md)).
RFC-0047 (27 Kasım) bunların "en küçük çalışan uç" tasarımını yaptı; ürün 13
açık sorunun hepsine görüş verdi. Proje sahibi 28 Kasım 2026'da "RFC-0047 ilk
dilimini uygula" kararını verdi; iş `dogfood-change` (semantic-change)
sınıfındadır. Core freeze (ADR-059) altında yüzey ürün kanıtına göre en dar
biçimde ve dört ayrı PR ile açılır.

## RFC-0040 sırasından sapmanın gerekçesi

RFC-0040 "Core freeze gerekçesi" madde 4, sıra değişirse gerekçenin ADR'ye
yazılmasını ister. RFC-0040 "E+F ile A birlikte" önerdi; bu ADR A + dar B +
dar C + E + F'yi birlikte açar:

1. **E+F+A tek başına çalışan istemci vermez.** `rfc0040-b-depth-basligi`
   `Depth`in okunamadığını gösterir; RFC-0040 B'ye göre başlıksız PROPFIND
   `infinity` sayılır ve reddedilir. Başlık okunamazsa rota `Depth: 0` ile
   `Depth: 1`i ayıramaz.
2. **Multistatus 207 + `application/xml` ister.** `rfc0040-c-207-xml` K-406
   türlü yanıtla durumlu yanıtın birlikte yazılamadığını (S037) gösterir;
   207'siz PROPFIND yanıtını istemciler reddeder.
3. **Asıl kazanç iki yönlü eşitlemedir.** Salt okunur abonelik bugün K-406
   takvim yayınıyla vardır; yeni değer istemciden yazmadır (PUT, ETag,
   `If-Match`/`If-None-Match`). Bu yüzden F PUT ve DELETE'i kapsar, B/C'nin dar
   kısmı ilk dilime girer.
4. **D ve G'yi ertelemek güvenlik sınırını gevşetmez.** Dil XML ayrıştırmaz
   (ADR-179); ham gövde `Metin` olarak verildiğinde DTD işlenmez, XXE ve
   varlık genişlemesi yapısal olarak yoktur. XML ağacı yeni ayrıştırıcı crate'i
   ister; ayrı alt RFC'de ölçülür. Well-known (G) bugün GET/303 ile yazılabilir.

## Dilim sırası

RFC-0047'nin önerdiği sıra uygulama sırasında yeniden düzenlendi: başlık
okuma, ham gövde ve ETag ürünün görüşüyle (soru 5) yalnız DAV sınıfında
açıldığından sınıf onlardan önce gelmek zorundadır; sınıf ise kimlik
doğrulaması olmadan açılamaz (CSRF'siz, kimliksiz PUT/DELETE olurdu).

| Dilim | İçerik | Durum |
|---|---|---|
| A | Durumlu türlü yanıt (her rota), `text/vcard`, 405 + `Allow` ve 501 (her rota), durum satırı açıklamaları, 204 gövdesizliği | **uygulandı** (bu ADR, GB-185) |
| B | Uygulama parolası ilkeli: `yeni uygulama parolası`, `uygulama_parolası_anahtarları` (kimlikli HMAC credential'ları, döndürme), doğrulama kitaplığı (biçim kapısı, sabit zamanlı özet, katı Basic, ASCII katlama) | **uygulandı** (dilim B eki, GB-186) |
| C | `dav istemcisi gerekli` sınıfı ve `dav_kimlik_çözücüsü` bildirimi (salt okuma kanıtı): PROPFIND/REPORT kaydı, yöntem-politika tanıları, statik kapı, CSRF/Origin muafiyeti, `Sec-Fetch-Site` reddi, çerezsiz Basic doğrulaması, ayrı oran kovaları, salt okuma kapsamı, ayrı DAV origin'i, otomatik OPTIONS | **uygulandı** (dilim C eki, GB-187) |
| D | `Depth`/`If-Match`/`If-None-Match` okuma, DAV ham gövdesi (tür ve tavan kuralları), `propfind-finite-depth` 403, `ETag` yazımı, korpus geçişi ve çözüm vakaları, RFC-0047 durumu | **uygulandı** (dilim D eki, GB-188) |

Her dilim kendi spec maddesi, testi, tanı kataloğu, K-167 satırı, GB satırı
ve `dogfood-change` beyanıyla kapanır. Ara durumda kayıt dışı yöntemler
(PROPFIND, REPORT) 501'dir; sınıf dilim C'ye kadar yoktur, yani hiçbir
CSRF'siz kimlikli yüzey erken açılmaz (fail-closed sıra).

## Karar — dilim A

1. **Durumlu türlü yanıt (RFC-0047 C).** K-406 cümlesinin kardeşi
   `<ifade> metnini "<tür>" türünde <durum> durumuyla gönder`. Durum kaynakta
   görünen literal ve kapalı küme `TURLU_YANIT_DURUMLARI` = 200, 201, 207,
   400, 403, 404, 409, 412, 415; aksi önerili **S037** (yeni tanı kodu yok).
   204 bu kümede yoktur: gövdesiz yanıt `"" yanıtını 204 durumuyla gönder`
   ile yazılır. Kural her rotada geçerlidir (soru 13'ün türlü yanıt karşılığı;
   tür ve durum ikisi de kapalı literaldir, yeni yetenek getirmez). Başlık
   kümesi K-406 ile aynıdır.
2. **`text/vcard`.** İçerik türü kaydına `vcf` satırı (web metin ve dosya
   yanıtı; SMTP adayı değil) ve `WEB_METIN_TURLERI`ne eklendi.
3. **405 + `Allow` (her rota).** Yol eşleşir, yöntem eşleşmezse `Allow` o
   yolun yöntemlerini kanonik kayıt sırasıyla taşır. Gerçek adaptör başlığı
   yazar; hermetik adaptör `izinli_yontemler` alanına kaydeder; IO izi olayı
   değişmez (`durum_yaniti_gonder` 405), çünkü değer program ve yoldan
   deterministik türer — eski izler aynen oynar.
4. **501 (her rota).** `HttpYontemi::KAYIT` dışındaki yöntem yoldan bağımsız
   501'dir; rota, `_csrf` ve zorunlu alan denetimi çalışmaz. OPTIONS bu dilimde
   501'dir; DAV öneki altındaki otomatik OPTIONS yanıtı dilim C'nin işidir.
5. **Durum satırı ve 204.** 200/201/204/207/409/412/415/501 kanonik
   açıklamaları eklendi (önceden `Error`). 204 yanıtı gövde, `Content-Type`
   ve `Content-Length` taşımaz (RFC 9110 §8.6).
6. **Güvenlik sırası (spec/12).** Proxy kapısı (Host, `Forwarded`, HTTPS,
   durum değiştiren yöntemde `Origin`) 501/405'ten önce gelir ve kayıt dışı
   yöntemi durum değiştiren sayar. 405/501 yanıtı `Set-Cookie` taşımaz.

## Açık soruların kapanışı (RFC-0047, ürün görüşü 27 Kasım 2026)

| # | Soru | Karar | Dilim |
|---|---|---|---|
| 1 | Kimlik çözücü | **Kabul:** proje alanıyla bildirilen işlem (`dav_kimlik_çözücüsü`); çalışma zamanı deposu ve rota önsözünde ikinci cümle yok; önbellek yok (iptal bir sonraki istekte etkili). Kullanıcı adı **ASCII küçük harfe katlanarak** karşılaştırılır (Unicode katlama yok; ASCII dışı bayt bayt). `oturum` `kullanıcı` değeri Basic'ten değil çözücünün döndürdüğü kanonik addan gelir. | B, C |
| 2 | HMAC anahtarı | **Kabul:** ayrı credential (`uygulama_parolası_anahtar_değişkeni`, K-339 ortam/systemd credential modeli); `gizli_anahtar`'dan türetme reddedildi. Özet anahtar kimliğini taşır; döndürmede eski anahtar kimliği süreyle değil, ona bağlı kayıt kaldıkça okuma için bildirilir (dil süreyle silmez; "eski anahtarla N parola" sorgusu ürünündür). | B |
| 3 | Biçim ve göç | **Kabul:** `zua1-` + 8 Base32 önek (40 bit, gizli değil) + 32 Base32 gizli (160 bit), dörtlü grup gösterimi. Zorla göç yok; eski Argon2id kayıtları DAV'da kabul edilmez, kapanış ürün kararıdır. | B |
| 4 | `DAV` başlığı | **Kabul:** PROPPATCH/MKCOL/COPY/MOVE yokken sınıf 1/3 iddia edilmez; değer yalnız `calendar-access, addressbook`. Otomatik OPTIONS DAV öneki altında, kimliksiz, CORS'suz; iOS/Thunderbird ölçümü dilim C kabulüdür. | C |
| 5 | Başlık okuma/ETag kapsamı | **Kabul:** ilk uygulamada yalnız DAV sınıfında. | D |
| 6 | `If-Match`/`If-None-Match` türü | **Kabul:** `Liste<Metin>`; `*` listede tek ayrı değerdir (entity-tag tırnaklı olduğundan çakışmaz). | D |
| 7 | Gövde tavanı | **Kısmi kabul:** PUT (`text/calendar`, `text/vcard`) 1 MiB; REPORT ve PROPFIND 256 KiB; sabit yöntem tavanları. Tavanın rota başına bildirimi ve K-391 `geniş_gövde_rotaları` kapsamına alma **reddedildi**: yöntem tavanı ürün ölçümünü karşılar, rota başına bildirim yeni proje yüzeyi açar; ihtiyaç ölçülürse ayrı K işi. | D |
| 8 | Ayrı origin | **Kabul:** üretim profilinde (`--web-proxy`) DAV sınıfı rotalar ayrı bir origin'de zorunludur; çerezli rotalar o origin'de sunulmaz. | C |
| 9 | `--deneysel-web` | **Kabul:** Basic yalnız loopback'te (K-400/ADR-176 deneysel sözleşmesi zaten yalnız localhost); düz HTTP üretimde yoktur. | C |
| 10 | `Sec-Fetch-Site` | **Kabul:** `cross-site` değeri DAV sınıfında rota çalışmadan 403; yabancı `Origin` reddine ek. | C |
| 11 | Salt okuma | **Kabul:** ilk uygulamada; çözücünün `kapsam` = `salt_okuma` kaydında PUT/DELETE 403. | C |
| 12 | Başarılı DAV kovası | **Kabul (aday değerle):** anahtar uygulama parolası kimliği, aday eşik 600/60 sn; kesin sayı ürünle iOS/Thunderbird ilk eşitleme ölçümünde bağlanır. | C |
| 13 | 405 + `Allow` ve 501 | **Kabul:** bütün rotalarda. | **A (uygulandı)** |

## Ek — dilim B: uygulama parolası ilkeli (28 Kasım 2026, GB-186)

1. **İfade.** `yeni uygulama parolası` (`kriptografi`, `guvenlik.uygulama_parolasi`
   intrinsic'i, yeni kalıp kelimeleri `uygulama`, `parolası`) `{görünen, önek,
   özet}` Metin sözlüğü verir. Biçim soru 3'ün kabulüdür: `zua1-` + 8 Base32
   önek + 32 Base32 gizli, dörtlü gruplar; özet `zuah1:<kimlik>:<base64url
   HMAC-SHA256>`, HMAC girdisi alan ayraçlı (`zee-uygulama-parolasi-1\0` ‖
   önek ‖ gizli). Argon2id kullanılmaz, giriş kovasına dokunulmaz.
2. **Üretim yeri (T063).** Yalnız `oturum gerekli`/`"<rol>" yetkisi gerekli`
   önsözlü POST/PUT/PATCH/DELETE rotasında (gövde ya da geçişli işlem) ya da
   rota dışı kodda. Herkese açık, makine ucu, önbelleklenebilir ve GET/HEAD
   rotası derleme tanısıdır: parola CSRF korumalı bir kullanıcı eylemiyle
   doğar, önbelleğe ya da çapraz kökenli okumaya düşmez.
3. **Anahtarlar (soru 2'nin uygulaması).** RFC-0047'nin tek alanlı adayı
   (`uygulama_parolası_anahtar_değişkeni`) döndürmeyi taşıyamadığı için
   `uygulama_parolası_anahtarları "<kimlik> <credential>", …` listesi seçildi:
   ilk öğe etkin, sonrakiler yalnız doğrulama; en çok 8; kimlik özete
   gömülüdür, böylece döndürmede kayıt kendi anahtarını bulur ve eski anahtar
   ona bağlı kayıt kaldıkça operatörce tutulur (dil süreyle silmez). Credential
   öteki gizli değerlerle (şifreleme anahtarı, DKIM, TLS, veritabanı, HTTP,
   veri kökü) aynı adı taşıyamaz (P015) — anahtar ayrımı.
4. **IO izi.** Görünen değer üretim yanıtının gövdesine de gireceğinden kayıt
   açıkken üretim reddedilir (C028; iç adaptör çağrılmaz, olay yok); oynatıcı
   aynı reddi verir. Hermetik adaptör zee-io-1 tohumuyla deterministiktir.
   Bu, RFC-0047 §8 "görünen ize hiç girmez" kabul ölçütünün en dar
   karşılığıdır.
5. **Doğrulama kitaplığı.** `guvenlik::uygulama_parolasi`: biçim kapısı (hesap
   parolası yapısal olarak geçemez, soru 3 b), sabit zamanlı özet
   karşılaştırması ve bilinmeyen kimlik/kayıt için sahte karşılaştırma, katı
   RFC 7617 Basic çözümü, ASCII kullanıcı adı katlaması (soru 1). HMAC-SHA256
   bağımlılıksız SHA-256 üstündedir (yeni crate yok, wasm dahil her hedefte).
   Bu dilimde hiçbir rota Basic kabul etmez; kitaplık dilim C'de DAV kimlik
   kararına bağlanır.
6. **Kapsam dışı (dilim C'ye kaydı).** `dav_kimlik_çözücüsü` bildirimi ve
   salt okuma kanıtı çözücüyü kullanan sınıfla birlikte açılır; tek başına
   bildirilen ama çağrılmayan bir proje alanı açmamak için B'den C'ye alındı.

Kanıt: `compiler/tests/urun_e_yetkinlik_testi.rs::uygulama_parolasi_*`,
`compiler/tests/proje_testi.rs::uygulama_parolasi_anahtarlari_bildirimi_ve_p015`,
`compiler/tests/systemd_credential_testi.rs::systemd_credential_uygulama_parolasi_etkin_anahtarla_uretilir`,
`compiler/src/guvenlik/uygulama_parolasi.rs` birim testleri (RFC 4231
vektörleri dahil).

## Ek — dilim C: `dav istemcisi gerekli` sınıfı (28 Kasım 2026, GB-187)

1. **Yüzey.** Politika cümlesi `dav istemcisi gerekli` (AST:
   `RotaPolitikasi { erisim: HerkeseAcik, kip: Dav }`; yeni kalıp kelimesi
   `dav`), yöntem kaydı `PROPFIND`, `REPORT` (salt okuma sınıfı). Statik
   kapılar T064 (PROPFIND/REPORT DAV dışında; `rfc0040-a-*` S036 → T064),
   T065 (DAV + POST/PATCH ya da sabit olmayan yol), T066 (DAV rotası/işlemleri
   çerez/oturum/CSRF yüzeyine dokunuyor), T063 (DAV'da parola üretimi).
   Erişim `herkese açık` sayılır çünkü karar çerez oturumundan değil, rota
   öncesi kimlik kararından gelir; politika `kriptografi` ister.
2. **Kimlik çözücü (soru 1).** Proje alanı `dav_kimlik_çözücüsü`; çözücü iki
   parametreli, değer döndüren, durum/web/geri alınamaz etkisiz `işlem`
   (proje kipinde T067). Her istekte önbelleksiz çağrılır.
3. **Karar (RFC-0047 §6).** Adaptör Basic'i çözer, gizli kısmı tutar;
   yorumlayıcı kova bakışı → çözücü (biçimli parolada) → sabit zamanlı
   özet karşılaştırması + ASCII katlanmış kullanıcı eşitliği → başarısız
   kovalar ya da başarılı kova → salt okuma (soru 11) → istek yerel `oturum`.
   Kovalar (soru 12, aday eşikler) `RateLimitTuru::Dav*` ve artırmadan bakan
   `rate_limit_bak`; başarı başarısız sayacı sıfırlamaz.
4. **CSRF/Origin (soru 10).** CSRF ve Origin zorunluluğu yok; `Sec-Fetch-Site:
   cross-site` ve yabancı `Origin` 403; `Cookie` okunmaz, `Set-Cookie` yok.
5. **Ayrı origin (soru 8).** `dav_kökeni` üretim profilinde zorunlu ve web
   origin'inden farklı; proxy Host/Forwarded denetimi DAV isteğinde
   `dav_kökeni`ne, öteki isteklerde web origin'ine göre yapılır. Deneysel
   profil zaten yalnız loopback'tir (soru 9).
6. **OPTIONS (soru 4).** DAV yolu altında kimliksiz 200 + `Allow` + `DAV:
   calendar-access, addressbook`, CORS yok; başka yolda 501.
7. **IO izi.** `dav_kimligi`, `dav_ozet_dogrula`, `dav_kovasi` olayları;
   gizli kısım ve görünen parola ize girmez; 401/429/OPTIONS
   `durum_yaniti_gonder` biçiminde.
8. **Bu dilimde açık.** DAV gövdesi okunur ve atılır (64 KiB); ham gövde,
   başlık okuma ve ETag dilim D'dir. 401 süre dağılımı ve istemci kabulü
   ürünle ölçülecek.

Kanıt: `compiler/tests/web_testi.rs::dav_*`,
`compiler/tests/dosya_yaniti_testi.rs::dav_istemci_rotasi_gercek_worker_ayri_origin_basic_ve_csrf_muafiyeti`,
`compiler/tests/proje_testi.rs::dav_bildirimi_p015_ve_cozucu_t067`.

## Ek — dilim D: başlık okuma, ham DAV gövdesi ve ETag (28 Kasım 2026, GB-188)

1. **Başlık okuma (soru 5, 6).** `isteğin "<ad>" başlığı`; ad tırnaklı
   literal ve kapalı liste (`Depth` → Metin, `If-Match`/`If-None-Match` →
   `Liste<Metin>`, `*` tek öğeli ayrı değer). Liste dışı/hesaplanmış ad S051,
   güven başlıkları S052 (`rfc0040-e-basic-kimlik` S015 → S052, kalıcı ret);
   yalnız DAV rotasının kendi gövdesinde (T068; `rfc0040-b-depth-basligi`
   S015 → T068). Adaptör üç başlığın ham değerini verir; yorumlayıcı kimlik
   kararından sonra, rota öncesi doğrular (400; PROPFIND sonsuz/eksik Depth
   403 `propfind-finite-depth`; REPORT eksik Depth `0`).
2. **Ham gövde (soru 7, kısmi kabul).** DAV yolunun gövdesi form değildir;
   `isteğin ham gövdesi` DAV rotasında okunur (T062 orada kalkar), CRLF
   dönüştürülmez. Sabit yöntem tavanları PUT 1 MiB, REPORT/PROPFIND 256 KiB;
   GET/HEAD/DELETE gövdesiz; PUT/REPORT gövde zorunlu. İçerik türü (PUT
   `text/calendar`/`text/vcard`, XML yöntemleri `application/xml`/`text/xml`,
   `charset` yalnız `utf-8`) gerçek adaptörde 415.
3. **ETag.** `yanıta "ETag" başlığını <ifade> olarak ekle` (AST
   `Cumle::YanitEtagi`); başka başlık adı S053; yalnız DAV rotasında bir kez
   (T069; `rfc0040-c-etag-basligi` S015 → T069); değer güçlü entity-tag,
   geçersiz değer C022 → 500 ve başlığa girmez; yalnız 2xx yanıtta.
4. **IO izi.** `dav_istek_basliklari` (ham çiftler) ve `etag_yaz` olayları.
5. **Korpus.** Dokuz vakanın hepsi beklenen sonucunda: `rfc0040-c-207-xml`
   başarılı; `a-*` T064, `b` T068, `c-etag` T069, `e` S052, `f-*` T050/T060/
   T062 bilinçli ret. Ürün kaynağından alınacak çözüm vakaları ZeeMail'den
   istendi; ürün provenance'ıyla birebir gelince korpusa eklenir (merkez
   uydurmaz) — bu ADR'nin açık maddesidir.
6. **Kabul ölçütleri (RFC-0047).** Dil tarafı (madde 2 ve 5'in derleme/
   çalışma zamanı kısmı, madde 6) kanıtlı. Thunderbird/DAVx5/iOS ile hesap
   kurma, iki cihaz eşitlemesi, 401 süre dağılımı ve ilk eşitleme kova
   ölçümü ürünle yapılacak saha kabulüdür.

## Değerlendirilen seçenekler

- **405/501'i yalnız DAV sınıfında düzeltmek** — reddedildi; tanımsız
  yönteme verilen yanıtın tutarlılığı güvenlik ve istemci davranışı için
  her rotada gereklidir (ürün, soru 13); davranış farkı yalnız başlık ve
  durum kodudur.
- **`Allow`'u yeni IO izi olayıyla kaydetmek** — reddedildi; değer
  deterministik türer, yeni olay eski izleri bozardı.
- **Türlü yanıtta serbest durum (100..599)** — reddedildi; 3xx gövdeli türlü
  yanıt yönlendirme yüzeyini, 5xx program kaynaklı sunucu hatası taklidini
  açardı. Kapalı küme RFC-0047'nin listesidir.
- **204'ü türlü kümeye almak** — reddedildi; 204 gövde ve tür taşıyamaz,
  durumlu düz yanıtla yazılır.
- **RFC-0047'nin sırası (yöntem → başlık → kimlik → politika)** — sınıf
  kimliksiz açılamayacağı ve başlık/ETag sınıfa bağlı olduğu için yukarıdaki
  sıraya çevrildi.

## Sonuçlar

- `rfc0040-c-207-xml` reproducer'ı dosya değişmeden `basarili`dır
  (`dogfood/korpus-v1.tsv`). Öteki sekiz vaka dilim C/D'ye kadar bugünkü
  tanılarıyla kalır; PROPFIND/REPORT gerçek istekte 501'dir.
- Yeni kalıp kelimesi, tanı kodu, AST değer türü, bağımlılık ve `dil::api::v1`
  yüzeyi yok. K-167 satırları: `bicim/http-yontem-kaydi`,
  `bicim/turlu-yanit-durumlari`, `bicim/icerik-turu.vcf`,
  `bicim/web-metin-yaniti-turleri` (güncellendi), `api/io-izi.turlu_durumlu_yanit_gonder`.
- Görünür davranış değişikliği: önceden 405 ya da 404 alan kayıt dışı yöntem
  artık 501; 405 `Allow` taşır; 200/201/204/207 vb. durum satırı `Error`
  yerine kanonik açıklamayla gider; 204 gövdesizdir. Göç gerekmez.
- Açık: OPTIONS'ın Zee rotası olarak yazılması, PROPPATCH/MKCOL/MOVE/COPY,
  `Prefer`/`Brief`/`Location` (RFC-0047 kapsam tablosu); D (XML ağacı) ve G
  (well-known) ayrı alt RFC.

## Kanıt

- `compiler/tests/dosya_yaniti_testi.rs::dav_a_turlu_durumlu_yanit_405_allow_501_ve_204_gercek_surecte`
- `compiler/tests/dosya_yaniti_testi.rs::dav_a_turlu_durum_kapali_kumede_s037`
- `compiler/tests/dosya_yaniti_testi.rs::dav_a_hermetik_adaptor_ve_io_izi_ayni_kural`
- `compiler/tests/dosya_yaniti_testi.rs::dav_a_yontem_reddi_proxy_kapisindan_sonra_cerezsiz`
- `compiler/tests/web_testi.rs::yanlis_yontem_405_olmayan_yol_404_doner`
- `compiler/src/icerik_turu.rs::web_metin_turu_yalniz_metin_satirlarini_acar`
- `dogfood/korpus-v1.tsv` `rfc0040-c-207-xml` (`basarili`)
