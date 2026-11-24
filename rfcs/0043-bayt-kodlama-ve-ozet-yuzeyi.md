# RFC-0043 — Bayt, kodlama ve özet yüzeyi

- **Durum:** **kabul — Dilim 1 uygulandı** (K-389, [ADR-167](../adr/167-dosya-kodlanmis-icerik-ve-bellek-ozeti.md),
  24 Kasım 2026: aralıklı metin/base64 okuma, base64/QP çözerek yazma ve
  akışlı ekleme, bellekte SHA-256, C032). **Dilim 2 taslak** (ZM-F023: UTF-8
  bayt sayısı, metin↔base64/onaltılık, kod noktası); core freeze altında
  ayrı `dogfood-change` K-işi ve reproducer ister. Aşağıdaki "Dilim 1 kararı"
  bölümü bu taslağın Dilim 1 açık sorularını kapatır ve önceki fikir
  taslağının üstündedir.
- **Tarih:** 24 Kasım 2026
- **İş:** K-383 (taslak), K-389 (Dilim 1 uygulaması)
- **İlgili belgeler:** [spec/04](../spec/04-turler.md),
  [spec/06](../spec/06-hata-modeli.md),
  [spec/15](../spec/15-yapilandirilmis-hata-degeri.md),
  [spec/21](../spec/21-deterministik-io-izi.md),
  [spec/23](../spec/23-yetkinlik-ve-outbound-guvenligi.md),
  [spec/24](../spec/24-kaynak-guvenlik-profili.md),
  [spec/26](../spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md),
  [spec/27](../spec/27-uyumluluk-ve-surumleme.md),
  [ADR-011](../adr/011-intrinsic-yetkinlik-siniri.md),
  [RFC-0007](0007-temel-tur-sistemi.md),
  [RFC-0008](0008-secenek-ve-sonuc.md),
  [RFC-0036](0036-dosya-imzasi-okuma.md),
  [RFC-0039](0039-ag-sunucusu-yetkinligi.md) (katman D),
  [RFC-0041](0041-ikili-yanit-ve-ek-akisi.md),
  [RFC-0044](0044-dns-posta-kimlik-dogrulama-ve-teslim-guvenligi.md)
- **Freeze sınıfı adayı:** iki dilimli `dogfood-change` (`urun-h`):
  Dilim 1 ZM-F034 + ZM-F024, Dilim 2 ZM-F023.

## Özet

ZeeMail (`urun-h`) SMTP/IMAP motorunu Zee ile yazarken üç küçük ama temel
eksiğe çarptı: (1) bir `Metin`in **UTF-8 bayt** karşılığına ulaşamıyor —
bayt sayısı (SMTP `SIZE`, RFC 5322 998 oktet satır sınırı), base64 kodlama
ve çözme (AUTH PLAIN/LOGIN, MIME, DKIM `b=`/`bh=`), katı UTF-8 çözme
(ZM-F023); (2) tek harfle **kod noktası** arasında geçemiyor (IMAP değiştirilmiş
UTF-7 posta kutusu adları, RFC 2047 encoded-word, ASCII denetimleri);
(3) **bellekteki** metnin SHA-256 özetini alamıyor — bugün yalnız dosya
özeti var (ZM-F024; DKIM gövde özeti ve küçük kanonik başlık bloğu için
şart). Bu RFC yeni bir değer türü açmadan, **saf (etkisiz) ve toplam ya da
`Sonuç` döndüren** dar bir kalıp ailesi önerir: bayt verisi Zee'de her
zaman ya geçerli UTF-8 `Metin` ya da adı konmuş bir **kodlanmış bayt
metni** (küçük harf onaltılık veya RFC 4648 base64) olarak görünür. Büyük
ikili veri RFC-0039 katman D'nin dosya tutamağı ilkesinde kalır.

24 Kasım 2026'da ürün dördüncü bir somut engel bildirdi (ZM-F034): A2c'nin
`nokta bloğunu dosyaya al` / `n baytı dosyaya al` çıktıları Zee'de
ayrıştırılamıyor, MIME parçasının base64/quoted-printable içeriği bayt
olarak dosyaya yazılamıyor. Bu yüzden **önerilen ilk uygulama dilimi**
(Dilim 1) yalnız şunları kapsar: dosyayı sınırlı ve katı UTF-8 `Metin`
olarak okuma (çoğu zaten var; eksikleri kapatılır), base64/QP içeriğini
**çözerek dosyaya bayt-kesin yazma** ve bellekte SHA-256. Bayt sayısı,
metin↔base64/onaltılık, kod noktası Dilim 2'dir.

## Dilim 1 kararı (K-389, 24 Kasım 2026)

Normatif metin [spec/26 §Kodlanmış içerik](../spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md),
gerekçe [ADR-167](../adr/167-dosya-kodlanmis-icerik-ve-bellek-ozeti.md).
Uygulanan exact yüzey:

```text
bölüm yol dosyasının konum baytından n baytını metin olarak okumayı dene olsun   # Sonuç<Metin>
ham yol dosyasının konum baytından n baytını base64 okumayı dene olsun          # Sonuç<Metin>
tutamak parça base64 çözerek yol dosyasına yazmayı dene olsun                   # Sonuç<Sözlük>
tutamak parça quoted_printable çözerek yol dosyasına yazmayı dene olsun         # Sonuç<Sözlük>
tutamak parça base64 çözerek yol dosyasına eklemeyi dene olsun                  # akışlı ekleme
tutamak parça quoted_printable çözerek yol dosyasına eklemeyi dene olsun
özet metnin sha256 özeti olsun                                                   # Metin
```

Açık soruların Dilim 1 kapanışı:

| # | Karar |
|---|---|
| 1 | Dilim 1 kelimeleri yukarıdaki gibi; `quoted-printable` sözcükleyicide tek kelime olamadığından `quoted_printable`. `hâli`/`karşılığı` seçimi Dilim 2'de (açık). |
| 2 | Aralıklı okuma ve çağrı başına çözülen çıktı 1 MiB (1 048 576) sabit; bellekte özet ayrı tavan almaz, spec/24 metin/heap bütçesine bağlıdır. Hedef dosya toplamı spec/26 16 MiB yükleme/özet tavanı. |
| 3, 4 | Dilim 2 (kod noktası, base64url); açık. |
| 5 | Dilim 1 yeni doğrudan crate eklemez; tek iç katı çözücü (`compiler/src/kodlama.rs`). Üç eski iç yardımcının birleştirilmesi Dilim 2'ye bırakıldı (açık). |
| 6 | Yeni C032 (kodlama: `tür` + `konum`); dosya/aralık/yol/no-clobber/tavan/eylem arızaları C013. |
| 7 | QP bozuk `=` dizisi **katı ret** (`kaçış` + `konum`). |
| 8 | Ekleme hedefi her proje dosyası (mevcut `ekle` ile aynı sınır); spec/08 atomik ekleme modeli; çağrılar arası yarım dörtlü durumu **tutulmaz**. `eklemeyi dene` eylem içinde C013; `yazmayı dene` eylem savepoint'ine kayıtlıdır. |
| 9 | Aralık tavanı 1 MiB; mevcut `dosyasını okumayı dene` izi uyumluluk için değişmez. Yeni olaylar içeriksizdir; başarılı aralıklı okuma replay'de fail-closed. |
| 10 | ZM-F035 K-388'de (protokol birimi sınırında bütçe yenileme); Dilim 1 ayrı kabul edildi, çok büyük iletide birikim o kararla okunur. |

