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
- **Programlar:** tümü.

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
