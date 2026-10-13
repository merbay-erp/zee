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

// Tarihi ölçüm özgün SHA'yı korur; K-214/K-215 yalnız kaynak soyunu eşler.
fn tarihi_kanit_kaynagi(kok: &Path, sha: &str) -> Result<String, String> {
    let exact = |s: &str| {
        s.len() == 40
            && s.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    };
    if !exact(sha) {
        return Err("tarihsel kaynak exact SHA değil".into());
    }
    let mut eslemeler = std::collections::BTreeMap::new();
    for yol in [
        "docs/incelemeler/k214/sha-esleme-v1.tsv",
        "docs/incelemeler/k215/sha-esleme-v1.tsv",
    ] {
        let metin = std::fs::read_to_string(kok.join(yol)).map_err(|e| e.to_string())?;
        for satir in metin
            .lines()
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
        {
            let alanlar = satir.split('\t').collect::<Vec<_>>();
            let [eski, yeni] = alanlar.as_slice() else {
                return Err("bozuk kaynak eşlemesi".into());
            };
            if !exact(eski) || !exact(yeni) {
                return Err("eşleme exact SHA değil".into());
            }
            if let Some(onceki) = eslemeler.insert((*eski).to_string(), (*yeni).to_string()) {
                if onceki != *yeni {
                    return Err("çelişkili kaynak eşlemesi".into());
                }
            }
        }
    }
    let kaynak = eslemeler.get(sha).map(String::as_str).unwrap_or(sha);
    let var = Command::new("git")
        .current_dir(kok)
        .args(["merge-base", "--is-ancestor", kaynak, "HEAD"])
        .output()
        .map_err(|e| e.to_string())?;
    if !var.status.success() {
        return Err("tarihsel kaynak veya eşlenmiş kaynak HEAD atası değil".into());
    }
    Ok(kaynak.to_string())
}

#[test]
#[cfg(unix)]
fn tarihsel_kanit_anonim_ata_ister_eski_nesne_ve_rastgele_esleme_kabul_etmez() {
    let kok = std::env::temp_dir().join(format!(
        "zee-tarihsel-kanit-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(kok.join("docs/incelemeler/k214")).unwrap();
    std::fs::create_dir_all(kok.join("docs/incelemeler/k215")).unwrap();
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .current_dir(&kok)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };
    git(&["init", "-q", "--initial-branch=main"]);
    git(&["config", "user.name", "Sentetik CI"]);
    git(&["config", "user.email", "ci@ornek.com"]);
    std::fs::write(kok.join("kaynak"), "anonim kaynak\n").unwrap();
    git(&["add", "kaynak"]);
    git(&["commit", "-qm", "anonim ata"]);
    let ata = git(&["rev-parse", "HEAD"]);
    git(&["checkout", "-qb", "ayri"]);
    std::fs::write(kok.join("kaynak"), "ilgisiz kaynak\n").unwrap();
    git(&["commit", "-qam", "ilgisiz dal"]);
    let ayri = git(&["rev-parse", "HEAD"]);
    git(&["checkout", "-q", "main"]);
    let eski = "1".repeat(40);
    let k214 = kok.join("docs/incelemeler/k214/sha-esleme-v1.tsv");
    let k215 = kok.join("docs/incelemeler/k215/sha-esleme-v1.tsv");
    std::fs::write(&k214, "# map\n").unwrap();
    let satir = format!("{eski}\t{ata}\n");
    std::fs::write(&k215, &satir).unwrap();
    assert!(!Command::new("git")
        .current_dir(&kok)
        .args(["cat-file", "-e", &eski])
        .output()
        .unwrap()
        .status
        .success());
    assert_eq!(tarihi_kanit_kaynagi(&kok, &eski).unwrap(), ata);
    assert_eq!(std::fs::read_to_string(&k215).unwrap(), satir);
    assert_eq!(tarihi_kanit_kaynagi(&kok, &ata).unwrap(), ata);
    assert!(tarihi_kanit_kaynagi(&kok, &"2".repeat(40)).is_err());
    assert!(tarihi_kanit_kaynagi(&kok, "kisa").is_err());
    std::fs::write(&k215, format!("{eski}\t{ayri}\n")).unwrap();
    assert!(tarihi_kanit_kaynagi(&kok, &eski).is_err());
    std::fs::write(&k214, &satir).unwrap();
    assert!(tarihi_kanit_kaynagi(&kok, &eski).is_err());
    std::fs::write(&k214, "# map\n").unwrap();
    std::fs::write(&k215, format!("{eski}\tbozuk\n")).unwrap();
    assert!(tarihi_kanit_kaynagi(&kok, &eski).is_err());
    std::fs::remove_dir_all(kok).unwrap();
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
        tarihi_kanit_kaynagi(&depo(), kayit[0])
            .unwrap_or_else(|hata| panic!("tarihsel kaynak {}: {hata}", kayit[0]));
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
