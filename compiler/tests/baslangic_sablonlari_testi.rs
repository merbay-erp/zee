//! K-269: copied project templates run through the real CLI and owned HTTP host.
#[test]
#[cfg_attr(not(unix), ignore = "actual POSIX owned process groups unavailable")]
fn site_ve_panel_sablonlari_gercek_cli_http_ile_korunur() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output = root.join(format!(
        "compiler/target/baslangic-sablonlari-{}-{nonce}",
        std::process::id()
    ));
    let result = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(root.join("scripts/tests/baslangic-sablonlari-test.py"))
        .arg("--framework-root")
        .arg(root)
        .arg("--dil")
        .arg(env!("CARGO_BIN_EXE_dil"))
        .arg("--output")
        .arg(&output)
        .output()
        .expect("owned starter acceptance runner");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let proof: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output.join("proof.json")).expect("actual proof"))
            .unwrap();
    assert_eq!(proof["status"], "passed");
    assert_eq!(proof["sources_unchanged"], true);
    let cases = proof["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 2);
    assert!(cases.iter().all(|case| case["passed"] == true));
    assert_eq!(proof["cleanup"], "owned groups stopped and reaped");
}
