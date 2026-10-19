//! K-270: owned local import and project launcher; dependency lock is not compiler pin.
#[allow(dead_code)]
#[path = "ortak/ozel_ikili.rs"]
mod ozel_ikili;
use std::path::Path;
use std::process::Command;

#[test]
#[cfg_attr(
    not(unix),
    ignore = "K-270 actual POSIX directory ownership/lock/publication unavailable; Windows acceptance open"
)]
fn yerel_bundle_import_ve_proje_launcher_kilit_sinirlarini_korur() {
    let tools = Path::new(env!("CARGO_BIN_EXE_dil")).parent().unwrap();
    assert!(tools.join("surum_artefakti").is_file());
    assert!(Path::new(env!("CARGO_BIN_EXE_dillsp")).is_file());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let result = Command::new("python3")
        .arg("-I")
        .arg(root.join("scripts/tests/derleyici-yerel-test.py"))
        .arg("--compiler-dir")
        .arg(ozel_ikili::ozel_klasor(
            "yerel",
            &[
                env!("CARGO_BIN_EXE_dil"),
                env!("CARGO_BIN_EXE_dillsp"),
                tools.join("surum_artefakti").to_str().unwrap(),
            ],
        ))
        .current_dir(root)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(result.status.success(), "local selector failed: {stderr}");
    assert!(stderr.contains("Ran 14 tests"), "count changed: {stderr}");
    assert!(
        stderr.lines().any(|line| line == "OK"),
        "not complete: {stderr}"
    );
    assert!(
        !stderr.contains("skipped"),
        "POSIX cases must run: {stderr}"
    );
}
