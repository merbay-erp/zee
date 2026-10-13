# K-205 — 13 Ekim 2026 dış inceleme eşlemesi

Gönderilen inceleme 12c59218 kaynak tabanını kullanır; yazar testleri yerelde
çalıştırmadığını belirtmiştir. Bu belge 29be85bdc51f3d1f395fdd06d06e71181f5c73e0
üzerindeki mevcut kod/karar/test yollarına ilk eşlemedir. Kaynak ekinin
SHA-256 değeri: `6896f5fefe7b4e2dec25714c0340f0ebb771ecd92079decd3daea483ffcb50c9`.
İncelemedeki P0/P1/P2 önerileri resmi backlog sırasının yerine geçmez.

Durumlar bir kod incelemesi veya tüm saldırı yüzeylerinin kapandığı iddiası
değildir. Seçili kaynaklar okundu; son yerel 726/726 faz kanıtı, native
üç-platform tatbikatı ve ayrı sır taraması kaydı vardır. Sonraki bulgular
gerçek reproducer/etki/kabul testiyle eklenecek; doğrulanmamış risk güvenlik
açığı olarak kaydedilmez. K-203/ADR-081 site PR #6'ya ayrılmıştır.

| Dış kimlik | Alan | Durum | Resmi iş | Mevcut kanıt/kod | Sonraki doğrulama |
|---|---|---|---|---|---|
| ZEE-001 | Native backend | Eksik kapasite | K-193 | [README.md](../README.md) · [docs/derleme-ve-bagimliliklar.md](../docs/derleme-ve-bagimliliklar.md) | V1/ürün önkoşulları sonrası backend karar ve eşdeğerlik korpusu. |
| ZEE-002 | Zee → WASM | Eksik kapasite | K-193 | [README.md](../README.md) · [docs/wasm-c-abi.md](../docs/wasm-c-abi.md) | Playground compiler WASM ile Zee program codegen ayrımı korunur. |
| ZEE-003 | Self-hosting | Eksik kapasite | K-193 | [README.md](../README.md) | Stage 1 bileşen ve bootstrap zinciri ürün kapıları sonrası tasarlanır. |
| ZEE-004 | V1 çağrı grameri | İnsan kanıtı açık | B-001/K-096 | [docs/k016-cagri-karar-paketi.md](../docs/k016-cagri-karar-paketi.md) | Gerçek anonim kullanıcı formları beklenir; veri üretilmez. |
| ZEE-005 | Incremental LSP/cache | Eksik kapasite | K-207 | [compiler/src/lsp.rs](../compiler/src/lsp.rs) · [docs/olcumler.md](../docs/olcumler.md) | Önce mevcut ön uç maliyeti ölçülür; cache eklenirse 30 dk soak gerekir. |
| ZEE-006 | Büyük dosya LSP | Tarihsel ölçüm doğrulandı | K-154/K-207 | [docs/performans-gecmisi-v2.tsv](../docs/performans-gecmisi-v2.tsv) | 20k p95 20.159,636 ms kaydı eski exact kaynaktadır; güncel temiz ölçüm alınır. |
| ZEE-007 | Yorumlayıcı maliyeti | İş yüküne bağlı ölçüm | K-165 | [docs/dogfood-karsilastirma.md](../docs/dogfood-karsilastirma.md) | Bütün programlar için genellenmez; aynı ürün/çıktı/ortamla karşılaştırılır. |
| ZEE-008 | Ara değişken ergonomisi | Kayıtlı sürtünme | K-164/F003/F014 | [docs/dogfood-karsilastirma.md](../docs/dogfood-karsilastirma.md) | Tekrarlayan gerçek ürün ihtiyacı olmadan grammar büyütülmez. |
| ZEE-009 | İndeks/sekme kaçışı | Kayıtlı sürtünme | K-164/F010/F011 | [docs/dogfood-karsilastirma.md](../docs/dogfood-karsilastirma.md) | Mevcut workaround ve ürün tekrar kanıtı ayrı tutulur. |
| ZEE-010 | Minimum OS | Açık doğrulama | K-192 | [docs/derleme-ve-bagimliliklar.md](../docs/derleme-ve-bagimliliklar.md) | Mach-O minos gerçek minimum OS testi yerine geçmez. |
| ZEE-011 | Temiz fiziksel OS | Kısmi kanıt | K-192/K-197/K-198 | [docs/temiz-windows-platform-kaniti-v1.tsv](../docs/temiz-windows-platform-kaniti-v1.tsv) | Container ile fiziksel/desktop kanıtı ayrılır; yerel Parallels service bağlantısı yok. |
| ZEE-012 | Genel native FFI | Taslak kapasite | RFC-0012/K-193 | [rfcs/0012-ffi-ve-tehlikeli-sinir.md](../rfcs/0012-ffi-ve-tehlikeli-sinir.md) | Mevcut WASM ABI sınırından ayrı tasarım ve unsafe kabul testleri gerekir. |
| ZEE-013 | Web worker sıralılığı | Kayıtlı sınır | K-137 | [docs/web-production-profili.md](../docs/web-production-profili.md) | N worker modeli vardır; worker iç concurrency ayrı ürün ihtiyacıdır. |
| ZEE-014 | Shared CI hard eşik | Bilinçli politika | ADR-045/049 | [docs/olcumler.md](../docs/olcumler.md) | Hard eşik adanmış sabit koşucuya aittir; shared CI gözlemseldir. |
| ZEE-015 | Morfoloji ad ergonomisi | Kayıtlı profil/sürtünme | K-164/F006–F013 | [docs/morfoloji-profil-uyumlulugu.md](../docs/morfoloji-profil-uyumlulugu.md) | Immutable zee-tr-1 anlamı korunur; belirsizlik A002 ile görünürdür. |
| ZEE-016 | PG sonuç belleği | Ön-limit Vec ve RSS etkisi doğrulandı | K-206 | [compiler/src/postgresql.rs](../compiler/src/postgresql.rs) | İzole PG 16.15 deneyi 64 MiB girişte 129–137 MiB süreç RSS gözledi; bounded toplama ve wire/tek-satır sınırı ayrı ele alınır. |
| ZEE-017 | PG typed-result | Bilinçli Metin sözleşmesi | RFC-0026 | [spec/25-postgresql-veri-erisimi.md](../spec/25-postgresql-veri-erisimi.md) | Typed API gerçek ürün ihtiyacı ve sürümlü sözleşme ister. |
| ZEE-018 | PG hata matrisi | Kısmi mevcut kanıt; ek denetim | K-206 | [docs/postgresql-commit-belirsizligi.md](../docs/postgresql-commit-belirsizligi.md) · [docs/postgresql-tls-havuz-profili.md](../docs/postgresql-tls-havuz-profili.md) | COMMIT/recovery/TLS/pool kanıtı vardır; deadlock/serialization/multi-worker bütünlüğü ayrıca doğrulanır. |
| ZEE-019 | IO yetkinlik atlatma | Denetim hedefi; açık iddiası değil | K-205 | [compiler/tests/yetkinlik_testi.rs](../compiler/tests/yetkinlik_testi.rs) · [docs/yetkinlik-ve-ag-guvenligi.md](../docs/yetkinlik-ve-ag-guvenligi.md) | Mevcut compile/runtime kontrollerinin bütün adaptör yollarına eşlemesi çıkarılır. |
| ZEE-020 | Paket traversal/symlink | Denetim hedefi; mevcut ret korpusu | K-205 | [docs/zep-conformance.md](../docs/zep-conformance.md) · [docs/tedarik-zinciri.md](../docs/tedarik-zinciri.md) | Kaynak/cache/proje kurulumunda adversarial ve yarış yolları karşılaştırılır. |
| ZEE-021 | FFI bellek/tür | Denetim hedefi; kapsam ayrımı | K-205 | [docs/wasm-c-abi.md](../docs/wasm-c-abi.md) · [rfcs/0012-ffi-ve-tehlikeli-sinir.md](../rfcs/0012-ffi-ve-tehlikeli-sinir.md) | Gerçek WASM ABI testleri ve gelecekteki native FFI ayrı kanıtlanır. |
| ZEE-022 | HTTP differential fuzz | Denetim hedefi | K-205 | [compiler/tests/http_istegi_testi.rs](../compiler/tests/http_istegi_testi.rs) · [docs/fuzzing.md](../docs/fuzzing.md) | Mevcut hermetik/fuzz kapısı bağımsız parser eşdeğerliği diye adlandırılmaz. |
| ZEE-023 | Unicode spoofing | Denetim hedefi; profil kanıtı mevcut | K-205 | [rfcs/0002-lexical-ve-unicode.md](../rfcs/0002-lexical-ve-unicode.md) · [docs/morfoloji-dogrulama.md](../docs/morfoloji-dogrulama.md) | NFC/NFD/morfoloji testleri confusable politikasını bütünüyle kanıtlamaz. |
| ZEE-024 | Oturum race/crash | Denetim hedefi; mevcut ortak depo sınırı | K-205 | [docs/web-production-profili.md](../docs/web-production-profili.md) · [adr/079-erp-oturum-listeleme-ve-iptal.md](../adr/079-erp-oturum-listeleme-ve-iptal.md) | Normal yarış kanıtı power-loss/crash consistency diye sunulmaz. |
| ZEE-025 | Uzun fuzz coverage | Exact HEAD RC kanıtı açık | K-157 | [docs/fuzz-rc.md](../docs/fuzz-rc.md) · [docs/fuzz-rc-gecmisi-v1.tsv](../docs/fuzz-rc-gecmisi-v1.tsv) | 30–60 dk dört ASAN hedefi + saf Miri aynı kaynak/çevre ile gerekir. |
| ZEE-026 | Native/C envanter | cc driver dilimi kapandı; toplam kapsam açık | K-192/K-201 | [docs/derleme-platform-kaniti-v1.tsv](../docs/derleme-platform-kaniti-v1.tsv) | Üç platform cc/archiver path/hash/command kanıtı var; driver iç süreçleri/pkg-config/vcpkg açık. |
| ZEE-027 | Tehdit/saldırı matrisi | Dağıtık kararlar mevcut; toplu eşleme işi | K-205 | [SECURITY.md](../SECURITY.md) · [docs/guvenlik-bulgulari-v1.tsv](../docs/guvenlik-bulgulari-v1.tsv) | Mevcut güvenlik sınırları yüzey→kontrol→negatif test zincirine bağlanır. |

