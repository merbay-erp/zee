//! B-005/B-006/B-008/B-010/B-018/B-019/B-020/B-025/B-038/B-040/B-048/B-050/B-055/B-056 mimari sınır regresyonları.

use std::fs;
use std::path::{Path, PathBuf};

fn kaynak_yolu(goreli: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(goreli)
}

fn kaynak(goreli: &str) -> String {
    fs::read_to_string(kaynak_yolu(goreli)).expect("derleyici kaynağı okunmalı")
}

fn satir_butcesini_denetle(goreli: &str, butce: usize) {
    let sayi = kaynak(goreli).lines().count();
    assert!(
        sayi <= butce,
        "{goreli} {sayi} satıra çıktı; {butce} satırlık faz bütçisini aşıyor. Yeni alanı ayrı bir handler/modüle taşı."
    );
}

#[test]
fn kok_dosyalar_mega_handlerlari_yeniden_yutmaz() {
    let parser = kaynak("src/ayristirici.rs");
    assert!(!parser.contains("fn cumle_ayristir"));
    assert!(!parser.contains("fn yapili_kalip"));

    let checker = kaynak("src/cozumleyici.rs");
    assert!(!checker.contains("fn blok_denetle"));
    assert!(!checker.contains("fn ifade_denetle"));
    assert!(!checker.contains("fn cagri_denetle"));
    assert!(!checker.contains("fn ad_cozumle"));
    assert!(!checker.contains("fn daraltma_cikar"));
    assert!(!checker.contains("fn donusleri_birlestir"));
    assert!(!checker.contains("fn acik_parametre_turleri"));
    assert!(!checker.contains("fn alan_turu"));

    let runtime = kaynak("src/yorumlayici.rs");
    assert!(!runtime.contains("fn blok_calistir_async"));
    assert!(!runtime.contains("fn degerlendir_async"));
}

#[test]
fn kok_faz_dosyalari_yeniden_sismez() {
    satir_butcesini_denetle("src/ayristirici.rs", 1_300);
    satir_butcesini_denetle("src/cozumleyici.rs", 250);
    satir_butcesini_denetle("src/lsp.rs", 1_500);
    satir_butcesini_denetle("src/yorumlayici.rs", 2_000);
}

