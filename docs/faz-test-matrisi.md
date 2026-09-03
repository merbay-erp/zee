# Faza özgü test matrisi

> Bu dosya `cargo run --locked --bin faz_test_matrisi -- --dokuman-yaz` ile
> üretilir. Faz sahipliği ve saldırı yüzeyi ilişkileri elle değiştirilmez;
> gerçek test sayıları her platformun dinamik CI raporunda çıkar.

- Şema: `zee-faz-test-matrisi-2`
- Faz: **22**
- Sayım: Her CI işletim sisteminde derlenen gerçek test envanteri

## Envanter ve saldırı yüzeyi ilişkileri

| Faz | Kalıcı regresyon kaynağı | Fuzz | Conformance | Aşağı akış |
|---|---|---|---|---|
| Lexer | `compiler/tests/fuzz_korpusu_testi.rs`<br>`compiler/fuzz/corpus/lexer_parser` | `compiler/fuzz/fuzz_targets/lexer_parser.rs`<br>`compiler/fuzz/corpus/lexer_parser` | — | `parser`<br>`ast`<br>`cozumleyici`<br>`tur`<br>`hir`<br>`runtime`<br>`lsp`<br>`wasm`<br>`cli`<br>`proje`<br>`uctan_uca` |
| Parser | `compiler/tests/ifade_grameri_testi.rs`<br>`compiler/tests/parser_kurtarma_testi.rs`<br>`regression/parser`<br>`golden` | `compiler/fuzz/fuzz_targets/lexer_parser.rs`<br>`compiler/fuzz/corpus/lexer_parser` | — | `ast`<br>`cozumleyici`<br>`tur`<br>`hir`<br>`runtime`<br>`lsp`<br>`wasm`<br>`cli`<br>`proje`<br>`semantic_regresyon`<br>`uctan_uca` |
| AST | `compiler/tests/faz_modeli_testi.rs`<br>`compiler/tests/invariant_testi.rs` | — | — | `cozumleyici`<br>`tur`<br>`hir`<br>`lsp`<br>`wasm`<br>`uctan_uca` |
| Resolver | `compiler/tests/birim_testi.rs`<br>`compiler/tests/kapsam_testi.rs`<br>`compiler/tests/semantic_kimlik_testi.rs` | — | — | `tur`<br>`hir`<br>`runtime`<br>`lsp`<br>`proje`<br>`uctan_uca` |
| Type checker | `compiler/tests/acik_imza_testi.rs`<br>`compiler/tests/cagri_cikarimi_testi.rs`<br>`compiler/tests/daraltma_testi.rs`<br>`compiler/tests/sonuc_donusu_testi.rs`<br>`regression/checker` | — | — | `hir`<br>`runtime`<br>`lsp`<br>`wasm`<br>`cli`<br>`proje`<br>`semantic_regresyon`<br>`uctan_uca` |
| Typed HIR | `compiler/tests/hir_modeli_testi.rs`<br>`compiler/src/hir/testler.rs`<br>`regression/hir` | — | — | `runtime`<br>`lsp`<br>`wasm`<br>`semantic_regresyon`<br>`uctan_uca` |
| Morphology | `compiler/tests/morfoloji_v1.snapshot`<br>`compiler/tests/fixtures/morfoloji-zee-tr-1.sha256`<br>`regression/morphology` | `compiler/fuzz/fuzz_targets/morfoloji.rs`<br>`compiler/fuzz/corpus/morfoloji` | `conformance/morfoloji/sema-v1.schema.json`<br>`conformance/morfoloji/zee-tr-1.json` | `lexer`<br>`cozumleyici`<br>`lsp`<br>`cli`<br>`proje`<br>`semantic_regresyon`<br>`uctan_uca` |
| Runtime | `compiler/tests/dene_ve_sure_testi.rs`<br>`compiler/tests/ondalik_testi.rs`<br>`compiler/tests/yapilandirilmis_hata_testi.rs`<br>`regression/runtime` | — | — | `io`<br>`eszamanlilik`<br>`web_guvenlik`<br>`wasm`<br>`cli`<br>`semantic_regresyon`<br>`uctan_uca` |
| IO | `compiler/tests/deterministik_io_profili_testi.rs`<br>`compiler/tests/io_izi_testi.rs`<br>`compiler/tests/postgresql_testi.rs` | — | — | `eszamanlilik`<br>`web_guvenlik`<br>`cli`<br>`proje`<br>`uctan_uca` |
| Concurrency | `compiler/tests/ag_ve_esz_testi.rs`<br>`conformance/eszamanlilik/zee-esz-1.json`<br>`regression/concurrency` | — | `conformance/eszamanlilik/sema-v1.schema.json`<br>`conformance/eszamanlilik/zee-esz-1.json` | `runtime`<br>`io`<br>`web_guvenlik`<br>`wasm`<br>`semantic_regresyon`<br>`uctan_uca` |
| Web/security | `compiler/tests/web_testi.rs`<br>`compiler/tests/web_cok_surec_testi.rs`<br>`compiler/tests/kaynak_sinirlari_testi.rs`<br>`regression/security` | — | — | `http`<br>`io`<br>`eszamanlilik`<br>`cli`<br>`proje`<br>`semantic_regresyon`<br>`uctan_uca` |
| HTTP parser | `compiler/tests/http_istegi_testi.rs`<br>`compiler/fuzz/corpus/http_istegi` | `compiler/fuzz/fuzz_targets/http_istegi.rs`<br>`compiler/fuzz/corpus/http_istegi` | — | `web_guvenlik`<br>`cli`<br>`uctan_uca` |
| Package | `compiler/tests/tedarik_testi.rs`<br>`compiler/tests/fixtures/zep-kanonik-v1.hex`<br>`compiler/tests/fixtures/zep-saldiri-korpusu` | — | `docs/zep-conformance.md`<br>`compiler/tests/fixtures/zep-kanonik-v1.hex` | `registry`<br>`tedarik_zinciri`<br>`cli`<br>`proje`<br>`uctan_uca` |
| Registry | `compiler/src/registry/istemci/testler.rs`<br>`compiler/tests/fixtures/zep-saldiri-korpusu` | — | — | `paket`<br>`tedarik_zinciri`<br>`cli`<br>`proje` |
| Supply-chain | `compiler/tests/tedarik_kapisi_testi.rs`<br>`compiler/deny.toml`<br>`compiler/Cargo.lock`<br>`compiler/fuzz/Cargo.lock`<br>`docs/github-actions-pinleri-v1.tsv` | — | — | `paket`<br>`registry`<br>`proje`<br>`muhe_kapilari` |
| LSP | `compiler/tests/lsp_testi.rs`<br>`compiler/tests/olcum_lsp_process_testi.rs` | — | — | `cli`<br>`uctan_uca` |
| WASM | `compiler/tests/playground_testi.rs`<br>`compiler/fuzz/corpus/wasm_abi` | `compiler/fuzz/fuzz_targets/wasm_abi.rs`<br>`compiler/fuzz/corpus/wasm_abi` | — | `uctan_uca` |
| CLI | `compiler/tests/io_izi_cli_testi.rs`<br>`golden/28-cli-araci.dil` | — | — | `proje`<br>`uctan_uca` |
| Project system | `compiler/tests/proje_testi.rs`<br>`projeler` | — | — | `paket`<br>`registry`<br>`cli`<br>`uctan_uca` |
| Semantic regression | `regression/v2.tsv`<br>`regression`<br>`scripts/semantic-regresyon-korugu.sh`<br>`scripts/core-freeze-korugu.sh`<br>`docs/compiler-degisiklik-beyanlari-v1.tsv`<br>`docs/core-freeze-beyanlari-v1.tsv` | — | — | `uctan_uca` |
| End-to-end | `golden` | — | — | — |
| Engineering gates | `compiler/tests/fixtures/islev-egilimi-v1.tsv`<br>`compiler/tests/fixtures/izinli-katman-cevrimleri-v1.tsv`<br>`compiler/tests/fixtures/katman-mimarisi-v1.tsv`<br>`compiler/tests/fixtures/tani-kimlikleri-v1.tsv`<br>`docs/kanit-haritasi-v1.tsv`<br>`docs/performans-gecmisi-v2.tsv`<br>`docs/olcumler.md` | — | — | — |

