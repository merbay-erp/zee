//! K-256: gerçek TLS/statement koşucusu sahte kabulü reddeder.
#[cfg(unix)]
#[test]
fn tls_ve_capable_positive_kapisi_tam_on_dort_vakayi_yurutur() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/postgresql-tls-ci-test.py"))
        .current_dir(depo)
        .output()
        .expect("Python3 TLS strict kapısı");
    let rapor = String::from_utf8_lossy(&sonuc.stderr);
    assert!(sonuc.status.success(), "{rapor}");
    assert!(
        rapor.contains("\nRan 14 tests in ") && rapor.trim_end().ends_with("\nOK"),
        "on dört kapı vakası atlanamaz: {rapor}"
    );
    assert!(sonuc.stdout.is_empty());
}
