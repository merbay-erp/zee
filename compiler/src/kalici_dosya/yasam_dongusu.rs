//! Hazırlanmış dosyayı yayımlama ve idempotent fiziksel temizleme işlemleri.

use super::adim::{nokta, sar, Adim};
use super::{ebeveyn, gecici_dosya_ac, klasoru_eszamanla, metadata::MetadataKaynagi, DosyaKilidi};
use std::fs::File;
use std::io;
use std::path::Path;

use crate::kaynak_sinirlari::degistirme_penceresi::yarim_degistirme as yarim;

/// Var olan hedefi ezmeden hazırlanmış dosyayı tek adlandırma adımıyla
/// görünür kılar. K-329: kaynak ile hedef ayrı bağlama noktalarındaysa
/// (`EXDEV`; systemd `ReadWritePaths=` her yolu ayrı bind mount eder) hedef
/// klasöründe geçici kopya üzerinden yine atomik görünürlükle taşır.
pub fn atomik_tasi(kaynak: &Path, hedef: &Path) -> io::Result<()> {
    atomik_tasi_ile(kaynak, hedef, false, |eski, yeni| {
        std::fs::rename(eski, yeni)
    })
    .map(|_| ())
}

/// K-347: [`atomik_tasi`] ile aynı taşıma; taşınan bütün içeriği adlandırmayla
/// aynı kilit penceresinden döndürür. Eylem geri alması kaynağa bu içeriği
/// koyar; kilit dışında okunan kopya araya giren yazarın satırını içermiyordu.
pub fn atomik_tasi_izli(kaynak: &Path, hedef: &Path) -> io::Result<Vec<u8>> {
    atomik_tasi_ile(kaynak, hedef, true, |eski, yeni| {
        std::fs::rename(eski, yeni)
    })
    .map(Option::unwrap_or_default)
}

/// K-347: kaynak ve hedef klasörlerinin yazma kilitleri. Aynı klasörde tek
/// kilit alınır (aynı süreçte ikinci kilit kendini bekletirdi); ayrı
/// klasörlerde kanonik klasör yolu sırasıyla alınır ki iki ters yönlü taşıma
/// birbirini kilitlenmeye sokmasın.
fn kilit_cifti(kaynak: &Path, hedef: &Path) -> io::Result<(DosyaKilidi, Option<DosyaKilidi>)> {
    let kanonik = |yol: &Path| {
        let klasor = ebeveyn(yol);
        std::fs::canonicalize(klasor).unwrap_or_else(|_| klasor.to_path_buf())
    };
    let (kaynak_ust, hedef_ust) = (kanonik(kaynak), kanonik(hedef));
    if kaynak_ust == hedef_ust {
        return Ok((DosyaKilidi::al(hedef)?, None));
    }
    let (ilk, ikinci) = if kaynak_ust < hedef_ust {
        (kaynak, hedef)
    } else {
        (hedef, kaynak)
    };
    let ilk = DosyaKilidi::al(ilk)?;
    Ok((ilk, Some(DosyaKilidi::al(ikinci)?)))
}

fn atomik_tasi_ile(
    kaynak: &Path,
    hedef: &Path,
    izli: bool,
    adlandir: impl Fn(&Path, &Path) -> io::Result<()>,
) -> io::Result<Option<Vec<u8>>> {
    if kaynak == hedef {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "kaynak ve hedef dosya aynı olamaz",
        ));
    }
    let _kilitler = kilit_cifti(kaynak, hedef)?;
    if hedef.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "hedef dosya zaten var",
        ));
    }
    let icerik = if izli {
        let icerik = nokta(Adim::MevcutOkuma)
            .and_then(|()| crate::kaynak_sinirlari::veri_dosyasi_baytlarini_oku(kaynak))
            .map_err(sar(Adim::MevcutOkuma))?;
        Some(icerik)
    } else {
        None
    };
    match adlandir(kaynak, hedef) {
        Ok(()) => {}
        Err(hata) if hata.kind() == io::ErrorKind::CrossesDevices => {
            return aygitlar_arasi_tasi(kaynak, hedef, &adlandir).map(|()| icerik);
        }
        Err(hata) => return Err(hata),
    }
    let kaynak_ust = ebeveyn(kaynak);
    let hedef_ust = ebeveyn(hedef);
    klasoru_eszamanla(hedef_ust)?;
    if kaynak_ust != hedef_ust {
        klasoru_eszamanla(kaynak_ust)?;
    }
    Ok(icerik)
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
    sil_kilit_altinda(yol)
}