## Birincil sahiplik

Bir test tam bir birincil faza aittir. Seçici düzeyindeki isteğe bağlı
ek kapsam, test grubunun gerçekten yokladığı diğer fazları bildirir.

| Faz | Cargo/libtest seçicileri | Ek kapsam | Kapsam |
|---|---|---|---|
| Lexer | `test:fuzz_korpusu_testi`<br>`test:kacis_ve_negatif_testi` | `test:fuzz_korpusu_testi → parser, morfoloji, http, wasm` | Token, Unicode, kacis, sayi ve girinti siniri. |
| Parser | `test:bicimleyici_testi`<br>`test:ifade_grameri_testi`<br>`test:parser_kurtarma_testi` | `test:bicimleyici_testi → lexer, ast`<br>`test:ifade_grameri_testi → lexer, ast`<br>`test:parser_kurtarma_testi → lexer, ast` | Ifade onceligi, tam tuketim, kurtarma ve bicim esdegerligi. |
| AST | `test:faz_modeli_testi`<br>`test:invariant_testi`<br>`doc:dil` | — | Faz tipleri, kaynak araliklari ve semantic alansizlik degismezleri. |
| Resolver | `test:birim_testi`<br>`test:kapsam_testi`<br>`test:semantic_kimlik_testi` | — | Kapsam, birim ve kararli semantic kimlik baglari. |
| Type checker | `test:acik_imza_testi`<br>`test:cagri_cikarimi_testi`<br>`test:daraltma_testi`<br>`test:ffi_sinir_testi`<br>`test:intrinsic_lowering_testi`<br>`test:koleksiyon_testi`<br>`test:sonuc_donusu_testi` | — | Imza, cikarim, daraltma, koleksiyon, intrinsic ve sonuc turleri. |
| Typed HIR | `test:hir_modeli_testi`<br>`lib:hir::testler::` | — | SymbolId, tur ve kesin kaynak araligi tasiyan baglanmis temsil. |
| Morphology | `test:morfoloji_conformance_testi`<br>`test:morfoloji_testi` | `test:morfoloji_conformance_testi → lexer, cozumleyici`<br>`test:morfoloji_testi → lexer, cozumleyici, lsp` | zee-tr-1 uretim, cozum, belirsizlik ve Unicode profili. |
| Runtime | `test:dene_ve_sure_testi`<br>`test:metin_testi`<br>`test:ondalik_testi`<br>`test:ozyineleme_testi`<br>`test:siralama_testi`<br>`test:yapilandirilmis_hata_testi`<br>`lib:ondalik::testler::`<br>`lib:zaman::testler::` | — | Deger, islem, hata, sure, ondalik ve yurutme semantigi. |
| IO | `test:deterministik_io_profili_testi`<br>`test:io_izi_testi`<br>`test:postgresql_testi`<br>`lib:ag_istemcisi::testler::`<br>`lib:kalici_dosya::tests::`<br>`lib:postgresql::testler::` | — | Deterministik profil, replay, ag yaniti, PostgreSQL parametre/Sonuc siniri ve atomik kalici dosya. |
| Concurrency | `test:ag_ve_esz_testi`<br>`test:eszamanlilik_conformance_testi` | — | Scheduler, gorev agaci, iptal ve kararli gozlem sirasi. |
| Web/security | `test:guvenli_testi`<br>`test:kaynak_sinirlari_testi`<br>`test:web_cok_surec_testi`<br>`test:web_testi`<br>`test:yetkinlik_testi`<br>`lib:guvenlik::tests::`<br>`lib:kaynak_sinirlari::testler::`<br>`lib:web_guvenligi::tests::`<br>`lib:yetkinlik::testler::`<br>`bin:dil:web_profili_testleri::` | `test:web_testi → runtime, http, io, eszamanlilik`<br>`test:kaynak_sinirlari_testi → http, lsp, wasm` | Yetkinlik, kaynak butcesi, oturum, CSRF, proxy ve transaction guvenligi. |
| HTTP parser | `test:http_istegi_testi` | — | Request-line, CRLF, header ve govde framing byte siniri. |
| Package | `test:kitaplik_testi`<br>`test:tedarik_testi`<br>`lib:paket::uzak::politika::testler::`<br>`lib:artefakt_dogrulama::testler::` | — | Saf model, cozum, registry aktarimi, ZEP arsivi, imza, SBOM ve provenance davranisi. |
| Registry | `lib:registry::istemci::testler::`<br>`lib:registry::testler::`<br>`bin:dil:dil_registry::testler::` | — | Metadata zinciri, rollback, cache, exact pin ve CLI secenekleri. |
| Supply-chain | `test:tedarik_kapisi_testi` | — | RustSec, lisans, kilit, action pini ve offline vendor politikasinin yurutulebilir kapisi. |
| LSP | `test:lsp_testi`<br>`test:olcum_lsp_process_testi`<br>`lib:lsp::cikti::testler::`<br>`lib:lsp::kaynak_siniri_testleri::`<br>`bin:dillsp:testler::` | `test:lsp_testi → lexer, parser, cozumleyici, tur, hir` | JSON-RPC framing tani hover completion definition rename process cold-start ve tam-metin degisim olcegi. |
| WASM | `test:playground_testi` | `test:playground_testi → lexer, parser, cozumleyici, tur, hir, runtime` | Playground, ABI sahipligi, limitler ve gercek wasm hostu oncesi native regresyonlar. |
| CLI | `test:io_izi_cli_testi` | — | Komut satiri IO kaydi ve yeniden oynatma siniri. |
| Project system | `test:proje_testi`<br>`test:projeler_testi` | `test:proje_testi → paket, registry, cli`<br>`test:projeler_testi → lexer, parser, cozumleyici, tur, hir, runtime, cli` | Bildirim, kilit, bagimlilik, yetkinlik ve tam proje ornekleri. |
| Semantic regression | `test:core_freeze_korugu_testi`<br>`test:semantic_regresyon_korpusu_testi`<br>`test:semantic_regresyon_korugu_testi` | `test:semantic_regresyon_korpusu_testi → parser, cozumleyici, tur, hir, morfoloji, runtime, eszamanlilik, web_guvenlik` | Duzeltilmis bug provenance'i semantic beyan ve gercek dogfood kaniti isteyen executable core-freeze kapisi. |
| End-to-end | `test:golden_testi` | `test:golden_testi → lexer, parser, ast, cozumleyici, tur, hir, runtime, cli` | 33 golden program ile kullanici yuzeyinden tam derleme ve yurutme hatti. |
| Engineering gates | `test:bagimlilik_cevrimi_testi`<br>`test:dokuman_tazelik_testi`<br>`test:katalog_testi`<br>`test:katman_mimarisi_testi`<br>`test:mimari_sinir_testi`<br>`test:panic_guvenligi_testi`<br>`test:public_api_testi`<br>`test:tani_kimligi_testi`<br>`bin:islev_egilimi:testler::`<br>`bin:faz_test_matrisi:testler::`<br>`bin:olcum:testler::` | `test:public_api_testi → lexer, parser, cozumleyici, tur, hir, runtime` | Belge tani mimari panic bicim katman cevrim public API ve exact provenance/esik performans gozetim kapilarinin kendi regresyonlari. |

