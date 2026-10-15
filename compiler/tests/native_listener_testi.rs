//! K-259: actual owned listener/control capability and rollover boundaries.
#[cfg(unix)]
fn kampanya(betik: &str, adet: usize, kompozisyon: bool) {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let nesil = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let rol = if kompozisyon { "native" } else { "preflight" };
    let cikti = depo.join(format!(
        "compiler/target/native-listener-{rol}-{}-{nesil}",
        std::process::id()
    ));
    let mut komut = std::process::Command::new("python3");
    komut
        .args(["-I", "-B"])
        .arg(depo.join(betik))
        .arg("--framework-root")
        .arg(depo)
        .arg("--dil")
        .arg(env!("CARGO_BIN_EXE_dil"))
        .arg("--output")
        .arg(&cikti)
        .current_dir(depo);
    if kompozisyon {
        komut.arg("--metrics-composition");
    }
    let sonuc = komut.output().unwrap();
    assert!(
        sonuc.status.success(),
        "{} {}",
        String::from_utf8_lossy(&sonuc.stdout),
        String::from_utf8_lossy(&sonuc.stderr)
    );
    let kanit: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cikti.join("proof.json")).unwrap()).unwrap();
    assert_eq!(kanit["status"], "passed");
    assert_eq!(kanit["cases"].as_array().unwrap().len(), adet);
}

#[cfg(unix)]
#[test]
fn native_listener_quiet_fd_ve_config_siniri() {
    kampanya("scripts/tests/native-listener-preflight-test.py", 16, false);
}

#[cfg(unix)]
#[test]
fn native_listener_nesil_admission_kota_ve_metrik_kompozisyonu() {
    kampanya("scripts/tests/native-listener-test.py", 17, true);
}

#[cfg(unix)]
#[test]
fn native_listener_gec_eof_mutlak_butceden_sonra_admission_acmaz() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let nesil = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cikti = depo.join(format!(
        "compiler/target/native-listener-deadline-{}-{nesil}.json",
        std::process::id()
    ));
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/native-listener-deadline-test.py"))
        .arg("--framework-root")
        .arg(depo)
        .arg("--dil")
        .arg(env!("CARGO_BIN_EXE_dil"))
        .arg("--output")
        .arg(&cikti)
        .args(["--expect", "rejected"])
        .current_dir(depo)
        .output()
        .unwrap();
    assert!(
        sonuc.status.success(),
        "{} {}",
        String::from_utf8_lossy(&sonuc.stdout),
        String::from_utf8_lossy(&sonuc.stderr)
    );
    let kanit: serde_json::Value = serde_json::from_slice(&std::fs::read(cikti).unwrap()).unwrap();
    assert_eq!(kanit["passed"], true);
    assert_eq!(kanit["response_bytes"], 0);
    assert_eq!(kanit["WNOWAIT_status"], 0);
    assert_eq!(kanit["owned_handles_reaped"], true);
    assert_eq!(kanit["parent_listener_retained_during_probe"], true);
    assert!(
        kanit["EOF_elapsed"].as_f64().unwrap()
            > kanit["deadline_latest_upper_bound"].as_f64().unwrap()
    );
}
