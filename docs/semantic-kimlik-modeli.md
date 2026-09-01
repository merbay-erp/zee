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
       │ B-019
       ▼
Typed HIR (gelecek)
  açık kimlik + tür + source span
```

K-102/B-018 bu aşamaları ayrı faz tiplerinde görünür yaptı. Kaynak adı şimdilik
runtime ve kaliteli Türkçe tanılar için korunur. Bu çift taşıma geçiş
köprüsüdür; B-019 HIR işi tamamlanınca runtime çözülmemiş kaynak adıyla
semantic karar vermemelidir.

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
kanıtlar. `mimari_sinir_testi.rs`, `Yapi(usize)` ve `yapilar[id]` gerilemesini
reddeder. Tam test, Clippy ve WASM kapıları her değişiklikte yine zorunludur.
