//! K273 managed scaffold lifecycle; real local compiler, not DB migration or P20 closure.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix flock/process capability unavailable")]
fn yonetilen_modul_yirmi_uc_actual_ve_kontrollu_fault_vakasi() {
    #[cfg(not(unix))]
    panic!("actual Unix owned flock/process capability unavailable");
    #[cfg(unix)]
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let output = root.join(format!(
            "compiler/target/yonetilen-modul-{}-{nonce}.json",
            std::process::id()
        ));
        std::fs::create_dir_all(output.parent().unwrap()).unwrap();
        let result = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/modul-yoneticisi-test.py"))
            .arg("--binary")
            .arg(env!("CARGO_BIN_EXE_dil"))
            .arg("--expected-source")
            .arg(env!("ZEE_BUILD_SHA"))
            .arg("--proof")
            .arg(&output)
            .current_dir(root)
            .output()
            .expect("managed module actual test driver");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains("\nRan 23 tests in ") && stderr.trim_end().ends_with("\nOK"));
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
        assert_eq!(proof["schema"], "zee-managed-module-tests-1");
        assert_eq!(proof["status"], "passed");
        assert_eq!(proof["tests"], 23);
        assert_eq!(proof["build"]["source_sha"], env!("ZEE_BUILD_SHA"));
        assert_eq!(proof["source_pins_unchanged"], true);
        assert_eq!(proof["owned_test_scopes_completed"], true);
        assert_eq!(proof["sources"].as_object().unwrap().len(), 13);
    }
}
