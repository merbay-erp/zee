//! K-257: finite native metadata consumer must reject malformed/poisoned streams.
#[cfg(unix)]
#[test]
fn native_metrik_host_counter_ve_rollup_kapisi() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/native-metrics-model-test.py"))
        .current_dir(depo)
        .output()
        .unwrap();
    let rapor = String::from_utf8_lossy(&sonuc.stderr);
    assert!(sonuc.status.success(), "{rapor}");
    assert!(
        rapor.contains("\nRan 15 tests in ") && rapor.trim_end().ends_with("\nOK"),
        "15 model negative/boundary cases required: {rapor}"
    );
    assert!(sonuc.stdout.is_empty());
}

#[cfg(unix)]
#[test]
#[ignore = "owned PostgreSQL TLS runtime; set ZEE_NATIVE_METRIK_PG_BIN and run this dedicated case"]
fn native_metrik_http_pg_ve_fd_sahipligi() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let pg = std::env::var_os("ZEE_NATIVE_METRIK_PG_BIN")
        .expect("explicit owned PostgreSQL bin directory required");
    let nesil = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cikti = depo.join(format!(
        "compiler/target/native-metrik-owned-{}-{nesil}",
        std::process::id()
    ));
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/native-metrics-test.py"))
        .arg("--dil")
        .arg(env!("CARGO_BIN_EXE_dil"))
        .arg("--framework-root")
        .arg(depo)
        .arg("--postgres-bin-dir")
        .arg(pg)
        .arg("--output")
        .arg(&cikti)
        .current_dir(depo)
        .output()
        .unwrap();
    assert!(
        sonuc.status.success(),
        "native owned campaign failed: {} {}",
        String::from_utf8_lossy(&sonuc.stdout),
        String::from_utf8_lossy(&sonuc.stderr)
    );
    let kanit: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cikti.join("proof.json")).unwrap()).unwrap();
    assert_eq!(kanit["status"], "passed");
    assert_eq!(kanit["cases"].as_array().unwrap().len(), 8);
}
