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
    for betik in [
        "scripts/kaldir.ps1",
        "scripts/tests/kurulum-literal-yol-test.ps1",
    ] {
        assert!(
            std::fs::read(depo().join(betik))
                .unwrap()
                .starts_with(b"\xef\xbb\xbf"),
            "Windows PowerShell 5.1 için UTF-8 BOM korunmalı: {betik}"
        );
    }
    let workflow = oku(".github/workflows/kurulum-tatbikati.yml");
    for parca in [
        "os: [ubuntu-latest, macos-latest, windows-latest]",
        "bash scripts/surum-artefakti.sh --cikti",
        "bash scripts/kur.sh --artefakt",
        "bash scripts/kaldir.sh --veri",
        "powershell -File scripts/kur.ps1",
        "powershell -File scripts/kaldir.ps1",
        "-File scripts/tests/kurulum-literal-yol-test.ps1",
        "literal yol kampanyası başarısız",
        "persist-credentials: false",
        "scripts/calisma-ortami-denetle.py --artefakt",
        "scripts/tests/calisma-ortami-test.py",
        "calisma-ortami-${{ github.sha }}-${{ runner.os }}",
        "if-no-files-found: error",
        "statik CRT kanıtı yok",
    ] {
        assert!(
            workflow.contains(parca),
            "tatbikat workflow adımı eksik: {parca}"
        );
    }
    assert!(!workflow.contains("Windows: tek derleme"));
    assert!(!workflow.contains("if: runner.os != 'Windows'\n        shell: bash\n        run: bash scripts/surum-artefakti.sh"));
    assert!(oku("scripts/calisma-ortami-denetle.py").contains("statik_crt_denetle"));
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
    let gecici = std::env::temp_dir().join(format!("zee-kurulum-Türkçe-{}", std::process::id()));
    std::fs::create_dir_all(&gecici).unwrap();
    #[cfg(windows)]
    {
        // K-244: gerçek Windows PowerShell provider davranışı aynı birincil vakaya aittir.
        let literal_rapor = gecici.join("kurulum-literal-yol.json");
        let literal = Command::new("powershell")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(depo().join("scripts/tests/kurulum-literal-yol-test.ps1"))
            .arg("-SourceRoot")
            .arg(depo())
            .arg("-Report")
            .arg(&literal_rapor)
            .output()
            .expect("Windows literal yol kampanyası çalışmalı");
        assert!(
            literal.status.success(),
            "{}",
            String::from_utf8_lossy(&literal.stderr)
        );
        let rapor: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&literal_rapor).unwrap()).unwrap();
        std::fs::remove_file(&literal_rapor).unwrap();
        assert_eq!(rapor["schema"], "zee-kurulum-literal-yol-1");
        assert_eq!(rapor["result"], "gecti");
        assert_eq!(rapor["case_count"], 6);
        assert_eq!(rapor["owned_cleanup"], true);
        assert_eq!(rapor["synthetic_binaries_executed"], false);
        let vakalar = rapor["cases"].as_array().unwrap();
        assert_eq!(vakalar.len(), 6);
        for (vaka, ad) in vakalar.iter().zip([
            "normal",
            "unicode",
            "bracket-sibling",
            "changed-hash-no-delete-all",
            "force-literal-only",
            "missing-and-schema-fail-closed",
        ]) {
            assert_eq!(vaka["name"], ad);
            assert_eq!(vaka["result"], "gecti");
        }
    }
    let dil_ad = format!("dil{}", std::env::consts::EXE_SUFFIX);
    let dillsp_ad = format!("dillsp{}", std::env::consts::EXE_SUFFIX);
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

