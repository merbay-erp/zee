//! K-381 (RFC-0039 A2b): adlı dinleyicide sunucu TLS'i — implicit TLS,
//! STARTTLS, STARTTLS komut enjeksiyonu (CVE-2011-0411 sınıfı) reddi, TLS
//! sürüm/şifre politikası ve kimlik yükleme. Gerçek süreç + gerçek TCP;
//! sertifikalar test anında `openssl` ile üretilir (depoda özel anahtar yok).
#![cfg(unix)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

static SIRA: AtomicUsize = AtomicUsize::new(0);

const SUNUCU: &str = r#"# K-381 A2b test: STARTTLS'li SMTP benzeri yankı.
"posta" dinleyicisinde bağlantı geldiğinde
    bağlantıya "220 zee.test ESMTP tls=" ile bağlantının "tls" değeri satırını yaz
    açık doğru olsun
    açık olduğu sürece
        komut bağlantıdan satır okumayı dene olsun
        komut başarılıysa
            metin komutun değeri olsun
            metin "STARTTLS" e eşitse
                bağlantıya "220 2.0.0 hazır" satırını yaz
                yukseltme bağlantıda TLS başlatmayı dene olsun
                yukseltme başarılıysa
                    bagli yukseltmenin değeri olsun
                    bağlantıya "250 tls " ile bagli satırını yaz
                değilse
                    hata yukseltmenin hatası olsun
                    veri hatanın verisi olsun
                    tür verinin "tür" değeri olsun
                    son bağlantıya "454 " ile tür satırını yazmayı dene olsun
                    açık yanlış olsun
            değilse
                metin "QUIT" e eşitse
                    bağlantıya "221 hoşça kal" satırını yaz
                    açık yanlış olsun
                değilse
                    bağlantıya "250 " ile metin satırını yaz
        değilse
            açık yanlış olsun
    bağlantıyı kapat
"#;

struct Kimlik {
    kok: PathBuf,
    ca: Vec<u8>,
    zincir: String,
    anahtar: String,
}

impl Drop for Kimlik {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.kok);
    }
}

fn openssl(kok: &Path, argumanlar: &[&str]) {
    let cikti = Command::new("openssl")
        .args(argumanlar)
        .current_dir(kok)
        .output()
        .expect("openssl bulunamadı (TLS testleri test sertifikasını openssl ile üretir)");
    assert!(
        cikti.status.success(),
        "openssl {argumanlar:?}: {}",
        String::from_utf8_lossy(&cikti.stderr)
    );
}

