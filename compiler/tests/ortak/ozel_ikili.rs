//! K-319: native kampanyalar okudukları ikiliden tek sabit bağlantı (`st_nlink == 1`) ister.
//! Linux'ta Cargo `target/<profil>/<ad>` ile `deps/` kopyasını sabit bağlantıyla yayımlar
//! (bağlantı sayısı 2); macOS kopyalar. Kampanyaya ikilinin özel, tek bağlantılı kopyası verilir.
//! Linux debug derlemesi tam debug bilgisiyle kampanyaların 64 MiB ikili sınırını aşar; özel kopyadan
//! yalnız debug bilgisi silinir (davranış aynı, özetleri kampanya kendisi hesaplar). macOS'ta strip
//! imzayı bozacağı ve debug bilgisi zaten ayrı tutulduğu için kopya olduğu gibi kalır.

use std::path::{Path, PathBuf};

/// Kaynak ikilileri yeni bir 0700 klasöre kopyalar (yeni inode, bağlantı sayısı 1) ve klasörü döndürür.
pub fn ozel_klasor(etiket: &str, ikililer: &[&str]) -> PathBuf {
    let zaman = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("saat")
        .as_nanos();
    let klasor = std::env::temp_dir().join(format!(
        "zee-ozel-ikili-{etiket}-{}-{zaman}",
        std::process::id()
    ));
    std::fs::create_dir(&klasor).expect("özel ikili klasörü");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&klasor, std::fs::Permissions::from_mode(0o700))
            .expect("özel klasör izni");
    }
    for kaynak in ikililer {
        let kaynak = Path::new(kaynak);
        let hedef = klasor.join(kaynak.file_name().expect("ikili adı"));
        std::fs::copy(kaynak, &hedef).expect("ikili kopyası");
        #[cfg(target_os = "linux")]
        {
            let durum = std::process::Command::new("strip")
                .arg("--strip-debug")
                .arg(&hedef)
                .status()
                .expect("strip çalıştırılmalı");
            assert!(durum.success(), "strip --strip-debug başarısız");
        }
    }
    klasor
}

/// `CARGO_BIN_EXE_dil` gibi tek bir ikilinin özel kopyası.
pub fn ozel_kopya(etiket: &str, ikili: &str) -> PathBuf {
    let ad = Path::new(ikili).file_name().expect("ikili adı").to_owned();
    ozel_klasor(etiket, &[ikili]).join(ad)
}
