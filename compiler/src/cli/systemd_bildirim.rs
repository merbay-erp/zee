//! K-376: systemd hizmet bildirimi (sd_notify `READY=1` / `STOPPING=1`).
//!
//! Private CLI adapter. `NOTIFY_SOCKET` süreç başında bir kez okunur ve
//! ortamdan silinir: işçi süreçlerine ve Zee alt süreçlerine sızmaz, böylece
//! `NotifyAccess=main` altında yalnız ana süreç bildirir. Değişken yoksa
//! hiçbir şey gönderilmez (bugünkü `Type=simple`/`Type=exec` birimleri
//! değişmez). Her ileti süreç ömründe en çok bir kez gönderilir.

use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

static HEDEF: OnceLock<Option<OsString>> = OnceLock::new();
static HAZIR: AtomicBool = AtomicBool::new(false);
static DURUYOR: AtomicBool = AtomicBool::new(false);

/// `govde` başında çağrılır; sonraki çağrılar etkisizdir.
pub(crate) fn al() {
    let deger = std::env::var_os("NOTIFY_SOCKET");
    std::env::remove_var("NOTIFY_SOCKET");
    let _ = HEDEF.set(deger.filter(|d| !d.is_empty()));
}

/// Bütün dinleyiciler doğrulanıp açılış işi bittiğinde bir kez.
pub(crate) fn hazir() {
    if !HAZIR.swap(true, Ordering::SeqCst) {
        gonder("READY=1");
    }
}

/// Kapanış başladığında (SIGTERM) bir kez; yeni kabul durur, boşaltma sürer.
pub(crate) fn durduruluyor() {
    if !DURUYOR.swap(true, Ordering::SeqCst) {
        gonder("STOPPING=1");
    }
}

fn gonder(ileti: &str) {
    let Some(Some(hedef)) = HEDEF.get() else {
        return;
    };
    if let Err(hata) = yolla(hedef, ileti) {
        eprintln!("systemd bildirimi ({ileti}) gönderilemedi: {hata}");
    }
}

#[cfg(unix)]
fn yolla(hedef: &std::ffi::OsStr, ileti: &str) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::net::UnixDatagram;
    let baytlar = hedef.as_bytes();
    // std UnixDatagram SOCK_CLOEXEC ile açar; soket yalnız bu gönderim için yaşar.
    let soket = UnixDatagram::unbound()?;
    if let Some(soyut) = baytlar.strip_prefix(b"@") {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            use std::os::linux::net::SocketAddrExt;
            let adres = std::os::unix::net::SocketAddr::from_abstract_name(soyut)?;
            soket.send_to_addr(ileti.as_bytes(), &adres)?;
            return Ok(());
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            let _ = soyut;
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "soyut NOTIFY_SOCKET yalnız Linux'ta",
            ));
        }
    }
    if baytlar.first() != Some(&b'/') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "NOTIFY_SOCKET mutlak yol ya da @soyut ad olmalı",
        ));
    }
    soket.send_to(ileti.as_bytes(), std::path::Path::new(hedef))?;
    Ok(())
}

#[cfg(not(unix))]
fn yolla(_: &std::ffi::OsStr, _: &str) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "sd_notify yalnız Unix'te",
    ))
}
