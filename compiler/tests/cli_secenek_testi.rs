//! K-221: CLI seçenek reddi program argümanlarını tüketmez.

use std::process::Command;

#[test]
fn bilinmeyen_secenek_dosya_hatasi_olmayan_acik_ret_verir() {
    for argumanlar in [
        vec!["çalıştır", "--bilinmeyen-secenek"],
        vec!["denetle", "--bilinmeyen-secenek", "yok.dil"],
        vec!["dene", "--bilinmeyen-secenek"],
        vec!["biçimle", "--bilinmeyen-secenek"],
        vec!["--bilinmeyen-secenek"],
    ] {
        let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
            .args(&argumanlar)
            .output()
            .unwrap();
        assert_eq!(cikti.status.code(), Some(2), "{argumanlar:?}");
        let stderr = String::from_utf8(cikti.stderr).unwrap();
        assert!(stderr.contains("Bilinmeyen seçenek: --bilinmeyen-secenek"));
        assert!(stderr.contains(env!("CARGO_PKG_VERSION")));
        assert!(!stderr.contains("dosyası okunamadı"));
        assert!(cikti.stdout.is_empty());
    }
}

#[test]
fn kaynak_yolundan_sonraki_program_bayragi_korunur() {
    let kaynak =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../golden/28-cli-araci.dil");
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("çalıştır")
        .arg(kaynak)
        .arg("--uygulama-secimi")
        .output()
        .unwrap();
    assert!(cikti.status.success(), "{:?}", cikti);
    assert!(String::from_utf8(cikti.stdout)
        .unwrap()
        .contains("--uygulama-secimi"));
}

#[test]
fn mevcut_json_secimi_gecerli_kalmali() {
    let kaynak = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../regression/cli/bilinmeyen-secenek.dil");
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["denetle", "--json"])
        .arg(kaynak)
        .output()
        .unwrap();
    assert!(cikti.status.success(), "{cikti:?}");
    assert_eq!(
        String::from_utf8(cikti.stdout).unwrap().trim(),
        "{\"durum\":\"temiz\",\"tanilar\":[]}"
    );
}