## Sıradaki ölçülebilir dilimler

- K-206: PostgreSQL Vec toplama davranışı için izole gerçek sunucu/RSS
  reproducer; 16 MiB uygulama sonucu ile driver/wire belleği ayrılır.
  Streaming tasarımı tek büyük satır, timeout, retry, transaction/savepoint
  ve başarısız sorgu sonrası bağlantı sağlığını birlikte ele alır.
- K-207: Ad çözümleme her başvuruda bütün tabloyu clone/sort ediyor.
  Exact ad ve morfoloji adayları hash üyeliğiyle çözülebilir; A001/A002
  liste sırası, span, öneri ve exact-ad önceliği uyumluluk testleriyle
  korunmalıdır. Temiz exact kaynakta aynı 2k/5k/10k/20k iş yükü ölçülür.
- İnsan kapıları B-001/B-002 gerçek anonim formları bekler. Backend,
  typed-result veya ergonomi yüzeyi için ürün→karar→spec→test zinciri
  kurulmadan core genişletilmez.

## K-206 ilk gerçek bellek deneyi

Exact temiz compiler 29be85b, Rust 1.93.1 release, M4 Pro/macOS 26.6.2
ve immutable PG 16.15 image ile izole loopback geçici sunucu kullanıldı.
Mevcut veritabanlarına erişilmedi. Her satır bir ayrı süreç, 1 örnek/0
ısınmadır; performans tarihçesi tabanı değildir. RSS süreç toplamıdır.

