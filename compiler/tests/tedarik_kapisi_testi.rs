use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn depo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü")
        .to_path_buf()
}

fn oku(goreli: &str) -> String {
    std::fs::read_to_string(depo().join(goreli)).expect("depo dosyası okunmalı")
}

fn public_runner_gecerli(satir: &str) -> bool {
    matches!(
        satir.trim().strip_prefix("runs-on: "),
        Some("ubuntu-latest" | "windows-latest" | "macos-latest" | "${{ matrix.os }}")
    )
}

#[test]
fn public_github_akislari_kurum_kosucusuna_yonlenemez() {
    for hatali in [
        "runs-on: self-hosted",
        "runs-on: [self-hosted, linux]",
        "runs-on: ${{ vars.CI_RUNNER }}",
        "runs-on: ${{ fromJSON(vars.CI_RUNNER || '\"ubuntu-latest\"') }}",
    ] {
        assert!(!public_runner_gecerli(hatali));
    }
    for dosya in std::fs::read_dir(depo().join(".github/workflows")).unwrap() {
        let yol = dosya.unwrap().path();
        if yol.extension().is_some_and(|e| e == "yml") {
            let metin = std::fs::read_to_string(&yol).unwrap();
            assert!(!metin.contains("CI_RUNNER"), "{}", yol.display());
            for satir in metin.lines().filter(|s| s.trim().starts_with("runs-on:")) {
                assert!(public_runner_gecerli(satir), "{}: {satir}", yol.display());
            }
        }
    }
}

