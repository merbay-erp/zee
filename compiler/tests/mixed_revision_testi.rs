use std::path::PathBuf;
use std::process::Command;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("canonical repository root")
}
#[test]
#[cfg_attr(not(unix), ignore = "Unix owned socket/descriptor preparation")]
fn mixed_revision_hazirlik() {
    let status = Command::new("python3")
        .args(["-I", "-B"])
        .arg(root().join("scripts/tests/mixed-revision-test.py"))
        .status()
        .expect("Python preparation");
    assert!(status.success());
}
#[test]
#[ignore = "dedicated exact-clean owned native mixed-revision campaign"]
fn mixed_revision_native() {
    assert_eq!(std::env::consts::FAMILY, "unix");
    let sha = std::env::var("ZEE_NATIVE_WAVE_SHA").expect("exact clean SHA");
    let hash = std::env::var("ZEE_NATIVE_WAVE_BINARY_SHA256").expect("native binary hash");
    let out = std::env::var("ZEE_MIXED_REVISION_OUT").expect("fresh private-parent output");
    let status = Command::new("python3")
        .args(["-I", "-B"])
        .arg(root().join("scripts/tests/mixed-revision-native.py"))
        .arg("--repo")
        .arg(root())
        .args(["--sha", &sha, "--dil", env!("CARGO_BIN_EXE_dil")])
        .args(["--binary-sha256", &hash, "--out", &out])
        .status()
        .expect("actual native wave");
    assert!(status.success());
}
