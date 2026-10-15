//! K279: exact old corpus and an explicit existing migration, never silent rollback.
#[test]
#[cfg_attr(not(unix), ignore = "actual POSIX owned command profile unavailable")]
fn onceki_kucuk_surum_korpusu_ve_bilinen_goc_gercek_cli_ile_korunur() {
    #[cfg(not(unix))]
    panic!("actual POSIX owned command profile unavailable");
    #[cfg(unix)]
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let output = root.join("compiler/target").join(format!(
            "onceki-kucuk-surum-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let result = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/onceki-kucuk-surum-test.py"))
            .arg("--binary")
            .arg(env!("CARGO_BIN_EXE_dil"))
            .arg("--expected-source")
            .arg(env!("ZEE_BUILD_SHA"))
            .arg("--output")
            .arg(&output)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stdout.is_empty() && result.stderr.is_empty());
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(output.join("proof.json")).unwrap()).unwrap();
        assert_eq!(proof["status"], "passed");
        assert_eq!(proof["case_count"], 49);
        let cases = proof["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 49);
        assert!(cases.iter().all(|case| case["passed"] == true));
        assert_eq!(proof["build"]["source_sha"], env!("ZEE_BUILD_SHA"));
        assert_eq!(proof["source_pins_unchanged"], true);
        assert_eq!(proof["owned_commands_closed"], true);
        assert_eq!(proof["unmodified_corpus_all_compatible"], false);
        assert_eq!(proof["legacy_rollback_accepted"], false);
    }
}

#[test]
#[cfg_attr(not(unix), ignore = "actual POSIX FIFO/bootstrap profile unavailable")]
fn onceki_surum_fixture_bozuklugu_native_baslamadan_reddedilir() {
    #[cfg(not(unix))]
    panic!("actual POSIX FIFO/bootstrap profile unavailable");
    #[cfg(unix)]
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let result = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/onceki-kucuk-surum-preflight-test.py"))
            .current_dir(root)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(result.status.success(), "{stderr}");
        assert!(result.stdout.is_empty());
        assert!(stderr.contains("\nRan 5 tests in ") && stderr.trim_end().ends_with("\nOK"));
        assert!(!stderr.contains("skipped"));
    }
}