Kabul ölçütü 5'in (Dilim 1) kanıtı `compiler/tests/kodlanmis_dosya_testi.rs`:
2,5 MB ikili ek MIME iletisinden 1 MiB aralıklarla okunup akışlı çözülür,
SHA-256 özgünle eşleşir, ek RFC-0041 dosya yanıtıyla indirilir; 8bit gövde
metin kipinde `utf8`, base64 kipinde başarı; bozuk QP/base64, 4'e
bölünmeyen girdi, var olan hedef, tavan üstü ekleme hedefe dokunmaz; izde
içerik yok. "25 MB / tek bağlantıda üç ileti" ölçümü 16 MiB hedef tavanı
ve K-388 ile birlikte açık kalır. Ölçüt 4 (NIST vektörleri, dosya/metin
özet eşitliği) karşılandı; 1–3, 6–8'in Dilim 2 kısmı açık.

## Motivasyon

### Ürün sürtünmesi

ZeeMail'in 23 Kasım 2026 sürtünme kaydı (depo dışı ürün deposu; merkez
kaydı) iki maddeyi bu RFC'ye bağlar:

- **ZM-F023 bayt ↔ metin:** metnin UTF-8 baytları, katı UTF-8 çözme,
  base64 kodla/çöz, kod noktası ↔ harf. Ürün bugün base64'ü Zee'de
  tablo-arama döngüsüyle yazmayı denedi; ama `Metin` üzerinde bayt düzeyine
  inilemediği için ASCII dışı her girdide yanlış sonuç veriyor ve
  `uzunluğu` harf (Unicode skaler değer) saydığı için SMTP oktet sınırları
  doğru uygulanamıyor.
- **ZM-F024 bellekte SHA-256:** DKIM (RFC 6376 §3.7) `bh=` gövde özetini ve
  imzalanan başlık bloğunun özetini ister. Spec/26 `yol dosyasının sha256
  özetini almayı dene` yalnız diskteki dosyayı özetler; küçük kanonik metni
  özetlemek için geçici dosyaya yazıp okumak hem `dosya-yazma` yetkinliği
  hem IO izi gürültüsü getirir.
- **ZM-F034 dosya ↔ kodlanmış içerik (24 Kasım 2026):** A2c bloğu dosyaya
  alındıktan sonra ürün (1) dosyayı `Metin` olarak geri okuyup MIME
  sınırlarını ayrıştırmak, (2) parçanın base64/QP gövdesini çözüp **bayt
  olarak** ek dosyasına yazmak istiyor. Ürünün bildirdiği etki: SMTP/IMAP
  ile gelen ekler webmail'e çıkarılamıyor, istemciden ekli gönderim `554`
  ile düşüyor, IMAP `APPEND` 64 KB'ta takılıyor. Ürün ayrıca ham iletinin
  `.eml` olarak indirilebilmesi için dosya yanıtı tür kaydına
  `message/rfc822` istiyor (bkz. "RFC-0041 ile ilişki").

### Bugünkü durum (bd8d44f üzerinde doğrulandı)

| Gerekli | Bugün | Kaynak |
|---|---|---|
| Bayt değer türü | **Yok.** Tür envanteri `TamSayı · Ondalık · Metin · Mantıksal · Liste · Sözlük · Seçenek · Sonuç · Hata · Yapı · Tarih …`; `Metin` çalışma zamanında Rust `String`'dir (her zaman geçerli UTF-8) | spec/04 §Tür envanteri; `compiler/src/yorumlayici.rs` `enum Deger` |
| Metin uzunluğu | **Var, harf sayısı.** `metnin uzunluğu` `chars().count()` (Unicode skaler değer); UTF-8 bayt sayısı yok | `compiler/src/yorumlayici/ifade.rs` `Ozellik::Uzunluk` |
| Harflere bölme | **Var.** `metnin harfleri` → `Liste<Metin>` (skaler değer başına bir öğe, koleksiyon sınırıyla); kod noktası sayısal değeri yok | `ifade.rs` `Ozellik::Harfler` |
| base64 | **Kısmi, yalnız iç yardımcı.** Üç ayrı özel kodlayıcı (SMTP AUTH/MIME `eposta.rs::base64`, `main.rs::base64_kodla`, gizli değer `guvenlik/gizli.rs::base64_yaz/oku`); Zee yüzeyi yok | `compiler/src/eposta.rs:124`, `compiler/src/main.rs:2846`, `compiler/src/guvenlik/gizli.rs:76` |
| SHA-256 | **Kısmi.** Dosya özeti var (`dosya-okuma`, ≤ 16 MiB akış, `Metin` 64 hex); yükleme/blok tutamağında `sha256` alanı var; bellekteki metin için yok. İç yardımcı `guvenlik::sha256_hex` mevcut | spec/26 §Beklenen dosya işlemleri; spec/12 §9 A2c; `compiler/src/guvenlik.rs:78` |
| Onaltılık | **Kısmi.** Dosyanın ilk 1–64 baytı küçük harf onaltılık (RFC-0036); metin için yok | spec/26 K-333 |
| Dosyayı `Metin` olarak okuma | **Var, kısmi.** `"yol" dosyasını okumayı dene` → `Sonuç<Metin>`; `dosya-okuma`, proje dosya sınırı, 16 MiB tavan (`dosya_okuma_bayti`), katı UTF-8 (`String::from_utf8`, onarım yok), baytlar olduğu gibi (CRLF korunur); `"yol" dosyasının satırları` ayrıca var. **Eksik:** geçerli UTF-8 olmayan içerik (8bit gövde, ikili ek) tümüyle başarısız olur; dosyanın bir bölümünü (başlık bölümü, MIME parçası aralığı) okuma yok; okunan içeriğin **tamamı** IO izine `dosya_oku` sonucu olarak yazılır — A2c'nin "blok baytı IO izi içeriği olmaz" sözüyle çelişir ve 16 MiB'lık izler üretir | `compiler/src/agac.rs` `DosyaOkumayiDene`; `compiler/src/kaynak_sinirlari/okuma.rs::sinirli_metin_oku`; `compiler/src/yorumlayici/io_izi.rs` `dosya_oku`; spec/12 §9 A2c |
| Dosyaya bayt-kesin yazma | **Yok.** `"yol" dosyasına değer yaz/ekle` yalnız değerin metin gösterimini ve ardından **tek `\n`** yazar (spec/08 §Satır yazma); kodlanmış içerikten bayt yazma ya da satır sonusuz yazma yok | spec/08; `compiler/src/kalici_dosya.rs::atomik_satir_yaz` |
| Kripto/kodlama crate'leri | `sha2 0.11.0` doğrudan; `base64 0.23.1` geçişli (ureq, bcrypt) ve `0.22.1` geçişli (postgres-protocol); `ring 0.17.14` geçişli (rustls) | `compiler/Cargo.toml`, `compiler/Cargo.lock` |

