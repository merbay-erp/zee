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

/// Süreç içinde benzersiz boş kapı. Seçilen kapı bırakılıp worker sonra
/// bağladığından, aynı test ikilisinde paralel koşan iki test aynı kapıyı
/// alamamalıdır; aksi hâlde biri bağlayamaz ve istekleri ötekinin worker'ına
/// gider (Windows CI'da 401/"dinlemeye geçmedi").
///
/// Kapı işletim sisteminin geçici (ephemeral) aralığından alınmaz: Linux
/// 32768–60999, macOS/Windows 49152–65535. O aralık her giden bağlantının
/// yerel kapısıdır; sağlık yoklaması gibi binlerce kısa bağlantı açan paralel
/// test ikilileri bırakılan kapıyı bu arada yerel kapı olarak alır ve sunucu
/// "Address already in use" ile açılamaz ya da istek başka sürece gider.
/// 20000–32767 arasında süreç başına rastgele başlangıçla taranır.
///
/// Paralel koşan test ikilileri aynı kapıyı "boş" görüp seçebilir (her biri
/// sınayıp bırakır, sunucu sonra bağlar). Bu yüzden seçilen kapı, geçici
/// klasördeki `zee-test-kapilari/<kapı>.kilit` dosyasının dışlayıcı kilidiyle
/// süreç ömrü boyunca ayrılır; başka ikili kilitli kapıyı atlar, kilidi
/// çekirdek süreç bitince bırakır.
pub fn bos_kapi() -> u16 {
    const ILK: u32 = 20_000;
    const ARALIK: u32 = 12_768;
    static VERILEN: std::sync::Mutex<Vec<u16>> = std::sync::Mutex::new(Vec::new());
    static AYRILAN: std::sync::Mutex<Vec<std::fs::File>> = std::sync::Mutex::new(Vec::new());
    let mut verilen = VERILEN.lock().unwrap_or_else(|e| e.into_inner());
    let kilitler = std::env::temp_dir().join("zee-test-kapilari");
    std::fs::create_dir_all(&kilitler).expect("kapı kilit klasörü");
    let tohum = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|s| s.subsec_nanos())
        .unwrap_or(0)
        ^ std::process::id().wrapping_mul(2_654_435_761);
    for adim in 0..ARALIK {
        let kapi = (ILK + (tohum.wrapping_add(adim.wrapping_mul(7_919))) % ARALIK) as u16;
        if verilen.contains(&kapi) {
            continue;
        }
        let Ok(kilit) = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(kilitler.join(format!("{kapi}.kilit")))
        else {
            continue;
        };
        if kilit.try_lock().is_err() || TcpListener::bind(("127.0.0.1", kapi)).is_err() {
            continue;
        }
        AYRILAN
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(kilit);
        verilen.push(kapi);
        return kapi;
    }
    panic!("20000–32767 aralığında boş kapı yok");
}

/// `komut`u stdout borusuyla başlatır ve sürecin kendi "Sunucu dinliyor"
/// satırını bekler: kapıyı gerçekten bu süreç bağlamıştır. Süreç satırdan
/// önce çıkarsa (kapı geçici olarak meşgul: TIME_WAIT, istemci geçici kapısı
/// ya da yeniden başlatılan worker) aynı komut sınırlı kez yeniden denenir.
pub fn dinleyene_dek_baslat(mut komut: impl FnMut() -> Command) -> Child {
    let son = Instant::now() + Duration::from_secs(30);
    let mut son_durum = String::new();
    while Instant::now() < son {
        let mut cocuk = komut()
            .stdout(Stdio::piped())
            .spawn()
            .expect("web worker süreci");
        let cikti = cocuk.stdout.take().expect("worker stdout borusu");
        let (gonder, al) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            use std::io::BufRead;
            let mut satirlar = std::io::BufReader::new(cikti).lines();
            let mut bildirildi = false;
            for satir in satirlar.by_ref().map_while(Result::ok) {
                if !bildirildi && satir.starts_with("Sunucu dinliyor") {
                    bildirildi = true;
                    let _ = gonder.send(());
                }
            }
        });
        match al.recv_timeout(son.saturating_duration_since(Instant::now())) {
            Ok(()) => return cocuk,
            Err(_) => {
                let _ = cocuk.kill();
                son_durum = cocuk
                    .wait()
                    .map(|d| d.to_string())
                    .unwrap_or_else(|e| e.to_string());
                std::thread::sleep(Duration::from_millis(200));
            }
        }
    }
    panic!("web worker 30 sn içinde dinlemeye geçmedi; son durum: {son_durum}");
}

