# ADR-101 — rustls güvenlik yaması ve erişilebilirlik sınırı (K-238)

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-238, GB-062 (yüksek, kapalı)
- **Karar zinciri:** ADR-038/068, ADR-045/049; P33/P37

## Bağımlılık kararı

Güncel RustSec kapısı `rustls 0.23.45` için RUSTSEC-2026-0346, 0347, 0348
ve 0349 bildirir. Dördünün ortak yamalı alt sınırı `0.23.46`dır. Native
`ureq =3.4.0` bağımlılığı korunur; compiler ve fuzz lock'ları aynı
`rustls 0.23.46` registry checksum'ına yükseltilir. Advisory ignore kümesi
boş kalır. Manifest, grammar, dil API’si ve desteklenen facade aynı kalır.
Aynı toplu işte K-240, mevcut GET gövdesini HTTP handler’ına ve replay
trait uygulamasını aynı yorumlayıcı sahibinin alt dosyasına taşır; kaynak
semantiği, deadline, tanı, tel baytları ve durum korunur. Bu Rust kaynak
commit’i aynı security diliminin exact compiler/core-freeze beyanıyla izlenir.


## Gerçek Zee yüzeyi

| Advisory | Upstream tetik | İncelenen standart Zee yolu |
|---|---|---|
| [0346](https://github.com/RustSec/advisory-db/blob/main/crates/rustls/RUSTSEC-2026-0346.md) | Özel ClientSessionStore, session değerini Debug ile loglar; built-in cache ve ordinary connection Debug etkilenmez. | Zee özel session store veya bu session değerlerinin Debug çıktısını kurmuyor. Bu saldırı bu yolda yürütülmüş değildir. |
| [0347](https://github.com/RustSec/advisory-db/blob/main/crates/rustls/RUSTSEC-2026-0347.md) | Explicit ECH Enable, handshake öncesi buffered application data ve ECH ret birleşir. | Zee ECH config kurmuyor; normal ureq connector/builder kullanılır. ECH açığına ürün exploit kanıtı çıkarılmaz. |
| [0348](https://github.com/RustSec/advisory-db/blob/main/crates/rustls/RUSTSEC-2026-0348.md) | Non-default rustls Brotli certificate decompression büyük history window ayırır. | Her iki gerçek feature tree ring/logging/std/tls12 gösterir; rustls brotli etkin değildir. HTTP content decompression ile TLS certificate decompression aynı özellik değildir. |
| [0349](https://github.com/RustSec/advisory-db/blob/main/crates/rustls/RUSTSEC-2026-0349.md) | Unauthenticated KX hint sonraki hibrit tercihli oturumu klasik key share'e indirir. Authentication bypass/key compromise değildir. | Standart provider ring'dir; aws-lc/prefer-post-quantum/hybrid yüzeyi etkin değildir. Hibrit downgrade etkisi bu config için kanıtlanmış değildir. |

Bu değerlendirme kaynak/feature sınırıdır, dört advisory'nin Zee'de ayrı ayrı
exploitable olduğu iddiası veya tüm olası consumer konfigurasyonlarının
güvenlik denetimi değildir. `compiler/src/ag_istemcisi/istek.rs` ve
`ureq 3.4.0` ClientConfig builder'ı incelendi; patched sürüme geçmek
ulaşılabilirlik tahminine dayalı ignore vermekten bağımsız tedarik kararıdır.
Resmî düzeltmeler [3299](https://github.com/rustls/rustls/pull/3299),
[3300](https://github.com/rustls/rustls/pull/3300),
[3297](https://github.com/rustls/rustls/pull/3297) ve
[3355](https://github.com/rustls/rustls/pull/3355) içindedir.

## Kanıt ve dağıtım etkisi

Her iki `cargo deny --locked check -D warnings` ve gerçek offline vendor
kapısı yeni lock'ları kabul etmelidir. Exact yerel sonuçlar kaynak checkpoint
kanıtına bağlanır; eski yeşil tedarik/uzak CI kaydı yeni lock kabulü sayılmaz.
`tedarik_kapisi_testi::registry_lock_girdilerinin_sha256_ozeti_zorunludur`,
`cargo_deny_politikasi_fail_closed_ve_istisnalari_gerekcelidir` ve
`surum_artefakti_iki_temiz_klon_sbom_provenance_ve_imza_kablosunu_tasir`
lock/ignore/canonical artifact sınırlarını korur. HTTP primitive kabulü
`ag_istemcisi::istek::tests::tls_post_baslik_govde_json_ve_non2xx_durum_kayipsizdir`,
`tls_dogrulamasi_kapatilmaz_ve_yanit_timeout_sirri_gostermez` ve
`http_post_testi::http_transport_hatasi_sonuctur_ve_adaptor_sirri_sizmaz`
işlevlerine bağlanır. Bunlar dört upstream exploit reproducer'ı değildir.

Canonical `scripts/surum-artefakti.sh` ve `surum_artefakti` SBOM üreticisi
Cargo.lock'taki exact paket/sürüm/checksum'ları işler; betik, SBOM/provenance
şeması, imza ve dosya kümesi değişmez. Sonraki yeni çıktı rustls 0.23.46
ve yeni lock digest'i taşır; eski 99f31d7 SBOM/two-clone kanıtı tarihsel
olarak aynen korunur, bu yamaya veya final release'e yeniden etiketlenmez.
Yeni exact iki-klon, hedef platform, güvenilir yayın imzası ve kurulum kabulü
ayrı kapılardır.

ADR-045/049 sabit workload/sample/warmup sözleşmesi değişmez. TLS bağımlılık
kodunun performansı etkilenebilir; p95/RSS eşliği ya da iyileşmesi ölçülmedi.
`docs/performans-gecmisi-v2.tsv`ye yeni taban yazılmaz. Gelecek kalıcı ölçüm
yalnız temiz exact yama checkpoint'i ve tam OS/CPU/RAM/Rust/release/sample
provenance ile yapılır. Cache, kalıcı state veya benchmark workload eklenmedi;
sırf bu lock yaması yeni 1800 s soak kabulü olarak gösterilmez.
