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
  (`derinlik_korkulugu`) CLI gibi kendi yığınını getirir. K-129'un kaynak
  muhasebesiyle büyüyen debug gelecekleri için test yığını 128 MiB'a çıkarıldı;
  sürümlü C019/500 davranışı ve üretim profili değişmedi.

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
- **Sınır düzeltmesi (1 Eyl 2026, K-081):** K-067 yalnız saklanan sayısal
  imzayı en geniş tipe ulaştırır; public işlem sözleşmesini çağrı yerlerinden
  bağımsızlaştırmaz. Önceki “çağrı sırasından bağımsız sonuç” cümlesi fazla
  güçlüdür. Açık tür/generic/global kısıt seçimi V1-P0-01'de kapanacaktır.
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
- **Sınırlar (K-072 anı):** Bu aşamada `fiyatıyla` gibi zincirler rename'de
  dokunulmadan kalıyordu; K-089 `Iyelik + dış ek` modeliyle bu sınırı kapattı.
  Metin sabitleri ve # yorumları DOKUNULMAZ (testli). Tek-heceli yumuşama
  istisnaları ile sözlüksel biçimler profilin kanonik üretim sınırındadır.
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

## K-080 — Paket grafiği görünür; kullanılan paket sessizce çıkarılmaz

- **Görünürlük:** `dil paketler [proje]` çözülmüş grafiği doğrudan/geçişli
  ayrımı, sürüm, taşınabilir yol ve SHA-256 kilit özetiyle listeler.
- **Kaldırma:** `dil çıkar <paket> [proje]` yalnız doğrudan bağımlılığı hedefler.
  Ana projenin herhangi bir `.dil` kaynağında `X paketini kullan` kalmışsa
  açık P010 verir; kullanıcı önce kullanımı ve bağlı çağrıları kaldırır.
- **Neden güvenli varsayılan:** Kullanılan paketi bildirimin altından çekip
  sonraki derlemeyi bozmak yerine hata paketin ilk kullanım yerini gösterir.
  Başarısız işlem bildirim ve kilidi byte-byte korur.
- **Bütünlük:** Başarıda aday grafik yazmadan önce doğrulanır; bildirim/kilit
  K-079'un iki dosyalı geri alma yolu ile güncellenir. Doğrudan kenarın
  kaldırılması, başka paketin ihtiyaç duyduğu geçişli düğümü silmez.
- **Kanıt:** Entegrasyon testi doğrudan/geçişli listelemeyi, P010 konumunu,
  başarısız işlem değişmezliğini ve kullanım kaldırıldıktan sonraki başarılı
  çıkarma/kilit yenilemeyi sınar.

## K-081 — Spec bugünün dili; RFC değişikliğin yetkisidir

- **Sorun:** Kaynak denetiminde RFC-0006'nın eski tanım/özyineleme kısıtını,
  RFC-0011'in ise hedef deadline iptalini bugünkü sıralı/geç-ölçümlü runtime'la
  aynı kipte anlattığı doğrulandı. İki belge çatışınca “niyeti tahmin et” bir
  dil için kabul edilemez.
- **Karar:** ADR-010 belge rollerini bağladı. Manifesto anayasa; kabul ADR'si
  mimari sınır; spec geçerli dilin kesin sözleşmesi; RFC değişiklik
  gerekçesi/yetkisi; test yürütülebilir kanıt; gerçekleme bunlara uyar.
  Çelişki sürüm engelidir, sessizce bir taraf seçilmez.
- **v1 disiplini:** `docs/v1-surum-kapilari.md` P0/P1 bulgularını kaynak
  kanıtıyla dondurur. Paket/modül eksikliği K-076–K-080 ile kapalıdır; işlem
  imzası, action/session, atomik durum, deadline, morfoloji, typed hata ve
  registry kapıları açıktır.
- **Web yönü:** RFC-0015 route'u adaptör, tekrar kullanılabilir iş kuralını
  eylem olarak ayırır. Production sözü verilene dek çalışan TCP/çerez yüzeyi
  deneysel ve açık opt-in olmak zorundadır.

## K-082 — Production sözü yoksa gerçek soket güvenli varsayılanla kapalıdır

- **Korkuluk:** `sunucu başlat` gerçek TCP'yi sıradan `dil çalıştır` altında
  artık açmaz; C017 açıkça `--deneysel-web` yolunu gösterir. Opt-in verilirse
  de stderr'de bunun yalnız localhost eğitim/prototipi olduğu yazılır.
- **Örnek güvenliği:** mini-site, panel-not-defteri ve girisli-panel açıkça
  deneysel demo olarak adlandırıldı. Giriş, kaydet, sil ve çıkış mutasyonları
  POST kontrolü taşır; silme bağlantısı GET yerine POST formudur.
- **Sınır:** Bu production web'i tamamlamaz. Method-aware route, action,
  CSPRNG session, CSRF, atomik durum ve transaction V1-P0-02..04'te açıktır.
- **Kanıt:** CLI entegrasyonu opt-in olmadan C017 ve değişken argümana
  sızmayan bayrağı; web entegrasyonu GET silmenin durumu koruduğunu, POST'un
  sildiğini sınar. Toplam 262 test.

## K-083 — Başlangıçta çıkarım, kalıcı API'de açık işlem imzası

- **Yüzey:** Eski `sayıyı al` çocuk/başlangıç akışında değişmez.
  `sayıyı Ondalık olarak al`, `sayıları Ondalık listesi olarak al` public ve
  paket sözleşmesine aşamalı geçiştir. Yapı adı ve kontrollü
  liste/sözlük/Seçenek/Sonuç tür yazımları geçerlidir.
- **Semantik:** Bir işlemde bütün parametreler açık ya da bütünü çıkarımlıdır
  (T037); bilinmeyen yazım T038. Açık gövde hiç çağrılmasa bile tanım
  aşamasında denetlenir; dönüş gövdeden çıkarılıp imzaya bağlanır ve sonraki
  çağrı bu imzayı terfi ettiremez.
- **Tür bütünlüğü:** Açık Ondalık parametreye TamSayı kayıpsız genişleyebilir;
  yalnız checker etiketi değil gerçek runtime değeri de Ondalığa çevrilir.
  Liste/sözlük/Seçenek/Sonuç kapsayıcılarında aynı kural özyinelemelidir.
- **Sınır:** V1-P0-01 bütünüyle kapanmadı. Paket/public işlemlerde zorunluluk,
  açık ABI uyumluluğu ve generic model sıradaki karardır. K-083 bu kararın
  güvenilir çekirdeğini kırmadan ekler.
- **Kanıt:** golden 33; çağrılmayan hatalı gövde; kısmi/bilinmeyen imza;
  yapı/listeler; dar→geniş ve geniş→dar çağrı sırası; runtime tür eşitliği.
  Toplam 270 test ve 122 kataloglu Türkçe tanı.

## K-084 — Kalıcı dosya tek commit, yazarlar tek sıra

- **Karar:** Mevcut `dosyasına ... yaz/ekle` yüzeyi değişmez; resmî gerçek
  runtime tek-dosya güncellemesini RFC-0016/spec-08 sözleşmesiyle atomik yapar.
- **Commit modeli:** Aynı klasörde işletim sistemi süreç kilidi; aynı klasörde
  geçici dosyaya tam yazma + disk eşzamanlama; Unix atomik rename / Windows
  replace+write-through. Okuyucu eski ya da yeni bütünü görür, arasını görmez.
- **Yarış:** `ekle` kilit altında oku-değiştir-replace'tir; iki thread ve iki
  bağımsız zee süreci 80 satırın tamamını korur. Kilit dosyası silinmez; ani
  süreç sonu tanıtıcıyı kapattığında işletim sistemi kilidi bırakır.
- **Hata sınırı:** Replace öncesi hata eski hedefi korur ve geçiciyi temizler.
  Kilit/durability desteği olmayan platform güvenli olmayan fallback yerine
  görünür hata verir. Çok-dosyalı iş transaction'ı RFC-0015'te kalır.
- **Araç zinciri:** `dil biçimle`, paket bildirimi geri-yüklemesi ve
  `proje.kilit` de aynı atomik tek-dosya çekirdeğine taşındı.
- **Kanıt:** 5 kalıcılık birim testi (kısmi okuma karşı-örneği dahil) + iki
  bağımsız CLI süreci regresyonu; toplam 276 test. V1-P0-04 kapandı.

## K-085 — `içinde` geç-kalma raporu değil, sahipli işbirlikli iptaldir

- **Karar:** `N saniye içinde` giriş anı + süreyle mutlak son tarih kurar.
  Runtime bunu blok, cümle, döngü ve kullanıcı işlemi sınırlarına yayar.
- **İptal:** Süre dolunca kalan gövde çalışmaz; blokta doğan adlar düşer ve
  yalnız o deadline'ın `yetişmezse` kolu çalışır. Kol sonrası dış akış sürer.
  Ç001 kullanıcıya sızmayan, benzersiz sahip kimlikli iç nöbetçidir.
- **Bekleme:** `bekle` kalan süreden uzun uyumaz. HTTP DNS sonrası bağlantı,
  yazma ve okuma aşamaları aynı kalan bütçeyi kullanır; geç yanıt değere ya da
  çıktıya çevrilmez.
- **İç içelik:** En erken mutlak tarih kazanır. İç tarih dolarsa iç kol ve dış
  devam; dış tarih dolarsa iptal iç kol tarafından yutulmadan dış kola çıkar.
- **Sınır:** Model işbirliklidir, önleyici değildir. Tek kesintisiz ifade veya
  iptal edilemeyen platform syscall'ı bir sonraki kontrol noktasına taşabilir;
  sonra yeni yan etki başlamaz. Önceden tamamlanan etki rollback edilmez.
- **Kanıt:** geç ağ yanıtı, kırpılan 10→5 saniye bekleme, iptal sonrası yasak
  çıktı, iki yönlü iç/dış sahiplik ve playground sanal saat testleri. 123
  katalog kodu, 279 test; V1-P0-05 kapandı. Gerçek paralel scheduler
  V1-P1-03'te açık.

## K-086 — Public işlem imzası kaynakta tam ve çağrıdan bağımsızdır

- **Progressive disclosure:** Ana programdaki başlangıç işlemi `sayıyı al`
  çıkarımını korur. Birim ya da paket üzerinden dışa çıkan işlem bütün
  parametrelerini `<ad> <Tür> olarak al`, dönüşünü `<Tür> döndürür` veya
  `değer döndürmez` biçiminde yazmak zorundadır (T039).
- **Gövde kanıtı:** Bilinmeyen dönüş T040, bildirim/gövde uyuşmazlığı T041,
  değer bildiren ama olağan bir yoldan sona düşebilen gövde T042'dir. Açık
  işlem çağrılmasa bile denetlenir.
- **Model:** v1 public API bilinçli monomorfik **kaynak ABI**'sidir; ikili
  ABI/FFI sözü değildir. TamSayı→Ondalık genişlemesi çağrı uyarlamasıdır,
  imzayı değiştirmez. Generic yüzey v2+ işidir.
- **Uyumluluk:** Ad/parametre sırası-sayısı-türü/dönüş türü ve public yapı
  alanı değişikliği ana sürüm; aynı imzalı düzeltme yama, yeni işlem/yapı küçük
  sürümdür. Kilidin içerik özeti tekrar üretilebilirliği ayrıca korur.
- **Geçiş:** Dört gömülü standart birim ve golden hesap birimi tam imzaya
  taşındı. Liste genişlemesi, özyineleme, iki çağrı sırası, eksik birim ve
  gerçek yerel paket olumsuzu ve geçişli işlemin örtük yeniden açılmaması
  testlidir. 127 katalog kodu, 289 test;
  V1-P0-01 kapandı.

## K-087 — Rota adaptördür; uygulama kuralı açık imzalı eylemdir

- **Ayrım:** `eylem <ad>` HTTP bilmeyen uygulama iş kuralıdır; bütün girdi
  türlerini ve dönüşünü açık yazar. Aynı çağrı web, CLI, görev ve test
  bağlamında kullanılabilir. Yanıt/yönlendirme/çerez T044; geri alınamayan
  ekran/girdi/donanım etkisi T048 ile eylem sınırından çıkarılır.
- **Yöntem:** Rota `GET|HEAD|POST|PUT|PATCH|DELETE "<yol>" adresine istek
  geldiğinde` biçimindedir. Tarihsel yöntemsiz yazım yalnız GET sayılır.
  GET/HEAD çağrı grafiğindeki dolaylı yazma dahil T045 ile reddedilir. Rota
  dosya/donanım yazmasını açık eylem yerine doğrudan ya da yazıcı normal işlem
  üzerinden yapamaz (T046).
- **Protokol korkulukları:** aynı yöntem+yol T047; yol/yöntem ayrımı 404/405;
  64 KiB gövde ve toplam 100 alan 413; her istek K-085 üstünde 30 saniye son
  tarih taşır ve aşım 504'tür. PUT/PATCH/DELETE gövdesi de çözülür; gerçek HEAD
  gövde göndermez.
- **Transaction:** Her eylem iç içe savepoint'tir. Olağan/başarılı sonuç
  tamamlar; çalışma hatası veya başarısız Sonuç geri alır. Test IO'su dosya
  tablosunu, gerçek IO dokunulan dosyaların eylem başındaki içeriğini saklar;
  desteği olmayan adaptör C021 ile fail-closed davranır. Geri alma K-084 atomik
  replace kullanır.
- **Sınır:** Bu, yorumlayıcı-hatasında çok-dosyalı geri alma sözüdür; süreç
  çökmesinde bütün dosyaların tek kalıcı commit'i değildir. Kimlik/yetki,
  CSRF, güvenli çerez ve TLS/proxy K-088/spec-12'de tamamlandı; idempotency
  ayrı açık kapıdır.
- **Kanıt:** CLI+web ortak eylem, doğrudan/dolaylı GET olumsuzları, POST→eylem
  zorunluluğu, 404/405/413, çalışma hatasında iki dosya rollback'i ve başarısız
  iç savepoint, geri alınamayan etki ve araya giren yazarı ezmeme olumsuzları.
  Gerçek CLI da eski dosyayı geri yükleyip yarım oluşturulanı kaldırır. 134
  katalog kodu, 304 test;
  V1-P0-02 kapandı.

