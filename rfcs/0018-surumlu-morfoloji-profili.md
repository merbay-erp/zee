# RFC-0018 — Sürümlü Morfoloji Profili

- **Durum:** geçici kabul — K-089 gerçeklendi; K-111 property/fuzz, K-122
  immutable uyumluluk, K-123 bağımsız conformance kapısını bağladı
- **Tarih:** 1 Eylül 2026
- **İlgili kararlar:** K-011, K-041, K-049, K-061, K-072, K-089, K-111,
  K-122, K-123; B-008/B-009, V1-P1-02, V1-P0-21
- **Normatif karşılık:** spec/03 ve spec/13
- **Gerçekleme:** `compiler/src/morfoloji.rs`; `morfoloji_testi.rs`,
  `morfoloji_conformance_testi.rs`, `lsp_testi.rs`, `proje_testi.rs`

## Sorun

Zee'de Türkçe ekler tanımlayıcının parçasıdır. Bu nedenle ek ayıklamak yalnız
bir editör kolaylığı değil, kaynak programın hangi ada bağlandığını belirleyen
semantiktir. K-089 öncesinde çözümleyici ile LSP üreteci ayrı yüzey listeleri
tutuyor; iki katmanlı çözüm tek katmanlı üretimle simetrik davranmıyordu.
Derleyici güncellemesiyle ek listesine sessizce yapılacak bir ekleme, eski bir
kaynakta A001'i çözüme veya tek çözümü A002'ye çevirebilirdi.

## Karar

1. Zee v1 morfolojisi **`zee-tr-1`** adlı açık bir profildir.
2. Soyut ekler, kabul edilen yüzeyler, en çok katman sayısı ve kanonik üretim
   tek tabloda tutulur. Çözümleyici, LSP ve inceleme CLI'ı aynı modülü kullanır.
3. Doğrudan ad eşleşmesi her zaman önce kazanır. Ekli çözümde sıfır kapsam
   eşleşmesi A001, bir eşleşme çözüm, birden fazlası A002'dir. Sözlük veya
   olasılıksal/heuristik seçim yoktur.
4. v1 en çok iki katman çözer. İki katmanın iç eki üçüncü tekil iyelik,
   dış eki belirtme/tamlayan/yönelme/ayrılma/bulunma/araçtır.
5. Üreteç her kök+ek kimliği için tek **kanonik** Zee yüzeyi üretir. Türkçedeki
   sözlüksel ünlü düşmesi ve ikizleşme biçimleri geriye doğru kabul edilir;
   sözlük taşımayan üretecin onların özgün yazımını tahmin etmesi gerekmez.
   Değişmez, `üret → çöz` yönünde kök ve soyut ek zincirinin korunmasıdır.
6. `proje.dil`, `morfoloji "zee-tr-1" olsun` ile profili sabitleyebilir.
   Eski bildirimin yazmaması `zee-tr-1` demektir. Bilinmeyen profil P011 ile
   fail-closed reddedilir. Yeni proje iskeleti alanı açık yazar.
7. `proje.kilit` sürüm 3, ana proje ve her yerel/uzak paket için morfoloji profilini
   kaydeder. `dil sürüm` etkin profili, `dil morfoloji [kelime]` profil
   tablosunu veya bütün yapısal çözümleri gösterir.
8. Her yayımlanmış profil, compiler kaynak ağacından bağımsız sürümlü JSON
   conformance verisi taşır. Veri; profil tablosunu, yüzeyin sıralı bütün
   kök+ek çözümlerini, kapsam içindeki kök/A001/A002 kararını ve kanonik
   üretimi içerir. Şema Rust iç adlarına bağlı değildir.

## Sürümleme ve uyumluluk

Aşağıdakiler mevcut profil içinde **YASAK** kırıcı değişikliklerdir:

- yüzey eklemek, kaldırmak veya başka soyut eke taşımak;
- izin verilen zincirleri ya da azami katman sayısını değiştirmek;
- kök geri-çevirme veya kanonik üretim kuralını değiştirerek çözüm kümesini
  değiştirmek;
- doğrudan eşleşme/A001/A002 karar sırasını değiştirmek.

Böyle bir değişiklik yeni profil kimliği (`zee-tr-2`) ister. Yeni profil aynı
derleyicide bilinçli geçiş dönemiyle desteklenmeden varsayılan yapılamaz.
Profil varsayılanının değişmesi ana dil sürümü/edition sınırıdır; paket grafiği
farklı profilleri sessizce karıştıramaz.

