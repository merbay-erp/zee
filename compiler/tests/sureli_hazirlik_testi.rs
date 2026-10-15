//! K-274: süreli hazırlık ve finite DB watcher yaşam döngüsü.
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Command;
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned process capability unavailable")]
fn sureli_hazirlik_ve_db_watch_sinirlari() {
    #[cfg(not(unix))]
    panic!("actual Unix owned process/PG capability unavailable");
    #[cfg(unix)]
    {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        for (script, count) in [
            ("readiness-ttl-test.py", 6),
            ("db-readiness-watch-test.py", 10),
        ] {
            let result = Command::new("python3")
                .args(["-I", "-B"])
                .arg(root.join("scripts/tests").join(script))
                .current_dir(root)
                .output()
                .unwrap();
            let stderr = String::from_utf8_lossy(&result.stderr);
            assert!(result.status.success(), "{stderr}");
            assert!(
                stderr.contains(&format!("\nRan {count} tests in "))
                    && stderr.trim_end().ends_with("\nOK"),
                "{stderr}"
            );
            assert!(result.stdout.is_empty());
        }
    }
}

#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned process capability unavailable")]
fn native_iki_worker_ttl_ve_config_hup_yenileme() {
    #[cfg(not(unix))]
    panic!("actual Unix owned process/PG capability unavailable");
    #[cfg(unix)]
    {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let output = root.join("compiler/target").join(format!(
            "readiness-ttl-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let result = Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/readiness-ttl-native.py"))
            .arg("--dil")
            .arg(env!("CARGO_BIN_EXE_dil"))
            .arg("--output")
            .arg(&output)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stderr.is_empty());
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(output.join("proof.json")).unwrap()).unwrap();
        assert_eq!(proof["status"], "passed");
        assert_eq!(proof["case_count"], 5);
        for field in [
            "source_unchanged",
            "binary_unchanged",
            "owned_workers_closed",
        ] {
            assert_eq!(proof[field], true, "{field}");
        }
    }
}

#[test]
#[ignore = "actual Unix owned PG16.15 + exact clean native binary; explicit final campaign"]
fn native_pg_tekrarlanan_hazirlik_ve_sigkill_ttl() {
    #[cfg(not(unix))]
    panic!("actual Unix owned process/PG capability unavailable");
    #[cfg(unix)]
    {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let binary = std::env::var("ZEE_K274_CLEAN_DIL").expect("explicit clean source binary");
        let postgres = std::env::var("ZEE_K274_PG_BIN").expect("owned PG runtime");
        let output = root.join("compiler/target").join(format!(
            "db-readiness-watch-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let result = Command::new("python3")
            .args(["-I", "-B"])
            .arg(root.join("scripts/tests/db-readiness-watch-native.py"))
            .arg("--dil")
            .arg(binary)
            .arg("--framework-root")
            .arg(root)
            .arg("--postgres-bin-dir")
            .arg(postgres)
            .arg("--output")
            .arg(&output)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stderr.is_empty());
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(output.join("proof.json")).unwrap()).unwrap();
        assert_eq!(proof["status"], "passed");
        assert_eq!(proof["case_count"], 15);
    }
}
