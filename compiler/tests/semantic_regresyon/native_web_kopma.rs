//! K-327 (W1): yanıt yazımı ortasında RST ile kopan istemcinin gerçek native
//! `dil` worker'ında istek kapsamlı kaldığını sınayan sahip olunan soket
//! yardımcıları. Korpus kipi `native-web-kopma` ve `web_cok_surec_testi`
//! aynı yardımcıyı kullanır; ToplayanIo socket kopmasını taklit edemez.
#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub const PROXY: &str = "https://panel.example";

pub fn bos_kapi() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .expect("boş kapı")
        .local_addr()
        .expect("yerel adres")
        .port()
}

pub fn worker_baslat(kaynak: &Path, kapi: u16, stderr: Stdio) -> Child {
    Command::new(env!("CARGO_BIN_EXE_dil"))
        .args([
            "çalıştır",
            "--web-proxy",
            PROXY,
            "--web-worker-port",
            &kapi.to_string(),
            kaynak.to_str().expect("UTF-8 kaynak yolu"),
        ])
        .stdout(Stdio::null())
        .stderr(stderr)
        .spawn()
        .expect("web worker süreci")
}

fn ham_istek(yontem_hedef: &str, kimlik: &str) -> String {
    format!(
        "{yontem_hedef} HTTP/1.1\r\nHost: panel.example\r\nForwarded: for={kimlik};proto=https;host=panel.example\r\nConnection: close\r\n\r\n"
    )
}

/// Tam istek gönderir; bağlantı kurulamazsa ya da yanıt okunamazsa `None`.
pub fn tam_istek(kapi: u16, yontem_hedef: &str, kimlik: &str) -> Option<String> {
    let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
    let mut akis = TcpStream::connect_timeout(&adres, Duration::from_secs(2)).ok()?;
    akis.set_read_timeout(Some(Duration::from_secs(10))).ok()?;
    akis.write_all(ham_istek(yontem_hedef, kimlik).as_bytes())
        .ok()?;
    let mut yanit = Vec::new();
    akis.read_to_end(&mut yanit).ok()?;
    String::from_utf8(yanit).ok()
}

/// Worker dinlemeye geçene dek bekler.
pub fn dinlemeyi_bekle(kapi: u16) {
    let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
    let son = Instant::now() + Duration::from_secs(10);
    while TcpStream::connect_timeout(&adres, Duration::from_millis(100)).is_err() {
        assert!(Instant::now() < son, "web worker {kapi} dinlemeye geçmedi");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Küçük alma tamponlu (SO_RCVBUF) bağlantı: büyük yanıt çekirdek
/// tamponlarına sığmaz, worker yazımın ortasında kalır.
#[cfg(unix)]
fn kucuk_tamponlu_baglan(kapi: u16) -> TcpStream {
    use std::os::fd::FromRawFd;
    // SAFETY: yeni soket tanımlayıcısı yalnız bu blokta sahiplenilir; bağlantı
    // başarılıysa sahiplik TcpStream'e devredilir, değilse kapatılır.
    unsafe {
        let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0);
        assert!(fd >= 0, "soket açılmalı");
        let tampon: libc::c_int = 4096;
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            &tampon as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        );
        let mut adres: libc::sockaddr_in = std::mem::zeroed();
        adres.sin_family = libc::AF_INET as libc::sa_family_t;
        adres.sin_port = kapi.to_be();
        adres.sin_addr.s_addr = u32::from_ne_bytes([127, 0, 0, 1]);
        let sonuc = libc::connect(
            fd,
            &adres as *const libc::sockaddr_in as *const libc::sockaddr,
            std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
        );
        if sonuc != 0 {
            libc::close(fd);
            panic!("web worker {kapi} bağlantıyı kabul etmedi");
        }
        TcpStream::from_raw_fd(fd)
    }
}

