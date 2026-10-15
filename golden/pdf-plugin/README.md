# K-281 — imzalı owned PDF çağrısı

`ana.dil` yeni dil özelliği kullanmaz: job JSON'u mevcut `diye sor` ile host'a
verir, `yanıt` receipt'inin produced durumunu okuyup PDF_PLUGIN_DONE ile devam
eder. Bu typed stdio invocation, eski K-275 yalnız emitter/ayrı backend örneğinin
yerine gerçek program içi request/reply sağlar. İn-process native FFI değildir.

[Host protokolü](../../docs/pdf-plugin-host.md) ve
[ADR-141](../../adr/141-imzali-pdf-stdio-host.md) paket/trust/capture/owned cleanup
sınırlarını taşır. PDF kodu ve üç asset ile iki özgün notice
[eski fixed fixture'dan](../pdf-host/README.md) gelir; ikinci asset kopyası veya
anahtar kaynak kontrolüne eklenmez. Tasarım/teknik golden'dır; anonim gerçek
ürün korpusu/adoption/P08 diye dogfood kaydı yapılmaz.
