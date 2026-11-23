//! K-381 (RFC-0039 A2a): gerçek süreç + gerçek TCP ile bağlantı yüzeyi.
//!
//! systemd soket etkinleştirmesi K-376 testindeki gibi taklit edilir: test
//! dinleyiciyi açar, `sh -c 'LISTEN_PID=$$ … exec dil …'` ile fd 3'e devreder.
//! Sabit kapı yoktur (çekirdek seçer); zaman eşikleri CI yüküne dayanıklı
//! geniş pencerelerdir.
#![cfg(unix)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixDatagram;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static SIRA: AtomicUsize = AtomicUsize::new(0);

pub const YANKI_SUNUCUSU: &str = r#"# K-381 test: SMTP benzeri satır yankı sunucusu.
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    bağlantıya "220 zee.test ESMTP" satırını yaz
    açık doğru olsun
    açık olduğu sürece
        komut bağlantıdan satır okumayı dene olsun
        komut başarılıysa
            metin komutun değeri olsun
            metin "QUIT" e eşitse
                bağlantıya "221 hoşça kal" satırını yaz
                açık yanlış olsun
            değilse
                bağlantıya "250 " ile metin satırını yaz
        değilse
            hata komutun hatası olsun
            veri hatanın verisi olsun
            tür verinin "tür" değeri olsun
            tür "kapandı" e eşitse
                açık yanlış olsun
            değilse
                tür "zaman_aşımı" e eşitse
                    bağlantıya "421 4.4.2 zaman aşımı" satırını yaz
                    açık yanlış olsun
                değilse
                    tür "kapanıyor" e eşitse
                        bağlantıya "421 4.3.2 kapanıyor" satırını yaz
                        açık yanlış olsun
                    değilse
                        bağlantıya "500 " ile tür satırını yaz
    bağlantıyı kapat
"#;

struct Proje(PathBuf);

