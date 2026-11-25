//! K-084 — kalıcı dosya değişiklikleri için atomik ve yarış-güvenli çekirdek.
//!
//! Veri önce hedefle aynı klasördeki geçici dosyaya yazılıp diske eşzamanlanır,
//! sonra tek atomik replace ile görünür olur. Klasörün kalıcı kilit dosyası
//! süreçler arasında da kayıp güncellemeyi önler; süreç çökerse işletim sistemi
//! kilidi otomatik bırakır.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::kaynak_sinirlari::degistirme_penceresi as pencere;
use crate::kaynak_sinirlari::degistirme_penceresi::yarim_degistirme as yarim;
use adim::{nokta, sar, Adim};

mod adim;
mod cozulmus;
#[cfg(test)]
pub(crate) use adim::{test_kancasi, Adim as YazimAdimi};
pub use cozulmus::{sinirli_sona_ekle, yeni_dosyaya_yaz};
mod kilitli;
pub use kilitli::{
    karsilastirip_cift_yaz, kilit_altinda_guncelle, CiftYazimHatasi, KilitliGuncellemeHatasi,
};
mod metadata;
mod ozel;
pub use ozel::atomik_ozel_yaz;
mod yasam_dongusu;

pub use yasam_dongusu::{atomik_sil, atomik_sil_izli, atomik_tasi, atomik_tasi_izli};

const KILIT_BEKLEME: Duration = Duration::from_millis(
    crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
        .kalici_dosya()
        .kilit_bekleme_ms(),
);
static GECICI_SAYACI: AtomicU64 = AtomicU64::new(0);

/// İçeriğin tamamını atomik olarak hedefe yerleştirir.
pub fn atomik_yaz(yol: &Path, icerik: &[u8]) -> io::Result<()> {
    let _kilit = DosyaKilidi::al(yol)?;
    atomik_icerik_yaz(yol, icerik, atomik_degistir)
}

/// Hedef hâlâ çağıranın okuduğu byte'lardaysa yeni içeriği atomik yayımlar.
/// Registry gibi oku-doğrula-yaz akışlarında iki sürecin daha eski durumu
/// daha yenisinin üstüne yazmasını engeller.
pub fn atomik_karsilastir_ve_yaz(
    yol: &Path,
    beklenen: Option<&[u8]>,
    yeni: &[u8],
) -> io::Result<()> {
    let _kilit = DosyaKilidi::al(yol)?;
    let guncel = match crate::kaynak_sinirlari::bagsiz_okuma::kalici_ic_baytlari_oku(yol) {
        Ok(icerik) => Some(icerik),
        Err(hata) if hata.kind() == io::ErrorKind::NotFound => None,
        Err(hata) => return Err(hata),
    };
    if guncel.as_deref() != beklenen {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "kalıcı durum doğrulama sırasında başka bir süreççe değiştirildi",
        ));
    }
    atomik_icerik_yaz(yol, yeni, atomik_degistir)
}

/// zee'nin dosyaya yaz/ekle semantiği: satır sonu ekler; ekleme de kilit
/// altında oku-değiştir-atomik replace olduğundan iki süreç veri kaybetmez.
pub fn atomik_satir_yaz(yol: &Path, satir: &str, ekleme: bool) -> io::Result<()> {
    satir_yaz_kilitli(yol, satir, ekleme, false).map(|_| ())
}

/// K-346: eylem savepoint'inin gördüğü değişim. `onceki` ve `sonraki` aynı
/// süreçler-arası kilit altında yakalanır; kilit dışında okunan yedek, araya
/// giren yazarın başarılı eklemesini geri almada siliyordu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KilitliDegisim {
    pub onceki: Option<Vec<u8>>,
    pub sonraki: Option<Vec<u8>>,
}

/// [`atomik_satir_yaz`] ile aynı yazım; hedefin değişimden hemen önceki ve
/// yayımlanan bütün içeriğini aynı kilit penceresinden döndürür (K-346).
pub fn atomik_satir_yaz_izli(yol: &Path, satir: &str, ekleme: bool) -> io::Result<KilitliDegisim> {
    satir_yaz_kilitli(yol, satir, ekleme, true)
}