## Gerçek blast radius

Bir faz değiştiğinde kendi birincil gruplarına ek olarak aşağıdaki çapraz
seçiciler doğrudan kanıt taşır. Aşağı akış sütunu mimari yayılımı gösterir.

| Değişen faz | Birincil test grupları | Çapraz kapsayan test grupları | Aşağı akış |
|---|---|---|---|
| Lexer | `test:fuzz_korpusu_testi`<br>`test:kacis_ve_negatif_testi` | `parser / test:bicimleyici_testi`<br>`parser / test:ifade_grameri_testi`<br>`parser / test:parser_kurtarma_testi`<br>`morfoloji / test:morfoloji_conformance_testi`<br>`morfoloji / test:morfoloji_testi`<br>`lsp / test:lsp_testi`<br>`wasm / test:playground_testi`<br>`proje / test:projeler_testi`<br>`uctan_uca / test:golden_testi`<br>`muhe_kapilari / test:public_api_testi` | `parser`<br>`ast`<br>`cozumleyici`<br>`tur`<br>`hir`<br>`runtime`<br>`lsp`<br>`wasm`<br>`cli`<br>`proje`<br>`uctan_uca` |
| Parser | `test:bicimleyici_testi`<br>`test:ifade_grameri_testi`<br>`test:parser_kurtarma_testi` | `lexer / test:fuzz_korpusu_testi`<br>`lsp / test:lsp_testi`<br>`wasm / test:playground_testi`<br>`proje / test:projeler_testi`<br>`semantic_regresyon / test:semantic_regresyon_korpusu_testi`<br>`uctan_uca / test:golden_testi`<br>`muhe_kapilari / test:public_api_testi` | `ast`<br>`cozumleyici`<br>`tur`<br>`hir`<br>`runtime`<br>`lsp`<br>`wasm`<br>`cli`<br>`proje`<br>`semantic_regresyon`<br>`uctan_uca` |
| AST | `test:faz_modeli_testi`<br>`test:invariant_testi`<br>`doc:dil` | `parser / test:bicimleyici_testi`<br>`parser / test:ifade_grameri_testi`<br>`parser / test:parser_kurtarma_testi`<br>`uctan_uca / test:golden_testi` | `cozumleyici`<br>`tur`<br>`hir`<br>`lsp`<br>`wasm`<br>`uctan_uca` |
| Resolver | `test:birim_testi`<br>`test:kapsam_testi`<br>`test:semantic_kimlik_testi` | `morfoloji / test:morfoloji_conformance_testi`<br>`morfoloji / test:morfoloji_testi`<br>`lsp / test:lsp_testi`<br>`wasm / test:playground_testi`<br>`proje / test:projeler_testi`<br>`semantic_regresyon / test:semantic_regresyon_korpusu_testi`<br>`uctan_uca / test:golden_testi`<br>`muhe_kapilari / test:public_api_testi` | `tur`<br>`hir`<br>`runtime`<br>`lsp`<br>`proje`<br>`uctan_uca` |
| Type checker | `test:acik_imza_testi`<br>`test:cagri_cikarimi_testi`<br>`test:daraltma_testi`<br>`test:ffi_sinir_testi`<br>`test:intrinsic_lowering_testi`<br>`test:koleksiyon_testi`<br>`test:sonuc_donusu_testi` | `lsp / test:lsp_testi`<br>`wasm / test:playground_testi`<br>`proje / test:projeler_testi`<br>`semantic_regresyon / test:semantic_regresyon_korpusu_testi`<br>`uctan_uca / test:golden_testi`<br>`muhe_kapilari / test:public_api_testi` | `hir`<br>`runtime`<br>`lsp`<br>`wasm`<br>`cli`<br>`proje`<br>`semantic_regresyon`<br>`uctan_uca` |
| Typed HIR | `test:hir_modeli_testi`<br>`lib:hir::testler::` | `lsp / test:lsp_testi`<br>`wasm / test:playground_testi`<br>`proje / test:projeler_testi`<br>`semantic_regresyon / test:semantic_regresyon_korpusu_testi`<br>`uctan_uca / test:golden_testi`<br>`muhe_kapilari / test:public_api_testi` | `runtime`<br>`lsp`<br>`wasm`<br>`semantic_regresyon`<br>`uctan_uca` |
| Morphology | `test:morfoloji_conformance_testi`<br>`test:morfoloji_testi` | `lexer / test:fuzz_korpusu_testi`<br>`semantic_regresyon / test:semantic_regresyon_korpusu_testi` | `lexer`<br>`cozumleyici`<br>`lsp`<br>`cli`<br>`proje`<br>`semantic_regresyon`<br>`uctan_uca` |
| Runtime | `test:dene_ve_sure_testi`<br>`test:metin_testi`<br>`test:ondalik_testi`<br>`test:ozyineleme_testi`<br>`test:siralama_testi`<br>`test:yapilandirilmis_hata_testi`<br>`lib:ondalik::testler::`<br>`lib:zaman::testler::` | `web_guvenlik / test:web_testi`<br>`wasm / test:playground_testi`<br>`proje / test:projeler_testi`<br>`semantic_regresyon / test:semantic_regresyon_korpusu_testi`<br>`uctan_uca / test:golden_testi`<br>`muhe_kapilari / test:public_api_testi` | `io`<br>`eszamanlilik`<br>`web_guvenlik`<br>`wasm`<br>`cli`<br>`semantic_regresyon`<br>`uctan_uca` |
| IO | `test:deterministik_io_profili_testi`<br>`test:io_izi_testi`<br>`test:postgresql_testi`<br>`lib:ag_istemcisi::testler::`<br>`lib:kalici_dosya::tests::`<br>`lib:postgresql::testler::` | `web_guvenlik / test:web_testi` | `eszamanlilik`<br>`web_guvenlik`<br>`cli`<br>`proje`<br>`uctan_uca` |
| Concurrency | `test:ag_ve_esz_testi`<br>`test:eszamanlilik_conformance_testi` | `web_guvenlik / test:web_testi`<br>`semantic_regresyon / test:semantic_regresyon_korpusu_testi` | `runtime`<br>`io`<br>`web_guvenlik`<br>`wasm`<br>`semantic_regresyon`<br>`uctan_uca` |
| Web/security | `test:guvenli_testi`<br>`test:kaynak_sinirlari_testi`<br>`test:web_cok_surec_testi`<br>`test:web_testi`<br>`test:yetkinlik_testi`<br>`lib:guvenlik::tests::`<br>`lib:kaynak_sinirlari::testler::`<br>`lib:web_guvenligi::tests::`<br>`lib:yetkinlik::testler::`<br>`bin:dil:web_profili_testleri::` | `semantic_regresyon / test:semantic_regresyon_korpusu_testi` | `http`<br>`io`<br>`eszamanlilik`<br>`cli`<br>`proje`<br>`semantic_regresyon`<br>`uctan_uca` |
| HTTP parser | `test:http_istegi_testi` | `lexer / test:fuzz_korpusu_testi`<br>`web_guvenlik / test:web_testi`<br>`web_guvenlik / test:kaynak_sinirlari_testi` | `web_guvenlik`<br>`cli`<br>`uctan_uca` |
| Package | `test:kitaplik_testi`<br>`test:tedarik_testi`<br>`lib:paket::uzak::politika::testler::`<br>`lib:artefakt_dogrulama::testler::` | `proje / test:proje_testi` | `registry`<br>`tedarik_zinciri`<br>`cli`<br>`proje`<br>`uctan_uca` |
| Registry | `lib:registry::istemci::testler::`<br>`lib:registry::testler::`<br>`bin:dil:dil_registry::testler::` | `proje / test:proje_testi` | `paket`<br>`tedarik_zinciri`<br>`cli`<br>`proje` |
| Supply-chain | `test:tedarik_kapisi_testi` | — | `paket`<br>`registry`<br>`proje`<br>`muhe_kapilari` |
| LSP | `test:lsp_testi`<br>`test:olcum_lsp_process_testi`<br>`lib:lsp::cikti::testler::`<br>`lib:lsp::kaynak_siniri_testleri::`<br>`bin:dillsp:testler::` | `morfoloji / test:morfoloji_testi`<br>`web_guvenlik / test:kaynak_sinirlari_testi` | `cli`<br>`uctan_uca` |
| WASM | `test:playground_testi` | `lexer / test:fuzz_korpusu_testi`<br>`web_guvenlik / test:kaynak_sinirlari_testi` | `uctan_uca` |
| CLI | `test:io_izi_cli_testi` | `proje / test:proje_testi`<br>`proje / test:projeler_testi`<br>`uctan_uca / test:golden_testi` | `proje`<br>`uctan_uca` |
| Project system | `test:proje_testi`<br>`test:projeler_testi` | — | `paket`<br>`registry`<br>`cli`<br>`uctan_uca` |
| Semantic regression | `test:core_freeze_korugu_testi`<br>`test:semantic_regresyon_korpusu_testi`<br>`test:semantic_regresyon_korugu_testi` | — | `uctan_uca` |
| End-to-end | `test:golden_testi` | — | — |
| Engineering gates | `test:bagimlilik_cevrimi_testi`<br>`test:dokuman_tazelik_testi`<br>`test:katalog_testi`<br>`test:katman_mimarisi_testi`<br>`test:mimari_sinir_testi`<br>`test:panic_guvenligi_testi`<br>`test:public_api_testi`<br>`test:tani_kimligi_testi`<br>`bin:islev_egilimi:testler::`<br>`bin:faz_test_matrisi:testler::`<br>`bin:olcum:testler::` | — | — |

## Çalıştırma sözleşmesi

`cargo run --locked --bin faz_test_matrisi -- --denetle --rapor target/faz-test-matrisi.md` önce bütün test hedeflerini JSON Cargo çıktısından
derler. Her gerçek test kimliğinin tam bir faz sahibi olduğunu, ek kapsamın
yalnız bilinen fazlara ve kendi birincil seçicisine bağlandığını ve bu belgenin
güncel kaldığını doğrular; ardından fazları ayrı çalıştırıp test sayısı,
pass/fail/ignored ve duvar süresini dinamik Markdown raporuna yazar. `cfg`
koşullu testler nedeniyle sayı işletim sistemine göre değişebilir; sahiplik
kuralı her platformda yeniden kanıtlanır. Derleme
hatası, sahipsiz/yinelenen test, boş seçici, bayat belge, beklenmeyen test
sayısı veya başarısız test kapıyı kapatır. CI raporu step summary ve indirilebilir
artefakt olarak yayımlar; süre gözlemseldir, performans eşiği değildir.
