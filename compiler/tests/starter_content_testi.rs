//! K290: separate owned content/admin/media starter; no compiler command added.
use std::path::Path;
use std::process::Command;

#[test]
#[cfg_attr(
    not(unix),
    ignore = "actual POSIX nofollow/flock/owned-process profile unavailable"
)]
fn icerik_baslangici_stdlib_sozlesmesi() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let result = Command::new("python3")
        .args(["-I", "-B"])
        .arg(root.join("scripts/tests/starter-content-test.py"))
        .current_dir(root)
        .output()
        .expect("owned starter stdlib contracts");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("\nRan 27 tests in ") && stderr.trim_end().ends_with("\nOK"));
}

#[test]
#[ignore = "fresh owned PostgreSQL16.15 and local HTTPS; exact clean source required"]
fn icerik_baslangici_native_crud_medya_yonetim() {
    #[cfg(not(unix))]
    panic!("actual POSIX owned HTTPS/PostgreSQL profile required");
    #[cfg(unix)]
    {
        use sha2::{Digest, Sha256};
        use std::os::unix::fs::PermissionsExt;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let postgres = std::env::var_os("ZEE_STARTER_POSTGRES_BIN_DIR")
            .expect("explicit owned PostgreSQL binaries");
        let binary = Path::new(env!("CARGO_BIN_EXE_dil"));
        let head = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap();
        assert!(head.status.success());
        let head = String::from_utf8(head.stdout).unwrap();
        let bytes = std::fs::read(binary).unwrap();
        assert!(bytes.len() <= 134_217_728);
        let hash: String = Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let output = if let Some(requested) = std::env::var_os("ZEE_STARTER_NATIVE_OUT") {
            let output = std::path::PathBuf::from(requested);
            assert!(output.is_absolute());
            let parent = output
                .parent()
                .expect("explicit native starter output parent");
            assert_eq!(parent.canonicalize().unwrap(), parent);
            assert!(!output.exists());
            output
        } else {
            let parent = std::env::temp_dir().join(format!(
                "zee-starter-content-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&parent).unwrap();
            std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).unwrap();
            let parent = parent.canonicalize().unwrap();
            parent.join("native")
        };
        let result = Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/starter-content-native.py"))
            .arg("--framework-root")
            .arg(root)
            .arg("--dil")
            .arg(binary)
            .arg("--sha")
            .arg(head.trim())
            .arg("--binary-sha256")
            .arg(hash)
            .arg("--postgres-bin-dir")
            .arg(postgres)
            .arg("--output")
            .arg(&output)
            .current_dir(root)
            .output()
            .expect("actual owned content starter");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stdout.is_empty());
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(output.join("proof.json")).unwrap()).unwrap();
        assert_eq!(value["status"], "passed");
        assert_eq!(value["mode"], "clean-exact");
        assert_eq!(value["case_count"], 14);
        assert_eq!(value["failed"], 0);
        assert_eq!(value["private_root_removed"], true);
        assert_eq!(
            value["cleanup"],
            serde_json::json!(["owned-front-and-native-closed", "owned-pg-closed"])
        );
        assert_eq!(value["cancelled"], false);
        assert_eq!(value["deadline_expired"], false);
        let clusters = value["postgres_clusters"]
            .as_array()
            .expect("foreground PG owners");
        assert_eq!(clusters.len(), 1);
        let cluster = &clusters[0];
        assert_eq!(cluster["status"], "closed");
        assert_eq!(cluster["SIGINT_requested"], true);
        assert_eq!(cluster["normal_shutdown"], true);
        assert_eq!(cluster["observed_exit"], 0);
        assert_eq!(cluster["exit"], 0);
        assert_eq!(cluster["reaped"], true);
        assert_eq!(cluster["fallback_used"], false);
        assert_eq!(cluster["owned_child_cleanup_complete"], true);
        assert_eq!(cluster["cleanup_errors"], serde_json::json!([]));
        let readiness = cluster["readiness"]
            .as_array()
            .expect("owned starter readiness");
        assert!(!readiness.is_empty() && readiness.len() <= 300);
        for probe in readiness {
            assert_eq!(probe["reaped"], true);
            assert_eq!(probe["exit"], probe["observed_exit"]);
            assert!(probe["exit"] == 0 || probe["exit"] == 2);
            assert!(probe["status"] == "passed" || probe["status"] == "readiness-retry");
        }
        assert_eq!(value["front_terminal"]["observed_exit"], 0);
        assert_eq!(value["front_terminal"]["exit"], 0);
        assert_eq!(value["front_terminal"]["reaped"], true);
        assert_eq!(value["front_terminal"]["stderr_bytes"], 0);
        assert_eq!(
            value["front_terminal"]["stderr_sha256"],
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
