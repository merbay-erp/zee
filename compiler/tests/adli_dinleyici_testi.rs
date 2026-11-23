//! K-376 (RFC-0039 Aşama A1): adlı çoklu dinleyici devri ve sd_notify.
//!
//! systemd soket etkinleştirmesi gerçek süreçle taklit edilir: soketleri test
//! tutar (systemd gibi), `sh -c 'LISTEN_PID=$$ … exec dil …'` ile fd 3'ten
//! ardışık devreder; `$$` exec sonrası `dil`in kendi PID'idir. Adlar
//! bildirim dışı sırayla verilir (systemd 257'de gözlenen davranış).
#![cfg(unix)]

use std::io::Read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::net::UnixDatagram;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static SIRA: AtomicUsize = AtomicUsize::new(0);

struct Proje(PathBuf);

impl Proje {
    fn yeni(dinleyiciler: &[String]) -> Self {
        // macOS AF_UNIX yol sınırı (104) için kısa kök.
        let kok = PathBuf::from("/tmp").join(format!(
            "zk376-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&kok).expect("proje kökü");
        let liste = if dinleyiciler.is_empty() {
            "dinleyiciler boş liste olsun\n".to_string()
        } else {
            format!(
                "dinleyiciler {} listesi olsun\n",
                dinleyiciler
                    .iter()
                    .map(|d| format!("\"{d}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        std::fs::write(
            kok.join("proje.dil"),
            format!(
                "proje \"adli-devir\" olsun\nsürüm \"0.1.0\" olsun\nmorfoloji \"zee-tr-1\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ-sunucusu\" listesi olsun\nağ_hedefleri boş liste olsun\n{liste}"
            ),
        )
        .expect("proje.dil");
        std::fs::write(
            kok.join("ana.dil"),
            "8080 kapısında sunucu başlat\n\nGET \"/\" adresine istek geldiğinde\n    \"tamam\" yanıtını gönder\n",
        )
        .expect("ana.dil");
        Self(kok)
    }
}

impl Drop for Proje {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Belirli aileye bağlı dinleyen soket; IPv6'da V6ONLY açık/kapalı seçilir.
fn soket(adres: SocketAddr, v6only: bool) -> OwnedFd {
    let aile = if adres.is_ipv4() {
        libc::AF_INET
    } else {
        libc::AF_INET6
    };
    // SAFETY: fresh socket owned below; option/bind/listen use correctly sized
    // initialized native address records.
    unsafe {
        let fd = libc::socket(aile, libc::SOCK_STREAM, 0);
        assert!(fd >= 0, "socket");
        let sahip = OwnedFd::from_raw_fd(fd);
        let bir = 1i32;
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_REUSEADDR,
            (&bir as *const i32).cast(),
            4,
        );
        let sonuc = match adres {
            SocketAddr::V4(a) => {
                let mut s: libc::sockaddr_in = std::mem::zeroed();
                #[cfg(target_vendor = "apple")]
                {
                    s.sin_len = std::mem::size_of::<libc::sockaddr_in>() as u8;
                }
                s.sin_family = libc::AF_INET as _;
                s.sin_port = a.port().to_be();
                s.sin_addr.s_addr = u32::from_ne_bytes(a.ip().octets());
                libc::bind(
                    fd,
                    (&s as *const libc::sockaddr_in).cast(),
                    std::mem::size_of_val(&s) as _,
                )
            }
            SocketAddr::V6(a) => {
                let deger = i32::from(v6only);
                libc::setsockopt(
                    fd,
                    libc::IPPROTO_IPV6,
                    libc::IPV6_V6ONLY,
                    (&deger as *const i32).cast(),
                    4,
                );
                let mut s: libc::sockaddr_in6 = std::mem::zeroed();
                #[cfg(target_vendor = "apple")]
                {
                    s.sin6_len = std::mem::size_of::<libc::sockaddr_in6>() as u8;
                }
                s.sin6_family = libc::AF_INET6 as _;
                s.sin6_port = a.port().to_be();
                s.sin6_addr.s6_addr = a.ip().octets();
                libc::bind(
                    fd,
                    (&s as *const libc::sockaddr_in6).cast(),
                    std::mem::size_of_val(&s) as _,
                )
            }
        };
        assert_eq!(
            sonuc,
            0,
            "bind {adres}: {}",
            std::io::Error::last_os_error()
        );
        assert_eq!(libc::listen(fd, 16), 0, "listen");
        sahip
    }
}

/// Çekirdeğin seçtiği kapıyla bağlı soket ve gerçek adresi.
fn bagli(ip: &str, v6only: bool) -> (OwnedFd, SocketAddr) {
    let fd = soket(format!("{ip}:0").parse().unwrap(), v6only);
    // SAFETY: borrow the owned listener only to read its local address.
    let gecici = unsafe { TcpListener::from_raw_fd(fd.as_raw_fd()) };
    let adres = gecici.local_addr().expect("adres");
    std::mem::forget(gecici);
    (fd, adres)
}

fn bos_kapi() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// `dil`i systemd gibi başlatır: `fdler` sırayla 3, 4, … olur.
fn baslat(proje: &Proje, fdler: &[i32], ortam: &str, kapi: u16) -> Child {
    use std::os::unix::process::CommandExt;
    let mut komut = Command::new("/bin/sh");
    komut
        .arg("-c")
        .arg(format!("{ortam} exec \"$0\" \"$@\""))
        .arg(env!("CARGO_BIN_EXE_dil"))
        .args([
            "çalıştır",
            "--web-proxy",
            "https://panel.example",
            "--web-worker-port",
            &kapi.to_string(),
        ])
        .arg(&proje.0)
        .current_dir(&proje.0)
        .env_remove("NOTIFY_SOCKET")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .process_group(0);
    let kaynak = fdler.to_vec();
    // SAFETY: async-signal-safe fcntl/dup2 in the forked child before exec.
    unsafe {
        komut.pre_exec(move || {
            // Önce hepsini yükseğe kopyala: kaynak fd'ler 3.. aralığına düşebilir.
            let mut yuksek = [0i32; 16];
            for (i, fd) in kaynak.iter().enumerate() {
                yuksek[i] = libc::fcntl(*fd, libc::F_DUPFD, 200);
                if yuksek[i] < 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            for (i, fd) in yuksek.iter().take(kaynak.len()).enumerate() {
                if libc::dup2(*fd, 3 + i as i32) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            Ok(())
        });
    }
    komut.spawn().expect("dil")
}

fn cikis_ve_stderr(mut cocuk: Child, sure: Duration) -> (Option<i32>, String) {
    let son = Instant::now() + sure;
    let durum = loop {
        if let Some(d) = cocuk.try_wait().expect("durum") {
            break d;
        }
        if Instant::now() >= son {
            // SAFETY: SIGKILL to the test-owned process group.
            unsafe { libc::kill(-(cocuk.id() as libc::pid_t), libc::SIGKILL) };
            break cocuk.wait().expect("sonuç");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let mut hata = String::new();
    if let Some(mut boru) = cocuk.stderr.take() {
        let _ = boru.read_to_string(&mut hata);
    }
    (durum.code(), hata)
}

fn bildirim_bekle(soket: &UnixDatagram, sure: Duration) -> Option<String> {
    soket.set_read_timeout(Some(sure)).unwrap();
    let mut tampon = [0u8; 256];
    soket
        .recv(&mut tampon)
        .ok()
        .map(|n| String::from_utf8_lossy(&tampon[..n]).into_owned())
}

/// Dört soket (iki IPv4, iki V6ONLY IPv6) bildirim dışı sırayla gelir; süreç
/// adla eşler, READY=1 gönderir, soketleri açık tutar (bağlantı kuyrukta
/// bekler) ve SIGTERM'de STOPPING=1 ile temiz çıkar.
#[test]
fn karisik_sirali_dort_adli_soket_adla_devralinir_ve_bildirim_gonderilir() {
    let (smtp4, smtp4_a) = bagli("127.0.0.1", false);
    let (smtp6, smtp6_a) = bagli("[::1]", true);
    let (imap4, imap4_a) = bagli("127.0.0.1", false);
    let (imap6, imap6_a) = bagli("[::1]", true);
    let proje = Proje::yeni(&[
        format!("smtp_v4 {smtp4_a}"),
        format!("smtp_v6 {smtp6_a}"),
        format!("imap_v4 {imap4_a}"),
        format!("imap_v6 {imap6_a}"),
    ]);
    let yol = proje.0.join("n.sock");
    let bildirim = UnixDatagram::bind(&yol).expect("notify soketi");
    let fdler = [
        imap6.as_raw_fd(),
        smtp4.as_raw_fd(),
        imap4.as_raw_fd(),
        smtp6.as_raw_fd(),
    ];
    let ortam = format!(
        "NOTIFY_SOCKET={} LISTEN_PID=$$ LISTEN_FDS=4 LISTEN_FDNAMES=imap_v6:smtp_v4:imap_v4:smtp_v6",
        yol.display()
    );
    let kapi = bos_kapi();
    let cocuk = baslat(&proje, &fdler, &ortam, kapi);
    // Devralınan kopyalar çocukta; test kendi uçlarını kapatır ki soketin
    // açık kalması yalnız `dil`in tutmasına bağlı olsun.
    drop((smtp4, smtp6, imap4, imap6));
    let hazir = bildirim_bekle(&bildirim, Duration::from_secs(60));
    assert_eq!(hazir.as_deref(), Some("READY=1"));
    // HTTP dinleyicisi de hazır.
    TcpStream::connect(("127.0.0.1", kapi)).expect("http");
    // Adlı soketler dil'de açık: bağlantı çekirdek kuyruğunda kabul edilir.
    for adres in [smtp4_a, smtp6_a, imap4_a, imap6_a] {
        TcpStream::connect_timeout(&adres, Duration::from_secs(5))
            .unwrap_or_else(|h| panic!("{adres} açık tutulmalı: {h}"));
    }
    // SAFETY: SIGTERM to the test-owned child.
    unsafe { libc::kill(cocuk.id() as libc::pid_t, libc::SIGTERM) };
    let duruyor = bildirim_bekle(&bildirim, Duration::from_secs(30));
    let (kod, hata) = cikis_ve_stderr(cocuk, Duration::from_secs(60));
    assert_eq!(duruyor.as_deref(), Some("STOPPING=1"), "{hata}");
    assert_eq!(kod, Some(0), "{hata}");
    assert!(
        hata.contains("4 dinleyici adla eşlendi") && hata.contains(&format!("smtp_v6={smtp6_a}")),
        "{hata}"
    );
    // Her ileti tek kez: kuyrukta başka bildirim yok.
    assert_eq!(bildirim_bekle(&bildirim, Duration::from_millis(300)), None);
}

/// (ad, bildirim, devredilen fd'ler, LISTEN_* ortamı, beklenen ileti parçası)
type Durum<'a> = (&'a str, &'a [String], Vec<i32>, &'a str, &'a str);

/// Sözleşme ihlalleri çıkış 2 ve düzeltme önerisiyle reddedilir; kaynak
/// koşmaz (HTTP kapısı açılmaz), READY gönderilmez.
#[test]
fn gecersiz_adli_devir_oneriyle_cikis_2_verir() {
    let (a4, a4_a) = bagli("127.0.0.1", false);
    let (a6, a6_a) = bagli("[::1]", true);
    // Linux belirli IPv6 adresine bağlanan sokette V6ONLY'yi kendisi açar;
    // çift yığın yalnız joker adreste anlamlıdır (systemd BindIPv6Only=both).
    let (dual6, dual6_a) = bagli("[::]", false);
    let (baska4, _) = bagli("127.0.0.1", false);
    let iki = [format!("web_v4 {a4_a}"), format!("web_v6 {a6_a}")];
    let ikili_yigin = [format!("web_v4 {a4_a}"), format!("web_v6 {dual6_a}")];
    let durumlar: Vec<Durum> = vec![
        (
            "devir yok",
            &iki,
            vec![],
            "",
            "LISTEN_PID/LISTEN_FDS tanımsız",
        ),
        (
            "başka süreç",
            &iki,
            vec![a4.as_raw_fd(), a6.as_raw_fd()],
            "LISTEN_PID=1 LISTEN_FDS=2 LISTEN_FDNAMES=web_v4:web_v6",
            "bu sürece ait değil",
        ),
        (
            "adsız",
            &iki,
            vec![a4.as_raw_fd(), a6.as_raw_fd()],
            "LISTEN_PID=$$ LISTEN_FDS=2",
            "LISTEN_FDNAMES tanımsız",
        ),
        (
            "eksik ad",
            &iki,
            vec![a4.as_raw_fd()],
            "LISTEN_PID=$$ LISTEN_FDS=1 LISTEN_FDNAMES=web_v4",
            "devredilmeyen: web_v6",
        ),
        (
            "fazla ad",
            &iki,
            vec![a4.as_raw_fd(), a6.as_raw_fd(), baska4.as_raw_fd()],
            "LISTEN_PID=$$ LISTEN_FDS=3 LISTEN_FDNAMES=web_v4:web_v6:smtp_v4",
            "bildirimde olmayan: smtp_v4",
        ),
        (
            "sayı/ad uyuşmazlığı",
            &iki,
            vec![a4.as_raw_fd(), a6.as_raw_fd()],
            "LISTEN_PID=$$ LISTEN_FDS=2 LISTEN_FDNAMES=web_v4",
            "1 ad taşıyor, LISTEN_FDS=2",
        ),
        (
            "adlar yer değiştirmiş",
            &iki,
            vec![a4.as_raw_fd(), a6.as_raw_fd()],
            "LISTEN_PID=$$ LISTEN_FDS=2 LISTEN_FDNAMES=web_v6:web_v4",
            "adresine bağlı, bildirim",
        ),
        (
            "yanlış kapı",
            &iki,
            vec![baska4.as_raw_fd(), a6.as_raw_fd()],
            "LISTEN_PID=$$ LISTEN_FDS=2 LISTEN_FDNAMES=web_v4:web_v6",
            "adresine bağlı, bildirim",
        ),
        (
            "V6ONLY kapalı çift yığın",
            &ikili_yigin,
            vec![a4.as_raw_fd(), dual6.as_raw_fd()],
            "LISTEN_PID=$$ LISTEN_FDS=2 LISTEN_FDNAMES=web_v4:web_v6",
            "IPV6_V6ONLY kapalı",
        ),
    ];
    for (ad, bildirim, fdler, ortam, ifade) in durumlar {
        let proje = Proje::yeni(bildirim);
        let kapi = bos_kapi();
        let cocuk = baslat(&proje, &fdler, ortam, kapi);
        let (kod, hata) = cikis_ve_stderr(cocuk, Duration::from_secs(30));
        assert_eq!(kod, Some(2), "{ad}: {hata}");
        assert!(hata.contains(ifade), "{ad}: {hata}");
        assert!(
            hata.contains("dinleyici devri:") && hata.contains("Öneri:"),
            "{ad}: {hata}"
        );
        assert!(
            !hata.contains("Sunucu dinliyor"),
            "{ad}: kaynak koştu: {hata}"
        );
        assert!(TcpStream::connect(("127.0.0.1", kapi)).is_err(), "{ad}");
    }
    // Reddedilen süreçler testin soketlerini kapatmaz.
    for fd in [&a4, &a6, &dual6, &baska4] {
        // SAFETY: borrowed descriptor stays owned by the test.
        let l = std::mem::ManuallyDrop::new(unsafe { TcpListener::from_raw_fd(fd.as_raw_fd()) });
        l.local_addr().expect("soket açık");
    }
}

/// Geriye uyum: `dinleyiciler` bildirimi olmayan projede LISTEN_* ve
/// NOTIFY_SOCKET bulunsa da devir denenmez (bugünkü tek süreç davranışı);
/// web sunucusu NOTIFY_SOCKET varsa yalnız READY/STOPPING ekler.
#[test]
fn bildirimsiz_proje_listen_ortamini_yok_sayar() {
    let proje = Proje::yeni(&[]);
    let yol = proje.0.join("n.sock");
    let bildirim = UnixDatagram::bind(&yol).expect("notify soketi");
    let kapi = bos_kapi();
    let ortam = format!(
        "NOTIFY_SOCKET={} LISTEN_PID=$$ LISTEN_FDS=3 LISTEN_FDNAMES=a:b:c",
        yol.display()
    );
    let cocuk = baslat(&proje, &[], &ortam, kapi);
    assert_eq!(
        bildirim_bekle(&bildirim, Duration::from_secs(60)).as_deref(),
        Some("READY=1")
    );
    TcpStream::connect(("127.0.0.1", kapi)).expect("kendi bind'i çalışır");
    // SAFETY: SIGTERM to the test-owned child.
    unsafe { libc::kill(cocuk.id() as libc::pid_t, libc::SIGTERM) };
    assert_eq!(
        bildirim_bekle(&bildirim, Duration::from_secs(30)).as_deref(),
        Some("STOPPING=1")
    );
    let (kod, hata) = cikis_ve_stderr(cocuk, Duration::from_secs(60));
    assert_eq!(kod, Some(0), "{hata}");
    assert!(!hata.contains("dinleyici devri"), "{hata}");
}

/// `--güvenli` çocuk modu adlı dinleyici devralmaz.
#[test]
fn guvenli_cocuk_modu_adli_devri_reddeder() {
    let proje = Proje::yeni(&["web_v4 127.0.0.1:18999".into()]);
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", "--güvenli"])
        .arg(&proje.0)
        .env_remove("LISTEN_PID")
        .output()
        .expect("dil");
    assert_eq!(cikti.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&cikti.stderr).contains("çocuk modu adlı dinleyici"));
}