pub fn worker_baslat(kaynak: &Path, kapi: u16, stderr: impl Fn() -> Stdio) -> Child {
    dinleyene_dek_baslat(|| {
        let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
        komut
            .args([
                "çalıştır",
                "--web-proxy",
                PROXY,
                "--web-worker-port",
                &kapi.to_string(),
                kaynak.to_str().expect("UTF-8 kaynak yolu"),
            ])
            .stderr(stderr());
        komut
    })
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
    // Runtime yanıt yazımına 10 sn socket zaman aşımı tanır (spec/12 §5);
    // kopan bir önceki yanıt en kötü bu süre kadar sürebilir. İstemci bütçesi
    // bu sözleşmeye eşit olursa yarış olur; açıkça üstünde tutulur.
    akis.set_read_timeout(Some(Duration::from_secs(25))).ok()?;
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
///
/// K-369: tanımlayıcı exec'te kapanır (CLOEXEC). Test ikilisi paralel
/// testlerde sürekli süreç başlatır; miras kalabilen soket o çocuklara
/// sızıyor, `drop` gerçek kapanış olmadığından RST gitmiyor ve worker yazımda
/// socket zaman aşımını bekliyordu (tam yükte sonraki istek 25 sn yanıtsız).
/// Linux'ta bayrak atomiktir; diğer Unix'lerde std soketleri gibi oluşturma
/// anında kurulur.
#[cfg(unix)]
fn kucuk_tamponlu_baglan(kapi: u16) -> TcpStream {
    use std::os::fd::FromRawFd;
    // SAFETY: yeni soket tanımlayıcısı yalnız bu blokta sahiplenilir; bağlantı
    // başarılıysa sahiplik TcpStream'e devredilir, değilse kapatılır.
    unsafe {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0);
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        let fd = {
            let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0);
            if fd >= 0 && libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) != 0 {
                libc::close(fd);
                panic!("soket CLOEXEC kurulamadı");
            }
            fd
        };
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
    // K-369: kopuşun RST'si yalnız tanımlayıcı hiçbir çocuğa miras kalmadıysa
    // `drop` anında gider; bu değişmez bozulursa test zamanlamaya bırakılmaz.
    // SAFETY: geçerli, sahiplenilmiş soket üzerinde salt okuma fcntl.
    let bayraklar = unsafe { libc::fcntl(akis.as_raw_fd(), libc::F_GETFD) };
    assert!(
        bayraklar >= 0 && bayraklar & libc::FD_CLOEXEC != 0,
        "kopan istemci soketi exec'te kapanmalı (FD_CLOEXEC)"
    );
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
        let mut worker = worker_baslat(kaynak_yolu, kapi, Stdio::piped);
        dinlemeyi_bekle(kapi);
        let ilk = yazim_ortasinda_kopar(kapi, kopan, "198.51.100.1");
        assert!(
            ilk.starts_with(b"HTTP/1.1 200 OK"),
            "kopan istek yazıma başlamış olmalı: {}",
            String::from_utf8_lossy(&ilk)
        );
        let baslangic = Instant::now();
        let yanit = tam_istek(kapi, sonraki, "198.51.100.2");
        let sure = baslangic.elapsed();
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
        if yanit.is_none() {
            // Tanı: CI'da gözlenen yanıtsızlığın nedeni stderr ve süreyle görünür.
            let mut hata = String::new();
            if let Some(mut boru) = worker.stderr.take() {
                let _ = boru.read_to_string(&mut hata);
            }
            eprintln!(
                "native-web-kopma: sonraki istek {sure:?} sonra yanıtsız; worker stderr:\n{hata}"
            );
        }
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