#[test]
fn handler_modulleri_yeni_domain_icin_sinir_tasir() {
    for (goreli, butce) in [
        ("src/ayristirici/cumle.rs", 540),
        ("src/ayristirici/cumle/web.rs", 80),
        ("src/ayristirici/ifade.rs", 1_220),
        ("src/ayristirici/kaynak.rs", 80),
        ("src/ayristirici/kurtarma.rs", 160),
        ("src/cozumleyici/cumle.rs", 1_200),
        ("src/cozumleyici/ifade.rs", 1_080),
        ("src/cozumleyici/kaynak.rs", 40),
        ("src/cozumleyici/cagri.rs", 380),
        ("src/cozumleyici/cikarim.rs", 120),
        ("src/cozumleyici/akis.rs", 150),
        ("src/cozumleyici/baglam.rs", 180),
        ("src/cozumleyici/donus.rs", 180),
        ("src/cozumleyici/etki.rs", 650),
        ("src/cozumleyici/yetkinlik.rs", 380),
        ("src/cozumleyici/sembol.rs", 200),
        ("src/cozumleyici/sozlesme.rs", 180),
        ("src/cozumleyici/turler.rs", 160),
        ("src/guvenlik.rs", 220),
        ("src/kimlik.rs", 80),
        ("src/semantic_model.rs", 180),
        ("src/tani_politikasi.rs", 20),
        ("src/faz.rs", 160),
        ("src/hir.rs", 180),
        ("src/hir/gezinme.rs", 140),
        ("src/hir/kaynak.rs", 100),
        ("src/morfoloji/uyumluluk.rs", 140),
        ("src/yorumlayici/cumle.rs", 860),
        ("src/yorumlayici/ifade.rs", 900),
        ("src/yorumlayici/hir_gecisi.rs", 140),
        ("src/yorumlayici/io_izi.rs", 1_250),
        ("src/yorumlayici/io_profili.rs", 80),
        ("src/yorumlayici/kaynak.rs", 280),
        ("src/yorumlayici/metin.rs", 240),
        ("src/yorumlayici/web_istek.rs", 180),
        ("src/yorumlayici/yetkinlik.rs", 250),
        ("src/yetkinlik.rs", 520),
        ("src/yetkinlik/origin.rs", 100),
        ("src/http_istegi.rs", 260),
        ("src/wasm_api/abi.rs", 260),
        ("src/ag_istemcisi.rs", 180),
        ("src/web_guvenligi.rs", 1_050),
        ("src/web_guvenligi/depo.rs", 160),
        ("src/web_guvenligi/depo/kalici.rs", 180),
        ("src/kaynak_sinirlari/baglanti.rs", 60),
        ("src/kaynak_sinirlari/okuma.rs", 120),
        ("src/kaynak_sinirlari/profiller.rs", 300),
        ("src/kaynak_sinirlari/web.rs", 80),
        ("src/lsp/cikti.rs", 120),
        ("src/lsp/json.rs", 300),
        ("src/kalici_dosya.rs", 850),
        ("src/kalici_dosya/metadata.rs", 260),
        ("src/kaynak_sinirlari.rs", 280),
        ("src/registry/istemci.rs", 600),
        ("src/registry/istemci/depo.rs", 100),
        ("src/registry/istemci/tasima.rs", 120),
        ("src/paket/uzak.rs", 480),
        ("src/paket/uzak/politika.rs", 180),
        ("src/paket/uzak_wasm.rs", 140),
        ("src/artefakt_dogrulama/kurulum.rs", 220),
        ("src/cli/registry.rs", 500),
        ("src/zaman.rs", 80),
    ] {
        satir_butcesini_denetle(goreli, butce);
    }
}

#[test]
fn kritik_islevler_kor_bir_hard_limit_yerine_incelenmis_egilim_tasir() {
    let arac = kaynak("src/bin/islev_egilimi.rs");
    let ci = kaynak("../.github/workflows/ci.yml");
    let taban = kaynak("tests/fixtures/islev-egilimi-v1.tsv");
    let rapor = kaynak("../docs/islev-egilimi.md");

    assert!(arac.contains("clippy::too_many_lines"));
    assert!(arac.contains("clippy::cognitive_complexity"));
    assert!(arac.contains("--force-warn"));
    assert!(arac.contains("satir_artis_pay"));
    assert!(arac.contains("karmasiklik_artis_pay"));
    assert!(arac.contains("işlev eğilim raporu bayat"));
    assert!(ci.contains("cargo run --locked --bin islev_egilimi -- --denetle"));
    assert!(ci.contains("cargo fmt --all -- --check"));
    assert!(taban.starts_with("# zee-islev-egilimi-v1\n# inceleme: "));
    assert!(
        taban
            .lines()
            .filter(|satir| !satir.starts_with('#'))
            .count()
            >= 40,
        "ilk incelenmiş taban kritik üretim işlevlerini kapsamalı"
    );
    assert!(rapor.contains("Mutlak bir\n\"iyi işlev N satırdır\" kuralı koymaz"));
    assert!(rapor.contains("Düşüşler\ntabanı kendiliğinden aşağı çekmez"));
}

#[test]
fn lsp_json_ayristirma_ve_cikti_sahipleri_ayridir() {
    let kok = kaynak("src/lsp.rs");
    let json = kaynak("src/lsp/json.rs");
    let cikti = kaynak("src/lsp/cikti.rs");
    assert!(!kok.contains("fn sayi("));
    assert!(json.contains("struct JsonSayisi"));
    assert!(json.contains("fn sayi("));
    assert!(cikti.contains("struct SinirliJson"));
}