Sonuç: ihtiyaç duyulan algoritmaların hepsi derleyici ağacında zaten var;
eksik olan **Zee yüzeyi ve tür modeli kararıdır**.

## Amaç

1. `Metin` ile UTF-8 bayt dizisi arasında kayıpsız, katı ve açık kodlamalı
   geçiş.
2. RFC 4648 base64 (standart alfabe, dolgu zorunlu) ve küçük harf
   onaltılık kodlama/çözme; base64url ayrı adla.
3. Tek harf ↔ Unicode kod noktası.
4. Bellekteki metin ve kodlanmış bayt metni üzerinde SHA-256 (onaltılık ya
   da base64 çıktı).
5. Bunların hepsi saf, deterministik, yetkinliksiz, IO izine girmeyen ve
   sınırlı kaynaklı işlemler olsun.
6. (ZM-F034) Dosya tutamağındaki içeriği sınırlı `Metin` olarak okumak ve
   base64/QP kodlu `Metin`i çözerek dosyaya **bayt-kesin** yazmak; bunlar
   etkili işlemlerdir, `dosya-okuma`/`dosya-yazma` ister ve IO izine içerik
   değil yol + bayt + SHA-256 olarak girer.

## Kapsam dışı

- Yeni `Bayt`/`Baytlar` değer türü (bkz. "Tür modeli"; ölçülmüş sürtünme
  çıkarsa ayrı RFC).
- HMAC, PBKDF2, SCRAM-SHA-256, MD5, SHA-1, SHA-512. SCRAM ihtiyacı çıkarsa
  ayrı madde; MD5/SHA-1 güvenlik amaçlı hiç açılmaz.
- Quoted-printable **kodlaması**, RFC 2047 encoded-word kodlaması/çözümü ve
  MIME yapı ayrıştırması: MIME alt RFC'si (RFC-0039 katman D/MIME). Bu RFC
  yalnız QP'yi **çözerek dosyaya yazmayı** (Dilim 1) ve yapı taşlarını
  (kod noktası, base64) verir; MIME sınırlarını Zee kodu bulur.
- Akışlı (parça parça beslenen) özet bağlamı ve büyük gövde kanonikleştirmesi:
  DKIM gövde özeti dosya üzerinde RFC-0044 B2/B3 host işlemidir.
- Sıkıştırma, şifreleme, imza, rastgele bayt üretimi.
- Karakter kümesi dönüşümü (ISO-8859-9, Windows-1254 → UTF-8): MIME alt
  RFC'si; burada yalnız UTF-8 vardır.
- Unicode normalizasyonu ve büyük/küçük harf katlama (mevcut yüzeyler).

## Tasarım ilkeleri

1. **Bayt Zee'de çıplak durmaz.** Zee değeri ya geçerli UTF-8 `Metin`dir ya
   da kodlaması adıyla belli `Metin`dir (`onaltılık`, `base64`, `base64url`).
   "Herhangi bir baytı tutan `Metin`" yoktur; Rust `String` değişmezi
   korunur, bozuk UTF-8 sessizce `U+FFFD`'ye dönmez.
2. **Kodlama adı her çağrıda yazılıdır.** Örtük varsayılan kodlama yok;
   okuyan kişi `base64` ile `onaltılık` ayrımını cümleden görür.
3. **Çözme her zaman `Sonuç`tur, kodlama toplamdır.** Kodlama (metin →
   kodlanmış) her girdi için tanımlıdır ve doğrudan değer döner; çözme
   (kodlanmış → metin) geçersiz alfabe/dolgu/UTF-8'de yakalanabilir
   başarısız `Sonuç` verir (RFC-0008).
4. **Saflık (Dilim 2 ve bellekte özet).** Metin işlemleri etkisizdir: yetkinlik istemez, IO izine olay yazmaz,
   replay'de aynı girdiye aynı çıktıyı verir. `json metni`, `html güvenli`
   gibi mevcut dönüşümlerle aynı sınıftadır. Dosya işlemleri (§5) etkilidir
   ve mevcut dosya yetkinlik/sınır kurallarına tabidir.
5. **Algoritma yazılmaz.** base64 ve SHA-256 ağaçtaki doğrulanmış
   crate'lerden gelir; üç iç base64 yardımcısı tek uygulamada toplanır
   (bakım borcu da kapanır).
6. **Bütçe önce.** Girdi boyutu ayırmadan önce denetlenir; aşım yakalanabilir
   `Sonuç`/çalışma hatası olur, sessiz kırpma yoktur.

## Tür modeli — `Bayt` türü gerekiyor mu?

| Seçenek | Ne verir | Freeze maliyeti | Karar |
|---|---|---|---|
| **A. Yeni `Baytlar` değer türü** | Gerçek ikili değer; eşitlik, dilimleme, birleştirme, dosya/ağ ile doğrudan akış | Çok yüksek: spec/04 tür envanteri, RFC-0007 temel tür sistemi, tür denetleyici birleşim/daraltma kuralları, `Deger` enum'u, eşitlik/sıralama/gezme (spec/17), JSON/CSV/HTML dönüşümleri, IO izi şeması (spec/21) ve replay, LSP hover/tamamlama, morfoloji (tür adı çekimleri), golden ve uyumluluk yüzeyi (K-167). V1 syntax/type freeze penceresinde "grammar en son çaredir" ilkesine (RFC-0039 §ilke 4) aykırı | **Ertelenir.** ZM-F023'teki ihtiyaçların hiçbiri keyfî ikili değeri Zee'de tutmayı gerektirmez |
| **B. `Liste<TamSayı>` (0–255)** | Mevcut türle bayt dizisi | Düşük tür maliyeti, ama bellek: her öğe bir `Deger` (≥ 32 bayt) → 1 MiB gövde ≥ 32 MiB; koleksiyon sınırı (spec/24) hızla dolar; yorumlanan döngüyle base64/özet pratik değil (ADR-003). 256 dışı değer her işlemde ayrı tanı ister | **Ret** (yalnız küçük denetimlerde dolaylı kullanım: kod noktası) |
| **C. Kodlanmış bayt metni** (`onaltılık`/`base64` `Metin`) | Taşıma temsili; protokoller zaten bu biçimleri ister (DKIM `b=`, `bh=`, `p=`; AUTH PLAIN; MIME) | Düşük: yeni tür yok; yalnız yeni kalıp kelimeleri ve bir tanı/neden kümesi (K-167 kaydı) | **Seçilen** |
| **D. Dosya tutamağı** (yol + SHA-256 + bayt) | Büyük ikili veri (ek, ileti gövdesi) diskte | Zaten var (spec/26, RFC-0041, RFC-0039 A2c) | **Büyük veri için korunur** |

**Ölçülmüş girdi (ürün, 24 Kasım 2026):** güvenli profilin 64 MiB heap
sınırı MB ölçeğinde metin birleştirmeyi C024 ile kesiyor — 1 MB'lık gövdeyi
satır satır `Metin` olarak kurmak ~67 MB çalışma değeri üretiyor. `DATA`/
`APPEND` gövdesini bellekte biriktirmek bu yüzden yapısal olarak mümkün
değildir; büyük veri için D (dosya tutamağı + aralıklı okuma) zorunludur ve
§5 aralık tavanı (≤ 1 MiB/çağrı) tek çağrı için bu bütçeyle uyumludur.