/// K-372 korpus kipi `native-web-yazim-butcesi`: `# web-istegi:` yanıtını
/// sürekli ama yavaş okuyan (4 KiB / 100 ms ≈ 40 KiB/s; her pencerede
/// ilerler, syscall zaman aşımına düşmez) bir istemci bağlanır, ardından
/// `# web-sonraki:` isteği gönderilir. Tek işçi yavaş okuyucuyu en çok
/// mutlak yanıt yazım bütçesi kadar (2 MiB yanıt için 10 + 16 sn) tutabilir;
/// kesilirse neden bütçe ya da 10 sn ilerlemesizliktir.
/// Çıktı: sonraki isteğin `durum gövde` satırı ve işçinin bütçe içinde
/// serbest kalıp kalmadığı. Düzeltme öncesi işçi yanıtın tamamı aktarılana
/// dek (Linux ~52 sn; Darwin ~40 sn) tutulur.
pub fn butce_calistir(kaynak_yolu: &Path) -> (i64, Vec<String>) {
    #[cfg(not(unix))]
    {
        let _ = kaynak_yolu;
        panic!("actual Unix owned socket capability unavailable; no synthetic pass");
    }
    #[cfg(unix)]
    {
        let kaynak = std::fs::read_to_string(kaynak_yolu).expect("fixture kaynağı");
        let yavas = baslik_degeri(&kaynak, "# web-istegi: ");
        let sonraki = baslik_degeri(&kaynak, "# web-sonraki: ");
        let kapi = bos_kapi();
        let mut worker = worker_baslat(kaynak_yolu, kapi, Stdio::piped);
        dinlemeyi_bekle(kapi);
        let mut akis = kucuk_tamponlu_baglan(kapi);
        akis.write_all(ham_istek(yavas, "198.51.100.3").as_bytes())
            .expect("HTTP isteği");
        akis.set_read_timeout(Some(Duration::from_secs(15)))
            .expect("okuma timeout");
        // Üst sınır bütçe formülünden: başlık payıyla yanıt baytı, kaynak
        // sınırlarındaki taban ve alt hız; CI yavaşlığı için 8 sn pay.
        let mut ilk = [0u8; 4096];
        let ilk_okunan = akis.read(&mut ilk).expect("yanıtın ilk baytları");
        let uzunluk: u64 = String::from_utf8_lossy(&ilk[..ilk_okunan])
            .lines()
            .find_map(|satir| satir.strip_prefix("Content-Length: "))
            .and_then(|deger| deger.trim().parse().ok())
            .expect("yanıt Content-Length taşımalı");
        let http = dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.http();
        let butce = Duration::from_secs(http.istek_okuma_saniyesi())
            + Duration::from_secs_f64((uzunluk + 1024) as f64 / http.yanit_yazim_alt_hizi() as f64);
        let sinir = butce + Duration::from_secs(8);
        let dur = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let durdur = dur.clone();
        let okuyucu = std::thread::spawn(move || {
            let mut toplam = ilk_okunan;
            let mut parca = [0u8; 4096];
            while !durdur.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(100));
                match akis.read(&mut parca) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => toplam += n,
                }
            }
            toplam
        });
        std::thread::sleep(Duration::from_millis(500));
        let baslangic = Instant::now();
        let yanit = (|| {
            let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
            let mut akis = TcpStream::connect_timeout(&adres, Duration::from_secs(2)).ok()?;
            akis.set_read_timeout(Some(Duration::from_secs(90))).ok()?;
            akis.write_all(ham_istek(sonraki, "198.51.100.4").as_bytes())
                .ok()?;
            let mut yanit = Vec::new();
            akis.read_to_end(&mut yanit).ok()?;
            String::from_utf8(yanit).ok()
        })();
        let sure = baslangic.elapsed();
        dur.store(true, std::sync::atomic::Ordering::Relaxed);
        let okunan = okuyucu.join().expect("yavaş okuyucu");
        let cikis = match worker.try_wait().expect("worker durumu") {
            Some(durum) => i64::from(durum.code().unwrap_or(-1)),
            None => 0,
        };
        let _ = worker.kill();
        let _ = worker.wait();
        let mut hata = String::new();
        if let Some(mut boru) = worker.stderr.take() {
            let _ = boru.read_to_string(&mut hata);
        }
        eprintln!(
            "native-web-yazim-butcesi: sonraki istek {sure:?} (sınır {sinir:?}), yavaş okuyucu {okunan} bayt; worker stderr:\n{hata}"
        );
        let mut cikti = yanit
            .map(|yanit| {
                let durum = yanit.split_whitespace().nth(1).unwrap_or("").to_string();
                let govde = yanit.split("\r\n\r\n").nth(1).unwrap_or("");
                vec![format!("{durum} {govde}")]
            })
            .unwrap_or_default();
        // Yeni kod işçiyi ya mutlak bütçe ya 10 sn ilerlemesizlik kesmesiyle
        // bırakır (macOS'ta küçük alma penceresi TCP'yi duraklatabilir) ya da
        // çekirdek kalan yanıtı tamponlar ve yazım biter (Linux). Eski kodun
        // kopma nedeni socket zaman aşımıdır (`os error 35/11`).
        let kopma_nedeni_yeni = hata
            .lines()
            .filter(|satir| satir.contains("yanıt sırasında koptu"))
            .all(|satir| {
                satir.contains("yanıt yazım bütçesi doldu") || satir.contains("sn ilerlemedi")
            });
        cikti.push(if sure <= sinir && kopma_nedeni_yeni {
            "bütçede serbest".to_string()
        } else {
            format!("işçi {} sn tutuldu", sure.as_secs())
        });
        (cikis, cikti)
    }
}