## K-088 — Güvenlik uygulama disiplinine değil, dil/runtime kapısına aittir

- **Rota önsözü:** POST/PUT/PATCH/DELETE ilk satırda `herkese açık`,
  `oturum gerekli` ya da `"rol" yetkisi gerekli` yazar. Public olmak CSRF'yi
  kapatmaz. Hemen ardındaki `"alan" alanı gerekli` cümleleri gövde çalışmadan
  400 üretir; politikasız/dağınık önsöz T049/T050'dir.
- **Kimlik:** Parola yalnız Argon2id PHC ile doğrulanır. 256 bit session ve
  ayrı CSRF işletim sistemi CSPRNG'sinden gelir; depoda session'ın yalnız
  SHA-256 özeti bulunur. Giriş anonim/eski kaydı silip ikisini de döndürür;
  logout/süre dolumu iptal eder. Rol istemciden değil sunucu kaydından gelir.
- **Tarayıcı:** `csrf belirteci` 10 dakikalık anonim form oturumu açabilir;
  unsafe yöntem `_csrf` olmadan 403'tür. Production çerezi `__Host-`, Secure,
  HttpOnly, SameSite=Lax, Path=/ ve sınırlı Max-Age taşır.
- **TLS sınırı:** Runtime sertifika yönetmez; `--web-proxy https://host`
  kipinde yalnız 127.0.0.1 dinler, Host/X-Forwarded-Proto ve unsafe Origin'i
  birebir doğrular. Güvenlik başlıkları ve başlık/gövde-smuggling limitleri
  adaptörde uygulanır.
- **Kanıt:** CSPRNG/Argon2id, rotation/revoke/expiry/rol, eksik-sahte-geçerli
  CSRF, 400/401/403, çerez nitelikleri, CRLF, çift Host ve yanlış proto/Origin
  olumsuzları. `girisli-panel` elle yazılan token/düz paroladan bu profile taşındı.
  RFC-0017 + spec/12; 321 test ve 138 katalog koduyla V1-P0-03 kapandı.

## K-089 — Morfoloji derleyici ayrıntısı değil, sürümlü kaynak anlamıdır

- **Profil:** Zee v1 ek semantiği `zee-tr-1` adıyla sabitlendi. Soyut ekler,
  bütün yüzey biçimleri, iki katman sınırı, ses geri çevrimleri ve kanonik
  üretim `compiler/src/morfoloji.rs` içinde tek kaynaktır. Çözümleyici ve LSP
  artık ayrı liste taşımaz.
- **İyelik ayrımı:** `fiyatıyla` gibi zincirin içindeki `-ı`, tek kullanımdaki
  belirtme ekiyle aynı yüzeyi paylaşsa da ayrı `Iyelik` kimliğidir. Böylece
  ünlü kökte `elma+sı+ndan → elmasından` doğal ve dönüşümlü üretilebilir.
  v1 yalnız `iyelik + hâl/tamlayan/araç` biçiminde iki katmana izin verir.
- **Değişmez:** Doğrudan eşleşme önce; sonra bütün adaylar. 0=A001, 1=çözüm,
  2+=A002; tahmin/sözlük yok. Düzenli kökler × bütün tek ekler ve bütün iki
  katmanlar `üret → çöz` property korpusudur. İkizleşme/ünlü düşmesi ters
  korpusu ile yapısal belirsizlik korpusu ayrıca sabittir.
- **Araç zinciri:** LSP iki katmanlı rename'i de yeniden giydirir
  (`fiyatıyla→elmasıyla`, `fiyatından→elmasından`). `dil morfoloji [kelime]`
  profil veya adayları, `dil sürüm` etkin profili gösterir.
- **Edition etkisi:** Yeni proje `morfoloji "zee-tr-1" olsun` yazar; eski
  bildirim aynı profile varsayılır, bilinmeyen açık profil P011'dir.
  `proje.kilit` v2 ana/paket profilini taşır. Aday kümesini değiştiren tablo,
  zincir ya da üretim değişikliği `zee-tr-2` ve ana sürüm/edition kararı
  olmadan yapılamaz.
- **Kanıt:** RFC-0018 + spec/13 + profil snapshot'ı, property/ambiguity/LSP/
  proje/CLI testleri; 333 test ve 139 katalog koduyla V1-P1-02 kapandı.

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
  - **Tarihsel/EMEKLİ:** Çağrı tanıma "satır, tanımlı bir işlem adıyla bitiyor
    mu?" kuralıyla deterministik oluyor; ~~bu yüzden işlem çağrıdan önce
    tanımlanmalı~~ (yalnız ilk bootstrap davranışı).
  - En uzun işlem adı önce eşlenir; tanımlı bir işlem adıyla biten ama ayraçsız
    bölge hata verir (S019) — sessiz yanlış yorum yok.
  - **Tarihsel/EMEKLİ:** ~~İmza yalnız ilk çağrıyla sabitlenir ve özyineleme
    yoktur.~~ Bunlar ilk bootstrap kısıtlarıydı; güncel ayrım aşağıdadır.
- **Tarihsel not (1 Eyl 2026, K-081/K-086):** Yukarıdaki üç madde ilk bootstrap
  anını kaydeder; güncel dil davranışı değildir. Başlık ön-tarama,
  özyineleme/T035/C019 ve K-067 sayısal imza terfisi gerçeklenmiştir. Güncel
  sözleşme spec/02 + spec/04'tedir; public imza modeli K-086/spec-10 ile
  tamamlanmış ve V1-P0-01 kapanmıştır.

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
  geri çevrimi. Bu tarihsel açık, K-089'da `zee-tr-1` profil/snapshot
  sözleşmesiyle kapandı.

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

## K-090 — Deterministik görev scheduler'ı ve sözcüksel sahiplik (1 Eyl)

- **Karar:** `eşzamanlı olarak` görev grubu dış ortamın değer snapshot'ını
  kaydeder; `hepsini bekle` bu grubu tek iş parçacıklı işbirlikli scheduler'da
  çalıştırır. Hazır görevler ve aynı anda uyananlar kaynak sırasıyla ilerler;
  görev yalnız `bekle`, tamamlanma veya hata noktasında el değiştirir.
- **Sahiplik:** Her grup aynı sözcüksel kapsamda tam bir kez birleştirilir.
  Boş join, açık grupla kapsamdan çıkma/döndürme/bitirme ve üst üste grup T051;
  join öncesi sonuç erişimi T033 kalır. Bu kuralla sahipsiz görev üretilemez.
- **Hata/iptal:** İlk yönetilmemiş görev hatası kendi kodunu koruyarak görev
  adıyla yayılır ve bekleyen kardeş future'larını düşürür. Dış deadline Ç001
  sahibini bozmadan bütün görev ağacını iptal eder. Tamamlanan sıradan etkiler
  geri alınmaz; `eylem` transaction'ı savepoint'ler karışmasın diye atomik bir
  scheduler dilimidir.
- **Kanıt:** 2 sn + 1 sn görevler 2 sn sanal zamanda `yavaş başladı → hızlı
  başladı → hızlı bitti → yavaş bitti` izini verir. Kardeş iptalinde bekleme
  sonrası yan etki yoktur; iç görev ağacının beklemesi dış kardeşe yol verir;
  dış deadline yalnız doğru `yetişmezse` koluna ulaşır; T033/T051 olumsuzları
  hermetiktir.
- **Sınır:** Çok çekirdekli/önleyici paralellik, yarış/ilk sonuç, stream ve
  dinamik görev sayısı v1 sözü değildir. Senkron platform çağrısı dönene kadar
  atomik dilimdir; dilin `bekle` cümlesi iç içe işlem zincirinde gerçek
  scheduler noktasıdır.
- **Durum:** geçici kabul; RFC-0011 ve normatif spec/14 ile V1-P1-03 kapandı.

## K-091 — Sonuç'un yapılandırılmış Hata değeri (1 Eyl)

- **Karar:** `Sonuç<T>` hata tarafı düz Metin değil, değişmez `Hata`dır.
  `Hata`; kararlı `kodu`, Türkçe `mesajı`, `Seçenek<Hata>` `nedeni` ve
  sıra korumalı Metin sözlüğü `verisi` alanlarını taşır. Kod
  `[A-Z][A-Z0-9_]*` biçimindedir ve `göre / ise` ile eşlenir.
- **Yüzey:** `"DOSYA_YOK" kodlu "Dosya bulunamadı" hatasını döndür` temel
  biçimdir. Var olan Hata `nedeniyle`, Metin sözlüğü `verisiyle` eklenir;
  ikisi birlikteyse neden önce gelir. Neden `varsa` ile güvenle açılır.
- **Geriye uyum:** Eski `"mesaj" hatasını döndür` `GENEL` kodlu Hata üretir.
  `sonucun hatası`, `hata yaz`, metin birleştirme ve `hatanın metni` yine
  yalnız mesajı gösterir. Böylece eski kaynak ve kullanıcı çıktısı değişmez;
  yapı yalnız açık alan erişimi ya da `hatanın json metni` ile görünür.
- **Yeniden yayma:** Bir Hata `hata hatasını döndür` ile kayıpsız yayılır.
  Zenginleştirme eski değeri değiştirmez; yeni kodlu hata onu neden olarak
  sarar. Değişmez üretim, neden zincirinin döngüsüz kalmasını sağlar.
- **Yerleşikler:** Dosya/sayı/ondalık denemeleri sırasıyla `DOSYA_OKUMA`,
  `SAYI_BICIMI`, `ONDALIK_BICIMI` kodlarını üretir. Bu etiketler public
  eşleme sözleşmesidir; anlam değişimi semver incelemesi ister.
- **Kanıt:** Hata açık imzada taşınır; kod/mesaj/eşleme, neden daraltması,
  veri+deterministik JSON, yeniden yayma, üç yerleşik kod, S044/T052
  olumsuzları ve eski kaynak regresyonları 350 testte yeşildir.
- **Durum:** geçici kabul; RFC-0008 §3 ve normatif spec/15 ile V1-P1-04
  kapandı.

## K-092 — Ondalıkta keyfî hassasiyet ve açık bölüm bağlamı (1 Eyl)

- **Karar:** Kullanıcıya çocuk ve profesyonel için ayrı sayı türleri
  açılmayacak. `Ondalık`, imzalı keyfî uzunlukta katsayı + onluk ölçek taşıyan
  tek exact onluk türdür. İlk bootstrap'ın dokuz kesir hanesi ile i64/i128
  katsayı sınırı dil semantiği değildi; kaldırıldı.
- **Exact sınır:** Toplama, çıkarma, çarpma ve sade paydası yalnız 2/5 asal
  çarpanlı bölme kayıpsızdır. İkilik kayan nokta çekirdeğe girmez. TamSayı i64
  kalır ve Ondalığa kayıpsız genişler; Ondalıktan TamSayıya daralma görünürdür.
- **Sonsuz bölüm:** Sonlu olmayan açılım tam 34 anlamlı haneye, yarımlar
  sıfırdan uzağa yuvarlanır. Bağlam platformdan bağımsızdır ve sessiz küresel
  ayarla değişmez. `1/3` için ilk sıfırlar sayılmaz; `10/3` tam kısımdaki
  haneyi sayar. İleride özel bağlam gerekirse görünür API + ayrı RFC ister.
- **Gerekçe:** Dokuz hane para örneklerine yetse de genel amaçlı dil sözü için
  yapaydı; büyük ERP tutarı, uzun kur zinciri ve küçük bilimsel ölçüm aynı
  anda duvara çarpıyordu. Sınırsız exact bölme matematiksel olarak mümkün
  değildir; 34 hane bu tek zorunlu yuvarlama noktasını geniş ve denetlenebilir
  kılar. Çocuk yine yalnız virgüllü sayıyı öğrenir, profesyonel aynı türü terk
  etmek zorunda kalmaz.
- **Göç:** S032 üretimden kaldırıldı ve kod kataloğunda başka anlamda
  kullanılamayacak tarihsel kayıt olarak ayrıldı. `3 ,14` S033 ve `3.14` S001
  yönlendirmesi değişmedi.
- **Kanıt:** Keyfî sabit/katsayı, çok küçük değer, exact `1/8`, 34 haneli
  `10/3`, negatif uzun metin dönüşümü, karşılaştırma, JSON, para ve i64
  daraltma taşması; toplam 356 test. RFC-0013 revizyonu ve normatif spec/16
  ile V1-P1-01 kapandı.
- **Durum:** geçici kabul; usability onayı RFC'nin tam kabul kapısıdır.

## K-093 — Değer semantiği ve gezme imleci (1 Eyl)

- **Karar:** zee'nin bütün kullanıcı değerleri derin değer semantiğindedir;
  atama, argüman, dönüş ve koleksiyona ekleme paylaşılan değiştirilebilir
  nesne kimliği oluşturmaz. Açık referans/kimlik türü v1 sözü değildir.
- **Gezme modeli:** Liste `her X için` gezmesi değer-sonuç imlecidir. Her tur
  giriş snapshot'ındaki öğeyi kopyalar; gövde sonunda `X`in son değerini canlı
  listenin aynı sırasına geri yazar. Alan yazma ve aynı türde yeniden bağlama
  aynı kalıcı sonucu verir. `döndür` akışı yayılmadan önce o turun geri yazması
  tamamlanır.
- **Kaynak sabitliği:** Gezilen liste/sözlüğü yeniden bağlamak, eklemek,
  silmek, sözlüğüne yazmak veya aynı kaynağı iç içe gezmek T053'tür. Denetim
  morfolojik hedef yüzeyinde A002'den önce çalışır; `sayılara` doğrudan doğru
  tanıyı alır. Değişiklikler ayrı listede toplanıp gezme sonrasında uygulanır.
- **Sözlük:** Giriş anındaki anahtarları ekleme sırasıyla snapshot alır;
  döngü adı Metin kopyasıdır ve geri yazılmaz. Kaynak sabitliği listeyle aynıdır.
- **Gerekçe:** Yalnız alan yazmayı yansıtmak `sayı 0 olsun` değişikliğini
  sessizce kaybettirir; gerçek paylaşılan referans ise sahiplik/alias/yarış
  yükünü başlangıç modeline taşır. Değer-sonuç + T053, “öğe geri gider, kap
  gezerken sabit kalır” diye tek cümlede öğretilebilir ve deterministiktir.
