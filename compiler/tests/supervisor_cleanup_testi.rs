//! K263 owned lifecycle: permission transitions never release PID authority early.
#[cfg(unix)]
#[test]
fn supervisor_cleanup_sekiz_sahiplik_ve_mutlak_butce_vakasi() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/supervisor-cleanup-test.py"))
        .current_dir(depo)
        .output()
        .unwrap();
    let rapor = String::from_utf8_lossy(&sonuc.stderr);
    assert!(sonuc.status.success(), "{rapor}");
    assert!(
        rapor.contains("\nRan 8 tests in ") && rapor.trim_end().ends_with("\nOK"),
        "{rapor}"
    );
    assert!(sonuc.stdout.is_empty());
}
