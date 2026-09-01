# AST/HIR invariant rehberi

Bu belge ADR-023'ün uygulama rehberidir. Fazların anlamı için
[derleyici faz modeli](derleyici-faz-modeli.md), HIR kayıtları için
[typed HIR modeli](typed-hir-modeli.md) birlikte okunur.

## İki doğrulama sınırı

| Sınır | Girdi | Zorunlu gerçek |
|---|---|---|
| Parser sonrası | `AyristirilmisAst` | Semantic bağlar boş; AST biçimi mümkün |
| Checker sonrası | `BaglanmisProgram` / `HirProgram` | Her AST ifadesi tek ve eksiksiz typed HIR kaydına sahip |

Parser sonrası kapı `cozulmus`, `SymbolId`, `IslemId`, `YapiId` ve
`sonuca_sarmala` gibi checker sahipli alanları reddeder. Ayrıca tek parçalı
birleştirme/mantıksal zincir, boş `ListeSabiti`, koşulsuz `Ise`/`Gore` ve
çağrı taşımayan `CagriCumlesi` gibi normal parser'ın üretemeyeceği biçimleri
yakalar.

K-113'ün kurtarmalı parser'ı da ürettiği kısmi `AyristirilmisAst` üzerinde bu
kapıdan geçer. Hatalı satırlar çıkarılabilir; fakat recovery boş/imkânsız
sentetik düğüm üretemez ve sağlam kardeşi yanlış ebeveyne taşıyamaz.

Checker sonrası kapı programın ana cümlelerini, deterministik sırada işlem
gövdelerini ve testleri dolaşır. Her ifade için şunları birlikte doğrular:

- AST adresinin bir kez ziyaret edilmesi,
- benzersiz `HirDugumId`, açık tür ve zorunlu kaynak aralığı kaydı,
- varyanta uygun `HirBagi`,
- AST'deki semantic ID ile canonical HIR tablosundaki adın eşleşmesi,
- değer konumunda `HirIfadeTuru::DegerDondurmez` bulunmaması.

Son adımda ziyaret edilen ifade sayısı `HirProgram` kayıt sayısıyla eşitlenir.
Bu karşılaştırma, canlı AST düğümü olmayan yetim HIR kayıtlarını da görünür
kılar.

## Otomatik ve açık kullanım

Debug/test derlemelerinde normal faz geçişleri doğrulayıcıyı otomatik çağırır:

```text
TokenAkisi::ayristir ──> parsed AST invariantleri
BaglanmamisProgram::denetle ──> bound AST/HIR invariantleri
```

İhlal `InvariantHatasi` olarak faz adı, yapısal yol, açıklama ve satır taşır;
normal derleme hattında `C000` tanısına çevrilir. Release hattında her derleme
için ikinci tam-ağaç taraması yapılmaz. Gömülü araç veya test açık denetim
istiyorsa `invariantleri_dogrula()` metodunu doğrudan çağırabilir.

## AST dönüşümü kuralı

HIR üretimi sırasında bir AST düğümü yerinde değiştirilecekse mevcut kutulu
alt düğüm taşınır; klonlanıp eski adres yetim bırakılamaz. K-112 bu kuralla
`Ifade::Ozellik` → `Ifade::AlanErisim` dönüşümündeki gerçek bir sızıntıyı
buldu: klonlanan `nesne` için eski HIR kaydı tabloda kalıyordu. Dönüşüm artık
kutuyu taşır ve beklenmeyen varyantta `T016` döndürür.

## Yeni varyant ekleme kontrol listesi

1. Parser-fazında hangi semantic alanların boş olacağını yaz.
2. Checker-fazında beklenen `HirBagi` ve değer türünü tanımla.
3. Bütün alt ifadeleri yapısal yol üreterek ziyaret et.
4. İmkânsız sentetik biçimleri fail-closed kontrol et.
5. Olumlu uçtan uca ve en az bir bozuk AST/HIR regresyonu ekle.
6. Bu rehberi, typed HIR rehberini ve ilgili ADR/backlog/sürüm kapısını aynı
   değişiklikte güncelle.

## Kanıt

- `tests/invariant_testi.rs`: geçerli parsed+bound hat, parser semantic bağ
  sızıntısı ve imkânsız AST biçimleri.
- `hir/testler.rs`: eksik `SymbolId`, `YapiId` ve `IslemId` bağlarının reddi.
- Koleksiyon regresyonları: özellik→alan dönüşümünden sonra AST/HIR kayıt
  sayısı ve davranış aynı anda korunur.
- `mimari_sinir_testi.rs`, modül bütçeleri ve faz sahipliğinin gerilemesini
  durdurur.
