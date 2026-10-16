//! K-277: finite native SQL latch + SMTP effect; preparation and actual cohorts differ.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned process capability unavailable")]
fn native_effect_sekiz_host_hazirlik_vakasi() {
    // Existing Cargo preparation owner also owns the nested selected-AST preflight.
    let _ = include_str!("../../scripts/tests/effect-pg-owner-preflight-test.py");
    let _ = include_str!("../../scripts/tests/effect-pg-owner.py");
    #[cfg(not(unix))]
    panic!("actual Unix owned process capability unavailable");
    #[cfg(unix)]
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let output = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/effect-dispatcher-test.py"))
            .arg("--dil")
            .arg(env!("CARGO_BIN_EXE_dil"))
            .current_dir(root)
            .output()
            .expect("effect host preparation");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{stderr}");
        assert!(output.stdout.is_empty());
        assert!(stderr.contains("\nRan 9 tests in ") && stderr.trim_end().ends_with("\nOK"));
    }
}

#[test]
#[ignore = "actual Unix owned PG16.15; set ZEE_EFFECT_PG_BIN for dedicated native11"]
fn native_effect_onbir_sql_smtp_no_replay_vakasi() {
    #[cfg(not(unix))]
    panic!("actual Unix owned process/PG capability unavailable");
    #[cfg(unix)]
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let postgres =
            std::env::var_os("ZEE_EFFECT_PG_BIN").expect("explicit owned PG16.15 bin directory");
        assert!(std::path::Path::new(&postgres).is_absolute());
        let output_dir = if let Some(value) = std::env::var_os("ZEE_EFFECT_NATIVE_OUT") {
            let path = std::path::PathBuf::from(value);
            assert!(path.is_absolute());
            let parent = path.parent().expect("fresh public native output parent");
            assert_eq!(parent.canonicalize().unwrap(), parent);
            assert!(matches!(
                std::fs::symlink_metadata(&path),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound
            ));
            path
        } else {
            let path = root.join(format!(
                "compiler/target/native-effect-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            path
        };
        let output = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/effect-dispatcher-native-test.py"))
            .arg("--dil")
            .arg(env!("CARGO_BIN_EXE_dil"))
            .arg("--postgres-bin-dir")
            .arg(postgres)
            .arg("--output")
            .arg(&output_dir)
            .current_dir(root)
            .output()
            .expect("owned native SQL/SMTP cohort");
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(output_dir.join("proof.json")).unwrap()).unwrap();
        assert_eq!(proof["schema"], "zee-native-effect-campaign-1");
        assert_eq!(proof["status"], "passed");
        assert_eq!(proof["case_count"], 11);
        let cases = proof["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 11);
        assert!(cases.iter().all(|case| case["passed"] == true));
        for key in [
            "sources_unchanged",
            "binaries_unchanged",
            "owned_groups_closed",
            "foreign_survived",
        ] {
            assert_eq!(proof[key], true, "{key}");
        }
        assert_eq!(proof["build"]["source_sha"], env!("ZEE_BUILD_SHA"));
        assert_eq!(proof["exactly_once_claim"], false);
        assert_eq!(proof["source_pins"].as_object().unwrap().len(), 14);
        // Raw proof and faults remain in the owned target directory for phase evidence.
    }
}
