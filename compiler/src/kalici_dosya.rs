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
use std::time::{Duration, Instant};

mod metadata;

const KILIT_BEKLEME: Duration = Duration::from_millis(
    crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
        .kalici_dosya()
        .kilit_bekleme_ms(),
);
const KILIT_YENIDEN_DENE: Duration = Duration::from_millis(
    crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI
        .kalici_dosya()
        .kilit_yeniden_dene_ms(),
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
        let baslangic = Instant::now();
        loop {
            match platform::kilitlemeyi_dene(&dosya) {
                Ok(true) => return Ok(Self { dosya }),
                Ok(false) if baslangic.elapsed() < KILIT_BEKLEME => {
                    std::thread::sleep(KILIT_YENIDEN_DENE);
                }
                Ok(false) => {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "zee dosya yazma kilidi 5 saniye içinde alınamadı",
                    ));
                }
                Err(hata) => return Err(hata),
            }
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

#[cfg(unix)]
mod platform {
    use std::fs::File;
    use std::io;
    use std::os::fd::AsRawFd;

    const LOCK_EX: i32 = 2;
    const LOCK_NB: i32 = 4;
    const LOCK_UN: i32 = 8;

    extern "C" {
        fn flock(fd: i32, islem: i32) -> i32;
    }

    pub fn kilitlemeyi_dene(dosya: &File) -> io::Result<bool> {
        // SAFETY: geçerli File tanıtıcısı verilir; flock işaretçi kullanmaz.
        let sonuc = unsafe { flock(dosya.as_raw_fd(), LOCK_EX | LOCK_NB) };
        if sonuc == 0 {
            return Ok(true);
        }
        let hata = io::Error::last_os_error();
        if hata.kind() == io::ErrorKind::WouldBlock {
            Ok(false)
        } else {
            Err(hata)
        }
    }

    pub fn kilidi_birak(dosya: &File) {
        // SAFETY: kilitlemedeki aynı geçerli File tanıtıcısı kullanılır.
        let _ = unsafe { flock(dosya.as_raw_fd(), LOCK_UN) };
    }
}

#[cfg(windows)]
mod platform {
    use std::ffi::c_void;
    use std::fs::File;
    use std::io;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::AsRawHandle;
    use std::path::{Path, PathBuf};

    type Handle = *mut c_void;

    #[repr(C)]
    struct Overlapped {
        internal: usize,
        internal_high: usize,
        offset: u32,
        offset_high: u32,
        olay: Handle,
    }

    const LOCKFILE_FAIL_IMMEDIATELY: u32 = 0x00000001;
    const LOCKFILE_EXCLUSIVE_LOCK: u32 = 0x00000002;
    const MOVEFILE_REPLACE_EXISTING: u32 = 0x00000001;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x00000008;
    const ERROR_FILE_NOT_FOUND: i32 = 2;
    const ERROR_PATH_NOT_FOUND: i32 = 3;
    const ERROR_LOCK_VIOLATION: i32 = 33;
    const ERROR_UNABLE_TO_MOVE_REPLACEMENT_2: i32 = 1177;

    #[link(name = "kernel32")]
    extern "system" {
        fn LockFileEx(
            dosya: Handle,
            bayraklar: u32,
            ayrilmis: u32,
            dusuk: u32,
            yuksek: u32,
            ortusen: *mut Overlapped,
        ) -> i32;
        fn UnlockFileEx(
            dosya: Handle,
            ayrilmis: u32,
            dusuk: u32,
            yuksek: u32,
            ortusen: *mut Overlapped,
        ) -> i32;
        fn MoveFileExW(eski: *const u16, yeni: *const u16, bayraklar: u32) -> i32;
        fn ReplaceFileW(
            degistirilen: *const u16,
            yeni: *const u16,
            yedek: *const u16,
            bayraklar: u32,
            dislanan: *mut c_void,
            ayrilmis: *mut c_void,
        ) -> i32;
    }

    fn ortusen() -> Overlapped {
        Overlapped {
            internal: 0,
            internal_high: 0,
            offset: 0,
            offset_high: 0,
            olay: std::ptr::null_mut(),
        }
    }