/// K-374 korpus kipi `native-web-erken-red`: istemci isteğin tamamını
/// (başlık + gövde) tek seferde gönderir, sonra yanıtı okur. `# web-govde:`
/// isteği gövde sınırını aşan ham gövde (413), `# web-baslik:` isteği başlık
/// sınırını aşan başlık (431) taşır. Ardından gövde gönderip hiç kapatmayan
/// ve bayt damlatan bir istemci 413 alır; `# web-sonraki:` isteği tek işçinin
/// bu istemciyi boşaltma bütçesinde bırakıp bırakmadığını gösterir. Düzeltme
/// öncesi sunucu okunmamış gövdeyle kapattığından çekirdek RST gönderir ve
/// istemci yanıtı göremez (`yanıt yok`).
pub fn erken_red_calistir(kaynak_yolu: &Path) -> (i64, Vec<String>) {
    #[cfg(not(unix))]
    {
        let _ = kaynak_yolu;
        panic!("actual Unix owned socket capability unavailable; no synthetic pass");
    }
    #[cfg(unix)]
    {
        const BOSALTMA_SINIRI: Duration = Duration::from_secs(6);
        let kaynak = std::fs::read_to_string(kaynak_yolu).expect("fixture kaynağı");
        let govdeli = baslik_degeri(&kaynak, "# web-govde: ");
        let baslikli = baslik_degeri(&kaynak, "# web-baslik: ");
        let sonraki = baslik_degeri(&kaynak, "# web-sonraki: ");
        let kapi = bos_kapi();
        let mut worker = worker_baslat(kaynak_yolu, kapi, Stdio::piped);
        dinlemeyi_bekle(kapi);
        let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
        let ozetle = |yanit: std::io::Result<Vec<u8>>| match yanit {
            Ok(yanit) if !yanit.is_empty() => {
                let yanit = String::from_utf8_lossy(&yanit).into_owned();
                let durum = yanit.split_whitespace().nth(1).unwrap_or("").to_string();
                let govde = yanit.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
                format!("{durum} {govde}")
            }
            Ok(_) => "yanıt yok: boş".to_string(),
            Err(hata) => format!("yanıt yok: {:?}", hata.kind()),
        };
        // Tüm isteği yazar (sunucu erken kapatırsa yazım hatası beklenir),
        // ardından yanıtı sonuna dek okur.
        let gonder_oku = |istek: Vec<u8>| -> std::io::Result<Vec<u8>> {
            let mut akis = TcpStream::connect_timeout(&adres, Duration::from_secs(2))?;
            akis.set_read_timeout(Some(Duration::from_secs(15)))?;
            let _ = akis.write_all(&istek);
            let mut yanit = Vec::new();
            akis.read_to_end(&mut yanit)?;
            Ok(yanit)
        };
        let govde = vec![b'x'; 70 * 1024];
        let mut istek = format!(
            "{govdeli} HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=198.51.100.5;proto=https;host=panel.example\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            govde.len()
        )
        .into_bytes();
        istek.extend_from_slice(&govde);
        let mut cikti = vec![ozetle(gonder_oku(istek))];
        let istek = format!(
            "{baslikli} HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=198.51.100.6;proto=https;host=panel.example\r\nX-Dolgu: {}\r\nConnection: close\r\n\r\n",
            "a".repeat(40 * 1024)
        );
        cikti.push(ozetle(gonder_oku(istek.into_bytes())));
        // Damlatan istemci: büyük Content-Length, sonra 50 ms'de bir bayt;
        // yanıtı okumaz, bağlantıyı kendisi kapatmaz.
        let dur = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let durdur = dur.clone();
        let mut damla = TcpStream::connect_timeout(&adres, Duration::from_secs(2)).expect("damla");
        damla
            .write_all(format!(
                "{govdeli} HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=198.51.100.7;proto=https;host=panel.example\r\nContent-Length: 900000\r\nConnection: close\r\n\r\n"
            ).as_bytes())
            .expect("damla başlığı");
        let damlatici = std::thread::spawn(move || {
            while !durdur.load(std::sync::atomic::Ordering::Relaxed) {
                if damla.write_all(b"x").is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });
        std::thread::sleep(Duration::from_millis(300));
        let baslangic = Instant::now();
        let yanit = tam_istek(kapi, sonraki, "198.51.100.8");
        let sure = baslangic.elapsed();
        dur.store(true, std::sync::atomic::Ordering::Relaxed);
        damlatici.join().expect("damlatıcı");
        cikti.push(match yanit {
            Some(yanit) => {
                let durum = yanit.split_whitespace().nth(1).unwrap_or("").to_string();
                let govde = yanit.split("\r\n\r\n").nth(1).unwrap_or("");
                format!("{durum} {govde}")
            }
            None => "sonraki yanıtsız".to_string(),
        });
        cikti.push(if sure <= BOSALTMA_SINIRI {
            "damlatan bütçede bırakıldı".to_string()
        } else {
            format!("işçi {} sn tutuldu", sure.as_secs())
        });
        let cikis = match worker.try_wait().expect("worker durumu") {
            Some(durum) => i64::from(durum.code().unwrap_or(-1)),
            None => 0,
        };
        let _ = worker.kill();
        let _ = worker.wait();
        let mut hata = String::new();
        if let Some(mut boru) = worker.stderr.take() {
            let _ = boru.read_to_string(&mut hata);
        }
        eprintln!("native-web-erken-red: sonraki istek {sure:?}; worker stderr:\n{hata}");
        (cikis, cikti)
    }
}
