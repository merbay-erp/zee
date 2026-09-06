//! K-169/ADR-071 kurulum-kaldırma tatbikatı: gerçek `dil`/`dillsp` ikilileri
//! SHA256SUMS'lu sahte artefakttan kurulur, `dil sürüm` çalışır, manifestle
//! kaldırılır; oynanmış artefakt ve ikinci kurulum reddedilir.

use std::path::{Path, PathBuf};
use std::process::Command;

fn depo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü")
        .to_path_buf()
}

fn oku(goreli: &str) -> String {
    std::fs::read_to_string(depo().join(goreli)).expect("depo dosyası okunmalı")
}

#[test]
fn kurulum_betikleri_ve_tatbikat_workflowu_sozlesmeyi_tasir() {
    for betik in ["scripts/kur.sh", "scripts/kaldir.sh"] {
        let metin = oku(betik);
        assert!(metin.contains("set -euo pipefail"), "{betik}");
        assert!(metin.contains("zee-kurulum-1"), "{betik}");
        assert!(metin.contains("sha256"), "{betik}");
    }
    assert!(oku("scripts/kur.sh").contains("zaten var; kaldırmadan üzerine yazılmaz"));
    assert!(oku("scripts/kaldir.sh").contains("kurulumdan sonra değişmiş; hiçbir dosya silinmedi"));
    for betik in ["scripts/kur.ps1", "scripts/kaldir.ps1"] {
        let metin = oku(betik);
        assert!(metin.contains("Get-FileHash -Algorithm SHA256"), "{betik}");
        assert!(metin.contains("zee-kurulum-1"), "{betik}");
    }
    let workflow = oku(".github/workflows/kurulum-tatbikati.yml");
    for parca in [
        "os: [ubuntu-latest, macos-latest, windows-latest]",
        "bash scripts/surum-artefakti.sh --cikti",
        "bash scripts/kur.sh --artefakt",
        "bash scripts/kaldir.sh --veri",
        "powershell -File scripts/kur.ps1",
        "powershell -File scripts/kaldir.ps1",
        "persist-credentials: false",
    ] {
        assert!(
            workflow.contains(parca),
            "tatbikat workflow adımı eksik: {parca}"
        );
    }
    for satir in workflow.lines().filter(|s| s.contains("cargo ")) {
        assert!(
            satir.contains("--locked"),
            "kilitsiz tatbikat komutu: {satir}"
        );
    }
    assert!(oku("docs/surum-runbook.md").contains("guvenlik-kapisi.sh --surum-adayi"));
}

#[cfg(unix)]
fn sha256_hex(yol: &Path) -> String {
    use sha2::{Digest, Sha256};
    let ozet = Sha256::digest(std::fs::read(yol).expect("okunmalı"));
    ozet.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(unix)]
fn betik(ad: &str, argumanlar: &[&str]) -> std::process::Output {
    Command::new("bash")
        .arg(depo().join("scripts").join(ad))
        .args(argumanlar)
        .env("HOME", std::env::temp_dir())
        .output()
        .expect("betik çalışmalı")
}

#[cfg(unix)]
#[test]
fn gercek_ikililer_kurulur_calisir_kaldirilir_ve_oynanmis_artefakt_reddedilir() {
    let gecici = std::env::temp_dir().join(format!("zee-kurulum-{}", std::process::id()));
    let artefakt = gecici.join("surum");
    let hedef = gecici.join("bin");
    let veri = gecici.join("veri");
    std::fs::create_dir_all(&artefakt).unwrap();
    std::fs::copy(env!("CARGO_BIN_EXE_dil"), artefakt.join("dil")).unwrap();
    std::fs::copy(env!("CARGO_BIN_EXE_dillsp"), artefakt.join("dillsp")).unwrap();
    std::fs::write(
        artefakt.join("SHA256SUMS"),
        format!(
            "{}  dil\n{}  dillsp\n",
            sha256_hex(&artefakt.join("dil")),
            sha256_hex(&artefakt.join("dillsp"))
        ),
    )
    .unwrap();
    std::fs::write(
        artefakt.join("GIT_SHA"),
        "0123456789abcdef0123456789abcdef01234567\n",
    )
    .unwrap();
    let (artefakt_s, hedef_s, veri_s) = (
        artefakt.to_str().unwrap(),
        hedef.to_str().unwrap(),
        veri.to_str().unwrap(),
    );

    let kur = betik(
        "kur.sh",
        &[
            "--artefakt",
            artefakt_s,
            "--hedef",
            hedef_s,
            "--veri",
            veri_s,
        ],
    );
    assert!(
        kur.status.success(),
        "{}",
        String::from_utf8_lossy(&kur.stderr)
    );
    let manifest = std::fs::read_to_string(veri.join("kurulum-v1.tsv")).unwrap();
    assert!(manifest.starts_with("# zee-kurulum-1\n"));
    assert_eq!(manifest.lines().filter(|s| !s.starts_with('#')).count(), 2);
    let surum = Command::new(hedef.join("dil"))
        .arg("sürüm")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&surum.stdout).starts_with("dil "));

    let ikinci = betik(
        "kur.sh",
        &[
            "--artefakt",
            artefakt_s,
            "--hedef",
            hedef_s,
            "--veri",
            veri_s,
        ],
    );
    assert!(
        !ikinci.status.success(),
        "kurulu sistemde ikinci kurulum reddedilmeli"
    );

    std::fs::write(hedef.join("dillsp"), b"degistirildi").unwrap();
    let kaldir_reddi = betik("kaldir.sh", &["--veri", veri_s]);
    assert!(
        !kaldir_reddi.status.success(),
        "değişmiş dosya zorlamadan silinmez"
    );
    assert!(hedef.join("dil").exists());
    let kaldir = betik("kaldir.sh", &["--veri", veri_s, "--zorla"]);
    assert!(
        kaldir.status.success(),
        "{}",
        String::from_utf8_lossy(&kaldir.stderr)
    );
    assert!(!hedef.join("dil").exists() && !hedef.join("dillsp").exists());
    assert!(!veri.join("kurulum-v1.tsv").exists());

    std::fs::write(artefakt.join("dil"), b"oynanmis").unwrap();
    let oynanmis = betik(
        "kur.sh",
        &[
            "--artefakt",
            artefakt_s,
            "--hedef",
            hedef_s,
            "--veri",
            veri_s,
        ],
    );
    assert!(!oynanmis.status.success());
    assert!(String::from_utf8_lossy(&oynanmis.stderr).contains("SHA-256 uyuşmuyor"));
    assert!(
        !hedef.join("dil").exists(),
        "oynanmış artefakttan hiçbir dosya kurulmaz"
    );
    let _ = std::fs::remove_dir_all(&gecici);
}
