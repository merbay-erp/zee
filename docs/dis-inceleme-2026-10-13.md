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

**Tarihsel ilk eşleme:** aşağıdaki tablo ve ilk deney/sıradaki iş notları,
13 Ekim'deki seçili kaynak anının kaydıdır. Sonraki kapanışları silmez veya
yeniden açmaz. Güncel değerlendirme bu belgenin sonundaki **K-230 — 14 Ekim
2026 uzlaştırması** tablosudur; özellikle eski PG frame ve çok saniyeli LSP
ifadeleri bugün açık/başarısız kabul olarak okunmaz.

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

## Tarihsel sıradaki ölçülebilir dilimler — ilk eşleme

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

## K-230 — 14 Ekim 2026 uzlaştırması

K-205'in 27 dış kimliği aynı kapsamda korunur. Buradaki kimlikler **13 Ekim
incelemesinin ZEE-001..027 kimlikleridir**; 14 Ekim tarihli 18 maddelik listenin
aynı yazılan kimlikleriyle karıştırılmaz. Bu dilim kaynak/karar/test
izlenebilirliğini günceller; tam tehdit denetimi veya yeni bir saldırı
kampanyasının tamamlandığı anlamına gelmez.

Okunan commit noktası `0b9b75e2e2595d1565fd89ed8c1e313b05f25460`;
son compiler kaynak commit'i `81df2bd5aee1bc55a45e75098ac9e324ca6c9876`
(K-221 CLI bugfix)'tir. Tablodaki kaynak/test yolları bu commit ve incelenen
çalışma ağacına bağlanır. **Aday** işaretli K-222 TLS kabulü ve K-226 build
kimliği çalışma ağacındadır; bunların yerel sonucu yeni bir exact commit,
uzak CI, RC veya canlı ürün kabulü olarak sunulmaz. Bu uzlaştırmada Cargo,
fuzz, provider veya canlı ürün testleri yeniden çalıştırılmadı; test işlevinin
kaynakta bulunması tek başına testin geçtiği iddiası değildir.

Sınıflar: **eksik kapasite** gerçeklenmemiş işlev; **bilinçli sınır** karar
verilmiş destek kapsamı; **tarihsel** eski ölçüm/sürtünme; **kapalı alt kapsam**
yalnız belirtilen arızanın kabulü; **doğrulanmamış audit** ek insan/platform
veya hasım doğrulaması gereken iddia. Hiçbiri bütün maddenin veya bütün ürünün
tamamlandığı anlamına gelmez.

