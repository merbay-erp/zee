//! K-272 fixed typed TR/EN full-page/list-detail HTTP golden, not product adoption.
use serde_json::Value;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
#[cfg_attr(
    not(unix),
    ignore = "K-272 owned inherited listener requires actual Unix capabilities; Windows acceptance open"
)]
fn tam_tr_en_liste_detay_http_golden_ve_eski_fragment_korunur() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let output = root.join(format!(
        "compiler/target/tr-en-list-detail-native-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    let result = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(root.join("scripts/tests/tr-en-list-detail-test.py"))
        .arg("--framework-root")
        .arg(root)
        .arg("--dil")
        .arg(env!("CARGO_BIN_EXE_dil"))
        .arg("--output")
        .arg(&output)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stderr.is_empty());
    let proof: Value =
        serde_json::from_slice(&std::fs::read(output.join("proof.json")).unwrap()).unwrap();
    assert_eq!(proof["schema"], "zee-tr-en-list-detail-native-1");
    assert_eq!(proof["http_requests"], 9);
    assert_eq!(proof["source_tests"], 5);
    assert_eq!(proof["old_fragment_native"], 7);
    assert_eq!(proof["old_fragment_source_tests"], 4);
    assert_eq!(proof["cases"].as_array().unwrap().len(), 11);
    assert!(proof["cases"]
        .as_array()
        .unwrap()
        .iter()
        .all(|case| case["passed"] == true));
    for key in [
        "old_fragment_unchanged",
        "owned_cleanup",
        "foreign_survived",
        "original_inputs_unchanged",
    ] {
        assert_eq!(proof[key], true, "{key}");
    }
    assert_eq!(proof["source_files"].as_array().unwrap().len(), 5);
    assert_eq!(proof["snapshots"].as_array().unwrap().len(), 7);
    assert_eq!(proof["live_product_acceptance"], false);
    assert_eq!(proof["full_P08"], false);
    // Retain the raw owned proof for full-phase evidence collection.
}
