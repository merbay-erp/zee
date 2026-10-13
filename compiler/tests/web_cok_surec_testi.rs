//! K-137: iki bağımsız `dil` sürecinin aynı kalıcı oturum/rate-limit
//! durumunu gerçekten paylaştığı uçtan uca güvenli-proxy kanıtı.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static GECICI_SIRA: AtomicU64 = AtomicU64::new(0);

struct GeciciKlasor(PathBuf);

impl GeciciKlasor {
    fn yeni() -> Self {
        let yol = std::env::temp_dir().join(format!(
            "zee-web-cok-surec-{}-{}",
            std::process::id(),
            GECICI_SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&yol).expect("geçici web proje klasörü");
        Self(yol)
    }
}

impl Drop for GeciciKlasor {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Sunucu(Child);

impl Drop for Sunucu {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn bos_kapi() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .expect("boş kapı")
        .local_addr()
        .expect("yerel adres")
        .port()
}

fn sunucuyu_baslat(kaynak: &Path, kapi: u16) -> Sunucu {
    sunucuyu_baslat_oranla(kaynak, kapi, false)
}

fn sunucuyu_baslat_oranla(kaynak: &Path, kapi: u16, bellek: bool) -> Sunucu {
    let kapi_yazisi = kapi.to_string();
    let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
    komut.args([
        "çalıştır",
        "--web-proxy",
        "https://panel.example",
        "--web-worker-port",
        &kapi_yazisi,
        kaynak.to_str().expect("UTF-8 kaynak yolu"),
    ]);
    if bellek {
        komut.args(["--web-oran-deposu", "bellek"]);
    }
    let cocuk = komut
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("web worker süreci");
    Sunucu(cocuk)
}

fn istek(
    kapi: u16,
    kimlik: &str,
    yontem: &str,
    yol: &str,
    cerez: Option<&str>,
    govde: &str,
) -> String {
    let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
    let son = Instant::now() + Duration::from_secs(10);
    let mut akis = loop {
        match TcpStream::connect_timeout(&adres, Duration::from_millis(100)) {
            Ok(akis) => break akis,
            Err(_) if Instant::now() < son => std::thread::sleep(Duration::from_millis(20)),
            Err(hata) => panic!("web worker {kapi} dinlemeye geçmedi: {hata}"),
        }
    };
    akis.set_read_timeout(Some(Duration::from_secs(10)))
        .expect("okuma timeout");
    let mut ham = format!(
        "{yontem} {yol} HTTP/1.1\r\nHost: panel.example\r\nForwarded: for={kimlik};proto=https;host=panel.example\r\nConnection: close\r\n"
    );
    if !matches!(yontem, "GET" | "HEAD" | "OPTIONS") {
        ham.push_str("Origin: https://panel.example\r\n");
    }
    if let Some(cerez) = cerez {
        ham.push_str(&format!("Cookie: __Host-zee-oturum={cerez}\r\n"));
    }
    if !govde.is_empty() {
        ham.push_str(&format!(
            "Content-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n",
            govde.len()
        ));
    }
    ham.push_str("\r\n");
    ham.push_str(govde);
    akis.write_all(ham.as_bytes()).expect("HTTP isteği");
    akis.shutdown(std::net::Shutdown::Write).ok();
    let mut yanit = String::new();
    akis.read_to_string(&mut yanit).expect("HTTP yanıtı");
    yanit
}

fn durum(yanit: &str) -> u16 {
    yanit
        .split_whitespace()
        .nth(1)
        .and_then(|deger| deger.parse().ok())
        .expect("HTTP durum kodu")
}

fn cerez(yanit: &str) -> String {
    yanit
        .lines()
        .find_map(|satir| satir.strip_prefix("Set-Cookie: __Host-zee-oturum="))
        .and_then(|kalan| kalan.split(';').next())
        .expect("oturum çerezi")
        .to_string()
}

fn csrf(yanit: &str) -> String {
    yanit
        .split("name=_csrf value=\"")
        .nth(1)
        .and_then(|kalan| kalan.split('"').next())
        .expect("CSRF alanı")
        .to_string()
}

#[test]
fn iki_worker_oturumu_iptali_restarti_ve_giris_limitini_paylasir() {
    iki_worker_kabul(false);
}

#[test]
fn bellek_oran_iki_worker_oturum_revoke_restart_paylasir_kota_yereldir() {
    iki_worker_kabul(true);
}

fn iki_worker_kabul(bellek: bool) {
    let gecici = GeciciKlasor::yeni();
    let a_kapi = bos_kapi();
    let b_kapi = bos_kapi();
    let kaynak = include_str!("../../projeler/girisli-panel.dil");
    let a_yolu = gecici.0.join("worker-a.dil");
    let b_yolu = gecici.0.join("worker-b.dil");
    std::fs::write(&a_yolu, kaynak).expect("A kaynağı");
    std::fs::write(&b_yolu, kaynak).expect("B kaynağı");

    let mut a = sunucuyu_baslat_oranla(&a_yolu, a_kapi, bellek);
    let b = sunucuyu_baslat_oranla(&b_yolu, b_kapi, bellek);
    let form = istek(a_kapi, "203.0.113.7", "GET", "/giris", None, "");
    assert_eq!(durum(&form), 200);
    let anonim = cerez(&form);
    let giris_csrf = csrf(&form);
    let giris = istek(
        a_kapi,
        "203.0.113.7",
        "POST",
        "/giris-yap",
        Some(&anonim),
        &format!("_csrf={giris_csrf}&parola=zee2026"),
    );
    assert_eq!(durum(&giris), 303);
    let oturum = cerez(&giris);

    let b_okuma = istek(b_kapi, "203.0.113.7", "GET", "/yonet", Some(&oturum), "");
    assert_eq!(durum(&b_okuma), 200, "B worker A oturumunu görmeli");
    let cikis_csrf = csrf(&b_okuma);

    a.0.kill().expect("A worker durdurulmalı");
    a.0.wait().expect("A worker sonucu");
    drop(a);
    let a = sunucuyu_baslat_oranla(&a_yolu, a_kapi, bellek);
    let restart_okuma = istek(a_kapi, "203.0.113.7", "GET", "/yonet", Some(&oturum), "");
    assert_eq!(durum(&restart_okuma), 200, "restart oturumu kaybetmemeli");

    let cikis = istek(
        b_kapi,
        "203.0.113.7",
        "POST",
        "/cikis",
        Some(&oturum),
        &format!("_csrf={cikis_csrf}"),
    );
    assert_eq!(durum(&cikis), 303);
    let iptal = istek(a_kapi, "203.0.113.7", "GET", "/yonet", Some(&oturum), "");
    assert_eq!(durum(&iptal), 401, "B logout A worker'da görünmeli");

    let rate_form = istek(b_kapi, "198.51.100.9", "GET", "/giris", None, "");
    let rate_cerez = cerez(&rate_form);
    let rate_csrf = csrf(&rate_form);
    let mut durumlar = Vec::new();
    for sira in 0..6 {
        let kapi = if sira % 2 == 0 { a_kapi } else { b_kapi };
        let yanit = istek(
            kapi,
            "198.51.100.9",
            "POST",
            "/giris-yap",
            Some(&rate_cerez),
            &format!("_csrf={rate_csrf}&parola=yanlis"),
        );
        durumlar.push(durum(&yanit));
    }
    assert_eq!(&durumlar[..5], &[303, 303, 303, 303, 303]);
    assert_eq!(durumlar[5], if bellek { 303 } else { 429 });
    if bellek {
        for _ in 0..2 {
            assert_eq!(
                durum(&istek(
                    a_kapi,
                    "198.51.100.9",
                    "POST",
                    "/giris-yap",
                    Some(&rate_cerez),
                    &format!("_csrf={rate_csrf}&parola=yanlis")
                )),
                303
            );
        }
        assert_eq!(
            durum(&istek(
                a_kapi,
                "198.51.100.9",
                "POST",
                "/giris-yap",
                Some(&rate_cerez),
                &format!("_csrf={rate_csrf}&parola=yanlis")
            )),
            429
        );
    }

    drop(a);
    drop(b);
    assert!(gecici.0.join(".zee/web-durumu-v1.json").is_file());
}

fn ham_ip_istegi(kapi: u16, basliklar: &str, yol: &str) -> String {
    let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
    let son = Instant::now() + Duration::from_secs(10);
    let mut stream = loop {
        match TcpStream::connect_timeout(&adres, Duration::from_millis(100)) {
            Ok(s) => break s,
            Err(_) if Instant::now() < son => std::thread::sleep(Duration::from_millis(20)),
            Err(h) => panic!("worker dinlemeye geçmedi: {h}"),
        }
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream
        .write_all(
            format!(
                "GET {yol} HTTP/1.1\r\nHost: panel.example\r\n{basliklar}Connection: close\r\n\r\n"
            )
            .as_bytes(),
        )
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

#[test]
fn istemci_adresi_proxy_ve_socketten_gelir_xff_ve_form_alanindan_gelmez() {
    let gecici = GeciciKlasor::yeni();
    let kaynak = gecici.0.join("adres.dil");
    std::fs::write(&kaynak, "8092 kapısında sunucu başlat\nGET \"/ip\" adresine istek geldiğinde\n    istemci adresi yanıtını gönder\n").unwrap();
    let cli_kaynak = gecici.0.join("cli.dil");
    std::fs::write(&cli_kaynak, "istemci adresi yaz\n").unwrap();
    let cli = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", cli_kaynak.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    assert_eq!(String::from_utf8(cli.stdout).unwrap(), "\n");
    let proxy_kapi = bos_kapi();
    let proxy = sunucuyu_baslat(&kaynak, proxy_kapi);
    for (forwarded, expected) in [
        ("203.0.113.7", "203.0.113.7"),
        (
            "\"[2001:0DB8:0000:0000:0000:0000:0000:0001]\"",
            "2001:db8::1",
        ),
        ("198.51.100.9", "198.51.100.9"),
    ] {
        let headers = format!("Forwarded: for={forwarded};proto=https;host=panel.example\r\nX-Forwarded-For: 192.0.2.99\r\n");
        let response = ham_ip_istegi(
            proxy_kapi,
            &headers,
            "/ip?istemci=192.0.2.42&adresi=192.0.2.43",
        );
        assert_eq!(durum(&response), 200, "{response}");
        assert_eq!(response.split_once("\r\n\r\n").unwrap().1, expected);
    }
    let rejected = ham_ip_istegi(
        proxy_kapi,
        "Forwarded: for=unknown;proto=https;host=panel.example\r\nX-Forwarded-For: 192.0.2.99\r\n",
        "/ip",
    );
    assert_eq!(durum(&rejected), 400);
    drop(proxy);
    let direct_kapi = bos_kapi();
    let direct_kaynak = gecici.0.join("socket.dil");
    std::fs::write(&direct_kaynak, format!("{direct_kapi} kapısında sunucu başlat\nGET \"/ip\" adresine istek geldiğinde\n    istemci adresi yanıtını gönder\n")).unwrap();
    let direct = Sunucu(
        Command::new(env!("CARGO_BIN_EXE_dil"))
            .args([
                "çalıştır",
                "--deneysel-web",
                direct_kaynak.to_str().unwrap(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let response = ham_ip_istegi(direct_kapi, "X-Forwarded-For: 192.0.2.99\r\nForwarded: for=203.0.113.7;proto=https;host=panel.example\r\n", "/ip");
    assert_eq!(durum(&response), 200, "{response}");
    assert_eq!(response.split_once("\r\n\r\n").unwrap().1, "127.0.0.1");
    drop(direct);
}
