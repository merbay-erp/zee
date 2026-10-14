# K-220 — HTTP POST kabul kanıtı

[Ürün kökeni](../../../dogfood/zee-cekirdek/http-post/README.md), [karar](../../../adr/091-http-post-baslik-ve-gizli-ortam.md).

## Yerel kabul

14 Ekim 2026, macOS / Rust 1.93.1: [tam faz 801/801](macos-faz.md), sıfır kalan ve atlanan test. Format ve all-targets Clippy -D warnings, katalog/tazelik, katman/SCC/public API/uyumluluk, spec drift (187 madde), tanı kalitesi (244 vaka, sıfır ihlal) ve sürekli güvenlik kapısı geçti. Açık depo taraması tam erişilebilir Git geçmişi ve çalışma ağacında temizdir.

11 HTTP ürün/CLI testi ve 4 ağ adaptörü unit testi aynı faz matrisi içindedir. Gerçek localhost TLS taşıması 200/403/302, form gövdesi/Bearer, güvenilmeyen sertifika ve timeout yollarını sınar. Sentetik CA yalnız test Agent’ındadır; production doğrulaması kapanmaz. Gerçek Zee Turnstile helper’ı dokuz başarı/başarısızlık senaryosuyla yürütüldü. Cloudflare’e gerçek anahtar/jeton veya canlı istek gönderilmedi.

## Sınırlar

Canlı sunucu veya ürün rotası yükseltilmedi. Genel heterojen JSON türü, form encoder, PUT/DELETE, WASM native HTTP adaptörü ve otomatik sır taint sistemi eklenmedi. HTTP sır işlemleri iz kaydında yürütülmez ve iz export’u kapanır. Mevcut GET semantiği korunur. Yeni büyüyen cache/kalıcı durum yoktur; ADR-069 etki incelemesi ADR-091’dedir.

Gitsrv exact commit kabulü henüz beklenir; bu kayıt yerel kabulü gösterir. K-220 backlog kapanışı Gitsrv sonrası yapılır.

Compiler kaynak commit’i `488dc7dede31df2bddcec11cfbbc28246e076398`; sentetik TLS fixture’ı aynı commit’te izlenir. Sonraki beyan commit’i exact compiler/core-freeze kayıtlarını ve üretilen kanıt özetini birlikte taşır.

## İlk Gitsrv sonucu

#28 / run 760, c7433123ac1755bb81c476e2698ffee661500a69, 08:10:41–08:28:51 UTC: başarısız. Görünür Actions günlüğünde Linux faz/kalite geçti, tarihçe gizliliği ve Gitleaks Git taraması temiz; directory taraması sentetik server.p12 için tek pkcs12-file eşleşmesi verdi. Sonraki tedarik/WASM/Windows adımları koşmadı. Bu koşu geçiş sayılmaz. Artefakt indirme izni reddedildi; artefakt içeriği veya ayrı Linux test sayısı iddia edilmez. GB-054 yalnız exact kaynak/kopya hash’ine bağlı fixture fingerprint’iyle giderilir; yeni Gitsrv koşusu gerekir.