fn satir_yaz_kilitli(
    yol: &Path,
    satir: &str,
    ekleme: bool,
    izli: bool,
) -> io::Result<KilitliDegisim> {
    let _kilit = DosyaKilidi::al(yol)?;
    let onceki = if ekleme || izli {
        nokta(Adim::MevcutOkuma)
            .and_then(|()| mevcut_icerik(yol))
            .map_err(sar(Adim::MevcutOkuma))?
    } else {
        None
    };
    let mut icerik = if ekleme {
        onceki.clone().unwrap_or_default()
    } else {
        Vec::new()
    };
    icerik.extend_from_slice(satir.as_bytes());
    icerik.push(b'\n');
    atomik_icerik_yaz(yol, &icerik, atomik_degistir)?;
    Ok(KilitliDegisim {
        onceki: if izli { onceki } else { None },
        sonraki: izli.then_some(icerik),
    })
}

fn mevcut_icerik(yol: &Path) -> io::Result<Option<Vec<u8>>> {
    match crate::kaynak_sinirlari::bagsiz_okuma::kalici_ic_baytlari_oku(yol) {
        Ok(icerik) => Ok(Some(icerik)),
        Err(hata) if hata.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(hata) => Err(hata),
    }
}

/// Bir eylemin iyimser geri alması: hedef hâlâ bu eylemin bıraktığı
/// `beklenen` içerikteyse eski içeriği aynı süreçler-arası kilit altında geri
/// koyar. Araya başka bir yazar girdiyse onun verisini ezmek yerine hata verir.
pub fn atomik_karsilastir_ve_geri_al(
    yol: &Path,
    beklenen: Option<&[u8]>,
    onceki: Option<&[u8]>,
) -> io::Result<()> {
    let _kilit = DosyaKilidi::al(yol)?;
    let guncel = match crate::kaynak_sinirlari::bagsiz_okuma::kalici_ic_baytlari_oku(yol) {
        Ok(icerik) => Some(icerik),
        Err(hata) if hata.kind() == io::ErrorKind::NotFound => None,
        Err(hata) => return Err(hata),
    };
    if guncel.as_deref() != beklenen {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "dosya eylem sırasında başka bir yazar tarafından değiştirildi; geri alma onun verisini ezmedi",
        ));
    }
    match onceki {
        Some(icerik) => atomik_icerik_yaz(yol, icerik, atomik_degistir),
        None => match std::fs::remove_file(yol) {
            Ok(()) => klasoru_eszamanla(ebeveyn(yol)),
            Err(hata) if hata.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(hata) => Err(hata),
        },
    }
}

struct DosyaKilidi {
    dosya: File,
    // K-342: bu iş parçacığının kilit altındaki okuması kendi kilidini beklemez.
    _isaret: pencere::YazarIsareti,
}

impl DosyaKilidi {
    fn al(hedef: &Path) -> io::Result<Self> {
        let kilit = Self::kilidi_al(hedef).map_err(sar(Adim::Kilit))?;
        // K-348: önceki yazar ReplaceFileW ortasında öldüyse hedef yoktur;
        // kilit altında eski bütün içerik yedekten geri konur. K-351: hedef
        // varsa başarılı yazımın silinemeyen artık yedeği temizlenir.
        #[cfg(windows)]
        nokta(Adim::Kurtarma)
            .and_then(|()| yasam_dongusu::yarim_degistirmeyi_kurtar(hedef))
            .map_err(sar(Adim::Kurtarma))?;
        Ok(kilit)
    }

