//! K-422 (ZM-F051, ADR-170 eki): `fs.protected_hardlinks` okunamayan
//! (Ubuntu 22.04, `0600 root`) ya da hiç olmayan (macOS) ortamda paylaşılan
//! veri kökünün son klasörüne eş hizmet, yazarın okuyabildiği kök dışı bir
//! dosyayı sert bağlayabilir. Harness kökü ve bağı kurar, gerçek native
//! `dil`i manifestin kaynağıyla koşar. Düzeltme öncesi ekleme iç okuması
//! bağı izliyordu: `eklendi` ve dış içerik yeni tek bağlı dosyaya aklanırdı
//! (`aklanan 1`). Veri kökü unix dışında yoktur; kip orada ignored.
#![allow(dead_code)]

use std::path::Path;

#[cfg(unix)]
const DEGISKEN: &str = "ZEE_K422_DEPO";
#[cfg(unix)]
const SIR: &[u8] = b"K422_KOK_DISI_SIR";

#[cfg(not(unix))]
pub fn calistir(_kaynak: &Path) -> (i64, Vec<String>) {
    panic!("veri kökü unix dışında desteklenmez; sahte başarı yok")
}

#[cfg(unix)]
pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    struct Temizle(std::path::PathBuf);
    impl Drop for Temizle {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    // Veri kökünün üst klasörleri başkalarınca yazılamamalı: yapışkan
    // geçici dizin altında 0700 alan (spec/07 §Paylaşılan veri kökü).
    let alan = std::env::temp_dir()
        .canonicalize()
        .expect("geçici dizin")
        .join(format!(
            "zee-k422-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("saat")
                .as_nanos()
        ));
    let _temizle = Temizle(alan.clone());
    let depo = alan.join("depo");
    let proje = alan.join("proje");
    for klasor in [depo.join("ekler"), alan.join("dis"), proje.clone()] {
        std::fs::create_dir_all(klasor).expect("klasör");
    }
    for (klasor, kip) in [(&alan, 0o700), (&depo, 0o755)] {
        std::fs::set_permissions(klasor, std::fs::Permissions::from_mode(kip)).expect("izin");
    }
    std::fs::write(alan.join("dis/sir"), SIR).expect("dış dosya");
    std::fs::hard_link(alan.join("dis/sir"), depo.join("ekler/gunluk")).expect("sert bağ");
    std::fs::write(
        proje.join("proje.dil"),
        format!(
            "proje \"k422\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"veri-okuma\", \"veri-yazma\" listesi olsun\nağ_hedefleri boş liste olsun\nveri_kökü \"{DEGISKEN}\" olsun\n"
        ),
    )
    .expect("proje bildirimi");
    std::fs::copy(kaynak, proje.join("ana.dil")).expect("kaynak kopyası");

    let cikti = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", "."])
        .current_dir(&proje)
        .env_remove("CREDENTIALS_DIRECTORY")
        .env(DEGISKEN, &depo)
        .output()
        .expect("dil süreci");
    let mut satirlar: Vec<String> = String::from_utf8_lossy(&cikti.stdout)
        .lines()
        .filter(|satir| !satir.is_empty())
        .map(str::to_string)
        .collect();
    let aklanan = std::fs::read_dir(depo.join("ekler"))
        .expect("ekler listesi")
        .filter_map(Result::ok)
        .filter(|girdi| {
            let yol = girdi.path();
            std::fs::symlink_metadata(&yol).is_ok_and(|bilgi| bilgi.is_file() && bilgi.nlink() == 1)
                && std::fs::read(&yol)
                    .is_ok_and(|icerik| icerik.windows(SIR.len()).any(|p| p == SIR))
        })
        .count();
    satirlar.push(format!("aklanan {aklanan}"));
    let dis = std::fs::read(alan.join("dis/sir")).unwrap_or_default();
    satirlar.push(format!(
        "dış {}",
        if dis == SIR { "aynı" } else { "değişti" }
    ));
    (i64::from(cikti.status.code().unwrap_or(-1)), satirlar)
}