- **Kanıt:** Alan yazma, yeniden bağlama, derin kopya/bağımsız alias, başka
  kopyayı gezme içinde güvenle büyütme, beş T053 olumsuzu ve 1–24 uzunluk
  property korpusu conformance testidir. RFC-0019 ve normatif spec/17 yürürlükte.
- **Dürüst kapı:** Makine sözleşmesi tamamlandı; insan zihinsel modeli
  uydurulmadı. `docs/usability-kiti.md` G1/G2/G3 kartları ve önceden
  taahhütlü eşikleri taşır. Gerçek 10 öğrenci + 5 profesyonel formu gelene
  kadar V1-P1-05 AÇIK kalır.
- **Durum:** geçici kabul; usability onayı bekleniyor.

## K-094 — Tekrar üretilebilir ve imzalı paket yayını (1 Eyl)

- **Karar:** `dil paketle`, aynı kaynak+manifest+`SOURCE_DATE_EPOCH` için
  platform metadata'sından arındırılmış, sıralı ve byte-byte aynı `.zep`, SPDX
  3.0.1 SBOM, SLSA provenance ve üçünü özetleriyle bağlayan imzalı yayın
  zarfı üretir. İmza Ed25519 ve alan ayrımlı `zee-yayin-v1` mesajıdır.
- **Anahtar sınırı:** `dil anahtar üret` var olan dosyayı ezmez; Unix'te 0600
  izin ister. Yeni proje iskeleti `*.zee-anahtar` dosyasını Git dışında tutar.
  Yayın öz-imzası registry güveni değildir; güven kökü K-095'in işidir.
- **Fail-closed:** Paket yolu, sıra/fazladan byte, sembolik bağ, yerel yol
  bağımlılığı, SBOM/provenance/içerik oynama ve yanlış imza reddedilir.
- **Kanıt:** ADR-006 + RFC-0020 + spec/18 ve olumlu/olumsuz yayın korpusu;
  P012 ile 370 test.

## K-095 — Registry metadata güven zinciri (1 Eyl)

- **Karar:** Ağ dışı sabitlenen root, rol başına Ed25519 eşik ve eski+yeni
  eşiğin ikisini isteyen ardışık root rotasyonu güven temelidir. Çevrimiçi
  doğrulama sırası timestamp→snapshot→targets'tır; her bağ sürüm, boyut ve
  SHA-256 ile kapalı, kanonik JSON zarfına bağlıdır.
- **Durum güveni:** Sürüm+aynı-sürüm-özeti rollback/equivocation engelidir.
  Zincir bütünüyle doğrulanmadan veya daha yeni sonuç uygulandıktan sonra eski
  işlem kalıcı durumu değiştiremez. Süre sonu, mix-and-match, fast-forward
  zehirleme, bozuk yerel durum ve kaynak limitleri fail-closed'dur.
- **Hedef politikası:** Paket sahibinin yayın anahtarı exact dört yayın
  dosyasına bağlanır; yanked sürüm ve etkin kritik duyuru varsayılan reddir.
- **Kanıt ve açık sınır:** RFC-0020 + spec/19; P013/P014 dahil 378 test.
  Limitli taşıma, atomik kalıcı durum, doğrulanmış cache/offline ve exact
  manifest/kilit/CLI entegrasyonu bitmeden V1-P1-07 açık kalır.

## K-096 — K-016 çağrı kararı deneyden önce bağlandı (1 Eyl)

- **Karar verilmedi:** A çalışan geçici yüzeydir; gerçek katılımcı sonucu
  değildir. K-016, ham anonim form ve önden ilan edilmiş eşik olmadan
  kapatılamaz.
- **Deney:** Zee çağrısı gösterilmeden serbest üretim; ardından değer yüklü
  etiketsiz A/B/C kartları. Sıralar `A→B→C`, `B→C→A`, `C→A→B` olarak beşer
  kişiye dağıtılır. 10 çocuk + 5 profesyonel için toplam ve alt grup eşikleri
  ayrı tutulur; isim/e-posta/ses/video depoya alınmaz.
- **Tek yüzey:** V1 değere bağlama, cümle çağrısı, iç içe ifade, dönüş,
  özyineleme, tanım sırası ve çok-tokenli argümanı tek genel grammar ile
  karşılar. B güçlü çıkarsa doğrudan ek sözdizimi olmaz; B-003/RFC-0021
  expression grammar turu açılır. C de grammar+migration kanıtı olmadan
  seçilemez.
- **Makine hazırlığı:** Karar paketi, uygulama kiti, anonim katılımcı/özet
  şablonları hazırdır. Parser'daki “tanım önce” tarihsel yorumu, gerçek
  başlık ön-taraması ve karşılıklı özyineleme davranışına düzeltildi.
- **Durum:** B-001 KISMEN; V1-P0-07 KARAR. İnsan kanıtı beklenirken dil
  yüzeyi ve 378 test tabanı değişmedi.

## K-097 — İfade grameri katmanları ve tam tüketim (1 Eyl)

- **Karar:** İfadeler en güçlüden en zayıfa primary → erişim/postfix → çağrı
  → aritmetik → birleştirme → karşılaştırma → boolean katmanlarıyla büyür.
  Elle yazılmış yüklem-sonlu recursive descent korunur; bugünkü Türkçe
  tam-bölge kalıpları için Pratt zorunlu değildir.
- **Tam tüketim:** Bir katman bütün bölgeyi tek AST olarak tüketir veya
  başarısız olur; önek eşleşmesiyle artan tokenı başka anlama bırakmaz. `ile`
  önce tam çağrı/aritmetik kalıbına, aksi halde birleştirmeye aittir. Çağrı
  adlarında en uzun görünür ad kazanır. İşlem-adının yalnız kuyruk eşleşmesi
  daha güçlü primary/postfix ifadesini gölgeleyemez; bütün bölge işlem adıyla
  birebir aynıysa geriye uyumlu sıfır-argüman çağrısı korunur. Bu ayrım,
  görünür `sayısı` işleminin `metnin sayısı` ifadesini yanlış S019'a çeviren
  yanlış tanıyı geçerli çağrıları bozmadan kapatır.
- **Belirsizlik:** `ve`/`veya` aynı koşulda karışırsa öncelik uydurulmaz
  (S030). Çağrı sonrası genel postfix ve genel aritmetik-karşılaştırma
  bileşimleri bugün ara ad ister; sessiz AST yerine S015 verir. K-016/C sonucu
  gelmeden parantez varsayılmaz.
- **Büyüme kapısı:** Yeni ifade yüzeyi katmanını, bölge sınırını, `ile/ve/veya`
  ve morfoloji çakışmasını, AST/lowering sonucunu, formatter etkisini ve
  olumlu/olumsuz testlerini aynı RFC/spec değişikliğinde göstermek zorundadır.
- **Kanıt:** RFC-0021 + spec/20 + ADR-002 revizyonu; çağrı/postfix→aritmetik,
  karşılaştırma→boolean, postfix/işlem-adı kuyruğu, tam sıfır-argüman, en uzun
  çağrı, S030 ve S015 sınırlarını kapsayan sekiz bağımsız test. Toplam 386 test; V1-P0-08 ve
  B-003 kapandı. Dosya/fonksiyon parçalama davranış-korumalı B-005 işidir.

## K-098 — Core AST intrinsic/yetkinlik sınırı (1 Eyl)

- **Karar:** HTTP, sensör, CSRF ve parola artık core AST'de alan başına ifade
  varyantı değildir. Mevcut Türkçe yüzeyler ad alanlı kararlı kimlik ve sıralı
  argüman taşıyan tek `Intrinsic` düğümüne indirilir.
- **Tek kayıt:** Argüman/dönüş türü, gereken ağ/donanım/web oturumu/kriptografi
  yetkinliği ve statik etki `compiler/src/intrinsic.rs` kaydında birleşir.
  Checker, etki çözümleyici ve runtime aynı kimliği kullanır; bilinmeyen
  kimlik sessizce çalışmaz.
- **Genellik:** `kapı kapalıysa`, ayrı bir sensör varyantı ya da ikinci
  intrinsic yerine genel `Değil` düğümüdür. Yeni adaptörün AST şemasını
  büyütmesi gerekmez; yine de yeni kullanıcı yüzeyi RFC/spec ve K-097 ifade
  kapısından geçmek zorundadır.
- **Sınır:** Bu kayıtta yetkinlik ihtiyacı sınıflandırıyor, izin vermiyordu.
  Proje/paket izin politikası B-023/K-127; parser/checker/runtime fiziksel
  handler ayrımı B-005; checker faz ayrımı B-006/K-100 ile sonradan tamamlandı.
- **Kanıt:** ADR-011 ve intrinsic/yetkinlik uygulama rehberi; kayıt tekilliği,
  dört lowering ve iki tür olumsuzunu kapsayan yedi bağımsız test. Mevcut
  HTTP/sensör/web/parola davranış korpusu korunarak toplam 393 test yeşil;
  B-004 ve V1-P0-09 kapandı.

## K-099 — Derleyici fiziksel faz ve handler sınırları (1 Eyl)

- **Karar:** Parser cümle/ifade; checker cümle/ifade/çağrı; runtime
  cümle/ifade handler modüllerine ayrılır. Kök dosya token/bağlam/değer/IO ve
  faz orkestrasyonunu taşır; alt handler'lar yalnız `pub(super)` görünürdür.
- **Ölçü:** Parser kökü 2709→1160, checker 3067→963, runtime 3181→1965
  satıra indi. Taşınan kodun sırası ve içeriği korunarak public API ve zee
  kaynak semantiği değiştirilmedi.
- **Büyüme kapısı:** Kaynak-mimari testi büyük handler adlarının köke geri
  dönmesini ve kök/handler dosyalarının ilanlı satır bütçesini sessizce
  aşmasını engeller. Bütçe dolunca sayı yükseltmek yerine yeni sorumluluk
  modülü ya da ortak handler çıkarılır; gerekçeli değişiklik ADR-012 ve faz
  rehberini aynı committe günceller. Hata kataloğu bekçisi de sabit kök liste
  yerine bütün `src` alt modüllerini özyinelemeli ve sıralı tarar.
- **Kanıt:** ADR-012, ADR-002/003 revizyonları ve derleyici faz rehberi; üç
  mimari sınır testi. Mevcut 393 davranış testi korunup toplam 396 test yeşil;
  B-005 ve V1-P0-10 kapandı. Checker'ın semantik sahipliği B-006/K-100'dür.

## K-100 — Checker semantik katmanları ve tek sahiplik (1 Eyl)

- **Karar:** Checker kökü semantik kural deposu değildir; yalnız denetim
  sırasını orkestre eder ve korunması gereken public Rust API'sini yeniden
  dışa aktarır. Türler, bağlam, sembol, akış, çağrı, public sözleşme,
  etki/yetkinlik ve dönüş/control-flow ayrı sahip modüllerdedir.
- **Geçiş sırası:** Etki/yetkinlik denetimi → tür yazımları → açık işlem
  sözleşmeleri → cümle/ifade ile sembol/akış/çağrı → dönüş kanıtı. Böylece
  fail-closed etki geçişi tür hatalarından bağımsız, dönüş birleşimi çağrı
  çıkarımından ayrı kalır.
- **Uyumluluk:** `cozumleyici.rs` 963→143 satıra indi. Eski `eylem.rs` aynı
  davranış ve tanılarla checker'ın `etki` katmanına taşındı. Public `Tur`,
  `VeriTuru`, `SozlukDegerTuru` ve `ad_cozumle` yüzeyi korunur; zee kaynak
  dili, tanı sözleşmesi ve normatif spec değişmedi.
- **Kanıt:** ADR-013, checker katman rehberi ve kök bütçesi/tek sahiplik/public
  API'yi koruyan toplam beş mimari test. İki yeni testle toplam 398 test yeşil;
  B-006 ve V1-P0-11 kapandı. Semantic ID omurgası B-010/K-101 ile sonradan
  tamamlandı.

## K-101 — Semantic identity ile depolama konumunu ayır (1 Eyl)

- **Karar:** `YapiId`, `IslemId` ve `SymbolId` derleyici içi tür güvenli
  newtype'lardır. Kaynak adı tanı/okunabilirlik, kimlik semantic bağ,
  koleksiyon konumu yalnız fiziksel depolama görevi taşır.
- **Yapı/işlem:** `Tur::Yapi(usize)` kaldırıldı; kimlik→konum dizini yapı
  erişiminin tek yoludur. Yapı ve işlem katalogları ada göre sıralandığı için
  yapı vektörü, `HashMap` ve çağrı sırası kimliği değiştirmez. İmza ve
  özyineleme kayıtları `IslemId` ile anahtarlanır.
- **Sembol:** Checker ortamı artık ad→(`SymbolId`, tür) tablosudur. Yeniden
  atama ID'yi korur; yeni sözcüksel tanım yeni ID alır. Başarılı checker,
  `Degisken`/`YeniYapi`/`IslemCagrisi` AST düğümünde kaynak adının yanına
  semantik kimliği yazar.
- **Sınır:** Kimlikler derleme birimi içindir; kalıcı paket/ABI ID'si değildir.
  Faz tipleri B-018/K-102 ile tamamlandı; runtime'ın kaynak adından tamamen
  ayrılıp typed HIR tüketmesi B-019 kapsamındadır. Zee kaynak semantiği ve
  normatif spec değişmedi.
- **Kanıt:** ADR-014, semantic kimlik rehberi; yapı depolama sırası ve işlem
  çağrı sırası tersleme, SymbolId bağı ve indeks-gerileme mimari testi. Dört
  yeni testle toplam 402 test yeşil; B-010 ve V1-P0-12 kapandı.

## K-102 — Derleyici veri fazlarını türlerde görünür kıl (1 Eyl)

- **Karar:** Kaynak, token, parsed AST, hoist edilmiş bağlanmamış program ve
  checker'dan geçmiş bağlı program aynı çıplak veri tipi gibi taşınmaz.
  `KaynakMetni`, `TokenAkisi`, `AyristirilmisAst`, `BaglanmamisProgram` ve
  `BaglanmisProgram` gerçek standart hattın ayrı Rust türleridir.
