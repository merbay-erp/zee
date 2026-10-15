//! K294 before/after real pipe admission; no network/service is started.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned FD capability unavailable")]
fn native_metric_stdio_alias_reti_sessiz_olagan_cli_tanisi_gorunur() {
    #[cfg(not(unix))]
    panic!("actual Unix owned FD capability unavailable");
    #[cfg(unix)]
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let out = root.join(format!(
            "compiler/target/native-metric-stdio-{}-{nonce}",
            std::process::id()
        ));
        let result = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/native-metric-stdio-test.py"))
            .arg("--framework-root")
            .arg(root)
            .arg("--dil")
            .arg(env!("CARGO_BIN_EXE_dil"))
            .arg("--output")
            .arg(&out)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{} {}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(out.join("proof.json")).unwrap()).unwrap();
        assert_eq!(proof["status"], "passed");
        assert_eq!(proof["source_pins_unchanged"], true);
        assert_eq!(proof["owned_handles_reaped"], true);
        assert_eq!(proof["cases"].as_array().unwrap().len(), 26);
    }
}
