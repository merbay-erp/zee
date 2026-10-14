//! K-253: gerçek admitted handler, SIGTERM no-admission ve idle shutdown.
#[cfg(unix)]
#[test]
fn native_sigterm_current_handleri_bitirir_yeni_handler_almaz() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    use std::os::unix::fs::DirBuilderExt;
    struct Gecici(std::path::PathBuf);
    impl Drop for Gecici {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).ok();
        }
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let gecici = Gecici(
        std::env::temp_dir().join(format!("zee-native-drain-{}-{nonce}", std::process::id())),
    );
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&gecici.0)
        .unwrap();
    let kanit = gecici.0.join("proof");
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/native-web-drain-test.py"))
        .args([
            "--dil",
            env!("CARGO_BIN_EXE_dil"),
            "--expect-drain",
            "--output",
        ])
        .arg(&kanit)
        .arg("--fixture-project")
        .arg(depo.join("dogfood/zee-cekirdek/hizmet"))
        .current_dir(depo)
        .output()
        .expect("native drain kampanyası");
    assert!(
        sonuc.status.success(),
        "{}{}",
        String::from_utf8_lossy(&sonuc.stdout),
        String::from_utf8_lossy(&sonuc.stderr)
    );
    let proof: serde_json::Value =
        serde_json::from_slice(&std::fs::read(kanit.join("proof.json")).unwrap()).unwrap();
    assert_eq!(proof["status"], "passed");
    assert_eq!(proof["cases"].as_array().unwrap().len(), 4);
    assert_eq!(proof["owned_worker_closed"], true);
}
