//! K-251: DB hazırlık host kapısının olumsuz sahiplik ve süre sınırları.
#[cfg(unix)]
#[test]
fn db_hazirlik_host_kapisi_sahiplik_ve_child_sinirlari() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/db-hazirlik-strict-test.py"))
        .current_dir(depo)
        .output()
        .expect("Python3 K-251 strict kampanyası");
    let rapor = String::from_utf8_lossy(&sonuc.stderr);
    assert!(sonuc.status.success(), "{rapor}");
    assert!(
        rapor.contains("\nRan 10 tests in ") && rapor.trim_end().ends_with("\nOK"),
        "on sahiplik/child vakası atlamasız yürümeli: {rapor}"
    );
    assert!(sonuc.stdout.is_empty());
}
