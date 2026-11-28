//! K-394 (RFC-0045, ADR-170): sembolik bağ izlemeyen sınırlı okumalar.
//!
//! Kalıcı dosya katmanının kilit altındaki iç okumaları (ekleme, eylem
//! yedeği, taşıma/silme izi, geri alma) yaprak bağını izlemez ve FIFO/
//! aygıtta bloklanmaz: aynı klasöre yazan başka süreç yaprağı yarışla bağa
//! çevirip kök dışı içeriği hedefe kopyalatamaz (spec/08 K-394).
//!
//! K-422 (ADR-170 eki): veri kökü yazımları ayrıca **tek-bağ kipinde**
//! koşar. Kipte bu iç okumalar ve metadata kaynağı açılan tanıtıcıda
//! `st_nlink == 1` ister: eş hizmetin paylaşılan klasöre bağladığı sert bağ
//! (yazarın okuyabildiği dış dosya) ekleme, eylem yedeği, taşıma/silme izi
//! ya da EXDEV kopyasıyla tek bağlı yeni dosyaya aklanamaz. Kip yalnız
//! `veri:` yollarında açılır; proje dosyalarının davranışı değişmez.

use std::cell::Cell;

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

thread_local! {
    static TEK_BAG_DERINLIGI: Cell<u32> = const { Cell::new(0) };
}

/// K-422: tek-bağ kipini bu iş parçacığında, değer düşene dek açık tutar
/// (iç içe ve panik güvenli).
pub(crate) struct TekBagKipi(());

impl TekBagKipi {
    pub(crate) fn ac() -> Self {
        TEK_BAG_DERINLIGI.with(|d| d.set(d.get().saturating_add(1)));
        Self(())
    }
}

impl Drop for TekBagKipi {
    fn drop(&mut self) {
        TEK_BAG_DERINLIGI.with(|d| d.set(d.get().saturating_sub(1)));
    }
}

/// Tek-bağ kipi bu iş parçacığında açık mı.
pub(crate) fn tek_bag_kipi_acik() -> bool {
    TEK_BAG_DERINLIGI.with(|d| d.get() > 0)
}

/// K-422: kip açıksa açılmış tanıtıcı tek sert bağ taşımalıdır. Kapalıyken
/// ve unix dışında (veri kökü yok) koşulsuz kabul.
pub(crate) fn tek_bag_denetle(dosya: &std::fs::File) -> io::Result<()> {
    if !tek_bag_kipi_acik() {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if dosya.metadata()?.nlink() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "veri kökünde birden çok sert bağ (hard-link) taşıyan dosya okunmaz ve üzerine yazılmaz",
            ));
        }
    }
    #[cfg(not(unix))]
    let _ = dosya;
    Ok(())
}

/// K-394/K-422: yazma kilidi açılış bayrakları; tek-bağ kipinde açılış
/// bloklamaz ve denetim terminali almaz.
#[cfg(unix)]
pub(crate) fn kilit_bayraklari() -> libc::c_int {
    let temel = libc::O_NOFOLLOW | libc::O_CLOEXEC;
    if tek_bag_kipi_acik() {
        temel | libc::O_NONBLOCK | libc::O_NOCTTY
    } else {
        temel
    }
}

/// K-422: tek-bağ kipinde yazma kilidi normal dosya ve tek sert bağ olmalı.
pub(crate) fn kilit_dosyasini_denetle(dosya: &std::fs::File) -> io::Result<()> {
    if tek_bag_kipi_acik() && !dosya.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "veri kökünde yazma kilidi normal dosya olmalı",
        ));
    }
    tek_bag_denetle(dosya)
}

/// Kalıcı katmanın içerik kopyalayan okuyucusu (EXDEV taşıma kopyası).
/// Tek-bağ kipinde unix'te bağ izlenmez, bloklanmaz, normal dosya ve tek
/// sert bağ istenir; kip kapalıyken önceki `File::open` davranışıdır.
pub(crate) fn kalici_okuyucu_ac(yol: &Path) -> io::Result<std::fs::File> {
    #[cfg(unix)]
    if tek_bag_kipi_acik() {
        return bagsiz_ac(yol);
    }
    let dosya = std::fs::File::open(yol)?;
    tek_bag_denetle(&dosya)?;
    Ok(dosya)
}

#[cfg(unix)]
fn bagsiz_ac(yol: &Path) -> io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::io::AsRawFd;
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
    tek_bag_denetle(&dosya)?;
    // SAFETY: fd açık; F_GETFL/F_SETFL yalnız durum bayraklarını değiştirir.
    unsafe {
        let bayrak = libc::fcntl(dosya.as_raw_fd(), libc::F_GETFL);
        if bayrak >= 0 {
            libc::fcntl(dosya.as_raw_fd(), libc::F_SETFL, bayrak & !libc::O_NONBLOCK);
        }
    }
    Ok(dosya)
}

/// Kalıcı dosya katmanının bağ izlemeyen iç okuması (genel okuma sınırı).
pub(crate) fn kalici_ic_baytlari_oku(yol: &Path) -> io::Result<Vec<u8>> {
    kalici_ic_baytlari_oku_azami(yol, VARSAYILAN_KAYNAK_SINIRLARI.dosya_okuma_bayti())
}

/// Bağ izlemeyen iç okuma, çağıranın kilit altında denetlediği tavanla
/// (K-395 çözerek ekleme 32 MiB).
pub(crate) fn kalici_ic_baytlari_oku_azami(yol: &Path, azami: usize) -> io::Result<Vec<u8>> {
    #[cfg(unix)]
    {
        let dosya = bagsiz_ac(yol)?;
        sinirli_oku(dosya, azami, || {
            sinir_hatasi(&yol.display().to_string(), azami)
        })
    }
    #[cfg(not(unix))]
    super::sinirli_bayt_oku(yol, azami)
}
