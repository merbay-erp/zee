//! K-348: Windows `ReplaceFileW` hedefi iki adımda değiştirir (hedef →
//! `.<ad>.zee-gecici-*.zee-yedek`, sonra geçici → hedef). Yazar arada ölürse
//! hedef yok, eski bütün yedekte, yayımlanmamış yeni içerik geçicide kalır
//! (Windows CI'da öldürülen yazarların %4'ü). Harness bu durumu CI teşhisindeki
//! adlarla kurar; gerçek native `dil` önce okur sonra ekler. Düzeltme öncesi
//! okuma `okunamadı`, ekleme eski satırları silip dosyayı `b-1`e indirir.
//! Kurtarma yalnız Windows yazarında olduğundan kip Windows dışında ignored.
#![allow(dead_code)]

use std::path::Path;

pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    struct Temizle(std::path::PathBuf);
    impl Drop for Temizle {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "zee-coken-degistirme-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    ));
    let _temizle = Temizle(proje.clone());
    std::fs::create_dir_all(&proje).expect("proje klasörü");
    std::fs::copy(kaynak, proje.join("ana.dil")).expect("kaynak kopyası");
    std::fs::write(proje.join(".zee-yazma-kilidi"), b"").expect("kilit dosyası");
    std::fs::write(
        proje.join(".ortak.txt.zee-gecici-4242-1"),
        "a-1\na-2\na-3\n",
    )
    .expect("yayımlanmamış yeni içerik");
    std::fs::write(
        proje.join(".ortak.txt.zee-gecici-4242-1.zee-yedek"),
        "a-1\na-2\n",
    )
    .expect("eski bütün yedek");

    let cikti = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", "ana.dil"])
        .current_dir(&proje)
        .output()
        .expect("dil süreci");
    let mut satirlar: Vec<String> = String::from_utf8_lossy(&cikti.stdout)
        .lines()
        .filter(|satir| !satir.is_empty())
        .map(str::to_string)
        .collect();
    let artik = std::fs::read_dir(&proje)
        .expect("proje listesi")
        .filter(|girdi| {
            girdi
                .as_ref()
                .map(|g| g.file_name().to_string_lossy().contains("zee-gecici"))
                .unwrap_or(false)
        })
        .count();
    satirlar.push(format!("iz {artik}"));
    (i64::from(cikti.status.code().unwrap_or(-1)), satirlar)
}
