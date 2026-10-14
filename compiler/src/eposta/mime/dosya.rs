//! Yetki/yol denetiminden sonra yalnız normal readonly ek dosyasını açar.
use std::fs::{File, OpenOptions};
use std::path::Path;

pub fn dosyayi_guvenle_ac(yol: &Path) -> Result<File, String> {
    let once = std::fs::symlink_metadata(yol).map_err(|_| "MIME ek bilgisi okunamadı")?;
    if !once.is_file() || once.file_type().is_symlink() {
        return Err("MIME ek normal dosya olmalı".into());
    }
    let mut ac = OpenOptions::new();
    ac.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        ac.custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        ac.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    #[cfg(windows)]
    let (_once_dosya, once_kimlik) = {
        use std::os::windows::fs::OpenOptionsExt;
        let meta = OpenOptions::new()
            .access_mode(0x80) // FILE_READ_ATTRIBUTES
            .custom_flags(0x0020_0000)
            .open(yol)
            .map_err(|_| "MIME ek kimliği okunamadı")?;
        let bilgi = meta.metadata().map_err(|_| "MIME ek bilgisi okunamadı")?;
        if !bilgi.is_file() || bilgi.file_type().is_symlink() {
            return Err("MIME ek normal dosya olmalı".into());
        }
        let kimlik = windows_kimligi(&meta)?;
        (meta, kimlik)
    };
    #[cfg(not(any(unix, windows)))]
    return Err("Güvenli MIME ek açma bu platformda desteklenmiyor".into());
    #[cfg(any(unix, windows))]
    {
        let dosya = ac.open(yol).map_err(|_| "MIME ek erişimi reddedildi")?;
        let sonra = dosya.metadata().map_err(|_| "MIME ek bilgisi okunamadı")?;
        if !sonra.is_file() || sonra.file_type().is_symlink() {
            return Err("MIME ek açılırken normal dosya olmaktan çıktı".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if (once.dev(), once.ino()) != (sonra.dev(), sonra.ino()) {
                return Err("MIME ek açılırken dosya kimliği değişti".into());
            }
        }
        #[cfg(windows)]
        if once_kimlik != windows_kimligi(&dosya)? {
            return Err("MIME ek açılırken dosya kimliği değişti".into());
        }
        Ok(dosya)
    }
}

#[cfg(windows)]
fn windows_kimligi(dosya: &File) -> Result<(u32, u64), String> {
    use std::os::windows::io::AsRawHandle;
    #[repr(C)]
    #[derive(Default)]
    struct Bilgi {
        ozellik: u32,
        olusturma: [u32; 2],
        erisim: [u32; 2],
        yazma: [u32; 2],
        disk: u32,
        boyut_ust: u32,
        boyut_alt: u32,
        baglar: u32,
        kimlik_ust: u32,
        kimlik_alt: u32,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetFileInformationByHandle(h: *mut std::ffi::c_void, bilgi: *mut Bilgi) -> i32;
    }
    let mut bilgi = Bilgi::default();
    // SAFETY: canlı File handle'ı ve Win32 BY_HANDLE_FILE_INFORMATION yerleşimli
    // yazılabilir çıktı çağrı boyunca geçerlidir; pointer saklanmaz.
    if unsafe { GetFileInformationByHandle(dosya.as_raw_handle(), &mut bilgi) } == 0 {
        return Err("MIME ek kimliği okunamadı".into());
    }
    Ok((
        bilgi.disk,
        u64::from(bilgi.kimlik_ust) << 32 | u64::from(bilgi.kimlik_alt),
    ))
}
