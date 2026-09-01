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

const KILIT_BEKLEME: Duration = Duration::from_secs(5);
const KILIT_YENIDEN_DENE: Duration = Duration::from_millis(5);
static GECICI_SAYACI: AtomicU64 = AtomicU64::new(0);

/// İçeriğin tamamını atomik olarak hedefe yerleştirir.
pub fn atomik_yaz(yol: &Path, icerik: &[u8]) -> io::Result<()> {
    let _kilit = DosyaKilidi::al(yol)?;
    atomik_icerik_yaz(yol, icerik, atomik_degistir)
}

/// zee'nin dosyaya yaz/ekle semantiği: satır sonu ekler; ekleme de kilit
/// altında oku-değiştir-atomik replace olduğundan iki süreç veri kaybetmez.
pub fn atomik_satir_yaz(yol: &Path, satir: &str, ekleme: bool) -> io::Result<()> {
    let _kilit = DosyaKilidi::al(yol)?;
    let mut icerik = if ekleme {
        match std::fs::read(yol) {
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
    let (mut dosya, mut gecici) = gecici_dosya_ac(hedef)?;
    dosya.write_all(icerik)?;
    if let Ok(eski) = std::fs::metadata(hedef) {
        std::fs::set_permissions(
            gecici.yol.as_deref().expect("geçici yol var"),
            eski.permissions(),
        )?;
    }
    dosya.sync_all()?;
    drop(dosya);

    let gecici_yol = gecici.yol.as_deref().expect("geçici yol var");
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
    use std::path::Path;

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
    const ERROR_LOCK_VIOLATION: i32 = 33;

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
        // SAFETY: Diziler NUL ile sonlandırılmış ve çağrı boyunca yaşamda.
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
}
