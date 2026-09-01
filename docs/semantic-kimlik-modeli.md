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
       │ B-019 kalan dilim / B-020
       ▼
HIR-bağlı runtime + zorunlu source span
```

K-102/B-018 aşamaları ayrı faz tiplerinde görünür yaptı; K-103 her denetlenmiş
ifadenin türünü ve semantic bağını ayrı HIR kaydına taşıdı. Kaynak adı kaliteli
Türkçe tanı ve v0 uyumluluğu için korunur. B-019 tamamlandığında standart
runtime kaynak adıyla semantic karar vermemelidir.

## Yeni kod için kurallar

- Yeni bir yapı türü `usize` ya da vektör konumuyla taşınmaz.
- İşlem imzası/call graph kaydı `String` yerine `IslemId` ile anahtarlanır;
  ad yalnız gösterim ve ilk katalog çözümü içindir.
- Sembol yeniden atamasında ID korunur; yeni sözcüksel tanım yeni ID alır.
- Yeni AST bağ alanı parser'da `None`, checker başarısında `Some(id)` olur.
- Malformed/elle kurulmuş AST kimlik değişmezleri B-017 doğrulayıcısının
  kapsamıdır; production runtime sessiz kimlik uydurmaz.

## Kanıt

`semantic_kimlik_testi.rs` yapı depolama sırasını ve işlem çağrı sırasını ters
çevirerek kimliklerin değişmediğini, çözülmüş değişkenin `SymbolId` taşıdığını
kanıtlar. `hir_modeli_testi.rs` bu bağların açık türle HIR'a geçtiğini;
`mimari_sinir_testi.rs`, `Yapi(usize)`, `yapilar[id]` ve HIR'sız bağlı program
gerilemesini reddeder. Tam test, Clippy ve WASM kapıları yine zorunludur.
