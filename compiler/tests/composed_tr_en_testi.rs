//! K289 actual closed-component/catalog TR/EN HTTP application composition.
use serde_json::Value;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
#[cfg_attr(not(unix), ignore = "K289 requires owned Unix inherited listener")]
fn bilesen_yerlesim_slot_katalog_tr_en_http_ve_kacis() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let output = root.join(format!(
        "compiler/target/composed-tr-en-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let result = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(root.join("scripts/tests/composed-tr-en-test.py"))
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
    assert_eq!(proof["schema"], "zee-composed-tr-en-1");
    assert_eq!(proof["result"], "passed");
    assert_eq!(proof["http_requests"], 10);
    assert_eq!(proof["source_tests"], 4);
    assert_eq!(proof["cases"].as_array().unwrap().len(), 10);
    assert_eq!(proof["owned_cleanup"], true);
    assert_eq!(proof["foreign_survived"], true);
    assert_eq!(proof["product_adoption"], false);
    assert_eq!(proof["typed_html_capability"], false);
}
