//! K281: signed owned PDF stdio invocation; no compiler grammar or native FFI.
use std::path::Path;
use std::process::Command;

#[test]
#[cfg_attr(
    not(unix),
    ignore = "actual POSIX nofollow/owned-process profile unavailable"
)]
fn imzali_pdf_host_stdlib_sozlesmesi() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let result = Command::new("python3")
        .args(["-I", "-B"])
        .arg(root.join("scripts/tests/pdf-plugin-test.py"))
        .current_dir(root)
        .output()
        .expect("owned PDF stdlib contract tests");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(result.status.success(), "{stderr}");
    assert!(result.stdout.is_empty());
    assert!(stderr.contains("\nRan 7 tests in ") && stderr.trim_end().ends_with("\nOK"));
}

#[test]
#[ignore = "operator-selected exact PDF backend; actual signed Zee request/reply campaign"]
fn imzali_pdf_zee_native_istek_yanit_kabulu() {
    #[cfg(not(unix))]
    panic!("actual owned POSIX PDF invocation required");
    #[cfg(unix)]
    {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let python =
            std::env::var_os("ZEE_PDF_BACKEND_PYTHON").expect("explicit owned backend runtime");
        assert!(Path::new(&python).is_absolute());
        let output = std::env::temp_dir().join(format!(
            "zee-pdf-plugin-native-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let result = Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/pdf-plugin-native.py"))
            .arg("--framework-root")
            .arg(root)
            .arg("--dil")
            .arg(env!("CARGO_BIN_EXE_dil"))
            .arg("--verifier")
            .arg(env!("CARGO_BIN_EXE_surum_artefakti"))
            .arg("--python")
            .arg(python)
            .arg("--output")
            .arg(&output)
            .current_dir(root)
            .output()
            .expect("actual signed PDF invocation");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stdout.is_empty());
        let proof = std::fs::read_to_string(output.join("proof.json")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&proof).unwrap();
        assert_eq!(value["status"], "passed");
        assert_eq!(value["case_count"], 9);
        assert_eq!(value["failed"], 0);
        assert_eq!(value["owned_groups_closed"], true);
        assert_eq!(value["synthetic_private_keys_removed"], true);
        assert!(output.join("pdf/catalog.pdf").is_file());
    }
}