`zee-tr-1`in tablo ve kapsanan üretim/çözüm davranışı, şema
`zee-morfoloji-uyumluluk-v1` kanonik akışının SHA-256 kaydıdır. Yayımlanmış
`morfoloji-zee-tr-1.sha256` fixture'ı güncellenemez veya silinemez. Davranış
değişikliği testi, fixture güncellemesi ise Git-geçmişli CI koruğunu kırar;
tek geçerli yol yeni `zee-tr-N` kimliği ve yeni fixture eklemektir.

## Yürütülebilir kanıt

- Profilin bütün tablosu snapshot'tır; sessiz tablo değişikliği testi kırar.
- K-122, 4.096 kökün 53.248 üretim+çözüm vektörü ile 11 ham sınır yüzeyini
  tek SHA-256 kaydında dondurur. `dil morfoloji --uyumluluk` kaydı gösterir;
  geçmişte yayımlanmış profil fixture'larını değiştirmek CI'da yasaktır.
- K-123, depo kökündeki JSON Schema + `zee-tr-1.json` korpusunda 27
  çözüm/karar ve 21 üretim vakasını ikinci compiler'lara açar; Rust bootstrap
  aynı veriyi yalnız tüketen veri-güdümlü conformance testiyle doğrular.
- Düzenli kök korpusu × bütün tek ekler için `üret → çöz` property testi vardır.
- Aynı kök korpusu × bütün geçerli iyelik zincirleri iki katmanlı property
  testinden geçer.
- Ünlü/ünsüz, yumuşayan ve tek-heceli kök sınıflarının kanonik tek/iki
  katmanlı çıktıları golden beklentilerle sabittir.
- Yumuşama, nk→ng, ikizleşme ve ünlü düşmesi ayrı ters-dönüş korpusudur.
- `payı`, `sayacı`, `fiyatıyla`, `zarından` belirsizlik korpusu, adayların
  tamamını ve A002 sonucunu kilitler.
- LSP önce başarılı checker HIR'ındaki `SymbolId` ile tek semantic varlığı
  seçer; yalnız o kimliğin tek/iki katmanlı biçimlerini yeni kökün
  ünlü/ünsüz yapısına göre yeniden üretir. A002 veya başka derleme hatasında
  metin benzerliğiyle tahmin yapmaz (K-120/B-041).
- Bildirim, kilit ve CLI profil görünürlüğü entegrasyon testlidir.
- K-111 geniş katmanı 4.096 deterministik kökün bütün geçerli zincirlerini,
  2.048 bütün-aday A002 kararını ve NFC olumlu/NFD→S029 olumsuzlarını stable
  testte yürütür. Ayrı gecelik libFuzzer hedefi byte girdiden geçerli Zee kökü
  üretip aynı değişmezleri mutation ile arar; işletim ayrıntıları
  [morfoloji doğrulama rehberindedir](../docs/morfoloji-dogrulama.md).

## Reddedilen yollar

- **En olası kökü seçmek:** aynı kaynak bağlama/kapsama göre sessiz anlam
  değiştirir; manifesto determinizmine aykırıdır.
- **Ekleri lexer anahtar kelimesi yapmak:** tanımlayıcının doğal bütünlüğünü
  bozar ve keyfi kullanıcı adlarında çalışmaz.
- **Sözlükle bütün Türkçeyi tahmin etmek:** özel adları, yeni kelimeleri ve
  alan terimlerini dışlar; dil anlamını büyüyen harici veriye bağlar.
- **Derleyici sürümüne örtük bağlamak:** eski projenin yeni derleyiciyle başka
  ada bağlanmasına izin verir; yeniden üretilebilir paket sözünü bozar.

## Dört soru süzgeci

1. **Doğal mı?** Ekler kaynakta doğal Türkçe biçiminde kalır.
2. **Deterministik mi?** Profil, aday kümesi ve A002 kuralı tam sabittir.
3. **Öğrenilebilir mi?** Çocuk yalnız doğal eki yazar; profil ayrıntısı proje
   ve araç katmanında gerektiğinde görünür.
4. **Savunulabilir mi?** Profesyonel kullanıcı aynı kaynak/paket grafiğinin
   yıllar sonra hangi morfolojiyle çözüleceğini kilitleyebilir.
