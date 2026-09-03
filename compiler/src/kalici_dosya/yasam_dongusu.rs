//! Hazırlanmış dosyayı yayımlama ve idempotent fiziksel temizleme işlemleri.

use super::{ebeveyn, klasoru_eszamanla, DosyaKilidi};
use std::io;
use std::path::Path;

/// Var olan hedefi ezmeden, aynı dosya sistemindeki hazırlanmış dosyayı tek
/// adlandırma adımıyla görünür kılar.
pub fn atomik_tasi(kaynak: &Path, hedef: &Path) -> io::Result<()> {
    if kaynak == hedef {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "kaynak ve hedef dosya aynı olamaz",
        ));
    }
    let _kilit = DosyaKilidi::al(hedef)?;
    if hedef.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "hedef dosya zaten var",
        ));
    }
    std::fs::rename(kaynak, hedef)?;
    let kaynak_ust = ebeveyn(kaynak);
    let hedef_ust = ebeveyn(hedef);
    klasoru_eszamanla(hedef_ust)?;
    if kaynak_ust != hedef_ust {
        klasoru_eszamanla(kaynak_ust)?;
    }
    Ok(())
}

/// Bulunmayan hedefi başarı sayan, dizin metadata'sını da eşzamanlayan silme.
pub fn atomik_sil(yol: &Path) -> io::Result<bool> {
    let _kilit = DosyaKilidi::al(yol)?;
    match std::fs::remove_file(yol) {
        Ok(()) => {
            klasoru_eszamanla(ebeveyn(yol))?;
            Ok(true)
        }
        Err(hata) if hata.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(hata) => Err(hata),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SIRA: AtomicU64 = AtomicU64::new(0);

    struct GeciciKlasor(PathBuf);

    impl GeciciKlasor {
        fn yeni() -> Self {
            let yol = std::env::temp_dir().join(format!(
                "zee-yasam-dongusu-{}-{}",
                std::process::id(),
                SIRA.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&yol).unwrap();
            Self(yol)
        }
    }

    impl Drop for GeciciKlasor {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn hazirlanmis_dosya_atomik_yayinlanir_ve_hedef_ezilmez() {
        let gecici = GeciciKlasor::yeni();
        let kaynak = gecici.0.join("gecici.bin");
        let hedef = gecici.0.join("medya.bin");
        std::fs::write(&kaynak, [0, 255, 1]).unwrap();
        atomik_tasi(&kaynak, &hedef).unwrap();
        assert!(!kaynak.exists());
        assert_eq!(std::fs::read(&hedef).unwrap(), [0, 255, 1]);

        std::fs::write(&kaynak, b"yeni").unwrap();
        assert_eq!(
            atomik_tasi(&kaynak, &hedef).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(std::fs::read(&kaynak).unwrap(), b"yeni");
        assert_eq!(std::fs::read(&hedef).unwrap(), [0, 255, 1]);
    }

    #[test]
    fn atomik_silme_bulunmayan_hedefte_idempotenttir() {
        let gecici = GeciciKlasor::yeni();
        let yol = gecici.0.join("medya.bin");
        std::fs::write(&yol, b"icerik").unwrap();
        assert!(atomik_sil(&yol).unwrap());
        assert!(!atomik_sil(&yol).unwrap());
    }
}
