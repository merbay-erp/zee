//! Hazırlanmış dosyayı yayımlama ve idempotent fiziksel temizleme işlemleri.

use super::{ebeveyn, gecici_dosya_ac, klasoru_eszamanla, metadata::MetadataKaynagi, DosyaKilidi};
use std::fs::File;
use std::io;
use std::path::Path;

/// Var olan hedefi ezmeden hazırlanmış dosyayı tek adlandırma adımıyla
/// görünür kılar. K-329: kaynak ile hedef ayrı bağlama noktalarındaysa
/// (`EXDEV`; systemd `ReadWritePaths=` her yolu ayrı bind mount eder) hedef
/// klasöründe geçici kopya üzerinden yine atomik görünürlükle taşır.
pub fn atomik_tasi(kaynak: &Path, hedef: &Path) -> io::Result<()> {
    atomik_tasi_ile(kaynak, hedef, |eski, yeni| std::fs::rename(eski, yeni))
}

fn atomik_tasi_ile(
    kaynak: &Path,
    hedef: &Path,
    adlandir: impl Fn(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
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
    match adlandir(kaynak, hedef) {
        Ok(()) => {}
        Err(hata) if hata.kind() == io::ErrorKind::CrossesDevices => {
            return aygitlar_arasi_tasi(kaynak, hedef, &adlandir);
        }
        Err(hata) => return Err(hata),
    }
    let kaynak_ust = ebeveyn(kaynak);
    let hedef_ust = ebeveyn(hedef);
    klasoru_eszamanla(hedef_ust)?;
    if kaynak_ust != hedef_ust {
        klasoru_eszamanla(kaynak_ust)?;
    }
    Ok(())
}

/// EXDEV geri düşüşü: hedef klasöründe geçici dosyaya akışla kopya, kaynak
/// metadata'sı (mod, sahip, ACL/xattr) ve fsync; aynı klasörde atomik
/// adlandırma ile yayın; hedef klasörü fsync; en son kaynak silinir. Yayından
/// önceki her hata geçiciyi siler ve kaynağa dokunmaz; kaynak silinemezse
/// yayımlanan hedef geri çekilir, böylece iki kopya kalmaz.
fn aygitlar_arasi_tasi(
    kaynak: &Path,
    hedef: &Path,
    adlandir: &impl Fn(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    let metadata = MetadataKaynagi::yakala(kaynak)?;
    let mut okuyucu = File::open(kaynak)?;
    let (mut dosya, mut gecici) = gecici_dosya_ac(hedef)?;
    io::copy(&mut okuyucu, &mut dosya)?;
    metadata.geciciye_uygula(&dosya)?;
    dosya.sync_all()?;
    drop(dosya);
    drop(okuyucu);
    let gecici_yol = gecici
        .yol
        .clone()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "zee geçici dosya yolu kayboldu"))?;
    if hedef.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "hedef dosya zaten var",
        ));
    }
    adlandir(&gecici_yol, hedef)?;
    gecici.yol = None;
    let hedef_ust = ebeveyn(hedef);
    if let Err(hata) = klasoru_eszamanla(hedef_ust) {
        let _ = std::fs::remove_file(hedef);
        return Err(hata);
    }
    if let Err(hata) = std::fs::remove_file(kaynak) {
        let _ = std::fs::remove_file(hedef);
        let _ = klasoru_eszamanla(hedef_ust);
        return Err(hata);
    }
    klasoru_eszamanla(ebeveyn(kaynak))
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

    fn aygitlar_arasi_hatasi() -> io::Error {
        io::Error::from(io::ErrorKind::CrossesDevices)
    }

    fn klasor_adlari(klasor: &Path) -> Vec<String> {
        let mut adlar: Vec<_> = std::fs::read_dir(klasor)
            .unwrap()
            .map(|giris| giris.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|ad| ad != ".zee-yazma-kilidi")
            .collect();
        adlar.sort();
        adlar
    }

    /// K-329: ayrı bind mount'lar arasında rename EXDEV verir; taşıma geçici
    /// kopya + atomik yayın ile tamamlanır, kaynak silinir, izinler korunur.
    #[test]
    fn aygitlar_arasi_tasima_kopya_ile_atomik_tamamlanir() {
        let gecici = GeciciKlasor::yeni();
        let yukleme = gecici.0.join("yuklemeler");
        let medya = gecici.0.join("medya");
        std::fs::create_dir_all(&yukleme).unwrap();
        std::fs::create_dir_all(&medya).unwrap();
        let kaynak = yukleme.join("parca");
        let hedef = medya.join("resim.bin");
        let icerik: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
        std::fs::write(&kaynak, &icerik).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&kaynak, std::fs::Permissions::from_mode(0o640)).unwrap();
        }
        let kaynak_adi = kaynak.clone();
        atomik_tasi_ile(&kaynak, &hedef, |eski, yeni| {
            if eski == kaynak_adi {
                Err(aygitlar_arasi_hatasi())
            } else {
                assert_eq!(eski.parent(), yeni.parent(), "yayın hedef klasöründe");
                std::fs::rename(eski, yeni)
            }
        })
        .unwrap();
        assert!(!kaynak.exists());
        assert_eq!(std::fs::read(&hedef).unwrap(), icerik);
        assert_eq!(klasor_adlari(&medya), ["resim.bin"]);
        assert!(klasor_adlari(&yukleme).is_empty());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mod_ = std::fs::metadata(&hedef).unwrap().permissions().mode();
            assert_eq!(mod_ & 0o777, 0o640);
        }
    }

    #[test]
    fn aygitlar_arasi_yayin_hatasi_yarim_dosya_birakmaz_kaynagi_korur() {
        let gecici = GeciciKlasor::yeni();
        let kaynak = gecici.0.join("parca");
        let medya = gecici.0.join("medya");
        std::fs::create_dir_all(&medya).unwrap();
        let hedef = medya.join("resim.bin");
        std::fs::write(&kaynak, b"veri").unwrap();
        let hata =
            atomik_tasi_ile(&kaynak, &hedef, |_, _| Err(aygitlar_arasi_hatasi())).unwrap_err();
        assert_eq!(hata.kind(), io::ErrorKind::CrossesDevices);
        assert_eq!(std::fs::read(&kaynak).unwrap(), b"veri");
        assert!(klasor_adlari(&medya).is_empty(), "geçici kopya temizlenir");
    }

    #[test]
    fn aygitlar_arasi_disindaki_hata_bugunku_gibi_doner() {
        let gecici = GeciciKlasor::yeni();
        let kaynak = gecici.0.join("parca");
        let hedef = gecici.0.join("resim.bin");
        std::fs::write(&kaynak, b"veri").unwrap();
        let cagri = std::cell::Cell::new(0);
        let hata = atomik_tasi_ile(&kaynak, &hedef, |_, _| {
            cagri.set(cagri.get() + 1);
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        })
        .unwrap_err();
        assert_eq!(hata.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(cagri.get(), 1, "kopya geri düşüşü yalnız EXDEV'de");
        assert_eq!(std::fs::read(&kaynak).unwrap(), b"veri");
        assert_eq!(klasor_adlari(&gecici.0), ["parca"]);
    }

    /// Gerçek aygıt sınırı: Linux'ta `/dev/shm` (tmpfs) ile Cargo hedef
    /// klasörü genelde ayrı dosya sistemleridir; aynıysa sınama kanıt
    /// üretemeyeceğini söyleyerek erken döner.
    #[cfg(target_os = "linux")]
    #[test]
    fn linux_gercek_aygit_siniri_arasinda_tasir() {
        use std::os::unix::fs::MetadataExt;
        let shm = Path::new("/dev/shm");
        if !shm.is_dir() {
            eprintln!("/dev/shm yok; gerçek EXDEV sınaması atlandı");
            return;
        }
        let gecici = GeciciKlasor::yeni();
        let ayri = shm.join(format!(
            "zee-exdev-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&ayri).unwrap();
        let _ayri = GeciciKlasor(ayri.clone());
        if std::fs::metadata(&ayri).unwrap().dev() == std::fs::metadata(&gecici.0).unwrap().dev() {
            eprintln!("geçici klasör ile /dev/shm aynı aygıtta; gerçek EXDEV sınaması atlandı");
            return;
        }
        let kaynak = ayri.join("parca");
        let hedef = gecici.0.join("resim.bin");
        std::fs::write(&kaynak, b"bind mount sinirinda medya").unwrap();
        assert_eq!(
            std::fs::rename(&kaynak, gecici.0.join("deneme"))
                .unwrap_err()
                .kind(),
            io::ErrorKind::CrossesDevices
        );
        atomik_tasi(&kaynak, &hedef).unwrap();
        assert!(!kaynak.exists());
        assert_eq!(
            std::fs::read(&hedef).unwrap(),
            b"bind mount sinirinda medya"
        );
        assert_eq!(klasor_adlari(&gecici.0), ["resim.bin"]);
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
