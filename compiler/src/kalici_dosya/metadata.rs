//! Atomik replace öncesi hedef inode metadata'sını yeni dosyaya taşır.

use std::fs::File;
use std::io;
use std::path::Path;

pub(super) struct MetadataKaynagi {
    #[cfg(unix)]
    dosya: Option<File>,
}

impl MetadataKaynagi {
    pub(super) fn yakala(hedef: &Path) -> io::Result<Self> {
        let bilgi = match std::fs::symlink_metadata(hedef) {
            Ok(bilgi) => Some(bilgi),
            Err(hata) if hata.kind() == io::ErrorKind::NotFound => None,
            Err(hata) => return Err(hata),
        };
        if let Some(bilgi) = &bilgi {
            if bilgi.file_type().is_symlink() || !bilgi.is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "atomik replace metadata sözleşmesi yalnız normal dosya hedefini destekler",
                ));
            }
        }
        #[cfg(unix)]
        let dosya = bilgi
            .map(|_| {
                use std::os::unix::fs::OpenOptionsExt;

                std::fs::OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW)
                    .open(hedef)
            })
            .transpose()
            .map_err(|hata| {
                io::Error::new(
                    hata.kind(),
                    format!("eski dosya metadata için açılamadı: {hata}"),
                )
            })?;
        #[cfg(unix)]
        if let Some(dosya) = &dosya {
            if !dosya.metadata()?.is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "metadata kaynağı açılırken normal dosya olmaktan çıktı",
                ));
            }
        }
        Ok(Self {
            #[cfg(unix)]
            dosya,
        })
    }

    pub(super) fn geciciye_uygula(&self, gecici: &File) -> io::Result<()> {
        #[cfg(unix)]
        if let Some(eski) = &self.dosya {
            unix_tabanini_koru(eski, gecici)?;
            platform_metadata_koru(eski, gecici)?;
        }
        #[cfg(not(unix))]
        let _ = gecici;
        Ok(())
    }
}

#[cfg(unix)]
fn unix_tabanini_koru(eski: &File, gecici: &File) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::MetadataExt;

    let bilgi = eski.metadata()?;
    // SAFETY: Her iki değer de yaşamda olan File nesnelerinden gelir.
    if unsafe { libc::fchown(gecici.as_raw_fd(), bilgi.uid(), bilgi.gid()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    gecici.set_permissions(bilgi.permissions())
}

#[cfg(target_os = "macos")]
fn platform_metadata_koru(eski: &File, gecici: &File) -> io::Result<()> {
    use std::os::fd::AsRawFd;

    let bayraklar = libc::COPYFILE_ACL | libc::COPYFILE_XATTR;
    // SAFETY: File descriptor'ları çağrı boyunca geçerli; null state ile
    // yalnız ACL ve xattr kopyalanır, içerik/timestamp taşınmaz.
    if unsafe {
        libc::fcopyfile(
            eski.as_raw_fd(),
            gecici.as_raw_fd(),
            std::ptr::null_mut(),
            bayraklar,
        )
    } == 0
    {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(target_os = "linux")]
fn platform_metadata_koru(eski: &File, gecici: &File) -> io::Result<()> {
    linux_xattr::kopyala(eski, gecici)
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "linux"))))]
fn platform_metadata_koru(_eski: &File, _gecici: &File) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "bu Unix platformunda ACL/xattr korumalı atomik replace henüz desteklenmiyor",
    ))
}

#[cfg(target_os = "linux")]
mod linux_xattr {
    use std::ffi::CString;
    use std::fs::File;
    use std::io;
    use std::os::fd::AsRawFd;

    const AZAMI_AD_LISTESI: usize = 64 * 1024;
    const AZAMI_DEGER: usize = 64 * 1024;
    const AZAMI_TOPLAM: usize = 1024 * 1024;

    pub(super) fn kopyala(eski: &File, yeni: &File) -> io::Result<()> {
        let adlar = adlari_oku(eski)?;
        let mut toplam = 0usize;
        for ad in adlar {
            let deger = degeri_oku(eski, &ad)?;
            toplam = toplam.checked_add(deger.len()).ok_or_else(sinir_hatasi)?;
            if toplam > AZAMI_TOPLAM {
                return Err(sinir_hatasi());
            }
            // SAFETY: CString NUL sonlu; değer tamponu çağrı boyunca geçerli.
            if unsafe {
                libc::fsetxattr(
                    yeni.as_raw_fd(),
                    ad.as_ptr(),
                    deger.as_ptr().cast(),
                    deger.len(),
                    0,
                )
            } != 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    fn adlari_oku(dosya: &File) -> io::Result<Vec<CString>> {
        // SAFETY: Sıfır boyutlu ilk çağrı yalnız gereken uzunluğu döndürür.
        let boyut = unsafe { libc::flistxattr(dosya.as_raw_fd(), std::ptr::null_mut(), 0) };
        if boyut < 0 {
            let hata = io::Error::last_os_error();
            if hata.raw_os_error() == Some(libc::ENOTSUP) {
                return Ok(Vec::new());
            }
            return Err(hata);
        }
        let boyut = usize::try_from(boyut).map_err(|_| sinir_hatasi())?;
        if boyut > AZAMI_AD_LISTESI {
            return Err(sinir_hatasi());
        }
        let mut tampon = vec![0u8; boyut];
        if boyut == 0 {
            return Ok(Vec::new());
        }
        // SAFETY: Tampon bildirilen boyutta ve yazılabilir.
        let okunan = unsafe {
            libc::flistxattr(dosya.as_raw_fd(), tampon.as_mut_ptr().cast(), tampon.len())
        };
        if okunan < 0 {
            return Err(io::Error::last_os_error());
        }
        tampon.truncate(usize::try_from(okunan).map_err(|_| sinir_hatasi())?);
        if tampon.last() != Some(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "xattr ad listesi NUL ile sonlanmıyor",
            ));
        }
        tampon[..tampon.len() - 1]
            .split(|bayt| *bayt == 0)
            .map(|ad| {
                CString::new(ad).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "xattr adı iç NUL taşıyor")
                })
            })
            .collect()
    }

    fn degeri_oku(dosya: &File, ad: &CString) -> io::Result<Vec<u8>> {
        // SAFETY: CString NUL sonlu; sıfır boyutlu çağrı uzunluğu sorgular.
        let boyut =
            unsafe { libc::fgetxattr(dosya.as_raw_fd(), ad.as_ptr(), std::ptr::null_mut(), 0) };
        if boyut < 0 {
            return Err(io::Error::last_os_error());
        }
        let boyut = usize::try_from(boyut).map_err(|_| sinir_hatasi())?;
        if boyut > AZAMI_DEGER {
            return Err(sinir_hatasi());
        }
        let mut deger = vec![0u8; boyut];
        // SAFETY: Değer tamponu bildirilen boyutta ve yazılabilir.
        let okunan = unsafe {
            libc::fgetxattr(
                dosya.as_raw_fd(),
                ad.as_ptr(),
                deger.as_mut_ptr().cast(),
                deger.len(),
            )
        };
        if okunan < 0 {
            return Err(io::Error::last_os_error());
        }
        deger.truncate(usize::try_from(okunan).map_err(|_| sinir_hatasi())?);
        Ok(deger)
    }

    fn sinir_hatasi() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "dosya xattr metadata'sı 64 KiB ad/değer veya 1 MiB toplam sınırını aşıyor",
        )
    }
}
