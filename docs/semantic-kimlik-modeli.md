# Semantic kimlik modeli

Bu belge ADR-014'ün uygulama rehberidir. Amaç kaynak adını silmek değil;
kullanıcının yazımı, semantic identity ve fiziksel depolamayı ayrı tutmaktır.

## Üç kimlik

| Kimlik | Üretim | Bugünkü tüketiciler | Kullanılmaması gereken yer |
|---|---|---|---|
| `YapiId` | yapı adlarının deterministik sıralı kataloğu | `Tur`, `VeriTuru`, yapı oluşturma/alan denetimi | `Program.yapilar[id]` biçiminde vektör indeksi |
| `IslemId` | işlem adlarının deterministik sıralı kataloğu | çağrı bağı, imza tablosu, özyineleme yığını | `HashMap` iterasyon sırası veya çağrı sırası |
| `SymbolId` | kapsam + o kapsamdaki tanım sırası | sembol tablosu ve çözülmüş değişken AST'si | kullanıcıya gösterilen adın yerine tanı metni |

`sirasi()` gözlem/debug değeridir; depolama adresi değildir. Yapılar yalnız
checker bağlamının kimlik→konum dizini üzerinden alınır.

## Faz geçişi

```text
Parsed AST
  ad + kimlik=None
       │ checker katalogları / sembol tablosu
       ▼
Bound AST
  kaynak ad + YapiId / IslemId / SymbolId
       │ K-103 / ADR-016
       ▼
Typed HIR çekirdeği
  HirDugumId + açık kimlik + tür
       │ K-104 / K-108
       ▼
HIR-bağlı runtime + zorunlu source span
```

K-102/B-018 aşamaları ayrı faz tiplerinde görünür yaptı; K-103 her denetlenmiş
ifadenin türünü ve semantic bağını ayrı HIR kaydına taşıdı. K-104 standart
runtime ve `dene` hattını bu bağlara geçirdi. Kaynak adı kaliteli Türkçe tanı
ve v0 uyumluluğu için korunur; bağlı runtime kaynak adıyla semantic karar
vermez. K-108/B-020 her HIR ifadesine kesin token konumu veya kaynak satırı
zarfı ekledi; K-126/B-050 bütün parser AST ifadelerini kesinleştirip HIR ile
birebir eşledi. Semantic düğüm artık kaynak kökeninden ayrı kurulamaz.
K-120/B-041 definition ve rename'i aynı zincirin production tüketicisi yaptı:
HIR ilk tanım/yazım/okuma aralıklarını `SymbolId`, işlem ve yapı kullanımını
`IslemId`/`YapiId` ile açar. LSP kaynak adını ancak ID seçildikten sonra aralık
ve morfolojik yüzey için okur. Dış birim tanımının satırı açık belgeyle
çakışsa bile yerel başlık kanıtı yoksa tek-dosya düzenlemesi reddedilir; ayrıntı
[semantic gezinme rehberindedir](lsp-semantic-gezinme.md).

## Yeni kod için kurallar

- Yeni bir yapı türü `usize` ya da vektör konumuyla taşınmaz.
- İşlem imzası/call graph kaydı `String` yerine `IslemId` ile anahtarlanır;
  ad yalnız gösterim ve ilk katalog çözümü içindir.
- Sembol yeniden atamasında ID korunur; yeni sözcüksel tanım yeni ID alır.
- Semantic araç yeni hedefi metin benzerliğiyle seçmez; başarılı bağlı
  programın ID dizinini tüketir ve derleme hatasında fail-closed durur.
- Yeni AST bağ alanı parser'da `None`, checker başarısında `Some(id)` olur.
- Malformed/elle kurulmuş AST kimlik değişmezleri B-017/K-112'nin
  [invariant doğrulayıcısında](ast-hir-invariantleri.md) yürütülebilirdir:
  parser alanları boş, bağlı AST kimliği canonical HIR adıyla eşleşmelidir.
  Production runtime sessiz kimlik uydurmaz.

## Kanıt

`semantic_kimlik_testi.rs` yapı depolama sırasını ve işlem çağrı sırasını ters
çevirerek kimliklerin değişmediğini, çözülmüş değişkenin `SymbolId` taşıdığını
kanıtlar. `hir_modeli_testi.rs` bu bağların açık türle HIR'a ve tanım/yazım
dizinine geçtiğini; `lsp_testi.rs` ayrı kapsamların doğru semantic hedefte
kaldığını, A002'de tahmin yapılmadığını, işlem/yapı kimliklerini ve dış
tanım için eksik rename üretilmediğini;
`mimari_sinir_testi.rs`, `Yapi(usize)`, `yapilar[id]` ve HIR'sız bağlı program
gerilemesini reddeder. `invariant_testi.rs` ile HIR unit testleri eksik veya
faz dışı kimliği reddeder. Tam test, Clippy ve WASM kapıları yine zorunludur.
