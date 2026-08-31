# Syntax karar günlüğü

Golden korpus yazılırken alınan sözdizimi kararları. **Hiçbiri kesin değildir** —
kesinleşme ilgili RFC ile olur. Her kayıt: karar, gerekçe, durum, ilgili programlar.

Durum etiketleri: `geçici` (korpusta kullanılıyor, RFC bekliyor) · `AÇIK` (henüz
karar verilemedi, korpusta işaretli) · `bulgu` (korpusun ortaya çıkardığı sorun).

---

## K-001 — Yorum işareti: `#`

- **Karar:** Satır yorumu `#` ile başlar.
- **Gerekçe:** Girinti tabanlı dillerde yerleşik, tek karakter, blok yorum
  karmaşası yok. "Noktalama minimum" ilkesine tek istisna olarak kabul edilebilir;
  yorum işaretsiz olamaz.
- **Durum:** geçici. Alternatifler: `--`, `not:` (K-014'teki "not" çakışması nedeniyle elendi).
- **Programlar:** tümü.

## K-002 — Nesne→eylem akışı: `"..." yaz`

- **Karar:** Eylem sona gelir (yüklem-sonlu): `"Dünyaya merhaba" yaz`, `sayıyı yaz`.
- **Gerekçe:** Türkçe söz dizimi; manifesto maddesi 2.
- **Durum:** geçici (fiilen değişmez ilke).
- **Programlar:** 01 ve tümü.

## K-003 — Değer bağlama: `<ad> <ifade> olsun`

- **Karar:** `isim "Ayşe" olsun`. Yeniden atama da şimdilik aynı kalıp
  (07'de `bildi doğru olsun` mevcut değişkeni günceller).
- **Gerekçe:** Master plan bölüm 6/7. Çocuğun sesli okuyabileceği kalıp.
- **AÇIK alt soru:** ilk tanım ile yeniden atama ayrışmalı mı? (gölgeleme,
  sabitler — `hep ... olsun`? `değişmez ... olsun`?) → RFC-0004.
- **Programlar:** 02, 07.

## K-004 — Metin birleştirme: `ile`

- **Karar:** `isim ile " yaşında" yaz` — `ile` değerleri yazım/birleştirme için zincirler.
- **Gerekçe:** Master plan bölüm 6. `+` sembolü gerekmez.
- **AÇIK alt soru:** `ile` aynı zamanda K-008'de aritmetikte ("a ile b nin toplamı")
  ve K-016'da argüman geçişinde kullanılıyor. Üç rol tek kelimede — belirsizlik
  analizi gerekli. Parser testine en erken girecek konulardan.
- **Programlar:** 02, 04, 08, 24.

## K-005 — Koşul: `... ise / değilse`, zincir `değilse ... ise`

- **Karar:** Koşul eki `-se/-sa` yüzeyde `ise` ya da bitişik (`çiftse`, `açıksa`,
  `varsa`, `boşsa`, `başarılıysa`, `içeriyorsa`). else-if zinciri: `değilse <koşul> ise`.
- **Gerekçe:** Master plan bölüm 6/7; `elif` benzeri ayrı kelimeye gerek kalmıyor.
- **Durum:** geçici.
- **Programlar:** 05, 07, 15, 16, 23, 29.

## K-006 — Döngüler: `kez tekrarla`, `her ... için`, `olduğu sürece`, `olana kadar`

- **Karar:**
  - Sayılı: `10 kez tekrarla`
  - Aralık: `1 den 100 e kadar her sayı için`
  - Koleksiyon: `her sayı için` (bkz. K-013)
  - while: `<koşul> olduğu sürece`
  - until: `<koşul> olana kadar tekrarla`
- **Gerekçe:** Master plan bölüm 6/7; while/until'in Türkçe doğal karşılıkları.
- **Durum:** geçici. while ve until'den yalnız biri çekirdekte kalabilir → RFC.
- **Programlar:** 06, 07, 08.

## K-007 — Girdi: `"..." diye sor` + örtük `yanıt`

- **Karar:** `"Adın ne?" diye sor` sorar; son cevap `yanıt` adıyla erişilir.
- **Gerekçe:** Scratch'in "sor ve bekle / yanıt" modeli çocuklarda kanıtlı;
  deterministik (son sor'un değeri).
- **bulgu:** `yanıt` adı HTTP tarafında da doğal ("adresinden gelen yanıt", 24).
  Korpusta çakışmayı görünür kılmak için 24'te `cevap` kullanıldı. Çözüm
  adayları: girdide `cevap`ı örtük ad yapmak, ya da örtük ad yerine
  `ad "Adın ne?" sorusunun yanıtı olsun` ifade biçimi. → RFC-0006 kapsamı.
- **Programlar:** 03, 04, 07, 24.

## K-008 — Aritmetik: genitif kalıp `a ile b nin toplamı`

- **Karar:** `toplam birinci ile ikincinin toplamı olsun`,
  `bölüm birincinin ikinciye bölümü olsun`. Değiştirme eylemleri:
  `toplamı sayıyla artır`, `sayacı 1 azalt`, `sonucu toplamı adede böl`.
- **Gerekçe:** Master plan bölüm 6 örnekleri; sembolsüz aritmetik hedefi.
- **AÇIK alt soru:** Karmaşık ifadelerde okunabilirlik hızla düşer
  ("a ile b nin toplamının c ye bölümü"). Profesyonel katman için `+ - * /`
  sembolleri opsiyonel mi olacak? Manifesto "başlamak zorunda kalmamalı" diyor,
  yasak demiyor → RFC.
- **Programlar:** 04, 09, 12, 30.

## K-009 — Tür dönüşümü: `yanıtın sayısı`

- **Karar:** Metinden sayıya: `<metnin> sayısı`. Ters yön otomatik (yaz bağlamında).
- **AÇIK:** dönüşüm başarısız olursa? Muhtemel cevap: `Sonuç` döner ya da
  `sayısını almayı dene` gerekir. Tehlikeli implicit conversion yasağıyla
  (bölüm 8) çelişmemeli → RFC-0007.
- **Programlar:** 04, 07.

## K-010 — Karşılaştırmalar

- **Karar:** `90 veya daha büyükse` (>=), `gizliden küçükse` (<),
  `0 dan büyükse` (>), `16 ya eşitse` (==), eşit değil: `eşit değilse`.
- **Gerekçe:** Sembolsüz, sesli okunabilir.
- **Durum:** geçici. "veya daha büyükse" uzun; kısa biçim araştırılacak
  ("en az 90 ise"? "90'dan aşağı değilse"?).
- **Programlar:** 05, 06, 07, 09, 14, 30.

## K-011 — Ek yazımı (ortografi)

- **Karar:** Tanımlayıcılarda ekler bitişik yazılır (`sayıyı`, `toplamı`,
  `geçenlere`, `ayşenin`). Sayı sabitlerinde ve tırnaklı sabitlerde ekler ayrı
  kelime olarak yazılır (`1 den 100 e`, `10 un`, `0 a eşit`).
- **Gerekçe:** Master plan bölüm 6 ile tutarlı ("10 un toplamı", "sayıyı yaz").
  Kesme işareti (`10'un`) noktalama-minimum ilkesine aykırı ve klavyede yavaş.
- **bulgu:** Ek bitişince tanımlayıcı çözümü morfolojik analiz ister:
  `sayıyı` → `sayı` + `-yı`. Lexer değil, ad çözümleme katmanının işi olmalı;
  desteklenen ek listesi sürümlemeli grammar'da sabitlenmeli (bölüm 4).
  → RFC-0002'nin en kritik konusu.
- **bulgu (31 Ağu, y-tamponu belirsizliği):** "payı" hem pa+yı hem pay+ı
  okunabilir; kapsamsız yapısal ayıklamada (işlem parametre bildirimi) bu
  KÖKTEN belirsizdir. Kapsamlı çözümde aday-kök eşlemesi sorunu zaten çözer
  ("payın" tek adaylı: pay). v0 kuralı: parametre bildirimi tampon biçimi
  yeğler (sayıyı→sayı); y ile biten kök adlar (pay, boy, köy) parametre
  bildiriminde belirsizliğe düşer — RFC-0006 adayı: yalın bildirime izin
  (`pay al` gibi) ya da sözlükçe. Şimdilik: net gövdeli adlar öner.
- **Programlar:** tümü.

## K-026 — Homoglyph ve birleştirici im reddi (31 Ağu, v0 gerçeklemesi)

- **Karar:** Tanımlayıcı alfabesi ASCII + Türkçe harfler + şapkalı ünlüler
  (â î û — "kâr"). Başka alfabeden harf → S028 (kod noktasıyla); birleştirici
  im (U+0300–U+036F) → S029 ("birleşik karakteri kullan"). A08 anti-örneği
  artık makine doğrulamalı.
- **Gerekçe:** Determinizm + supply-chain güvenliği (bölüm 18); "görünüşte
  özdeş, gerçekte farklı" tanımlayıcı sınıfı kökten kapandı. Tam NFC yerine
  daha katı "birleşik biçim zorunlu" kuralı: tablo gerektirmez, v0'da yeterli.
- **Durum:** geçici — RFC-0002 §2'de belgelendi.

## K-027 — Mantıksal bağlaçlar: `ve` / `veya` / `değilse` (31 Ağu, v0 gerçeklemesi)

- **Karar:** Koşullar `ve` ya da `veya` ile zincirlenir (kısa devreli);
  olumsuzlama yüklem sonundaki `değilse` ile yapılır (`x 5 e eşit değilse`,
  `bildi doğru değilse`, `bayrak değilse`). **ve/veya karışımı hatadır (S030)**:
  parantez olmadığı için öncelik belirsiz kalırdı; determinizm ilkesi gereği
  kullanıcı koşulu böler ya da tek tür bağlaç kullanır.
- **Bulgu:** `veya daha` ikilisi karşılaştırma kalıbına aittir
  ("90 veya daha büyükse") — zincir ayracı sayılmaz; ayrıştırıcı bir token
  ileri bakarak ayırt eder. A03 anti-örneğinin "doğrusu" bölümü artık
  birebir çalışıyor ve regression testinde.
- **Durum:** geçici — RFC-0005'te belgelendi.

## K-028 — Ondalık sayılar (31 Ağu, RFC-0013 gerçeklemesi)

- **Karar:** Türkçe ondalık virgülü bitişiklik kuralıyla dile girdi:
  `3,14` (bitişik) ondalık sabit, `3, 14` (boşluklu) liste ayracı. Ara durum
  `3 ,14` S033 öğretici tanısı alır; `3.14` yazana S001 önerisi virgülü
  gösterir. Tür onluk TAM değerlidir: 0,1+0,2 tam olarak 0,3 (float sürprizi
  yok). Bölme 9 haneye, yarımlar sıfırdan uzağa (okul kuralı). TamSayı→Ondalık
  genişlemesi kayıpsız olduğundan örtük serbest; tersi açık kalıp ister
  (`tam kısmı`, `yuvarlanmışı`). TamSayı hedefe ondalık artış T006.
- **Gerekçe ve ayrıntı:** RFC-0013. Biçimleyici ondalık tokeni bölmez ve
  liste virgülünden sonra hep boşluk basar — iki kullanım görsel olarak ayrışır.
- **Durum:** geçici (RFC taslak; yüzey gerçeklendi, 18 testle sabitlendi).

## K-029 — Birim sistemi: `X birimini kullan` (31 Ağu, RFC-0009 gerçeklemesi)

- **Karar:** Dosya = birim; `hesaplar birimini kullan` aynı klasördeki
  hesaplar.dil'i alır. Birimin işlem/yapı/test tanımları görünür olur; üst
  düzey cümleleri İÇE ALINMAZ (kapsülleme — dosya kendi başına da
  çalıştırılabilir kalır). Çakışma sessiz gölgelenmez (A008, iki kaynak da
  söylenir); döngü A009; bulunamayan birim A010. `dil dene` birim testlerini
  "birim: test adı" önekiyle birlikte koşar.
- **Mimari not:** Çağrı tanıma işlem adlarına dayandığından birimler tam
  ayrıştırmadan ÖNCE yüklenir (ön tarama + tohumlu ayrıştırma). Birim
  yükleme de IO soyutlamasındadır: testler sahte tabloyla, CLI gerçek
  dosyayla — determinizm korunur. Arama yolu TEK: ana dosyanın klasörü.
- **Durum:** geçici — RFC-0009 §2 gerçeklendi; paket katmanı (manifest,
  registry) Faz 3/5'te.

## K-030 — Kullanıcı işlemlerinden Sonuç dönüşü (31 Ağu, RFC-0008 §4.1)

- **Karar:** `"sıfıra bölünmez" hatasını döndür` işlemi Sonuç-hata ile bitirir.
  Dönüş birleşimi: değer T + hata → Sonuç<T>; başarı dallarındaki `döndür`ler
  çözümleyicide işaretlenir ve çalışma zamanında otomatik Sonuç'a sarılır —
  kullanıcı sarmalama diye bir kavram öğrenmez. Sonuç türü değer-parametreli
  oldu (Sonuç<TamSayı>, Sonuç<Ondalık>...); hata tarafı v0'da Metin. Yalnız
  hata döndüren işlem T018; hata+yok karışımı T018; mesaj Metin değilse T032.
  Sonuç'u geçiren işlem çift sarılmaz.
- **bulgu:** "böl" ile biten işlem adları cümle konumunda BolVeAta ile çakışır
  (ifade konumunda sorun yok — çağrı kalıbı önce denenir). RFC-0006 kelime
  kuralına aday: işlem adı `böl`/`al` ile bitmesin.
- **Durum:** geçici — RFC-0008'de belgelendi, 10 testle sabit.

## K-031 — Korpus tamamlandı ve genişledi (31 Ağu akşamı)

- **Durum:** 30 golden programın 30'u da bootstrap zincirinden geçip
  çalışıyor; korpus 32'ye genişledi (31 birimler, 32 ondalık market) ve
  A11 (nokta-ondalık) anti-örneği eklendi.
- **Revizyon:** golden 26, RFC-0011'in K-023 çözümüne göre yeniden yazıldı
  (görev bağlamaları + hepsini bekle; bekle-öncesi erişim T033).
- **v0 yaklaşımları (dürüst kayıt):** zaman aşımı erken iptal etmez, geç
  kalmayı bildirir (RFC-0011 §3 notu); eşzamanlı blok tek iş parçacıklı
  modelde sıralı yürür (gözlemsel eşdeğer, RFC-0011 §4); https yok (C018
  tanısı yönlendirir); sunucu testte istek kuyruğuyla hermetik.
- **bulgu:** birim adında tire kullanılamıyor (tanımlayıcı kuralı) —
  "hesap-araclari" → "hesap_araclari". RFC-0009 dosya adı kuralına not.

## K-032 — Devredilen üç karar (31 Ağu akşamı, kurucu yetki devriyle)

- **Dil adı:** zee (ADR-009). Uzantı .dil ve CLI `dil` korunur; çakışma
  taraması yapıldı (hobi deneyleri dışında temiz), TÜRKPATENT tescili hukuk
  incelemesine not edildi.
- **K-016 işlem çağrısı:** Seçenek A (için/ile) GEÇİCİ KABUL — 5+ golden
  programlık gerçekleme deneyimi, yüklem-sonlu ritimle uyum ve sıfır ek
  noktalama gerekçesiyle. Usability oturumları onay kapısıdır; protokol
  docs/usability-kiti.md'de hazır. Oturumlar B'yi (sonuç-bağlama) güçlü
  gösterirse RFC-0006 revize edilir — korpus etkisi bölümü bunu zaten planlıyor.
- **Uzak depo:** GitHub'a özel (private) depo olarak itilir — CI üç platformda
  koşar, v0.1'in son kriteri kapanır. Herkese açma kararı kurucuya kalır
  (lisans seçimi — bölüm 26 — henüz yapılmadı; açmadan önce LİSANS şart).

## K-033 — İlk CI dersi: araç zinciri sabitlenir (31 Ağu akşamı)

- **Olay:** İlk push'ta CI üç platformda kırıldı — CI'daki gezici "stable"
  Rust (1.98) yereldekinden (1.93) yeniydi ve yeni clippy lint'i (while_let_loop)
  ile farklı davranış getirdi. Reproducible-build ilkesinin (bölüm 27) tam da
  öngördüğü tuzak, ilk gün yaşandı ve kapatıldı.
- **Karar:** `compiler/rust-toolchain.toml` sürümü sabitler (1.93.1); yükseltme
  bilinçli commit'tir. CI "uyarı yok" adımı kabuk tuhaflıklarından bağımsız,
  çıktısı görünür biçimde yeniden yazıldı. Clippy'nin bulduğu lint haklıydı
  ve düzeltildi (dillsp while let).

## K-034 — Blok kapsamı kararı (v0.2, RFC-0004 kapanışı)

- **Karar:** Gövdede doğan ad gövdeyle ölür; döngü değişkeni dahil. Dıştaki
  ada atama kalıcıdır. Gölgeleme yapısal olarak yoktur (içerde aynı adla
  "olsun" = dıştakine atama; tür bekçisi T002 aynen çalışır). Çözümleyici ve
  yorumlayıcı birebir aynı kuralı uygular (kapsam_baslat/kapsam_bitir).
- **Etki:** 32 golden programın hiçbiri sızıntıya dayanmıyordu — korpus
  değişmeden yeşil kaldı; RFC-0004 §Kapsam bölümü karara güncellenecek.

## K-012 — Liste sabiti: `3, 7, 1, 9 listesi`

- **Karar:** Virgülle ayrılmış değerler + `listesi`. Boş: `boş liste`.
- **Gerekçe:** Köşeli parantezsiz; sesli okunabilir. Virgül kabul edilen
  minimum noktalamadan.
- **Programlar:** 08, 09, 12, 14, 15.

## K-013 — Çoğuldan tekile gezinme

- **Karar:** `her sayı için` — gezilecek koleksiyon, döngü değişkeninin çoğulu
  olan `sayılar` olarak çözülür. Ad uymuyorsa açık biçim: `tablodaki her satır için`,
  `yaşlardaki her ad için`.
- **Gerekçe:** Türkçenin çoğul ekinden gelen doğal bir imkân; master plan
  bölüm 6 örtük biçimi kullanıyor.
- **AÇIK:** Kapsamda birden çok çoğul aday varsa çözüm kuralı ne? Muhtemelen
  derleyici hatası ("hangisini kastettiğin belirsiz") → determinizm korunur.
- **Programlar:** 08, 09, 10, 12, 17, 19, 28.

## K-014 — Ayrılmış kelime çakışmaları (bulgu)

- **bulgu 1:** `not` — hem yaygın tanımlayıcı (09, 19) hem olası mantıksal
  değilleme kelimesi. Karar: değilleme için `değil` kullanılır, `not` serbest kalır.
- **bulgu 2:** `al` — parametre bildirimi (`sayıları al`) ile işlem adlarında
  geçen `al` ("karesini al") çakışır. 30'da işlem adı bu yüzden
  "karesini hesapla" seçildi. Parametre bildirimine ayrı kelime
  (`sayıları alır`? `girdi: sayılar`?) düşünülebilir → RFC-0006.
- **bulgu 3:** `sonuç` — hem `sonucu döndür` kalıbı hem `Sonuç<T,Hata>` tür adı
  hem 16'daki değişken adı. Büyük/küçük harf ayrımı (tür `Sonuç`, değer `sonuç`)
  yeterli mi? Case-sensitivity kararına bağlı (master plan bölüm 36).
- **Durum:** AÇIK — ayrılmış kelime listesi RFC-0002'de sabitlenecek.

## K-015 — Sözlük/alan erişimi: `X in Y değeri`

- **Karar:** Okuma: `yaşların "Ayşe" değeri`. Yazma: `yaşların "Ayşe" değeri 10 olsun`.
  Üyelik: `yaşlarda "Ayşe" varsa`. CSV satırı ve JSON alanı aynı kalıbı kullanır.
- **bulgu:** İç içe erişim uzuyor: `kişinin adres değerinin şehir değeri`.
  Yapılarda (22) iyelik eki daha kısa: `ayşenin adı`. JSON'u yapıya bağlayınca
  (typed parse) sorun küçülür → stdlib tasarımına not.
- **Programlar:** 10, 19, 20.

## K-016 — İşlem çağrısı sözdizimi (AÇIK — en kritik açık karar)

- **Sorun:** İşlem adları eylem cümlesi ("ortalamayı hesapla"). Değer döndüren
  çağrıyı ifade konumunda yazmak gerekiyor.
- **Korpustaki geçici biçim:** `ortalama notlar için ortalamayı hesapla olsun`
  (argüman `için` ile), çok argüman: `"Ayşe" ve 10 ile selamla`.
- **Sorunlar:** `için` döngüdeki `her ... için` ile, `ile` K-004/K-008 ile
  çakışıyor; okunuş tartışmalı.
- **Alternatifler (RFC-0006'da karşılaştırılacak):**
  1. Parantezli izin: `ortalama (notlar için ortalamayı hesapla) olsun`
  2. Sonuç-bağlama biçimi: `notlar için ortalamayı hesapla, sonucu ortalama olsun`
  3. İsimlendirilmiş argümanlar: `ortalamayı hesapla: sayılar notlar olsun ...`
- **Programlar:** 12, 13, 14, 15, 30.
- **Güncelleme (31 Ağu 2026, v0 gerçeklemesi):** Geçici biçim bootstrap parser'da
  gerçeklendi ve golden 12/13/14 çalışıyor. Uygulamada öğrenilenler:
  - Çağrı tanıma "satır, tanımlı bir işlem adıyla bitiyor mu?" kuralıyla
    deterministik oluyor; bu yüzden **işlem çağrıdan önce tanımlanmalı** (v0 kuralı).
  - En uzun işlem adı önce eşlenir; tanımlı bir işlem adıyla biten ama ayraçsız
    bölge hata verir (S019) — sessiz yanlış yorum yok.
  - v0 monomorfizmi: işlem gövdesi İLK çağrının argüman türleriyle denetlenir,
    imza sabitlenir; sonraki çağrılar imzaya uymalı (T017). Özyineleme v0'da
    yok (T016). RFC-0006 bu kısıtları da ele almalı.

## K-017 — Seçenek türü yüzeyi: `var/yok`

- **Karar:** Yokluk sabiti `yok` (`yok döndür`); kontrol `varsa`/`yoksa`;
  içteki değer `bulunanın değeri`.
- **Gerekçe:** Türkçede var/yok ikilisi null kavramından çok daha doğal.
  "boş" kelimesi koleksiyonlara ayrıldı (`boş liste`, `argümanlar boşsa`).
- **Programlar:** 15.

## K-018 — Sonuç türü yüzeyi: `...meyi dene`

- **Karar:** `"veriler.txt" dosyasını okumayı dene` → `Sonuç` döner.
  Kontrol: `başarılıysa`; erişim: `sonucun değeri` / `sonucun hatası`.
- **AÇIK:** `dene`siz düz biçim (17'deki `dosyasının satırları`) hata anında
  ne yapar? Aday: derleyici `Sonuç` beklenmeyen bağlamda `dene` ister
  (Rust'taki `?` disiplinine benzer ama görünür Türkçe kelimeyle).
- **Programlar:** 16, 17.

## K-019 — Hedefli yazma: `X dosyasına ... yaz`

- **Karar:** `yaz` hedef alabilir: hedefsiz → ekran; `"günlük.txt" dosyasına ... yaz`;
  sona ekleme: `... ekle`. Aynı kalıp ileride `panoya yaz`, `günlüğe yaz` gibi genişler.
- **Programlar:** 18.

## K-020 — Yapı alan erişimi: iyelik eki

- **Karar:** `ayşenin adı`, `ayşenin yaşı 10 olsun`. Nokta operatörü yok.
- **AÇIK:** Zincirlenme: `öğrencinin okulunun adı` — kaç seviye doğal kalır?
- **Programlar:** 22, 26.
- **bulgu (31 Ağu, v0 gerçeklemesi):** Alan-atama deseni `-in` ile doğal biten
  adlarla çakışıyor: `tahmin yanıtın sayısı olsun` satırında "tahmin" tamlayan
  ekli sanılabiliyor. v0 çözümü: kuyruk yapılı bir kalıpsa (örn. "yanıtın
  sayısı") normal değer tanımı kazanır; kalıplar alan-atamadan önce denenir.
  Deterministik ama incelikli — RFC-0004/0006 bu öncelik sırasını resmî grammar
  kuralı olarak yazmalı. Ayrıca ünlü düşmesi geri çevrimi morfoloji motoruna
  girdi (şekle→şekil, burnu→burun) — desteklenen ek/çekim listesi artık üç
  mekanizma içeriyor: ek ayıklama, ünsüz yumuşaması geri çevrimi, ünlü düşmesi
  geri çevrimi (K-011 sürümlemeli grammar kapsamına eklenecek).

## K-021 — Desen eşleştirme: `X e göre / ... ise`

- **Karar:** `şekle göre` başlığı altında `"kare" ise` kolları, varsayılan `değilse`.
- **AÇIK:** Yapısal desenler (Seçenek açma, yapı parçalama, aralıklar) bu
  yüzeye nasıl biner? → RFC ayrıca gerekecek (master plan bölüm 36).
- **Programlar:** 23.

## K-022 — Olay blokları: `... geldiğinde`

- **Karar:** `"/durum" adresine istek geldiğinde` — `-diğinde` eki olay
  dinleyicisi tanımlar. IoT tarafında da aynı kalıp beklenir
  ("düğmeye basıldığında").
- **Programlar:** 25.

## K-023 — Eşzamanlılık yüzeyi

- **Karar:** `eşzamanlı olarak` bloğu çocuk görevleri başlatır; `hepsini bekle`
  birleştirir; timeout: `5 saniye içinde ... yetişmezse`.
- **bulgu:** Master planın kendi örneğinde (26) `hepsini bekle` sonrası
  `sonucu döndür` neyi döndürdüğü tanımsız — üç değerin birleşimi nasıl
  adlanıyor? → RFC-0011'in ilk sorusu.
- **Programlar:** 26, 27.

## K-024 — Program kontrolü: `programı bitir`

- **Karar:** Erken çıkış `programı bitir`. Çıkış kodu: `programı 1 ile bitir` (aday).
- **Programlar:** 28.

## K-025 — Test doğrulaması: `... olmalı`

- **Karar:** `test "<açıklama>"` bloğu; doğrulama `kare 16 ya eşit olmalı`.
  `olmalı` kalıbı karşılaştırma kalıplarının (K-010) üstüne biner:
  `küçük olmalı`, `içermeli` (aday), `boş olmamalı` (aday).
- **Programlar:** 30.

---

## Sonraki adım

Korpus 10 öğrenci + 5 profesyonel usability oturumuna (Hafta 12 hedefi, erkeni
Hafta 2'de kağıt üstünde) sesli okutulacak; her kayıt için "doğal mı /
deterministik mi / öğrenilebilir mi / savunulabilir mi" dört soru süzgeci
işletilip durumlar güncellenecek. `AÇIK` kayıtlar ilgili RFC'lere taşınacak.