| Dış kimlik | Güncel sınıf | Mevcut K/GB | Exact kaynak / test işlevi ve kayıt | Güncel sonuç / kalan kabul |
|---|---|---|---|---|
| ZEE-001 | eksik kapasite | K-193 | [Derleme rehberi](derleme-ve-bagimliliklar.md), `compiler/Cargo.toml`; Zee→native backend kabul işlevi yok. | Rust bootstrap/interpreter çalışır; native codegen, ABI/bellek modeli ve semantic oracle ürün/V1 önkoşulları sonrası ayrı tasarım/kabul ister. |
| ZEE-002 | eksik kapasite | K-193; K-142/GB-013 kapalı alt kapsam | `compiler/src/wasm_api.rs`; `compiler/tests/playground_testi.rs::cabi_katmani_gidis_donus`; [WASM C ABI](wasm-c-abi.md). | Rust compiler'ın tarayıcı WASM derlemesi vardır; Zee→WASM codegen değildir. Backend eşdeğerlik korpusu/ABI kabulü yok. |
| ZEE-003 | eksik kapasite | K-193 | `compiler/Cargo.toml`, `compiler/src/lib.rs`; [README](../README.md). Self-hosting stage kabul işlevi yok. | Derleyici Rust tabanlıdır; self-hosting gerçekleşmiş gösterilmez. Stage 1 kapsamı ve bootstrap güven zinciri ayrı karar ister. |
| ZEE-004 | doğrulanmamış audit | B-001/B-002, K-161/K-162, K-174 | [Çağrı karar paketi](k016-cagri-karar-paketi.md); `compiler/tests/cagri_cikarimi_testi.rs::cikarimli_cagrida_eksik_ve_fazla_arguman_t015tir`; [freeze kaydı](v1-syntax-freeze-v1.tsv). | Kaynak/tür davranışı testi insan kullanılabilirliği değildir. Gerçek anonim formlar yok; 14 Ekim–11 Kasım freeze ve insan kararı açık. |
| ZEE-005 | eksik kapasite | K-207 | `compiler/src/lsp.rs::degisim_parametreleri`, `compiler/src/lsp.rs::belgeyi_guncelle`; `compiler/tests/lsp_testi.rs::didchange_temiz_metinle_tanilari_sifirlar`. | Tam metin didChange/ön uç analizi sürer; incremental/cache yok. Önce hatasız gerçek proje profili, sonra ihtiyaç kanıtı ve tanı eşliği/invalidation/1800 sn soak gerekir. |
| ZEE-006 | tarihsel | K-154/K-207 | [14 Ekim LSP ham ölçümü](incelemeler/k207/http-master-20261014/README.md), [değiştirilmeyen tarihçe](performans-gecmisi-v2.tsv); `compiler/tests/lsp_testi.rs::didchange_temiz_metinle_tanilari_sifirlar`. | Eski 20k p95 20.159,636 ms güncel performans diye taşınmaz. c743312 exact kaynakta gerçek düzenleme 20k p95 **165,731 ms**, 5 örnek/2 ısınmadır; sentetik gözlem, tüm proje/editör garantisi değildir. |
| ZEE-007 | tarihsel | K-165/ADR-073 | [Zee–Rust–Go kaydı](dogfood-karsilastirma.md); `compiler/tests/dogfood_karsilastirma_testi.rs::rust_esdegeri_zee_sayfasini_bayt_bayt_uretir`, `compiler/tests/dogfood_karsilastirma_testi.rs::karsilastirma_tarihcesi_exact_sha_ve_tekrar_hesaplanabilir_olculerle_tutarlidir`. | Tek kanıt-özeti işinde Zee derle+yürüt ile önceden derlenmiş Rust/Go karşılaştırılır. Oran tüm ürünlere genellenmez; yeni exact iş yükü/çıktı/ortam ölçümü olmadan yeni performans sonucu yok. |
| ZEE-008 | tarihsel | K-164/F003/F014; K-180 | [Kayıtlı ürün sürtünmesi](dogfood-karsilastirma.md), `dogfood/kanit-ozeti/kaynak/ana.dil`; `compiler/tests/dogfood_korpusu_testi.rs::dogfood_korpusu_basari_ve_basarisizlik_beklentilerini_korur`. | Ara değişken workaround'u gerçek ürün kaydıdır; bütün argüman/postfix biçimlerinin desteklendiği kanıtı değildir. Tekrarlanan ürün ihtiyacı ve minimal RFC/spec/olumlu-olumsuz reproducer olmadan grammar işi açılmaz. |
| ZEE-009 | tarihsel | K-164/F010/F011; K-180/RFC-0029 | [Kayıtlı indeks/kaçış sürtünmesi](dogfood-karsilastirma.md), `dogfood/kanit-ozeti/kaynak/ana.dil`; `compiler/tests/dogfood_korpusu_testi.rs::dogfood_korpusu_basari_ve_basarisizlik_beklentilerini_korur`. | Mevcut ürünün workaround kabulü yeni indeks/sekme kaçışı yüzeyi değildir. Ergonomi tasarımı insan/ürün kanıtı ve freeze kararı bekler. |
| ZEE-010 | doğrulanmamış audit | K-192/K-200; GB-037 kapalı alt kapsam | [Platform kayıtları](derleme-platform-kaniti-v1.tsv); `scripts/tests/native-deployment-test.py::test_yeni_native_hedef_rust_esigini_asarsa_reddedilir`; [derleme rehberi](derleme-ve-bagimliliklar.md). | C/Rust deployment/minos uyumu kayıtlıdır; gerçek minimum OS runtime kabulü değildir. Desteklenen minimum işletim sistemleri temiz fiziksel cihazda program/LSP/kur-kaldır kanıtı ister. |
| ZEE-011 | doğrulanmamış audit | K-192/K-197/K-198; GB-034/035 kabul | [Temiz Windows kayıtları](temiz-windows-platform-kaniti-v1.tsv); `compiler/tests/kurulum_testi.rs::kurulum_betikleri_ve_tatbikat_workflowu_sozlesmeyi_tasir`. | Container/runtime ve kurulum tatbikatı kapalı alt dilimlerdir. Temiz fiziksel Windows/macOS ve minimum OS kanıtı yok; container veya yerel development makinesi bunların yerine geçmez. |
| ZEE-012 | eksik kapasite | RFC-0012/K-193 | [FFI taslağı](../rfcs/0012-ffi-ve-tehlikeli-sinir.md); `compiler/tests/public_api_testi.rs::v1_facade_exact_allowlist_disinda_yeni_ihracat_kabul_etmez`; [public API](public-api-politikasi.md). | Desteklenen Rust facade `api::v1` ve WASM C ABI, genel native FFI değildir. Native çağrı/bellek/tür sahipliği tasarımı ve unsafe negatif korpus ayrı gerekir. |
| ZEE-013 | bilinçli sınır | K-137/K-218/K-219; GB-008 kapalı, GB-051/052 kabul | [Web production profili](web-production-profili.md); `compiler/tests/web_cok_surec_testi.rs::iki_worker_oturumu_iptali_restarti_ve_giris_limitini_paylasir`, `compiler/tests/web_cok_surec_testi.rs::bellek_oran_iki_worker_oturum_revoke_restart_paylasir_kota_yereldir`. | N-worker ve ortak oturum/revoke vardır; worker içi eşzamanlılık yok. Explicit bellek kotası süreç/restart bazındadır; çok-worker ortak saldırı bütçesi proxy koşuluyla kabul edilir. Draining/çok-host bütçe ayrı ürün kabulüdür. |
| ZEE-014 | bilinçli sınır | ADR-045/049 | `compiler/src/bin/olcum.rs::shared_ci_esiksizdir_adanmis_kosu_acikca_ihlal_uretir`; [ölçüm politikası](olcumler.md). | Shared CI ölçümü gözlemseldir; hard eşik kapatılmış eksik özellik değildir. Adanmış sabit koşucu, exact kaynak/ortam ve gerçek sample/warmup kabul edilmeden eşik açılmaz. |
| ZEE-015 | bilinçli sınır | K-164/F006–F013, K-180; K-089 | [Morfoloji profil uyumu](morfoloji-profil-uyumlulugu.md); `compiler/tests/morfoloji_testi.rs::zee_tr_1_semantik_parmak_izi_immutable_fixture_ile_ayni_kalir`, `compiler/tests/morfoloji_testi.rs::belirsizlik_korpusu_tahmin_edilmeden_a002_olur`. | Immutable zee-tr-1 ve A002 fail-closed korunur; ad ergonomisi insan/ürün kararıdır. Bu testler yeni profil veya confusable politikasını tamamlamaz. |
| ZEE-016 | kapalı alt kapsam | K-206; GB-041 kapalı | `compiler/src/postgresql/sonuc.rs::oku`; `compiler/src/postgresql/sonuc/testler.rs::hata_sonrasi_kopyalama_durur_ve_satir_siniri_onceliklidir`; `compiler/src/postgresql.rs::izole_wire_siniri_retry_transaction_rls_ve_commit_sinirini_korur`; [exact frame/native/RSS kapanışı](incelemeler/k206/frame-guard/README.md). | Bounded uygulama toplaması ve iki parser'da gövde/reserve öncesi 32 MiB guard üretimde etkindir. a4e1045 exact kaydında 13 native vaka ve RSS gözlemi var; eski “GB-041/tek frame açık” yalnız tarihsel kayıttır. **Global RSS tavanı ve geniş fault matrisi yok.** |
| ZEE-017 | bilinçli sınır | K-163/RFC-0026; aday K-222 | `compiler/src/postgresql/sonuc.rs::alanlar`; `compiler/tests/postgresql_testi.rs::okuma_parametreyi_sql_metininden_ayri_tasir`; aday `compiler/tests/postgresql_testi.rs::izole_tls_null_ve_metin_disini_reddeder_transaction_sagligi_korunur`; [spec/25](../spec/25-postgresql-veri-erisimi.md). | Sonuç Metin sütun sözleşmesidir; NULL/non-text açık COALESCE/::text ister. Typed-result genel API talebi yeni ürün/versiyon kanıtı ister; aday NULL gerçek testi typed API eklenmiş sayılmaz. |
| ZEE-018 | kapalı alt kapsam | K-206; aday K-222/GB-022 kapanış kaydı | `compiler/src/postgresql.rs::yalniz_transaction_disindaki_okuma_bir_kez_yeniden_baglanabilir`, `compiler/src/postgresql.rs::izole_wire_siniri_retry_transaction_rls_ve_commit_sinirini_korur`; [frame 13-vaka kaydı](incelemeler/k206/frame-guard/README.md), [aday TLS kapısı](postgresql-tls-ci.md). | Retry/nested/RLS/cleanup/gönderilmiş COMMIT sınırı mevcut alt kapsamdır. Aday TLS kapısı dört gerçek CA/hostname/NULL vakasını yerelde sıfır atlamayla koştu; uzak exact kaynak sonucu açık. Deadlock/serialization/fault-proxy/çok-worker tam matrisi, TLS 1.1 downgrade ve provider rotasyonu tamamlanmadı. |
| ZEE-019 | doğrulanmamış audit | K-205/K-230; GB-016 kabul | `compiler/tests/yetkinlik_testi.rs::derleme_oncesi_kullanilan_yetkinligi_kesin_kaynakta_reddeder`, `compiler/tests/yetkinlik_testi.rs::kullanilmayan_islem_de_yetkinligi_gizleyemez`; `compiler/src/yorumlayici/yetkinlik/dosya.rs::yolu_denetle`; [yetkinlik sınırı](yetkinlik-ve-ag-guvenligi.md). | Compile/runtime kontrolleri vardır; bütün dosya/ağ/SMTP/DB/iz adaptörlerinin bütün çağrı ve yarış yollarını kapsayan threat audit değildir. Dil yetkinliği düşman yerel süreçlere karşı OS sandbox'ı diye sunulmaz. |
| ZEE-020 | doğrulanmamış audit | K-205/K-230; RFC-0020/ADR-028 | `compiler/src/artefakt_dogrulama.rs::kalici_saldiri_korpusu_butun_unicode_yol_saldirilarini_reddeder`; `compiler/tests/proje_testi.rs::proje_dosya_siniri_sembolik_bag_kacisini_reddeder`; `compiler/tests/tedarik_testi.rs::imza_arsiv_sbom_ve_provenance_oynamalari_fail_closed_reddedilir`; [ZEP korpusu](zep-conformance.md). | 80 yol saldırısı, arşiv tekilliği ve symlink/imza retleri mevcuttur. Paket/cache/proje kurulumu ile eşzamanlı düşman yerel değişiklik/TOCTOU yollarının birleşik matrisi ayrıca gerekir; mevcut örnekler bütün traversal/race audit'i değildir. |
| ZEE-021 | doğrulanmamış audit | K-142/GB-013 kapalı alt kapsam; RFC-0012 | `compiler/tests/playground_testi.rs::cabi_kayit_disi_pointer_uzunluk_ve_tasmayi_reddeder`, `compiler/tests/playground_testi.rs::cabi_yanlis_ve_cift_birakmada_sahipligi_korur`; `compiler/src/wasm_api/abi.rs`; [WASM sınırı](wasm-c-abi.md). | Mevcut WASM pointer/boy/UTF-8/sahiplik ve bütçe retleri kanıtlı alt kapsamdır. Gelecekteki native FFI, reentrancy/iş parçacığı/host yaşam döngüsü veya geniş bellek audit'i bunlardan türetilmez. |
| ZEE-022 | doğrulanmamış audit | K-140/GB-011 kapalı alt kapsam; K-205/K-230 | `compiler/tests/http_istegi_testi.rs::transfer_encoding_duplicate_ve_kuralsiz_content_length_reddedilir`, `compiler/tests/http_istegi_testi.rs::kalici_http_fuzz_korpusu_her_ana_testte_yeniden_oynatilir`; `compiler/fuzz/fuzz_targets/http_istegi.rs`; [fuzz rehberi](fuzzing.md). | Framing negatif korpusu ve fuzz girişi vardır. Hedef yalnız Zee parser'ı çağırır; bağımsız parser/proxy oracle'ıyla differential fuzz ve kaynak/sürüm bağlı farklılık sınıflaması yapılmış sayılmaz. |
| ZEE-023 | doğrulanmamış audit | K-089/K-205/K-230; RFC-0002 | `compiler/tests/morfoloji_testi.rs::unicode_normalizasyon_varyantlari_lexer_sinirinda_fail_closed_kalir`; `compiler/tests/morfoloji_conformance_testi.rs::bagimsiz_json_korpusu_profil_cozum_karar_ve_uretimi_birebir_tanimlar`; [Unicode kararı](../rfcs/0002-lexical-ve-unicode.md). | NFC/NFD/morfoloji karar korpusu vardır. Genel tanımlayıcı confusable/bidi/spoof politikası ve bütün display/rename/registry yollarının saldırı korpusu ayrıca denetlenir; normalizasyon testi tam spoof audit'i değildir. |
| ZEE-024 | doğrulanmamış audit | K-134/K-137/K-188; GB-007/008/028 kapalı alt kapsam | `compiler/src/web_guvenligi/tests.rs::kalici_oturum_commiti_yanit_hatasinda_kosullu_geri_alinir`, `compiler/src/web_guvenligi/tests.rs::kalici_rate_limit_paralel_artisla_esigi_deldirmez`; `compiler/src/kalici_dosya.rs::aniden_biten_surecin_kilidi_isletim_sistemi_birakir`; [oturum kararı](../adr/079-erp-oturum-listeleme-ve-iptal.md). | Normal eşzamanlı worker, koşullu rollback ve process-exit kilit bırakma vardır. Her write/fsync/rename/yanıt sınırındaki kill, disk arızası ve power-loss/crash consistency birlikte kanıtlanmadı. |
| ZEE-025 | doğrulanmamış audit | K-157/K-170 | `compiler/tests/fuzz_korpusu_testi.rs::rc_kampanyasi_dort_hedefi_address_sanitizer_ve_miriyle_korur`; [RC tarihçesi](fuzz-rc-gecmisi-v1.tsv), [RC sözleşmesi](fuzz-rc.md). | Tarihçenin dört 1800 sn ASan satırı eski b75072be exact kaynak içindir; workflow kablosu güncel kampanya sonucu değildir. Güncel temiz exact kaynakta dört hedef 30–60 dakika explicit ASan, sıfır crash/timeout ve seçili saf Miri gerekir. |
| ZEE-026 | kapalı alt kapsam | K-192/K-201; GB-038 kabul | [Üç-platform derleme kayıtları](derleme-platform-kaniti-v1.tsv); `scripts/tests/derleme-ortami-test.py::test_native_compile_ve_archive_kaniti_zorunludur`, `scripts/tests/derleme-ortami-test.py::test_native_komutlar_calistirilmadan_kimliklenir`. | Üreticinin iki klon logundan cc/archiver path/hash/komut kimliği kapalı alt dilimdir. Driver iç süreçleri, pkg-config/vcpkg, SDK tamamı ve yeni kaynakta fiziksel/minimum OS kabulü açık kalır. |
| ZEE-027 | doğrulanmamış audit | K-205/K-230/K-170; GB-046 açık | `compiler/tests/guvenlik_kapisi_testi.rs::bulgu_kaydi_gecerli_ve_acik_kritik_yuksek_bulgu_yoktur`, `compiler/tests/guvenlik_kapisi_testi.rs::kayit_semasi_bozuk_ve_acik_agir_bulguyu_reddeder`; [güvenlik kayıtları](guvenlik-bulgulari-v1.tsv), [pilot sınırı](urun-destek-politikasi.md). | 27/27 izlenebilirlik bu tabloda vardır. Her saldırı yüzeyinin negative test/tehdit modeli/owner ve güncel exact koşu matrisi tamamlanmış değildir; kayıt şeması kontrolü olmayan audit'i var yapmaz. GitHub yönetilen PR refs/cache dış teyidi açık kalır. |

