//! K-360: grup işçisi ortak dinleyiciyi kapatamaz (yönetici, kardeş işçiler
//! ve systemd soketi aynı açık dosyayı tutar). Kabul ile SIGTERM denetimi
//! arasına düşen bağlantıyı düşürürse istek kaybolur; K-343 sözü sıfır
//! kayıptır.
//!
//! Harness yönetici rolünü kendisi üstlenir: ortak dinleyiciyi bağlar ve tek
//! birincil işçiyi gerçek grup ortamıyla (`ZEE_WEB_ISCI_*`) başlatır. Pencere
//! belirlenimli kurulur, uyku ile değil: ısınma isteğinden sonra işçinin
//! `dil` iş parçacığı dinleyici `poll`'unda iken ptrace ile yakalanır,
//! bağlantı gönderilir ve iş parçacığı başarılı `accept4` dönüşünde
//! (kabul sonrası denetimden önce) durdurulur. Sürece SIGTERM gönderilir;
//! sinyal durmuş iş parçacığına değil `join` bekleyen ana iş parçacığına
//! düşer. İşleyici kapanış bayrağını kurup `futex`'e dönünce iş parçacığı
//! bırakılır: denetim artık kapanışı görür. Eski sürüm kabul edilmiş akışı
//! düşürür (`kayıp`); düzeltilmiş işçi yanıtlar ve sonra kabulü durdurur.
//! Çıktı korunan ve kaybolan istek sayısıdır. Yalnız Linux (ptrace,
//! `/proc`); başka platformda sahte başarı yoktur.
#![allow(dead_code)]

use std::path::Path;

const TUR: usize = 3;

