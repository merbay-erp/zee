# ADR-138 — JetBrains tamamlama ve düzenli kapanış kanıtı (K-280/P19)

- **Durum:** kabul; macOS ARM64 yerel POSIX IDE profili.

K-278'in açık kalan tamamlama ve kapanış denemesi, IntelliJ IDEA 2025.3.1
IU-253.29346.138 ve resmî LSP4IJ 0.21.0 ile ayrı owned profilde tamamlandı.
Çalışan launcher kaynağı `8b49113c413f62b35d4b251ed31068524d2fe26c`;
belge tabanı `9214a3e676bc1940ae9c3b82af4fafadb5f9f0fd` idi. Bu kanıt daha
sonraki compiler değişikliklerine aktarılmaz.

`"$PROJECT_DIR$/.zee-araclar/zee" lsp` komutu ve boşluklu proje kökü
korundu. Mevcut TextMate bundle ve pattern mapping kullanıldı. LSP4IJ'nin
Debug sekmesinde trace gerçekten `verbose` seçilerek uygulandı; hazırlık
XML'inin tek başına yeterli olmadığı gözlendi. Built-in trace gerçek
initialize, didOpen, completion ve didChange trafiğini gösterdi.

`ol` öneki Basic eylemi, Down ve Return ile `olduğu` oldu. Geçerli cümlede
`sayi 7 ols`, completion isteği 7 sonrasında `sayi 7 olsun` oldu ve
publishDiagnostics boş döndü. Bu, mevcut anahtar kelime tamamlamasıdır;
değişken simgesi tamamlama iddiası değildir. Ekran görüntüsü popup'ı
göstermediğinden popup görünürlüğü ayrıca kanıtlanmış sayılmaz; uygulanan
editör metni ve gerçek protokol yanıtı birlikte saklanır.

Trace ayarı uygulanırken eski sunucunun shutdown isteği 5, yanıtı ve exit
bildirimi görüldü. Son normal IDE çıkışındaki protokol konsolu ayrıca dışa
aktarılmadı; iki olay birleştirilmez. IDE menüsünden Quit, Confirm Exit ve
Exit seçildi; original Popen/WNOWAIT sahibi çıkış 0 ve kapanmış/toplanmış
süreç grubunu kaydetti. Zorunlu sinyal sonlandırması gerekmedi.

GB-096 aynı UID yerel plugin/executable güven sınırını korur; rastgele
plugin güvenliği, Windows, yayımlanmış eklenti, incremental analiz, lint,
format sağlayıcısı veya 20 bin satır/200 ms garantisi eklenmez. Compiler,
grammar, spec normu, facade, mimari graph ve benchmark değişmedi. Katalog,
RFC/spec, tanı ve faz sayıları bu belge/kanıt işi nedeniyle değişmez.

[Kabul kaydı](../docs/incelemeler/urunlesme-20261014/k280-jetbrains/ACCEPTANCE.json)
ve [dosya manifesti](../docs/incelemeler/urunlesme-20261014/k280-jetbrains/MANIFEST.json)
ham UI/protokol kanıtlarına bağlanır. Önceki başarısız denemeler korunur.