### Kanıtın güncellik sınırı

PG akış ve frame kapanışı yukarıdaki tarihsel negatif deneyleri geçersiz veri
yapmaz; onları önce/sonra sınırıyla tamamlar. [Frame kaydı](incelemeler/k206/frame-guard/README.md)
ilk aday, üretime bağlanma ve exact native/RSS kabulünü ayrı tutar. GB-041
kapanmıştır; sınırsız tüm-süreç RSS veya bütün PG fault yolları sözü verilmez.

LSP'de güncel sentetik eşik gözlemi eski çok saniyeli maliyeti güncel sorun
diye taşımayı engeller. Aynı [14 Ekim kaydı](incelemeler/k207/http-master-20261014/README.md)
üretilen modül kataloğu bulunmayan proje snapshot'ında iki A010 olduğunu da
yazar; bu sayı hatasız ürün/proje kabulü veya compiler bug kanıtı değildir.
Cache/artımlılık bu belge işiyle eklenmedi ve yeniden ölçüm yapılmadı.

K-226 aday build kimliği yerel `scripts/tests/build-kimligi-test.py` içindeki
12 bağımlılıksız fixture testiyle kaynak SHA/tarih/kanal ve iki-klon fixture
eşliğini sınar. Bu, compiler'ın signed release/provenance/fiziksel platform
kabulü değildir; [izolasyon sınırı](incelemeler/urunlesme-20261014/build-kimligi-isolasyonu.md)
ve yeni exact kaynak commit'i ayrı kalır. K-222, K-223/K-225 ürün helper ve
diğer çalışma ağacı kabulleri de yalnız kendi kayıtlı yerel kapsamlarında
değerlendirilir; bu 27 maddeyi topluca kapatmaz.

Bu uzlaştırmanın kapanışı yalnız **27 kimliğin sınıf/K/GB/kaynak/test/kalan
kabul eşlemesidir**. ZEE-019..024 ve ZEE-027'nin adversarial denetimleri,
insan/platform/RC kanıtları ve yeni kapasite kararları açık kayıt olarak
korunur. Yeni doğrulanmış arıza çıkarsa tek reproducer, etkisi, güvenlik
sınıfı ve ilgili K/GB/test/regression zinciri olmadan “kapalı” yazılmaz.