- **Geçiş:** `BaglanmamisProgram` crate dışından kurulamaz;
  `BaglanmisProgram` yalnız başarılı ad/ID çözümü + tür/akış/etki checker
  geçişinden doğar. Standart denetle/çalıştır/dene API'leri fazlı hattı ve
  `calistir_baglanmis[_io]` runtime girişini kullanır.
- **Uyumluluk:** Raw `Program` döndüren/alan v0 Rust API'leri korunur; faz
  bilgisini yalnız checker başarısından sonra silen adaptörlerdir. Zee kaynak
  dili, tanılar, runtime sırası ve normatif spec değişmedi.
- **Dürüst sınır:** Bağlı program hâlâ AST'dir; ayrı Resolution ürünü, typed
  HIR ve zorunlu semantic span B-019/B-020'dir. B-018 bu işleri tamamlanmış
  saymaz.
- **Kanıt:** ADR-015, derleyici faz rehberi, parsed/bound ID farkı, eski API
  eşdeğerliği, standart-hat mimari testi ve yanlış parsed→runtime geçişi için
  compile-fail. Dört yeni testle toplam 406 test yeşil; B-018 ve V1-P0-13
  kapandı.

## K-103 — Checker kanıtını zorunlu typed HIR'a taşı (1 Eyl)

- **Karar:** Başarılı checker geçişinin tür sonuçları geçici dönüş değeri
  olarak kaybolmaz. Her denetlenmiş ifade program içi `HirDugumId`, açık
  `Tur` ve varsa `SymbolId`/`IslemId`/`YapiId` bağıyla `HirProgram`a yazılır.
- **Faz sahipliği:** `BaglanmisProgram` artık çıplak AST değil zorunlu HIR
  sahibidir. Kaynak AST Türkçe tanı, formatter/LSP ve v0 Rust API uyumluluğu
  için salt-okunur korunur; eski `into_program` yalnız HIR üretiminden sonra
  faz bilgisini siler.
- **Ömür:** Checker'daki AST adresi public kimlik değildir; yalnız aynı süreçte
  HIR kaydını bulmak için private locator'dır. Public semantic düğüm kimliği
  `HirDugumId`dir; kutulu program yer değiştirmez ve bağlı program klonlanmaz.
- **Dürüst sınır:** Runtime henüz bütün semantic kararlarını HIR bağından
  almıyordu. K-104 sonradan bu geçişi tamamlayıp B-019/V1-P0-14'ü kapattı;
  o anda açık kalan zorunlu source span'i K-108 sonradan B-020 kapsamında
  kapattı. Zee kaynak semantiği ve normatif spec değişmedi.
- **Kanıt:** ADR-016, typed HIR rehberi; ifade türü+SymbolId, işlem/yapı ID
  dizini ve HIR'sız bağlı programı reddeden üç yeni test. Toplam 409 test.

## K-104 — Standart runtime'ı HIR bağlarına geçir (1 Eyl)

- **Karar:** Fazlı çalıştırma ve `dene`, `CalistirmaProgrami::Hir` kolunu
  kullanır. Bu kol değişken, işlem ve yapı seçimlerini yalnız `HirBagi`ndan
  alır; kaynak adı eksik/bozuksa ona geri düşerek semantic karar uydurmaz.
- **Uyumluluk:** Raw `Program` alan v0 embedding API'si ayrı `Ham` kolunda
  ad-temelli davranışını korur. Zee kaynak dili, çıktı, tanı ve scheduler
  sırası değişmedi.
- **Eşzamanlılık:** Görev ifadesi klonlanmaz; özgün AST/HIR düğümünü ödünç
  alır. Böylece checker'ın düğüm bağı scheduler future'ında da korunur.
- **Mimari:** HIR/raw adaptörü 140 satır bütçeli
  `yorumlayici/hir_gecisi.rs` sahibidir; runtime ve faz kökleri ilanlı
  bütçelerini aşmaz.
- **Kanıt:** Değişken kaynak adını ve işlem+yapı adlarını checker sonrasında
  bilerek bozan iki test yalnız HIR ID'leriyle aynı sonucu üretir. Standard
  runtime mimari testi ve dönüşsüz çağrının açık HIR türüyle dört yeni test,
  toplam 413 test; B-019 ve V1-P0-14 kapandı.

## K-105 — Native ağ bekleme ve bellek sınırını kapat (1 Eyl)

- **Karar:** Native HTTP istemcisi açık `içinde` yoksa 30 saniyelik mutlak
  son tarih kurar; bağlantı denemeleri, yazma ve her okuma tek kalan bütçeyi
  tüketir. Başlıklar dahil wire yanıt en çok 8 MiB'dır.
- **Sunucu:** Yerel TCP adaptörü başlık ve ilan edilmiş gövdenin tamamını
  kabulden başlayan 10 saniyelik mutlak sürede ister. Her okumada yalnız kalan
  süre kullanılır; yavaş bayt akışı süreyi yenilemez ve aşım 408'dir. Yanıt
  yazımı da 10 saniye socket sınırı taşır.
- **Sınır:** DNS çözümlemesi işbirlikli platform sınırıdır. Native istemci
  bugün yalnız `http://` taşır; Zee TLS'yi elle yazmaz. Sınırlı oturum deposu,
  LSP girdisi, HTTPS/hedef capability politikası ve ortak `KaynakSinirlari`
  ardıl işlerdir.
- **Kanıt:** ADR-017 ve spec/09/11/12; geçmiş mutlak tarih, 8 MiB okuyucu
  sınırı ve deadline'sız küçük loopback yanıtı için üç yeni test. Toplam 416
  test; B-025'in ağ dilimi ve V1-P0-15 kapandı.

## K-106 — Web oturum deposunu sınırla, ömrü mutlaklaştır (1 Eyl)

- **Karar:** Process içi depo en çok 4096 toplam ve 1024 anonim oturum taşır.
  Süresi dolanlar istek başında silinir. Kota dolunca en uzun süredir
  kullanılmayan anonim kayıt; eşit erişimde oluşturma sırasına göre tahliye
  edilir. Kimlikli kayıt anonimden önce kurban edilmez.
- **Fail-closed:** Depo yalnız kimlikli oturumlarla doluysa yeni giriş mevcut
  bir kullanıcıyı düşürmek yerine hata olur. Anonim form isteği bellek
  kullanımını sınırsız büyütemez.
- **Ömür:** Anonim 10 dakika, kimlikli 30 dakika oluşturma anından başlayan
  mutlak ömürdür. Erişim yalnız LRU sırasını günceller, expiry'yi kaydırmaz.
- **Deployment sınırı:** Depo process-local'dır; mevcut güvenli profil tek
  runtime process'i içindir. Per-IP/rate-limit ve atomik paylaşımlı depo
  B-046'da açık kalır; sticky session ortak revoke değildir.
- **Kanıt:** ADR-018 + spec/12; anonim LRU, anonimin kimlikli girişe yer açması,
  yalnız kimlikli doluluk reddi ve kaymayan expiry. Üç yeni testle toplam 419;
  V1-P0-16 kapandı, B-046 process içi dilimde tamamlandı.

## K-107 — LSP çerçeve ve JSON girdisini fail-closed sınırla (1 Eyl)

- **Çerçeve:** `dillsp` en çok 8 KiB başlık ve 8 MiB gövde kabul eder. Tam bir
  ve yalnız bir `Content-Length` tahsisten önce doğrulanır; UTF-8/çerçeve
  hatasında akış konumu uydurulmadan process Türkçe hatayla kapanır.
- **JSON bütçesi:** En çok 128 iç içelik ve 100.000 değer düğümü ayrıştırılır.
  Doğrudan API de 8 MiB metin sınırını taşır.
- **Unicode:** Yüksek surrogate yalnız geçerli düşük surrogate ile birleşir;
  yanlış/eksik çift, tek düşük surrogate ve kaçışsız U+0000..U+001F reddedilir.
  Önceki çıkarma taşması/panic yolu yoktur.
- **Dürüst sınır:** Açık belgelerin toplam belleği, çıktı büyümesi ve istek
  zaman bütçesi B-025'te kalır; tek-girdi parser/framing kapısı kapanmıştır.
- **Kanıt:** ADR-019, üç çerçeve ve dört JSON olumsuz/sınır testi. Yedi yeni
  testle toplam 426; B-047 ve V1-P0-17 kapandı.

## K-108 — Semantic HIR düğümünde kaynak aralığını zorunlu yap (1 Eyl)

- **Karar:** Her `HirIfadeBilgisi` kimlik, tür ve bağın yanında kurucuda
  zorunlu `HirKaynakAraligi` alır. Kaynak kökeni `Option` değildir; spansiz
  semantic HIR ifadesi kurulamaz.
- **Hassasiyet:** Lexer token konumu AST'de korunmuş değişkenler
  `Kesin { satir, sutun, uzunluk }` taşır. Diğer mevcut AST ifadeleri eksik
  bilgiyi `1:1` diye uydurmak yerine açık `Satir { satir }` zarfı taşır.
  Satır/sütun/uzunluk bileşenleri `NonZeroUsize`dır.
- **Mimari:** Kaynak aralığı tipi `hir/kaynak.rs` sahibidir; `hir.rs` 180
  satırlık faz bütçesini aşmaz. Checker normal değer ifadelerini ve dönüşsüz
  çağrı cümlelerini aynı zorunlu sözleşmeyle kaydeder.
- **Dürüst sınır:** B-020 yapısal kaynak-kökeni değişmezi kapandı. Bütün AST
  varyantlarında kesin sütun+uzunluk korumak tanı hassasiyeti işi B-050'dir;
  bu tarihsel açık K-126/ADR-030 ile sonradan kapandı. Kaynak dil semantiği
  değişmedi.
- **Kanıt:** ADR-020, typed-HIR/faz/kimlik rehberleri; mevcut değişken testine
  kesin konum kanıtı, bileşik ifadeye satır zarfı davranışı ve mimari sahiplik
  kontrolü. Bir yeni testle toplam 427; B-020 ve V1-P0-18 kapandı.

## K-109 — Production doğrudan panic yüzeyini kapat (1 Eyl)

- **Karar:** `lib`, `dil`, `dillsp` ve `olcum` crate kökleri test dışı
  derlemede Clippy `unwrap_used`, `expect_used`, `panic`, `unreachable`,
  `todo` ve `unimplemented` lintlerini `deny` eder. Yeni doğrudan panic yüzeyi
  CI derlemesini durdurur; test fixture'ları bilinçli olarak kapsam dışıdır.
- **Audit:** Lexer/parser, formatter/LSP, checker/runtime, paket/arşiv/kalıcı
  dosya ve CLI/ölçüm yollarındaki 46 production nokta `Result`, T016, C000
  ya da açık başarısız süreç koduna çevrildi. Scheduler iç tutarsızlığı
  process panic'i değildir.
- **Kimlik kapasitesi:** `SymbolId`nin kapsam+sırayı iki 32-bit dilime
  sıkıştıran assertion'ları kaldırıldı. Program-içi, kalıcı ABI olmayan kimlik
  iki `usize` bileşen taşır; semantic identity ve depolama ayrımı değişmedi.
- **Dürüst sınır:** Bu kapı kolay/doğrudan panic makrolarını ve convenience
  unwrap'ları kapatır. Keyfî UTF-8 parser girdisi fuzz kanıtını B-015/K-110
  tamamladı; malformed AST/HIR genel invariant doğrulamasını
  B-017/K-112/ADR-023 sonradan tamamladı. Kaynak semantiği değişmedi.
- **Kanıt:** ADR-021 ve production panic rehberi; sıfır konumlu elle kurulmuş
  AST'nin panic yerine T016 vermesi ile dört crate lint sahipliği. İki yeni
  testle toplam 429; B-014 ve V1-P0-19 kapandı.

## K-110 — Lexer/parser fuzz hattını kalıcılaştır (1 Eyl)

- **Hedef:** `&str` kabul eden libFuzzer hedefi lexer başarılıysa token akışını
  hem normal hem hata-kurtarmalı parser'da yürütür; tanı geçerli sonuçtur,
  panic/crash değildir.
- **Korpus:** Sekiz tohum geçerli program, Unicode/homoglyph, emoji, combining
  im, CRLF, sekme/girinti, metin kaçışı, dev sayı, virgül ve blok saldırılarını
  taşır. Zee sözlüğü mutation'ı anlamlı token ve UTF-8 byte dizilerine iter.
- **Kalıcı kanıt:** Ana testte korpus replay, 4.096 deterministik üretilmiş
  UTF-8 kaynak ve 64 KiB sayı/ondalık/virgül/girinti uçları vardır. Üç yeni
  testle toplam 432.
- **Operasyon:** `nightly-2026-08-31`, `cargo-fuzz 0.13.2`, ayrı kilit dosyası,
  64 KiB max input ve beş saniye timeout sabittir. Gece işi korpusu cache'ler;
  crash girdisini artifact yapar. İlk yerel smoke 1.048.287 girdiyi 31 saniyede
  crash, panic ve timeout olmadan tamamladı.
- **Dürüst sınır:** Bu mutation kanıtıdır, bütün dizilerin biçimsel ispatı
  değildir. Geçersiz UTF-8 dosya okuma sınırında; malformed elle kurulmuş
  token/AST, B-017/K-112'nin invariant kapısında; daha büyük kaynakların
  resource bütçesi ayrı kapıdadır.
  ADR-022 ve fuzz rehberiyle B-015/V1-P0-20 kapandı.

## K-111 — Morfoloji değişmezlerini sürekli mutation'a bağla (1 Eyl)

- **Üret→çöz:** `zee-tr-1` profilindeki bütün tek ekler ve bütün geçerli
  `iyelik + dış ek` zincirleri 4.096 deterministik üretilmiş, 2–32 kod noktalı
  Zee kökünde özgün `(kök, ek-zinciri)` çözümünü korur.
- **Belirsizlik:** Üretilmiş 2.048 yüzeyin yapısal kök adaylarının tamamı aynı
  kapsama konur. Tek aday aynı köke çözülür; çoklu aday sırasından bağımsız
  A002 üretir ve asla sessiz seçim yapmaz.
- **Unicode sınırı:** `ğ/ö/ş/â/İ` birleşik NFC yazımları lexer'dan geçer;
  aynı görünen ayrıştırılmış NFD biçimleri var olan RFC-0002 kuralıyla S029
  verir. Morfoloji profiline örtük normalizasyon veya yeni semantik eklenmedi.
