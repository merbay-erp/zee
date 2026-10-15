//! K267 typed credential/listener composition; Unix actual FD authority only.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned FD capability unavailable")]
fn credential_listener_bes_capability_ve_presecret_siniri() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/credential-listener-test.py"))
        .current_dir(depo)
        .output()
        .unwrap();
    let rapor = String::from_utf8_lossy(&sonuc.stderr);
    assert!(sonuc.status.success(), "{rapor}");
    assert!(
        rapor.contains("\nRan 5 tests in ") && rapor.trim_end().ends_with("\nOK"),
        "{rapor}"
    );
    assert!(sonuc.stdout.is_empty());
}

#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned FD capability unavailable")]
fn credential_listener_bes_gercek_native_crypto_ve_nesil_kampanyasi() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let nesil = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cikti = depo.join(format!(
        "compiler/target/credential-listener-native-{}-{nesil}",
        std::process::id()
    ));
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/credential-listener-native.py"))
        .arg("--dil")
        .arg(env!("CARGO_BIN_EXE_dil"))
        .arg("--output")
        .arg(&cikti)
        .current_dir(depo)
        .output()
        .unwrap();
    assert!(
        sonuc.status.success(),
        "{} {}",
        String::from_utf8_lossy(&sonuc.stdout),
        String::from_utf8_lossy(&sonuc.stderr)
    );
    let kanit: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cikti.join("proof.json")).unwrap()).unwrap();
    assert_eq!(kanit["schema"], "zee-credential-listener-native-1");
    assert_eq!(kanit["status"], "passed");
    let cases = kanit["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 5);
    assert!(cases.iter().all(|c| c["passed"] == true));
    assert_eq!(kanit["owned_groups_reaped"], true);
    assert_eq!(kanit["build"]["schema"], "zee-build-v1");
}
