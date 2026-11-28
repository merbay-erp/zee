# ADR-182 — DAV istemci rotası: RFC-0047 uygulaması, dilim sırası ve açık soruların kapanışı

- **Durum:** kabul — dilim A uygulandı (28 Kasım 2026); B, C, D aynı ADR'nin
  ekleriyle açılır
- **Tarih:** 28 Kasım 2026
- **İlgili kayıt:** K-364, [RFC-0047](../rfcs/0047-dav-istemci-rotasi-ilk-dilim.md),
  [RFC-0040](../rfcs/0040-webdav-caldav-carddav-adaptoru.md), ADR-059 (core
  freeze), ADR-064 (dil yüzeyi), ADR-067 (dogfood korpusu), ADR-163 K-406 eki
  (türlü metin yanıtı), ADR-178 (rota sonu), ADR-179 (makine uç noktası),
  ADR-036 (kanonik proxy origin), spec/11 §4, spec/12 §RFC-0047, GB-185

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
| B | Uygulama parolası ilkeli: `yeni uygulama parolası`, HMAC anahtar credential'ı ve anahtar kimliği, `dav_kimlik_çözücüsü` bildirimi ve salt okuma kanıtı | açık (ADR-182 eki) |
| C | `dav istemcisi gerekli` sınıfı: PROPFIND/REPORT kaydı, yöntem-politika tanıları, statik kapı, CSRF/Origin muafiyeti, `Sec-Fetch-Site` reddi, çerezsiz Basic doğrulaması, ayrı oran kovaları, salt okuma kapsamı, ayrı DAV origin'i, otomatik OPTIONS | açık (ADR-182 eki) |
| D | `Depth`/`If-Match`/`If-None-Match` okuma, DAV ham gövdesi (tür ve tavan kuralları), `propfind-finite-depth` 403, `ETag` yazımı, korpus geçişi ve çözüm vakaları, RFC-0047 durumu | açık (ADR-182 eki) |

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