- **Mutation:** Ayrı libFuzzer hedefi her byte girdiden geçerli Zee kökü
  üretir, seçilmiş tek/iki katmanlı zinciri geri çözer ve bütün adaylarda
  A001/A002 kararını denetler. Gecelik matrix iki fuzz korpusunu bağımsız
  büyütür. İlk morfoloji smoke'u 527.966 girdiyi 31 saniyede crash, panic veya
  property ihlali olmadan yürüttü.
- **Kanıt ve sınır:** Üç yeni stable regresyonla toplam 435 test yeşildir.
  Bu geniş arama biçimsel ispat değildir; `zee-tr-1` yüzey tablosu ve üretim
  semantiği değişmedi. Morfoloji doğrulama + ortak fuzz rehberiyle
  B-016/V1-P0-21 kapandı.
- **Doğrulama hijyeni:** Tam kapı sırasında görülen K-105 HTTP istemci testi
  kararsızlığı production kodunda değil, sahte sunucunun isteği tek `read` ile
  eksik tüketip macOS'ta RST üretmesindeydi. Stub artık başlık sonuna kadar
  okur; default-timeout regresyonu tekrarlanabilir kaldı.

## K-112 — AST/HIR faz değişmezlerini yürütülebilir yap (1 Eyl)

- **Parser saflığı:** `AyristirilmisAst` çözülmüş ad, `SymbolId`, `IslemId`,
  `YapiId` veya checker'a ait `sonuca_sarmala` işareti taşıyamaz. Tek parçalı
  zincir, boş sentetik liste ve çağrı olmayan `CagriCumlesi` gibi normal
  parser'ın üretemeyeceği biçimler fail-closed reddedilir.
- **Bağlı tamlık:** Programın ana cümleleri, deterministik işlem sırasındaki
  gövdeler ve testler bütünüyle dolaşılır. Her AST ifadesi tek HIR kaydı,
  benzersiz `HirDugumId`, açık tür, zorunlu kaynak aralığı ve varyantla uyumlu
  `HirBagi` taşır. Semantic ID canonical adla eşleşir; değer konumunda
  `DegerDondurmez` bulunamaz.
- **Yetim kayıt yasağı:** Ziyaret edilen AST ifade sayısı HIR kayıt sayısıyla
  eşitlenir. Böylece yalnız eksik bağ değil, artık canlı AST düğümüne ait
  olmayan HIR kaydı da görünür olur.
- **Gerçek bulgu:** Özellik erişimi yapı alanına çevrilirken iç `nesne`
  kutusunun klonlanması, eski AST adresinin HIR kaydını yetim bırakıyordu.
  Dönüşüm artık alt düğümü taşır; beklenmeyen varyant panic olmadan T016'dır.
- **Hata ve maliyet:** Normal parser/checker çıkışlarında debug/test kapısı
  otomatik çalışır ve ihlali faz+yapısal yol+satır taşıyan C000'e çevirir.
  Release hattında ikinci tam-ağaç taraması otomatik değildir; açık doğrulama
  API'si kullanılabilir.
- **Kanıt:** ADR-023 ve AST/HIR invariant rehberi; üç integration, iki HIR
  unit regresyonu ve mevcut koleksiyon/mimari korpusu. Toplam 440 test
  yeşildir; B-017/V1-P0-22 kapandı.

## K-113 — Parser kurtarmasını cümle ve girinti sınırına bağla (1 Eyl)

- **Sorun:** İç bloktaki ilk parser hatası ebeveyne kaçıyor, ebeveyn AST
  düğümünü düşürüyor ve sağlam kardeş cümleyi dış kapsama sızdırabiliyordu.
  Erken dönüş `derinlik` sayacını yüksek bırakınca sonraki geçerli üst düzey
  tanım yanlış S021 alabiliyordu.
- **Senkronizasyon:** Kurtarma yatayda `SatirSonu`, dikeyde yalnız hatalı
  başlığa ait dengeli `Girinti`…`Cikinti` sınırını kullanır. Kör token tahmini
  yapılmaz; sonraki aynı-girintili kardeş kendi ebeveyninde sürer.
- **Özel bloklar:** Yapı alanı, eşzamanlı görev, `göre` kolu, `değilse` ve
  `yetişmezse` ilk bozuk satırdan sonra sağlam satırı korur. Bütün zorunlu
  çocuklar bozuksa ADR-023'ü ihlal eden boş sentetik AST üretilmez.
- **Tanı sözleşmesi:** Parser, birim ve checker tanıları `(satır, sütun, kod,
  mesaj)` ile kararlı kaynak sırasındadır. CLI `denetle` ve LSP belge başına
  ortak en çok 20 tanı yayımlar; normal derleme/çalıştırma ilk tanıda durur.
- **Mimari:** Recovery `ayristirici/kurtarma.rs` sahibine ve 160 satır
  bütçesine ayrıldı. Parser kökü 1.200 satırlık sınırı aşmadı.
- **Kanıt:** RFC-0010 §2.1, ADR-024, spec/06 ve parser kurtarma rehberi; altı
  parser + bir gerçek LSP regresyonu. Toplam 447 test yeşildir. Recovery
  yolunu da çalıştıran 31 saniyelik lexer/parser smoke'u 977.014 mutation'ı
  çökmesiz tamamladı; B-021/V1-P0-23 kapandı.

## K-114 — Tanı kodunu sürümler arası anlam kimliğine bağla (1 Eyl)

- **Sorun:** Kaynak↔katalog birebirlik testi kod ekleme/silmeyi yakalıyordu,
  fakat aynı kodun kaynak ve katalogda birlikte başka bir olaya taşınmasını
  ayırt edemiyordu. A004/C014 yalnız dipnotta kalan sessiz boşluklardı.
- **Kimlik tabanı:** 145 etkin ve 3 ayrılmış tanı `durum, kod,
  kararlı_kimlik, kanonik_özet` alanlı şema-1 TSV fixture'ına alındı. Anahtar
  tekil, küçük ASCII ve S/A/T/C/D/P/Ç ailesiyle uyumludur.
- **Mezar taşı:** A004, C014 ve S032 katalogda açık `ayrılmış` kayıttır. Emekli
  kod silinmez, başka anlamla tekrar etkinleştirilmez.
- **Karar:** Dinamik mesaj/öneri/konum değil, kodun temsil ettiği semantik olay
  kararlıdır. Anlam değişikliği yeni kod ister; yalnız editoryal özet düzeltmesi
  fixture farkıyla incelemede görünür olur. Program değeri `Hata.kod` ayrı
  sözleşmedir.
- **Kanıt:** ADR-025, RFC-0010 §2.2, spec/06, tanı kimliği bakım rehberi ve
  katalog↔fixture regresyonu. Kaynak↔katalog kapısıyla birlikte bütün 148 kayıt
  çift yönden korunur; toplam 448 test yeşildir. B-022/V1-P0-24 kapandı.

## K-115 — Bütün runtime IO sınırını sürümlü iz ve replay'e bağla (1 Eyl)

- **Sorun:** Hermetik `ToplayanIo` testlerde dış dünyayı denetliyordu; gerçek
  bir CLI koşusunun klavye, rastgelelik, saat, dosya, ağ, web ve sensör
  konuşmasını sonradan birebir üretmenin sürümlü artefaktı yoktu.
- **Karar:** `GirdiCikti` sözleşmesinin 27 yönteminin tamamı işlem,
  argümanlar, sonuç ve 1 tabanlı küresel sırayla şema-1 izine alınır. Replay
  sıradaki olay dışında arama yapmaz; işlem/argüman farkında veya artan olayda
  fail-closed durur ve hiçbir dış etkiyi yeniden uygulamaz.
- **Biçim ve sınır:** Başlık+sıralı satırlar, kanonik onluklar ve küçük harfli
  hex UTF-8 alanları tek byte biçimini verir. 64 MiB, 100.000 olay ve olay
  başına 4.096 alan bütçesiyle kapalı işlem/sonuç şeması yürütme öncesi
  doğrulanır; kırıcı sözlük değişimi yeni sürüm ister.
- **Kullanıcı ve sır sınırı:** `dil iz kaydet/oynat` izi atomik yazar, kaynak
  üzerine yazmayı reddeder ve replay çıktısını yalnız tam tüketimden sonra
  gösterir. İz varsayılan olarak Git dışıdır ve özel veri kabul edilir;
  parola ile PHC özeti yalnız SHA-256 parmak izi taşır.
- **Kanıt:** RFC-0022, ADR-026, spec/21 ve IO izi rehberi; byte snapshot'ı,
  bütün IO yüzeyli round-trip, sıra/argüman/tam tüketim, bozuk şema, sır
  sızıntısı, kaynak değişimi ve üzerine yazma reddi için beş çekirdek+iki CLI
  regresyonu. Toplam 455 test yeşildir; B-027/V1-P0-25 kapandı.

## K-116 — Tohum, saat ve hermetik dünyayı `zee-io-1` profiline bağla (2 Eyl)

- **Sorun:** CLI ve playground aynı xorshift ailesini kopya kodla, farklı
  durum hazırlama noktalarıyla yürütüyordu. Sanal saatin ve `ToplayanIo`
  fallback'lerinin testlerde gözlenebilir olmasına rağmen sürüm kimliği yoktu;
  tam i64 aralığı mevcut çıkarma hesabını taşırabiliyordu.
- **Karar:** Tek `SurumluRastgele` sahibi, `zee-io-1` profilinde tohum XOR
  karışımını, xorshift64* adımlarını, yansız reddetme eşikli uçları-dahil
  eşlemeyi ve tam i64 desteğini taşır. Tohum 7'nin `[1,100]` ilk altı sonucu
  `47,29,69,71,26,60` kalıcı conformance vektörüdür.
- **Zaman:** Takvim UTC görünümü ile deadline'a ait tekdüze an ayrıdır.
  Hermetik an 0'dan başlar; negatif bekleme ilerletmez, eski enjeksiyon saati
  geri götürmez, bekleme takvimi oynatmaz. Scheduler en yakın ortak uyanışa
  ilerleme sözünü korur.
- **Fake IO:** Girdi/rastgele FIFO, aralığa kırpma+alt-uç fallback'i, istem
  çıktı sırası, LF'li dosya, exact URL/yol, kapalı varsayılan sensör ve ışık
  çıktı biçimi normatifleşti. Güvenlik tokenları OS CSPRNG'sinde kalır.
- **Kanıt:** RFC-0023, ADR-027, spec/22 ve profil rehberi; profil+dizi, tam
  i64, rastgele kuyruk, sanal saat ve bütünleşik fake-IO/playground için beş
  conformance testi. Toplam 460 test yeşildir; B-028/V1-P0-26 kapandı.

## K-117 — `.zep` yolunu platformdan ve Unicode gösteriminden bağımsız kıl (2 Eyl)

- **Sorun:** `.zep` metadata'sız ve sıralıydı; fakat aynı görünen Türkçe dosya
  adı bir dosya sisteminden NFC, diğerinden NFD dönerse paket byte'ı
  değişebilirdi. Klasik traversal testleri de Unicode ayraç benzerleri ve
  görünmez yön denetleyicilerini kalıcı bir veri korpusunda kapsamıyordu.
- **Kanonik üretim:** Dosya sistemi yolunun her bileşeni Unicode 17.0 UAX #15
  NFC'ye çevrilir, sonra `/` ile birleştirilip UTF-8 byte sırasında yazılır.
  NFC sonrası çakışan iki dosya seçim yapılmadan reddedilir. Tüketici gelen
  arşiv yolunu dönüştürmez; zaten NFC değilse fail-closed durur.
- **Güvenlik:** Unicode 17.0 UTS #39'da `/`, `\\`, `.`, `:` iskeletine giden
  işaretler, tam genişlikli ayraç/nokta ve görünmez bidi/biçim denetleyicileri
  `.zep` v1 yolunda yasaktır. 80 vaka kod noktası olarak kalıcı TSV'ye
  bağlandı; UAX #15 için `unicode-normalization 0.1.25` lockfile'da sabitlendi.
- **Çapraz platform kanıtı:** `kaynak/çağrı.dil` içeren gerçek proje ağacının
  tam `.zep` hex fixture'ı tek integration testinde Ubuntu/macOS/Windows CI
  matrisinde çalışır. Fixture, NFD→NFC ve saldırı korpusu için üç yeni
  regresyonla toplam 463 test yeşildir.
- **Sözleşme:** RFC-0020, ADR-006/028, spec/18,
  [`.zep` conformance rehberi](../docs/zep-conformance.md), sürüm notu ve
  V1-P1-08 aynı değişiklikte güncellendi. B-030/B-031 kapandı.

## K-118 — Normatif kanıtı ve hareketli depo sayılarını makineye bağla (2 Eyl)

- **Sorun:** RFC/ADR/spec indeksleri dosyanın varlığını gösteriyor, hangi testin
  hangi sözü kanıtladığını tek yerde göstermiyordu. README'deki golden, test,
  tanı ve karar sayıları da her dilimde elle aranıp değiştiriliyor; unutulan
  sayı semantik belgeden daha az ciddi olmayan drift üretiyordu.
- **Kanıt haritası:** 23 RFC, 26 ADR ve 22 spec bölümü; `kanitli`, `kismi` veya
  `taslak` durumu, gerçek Rust test dosyaları ve açık kapsam notuyla 71 satırlı
  sürümlü TSV'de birebir bağlandı. Yeni/yinelenen/eksik belge, olmayan ya da
  test taşımayan `.rs` yolu ve testsiz tamamlanmış kayıt CI'da reddedilir.
- **Tek sayı kaynağı:** `depo_sayilari`; numaralı golden dosyalarını,
  `#[test]`+compile-fail doctest vakalarını, tanı fixture durumlarını, RFC durum
  satırlarını ve ADR/spec dosyalarını doğrudan sayar. `--yaz` README işaretli
  bloğunu atomik günceller; `--denetle` byte farkında başarısız olur.
- **Kapsam:** Tarihsel K kayıtlarındaki o güne ait test toplamları snapshot'tır;
  araç yalnız README'deki bugünkü canlı tabloyu sahiplenir. Harita test
  dosyasının varlığını yapısal kanıtlar; testin normatif iddiayı gerçekten
  karşıladığı yine kod incelemesinin sorumluluğudur.