/// İsteği gönderir, yanıtın ilk baytlarını okur ve bağlantıyı RST ile keser
/// (SO_LINGER 0); worker yanıtı yazarken ECONNRESET/EPIPE alır. Okunan ilk
/// baytları döndürür.
#[cfg(unix)]
pub fn yazim_ortasinda_kopar(kapi: u16, yontem_hedef: &str, kimlik: &str) -> Vec<u8> {
    use std::os::fd::AsRawFd;
    let mut akis = kucuk_tamponlu_baglan(kapi);
    akis.set_read_timeout(Some(Duration::from_secs(10)))
        .expect("okuma timeout");
    akis.write_all(ham_istek(yontem_hedef, kimlik).as_bytes())
        .expect("HTTP isteği");
    let mut ilk = [0u8; 1024];
    let okunan = akis.read(&mut ilk).expect("yanıtın ilk baytları");
    let linger = libc::linger {
        l_onoff: 1,
        l_linger: 0,
    };
    // SAFETY: geçerli, sahiplenilmiş soket üzerinde sabit boyutlu seçenek.
    let sonuc = unsafe {
        libc::setsockopt(
            akis.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_LINGER,
            &linger as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::linger>() as libc::socklen_t,
        )
    };
    assert_eq!(sonuc, 0, "SO_LINGER 0 ayarlanmalı");
    drop(akis);
    ilk[..okunan].to_vec()
}

fn baslik_degeri<'a>(kaynak: &'a str, anahtar: &str) -> &'a str {
    kaynak
        .lines()
        .find_map(|satir| satir.strip_prefix(anahtar))
        .unwrap_or_else(|| panic!("native-web-kopma fixture `{anahtar}` başlığı taşımıyor"))
}

/// Korpus kipi: `# web-istegi:` yanıtı yazılırken RST, ardından
/// `# web-sonraki:` isteği. Exit, sonraki istekten sonra worker hâlâ
/// çalışıyorsa 0, değilse sürecin çıkış kodudur; çıktı sonraki isteğin
/// `durum gövde` satırıdır (worker düştüyse boş).
pub fn calistir(kaynak_yolu: &Path) -> (i64, Vec<String>) {
    #[cfg(not(unix))]
    {
        let _ = kaynak_yolu;
        panic!("actual Unix owned socket capability unavailable; no synthetic pass");
    }
    #[cfg(unix)]
    {
        let kaynak = std::fs::read_to_string(kaynak_yolu).expect("fixture kaynağı");
        let kopan = baslik_degeri(&kaynak, "# web-istegi: ");
        let sonraki = baslik_degeri(&kaynak, "# web-sonraki: ");
        let kapi = bos_kapi();
        let mut worker = worker_baslat(kaynak_yolu, kapi, Stdio::null());
        dinlemeyi_bekle(kapi);
        let ilk = yazim_ortasinda_kopar(kapi, kopan, "198.51.100.1");
        assert!(
            ilk.starts_with(b"HTTP/1.1 200 OK"),
            "kopan istek yazıma başlamış olmalı: {}",
            String::from_utf8_lossy(&ilk)
        );
        let yanit = tam_istek(kapi, sonraki, "198.51.100.2");
        // Yanıt alınamadıysa worker çıkıyor olabilir; çıkış kodunu sınırlı bekle.
        let son = Instant::now() + Duration::from_secs(if yanit.is_some() { 0 } else { 5 });
        let cikis = loop {
            match worker.try_wait().expect("worker durumu") {
                Some(durum) => break i64::from(durum.code().unwrap_or(-1)),
                None if Instant::now() < son => std::thread::sleep(Duration::from_millis(20)),
                None => break 0,
            }
        };
        let _ = worker.kill();
        let _ = worker.wait();
        let cikti = yanit
            .map(|yanit| {
                let durum = yanit.split_whitespace().nth(1).unwrap_or("").to_string();
                let govde = yanit.split("\r\n\r\n").nth(1).unwrap_or("");
                vec![format!("{durum} {govde}")]
            })
            .unwrap_or_default();
        (cikis, cikti)
    }
}
