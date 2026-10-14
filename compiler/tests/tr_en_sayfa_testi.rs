//! K-249: plain Metin slotları ve sabit TR–EN sözlüğü gerçek native CLI'de sınanır.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

fn sha256(yol: &Path) -> String {
    Sha256::digest(std::fs::read(yol).unwrap())
        .iter()
        .map(|bayt| format!("{bayt:02x}"))
        .collect()
}

fn girdileri_dogrula(rapor: &Value, alan: &str, kok: &Path, yollar: &[&str]) {
    let girdiler = rapor[alan].as_array().expect("girdi hash dizisi");
    assert_eq!(girdiler.len(), yollar.len(), "exact girdi sayısı");
    for (girdi, yol) in girdiler.iter().zip(yollar) {
        assert_eq!(girdi["path"].as_str(), Some(*yol));
        assert_eq!(
            girdi["sha256"].as_str(),
            Some(sha256(&kok.join(yol)).as_str())
        );
    }
}

#[test]
fn tr_en_anonim_sayfa_native_cli_kampanyasi_atlanamaz() {
    let depo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let ikili = Path::new(env!("CARGO_BIN_EXE_dil"));
    let proje = depo.join("dogfood/zee-cekirdek/tr-en");
    let snapshotlar = depo.join("compiler/tests/fixtures/tr-en-sayfa");
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/tr-en-sayfa-test.py"))
        .arg("--dil")
        .arg(ikili)
        .arg("--project")
        .arg(&proje)
        .arg("--snapshots")
        .arg(&snapshotlar)
        .current_dir(depo)
        .output()
        .expect("Python3 native TR–EN sayfa kampanyası");
    assert!(
        sonuc.status.success(),
        "{}{}",
        String::from_utf8_lossy(&sonuc.stderr),
        String::from_utf8_lossy(&sonuc.stdout)
    );
    assert!(sonuc.stderr.is_empty(), "native kampanya stderr boş olmalı");
    let rapor: Value = serde_json::from_slice(&sonuc.stdout).expect("tek native JSON raporu");
    assert_eq!(rapor["schema"], "zee-tr-en-native-1");
    assert_eq!(rapor["result"], "passed");
    assert_eq!(rapor["case_count"], 7);
    assert_eq!(rapor["skip_count"], 0);
    assert_eq!(rapor["source_test_count"], 4);
    assert_eq!(rapor["source_test_failed"], 0);
    for alan in [
        "isolated_python",
        "bytecode_disabled",
        "html_structure_checked",
        "owned_temp_removed",
        "source_unchanged",
    ] {
        assert_eq!(rapor[alan], true, "{alan}");
    }
    for alan in [
        "compiler_feature",
        "live_product_acceptance",
        "physical_platform_acceptance",
    ] {
        assert_eq!(rapor[alan], false, "{alan}");
    }
    let vakalar = rapor["cases"].as_array().expect("native vaka dizisi");
    let adlar = [
        "denetle",
        "tr-en",
        "dene",
        "xss",
        "unknown-language",
        "unknown-key",
        "lang-attribute-injection",
    ];
    assert_eq!(vakalar.len(), adlar.len());
    for (vaka, ad) in vakalar.iter().zip(adlar) {
        assert_eq!(vaka["name"], ad);
        assert_eq!(vaka["status"], "passed");
        assert_eq!(vaka["exit"], 0);
        assert_eq!(vaka["stderr_empty"], true);
    }
    assert_eq!(vakalar[2]["source_tests"], 4);
    assert_eq!(vakalar[2]["failed"], 0);
    assert_eq!(rapor["native_binary_sha256"], sha256(ikili));
    assert_eq!(rapor["native_identity_start"], rapor["native_identity_end"]);
    let kimlik = &rapor["native_identity_start"];
    assert_eq!(kimlik["schema"], "zee-build-v1");
    assert!(kimlik["dirty"].is_boolean(), "gerçek build dirty alanı");
    let kaynak_sha = kimlik["source_sha"].as_str().expect("native kaynak SHA");
    assert!(
        kaynak_sha.len() == 40
            && kaynak_sha
                .bytes()
                .all(|bayt| bayt.is_ascii_digit() || (b'a'..=b'f').contains(&bayt)),
        "exact native kaynak SHA"
    );
    girdileri_dogrula(
        &rapor,
        "source_files",
        &proje,
        &[
            "kaynak/ana.dil",
            "kaynak/ceviri.dil",
            "kaynak/sayfa.dil",
            "proje.dil",
        ],
    );
    girdileri_dogrula(
        &rapor,
        "snapshots",
        &snapshotlar,
        &["en.html", "tr.html", "xss.html"],
    );
}
