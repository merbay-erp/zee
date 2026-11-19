//! K-351: kalıcı yazımın IO hatası hangi adımdan geldiğini taşır.
//!
//! Windows CI'da iki yazarlı ekleme bir kez `NotFound (os 2)` ile düştü; hata
//! `ReplaceFileW`, ilk yerleştirme `MoveFileExW`, yedek silme ya da kurtarma
//! adımlarından hangisinden geldiğini söylemediği için kaynak bulunamadı.
//! Her adım hatası burada adla sarılır; `io::ErrorKind` aynen korunur, özgün
//! hata `source()` ve metin içinde kalır. Test derlemesinde aynı adım noktaları
//! iş parçacığına özel hata/gecikme enjeksiyonu kabul eder (disk durması).

use std::fmt;
use std::io;
use std::time::Duration;

/// Atomik yazım zincirinin adımları. Bir kısmı yalnız Windows'ta ya da
/// yalnız POSIX'te kurulur.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Adim {
    /// Klasör kilit dosyasını açma ve süreçler-arası kilidi alma.
    Kilit,
    /// Kilit altında ölen yazarın yarım değiştirmesini kurtarma / artık yedek temizliği.
    Kurtarma,
    /// Hedefin mevcut içeriğini okuma (ekleme, izli yazım).
    MevcutOkuma,
    /// Hedef metadata'sını yakalama ve geçiciye uygulama.
    Metadata,
    /// Geçici dosyayı açma, yazma ve diske eşzamanlama.
    GeciciYazim,
    /// POSIX `rename` ile atomik değiştirme.
    Rename,
    /// Windows `ReplaceFileW` ile metadata korumalı değiştirme.
    ReplaceFileW,
    /// Windows ilk yerleştirme `MoveFileExW` (hedef henüz yoksa).
    MoveFileExW,
    /// `ReplaceFileW` kısmi hatasından sonra eski içeriği yedekten geri koyma.
    YedekGeriAlma,
    /// Başarılı `ReplaceFileW` sonrası yedeği silme; hatası yazımı düşürmez.
    YedekSilme,
    /// Hedef klasörünü eşzamanlama.
    KlasorEszamanlama,
}

impl Adim {
    fn ad(self) -> &'static str {
        match self {
            Adim::Kilit => "yazma kilidi",
            Adim::Kurtarma => "yarım değiştirme kurtarma",
            Adim::MevcutOkuma => "mevcut içerik okuma",
            Adim::Metadata => "metadata aktarımı",
            Adim::GeciciYazim => "geçici dosya yazımı",
            Adim::Rename => "rename",
            Adim::ReplaceFileW => "ReplaceFileW",
            Adim::MoveFileExW => "MoveFileExW ilk yerleştirme",
            Adim::YedekGeriAlma => "MoveFileExW yedek geri alma",
            Adim::YedekSilme => "yedek silme",
            Adim::KlasorEszamanlama => "klasör eşzamanlama",
        }
    }
}

impl fmt::Display for Adim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.ad())
    }
}

#[derive(Debug)]
struct AdimHatasi {
    adim: Adim,
    kaynak: io::Error,
}

impl fmt::Display for AdimHatasi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "kalıcı yazım adımı `{}`: {}", self.adim, self.kaynak)
    }
}

impl std::error::Error for AdimHatasi {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kaynak)
    }
}

/// Hatayı adımla sarar; tür (`kind`) korunur. Zaten adım taşıyan hata ilk
/// (en içteki) adımını korur.
pub(crate) fn baglamla(adim: Adim, hata: io::Error) -> io::Error {
    if hata_adimi(&hata).is_some() {
        return hata;
    }
    io::Error::new(hata.kind(), AdimHatasi { adim, kaynak: hata })
}

/// `map_err` için kısayol.
pub(crate) fn sar(adim: Adim) -> impl FnOnce(io::Error) -> io::Error {
    move |hata| baglamla(adim, hata)
}

/// Hatanın geldiği kalıcı yazım adımı (bağlamsız hata için `None`).
pub fn hata_adimi(hata: &io::Error) -> Option<Adim> {
    hata.get_ref()?
        .downcast_ref::<AdimHatasi>()
        .map(|baglamli| baglamli.adim)
}

