//! K-247: native durable queue skor kartının yanlış başarı ve tekrar retleri.
#[test]
fn is_kuyrugu_native_skor_karti_eksik_sahte_ve_tekrari_reddeder() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/is-kuyrugu-ci-test.py"))
        .current_dir(depo)
        .output()
        .expect("Python3 queue strict skor kartı");
    let rapor = String::from_utf8_lossy(&sonuc.stderr);
    assert!(
        sonuc.status.success(),
        "{rapor}{}",
        String::from_utf8_lossy(&sonuc.stdout)
    );
    assert!(sonuc.stdout.is_empty());
    assert!(
        rapor.contains("\nRan 16 tests in ") && rapor.trim_end().ends_with("\nOK"),
        "bütün16 validator vakası atlamasız yürütülmeli: {rapor}"
    );
}
