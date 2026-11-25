//! K-394 (RFC-0045, ADR-170): sembolik bağ izlemeyen sınırlı okumalar.
//!
//! Kalıcı dosya katmanının kilit altındaki iç okumaları (ekleme, eylem
//! yedeği, taşıma/silme izi, geri alma) yaprak bağını izlemez ve FIFO/
//! aygıtta bloklanmaz: aynı klasöre yazan başka süreç yaprağı yarışla bağa
//! çevirip kök dışı içeriği hedefe kopyalatamaz (spec/08 K-394).

#[cfg(unix)]
use crate::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI;
use std::io;
#[cfg(unix)]
use std::io::Read;
use std::path::Path;

#[cfg(unix)]
fn sinirli_oku(
    dosya: std::fs::File,
    azami: usize,
    asim: impl Fn() -> io::Error,
) -> io::Result<Vec<u8>> {
    if dosya.metadata()?.len() > azami as u64 {
        return Err(asim());
    }
    let mut baytlar = Vec::new();
    dosya
        .take(azami.saturating_add(1) as u64)
        .read_to_end(&mut baytlar)?;
    if baytlar.len() > azami {
        return Err(asim());
    }
    Ok(baytlar)
}

#[cfg(unix)]
fn sinir_hatasi(ad: &str, azami: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("`{ad}` {} MiB okuma sınırını aşıyor", azami / 1024 / 1024),
    )
}

/// Kalıcı dosya katmanının bağ izlemeyen iç okuması.
pub(crate) fn kalici_ic_baytlari_oku(yol: &Path) -> io::Result<Vec<u8>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        use std::os::unix::io::AsRawFd;
        let azami = VARSAYILAN_KAYNAK_SINIRLARI.dosya_okuma_bayti();
        let dosya = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(yol)?;
        if !dosya.metadata()?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "hedef normal dosya değil",
            ));
        }
        // SAFETY: fd açık; F_GETFL/F_SETFL yalnız durum bayraklarını değiştirir.
        unsafe {
            let bayrak = libc::fcntl(dosya.as_raw_fd(), libc::F_GETFL);
            if bayrak >= 0 {
                libc::fcntl(dosya.as_raw_fd(), libc::F_SETFL, bayrak & !libc::O_NONBLOCK);
            }
        }
        sinirli_oku(dosya, azami, || {
            sinir_hatasi(&yol.display().to_string(), azami)
        })
    }
    #[cfg(not(unix))]
    crate::kaynak_sinirlari::veri_dosyasi_baytlarini_oku(yol)
}
