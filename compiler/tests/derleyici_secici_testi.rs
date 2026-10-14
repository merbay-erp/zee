//! K-233: Cargo kapısı aynı exact build araçlarıyla signed pilot seçiciyi denetler.
use std::path::Path;
use std::process::Command;

#[test]
#[cfg_attr(
    target_os = "windows",
    ignore = "K-233 sentetik executable protokolü Unix pilotunda kanıtlı; Windows native fixture kabulü açık"
)]
fn signed_pilot_secici_exact_kripto_ve_proje_kapsamini_korur() {
    assert_ne!(
        std::env::consts::OS,
        "windows",
        "Windows explicit fixture kabulü henüz sağlanmıyor"
    );
    let compiler_dir = Path::new(env!("CARGO_BIN_EXE_dil")).parent().unwrap();
    assert!(compiler_dir
        .join(if cfg!(target_os = "windows") {
            "surum_artefakti.exe"
        } else {
            "surum_artefakti"
        })
        .is_file());
    assert!(Path::new(env!("CARGO_BIN_EXE_dillsp")).is_file());
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let result = Command::new("python3")
        .arg("-I")
        .arg(repo.join("scripts/tests/derleyici-secici-test.py"))
        .arg("--compiler-dir")
        .arg(compiler_dir)
        .current_dir(repo)
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&result.stderr);
    assert!(
        result.status.success(),
        "signed selector campaign failed: {report}"
    );
    assert!(
        report.contains("Ran 25 tests"),
        "campaign count changed: {report}"
    );
    assert!(
        report.lines().any(|line| line == "OK"),
        "campaign did not complete: {report}"
    );
    assert!(
        !report.contains("skipped"),
        "pilot campaign must run all cases: {report}"
    );
}