| Sentetik sonuç | Tepe RSS | Gözlem |
|---|---:|---|
| 1 × 1 MiB | 8,69 MiB | Başarılı; sonraki sorgu sağlıklı |
| 64 × 1 MiB | 128,77 MiB | 16 MiB sonuç reddi; sonraki sorgu sağlıklı |
| 1 × 64 MiB | 136,77 MiB | 16 MiB sonuç reddi; sonraki sorgu sağlıklı |

[Ham veri/çevre/hash](incelemeler/k206/rss-once.json),
[internal API deney helper'ı](incelemeler/k206/probe.rs) ve
[deney driver'ı](incelemeler/k206/driver.py) saklandı. Helper ve driver
`compiler/target/` altında çalıştırıldı; relative yollar o çalışma
alanına aittir. Helper normal gömme API örneği değildir. Üretim PG
modülü bu deneyde değiştirilmedi.

Çok satırın Vec'e toplanması ve büyük satırın String'e kopyalanması
farklı maliyetlerdir. K-206 bounded uygulama toplaması, hata önceliği,
retry ve tam-drain bağlantı sağlığını birlikte test edecektir. Tek wire
satırı/driver buffer sınırı 16 MiB uygulama sonucundan çıkarılmaz (GB-041).

K-207 kapsam dilimi exact 9e80b69 üzerinde 20k p95 165,137 ms ve
1800 sn/264.343 döngü RSS soak ile kapandı; incremental/cache açık.
K-206 [izole prototip/uzun TLS kanıtı](incelemeler/k206/README.md):
15 sorgu aynı sonuç/hata verisi ve transaction sağlığını verdi. 64×1 MiB
prototip RSS 23,7 MiB, tek 64 MiB satır 72,7 MiB; doğrudan stream helper
production adaptörü değildir. GB-041 ve üretim entegrasyonu açıktır.

K-206 akış toplama dilimi kapalı: source f2ba654, temiz beyan/ölçüm
6463a7f; 731/731 faz ve gerçek native oracle/transaction kabulü geçti.
64×1 MiB sonuçta native süreç RSS 128,77→23,81 MiB, tek 64 MiB satırda
136,77→72,72 MiB; üç vakada sonraki sorgu başarılıdır. Bir örnek/sıfır
ısınma gözlemi, genel RSS garantisi değildir. [Ham kanıt](incelemeler/k206/README.md).
**K-206 KISMEN:** tek wire satırı/driver sınırı ve GB-041 açık kalır.