/// K-346: silinen içeriği silmeyle aynı kilit penceresinde yakalar; eylem
/// geri alması araya giren yazarın içeriğini eski yedekle ezmez.
pub fn atomik_sil_izli(yol: &Path) -> io::Result<super::KilitliDegisim> {
    let _kilit = DosyaKilidi::al(yol)?;
    let onceki = super::mevcut_icerik(yol)?;
    sil_kilit_altinda(yol)?;
    Ok(super::KilitliDegisim {
        onceki,
        sonraki: None,
    })
}

/// K-348: kilit altında, hedef yoksa ona ait yarım Windows değiştirmesini
/// kurtarır. Yazar başarı bildirmediğinden geçerli bütün eski içeriktir: en
/// yeni yedek hedefe döner, eş geçici (yayımlanmamış yeni içerik) ve bayat
/// yedekler silinir. Kilit tutulurken başka zee değiştirmesi süremez.
///
/// K-351: hedef varken duran yedek yarım değiştirme değildir (ReplaceFileW
/// hedefi yalnız iki adımının arasında yok bırakır); başarılı yazımın
/// silinemeyen artığıdır ve temizlenir. Kalsaydı hedef sonradan zee dışından
/// silindiğinde eski içerik kurtarma diye diriltilirdi.
#[cfg_attr(not(windows), allow(dead_code))]
pub(super) fn yarim_degistirmeyi_kurtar(hedef: &Path) -> io::Result<bool> {
    match std::fs::symlink_metadata(hedef) {
        Ok(_) => {
            // En iyi çaba: liste hatası yeni yazımı engellemez.
            if let Ok(artiklar) = yarim::yedekler(hedef) {
                yedek_eslerini_sil(artiklar.into_iter());
            }
            return Ok(false);
        }
        Err(hata) if hata.kind() == io::ErrorKind::NotFound => {}
        Err(hata) => return Err(hata),
    }
    let mut yedekler = yarim::yedekler(hedef)?.into_iter();
    let Some(en_yeni) = yedekler.next() else {
        return Ok(false);
    };
    std::fs::rename(&en_yeni, hedef)?;
    yedek_eslerini_sil(std::iter::once(en_yeni).chain(yedekler));
    klasoru_eszamanla(ebeveyn(hedef))?;
    Ok(true)
}

/// Yedeklerin yayımlanmamış geçici eşlerini ve hâlâ duran yedekleri siler.
fn yedek_eslerini_sil(yedekler: impl Iterator<Item = std::path::PathBuf>) {
    for yedek in yedekler {
        if let Some(gecici) = yarim::yedegin_gecicisi(&yedek) {
            let _ = std::fs::remove_file(gecici);
        }
        let _ = std::fs::remove_file(yedek);
    }
}