pub fn calistir(kaynak_yolu: &Path) -> (i64, Vec<String>) {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = kaynak_yolu;
        panic!("Linux ptrace accept window capability unavailable; no synthetic pass");
    }
    #[cfg(target_os = "linux")]
    {
        linux::calistir(kaynak_yolu)
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::TUR;
    use std::io::{Read, Write};
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::process::CommandExt;
    use std::path::Path;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    #[cfg(target_arch = "x86_64")]
    const ACCEPT4: u64 = 288;
    #[cfg(target_arch = "x86_64")]
    const FUTEX: &str = "202";
    #[cfg(target_arch = "x86_64")]
    const POLL: &[&str] = &["7", "271"];
    #[cfg(target_arch = "aarch64")]
    const ACCEPT4: u64 = 242;
    #[cfg(target_arch = "aarch64")]
    const FUTEX: &str = "98";
    #[cfg(target_arch = "aarch64")]
    const POLL: &[&str] = &["73"];

    const SURE: Duration = Duration::from_secs(30);

    struct Temizle(std::path::PathBuf, Option<std::process::Child>);
    impl Drop for Temizle {
        fn drop(&mut self) {
            if let Some(mut cocuk) = self.1.take() {
                let _ = cocuk.kill();
                let _ = cocuk.wait();
            }
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    pub fn calistir(kaynak_yolu: &Path) -> (i64, Vec<String>) {
        let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "zee-k360-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("saat")
                .as_nanos()
        ));
        std::fs::create_dir_all(&proje).expect("proje klasörü");
        let mut temizle = Temizle(proje.clone(), None);
        let ana = proje.join("ana.dil");
        std::fs::copy(kaynak_yolu, &ana).expect("kaynak kopyası");
        let dinleyici = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("ortak dinleyici");
        let kapi = dinleyici.local_addr().expect("dinleyici adresi").port();
        let istek = |kimlik: &str| -> std::net::TcpStream {
            let mut akis = std::net::TcpStream::connect(("127.0.0.1", kapi)).expect("bağlantı");
            akis.set_read_timeout(Some(SURE)).expect("okuma süresi");
            write!(
                akis,
                "GET / HTTP/1.1\r\nHost: panel.example\r\nForwarded: for={kimlik};proto=https;host=panel.example\r\nConnection: close\r\n\r\n"
            )
            .expect("istek yazımı");
            akis
        };
        let yanit_200 = |akis: &mut std::net::TcpStream| -> bool {
            let mut yanit = Vec::new();
            let _ = akis.read_to_end(&mut yanit);
            yanit.starts_with(b"HTTP/1.1 200 ")
        };

        let (mut korundu, mut kayip) = (0, 0);
        for tur in 0..TUR {
            let mut hazir = [0; 2];
            // SAFETY: two-element out array for pipe.
            assert_eq!(unsafe { libc::pipe(hazir.as_mut_ptr()) }, 0);
            // SAFETY: fresh pipe descriptors owned here exactly once.
            let mut hazir_okur = unsafe { std::fs::File::from_raw_fd(hazir[0]) };
            let hazir_yazar = unsafe { std::fs::File::from_raw_fd(hazir[1]) };
            let devir = [dinleyici.as_raw_fd(), hazir_yazar.as_raw_fd()];
            let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
            komut
                .args([
                    "çalıştır",
                    "--web-proxy",
                    super::super::native_web_kopma_fixture::PROXY,
                    "--web-worker-port",
                    &kapi.to_string(),
                    ana.to_str().expect("UTF-8 kaynak yolu"),
                ])
                .env("ZEE_WEB_ISCI_ROLU", "birincil")
                .env("ZEE_WEB_ISCI_DINLEYICI_FD", devir[0].to_string())
                .env("ZEE_WEB_ISCI_HAZIR_FD", devir[1].to_string())
                .env("ZEE_WEB_ISCI_EBEVEYN", std::process::id().to_string())
                .current_dir(&proje)
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            // SAFETY: only async-signal-safe fcntl between fork and exec, on
            // descriptors inherited by number, as the group manager does.
            unsafe {
                komut.pre_exec(move || {
                    for fd in devir {
                        let bayrak = libc::fcntl(fd, libc::F_GETFD);
                        if bayrak < 0
                            || libc::fcntl(fd, libc::F_SETFD, bayrak & !libc::FD_CLOEXEC) != 0
                        {
                            return Err(std::io::Error::last_os_error());
                        }
                    }
                    Ok(())
                });
            }
            temizle.1 = Some(komut.spawn().expect("işçi süreci"));
            drop(hazir_yazar);
            let mut bayt = [0u8; 1];
            hazir_okur
                .read_exact(&mut bayt)
                .expect("işçi hazır bildirmeden çıktı");
            let mut isinma = istek(&format!("203.0.113.{}", tur + 1));
            assert!(yanit_200(&mut isinma), "ısınma isteği yanıtlanmalı");
            let pid = temizle.1.as_ref().expect("işçi").id() as libc::pid_t;
            let tid = poll_eden_is_parcacigi(pid);
            yakala(tid);
            let mut akis = istek(&format!("198.51.100.{}", tur + 1));
            kabul_donusune_dek_ilerlet(tid);
            // SAFETY: SIGTERM to our own live, unreaped child.
            assert_eq!(unsafe { libc::kill(pid, libc::SIGTERM) }, 0);
            ana_is_parcacigi_isledi(pid);
            // SAFETY: detach our own ptrace-stopped tracee without a signal.
            let birak = unsafe { libc::ptrace(libc::PTRACE_DETACH as _, tid, 0, 0) };
            assert_eq!(birak, 0, "ptrace bırakma");

            let mut cocuk = temizle.1.take().expect("işçi");
            let son = Instant::now() + SURE;
            let durum = loop {
                if let Some(durum) = cocuk.try_wait().expect("işçi durumu") {
                    break durum;
                }
                if Instant::now() >= son {
                    let _ = cocuk.kill();
                    break cocuk.wait().expect("işçi sonucu");
                }
                std::thread::sleep(Duration::from_millis(10));
            };
            assert!(
                durum.success(),
                "işçi SIGTERM sonrası temiz çıkmalı: {durum}"
            );
            dinleyici.set_nonblocking(true).expect("kuyruk denetimi");
            let kuyrukta = match dinleyici.accept() {
                Ok(_) => true,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => false,
                Err(e) => panic!("kuyruk denetimi: {e}"),
            };
            dinleyici.set_nonblocking(false).expect("kuyruk denetimi");
            assert!(!kuyrukta, "bağlantı işçide kabul edilmiş olmalı");
            if yanit_200(&mut akis) {
                korundu += 1;
            } else {
                kayip += 1;
            }
        }
        (
            0,
            vec![format!("korundu {korundu}"), format!("kayıp {kayip}")],
        )
    }

    fn sistem_cagrisi(pid: libc::pid_t, tid: libc::pid_t) -> String {
        std::fs::read_to_string(format!("/proc/{pid}/task/{tid}/syscall"))
            .unwrap_or_default()
            .split(' ')
            .next()
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    /// Isınma yanıtından sonra dinleyici `poll`'unda uyuyan iş parçacığı.
    fn poll_eden_is_parcacigi(pid: libc::pid_t) -> libc::pid_t {
        let son = Instant::now() + SURE;
        while Instant::now() < son {
            let gorevler = std::fs::read_dir(format!("/proc/{pid}/task")).expect("işçi görevleri");
            for gorev in gorevler.flatten() {
                let Some(tid) = gorev.file_name().to_str().and_then(|t| t.parse().ok()) else {
                    continue;
                };
                if tid != pid && POLL.contains(&sistem_cagrisi(pid, tid).as_str()) {
                    return tid;
                }
            }
            std::thread::yield_now();
        }
        panic!("işçi 30 sn içinde dinleyici poll'una girmedi");
    }

    fn bekle(tid: libc::pid_t) -> libc::c_int {
        let mut durum = 0;
        // SAFETY: waitpid on our own tracee with a live out pointer.
        let sonuc = unsafe { libc::waitpid(tid, &mut durum, libc::__WALL) };
        assert_eq!(sonuc, tid, "ptrace beklemesi");
        assert!(libc::WIFSTOPPED(durum), "tracee durmadan çıktı: {durum:#x}");
        durum
    }

    fn yakala(tid: libc::pid_t) {
        // SAFETY: seize and interrupt a thread of our own child process.
        unsafe {
            assert_eq!(
                libc::ptrace(
                    libc::PTRACE_SEIZE as _,
                    tid,
                    0,
                    libc::PTRACE_O_TRACESYSGOOD as libc::c_long
                ),
                0,
                "ptrace yakalama: {}",
                std::io::Error::last_os_error()
            );
            assert_eq!(libc::ptrace(libc::PTRACE_INTERRUPT as _, tid, 0, 0), 0);
        }
        bekle(tid);
    }

    /// İş parçacığını sistem çağrısı durakları arasında ilerletir; başarılı
    /// `accept4` dönüşünde durmuş bırakır.
    fn kabul_donusune_dek_ilerlet(tid: libc::pid_t) {
        let mut son_giris = None;
        let mut sinyal = 0;
        loop {
            // SAFETY: resume our own stopped tracee until the next syscall stop.
            let devam = unsafe { libc::ptrace(libc::PTRACE_SYSCALL as _, tid, 0, sinyal) };
            assert_eq!(devam, 0, "ptrace ilerletme");
            sinyal = 0;
            let durum = bekle(tid);
            let durak = libc::WSTOPSIG(durum);
            if durak != (libc::SIGTRAP | 0x80) {
                // Olay/grup durağı sinyal taşımaz; sinyal durağı iletilir.
                if durum >> 16 == 0 && durak != libc::SIGTRAP {
                    sinyal = durak as libc::c_long;
                }
                continue;
            }
            // struct ptrace_syscall_info: op @0, entry.nr / exit.rval @24.
            let mut bilgi = [0u64; 11];
            // SAFETY: buffer is larger than the size passed; kernel writes at most that.
            let boy = unsafe {
                libc::ptrace(
                    libc::PTRACE_GET_SYSCALL_INFO as _,
                    tid,
                    std::mem::size_of_val(&bilgi),
                    bilgi.as_mut_ptr(),
                )
            };
            assert!(boy > 0, "ptrace sistem çağrısı bilgisi");
            match bilgi[0] as u8 {
                1 => son_giris = Some(bilgi[3]),
                2 => {
                    if son_giris == Some(ACCEPT4) && (bilgi[3] as i64) >= 0 {
                        return;
                    }
                    son_giris = None;
                }
                _ => {}
            }
        }
    }

    /// SIGTERM kuyruktan alınmış ve işleyici dönüp ana iş parçacığı yeniden
    /// `join` futex'inde uyuyor: kapanış bayrağı kurulmuştur.
    fn ana_is_parcacigi_isledi(pid: libc::pid_t) {
        let bekleyen = |yol: String, alan: &str| -> bool {
            std::fs::read_to_string(yol)
                .unwrap_or_default()
                .lines()
                .find_map(|satir| satir.strip_prefix(alan))
                .is_some_and(|maske| maske.trim().trim_start_matches('0').is_empty())
        };
        let son = Instant::now() + SURE;
        while Instant::now() < son {
            let alindi = bekleyen(format!("/proc/{pid}/status"), "ShdPnd:")
                && bekleyen(format!("/proc/{pid}/task/{pid}/status"), "SigPnd:");
            if alindi && sistem_cagrisi(pid, pid) == FUTEX {
                return;
            }
            std::thread::yield_now();
        }
        panic!("ana iş parçacığı SIGTERM'i 30 sn içinde işlemedi");
    }
}
