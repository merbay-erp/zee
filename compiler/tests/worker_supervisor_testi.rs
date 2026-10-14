//! K-246: gerçek worker, explicit readiness ve sahipli süreç grubu kampanyası.
#[cfg(unix)]
#[test]
fn worker_host_native_credential_ve_launcher_kampanyasi_atlanamaz() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/worker-supervisor-test.py"))
        .args(["--dil", env!("CARGO_BIN_EXE_dil")])
        .current_dir(depo)
        .output()
        .expect("Python3 actual worker kampanyası");
    let rapor = String::from_utf8_lossy(&sonuc.stderr);
    assert!(
        sonuc.status.success(),
        "{rapor}{}",
        String::from_utf8_lossy(&sonuc.stdout)
    );
    assert!(
        rapor.contains("\nRan 25 tests in ") && rapor.trim_end().ends_with("\nOK"),
        "native+credential+public launcher25 vakası atlamasız yürümeli: {rapor}"
    );
    let ozet: serde_json::Value = serde_json::from_slice(&sonuc.stdout).expect("tek JSON özet");
    assert_eq!(ozet["schema"], "zee-worker-supervisor-tests-1");
    for (alan, sayi) in [("tests", 25), ("failed", 0), ("errors", 0), ("ignored", 0)] {
        assert_eq!(ozet[alan].as_u64(), Some(sayi), "{alan}");
    }
    assert_eq!(ozet["actual_native_workers"].as_bool(), Some(true));
    assert_eq!(
        ozet["credential_composition_native_crypto_and_group_cleanup"].as_bool(),
        Some(true)
    );
    let build = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["sürüm", "--json"])
        .output()
        .expect("bağımsız native kimlik");
    assert!(build.status.success());
    assert_eq!(
        ozet["native_build"],
        serde_json::from_slice::<serde_json::Value>(&build.stdout).unwrap()
    );
}
