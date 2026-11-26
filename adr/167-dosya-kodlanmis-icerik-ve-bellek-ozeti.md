# ADR-167 — Dosya ↔ kodlanmış içerik ve bellekte SHA-256 (RFC-0043 Dilim 1)

- **Durum:** kabul
- **Tarih:** 24 Kasım 2026
- **İlgili kayıt:** K-389, K-402 (Dilim 2 eki), RFC-0043 (Dilim 1–2), ADR-059 (core freeze),
  ADR-159 (ilk bayt imzası), ADR-163 (dosya yanıtı ve ek türü kaydı),
  ADR-165 (A2c blok dosyası), ADR-064 (dil yüzeyi), ADR-067 (dogfood
  korpusu), spec/26, spec/08, spec/21, GB-163

## Bağlam

`urun-h` posta motoru K-384 (A2c) ile SMTP `DATA` ve IMAP literal'ini
`yol`/`sha256`/`bayt` tutamaklı blok dosyasına alabiliyor; ama dosyayı
parça parça `Metin` olarak geri okuyamıyor, MIME parçasının base64 ya da
quoted-printable gövdesini bayt olarak ek dosyasına yazamıyor ve bellekteki
metnin SHA-256'sını alamıyordu (ZM-F034, ZM-F024). Ürünün bildirdiği etki:
gelen ekler webmail'e çıkarılamıyor, ekli gönderim `554` ile düşüyor, IMAP
`APPEND` 64 KB'ta takılıyor. Mevcut `dosyasını okumayı dene` 16 MiB'ın
tamamını katı UTF-8 okur; 8bit gövde ya da ikili ekte başarısız olur ve
içeriği IO izine yazar.

Core freeze yeni kalıbı varsayılan olarak reddeder; ürün kaydı etkin
(`urun-h`), reproducer'lar depo içinde (`dogfood/urun-h/korpus/zm-r005…`,
`dogfood/urun-h/ek-cikarma/`), yeni tür ya da grammar kuralı yok.

## Karar

1. **Aralıklı okuma** (`dosya-okuma`, `DisOkuma`):

   ```zee
   bölüm yol dosyasının konum baytından n baytını metin olarak okumayı dene olsun
   ham yol dosyasının konum baytından n baytını base64 okumayı dene olsun
   ```

   `konum` ≥ 0 ve `n` 1…1 048 576 (1 MiB) TamSayıdır; dışı dosyaya
   dokunmadan C013. Dosya sonu aşılırsa olan kadarı, konum dosya sonundaysa
   `""`. Metin kipi katı UTF-8'dir (onarım YASAK); aralık çok baytlı diziyi
   bölerse ya da içerik UTF-8 değilse C032 `tür: utf8`, `konum` = ilk
   geçersiz baytın **dosya içi** konumu. base64 kipi aralığı RFC 4648 §4
   dolgulu base64 `Metin` olarak verir. Yol kuralı `sha256 özetini almayı
   dene` ile aynıdır; normal dosya olmayan hedef C013.
2. **Çözerek yazma** (`dosya-yazma`, `DisYazma`):

   ```zee
   tutamak parça base64 çözerek yol dosyasına yazmayı dene olsun
   tutamak parça quoted_printable çözerek yol dosyasına yazmayı dene olsun
   tutamak satırlar base64 çözerek yol dosyasına eklemeyi dene olsun
   tutamak satırlar quoted_printable çözerek yol dosyasına eklemeyi dene olsun
   ```

   Sonuç `Metin sözlüğü sonucu`: `yol`, `sha256` (hedefin bütün içeriği, 64
   küçük harf onaltılık), `bayt` (toplam, Metin) — A2c ve web yükleme
   tutamağıyla aynı biçim; `yol` doğrudan RFC-0041 dosya yanıtına ve SMTP
   ek listesine verilir.
   - **Çözme önce, IO sonra:** içerik saf `kodlama` modülünde tamamen
     çözülür; hata hedefe hiç dokunmadan C032'dir. Çağrı başına çözülen
     çıktı ≤ 1 MiB, ayırmadan önce kestirilir (`boyut_aşımı`).
   - **MIME base64:** yalnız CR, LF, boşluk ve sekme atlanır (RFC 2045
     §6.8); dolgu zorunlu ve yalnız sonda; alfabe dışı karakter (`alfabe`),
     4'e bölünmeyen uzunluk (`uzunluk`), ortada/fazla dolgu ve kanonik
     olmayan dolgu bitleri (`dolgu`) ret. Her çağrı kendi içinde tam
     dörtlülerden oluşur; çağrılar arası durum tutulmaz.
   - **Quoted-printable:** `=XY` (büyük/küçük onaltılık), yumuşak satır sonu
     `=`+CRLF birleşir, satır sonu boşluk/sekme atılır, sert CRLF aynen
     yazılır. Bozuk `=` dizisi `kaçış`, çıplak CR/LF `satır_sonu`,
     yazdırılabilir ASCII dışı karakter `alfabe` ile **katı** ret (RFC 2045'in
     "olduğu gibi bırak" gevşekliği seçilmedi: filtre/MUA farkı üretir).
   - **Yazma:** `yazmayı dene` yeni dosyadır; kilit altında var olan hedef
     (dosya, dizin, sembolik bağ) ezilmez (C013 `hedef dosyası zaten var`),
     içerik geçici dosya + fsync + atomik adlandırma ile görünür.
     `eklemeyi dene` spec/08 eklemesiyle aynı kilitli oku-birleştir-atomik
     değiştir modelidir; hedef yoksa oluşturulur.
   - **Tavan:** hedefin toplam boyutu spec/26 yükleme/özet tavanını (16 MiB;
     dosya yanıtı ve `sha256 özetini almayı dene` ile aynı) aşamaz; aşan
     çağrı hiçbir bayt yazmaz. **K-395 eki:** tavan 32 MiB'a çıktı (aşağıda).
   - **Eylem:** `yazmayı dene` eylem savepoint'ine yeni dosya olarak
     kaydedilir (geri almada silinir; GET/HEAD rotasında T045, rota
     gövdesinde T046 her dosya yazımı gibi). `eklemeyi dene` eylem içinde
     C013 ile reddedilir: akışlı çalışma dosyası eylem dışında biriktirilip
     eylemde atomik taşınır. Aynı eylemde PostgreSQL yazımıyla birlikte
     kullanılamaz.
