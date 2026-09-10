//! Süreçler arası dosya kilidi ve atomik değiştirme için platform katmanı
//! (K-183/ADR-075 ile `kalici_dosya.rs` mimari satır bütçesine sığmak için
//! ayrıldı). Unix `flock`, Windows `LockFileEx`/`ReplaceFileW`, diğerleri
//! desteklenmez.

#[cfg(unix)]
mod uygulama {
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

    /// Çekirdek kuyruğunda bloklayarak alır; EINTR'de yeniden dener (K-183).
    pub fn kilitle_bloklayarak(dosya: &File) -> io::Result<()> {
        loop {
            // SAFETY: geçerli File tanıtıcısı verilir; flock işaretçi kullanmaz.
            if unsafe { flock(dosya.as_raw_fd(), LOCK_EX) } == 0 {
                return Ok(());
            }
            let hata = io::Error::last_os_error();
            if hata.kind() != io::ErrorKind::Interrupted {
                return Err(hata);
            }
        }
    }

    pub fn kilidi_birak(dosya: &File) {
        // SAFETY: kilitlemedeki aynı geçerli File tanıtıcısı kullanılır.
        let _ = unsafe { flock(dosya.as_raw_fd(), LOCK_UN) };
    }
}

#[cfg(windows)]
mod uygulama {
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

    /// FAIL_IMMEDIATELY olmadan bloklayarak alır; Windows bekleyenleri
    /// kendi kuyruğunda uyandırır (K-183).
    pub fn kilitle_bloklayarak(dosya: &File) -> io::Result<()> {
        let mut bilgi = ortusen();
        // SAFETY: File yaşamda, OVERLAPPED çağrı boyunca geçerli ve sıfırlı.
        let sonuc = unsafe {
            LockFileEx(
                dosya.as_raw_handle() as Handle,
                LOCKFILE_EXCLUSIVE_LOCK,
                0,
                u32::MAX,
                u32::MAX,
                &mut bilgi,
            )
        };
        if sonuc != 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
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
mod uygulama {
    use std::fs::File;
    use std::io;

    pub fn kilitlemeyi_dene(_dosya: &File) -> io::Result<bool> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "bu platformda süreçler arası dosya kilidi desteklenmiyor",
        ))
    }

    pub fn kilitle_bloklayarak(_dosya: &File) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "bu platformda süreçler arası dosya kilidi desteklenmiyor",
        ))
    }

    pub fn kilidi_birak(_dosya: &File) {}
}

pub(super) use uygulama::*;