    fn kilidi_al(hedef: &Path) -> io::Result<Self> {
        nokta(Adim::Kilit)?;
        let klasor = ebeveyn(hedef);
        let kilit_yolu = klasor.join(pencere::YAZMA_KILIDI_ADI);
        let mut secenek = OpenOptions::new();
        secenek.create(true).truncate(false).read(true).write(true);
        // K-394: paylaşılan klasörde kilit adına konmuş sembolik bağ izlenmez
        // (kök dışında dosya oluşturtma / açtırma yok).
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            secenek.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
        }
        let dosya = secenek.open(kilit_yolu)?;
        // Hızlı yol: boş kilit iş parçacığı açmadan alınır.
        if platform::kilitlemeyi_dene(&dosya)? {
            Ok(Self::tutulan(dosya))
        } else {
            Self::bekleyerek_al(dosya)
        }
    }

    fn tutulan(dosya: File) -> Self {
        let _isaret = pencere::YazarIsareti::koy();
        Self { dosya, _isaret }
    }

    /// K-183/ADR-075: bekleyen yazar 5 ms yoklamaz; çekirdeğin kilit
    /// kuyruğunda bloklanır. Yoklama, kilidi ardışık yeniden alan bir yazarın
    /// diğerini yavaş diskte 5 sn boyunca aç bırakmasına yol açıyordu (CI'da
    /// iki kez). Bloklayan çağrı yardımcı iş parçacığında yapılır ki spec/08'in
    /// 5 sn sınırı korunsun; süre dolarsa geç gelen kilit anında bırakılır.
    fn bekleyerek_al(dosya: File) -> io::Result<Self> {
        let (gonder, al) = std::sync::mpsc::channel::<io::Result<File>>();
        std::thread::Builder::new()
            .name("zee-kilit-bekleyen".into())
            .spawn(move || {
                let sonuc = platform::kilitle_bloklayarak(&dosya).map(|()| dosya);
                if let Err(geri) = gonder.send(sonuc) {
                    if let Ok(dosya) = geri.0 {
                        platform::kilidi_birak(&dosya);
                    }
                }
            })?;
        match al.recv_timeout(adim::kilit_bekleme()) {
            Ok(Ok(dosya)) => Ok(Self::tutulan(dosya)),
            Ok(Err(hata)) => Err(hata),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "zee dosya yazma kilidi 5 saniye içinde alınamadı",
            )),
        }
    }
}

impl Drop for DosyaKilidi {
    fn drop(&mut self) {
        platform::kilidi_birak(&self.dosya);
    }
}

struct GeciciDosya {
    yol: Option<PathBuf>,
}

impl Drop for GeciciDosya {
    fn drop(&mut self) {
        if let Some(yol) = self.yol.take() {
            let _ = std::fs::remove_file(yol);
        }
    }
}

fn ebeveyn(yol: &Path) -> &Path {
    yol.parent()
        .filter(|klasor| !klasor.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn gecici_dosya_ac(hedef: &Path) -> io::Result<(File, GeciciDosya)> {
    gecici_dosyayi_acarak(hedef, |yol| {
        OpenOptions::new().create_new(true).write(true).open(yol)
    })
}

fn gecici_dosyayi_acarak(
    hedef: &Path,
    ac: impl Fn(&Path) -> io::Result<File>,
) -> io::Result<(File, GeciciDosya)> {
    for _ in 0..128 {
        let sira = GECICI_SAYACI.fetch_add(1, Ordering::Relaxed);
        let ad = yarim::gecici_ad(hedef, &format!("{}-{}", std::process::id(), sira))?;
        let yol = ebeveyn(hedef).join(ad);
        match ac(&yol) {
            Ok(dosya) => return Ok((dosya, GeciciDosya { yol: Some(yol) })),
            Err(hata) if hata.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(hata) => return Err(hata),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "benzersiz zee geçici dosyası oluşturulamadı",
    ))
}

fn atomik_icerik_yaz<F>(hedef: &Path, icerik: &[u8], degistir: F) -> io::Result<()>
where
    F: FnOnce(&Path, &Path) -> io::Result<()>,
{
    // K-351: her adımın hatası adını taşır; tür (`kind`) korunur.
    let eski_metadata = nokta(Adim::Metadata)
        .and_then(|()| metadata::MetadataKaynagi::yakala(hedef))
        .map_err(sar(Adim::Metadata))?;
    let (dosya, mut gecici) = nokta(Adim::GeciciYazim)
        .and_then(|()| gecici_dosya_ac(hedef))
        .map_err(sar(Adim::GeciciYazim))?;
    let gecici_yol = gecici
        .yol
        .clone()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "zee geçici dosya yolu kayboldu"))
        .map_err(sar(Adim::GeciciYazim))?;
    gecici_icerigi_hazirla(dosya, icerik, &eski_metadata)?;
    degistir(&gecici_yol, hedef)?;
    gecici.yol = None;
    nokta(Adim::KlasorEszamanlama)
        .and_then(|()| klasoru_eszamanla(ebeveyn(hedef)))
        .map_err(sar(Adim::KlasorEszamanlama))
}

fn gecici_icerigi_hazirla(
    mut dosya: File,
    icerik: &[u8],
    metadata: &metadata::MetadataKaynagi,
) -> io::Result<()> {
    dosya.write_all(icerik).map_err(sar(Adim::GeciciYazim))?;
    metadata
        .geciciye_uygula(&dosya)
        .map_err(sar(Adim::Metadata))?;
    dosya.sync_all().map_err(sar(Adim::GeciciYazim))
}

