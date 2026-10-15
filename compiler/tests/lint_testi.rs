//! K-287: lint uyarıları semantic SymbolId ve gerçek kontrol akışından doğar.
use dil::tani::Tani;
fn uyar(kaynak: &str) -> Vec<Tani> {
    let _koken = dil::agac::kaynak_kokenlerini_izle(true);
    dil::kaynagi_fazli_derle(kaynak)
        .unwrap_or_else(|e| panic!("{}", e.raporla(kaynak)))
        .hir()
        .uyarilari(None)
}

#[test]
fn yeniden_atama_okuma_sayilmaz_gercek_okuma_uyariyi_kaldirir() {
    let u = uyar("kalan 1 olsun\nkalan 2 olsun\nokunan 3 olsun\nokunanı yaz\n");
    assert_eq!(u.len(), 1);
    assert_eq!(u[0].kod, "A012");
    assert_eq!((u[0].satir, u[0].sutun, u[0].uzunluk), (1, 1, 5));
    assert!(u[0].mesaj.contains("kalan"));
    assert!(uyar("kalan 1 olsun\nkalan 2 olsun\nkalanı yaz\n").is_empty());
}

#[test]
fn parametre_ve_ayri_kapsam_adlari_karistirilmaz() {
    let u = uyar("işlem kimlik ver\n    kimlik TamSayı olarak al\n    TamSayı döndürür\n    kimliği döndür\n\nkimlik 9 olsun\n3 için kimlik ver yaz\n");
    assert_eq!(u.iter().filter(|t| t.kod == "A012").count(), 1);
    assert_eq!(u[0].satir, 6);
}

#[test]
fn golgeleme_olmayan_dis_ada_atama_sahte_uyari_uretmez() {
    assert!(uyar("sayı 1 olsun\ndoğru ise\n    sayı 2 olsun\nsayıyı yaz\n").is_empty());
}

#[test]
fn kesin_donus_sonrasi_ve_sabit_yanlis_dal_erisilemezdir() {
    let u = uyar("işlem sayı ver\n    TamSayı döndürür\n    1 döndür\n    \"ulaşılmaz\" yaz\n\nyanlış ise\n    \"ölü dal\" yaz\nsayı ver yaz\n");
    let satirlar: Vec<_> = u
        .iter()
        .filter(|t| t.kod == "S047")
        .map(|t| t.satir)
        .collect();
    assert_eq!(satirlar, [4, 7]);
}

#[test]
fn ithal_ayni_satirli_tanim_ana_kaynak_uyarisi_olamaz() {
    let _koken = dil::agac::kaynak_kokenlerini_izle(true);
    let mut yukle = |_: dil::BirimIstegi<'_>| {
        Ok(dil::YuklenenBirim {
            kaynak: "işlem sayi ver\n    TamSayı döndürür\n    gereksiz 9 olsun\n    4 döndür\n"
                .into(),
            koken: "yardimci.dil".into(),
        })
    };
    let p = dil::kaynagi_fazli_derle_kokenlerle(
        "yardimci birimini kullan\nsayi ver yaz\n",
        Some("ana.dil"),
        &mut yukle,
    )
    .unwrap();
    assert!(p.hir().uyarilari(Some("ana.dil")).is_empty());
}

#[test]
fn rota_dal_yaniti_kontrol_edilir_yanit_sonlandirma_sayilmaz() {
    let kismi =
        "GET \"/\" adresine istek geldiğinde\n    doğru ise\n        \"evet\" yanıtını gönder\n";
    // Muhafazakâr yapısal kontrol bütün dalları ister; sabit koşul kanıtını
    // yanıt denetimine taşıdığı iddia edilmez.
    assert!(uyar(kismi).iter().any(|t| t.kod == "S048"));
    let tam =
        "GET \"/\" adresine istek geldiğinde\n    \"evet\" yanıtını gönder\n    \"çalışır\" yaz\n";
    assert!(!uyar(tam)
        .iter()
        .any(|t| matches!(t.kod.as_str(), "S047" | "S048")));
}

#[test]
fn ifade_icindeki_kullanici_cagrisi_icin_yanit_etkisi_tahmin_edilmez() {
    let kaynak = "işlem bir ver\n    TamSayı döndürür\n    1 döndür\n\nGET \"/\" adresine istek geldiğinde\n    sayı bir ver olsun\n    sayıyı yaz\n";
    assert!(!uyar(kaynak).iter().any(|t| t.kod == "S048"));
}

#[test]
fn cli_lint_istege_bagli_uyari_ile_basari_kodu_verir() {
    let d = std::env::temp_dir().join(format!(
        "zee-lint-{}-{}.dil",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&d, "kullanilmayan 7 olsun\n").unwrap();
    let normal = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["denetle", "--json"])
        .arg(&d)
        .output()
        .unwrap();
    let lint = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["denetle", "--json", "--lint"])
        .arg(&d)
        .output()
        .unwrap();
    std::fs::remove_file(d).unwrap();
    assert!(normal.status.success());
    assert!(
        lint.status.success(),
        "{}",
        String::from_utf8_lossy(&lint.stderr)
    );
    assert!(!String::from_utf8_lossy(&normal.stdout).contains("A012"));
    let deger: serde_json::Value = serde_json::from_slice(&lint.stdout).unwrap();
    assert_eq!(deger["durum"], "temiz");
    assert_eq!(deger["uyarilar"][0]["kod"], "A012");
    assert_eq!(deger["tanilar"], serde_json::json!([]));
}
