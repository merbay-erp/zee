# ADR-023 — AST/HIR invariant doğrulama politikası

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-112, B-017, V1-P0-22

## Bağlam

ADR-015 derleyici fazlarını ayırdı; ADR-016 typed HIR'ı, ADR-020 zorunlu
kaynak aralığını kurdu. Buna rağmen parser AST'sinin checker alanlarını boş
taşıdığı ve başarılı checker çıktısındaki her AST ifadesinin tam bir HIR
kaydına karşılık geldiği yürütülebilir tek bir kapıyla doğrulanmıyordu.

Bu boşluk özellikle yerinde AST dönüşümlerinde tehlikelidir. Bir alt düğümü
klonlayıp dış varyantı değiştirmek kaynak programın davranışını koruyor gibi
görünebilir; fakat adresle kurulan private AST→HIR eşlemesinde eski kaydı
yetim bırakabilir. Eksik kimlik, yanlış canonical ad veya imkânsız düğüm
biçimi ancak sonraki runtime/lowering aşamasında görülebilirdi.

## Karar

Derleyici `invariant` modülünde iki açık doğrulama yüzeyi taşır:

1. `AyristirilmisAst::invariantleri_dogrula`, parser çıktısında çözülmüş ad,
   `SymbolId`, `IslemId`, `YapiId` ve checker'a ait işaret bulunmadığını;
   temel AST biçimlerinin mümkün olduğunu denetler.
2. `BaglanmisProgram::invariantleri_dogrula`, bütün program/işlem/test
   gövdelerini dolaşır. Her AST ifadesinin tam bir HIR tür/bağ/kaynak kaydı,
   benzersiz `HirDugumId`si ve AST alanlarıyla uyumlu canonical kimliği
   olmasını zorunlu kılar.

Bağlanmış doğrulama ziyaret edilen AST ifade sayısını HIR kayıt sayısıyla
karşılaştırır. Böylece yalnız eksik kayıt değil, hiçbir canlı AST düğümüne ait
olmayan yetim kayıt da reddedilir. Değer konumunda `DegerDondurmez`, hoist
sonrasında tanım/kullanım cümlesi, boş veya tek parçalı sentetik zincir ve
çağrı olmayan `CagriCumlesi` gibi imkânsız durumlar fail-closed'dur.

Parser ve checker başarılı çıkışlarında bu kapı debug/test derlemelerinde
otomatik çalışır. İhlal kullanıcı hatası sayılmaz; faz adı, yapısal yol ve
kaynak satırı taşıyan `InvariantHatasi`, süreç panic'i yerine `C000` tanısına
dönüşür. Release derlemesinde otomatik tam-ağaç taraması yoktur; public açık
doğrulama API'si gerektiğinde çağrılabilir.

K-113/ADR-024 kurtarmalı parser'ın kısmi AST'sini de aynı parser-fazı
değişmezlerine bağladı; recovery hata düğümü uydurmaz, yalnız güvenle
ayrıştırılmış cümleleri doğru ebeveyn blokta korur.

AST dönüşümleri mevcut kutulu alt düğümü klonlamak yerine taşımak zorundadır.
K-112 sırasında özellik→alan dönüşümündeki klon kaldırılmış, beklenmeyen
varyant `T016` ile sonuçlanan tek sahipli yardımcıya alınmıştır.

## Değişmezler

1. Parser AST'si checker'a ait çözülmüş ad veya semantic ID taşıyamaz.
2. Bağlanmış programdaki her AST ifadesi tam bir ve yalnız bir HIR kaydı taşır.
3. HIR tablosunda canlı AST ifadesine karşılık gelmeyen kayıt bulunamaz.
4. `HirDugumId` program içinde benzersizdir; HIR bağ türü AST varyantıyla
   uyumludur.
5. `SymbolId`/`IslemId`/`YapiId`, HIR canonical tablolarındaki adla eşleşir.
6. Sıradan değer konumunda `DegerDondurmez` bulunamaz.
7. Yeni AST/HIR varyantı doğrulayıcı ziyaretçisi ve olumlu/olumsuz kanıtı
   güncellenmeden tamamlanmış sayılmaz.
8. İç değişmez ihlali production panic'i değildir; kodlu tanı/sonuçtur.

## Sonuçlar

- B-017 ve V1-P0-22 kapanır.
- Özellik→alan dönüşümündeki gerçek bir yetim HIR kaydı bulundu ve düzeltildi.
- Üç integration ve iki HIR unit regresyonu parser saflığını, imkânsız AST
  biçimlerini, eksik semantic ID'leri ve geçerli uçtan uca hattı korur.
- Kaynak dil semantiği değişmediğinden yeni normatif dil spec'i gerekmez.
- Doğrulayıcı bellek güvenliğinin biçimsel ispatı değildir; fakat fazlar arası
  iç sözleşmeyi tek, yürütülebilir ve hata yolu belirli bir kapıya bağlar.
