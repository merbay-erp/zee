//! K-268: Python/owned-process preparation; native Linux SQL/systemd acceptance separate.
#[test]
#[cfg_attr(
    not(unix),
    ignore = "K-268 owned POSIX process preparation unavailable"
)]
fn sonlu_timer_latch_snapshot_ve_sahipli_surec_sinirlarini_korur() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let result = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(root.join("scripts/tests/finite-timer-test.py"))
        .current_dir(root)
        .output()
        .expect("finite timer preparation");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(result.status.success(), "{stderr}");
    assert!(stderr.contains("\nRan 37 tests in "), "{stderr}");
    assert!(stderr.trim_end().ends_with("\nOK"), "{stderr}");
    assert!(!stderr.contains("skipped"), "{stderr}");
    assert!(result.stdout.is_empty());
}