3. **Bellekte SHA-256** (saf, yetkinliksiz): `özet metnin sha256 özeti olsun`
   → UTF-8 baytlarının SHA-256'sı, 64 küçük harf onaltılık; satır sonu
   dönüşümü yok. Metin dışı nesne derlemede tür hatasıdır. Ayrı bir girdi
   tavanı yoktur; girdi zaten spec/24 metin/heap bütçesiyle sınırlıdır.
4. **Tanı:** yeni C032 "kodlanmış içerik çözülemedi"; `Hata` verisi `tür`
   (`alfabe`, `dolgu`, `uzunluk`, `kaçış`, `satır_sonu`, `utf8`,
   `boyut_aşımı`) ve varsa `konum` (Metin) taşır. Mesaj girdiyi taşımaz.
   Dosya/aralık/yol/no-clobber/tavan/eylem arızaları C013'tür.
5. **IO izi (içeriksiz):** `dosya_araligi_oku [yol, konum, adet] →
   ok,<bayt>,<sha256> | hata,<mesaj>` ve `dosyaya_bayt_yaz [yol, yaz|ekle,
   <bayt>, <sha256>] → ok,<sha256>,<toplam> | hata,<mesaj>`. İz içerik
   taşımaz. Yazım olayı replay'de exact sonucu döndürür; **başarılı aralıklı
   okuma replay'de içerik üretemez** ve iz uyuşmazlığıyla fail-closed biter.
   Mevcut `dosya_oku` olayı değişmez (uyumluluk).

## Ek — K-395: hedef tavanı 32 MiB (24 Kasım 2026)

`urun-h` 25 MB'lık iletideki tek ~18 MB'lık eki kendi `ekdeposu` paketiyle
(satıra hizalı pencereler, dil değişikliği yok) çıkarabildi; tek engel
16 MiB hedef tavanıydı (ZM-F039). Mustafa kararıyla (24 Kasım 2026) yalnız
bu tavan yükseltildi. Önce hazırlanan dosyadan dosyaya akışlı base64 çözme
ve dosyada bayt arama yüzeyleri birleştirilmeden geri çekildi (PR #100
kapatıldı); RFC-0043 Dilim 2 kalanıyla birlikte taslaktır.

- `çözerek … yazmayı|eklemeyi dene` hedef toplamı **32 MiB**
  (33 554 432 bayt); tam sınır kabul, bir bayt fazlası C013 ve hedef
  değişmez. Çağrı başına çözülen çıktı 1 MiB kalır.
- Ekleme var olan hedefi artık genel 16 MiB okuma sınırıyla değil, kilit
  altında önceden denetlenen bu tavanla okur (aksi hâlde 16 MiB üstü hedefe
  ekleme okuma sınırına çarpıyordu).
- Yükleme, dosya yanıtı ve dosya özeti tavanları 16 MiB kalır: 18 MB'lık ek
  dosya yanıtıyla indirilemez ve özetlenemez; ürün tutamağın `sha256`
  alanını kullanır (açık sınır, ayrı karar).
- DoS gerekçesi GB-167: bellek tepe noktası ekleme başına ~32 MiB (mevcut
  içerik + yeni baytlar), disk proje başına iki katı; tavan ürünün 25 MB
  ileti sınırına göre seçildi.

