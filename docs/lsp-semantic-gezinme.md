# LSP semantic gezinme ve yeniden adlandırma

Bu belge K-120/B-041 ve K-126/B-050'nin bakım sözleşmesidir. `dillsp` içindeki
`textDocument/definition` ve `textDocument/rename`, geçerli bir Zee belgesinde
metin benzerliğiyle sembol tahmini yapmaz; checker'ın ürettiği typed HIR
bağlarını tüketir. JSON-RPC zarfı, sayı ve hata davranışının ayrı sözleşmesi
[LSP JSON-RPC bakım profilindedir](lsp-json-rpc-profili.md).

## Semantic hat

```text
açık belge
  → lexer/parser
  → checker
  → SymbolId / IslemId / YapiId + HIR kaynak aralıkları
  → imleçteki tek semantic varlık
  → tanım veya WorkspaceEdit
```

Yerel semboller için HIR şunları birlikte taşır:

- canonical kaynak adı;
- ilk tanımın kaynak aralığı;
- aynı `SymbolId`ye ait sonraki yeniden atama yazımları;
- okuma ifadelerinin zorunlu HIR kaynak aralıkları.

İşlem ve yapı tanımları ile kullanımları sırasıyla `IslemId` ve `YapiId`
üzerinden eşleşir. Çok kelimeli işlem adı tek varlıktır; rename yalnız seçilen
kelimeyi değil tanım ve çağrılardaki tam işlem adını değiştirir.
K-126 ile işlem/yapı kullanımının lexical araması bütün satıra yayılmaz;
semantic AST ifadesinin kesin aralığında kalır ve çağrı/`yeni` kuyruğundaki
son canonical eşleşmeyi seçer. Böylece argüman, işlem adıyla aynı yazılsa bile
yanlış token düzenlenmez.

## Morfoloji sırası

Morfoloji kimlik seçmez. Önce imleçteki kaynak aralığı typed HIR üzerinden tek
`SymbolId`ye bağlanır. Ancak bundan sonra o kimliğe ait her görünür yazımın ek
zinciri `zee-tr-1` ile çözülüp yeni köke giydirilir:

```text
sayaç → puan
sayacı → puanı
sayaçla → puanla
```

Bu sıra, iki ayrı blokta aynı `dal` adı kullanıldığında yalnız seçilen
kapsamın değişmesini sağlar. Metin sabitleri, yorumlar ve başka semantic
kimliğe ait aynı yazımlar düzenleme kümesine giremez.

## Fail-closed davranış

Belge lexer/parser/checker hattından başarıyla geçmiyorsa definition/rename
metin tahmini yapmaz ve `null` döner. Kullanıcı nedenini aynı didOpen/didChange
akışında yayımlanan Türkçe tanıdan görür. Özellikle A002 morfoloji
belirsizliğinde bir ad rastgele seçilmez.

K-120 değişiklik kümesi tek açık belgeyle sınırlıdır. Birim/paket grafiği
semantic derleme için yüklenir; fakat başka dosyadaki tanımı eksik bir
WorkspaceEdit ile değiştirmek yerine işlem güvenle reddedilir. Çok dosyalı
yeniden adlandırma, kaynak aralığının dosya kimliği taşıdığı ayrı bir LSP
workspace kapısıdır. Dış tanımın satır numarası açık belgedeki kullanımla
çakışsa bile `işlem`/`eylem`/`yapı` yerel başlığı ve tam canonical ad
doğrulanmadan tanım aralığı kabul edilmez. Parametre/döngü tanımları da kimlik
seçildikten sonra yalnız kendi kesin HIR aralığına indirilir.

## Kanıt

- `compiler/tests/hir_modeli_testi.rs`: ilk tanım + yeniden atama + okuma
  aralıklarının aynı `SymbolId` altında tutulması;
- `compiler/tests/lsp_testi.rs`: ayrı kapsamlı aynı adlarda doğru tanım,
  yalnız seçilen kapsamın rename'i, yeniden atamalar, tek/iki katmanlı Türkçe
  ekler, A002'de fail-closed davranış, tam `IslemId` adı, `YapiId` gezintisi
  çakışan satır numarasında dış-birim rename reddi ve aynı yazımlı
  argüman/işlem kuyruğunda yalnız semantic çağrı adının seçilmesi;
- tam test, Clippy, native release, fuzz derleme ve WASM kapıları.
