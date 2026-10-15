//! K-259: actual owned native listener EOF/deadline regression, not parser replay.
#[cfg(unix)]
use sha2::{Digest, Sha256};
use std::path::Path;

#[cfg(unix)]
fn sha256(path: &Path) -> String {
    Sha256::digest(std::fs::read(path).expect("pinned replay source/binary"))
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn calistir(depo: &Path, kaynak: &Path) -> (i64, Vec<String>) {
    #[cfg(not(unix))]
    {
        let _ = (depo, kaynak);
        panic!("native-listener-deadline requires actual Unix owned FD fixture; no synthetic pass");
    }
    #[cfg(unix)]
    {
        let kaynak_hash = sha256(kaynak);
        let binary_hash = sha256(Path::new(env!("CARGO_BIN_EXE_dil")));
        let nesil = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let cikti = depo.join(format!(
            "compiler/target/native-listener-deadline-regression-{}-{nesil}.json",
            std::process::id()
        ));
        let sonuc = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(depo.join("scripts/tests/native-listener-deadline-test.py"))
            .arg("--framework-root")
            .arg(depo)
            .arg("--dil")
            .arg(env!("CARGO_BIN_EXE_dil"))
            .arg("--output")
            .arg(&cikti)
            .arg("--fixture-source")
            .arg(kaynak)
            .args(["--expect", "rejected"])
            .current_dir(depo)
            .output()
            .expect("actual owned native deadline helper");
        assert!(
            sonuc.status.success(),
            "{} {}",
            String::from_utf8_lossy(&sonuc.stdout),
            String::from_utf8_lossy(&sonuc.stderr)
        );
        let kanit: serde_json::Value =
            serde_json::from_slice(&std::fs::read(cikti).unwrap()).unwrap();
        assert_eq!(kanit["schema"], "zee-native-listener-deadline-1");
        for kent in [
            "passed",
            "single_generation_only",
            "owned_handles_reaped",
            "parent_listener_retained_during_probe",
        ] {
            assert_eq!(kanit[kent], true);
        }
        assert_eq!(kanit["expected"], "rejected");
        assert_eq!(kanit["fixture_source_sha256"], kaynak_hash);
        assert_eq!(kanit["fixture_hashes"]["ana.dil"], kaynak_hash);
        assert_eq!(kanit["binary_sha256"], binary_hash);
        assert_eq!(kanit["build"]["schema"], "zee-build-v1");
        for path in [
            "scripts/worker-supervisor.py",
            "scripts/tests/native-listener-deadline-test.py",
            "compiler/src/web_listener.rs",
            "compiler/src/main.rs",
        ] {
            let observed = sha256(&depo.join(path));
            assert_eq!(kanit["sources"][path], observed);
        }
        assert_eq!(kanit["response_bytes"], 0);
        assert_eq!(kanit["late_activation_HTTP200"], false);
        assert!(
            kanit["EOF_elapsed"].as_f64().unwrap()
                > kanit["deadline_latest_upper_bound"].as_f64().unwrap()
        );
        // This mode observes native leader exit and the HTTP response channel.
        let kod = kanit["WNOWAIT_status"]
            .as_i64()
            .expect("observed terminal native leader");
        assert_eq!(kod, 0);
        assert_eq!(sha256(Path::new(env!("CARGO_BIN_EXE_dil"))), binary_hash);
        (kod, Vec::new())
    }
}