/// Adım noktası: üretimde boş; testte bu iş parçacığına kurulmuş enjeksiyonu
/// uygular. Dönen hata çağıranın gerçek sistem çağrısı hatası gibi sarılır.
#[inline]
pub(super) fn nokta(adim: Adim) -> io::Result<()> {
    #[cfg(test)]
    {
        test_kancasi::uygula(adim)
    }
    #[cfg(not(test))]
    {
        let _ = adim;
        Ok(())
    }
}

/// Bekleyen yazarın kilit bütçesi (spec/08: 5 sn). Test, durma senaryosunu
/// saniyelerce beklemeden kurmak için yalnız kendi iş parçacığında kısaltır.
pub(super) fn kilit_bekleme() -> Duration {
    #[cfg(test)]
    if let Some(sure) = test_kancasi::kilit_bekleme() {
        return sure;
    }
    super::KILIT_BEKLEME
}

#[cfg(test)]
pub(super) mod test_kancasi {
    use super::Adim;
    use std::cell::RefCell;
    use std::io;
    use std::time::Duration;

    /// Bir adım noktasında yapılacak enjeksiyon.
    #[derive(Debug, Clone, Copy)]
    pub enum Enjeksiyon {
        /// Ham işletim sistemi hata kodu döndür.
        OsHatasi(i32),
        /// Adım öncesi bekle (disk durması), sonra devam et.
        Gecikme(Duration),
    }

    thread_local! {
        static KURULU: RefCell<Vec<(Adim, Enjeksiyon)>> = const { RefCell::new(Vec::new()) };
        static KILIT_BEKLEME: RefCell<Option<Duration>> = const { RefCell::new(None) };
    }

    /// Bu iş parçacığında `adim` noktasına tek kullanımlık enjeksiyon kurar.
    pub fn kur(adim: Adim, enjeksiyon: Enjeksiyon) {
        KURULU.with(|kurulu| kurulu.borrow_mut().push((adim, enjeksiyon)));
    }

    pub fn kilit_beklemesini_kisalt(sure: Duration) {
        KILIT_BEKLEME.with(|bekleme| *bekleme.borrow_mut() = Some(sure));
    }

    pub(super) fn kilit_bekleme() -> Option<Duration> {
        KILIT_BEKLEME.with(|bekleme| *bekleme.borrow())
    }

