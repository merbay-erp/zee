# ADR-136 — JetBrains için proje pinli yerel LSP şablonu (K-278/P19)

- **Durum:** kabul, dar POSIX şablon hazırlığı; IDE etkileşim kabulü açık.

P19 iç araç eksikliği mevcut dillsp sunucusunun JetBrains kullanıcısına
bağlanmasıdır. Compiler freeze kapsamında yeni grammar, lint, cache veya
incremental motor açılmaz. LSP4IJ kullanıcı şablonu mevcut K-270 proje
launcher'ını `lsp` kipinde çağırır. Yalnız JSON descriptor ve rehber eklenir;
plugin indirme, installer veya global PATH kurulumu yürütülmez.

Command içindeki quoted `$PROJECT_DIR$/.zee-araclar/zee` yolunu IDE genişletir.
Mapping yalnız `*.dil` ve `dil` dil kimliğidir. Sözdizimi için mevcut VS Code
TextMate bundle dizini kullanılır; ikinci bir grammar kopyası yoktur.
LSP4IJ schema/macro/import açıklamaları 12703a3301557acd6d57bc903fed07cd826f632f
resmî upstream commit'ine bağlıdır. Belge hashleri plugin provenance'ı veya
kurulmuş IDE sürümü değildir.

Mevcut `compiler/tests/derleyici_yerel_testi.rs` altındaki native launcher
vakası initialize/shutdown/exit ve aynı BuildJSON'u doğrular; bozuk currentpin,
selector ve dependency lock olumsuzları fallback'i reddeder. `lsp_testi.rs`
protokol davranışını korur. Bunlar JetBrains UI kurulum, mapping, boşluklu macro
önizlemesi, gezinme/rename ve kapatmanın gerçek IDE kabulü değildir. Bu kabul
ve Windows launcher profili açık kalır. Şablon yeni LSP capability ilan etmez;
LSP format/lint/incremental/p95 kapsamı P19'da ayrıca açık kalır.

GB-094 yerel executable/plugin/operator güven sınırıdır. Proje dosyası ve
launcher kurulumu bilinçli yerel yürütme yetkisi ister; aynı UID saldırısına
sandbox veya rastgele editör plugin güvenliği sözü verilmez. Spec normu,
compiler/src, public Rust API, katman grafı, tanı kodu ve performans iş yükü
bu değişiklikle değişmez. İlgili [şablon rehberi](../editors/jetbrains/README.md)
ve [yerel araç rehberi](../docs/derleyici-yerel.md) aynı sınırı taşır.