- **Kanıt:** ADR-010 revizyonu, [depo bütünlüğü rehberi](../docs/depo-butunlugu.md),
  harita kapsamı ve README üretici tazeliği için iki yeni regresyon. Toplam 465
  test yeşildir; B-043/B-044 kapandı.

## K-119 — Formatter'ı tam parser-token eşdeğerliğine bağla (2 Eyl)

- **Sorun:** C011 biçimleme öncesi/sonrası token türlerini kıyaslıyor fakat
  `SatirSonu`nu bilinçli olarak süzüyordu. Formatter bugün cümleleri
  birleştirmese de bu oracle, gelecekte iki cümleyi tek satıra taşıyan bir
  gerilemeyi anlam değişikliği saymayabilirdi. B-042 bu nedenle kısmi kaldı.
- **Karar:** Parser'ın yapısal girdisi `SatirSonu`, `Girinti`, `Cikinti` ve
  `DosyaSonu` dahil bütündür. Yalnız satır/sütun/uzunluk gibi kaynak haritası
  konumları biçimlemeyle değişebilir. Tam `TokenTur` dizisi eşit değilse C011
  sonucu yazmadan durur.
- **Property korpusu:** Sayısal 33 golden programın dış boşlukları ve girinti
  genişlikleri deterministik dağıtılır; metin sabiti ile yorum gövdeleri aynen
  kalır. Her varyant biçimlenir, önce/sonra tam parser izi eşitlenir ve iki
  taraf da birim işlem adlarıyla tohumlanmış gerçek parser'dan geçirilir.
  Ayrı regresyon, iki cümle ile tek satırın izini özellikle farklı tutar.
- **Sözleşme:** RFC-0021, ADR-002 ve spec/20 aynı kapıyı tanımlar; kanıt
  haritası yeni testi üç belgeye bağlar. Kullanıcı sözdizimi ve resmî çıktı
  değişmedi. İki yeni regresyonla toplam 467 test yeşildir; B-042 kapandı.

## K-120 — LSP tanım ve rename hedefini typed HIR kimliğine bağla (2 Eyl)

- **Sorun:** K-072/K-089 morfolojik yüzeyi doğru üretiyordu fakat önce hangi
  adın seçildiğini belge satırlarında kök adayları arayarak tahmin ediyordu.
  İki ayrı bloktaki aynı ad ilk tanıma gidebilir veya birlikte değişebilirdi;
  hatalı belgede kısmi metin eşleşmesi güvenilir bir semantic seçim değildi.
- **HIR dizini:** Checker ilk tanım ve her yeniden atama LHS aralığını
  `SymbolId` ile kaydeder; ifade okumaları mevcut HIR bağlarından aynı akışa
  katılır. `HirProgram` işlem/yapı tanım ve kullanımını da
  `IslemId`/`YapiId` üzerinden semantic araçlara açar.
- **LSP kararı:** Definition ve rename önce açık belgeyi birim/paket grafiğiyle
  bağlı programa derler, imleç aralığından tek ID seçer. Morfoloji bundan sonra
  yalnız seçilen `SymbolId`nin yüzeylerini yeni köke giydirir. Çok kelimeli
  işlem adı tek düzenlemedir; yapı tanım ve kurulumları aynı `YapiId`yi izler.
- **Fail-closed:** Parser/checker hatası veya A002 belirsizliğinde metin
  fallback'i yoktur; yanıt `null`, neden didOpen/didChange tanısıdır. Metin
  sabiti, yorum ve başka kapsamdaki aynı yazım düzenleme kümesine giremez.
- **Sınır ve kanıt:** K-120 tek açık belge WorkspaceEdit'idir; dosya kimlikli
  çok-dosyalı rename ve bütün AST'nin kesin span'i ayrı kapıdır. Dış
  birim tanımının satır numarası açık belgedeki kullanımla çakışsa bile yerel
  tanım başlığı doğrulanmadan definition/rename üretilmez. ADR-014/016,
  RFC-0018/spec-13, [semantic LSP rehberi](../docs/lsp-semantic-gezinme.md), bir
  HIR ve yedi LSP regresyonuyla toplam 475 test yeşildir; B-041 kapandı.

## K-121 — Yerel çağrı çıkarımını kaynak sırasından bağımsızlaştır (2 Eyl)

- **Sorun:** Yerel `<ad> al` işlem imzası ilk çağrıda kuruluyor, o çağrının
  sonucu da hemen HIR'a yazılıyordu. Sonraki Ondalık kısıt imzayı genişletse
  bile daha önce türlenen sonuç dar kalabiliyor; aynı çağrıların yerini
  değiştirmek programın kabulünü ve HIR türünü değiştirebiliyordu.
- **Karar:** Checker önce programın kopyasında erişilebilir ana/test çağrılarını
  keşfeder ve `IslemId` ile parametre konumu başına kısıtları birleştirir.
  Asıl AST daha sonra bu nihai imzalarla bir kez denetlenir; kullanıcı tanısı,
  semantic bağ ve typed HIR yalnız bu son geçişten çıkar.
- **Birleşim:** Eşit tür sabit kalır; TamSayı+Ondalık Ondalık olur. Aynı kural
  Liste, Sözlük, Seçenek ve Sonuç kapsayıcılarının sayısal iç türüne uygulanır.
  Birleşmeyen çift asıl geçişte T017, parametre sayısı farkı T015'tir. İşlem
  parametreleri birbirinden bağımsız birleşir; açık public imza değişmez.
- **Mimari sahiplik:** Kopya-AST keşfi ve kafes birleşimi
  `cozumleyici/cikarim.rs` yaprağına taşındı; `cagri.rs` nihai imzayla gövde ve
  çağrı doğrulamasını sürdürür. ADR-013, RFC-0006, spec/04 ve spec/10 ile
  [çağrı çıkarımı rehberi](../docs/cagri-cikarimi.md) aynı sözleşmeyi taşır.
- **Kanıt:** Dar→geniş ve geniş→dar skaler, liste, iki parametre, iç içe çağrı
  grafiği, iç blokta hata sonrası kısıt toplama ve iki sırada da T017 üreten
  altı regresyon eklendi. Toplam 481 test yeşildir; B-007 ve V1-P0-01 kapandı.

## K-122 — `zee-tr-1` profilini immutable uyumluluk kaydına bağla (2 Eyl)

- **Sorun:** Profil tablosu snapshot ve geniş property korpusuyla sabitti;
  ancak davranışla birlikte snapshot'ı güncellemek teknik olarak mümkündü.
  `zee-tr-1` adı aynı kalırken çözüm/üretim anlamının değişmesini CI tarihine
  karşı reddeden bağımsız bir kapı yoktu.
- **Semantic kayıt:** Şema `zee-morfoloji-uyumluluk-v1`; profil dökümü, 11 ham
  ses-değişimi/belirsizlik yüzeyi ve 4.096 deterministik kökün yedi tek+altı
  iyelikli zincirindeki 53.248 kanonik üretim ile sıralı bütün çözüm kümeleri
  tek akışta SHA-256'ya indirilir. `zee-tr-1` kaydı
  `e6034e7359e5d5d1bf5f3f06b6a7767616b2b6daf8ab916226b48d522f99f220`dur.
- **İki kapı:** `morfoloji_testi.rs` çalışan kaydı immutable fixture ile byte
  eşitler. CI ise taban Git revizyonunda zaten bulunan bütün
  `morfoloji-zee-tr-N.sha256` dosyalarının değiştirilmesini, silinmesini veya
  yeniden adlandırılmasını reddeder. Bu nedenle kodla fixture'ı birlikte
  güncellemek çıkış yolu değildir.
- **Geçiş yolu:** `zee-tr-1` fixture'ı asla güncellenmez. Semantic değişiklik
  yeni `zee-tr-N` gerçekleme+fixture'ı, eski profil desteği ve proje/kilit ile
  ana sürüm/edition geçiş kararı ister. `dil morfoloji --uyumluluk` çalışan
  kaydı gösterir.
- **Kanıt ve sınır:** RFC-0018/spec-13 ile
  [profil uyumluluk rehberi](../docs/morfoloji-profil-uyumlulugu.md) aynı
  protokolü taşır. Bir yeni regresyonla toplam 482 test yeşildir; B-008
  kapandı. Bütün olası Unicode dizilerinin biçimsel ispatı iddia edilmez;
  K-111 fuzz/property kalır, bağımsız veri korpusu B-009'dur.

## K-123 — Morfoloji conformance korpusunu derleyiciden bağımsızlaştır (2 Eyl)

- **Sorun:** `zee-tr-1` davranışı güçlü Rust property testleri, tablo snapshot'ı
  ve K-122 semantic kaydıyla korunuyordu; fakat başka dilde yazılacak bir
  derleyicinin aynı sözü Rust modül adlarını ya da test kodunu taklit etmeden
  tüketebileceği sürümlü veri protokolü yoktu.
- **Karar:** Kök `conformance/morfoloji/` alanı implementation'dan bağımsız
  normatif tüketici yüzeyidir. `sema-v1.schema.json` biçimi; `zee-tr-1.json`
  profil kimliği, katman sınırı, K-122 kaydı, ek tablosu, izinli zincirler ve
  beklenen sonuçları taşır. Alanlarda Rust tür/fonksiyon adı bulunmaz.
- **Kapsam:** 27 çözüm/karar vakası tek ve iki katmanlı ekleri, p/nk
  yumuşaması, ikizleşme, sertleşme, ünlü düşmesi, doğrudan exact önceliği,
  A001/A002 ve sıralı bütün yapısal adayları kapsar. 21 üretim vakası yedi tek
  eki, altı iki katmanlı zinciri, ses olaylarını ve üç geçersiz zinciri kapsar.
- **Yürütülebilir bağ:** Veri odaklı Rust regresyonu şema kimliğiyle profil
  metadata'sını, bütün tabloları, aday sırasını, kararları ve üretimleri çalışan
  motorla eşitler. JSON Schema ve profil korpusu K-122'nin Git-geçmişli
  immutable koruğuna dahildir; yayımlanmış profilde sessiz fixture güncellemesi
  yapılamaz.
- **Kanıt ve sınır:** RFC-0018/spec-13,
  [conformance rehberi](../docs/morfoloji-conformance.md) ve kök korpus aynı
  tüketici protokolünü taşır. JSON Schema biçimi tanımlar; Rust'ın typed veri
  yükleyicisi ve açık değişmez kontrolleri mevcut korpusu yürütür. Bir yeni
  regresyonla toplam 483 test yeşildir; B-009 kapandı. Bu kaydın ardından
  B-011, K-124 ile kapatıldı.

## K-124 — Scheduler'ın gözlenebilir sonucunu `zee-esz-1` profiline bağla (2 Eyl)

- **Sorun:** K-090/spec-14 tek iş parçacıklı referans scheduler'ı ve ayrıntılı
  Rust regresyonları deterministikti; fakat gelecekteki ikinci derleyici ya da
  çok çekirdekli runtime için hangi gözlemlerin uyumluluk sözü, hangilerinin iç
  gerçekleme ayrıntısı olduğu makine-okunur ve immutable değildi.
- **Karar:** `zee-esz-1`; bildirimde ortam snapshot'ını, ilk poll'un join'de
  başlamasını, hazır/eşit uyanışta kaynak sırasını, örtüşen beklemeyi, kaynak
  sıralı sonuç bağını, iç ağaç ilerlemesini, hata/son tarih/çıkış iptalini ve
  atomik eylem dilimini 11 maddelik gözlenebilir söz olarak yayımlar.
- **Uyumluluk sınırı:** Çıktı ve ortak IO sırası, sanal geçen süre, dosya son
  durumu ve sonlanma türü/kodu dildir. Future/poll türü, thread sayısı ve host
  scheduler'ı değildir. Çok çekirdek ancak bu gözlemleri birebir koruyan iç
  optimizasyon olabilir; yarış programa sızarsa profil uyumsuzdur.
- **Korpus:** Kök `conformance/eszamanlilik/` JSON Schema ve 10 doğrudan Zee
  kaynak+beklenen gözlem vakası taşır. Snapshot/tembel başlangıç, eşit/farklı
  uyanış, çoklu tur, ortak dosya etkisi, iç görev ağacı, hata/son tarih/çıkış
  iptali ve atomik rollback aynı veri-güdümlü testten geçer.
- **Değişmezlik ve kanıt:** Genel `conformance-korugu.sh`, geçmişte yayımlanmış
  bütün JSON veri/şemalarını ve morfoloji semantic kayıtlarını korur; eski
  morfoloji betiği uyumluluk sarmalayıcısıdır. RFC-0011, spec/14,
  [conformance rehberi](../docs/eszamanlilik-conformance.md) ve bir yeni
  regresyonla toplam 484 test yeşildir; B-011 kapandı. Bu kaydın ardından
  B-012, K-125/ADR-029 ile kapatıldı.

## K-125 — Ondalık ile binary float arasındaki örtük FFI köprüsünü yasakla (2 Eyl)

- **Sorun:** RFC-0012'nin eski taslağı dilde artık bulunmayan `GerçekSayı`yı
  C `double` ile doğrudan eşliyordu. Bu kalıntı, keyfî hassasiyetli Ondalık'ın
  görünmeden binary yuvarlamaya düşebileceği yanlış bir gelecek tasarımıydı.
- **Karar:** ADR-029 ile Ondalık↔`float`/`double`/binary32/binary64 örtük
  eşlemesi yasaktır. Binary kayan nokta genel Zee türü değil, yalnız gelecekteki
  işaretli FFI adaptörünün sınır temsili olabilir. Stage 0 FFI sunmaz.
- **Gelecek kapısı:** Köprü eklenirse deklarasyon ve çağrıda görünür `kayıplı`
  işareti, `Sonuç` dönüşü ve IEEE 754 yuvarlama, signed zero, subnormal, taşma,
  NaN/sonsuzluk ile binary64→Ondalık kanonikleştirme sözleşmesi ister. Kesin
  Türkçe yüzey golden/usability kararı olmadan parser'a girmez.