#[cfg(unix)]
fn klasoru_eszamanla(klasor: &Path) -> io::Result<()> {
    File::open(klasor)?.sync_all()
}

#[cfg(not(unix))]
fn klasoru_eszamanla(_klasor: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(not(windows))]
fn atomik_degistir(gecici: &Path, hedef: &Path) -> io::Result<()> {
    nokta(Adim::Rename)
        .and_then(|()| std::fs::rename(gecici, hedef))
        .map_err(sar(Adim::Rename))
}

#[cfg(windows)]
fn atomik_degistir(gecici: &Path, hedef: &Path) -> io::Result<()> {
    platform::atomik_degistir(gecici, hedef)
}

mod platform;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    use std::collections::HashSet;
    use std::sync::atomic::{AtomicBool, Ordering as AtomikSira};
    use std::sync::{Arc, Barrier};

    pub(super) struct GeciciKlasor(pub(super) PathBuf);

    impl GeciciKlasor {
        pub(super) fn yeni() -> Self {
            let sira = GECICI_SAYACI.fetch_add(1, Ordering::Relaxed);
            let yol = std::env::temp_dir().join(format!(
                "zee-atomik-test-{}-{}",
                std::process::id(),
                sira
            ));
            std::fs::create_dir(&yol).expect("geçici klasör");
            Self(yol)
        }
    }

    impl Drop for GeciciKlasor {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn yaz_ve_ekle_semantigi_korunur() {
        let gecici = GeciciKlasor::yeni();
        let yol = gecici.0.join("gunluk.txt");
        atomik_satir_yaz(&yol, "ilk", false).expect("ilk yazma");
        atomik_satir_yaz(&yol, "ikinci", true).expect("ekleme");
        assert_eq!(std::fs::read_to_string(yol).unwrap(), "ilk\nikinci\n");
    }

    #[cfg(unix)]
    #[test]
    fn replace_mod_sahip_ve_grubu_korur() {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let gecici = GeciciKlasor::yeni();
        let yol = gecici.0.join("izinli.txt");
        std::fs::write(&yol, "eski").unwrap();
        std::fs::set_permissions(&yol, std::fs::Permissions::from_mode(0o640)).unwrap();
        let once = std::fs::metadata(&yol).unwrap();

        atomik_yaz(&yol, b"yeni").expect("metadata korunarak replace");
        let sonra = std::fs::metadata(&yol).unwrap();
        assert_eq!(sonra.mode() & 0o7777, once.mode() & 0o7777);
        assert_eq!((sonra.uid(), sonra.gid()), (once.uid(), once.gid()));
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn replace_genisletilmis_ozniteligi_korur() {
        let gecici = GeciciKlasor::yeni();
        let yol = gecici.0.join("xattr.txt");
        std::fs::write(&yol, "eski").unwrap();
        test_xattr_yaz(&yol, "user.zee.miras", b"Eliz").unwrap();

        atomik_yaz(&yol, b"yeni").expect("xattr korunarak replace");
        assert_eq!(test_xattr_oku(&yol, "user.zee.miras").unwrap(), b"Eliz");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn replace_macos_acl_kuralini_korur() {
        let gecici = GeciciKlasor::yeni();
        let yol = gecici.0.join("acl.txt");
        std::fs::write(&yol, "eski").unwrap();
        let kullanici = std::env::var("USER").expect("macOS kullanıcı adı");
        let kural = format!("{} allow read", kullanici);
        assert!(std::process::Command::new("chmod")
            .arg("+a")
            .arg(&kural)
            .arg(&yol)
            .status()
            .expect("chmod")
            .success());

        atomik_yaz(&yol, b"yeni").expect("ACL korunarak replace");
        let cikti = std::process::Command::new("ls")
            .arg("-le")
            .arg(&yol)
            .output()
            .expect("ls -le");
        assert!(String::from_utf8_lossy(&cikti.stdout)
            .contains(&format!("user:{} allow read", kullanici)));
    }

    #[cfg(unix)]
    #[test]
    fn sembolik_bag_hedefi_sessizce_normal_dosyaya_donusmez() {
        let gecici = GeciciKlasor::yeni();
        let asil = gecici.0.join("asil.txt");
        let bag = gecici.0.join("bag.txt");
        std::fs::write(&asil, "eski").unwrap();
        std::os::unix::fs::symlink(&asil, &bag).unwrap();

        let hata = atomik_yaz(&bag, b"yeni").expect_err("symlink reddedilmeli");
        assert_eq!(hata.kind(), io::ErrorKind::InvalidInput);
        assert!(std::fs::symlink_metadata(&bag)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(std::fs::read_to_string(asil).unwrap(), "eski");
    }

    #[cfg(windows)]
    #[test]
    fn replace_windows_named_stream_metadata_sini_korur() {
        let gecici = GeciciKlasor::yeni();
        let yol = gecici.0.join("stream.txt");
        let stream = PathBuf::from(format!("{}:zee-miras", yol.display()));
        std::fs::write(&yol, "eski").unwrap();
        std::fs::write(&stream, "Eliz").unwrap();

        atomik_yaz(&yol, b"yeni").expect("ReplaceFileW metadata merge");
        assert_eq!(std::fs::read_to_string(stream).unwrap(), "Eliz");
    }

    #[test]
    fn geri_alma_araya_giren_yazari_ezmez() {
        let gecici = GeciciKlasor::yeni();
        let yol = gecici.0.join("durum.txt");
        atomik_yaz(&yol, b"eylem-oncesi").expect("ilk durum");
        atomik_yaz(&yol, b"eylem-yazdi").expect("eylem yazısı");
        atomik_yaz(&yol, b"baska-yazar").expect("araya giren yazar");

        let hata = atomik_karsilastir_ve_geri_al(&yol, Some(b"eylem-yazdi"), Some(b"eylem-oncesi"))
            .expect_err("çakışma sessizce ezilmemeli");
        assert_eq!(hata.kind(), io::ErrorKind::WouldBlock);
        assert_eq!(std::fs::read(&yol).unwrap(), b"baska-yazar");
    }

    #[test]
    fn karsilastir_ve_yaz_bayat_registry_durumunu_ezmez() {
        let gecici = GeciciKlasor::yeni();
        let yol = gecici.0.join("registry-durumu.json");
        atomik_karsilastir_ve_yaz(&yol, None, b"surum-1").expect("ilk durum");
        atomik_karsilastir_ve_yaz(&yol, Some(b"surum-1"), b"surum-2").expect("ileri durum");

        let hata = atomik_karsilastir_ve_yaz(&yol, Some(b"surum-1"), b"bayat")
            .expect_err("bayat yazar reddedilmeli");
        assert_eq!(hata.kind(), io::ErrorKind::WouldBlock);
        assert_eq!(std::fs::read(&yol).unwrap(), b"surum-2");
    }

    #[test]
    fn degisim_hatasi_eski_veriyi_korur_ve_geciciyi_siler() {
        let gecici = GeciciKlasor::yeni();
        let yol = gecici.0.join("durum.json");
        std::fs::write(&yol, "eski ve bütün").unwrap();
        let hata = atomik_icerik_yaz(&yol, b"yeni", |_, _| {
            Err(io::Error::other("enjekte edilen replace hatası"))
        })
        .expect_err("hata bekleniyor");
        assert_eq!(hata.kind(), io::ErrorKind::Other);
        assert_eq!(std::fs::read_to_string(&yol).unwrap(), "eski ve bütün");
        let artik = std::fs::read_dir(&gecici.0)
            .unwrap()
            .filter_map(Result::ok)
            .any(|girdi| girdi.file_name().to_string_lossy().contains("zee-gecici"));
        assert!(!artik, "başarısız değişim geçici dosya bırakmamalı");
    }

    #[test]
    fn iki_yazar_satir_kaybetmez() {
        let gecici = GeciciKlasor::yeni();
        let yol = Arc::new(gecici.0.join("olaylar.txt"));
        let basla = Arc::new(Barrier::new(3));
        let mut kollar = Vec::new();
        for onek in ["a", "b"] {
            let yol = Arc::clone(&yol);
            let basla = Arc::clone(&basla);
            // K-183/ADR-075: 2×40 ardışık fsync'li ekleme; bekleyen yazar
            // çekirdek kuyruğunda bloklandığından yavaş diskte de aç kalmaz.
            kollar.push(std::thread::spawn(move || {
                basla.wait();
                for sayi in 0..40 {
                    atomik_satir_yaz(&yol, &format!("{}-{}", onek, sayi), true)
                        .expect("yarışlı ekleme");
                }
            }));
        }
        basla.wait();
        for kol in kollar {
            kol.join().expect("yazar düşmemeli");
        }
        let satirlar = std::fs::read_to_string(&*yol)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect::<HashSet<_>>();
        assert_eq!(satirlar.len(), 80);
        for onek in ["a", "b"] {
            for sayi in 0..40 {
                assert!(satirlar.contains(&format!("{}-{}", onek, sayi)));
            }
        }
    }

    #[test]
    fn izli_yazim_ve_silme_kilit_altindaki_onceki_ve_sonrakini_dondurur() {
        // K-346: eylem defteri yedeği yazımla aynı kilit penceresinden alır.
        let gecici = GeciciKlasor::yeni();
        let yol = gecici.0.join("defter.txt");
        let ilk = atomik_satir_yaz_izli(&yol, "a", true).expect("ilk ekleme");
        assert_eq!(ilk.onceki, None);
        assert_eq!(ilk.sonraki.as_deref(), Some(&b"a\n"[..]));
        let ikinci = atomik_satir_yaz_izli(&yol, "b", false).expect("baştan yazma");
        assert_eq!(ikinci.onceki.as_deref(), Some(&b"a\n"[..]));
        assert_eq!(ikinci.sonraki.as_deref(), Some(&b"b\n"[..]));
        let silme = atomik_sil_izli(&yol).expect("silme");
        assert_eq!(silme.onceki.as_deref(), Some(&b"b\n"[..]));
        assert_eq!(silme.sonraki, None);
        assert!(!yol.exists());
        assert_eq!(atomik_sil_izli(&yol).expect("yok").onceki, None);
    }

    #[test]
    fn kilidi_ardisik_yeniden_alan_yazar_bekleyeni_ac_birakmaz() {
        // K-183/ADR-075: A durmadan ekler; B tek bir ekleme için en çok bir
        // yazar kuyruğuna katılır (5 ms yoklamayla B yavaş diskte aç
        // kalıyordu). Shared CI 2 sn disk performansı garantilemez; K-217
        // Gitsrv'de 3,593 sn gözlendi. Correctness sınırı spec/08'in 5 sn
        // kilit bütçesidir; test benchmark hard eşiği koymaz.
        let gecici = GeciciKlasor::yeni();
        let yol = Arc::new(gecici.0.join("kuyruk.txt"));
        let basla = Arc::new(Barrier::new(2));
        let yazar_yolu = Arc::clone(&yol);
        let yazar_basla = Arc::clone(&basla);
        let yazar = std::thread::spawn(move || {
            yazar_basla.wait();
            for sayi in 0..120 {
                atomik_satir_yaz(&yazar_yolu, &format!("a-{sayi}"), true).expect("A ekler");
            }
        });
        basla.wait();
        std::thread::sleep(Duration::from_millis(30));
        let baslangic = Instant::now();
        atomik_satir_yaz(&yol, "b-0", true).expect("B ekler");
        let bekleme = baslangic.elapsed();
        yazar.join().expect("A düşmemeli");
        assert!(
            bekleme < KILIT_BEKLEME,
            "bekleyen yazar aç bırakıldı: {bekleme:?}"
        );
        let icerik = std::fs::read_to_string(&*yol).unwrap();
        assert!(icerik.lines().any(|s| s == "b-0"));
        assert_eq!(icerik.lines().count(), 121);
    }

    #[test]
    fn okuyucu_yalniz_eski_ya_da_yeni_butunu_gorur() {
        let gecici = GeciciKlasor::yeni();
        let yol = Arc::new(gecici.0.join("anlik-goruntu.bin"));
        let a = vec![b'a'; 32 * 1024];
        let b = vec![b'b'; 32 * 1024];
        atomik_yaz(&yol, &a).expect("ilk görüntü");

        let basla = Arc::new(Barrier::new(2));
        let bitti = Arc::new(AtomicBool::new(false));
        let yazar_yolu = Arc::clone(&yol);
        let yazar_basla = Arc::clone(&basla);
        let yazar_bitti = Arc::clone(&bitti);
        let yazar_a = a.clone();
        let yazar_b = b.clone();
        let yazar = std::thread::spawn(move || {
            yazar_basla.wait();
            for sira in 0..20 {
                atomik_yaz(&yazar_yolu, if sira % 2 == 0 { &yazar_b } else { &yazar_a })
                    .expect("atomik görüntü");
            }
            yazar_bitti.store(true, AtomikSira::Release);
        });

        basla.wait();
        // Okuyucu ürünün kendi okuma yolunu kullanır: Windows'ta ReplaceFileW
        // penceresini kapatan kısa yeniden deneme oradadır (K-182).
        while !bitti.load(AtomikSira::Acquire) {
            let gorulen = crate::kaynak_sinirlari::veri_dosyasi_baytlarini_oku(&yol)
                .expect("eşzamanlı okuma");
            assert!(
                gorulen == a || gorulen == b,
                "okuyucu kısmi/karışık içerik görmemeli"
            );
        }
        yazar.join().expect("yazar düşmemeli");
    }

    /// K-342: yüklü Windows'ta `ReplaceFileW` penceresi kısa bütçeyi aşabilir
    /// ve iki değiştirmenin arasında geçici dosya izi görünmeyebilir. Okuyucu
    /// zee yazma kilidini paylaşımlı bekler; kilit sürdükçe hata görmez.
    #[test]
    fn okuyucu_suren_degistirmeyi_yazma_kilidiyle_bekler() {
        let gecici = GeciciKlasor::yeni();
        let hedef = Arc::new(gecici.0.join("durum.json"));
        let pencere_acik = Arc::new(AtomicBool::new(true));
        let basla = Arc::new(Barrier::new(2));
        let tutma = pencere::KISA_BUTCE * 5;
        let yazar = {
            let (hedef, acik, basla) = (
                Arc::clone(&hedef),
                Arc::clone(&pencere_acik),
                Arc::clone(&basla),
            );
            std::thread::spawn(move || {
                let kilit = DosyaKilidi::al(&hedef).expect("yazar kilidi");
                basla.wait();
                std::thread::sleep(tutma);
                // Gerçek yazar gibi: değiştirme biter, sonra kilit bırakılır.
                acik.store(false, AtomikSira::Release);
                drop(kilit);
            })
        };
        basla.wait();
        let baslangic = Instant::now();
        let sonuc = pencere::pencereyi_bekleyerek(&hedef, KILIT_BEKLEME, || {
            match pencere_acik.load(AtomikSira::Acquire) {
                true => Err(io::Error::from_raw_os_error(32)),
                false => Ok("yeni bütün"),
            }
        });
        yazar.join().expect("yazar düşmemeli");
        assert_eq!(sonuc.expect("değiştirme bitince okunur"), "yeni bütün");
        assert!(baslangic.elapsed() >= tutma - Duration::from_millis(20));
    }

    #[test]
    fn kilitsiz_takili_ya_da_kendi_kilidinde_pencere_sinirli_hata_verir() {
        let gecici = GeciciKlasor::yeni();
        let hedef = gecici.0.join("yok.txt");
        let hep = |kod| move || Err::<(), _>(io::Error::from_raw_os_error(kod));
        // Kilit dosyası yok: klasörde zee değiştirmesi olmamıştır.
        let baslangic = Instant::now();
        let hata = pencere::pencereyi_bekleyerek(&hedef, KILIT_BEKLEME, hep(2))
            .expect_err("gerçekten olmayan dosya");
        assert_eq!(hata.raw_os_error(), Some(2));
        assert!(baslangic.elapsed() < KILIT_BEKLEME / 2);

        // Aynı iş parçacığı kilidi tutuyor (ekleme okuması): kendini beklemez.
        let kilit = DosyaKilidi::al(&hedef).expect("kilit");
        let baslangic = Instant::now();
        let hata =
            pencere::pencereyi_bekleyerek(&hedef, KILIT_BEKLEME, hep(32)).expect_err("dış tutucu");
        assert_eq!(hata.raw_os_error(), Some(32));
        assert!(
            baslangic.elapsed() < KILIT_BEKLEME / 2,
            "kendi kilidini bekledi"
        );

        // Başka iş parçacığı kilidi bırakmıyor: okuyucu bütçeyi aşamaz.
        let uzun = pencere::KISA_BUTCE * 4;
        let hata = std::thread::scope(|kapsam| {
            kapsam
                .spawn(|| {
                    let baslangic = Instant::now();
                    let hata = pencere::pencereyi_bekleyerek(&hedef, uzun, hep(32))
                        .expect_err("takılı yazar bütçeyi aşamaz");
                    assert!(baslangic.elapsed() >= uzun);
                    assert!(baslangic.elapsed() < KILIT_BEKLEME);
                    hata
                })
                .join()
                .expect("okuyucu")
        });
        assert_eq!(hata.raw_os_error(), Some(32));
        drop(kilit);

        let mut cagri = 0;
        let hata = pencere::pencereyi_bekleyerek(&hedef, uzun, || {
            cagri += 1;
            Err::<(), _>(io::Error::from_raw_os_error(87))
        })
        .expect_err("pencere dışı hata");
        assert_eq!((hata.raw_os_error(), cagri), (Some(87), 1));
    }

    #[test]
    fn aniden_biten_surecin_kilidi_isletim_sistemi_birakir() {
        const COCUK_YOLU: &str = "ZEE_ATOMIK_KILIT_COCUK_YOLU";
        if let Some(yol) = std::env::var_os(COCUK_YOLU) {
            let _kilit = DosyaKilidi::al(Path::new(&yol)).expect("çocuk kilidi");
            // Drop çalıştırmadan çık: işletim sistemi süreç kilidini bırakmalı.
            std::process::exit(91);
        }

        let gecici = GeciciKlasor::yeni();
        let yol = gecici.0.join("durum.txt");
        let durum = std::process::Command::new(std::env::current_exe().expect("test ikilisi"))
            .args([
                "--exact",
                "kalici_dosya::tests::aniden_biten_surecin_kilidi_isletim_sistemi_birakir",
            ])
            .env(COCUK_YOLU, &yol)
            .status()
            .expect("çocuk süreç");
        assert_eq!(durum.code(), Some(91));

        atomik_satir_yaz(&yol, "çöküşten sonra sağlam", false)
            .expect("işletim sistemi kilidi bırakmalı");
        assert_eq!(
            std::fs::read_to_string(yol).unwrap(),
            "çöküşten sonra sağlam\n"
        );
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn test_xattr_yaz(yol: &Path, ad: &str, deger: &[u8]) -> io::Result<()> {
        use std::ffi::CString;
        use std::os::fd::AsRawFd;

        let dosya = OpenOptions::new().read(true).write(true).open(yol)?;
        let ad = CString::new(ad).expect("sabit xattr adı");
        #[cfg(target_os = "linux")]
        // SAFETY: File/CString/tampon çağrı boyunca geçerlidir.
        let sonuc = unsafe {
            libc::fsetxattr(
                dosya.as_raw_fd(),
                ad.as_ptr(),
                deger.as_ptr().cast(),
                deger.len(),
                0,
            )
        };
        #[cfg(target_os = "macos")]
        // SAFETY: File/CString/tampon çağrı boyunca geçerlidir.
        let sonuc = unsafe {
            libc::fsetxattr(
                dosya.as_raw_fd(),
                ad.as_ptr(),
                deger.as_ptr().cast(),
                deger.len(),
                0,
                0,
            )
        };
        if sonuc == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn test_xattr_oku(yol: &Path, ad: &str) -> io::Result<Vec<u8>> {
        use std::ffi::CString;
        use std::os::fd::AsRawFd;

        let dosya = File::open(yol)?;
        let ad = CString::new(ad).expect("sabit xattr adı");
        #[cfg(target_os = "linux")]
        // SAFETY: İlk çağrı yalnız boyutu sorgular.
        let boyut =
            unsafe { libc::fgetxattr(dosya.as_raw_fd(), ad.as_ptr(), std::ptr::null_mut(), 0) };
        #[cfg(target_os = "macos")]
        // SAFETY: İlk çağrı yalnız boyutu sorgular.
        let boyut = unsafe {
            libc::fgetxattr(
                dosya.as_raw_fd(),
                ad.as_ptr(),
                std::ptr::null_mut(),
                0,
                0,
                0,
            )
        };
        if boyut < 0 {
            return Err(io::Error::last_os_error());
        }
        let mut deger = vec![0; usize::try_from(boyut).unwrap()];
        #[cfg(target_os = "linux")]
        // SAFETY: Tampon sorgulanan boyuttadır.
        let okunan = unsafe {
            libc::fgetxattr(
                dosya.as_raw_fd(),
                ad.as_ptr(),
                deger.as_mut_ptr().cast(),
                deger.len(),
            )
        };
        #[cfg(target_os = "macos")]
        // SAFETY: Tampon sorgulanan boyuttadır.
        let okunan = unsafe {
            libc::fgetxattr(
                dosya.as_raw_fd(),
                ad.as_ptr(),
                deger.as_mut_ptr().cast(),
                deger.len(),
                0,
                0,
            )
        };
        if okunan < 0 {
            return Err(io::Error::last_os_error());
        }
        deger.truncate(usize::try_from(okunan).unwrap());
        Ok(deger)
    }
}
