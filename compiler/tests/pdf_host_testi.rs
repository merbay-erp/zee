//! K-275: stdlib models and explicit operator-selected owned backend acceptance.
use std::path::Path;
use std::process::Command;

#[test]
#[cfg_attr(
    not(unix),
    ignore = "actual POSIX nofollow/FIFO capability unavailable"
)]
fn pdf_job_stdlib_sozlesme_modelleri() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let result = Command::new("python3")
        .args(["-I", "-B"])
        .arg(root.join("scripts/tests/pdf-job-validation-test.py"))
        .current_dir(root)
        .output()
        .expect("Python standard library validation models");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(result.status.success(), "{stderr}");
    assert!(result.stdout.is_empty());
    assert!(stderr.contains("\nRan 8 tests in ") && stderr.trim_end().ends_with("\nOK"));
}

#[test]
#[ignore = "operator-selected exact PDF backend dependencies; explicit owned campaign"]
fn pdf_sabit_owned_backend_kabulu() {
    #[cfg(not(unix))]
    panic!("owned POSIX backend acceptance required");
    #[cfg(unix)]
    {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let python = std::env::var_os("ZEE_PDF_BACKEND_PYTHON")
            .expect("explicit operator-installed backend Python executable");
        assert!(Path::new(&python).is_absolute());
        let directory = std::env::temp_dir().join(format!(
            "zee-pdf-backend-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let result = Command::new(python)
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/pdf-backend-acceptance.py"))
            .arg("--framework-root")
            .arg(root)
            .arg("--output")
            .arg(&directory)
            .current_dir(root)
            .output()
            .expect("explicit owned PDF backend acceptance");
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(result.status.success(), "{stderr}");
        assert!(result.stdout.is_empty());
        assert!(stderr.contains("\nRan 8 tests in ") && stderr.trim_end().ends_with("\nOK"));
        assert!(directory.join("output/pdf/tr/catalog.pdf").is_file());
    }
}
