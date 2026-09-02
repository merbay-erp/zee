//! K-153: kullanıcı-perspektifli LSP process cold-start ölçümünün gerçek ikili kanıtı.

use std::process::Command;

#[test]
fn olcum_dillsp_process_spawn_stdio_ve_capabilities_yolunu_olcer() {
    let cikti = Command::new(env!("CARGO_BIN_EXE_olcum"))
        .args([
            "--hizli",
            "--dillsp",
            env!("CARGO_BIN_EXE_dillsp"),
            "--milestone",
            "K-153-test",
        ])
        .output()
        .expect("ölçüm ikilisi başlatılmalı");
    assert!(
        cikti.status.success(),
        "ölçüm başarısız: {}",
        String::from_utf8_lossy(&cikti.stderr)
    );
    let rapor = String::from_utf8(cikti.stdout).expect("ölçüm çıktısı UTF-8 olmalı");
    assert!(rapor.contains("`lsp_engine_initialize`"));
    assert!(rapor.contains("`lsp_process_cold_start`"));
    assert!(rapor.contains("process spawn + stdio initialize capabilities yanıtı"));
}
