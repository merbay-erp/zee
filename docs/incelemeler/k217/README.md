# K-217 — Tek merkez dal kararları

[Ürün talebi](urun-talebi.md) §2 kapsamı; ana kaynak merbay-erp/zee master,
Enterprise yalnız aynadır. Silme öncesi altı ref'in tam geçmişi özel Git
bundle yedeğinde doğrulandı; yedek ve ham özel kayıt açık depoya alınmaz.

| Dal / kaynak uç | Karar ve ana hatta uygulanan içerik |
|---|---|
| site/zeedili-com — 6833f72ed25efdb07b5b8f4a24470c4b71cc06b2 | K-203/ADR-081 site kaynağı ve belgeler temiz içerik olarak alınır. Sekiz commit eski sunucu adresini taşıdığı için master atası yapılmaz; özel yedekte korunur. GB-038 mevcut K-201'e aittir; site GB-049 olur. |
| dgx-issue-formlari — 4b82fecd79a4075bc96e92cda4dabc5725913ec8 | Üç form + config yeniden uygulanır. Mevcut bug/enhancement etiketleri, core freeze ve anonim reproducer kuralları kullanılır. Ana depoda doğrulanmamış otomatik DGX yorum/zaman sözü taşınmaz. |
| ci/tier1-gitsrv — a0572f1a3c01c837d8aceb64315789345cf6e969 | MSVC çapraz derleme/lint özel Forgejo akışına uyarlanır (ADR-080/088). GitHub public workflow self-hosted seçemez; özel Git2 metadata kapısı vardır. Windows runtime kanıtı açık kalır. |
| wip/spec27-snapshot — d6b516f30ca945ebd99bdf58adf43101c36aba60 | Gerekçeli kapatma: yalnız belge değildir; 1.228 satır snapshot modülü ve yeni CLI/export getirir. Gerçek ürün reproducer/K-işi/ADR, regression ve normatif madde test zinciri yoktur; core freeze eşiğini geçmez. spec/27 adı mevcut uyumluluk bölümüne de çakışır. Self-hosting gözlem sözleşmesi V1 sonrası kanıtlı iş olarak yeniden tasarlanabilir. |
| dependabot upload-artifact — aec425a3e09aeb1ab0f81f23d9fde2aeb4c27e64 | v7.0.2 resmî cf430e030ddbb5b0abf93d22962f4752f3646cd9 commit'i incelenerek GitHub pinleri/TSV'ye uygulanır; otomatik merge yoktur. Forgejo v3 artifact adaptörü değişmez. |
| dependabot rust-toolchain — 1b3ec570b2afe77667e739295602ab4cbb476f51 | e2a55d2ffb04f378e9626c28d38b36d230d1e12f resmî input güvenliği değişikliği incelenerek iki pin envanterine uygulanır; Rust sürümü 1.93.1 kalır. |

Resmî kaynaklar: [upload-artifact v7.0.2](https://github.com/actions/upload-artifact/releases/tag/v7.0.2),
[rust-toolchain değişikliği](https://github.com/dtolnay/rust-toolchain/commit/e2a55d2ffb04f378e9626c28d38b36d230d1e12f).

Site deploy dosyaları örnek /opt/zeedili ve /opt/zee/bin/dil yollarını
kullanır; SSH hostu, origin IP ve Cloudflare kimlik dosyası özel ortamdan
girilir. Bu iş canlı kurulum/yayın değildir. Yerel kaynak/test/statik,
Bash/YAML, katalog/tazelik, tedarik, faz ve gizlilik kapıları geçmeden
commit yoktur. Master push ve karşılaştırmalı ref teyidi olmadan uzak
PR/dal silinmiş sayılmaz. Silme sonuçları sonraki exact kayıtla eklenir.

## Sonraki öncelik

Talep §3/1 K-218: oran durumu dosya/fsync sıcak yolundan bellek içi veya
PostgreSQL backend'ine ayrılır. Oturum/revoke paylaşımı ve saldırı kotaları
sessizce zayıflatılmaz. Ürün tarafından bildirilen 11.000/55/56 istek/sn
ölçümleri bu kaynakta yeniden ölçülmüş sayılmaz; gerçek temiz kaynak ve
sample/çevre kanıtı olmadan performans tarihçesine yazılmaz.

CI çalışma image'i resmî docker.io/library/golang:1.26.1-bookworm,
OCI index digest'i sha256:ab3d6955bbc813a0f3fdf220c1d817dd89c0b3f283777db8ece4a32fe7858edd,
Linux amd64 manifest'i sha256:09fb8a652cf7a990b714c46a9f0f5fd2d5bc2222d995166b91907c1c05b7d0e8.
Kayıt sunucusunun Docker-Content-Digest değeri doğrulandı; GOVERSION
kontrolü ve gerçek Go eşdeğerliği zorunludur. Bu source'da yeni kalıcı
performans tabanı üretilmez. Gitleaks indirmesi süre/tekrar bütçelidir.
