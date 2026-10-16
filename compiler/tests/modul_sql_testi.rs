//! K285: owned typed SQL modules, native CRUD and append-only down/up.
use std::path::Path;
use std::process::Command;

#[test]
#[cfg_attr(not(unix), ignore = "actual POSIX nofollow/flock profile unavailable")]
fn modul_sql_stdlib_sozlesmesi() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let result = Command::new("python3")
        .args(["-I", "-B"])
        .arg(root.join("scripts/tests/modul-sql-test.py"))
        .current_dir(root)
        .output()
        .expect("owned SQL module contract tests");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("\nRan 23 tests in ") && stderr.trim_end().ends_with("\nOK"));
}

#[test]
#[ignore = "fresh operator-owned PostgreSQL 16.15; exact clean source native campaign"]
fn modul_sql_native_crud_goc_yasam_dongusu() {
    #[cfg(not(unix))]
    panic!("actual POSIX owned PostgreSQL profile required");
    #[cfg(unix)]
    {
        use sha2::{Digest, Sha256};
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let postgres = std::env::var_os("ZEE_MODULE_POSTGRES_BIN_DIR")
            .expect("explicit owned PostgreSQL binaries");
        let binary = Path::new(env!("CARGO_BIN_EXE_dil"));
        let sha = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap();
        assert!(sha.status.success());
        let sha = String::from_utf8(sha.stdout).unwrap();
        let bytes = std::fs::read(binary).unwrap();
        assert!(bytes.len() <= 134_217_728);
        let hash: String = Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let parent = std::env::temp_dir().join(format!(
            "zee-module-sql-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&parent).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).unwrap();
        let parent = parent.canonicalize().unwrap();
        let output = parent.join("native");
        let result = Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/modul-sql-native.py"))
            .arg("--framework-root")
            .arg(root)
            .arg("--dil")
            .arg(binary)
            .arg("--sha")
            .arg(sha.trim())
            .arg("--binary-sha256")
            .arg(hash)
            .arg("--postgres-bin-dir")
            .arg(postgres)
            .arg("--output")
            .arg(&output)
            .current_dir(root)
            .output()
            .expect("actual owned SQL module campaign");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stdout.is_empty());
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(output.join("proof.json")).unwrap()).unwrap();
        assert_eq!(proof["status"], "passed");
        assert_eq!(proof["mode"], "clean-exact");
        assert_eq!(proof["case_count"], 9);
        assert_eq!(proof["failed"], 0);
        assert_eq!(proof["temporary_private_root_removed"], true);
        assert_eq!(proof["cleanup"], serde_json::json!(["owned-pg-closed"]));
    }
}
