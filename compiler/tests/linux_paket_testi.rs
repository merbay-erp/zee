//! K-276 packaging preparation only; no installed package/container acceptance.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix nofollow/FIFO capability unavailable")]
fn linux_paket_hazirligi_sonlu_girdi_ve_tasiyici_sinirlarini_korur() {
    #[cfg(not(unix))]
    panic!("actual Unix nofollow/FIFO capability unavailable");
    #[cfg(unix)]
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        for (name, count) in [
            ("linux-package-preflight-test.py", 12),
            ("linux-private-carrier-test.py", 9),
        ] {
            let output = std::process::Command::new("python3")
                .args(["-I", "-B"])
                .arg(root.join("scripts/tests").join(name))
                .current_dir(root)
                .output()
                .expect("Python preparation test must start");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(output.status.success(), "{stderr}");
            assert!(output.stdout.is_empty());
            assert!(
                stderr.contains(&format!("\nRan {count} tests in "))
                    && stderr.trim_end().ends_with("\nOK"),
                "{stderr}"
            );
        }
    }
}