- **Kanıt:** `ffi_sinir_testi.rs`, `Tur`/`Deger` enumlarında binary varyant
  olmadığını, eski taslak FFI kaynağının derlenmediğini, `0,1+0,2=0,3`
  exactlığını ve RFC/ADR metin kapısını birlikte korur. RFC-0012 taslak kalır;
  RFC-0013/spec-16 sınırı normatif taşır. Bir yeni regresyonla toplam 485 test
  yeşildir; B-012/V1-P0-28 kapandı. Bu kaydın ardından gelen B-050,
  K-126/ADR-030 ile kapatıldı.

## K-126 — Bütün AST ifadelerini kesin kaynaklandır (2 Eyl)

- **Sorun:** K-108 HIR'da kaynak kökenini zorunlu kılmıştı fakat değişken
  dışındaki eski AST varyantları yalnız satır zarfına düşüyordu. Bileşik
  ifadenin sütun/uzunluğu tanı ve LSP için güvenilir değildi.
- **Karar:** ADR-030 ile parser'ın her yaprak ve bileşik ifadesi ayrı
  `Ifade::Kaynakli` + sıfır olamayan `AstKaynakAraligi` taşır. Aralıklar
  Unicode karakteri cinsindedir; çocuk ifade ebeveyn zarfını paylaşmaz.
- **Sentetik sınır:** `her sayı için` örtük `sayılar` ifadesi uydurma `1:1`
  yerine onu doğuran `sayı` tokenına bağlanır. Raw v0 embedding spansiz AST
  kurabilir ama parser invariantı bunu kabul etmez; checker panic yerine
  kodlu iç tanı verir.
- **Tüketim:** Checker AST aralığını HIR'a birebir aktarır; invariant eksik,
  iç içe veya uyuşmayan zarfı reddeder. Eski `1:1` checker tanısı ilgili kesin
  düğüme yükselir. LSP işlem/yapı kullanımını bütün satırda değil semantic
  ifade aralığında arar ve çağrı/`yeni` kuyruğundaki son canonical adı seçer.
- **Mimari:** Token→AST kaynaklandırma `ayristirici/kaynak.rs`, tanı
  yükseltme `cozumleyici/kaynak.rs` sahibidir; mevcut handler satır bütçeleri
  büyütülmedi.
- **Kanıt:** Bileşik+yaprak AST/HIR aralıkları, spansiz AST reddi, kesin tür
  tanısı, örtük çoğul kökeni ve aynı yazımlı argüman/işlem kuyruğu beş yeni
  regresyonda korunur. Toplam 490 test yeşildir; B-050 ve V1-P0-18'in kesinlik
  dilimi kapandı.

## K-127 — Dış dünya yetkisini proje sahibine ve iki uygulama kapısına bağla (2 Eyl)

- **Sorun:** Intrinsic kaydı ağ/donanım/web/kripto ihtiyacını biliyordu ama
  proje hangi yetkiyi verdiğini bildirmiyor; dosya ve çocuk modu ayrı runtime
  davranışları taşıyordu. Elle yazılmış native istemci HTTPS, exact origin,
  DNS sonrası private/metadata IP ve redirect yetki sınırına sahip değildi.
- **Karar:** RFC-0024/ADR-031/spec-23 ile sekiz kararlı yetkinlik ve exact
  `ağ_hedefleri` manifest alanı oldu. Checker ana program, test ve çağrılmayan
  işlem gövdelerini ayrı `yetkinlik` katmanında tarar; ihlal kesin kaynaklı
  T054'tür. Paket ana politika alt kümesi değilse grafik P015'tir.
- **Runtime:** `PolitikaliIo`, eski `GuvenliIo` çocuk API'sini koruyan genel
  ikinci kapıdır; native adaptör aynı policy'yi ayrıca uygular. Proje dosyası
  göreli/canonical kök ve symlink sınırındadır. Uygulama izni süreç üst
  sınırıdır; paket başına OS sandbox/tenant izolasyonu iddia edilmez.
- **Outbound:** Exact `ureq 3.4.0` yalnız rustls özelliğiyle kullanılır. Public
  internet HTTPS; private/loopback+düz HTTP ayrıca `yerel-ağ` ister;
  metadata/link-local ve IANA public olmayan özel-kullanım/geçiş önekleri
  daima kapalıdır. DNS cevabındaki her soket adresi bağlantı öncesi denetlenir;
  redirect ve ortam proxy'si kapalı, varsayılan 30 saniye ile 64 KiB header/
  8 MiB zarf korunur. Zee TLS yazmaz.
- **Kanıt:** Manifest bağımlılıkları, paket yükseltmesi, ölü kod, kesin T054,
  origin/port, bozuk URL, public/private/metadata DNS, symlink kaçışı, explicit
  loopback, geçiş öneki, redirect ve body limiti olumsuzlarıyla toplam 501 test yeşildir. P015/T054 ile
  147 etkin + 3 ayrılmış tanı; RFC/ADR/spec indeksleriyle 76 numaralı belge
  tazedir. B-023/B-049 ve V1-P0-29 kapandı.

## K-128 — Atomik replace metadata'yı da emanet kabul eder (2 Eyl)

- **Sorun:** K-084 yeni inode'a yalnız içerik ve izin bitlerini taşıyordu;
  Unix owner/group, ACL/xattr/security label ve Windows DACL/named stream
  sessizce kaybolabilirdi.
- **Karar:** ADR-032 ile hedef normal dosya değilse replace reddedilir.
  Linux/macOS mode+uid+gid zorunlu korunur. Linux görünür xattr/ACL/security
  alanını 64 KiB ad/değer + 1 MiB toplam bütçeyle; macOS ACL+xattr'ı
  `fcopyfile` ile descriptor'dan descriptor'a taşır. Taşınamayan alan commit
  öncesi hatadır; Tier-1 dışı Unix mevcut dosya replace'i `Unsupported` olur.
- **Windows:** `ReplaceFileW` metadata/ACL merge hata-yoksayma bayrağı olmadan
  ve aynı klasör kurtarma yedeğiyle kullanılır. Nadir kısmi taşıma hatası eski
  hedefi write-through `MoveFileExW` ile geri kurar; ilk yaratma eski atomik
  yoldadır.
- **Kanıt:** macOS mode+uid+gid, binary xattr ve gerçek ACL; Linux xattr,
  Windows NTFS named stream ve Unix symlink için platform-koşullu beş yeni
  regresyon tanımlıdır. Eski 10 atomiklik/yarış/hata testi yeşildir. Envanter
  506 test, 77 numaralı belge; B-048/V1-P0-30 kapandı.

## K-129 — Kaynak sınırı tek değişmez güvenlik profilidir (2 Eyl)

- **Sorun:** Ağ, LSP, registry ve IO izi kendi limitlerini taşısa da kaynak,
  token, toplam proje, runtime adımı, normal çıktı, koleksiyon, görev, veri
  dosyası ve LSP toplam belleği tek sahibin altında değildi.
- **Karar:** RFC-0025/ADR-033/spec-24 ile `KaynakSinirlari` kullanıcıdan
  değiştirilemeyen resmî profil oldu. Tek kaynak 8 MiB/1 milyon token; proje
  4.096 dosya/128 MiB; runtime 10 milyon adım, C019 ile 500 çağrı, 1 milyon
  koleksiyon öğesi, 1.024 görev ve 16 MiB/100 bin çıktı olayı taşır. Veri
  dosyası 16 MiB bounded reader'dan geçer.
- **LSP:** 256 açık belge, 128 MiB toplam metin ve 8 MiB outbound mesaj.
  Reddedilen güncelleme önceki belgeyi değiştirmez; aşım S045 veya JSON-RPC
  `-32001` olur.
- **Kanıt:** Bellek kaynağı, çıktı taşması, görev grubu, sparse büyük dosya,
  açık belge ve outbound mesaj sınırları yedi yeni regresyondur. Toplam 513
  test, 149 etkin + 3 ayrılmış tanı ve 80 numaralı belge tazedir.
- **Açık kapsam:** Canlı değer/metin grafiğinin toplam heap muhasebesi,
  bağlantı kotası ve eski domain sabitlerinin tamamının profile taşınması
  B-025'in ikinci dilimidir.

## K-130 — Değer/metin heap'i ve bağlantılar da aynı zarfta (2 Eyl)

- **Sorun:** K-129 öğe ve çıktı sayısını sınırlıyordu; fakat tek bir metin
  dönüşümü, saklanan iç metinlerle büyüyen değer grafiği, görev ortamı klonları
  ve aynı süreçteki bağlantılar hâlâ ayrı bir toplam zarf taşımıyordu.
- **Karar:** Tek metin 16 MiB'tır. Ortam/list/sözlük yazımları, işlem
  parametreleri, gezme geçicisi ve görev snapshot'ları çalışma/istek başına
  iade edilmeyen yaklaşık 64 MiB saklama fişi tüketir. Bu muhafazakâr sayaç
  allocator profiler'ı değildir; fazla sayarak canlı grafiğe üst sınır olur.
- **Allocation sırası:** Birleştirme/değiştirme, HTML kaçışı, değer metni,
  keyfî hassasiyetli sayı/para, JSON ve CSV bütçeli büyür; bilinen taşma sonuç
  tahsisinden önce C024'tür.
  C023'ün eski anlamı değiştirilmedi, yeni olay append-only C024 aldı.
- **Ağ:** Outbound istek ve kabul edilmiş inbound soket aynı süreç-geneli 64
  RAII iznini kullanır. Her başarı/hata/erken red yolu izni Drop ile bırakır;
  outbound C018 nedeni, inbound 503 üretir.
- **Kanıt:** Metin büyümesi, saklanan değer zarfı, görev ortamı klonunun
  preflight reddi ve bağlantı izin iadesi dört yeni regresyondur. Envanter
  517 test, 150 etkin + 3 ayrılmış tanı, 80 numaralı belgedir.
- **Açık kapsam:** B-025'in son dilimi eski registry/tedarik/IO izi/oturum/LSP
  sabitlerini davranış değiştirmeden `KaynakSinirlari` içine taşıyacaktır.

## K-131 — Kaynak limitlerinin tek sayısal sahibi (2 Eyl)

- **Sorun:** K-129/K-130 ortak profili kurmuş olsa da HTTP/ağ, oturum, IO izi,
  LSP, paket/registry, tanı ve kalıcı dosya modülleri aynı değerleri yerel
  sayılarla sahiplenmeye devam ediyordu; değişiklikte sessiz drift mümkündü.
- **Karar:** `KaynakSinirlari` domain görünümleri bütün sayısal varsayılanların
  tek sahibidir. Mevcut public/private sabit adları yalnız bu profile bağlı
  geriye uyum alias'ıdır; çalışma davranışı ve sınır değerleri değişmedi.
  Bounded reader `kaynak_sinirlari/okuma.rs`, domain tipleri
  `kaynak_sinirlari/profiller.rs` sahibine ayrıldı.
- **Kanıt:** Mimari regresyon on bir tüketici modülün ortak profili kullandığını;
  kök/profil/okuma dosyalarının 260/300/120 satır bütçesinde kaldığını denetler.
  Envanter 518 test, 150 etkin + 3 ayrılmış tanı ve 80 numaralı belgedir.
- **Canlı inceleme bulgusu:** LSP outbound 8 MiB üstünde `-32001` üretir,
  fakat yanıt önce bütçesiz `String` olarak kurulup sonra ölçülür. Bu nedenle
  B-025 dürüstçe kapanmadı; K-132 üretim-sırası bütçesini bağlayacaktır.

## K-132 — LSP JSON'unu üretim sırasında bütçele (2 Eyl)

- **Sorun:** LSP outbound gövdesi 8 MiB üstünde `-32001` üretse de yanıt,
  rename düzenleme listesi ve diagnostics önce bütçesiz `String`/`Vec<String>`
  olarak kuruluyor; limit ancak pahalı tahsisten sonra ölçülüyordu.
- **Karar:** `lsp/cikti.rs` tek `SinirliJson` sahibidir. Ham, biçimli ve JSON
  kaçışlı her parça append öncesi merkezî 8 MiB bütçesini tüketir. Initialize,
  diagnostics, completion, hover, definition, rename, shutdown ve hata yolları
  aynı yazıcıdan geçer. Rename yalnız semantic aralık planını toplar; yeni
  metin ve düzenleme JSON'u birer birer yazılır.
- **Hata davranışı:** Kısmi gövde yayımlanmaz. Kimlikli taşma `-32001`,
  bildirim taşması bounded `window/logMessage` olur; dev kimlik hata zarfına
  sığmazsa `id:null` ile fail-closed geri dönülür.
- **Kanıt:** Sınırdaki append kabul/bir bayt fazlası red, JSON kaçış
  genişlemesi, taşmanın RPC hatasına dönüşmesi ve bütün outbound yolların
  yeniden ayrıştırılabilen/bütçe içi JSON üretmesi üç yeni regresyondur.
  Envanter 521 test, 150 etkin + 3 ayrılmış tanı ve 80 numaralı belgedir.
- **Sonuç:** B-025 ve V1-P0-31 kapandı. Sıradaki makine işi B-026
  cancellation-safety audit'idir.

## K-133 — Yan etki öncesi iptal kapıları (2 Eyl)

- **Sorun:** Blok/cümle girişindeki deadline denetimi ile gerçek IO çağrısı
  arasında argüman değerlendirme veya scheduler sıra verme zamanı geçebildi.
  Özellikle görev HTTP'si kalan süreyi sıra vermeden önce hesaplıyor; kardeş
  saati deadline'a taşısa bile adaptöre eski süreyle girebiliyordu.
- **Karar:** Çıktı/girdi, dosya, sunucu, yanıt/yönlendirme, çerez/oturum,
  eyleyici, CSRF, parola doğrulama, rastgelelik ve `eylem` transaction
  başlangıcı gerçek etkinin hemen önünde `son_tarihi_denetle` kapısından
  geçer. HTTP kalan süresi scheduler turundan sonra yeniden hesaplanır.
- **Kanıt:** Sanal saatli iki regresyon, deadline'ın dosya yazma kapısında
  dolduğunda dosyanın değişmediğini ve HTTP görevi sıra verdikten sonra
  dolduğunda adaptör çağrısının hiç başlamadığını doğrular. Envanter 523 test,
  150 etkin + 3 ayrılmış tanı ve 80 numaralı belgedir.
