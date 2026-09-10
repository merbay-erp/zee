//! K-084 — kalıcı dosya değişiklikleri için atomik ve yarış-güvenli çekirdek.
//!
//! Veri önce hedefle aynı klasördeki geçici dosyaya yazılıp diske eşzamanlanır,
//! sonra tek atomik replace ile görünür olur. Klasörün kalıcı kilit dosyası
//! süreçler arasında da kayıp güncellemeyi önler; süreç çökerse işletim sistemi
//! kilidi otomatik bırakır.

use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

mod metadata;
mod yasam_dongusu;

pub use yasam_dongusu::{atomik_sil, atomik_tasi};

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
    let guncel = match crate::kaynak_sinirlari::veri_dosyasi_baytlarini_oku(yol) {
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
    let _kilit = DosyaKilidi::al(yol)?;
    let mut icerik = if ekleme {
        match crate::kaynak_sinirlari::veri_dosyasi_baytlarini_oku(yol) {
            Ok(icerik) => icerik,
            Err(hata) if hata.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(hata) => return Err(hata),
        }
    } else {
        Vec::new()
    };
    icerik.extend_from_slice(satir.as_bytes());
    icerik.push(b'\n');
    atomik_icerik_yaz(yol, &icerik, atomik_degistir)
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
    let guncel = match crate::kaynak_sinirlari::veri_dosyasi_baytlarini_oku(yol) {
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
}

impl DosyaKilidi {
    fn al(hedef: &Path) -> io::Result<Self> {
        let klasor = ebeveyn(hedef);
        let kilit_yolu = klasor.join(".zee-yazma-kilidi");
        let dosya = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(kilit_yolu)?;
        // Hızlı yol: boş kilit iş parçacığı açmadan alınır.
        if platform::kilitlemeyi_dene(&dosya)? {
            return Ok(Self { dosya });
        }
        Self::bekleyerek_al(dosya)
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
        match al.recv_timeout(KILIT_BEKLEME) {
            Ok(Ok(dosya)) => Ok(Self { dosya }),
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
    let dosya_adi = hedef.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "hedef bir dosya adı taşımalı")
    })?;
    for _ in 0..128 {
        let sira = GECICI_SAYACI.fetch_add(1, Ordering::Relaxed);
        let mut ad = OsString::from(".");
        ad.push(dosya_adi);
        ad.push(format!(".zee-gecici-{}-{}", std::process::id(), sira));
        let yol = ebeveyn(hedef).join(ad);
        match OpenOptions::new().create_new(true).write(true).open(&yol) {
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
    let eski_metadata = metadata::MetadataKaynagi::yakala(hedef)?;
    let (mut dosya, mut gecici) = gecici_dosya_ac(hedef)?;
    dosya.write_all(icerik)?;
    let gecici_yol = gecici
        .yol
        .as_deref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "zee geçici dosya yolu kayboldu"))?;
    eski_metadata.geciciye_uygula(&dosya)?;
    dosya.sync_all()?;
    drop(dosya);

    degistir(gecici_yol, hedef)?;
    gecici.yol = None;
    klasoru_eszamanla(ebeveyn(hedef))
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
    std::fs::rename(gecici, hedef)
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

    struct GeciciKlasor(PathBuf);

    impl GeciciKlasor {
        fn yeni() -> Self {
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
    fn kilidi_ardisik_yeniden_alan_yazar_bekleyeni_ac_birakmaz() {
        // K-183/ADR-075: A durmadan ekler; B tek bir ekleme için en çok bir
        // A yazması kadar bekler (5 ms yoklamayla B yavaş diskte 5 sn aç
        // kalıyordu). Eşik 2 sn: en yavaş CI diskinde bile tek yazmanın çok
        // üstü, spec/08 5 sn sınırının altında.
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
            bekleme < Duration::from_secs(2),
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
