//! K-245: host credential sınırı gerçek dosyalar, native CLI ve SMTP ile sınanır.

#[cfg(unix)]
#[test]
fn credential_host_kampanyasi_native_izin_ve_smtp_dahil_atlanamaz() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/credential-baslat-test.py"))
        .env("ZEE_CREDENTIAL_TEST_DIL", env!("CARGO_BIN_EXE_dil"))
        .current_dir(depo)
        .output()
        .expect("Python3 POSIX credential kampanyası");
    let rapor = String::from_utf8_lossy(&sonuc.stderr);
    assert!(
        sonuc.status.success(),
        "{rapor}{}",
        String::from_utf8_lossy(&sonuc.stdout)
    );
    assert!(sonuc.stdout.is_empty(), "yalnız unittest sonucu beklenir");
    assert!(
        rapor.contains("\nRan 14 tests in ") && rapor.trim_end().ends_with("\nOK"),
        "native ve ret vakalarının tamamı atlamasız yürütülmeli: {rapor}"
    );
}
