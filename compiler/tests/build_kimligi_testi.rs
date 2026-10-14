//! K-226: CLI/LSP/SBOM source identities describe the same compiled build.
use serde_json::Value;
use std::process::Command;

#[test]
fn cli_json_lsp_ve_sbom_ayni_kaynak_kimligini_tasir() {
    let cli = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["sürüm", "--json"])
        .output()
        .unwrap();
    assert!(cli.status.success());
    assert!(cli.stderr.is_empty());
    let cli: Value = serde_json::from_slice(&cli.stdout).unwrap();
    let mut sunucu = dil::lsp::Sunucu::yeni();
    let cevap = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
    let lsp: Value = serde_json::from_str(&cevap.govdeler[0]).unwrap();
    assert_eq!(cli, lsp["result"]["serverInfo"]["zeeBuild"]);
    assert_eq!(cli["version"], lsp["result"]["serverInfo"]["version"]);
    assert_eq!(cli["source_sha"], dil::surum::KAYNAK_SHA);
    assert_eq!(cli["dirty"], dil::surum::KIRLI == "true");
    assert_ne!(cli["channel"], "stable", "dev release cannot claim stable");

    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let tmp = std::env::temp_dir().join(format!("zee-build-sbom-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let sums = tmp.join("SHA256SUMS");
    std::fs::write(&sums, format!("{}  dil\n", "0".repeat(64))).unwrap();
    let epoch = Command::new("git")
        .args(["log", "-1", "--format=%ct"])
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(epoch.status.success());
    let epoch = String::from_utf8(epoch.stdout).unwrap();
    let sbom = Command::new(env!("CARGO_BIN_EXE_surum_artefakti"))
        .args([
            "sbom",
            "--sha",
            dil::surum::KAYNAK_SHA,
            "--epoch",
            epoch.trim(),
            "--ozet",
        ])
        .arg(&sums)
        .arg("--kilit")
        .arg(repo.join("compiler/Cargo.lock"))
        .current_dir(repo)
        .output()
        .unwrap();
    std::fs::remove_dir_all(tmp).unwrap();
    assert!(
        sbom.status.success(),
        "{}",
        String::from_utf8_lossy(&sbom.stderr)
    );
    let sbom: Value = serde_json::from_slice(&sbom.stdout).unwrap();
    let graph = sbom["@graph"].as_array().unwrap();
    let root = graph.iter().find(|v| v["name"] == "dil").unwrap();
    assert_eq!(root["software_packageVersion"], cli["version"]);
    assert!(root["software_downloadLocation"]
        .as_str()
        .unwrap()
        .ends_with(dil::surum::KAYNAK_SHA));
    let created = graph.iter().find(|v| v["type"] == "CreationInfo").unwrap()["created"]
        .as_str()
        .unwrap();
    assert!(created.starts_with(dil::surum::KAYNAK_TARIHI));
}