#[test]
fn calisma_ortami_kaydi_exact_kaynak_ve_ikili_hashleriyle_program_lsp_kaniti_tasir() {
    use sha2::{Digest, Sha256};
    use std::collections::BTreeSet;
    let kayit = oku("docs/derleme-platform-kaniti-v1.tsv");
    assert_eq!(
        kayit.lines().next(),
        Some("# zee-derleme-platform-kaniti-1")
    );
    assert!(kayit.ends_with('\n') && !kayit.contains('\r'));
    let mut gorulen = BTreeSet::new();
    let mut platformlar = BTreeSet::new();
    for satir in kayit
        .lines()
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
    {
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [sha, os, run, job, artifact, zip_sha, rapor_sha, yol, sonuc] = alanlar.as_slice()
        else {
            panic!("platform kanıt satırı dokuz alan taşımalı");
        };
        for (deger, uzunluk) in [(sha, 40), (zip_sha, 64), (rapor_sha, 64)] {
            assert_eq!(deger.len(), uzunluk);
            assert!(deger
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
        }
        assert!(matches!(*os, "Linux" | "macOS" | "Windows"));
        assert!(gorulen.insert((sha.to_string(), os.to_string())));
        platformlar.insert(os.to_string());
        for kimlik in [run, job, artifact] {
            assert!(kimlik.parse::<u64>().unwrap() > 0);
        }
        assert_eq!(*sonuc, "gecti");
        assert_eq!(
            *yol,
            format!(
                "compiler/tests/fixtures/calisma-ortami-v1/{sha}/{}.json",
                os.to_lowercase()
            )
        );
        let baytlar = std::fs::read(depo().join(yol)).unwrap();
        let bulunan = Sha256::digest(&baytlar)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(
            bulunan, *rapor_sha,
            "kanıt raporu kayıtlı byte özetiyle eşleşmeli"
        );
        let rapor: serde_json::Value = serde_json::from_slice(&baytlar).unwrap();
        assert_eq!(rapor["sema"], "zee-calisma-ortami-1");
        assert_eq!(rapor["sonuc"], "gecti");
        assert_eq!(rapor["git_sha"], *sha);
        assert!(!rapor["os"].as_str().unwrap().is_empty());
        assert!(!rapor["mimari"].as_str().unwrap().is_empty());
        assert!(rapor["sinir"].as_str().unwrap().contains("temiz makine"));
        let prova = &rapor["prova"];
        assert_eq!(prova["rust_araclari"], "yok");
        assert_eq!(prova["program_ciktisi"], "Dünyaya merhaba\n");
        assert_eq!(prova["program_exit"], 0);
        assert_eq!(prova["lsp_initialize_shutdown_exit"], "gecti");
        let ikililer = rapor["ikililer"].as_object().unwrap();
        assert_eq!(ikililer.len(), 2);
        let provenance_yolu = depo()
            .join(yol)
            .with_file_name(format!("{}-provenance.json", os.to_lowercase()));
        if rapor.get("crt_sozlesmesi").is_some() {
            assert!(
                provenance_yolu.exists(),
                "CRT kanıtının ham provenance belgesi korunmalı"
            );
        }
        if provenance_yolu.exists() {
            let provenance: serde_json::Value =
                serde_json::from_slice(&std::fs::read(provenance_yolu).unwrap()).unwrap();
            let predicate = &provenance["predicate"];
            assert_eq!(predicate["zeeTekrarUretim"]["ikiTemizKlonEsit"], true);
            assert_eq!(
                predicate["buildDefinition"]["externalParameters"]["gitCommit"],
                *sha
            );
            if let Some(ortam) =
                predicate["buildDefinition"]["externalParameters"].get("buildEnvironment")
            {
                assert_eq!(ortam["schema"], "zee-build-environment-1");
                assert_eq!(ortam["gitCommit"], *sha);
                if let Some(deployment) = ortam.get("nativeDeploymentTarget") {
                    assert_eq!(*os, "macOS");
                    assert_eq!(deployment, "MACOSX_DEPLOYMENT_TARGET=11.0");
                    for klon in ["a", "b"] {
                        let native = &ortam["clones"][klon]["nativeDeployment"];
                        assert_eq!(native["schema"], "zee-native-deployment-1");
                        assert_eq!(native["target"], "11.0");
                        let nesneler = native["objects"].as_array().unwrap();
                        assert!(!nesneler.is_empty());
                        let mut bildirilen = 0;
                        for nesne in nesneler {
                            let yol = nesne["path"].as_str().unwrap();
                            assert!(yol.contains("build/") && yol.ends_with(".o"));
                            assert!(!yol.starts_with('/') && !yol.split('/').any(|p| p == ".."));
                            let hash = nesne["sha256"].as_str().unwrap();
                            assert_eq!(hash.len(), 64);
                            assert!(hash
                                .bytes()
                                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
                            for surum in nesne["deployment_versions"].as_array().unwrap() {
                                let mut parcalar = surum
                                    .as_str()
                                    .unwrap()
                                    .split('.')
                                    .map(|p| p.parse::<u32>().unwrap())
                                    .collect::<Vec<_>>();
                                parcalar.resize(3, 0);
                                assert!(parcalar.as_slice() <= [11, 0, 0].as_slice());
                                assert!(nesne["header"]
                                    .as_str()
                                    .unwrap()
                                    .contains(surum.as_str().unwrap()));
                                bildirilen += 1;
                            }
                        }
                        assert!(bildirilen > 0);
                    }
                }
                let klonlar = ortam["clones"].as_object().unwrap();
                assert_eq!(klonlar.len(), 2);
                for klon in ["a", "b"] {
                    let kayit = &klonlar[klon];
                    if let Some(native) = kayit.get("nativeTools") {
                        assert_eq!(native["schema"], "zee-native-tools-1");
                        let tools = native["tools"].as_array().unwrap();
                        assert!(!tools.is_empty());
                        let mut paths = BTreeSet::new();
                        for tool in tools {
                            let path = tool["path"].as_str().unwrap();
                            assert!(!path.is_empty() && paths.insert(path));
                            let hash = tool["sha256"].as_str().unwrap();
                            assert_eq!(hash.len(), 64);
                            assert!(hash
                                .bytes()
                                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
                            let query = &tool["versionQuery"];
                            match query["status"].as_str().unwrap() {
                                "available" => {
                                    assert!(!query["value"].as_str().unwrap().is_empty())
                                }
                                "unavailable" => {
                                    assert!(!query["reason"].as_str().unwrap().is_empty());
                                    assert!(query.get("value").is_none());
                                }
                                status => panic!("bilinmeyen native sürüm sorgusu: {status}"),
                            }
                        }
                        let calls = native["invocations"].as_array().unwrap();
                        let mut roles = BTreeSet::new();
                        for call in calls {
                            assert!(paths.contains(call["tool"].as_str().unwrap()));
                            for field in ["owner", "program", "command"] {
                                assert!(!call[field].as_str().unwrap().is_empty());
                            }
                            assert!(call["arguments"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .all(|a| a.is_string()));
                            roles.insert(call["role"].as_str().unwrap());
                        }
                        assert!(roles.contains("compile") && roles.contains("archive"));
                    }
                    let araclar = kayit["binaries"].as_object().unwrap();
                    assert_eq!(araclar.len(), 2);
                    for ad in ["dil", "dillsp"] {
                        let arac = &araclar[ad];
                        for alan in ["program", "path", "version", "command"] {
                            assert!(!arac[alan].as_str().unwrap().is_empty());
                        }
                        let ozet = arac["sha256"].as_str().unwrap();
                        assert_eq!(ozet.len(), 64);
                        assert!(ozet
                            .bytes()
                            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
                    }
                    let sdk = kayit["sdkVersions"].as_object().unwrap();
                    if *os != "Linux" {
                        assert!(!sdk.is_empty(), "Apple/Windows SDK sürüm kanıtı gerekli");
                        assert!(sdk.values().all(|v| !v.as_str().unwrap().is_empty()));
                    }
                }
            }
            let subject = provenance["subject"].as_array().unwrap();
            assert_eq!(subject.len(), 2);
            let mut subject_adlari = BTreeSet::new();
            for ikili in subject {
                let ad = ikili["name"].as_str().unwrap();
                assert!(subject_adlari.insert(ad));
                assert_eq!(ikili["digest"]["sha256"], ikililer[ad]["sha256"]);
            }
            assert_eq!(
                subject_adlari,
                ikililer.keys().map(String::as_str).collect()
            );
            let ozetler = predicate["runDetails"]["byproducts"].as_array().unwrap();
            assert_eq!(ozetler.len(), 2);
            let kanonik_ozet = ikililer
                .iter()
                .map(|(ad, ikili)| format!("{}  {ad}\n", ikili["sha256"].as_str().unwrap()))
                .collect::<String>();
            let manifest_sha = Sha256::digest(kanonik_ozet.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            assert_eq!(ozetler[0]["digest"]["sha256"], manifest_sha);
            assert_eq!(
                ozetler[0]["digest"]["sha256"],
                ozetler[1]["digest"]["sha256"]
            );
            if *os == "Windows" {
                assert_eq!(
                    predicate["buildDefinition"]["externalParameters"]["crtLinkage"],
                    "static"
                );
                assert_eq!(rapor["crt_sozlesmesi"], "statik-crt-import-yok");
            }
        }
        let uzanti = if *os == "Windows" { ".exe" } else { "" };
        for ad in ["dil", "dillsp"] {
            let ikili = &ikililer[&format!("{ad}{uzanti}")];
            let hash = ikili["sha256"].as_str().unwrap();
            assert_eq!(hash.len(), 64);
            assert!(hash
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
            let kutuphaneler = ikili["dogrudan"].as_array().unwrap();
            assert!(!kutuphaneler.is_empty());
            assert!(kutuphaneler.iter().all(|v| !v.as_str().unwrap().is_empty()));
        }
    }
    assert_eq!(
        platformlar,
        BTreeSet::from(["Linux".into(), "macOS".into(), "Windows".into()])
    );
    let temiz = oku("docs/temiz-windows-platform-kaniti-v1.tsv");
    assert_eq!(
        temiz.lines().next(),
        Some("# zee-temiz-windows-platform-kaniti-1")
    );
    for satir in temiz
        .lines()
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
    {
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [release_sha, prova_sha, source_run, prova_run, job, artifact, zip_sha, rapor_sha, yol, sonuc] =
            alanlar.as_slice()
        else {
            panic!("temiz Windows kanıtı on alan taşımalı");
        };
        for (deger, uzunluk) in [
            (release_sha, 40),
            (prova_sha, 40),
            (zip_sha, 64),
            (rapor_sha, 64),
        ] {
            assert_eq!(deger.len(), uzunluk);
            assert!(deger
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
        }
        for kimlik in [source_run, prova_run, job, artifact] {
            assert!(kimlik.parse::<u64>().unwrap() > 0);
        }
        assert_eq!(*sonuc, "gecti");
        let baytlar = std::fs::read(depo().join(yol)).unwrap();
        let ozet = Sha256::digest(&baytlar)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(ozet, *rapor_sha);
        let rapor: serde_json::Value = serde_json::from_slice(&baytlar).unwrap();
        let kurulum_provasi = rapor["sema"] == "zee-temiz-windows-kurulum-1";
        assert!(kurulum_provasi || rapor["sema"] == "zee-temiz-windows-runtime-1");
        assert_eq!(rapor["sonuc"], "gecti");
        assert_eq!(rapor["git_sha"], *release_sha);
        assert_eq!(rapor["prova_git_sha"], *prova_sha);
        assert_eq!(rapor["program_ciktisi"], "Dünyaya merhaba\n");
        assert_eq!(rapor["program_exit"], 0);
        assert_eq!(rapor["rust_araclari"], "yok");
        assert_eq!(rapor["lsp_initialize_shutdown_exit"], "gecti");
        let image = rapor["base"].as_str().unwrap();
        assert!(image.starts_with(if kurulum_provasi {
            "mcr.microsoft.com/windows/servercore@sha256:"
        } else {
            "mcr.microsoft.com/windows/nanoserver@sha256:"
        }));
        assert_eq!(image.rsplit(':').next().unwrap().len(), 64);
        assert!(rapor["repo_digests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == image));
        let bagli = rapor["container_arguments"].as_array().unwrap();
        for deger in [
            "--isolation",
            "process",
            "--network",
            "none",
            "--user",
            "ContainerUser",
        ] {
            assert!(bagli.iter().any(|v| v == deger));
        }
        assert!(bagli
            .iter()
            .any(|v| v.as_str().unwrap().ends_with(",readonly")));
        let release: serde_json::Value = serde_json::from_slice(
            &std::fs::read(depo().join(format!(
                "compiler/tests/fixtures/calisma-ortami-v1/{release_sha}/windows.json"
            )))
            .unwrap(),
        )
        .unwrap();
        for ad in ["dil.exe", "dillsp.exe"] {
            assert_eq!(rapor["ikililer"][ad], release["ikililer"][ad]["sha256"]);
        }
        if kurulum_provasi {
            for alan in ["kurulum", "kaldirma", "container_temizligi"] {
                assert_eq!(rapor[alan], "gecti");
            }
            let manifest = rapor["manifest"].as_str().unwrap();
            assert!(manifest.starts_with("# zee-kurulum-1\n"));
            let kayitlar = manifest
                .lines()
                .filter(|s| !s.is_empty() && !s.starts_with('#'))
                .map(|s| s.split('\t').collect::<Vec<_>>())
                .collect::<Vec<_>>();
            assert_eq!(kayitlar.len(), 2);
            for (kayit, ad) in kayitlar.iter().zip(["dil.exe", "dillsp.exe"]) {
                assert_eq!(kayit.len(), 3);
                assert!(kayit[0].contains("Zee Türkçe\\bin\\"));
                assert!(kayit[0].ends_with(ad));
                assert_eq!(kayit[1], rapor["ikililer"][ad].as_str().unwrap());
                assert_eq!(kayit[2], *release_sha);
            }
            for ad in ["kur.ps1", "kaldir.ps1"] {
                let hash = rapor["kurulum_betikleri"][ad].as_str().unwrap();
                assert_eq!(hash.len(), 64);
                assert!(hash
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
            }
        }
        let cevaplar = rapor["lsp_cevaplari"].as_array().unwrap();
        assert_eq!(cevaplar.len(), 2);
        assert_eq!(cevaplar[0]["id"], 1);
        assert!(cevaplar[0]["result"]["capabilities"].is_object());
        assert_eq!(cevaplar[1]["id"], 2);
        assert!(cevaplar[1]["result"].is_null());
    }
}
