# Faza özgü test matrisi

> Bu dosya `cargo run --locked --bin faz_test_matrisi -- --dokuman-yaz` ile
> üretilir. Faz sahipliği ve saldırı yüzeyi ilişkileri elle değiştirilmez;
> gerçek test sayıları her platformun dinamik CI raporunda çıkar.

- Şema: `zee-faz-test-matrisi-1`
- Faz: **21**
- Sayım: Her CI işletim sisteminde derlenen gerçek test envanteri

## Envanter ve saldırı yüzeyi ilişkileri

| Faz | Kalıcı regresyon kaynağı | Fuzz | Conformance | Aşağı akış |
|---|---|---|---|---|
| Lexer | `compiler/tests/fuzz_korpusu_testi.rs`<br>`compiler/fuzz/corpus/lexer_parser` | `compiler/fuzz/fuzz_targets/lexer_parser.rs`<br>`compiler/fuzz/corpus/lexer_parser` | — | `parser`<br>`ast`<br>`cozumleyici`<br>`tur`<br>`hir`<br>`runtime`<br>`lsp`<br>`wasm`<br>`cli`<br>`proje`<br>`uctan_uca` |
| Parser | `compiler/tests/ifade_grameri_testi.rs`<br>`compiler/tests/parser_kurtarma_testi.rs`<br>`golden` | `compiler/fuzz/fuzz_targets/lexer_parser.rs`<br>`compiler/fuzz/corpus/lexer_parser` | — | `ast`<br>`cozumleyici`<br>`tur`<br>`hir`<br>`runtime`<br>`lsp`<br>`wasm`<br>`cli`<br>`proje`<br>`uctan_uca` |
| AST | `compiler/tests/faz_modeli_testi.rs`<br>`compiler/tests/invariant_testi.rs` | — | — | `cozumleyici`<br>`tur`<br>`hir`<br>`lsp`<br>`wasm`<br>`uctan_uca` |
| Resolver | `compiler/tests/birim_testi.rs`<br>`compiler/tests/kapsam_testi.rs`<br>`compiler/tests/semantic_kimlik_testi.rs` | — | — | `tur`<br>`hir`<br>`runtime`<br>`lsp`<br>`proje`<br>`uctan_uca` |
| Type checker | `compiler/tests/acik_imza_testi.rs`<br>`compiler/tests/cagri_cikarimi_testi.rs`<br>`compiler/tests/daraltma_testi.rs`<br>`compiler/tests/sonuc_donusu_testi.rs` | — | — | `hir`<br>`runtime`<br>`lsp`<br>`wasm`<br>`cli`<br>`proje`<br>`uctan_uca` |
| Typed HIR | `compiler/tests/hir_modeli_testi.rs`<br>`compiler/src/hir/testler.rs` | — | — | `runtime`<br>`lsp`<br>`wasm`<br>`uctan_uca` |
| Morphology | `compiler/tests/morfoloji_v1.snapshot`<br>`compiler/tests/fixtures/morfoloji-zee-tr-1.sha256` | `compiler/fuzz/fuzz_targets/morfoloji.rs`<br>`compiler/fuzz/corpus/morfoloji` | `conformance/morfoloji/sema-v1.schema.json`<br>`conformance/morfoloji/zee-tr-1.json` | `lexer`<br>`cozumleyici`<br>`lsp`<br>`cli`<br>`proje`<br>`uctan_uca` |
| Runtime | `compiler/tests/dene_ve_sure_testi.rs`<br>`compiler/tests/ondalik_testi.rs`<br>`compiler/tests/yapilandirilmis_hata_testi.rs` | — | — | `io`<br>`eszamanlilik`<br>`web_guvenlik`<br>`wasm`<br>`cli`<br>`uctan_uca` |
| IO | `compiler/tests/deterministik_io_profili_testi.rs`<br>`compiler/tests/io_izi_testi.rs` | — | — | `eszamanlilik`<br>`web_guvenlik`<br>`cli`<br>`proje`<br>`uctan_uca` |
| Concurrency | `compiler/tests/ag_ve_esz_testi.rs`<br>`conformance/eszamanlilik/zee-esz-1.json` | — | `conformance/eszamanlilik/sema-v1.schema.json`<br>`conformance/eszamanlilik/zee-esz-1.json` | `runtime`<br>`io`<br>`web_guvenlik`<br>`wasm`<br>`uctan_uca` |
| Web/security | `compiler/tests/web_testi.rs`<br>`compiler/tests/web_cok_surec_testi.rs`<br>`compiler/tests/kaynak_sinirlari_testi.rs` | — | — | `http`<br>`io`<br>`eszamanlilik`<br>`cli`<br>`proje`<br>`uctan_uca` |
| HTTP parser | `compiler/tests/http_istegi_testi.rs`<br>`compiler/fuzz/corpus/http_istegi` | `compiler/fuzz/fuzz_targets/http_istegi.rs`<br>`compiler/fuzz/corpus/http_istegi` | — | `web_guvenlik`<br>`cli`<br>`uctan_uca` |
| Package | `compiler/tests/tedarik_testi.rs`<br>`compiler/tests/fixtures/zep-kanonik-v1.hex`<br>`compiler/tests/fixtures/zep-saldiri-korpusu` | — | `docs/zep-conformance.md`<br>`compiler/tests/fixtures/zep-kanonik-v1.hex` | `registry`<br>`tedarik_zinciri`<br>`cli`<br>`proje`<br>`uctan_uca` |
| Registry | `compiler/src/registry/istemci/testler.rs`<br>`compiler/tests/fixtures/zep-saldiri-korpusu` | — | — | `paket`<br>`tedarik_zinciri`<br>`cli`<br>`proje` |
| Supply-chain | `compiler/tests/tedarik_kapisi_testi.rs`<br>`compiler/deny.toml`<br>`compiler/Cargo.lock`<br>`compiler/fuzz/Cargo.lock` | — | — | `paket`<br>`registry`<br>`proje`<br>`muhe_kapilari` |
| LSP | `compiler/tests/lsp_testi.rs` | — | — | `cli`<br>`uctan_uca` |
| WASM | `compiler/tests/playground_testi.rs`<br>`compiler/fuzz/corpus/wasm_abi` | `compiler/fuzz/fuzz_targets/wasm_abi.rs`<br>`compiler/fuzz/corpus/wasm_abi` | — | `uctan_uca` |
| CLI | `compiler/tests/io_izi_cli_testi.rs`<br>`golden/28-cli-araci.dil` | — | — | `proje`<br>`uctan_uca` |
| Project system | `compiler/tests/proje_testi.rs`<br>`projeler` | — | — | `paket`<br>`registry`<br>`cli`<br>`uctan_uca` |
| End-to-end | `golden` | — | — | — |
| Engineering gates | `compiler/tests/fixtures/islev-egilimi-v1.tsv`<br>`compiler/tests/fixtures/tani-kimlikleri-v1.tsv`<br>`docs/kanit-haritasi-v1.tsv` | — | — | — |

