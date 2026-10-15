//! K-258: the bounded host must mask argument errors and prove owned SQL effects.
#[cfg(unix)]
#[test]
fn dispatcher_arguman_tanisi_ozel_degeri_sizdirmaz() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/job-dispatcher-mask-test.py"))
        .current_dir(depo)
        .output()
        .unwrap();
    let rapor = String::from_utf8_lossy(&sonuc.stderr);
    assert!(sonuc.status.success(), "{rapor}");
    assert!(
        rapor.contains("\nRan 2 tests in ") && rapor.trim_end().ends_with("\nOK"),
        "{rapor}"
    );
    assert!(sonuc.stdout.is_empty());
}

#[cfg(unix)]
#[test]
#[ignore = "owned PostgreSQL TLS runtime; set ZEE_JOB_DISPATCHER_PG_BIN for the dedicated campaign"]
fn dispatcher_owned_pg_heartbeat_commit_ve_grup_sahipligi() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let pg = std::env::var_os("ZEE_JOB_DISPATCHER_PG_BIN")
        .expect("explicit owned PostgreSQL directory required");
    let nesil = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cikti = depo.join(format!(
        "compiler/target/job-dispatcher-owned-{}-{nesil}",
        std::process::id()
    ));
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/job-dispatcher-native-test.py"))
        .arg("--framework-root")
        .arg(depo)
        .arg("--dil")
        .arg(env!("CARGO_BIN_EXE_dil"))
        .arg("--postgres-bin-dir")
        .arg(pg)
        .arg("--output")
        .arg(&cikti)
        .current_dir(depo)
        .output()
        .unwrap();
    assert!(
        sonuc.status.success(),
        "owned campaign failed: {} {}",
        String::from_utf8_lossy(&sonuc.stdout),
        String::from_utf8_lossy(&sonuc.stderr)
    );
    let kanit: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cikti.join("proof.json")).unwrap()).unwrap();
    assert_eq!(kanit["status"], "passed");
    let cases = kanit["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 11);
    assert!(cases
        .iter()
        .all(|c| c["passed"] == true && c["ignored"] == false));
    assert_eq!(kanit["owned_cluster_closed"], true);
}