## Ek — K-401: özet ve dosya yanıtı 32 MiB, izsiz taşıma (25 Kasım 2026)

K-395 sonrasında `urun-h` 19 MiB'lık eki çözerek yazabildi ama sonraki üç
adım 16 MiB'ta kaldı (ZM-F044): eki yerine taşımak, özetlemek ve webmail'den
indirmek. Aynı kullanıcı kararının (25 MB iletideki ekin uçtan uca
çalışması) devamı olarak:

- `… dosyasının sha256 özetini almayı dene` en çok **32 MiB**'ı 16 KiB
  parçalarla okur; tam sınır özetlenir, bir bayt fazlası C013.
- `… ek olarak göndermeyi dene` dosya yanıtı tavanı **32 MiB** (ADR-163 §5
  güncellendi). Gövde zaten 16 KiB parçalarla akıyordu; tam sınırda işçinin
  yerleşik bellek tepesinin büyümediği gerçek süreçte örneklenerek
  sınanır. K-372 yazım bütçesi 32 MiB için ~266 sn olur.
- **Hata düzeltmesi:** eylem dışındaki `atomik taşımayı dene` de geri alma
  için içeriği 16 MiB veri okuma sınırıyla belleğe okuyordu; çözülmüş büyük
  ek taşınamıyordu. Silmedeki gibi eylem dışında izsiz taşıma kullanılır:
  içerik okunmaz, boyut sınırı yoktur; no-clobber ve proje kökü kuralları
  aynıdır. Eylem içinde geri alma kaydı içeriği tuttuğundan 16 MiB sınırı
  kalır (spec/26). Regresyon: `regression/runtime/buyuk-dosya-izsiz-tasima.dil`.
- Yükleme tavanı 16 MiB kalır (web gövdesi; ayrı karar). Bağlantıya base64
  aktarımın (K-396, master'da değil) kaynak tavanı kendi PR'ında 32 MiB'a
  hizalanır.
- DoS gerekçesi GB-169: dosya yanıtı ve özet akışlıdır (bellek tepesi
  büyümez); süre ve bant genişliği K-372 bütçesiyle sınırlıdır; izsiz taşıma
  bellek kullanmaz.

## Ek — K-402: Dilim 2, bellekteki metin ↔ bayt (25 Kasım 2026)

ZM-F023'ün ürün sürtünmesi (ürün commit'i `23af030`): ürün bayt sayımı,
kod noktası ↔ harf ve base64'ü saf Zee'de U+0000–04FF kod tablosuyla
yapıyor. Tablo dışındaki harf IMAP literal `{n}` sayımında 1 bayt
sayılıyor (oturum kayar), RFC 2047 konusunda ve AUTH PLAIN parolasında
U+FFFD'ye, giden B kodlamasında `?`'e dönüyor; matematik alfanümerik
harflerle taklit konu süzgece kör geliyor.
Karar (spec/26 §Bellekteki metin ↔ bayt):

- **Beş saf kalıp, yeni tür/grammar yok.** `<metin>in utf8 bayt sayısı`
  (TamSayı), `<metin>in base64 karşılığı` (Metin, dolgulu, toplam),
  `<kod>un base64 çözümünü almayı dene` (Sonuç<Metin>), `<harf>in kod
  noktası` (TamSayı), `<sayı>nın kod noktası harfini almayı dene`
  (Sonuç<Metin>). Hepsi mevcut `Ozellik` ailesine katılır; yetkinlik ve
  IO izi olayı yoktur (RFC-0043 ilke 4).
- **Açık soruların kapanışı.** (1) Kelime `karşılığı` (kodlama) ve
  `çözümünü` (çözme); `hâli` şapkalı harf ve `hal` çekimiyle belirsizdi.
  (2) Çözme çıktısı Dilim 1 ile aynı 1 MiB, ayırmadan önce; kodlama
  girdisi spec/24 metin sınırına bağlıdır (çıktı boyu önceden hesaplanır).
  (3) `kod noktası` tek harf dışında **çalışma hatası** (C032): `harfleri`
  öğesinde oluşmaz, programcı hatasıdır; ters yön `Sonuç`tur. (4)
  `base64url` ve (onaltılık) açılmadı: ürün kaydı yok, DKIM K-390 host
  yüzeyinde. (5) İç base64 yardımcıları (K-410 ile birleştirildi): yeni yüzey tek iç çözücüyü
  (`kodlama.rs`) kullanır; SMTP/CLI/gizli değer yardımcıları bu dilime
  katılmadı, K-410 ile ayrıca birleştirildi (ADR-083 eki). (6) Yeni tanı kodu yok: C032 `tür` kümesine
  `kod_noktası` eklendi.
