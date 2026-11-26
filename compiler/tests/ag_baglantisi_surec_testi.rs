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
        // K-384: ayarlar kendi yetkinlik listesini taşıyabilir (dosya aktarımı).
        let yetkinlik = if ayarlar.contains("yetkinlikler ") {
            ""
        } else {
            "yetkinlikler \"ağ-dinleyici\" listesi olsun\n"
        };
        std::fs::write(
            kok.join("proje.dil"),
            format!(
                "proje \"baglanti\" olsun\nsürüm \"0.1.0\" olsun\nmorfoloji \"zee-tr-1\" olsun\ngiriş \"ana.dil\" olsun\n{yetkinlik}ağ_hedefleri boş liste olsun\ndinleyiciler \"smtp_v4 {adres}\" listesi olsun\n{ayarlar}"
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
    /// K-384: blok geçici dizinindeki `.parca` dosyaları.
    fn blok_dosyalari(&self) -> Vec<PathBuf> {
        std::fs::read_dir(self._proje.0.join(".zee/baglanti-bloklari"))
            .map(|girdiler| girdiler.filter_map(|g| g.ok().map(|g| g.path())).collect())
            .unwrap_or_default()
    }

    fn kok(&self) -> &std::path::Path {
        &self._proje.0
    }
}

impl Sunucu {
    fn baslat(ayarlar: &str, kaynak: &str, bildirim: Option<&std::path::Path>) -> Self {
        Self::baslat_ortamla(ayarlar, kaynak, bildirim, "")
    }

    /// K-396: `ek_ortam` kabuk önekine eklenir (ör. `veri_kökü` değişkeni).
    fn baslat_ortamla(
        ayarlar: &str,
        kaynak: &str,
        bildirim: Option<&std::path::Path>,
        ek_ortam: &str,
    ) -> Self {
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
            .arg(format!("{ek_ortam} {ortam} exec \"$0\" \"$@\""))
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

    /// K-384: tam `adet` bayt (satır sonu beklemeden).
    fn bayt(&mut self, adet: usize) -> Vec<u8> {
        let mut tampon = vec![0u8; adet];
        self.okuyucu.read_exact(&mut tampon).expect("bayt");
        tampon
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

/// K-384 (RFC-0039 A2c): SMTP DATA kesiti — blok dosyaya akar, yanıt bayt,
/// SHA-256 ve göreli yol taşır; hata türü 554 ile döner.
pub const DATA_SUNUCUSU: &str = r#"# K-384 test: SMTP DATA gövdesi dosyaya akar.
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    bağlantıya "220 zee.test ESMTP" satırını yaz
    açık doğru olsun
    açık olduğu sürece
        komut bağlantıdan satır okumayı dene olsun
        komut başarılıysa
            metin komutun değeri olsun
            metin "DATA" e eşitse
                bağlantıya "354 bitir" satırını yaz
                ileti bağlantıdan nokta bloğunu dosyaya almayı dene olsun
                ileti başarılıysa
                    bilgi iletinin değeri olsun
                    bağlantıya "250 " ile bilginin "bayt" değeri ile " " ile bilginin "sha256" değeri ile " " ile bilginin "yol" değeri satırını yaz
                değilse
                    hata iletinin hatası olsun
                    veri hatanın verisi olsun
                    son bağlantıya "554 " ile verinin "tür" değeri satırını yazmayı dene olsun
            değilse metin "QUIT" e eşitse
                bağlantıya "221 hoşça kal" satırını yaz
                açık yanlış olsun
            değilse
                bağlantıya "250 " ile metin satırını yaz
        değilse
            hata komutun hatası olsun
            veri hatanın verisi olsun
            son bağlantıya "421 " ile verinin "tür" değeri satırını yazmayı dene olsun
            açık yanlış olsun
    bağlantıyı kapat
"#;

fn sha256(icerik: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(icerik)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn bekle_ki(mut kosul: impl FnMut() -> bool) -> bool {
    let son = Instant::now() + Duration::from_secs(30);
    while Instant::now() < son {
        if kosul() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn data_blogu_dosyaya_akar_smuggling_ve_sinir_oturumu_bozmaz() {
    let ayarlar = "bağlantı_boşta_süresi 20 olsun\nbağlantı_blok_sınırı 1048576 olsun\n";
    let mut sunucu = Sunucu::baslat(ayarlar, DATA_SUNUCUSU, None);
    let mut a = sunucu.baglan();
    assert_eq!(a.satir(), "220 zee.test ESMTP");

    // 900 KiB'lık gövde, 7 KiB'lık parçalar hâlinde; nokta doldurma çözülür.
    let mut govde = Vec::new();
    let mut beklenen = Vec::new();
    let satir = format!("{}\r\n", "z".repeat(900));
    while govde.len() < 900 * 1024 {
        govde.extend_from_slice(b"..b\r\n");
        beklenen.extend_from_slice(b".b\r\n");
        govde.extend_from_slice(satir.as_bytes());
        beklenen.extend_from_slice(satir.as_bytes());
    }
    govde.extend_from_slice(b".\r\n");
    assert_eq!(a.komut("DATA"), "354 bitir");
    for parca in govde.chunks(7 * 1024) {
        a.gonder(parca);
    }
    let yanit = a.satir();
    let alanlar: Vec<&str> = yanit.split(' ').collect();
    assert_eq!(
        alanlar[..3],
        ["250", &beklenen.len().to_string(), &sha256(&beklenen)]
    );
    let yol = sunucu.kok().join(alanlar[3]);
    assert!(alanlar[3].starts_with(".zee/baglanti-bloklari/"), "{yanit}");
    assert_eq!(std::fs::read(&yol).expect("blok dosyası"), beklenen);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let izin = std::fs::metadata(&yol).unwrap().permissions().mode() & 0o777;
        assert_eq!(izin, 0o600);
    }
    // Pipelining: sonlandırıcıdan sonra aynı pakette gelen komut işlenir.
    a.gonder(b"NOOP\r\n");
    assert_eq!(a.satir(), "250 NOOP");

    // SMTP smuggling: <LF>.<LF> sonlandırıcı değildir; kaçak komut blokla atılır.
    assert_eq!(a.komut("DATA"), "354 bitir");
    a.gonder(b"Subject: a\r\n\n.\nMAIL FROM:<kotu@ornek.test>\r\n.\r\n");
    assert_eq!(a.satir(), "554 satır_sonu");
    // Önceki taşınmamış blok yeni blokta silindi.
    assert!(!yol.exists());

    // Sınırı aşan gövde sonuna kadar tüketilir, oturum eşzamanlı sürer.
    assert_eq!(a.komut("DATA"), "354 bitir");
    let buyuk = vec![b'q'; 1_100_000];
    a.gonder(&buyuk);
    a.gonder(b"\r\n.\r\nNOOP\r\n");
    assert_eq!(a.satir(), "554 boyut_aşımı");
    assert_eq!(a.satir(), "250 NOOP");
    assert!(
        sunucu.blok_dosyalari().is_empty(),
        "{:?}",
        sunucu.blok_dosyalari()
    );

    // Tamamlanan ama taşınmayan blok bağlantı bitince silinir.
    assert_eq!(a.komut("DATA"), "354 bitir");
    a.gonder(b"kisa\r\n.\r\n");
    assert!(a.satir().starts_with("250 6 "));
    assert_eq!(sunucu.blok_dosyalari().len(), 1);
    assert_eq!(a.komut("QUIT"), "221 hoşça kal");
    assert!(a.kapandi_mi());
    assert!(bekle_ki(|| sunucu.blok_dosyalari().is_empty()));
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
}

#[test]
fn yavas_ve_kesilen_data_yarim_dosyayi_siler_okuma_yonu_kapanir() {
    let ayarlar = "bağlantı_boşta_süresi 2 olsun\n";
    let mut sunucu = Sunucu::baslat(ayarlar, DATA_SUNUCUSU, None);
    let mut a = sunucu.baglan();
    assert_eq!(a.satir(), "220 zee.test ESMTP");
    assert_eq!(a.komut("DATA"), "354 bitir");
    // Damla damla bayt (hız tabanının çok altında) bütçeyi uzatmaz.
    let baslangic = Instant::now();
    let mut yazici = a.akis.try_clone().unwrap();
    let damla = std::thread::spawn(move || {
        for _ in 0..80 {
            if yazici.write_all(b"d").is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
    assert_eq!(a.satir(), "554 zaman_aşımı");
    let gecen = baslangic.elapsed();
    assert!(gecen < Duration::from_secs(7), "{gecen:?}");
    // Akış eşzamanı bozuk: sonraki okuma `kapandı`, Zee 421 yazıp kapatır.
    assert_eq!(a.satir(), "421 kapandı");
    assert!(a.kapandi_mi());
    let _ = damla.join();
    assert!(bekle_ki(|| sunucu.blok_dosyalari().is_empty()));

    // Blok ortasında kopan istemci: yarım dosya silinir, sunucu sürer.
    let mut b = sunucu.baglan();
    assert_eq!(b.satir(), "220 zee.test ESMTP");
    assert_eq!(b.komut("DATA"), "354 bitir");
    b.gonder(b"Subject: yarim\r\n\r\nbaslangic");
    assert!(bekle_ki(|| sunucu.blok_dosyalari().len() == 1));
    let _ = b.akis.shutdown(std::net::Shutdown::Both);
    assert!(bekle_ki(|| sunucu.blok_dosyalari().is_empty()));
    let mut c = sunucu.baglan();
    assert_eq!(c.satir(), "220 zee.test ESMTP");
    assert_eq!(c.komut("QUIT"), "221 hoşça kal");
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
}

/// K-384 (ZM-F031): IMAP benzeri literal: `+ ` satır sonusuz, literal tam
/// bayt dosyaya, küçük literal bellekte, alınan dosya bağlantıya geri akar.
const LITERAL_SUNUCUSU: &str = r#"# K-384 test: IMAP literal'i ve dosya aktarımı.
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    komut bağlantıdan satır okumayı dene olsun
    komut başarılıysa
        bağlantıya "+ " parçasını yaz
        bağlantıya "hazır" satırını yaz
        ek bağlantıdan 70000 baytı dosyaya almayı dene olsun
        ek başarılıysa
            bilgi ekin değeri olsun
            bağlantıya "A1 OK " ile bilginin "sha256" değeri satırını yaz
            bağlantıya "+ " parçasını yaz
            bağlantıya "parola" satırını yaz
            parola bağlantıdan 6 bayt okumayı dene olsun
            parola başarılıysa
                bağlantıya "A2 OK " ile parolanın değeri satırını yaz
            bağlantıya "* 1 FETCH (BODY[] {70000}" satırını yaz
            geri bağlantıya bilginin "yol" değeri dosyasını aktarmayı dene olsun
            bağlantıya ")" satırını yaz
            geri başarılıysa
                bağlantıya "A3 OK " ile gerinin değeri satırını yaz
    bağlantıyı kapat
"#;

#[test]
fn literal_parca_bayt_okuma_ve_dosya_aktarimi_bayt_kesindir() {
    let ayarlar = "yetkinlikler \"ağ-dinleyici\", \"dosya-okuma\" listesi olsun\nbağlantı_boşta_süresi 20 olsun\n";
    let mut sunucu = Sunucu::baslat(ayarlar, LITERAL_SUNUCUSU, None);
    let mut a = sunucu.baglan();
    a.gonder(b"A1 APPEND kutu {70000}\r\n");
    // Satır sonu eklenmeden yazılan devam yanıtı: tam iki bayt.
    assert_eq!(a.bayt(2), b"+ ");
    assert_eq!(a.satir(), "hazır");
    // İkili literal (CRLF, NUL, 8 bit dahil) ve ardından gelen fazla bayt.
    let literal: Vec<u8> = (0..70000u32).map(|i| (i % 251) as u8).collect();
    a.gonder(&literal[..30000]);
    std::thread::sleep(Duration::from_millis(50));
    a.gonder(&literal[30000..]);
    assert_eq!(a.satir(), format!("A1 OK {}", sha256(&literal)));
    assert_eq!(a.bayt(2), b"+ ");
    assert_eq!(a.satir(), "parola");
    a.gonder(b"gizli1");
    assert_eq!(a.satir(), "A2 OK gizli1");
    assert_eq!(a.satir(), "* 1 FETCH (BODY[] {70000}");
    assert_eq!(a.bayt(70000), literal);
    assert_eq!(a.satir(), ")");
    assert_eq!(a.satir(), "A3 OK 70000");
    assert!(a.kapandi_mi());
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
}

/// K-384 (ZM-F032): IDLE benzeri kısa süreli okuma; zaman aşımında oturum
/// sürer, sunucu yoklama satırını yazar ve beklemeye döner.
const IDLE_SUNUCUSU: &str = r#"# K-384 test: IDLE benzeri yoklama.
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    bağlantıya "+ idling" satırını yaz
    açık doğru olsun
    yoklama 0 olsun
    açık olduğu sürece
        satır bağlantıdan satır okumayı 1 saniye içinde dene olsun
        satır başarılıysa
            bağlantıya "A1 OK " ile satırın değeri satırını yaz
            açık yanlış olsun
        değilse
            hata satırın hatası olsun
            veri hatanın verisi olsun
            tür verinin "tür" değeri olsun
            tür "zaman_aşımı" e eşitse
                yoklamayı 1 artır
                bağlantıya "* yoklama " ile yoklama satırını yaz
            değilse
                açık yanlış olsun
    bağlantıyı kapat
"#;

#[test]
fn sureli_okuma_zaman_asiminda_oturum_surer_yarim_satir_korunur() {
    let ayarlar = "bağlantı_boşta_süresi 120 olsun\n";
    let mut sunucu = Sunucu::baslat(ayarlar, IDLE_SUNUCUSU, None);
    let mut a = sunucu.baglan();
    let baslangic = Instant::now();
    assert_eq!(a.satir(), "+ idling");
    assert_eq!(a.satir(), "* yoklama 1");
    // Yarım satır zaman aşımında atılmaz; sonraki okuma onu tamamlar.
    a.gonder(b"DO");
    assert_eq!(a.satir(), "* yoklama 2");
    assert!(baslangic.elapsed() < Duration::from_secs(30));
    a.gonder(b"NE\r\n");
    assert_eq!(a.satir(), "A1 OK DONE");
    assert!(a.kapandi_mi());
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
}

/// K-396 (RFC-0039 A2d, ZM-F040): IMAP `BODY[n]` için satırlı base64 akışı,
/// kodlanmış çıktı üzerinde aralık (`BODY[n]<o.n>`), kaçış ve yokluk.
const BASE64_SUNUCUSU: &str = r#"# K-396 test: satırlı base64 aktarımı.
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    komut bağlantıdan satır okumayı dene olsun
    komut başarılıysa
        kodlu "ekler/ek.bin" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun
        bağlantıya ")" satırını yaz
        kodlu başarılıysa
            bağlantıya "A1 OK " ile kodlunun değeri satırını yaz
        parça "ekler/ek.bin" dosyasının base64 satırlarının 77 baytından 1024 baytını bağlantıya aktarmayı dene olsun
        bağlantıya ")" satırını yaz
        parça başarılıysa
            bağlantıya "A2 OK " ile parçanın değeri satırını yaz
        uç "ekler/ek.bin" dosyasının base64 satırlarının 999999999 baytından 10 baytını bağlantıya aktarmayı dene olsun
        uç başarılıysa
            bağlantıya "A3 OK " ile ucun değeri satırını yaz
        kaçış "../dis.bin" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun
        kaçış başarısızsa
            hata kaçışın hatası olsun
            veri hatanın verisi olsun
            bağlantıya "A4 NO " ile verinin "tür" değeri satırını yaz
        eksik "ekler/eksik.bin" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun
        eksik başarısızsa
            hata eksiğin hatası olsun
            veri hatanın verisi olsun
            bağlantıya "A5 NO " ile verinin "tür" değeri satırını yaz
    bağlantıyı kapat
"#;

/// Kütüphaneden bağımsız başvuru: bit tamponlu kodlama, 76'da bir CRLF.
fn basvuru_base64_satirli(veri: &[u8]) -> Vec<u8> {
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let (mut kod, mut tampon, mut bit) = (Vec::with_capacity(veri.len() / 3 * 4 + 4), 0u32, 0u32);
    for &b in veri {
        tampon = ((tampon << 8) | u32::from(b)) & 0xffff;
        bit += 8;
        while bit >= 6 {
            bit -= 6;
            kod.push(A[((tampon >> bit) & 63) as usize]);
        }
    }
    if bit > 0 {
        kod.push(A[((tampon << (6 - bit)) & 63) as usize]);
    }
    while kod.len() % 4 != 0 {
        kod.push(b'=');
    }
    let mut cikti = Vec::with_capacity(kod.len() + kod.len() / 38 + 2);
    for satir in kod.chunks(76) {
        cikti.extend_from_slice(satir);
        cikti.extend_from_slice(b"\r\n");
    }
    cikti
}

fn ikili_ek(boy: usize) -> Vec<u8> {
    (0..boy as u64)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8)
        .collect()
}

#[test]
fn base64_satirli_aktarim_16_mib_bayt_kesin_aralikli_ve_kacissizdir() {
    let ayarlar = "yetkinlikler \"ağ-dinleyici\", \"dosya-okuma\" listesi olsun\nbağlantı_boşta_süresi 20 olsun\n";
    let mut sunucu = Sunucu::baslat(ayarlar, BASE64_SUNUCUSU, None);
    // ZM-F040 ölçeği: 16 MiB + 5 bayt (son satır kısa, dolgulu).
    let ek = ikili_ek(16 * 1024 * 1024 + 5);
    std::fs::create_dir_all(sunucu.kok().join("ekler")).unwrap();
    std::fs::write(sunucu.kok().join("ekler/ek.bin"), &ek).unwrap();
    let beklenen = basvuru_base64_satirli(&ek);
    let e = 4 * ek.len().div_ceil(3);
    assert_eq!(beklenen.len(), e + 2 * e.div_ceil(76));
    let mut a = sunucu.baglan();
    let baslangic = Instant::now();
    a.gonder(b"A1 FETCH 1 BODY[2]\r\n");
    assert!(a.bayt(beklenen.len()) == beklenen, "kodlanmış çıktı farklı");
    let sure = baslangic.elapsed();
    assert_eq!(a.satir(), ")");
    assert_eq!(a.satir(), format!("A1 OK {}", beklenen.len()));
    // Aralık satır ortasından başlar (77 = birinci satırın LF'si).
    assert_eq!(a.bayt(1024), &beklenen[77..77 + 1024]);
    assert_eq!(a.satir(), ")");
    assert_eq!(a.satir(), "A2 OK 1024");
    assert_eq!(a.satir(), "A3 OK 0");
    assert_eq!(a.satir(), "A4 NO dosya");
    assert_eq!(a.satir(), "A5 NO dosya");
    assert!(a.kapandi_mi());
    // ZM-F040 geçici çözümü (57 baytlık okuma döngüsü) 29,6 sn sürüyordu.
    assert!(sure < Duration::from_secs(20), "16 MiB aktarım {sure:?}");
    eprintln!("K-396 16 MiB satırlı base64 aktarımı: {sure:?}");
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
}

/// K-396: kaynak dosya tavanı 32 MiB (K-395 yazma tavanıyla hizalı);
/// `bağlantı_blok_sınırı` 1 GiB olsa da tavanı aşan dosyadan bayt çıkmaz.
const BASE64_TAVAN: &str = r#"# K-396 test: 32 MiB kaynak tavanı.
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    komut bağlantıdan satır okumayı dene olsun
    komut başarılıysa
        tam "ekler/tavan.bin" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun
        bağlantıya ")" satırını yaz
        tam başarılıysa
            bağlantıya "B1 OK " ile tamın değeri satırını yaz
        son "ekler/tavan.bin" dosyasının base64 satırlarının 45916494 baytından 100 baytını bağlantıya aktarmayı dene olsun
        bağlantıya ")" satırını yaz
        son başarılıysa
            bağlantıya "B2 OK " ile sonun değeri satırını yaz
        fazla "ekler/fazla.bin" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun
        fazla başarısızsa
            hata fazlanın hatası olsun
            veri hatanın verisi olsun
            bağlantıya "B3 NO " ile verinin "tür" değeri satırını yaz
        dilim "ekler/fazla.bin" dosyasının base64 satırlarının 0 baytından 10 baytını bağlantıya aktarmayı dene olsun
        dilim başarısızsa
            hata dilimin hatası olsun
            veri hatanın verisi olsun
            bağlantıya "B4 NO " ile verinin "tür" değeri satırını yaz
    bağlantıyı kapat
"#;

#[test]
fn base64_aktarim_kaynak_tavani_32_mib_gecer_bir_bayt_fazlasi_reddedilir() {
    let ayarlar = "yetkinlikler \"ağ-dinleyici\", \"dosya-okuma\" listesi olsun\nbağlantı_boşta_süresi 20 olsun\nbağlantı_blok_sınırı 1073741824 olsun\n";
    let mut sunucu = Sunucu::baslat(ayarlar, BASE64_TAVAN, None);
    const TAVAN: usize = 32 * 1024 * 1024;
    let ek = ikili_ek(TAVAN + 1);
    std::fs::create_dir_all(sunucu.kok().join("ekler")).unwrap();
    std::fs::write(sunucu.kok().join("ekler/tavan.bin"), &ek[..TAVAN]).unwrap();
    std::fs::write(sunucu.kok().join("ekler/fazla.bin"), &ek).unwrap();
    let beklenen = basvuru_base64_satirli(&ek[..TAVAN]);
    // e = 4·⌈b/3⌉ = 44 739 244; toplam = e + 2·⌈e/76⌉ = 45 916 594.
    assert_eq!(beklenen.len(), 45_916_594);
    let mut a = sunucu.baglan();
    a.gonder(b"B1 FETCH 1 BODY[2]\r\n");
    assert!(a.bayt(beklenen.len()) == beklenen, "tavandaki dosya farklı");
    assert_eq!(a.satir(), ")");
    assert_eq!(a.satir(), format!("B1 OK {}", beklenen.len()));
    // Aralıklı biçim tavandaki dosyanın son 100 baytına konumlanır.
    assert_eq!(a.bayt(100), &beklenen[beklenen.len() - 100..]);
    assert_eq!(a.satir(), ")");
    assert_eq!(a.satir(), "B2 OK 100");
    assert_eq!(a.satir(), "B3 NO boyut_aşımı");
    assert_eq!(a.satir(), "B4 NO boyut_aşımı");
    assert!(a.kapandi_mi());
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
}

#[test]
fn base64_aktarim_dosya_okuma_yetkinligi_olmadan_denetimden_gecmez() {
    let dinleyici = TcpListener::bind("127.0.0.1:0").unwrap();
    let proje = Proje::yeni(dinleyici.local_addr().unwrap(), "", BASE64_SUNUCUSU);
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["denetle"])
        .arg(&proje.0)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&cikti.stderr);
    assert!(!cikti.status.success(), "{stderr}");
    assert!(
        stderr.contains("T054") && stderr.contains("dosya-okuma"),
        "{stderr}"
    );
}

const BASE64_KOPAN: &str = r#"# K-396 test: karşı taraf akış ortasında kapatır.
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    komut bağlantıdan satır okumayı dene olsun
    kodlu "ekler/ek.bin" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun
    kodlu başarısızsa
        hata kodlunun hatası olsun
        veri hatanın verisi olsun
        "sonuc.txt" dosyasına verinin "tür" değeri yaz
    bağlantıyı kapat
"#;

#[test]
fn base64_aktarimda_kopan_karsi_taraf_yazilamadi_sonucudur() {
    let ayarlar = "yetkinlikler \"ağ-dinleyici\", \"dosya-okuma\", \"dosya-yazma\" listesi olsun\nbağlantı_boşta_süresi 20 olsun\n";
    let mut sunucu = Sunucu::baslat(ayarlar, BASE64_KOPAN, None);
    std::fs::create_dir_all(sunucu.kok().join("ekler")).unwrap();
    std::fs::write(
        sunucu.kok().join("ekler/ek.bin"),
        ikili_ek(16 * 1024 * 1024),
    )
    .unwrap();
    let mut a = sunucu.baglan();
    a.gonder(b"A1 FETCH 1 BODY[2]\r\n");
    assert_eq!(a.bayt(4096).len(), 4096);
    drop(a);
    let sonuc = sunucu.kok().join("sonuc.txt");
    assert!(
        bekle_ki(|| std::fs::read_to_string(&sonuc).is_ok_and(|m| m.contains("yazılamadı"))),
        "{:?}",
        std::fs::read_to_string(&sonuc)
    );
    // Sunucu ayakta: sonraki bağlantı yeniden aktarıma başlar.
    let mut b = sunucu.baglan();
    b.gonder(b"A2 FETCH 1 BODY[2]\r\n");
    assert_eq!(b.bayt(76), &basvuru_base64_satirli(&ikili_ek(57))[..76]);
    drop(b);
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
}

const BASE64_VERI: &str = r#"# K-396 test: paylaşılan veri kökünden aktarım.
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    komut bağlantıdan satır okumayı dene olsun
    kodlu "veri:ekler/t1-ab12" dosyasını base64 satırlarıyla bağlantıya aktarmayı dene olsun
    kodlu başarılıysa
        bağlantıya "A1 OK " ile kodlunun değeri satırını yaz
    kaçış "veri:../sir.txt" dosyasının base64 satırlarının 0 baytından 10 baytını bağlantıya aktarmayı dene olsun
    kaçış başarısızsa
        hata kaçışın hatası olsun
        veri hatanın verisi olsun
        bağlantıya "A2 NO " ile verinin "tür" değeri satırını yaz
    bağlantıyı kapat
"#;

#[test]
fn base64_aktarim_veri_kokunden_okur_kok_disina_cikmaz() {
    use std::os::unix::fs::PermissionsExt;
    let taban = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "zee-k396-{}-{}",
        std::process::id(),
        SIRA.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(taban.join("depo/ekler")).unwrap();
    std::fs::write(taban.join("sir.txt"), b"kok disi sir").unwrap();
    for (klasor, kip) in [("", 0o700), ("depo", 0o755), ("depo/ekler", 0o770)] {
        std::fs::set_permissions(taban.join(klasor), std::fs::Permissions::from_mode(kip)).unwrap();
    }
    let ek = ikili_ek(1000);
    std::fs::write(taban.join("depo/ekler/t1-ab12"), &ek).unwrap();
    let ayarlar = "yetkinlikler \"ağ-dinleyici\", \"veri-okuma\" listesi olsun\nveri_kökü \"ZEE_K396_DEPO\" olsun\nbağlantı_boşta_süresi 20 olsun\n";
    let ortam = format!("ZEE_K396_DEPO='{}'", taban.join("depo").display());
    let mut sunucu = Sunucu::baslat_ortamla(ayarlar, BASE64_VERI, None, &ortam);
    let mut a = sunucu.baglan();
    a.gonder(b"A1 FETCH 1 BODY[2]\r\n");
    let beklenen = basvuru_base64_satirli(&ek);
    assert_eq!(a.bayt(beklenen.len()), beklenen);
    assert_eq!(a.satir(), format!("A1 OK {}", beklenen.len()));
    assert_eq!(a.satir(), "A2 NO dosya");
    assert!(a.kapandi_mi());
    sunucu.sigterm();
    let (kod, stderr) = sunucu.bitir(Duration::from_secs(30));
    assert_eq!(kod, Some(0), "{stderr}");
    let _ = std::fs::remove_dir_all(&taban);
}
