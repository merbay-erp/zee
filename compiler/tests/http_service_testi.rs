use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("canonical repository root")
}

#[test]
#[cfg_attr(not(unix), ignore = "Unix owned process and descriptor profile")]
fn http_service_hazirlik() {
    let status = Command::new("python3")
        .args(["-I", "-B"])
        .arg(root().join("scripts/tests/http-service-test.py"))
        .status()
        .expect("Python preparation runner");
    assert!(status.success());
}

#[test]
#[ignore = "dedicated clean Linux user-manager and PostgreSQL16.15 campaign"]
fn http_service_native_linux() {
    assert_eq!(std::env::consts::OS, "linux", "actual Linux required");
    let sha = std::env::var("ZEE_SERVICE_NATIVE_SHA").expect("exact clean source SHA");
    let pg = std::env::var("ZEE_SERVICE_NATIVE_PG_BIN").expect("owned PostgreSQL16.15");
    let out = std::env::var("ZEE_HTTP_SERVICE_NATIVE_OUT").expect("fresh private output path");
    let hash = std::env::var("ZEE_SERVICE_NATIVE_BINARY_SHA256").expect("actual binary hash");
    let status = Command::new("python3")
        .args(["-I", "-B"])
        .arg(root().join("scripts/tests/http-service-native.py"))
        .arg("--repo")
        .arg(root())
        .args(["--sha", &sha, "--dil", env!("CARGO_BIN_EXE_dil")])
        .args([
            "--binary-sha256",
            &hash,
            "--postgres-bin-dir",
            &pg,
            "--out",
            &out,
        ])
        .status()
        .expect("dedicated native campaign");
    assert!(status.success());
}
