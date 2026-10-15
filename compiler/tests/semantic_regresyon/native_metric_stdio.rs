//! K294 actual owned compiler/stdio capability regression.
#[cfg(unix)]
use sha2::{Digest, Sha256};
use std::path::Path;
pub fn calistir(depo: &Path, kaynak: &Path) -> (i64, Vec<String>) {
    #[cfg(not(unix))]
    {
        let _ = (depo, kaynak);
        panic!("actual Unix owned FD capability unavailable; no synthetic pass");
    }
    #[cfg(unix)]
    {
        let hash = |p: &Path| {
            Sha256::digest(std::fs::read(p).unwrap())
                .iter()
                .map(|x| format!("{x:02x}"))
                .collect::<String>()
        };
        let source_hash = hash(kaynak);
        let binary = Path::new(env!("CARGO_BIN_EXE_dil"));
        let binary_hash = hash(binary);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let out = depo.join(format!(
            "compiler/target/native-metric-stdio-regression-{}-{nonce}",
            std::process::id()
        ));
        let result = std::process::Command::new("python3")
            .args(["-I", "-B"])
            .arg(depo.join("scripts/tests/native-metric-stdio-test.py"))
            .arg("--framework-root")
            .arg(depo)
            .arg("--dil")
            .arg(binary)
            .arg("--output")
            .arg(&out)
            .arg("--fixture-source")
            .arg(kaynak)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{} {}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(out.join("proof.json")).unwrap()).unwrap();
        assert_eq!(proof["schema"], "zee-native-metric-stdio-1");
        assert_eq!(proof["status"], "passed");
        assert_eq!(proof["owned_handles_reaped"], true);
        assert_eq!(proof["source_pins_unchanged"], true);
        assert_eq!(proof["fixture_source_sha256"], source_hash);
        assert_eq!(proof["binary_sha256"], binary_hash);
        assert_eq!(proof["cases"].as_array().unwrap().len(), 26);
        let actual = proof["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "--web-metrik-fd-stderr-spaced")
            .unwrap();
        assert_eq!(actual["control_bytes"], 0);
        assert_eq!(actual["exit"], 2);
        assert_eq!(hash(binary), binary_hash);
        assert_eq!(hash(kaynak), source_hash);
        (actual["exit"].as_i64().unwrap(), Vec::new())
    }
}
