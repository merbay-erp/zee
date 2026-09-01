//! Gömülü standart kitaplık (RFC-0014, deneysel): birimlerin kendi
//! testleri, gömülü çözüm yolu ve playground erişimi. Kitaplık, golden
//! korpusla aynı hakemliğe tabidir.

use dil::yorumlayici::{calistir_io, ToplayanIo};

fn gomulu_yukleyici() -> impl FnMut(&str) -> Result<String, String> {
    |ad: &str| {
        dil::gomulu_birim(ad)
            .map(str::to_string)
            .ok_or_else(|| "gömülü değil".into())
    }
}

fn gomulu_kostur(kaynak: &str) -> Vec<String> {
    let mut yukleyici = gomulu_yukleyici();
    let program =
        dil::kaynagi_derle_birimlerle(kaynak, &mut yukleyici).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("çalışmalı");
    io.cikti
}

#[test]
fn gomulu_birimlerin_kendi_testleri_gecer() {
    for ad in dil::gomulu_birim_adlari() {
        let kaynak = dil::gomulu_birim(ad).expect("gömülü olmalı");
        let sonuclar = dil::kaynagi_dene(kaynak)
            .unwrap_or_else(|hata| panic!("{} derlenemedi: {:?}", ad, hata));
        assert!(!sonuclar.is_empty(), "{} birimi testsiz olamaz (RFC-0014 §5)", ad);
        for sonuc in &sonuclar {
            assert!(sonuc.hata.is_none(), "{}: {} düştü", ad, sonuc.ad);
        }
    }
}

#[test]
fn matematik_birimi_kullanilir() {
    let cikti = gomulu_kostur(
        "matematik birimini kullan\n\nx 48 ve 36 ile obebini hesapla olsun\nx yaz\ny 2 ve 8 ile üssünü hesapla olsun\ny yaz\n",
    );
    assert_eq!(cikti, vec!["12", "256"]);
}

#[test]
fn liste_araclari_birimi_kullanilir() {
    let cikti = gomulu_kostur(
        "liste_araclari birimini kullan\n\nnotlar 50, 60, 71 listesi olsun\no notlar için ortalamasını hesapla olsun\no yaz\nb notlar için en büyüğünü bul olsun\nb yaz\n",
    );
    assert_eq!(cikti, vec!["60,333333333", "71"]);
}

#[test]
fn playgroundda_gomulu_birim_calisir() {
    let cikti = dil::wasm_api::playgroundda_calistir(
        "matematik birimini kullan\n\nx 90 ve 60 ile obebini hesapla olsun\n\"obeb: \" ile x yaz\n",
        "",
        1,
    );
    assert_eq!(cikti, "obeb: 30");
}

#[test]
fn bilinmeyen_birim_gomulu_listeyle_reddedilir() {
    let mut yukleyici = gomulu_yukleyici();
    let hata = dil::kaynagi_derle_birimlerle("uzay birimini kullan\n", &mut yukleyici)
        .expect_err("A010");
    assert_eq!(hata.kod, "A010");
}

#[test]
fn depodaki_kaynak_gomuluyle_ayni() {
    // include_str depodaki dosyayı gömer; bu test dosya taşınırsa/kopyalanırsa
    // ayrılmayı yakalar.
    for (ad, yol) in [
        ("matematik", "../kitaplik/matematik.dil"),
        ("liste_araclari", "../kitaplik/liste_araclari.dil"),
    ] {
        let diskteki = std::fs::read_to_string(yol).expect("kitaplik/ okunmalı");
        assert_eq!(diskteki, dil::gomulu_birim(ad).unwrap(), "{} ayrık", ad);
    }
}

#[test]
fn birim_ozeti_islemleri_listeler() {
    let ozet = dil::birim_ozeti(dil::gomulu_birim("matematik").unwrap());
    assert!(ozet.contains("işlem obebini hesapla  (birinciyi al, ikinciyi al)"), "{}", ozet);
    assert!(ozet.contains("işlem üssünü hesapla  (tabanı al, üssü al)"), "{}", ozet);
    assert!(ozet.contains("testler: 5"), "{}", ozet);
}

#[test]
fn matematik_ussu_dogal_adla_cagrilir() {
    // K-049 ikizleşme geri çevrimi: parametre "üssü al" → üs.
    let cikti = gomulu_kostur("matematik birimini kullan\n\nx 3 ve 4 ile üssünü hesapla olsun\nx yaz\n");
    assert_eq!(cikti, vec!["81"]);
}