**Ürün ölçümü (24 Kasım 2026, bd8d44f, macOS arm64, güvenli profil):**
gövdeyi bellekte biriktiren eski SMTP alıcısı 1 MB, 5 MB ve 12 MB `DATA`'da
C024 ile kesildi (çalışma değeri ~67 MB; bağlantı yanıtsız kapanır, süreç
ayakta); eşik 1 MB'ın altında. Kök neden **bütçe modelidir (ZM-F035):**
spec/24 64 MiB zarfı "iade edilmeyen tahsis zarfı" olarak TANIMLAR
(`compiler/src/yorumlayici/kaynak.rs::heap_butcesini_tuket` yalnız düşer)
ve web'de her istek taze bütçe alır; ama A2 bağlantı görevi bütün oturum
boyunca **tek** bütçe taşır. Ölçümde 2 000 000 × ~48 baytlık atılan geçici
metin ve 4 800 küçük QP kodlama çağrısı da C024 verdi. Ürünün geçici çözümü
(yalnız ilk 256 KB'ı işlemek, kalanı saymak) ile 12 MB 1,4 sn'de kabul,
26 MB `552`, C024 yok. Ayrı bir sınır: tek SQL parametresi 64 KiB (64 KB
üstü gövde `451`; spec/25, bu RFC'nin dışında).

**Sonuç — Dilim 1'in önkoşulu:** 25 MB'lık dosyayı 1 MiB'lık aralıklarla
okuyup ayrıştırmak da bugünkü modelde birikimle C024'e yürür. Bu yüzden
Dilim 1 ZM-F035 kararıyla birlikte (ya da ondan sonra) uygulanır. Seçenekler:
(a) bağlantı görevinde bütçenin protokol birimi sınırında (blok/komut,
örn. her `bağlantıdan … almayı dene` dönüşünde) yenilenmesi — web
isteğinin taze bütçesinin bağlantı karşılığı, en küçük değişiklik;
(b) canlı değer ölçümü (atılan değerin zarfa iadesi) — spec/24
"iade edilmeyen" sözünü değiştirir, ölçüm maliyeti ve muhafazakârlık
kanıtı ister; (c) MIME yapısını host intrinsic'ine taşıyıp Zee'nin daha az
metin görmesi — RFC-0039 MIME alt RFC'si, tek başına yetmez. Öneri (a);
spec/24 normatif değişikliği olduğu için ayrı correctness K-işi (merkezde
ZM-F035) ve aynı committe spec/24 + K-173 soak koşusu.

**K-388 sonucu (24 Kasım 2026):** kaynak incelemesi "bağlantı görevi tek
bütçe taşır" ifadesini düzeltti: K-381'den beri **başarılı** satır/blok
okuması bütçeyi yeniliyordu. Eksik olan başarısız dönüşlerdi; K-388 ile
girdi tüketen ya da süresini dolduran her bağlantı okuma dönüşü birim
sınırıdır (spec/12 §9, spec/24, ADR-165 eki). Ölçülen 1 MB+ `DATA`
düşmesi ise **tek birim** içindedir (gövdeyi Zee metni olarak kurmak) ve
K-388'den sonra da C024'tür; seçenek (a) bunu çözmez. Dilim 1 için sonuç
aşağıdaki açık soru 10'dadır.

Karar (taslak): **C + D**. Küçük bayt verisi kodlanmış `Metin`, büyük bayt
verisi dosya tutamağıdır. `Baytlar` türü ancak C+D ile yazılmış gerçek ürün
kodunda ölçülmüş sürtünme (ör. kodlanmış metin üzerinde dilimleme ihtiyacı
ya da ölçülmüş bellek/zaman maliyeti) çıkarsa ayrı RFC'dir.

## Tasarım

Aşağıdaki sözdizimi **fikir taslağıdır**; kesin kelimeler uygulama
K-işinde K-096 kullanılabilirlik kartı ve morfoloji profiliyle (zee-tr-1)
seçilir ve K-167 yüzey kaydına o zaman yazılır.

### 1. UTF-8 bayt sayısı ve onaltılık

```text
oktet metnin utf8 bayt sayısı olsun                         # TamSayı
gösterim metnin utf8 baytlarının onaltılık hâli olsun       # Metin, "c3a7" gibi
geri gösterimin onaltılıktan utf8 çözümünü almayı dene olsun # Sonuç<Metin>
```

- `utf8 bayt sayısı` toplamdır; `uzunluğu` (harf sayısı) değişmez.
- Onaltılık çıktı küçük harf, ayraçsız (RFC-0036 ile aynı biçim).
- Onaltılık çözme: tek sayıda hane, `[0-9a-fA-F]` dışı karakter (boşluk
  dahil) başarısız `Sonuç`; büyük harf kabul edilir, çıktı her zaman
  küçük harfle yeniden kodlanabilir.
- **Katı UTF-8:** çözülen bayt dizisi geçerli UTF-8 değilse (aşırı uzun
  kodlama, vekil `U+D800–DFFF`, `> U+10FFFF`, yarım dizi) başarısız
  `Sonuç`; `U+FFFD` ile onarma YASAK olacaktır. Hata verisi ilk geçersiz
  baytın konumunu (`konum`, TamSayı) taşır, baytın kendisini taşımaz.
- BOM (`EF BB BF`) metne `U+FEFF` olarak çözülür; atlanmaz (öngörülebilirlik).
- NUL (`U+0000`) geçerli UTF-8'dir ve çözülür; AUTH PLAIN `\0kullanıcı\0parola`
  biçimini kurabilmek için `kod noktasının harfi` ile de üretilebilir.

### 2. base64

```text
kod metnin base64 hâli olsun                           # Metin (UTF-8 baytlarının base64'ü)
kod2 gösterimin onaltılıktan base64 hâlini almayı dene olsun  # Sonuç<Metin>
düz kodun base64 çözümünü almayı dene olsun            # Sonuç<Metin>  (UTF-8 katı)
ham kodun base64 çözümünü onaltılık almayı dene olsun  # Sonuç<Metin>  (ikili → onaltılık)
```

- Alfabe RFC 4648 §4 standart; dolgu (`=`) kodlamada her zaman yazılır,
  çözmede **zorunludur**; dolgusuz, fazla dolgulu, alfabe dışı, gömülü
  boşluk/CRLF taşıyan girdi ve kanonik olmayan son bitler (RFC 4648 §3.5)
  başarısız `Sonuç`tur.
- MIME gövdesi gibi 76 karakterde kırılmış base64 bu yüzeyin işi değildir
  (MIME alt RFC'si, dosya üzerinde akışla); bu yüzey tek satır protokol
  alanları içindir.
- `base64url` (RFC 4648 §5) ayrı ad: `… base64url hâli` (dolgusuz kodlama,
  çözmede dolgu isteğe bağlı değil — tasarım günü RFC 7515 pratiğine göre
  açık soru).
- Onaltılık ↔ base64 dönüşümü ikili veriyi UTF-8'e zorlamadan taşır (DKIM
  `p=` açık anahtarı, özet çıktıları). Böylece ikili veri Zee'ye hiçbir
  zaman ham bayt olarak girmez.

### 3. Kod noktası

```text
sayı harfin kod noktası olsun                 # TamSayı; tek harfli Metin değilse çalışma hatası
harf sayının kod noktası harfini almayı dene olsun   # Sonuç<Metin>
```

- `harfin kod noktası` tam bir Unicode skaler değerden oluşan `Metin` ister;
  boş ya da çok harfli girdi T034 benzeri tür/değer hatası mı yoksa
  `Sonuç` mu olmalı — açık soru (öneri: çalışma hatası, çünkü `harfleri`
  listesinden gelen öğede programcı hatasıdır).
- Ters yön `Sonuç`tur: negatif, `> 0x10FFFF` ve vekil aralığı başarısızdır.
- Grafem kümesi (birleşik karakter) kavramı yoktur; `harfleri` ile aynı
  skaler değer modeli.

### 4. Bellekte SHA-256

```text
özet metnin sha256 özeti olsun                          # Metin, 64 küçük harf hex
bh kanonik_gövdenin sha256 özetinin base64 hâli olsun   # Metin, DKIM bh= biçimi
ikili gösterimin onaltılıktan sha256 özetini almayı dene olsun  # Sonuç<Metin>
```

- Girdi metnin UTF-8 baytlarıdır; satır sonu dönüşümü yapılmaz (DKIM
  kanonikleştirmesi CRLF'yi açıkça ister, Zee kodu yazar).
- Çıktı biçimi spec/26 dosya özetiyle aynıdır (64 küçük harf hex); böylece
  `dosyanın sha256 özeti` ile `metnin sha256 özeti` doğrudan karşılaştırılır.
- `… özetinin base64 hâli` ham 32 baytlık özetin base64'üdür (hex
  metnin base64'ü değil); bu ayrım tanının ve testin konusudur.
- Sabit zamanlı karşılaştırma: özet karşılaştırması gizli değer içermez
  (DKIM özetleri kamusaldır); gizli değer karşılaştırması bu RFC'nin işi
  değildir ve parola/TOTP yüzeylerinde zaten host'tadır.

### 5. Dosya ↔ kodlanmış içerik (ZM-F034, Dilim 1)

**(a) Dosyayı `Metin` olarak okuma.** Mevcut `"yol" dosyasını okumayı dene`
korunur (16 MiB, katı UTF-8, CRLF korunur) ve A2c blok dosyalarında
kullanılabildiği spec'te açıkça yazılır ve testlenir. Üç eksik kapatılır:

```text
bölüm yol dosyasının konum baytından n baytını metin olarak okumayı dene olsun   # Sonuç<Metin>
ham yol dosyasının konum baytından n baytını base64 okumayı dene olsun          # Sonuç<Metin>
```

- **Aralıklı okuma** (konum + uzunluk, bayt): başlık bölümünü ya da tek bir
  MIME parçasını okumak için; `n` ≤ 1 MiB (açık soru), dosya sonu aşılırsa
  olan kadarı. Metin kipinde aralık UTF-8 dizisini ortadan bölerse
  başarısız `Sonuç` (`tür: utf8`, `konum`); ürün sınırı CRLF'ye hizalar.
- **base64 okuma:** UTF-8 olmayan içerik (8bit gövde, ikili ek) Zee'ye
  kodlanmış `Metin` olarak gelir; tasarım ilkesi 1 korunur.
- **IO izi:** bu iki yeni okuma izde **yol + konum + n + okunan bayt +
  SHA-256** olarak yer alır, içerik olarak değil; replay için içerik, iz
  yanındaki içerik-adresli dosyadan ya da hermetik test dosyasından gelir
  (spec/21 ayrıntısı uygulama ADR'sinde). Mevcut `dosyasını okumayı dene`
  izi değişmez (uyumluluk); büyük blok dosyalarında aralıklı okuma önerilir.

**(b) Çözerek dosyaya bayt-kesin yazma.**

```text
tutamak parçayı base64 çözerek yol dosyasına yazmayı dene olsun              # Sonuç<Sözlük>
tutamak parçayı quoted-printable çözerek yol dosyasına yazmayı dene olsun    # Sonuç<Sözlük>
tutamak satırı base64 çözerek yol dosyasına eklemeyi dene olsun              # akışlı ekleme
```

- Sonuç, A2c/spec/26 ile **aynı tutamak biçimidir**: `{yol, sha256, bayt}`;
  böylece çıktı doğrudan RFC-0041 dosya yanıtına (`… türünde … adıyla ek
  olarak göndermeyi dene`) ve SMTP ekine bağlanır.
- **base64:** MIME gövdesindeki satır sonları (CRLF) ve satır sonu
  boşlukları çözümde atlanır (RFC 2045 §6.8); bunun dışında katı: alfabe
  dışı karakter, kanonik olmayan son bitler, ortada dolgu başarısız `Sonuç`.
  (§2'deki tek satır protokol çözümünden farkı yalnız CRLF/boşluk atlamadır;
  iki kip ayrı adla.)
- **Quoted-printable:** RFC 2045 §6.7; `=XY` (büyük/küçük onaltılık), yumuşak
  satır sonu `=` + CRLF birleştirilir, sert CRLF aynen yazılır, satır sonu
  boşlukları atılır. Bozuk `=` dizisi: katı ret mi, RFC 2045'in önerdiği
  "olduğu gibi bırak" mı — açık soru; öneri **katı ret + `konum`**, çünkü
  gevşek çözme filtre/MUA ayrıştırma farkı üretir.
- **Yazma semantiği:** `yazmayı dene` hedefin bütün yeni içeriğidir;
  geçici dosyaya yazılır, `fsync`, var olan hedef **ezilmez** (spec/26
  no-clobber taşıma ile aynı; ezme gerekiyorsa ürün önce siler), dizin
  eşzamanlanır. `eklemeyi dene` akışlı çözüm içindir: satır satır gelen
  base64'te her çağrı tam 4'lük gruplardan oluşmalı (dolgu yalnız son
  çağrıda); QP'de her çağrı tam satırdır. Ekleme hedefi yalnız aynı
  bağlantı/istek içinde açılmış `.parca` çalışma dosyası olabilir mi, yoksa
  her proje dosyası mı — açık soru; öneri: ekleme yalnız `.zee/` altındaki
  çalışma dosyasına, bitince atomik taşıma.
- **Bütçe:** çıktı boyutu ayırmadan önce kestirilir; hedefin toplam boyutu
  bağlantı/istek blok tavanını (`bağlantı_blok_sınırı`, spec/26 yükleme
  tavanı) aşamaz. Aşım: hedef yarım kalmaz, geçici dosya silinir.
- **Yetkinlik:** `dosya-yazma` (ve kaynak okunursa `dosya-okuma`), proje
  dosya sınırı (spec/07 K-382) aynı.
- **IO izi:** `{yol, bayt, sha256}` ve çözme türü; içerik yok.

**(c) Bellekte SHA-256:** §4 aynen Dilim 1'e alınır (DKIM ve parça bütünlüğü
için `metnin sha256 özeti`).

### Sınırlar

| Sınır | Önerilen değer | Gerekçe |
|---|---|---|
| Kodlama/özet girdisi | 1 MiB UTF-8 bayt (açık soru) | Kanonik başlık bloğu, AUTH, DKIM alanları KiB düzeyinde; büyük gövde dosya yolundan geçer |
| Çözme girdisi | çıktı 1 MiB'ı aşmayacak uzunluk; ayırmadan önce hesaplanır | Bellek amplifikasyonu yok |
| Kod noktası | tek skaler değer | — |
| Aralıklı okuma `n` | ≤ 1 MiB/çağrı (açık soru) | Parça parça işleme; bellek |
| Çözerek yazma | hedef ≤ blok/yükleme tavanı; çağrı girdisi ≤ 1 MiB | Amplifikasyon yok (base64 çıktısı girdinin ¾'ü) |

Sınır aşımı spec/24 kaynak bütçesi sınıfında yakalanabilir başarısız
`Sonuç` (çözme) ya da çalışma hatasıdır (toplam işlemler); ayrıntı
uygulama K-işinin kararıdır.

### Hata modeli

- Yeni bir çalışma zamanı tanı kodu (sıradaki boş `C0nn`) "kodlama/çözme
  başarısız" için önerilir; `Hata` verisinde `tür` alanı: `alfabe`,
  `dolgu`, `uzunluk`, `utf8`, `kod_noktası`, `boyut_aşımı`; `konum`
  (TamSayı) varsa.
- Hata mesajı girdinin kendisini yazmaz (girdi gizli olabilir: AUTH PLAIN
  parolası). Bu, K-236 gizli değer maskesi ilkesinin genellemesidir.
- Katalog (`docs/hata-katalogu.md`) ve `katalog_testi` aynı K-işinde.

## Güvenlik etkisi

| Tehdit | Önlem |
|---|---|
| Bozuk UTF-8'in `U+FFFD` ile onarılması → iki ayrı bayt dizisinin aynı metne eşlenmesi (özet/imza karışıklığı, başlık kaçakçılığı) | Katı çözme; onarma YASAK; hata konumu |
| Gevşek base64 (boşluk, dolgusuz, kanonik olmayan bitler) → aynı değerin birden çok gösterimi, ayrıştırıcı farkı saldırıları | RFC 4648 katı kip; çok satırlı base64 bu yüzeyde yok |
| Gizli değerin (AUTH PLAIN parolası) hata mesajına, IO izine ya da günlüğe sızması | Saf işlemler izde yer almaz; hata mesajı girdi taşımaz; `gizli` değer modeli (spec/23 K-339) ayrıca değer içeriğini korur |
| Bellek/CPU tüketimi (dev girdi, çözmede amplifikasyon) | Ayırmadan önce boyut; 1 MiB tavan; tek geçişli crate uygulaması |
| Zamanlama yan kanalı | base64/hex çözme gizli anahtar için kullanılmaz (anahtar Zee değeri olmaz — RFC-0039 ilke 2); kamu verisi |
| Ek dosyasının yarım/bozuk kalması ya da var olan dosyanın ezilmesi | Geçici dosya + `fsync` + no-clobber; hatada geçici silinir |
| Gevşek QP/base64 çözümünün filtre ile MUA arasında farklı içerik üretmesi | Katı çözme + `konum`; gevşeklik yalnız RFC 2045'in açıkça istediği CRLF/boşluk atlaması |
| Büyük blok içeriğinin IO izine (ve oradan yedeklere/günlüklere) sızması | Yeni okuma/yazma işlemleri izde yalnız yol + bayt + SHA-256 |
| Yol enjeksiyonu (ek adı dosya yoluna) | Hedef yol Zee'nin kurduğu göreli yol; spec/26 kuralı: gönderilen dosya adı hiçbir koşulda disk yoluna girmez — ürün kendi adlandırır |
| `Metin` uzunluğu ile oktet sınırının karışması (SMTP satır/boyut sınırının aşılması) | Ayrı `utf8 bayt sayısı`; spec/23 SMTP istemcisi ve A2 sunucu sınırları zaten bayt sayar |

Güvenlik bulgusu kaydı: bu taslak yeni bulgu açmaz. Uygulama K-işinde katı
çözme kararı `docs/guvenlik-bulgulari-v1.tsv` içinde "bilinçli güvenlik
sınırı" satırı (GB-NNN) olarak kaydedilir.

## Bağımlılık seçimi

- **base64:** `base64 = "=0.23.1"` doğrudan bağımlılığa yükseltilir — kilit
  dosyasında zaten var (ureq, bcrypt), yeni crate/sürüm eklenmez; `ADR-038`
  tedarik zinciri incelemesi yalnız doğrudan kullanım kaydıdır. Üç iç
  yardımcı (`eposta.rs`, `main.rs`, `guvenlik/gizli.rs`) aynı işte bu
  uygulamaya taşınır ve mevcut testleri (RFC 4648 vektörleri) korunur.
  Alternatif: iç yardımcılardan birini tek kaynak yapmak (crate'siz); seçim
  uygulama günü kod büyüklüğü ve fuzz kapsamıyla verilir.
- **SHA-256:** `sha2 0.11.0` (doğrudan, mevcut) ve `guvenlik::sha256_hex`.
- **Onaltılık / UTF-8:** standart kitaplık (`str::from_utf8`), yeni crate yok.
- Yeni crate gerekmez. Fuzz hedefi önerisi: `kodlama` (base64/hex/utf8
  çözme gidiş-dönüş eşdeğerliği).

## Yetkinlik etkisi

Dilim 1 §5 dosya işlemleri mevcut `dosya-okuma`/`dosya-yazma`
yetkinliklerine bağlıdır; yeni yetkinlik yoktur. Diğer işlemler saf
hesaplamadır; `kriptografi` yetkinliği istemez (özet
gizli değil, anahtar yok). `dosya-okuma` gerektiren dosya özeti spec/26'da
olduğu gibi kalır. Açık soru: SHA-256'nın "kriptografi" sınıfında sayılıp
sayılmayacağı — öneri hayır, çünkü yetkinlik modeli (spec/23) **dış etki**
sınırıdır, algoritma sınıfı değildir.

## Deterministik IO

Saf işlemler IO izine olay yazmaz; replay aynı sonucu kendiliğinden üretir.
§5 dosya işlemleri izde yol/konum/bayt/SHA-256 ile yer alır (içerik yok);
bu, mevcut `dosya_oku` olayının içerik taşıyan şemasından bilinçli bir
ayrılıştır ve spec/21 ile uygulama ADR'sinde kayda geçer.
`dil dene` hermetik çalışmada da aynıdır.

## Core freeze gerekçesi ve kanıt kapısı

1. **Ürün kaydı.** `urun-h` (`docs/dogfood-projeleri-v1.tsv`, K-355) etkin;
   kanıt kökü [`dogfood/urun-h`](../dogfood/urun-h/README.md). Karşılanmış.
2. **Ürün sürtünmesi.** ZM-F023/ZM-F024 ürünün kendi kaydındadır (depo
   dışı). Uygulama K-işi açılmadan önce ürün, eksik yüzeye çarpan gerçek
   Zee kaynağını (AUTH PLAIN çözümü, SMTP `SIZE`/satır oktet sınırı, DKIM
   `bh=` hesabı) `dogfood/urun-h/korpus/` altına exact beklentiyle koyar.
   Elle base64 döngüsünün ASCII dışı girdide yanlış sonucu ve `uzunluğu`nun
   oktet saymaması iki ayrı reproducer'dır. Ürün korpusunda hazır (ürün
   commit'i `7f7b1fe48035ccff1fe4c78810b19e00234ca8ac`, 24 Kasım 2026; depo
   içine aşama açılırken aynalanır):
   `korpus/zm-r004-uzunluk-oktet-saymaz.dil` (blob `60e565e7…`, SHA-256
   `77adcdc0e727bf1f56dace255f337b70182f16ff450a4c795ee2571950acaa4c`;
   bugün geçer: `"ğ"` uzunluğu 1, Dilim 2'de oktet sayısı 2) ve
   `korpus/zm-r005-bellekte-sha256-yok.dil` (blob `f994eaab…`, SHA-256
   `cd268588fe23277b5a8adc541ded1a5776e4d75dfee178ddb2734b3bd400a1db`;
   bugün S015). ZM-F034 için: A2c ile alınmış
   base64 ekli ileti dosyasından eki çıkaramayan ürün kodu (bugün
   `dosyasını okumayı dene` 8bit içerikte başarısız, eki bayt yazacak kalıp
   yok) ve ekli gönderimin `554`'e düşen yolu.
3. **Tekrar sinyali.** base64 ihtiyacı derleyicinin kendi içinde üç kez
   yazılmıştır (SMTP, gizli değer, CLI); Zee tarafında SMTP istemcisi
   (spec/23), A2 sunucusu ve RFC-0044 DKIM/DNS yolları aynı yapı taşını
   ister. K-164 kataloğuyla karşılaştırma uygulama K-işinde yapılır.
4. **Minimal değişiklik.** Yeni tür, grammar kuralı ya da anahtar kelime
   yoktur; mevcut iyelik/özellik kalıp ailesi (`metnin uzunluğu`, `metnin
   harfleri`, `dosyanın sha256 özeti`) genişler.
5. **ZeeMail güvenlik önceliği.** RFC-0044'ün DKIM doğrulaması ve imzası bu
   yapı taşları olmadan yazılamaz; gelen postanın sahteciliğe karşı
   denetlenmesi (B2) ve giden postanın büyük sağlayıcılarca kabulü (B3)
   ürünün güvenlik önceliğidir. Bu RFC o zincirin ilk, en küçük halkasıdır.
6. **Zincir.** ADR → spec/04 ya da spec/06 + ilgili kalıp bölümü →
   `spec-madde-kaniti` → olumlu/olumsuz test → reproducer → K-167
   (`kalip` + tanı satırları) → GB kaydı → `docs/surumler.md` aynı
   değişiklikte.

V1 kapı listesine eklenmez; V1 sözü değildir.

## RFC-0039 ve RFC-0041 ile ilişki

- **RFC-0039 katman D** "tutamak temelli yol" ve "küçük `Metin` girdiler
  (AUTH PLAIN, DKIM `b=`) için sınırlı `Metin`→`Metin` biçimi"ni önermişti;
  bu RFC o maddenin alt RFC'sidir ve `Baytlar` türünü D'nin "ertelenen"
  maddesindeki gibi erteler. Akışlı özet bağlamı ve DKIM kanonikleştirmesi
  RFC-0044'e taşınır (dosya üzerinde host işlemi).
- **RFC-0039 A2c** blok dosyası tutamağı (`yol`, `sha256`, `bayt`) değişmez;
  bu RFC'nin `metnin sha256 özeti` çıktısı o alanla aynı biçimdedir.
- **RFC-0041** dosya yanıtı ve SMTP eki ikili veriyi dosya tutamağıyla taşır;
  base64 MIME **kodlaması** orada host'ta kalır. Dilim 1'in çözerek yazma
  çıktısı aynı `{yol, sha256, bayt}` tutamağıdır; gelen ek webmail'e RFC-0041
  A dosya yanıtıyla, istemciye yeniden gönderim RFC-0041 C ek kaydıyla gider.
- **`message/rfc822` (sonraki küçük iş, bu RFC'nin dışında):** ham iletiyi
  `.eml` olarak indirmek için RFC-0041 katman A / ADR-163 dosya yanıtı tür
  kaydına `message/rfc822` satırı (yalnız `attachment`, `nosniff`, sandbox
  CSP; K-385 `application/xml` ekiyle aynı desen). Ayrı K-işi adayı;
  ürün reproducer'ı ile açılır.

## Dört soru süzgeci

1. **Türkçe doğal mı?** — `metnin base64 hâli`, `metnin sha256 özeti`,
   `harfin kod noktası` mevcut iyelik kalıplarını (`metnin uzunluğu`,
   `dosyanın sha256 özeti`) izler. "Hâl" yerine "karşılığı"/"kodu" adayları
   kullanılabilirlik kartıyla seçilir.
2. **Deterministik mi?** — Evet; saf işlemler.
3. **Öğrenilebilir mi?** — Bir kavram eklenir: "bayt Zee'de kodlamasıyla
   görünür". Kodlama adı her cümlede yazılı olduğundan yanlış kodlama
   okunarak fark edilir.
4. **Profesyonel ölçekte savunulabilir mi?** — Katı RFC 4648 ve katı UTF-8,
   Go `encoding/base64.StdEncoding.Strict()`, Rust `base64` `STANDARD`
   motoru ve Python `base64.b64decode(validate=True)` pratiğiyle uyumludur;
   Python `bytes`/Go `[]byte` gibi ayrı bir bayt türünün yokluğu bilinçli
   bir freeze ödünüdür ve kapı (ölçülmüş sürtünme) açıkça yazılıdır.

## Kabul ölçütleri

1. RFC 4648 §10 test vektörleri (base64, base16) gidiş-dönüş geçer;
   dolgusuz/fazla dolgulu/boşluklu/kanonik olmayan bitli girdiler reddedilir.
2. Unicode/UTF-8 olumsuz vektörleri (aşırı uzun `C0 AF`, vekil `ED A0 80`,
   `F4 90 80 80`, yarım dizi) reddedilir ve `konum` doğru gelir.
3. `"ç"` için `utf8 bayt sayısı` 2, `uzunluğu` 1; U+1F600 (😀) için 4 ve 1.
4. SHA-256 NIST FIPS 180-4 örnek vektörleri; boş metin
   `e3b0c442…b855`; aynı içerikli dosya ile metin özeti eşit.
5. (Dilim 1) A2c ile alınmış, base64 ve QP ekli örnek ileti dosyasından
   aralıklı okuma + çözerek yazma ile çıkarılan ekin SHA-256'sı özgün ekle
   eşleşir; ek RFC-0041 dosya yanıtıyla indirilir. 8bit gövdeli dosya
   `metin olarak` okumada `utf8` hatası, `base64 okuma`da başarı verir.
   Bozuk QP `=G1`, ortada dolgu, 4'e bölünmeyen ekleme çağrısı, var olan
   hedef, tavan üstü çıktı → başarısız `Sonuç` ve hedefte yarım dosya yok.
   IO izinde içerik yok, yol/bayt/SHA-256 var.
   25 MB'lık blok dosyası 1 MiB'lık aralıklarla okunup ekleri çıkarılır;
   tek bağlantıda art arda üç ileti C024 vermez (ZM-F035 kararıyla; K-388
   üç iletiyi ayrı birim yapar, tek iletinin aralıklı okuması için açık
   soru 10).
6. Boş gövdenin `simple` kanonik biçimi (`CRLF`) için DKIM `bh=` değeri
   (`frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY=`) ve bağımsız bir DKIM
   uygulamasının (ör. OpenDKIM) ürettiği vektör Zee'de kanonik gövde
   metninden yeniden üretilir.
7. AUTH PLAIN reproducer'ı (`\0kullanıcı\0parola`) base64 gidiş-dönüşüyle
   geçer; hata yolunda parola hiçbir çıktıda görünmez.
8. 1 MiB + 1 bayt girdi ayırmadan reddedilir.

## Alternatifler

- **`Baytlar` türü şimdi** — bkz. tür modeli; freeze maliyeti ve ürün
  ihtiyacının C+D ile karşılanması nedeniyle ertelendi.
- **`Liste<TamSayı>`** — bellek ve hız; ret.
- **Kodlamasız "ham" `Metin` (Latin-1 hilesi: her baytı U+0000–U+00FF
  harfine eşleme)** — ikili veri `Metin` gibi görünür, `uzunluğu` doğru
  sayar ama UTF-8 metinle karıştırılınca sessiz bozulma (çift kodlama)
  üretir; öngörülebilirlik ilkesine aykırı. Ret.
- **Yalnız dosya üzerinde özet/kodlama** — ZM-F024'ün nedeni tam da bu
  (gereksiz `dosya-yazma`, IO izi gürültüsü); ret.
- **Zee'de elle base64/SHA-256 kitaplığı** — ilke 2 (algoritma yazılmaz),
  ADR-003 performansı; ret.

## Korpus etkisi

Golden değişmez. Uygulama K-işinde bir golden (kodlama turu) ve
`dogfood/urun-h/korpus/` altında ZM-F023/ZM-F024 reproducer'ları eklenir.

## Açık sorular

1. Kelime seçimi: `hâli` / `karşılığı` / `kodu`; `çözümünü` / `çözülmüşünü`.
2. Girdi tavanı: 1 MiB mı, spec/24 genel metin tavanına mı bağlanır?
3. `harfin kod noktası` hatalı girdide çalışma hatası mı, `Sonuç` mu?
4. `base64url` çözmede dolgu politikası (RFC 7515 dolgusuz) ve ilk
   sürümde gerekip gerekmediği (DKIM/SMTP istemez; WebAuthn/JWT ihtiyacı
   çıkarsa).
5. Üç iç base64 yardımcısı `base64` crate'ine mi taşınır, yoksa tek iç
   uygulama mı kalır?
6. Yeni tanı kodu mu, yoksa C013/C029 benzeri mevcut bir koda `tür` ile mi
   eklenir? (Öneri: kodlama hataları için yeni kod; §5 dosya işlemlerinin
   IO arızaları C013.)
7. (ZM-F034) QP bozuk dizide katı ret mi, RFC 2045 "olduğu gibi bırak" mı?
8. (ZM-F034) Akışlı `eklemeyi dene` hedefi yalnız `.zee/` çalışma dosyası
   mı, her proje dosyası mı? Çözme durumu (yarım 4'lük grup) çalışma
   zamanında tutulsun mu (öneri: hayır; çağrı başına tam grup)?
9. (ZM-F034) Aralıklı okuma tavanı 1 MiB mı; mevcut `dosyasını okumayı
   dene`nin izine içerik yazması blok dosyaları için ayrıca daraltılsın mı
   (uyumluluk yüzeyi, K-167)?
10. (ZM-F035) Bağlantı görevinde heap bütçesi hangi sınırda yenilenir
   (blok/komut dönüşü mü, açık bir `tur` kalıbı mı) ve bu web'deki istek
   başına bütçe ile nasıl aynı dile getirilir? **K-388 yanıtı:** bağlantı
   okuma dönüşünde (başarı, `zaman_aşımı`, girdi tüketen ret); açık `tur`
   kalıbı eklenmedi. **Açık kalan (Dilim 1):** dosya aralığı okuması bağlantı
   okuması değildir ve bütçeyi yenilemez; 25 MB'lık blok dosyasını tek
   komut içinde aralıklarla ayrıştırmak hâlâ birikimle C024'e yürür.
   Dilim 1 ya aralıklı okuma çağrısını ayrıca birim sınırı sayan (ve onu
   yok sayan döngünün adım sınırından kaçamayacağını gösteren) bir karar,
   ya da seçenek (b) ister.

## Önerilen uygulama sırası ve K-işi bölümlemesi

İki K-işi (numaralar açılışta verilir), RFC-0044'ten **önce**:

**Önkoşul — ZM-F035 (ayrı correctness K-işi):** bağlantı görevinde heap
bütçesinin protokol birimi sınırında yenilenmesi (bkz. tür modeli, ürün
ölçümü). Dilim 1 bununla aynı dalgada ya da sonra kabul edilir; aksi hâlde
aralıklı okuma büyük iletide birikimle C024'e yürür.

**Dilim 1 — uygulandı (K-389, ADR-167; ZM-F034 + ZM-F024).** Ürünün bugünkü
engelini (gelen ekler webmail'e çıkmıyor, ekli gönderim `554`, `APPEND`
64 KB) doğrudan kapatır:

1. ADR + spec/26 (ya da spec/08) maddeleri: mevcut `dosyasını okumayı dene`
   nin blok dosyalarında kullanımı; aralıklı metin/base64 okuma; base64 ve
   QP çözerek no-clobber bayt yazma ve akışlı ekleme; bellekte SHA-256;
   spec/21 iz şeması (içeriksiz). K-167 satırları, tanı/katalog.
2. Uygulama: tek base64 uygulaması (iç yardımcıların birleştirilmesi
   dahil), QP çözücü, fuzz hedefi `kodlama` (base64/QP çözme).
3. Olumlu/olumsuz test, `urun-h` reproducer'ı (ekli ileti → ek dosyası →
   dosya yanıtı), GB satırı (katı çözme sınırı), `surumler.md`.

**Dilim 2 (ZM-F023).** UTF-8 bayt sayısı, metin ↔ base64/onaltılık, kod
noktası; saf `Ozellik` dalları, golden, reproducer'lar.

**Sonraki küçük iş (ayrı):** `message/rfc822` dosya yanıtı türü (RFC-0041
A kaydı).

Bağımlılığı yoktur; RFC-0044 B2/B3 bunların üstüne kurulur.

## İlgili kayıt

- K-383 (bu taslak; ZM-F034 24 Kasım 2026 eki), K-355 (`urun-h`), K-384
  (A2c blok dosyası), K-385 (dosya yanıtı tür eki deseni), K-354/RFC-0039 katman D,
  K-333/RFC-0036 (onaltılık biçim), K-339 (gizli değer kaynağı).