    pub(super) fn uygula(adim: Adim) -> io::Result<()> {
        let bulunan = KURULU.with(|kurulu| {
            let mut kurulu = kurulu.borrow_mut();
            let sira = kurulu.iter().position(|(a, _)| *a == adim)?;
            Some(kurulu.remove(sira).1)
        });
        match bulunan {
            Some(Enjeksiyon::OsHatasi(kod)) => Err(io::Error::from_raw_os_error(kod)),
            Some(Enjeksiyon::Gecikme(sure)) => {
                std::thread::sleep(sure);
                Ok(())
            }
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_kancasi::{kilit_beklemesini_kisalt, kur, Enjeksiyon};
    use super::*;
    use crate::kalici_dosya::{atomik_satir_yaz, atomik_yaz};
    use std::path::PathBuf;
    use std::sync::Arc;

    struct GeciciKlasor(PathBuf);

    impl GeciciKlasor {
        fn yeni(ad: &str) -> Self {
            let yol = std::env::temp_dir().join(format!(
                "zee-k351-{ad}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
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
    fn baglam_turu_korur_ve_ilk_adimi_saklar() {
        let hata = baglamla(Adim::ReplaceFileW, io::Error::from_raw_os_error(2));
        assert_eq!(hata.kind(), io::ErrorKind::NotFound);
        assert_eq!(hata_adimi(&hata), Some(Adim::ReplaceFileW));
        assert!(hata.to_string().contains("`ReplaceFileW`"), "{hata}");
        let dis = baglamla(Adim::GeciciYazim, hata);
        assert_eq!(hata_adimi(&dis), Some(Adim::ReplaceFileW));
        assert_eq!(dis.kind(), io::ErrorKind::NotFound);
        let kaynak = std::error::Error::source(dis.get_ref().unwrap()).unwrap();
        assert_eq!(
            kaynak.downcast_ref::<io::Error>().unwrap().raw_os_error(),
            Some(2)
        );
        assert_eq!(hata_adimi(&io::Error::other("bağlamsız")), None);
    }

    /// Her platformda koşan adımlar: enjekte edilen sistem hatası adımın
    /// adıyla ve özgün `ErrorKind` ile çağırana ulaşır, eski içerik bozulmaz.
    #[test]
    fn enjekte_edilen_adim_hatasi_adimla_ve_turuyla_doner() {
        let degistirme = if cfg!(windows) {
            Adim::ReplaceFileW
        } else {
            Adim::Rename
        };
        let mut adimlar = vec![
            Adim::Kilit,
            Adim::MevcutOkuma,
            Adim::Metadata,
            Adim::GeciciYazim,
            degistirme,
        ];
        if cfg!(unix) {
            adimlar.push(Adim::KlasorEszamanlama);
        }
        if cfg!(windows) {
            adimlar.push(Adim::Kurtarma);
        }
        for adim in adimlar {
            let klasor = GeciciKlasor::yeni("adim");
            let yol = klasor.0.join("defter.txt");
            atomik_satir_yaz(&yol, "eski", false).expect("ilk yazım");
            // EACCES / ERROR_ACCESS_DENIED: Windows'un FILE_NOT_FOUND ilk
            // yerleştirme dalına sapmayan, iki platformda aynı türlü hata.
            kur(
                adim,
                Enjeksiyon::OsHatasi(if cfg!(windows) { 5 } else { 13 }),
            );
            let hata = atomik_satir_yaz(&yol, "yeni", true).expect_err("enjekte hata");
            assert_eq!(hata_adimi(&hata), Some(adim), "{adim}: {hata}");
            assert_eq!(hata.kind(), io::ErrorKind::PermissionDenied, "{adim}");
            assert!(hata.to_string().contains(&format!("`{adim}`")), "{hata}");
            let kalan = std::fs::read_to_string(&yol).unwrap();
            if adim == Adim::KlasorEszamanlama {
                // Yayım sonrası eşzamanlama hatası: içerik zaten yenidir.
                assert_eq!(kalan, "eski\nyeni\n");
            } else {
                assert_eq!(kalan, "eski\n", "{adim}: eski bütün korunmalı");
            }
        }
    }

    /// K-351: Windows CI koşusu 44576442062'deki disk durması sınıfının
    /// deterministik yeniden üretimi. Yazar A kilit altında değiştirme
    /// adımında durur; B'nin bütçesi dolar ve hata `TimedOut` türünü ve
    /// `yazma kilidi` adımını taşır. A durma bitince satırını kaybetmez.
    #[test]
    fn disk_durmasinda_bekleyen_yazar_kilit_adimiyla_zaman_asimi_alir() {
        let klasor = GeciciKlasor::yeni("durma");
        let yol = Arc::new(klasor.0.join("olaylar.txt"));
        atomik_satir_yaz(&yol, "ilk", false).expect("ilk yazım");
        let degistirme = if cfg!(windows) {
            Adim::ReplaceFileW
        } else {
            Adim::Rename
        };
        let a = {
            let yol = Arc::clone(&yol);
            std::thread::spawn(move || {
                kur(degistirme, Enjeksiyon::Gecikme(Duration::from_millis(800)));
                atomik_satir_yaz(&yol, "a", true)
            })
        };
        // Geçici dosya göründüğünde A kilidi tutuyor ve değiştirmede duruyor.
        let baslangic = std::time::Instant::now();
        while !std::fs::read_dir(&klasor.0)
            .unwrap()
            .filter_map(Result::ok)
            .any(|girdi| girdi.file_name().to_string_lossy().contains("zee-gecici"))
        {
            assert!(baslangic.elapsed() < Duration::from_secs(5), "A başlamadı");
            std::thread::sleep(Duration::from_millis(2));
        }
        kilit_beklemesini_kisalt(Duration::from_millis(150));
        let hata = atomik_satir_yaz(&yol, "b", true).expect_err("durma bütçeyi aşar");
        assert_eq!(hata.kind(), io::ErrorKind::TimedOut, "{hata}");
        assert_eq!(hata_adimi(&hata), Some(Adim::Kilit), "{hata}");
        assert!(hata.to_string().contains("`yazma kilidi`"), "{hata}");
        a.join().unwrap().expect("duran yazar sonunda yazar");
        assert_eq!(std::fs::read_to_string(&*yol).unwrap(), "ilk\na\n");
    }

    /// K-351: başarılı `ReplaceFileW` sonrası yedek silinemezse yazım hata
    /// sayılmaz; artık yedek sonraki yazımın kilidi altında temizlenir.
    #[test]
    #[cfg_attr(not(windows), ignore = "ReplaceFileW backup exists only on Windows")]
    fn yedek_silinemezse_basarili_yazim_hata_olmaz_ve_artik_temizlenir() {
        let klasor = GeciciKlasor::yeni("yedek");
        let yol = klasor.0.join("durum.txt");
        atomik_yaz(&yol, b"eski").expect("ilk yazım");
        kur(Adim::YedekSilme, Enjeksiyon::OsHatasi(32));
        atomik_yaz(&yol, b"yeni").expect("yedek silme hatası yazımı düşürmez");
        assert_eq!(std::fs::read(&yol).unwrap(), b"yeni");
        let yedekler = || {
            crate::kaynak_sinirlari::degistirme_penceresi::yarim_degistirme::yedekler(&yol).unwrap()
        };
        assert_eq!(yedekler().len(), 1, "artık yedek kalmalı");
        atomik_yaz(&yol, b"sonraki").expect("sonraki yazım");
        assert!(yedekler().is_empty(), "sonraki yazım artığı temizlemeli");
        assert_eq!(std::fs::read(&yol).unwrap(), b"sonraki");
    }

    /// K-351: hedef varken `ReplaceFileW` 2/3 dönerse ilk yerleştirme
    /// `MoveFileExW`'ye sapılmaz (metadata merge'siz ezme); eski bütün kalır.
    #[test]
    #[cfg_attr(not(windows), ignore = "ReplaceFileW path exists only on Windows")]
    fn replace_bulunamadi_hedef_varken_ilk_yerlestirmeye_sapmaz() {
        let klasor = GeciciKlasor::yeni("bulunamadi");
        let yol = klasor.0.join("durum.txt");
        atomik_yaz(&yol, b"eski").expect("ilk yazım");
        kur(Adim::ReplaceFileW, Enjeksiyon::OsHatasi(2));
        let hata = atomik_yaz(&yol, b"yeni").expect_err("hedef varken ezilmemeli");
        assert_eq!(hata.kind(), io::ErrorKind::NotFound, "{hata}");
        assert_eq!(hata_adimi(&hata), Some(Adim::ReplaceFileW), "{hata}");
        assert_eq!(std::fs::read(&yol).unwrap(), b"eski");
        let iz = std::fs::read_dir(&klasor.0)
            .unwrap()
            .filter_map(Result::ok)
            .any(|girdi| girdi.file_name().to_string_lossy().contains("zee-gecici"));
        assert!(!iz, "geçici ya da yedek izi kalmamalı");
    }

    /// K-351: `ReplaceFileW` eski hedefi yedeğe taşıyıp 2 ile düşerse eski
    /// bütün yedekten geri kurulur; yedek silinip geçici metadata merge'siz
    /// yayımlanmaz.
    #[test]
    #[cfg_attr(not(windows), ignore = "ReplaceFileW path exists only on Windows")]
    fn kismi_replace_bulunamadi_eski_butunu_yedekten_geri_kurar() {
        let klasor = GeciciKlasor::yeni("kismi");
        let hedef = klasor.0.join("durum.txt");
        let gecici = klasor.0.join(".durum.txt.zee-gecici-1-1");
        let yedek = klasor.0.join(".durum.txt.zee-gecici-1-1.zee-yedek");
        std::fs::write(&gecici, "yeni").unwrap();
        // ReplaceFileW'nin ilk adımı: eski hedef → yedek.
        std::fs::write(&yedek, "eski").unwrap();
        kur(Adim::ReplaceFileW, Enjeksiyon::OsHatasi(2));
        let hata = crate::kalici_dosya::atomik_degistir(&gecici, &hedef)
            .expect_err("kısmi replace hata döner");
        assert_eq!(hata_adimi(&hata), Some(Adim::ReplaceFileW), "{hata}");
        assert_eq!(std::fs::read(&hedef).unwrap(), b"eski");
        assert!(!yedek.exists(), "yedek hedefe geri taşınmalı");
    }
}
