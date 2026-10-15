//! K271: developer scenario contract and separate actual owned PG/HTTP cohort.
#[test]
#[cfg_attr(
    not(unix),
    ignore = "actual Unix owned process/TCP capability unavailable"
)]
fn owned_web_kit_yirmi_contract_ve_bounded_helper_vakasi() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let result = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(root.join("scripts/tests/owned-web-kit-test.py"))
        .current_dir(root)
        .output()
        .expect("owned web kit preparation");
    let report = String::from_utf8_lossy(&result.stderr);
    assert!(result.status.success(), "{report}");
    assert!(report.contains("\nRan 20 tests in ") && report.trim_end().ends_with("\nOK"));
    assert!(result.stdout.is_empty());
}

#[test]
#[ignore = "requires actual Unix owned PG16.15; set ZEE_OWNED_WEB_KIT_PG_BIN and run dedicated case"]
fn owned_web_kit_dort_native_http_pg_izolasyon_vakasi() {
    #[cfg(not(unix))]
    panic!("actual Unix owned process capability unavailable");
    #[cfg(unix)]
    {
        use sha2::{Digest, Sha256};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let fixture = root.join("compiler/tests/fixtures/owned-web-kit");
        let suite = root.join("compiler/tests/fixtures/owned-web-kit-suite.json");
        let binary = std::path::Path::new(env!("CARGO_BIN_EXE_dil"));
        let hash = |path: &std::path::Path| {
            Sha256::digest(std::fs::read(path).unwrap())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let output = root.join(format!(
            "compiler/target/owned-web-kit-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(output.parent().unwrap()).unwrap();
        let result = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/owned-web-tests.py"))
            .arg("--fixture-root")
            .arg(fixture)
            .arg("--suite")
            .arg(&suite)
            .arg("--suite-sha256")
            .arg(hash(&suite))
            .arg("--dil")
            .arg(binary)
            .arg("--binary-sha256")
            .arg(hash(binary))
            .arg("--expected-source")
            .arg(env!("ZEE_BUILD_SHA"))
            .arg("--postgres-bin-dir")
            .arg(
                std::env::var_os("ZEE_OWNED_WEB_KIT_PG_BIN")
                    .expect("explicit owned PG16.15 bin directory"),
            )
            .arg("--output")
            .arg(&output)
            .arg("--candidate")
            .current_dir(root)
            .output()
            .expect("actual owned native kit");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(output.join("proof.json")).unwrap()).unwrap();
        assert_eq!(proof["schema"], "zee-owned-web-tests-1");
        assert_eq!(proof["status"], "passed");
        let cases = proof["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 4);
        assert!(cases.iter().all(|case| case["passed"] == true));
        assert_eq!(proof["source_pins_unchanged"], true);
        assert_eq!(proof["owned_cleanup"], true);
        assert_ne!(
            cases[2]["cluster_system_identifier"],
            cases[3]["cluster_system_identifier"]
        );
        assert_eq!(cases[2]["observer_readonly"], true);
        assert_eq!(cases[3]["observer_readonly"], true);
    }
}
