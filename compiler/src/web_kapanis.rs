//! Private CLI SIGTERM ownership: asynchronous handler performs only an atomic store.
use std::sync::atomic::{AtomicBool, Ordering};
static SAHIPLI: AtomicBool = AtomicBool::new(false);
static ISTENDI: AtomicBool = AtomicBool::new(false);
extern "C" fn sonlandir(_signal: libc::c_int) {
    ISTENDI.store(true, Ordering::Relaxed);
}
pub(super) struct WebKapanis {
    onceki: libc::sigaction,
}
impl WebKapanis {
    pub(super) fn kur() -> Result<Self, String> {
        SAHIPLI
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .map_err(|_| "web SIGTERM disposition already owned".to_string())?;
        ISTENDI.store(false, Ordering::Relaxed);
        // SAFETY: libc sigaction's C integer/pointer/sigset representation admits
        // zero initialization; sigemptyset initializes its mask before registration.
        let mut yeni: libc::sigaction = unsafe { std::mem::zeroed() };
        let mut onceki: libc::sigaction = unsafe { std::mem::zeroed() };
        yeni.sa_sigaction = sonlandir as *const () as libc::sighandler_t;
        yeni.sa_flags = libc::SA_RESTART;
        // SAFETY: live aligned sigaction pointers, valid SIGTERM and C ABI handler.
        // The handler has no allocation, IO, lock, interpreter state or panic path.
        let sonuc = unsafe {
            libc::sigemptyset(&mut yeni.sa_mask);
            libc::sigaction(libc::SIGTERM, &yeni, &mut onceki)
        };
        if sonuc != 0 {
            SAHIPLI.store(false, Ordering::Release);
            return Err("web SIGTERM disposition could not be installed".into());
        }
        Ok(Self { onceki })
    }
    pub(super) fn istendi(&self) -> bool {
        ISTENDI.load(Ordering::Relaxed)
    }
}
impl Drop for WebKapanis {
    fn drop(&mut self) {
        // SAFETY: restore the exact live disposition returned on installation.
        let sonuc = unsafe { libc::sigaction(libc::SIGTERM, &self.onceki, std::ptr::null_mut()) };
        if sonuc == 0 {
            SAHIPLI.store(false, Ordering::Release);
        } else {
            // Keep ownership reserved if OS restoration fails; a subsequent web
            // guard cannot mistake our remaining handler for its predecessor.
            eprintln!("web SIGTERM disposition could not be restored");
        }
    }
}
/// Wait for listener readability, not an unconditional request latency sleep.
/// A SIGTERM arriving during poll interrupts it; a signal racing before poll
/// is observed after at most the 100 ms idle timeout.
pub(super) fn bekle(dinleyici: &std::net::TcpListener) -> std::io::Result<()> {
    use std::os::fd::AsRawFd;
    bekle_fd(dinleyici.as_raw_fd())
}

fn bekle_fd(fd: std::os::fd::RawFd) -> std::io::Result<()> {
    if fd < 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid listener descriptor",
        ));
    }
    let mut olay = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: poll receives one live aligned pollfd for the entire synchronous
    // call. The owned listener is not closed/reused while this borrow is live.
    let sonuc = unsafe { libc::poll(&mut olay, 1, 100) };
    if sonuc < 0 {
        let hata = std::io::Error::last_os_error();
        return if hata.kind() == std::io::ErrorKind::Interrupted {
            Ok(())
        } else {
            Err(hata)
        };
    }
    if olay.revents & (libc::POLLNVAL | libc::POLLERR | libc::POLLHUP) != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "listener poll failed",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn poll_gecersiz_descriptor_ve_negatif_fd_reddedilir() {
        assert_eq!(
            bekle_fd(-1).unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
        assert_eq!(
            bekle_fd(i32::MAX).unwrap_err().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn poll_hazir_dinleyici_ve_idle_timeout_siniri() {
        let dinleyici = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let baslangic = std::time::Instant::now();
        bekle(&dinleyici).unwrap();
        assert!(baslangic.elapsed() < std::time::Duration::from_secs(1));
        let _istemci = std::net::TcpStream::connect(dinleyici.local_addr().unwrap()).unwrap();
        bekle(&dinleyici).unwrap();
        dinleyici.set_nonblocking(true).unwrap();
        assert!(dinleyici.accept().is_ok());
    }

    #[test]
    fn disposition_sahipligi_ve_onceki_handler_tam_geri_yuklenir() {
        // SAFETY: null new action queries the disposition into live output memory.
        let mut baslangic: libc::sigaction = unsafe { std::mem::zeroed() };
        assert_eq!(
            unsafe { libc::sigaction(libc::SIGTERM, std::ptr::null(), &mut baslangic) },
            0
        );
        let nobetci = WebKapanis::kur().unwrap();
        assert!(!nobetci.istendi());
        assert!(WebKapanis::kur().is_err());
        sonlandir(libc::SIGTERM);
        assert!(nobetci.istendi());
        drop(nobetci);
        let mut sonraki: libc::sigaction = unsafe { std::mem::zeroed() };
        assert_eq!(
            unsafe { libc::sigaction(libc::SIGTERM, std::ptr::null(), &mut sonraki) },
            0
        );
        assert_eq!(sonraki.sa_sigaction, baslangic.sa_sigaction);
        assert_eq!(
            sonraki.sa_flags & libc::SA_RESTART,
            baslangic.sa_flags & libc::SA_RESTART
        );
        for signal in 1..=128 {
            assert_eq!(
                unsafe { libc::sigismember(&sonraki.sa_mask, signal) },
                unsafe { libc::sigismember(&baslangic.sa_mask, signal) }
            );
        }
        let yeni = WebKapanis::kur().unwrap();
        assert!(!yeni.istendi());
    }
}
