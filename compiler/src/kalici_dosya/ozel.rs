//! K-184: IO izleri için metadata miras almayan özel atomik yazım.
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// İz dosyasını yalnız sahibi erişebilecek biçimde atomik yayımlar.
/// Genel atomik yazımdan farklı olarak eski hedefin izin/ACL'sini taşımaz.
pub fn atomik_ozel_yaz(hedef: &Path, icerik: &[u8]) -> io::Result<()> {
    let _kilit = super::DosyaKilidi::al(hedef)?;
    let _metadata = super::metadata::MetadataKaynagi::yakala(hedef)?;
    let (mut dosya, mut gecici) = super::gecici_dosyayi_acarak(hedef, ozel_ac)?;
    // İzinler dosya ilk açılırken uygulanır; geniş izinli bir pencere yoktur.
    dosya.write_all(icerik)?;
    dosya.sync_all()?;
    drop(dosya);
    let yol = gecici
        .yol
        .as_deref()
        .ok_or_else(|| io::Error::other("geçici iz yolu yok"))?;
    ozel_degistir(yol, hedef)?;
    gecici.yol = None;
    super::klasoru_eszamanla(super::ebeveyn(hedef))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn ozel_ac(yol: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(yol)
}

#[cfg(target_os = "macos")]
fn ozel_ac(yol: &Path) -> io::Result<File> {
    use std::ffi::{c_void, CString};
    use std::os::fd::FromRawFd;
    use std::os::unix::ffi::OsStrExt;
    extern "C" {
        fn acl_init(count: i32) -> *mut c_void;
        fn acl_free(acl: *mut c_void) -> i32;
        fn acl_get_flagset_np(acl: *mut c_void, flags: *mut *mut c_void) -> i32;
        fn acl_add_flag_np(flags: *mut c_void, flag: u32) -> i32;
        fn filesec_init() -> *mut c_void;
        fn filesec_free(sec: *mut c_void);
        fn filesec_set_property(sec: *mut c_void, property: i32, value: *const c_void) -> i32;
        fn openx_np(path: *const i8, flags: i32, sec: *mut c_void) -> i32;
    }
    let yol = CString::new(yol.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "dosya yolu NUL içeremez"))?;
    // SAFETY: ACL/filesec sistemden alınır, tüm göstericiler çağrı boyunca
    // yaşamda tutulur ve hata dahil her yolda tam bir kez serbest bırakılır.
    // openx_np, mode ve mirassız boş ACL'yi ilk açılışta birlikte uygular.
    unsafe {
        let acl = acl_init(0);
        if acl.is_null() {
            return Err(io::Error::last_os_error());
        }
        let sec = filesec_init();
        if sec.is_null() {
            let hata = io::Error::last_os_error();
            acl_free(acl);
            return Err(hata);
        }
        let sonuc = (|| {
            let mut flags = std::ptr::null_mut();
            let mode: libc::mode_t = 0o600;
            if acl_get_flagset_np(acl, &mut flags) != 0
                || acl_add_flag_np(flags, 1 << 17) != 0
                || filesec_set_property(sec, 4, (&mode as *const libc::mode_t).cast()) != 0
                || filesec_set_property(sec, 5, (&acl as *const *mut c_void).cast()) != 0
            {
                return Err(io::Error::last_os_error());
            }
            let fd = openx_np(
                yol.as_ptr(),
                libc::O_CREAT | libc::O_EXCL | libc::O_WRONLY | libc::O_CLOEXEC,
                sec,
            );
            if fd < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(File::from_raw_fd(fd))
            }
        })();
        filesec_free(sec);
        acl_free(acl);
        sonuc
    }
}
#[cfg(not(windows))]
fn ozel_degistir(eski: &Path, yeni: &Path) -> io::Result<()> {
    std::fs::rename(eski, yeni)
}
#[cfg(not(any(unix, windows)))]
fn ozel_ac(_yol: &Path) -> io::Result<File> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "özel iz dosyası bu platformda desteklenmiyor",
    ))
}
#[cfg(windows)]
use windows::{ozel_ac, ozel_degistir};
#[cfg(windows)]
mod windows {
    use super::*;
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::FromRawHandle;
    type Handle = *mut c_void;
    #[repr(C)]
    struct SecurityAttributes {
        uzunluk: u32,
        descriptor: Handle,
        miras: i32,
    }
    #[link(name = "advapi32")]
    extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            s: *const u16,
            revision: u32,
            out: *mut Handle,
            size: *mut u32,
        ) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateFileW(
            name: *const u16,
            access: u32,
            share: u32,
            security: *const SecurityAttributes,
            creation: u32,
            flags: u32,
            template: Handle,
        ) -> Handle;
        fn LocalFree(memory: Handle) -> Handle;
        fn MoveFileExW(old: *const u16, new: *const u16, flags: u32) -> i32;
    }
    fn genis(yol: &Path) -> io::Result<Vec<u16>> {
        let mut s: Vec<u16> = yol.as_os_str().encode_wide().collect();
        if s.contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "dosya yolu NUL içeremez",
            ));
        }
        s.push(0);
        Ok(s)
    }
    pub(super) fn ozel_ac(yol: &Path) -> io::Result<File> {
        let yol = genis(yol)?;
        // Protected DACL: yalnız nesne sahibi; üst dizin ACE'leri miras alınmaz.
        let sddl: Vec<u16> = "D:P(A;;FA;;;OW)\0".encode_utf16().collect();
        let mut descriptor = std::ptr::null_mut();
        // SAFETY: NUL sonlu SDDL ve yazılabilir descriptor işaretçisi geçerli.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                std::ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let attributes = SecurityAttributes {
            uzunluk: std::mem::size_of::<SecurityAttributes>() as u32,
            descriptor,
            miras: 0,
        };
        // SAFETY: Yol/descriptor yaşamda; CREATE_NEW mevcut yolu takip etmez.
        let handle = unsafe {
            CreateFileW(
                yol.as_ptr(),
                0x40000000,
                0,
                &attributes,
                1,
                0x80,
                std::ptr::null_mut(),
            )
        };
        let hata = io::Error::last_os_error();
        // SAFETY: LocalAlloc descriptor'ı tam bir kez bırakılır.
        unsafe {
            LocalFree(descriptor);
        }
        if handle == -1isize as Handle {
            return Err(hata);
        }
        // SAFETY: Geçerli ve sahipliği bize ait yeni handle.
        Ok(unsafe { File::from_raw_handle(handle) })
    }
    pub(super) fn ozel_degistir(eski: &Path, yeni: &Path) -> io::Result<()> {
        let eski = genis(eski)?;
        let yeni = genis(yeni)?;
        // ReplaceFileW eski DACL'yi taşır; aynı dizindeki MoveFileExW yeni
        // özel dosyanın DACL'sini korur (REPLACE_EXISTING|WRITE_THROUGH).
        // SAFETY: İki NUL sonlu yol çağrı boyunca geçerlidir.
        if unsafe { MoveFileExW(eski.as_ptr(), yeni.as_ptr(), 0x1 | 0x8) } != 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}