- **Audit sonucu:** Temp dosya/kilit/deadline/bütçe nöbetçilerinin `Drop`
  temizliği ve `eylem` rollback'i doğrulandı. B-026 kısmen kapalıdır; web
  isteğinin oturum mutation'ı ile henüz gönderilmemiş yanıtını birlikte
  commit/rollback eden yaşam döngüsü K-134'e kaldı.

## K-134 — Web isteği tek transaction yaşam döngüsüdür (2 Eyl)

- **Sorun:** Gerçek adaptör `yanıtını gönder` anında TCP'ye yazıyor, giriş/
  çıkış ve CSRF ise oturum deposunu hemen değiştiriyordu. Rota daha sonra
  timeout veya runtime hatası alırsa erken 200 yayımlanmış, yeni oturum kaydı
  çerez ulaşıp ulaşmadığından bağımsız kalmış olabiliyordu.
- **Karar:** `GirdiCikti` host-only request tamamla/geri-al kancaları kazandı.
  `yorumlayici/web_istek.rs` rota seçimi, güvenlik önsözü, taze ortam ve
  deadline yürütmesinin tek sahibidir. İlk yanıt/yönlendirme tamponlanır;
  session ve çerez mutation'ı yalnız rota başarıyla bitip gerçek socket yazımı
  eksiksiz tamamlanınca birlikte commit edilir. Deadline/runtime/socket hatası
  veya yanıtsız rota istek başı snapshot'ına döner.
- **Trace sınırı:** İki yaşam döngüsü kancası yeni kullanıcı IO'su değildir;
  `zee-io-1` şema-1'in 27 gözlenebilir işlemini değiştirmeden recorder iç
  adaptöre iletir.
- **Kanıt:** Hermetik timeout ve runtime hata regresyonları erken yanıt,
  giriş çerezi ve tahmin edilen yeni oturumun sızmadığını doğrular. Gerçek TCP
  testleri commit öncesi sıfır bayt ve socket yazma hatasında sıfır oturum
  kanıtıdır. Envanter 527 test, 150 etkin + 3 ayrılmış tanı ve 80 numaralı
  belgedir. B-026 kapandı.

## K-135 — Registry taşıma, cache ve kalıcı rollback zinciri (2 Eyl)

- **Sorun:** K-095 imza/metadata kararını byte dizileri üzerinde doğru
  veriyordu; fakat ağı, doğrulanmadan önce kullanılabilir cache'e yazmayı,
  süreç yeniden başladığında monoton geçmişi ve offline hit/miss'i yöneten
  çalışan bir istemci yoktu. Böylece B-029'un gerçek dünya sınırı açıktı.
- **Karar:** `registry/istemci.rs` tek güncelleme transaction'ını sahiplenir.
  `tasima.rs` yalnız yol taşımayan HTTPS origin, güvenli ASCII statik yol,
  redirect/proxy kapatma ve ortak DNS/IP korkuluklarını uygular. Timestamp
  bağından sürümlü snapshot, snapshot bağından sürümlü targets alınır; en çok
  64 ardışık çift-eşik root rotasyonu izlenir. Hedef URL'si adla değil SHA-256
  ile adreslenir.
- **Kalıcılık:** Tam root→metadata→targets→yayıncı zinciri geçmeden hiçbir
  nesne kullanılabilir cache'e girmez. Doğrulanan metadata ve dört hedef
  `nesneler/sha256/<özet>` altında salt-okunur tutulur. İlk root kimliği,
  etkin root, kabul zamanı ve rol sürüm+özetleri kanonik `durum-v1.json`da;
  immutable nesnelerden sonra atomik karşılaştır-ve-yaz ile yayımlanır.
  Yarışan bayat süreç yeni durumu ezemez; başarısız zincir durumu değiştirmez.
- **Offline ve hata:** Çevrimdışı kip ağ/duvar saati kullanmadan yalnız son
  kabul zamanında geçerli tam zinciri ve exact hedefi yeniden doğrular. Eksik,
  boyutu/özeti değişmiş ya da farklı root pinine ait cache P016'dır; ağ açıkken
  de bozuk nesne sessizce iyileştirilmez. P013/P014 güven/politika ayrımı
  korunur.
- **Kanıt:** Altı regresyon; doğrulama öncesi sıfır cache/durum, online→offline
  hit, bozuk nesne reddi, offline miss, süreçler arası rollback durumunun
  byte-byte korunması ve HTTPS/statik yol kapısını doğrular. Envanter 533 test,
  151 etkin + 3 ayrılmış tanı ve 80 numaralı belgedir. B-029 exact proje
  bildirimi/kilit/CLI entegrasyonu için kısmen açıktır.

## K-136 — Exact registry paketi proje sözleşmesine bağlandı (2 Eyl)

- **Sorun:** K-135 güvenilmeyen ağdan doğrulanmış byte'ı kalıcı cache'e kadar
  getiriyordu; fakat hangi registry/root kimliğinin projeye ait olduğu,
  `ad@X.Y.Z` seçiminin kilide nasıl taşınacağı ve kullanıcı komutlarının ne
  zaman ağ açabileceği henüz yürütülebilir proje sözleşmesi değildi.
- **Karar:** `proje.dil`; yol/sorgusuz HTTPS origin, pozitif ağ dışı root
  sürümü, `sha256:` root özeti ve exact `uzak_bağımlılıklar` alanlarını birlikte
  taşır. Eksik/bölünmüş/geçersiz yapı P017'dir. Yerel ve uzak paketler aynı
  kökenli grafikte ad, doğrudanlık, public ABI ve yetkinlik sınırına tabidir.
- **Kilit ve politika:** `proje.kilit` v3 ilk/etkin root ile bütün metadata
  sürüm+özetlerini, yayıncıyı, `.zep`/SBOM/provenance/yayın özetlerini,
  yanked durumunu ve kritik duyuruları sabitler. `--yanked-kabul` ve
  `--kritik-kabul` boş olmayan insan gerekçesi ister; karar exact root+paket
  kimliğine bağlı kilit kaydı olarak korunur.
- **Ağ ve kurulum:** `dil ekle ad@X.Y.Z` ile `dil kilitle` açıkça çevrimiçi,
  `--çevrimdışı` ağsızdır. `dil paketler` varsayılan ağsız, yalnız `--yenile`
  çevrimiçidir. Çalıştırma/denetleme/test/LSP sessiz ağ açmaz. Doğrulanmış
  `.zep`, proje-local `.zee/paketler/sha256/<özet>` altında görünmez geçicide
  açılır, exact ağaçla yeniden doğrulanır, atomik adlandırılır ve salt-okunur
  yapılır.
- **Atomiklik:** Uzak ekleme/çıkarma aday grafiği yazmadan önce bütünüyle
  çözer. Manifest ve kilit iki dosyalı geri alma sözleşmesiyle güncellenir;
  yarım aday görünür olmaz.
- **Kanıt:** Exact manifest/P017 olumluları ve yorum koruyan güncelleme, iki
  CLI seçenek testi, kritik kabul anahtarının güncel sıralı duyuru kümesine
  bağlanması ve gerçek imzalı `.zep`+root/timestamp/snapshot/targets zincirinin
  online hazırlanıp offline grafikte gerçek derlenmesi altı yeni regresyondur.
  Yeni kritik duyuru eski gerekçeyi kullanamaz; bozulmuş kurulu kaynak P016 ile
  reddedilir. Envanter 539 test,
  152 etkin + 3 ayrılmış tanıdır. B-029 ve V1-P1-07 kapandı.

## K-137 — Production web durumu süreçler arasında ortaktır (2 Eyl)

- **Sorun:** K-106 oturum kotasını, K-134 istek transaction'ını doğru
  kurmuştu; fakat production oturumu process belleğindeydi. Worker geçişi ve
  restart login'i kaybediyor, logout/revoke yayılmıyor, endpoint/CSRF ve
  CPU-pahalı Argon2id için ortak oran sınırı bulunmuyordu. Ayrıca loopback
  proxy arkasında socket eşinin `127.0.0.1` olması gerçek istemci kimliği
  değildi.
- **Karar:** `WebGuvenligi`, saklama ayrıntısını `Depo` sınırından tüketir.
  Deneysel kip süreç içi adaptörü korur; `--web-proxy` proje kökündeki
  `.zee/web-durumu-v1.json` kalıcı adaptörünü kullanır. Oturum mutation'ı ve
  oran sayacı süreçler arası kilit + eski-byte karşılaştırması + atomik replace
  CAS döngüsüdür. Bozuk şema, symlink, kaynak aşımı ve güvenle yönetilemeyen
  kapasite fail-closed 503'tür.
- **Kimlik ve oran:** Proxy istemcinin bütün proxy başlıklarını silip tek-hop
  `Forwarded: for=<IP>;proto=https;host=<host>` kurar. Runtime IP'yi
  `IpAddr` ile kanonikler; XFF kimlik değildir. İstemci+yöntem+sorgusuz path
  100/60 sn, CSRF 60/60 sn, Argon2id doğrulaması 5/300 sn ortak sabit
  penceredir. Altıncı parola denemesi Argon2id çalışmadan 429 olur; etkin sayaç
  tahliyesiyle eşik delinemaz.
- **Worker modeli:** Stage 0 process başına bilinçli tek worker'dır.
  Production concurrency'si aynı depoyu paylaşan N ayrı `dil` süreci ve
  `--web-worker-port` ile farklı loopback portlardan kurulur. Aynı process
  içinde thread-pool sözü verilmez; çok-hostlu harici backend ileriki
  adaptördür.
- **Kanıt:** İki gerçek CLI süreci A login→B authenticated GET, A restart,
  B logout→A revoke ve iki sürece dağıtılmış altıncı yanlış parola denemesinin
  429 olmasını uçtan uca doğrular. Bağımsız depo testleri restart/expiry,
  koşullu rollback, bozuk JSON/symlink, Unix 0700/0600 izinleri ve on threadde
  yüz artışın atomik eşiğini korur. Envanter 547 test, 152 etkin + 3 ayrılmış
  tanı ve 81 numaralı belgedir. B-046 kapandı.

## K-138 — LSP JSON-RPC sınırı protokol-kesindir (2 Eyl)

- **Sorun:** K-107 kaynak zarfını kapatmıştı fakat mini JSON parser sayıyı
  genel karakter toplama + `f64` ile çözüyor, duplicate alanı kabul ediyor ve
  bozuk JSON'u sessizce yutuyordu. JSON-RPC `id` yazılırken kesir/büyük sayı
  `i64`'e çevrilerek değişebiliyordu; sözdizim hatasıyla bozuk request zarfı
  aynı davranışı alıyordu.
- **Karar:** `lsp/json.rs` RFC 8259 sayısını durum makinesiyle doğrular ve
  kayıpsız özel lexeme olarak taşır. `NaN/Infinity`, kuralsız sayı ve Unicode
  kaçışından sonra aynı ada çıkan duplicate alan fail-closed reddedilir.
  Tek-nesne LSP profilinde UTF-8/sözdizimi `-32700`, bozuk JSON-RPC zarfı
  `-32600`, bilinmeyen method `-32601`, bozuk params `-32602`dir. Notification
  response üretmez; tam okunmuş UTF-8 dışı gövdeden sonra sunucu devam eder.
- **Mimari:** Ayrıştırıcı 300 satır, outbound yazıcı 120 satır bütçeli ayrı
  sahiplerdir. LSP kökü semantic dispatch'i taşır; sayı/Unicode/duplicate
  politikası köke geri alınamaz.
- **Kanıt:** RFC sayı olumlu/olumsuz ve `serde_json` differential korpusu,
  escaped duplicate alan, dört standart hata kodu, bildirim sessizliği,
  kayıpsız kesirli/çok büyük kimlik ve UTF-8 sonrası devam regresyonları
  eklendi. Gerçek ardışık çerçeve ve mimari sahiplik testiyle envanter 556
  test, 152 etkin + 3 ayrılmış tanı ve 82 numaralı belgedir. B-051 kapandı.

## K-139 — Web proxy origin'i tek kanonik güvenlik tipidir (2 Eyl)

- **Sorun:** Outbound allowlist `AgHedefi` ile sıkı DNS/IPv6/port semantiği
  taşırken `--web-proxy` daha gevşek, CLI'a özel `GuvenliOrigin` parser'ını
  kullanıyordu. Ham Host/Origin karşılaştırmaları varsayılan port ve kanonik
  kimliği tekilleştirmiyor; loopback güveni kabul edilen peer üzerinde ayrıca
  doğrulanmıyordu.
- **Karar:** CLI'a özel origin tipi kaldırıldı. `--web-proxy`, `Host`, tek-hop
  `Forwarded host` ve unsafe `Origin` aynı `AgHedefi` parser'ından geçer;
  yalnız HTTPS, geçerli ASCII DNS veya köşeli IPv6 ve 1–65535 port kabul edilir.
  DNS harfi ve varsayılan `:443` kanoniklenir. Listener sabit `127.0.0.1`e bind
  eder; production isteğinin socket peer'i ayrıca loopback değilse 403 döner.
- **Mimari:** Tek parser 100 satır bütçeli `yetkinlik/origin.rs` sahibindedir.
  Mimari test ikinci `GuvenliOrigin` tipinin, ortak kurucuların ve açık
  loopback bind+peer kapısının drift etmesini engeller.
- **Kanıt:** DNS harf farkı, açık varsayılan port, farklı port, IPv6, HTTP,
  kullanıcı bilgisi, yol, bozuk etiket/ayraç/port ile loopback ve dış peer
  olumluları/olumsuzları eklendi. Envanter 558 test, 152 etkin + 3 ayrılmış
  tanı ve 83 numaralı belgedir. B-052 kapandı.

---

## Sonraki adım

Korpus 10 öğrenci + 5 profesyonel usability oturumuna (Hafta 12 hedefi, erkeni
Hafta 2'de kağıt üstünde) sesli okutulacak; her kayıt için "doğal mı /
deterministik mi / öğrenilebilir mi / savunulabilir mi" dört soru süzgeci
işletilip durumlar güncellenecek. `AÇIK` kayıtlar ilgili RFC'lere taşınacak.
Makine hattında K-139 web proxy origin'ini ortak `AgHedefi` kimliğine ve
loopback bind+peer değişmezine bağlayarak B-052'yi kapattı. Sırada K-140 ile
B-053 byte tabanlı ve fuzz kanıtlı HTTP istek ayrıştırıcısı vardır.
