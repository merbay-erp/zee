# ADR-167 — Dosya ↔ kodlanmış içerik ve bellekte SHA-256 (RFC-0043 Dilim 1)

- **Durum:** kabul
- **Tarih:** 24 Kasım 2026
- **İlgili kayıt:** K-389, RFC-0043 (Dilim 1), ADR-059 (core freeze),
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
bu tavan yükseltildi. Aynı dalda önce uygulanan dosyadan dosyaya akışlı
base64 çözme ve dosyada bayt arama yüzeyleri (`1336876`) birleştirilmeden
geri alındı (`0adf5ef`); RFC-0043 Dilim 2 kalanıyla birlikte taslaktır.

- `çözerek … yazmayı|eklemeyi dene` hedef toplamı **32 MiB**
  (33 554 432 bayt); tam sınır kabul, bir bayt fazlası C013 ve hedef
  değişmez. Çağrı başına çözülen çıktı 1 MiB kalır.
- Ekleme var olan hedefi artık genel 16 MiB okuma sınırıyla değil, kilit
  altında önceden denetlenen bu tavanla okur (aksi hâlde 16 MiB üstü hedefe
  ekleme okuma sınırına çarpıyordu).
- Yükleme, dosya yanıtı ve dosya özeti tavanları 16 MiB kalır: 18 MB'lık ek
  dosya yanıtıyla indirilemez ve özetlenemez; ürün tutamağın `sha256`
  alanını kullanır (açık sınır, ayrı karar).
- DoS gerekçesi GB-166: bellek tepe noktası ekleme başına ~32 MiB (mevcut
  içerik + yeni baytlar), disk proje başına iki katı; tavan ürünün 25 MB
  ileti sınırına göre seçildi.

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
- Bayt sayısı (`uzunluğu` harf sayar), metin↔base64/onaltılık ve kod
  noktası RFC-0043 Dilim 2'dir; ZM-R004 bugün `1` verir (TANIMLI).
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
