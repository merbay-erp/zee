//! K-165/ADR-073: aynı aracın Zee, Rust ve Go gerçeklemeleri aynı sayfayı
//! bayt bayt üretir; karşılaştırma tarihçesi exact SHA ve tekrar hesaplanabilir
//! ölçülerle (kaynak satırı, test sayısı) tutarlıdır. Süre/RSS ölçümleri makineye
//! bağlıdır, kapı değildir.

use std::path::{Path, PathBuf};
use std::process::Command;

fn depo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü")
        .to_path_buf()
}

fn oku(goreli: &str) -> String {
    std::fs::read_to_string(depo().join(goreli))
        .unwrap_or_else(|hata| panic!("{goreli} okunmalı: {hata}"))
}

fn loc(dosyalar: &[&str], yorum: &str) -> usize {
    dosyalar
        .iter()
        .map(|d| {
            oku(d)
                .lines()
                .filter(|s| {
                    let t = s.trim_start();
                    !t.is_empty() && !t.starts_with(yorum)
                })
                .count()
        })
        .sum()
}

fn zee_kaynaklari() -> Vec<String> {
    let mut yollar = std::fs::read_dir(depo().join("dogfood/kanit-ozeti/kaynak"))
        .expect("kaynak klasörü")
        .map(|g| g.expect("girdi").path())
        .filter(|y| y.extension().and_then(|u| u.to_str()) == Some("dil"))
        .map(|y| {
            y.strip_prefix(depo())
                .expect("depo içi")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<Vec<_>>();
    yollar.sort();
    yollar
}

#[test]
fn rust_esdegeri_zee_sayfasini_bayt_bayt_uretir() {
    let manifest = depo().join("dogfood/kanit-ozeti-karsilastirma/rust/Cargo.toml");
    let derleme = Command::new(env!("CARGO"))
        .args([
            "build",
            "--locked",
            "--release",
            "--quiet",
            "--manifest-path",
        ])
        .arg(&manifest)
        .status()
        .expect("cargo çalışmalı");
    assert!(derleme.success(), "Rust eşdeğeri derlenmeli");
    let ikili = depo().join("dogfood/kanit-ozeti-karsilastirma/rust/target/release/kanit_ozeti_rs");
    let cikti = Command::new(&ikili)
        .arg(depo())
        .output()
        .expect("Rust eşdeğeri çalışmalı");
    assert!(cikti.status.success());
    assert_eq!(
        String::from_utf8(cikti.stdout).expect("UTF-8"),
        oku("docs/kanit-ozeti.md"),
        "Rust eşdeğeri Zee ürününün sayfasından sapıyor"
    );
}

#[test]
fn go_esdegeri_varsa_zee_sayfasini_bayt_bayt_uretir() {
    let go = match Command::new("go").arg("version").output() {
        Ok(cikti) if cikti.status.success() => "go",
        _ => {
            eprintln!("go araç zinciri yok; Go eşdeğerliği bu makinede atlandı");
            return;
        }
    };
    let cikti = Command::new(go)
        .args(["run", "."])
        .arg(depo())
        .current_dir(depo().join("dogfood/kanit-ozeti-karsilastirma/go"))
        .output()
        .expect("go run çalışmalı");
    assert!(
        cikti.status.success(),
        "{}",
        String::from_utf8_lossy(&cikti.stderr)
    );
    assert_eq!(
        String::from_utf8(cikti.stdout).expect("UTF-8"),
        oku("docs/kanit-ozeti.md"),
        "Go eşdeğeri Zee ürününün sayfasından sapıyor"
    );
}

#[test]
fn karsilastirma_tarihcesi_exact_sha_ve_tekrar_hesaplanabilir_olculerle_tutarlidir() {
    let tsv = oku("docs/dogfood-karsilastirma-v1.tsv");
    let mut satirlar = tsv.lines();
    assert_eq!(satirlar.next(), Some("# zee-dogfood-karsilastirma-1"));
    let kayitlar = satirlar
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(|s| s.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert!(!kayitlar.is_empty(), "en az bir karşılaştırma kaydı");
    for kayit in &kayitlar {
        assert_eq!(kayit.len(), 22, "22 alan: {kayit:?}");
        assert!(
            kayit[0].len() == 40 && kayit[0].bytes().all(|b| b.is_ascii_hexdigit()),
            "exact SHA: {}",
            kayit[0]
        );
        let var = Command::new("git")
            .args(["cat-file", "-e", &format!("{}^{{commit}}", kayit[0])])
            .current_dir(depo())
            .status()
            .expect("git");
        assert!(var.success(), "kayıt commit'i depoda yok: {}", kayit[0]);
    }
    let son = kayitlar.last().expect("son kayıt");
    let zee = zee_kaynaklari();
    let zee_loc = loc(&zee.iter().map(String::as_str).collect::<Vec<_>>(), "#");
    assert_eq!(son[6].parse::<usize>().ok(), Some(zee_loc), "Zee LOC bayat");
    assert_eq!(
        son[7].parse::<usize>().ok(),
        Some(loc(
            &["dogfood/kanit-ozeti-karsilastirma/rust/src/main.rs"],
            "//"
        )),
        "Rust LOC bayat"
    );
    assert_eq!(
        son[8].parse::<usize>().ok(),
        Some(loc(
            &[
                "dogfood/kanit-ozeti-karsilastirma/go/main.go",
                "dogfood/kanit-ozeti-karsilastirma/go/main_test.go"
            ],
            "//"
        )),
        "Go LOC bayat"
    );
    let zee_test: usize = zee
        .iter()
        .map(|d| oku(d).lines().filter(|s| s.starts_with("test \"")).count())
        .sum();
    assert_eq!(
        son[9].parse::<usize>().ok(),
        Some(zee_test),
        "Zee test sayısı bayat"
    );
    let md = oku("docs/dogfood-karsilastirma.md");
    assert!(
        md.contains(&format!("`{}`", &son[0][..12])),
        "karşılaştırma belgesi son kaydın SHA'sını taşımalı"
    );
}
