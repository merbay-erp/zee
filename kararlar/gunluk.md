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

## K-035 — Özyineleme v0.2 (T035 "temel durum önce")

- **Karar:** İşlem kendini (veya karşılıklı olarak birbirini) çağırabilir.
  Tür çıkarımı "o ana dek görülen dönüşler" üzerinden yapılır: özyinelemeli
  çağrı, temel durum dönüşünden ÖNCE gelirse T035 ("temel durumu özyinelemeli
  çağrıdan önce yaz"). Bu, tür değişkenleri olmadan deterministik çıkarım
  sağlar ve iyi özyineleme alışkanlığını dilin kendisi öğretir.
- **Karar:** Çalışma zamanı derinlik sınırı 5000 → C019 (taşma yerine Türkçe
  tanı). Karşılıklı özyineleme için işlem adları ön-taranır (tanım sırası
  serbest).
- **bulgu:** `verilen özyineleme` ifadesi son birleşimle tutarsızsa T018.

## K-036 — Metin kaçışları ve negatif sabitler

- **Karar:** Metin içinde `\"`, `\\`, `\n` kaçışları; bilinmeyen kaçış S040
  (önerili). Negatif sayı sabitleri (`-3`, `-3,14`) işaret-farkında okunur;
  ondalıkta işaret gövdeye bir kez uygulanır (K-011 bitişik virgül kuralı
  değişmedi).

## K-037 — Akış-duyarlı daraltma (T036, RFC-0008 kapanışı)

- **Karar:** `X varsa` / `X başarılıysa` / `X başarısızsa` dallarında (ve
  `yoksa`/`değilse` tersinmelerinde) `X in değeri` / `X in hatası` erişimi
  statik güvenlidir; dal dışında korumasız erişim T036 derleme hatasıdır.
  C008 artık yalnız iç savunma. Tam veri-akışı analizi YOKTUR — tek koşullu
  `... ise` kolları ve `değilse` tersinmesi kadar dar, o kadar da anlaşılır.
- **Etki:** "boş değeri açmak" hatası çalışma zamanından derleme zamanına
  taşındı; öğrenciye tanı, koşulun Türkçesiyle konuşur ("Önce kontrol et:
  deneme başarılıysa").

## K-038 — Çok-tokenli çağrı argümanları

- **Karar:** `A ile B için işle` çağrısında her "ve/ile" dilimi tam bir ifade
  bölgesi olarak ayrıştırılır (tek token sınırı kalktı): `tabanın tam kısmı
  için yuvarla` yazılabilir.

## K-039 — Playground: derleyici tarayıcıda (Faz 6 erken teslim)

- **Karar:** Derleyici wasm32-unknown-unknown hedefine `--lib` olarak derlenir
  (ADR-001 korunur: wasm-bindgen YOK, elle C-ABI: uzunluk-önekli UTF-8 tampon).
  `playground/olustur.sh` tek dosyalık `zee-playground.html` üretir — dil.wasm
  base64 gömülü, çift tıkla açılır, internet gerekmez.
- **Karar:** PlaygroundIo = ToplayanIo determinizmi + görünür tohum alanı:
  aynı tohum + aynı girdi = her zaman aynı çıktı (bölüm 27 tarayıcıda da
  geçerli). Ağ/sunucu playground'da kapalı (Türkçe hata ile).
- **Doğrulama:** Çekirdek doğal derlemede de testlenir (playground_testi, C-ABI
  gidiş-dönüş dahil); 7 örnek tarayıcıda elle koşuldu (faktöriyel, testler,
  tohum=1 ile kazanılan tahmin oyunu dahil).

## K-040 — İkinci CI dersi: derinlik sınırı doğal yığına sığmalı

- **Olay:** K-035'in 5000'lik C019 sınırı, yorumlayıcının Rust çerçeveleriyle
  çarpılınca 8 MB'lik test yığınlarının TAM KENARINA denk geldi — yerelde kıl
  payı geçti, CI'da üç platformda da doğal yığın taşmasıyla düştü. Windows'ta
  ayrıca Cargo.toml'daki `cdylib` bildirimi bin ile PDB adı çakıştırıp uyarı
  üretti ("uyarı yok" ilkesine aykırı).
- **Ölçüm:** wasm'da taşan doğrusal bellek DEĞİL, tarayıcı motorunun kendi
  çağrı yığını çıktı (V8 ~1 MB, ayarlanamaz): ölçümde ~700 dil-seviyesinde
  taşıyor. En dar platform bu.
- **Karar:** C019 sınırı **500** — V8'e bile ~%40 payla sığar, eğitim dili
  için fazlasıyla derin; sınır her platformda AYNI (determinizm: aynı program
  her yerde aynı sonucu verir). CLI işi 32 MB yığınlı iş parçacığında koşar
  (Windows ana iş parçacığı 1 MB'dır). `cdylib` Cargo.toml'dan çıktı;
  playground `cargo rustc --crate-type cdylib` ile hedefe özgü derlenir ve
  wasm gölge yığını 16 MB'a ayarlanır (`-zstack-size`).
- **İlke:** Dilin verdiği her sınır, dilin KENDİ tanısıyla karşılanmalı —
  altındaki makinenin taşmasıyla değil. Sınır seçerken en dar platform
  (Windows ana iş parçacığı, motorların çağrı yığını) ölçü alınır.
- **Ek:** Debug derlemede çerçeveler platforma göre şişer (ubuntu'da 500
  seviye bile 8 MB test yığınını aşabildi) — sınıra dokunan tek test
  (derinlik_korkulugu) CLI gibi kendi 64 MB yığınını getirir.

## K-041 — Zamir n'si: iyelikli köke hâl eki

- **bulgu:** Proje kitaplığı yazılırken çıktı: `bilgisayarın_zarı` adına
  ayrılma eki gelince Türkçe araya zamir n'si koyar (`zarından`), çözümleyici
  ek listesinde bu biçimler yoktu → A001.
- **Karar:** Ek listesine iyelikli-kök biçimleri eklendi: `ndan/nden`,
  `nda/nde`, `na/ne`, `nı/ni/nu/nü` (K-011 listesi sürümlemeli grammar'ın
  parçası). Yanlış-pozitif adaylar zararsızdır: çözüm kapsamdaki adlara
  bakar, çakışma zaten A002 ile hatadır.
- **Etki:** 154 test değişmeden yeşil; `ayşenin yaşından` gibi doğal
  biçimler artık çalışır. Test: projeler_testi::zamir_nsi_cozulur.

## K-042 — Proje kitaplığı: golden'ın oyun bahçesi

- **Karar:** `projeler/` açıldı (master plan bölüm 16/29 "çocuk proje
  kitaplığı"): golden korpus dilin belgesi, projeler dilin oyun bahçesi.
  İlk beş: çarpım tablosu, hikâye makinesi, quiz, zar oyunu, market hesabı.
  Her proje başında "Öğretilen: ..." satırı + README'de "şunu da dene" fikri.
- **Kural:** kitaplıktaki her proje regression testindedir
  (projeler_testi.rs) — çalışmayan örnek bu depoda barınamaz.
- **Not:** zar oyunu hermetik IO'da deterministiktir ve testi bunu sabitler
  (determinizm sözü örnek katmanında da sınanıyor).

## K-043 — RFC statü kuralı ve toplu geçiş

- **Kural:** Bir RFC ancak (a) yüzeyi gerçeklenmiş VE (b) korpus ya da
  regression testine bağlanmışsa **geçici kabul**e geçer; tam **kabul**
  usability kapısını bekler. Gerçeklenmemiş tasarım taslak kalır.
- **Uygulama (31 Ağu akşamı):** 0002, 0003, 0004, 0005, 0007, 0008, 0009
  (birim katmanı), 0011 (yüzey), 0013 → geçici kabul. 0012 (FFI) taslak —
  Faz 4/5. Sayım: 2 kabul, 10 geçici kabul, 1 taslak.
- **Gerekçe:** "taslak" etiketi, testle sabitlenmiş gerçek davranışı temsil
  etmiyordu; belge gerçeğin gerisine düşüyordu. Statü artık şu soruyu
  yanıtlar: "bu yüzeye güvenerek program yazabilir miyim?" — geçici kabul
  = evet, kırıcı değişiklik ancak sürüm notuyla.

## K-044 — Mantıksal ad tek başına koşuldur

- **bulgu:** Asal avcısı projesinde çıktı: bayrağı sınamanın doğal yolu
  yoktu — `asal doğru ya eşitse` çalışıyor ama eğreti; çocuk `asal ise`
  yazar. Olumsuzu (`bayrak değilse`) zaten vardı: bakışım eksikti.
- **Karar:** `X ise` — X Mantıksal bir ada çözülüyorsa koşuldur; saf koşaç
  ayrı kelime `ise` atom düzeyinde düşürülür. Tür bekçisi Mantıksal olmayanı
  T005 ile reddeder. `değilse` olumsuzlaması bedavaya gelir.
- **Test:** koleksiyon_testi::mantiksal_ad_tek_basina_kosuldur.

## K-045 — Boş koleksiyonun türü ilk eklemeyle somutlaşır

- **bulgu:** Gizli dil projesinde çıktı: `boş sözlük` değer türünü TamSayı
  varsayıyordu — METİN SÖZLÜĞÜ KURULAMIYORDU; `boş liste` de aynıydı.
  Çocuk programlarının yarısı metin koleksiyonu ister.
- **Karar:** `boş liste` / `boş sözlük` "henüz belirsiz" doğar; İLK
  ekleme/atama türü somutlar ve bağlama yazar. Belirsizken okuma (ilki,
  gezme, değeri) DERLEME hatasıdır ("önce öğe ekle" önerili). Boş sabit ile
  somut eş, yeniden atamada iki yönde uzlaşır (T002 değildir). Boş kalan
  ama türü somut listenin ilki C007 olarak yaşar (koşullu ekleme yolu).
- **Etki:** golden'ın bos_liste_ilki testi C007→T014 derleme yükseltmesi
  aldı; 7 yeni koleksiyon testi. Liste öğesi v0'da TamSayı/Ondalık/Metin/
  satır; sözlük değeri TamSayı/Metin (T011/T021 önerili söyler).

## K-046 — Kalan: `X in Y ye bölümünden kalanı`

- **bulgu:** Asal avcısı projesi kalan işlemi olmadan "(n/d)·d = n" hilesine
  mecbur kalmıştı — eksik, korpusla kanıtlandı.
- **Karar:** Okul diliyle: `17 nin 5 e bölümünden kalanı`. Yalnız TamSayı
  (T008; Ondalık için önce `tam kısmı`). Sıfıra kalan C003. Anlam okul
  kuralı: kalan DAİMA 0 ≤ kalan < |bölen| (rem_euclid; −7'nin 3'e kalanı 2).
- **Test:** koleksiyon_testi::kalan_kalibi_okul_kurali; asal projesi artık
  gerçek kalıbı kullanıyor.

## K-047 — Çocuk modu: `dil çalıştır --güvenli`

- **Karar:** Master plan bölüm 16'nın sandbox maddesi: `--güvenli` bayrağıyla
  ağ ve sunucu tamamen kapalı (Türkçe hata), dosya erişimi çalışma klasörüyle
  sınırlı (mutlak yol ve `..` kesilir). Gerçekleme IO katmanında sargıdır
  (GuvenliIo) — dil çekirdeğine tek satır dokunulmadı; sargı HER IO'yu
  sarabilir, testleri hermetiktir.
- **İncelik:** sargının hatası sıradan IO hatasıdır: `... okumayı dene`
  onu Sonuç'a çevirir — çocuk modunda bile hata YÖNETİLEBİLİR kalır.
- **Sınır bilinci:** bu bir süreç-düzeyi hapishane değildir (işletim sistemi
  izolasyonu Faz sonrası); okul senaryosundaki "yanlışlıkla dışarı yazma /
  ağa çıkma" sınıfını kapatır. Playground zaten en katı sandbox'tır.
- **Test:** guvenli_testi.rs (6 test).

## K-048 — Standart kitaplık: RFC-0014 + gömülü prototip

- **Karar:** RFC-0014 yazıldı: çekirdek/kitaplık sınırı (turnusol: "IO'ya ya
  da yeni sözdizimine muhtaç mı?"), adlandırma düzeni (yüklem-sonlu Türkçe
  işlem adları), dağıtım = GÖMÜLÜ kitaplık (include_str — kurulumsuz,
  internetsiz, playground dahil), kararlılık sözleşmesi (testsiz gömülü
  birim olamaz; imza değişikliği sürüm notu ister).
- **Prototip:** kitaplik/matematik.dil (mutlak, üs, tam karekök, obeb-Öklit,
  okek) ve kitaplik/liste_araclari.dil (toplam, uçlar, Ondalık ortalama) —
  zee'yle yazıldı, 7 zee-testi CI'da. Çözüm sırası: önce yerel klasör, yoksa
  gömülü; A010 gömülü adları listeler. Playground'da birimler ilk kez
  çalışır oldu (yükleyici gömülüden beslenir); playground raporu birimden
  miras testleri saymaz.
- **Sınır bilinci:** ADR-008 Stage 1 RESMÎ olarak başlamadı (usability
  kapısı) — bu, RFC'nin yürütülebilir taslağıdır, deneysel etiketlidir.
- **bulgu (ünsüz ikizleşmesi):** "üssü al" kökü çözülemedi (üs→üssü, ss);
  morfolojide ikizleşme geri çevrimi yok — K-011 ek listesi adayı. Geçici
  çözüm: parametre adı "kuvvet".

## K-049 — Ünsüz ikizleşmesi geri çevrimi + `dil belge`

- **Karar (morfoloji):** K-048'in bulgusu kapandı: ek ayıklamadan sonra kök
  ikiz ünsüzle bitiyorsa teklisi de adaydır (üssü→üss→üs, affı→af,
  zammı→zam); sertleşmeyle birleşir (reddi→red→ret, tıbbı→tıp). Aynı kural
  parametre bildiriminin yapısal ayıklamasında da geçerli — matematik
  biriminin doğal API'si geri geldi: `üssü al`.
- **Karar (araç):** `dil belge <birim>` — işlem başlıkları (parametreleriyle)
  ve test sayısı; önce gömülü kitaplık, sonra yerel dosya. RFC-0014 §8.2
  açık sorusu bu asgari biçimle kapandı; zengin belge üretimi ileride.
- **Test:** koleksiyon_testi::ikizlesme_geri_cevrimi,
  kitaplik_testi::birim_ozeti_islemleri_listeler / matematik_ussu_dogal_adla.

## K-050 — zee ile web sitesi: HTML servis edilir

- **Soru (kurucu):** "zee ile şu an web sitesi yapabilir miyiz?" — Cevap:
  EVET, ve kanıtı projeler/mini-site.dil: 3 rotalı, stilli, gömülü
  kitaplıkla hesap yapan gerçek site; tarayıcıda elle + hermetik testte
  doğrulandı.
- **Karar:** GercekIo yanıtı `<` ile başlıyorsa Content-Type text/html
  gönderir (aksi halde text/plain) — dil yüzeyi değişmedi, IO ayrıntısı.
- **Desen:** rota gövdeleri taze ortamda koşar (RFC-0011/K-022 gereği);
  ortak sayfa parçaları İŞLEMLE paylaşılır (`sayfayı giydir`). Bu, "framework
  istemez miyiz" sorusunun da cevabı: çatı, dilin kendi işlem mekanizmasıyla
  zee'nin İÇİNDE yazılır.
- **bulgu (yapısal ayıklama, K-011 ailesi):** parametre bildiriminde
  yumuşama tersine çevrilemez: "içeriği al" → içeriğ (çünkü "dağı al" → dağ
  meşru). Kapsamlı çözümdeki aday mekanizması bunu bilir ama bildirim
  yapısaldır → tuzaksız ad öner (gövde). Usability kitine not.

## K-051 — Web dalgası: istek sözlüğü, yönlendir, html güvenlisi

- **Mandat (kurucu):** "kusursuz bir web uygulaması yapabilecek seviyeye
  getirelim" + "bu dili artık tüm projelerimde kullanmak istiyorum".
- **Karar 1 — örtük `istek`:** rota gövdesinde `istek` adlı
  Sözlük<Metin,Metin> hazırdır: sorgu (?ad=...) ve POST form gövdesi
  (urlencoded, UTF-8 yüzde çözümüyle) birleşir; ayrılmış anahtarlar
  "yol" ve "yöntem". Okuma bilinen kalıp: `isteğin "not" değeri`;
  varlık: `istekte "not" varsa`. Rota dışında `istek` tanımsızdır.
- **Karar 2 — `"/x" adresine yönlendir`:** 303 + Location (S041 biçim
  tanısı). Kaydet-sonrası-yönlendir deseni dile girdi.
- **Karar 3 — `metnin html güvenlisi`:** & < > " ' kaçışlanır. Kullanıcı
  verisini HTML'e gömerken ZORUNLU alışkanlık — panel testinde ham <b>
  sızmadığı doğrulanıyor (XSS koruması testli).
- **Kanıt:** projeler/panel-not-defteri.dil — dosyada saklayan, gizli yollu,
  formlu admin panel; tarayıcıda canlı (form → kaydet → yönlendir → liste)
  ve hermetik tam-döngü testiyle.
- **Sınır bilinci:** çerez/oturum YOK (Faz 5 güvenlik dalgası) — panel
  koruması gizli yol düzeyindedir, yerel/ders kullanımı içindir; eşzamanlı
  istek işleme sıralıdır. Bunlar bilinçli sınırdır, eksik sayılmaz.
- **Mimari:** istek ayrıştırma tek yerde (istek_parcala) — sahte ve gerçek
  sunucu aynı yolu koşar; hermetik test gerçeği temsil eder.

## K-052 — Oturum kapısı: çerez okuma/yazma

- **Mandat (kurucu):** "daha giriş aşamasında; her projemde kullanabileceğim
  ileri seviye bir dil olmalı" — sıradaki sütun gerçek giriş/oturum.
- **Karar:** Dile yalnız IO kapısı girdi (turnusol, RFC-0014): rota
  gövdesinde örtük `çerezler` sözlüğü (Cookie başlığından) ve
  `"oturum" çerezine kimlik yaz` cümlesi (yanıtla Set-Cookie; Path=/;
  HttpOnly). OTURUM MANTIĞININ TAMAMI ZEE'DE: rastgele kimlik üret,
  oturumlar.txt'ye ekle, korumalı rotada dosyadan doğrula
  (projeler/girisli-panel.dil — `işlem oturumu geçerli mi`).
- **Kanıt:** tarayıcıda canlı (çerezsiz /yonet → /giris; yanlış parola
  reddi; girişten sonra korumalı form) + hermetik tam döngü testi
  (sahte çerez reddi dahil). İstek biçimi: "YÖNTEM yol\nçerez a=1; b=2\ngövde".
- **Sınır bilinci (değişmedi):** parola kaynak/dosyada DÜZ metin — hash
  yok (bölüm 13 güvenlik katmanı Faz sonrası); HTTPS yok. Bu panel yerel
  ağ/ders düzeyidir; internete çıkacak sürüm Faz 5 güvenlik dalgasını bekler.
- **bulgu (yumuşama, üçüncü kez):** "kimliği al" → kimliğ. Yapısal
  ayıklamada k→ğ tersinmesi belirsiz (dağı→dağ meşru). Ders kalıcı:
  parametre adını yalın-ünsüzle bitir (anahtar, gövde, kuvvet-yerine-üs
  ikizleşmeden çalışır). Usability kitine eklendi sayılır.

## K-053 — Metin dalgası: gerçek projelerin görünmez temeli

- **Karar:** Altı çekirdek kalıp (harf düzeyi ilk kez): `parçaları`
  (ayraçla bölme; boş ayraç = harflere), `birleşmişi` (Metin listesi +
  ayraç), `X yerine Y değişmişi`, `kırpılmışı`, `harfleri`
  (Liste<Metin>), koşullar `ile başlıyorsa / bitiyorsa`. Tümü Türkçe
  iyelik ritminde; tür bekçileri T022, boş "eski" C004.
- **Kitaplık:** harf erişimi doğunca RFC-0014'ün "bilinçli yok" dediği
  metin yardımcıları geldi: gömülü `metin_araclari` (tersi, ünlü sayımı,
  baş harf) — mantık saf zee.

## K-054 — JSON yazma: `değerin json metni`

- **Karar:** Sözlük/Liste/Metin/sayılar/Mantıksal → JSON metni (anahtar
  sırası korunur — determinizm; Ondalık noktayla serileşir). Dosyaya
  yazmak bedava: `"veri.json" dosyasına sözlüğün json metni yaz`.
  Yeni cümle YOK — turnusol gereği ifade özelliği yetti.

## K-055 — Önekli rota: `"/yazi/" önekli adrese istek geldiğinde`

- **Karar:** Rota bir önekle eşleşebilir; kalan kimlik metin dalgasıyla
  kullanıcı tarafında çıkarılır (`yolun "/yazi/" yerine "" değişmişi`) —
  dile şablon-rota karmaşası girmedi.

## K-056 — Sıralama: `listenin sıralanmışı` — Türk alfabesiyle

- **Karar:** `sıralanmışı` (TamSayı/Ondalık sayısal; Metin TÜRK ALFABESİ
  sırasıyla: a b c ç d e f g ğ h ı i j k l m n o ö p r s ş t u ü v y z —
  TANIMLI, gerçekleme turkce_karsilastir) ve `tersi` (liste ters çevirme).
  Çekirdeğe girdi çünkü kullanıcı katında yazılamazdı (indeks/çıkarma yok) —
  turnusolun "muhtaçlık" koşulu.
- **Not:** kararlı sıralama; eşitler özgün sırada kalır (determinizm).

## K-057 — Tarih farkı: `X ile Y arasındaki günler`

- **Karar:** İşaretli TamSayı: Y − X (X'ten Y'ye). "Doğum günüme kaç gün
  kaldı?" doğal olarak pozitif. İki taraf da Tarih (T029).

## K-058 — Liste üyeliği + CSV yazma

- **Karar:** `sayılarda 5 varsa` — sözlük üyeliğinin yüzeyi listelere
  genişledi (sayı/metin listeleri; tür bekçisi T021). `tablonun csv metni`:
  satır sözlükleri listesi → CSV (başlıklar ilk satırın anahtar sırasından;
  virgül/tırnak/yeni satır RFC 4180 gibi kaçar).

## K-059 — Silme: `sayılardan 5 i sil` / `defterden "elma" yı sil`

- **Karar:** Listeden İLK eşleşen öğe, sözlükten anahtar silinir; **yoksa
  sessizce hiçbir şey olmaz** (TANIMLI — silme idempotenttir; çocuk
  "zaten yoktu" durumunda cezalandırılmaz). Biçim bozuksa S042; tür
  bekçileri T011/T012/T021.
- **Pratik:** girisli-panel'e gerçek silme rotası eklendi — dosya-satırı
  silme SAF ZEE: süz (eşit değilse ekle) + "\n" ile birleştir + dosyaya
  yaz (üzerine). Dil çekirdeği dosya-silme kalıbına muhtaç değil.

## K-060 — Yapı listeleri: kayıt tabloları

- **Karar:** VeriTuru'ya Yapı girdi: `boş liste` + `ekle` yapı öğe türünü
  somutlar (K-045 çıkarımı aynen), gezmede alan erişimi çalışır, farklı
  yapı karışımı T011. `json metni` yapıyı ve yapı listesini nesne olarak
  serileştirir — kayıt tabloları JSON'a tek satırda döner.
- **Not:** Katalogdaki T027 satırı bayattı: Ondalık alan baştan beri
  destekliymiş — belge düzeltildi (para alanları serbest).
- **bulgu (K-011 ailesi, alan çözümü):** alan erişimi araç ekini tanımıyor:
  `kitabın fiyatıyla artır` → T028 ("fiyatıyla" alan sanılıyor). Geçici yol:
  ara ada al. Alan çözücüsüne ek ayıklama genişletmesi aday iş.
- **Test:** koleksiyon_testi (kayıt tablosu + karışım reddi). Ölçüm arşivi
  v0.4.0 satırı: tüm yükler bütçe içinde (en büyük +%12 döngü — izlemede).

## K-061 — Morfoloji: iki katmanlı ek zinciri

- **Karar:** kok_adaylari iki geçişli oldu: iyelik + hâl/araç zinciri
  çözülür — `kitabın fiyatıyla artır` (fiyat+ı+yla) çalışır. Belirsizlik
  güvenliği değişmedi: adaylar yine kapsam/alan adlarıyla eşleşmek zorunda,
  çoklu eşleşme A002/T028.
- **Test:** koleksiyon_testi yapı-listesi senaryosu artık doğal biçimiyle
  (K-060'taki bulgu kapandı).

## K-062 — CSV hücreleri Metin: gerçek tablolar isim taşır

- **Karar:** `dosyasından okunan tablo` satırları Sözlük<Metin,METİN>
  (yeni VeriTuru::MetinSozluk). "Hücreler TamSayı" v0 kısıtı kalktı;
  sayı gereken sütun `değerin sayısı` ile BİLİNÇLİ çevrilir (örtük
  dönüşüm yasağı ilkesiyle uyum). C015 artık yalnız biçim hataları
  (boş dosya / sütun uyuşmazlığı).
- **Korpus revizyonu:** golden 19 yeni anlamla güncellendi (ham → sayısı →
  artır); çıktı değişmedi. `csv metni` her iki satır türünü yazar.

## K-063 — JSON okuma hoşgörüsü: her değer Metin gelir

- **Karar:** `dosyasından okunan veri` artık sayı/true/false/null değerleri
  REDDETMEZ: hepsi Metin gelir (CSV felsefesi, K-062). Sayı noktası dilin
  virgülüne çevrilir ("1.35" → "1,35" — `ondalığı` doğrudan çalışır);
  true/false → doğru/yanlış; null → boş metin. İç içe nesne/dizi hâlâ C016.

## K-064 — Alan adı, özellik kelimesini gölgeleyebilir

- **bulgu:** Envanter projesi: `ürünün adedi` T014 verdi — "adet" alanı,
  `adedi` liste-özelliğine yenildi.
- **Karar:** Nesne YAPI ise ve özellik kelimesi bir alana çözülüyorsa bu
  alan erişimidir — denetleyici ifadeyi AlanErisim'e yeniden yazar (örtük
  çoğul emsali). Alan adları dilin özellik kelimeleriyle çakışabilir;
  çocuk "adet" alanını korkmadan kullanır.

## K-065 — Para biçimi: `tutarın kuruşlusu`

- **Karar:** Daima iki ondalık hane: 1824,5 → "1824,50"; 5 → "5,00";
  3,456 → "3,46" (yarımlar sıfırdan uzağa — dil kuralı). Ondalık ve
  TamSayı üzerinde; sonuç Metin. Envanter projesi geçti.
- **Hijyen kuralı:** biçimleyici testi projeler/ ve kitaplik/'i de kapsar:
  depodaki her .dil hem idempotent hem ZATEN resmi biçimde olmak zorunda.

## K-066 — Evrensel metin hali: `değerin metni`

- **Karar:** Her değerin resmî metin temsili (yaz ile birebir aynı biçim):
  `sayının metni` → "42", `oranın metni` → "3,5", `bayrağın metni` →
  "doğru". `"" ile sayı` hilesine gerek kalmadı. Tek özel durum: `json
  metni` / `csv metni` iki-kelimeli kalıpları önceliklidir (n==3 önce
  denenir); yapılar K-064 gölgelemesiyle "metni" ALANINI da kullanabilir.
- **Ek:** `dil belge` işlem üstündeki # satırlarını açıklama olarak basar;
  matematik biriminin işlemleri örnek açıklamalar aldı. Playground'a
  "Envanter (kayıtlar)" vitrini eklendi (wasm'da doğrulandı: kuruşlu +
  json çıktısı); öğretmen rehberi 12 oturuma çıktı.

## K-067 — Sayısal genişleme çağrıya (ve sözlüğe) uzandı

- **Karar 1:** Sözlük değerleri Ondalık olabilir (para sözlükleri:
  `fiyatların "çay" değeri 45,50`).
- **Karar 2 (çağrı genişlemesi):** TamSayı argüman Ondalık parametreye,
  Liste<TamSayı> argüman Liste<Ondalık> parametreye uyar.
- **Karar 3 (imza terfisi):** dar imza, geniş argüman görünce KALDIRILIP
  gövde geniş türlerle yeniden denetlenir — sonuç, çağrı SIRASINDAN
  bağımsız en geniş imzadır (determinizm). Özyineleme denetimi sürerken
  terfi yok. Bulgu kaynağı: birim testi TamSayı listesiyle imzayı
  kilitleyip Ondalık kullanıcıyı düşürüyordu (monomorfizm × birim testi
  köşesi) — kökten kapandı.
- **Ek:** `tam kısmı`/`yuvarlanmışı` TamSayı üzerinde kimliktir (terfi
  sonrası gövde güvenliği). Kitaplığa `medyanını hesapla` girdi (saf zee,
  gezme-sayma deseni; dönüş daima Ondalık).

## K-068 — Aralık iki yönde: `5 ten 1 e kadar` geri sayar

- **bulgu:** Geri aralık SESSİZCE boş dönüyordu — sessiz hiçlik,
  determinizm ilkesine aykırı bir tuzak sınıfı.
- **Karar:** `<a> dan <b> e kadar` iki yönde çalışır (adım daima 1;
  a>b ise azalarak). Çocuğun roket geri sayımı doğal yazımıyla çalışır.
- **Not (kitaplık sınırı):** `toplamını hesapla` gövdesi `toplam 0 olsun`
  ile TamSayı'ya bağlı — Ondalık listeyle terfi denetimi T006'ya düşer;
  Ondalık toplam şimdilik kullanıcı tarafında üç satırdır (medyan gibi
  hep-Ondalık yapmak TamSayı kullanıcılarını kırardı). RFC-0014 açık
  sorusuna eklendi sayılır.

## K-069 — Çıkış kodu: `programı 1 ile bitir` (K-024 kapanışı)

- **Karar:** K-024'ün adayı gerçek oldu: `programı <kod> ile bitir` —
  kod TamSayı (T034) ve 0–255 (C020); süreç çıkış kodu olur. `programı
  bitir` = kod 0. CLI otomasyonu/betikleri için kapı: `dil çalıştır`
  artık kabuğa anlamlı kod döndürür. Lib: calistir_io_kodla (calistir_io
  aynen korunur).
- **Proje:** roket.dil — geri sayımın (K-068) beş satırlık vitrini.

## K-070 — Kitaplık olgunlaşması: Ondalık toplam + sozluk_araclari

- **Karar (deneysel-kırıcı, sürüm notlu):** `toplamını hesapla` ve
  `ortalamasını hesapla` daima Ondalık döner — K-068 notundaki köşe
  (TamSayı'ya bağlı gövde × terfi) kökten kapandı; TamSayı listeleri
  çağrı genişlemesiyle (K-067) girer. Deneysel etiket tam bu yüzden
  vardı: kırıcı değişiklik erken ve ucuzken yapıldı.
- **Yeni birim:** `sozluk_araclari` — `en çok geçeni bul`
  (Sözlük<Metin,TamSayı>; eşitlikte İLK eklenen kazanır — determinizm).
  Gömülü birim sayısı 4.
- **Doğrulama:** `artır` karışık tipte genişliyor (Ondalık hedef +
  TamSayı miktar ✓) — toplamın tek gövdeyle iki tipe hizmeti bundan.

## K-071 — Doğrulamalar zenginleşti (K-025 adayları gerçek)

- **Karar:** `X P içermeli` (metin içerme doğrulaması), `... olmamalı`
  (HERHANGİ koşulun olumsuz doğrulaması — olmalı'nın simetriği) ve çıplak
  `boş`/`dolu` atomu (`liste boş olmamalı`, `sepet boş olduğu sürece`).
  Hepsi D001 mekanizmasında; usability onay kapısı K-025 ile aynı.
- **Ölçüm:** v0.6.0 arşiv satırı — v0.4'te izlemeye alınan döngü sapması
  KAYBOLDU (5,7 ms; v0.2 tabanının altında). İzleme kapandı.
- **Sayım düzeltmesi:** anti-örnek 11'dir (A11 nokta-ondalık) — README
  10 diyordu.

## K-072 — Morfoloji-farkındalıklı yeniden adlandırma

- **Karar:** dillsp `textDocument/rename` destekler ve EKLERİ YENİDEN
  GİYDİRİR: sayaç→puan denince sayacı→puanı, sayaçla→puanla,
  sayaçtan→puandan. Çekirdek: ileri morfoloji üreteci `ek_uydur`
  (ünlü uyumu dörtlü/ikili, y/n tamponları, sert ünsüz benzeşmesi ta/te,
  çok-hecede p→b, ç→c, t→d, k→ğ yumuşaması, nk→ng her hecede:
  renk→rengi) + çözümleyici `ek_coz` (yüzey ekten soyut ek kimliği;
  yumuşama/ikizleşme/ünlü düşmesi geri çevrimleriyle).
- **Sınırlar (dürüst):** tek katman ekler (fiyatıyla gibi zincirler
  değiştirilmez — dokunulmadan bırakılır), metin sabitleri ve # yorumları
  DOKUNULMAZ (testli), tek-heceli yumuşama istisnaları (top→topu ✓ ama
  hukuk→hukuku gibi istisnalar üretilmez — önizlemede elle düzeltilir).
- **Testler:** üç senaryo (ek seti, yumuşama+nk, ünlü tamponları) + gerçek
  dillsp'ye karşı Node birlikte-çalışma. VS Code istemcisine
  RenameProvider eklendi (F2 çalışır).

## K-073 — Çerez silme: `"oturum" çerezini sil`

- **Karar:** Çerez API'si tamamlandı: silme, tarayıcıya `Max-Age=0`
  başlığıyla gider (boş değer yazma hilesi yerine doğru HTTP). Girişli
  panelin çıkışı gerçek silmeye geçti.
- **Not:** `<sayı> kez tekrarla` sayacının çok-tokenli ifade aldığı
  doğrulandı (`sayıların adedi kez tekrarla` ✓ — zaten çalışıyormuş).

## K-074 — Gezmede yazma listeye yansır (kopya tuzağı kapandı)

- **bulgu:** `her kutu için / kutunun adedi 99 olsun` SESSİZCE kayboluyordu
  (kopya semantiği) — "değiştirdim ama değişmedi" tuzağı.
- **Karar (TANIMLI):** Gezme kaynağı bir ad olduğunda (dilbilgisi gereği
  hep öyledir), döngü değişkenine yapılan değişiklik her turun sonunda
  listedeki öğeye GERİ YAZILIR. Stok güncelleme gibi yerinde-değiştirme
  akışları doğal yazımıyla çalışır. Sözlük gezmesi anahtar verdiğinden
  etkilenmez.
- **not (kalıp sınırı):** AlanAta değeri tek token — çok-tokenli değer ara
  ada alınır (aday iyileştirme).

## K-075 — Türk para yazımı: `binlikli kuruşlusu`

- **Karar:** `tutarın binlikli kuruşlusu` → "1.234.567,89" — binlik ayraç
  NOKTA, ondalık VİRGÜL (Türk yazım kuralı). kuruşlusu ailesinin üstüne.

## K-076 — Proje bir klasördür: `proje.dil`

- **İhtiyaç (kurucu):** zee yalnız örneklerde değil, kurucunun bütün gerçek
  projelerinde kullanılacak; tek dosya akışı profesyonel ölçeğin temeli olamaz.
- **Karar:** Proje kökünde ayrı biçim öğretmeyen, geçerli zee sözdizimli
  `proje.dil`: `proje "ad" olsun`, `sürüm "X.Y.Z" olsun`,
  `giriş "program.dil" olsun`. Üç alan zorunlu/tek; giriş proje dışına
  çıkamaz. P001–P004 tanıları Türkçe ve katalogludur.
- **Araç:** `dil çalıştır/denetle/dene <klasör>` giriş dosyasını bildirimden
  bulur; doğrudan `.dil` yolu geriye uyumludur. `dil yeni` bildirimi ve
  klasör-temelli komutları hazır verir. zee deposunun kökü de kendi
  `proje.dil`ini taşır: dil, kendisini proje olarak tanır.
- **Yan bulgu:** `dil çalıştır --güvenli ...` bayrağı eski argüman hesabında
  programa kaynak yolunu sızdırıyordu; kaynak-sonrası ayrımıyla kapandı.
  Gerçek dosya IO'su da kabuğun çağrıldığı klasöre bağlıydı; artık göreli
  yollar giriş dosyasının klasöründen çözülür. Proje nereden çağrılırsa
  çağrılsın kendi verisini kendi yanında tutar.
- **Kanıt:** 7 proje entegrasyon testi; birim kullanan proje uçtan uca
  çalıştır/denetle/dene; güvenli argüman ve iskelet üretimi regression'da.
- **Sonraki katman:** yerel bağımlılık çözümü + deterministik kilit dosyası;
  bunun üstüne registry/provenance gelir. Kök sırayı atlamaz.

## K-077 — Proje çapında güvenli biçimleme

- **Karar:** `dil biçimle <proje-klasörü>` bütün `.dil` kaynaklarını
  deterministik yol sırasında toplar. Gizli klasör, `target`/`hedef` ve
  sembolik bağ izlenmez.
- **Bütünlük sözü:** Önce her kaynak bellekte biçimlenir; bir tanesi bile
  hatalıysa HİÇBİR dosyaya yazılmaz. Böylece büyük projede yarım kalmış
  biçimleme durumu doğmaz.
- **Kanıt:** alt klasörde birim + giriş + bildirim birlikte biçimlenir;
  üç dosyanın resmî çıktısı entegrasyon testinde sabittir.

## K-078 — Yerel paket, kaynak kökeni ve deterministik kilit

- **İhtiyaç:** Kurucu zee'yi bütün gerçek projelerinde kullanacak. Kod paylaşımı
  kopyala-yapıştır ya da kabuğun çalışma klasörüne bağlı dosya araması olamaz;
  aynı kaynak grafiği her makinede aynı anlama gelmeli.
- **Bildirim:** `yerel_bağımlılıklar "../hesap" listesi olsun`. Yalnız yol
  bildirilir; paket adı ve sürümü bağımlı projenin `proje.dil` dosyasından gelir.
  Böylece iki gerçek kaynağın zamanla ayrışması engellenir.
- **Dil yüzeyi:** `hesap paketini kullan`. `birimini` aynı kaynak klasörünü,
  `paketini` yalnız proje sahibinin doğrudan bağımlılığını anlatır. Geçişli
  paket grafikte bulunur ama açıkça bildirilmeden API sayılmaz (A011).
- **Köken:** Yükleyici artık yalnız metin döndürmez; yüklenen kaynağın kararlı
  kimliğini de taşır. Paket içindeki `yardimci birimini kullan`, uygulamanın
  değil paketin kendi klasöründen çözülür. Eski gömülü/sahte yükleyici API'si
  geriye uyumluluk için korunur.
- **Kilit:** `dil kilitle` bütün geçişli grafiği ada göre sıralı yazar: sürüm,
  ana projeye göre göreli yol, doğrudan kenarlar ve bütün gerçek `.dil`
  kaynaklarının SHA-256 özeti. Mutlak makine yolu yoktur. Eksik/bayat kilit
  P008 ile durur; sessiz güncelleme yapılmaz.
- **Güvenlik/bütünlük:** Bağımlılık döngüsü, aynı adlı ayrı kök, geçersiz paket
  adı, proje dışına çıkan giriş/birim sembolik bağı ve gizli/hedef klasör
  geçişi reddedilir. Kilit doğrulanan bellek görüntüsü derlenir; TOCTOU için
  kaynak ikinci kez okunmaz.
- **Araçlar:** `dil yeni` boş bağımlılık listesi ve ilk kilidi üretir. CLI ile
  dillsp aynı proje grafiğini kullanır. P005–P009 ve A011 katalogludur.
- **Kanıt:** geçişli paket + paket içi birim uçtan uca çalışır; paketin üst
  düzey cümlesi kapsüllenir; içerik değişince kilit bayatlar; yeniden kilit
  deterministiktir; geçişli pakete doğrudan erişim ve bildirim döngüsü
  regression testlerinde reddedilir.

## K-079 — Paketi elle değil, doğrulayarak ekle: `dil ekle`

- **Sorun:** K-078 güvenli bir çalışma grafiği kurdu ama kullanıcı listeyi
  elle düzenlemek, sonra ayrıca kilitlemek zorundaydı. Yazım hatası, yinelenen
  yol ve yarım işlem profesyonel araç sözleşmesine aykırıydı.
- **Karar:** `dil ekle <yerel-yol> [proje]`. Paket yolu çağıran kabuğa göre
  çözülür; bildirimde proje köküne göre göreli, `/` ayraçlı saklanır.
- **Doğrulama-önce:** Yeni manifest yalnız bellekte üretilir; bütün geçişli
  grafik döngü, ad, giriş ve kaynak sınırlarıyla çözülmeden tek bayt yazılmaz.
- **Koruma:** Var olan yorumlar ve alanlar kalır; bağımlılık yolları sıralanır
  ve tekilleştirilir. Aynı kanonik paket farklı yol yazımıyla verilirse ikinci
  kayıt açılmaz. Projenin kendisini eklemek açık P007'dir.
- **Bütünlük:** Başarıda manifest ve kilit güncellenir. Kilit yazımı başarısız
  olursa eski manifest ve kilit geri yüklenir; başarısız aday çözümde ikisi de
  byte-byte aynı kalır.
- **Kanıt:** CLI entegrasyonu yorum koruma, çalışır paket, idempotent tekrar,
  öz-bağımlılık reddi ve hata sonrası manifest/kilit değişmezliğini sınar.

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