impl Proje {
    fn yeni(adres: SocketAddr, ayarlar: &str, kaynak: &str) -> Self {
        let kok = PathBuf::from("/tmp").join(format!(
            "zk381-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&kok).expect("proje kökü");
        std::fs::write(
            kok.join("proje.dil"),
            format!(
                "proje \"baglanti\" olsun\nsürüm \"0.1.0\" olsun\nmorfoloji \"zee-tr-1\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ-dinleyici\" listesi olsun\nağ_hedefleri boş liste olsun\ndinleyiciler \"smtp_v4 {adres}\" listesi olsun\n{ayarlar}"
            ),
        )
        .expect("proje.dil");
        std::fs::write(kok.join("ana.dil"), kaynak).expect("ana.dil");
        Self(kok)
    }
}

impl Drop for Proje {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Sunucu {
    cocuk: Option<Child>,
    adres: SocketAddr,
    _proje: Proje,
}

impl Sunucu {
    fn baslat(ayarlar: &str, kaynak: &str, bildirim: Option<&std::path::Path>) -> Self {
        use std::os::unix::process::CommandExt;
        let dinleyici = TcpListener::bind("127.0.0.1:0").expect("dinleyici");
        let adres = dinleyici.local_addr().unwrap();
        let proje = Proje::yeni(adres, ayarlar, kaynak);
        let ortam = match bildirim {
            Some(yol) => format!(
                "NOTIFY_SOCKET={} LISTEN_PID=$$ LISTEN_FDS=1 LISTEN_FDNAMES=smtp_v4",
                yol.display()
            ),
            None => "LISTEN_PID=$$ LISTEN_FDS=1 LISTEN_FDNAMES=smtp_v4".into(),
        };
        let mut komut = Command::new("/bin/sh");
        komut
            .arg("-c")
            .arg(format!("{ortam} exec \"$0\" \"$@\""))
            .arg(env!("CARGO_BIN_EXE_dil"))
            .arg("çalıştır")
            .arg(&proje.0)
            .current_dir(&proje.0)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .process_group(0);
        if bildirim.is_none() {
            komut.env_remove("NOTIFY_SOCKET");
        }
        let fd = dinleyici.as_raw_fd();
        // SAFETY: async-signal-safe dup2 in the forked child before exec.
        unsafe {
            komut.pre_exec(move || {
                // Kaynak fd zaten 3 olabilir: dup2(3, 3) FD_CLOEXEC'i temizlemez.
                let yuksek = libc::fcntl(fd, libc::F_DUPFD, 200);
                if yuksek < 0 || libc::dup2(yuksek, 3) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let cocuk = komut.spawn().expect("dil");
        drop(dinleyici);
        Self {
            cocuk: Some(cocuk),
            adres,
            _proje: proje,
        }
    }

    fn baglan(&mut self) -> Istemci {
        let akis = match TcpStream::connect_timeout(&self.adres, Duration::from_secs(30)) {
            Ok(akis) => akis,
            Err(hata) => {
                let cocuk = self.cocuk.take().unwrap();
                let cikti = cocuk.wait_with_output().unwrap();
                panic!(
                    "bağlantı: {hata}; dil {:?}: {}",
                    cikti.status,
                    String::from_utf8_lossy(&cikti.stderr)
                );
            }
        };
        akis.set_read_timeout(Some(Duration::from_secs(60)))
            .unwrap();
        Istemci {
            okuyucu: BufReader::new(akis.try_clone().unwrap()),
            akis,
        }
    }

    fn sigterm(&self) {
        let pid = self.cocuk.as_ref().unwrap().id() as libc::pid_t;
        // SAFETY: signal to the test-owned child process.
        unsafe { libc::kill(pid, libc::SIGTERM) };
    }

    fn bitir(mut self, sure: Duration) -> (Option<i32>, String) {
        let mut cocuk = self.cocuk.take().unwrap();
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
}

impl Drop for Sunucu {
    fn drop(&mut self) {
        if let Some(mut cocuk) = self.cocuk.take() {
            // SAFETY: SIGKILL to the test-owned process group.
            unsafe { libc::kill(-(cocuk.id() as libc::pid_t), libc::SIGKILL) };
            let _ = cocuk.wait();
        }
    }
}

struct Istemci {
    akis: TcpStream,
    okuyucu: BufReader<TcpStream>,
}

impl Istemci {
    fn satir(&mut self) -> String {
        let mut satir = String::new();
        self.okuyucu.read_line(&mut satir).expect("satır");
        assert!(satir.ends_with("\r\n") || satir.is_empty(), "{satir:?}");
        satir.trim_end_matches("\r\n").to_string()
    }

    fn gonder(&mut self, bayt: &[u8]) {
        self.akis.write_all(bayt).expect("gönder");
    }

    fn komut(&mut self, satir: &str) -> String {
        self.gonder(format!("{satir}\r\n").as_bytes());
        self.satir()
    }

    fn kapandi_mi(&mut self) -> bool {
        let mut tampon = [0u8; 1];
        matches!(self.okuyucu.read(&mut tampon), Ok(0) | Err(_))
    }
}

const AYARLAR: &str = "bağlantı_satır_sınırı 128 olsun\nbağlantı_boşta_süresi 3 olsun\nbağlantı_sınırı 3 olsun\nuzak_adres_bağlantı_sınırı 3 olsun\nbağlantı_dolu_yanıtı \"421 4.3.2 Hizmet mesgul\" olsun\n";

#[test]
fn uctan_uca_satirlar_sinirlar_ve_quit() {
    let mut sunucu = Sunucu::baslat(AYARLAR, YANKI_SUNUCUSU, None);
    let mut a = sunucu.baglan();
    assert_eq!(a.satir(), "220 zee.test ESMTP");
    assert_eq!(a.komut("EHLO istemci.test"), "250 EHLO istemci.test");
    // SMTP smuggling sınıfı: çıplak LF ve <CR>.<CR> satır sonu sayılmaz.
    a.gonder(b"MAIL FROM:<a@b>\n");
    assert_eq!(a.satir(), "500 satır_sonu");
    a.gonder(b"x\r.\rRCPT TO:<c@d>\r\n");
    assert_eq!(a.satir(), "500 satır_sonu");
    // 128 baytı aşan satır kırpılmadan reddedilir; kalanı atlanır.
    a.gonder(format!("{}\r\n", "A".repeat(300)).as_bytes());
    assert_eq!(a.satir(), "500 satır_uzun");
    a.gonder(b"\xff\xfe\r\n");
    assert_eq!(a.satir(), "500 kodlama");
    // Aynı pakette iki komut sırayla işlenir (pipelining).
    a.gonder(b"NOOP\r\nRSET\r\n");
    assert_eq!(a.satir(), "250 NOOP");
    assert_eq!(a.satir(), "250 RSET");
    assert_eq!(a.komut("QUIT"), "221 hoşça kal");
    assert!(a.kapandi_mi());
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
    assert!(stderr.contains("bağlantı kabul ediliyor"), "{stderr}");
}

#[test]
fn eszamanli_oturumlar_birbirini_beklemez_ve_sinir_kibar_ret_verir() {
    let mut sunucu = Sunucu::baslat(AYARLAR, YANKI_SUNUCUSU, None);
    let mut sessiz = sunucu.baglan();
    assert_eq!(sessiz.satir(), "220 zee.test ESMTP");
    let mut b = sunucu.baglan();
    let mut c = sunucu.baglan();
    assert_eq!(b.satir(), "220 zee.test ESMTP");
    assert_eq!(c.satir(), "220 zee.test ESMTP");
    // Sessiz bağlantı okumada beklerken diğerleri hemen yanıt alır.
    let bas = Instant::now();
    assert_eq!(b.komut("EHLO b"), "250 EHLO b");
    assert_eq!(c.komut("EHLO c"), "250 EHLO c");
    assert_eq!(b.komut("NOOP"), "250 NOOP");
    assert!(
        bas.elapsed() < Duration::from_secs(2),
        "{:?}",
        bas.elapsed()
    );
    // Dördüncü bağlantı: sınır dolu → kibar ret satırı ve kapanış.
    let mut d = sunucu.baglan();
    assert_eq!(d.satir(), "421 4.3.2 Hizmet mesgul");
    assert!(d.kapandi_mi());
    assert_eq!(b.komut("QUIT"), "221 hoşça kal");
    assert_eq!(c.komut("QUIT"), "221 hoşça kal");
    drop(sessiz);
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
    assert!(stderr.contains("kibar retle"), "{stderr}");
}

#[test]
fn yavas_istemci_damla_damla_bayt_ile_sureyi_uzatamaz() {
    let mut sunucu = Sunucu::baslat(AYARLAR, YANKI_SUNUCUSU, None);
    let mut a = sunucu.baglan();
    assert_eq!(a.satir(), "220 zee.test ESMTP");
    let bas = Instant::now();
    // 3 sn boşta süresi mutlaktır: her 300 ms'de bir bayt süreyi uzatmaz.
    let mut akis = a.akis.try_clone().unwrap();
    let damla = std::thread::spawn(move || {
        for _ in 0..40 {
            if akis.write_all(b"E").is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(300));
        }
    });
    assert_eq!(a.satir(), "421 4.4.2 zaman aşımı");
    let gecen = bas.elapsed();
    assert!(
        gecen >= Duration::from_millis(2500) && gecen < Duration::from_secs(10),
        "{gecen:?}"
    );
    assert!(a.kapandi_mi());
    let _ = damla.join();
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
}

#[test]
fn sigterm_bosaltmasi_bekleyen_okumaya_kapaniyor_verir_ve_bildirir() {
    let kok = PathBuf::from("/tmp").join(format!(
        "zk381n-{}-{}",
        std::process::id(),
        SIRA.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&kok).unwrap();
    let yol = kok.join("n.sock");
    let bildirim = UnixDatagram::bind(&yol).expect("notify");
    bildirim
        .set_read_timeout(Some(Duration::from_secs(60)))
        .unwrap();
    let mut sunucu = Sunucu::baslat(
        "bağlantı_boşta_süresi 120 olsun\n",
        YANKI_SUNUCUSU,
        Some(&yol),
    );
    let mut a = sunucu.baglan();
    assert_eq!(a.satir(), "220 zee.test ESMTP");
    let mut tampon = [0u8; 64];
    let n = bildirim.recv(&mut tampon).expect("READY");
    assert_eq!(&tampon[..n], b"READY=1");
    sunucu.sigterm();
    assert_eq!(a.satir(), "421 4.3.2 kapanıyor");
    assert!(a.kapandi_mi());
    let n = bildirim.recv(&mut tampon).expect("STOPPING");
    assert_eq!(&tampon[..n], b"STOPPING=1");
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
    let _ = std::fs::remove_dir_all(&kok);
}

#[test]
fn baglanti_blogu_dinleyicisi_yetkinligi_ve_bildirimi_olmadan_calismaz() {
    // Yetkinlik yok: denetim aşamasında yetkinlik reddi, kaynak yürütülmez.
    let dinleyici = TcpListener::bind("127.0.0.1:0").unwrap();
    let adres = dinleyici.local_addr().unwrap();
    let proje = Proje::yeni(adres, "", YANKI_SUNUCUSU);
    let pd = std::fs::read_to_string(proje.0.join("proje.dil"))
        .unwrap()
        .replace(
            "yetkinlikler \"ağ-dinleyici\" listesi olsun",
            "yetkinlikler boş liste olsun",
        );
    std::fs::write(proje.0.join("proje.dil"), pd).unwrap();
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["denetle"])
        .arg(&proje.0)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&cikti.stderr);
    assert!(!cikti.status.success(), "{stderr}");
    let stdout = String::from_utf8_lossy(&cikti.stdout);
    assert!(
        stderr.contains("T054") && stderr.contains("ağ-dinleyici"),
        "{stdout}{stderr}"
    );
    // Devir yokken (systemd dışı) bind'e geri dönülmez: çıkış 2.
    let pd = std::fs::read_to_string(proje.0.join("proje.dil"))
        .unwrap()
        .replace(
            "yetkinlikler boş liste olsun",
            "yetkinlikler \"ağ-dinleyici\" listesi olsun",
        );
    std::fs::write(proje.0.join("proje.dil"), pd).unwrap();
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır"])
        .arg(&proje.0)
        .env_remove("LISTEN_PID")
        .env_remove("LISTEN_FDS")
        .output()
        .unwrap();
    assert_eq!(cikti.status.code(), Some(2));
    drop(dinleyici);
}

const BUYUK_YAZAN: &str = r#""smtp_v4" dinleyicisinde bağlantı geldiğinde
    komut bağlantıdan satır okumayı dene olsun
    parça "0123456789" olsun
    satır parça ile parça ile parça ile parça ile parça ile parça ile parça ile parça olsun
    sıra 0 olsun
    3000 kez tekrarla
        sıra 1 artır
        bağlantıya sıra ile " " ile satır satırını yaz
    bağlantıya "SON" satırını yaz
    bağlantıyı kapat
"#;

/// 64 KiB giden tampon dolunca yazma görevi askıya alınır (geri basınç);
/// okumayan istemci sonradan okuduğunda bütün satırlar sırayla gelir, kardeş
/// bağlantı bu sırada hizmet almaya devam eder.
#[test]
fn geri_basinc_giden_tamponu_sinirlar_ve_kayipsiz_teslim_eder() {
    let kaynak = format!("{BUYUK_YAZAN}\n");
    let mut sunucu = Sunucu::baslat("bağlantı_boşta_süresi 30 olsun\n", &kaynak, None);
    let mut a = sunucu.baglan();
    a.gonder(b"BASLA\r\n");
    std::thread::sleep(Duration::from_millis(800));
    // Kardeş bağlantı geri basınç altındaki görevi beklemez.
    let mut b = sunucu.baglan();
    b.gonder(b"BASLA\r\n");
    assert_eq!(b.satir(), format!("1 {}", "0123456789".repeat(8)));
    for i in 1..=3000 {
        assert_eq!(a.satir(), format!("{i} {}", "0123456789".repeat(8)));
    }
    assert_eq!(a.satir(), "SON");
    assert!(a.kapandi_mi());
    drop(b);
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
}

/// Hiç okumayan istemci: giden tampon boşta süresi içinde boşalmazsa yazma
/// cümlesi C030 ile görevi bitirir, bağlantı kapanır, süreç yaşar.
#[test]
fn hic_okumayan_istemci_yazma_suresinde_kesilir() {
    let kaynak = format!("{BUYUK_YAZAN}\n");
    let mut sunucu = Sunucu::baslat("bağlantı_boşta_süresi 2 olsun\n", &kaynak, None);
    let mut a = sunucu.baglan();
    a.akis.set_nonblocking(false).unwrap();
    // Alma tamponunu küçült ki çekirdek tamponları dolsun.
    a.gonder(b"BASLA\r\n");
    std::thread::sleep(Duration::from_secs(6));
    let mut yeni = sunucu.baglan();
    yeni.gonder(b"BASLA\r\n");
    assert!(yeni.satir().starts_with("1 "));
    drop(yeni);
    drop(a);
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
    assert!(
        stderr.contains("C030") && stderr.contains("yazılamadı"),
        "{stderr}"
    );
}
