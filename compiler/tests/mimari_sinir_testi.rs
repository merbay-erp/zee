//! B-005/B-006 derleyici faz ve checker katmanı mimarisi regresyonları.

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
    satir_butcesini_denetle("src/yorumlayici.rs", 2_000);
}

#[test]
fn handler_modulleri_yeni_domain_icin_sinir_tasir() {
    for (goreli, butce) in [
        ("src/ayristirici/cumle.rs", 500),
        ("src/ayristirici/ifade.rs", 1_150),
        ("src/cozumleyici/cumle.rs", 1_000),
        ("src/cozumleyici/ifade.rs", 920),
        ("src/cozumleyici/cagri.rs", 340),
        ("src/cozumleyici/akis.rs", 150),
        ("src/cozumleyici/baglam.rs", 80),
        ("src/cozumleyici/donus.rs", 180),
        ("src/cozumleyici/etki.rs", 650),
        ("src/cozumleyici/sembol.rs", 150),
        ("src/cozumleyici/sozlesme.rs", 180),
        ("src/cozumleyici/turler.rs", 300),
        ("src/yorumlayici/cumle.rs", 600),
        ("src/yorumlayici/ifade.rs", 730),
    ] {
        satir_butcesini_denetle(goreli, butce);
    }
}

#[test]
fn checker_katmanlari_tek_sorumlulukla_sahiplenilir() {
    for (goreli, kanit) in [
        ("src/cozumleyici/sembol.rs", "pub fn ad_cozumle"),
        ("src/cozumleyici/turler.rs", "pub enum Tur"),
        ("src/cozumleyici/akis.rs", "fn daraltma_cikar"),
        ("src/cozumleyici/cagri.rs", "fn cagri_denetle"),
        ("src/cozumleyici/sozlesme.rs", "fn acik_parametre_turleri"),
        ("src/cozumleyici/etki.rs", "fn denetle"),
        ("src/cozumleyici/donus.rs", "fn donusleri_birlestir"),
    ] {
        assert!(kaynak(goreli).contains(kanit), "{kanit} sahibi {goreli} olmalı");
    }
}

#[test]
fn checker_public_api_katmanlasmada_korunur() {
    use dil::cozumleyici::{ad_cozumle, Tur};
    use std::collections::HashMap;

    let ortam = HashMap::from([("sayı".to_string(), Tur::TamSayi)]);
    assert_eq!(ad_cozumle("sayıyı", &ortam, 1, 1, 6).expect("çözülmeli"), "sayı");
    assert_eq!(Tur::TamSayi.adi(), "TamSayı");
}