fn sil_kilit_altinda(yol: &Path) -> io::Result<bool> {
    // K-348: hedef silinince bayat yarım değiştirme yedeği sonraki yazımda
    // silinmiş içeriği diriltmemeli.
    if cfg!(windows) {
        yedek_eslerini_sil(yarim::yedekler(yol)?.into_iter());
    }
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
    fn yarim_degistirme_eski_butunu_geri_koyar_ve_izleri_siler() {
        // K-348: Windows'ta ReplaceFileW ortasında ölen yazarın bıraktığı
        // durum (CI teşhisi): hedef yok, eski bütün yedekte, yeni geçicide.
        let gecici = GeciciKlasor::yeni();
        let hedef = gecici.0.join("ortak.txt");
        let bayat = gecici.0.join(".ortak.txt.zee-gecici-1-1.zee-yedek");
        std::fs::write(&bayat, "bayat\n").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(gecici.0.join(".ortak.txt.zee-gecici-9-2"), "a\nb\nyarım\n").unwrap();
        std::fs::write(
            gecici.0.join(".ortak.txt.zee-gecici-9-2.zee-yedek"),
            "a\nb\n",
        )
        .unwrap();
        assert!(yarim_degistirmeyi_kurtar(&hedef).unwrap());
        assert_eq!(std::fs::read_to_string(&hedef).unwrap(), "a\nb\n");
        let artik: Vec<_> = std::fs::read_dir(&gecici.0)
            .unwrap()
            .map(|g| g.unwrap().file_name())
            .filter(|ad| ad.to_string_lossy().contains("zee-gecici"))
            .collect();
        assert!(artik.is_empty(), "izler silinmeli: {artik:?}");
        // Hedef varken kurtarma bir şey yapmaz; iz yokken de.
        assert!(!yarim_degistirmeyi_kurtar(&hedef).unwrap());
        // K-351: hedef varken duran yedek başarılı yazımın artığıdır; geri
        // konmaz, silinir. Kalsaydı hedef dışarıdan silinince dirilirdi.
        let artik_yedek = gecici.0.join(".ortak.txt.zee-gecici-9-3.zee-yedek");
        std::fs::write(&artik_yedek, "eski\n").unwrap();
        assert!(!yarim_degistirmeyi_kurtar(&hedef).unwrap());
        assert!(!artik_yedek.exists(), "artık yedek temizlenmeli");
        assert_eq!(std::fs::read_to_string(&hedef).unwrap(), "a\nb\n");
        std::fs::remove_file(&hedef).unwrap();
        assert!(!yarim_degistirmeyi_kurtar(&hedef).unwrap());
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
        // K-347: iki ayrı klasörün kilidi birlikte alınır; içerik kilit altında yakalanır.
        let tasinan = atomik_tasi_ile(&kaynak, &hedef, true, |eski, yeni| {
            if eski == kaynak_adi {
                Err(aygitlar_arasi_hatasi())
            } else {
                assert_eq!(eski.parent(), yeni.parent(), "yayın hedef klasöründe");
                std::fs::rename(eski, yeni)
            }
        })
        .unwrap();
        assert_eq!(tasinan.as_deref(), Some(icerik.as_slice()));
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
        let hata = atomik_tasi_ile(&kaynak, &hedef, false, |_, _| Err(aygitlar_arasi_hatasi()))
            .unwrap_err();
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
        let hata = atomik_tasi_ile(&kaynak, &hedef, false, |_, _| {
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

    /// K-347: taşınan içerik kilit altında yakalanır; kaynağa yazmak isteyen
    /// yazar taşıma bitene dek bekler ve yeni kaynağı ayrı oluşturur.
    #[test]
    fn tasima_icerigi_kilit_altinda_yakalanir() {
        use crate::kalici_dosya::adim::test_kancasi::{kur, Enjeksiyon};
        let gecici = GeciciKlasor::yeni();
        let kaynak = gecici.0.join("kaynak.txt");
        let hedef = gecici.0.join("hedef.txt");
        std::fs::write(&kaynak, "ilk\n").unwrap();
        let (kilitte, kilit_bildirimi) = std::sync::mpsc::channel();
        let tasiyici = {
            let (kaynak, hedef) = (kaynak.clone(), hedef.clone());
            std::thread::spawn(move || {
                kur(
                    Adim::MevcutOkuma,
                    Enjeksiyon::Eylem(Box::new(move || {
                        kilitte.send(()).unwrap();
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        Ok(())
                    })),
                );
                atomik_tasi_izli(&kaynak, &hedef).unwrap()
            })
        };
        kilit_bildirimi.recv().unwrap();
        crate::kalici_dosya::atomik_satir_yaz(&kaynak, "b-1", true).unwrap();
        assert_eq!(tasiyici.join().unwrap(), b"ilk\n");
        assert_eq!(std::fs::read(&hedef).unwrap(), b"ilk\n");
        assert_eq!(std::fs::read(&kaynak).unwrap(), b"b-1\n");
    }

    /// K-347: iki klasör kilidi kanonik sırayla alınır; ters yönlü eşzamanlı
    /// taşımalar birbirini kilit zaman aşımına sokmaz.
    #[test]
    fn ters_yonlu_tasimalar_kilitlenmez() {
        let gecici = GeciciKlasor::yeni();
        let (a, b) = (gecici.0.join("a"), gecici.0.join("b"));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(a.join("x"), "x").unwrap();
        std::fs::write(b.join("y"), "y").unwrap();
        let iplikler: Vec<_> = [(a.clone(), b.clone(), "x"), (b, a, "y")]
            .into_iter()
            .map(|(ilk, ikinci, ad)| {
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        atomik_tasi_izli(&ilk.join(ad), &ikinci.join(ad)).unwrap();
                        atomik_tasi_izli(&ikinci.join(ad), &ilk.join(ad)).unwrap();
                    }
                })
            })
            .collect();
        for iplik in iplikler {
            iplik.join().unwrap();
        }
    }
}