## Birincil sahiplik

Bir test tam bir birincil faza aittir; aşağı akış sütunu değişikliğin
yeniden koşulması gereken sonraki yüzeylerini gösterir.

| Faz | Cargo/libtest seçicileri | Kapsam |
|---|---|---|
| Lexer | `test:fuzz_korpusu_testi`<br>`test:kacis_ve_negatif_testi` | Token, Unicode, kacis, sayi ve girinti siniri. |
| Parser | `test:bicimleyici_testi`<br>`test:ifade_grameri_testi`<br>`test:parser_kurtarma_testi` | Ifade onceligi, tam tuketim, kurtarma ve bicim esdegerligi. |
| AST | `test:faz_modeli_testi`<br>`test:invariant_testi`<br>`doc:dil` | Faz tipleri, kaynak araliklari ve semantic alansizlik degismezleri. |
| Resolver | `test:birim_testi`<br>`test:kapsam_testi`<br>`test:semantic_kimlik_testi` | Kapsam, birim ve kararli semantic kimlik baglari. |
| Type checker | `test:acik_imza_testi`<br>`test:cagri_cikarimi_testi`<br>`test:daraltma_testi`<br>`test:ffi_sinir_testi`<br>`test:intrinsic_lowering_testi`<br>`test:koleksiyon_testi`<br>`test:sonuc_donusu_testi` | Imza, cikarim, daraltma, koleksiyon, intrinsic ve sonuc turleri. |
| Typed HIR | `test:hir_modeli_testi`<br>`lib:hir::testler::` | SymbolId, tur ve kesin kaynak araligi tasiyan baglanmis temsil. |
| Morphology | `test:morfoloji_conformance_testi`<br>`test:morfoloji_testi` | zee-tr-1 uretim, cozum, belirsizlik ve Unicode profili. |
| Runtime | `test:dene_ve_sure_testi`<br>`test:metin_testi`<br>`test:ondalik_testi`<br>`test:ozyineleme_testi`<br>`test:siralama_testi`<br>`test:yapilandirilmis_hata_testi`<br>`lib:ondalik::testler::` | Deger, islem, hata, sure, ondalik ve yurutme semantigi. |
| IO | `test:deterministik_io_profili_testi`<br>`test:io_izi_testi`<br>`lib:ag_istemcisi::testler::`<br>`lib:kalici_dosya::tests::` | Deterministik profil, replay, ag yaniti ve atomik kalici dosya siniri. |
| Concurrency | `test:ag_ve_esz_testi`<br>`test:eszamanlilik_conformance_testi` | Scheduler, gorev agaci, iptal ve kararli gozlem sirasi. |
| Web/security | `test:guvenli_testi`<br>`test:kaynak_sinirlari_testi`<br>`test:web_cok_surec_testi`<br>`test:web_testi`<br>`test:yetkinlik_testi`<br>`lib:guvenlik::tests::`<br>`lib:kaynak_sinirlari::testler::`<br>`lib:web_guvenligi::tests::`<br>`lib:yetkinlik::testler::`<br>`bin:dil:web_profili_testleri::` | Yetkinlik, kaynak butcesi, oturum, CSRF, proxy ve transaction guvenligi. |
| HTTP parser | `test:http_istegi_testi` | Request-line, CRLF, header ve govde framing byte siniri. |
| Package | `test:kitaplik_testi`<br>`test:tedarik_testi`<br>`lib:paket::testler::`<br>`lib:paket::uzak::politika::testler::`<br>`lib:tedarik::testler::` | Kutuphane, ZEP arsivi, imza, SBOM ve provenance davranisi. |
| Registry | `lib:registry::istemci::testler::`<br>`lib:registry::testler::`<br>`bin:dil:dil_registry::testler::` | Metadata zinciri, rollback, cache, exact pin ve CLI secenekleri. |
| Supply-chain | `test:tedarik_kapisi_testi` | RustSec, lisans, kilit ve offline vendor politikasinin yurutulebilir kapisi. |
| LSP | `test:lsp_testi`<br>`lib:lsp::cikti::testler::`<br>`lib:lsp::kaynak_siniri_testleri::`<br>`bin:dillsp:testler::` | JSON-RPC, framing, tani, hover, completion, definition ve rename. |
| WASM | `test:playground_testi` | Playground, ABI sahipligi, limitler ve gercek wasm hostu oncesi native regresyonlar. |
| CLI | `test:io_izi_cli_testi` | Komut satiri IO kaydi ve yeniden oynatma siniri. |
| Project system | `test:proje_testi`<br>`test:projeler_testi` | Bildirim, kilit, bagimlilik, yetkinlik ve tam proje ornekleri. |
| End-to-end | `test:golden_testi` | 33 golden program ile kullanici yuzeyinden tam derleme ve yurutme hatti. |
| Engineering gates | `test:dokuman_tazelik_testi`<br>`test:katalog_testi`<br>`test:mimari_sinir_testi`<br>`test:panic_guvenligi_testi`<br>`test:tani_kimligi_testi`<br>`bin:islev_egilimi:testler::`<br>`bin:faz_test_matrisi:testler::` | Belge, tani, mimari, panic, bicim ve olcum kapilarinin kendi regresyonlari. |

## Çalıştırma sözleşmesi

`cargo run --locked --bin faz_test_matrisi -- --denetle --rapor target/faz-test-matrisi.md` önce bütün test hedeflerini JSON Cargo çıktısından
derler. Her gerçek test kimliğinin tam bir faz sahibi olduğunu ve bu belgenin
güncel kaldığını doğrular; ardından fazları ayrı çalıştırıp test sayısı,
pass/fail/ignored ve duvar süresini dinamik Markdown raporuna yazar. `cfg`
koşullu testler nedeniyle sayı işletim sistemine göre değişebilir; sahiplik
kuralı her platformda yeniden kanıtlanır. Derleme
hatası, sahipsiz/yinelenen test, boş seçici, bayat belge, beklenmeyen test
sayısı veya başarısız test kapıyı kapatır. CI raporu step summary ve indirilebilir
artefakt olarak yayımlar; süre gözlemseldir, performans eşiği değildir.