#[test]
#[cfg(unix)]
fn forgejo_kapisi_ozel_exact_metadata_ister_public_ve_bozuk_veriyi_reddeder() {
    let ci = oku(".forgejo/workflows/zee-ci.yml");
    let bolum = ci
        .split_once("      - name: Yalnız özel Git2 deposu\n")
        .unwrap()
        .1
        .split("      - name:")
        .next()
        .unwrap();
    let kod = bolum
        .split_once("          node <<'NODE'\n")
        .unwrap()
        .1
        .split("          NODE\n")
        .next()
        .unwrap()
        .lines()
        .map(|s| s.strip_prefix("          ").unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        ci.find("Yalnız özel Git2 deposu").unwrap()
            < ci.find("Tam ve yalnız seçili kaynak").unwrap()
    );
    for (metadata, http_ok, izinli) in [
        (r#"{"private":true,"full_name":"github/zee"}"#, true, true),
        (r#"{"private":false,"full_name":"github/zee"}"#, true, false),
        (
            r#"{"private":"true","full_name":"github/zee"}"#,
            true,
            false,
        ),
        (
            r#"{"private":true,"full_name":"github/baska"}"#,
            true,
            false,
        ),
        (r#"{}"#, true, false),
        (r#"{"private":true,"full_name":"github/zee"}"#, false, false),
    ] {
        let program = format!(
            "globalThis.fetch = async (url, options) => {{\n\
             if (url !== 'https://ci.example/api/v1/repos/github/zee' || \
             options.headers.Authorization !== 'Basic ' + \
             Buffer.from('x-access-token:sentetik').toString('base64')) throw new Error('istek');\n\
             return {{ok: {http_ok}, json: async () => ({metadata})}};\n\
             }};\n{kod}"
        );
        let sonuc = std::process::Command::new("node")
            .args(["-e", &program])
            .env("ZEE_SERVER", "https://ci.example")
            .env("ZEE_REPOSITORY", "github/zee")
            .env("ZEE_JOB_TOKEN", "sentetik")
            .output()
            .expect("Node guard gerçek yürütmesi");
        assert_eq!(sonuc.status.success(), izinli, "metadata: {metadata}");
    }
}

fn github_action_pinleri() -> BTreeMap<String, String> {
    let kayit = oku("docs/github-actions-pinleri-v1.tsv");
    assert_eq!(kayit.lines().next(), Some("# zee-github-actions-pinleri-1"));
    assert!(kayit.ends_with('\n') && !kayit.contains('\r'));
    let mut pinler = BTreeMap::new();
    for satir in kayit
        .lines()
        .filter(|satir| !satir.is_empty() && !satir.starts_with('#'))
    {
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [action, surum, commit, kaynak] = alanlar.as_slice() else {
            panic!("action pin satırı dört alan taşımalı: {satir}");
        };
        assert!(!surum.is_empty(), "{action}: sürüm etiketi boş olamaz");
        assert_eq!(
            commit.len(),
            40,
            "{action}: commit SHA-1 uzunluğunda olmalı"
        );
        assert!(
            commit
                .bytes()
                .all(|bayt| bayt.is_ascii_hexdigit() && !bayt.is_ascii_uppercase()),
            "{action}: commit küçük hex olmalı"
        );
        assert_eq!(
            *kaynak,
            format!("https://github.com/{action}"),
            "{action}: kaynak resmî action deposu olmalı"
        );
        assert!(
            pinler
                .insert((*action).to_string(), (*commit).to_string())
                .is_none(),
            "yinelenen action kaydı: {action}"
        );
    }
    pinler
}

#[test]
fn forgejo_akisi_exact_pin_ve_prsiz_kaynak_siniri_tasir() {
    let ci = oku(".forgejo/workflows/zee-ci.yml");
    let envanter = oku("docs/forgejo-actions-pinleri-v1.tsv");
    assert_eq!(
        envanter.lines().next(),
        Some("# zee-forgejo-actions-pinleri-1")
    );
    let mut pinler = BTreeMap::new();
    for satir in envanter
        .lines()
        .filter(|s| !s.starts_with('#') && !s.is_empty())
    {
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [action, surum, sha, kaynak] = alanlar.as_slice() else {
            panic!("Forgejo pin kaydı dört alan ister");
        };
        assert!(!surum.is_empty());
        assert_eq!(action, kaynak);
        assert!(action.starts_with("https://"));
        assert_eq!(sha.len(), 40);
        assert!(sha
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
        assert!(pinler.insert(*action, *sha).is_none());
    }
    let mut gorulen = BTreeMap::new();
    for satir in ci.lines() {
        if let Some(kullanim) = satir.trim().strip_prefix("uses: ") {
            let (action, sha) = kullanim.split_once('@').expect("exact pin");
            assert_eq!(pinler.get(action).copied(), Some(sha));
            gorulen.insert(action, sha);
        }
    }
    assert_eq!(gorulen, pinler);
    assert!(!ci.contains("pull_request"));
    assert!(ci.contains("branches: [master, ci/zee-gitsrv]"));
    assert!(ci.contains("container: golang:1.26.1-bookworm@sha256:ab3d6955bbc813a0f3fdf220c1d817dd89c0b3f283777db8ece4a32fe7858edd"));
    assert!(ci.contains("if: ${{ forgejo.event.repository.private == true }}"));
    assert!(ci.contains("test \"$(go env GOVERSION)\" = go1.26.1"));
    assert!(ci.contains("retention-days: 90"));
    for komut in [
        "scripts/core-freeze-korugu.sh",
        "scripts/semantic-regresyon-korugu.sh",
        "scripts/conformance-korugu.sh",
        "scripts/acik-depo-denetle.py",
        "scripts/sir-taramasi.sh",
        "scripts/guvenlik-kapisi.sh --surekli",
        "scripts/offline-vendor-denetle.sh",
        "--bin faz_test_matrisi",
        "cargo clippy --locked --all-targets -- -D warnings",
        "cargo fmt --all -- --check",
        "scripts/wasm-abi-denetle.mjs",
    ] {
        assert!(ci.contains(komut), "Forgejo kapısı eksik: {komut}");
    }
}

#[test]
#[cfg(unix)]
fn forgejo_fetch_tam_secilen_gecmisi_alir_ozel_dal_ve_kimlik_almaz() {
    use std::process::Command;
    let ci = oku(".forgejo/workflows/zee-ci.yml");
    let bolum = ci
        .split_once("      - name: Tam ve yalnız seçili kaynak geçmişi\n")
        .unwrap()
        .1;
    let bolum = bolum.split("      - name:").next().unwrap();
    let betik = bolum
        .split_once("        run: |\n")
        .unwrap()
        .1
        .lines()
        .map(|s| s.strip_prefix("          ").unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kok = std::env::temp_dir().join(format!(
        "zee-forgejo-fetch-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let kaynak = kok.join("depo");
    std::fs::create_dir_all(&kaynak).unwrap();
    let git = |yol: &Path, args: &[&str]| {
        let out = Command::new("git")
            .current_dir(yol)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };
    git(&kaynak, &["init", "-q", "--initial-branch=ci/zee-gitsrv"]);
    git(&kaynak, &["config", "user.name", "Sentetik CI"]);
    git(&kaynak, &["config", "user.email", "ci@ornek.com"]);
    std::fs::write(kaynak.join("kayit"), "anonim kaynak\n").unwrap();
    git(&kaynak, &["add", "."]);
    git(&kaynak, &["commit", "-qm", "ilk kaynak"]);
    std::fs::write(kaynak.join("kayit"), "anonim tam kaynak\n").unwrap();
    git(&kaynak, &["commit", "-qam", "ikinci kaynak"]);
    let secilen = git(&kaynak, &["rev-parse", "HEAD"]);
    git(&kaynak, &["checkout", "-qb", "ilgisiz-ozel"]);
    std::fs::write(kaynak.join("ozel"), "yalniz sentetik ayri dal verisi\n").unwrap();
    git(&kaynak, &["add", "."]);
    git(&kaynak, &["commit", "-qm", "ilgisiz ozel kaynak"]);
    git(&kaynak, &["tag", "ilgisiz-etiket"]);
    let ilgisiz = git(&kaynak, &["rev-parse", "HEAD"]);
    let calistir = |ad: &str, sha: &str, dolu: bool| {
        let yol = kok.join(ad);
        std::fs::create_dir(&yol).unwrap();
        if dolu {
            std::fs::create_dir(yol.join("kaynak")).unwrap();
        }
        let out = Command::new("bash")
            .current_dir(&yol)
            .arg("-c")
            .arg(&betik)
            .env("ZEE_SERVER", &kok)
            .env("ZEE_REPOSITORY", "depo")
            .env("ZEE_REF", "refs/heads/ci/zee-gitsrv")
            .env("ZEE_SHA", sha)
            .env("ZEE_JOB_TOKEN", "yalniz-sentetik-token")
            .output()
            .unwrap();
        (yol.join("kaynak"), out)
    };
    let (klon, out) = calistir("olumlu", &secilen, false);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(git(&klon, &["rev-parse", "HEAD"]), secilen);
    assert_eq!(git(&klon, &["rev-list", "--count", "HEAD"]), "2");
    assert_eq!(
        git(&klon, &["rev-parse", "--is-shallow-repository"]),
        "false"
    );
    assert!(git(&klon, &["for-each-ref", "refs/tags", "refs/remotes"]).is_empty());
    assert!(!Command::new("git")
        .current_dir(&klon)
        .args(["cat-file", "-e", &ilgisiz])
        .output()
        .unwrap()
        .status
        .success());
    let config = std::fs::read_to_string(klon.join(".git/config")).unwrap();
    assert!(!config.contains("AUTHORIZATION") && !config.contains("yalniz-sentetik-token"));
    assert!(!calistir("yanlis-sha", &ilgisiz, false).1.status.success());
    assert!(!calistir("dolu-dizin", &secilen, true).1.status.success());
    std::fs::remove_dir_all(kok).unwrap();
}

#[test]
fn registry_lock_girdilerinin_sha256_ozeti_zorunludur() {
    for yol in ["compiler/Cargo.lock", "compiler/fuzz/Cargo.lock"] {
        let kilit = oku(yol);
        assert!(kilit.starts_with("# This file is automatically @generated by Cargo.\n"));
        assert!(kilit.contains("\nversion = 4\n"), "{yol}");

        let mut registry_paketi = 0;
        for paket in kilit.split("[[package]]").skip(1) {
            if !paket.contains("source = \"registry+") {
                continue;
            }
            registry_paketi += 1;
            assert!(
                paket
                    .contains("source = \"registry+https://github.com/rust-lang/crates.io-index\""),
                "{yol}: yalnız crates.io registry kaynağı kabul edilir: {paket}"
            );
            let ozet = paket
                .lines()
                .find_map(|satir| {
                    satir
                        .strip_prefix("checksum = \"")
                        .and_then(|s| s.strip_suffix('"'))
                })
                .expect("registry paketi checksum taşımalı");
            assert_eq!(ozet.len(), 64, "checksum SHA-256 uzunluğunda olmalı");
            assert!(
                ozet.bytes()
                    .all(|bayt| bayt.is_ascii_hexdigit() && !bayt.is_ascii_uppercase()),
                "checksum küçük hex olmalı"
            );
        }
        assert!(
            registry_paketi > 50,
            "{yol}: beklenmedik ölçüde küçük registry grafiği"
        );
    }
}

#[test]
fn cargo_deny_politikasi_fail_closed_ve_istisnalari_gerekcelidir() {
    let manifest = oku("compiler/Cargo.toml");
    let fuzz_manifesti = oku("compiler/fuzz/Cargo.toml");
    let politika = oku("compiler/deny.toml");
    assert!(manifest.contains("publish = false"));
    assert!(fuzz_manifesti.contains("publish = false"));
    assert!(fuzz_manifesti.contains(&format!("version = \"={}\"", env!("CARGO_PKG_VERSION"))));
    assert!(politika.contains("unknown-registry = \"deny\""));
    assert!(politika.contains("unknown-git = \"deny\""));
    assert!(politika.contains("multiple-versions = \"deny\""));
    assert!(politika.contains("wildcards = \"deny\""));
    assert!(politika.contains("ignore = []"));
    for satir in politika.lines().filter(|satir| satir.contains("crate = ")) {
        assert!(satir.contains("reason = "), "istisna gerekçesiz: {satir}");
    }
}

#[test]
fn ci_sabit_aracla_guncel_advisory_ve_offline_vendor_kapisi_kosar() {
    let ci = oku(".github/workflows/tedarik.yml");
    assert!(ci.contains("cargo install cargo-deny --version 0.20.2 --locked"));
    assert!(ci.contains("actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1"));
    assert!(ci.contains("cargo deny --locked check -D warnings"));
    assert!(ci.contains("--manifest-path fuzz/Cargo.toml --config deny.toml"));
    assert!(ci.contains("schedule:"));
    assert!(ci.contains("bash scripts/offline-vendor-denetle.sh"));
    assert!(ci.contains("bash scripts/tests/rust-arac-zinciri-test.sh"));

    let ana_ci = oku(".github/workflows/ci.yml");
    assert!(!ana_ci.contains("cargo test --locked || cargo test"));
    for komut in [
        "cargo test",
        "cargo build",
        "cargo clippy",
        "cargo check",
        "cargo run",
        "cargo rustc",
    ] {
        for satir in ana_ci.lines().filter(|satir| satir.contains(komut)) {
            assert!(satir.contains("--locked"), "kilitsiz CI komutu: {satir}");
        }
    }
}

#[test]
fn github_actionlari_incelenmis_immutable_commitlere_sabitlidir() {
    let pinler = github_action_pinleri();
    let workflow_dizini = depo().join(".github/workflows");
    let mut dosyalar = std::fs::read_dir(workflow_dizini)
        .expect("workflow dizini okunmalı")
        .map(|girdi| girdi.expect("workflow girdisi okunmalı").path())
        .filter(|yol| {
            matches!(
                yol.extension().and_then(|uzanti| uzanti.to_str()),
                Some("yml" | "yaml")
            )
        })
        .collect::<Vec<_>>();
    dosyalar.sort();
    assert!(!dosyalar.is_empty(), "en az bir workflow olmalı");

    let mut gorulen = BTreeMap::<String, String>::new();
    for yol in dosyalar {
        let icerik = std::fs::read_to_string(&yol).expect("workflow okunmalı");
        for satir in icerik.lines() {
            let kirpilmis = satir.trim_start().trim_start_matches("- ");
            let Some(kullanim) = kirpilmis.strip_prefix("uses:") else {
                continue;
            };
            let kullanim = kullanim
                .split_whitespace()
                .next()
                .expect("uses değeri olmalı")
                .trim_matches(['\'', '"']);
            if kullanim.starts_with("./") {
                continue;
            }
            let (action, commit) = kullanim
                .split_once('@')
                .unwrap_or_else(|| panic!("{}: action ref'i eksik: {satir}", yol.display()));
            assert_eq!(
                commit.len(),
                40,
                "{}: hareketli action ref'i yasak: {kullanim}",
                yol.display()
            );
            assert!(
                commit
                    .bytes()
                    .all(|bayt| bayt.is_ascii_hexdigit() && !bayt.is_ascii_uppercase()),
                "{}: action commit'i küçük hex olmalı: {kullanim}",
                yol.display()
            );
            assert_eq!(
                pinler.get(action).map(String::as_str),
                Some(commit),
                "{}: {action} incelenmiş pin kaydından farklı",
                yol.display()
            );
            if let Some(eski) = gorulen.insert(action.to_string(), commit.to_string()) {
                assert_eq!(eski, commit, "{action} workflow'larda farklı pinlenmiş");
            }
        }
        let checkout = icerik.matches("actions/checkout@").count();
        if checkout > 0 {
            assert!(
                icerik.matches("persist-credentials: false").count() >= checkout,
                "{}: checkout kalıcı GitHub kimliği bırakmamalı",
                yol.display()
            );
        }
    }
    assert_eq!(
        gorulen, pinler,
        "pin kaydı workflow kullanımıyla birebir olmalı"
    );

    let dependabot = oku(".github/dependabot.yml");
    for soz in [
        "package-ecosystem: \"github-actions\"",
        "directory: \"/\"",
        "interval: \"weekly\"",
        "timezone: \"Europe/Istanbul\"",
    ] {
        assert!(dependabot.contains(soz), "Dependabot ayarı eksik: {soz}");
    }
}

#[test]
fn vendor_kaniti_iki_uretimi_ve_bos_cargo_home_offline_derlemeyi_ister() {
    let betik = oku("scripts/offline-vendor-denetle.sh");
    assert!(betik.contains("zee_rust_arac_zinciri"));
    assert!(betik.contains("cargo vendor --locked --versioned-dirs"));
    assert!(betik.contains("--sync \"$fuzz_manifesti\""));
    assert!(betik.contains("for tur in bir iki"));
    assert!(betik.contains("cmp -s"));
    assert!(betik.contains("CARGO_HOME="));
    assert!(betik.contains("CARGO_NET_OFFLINE=true"));
    assert!(betik.contains("cargo check --quiet --locked --offline --all-targets --all-features"));
    assert!(betik.contains("cargo check --quiet --locked --offline --bins"));
    assert!(oku(".gitignore").lines().any(|satir| satir == "**/vendor/"));
    #[cfg(unix)]
    {
        let sonuc = std::process::Command::new("bash")
            .arg(depo().join("scripts/tests/rust-arac-zinciri-test.sh"))
            .current_dir(depo())
            .output()
            .expect("araç zinciri giriş regresyonu çalışmalı");
        assert!(
            sonuc.status.success(),
            "exact compiler/config/wrapper girişleri başarısız: {}{}",
            String::from_utf8_lossy(&sonuc.stdout),
            String::from_utf8_lossy(&sonuc.stderr)
        );
    }
    let pin_kaydi = oku("docs/rustc-pin-platform-kaniti-v1.tsv");
    assert_eq!(
        pin_kaydi.lines().next(),
        Some("# zee-rustc-pin-platform-kaniti-1")
    );
    let mut platformlar = std::collections::BTreeSet::new();
    for satir in pin_kaydi
        .lines()
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
    {
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [sha, os, run, job, artifact, zip_sha, rapor_sha, yol, sonuc] = alanlar.as_slice()
        else {
            panic!("compiler pin kanıtı dokuz alan taşımalı");
        };
        assert!(platformlar.insert(*os));
        assert_eq!(*sonuc, "gecti");
        for (deger, uzunluk) in [(sha, 40), (zip_sha, 64), (rapor_sha, 64)] {
            assert_eq!(deger.len(), uzunluk);
            assert!(deger
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
        }
        for kimlik in [run, job, artifact] {
            assert!(kimlik.parse::<u64>().unwrap() > 0);
        }
        let baytlar = std::fs::read(depo().join(yol)).unwrap();
        use sha2::Digest;
        let ozet = sha2::Sha256::digest(&baytlar)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(ozet, *rapor_sha);
        let rapor: serde_json::Value = serde_json::from_slice(&baytlar).unwrap();
        assert_eq!(rapor["sema"], "zee-rustc-pin-prova-1");
        assert_eq!(rapor["prova_git_sha"], *sha);
        assert_eq!(rapor["sonuc"], "gecti");
        assert_eq!(rapor["calisma_agaci_temiz"], true);
        assert_eq!(rapor["compiler_config_poisoned"], true);
        assert!(rapor["rustc_version"]
            .as_str()
            .unwrap()
            .starts_with("rustc 1.93.1 "));
        assert!(rapor["cargo_version"]
            .as_str()
            .unwrap()
            .starts_with("cargo 1.93.1 "));
        assert_eq!(rapor["wrapper"], "");
        assert_eq!(rapor["workspace_wrapper"], "");
        assert_eq!(
            rapor["rustc_path"].as_str().unwrap().replace('\\', "/"),
            rapor["compiler_env"].as_str().unwrap().replace('\\', "/")
        );
        for alan in [
            "rustc_sha256",
            "pin_sha256",
            "prova_araci_sha256",
            "helper_sha256",
        ] {
            let hash = rapor[alan].as_str().unwrap();
            assert_eq!(hash.len(), 64);
            assert!(hash
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
        }
        assert!(rapor["build_log"]
            .as_str()
            .unwrap()
            .contains("zee_exact_pin_probe"));
    }
    assert_eq!(
        platformlar,
        std::collections::BTreeSet::from(["Linux", "macOS", "Windows"])
    );
}

#[test]
fn surum_artefakti_iki_temiz_klon_sbom_provenance_ve_imza_kablosunu_tasir() {
    let betik = oku("scripts/surum-artefakti.sh");
    for adim in [
        "set -euo pipefail",
        "zee_rust_arac_zinciri",
        "git show \"$sha:compiler/rust-toolchain.toml\"",
        "git clone -q --local",
        "for klon in a b",
        "klon_temizligini_denetle",
        "git -C \"$1\" rev-parse HEAD",
        "--porcelain --untracked-files=all -- .",
        "[[ \"$klon_sha\" != \"$sha\" ]]",
        "$gecici/evidence/$klon/build.log",
        "$gecici/evidence/$klon/native-deployment.json",
        "ikili_kimligini_denetle",
        "subprocess.run([sys.argv[1], \"sürüm\", \"--json\"]",
        "identity.get(\"source_sha\") != sys.argv[2]",
        "identity.get(\"dirty\") is not False",
        "--remap-path-prefix=",
        "SOURCE_DATE_EPOCH",
        "build_argumanlari=(--locked --release --bin dil --bin dillsp)",
        "cargo build -vv \"${build_argumanlari[@]}\"",
        "build_argumanlari+=(--target \"$platform\")",
        "-C target-feature=+crt-static",
        "CARGO_ENCODED_RUSTFLAGS=",
        "RUSTC_LOG=rustc_codegen_ssa::back::link=info",
        "CC_ENABLE_DEBUG_OUTPUT=1",
        "derleme-ortami-kaydet.py",
        "--build-environment",
        "TEKRAR ÜRETİLEMEDİ",
        "surum_artefakti",
        "sbom.spdx.json",
        "provenance.intoto.json",
        "SHA256SUMS.zee-imza",
    ] {
        assert!(betik.contains(adim), "sürüm betiği adımı eksik: {adim}");
    }
    assert!(!betik.contains("$gecici/$klon/build.log"));
    assert!(!betik.contains("$gecici/$klon/native-deployment.json"));
    let workflow = oku(".github/workflows/surum-adayi.yml");
    assert!(workflow.contains("tags:\n      - \"v*\""));
    assert!(workflow.contains("workflow_dispatch:"));
    assert!(workflow.contains("bash scripts/guvenlik-kapisi.sh --surekli"));
    assert!(workflow.contains("bash scripts/surum-artefakti.sh --cikti"));
    assert!(workflow.contains("retention-days: 90"));
    assert!(workflow.contains("persist-credentials: false"));
    for satir in workflow.lines().filter(|s| s.contains("cargo ")) {
        assert!(satir.contains("--locked"), "kilitsiz sürüm komutu: {satir}");
    }
    let arac = oku("compiler/src/bin/surum_artefakti.rs");
    for parca in [
        "zee-surum-imza-v1",
        "https://spdx.org/rdf/3.0.1/spdx-context.jsonld",
        "https://slsa.dev/provenance/v1",
        "ikiTemizKlonEsit",
        "zee-ed25519-private-v1",
    ] {
        assert!(
            arac.contains(parca),
            "sürüm aracı sözleşmesi eksik: {parca}"
        );
    }
}

#[cfg(unix)]
#[test]
fn surum_klon_koku_sembolik_yoldan_kanoniklesir() {
    let kok = std::env::temp_dir().join(format!("zee-canonical-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&kok);
    std::fs::create_dir_all(kok.join("real")).unwrap();
    std::os::unix::fs::symlink(kok.join("real"), kok.join("alias")).unwrap();
    let betik = oku("scripts/surum-artefakti.sh");
    let adim = betik
        .lines()
        .find(|line| line.starts_with("gecici=\"$(cd "))
        .expect("remap klon kökü Cargo gibi fiziksel yola açılmalı");
    let cikti = std::process::Command::new("bash")
        .args([
            "-c",
            &format!("gecici=\"$1\"\n{adim}\nprintf '%s' \"$gecici\""),
            "zee-test",
        ])
        .arg(kok.join("alias"))
        .output()
        .unwrap();
    assert!(cikti.status.success());
    assert_eq!(
        std::path::Path::new(std::str::from_utf8(&cikti.stdout).unwrap()),
        std::fs::canonicalize(kok.join("real")).unwrap()
    );
    assert_ne!(
        kok.join("alias"),
        std::fs::canonicalize(kok.join("real")).unwrap()
    );
    std::fs::remove_dir_all(kok).unwrap();
}
