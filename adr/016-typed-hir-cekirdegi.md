# ADR-016 — Typed HIR çekirdeği ve aşamalı runtime geçişi

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-103, K-104, K-121, B-007, B-019, V1-P0-14
- **Revizyon:** 2 Eylül 2026 — K-120 semantic LSP ve K-121 nihai çıkarım
  tüketimi eklendi

## Bağlam

ADR-014 semantic kimlikleri kurdu, ADR-015 derleyici veri fazlarını ayrı
tiplerde görünür yaptı. Buna rağmen `BaglanmisProgram` yalnız checker
tarafından yerinde değiştirilmiş AST taşıyordu. Bir sonraki geçiş veya runtime,
bir ifadenin kanıtlanmış türünü yeniden çıkarımlamak ve kaynak adı ile semantic
kimlik arasından seçim yapmak zorunda kalabilirdi.

## Karar

Başarılı checker geçişi artık AST'den ayrı bir `HirProgram` üretir. Her
denetlenmiş ifade şu bilgileri taşır:

- program içi kararlı `HirDugumId`,
- değer üretiyorsa `Deger(Tur)`, çağrı cümlesiyse `DegerDondurmez` taşıyan
  açık `HirIfadeTuru`,
- varsa `SymbolId`, `IslemId` veya `YapiId` bağı.

`BaglanmisProgram` doğrudan `Program` değil, zorunlu `HirProgram` sahibidir.
Kaynak AST; tanı, formatter/LSP ve v0 Rust API uyumluluğu için HIR içinde
salt-okunur korunur. HIR ayrıca kimlikten canonical sembol adına, işlem
tanımına ve yapı tanımına güvenli erişim sunar.

Checker sırasında kullanılan düğüm adresi yalnız AST düğümü ile HIR kaydını
aynı süreç içinde eşleyen private locator'dır; semantic identity veya public
API değildir. AST, checker sonrasında kutulu HIR sahipliğine taşınır ve bağlı
program klonlanamaz; böylece locator HIR ömrü boyunca yer değiştirmez.

## Aşamalı geçiş

K-103 HIR üretimini ve faz sahipliğini kurdu. K-104 bağlı runtime ile `dene`
hattını `CalistirmaProgrami::Hir` koluna geçirdi: değişken, işlem ve yapı
seçimleri `HirBagi` üzerinden yapılır; kaynak adı HIR kolunda yedek çözüm
değildir. Raw `Program` runtime'ı v0 embedding uyumluluğu olarak ad-temelli
kalır. Böylece B-019 ve V1-P0-14 kapanmıştır.

Eşzamanlı görev ifadeleri klonlanmaz; özgün HIR düğümünü ödünç alır. Bu,
private AST locator'ının scheduler içinde de aynı semantic kayda gitmesini
sağlar.

K-108/ADR-020 her semantic düğümde zorunlu source span'i sonradan kurdu.
Kesin token konumu bulunan ifadeler tam aralık, diğer mevcut AST ifadeleri
sahte sütun yerine kaynak satırı zarfı taşır. Span eksikliği HIR'ın tür ve bağ
gerçeğini AST'ye geri itmek için gerekçe değildir.

K-126/ADR-030 bu geçişi tamamladı: başarılı parser çıktısındaki her yaprak ve
bileşik AST ifadesi ayrı kesin kaynak zarfı taşır; HIR aralığı bu değeri
birebir devralır. Başarılı parser→checker hattındaki ifade kayıtları artık
satır zarfına düşmez; AST dışı uyumluluk/tanım kayıtları için HIR'ın kaba
varyantı korunur.

K-112/ADR-023 her canlı AST ifadesi ile HIR kaydının birebirliğini ve semantic
bağın canonical tablolarla uyumunu debug/test faz çıkışında yürütülebilir
değişmez yaptı. Özellik→alan dönüşümünde klonlanan alt düğümün bıraktığı yetim
HIR kaydı bu kapıyla bulunup taşıma temelli dönüşümle düzeltildi.

K-120'de HIR ilk kez runtime dışındaki production tüketiciye bağlandı.
`HirProgram`; sembol tanımı/yazımı/okumasını `SymbolId`, işlem çağrılarını
`IslemId`, yapı kurulumlarını `YapiId` ile kaynak sırasına açar. LSP tanıma
git ve rename bu dizinleri kullanır; kaynak metni yalnız kimliği seçtikten
sonra LSP aralığı ve morfolojik yüzey üretmek için okur. HIR aralığı henüz
dosya kimliği taşımadığından dış tanım, açık belgede tam yerel başlık
olarak doğrulanamazsa definition/rename fail-closed durur.

K-121'de yerel işlem imzası keşfi, HIR üretmeyen kopya-AST geçişine ayrıldı.
Bütün erişilebilir çağrı kısıtları nihai imzada birleştikten sonra asıl checker
gövdeyi yeniden doğrular; `HirOlusturmaBilgisi` yalnız bu geçişten çıkar.
Geçici ilk çağrı sonucu veya kaynak sırası kalıcı HIR türüne sızamaz.

## Değişmezler

1. Başarılı checker bilgisi olmadan `HirProgram` kurulamaz.
2. Her kaydedilmiş ifade açık `HirDugumId` ve `Tur` taşır.
3. Değişken/çağrı/yapı ifadelerinin HIR bağı çıplak kaynak adı değildir.
4. `BaglanmisProgram`, HIR sahipliğini atlayıp yalnız AST taşıyamaz.
5. Eski `Program` adaptörü ancak HIR üretildikten sonra faz bilgisini siler.
6. Standart kaynak çalıştırma ve test hattı `CalistirmaProgrami::Hir` kullanır.
7. Her HIR ifade kaydı zorunlu `HirKaynakAraligi` taşır.
8. HIR tablosunda canlı AST ifadesine karşılık gelmeyen kayıt bulunamaz.
9. LSP semantic hedefi yalnız başarılı `BaglanmisProgram` HIR'ından seçilir;
   başarısız derlemede parser-metni fallback'i yoktur.
10. Dosya kökeni kanıtlanamayan içe alınmış tanım, açık belgenin satırıyla
    sayısal olarak çakışsa bile yerel tanım kabul edilmez.
11. Yerel işlem çıkarımının keşif geçişi HIR üretmez; HIR türleri bütün çağrı
    kısıtları birleştirildikten sonraki nihai checker sonucudur.
12. Parser kökenli her HIR ifade aralığı karşılık gelen AST ifadesinin kesin
    aralığıyla birebir aynıdır.

## Sonuçlar

- Checker tür sonucu artık geçici dönüş değeri olmaktan çıkıp sonraki fazın
  kalıcı girdisidir.
- Semantic bağlar AST alanlarında uyumluluk için dursa da tek gelecek yönü HIR'dır.
- HIR ve LSP davranış testleri tür/bağ kayıtlarını, sembol tanım/yazım
  aralıklarını, zorunlu faz sahipliğini ve production tüketicilerin kaynak
  adına geri düşmemesini korur.
- K-121'in sıra-bağımsız yerel çıkarım semantiği spec/04 ve spec/10'da
  normatifleşmiştir; HIR bunun yalnız nihai sonucunu taşır.