#[test]
fn web_proxy_origin_tek_ag_hedefi_parserini_kullanir() {
    let cli = kaynak("src/main.rs");
    assert!(!cli.contains("struct GuvenliOrigin"));
    assert!(cli.contains("AgHedefi::https_origininden"));
    assert!(cli.contains("AgHedefi::https_otoritesinden"));
    assert!(cli.contains("TcpListener::bind((WEB_BIND_IP"));
    assert!(cli.contains("guvenilir_proxy_esi_mi"));
}

#[test]
fn http_istek_framingi_byte_parserinda_tek_sahiplidir() {
    let cli = kaynak("src/main.rs");
    let parser = kaynak("src/http_istegi.rs");
    let fuzz = kaynak("fuzz/fuzz_targets/http_istegi.rs");
    assert!(cli.contains("HttpIstekBasligi::ayristir"));
    assert!(cli.contains("baslik_sonunu_bul"));
    assert!(!cli.contains("String::from_utf8_lossy(&tampon"));
    assert!(parser.contains("HTTP satırları yalnız CRLF ile bitmeli"));
    assert!(parser.contains("birden çok Content-Length başlığı reddedildi"));
    assert!(fuzz.contains("HttpIstegi::ayristir"));
}

#[test]
fn wasm_abi_kayitli_tampon_sahipligini_atlayamaz() {
    let kok = kaynak("src/wasm_api.rs");
    let abi = kaynak("src/wasm_api/abi.rs");
    let fuzz = kaynak("fuzz/fuzz_targets/wasm_abi.rs");
    let gece = kaynak("../.github/workflows/fuzz.yml");
    let ci = kaynak("../.github/workflows/ci.yml");
    let host = kaynak("../playground/sablon.html");
    let node_hostu = kaynak("../scripts/wasm-abi-denetle.mjs");
    let mut korpus: Vec<_> = fs::read_dir(kaynak_yolu("fuzz/corpus/wasm_abi"))
        .expect("WASM ABI fuzz korpusu okunmalı")
        .map(|girdi| {
            girdi
                .expect("korpus girdisi okunmalı")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    korpus.sort();
    assert!(!kok.contains("from_raw_parts"));
    assert!(!abi.contains("Vec::from_raw_parts"));
    assert!(!abi.contains("unsafe extern"));
    assert!(abi.contains("TamponKayitlari"));
    assert!(abi.contains("pointer ve uzunluk ayrılmış tamponla birebir eşleşmiyor"));
    assert!(abi.contains("dil_sonuc_tamponu_uzunlugu"));
    assert!(fuzz.contains("usize::MAX"));
    assert!(fuzz.contains("dil_bellek_birak(sonuc, toplam)"));
    assert!(gece.contains("hedef: wasm_abi"));
    assert!(gece.contains("azami_girdi: 4097"));
    assert!(ci.contains("node ../scripts/wasm-abi-denetle.mjs"));
    assert!(host.contains("wasm.dil_abi_surumu() !== 3"));
    assert!(host.contains("wasm.dil_sonuc_tamponu_uzunlugu(sonucPtr)"));
    assert!(host.contains("new TextDecoder(\"utf-8\", { fatal: true })"));
    assert!(host.contains("} finally {"));
    assert!(node_hostu.contains("WebAssembly.instantiate"));
    assert!(node_hostu.contains("wasm.dil_bellek_ayir(-1)"));
    assert!(node_hostu.contains("wasm.dil_bellek_birak(ptr, toplam)"));
    assert_eq!(
        korpus,
        ["bos", "gecerli.dil", "soru", "unicode.dil", "uzunluk"]
    );
}

#[test]
fn playground_girdisi_koleksiyondan_once_tek_profilden_sinirlanir() {
    let profil = kaynak("src/kaynak_sinirlari/profiller.rs");
    let sinirlar = kaynak("src/kaynak_sinirlari.rs");
    let playground = kaynak("src/wasm_api.rs");
    let abi = kaynak("src/wasm_api/abi.rs");
    let host = kaynak("../playground/sablon.html");
    let node_hostu = kaynak("../scripts/wasm-abi-denetle.mjs");
    assert!(profil.contains("struct PlaygroundSinirlari"));
    assert!(sinirlar.contains("girdi_bayti: 1024 * 1024"));
    assert!(sinirlar.contains("girdi_satiri: 4_096"));
    assert!(
        playground.find("playground_girdilerini_denetle").unwrap()
            < playground.find("girdiler.lines().map").unwrap()
    );
    assert!(playground.contains("girdiler.lines().count()"));
    assert!(abi.contains("uzunluk > azami_uzunluk"));
    assert!(abi.contains("dil_playground_kaynak_bayti"));
    assert!(abi.contains("dil_playground_girdi_bayti"));
    assert!(abi.contains("dil_playground_girdi_satiri"));
    assert!(host.contains("function utf8BaytUzunlugu"));
    assert!(host.contains("new TextEncoder().encodeInto"));
    assert!(!host.contains("new TextEncoder().encode(metin)"));
    assert!(node_hostu.contains("kaynakSiniri + 1"));
    assert!(node_hostu.contains("girdiSiniri + 1"));
    assert!(node_hostu.contains("girdiSatiriSiniri + 1"));
}

#[test]
fn domain_kaynak_limitleri_tek_profilden_beslenir() {
    for goreli in [
        "src/ag_istemcisi.rs",
        "src/main.rs",
        "src/web_guvenligi.rs",
        "src/yorumlayici.rs",
        "src/yorumlayici/io_izi.rs",
        "src/wasm_api.rs",
        "src/wasm_api/abi.rs",
        "src/lsp.rs",
        "src/artefakt_dogrulama.rs",
        "src/registry.rs",
        "src/registry/istemci.rs",
        "src/kalici_dosya.rs",
        "src/kalici_dosya/metadata.rs",
    ] {
        assert!(
            kaynak(goreli).contains("VARSAYILAN_KAYNAK_SINIRLARI"),
            "{goreli} kaynak limitini ortak profilden okumalı"
        );
    }
    let tani = kaynak("src/tani.rs");
    let profil = kaynak("src/kaynak_sinirlari.rs");
    assert!(tani.contains("tani_politikasi::AZAMI_TANI_SAYISI"));
    assert!(profil.contains("tani_politikasi::AZAMI_TANI_SAYISI"));
}

#[test]
fn checker_katmanlari_tek_sorumlulukla_sahiplenilir() {
    for (goreli, kanit) in [
        ("src/cozumleyici/sembol.rs", "pub fn ad_cozumle"),
        ("src/semantic_model.rs", "pub enum Tur"),
        ("src/cozumleyici/akis.rs", "fn daraltma_cikar"),
        ("src/cozumleyici/cagri.rs", "fn cagri_denetle"),
        ("src/cozumleyici/cikarim.rs", "fn cikarim_onbilgisi"),
        ("src/cozumleyici/sozlesme.rs", "fn acik_parametre_turleri"),
        ("src/cozumleyici/etki.rs", "fn denetle"),
        ("src/cozumleyici/donus.rs", "fn donusleri_birlestir"),
    ] {
        assert!(
            kaynak(goreli).contains(kanit),
            "{kanit} sahibi {goreli} olmalı"
        );
    }
}

#[test]
fn checker_public_api_katmanlasmada_korunur() {
    use dil::cozumleyici::{ad_cozumle, Tur};
    use std::collections::HashMap;

    let ortam = HashMap::from([("sayı".to_string(), Tur::TamSayi)]);
    assert_eq!(
        ad_cozumle("sayıyı", &ortam, 1, 1, 6).expect("çözülmeli"),
        "sayı"
    );
    assert_eq!(Tur::TamSayi.adi(), "TamSayı");
}

#[test]
fn semantic_kimlikler_depolama_indeksine_geri_donmez() {
    let kimlikler = kaynak("src/kimlik.rs");
    for kanit in [
        "sirali_kimlik!(YapiId)",
        "sirali_kimlik!(IslemId)",
        "pub struct SymbolId",
    ] {
        assert!(
            kimlikler.contains(kanit),
            "semantic kimlik newtype olmalı: {kanit}"
        );
    }

    let turler = kaynak("src/semantic_model.rs");
    assert!(!turler.contains("Yapi(usize)"));

    for goreli in ["src/cozumleyici/cumle.rs", "src/cozumleyici/ifade.rs"] {
        assert!(
            !kaynak(goreli).contains("yapilar["),
            "{goreli} YapiId'yi vektör indeksi gibi kullanmamalı"
        );
    }
}

#[test]
fn standart_hat_faz_turlerini_atlayamaz() {
    let faz = kaynak("src/faz.rs");
    for tur in [
        "struct KaynakMetni",
        "struct TokenAkisi",
        "struct AyristirilmisAst",
        "struct BaglanmamisProgram",
        "struct BaglanmisProgram",
    ] {
        assert!(
            faz.contains(tur),
            "derleme fazı kodda görünür olmalı: {tur}"
        );
    }

    let kok = kaynak("src/lib.rs");
    assert!(kok.contains("KaynakMetni::yeni(kaynak).sozcukle()"));
    assert!(kok.contains("BaglanmamisProgram::yeni"));
    assert!(kok.contains("calistir_baglanmis_io(&program"));
}

#[test]
fn baglanmis_program_typed_hir_olmadan_uretilemez() {
    let faz = kaynak("src/faz.rs");
    assert!(faz.contains("hir: HirProgram"));
    assert!(faz.contains("denetle_ve_hir_bilgisi"));

    let hir = kaynak("src/hir.rs");
    for kanit in [
        "struct HirProgram",
        "struct HirDugumId",
        "struct HirIfadeBilgisi",
        "enum HirBagi",
    ] {
        assert!(hir.contains(kanit), "typed HIR kanıtı eksik: {kanit}");
    }
    assert!(
        kaynak("src/hir/kaynak.rs").contains("enum HirKaynakAraligi"),
        "typed HIR kaynak aralığı ayrı modülde görünür olmalı"
    );
    let gezinme = kaynak("src/hir/gezinme.rs");
    for kanit in ["struct HirSembolKullanimi", "fn sembol_kullanimlari"] {
        assert!(
            gezinme.contains(kanit),
            "semantic HIR gezinme kanıtı eksik: {kanit}"
        );
    }
}

#[test]
fn standart_runtime_semantic_kararlari_hir_bagindan_alir() {
    let runtime = kaynak("src/yorumlayici/hir_gecisi.rs");
    assert!(runtime.contains("CalistirmaProgrami::Hir(program.hir())"));
    assert!(runtime.contains("HirBagi::Sembol(kimlik)"));
    assert!(runtime.contains("HirBagi::Islem(kimlik)"));
    assert!(runtime.contains("HirBagi::Yapi(kimlik)"));

    let kok = kaynak("src/lib.rs");
    assert!(kok.contains("programi_dene_baglanmis(&program)"));
}

#[test]
fn faz_test_matrisi_tier1_ci_raporundan_kopamaz() {
    let ci = kaynak("../.github/workflows/ci.yml");
    for kanit in [
        "os: [ubuntu-latest, macos-latest, windows-latest]",
        "cargo run --locked --bin faz_test_matrisi -- --denetle --rapor target/faz-test-matrisi.md",
        "cat target/faz-test-matrisi.md >> \"$GITHUB_STEP_SUMMARY\"",
        "uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02", // v4.6.2
        "name: faz-test-matrisi-${{ matrix.os }}",
    ] {
        assert!(ci.contains(kanit), "faz matrisi CI kanıtı eksik: {kanit}");
    }
}

#[test]
fn semantic_regresyon_korpusu_gecmisten_sessizce_silinemez() {
    let ci = kaynak("../.github/workflows/ci.yml");
    assert!(
        ci.contains("bash scripts/semantic-regresyon-korugu.sh \"$ZEE_REGRESYON_TABANI\""),
        "semantic regresyon soy ağacı CI'dan kopmamalı"
    );
    let koruk = kaynak("../scripts/semantic-regresyon-korugu.sh");
    for kanit in [
        "KALICI SEMANTIC REGRESYON VAKASI SİLİNDİ",
        "SEMANTIC REGRESYON KİMLİĞİ YENİDEN KULLANILDI",
        "SEMANTIC REGRESYON PROVENANCE'I YENİDEN YAZILDI",
        "COMPILER KAYNAK COMMIT'İ TEKİL SEMANTIC BEYAN TAŞIMIYOR",
        "SEMANTIC BUGFIX EXACT FIXTURE PROVENANCE'I TAŞIMIYOR",
        "SEMANTIC DEĞİŞİKLİK NORMATİF KANIT TAŞIMIYOR",
        "SAHİPSİZ COMPILER DEĞİŞİKLİK BEYANI",
        "docs/compiler-degisiklik-beyanlari-v1.tsv",
        "git show \"${taban}:${taban_manifest}\"",
        "git rev-list --reverse \"${enforcement_parent}..HEAD\" -- compiler/src",
    ] {
        assert!(
            koruk.contains(kanit),
            "regresyon koruğu kanıtı eksik: {kanit}"
        );
    }
}

#[test]
fn core_freeze_gercek_dogfood_kaniti_olmadan_acilamaz() {
    let ci = kaynak("../.github/workflows/ci.yml");
    assert!(
        ci.contains("bash scripts/core-freeze-korugu.sh \"$ZEE_CORE_FREEZE_TABANI\""),
        "core freeze koruğu CI'dan kopmamalı"
    );
    let koruk = kaynak("../scripts/core-freeze-korugu.sh");
    for kanit in [
        "docs/core-freeze-beyanlari-v1.tsv",
        "CORE FREEZE COMPILER COMMIT'İ TEKİL BEYAN TAŞIMIYOR",
        "CORE FREEZE MAINTENANCE BEYANI UYUŞMUYOR",
        "CORE FREEZE DOGFOOD BEYANI UYUŞMUYOR",
        "CORE FREEZE $alan KANITI GEÇERSİZ",
        "dosya_iste \"$commit\" \"REPRODUCER\"",
        "dosya_iste \"$commit\" \"ETKİLENEN PROJE\"",
        "CORE FREEZE ADR/SPEC KANITI GEÇERSİZ",
        "CORE FREEZE BEYANI YENİDEN YAZILDI",
        "git rev-list --reverse \"${enforcement_parent}..HEAD\" -- compiler/src",
    ] {
        assert!(koruk.contains(kanit), "core freeze kanıtı eksik: {kanit}");
    }
}

#[test]
fn performans_gozlemi_shared_ci_esigine_donusmez() {
    let ci = kaynak("../.github/workflows/ci.yml");
    let arac = kaynak("src/bin/olcum.rs");
    let gecmis = kaynak("../docs/performans-gecmisi-v2.tsv");

    for kanit in [
        "--tur 25",
        "--json target/performans.json",
        "--rapor target/performans.md",
        "--gecmis-cikti target/performans-gecmisi.tsv",
        "--git-sha \"$GITHUB_SHA\"",
        "--milestone shared-ci",
        "cargo build --locked --release --bin dillsp",
        "--dillsp target/release/dillsp",
        "--lsp-olcek",
        "cat target/performans.md >> \"$GITHUB_STEP_SUMMARY\"",
        "name: performans-${{ github.sha }}",
    ] {
        assert!(ci.contains(kanit), "performans CI kanıtı eksik: {kanit}");
    }
    assert!(
        !ci.lines()
            .any(|satir| satir.trim_start().starts_with("--esik-yuzde")),
        "shared CI gürültülü performans hard gate'i taşımamalı"
    );
    for kimlik in [
        "parse_gecikmesi",
        "typecheck_gecikmesi",
        "hir_olusturma",
        "runtime_baslangici",
        "yurutme_gecikmesi",
        "lsp_engine_initialize",
        "lsp_process_cold_start",
        "lsp_ac",
        "lsp_degistir_2k",
        "lsp_degistir_5k",
        "lsp_degistir_10k",
        "lsp_degistir_20k",
        "tepe_bellek",
    ] {
        assert!(arac.contains(kimlik), "ölçüm yüzeyi eksik: {kimlik}");
    }
    assert!(arac.contains("p50"));
    assert!(arac.contains("p95"));
    assert!(arac.contains("esik_yuzde"));
    for provenance in [
        "git_dirty",
        "ram_bytes",
        "build_profile",
        "sample_count",
        "warmup_count",
        "sampling_semantics",
        "tarihçe kirli çalışma ağacından üretilemez",
    ] {
        assert!(
            arac.contains(provenance) || gecmis.contains(provenance),
            "benchmark provenance kanıtı eksik: {provenance}"
        );
    }
    assert!(gecmis.starts_with("# zee-performans-gecmisi-2\n"));
    assert!(gecmis.contains("df737f643c4ee9c8525ce7e972660230e75f5f45\tK-148\tfalse"));
    assert!(gecmis.contains("a2693d6ec98d52b8e2e882b43ecaae682fcce48a\tK-153\tfalse"));
}

#[test]
fn spec_maddeleri_exact_test_kanitindan_kopamaz() {
    let arac = kaynak("src/bin/spec_drift.rs");
    let ci = kaynak("../.github/workflows/ci.yml");
    let harita = kaynak("../docs/spec-madde-kaniti-v1.tsv");
    let rapor = kaynak("../docs/spec-drift-raporu.md");
    for kok in [
        "\"ZORUNLU\"",
        "\"ZORUNDA\"",
        "\"YASAK\"",
        "\"TANIMLI\"",
        "\"AÇIK\"",
    ] {
        assert!(arac.contains(kok), "normatif işaretçi kökü eksik: {kok}");
    }
    for kural in [
        "spec drift raporu bayat",
        "kayıtsız madde",
        "bayat kayıt",
        "özet bayat",
        "AÇIK madde yalnız acik",
        "yürütülebilir test taşımıyor",
    ] {
        assert!(arac.contains(kural), "drift kuralı eksik: {kural}");
    }
    assert!(ci.contains("cargo run --locked --bin spec_drift -- --denetle"));
    assert!(harita.starts_with("# zee-spec-madde-kaniti-1\n"));
    assert!(rapor.contains("cargo run --locked --bin spec_drift -- --rapor-yaz"));
    let kayit = harita
        .lines()
        .filter(|satir| !satir.is_empty() && !satir.starts_with('#'))
        .count();
    assert!(
        kayit >= 150,
        "madde kanıt haritası beklenmedik biçimde küçüldü: {kayit}"
    );
}

#[test]
fn uzun_soak_kapisi_rss_egilimini_ve_provenance_tarihcesini_tasir() {
    let arac = kaynak("src/bin/soak.rs");
    let workflow = kaynak("../.github/workflows/soak.yml");
    let gecmis = kaynak("../docs/soak-gecmisi-v1.tsv");
    for parca in [
        "AZAMI_BUYUME_YUZDE",
        "AZAMI_BUYUME_KIB",
        "ISINMA_PENCERESI",
        "publishDiagnostics",
        "kaynagi_derle_birimlerle",
        "soak tarihçesi kirli çalışma ağacından yazılamaz",
        "zee-soak-gecmisi-1",
    ] {
        assert!(
            arac.contains(parca),
            "soak kapısı sözleşmesi eksik: {parca}"
        );
    }
    assert!(workflow.contains("--sure-sn 1800"));
    assert!(workflow.contains("schedule:"));
    assert!(workflow.contains("cargo build --locked --release --bin dillsp --bin soak"));
    assert!(gecmis.starts_with("# zee-soak-gecmisi-1\n# git_sha\t"));
    for satir in gecmis
        .lines()
        .filter(|s| !s.starts_with('#') && !s.is_empty())
    {
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        assert_eq!(
            alanlar.len(),
            17,
            "soak tarihçe satırı 17 alan taşımalı: {satir}"
        );
        assert_eq!(alanlar[0].len(), 40, "soak tarihçesi tam Git SHA ister");
        assert!(matches!(alanlar[16], "gecti" | "kaldi"));
    }
}
