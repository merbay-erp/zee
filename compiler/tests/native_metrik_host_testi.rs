//! K-257: finite native metadata consumer must reject malformed/poisoned streams.
#[cfg(unix)]
#[test]
fn native_metrik_host_counter_ve_rollup_kapisi() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/native-metrics-model-test.py"))
        .current_dir(depo)
        .output()
        .unwrap();
    let rapor = String::from_utf8_lossy(&sonuc.stderr);
    assert!(sonuc.status.success(), "{rapor}");
    assert!(
        rapor.contains("\nRan 16 tests in ") && rapor.trim_end().ends_with("\nOK"),
        "16 ordinary methods including nested PG/lifecycle suite required: {rapor}"
    );
    assert!(sonuc.stdout.is_empty());
}

#[cfg(unix)]
#[test]
#[ignore = "owned PostgreSQL TLS runtime; set ZEE_NATIVE_METRIK_PG_BIN and run this dedicated case"]
fn native_metrik_http_pg_ve_fd_sahipligi() {
    let depo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let pg = std::env::var_os("ZEE_NATIVE_METRIK_PG_BIN")
        .expect("explicit owned PostgreSQL bin directory required");
    let nesil = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cikti = match std::env::var_os("ZEE_NATIVE_METRIK_OUT") {
        Some(path) => std::path::PathBuf::from(path),
        None => depo.join(format!(
            "compiler/target/native-metrik-owned-{}-{nesil}",
            std::process::id()
        )),
    };
    assert!(cikti.is_absolute() && !cikti.exists());
    let parent = cikti.parent().expect("fresh output parent");
    assert_eq!(parent.canonicalize().unwrap(), parent);
    let sonuc = std::process::Command::new("python3")
        .args(["-I", "-B"])
        .arg(depo.join("scripts/tests/native-metrics-test.py"))
        .arg("--dil")
        .arg(env!("CARGO_BIN_EXE_dil"))
        .arg("--framework-root")
        .arg(depo)
        .arg("--postgres-bin-dir")
        .arg(pg)
        .arg("--output")
        .arg(&cikti)
        .current_dir(depo)
        .output()
        .unwrap();
    assert!(
        sonuc.status.success(),
        "native owned campaign failed: {} {}",
        String::from_utf8_lossy(&sonuc.stdout),
        String::from_utf8_lossy(&sonuc.stderr)
    );
    let kanit: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cikti.join("proof.json")).unwrap()).unwrap();
    assert_eq!(kanit["status"], "passed");
    assert_eq!(kanit["cases"].as_array().unwrap().len(), 8);
    for key in [
        "body_completed",
        "private_root_removed",
        "all_original_children_reaped",
        "all_fds_closed",
        "all_components_closed",
        "owned_pg_closed",
    ] {
        assert_eq!(kanit[key], true, "{key}");
    }
    for key in ["cancelled", "deadline_expired", "cleanup_deadline_expired"] {
        assert_eq!(kanit[key], false, "{key}");
    }
    assert_eq!(kanit["cleanup_errors"], serde_json::json!([]));
    let clusters = kanit["postgres_clusters"].as_array().unwrap();
    assert_eq!(clusters.len(), 1);
    assert_eq!(kanit["nested_pool_timeout_seconds"], 120);
    assert_eq!(
        kanit["nested_pool_selector"],
        "postgresql::havuz::testler::izole_pg_pool_metrikleri_wait_timeout_ve_donus_gercek_sinirlar"
    );
    let profile = std::path::Path::new(env!("CARGO_BIN_EXE_dil"))
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(
        kanit["nested_pool_profile"],
        if profile == "release" {
            "release"
        } else {
            "debug"
        }
    );
    let cluster = &clusters[0];
    for key in [
        "normal_shutdown",
        "SIGINT_requested",
        "reaped",
        "owned_child_cleanup_complete",
        "readiness_identity_verified",
    ] {
        assert_eq!(cluster[key], true, "{key}");
    }
    assert_eq!(cluster["status"], "closed");
    assert_eq!(cluster["fallback_used"], false);
    assert_eq!(cluster["observed_exit"], 0);
    assert_eq!(cluster["exit"], 0);
    assert_eq!(cluster["cleanup_errors"], serde_json::json!([]));
    let probes = cluster["readiness"].as_array().unwrap();
    assert!(!probes.is_empty());
    for (index, probe) in probes.iter().enumerate() {
        assert_eq!(probe["exit"], if index + 1 == probes.len() { 0 } else { 2 });
        assert_eq!(probe["reaped"], true);
        assert_eq!(probe["observed_exit"], probe["exit"]);
    }
    let workers = kanit["worker_owners"].as_array().unwrap();
    assert_eq!(workers.len(), 5);
    for worker in workers {
        assert_eq!(worker["reaped"], true);
    }
    // Alias modes intentionally reject with exit2; no all-worker-zero claim.
    assert_eq!(workers[3]["observed_exit"], 2);
    assert_eq!(workers[4]["observed_exit"], 2);
}
