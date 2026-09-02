//! B-005/B-006/B-008/B-010/B-018/B-019/B-020/B-025/B-048/B-050/B-055/B-056 mimari sınır regresyonları.

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
    satir_butcesini_denetle("src/ayristirici.rs", 1_200);
    satir_butcesini_denetle("src/cozumleyici.rs", 250);
    satir_butcesini_denetle("src/lsp.rs", 1_500);
    satir_butcesini_denetle("src/yorumlayici.rs", 2_000);
}

#[test]
fn handler_modulleri_yeni_domain_icin_sinir_tasir() {
    for (goreli, butce) in [
        ("src/ayristirici/cumle.rs", 500),
        ("src/ayristirici/ifade.rs", 1_150),
        ("src/ayristirici/kaynak.rs", 80),
        ("src/ayristirici/kurtarma.rs", 160),
        ("src/cozumleyici/cumle.rs", 1_000),
        ("src/cozumleyici/ifade.rs", 960),
        ("src/cozumleyici/kaynak.rs", 40),
        ("src/cozumleyici/cagri.rs", 380),
        ("src/cozumleyici/cikarim.rs", 120),
        ("src/cozumleyici/akis.rs", 150),
        ("src/cozumleyici/baglam.rs", 150),
        ("src/cozumleyici/donus.rs", 180),
        ("src/cozumleyici/etki.rs", 650),
        ("src/cozumleyici/yetkinlik.rs", 380),
        ("src/cozumleyici/sembol.rs", 200),
        ("src/cozumleyici/sozlesme.rs", 180),
        ("src/cozumleyici/turler.rs", 300),
        ("src/kimlik.rs", 80),
        ("src/faz.rs", 160),
        ("src/hir.rs", 180),
        ("src/hir/gezinme.rs", 140),
        ("src/hir/kaynak.rs", 100),
        ("src/morfoloji/uyumluluk.rs", 140),
        ("src/yorumlayici/cumle.rs", 620),
        ("src/yorumlayici/ifade.rs", 760),
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
        ("src/tedarik/kurulum.rs", 220),
        ("src/cli/registry.rs", 500),
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
    assert!(taban.starts_with("# zee-islev-egilimi-v1\n# inceleme: K-144/ADR-041"));
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
        "src/tedarik.rs",
        "src/registry.rs",
        "src/registry/istemci.rs",
        "src/tani.rs",
        "src/kalici_dosya.rs",
        "src/kalici_dosya/metadata.rs",
    ] {
        assert!(
            kaynak(goreli).contains("VARSAYILAN_KAYNAK_SINIRLARI"),
            "{goreli} kaynak limitini ortak profilden okumalı"
        );
    }
}

#[test]
fn checker_katmanlari_tek_sorumlulukla_sahiplenilir() {
    for (goreli, kanit) in [
        ("src/cozumleyici/sembol.rs", "pub fn ad_cozumle"),
        ("src/cozumleyici/turler.rs", "pub enum Tur"),
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

    let turler = kaynak("src/cozumleyici/turler.rs");
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