- **Tek satır katılığı.** MIME çözümünden farklı olarak CR/LF/boşluk/sekme
  de alfabe dışıdır (RFC 4648 §3.3): protokol alanında gömülü satır sonu
  komut kaçırma (smuggling) yüzeyidir. Çözülen baytlar katı UTF-8'dir;
  ikili veri dosya tutamağıyla taşınır (tür modeli C + D).
- **Gizlilik.** Hata mesajı ve `Hata` verisi girdiyi ya da çözülmüş
  içeriği taşımaz (AUTH PLAIN parolası); GB-178.

## Değerlendirilen seçenekler

- **`base64` crate'ini doğrudan bağımlılık yapmak** — Dilim 1'de
  seçilmedi: MIME satır atlaması ve hata konumu için crate çıktısının
  yeniden eşlenmesi gerekirdi; tek, küçük, RFC 4648 vektörleriyle sınanan iç
  çözücü yeni doğrudan bağımlılık ve tedarik kaydı açmaz. Üç eski iç base64
  yardımcısının birleştirilmesi Dilim 2'ye bırakıldı (açık).
- **Eklemeyi `O_APPEND` ile yapmak** — reddedildi: eşzamanlı okuyucu yarım
  ek görebilirdi; spec/08 ekleme atomikliği korunur (çağrı başına mevcut
  içerik yeniden yazılır, 16 MiB tavanla sınırlı).
- **Eklemeyi yalnız `.zee/` çalışma dosyasına sınırlamak** — reddedildi:
  `dosyasına … ekle` zaten her proje dosyasına ekler, ek güvenlik vermez;
  hedef Zee'nin kurduğu yoldur.
- **Çağrılar arası yarım base64 dörtlüsünü saklamak** — reddedildi: uzun
  ömürlü durum (K-173 soak), bağlantılar arası sızıntı riski; ürün satır
  hizalı okur.
- **İzde içerik saklamak** — reddedildi: A2c "blok baytı izde olmaz"
  sözü ve 16 MiB'lık izler; içerik-adresli yan dosya ileride ayrı karar.

## Sonuçlar

- Ürün gelen iletinin eklerini 1 MiB aralıklarla okuyup bayt-kesin ek
  dosyasına çıkarır; tutamak dosya yanıtıyla webmail'e, ek listesiyle SMTP'ye
  gider (`kodlanmis_dosya_testi`).
- Bayt sayısı, metin↔base64 ve kod noktası K-402 ekiyle (RFC-0043 Dilim
  2) açıldı; ZM-R004 `uzunluğu` için `1`, `utf8 bayt sayısı` için `2` verir.
  Onaltılık ve base64url açık.
- ZM-F035 (bağlantı bütçesi) K-388'dedir; çok büyük iletide birikim
  davranışı o kararla birlikte okunur.
- Freeze yüzeyi: kalıp kelimeleri `baytından`, `base64`,
  `quoted_printable`, `çözerek`, `özeti`; API `dosya.aralik_metin`,
  `dosya.aralik_base64`, `dosya.base64_cozerek_yaz/ekle`,
  `dosya.qp_cozerek_yaz/ekle`; iz olayları `dosya_araligi_oku`,
  `dosyaya_bayt_yaz`; tanı C032 (`dil-yuzeyi-v1.tsv`, 1.0.0-dev).

## Kanıt

- `compiler/tests/kodlanmis_dosya_testi.rs` (11 vaka: bellekte SHA-256 NIST
  vektörleri, C032 verisi, aralık/UTF-8 bölünmesi, 1 MiB tavanı, içeriksiz
  iz/replay, 2,5 MB ikili ekin MIME iletisinden bayt-kesin çıkarılması,
  8bit gövde, no-clobber/tavan/yol geçişi/sembolik bağ/eylem, T054, RFC-0041
  dosya yanıtı)
- `compiler/src/kodlama.rs` birim testleri (RFC 4648 vektörleri, katı ret,
  QP, UTF-8 konumu) ve `compiler/src/kalici_dosya/cozulmus.rs`
- Dogfood: [`dogfood/urun-h/ek-cikarma`](../dogfood/urun-h/ek-cikarma/README.md),
  `dogfood/urun-h/korpus/zm-r004…`, `zm-r005…`
- K-402: `compiler/tests/metin_kodlama_testi.rs` (10 vaka: oktet sayısı,
  RFC 4648 vektörleri, katı çözme retleri, UTF-8 olumsuz vektörleri, AUTH
  PLAIN gidiş-dönüşü ve gizli hata yolu, 1 MiB tavanı, kod noktası sınırları
  ve çalışma hatası, tür hataları, saflık/IO izi); dogfood
  `korpus/zm-f023-imap-literal-oktet.dil`, `zm-f023-rfc2047-emoji.dil`,
  `zm-f023-auth-plain.dil`
