//! K286: additive CLI guard and explicit actual owned native/PG/source cohort.
#[test]
fn web_test_koprusu_eksik_yanlis_komut_ve_duplicate_profili_yurutmez() {
    let binary = env!("CARGO_BIN_EXE_dil");
    for args in [
        vec!["dene", "--web-suite", "PRIVATE_CANARY"],
        vec!["çalıştır", "--web-suite", "PRIVATE_CANARY"],
        vec![
            "dene",
            "--web-suite",
            "PRIVATE_CANARY",
            "--web-suite",
            "PRIVATE_CANARY",
        ],
    ] {
        let out = std::process::Command::new(binary)
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&out.stderr).contains("PRIVATE_CANARY"));
    }
    #[cfg(unix)]
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let result = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/web-test-bridge-preflight-test.py"))
            .current_dir(root)
            .output()
            .expect("web bridge lifecycle preparation");
        let report = String::from_utf8_lossy(&result.stderr);
        assert!(result.status.success(), "{report}");
        assert!(report.contains("\nRan 15 tests in ") && report.trim_end().ends_with("\nOK"));
        assert!(result.stdout.is_empty());
    }
}

#[test]
#[ignore = "actual Unix owned PG16.15; explicit ZEE_WEB_TEST_PYTHON and ZEE_WEB_TEST_PG_BIN required"]
fn web_test_koprusu_unit_native_http_pg_ve_kaynak_kapsami_birlikte() {
    #[cfg(not(unix))]
    panic!("actual Unix owned process capability unavailable");
    #[cfg(unix)]
    {
        use sha2::{Digest, Sha256};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let out = root.join(format!(
            "compiler/target/dene-native-kit-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&out).unwrap();
        let binary = env!("CARGO_BIN_EXE_dil");
        let pg = std::env::var_os("ZEE_WEB_TEST_PG_BIN").expect("explicit owned PG16.15 runtime");
        let python =
            std::env::var_os("ZEE_WEB_TEST_PYTHON").expect("explicit trusted operator Python");
        let suite = root.join("compiler/tests/fixtures/owned-web-kit-suite.json");
        let pin = Sha256::digest(std::fs::read(&suite).unwrap())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let result = std::process::Command::new(binary)
            .arg("dene")
            .arg("--kapsam")
            .arg(out.join("unit-coverage.json"))
            .arg("--web-suite")
            .arg(&suite)
            .arg("--web-suite-sha256")
            .arg(pin)
            .arg("--web-fixtures")
            .arg(root.join("compiler/tests/fixtures/owned-web-kit"))
            .arg("--test-python")
            .arg(&python)
            .arg("--test-postgres-bin")
            .arg(&pg)
            .arg("--test-output")
            .arg(out.join("bridge"))
            .arg(root.join("dogfood/zee-cekirdek/test-kapsami"))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{} {}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(out.join("bridge/results/proof.json")).unwrap())
                .unwrap();
        assert_eq!(proof["status"], "passed");
        assert_eq!(proof["owned_cleanup"], true);
        assert_eq!(proof["source_pins_unchanged"], true);
        assert_eq!(proof["unit"]["passed"], true);
        assert_eq!(proof["unit"]["source_coverage"]["complete"], true);
        assert_eq!(proof["unit"]["source_coverage"]["product"]["payda"], 14);
        let cases = proof["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 4);
        assert_eq!(
            cases
                .iter()
                .map(|x| x["requests"].as_u64().unwrap())
                .sum::<u64>(),
            19
        );
        assert!(cases
            .iter()
            .all(|x| x["passed"] == true
                && x["source_coverage"]["sha256"].as_str().unwrap().len() == 64));
        assert_ne!(
            cases[2]["cluster_system_identifier"],
            cases[3]["cluster_system_identifier"]
        );
        assert_eq!(cases[2]["observations"][3]["value"], 0);
        assert_eq!(cases[3]["observations"][0]["value"], 0);
        let negative = std::process::Command::new(python)
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/web-test-bridge-test.py"))
            .arg("--framework-root")
            .arg(root)
            .arg("--dil")
            .arg(binary)
            .arg("--postgres-bin-dir")
            .arg(pg)
            .arg("--output")
            .arg(out.join("negatives"))
            .output()
            .unwrap();
        assert!(
            negative.status.success(),
            "{}",
            String::from_utf8_lossy(&negative.stderr)
        );
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(out.join("negatives/proof.json")).unwrap())
                .unwrap();
        assert_eq!(proof["status"], "passed");
        assert_eq!(proof["owned_groups_closed"], true);
        assert_eq!(proof["cases"].as_array().unwrap().len(), 6);
    }
}
