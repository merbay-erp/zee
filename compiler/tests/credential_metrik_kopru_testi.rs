//! K260 explicit single descriptor capability keeps default noinherit.
#[cfg(unix)]
#[test]
fn credential_metrik_kopru_yedi_owned_fd_siniri() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/credential-metrics-test.py"))
        .current_dir(depo)
        .output()
        .unwrap();
    let rapor = String::from_utf8_lossy(&sonuc.stderr);
    assert!(sonuc.status.success(), "{rapor}");
    assert!(
        rapor.contains("\nRan 7 tests in ") && rapor.trim_end().ends_with("\nOK"),
        "{rapor}"
    );
    assert!(sonuc.stdout.is_empty());
}

#[cfg(unix)]
#[test]
fn credential_metrik_native_worker_hup_ve_grup_temizligi() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let nesil = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cikti = depo.join(format!(
        "compiler/target/credential-metrik-native-{}-{nesil}",
        std::process::id()
    ));
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/credential-metrics-native.py"))
        .arg("--dil")
        .arg(env!("CARGO_BIN_EXE_dil"))
        .arg("--output")
        .arg(&cikti)
        .current_dir(depo)
        .output()
        .unwrap();
    assert!(
        sonuc.status.success(),
        "native campaign failed: {} {}",
        String::from_utf8_lossy(&sonuc.stdout),
        String::from_utf8_lossy(&sonuc.stderr)
    );
    let kanit: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cikti.join("proof.json")).unwrap()).unwrap();
    assert_eq!(kanit["status"], "passed");
    assert_eq!(kanit["cases"].as_array().unwrap().len(), 2);
    assert!(kanit["cases"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["status"] == "passed"));
    assert_eq!(kanit["compiler_identity"]["schema"], "zee-build-v1");
}