    pub fn kilitlemeyi_dene(dosya: &File) -> io::Result<bool> {
        let mut bilgi = ortusen();
        // SAFETY: File yaşamda, OVERLAPPED çağrı boyunca geçerli ve ayrılmış
        // alanlar Win32 sözleşmesine uygun sıfırdır.
        let sonuc = unsafe {
            LockFileEx(
                dosya.as_raw_handle() as Handle,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                u32::MAX,
                u32::MAX,
                &mut bilgi,
            )
        };
        if sonuc != 0 {
            return Ok(true);
        }
        let hata = io::Error::last_os_error();
        if hata.raw_os_error() == Some(ERROR_LOCK_VIOLATION)
            || hata.kind() == io::ErrorKind::WouldBlock
        {
            Ok(false)
        } else {
            Err(hata)
        }
    }

    pub fn kilidi_birak(dosya: &File) {
        let mut bilgi = ortusen();
        // SAFETY: Kilit için kullanılan aynı File ve geçerli OVERLAPPED.
        let _ = unsafe {
            UnlockFileEx(
                dosya.as_raw_handle() as Handle,
                0,
                u32::MAX,
                u32::MAX,
                &mut bilgi,
            )
        };
    }

    pub fn atomik_degistir(gecici: &Path, hedef: &Path) -> io::Result<()> {
        let eski = genis_yol(gecici)?;
        let yeni = genis_yol(hedef)?;
        let mut yedek_adi = gecici.as_os_str().to_os_string();
        yedek_adi.push(".zee-yedek");
        let yedek = PathBuf::from(yedek_adi);
        if yedek.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "benzersiz Windows metadata kurtarma yedeği oluşturulamadı",
            ));
        }
        let yedek_genis = genis_yol(&yedek)?;
        // ReplaceFileW var olan hedefin DACL, security attributes, şifreleme,
        // sıkıştırma ve named stream metadata'sını taşır. Merge hatalarını
        // yoksayan hiçbir bayrak verilmez; yedek nadir kısmi-hata durumunda
        // eski inode'u geri getirebilmek içindir.
        // SAFETY: Diziler NUL ile sonlandırılmış ve çağrı boyunca yaşamda.
        let sonuc = unsafe {
            ReplaceFileW(
                yeni.as_ptr(),
                eski.as_ptr(),
                yedek_genis.as_ptr(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if sonuc != 0 {
            return std::fs::remove_file(yedek);
        }
        let hata = io::Error::last_os_error();
        if hata.raw_os_error() == Some(ERROR_UNABLE_TO_MOVE_REPLACEMENT_2) && yedek.exists() {
            // SAFETY: NUL sonlu yollar geçerli; hedef eski yedekten atomik
            // write-through taşımayla geri kurulur.
            let geri_alindi = unsafe {
                MoveFileExW(
                    yedek_genis.as_ptr(),
                    yeni.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            };
            if geri_alindi == 0 {
                return Err(io::Error::other(format!(
                    "Windows replace başarısız oldu ({hata}); eski dosya `{}` yedeğinde kaldı: {}",
                    yedek.display(),
                    io::Error::last_os_error()
                )));
            }
        }
        if !matches!(
            hata.raw_os_error(),
            Some(ERROR_FILE_NOT_FOUND) | Some(ERROR_PATH_NOT_FOUND)
        ) {
            return Err(hata);
        }
        let _ = std::fs::remove_file(&yedek);
        // Hedef henüz yoksa metadata taşıma gerekmez; ilk yerleştirme yine
        // aynı hacimde atomik ve write-through MoveFileExW ile yapılır.
        let sonuc = unsafe {
            MoveFileExW(
                eski.as_ptr(),
                yeni.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        };
        if sonuc != 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn genis_yol(yol: &Path) -> io::Result<Vec<u16>> {
        let mut genis = yol.as_os_str().encode_wide().collect::<Vec<_>>();
        if genis.contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "dosya yolu NUL içeremez",
            ));
        }
        genis.push(0);
        Ok(genis)
    }
}

#[cfg(not(any(unix, windows)))]
mod platform {
    use std::fs::File;
    use std::io;

    pub fn kilitlemeyi_dene(_dosya: &File) -> io::Result<bool> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "bu platformda süreçler arası dosya kilidi desteklenmiyor",
        ))
    }

    pub fn kilidi_birak(_dosya: &File) {}
}

#[cfg(test)]
mod tests {
    use super::*;
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
        while !bitti.load(AtomikSira::Acquire) {
            let gorulen = std::fs::read(&*yol).expect("eşzamanlı okuma");
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
