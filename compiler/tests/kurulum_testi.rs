//! K-169/ADR-071 kurulum-kaldırma tatbikatı: gerçek `dil`/`dillsp` ikilileri
//! SHA256SUMS'lu sahte artefakttan kurulur, `dil sürüm` çalışır, manifestle
//! kaldırılır; oynanmış artefakt ve ikinci kurulum reddedilir.

use std::path::{Path, PathBuf};
#[cfg(any(unix, windows))]
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
        "scripts/calisma-ortami-denetle.py --artefakt",
        "scripts/tests/calisma-ortami-test.py",
        "calisma-ortami-${{ github.sha }}-${{ runner.os }}",
        "if-no-files-found: error",
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

#[cfg(any(unix, windows))]
fn sha256_hex(yol: &Path) -> String {
    use sha2::{Digest, Sha256};
    let ozet = Sha256::digest(std::fs::read(yol).expect("okunmalı"));
    ozet.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(any(unix, windows))]
fn betik(ad: &str, argumanlar: &[&str]) -> std::process::Output {
    let mut komut = if cfg!(windows) {
        let mut komut = Command::new("powershell");
        komut.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ]);
        komut.arg(depo().join("scripts").join(ad.replace(".sh", ".ps1")));
        for arguman in argumanlar {
            komut.arg(match *arguman {
                "--artefakt" => "-Artefakt",
                "--hedef" => "-Hedef",
                "--veri" => "-Veri",
                "--zorla" => "-Zorla",
                diger => diger,
            });
        }
        komut
    } else {
        let mut komut = Command::new("bash");
        komut.arg(depo().join("scripts").join(ad)).args(argumanlar);
        komut
    };
    komut.output().expect("betik çalışmalı")
}