/// Test CA + localhost yaprak sertifikası (EC P-256), yalnız bu koşu için.
fn kimlik_uret() -> Kimlik {
    let kok = PathBuf::from("/tmp").join(format!(
        "zk381t-{}-{}",
        std::process::id(),
        SIRA.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&kok).unwrap();
    let ec = [
        "-newkey",
        "ec",
        "-pkeyopt",
        "ec_paramgen_curve:prime256v1",
        "-nodes",
    ];
    let mut ca = vec!["req", "-x509"];
    ca.extend(ec);
    ca.extend([
        "-keyout",
        "ca.key",
        "-out",
        "ca.pem",
        "-days",
        "2",
        "-subj",
        "/CN=Zee K381 Test CA",
        "-addext",
        "basicConstraints=critical,CA:TRUE",
        "-addext",
        "keyUsage=critical,keyCertSign,cRLSign",
        "-addext",
        "subjectKeyIdentifier=hash",
    ]);
    openssl(&kok, &ca);
    let mut istek = vec!["req"];
    istek.extend(ec);
    istek.extend([
        "-keyout",
        "anahtar.pem",
        "-out",
        "yaprak.csr",
        "-subj",
        "/CN=localhost",
    ]);
    openssl(&kok, &istek);
    std::fs::write(
        kok.join("ek.cnf"),
        "basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature\nextendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost,IP:127.0.0.1\nsubjectKeyIdentifier=hash\nauthorityKeyIdentifier=keyid,issuer\n",
    )
    .unwrap();
    openssl(
        &kok,
        &[
            "x509",
            "-req",
            "-in",
            "yaprak.csr",
            "-CA",
            "ca.pem",
            "-CAkey",
            "ca.key",
            "-CAcreateserial",
            "-out",
            "yaprak.pem",
            "-days",
            "2",
            "-extfile",
            "ek.cnf",
        ],
    );
    let ca = std::fs::read(kok.join("ca.pem")).unwrap();
    let mut zincir = std::fs::read_to_string(kok.join("yaprak.pem")).unwrap();
    zincir.push_str(&String::from_utf8_lossy(&ca));
    let anahtar = std::fs::read_to_string(kok.join("anahtar.pem")).unwrap();
    Kimlik {
        kok,
        ca,
        zincir,
        anahtar,
    }
}

struct Sunucu {
    cocuk: Option<Child>,
    adres: SocketAddr,
    kok: PathBuf,
}

impl Drop for Sunucu {
    fn drop(&mut self) {
        if let Some(mut cocuk) = self.cocuk.take() {
            // SAFETY: SIGKILL to the test-owned process group.
            unsafe { libc::kill(-(cocuk.id() as libc::pid_t), libc::SIGKILL) };
            let _ = cocuk.wait();
        }
        let _ = std::fs::remove_dir_all(&self.kok);
    }
}

/// `implicit`: dinleyici `tls_dinleyicileri` içinde. `ortam`: TLS değişkenleri.
fn baslat(implicit: bool, ortam: &[(&str, &str)], ek_proje: &str) -> Sunucu {
    use std::os::unix::process::CommandExt;
    let dinleyici = TcpListener::bind("127.0.0.1:0").unwrap();
    let adres = dinleyici.local_addr().unwrap();
    let kok = PathBuf::from("/tmp").join(format!(
        "zk381s-{}-{}",
        std::process::id(),
        SIRA.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&kok).unwrap();
    let tls_liste = if implicit {
        "tls_dinleyicileri \"posta\" listesi olsun\n"
    } else {
        ""
    };
    std::fs::write(
        kok.join("proje.dil"),
        format!(
            "proje \"tls\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ-dinleyici\" listesi olsun\nağ_hedefleri boş liste olsun\ndinleyiciler \"posta {adres}\" listesi olsun\nbağlantı_boşta_süresi 5 olsun\ntls_sertifika_değişkeni \"ZK381_ZINCIR\" olsun\ntls_anahtar_değişkeni \"ZK381_ANAHTAR\" olsun\n{tls_liste}{ek_proje}"
        ),
    )
    .unwrap();
    std::fs::write(kok.join("ana.dil"), SUNUCU).unwrap();
    let mut komut = Command::new("/bin/sh");
    komut
        .arg("-c")
        .arg("LISTEN_PID=$$ LISTEN_FDS=1 LISTEN_FDNAMES=posta exec \"$0\" \"$@\"")
        .arg(env!("CARGO_BIN_EXE_dil"))
        .arg("çalıştır")
        .arg(&kok)
        .current_dir(&kok)
        .env_remove("NOTIFY_SOCKET")
        .env_remove("CREDENTIALS_DIRECTORY")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .process_group(0);
    for (ad, deger) in ortam {
        komut.env(ad, deger);
    }
    let fd = dinleyici.as_raw_fd();
    // SAFETY: async-signal-safe fcntl/dup2 in the forked child before exec.
    unsafe {
        komut.pre_exec(move || {
            let yuksek = libc::fcntl(fd, libc::F_DUPFD, 200);
            if yuksek < 0 || libc::dup2(yuksek, 3) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let cocuk = komut.spawn().unwrap();
    drop(dinleyici);
    Sunucu {
        cocuk: Some(cocuk),
        adres,
        kok,
    }
}

fn istemci_yapilandirmasi(
    ca: &[u8],
    surumler: &[&'static rustls::SupportedProtocolVersion],
) -> Arc<rustls::ClientConfig> {
    use rustls::pki_types::pem::PemObject;
    let mut kokler = rustls::RootCertStore::empty();
    for sertifika in rustls::pki_types::CertificateDer::pem_slice_iter(ca) {
        kokler.add(sertifika.unwrap()).unwrap();
    }
    Arc::new(
        rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_protocol_versions(surumler)
        .unwrap()
        .with_root_certificates(kokler)
        .with_no_client_auth(),
    )
}

type TlsAkis = rustls::StreamOwned<rustls::ClientConnection, TcpStream>;

fn tls_baglan(
    akis: TcpStream,
    ca: &[u8],
    surumler: &[&'static rustls::SupportedProtocolVersion],
) -> TlsAkis {
    let ad = rustls::pki_types::ServerName::try_from("localhost").unwrap();
    let oturum = rustls::ClientConnection::new(istemci_yapilandirmasi(ca, surumler), ad).unwrap();
    rustls::StreamOwned::new(oturum, akis)
}

fn satir<R: BufRead>(okuyucu: &mut R) -> String {
    let mut satir = String::new();
    okuyucu.read_line(&mut satir).expect("satır");
    satir.trim_end_matches("\r\n").to_string()
}

fn baglan(sunucu: &Sunucu) -> TcpStream {
    let akis = TcpStream::connect_timeout(&sunucu.adres, Duration::from_secs(30)).unwrap();
    akis.set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    akis
}

fn tls_ortami(kimlik: &Kimlik) -> Vec<(&'static str, String)> {
    vec![
        ("ZK381_ZINCIR", kimlik.zincir.clone()),
        ("ZK381_ANAHTAR", kimlik.anahtar.clone()),
    ]
}

fn ortam_ref<'a>(ortam: &'a [(&'static str, String)]) -> Vec<(&'static str, &'a str)> {
    ortam.iter().map(|(a, d)| (*a, d.as_str())).collect()
}

#[test]
fn implicit_tls_dinleyicisi_el_sikismadan_sonra_gorev_kurar() {
    let kimlik = kimlik_uret();
    let ortam = tls_ortami(&kimlik);
    let sunucu = baslat(true, &ortam_ref(&ortam), "");
    let mut tls = BufReader::new(tls_baglan(
        baglan(&sunucu),
        &kimlik.ca,
        rustls::ALL_VERSIONS,
    ));
    assert_eq!(satir(&mut tls), "220 zee.test ESMTP tls=evet");
    tls.get_mut().write_all(b"EHLO istemci\r\n").unwrap();
    assert_eq!(satir(&mut tls), "250 EHLO istemci");
    let surum = tls.get_ref().conn.protocol_version();
    assert_eq!(surum, Some(rustls::ProtocolVersion::TLSv1_3));
    tls.get_mut().write_all(b"QUIT\r\n").unwrap();
    assert_eq!(satir(&mut tls), "221 hoşça kal");
}

#[test]
fn starttls_yukseltir_tls12_aead_ile_de_calisir() {
    let kimlik = kimlik_uret();
    let ortam = tls_ortami(&kimlik);
    let sunucu = baslat(false, &ortam_ref(&ortam), "");
    for surumler in [
        &[&rustls::version::TLS13][..],
        &[&rustls::version::TLS12][..],
    ] {
        let akis = baglan(&sunucu);
        let mut duz = BufReader::new(akis.try_clone().unwrap());
        assert_eq!(satir(&mut duz), "220 zee.test ESMTP tls=hayır");
        (&akis).write_all(b"STARTTLS\r\n").unwrap();
        assert_eq!(satir(&mut duz), "220 2.0.0 hazır");
        let mut tls = BufReader::new(tls_baglan(akis, &kimlik.ca, surumler));
        let bilgi = satir(&mut tls);
        if surumler[0].version == rustls::ProtocolVersion::TLSv1_2 {
            assert!(bilgi.starts_with("250 tls TLSv1_2 TLS_ECDHE_"), "{bilgi}");
            assert!(
                bilgi.contains("GCM") || bilgi.contains("CHACHA20"),
                "{bilgi}"
            );
        } else {
            assert!(bilgi.starts_with("250 tls TLSv1_3 TLS13_"), "{bilgi}");
        }
        tls.get_mut().write_all(b"EHLO sifreli\r\n").unwrap();
        assert_eq!(satir(&mut tls), "250 EHLO sifreli");
        // İkinci yükseltme reddedilir: 454 tls ve kapanış.
        tls.get_mut().write_all(b"STARTTLS\r\n").unwrap();
        assert_eq!(satir(&mut tls), "220 2.0.0 hazır");
        assert_eq!(satir(&mut tls), "454 tls");
    }
}

/// CVE-2011-0411 sınıfı: STARTTLS ile aynı pakette gönderilen düz metin komut
/// TLS sonrasına taşınmaz; yükseltme yapılmaz, bağlantı kapanır.
#[test]
fn starttls_ara_bellek_enjeksiyonu_reddedilir() {
    let kimlik = kimlik_uret();
    let ortam = tls_ortami(&kimlik);
    let sunucu = baslat(false, &ortam_ref(&ortam), "");
    let akis = baglan(&sunucu);
    let mut duz = BufReader::new(akis.try_clone().unwrap());
    assert_eq!(satir(&mut duz), "220 zee.test ESMTP tls=hayır");
    (&akis)
        .write_all(b"STARTTLS\r\nRCPT TO:<kurban@ornek.test>\r\n")
        .unwrap();
    assert_eq!(satir(&mut duz), "220 2.0.0 hazır");
    let mut kalan = Vec::new();
    let _ = duz.read_to_end(&mut kalan);
    assert!(kalan.is_empty(), "enjeksiyon sonrası veri: {kalan:?}");
    // Süreç ve kardeş bağlantılar etkilenmez.
    let mut yeni = BufReader::new(baglan(&sunucu));
    assert_eq!(satir(&mut yeni), "220 zee.test ESMTP tls=hayır");
}

/// Sunucu TLS 1.0/1.1'i ve AEAD olmayan (CBC) takımları kabul etmez; Zee
/// görevi kurulmaz (karşılama satırı gelmez).
#[test]
fn tls_politikasi_eski_surum_ve_cbc_takimini_reddeder() {
    let kimlik = kimlik_uret();
    let ortam = tls_ortami(&kimlik);
    let sunucu = baslat(true, &ortam_ref(&ortam), "");
    for (surum, takim) in [
        ([3u8, 2u8], [0x00u8, 0x2f]),
        ([3, 3], [0x00, 0x2f]),
        ([3, 3], [0xc0, 0x13]),
    ] {
        let mut govde = vec![surum[0], surum[1]];
        govde.extend([7u8; 32]);
        govde.push(0); // oturum kimliği yok
        govde.extend([0, 2, takim[0], takim[1]]);
        govde.extend([1, 0]); // sıkıştırma: null
        let mut el = vec![1, 0, 0, govde.len() as u8];
        el.extend(&govde);
        let mut kayit = vec![0x16, 3, 1, 0, el.len() as u8];
        kayit.extend(&el);
        let mut akis = baglan(&sunucu);
        akis.write_all(&kayit).unwrap();
        let mut yanit = Vec::new();
        let _ = akis.read_to_end(&mut yanit);
        assert!(!yanit.is_empty(), "sunucu uyarı göndermeli");
        assert_eq!(
            yanit[0], 0x15,
            "{surum:?}/{takim:?}: TLS uyarısı bekleniyordu: {yanit:?}"
        );
        assert!(!String::from_utf8_lossy(&yanit).contains("220"));
    }
    // Politika içindeki istemci geçer (sunucu hâlâ hizmette).
    let mut tls = BufReader::new(tls_baglan(
        baglan(&sunucu),
        &kimlik.ca,
        rustls::ALL_VERSIONS,
    ));
    assert_eq!(satir(&mut tls), "220 zee.test ESMTP tls=evet");
}

/// Bozuk anahtar/zincir ve eksik kimlik yürütmeden önce çıkış 2; anahtar
/// içeriği hiçbir iletide görünmez. Credential dizini K-339 kuralıyla okunur.
#[test]
fn tls_kimligi_dogrulanir_anahtar_sizmaz_credential_dizini_okunur() {
    let kimlik = kimlik_uret();
    let bozuk = "-----BEGIN PRIVATE KEY-----\nGIZLIGIZLIGIZLI\n-----END PRIVATE KEY-----\n";
    for (zincir, anahtar) in [
        (kimlik.zincir.as_str(), bozuk),
        ("bozuk zincir", kimlik.anahtar.as_str()),
    ] {
        let mut sunucu = baslat(
            true,
            &[("ZK381_ZINCIR", zincir), ("ZK381_ANAHTAR", anahtar)],
            "",
        );
        let cikti = sunucu.cocuk.take().unwrap().wait_with_output().unwrap();
        let stderr = String::from_utf8_lossy(&cikti.stderr);
        assert_eq!(cikti.status.code(), Some(2), "{stderr}");
        assert!(stderr.contains("bağlantı TLS'i"), "{stderr}");
        assert!(
            !stderr.contains("GIZLI") && !stderr.contains("BEGIN"),
            "{stderr}"
        );
    }
    let mut sunucu = baslat(true, &[], "");
    let cikti = sunucu.cocuk.take().unwrap().wait_with_output().unwrap();
    assert_eq!(cikti.status.code(), Some(2));

    // CREDENTIALS_DIRECTORY: 0600 dosyalar okunur; grup/diğer okuyabiliyorsa ret.
    use std::os::unix::fs::PermissionsExt;
    let dizin = kimlik.kok.join("kimlikler");
    std::fs::create_dir_all(&dizin).unwrap();
    std::fs::set_permissions(&dizin, std::fs::Permissions::from_mode(0o700)).unwrap();
    for (ad, icerik) in [
        ("ZK381_ZINCIR", &kimlik.zincir),
        ("ZK381_ANAHTAR", &kimlik.anahtar),
    ] {
        std::fs::write(dizin.join(ad), icerik).unwrap();
        std::fs::set_permissions(dizin.join(ad), std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    // K-339: dizin bileşenleri sembolik bağ izlenmeden açılır (macOS /tmp → /private/tmp).
    let yol = dizin.canonicalize().unwrap().to_str().unwrap().to_string();
    let sunucu = baslat(true, &[("CREDENTIALS_DIRECTORY", &yol)], "");
    let mut tls = BufReader::new(tls_baglan(
        baglan(&sunucu),
        &kimlik.ca,
        rustls::ALL_VERSIONS,
    ));
    assert_eq!(satir(&mut tls), "220 zee.test ESMTP tls=evet");
    drop(tls);
    drop(sunucu);
    std::fs::set_permissions(
        dizin.join("ZK381_ANAHTAR"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    let mut sunucu = baslat(true, &[("CREDENTIALS_DIRECTORY", &yol)], "");
    let cikti = sunucu.cocuk.take().unwrap().wait_with_output().unwrap();
    assert_eq!(cikti.status.code(), Some(2));
    assert!(!String::from_utf8_lossy(&cikti.stderr).contains("BEGIN"));
}