#[cfg(any(unix, windows))]
#[test]
fn gercek_ikililer_kurulur_calisir_kaldirilir_ve_oynanmis_artefakt_reddedilir() {
    let dil_ad = format!("dil{}", std::env::consts::EXE_SUFFIX);
    let dillsp_ad = format!("dillsp{}", std::env::consts::EXE_SUFFIX);
    let gecici = std::env::temp_dir().join(format!("zee-kurulum-Türkçe-{}", std::process::id()));
    let artefakt = gecici.join("surum");
    let hedef = gecici.join("bin");
    let veri = gecici.join("veri");
    std::fs::create_dir_all(&artefakt).unwrap();
    std::fs::copy(env!("CARGO_BIN_EXE_dil"), artefakt.join(&dil_ad)).unwrap();
    std::fs::copy(env!("CARGO_BIN_EXE_dillsp"), artefakt.join(&dillsp_ad)).unwrap();
    std::fs::write(
        artefakt.join("SHA256SUMS"),
        format!(
            "{}  {}\n{}  {}\n",
            sha256_hex(&artefakt.join(&dil_ad)),
            dil_ad,
            sha256_hex(&artefakt.join(&dillsp_ad)),
            dillsp_ad
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

    // K-194/GB-032: unsigned kaynak metadata'sı manifest sahipliğini genişletemez.
    let yabanci = gecici.join("yabanci-dosya");
    std::fs::write(&yabanci, b"kurulumun sahibi olmadigi veri").unwrap();
    std::fs::write(
        artefakt.join("GIT_SHA"),
        format!(
            "{}\n{}\t{}\t-\n",
            "1".repeat(40),
            yabanci.display(),
            sha256_hex(&yabanci)
        ),
    )
    .unwrap();
    let metadata_reddi = betik(
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
        !metadata_reddi.status.success(),
        "GIT_SHA üzerinden manifest satırı eklenemez"
    );
    assert!(!hedef.join(&dil_ad).exists() && !hedef.join(&dillsp_ad).exists());
    assert!(!veri.join("kurulum-v1.tsv").exists());
    assert_eq!(
        std::fs::read(&yabanci).unwrap(),
        b"kurulumun sahibi olmadigi veri"
    );
    std::fs::write(
        artefakt.join("GIT_SHA"),
        "0123456789abcdef0123456789abcdef01234567\n",
    )
    .unwrap();

    for kontrol in ['\t', '\r', '\n'] {
        let bozuk_yol = gecici.join(format!("hedef{kontrol}satir"));
        for (hedef_degeri, veri_degeri) in [
            (bozuk_yol.to_str().unwrap(), veri_s),
            (hedef_s, bozuk_yol.to_str().unwrap()),
        ] {
            let yol_reddi = betik(
                "kur.sh",
                &[
                    "--artefakt",
                    artefakt_s,
                    "--hedef",
                    hedef_degeri,
                    "--veri",
                    veri_degeri,
                ],
            );
            assert!(
                !yol_reddi.status.success(),
                "TSV kontrol karakterli yol reddedilmeli"
            );
            assert!(!hedef.join(&dil_ad).exists() && !hedef.join(&dillsp_ad).exists());
            assert!(!veri.join("kurulum-v1.tsv").exists());
        }
    }
    #[cfg(unix)]
    {
        let fiziksel = gecici.join("fiziksel\n");
        std::fs::create_dir_all(&fiziksel).unwrap();
        let takma_yol = gecici.join("gecerli-takma-yol");
        std::os::unix::fs::symlink(&fiziksel, &takma_yol).unwrap();
        let kanonik_ret = betik(
            "kur.sh",
            &[
                "--artefakt",
                artefakt_s,
                "--hedef",
                takma_yol.to_str().unwrap(),
                "--veri",
                veri_s,
            ],
        );
        assert!(
            !kanonik_ret.status.success(),
            "fiziksel yoldaki son newline gizlenemez"
        );
        assert!(!fiziksel.join(&dil_ad).exists() && !fiziksel.join(&dillsp_ad).exists());
        assert!(!veri.join("kurulum-v1.tsv").exists());
    }

    // K-194: ikinci kaydın reddi ilk ikiliyi manifestsiz bırakamaz.
    let iyi_ozetler = std::fs::read_to_string(artefakt.join("SHA256SUMS")).unwrap();
    let kur_argumanlari = [
        "--artefakt",
        artefakt_s,
        "--hedef",
        hedef_s,
        "--veri",
        veri_s,
    ];
    std::fs::write(
        artefakt.join("SHA256SUMS"),
        format!(
            "{}  {}\n{}  {}\n",
            sha256_hex(&artefakt.join(&dil_ad)),
            dil_ad,
            "0".repeat(64),
            dillsp_ad
        ),
    )
    .unwrap();
    let bozuk_ikinci = betik("kur.sh", &kur_argumanlari);
    assert!(!bozuk_ikinci.status.success());
    assert!(
        !hedef.join(&dil_ad).exists(),
        "ikinci checksum reddinde ilk ikili de kurulmaz"
    );
    assert!(!hedef.join(&dillsp_ad).exists());
    assert!(!veri.join("kurulum-v1.tsv").exists());
    std::fs::write(artefakt.join("SHA256SUMS"), &iyi_ozetler).unwrap();

    let saklanan = artefakt.join("dillsp-saklanan");
    std::fs::rename(artefakt.join(&dillsp_ad), &saklanan).unwrap();
    let eksik_ikinci = betik("kur.sh", &kur_argumanlari);
    assert!(!eksik_ikinci.status.success());
    assert!(!hedef.join(&dil_ad).exists());
    assert!(!veri.join("kurulum-v1.tsv").exists());
    std::fs::rename(&saklanan, artefakt.join(&dillsp_ad)).unwrap();

    std::fs::create_dir_all(&hedef).unwrap();
    std::fs::write(hedef.join(&dillsp_ad), b"yabanci dosya").unwrap();
    let mevcut_ikinci = betik("kur.sh", &kur_argumanlari);
    assert!(!mevcut_ikinci.status.success());
    assert!(!hedef.join(&dil_ad).exists());
    assert_eq!(
        std::fs::read(hedef.join(&dillsp_ad)).unwrap(),
        b"yabanci dosya"
    );
    assert!(!veri.join("kurulum-v1.tsv").exists());
    std::fs::remove_file(hedef.join(&dillsp_ad)).unwrap();

    let ilk_kayit = iyi_ozetler.lines().next().unwrap();
    std::fs::write(
        artefakt.join("SHA256SUMS"),
        format!("{iyi_ozetler}{ilk_kayit}\n"),
    )
    .unwrap();
    let yinelenen = betik("kur.sh", &kur_argumanlari);
    assert!(!yinelenen.status.success());
    assert!(!hedef.join(&dil_ad).exists() && !hedef.join(&dillsp_ad).exists());
    assert!(!veri.join("kurulum-v1.tsv").exists());
    std::fs::write(artefakt.join("SHA256SUMS"), &iyi_ozetler).unwrap();

    #[cfg(unix)]
    {
        // İlk yayınlanan ikiliden sonra ikinci ln hatası tüm koşuyu geri alır.
        use std::os::unix::fs::PermissionsExt;
        let sahte_araclar = gecici.join("sahte-araclar");
        std::fs::create_dir_all(&sahte_araclar).unwrap();
        let ln = sahte_araclar.join("ln");
        std::fs::write(
            &ln,
            "#!/bin/sh\ncase \"$2\" in */dillsp) exit 1 ;; esac\nexec /bin/ln \"$@\"\n",
        )
        .unwrap();
        std::fs::set_permissions(&ln, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut yollar = vec![sahte_araclar];
        yollar.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
        let yayin_hatasi = Command::new("bash")
            .arg(depo().join("scripts/kur.sh"))
            .args(kur_argumanlari)
            .env("PATH", std::env::join_paths(yollar).unwrap())
            .output()
            .unwrap();
        assert!(!yayin_hatasi.status.success());
        assert!(!hedef.join(&dil_ad).exists() && !hedef.join(&dillsp_ad).exists());
        assert!(!veri.join("kurulum-v1.tsv").exists());

        std::os::unix::fs::symlink("/zee-olmayan-hedef", hedef.join(&dillsp_ad)).unwrap();
        let kirik_bag = betik("kur.sh", &kur_argumanlari);
        assert!(!kirik_bag.status.success());
        assert!(!hedef.join(&dil_ad).exists());
        assert!(hedef
            .join(&dillsp_ad)
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink());
        std::fs::remove_file(hedef.join(&dillsp_ad)).unwrap();
    }

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
    let surum = Command::new(hedef.join(&dil_ad))
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

    std::fs::write(hedef.join(&dillsp_ad), b"degistirildi").unwrap();
    let kaldir_reddi = betik("kaldir.sh", &["--veri", veri_s]);
    assert!(
        !kaldir_reddi.status.success(),
        "değişmiş dosya zorlamadan silinmez"
    );
    assert!(hedef.join(&dil_ad).exists());
    let kaldir = betik("kaldir.sh", &["--veri", veri_s, "--zorla"]);
    assert!(
        kaldir.status.success(),
        "{}",
        String::from_utf8_lossy(&kaldir.stderr)
    );
    assert!(!hedef.join(&dil_ad).exists() && !hedef.join(&dillsp_ad).exists());
    assert!(!veri.join("kurulum-v1.tsv").exists());

    std::fs::write(artefakt.join(&dil_ad), b"oynanmis").unwrap();
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
        !hedef.join(&dil_ad).exists(),
        "oynanmış artefakttan hiçbir dosya kurulmaz"
    );
    let _ = std::fs::remove_dir_all(&gecici);
}
