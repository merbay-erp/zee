//! K-137: iki bağımsız `dil` sürecinin aynı kalıcı oturum/rate-limit
//! durumunu gerçekten paylaştığı uçtan uca güvenli-proxy kanıtı.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[path = "semantic_regresyon/native_web_kopma.rs"]
mod kopma;

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
        // K-338: işçi grubu kendi süreç grubunda başlar; grup lideri henüz
        // biçilmediğinden bu kimlik başka gruba ait olamaz. Lider olmayan
        // süreçte ESRCH döner, etkisizdir. Donmuş (SIGSTOP) ya da sahipsiz
        // işçi test sonrasında yaşamaz.
        #[cfg(unix)]
        // SAFETY: signal to our own unreaped child's process group.
        unsafe {
            libc::kill(-(self.0.id() as libc::pid_t), libc::SIGKILL);
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// K-338: süre dolarsa işçi grubunu (süreç grubu) öldürür; böylece hiçbir
/// bekleme sonsuza dek asılı kalmaz ve test görünür biçimde başarısız olur.
#[cfg(unix)]
struct Bekci(std::sync::Arc<std::sync::atomic::AtomicBool>);

#[cfg(unix)]
impl Bekci {
    fn kur(grup: u32, sure: Duration) -> Self {
        let bitti = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let izle = std::sync::Arc::clone(&bitti);
        std::thread::spawn(move || {
            let son = Instant::now() + sure;
            while Instant::now() < son {
                if izle.load(Ordering::Relaxed) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            eprintln!("bekçi: {sure:?} doldu; işçi grubu {grup} öldürülüyor");
            // SAFETY: SIGKILL to the test-owned manager's process group.
            unsafe { libc::kill(-(grup as libc::pid_t), libc::SIGKILL) };
        });
        Self(bitti)
    }
}

#[cfg(unix)]
impl Drop for Bekci {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

fn bos_kapi() -> u16 {
    kopma::bos_kapi()
}

fn sunucuyu_baslat(kaynak: &Path, kapi: u16) -> Sunucu {
    sunucuyu_baslat_oranla(kaynak, kapi, false)
}

fn sunucuyu_baslat_oranla(kaynak: &Path, kapi: u16, bellek: bool) -> Sunucu {
    sunucuyu_baslat_ayarlarla(
        kaynak,
        kapi,
        if bellek {
            &["--web-oran-deposu", "bellek"]
        } else {
            &[]
        },
    )
}

fn sunucuyu_baslat_ayarlarla(kaynak: &Path, kapi: u16, ayarlar: &[&str]) -> Sunucu {
    let kapi_yazisi = kapi.to_string();
    Sunucu(kopma::dinleyene_dek_baslat(|| {
        let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
        komut.args([
            "çalıştır",
            "--web-proxy",
            "https://panel.example",
            "--web-worker-port",
            &kapi_yazisi,
            kaynak.to_str().expect("UTF-8 kaynak yolu"),
        ]);
        komut.args(ayarlar).stderr(Stdio::null());
        komut
    }))
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

fn cerez_satiri(yanit: &str) -> Option<&str> {
    yanit
        .lines()
        .find(|satir| satir.starts_with("Set-Cookie: __Host-zee-oturum="))
}

fn max_age(satir: &str) -> i64 {
    satir
        .split("; ")
        .find_map(|parca| parca.strip_prefix("Max-Age="))
        .and_then(|deger| deger.parse().ok())
        .expect("Max-Age")
}

/// K-330: proje bildirimiyle seçilen kayar oturum iki gerçek worker arasında
/// ve yeniden başlatmada korunur; her kimlikli istek aynı belirteçle çerez
/// Max-Age'ini yeniler, anonim oturum ve çıkış bundan etkilenmez.
#[test]
fn kayar_oturum_iki_worker_restart_ve_cikista_dogru_davranir() {
    let gecici = GeciciKlasor::yeni();
    std::fs::write(
        gecici.0.join("proje.dil"),
        "proje \"kayar-panel\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
         yetkinlikler \"dosya-okuma\", \"dosya-yazma\", \"ağ-sunucusu\", \"web-oturumu\", \"kriptografi\" listesi olsun\n\
         oturum_ömrü_saniye 28800 olsun\noturum_kayar doğru olsun\n",
    )
    .expect("proje bildirimi");
    std::fs::write(
        gecici.0.join("ana.dil"),
        include_str!("../../projeler/girisli-panel.dil"),
    )
    .expect("proje kaynağı");
    let kilit = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["kilitle", gecici.0.to_str().expect("UTF-8 yol")])
        .output()
        .expect("dil kilitle");
    assert!(
        kilit.status.success(),
        "{}",
        String::from_utf8_lossy(&kilit.stderr)
    );
    let a_kapi = bos_kapi();
    let b_kapi = bos_kapi();
    let mut a = sunucuyu_baslat(&gecici.0, a_kapi);
    let b = sunucuyu_baslat(&gecici.0, b_kapi);

    let form = istek(a_kapi, "203.0.113.8", "GET", "/giris", None, "");
    assert_eq!(durum(&form), 200);
    let anonim_satiri = cerez_satiri(&form).expect("anonim çerez").to_string();
    assert_eq!(
        max_age(&anonim_satiri),
        600,
        "anonim oturum kısa ve mutlak kalır"
    );
    let anonim = cerez(&form);
    let anonim_tekrar = istek(a_kapi, "203.0.113.8", "GET", "/giris", Some(&anonim), "");
    assert_eq!(durum(&anonim_tekrar), 200);
    assert!(
        cerez_satiri(&anonim_tekrar).is_none(),
        "anonim oturum kayar yenilenmez"
    );
    let giris = istek(
        a_kapi,
        "203.0.113.8",
        "POST",
        "/giris-yap",
        Some(&anonim),
        &format!("_csrf={}&parola=zee2026", csrf(&form)),
    );
    assert_eq!(durum(&giris), 303);
    let oturum = cerez(&giris);
    assert_ne!(oturum, anonim, "giriş oturum kimliğini döndürür");
    assert_eq!(max_age(cerez_satiri(&giris).unwrap()), 28800);

    let b_okuma = istek(b_kapi, "203.0.113.8", "GET", "/yonet", Some(&oturum), "");
    assert_eq!(durum(&b_okuma), 200, "B worker A oturumunu görmeli");
    let yenileme = cerez_satiri(&b_okuma).expect("kayar çerez yenilemesi");
    assert_eq!(cerez(&b_okuma), oturum, "yenileme aynı belirteci taşır");
    assert!((28799..=28800).contains(&max_age(yenileme)), "{yenileme}");
    assert!(yenileme.contains("HttpOnly") && yenileme.contains("Secure"));

    a.0.kill().expect("A worker durdurulmalı");
    a.0.wait().expect("A worker sonucu");
    drop(a);
    let a = sunucuyu_baslat(&gecici.0, a_kapi);
    let restart = istek(a_kapi, "203.0.113.8", "GET", "/yonet", Some(&oturum), "");
    assert_eq!(durum(&restart), 200, "restart kayar oturumu kaybetmemeli");
    assert_eq!(cerez(&restart), oturum);

    let cikis = istek(
        b_kapi,
        "203.0.113.8",
        "POST",
        "/cikis",
        Some(&oturum),
        &format!("_csrf={}", csrf(&restart)),
    );
    assert_eq!(durum(&cikis), 303);
    let cikis_cerezleri = cikis
        .lines()
        .filter(|satir| satir.starts_with("Set-Cookie: __Host-zee-oturum="))
        .collect::<Vec<_>>();
    assert_eq!(cikis_cerezleri.len(), 1, "{cikis_cerezleri:?}");
    assert_eq!(max_age(cikis_cerezleri[0]), 0, "çıkış çerezi hemen siler");
    let iptal = istek(a_kapi, "203.0.113.8", "GET", "/yonet", Some(&oturum), "");
    assert_eq!(durum(&iptal), 401, "B çıkışı A worker'da hemen görünmeli");
    assert!(cerez_satiri(&iptal).is_none());
    drop(a);
    drop(b);
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
    let direct = Sunucu(kopma::dinleyene_dek_baslat(|| {
        let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
        komut
            .args([
                "çalıştır",
                "--deneysel-web",
                direct_kaynak.to_str().unwrap(),
            ])
            .stderr(Stdio::null());
        komut
    }));
    let response = ham_ip_istegi(direct_kapi, "X-Forwarded-For: 192.0.2.99\r\nForwarded: for=203.0.113.7;proto=https;host=panel.example\r\n", "/ip");
    assert_eq!(durum(&response), 200, "{response}");
    assert_eq!(response.split_once("\r\n\r\n").unwrap().1, "127.0.0.1");
    drop(direct);
}

#[test]
fn ayarli_bellek_worker_kapasiteyi_asinca_hizmet_eder_ve_kotayi_uygular() {
    let gecici = GeciciKlasor::yeni();
    let kaynak = gecici.0.join("yuk.dil");
    std::fs::write(
        &kaynak,
        include_str!("../../dogfood/zee-cekirdek/oran-yuk/yuk.dil"),
    )
    .unwrap();
    let kapi = bos_kapi();
    let _sunucu = sunucuyu_baslat_ayarlarla(
        &kaynak,
        kapi,
        &[
            "--web-oran-deposu",
            "bellek",
            "--web-oran-siniri",
            "2",
            "--web-oran-kapasitesi",
            "2",
            "--web-oran-doluluk",
            "en-eski",
        ],
    );
    for i in 0..20 {
        let yanit = istek(kapi, "203.0.113.9", "GET", &format!("/wiki/{i}"), None, "");
        assert_eq!(durum(&yanit), 200);
        assert!(yanit.contains("Windows &lt;11&gt; güvenlik makalesi 100"));
    }
    let yanit = istek(kapi, "203.0.113.9", "GET", "/wiki/19", None, "");
    assert_eq!(durum(&yanit), 200);
    let yanit = istek(kapi, "203.0.113.9", "GET", "/wiki/19", None, "");
    assert_eq!(durum(&yanit), 429);
    assert!(!gecici.0.join(".zee/web-durumu-v1.json").exists());
}

#[test]
fn rota_hatasi_gercek_worker_ta_500_olur_ve_operator_stderr_satiri_yazar() {
    // K-324/ADR-157: girdiye bağlı çalışma hatası worker'ı düşürmez; tanı
    // istemciye değil, sorgusuz ve tek satır olarak operatöre gider.
    let gecici = GeciciKlasor::yeni();
    let kapi = bos_kapi();
    let yol = gecici.0.join("worker.dil");
    std::fs::write(
        &yol,
        "8080 kapısında sunucu başlat\n\nGET \"/sayi\" adresine istek geldiğinde\n    girdi isteğin \"n\" değeri olsun\n    sayı girdinin sayısı olsun\n    \"bitmemeliydi\" yanıtını gönder\n\nGET \"/saglik\" adresine istek geldiğinde\n    \"ayakta\" yanıtını gönder\n",
    )
    .expect("worker kaynağı");
    let mut cocuk = kopma::worker_baslat(&yol, kapi, Stdio::piped);

    let hata = istek(kapi, "203.0.113.9", "GET", "/sayi?n=gizli-abc", None, "");
    assert_eq!(durum(&hata), 500, "{hata}");
    assert!(hata.ends_with("istek işlenemedi; sunucu hatası"), "{hata}");
    assert!(!hata.contains("C004") && !hata.contains("gizli"), "{hata}");
    let saglik = istek(kapi, "203.0.113.9", "GET", "/saglik", None, "");
    assert_eq!(durum(&saglik), 200, "worker ilk hatadan sonra yaşamalı");
    assert!(cocuk.try_wait().expect("worker durumu").is_none());

    cocuk.kill().expect("worker durdurulmalı");
    let mut stderr = String::new();
    cocuk
        .stderr
        .take()
        .expect("stderr borusu")
        .read_to_string(&mut stderr)
        .expect("stderr okunmalı");
    cocuk.wait().expect("worker sonucu");
    let kayitlar: Vec<&str> = stderr
        .lines()
        .filter(|satir| satir.starts_with("zee-web istek-hatası "))
        .collect();
    assert_eq!(kayitlar.len(), 1, "{stderr}");
    assert!(
        kayitlar[0].starts_with(
            "zee-web istek-hatası durum=500 kod=C004 satır=5 yöntem=GET yol=\"/sayi\" mesaj=\""
        ),
        "{stderr}"
    );
    assert!(!stderr.contains("n=gizli"), "{stderr}");
}

/// Her kopma ayrı istemci kimliği taşır; uç nokta oran sınırı bu testin
/// konusu değildir.
#[cfg(unix)]
fn kopar(kapi: u16, sira: u16) {
    let ilk = kopma::yazim_ortasinda_kopar(
        kapi,
        "GET /buyuk",
        &format!("198.18.{}.{}", sira / 256, sira % 256),
    );
    assert!(
        ilk.starts_with(b"HTTP/1.1 200 OK"),
        "{}",
        String::from_utf8_lossy(&ilk)
    );
}

#[cfg(unix)]
fn rss_kib(pid: u32) -> u64 {
    let cikti = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .expect("ps çalışmalı");
    String::from_utf8_lossy(&cikti.stdout)
        .trim()
        .parse()
        .expect("RSS KiB")
}

#[cfg(unix)]
#[test]
fn yanit_yazimi_sirasinda_kopan_istemci_worker_i_dusurmez_ve_izin_sizdirmaz() {
    // W1: 2 MiB yanıtın yazımı ortasında gelen RST istek kapsamlıdır; worker
    // ayakta kalır, sonraki istek 200 olur. 100 ardışık kopma, 64'lük süreç
    // bağlantı izni sınırını aşar: izin sızsaydı sağlık isteği 503 alırdı.
    let gecici = GeciciKlasor::yeni();
    let kapi = bos_kapi();
    let yol = gecici.0.join("worker.dil");
    std::fs::write(
        &yol,
        "8080 kapısında sunucu başlat\n\nGET \"/buyuk\" adresine istek geldiğinde\n    metin \"x\" olsun\n    21 kez tekrarla\n        metin metin ile metin olsun\n    metin yanıtını gönder\n\nGET \"/saglik\" adresine istek geldiğinde\n    \"ayakta\" yanıtını gönder\n",
    )
    .expect("worker kaynağı");
    let mut cocuk = kopma::worker_baslat(&yol, kapi, Stdio::piped);

    let tam = istek(kapi, "203.0.113.7", "GET", "/buyuk", None, "");
    assert_eq!(durum(&tam), 200);
    assert!(tam.ends_with(&"x".repeat(1024)), "tam 2 MiB gövde gelmeli");
    assert!(tam.len() > 2 * 1024 * 1024, "{}", tam.len());

    kopar(kapi, 0);
    let saglik = istek(kapi, "203.0.113.7", "GET", "/saglik", None, "");
    assert_eq!(durum(&saglik), 200, "worker ilk kopmadan sonra yaşamalı");
    let rss_once = rss_kib(cocuk.id());

    for sira in 1..=100 {
        kopar(kapi, sira);
    }
    let saglik = istek(kapi, "203.0.113.8", "GET", "/saglik", None, "");
    assert_eq!(
        durum(&saglik),
        200,
        "100 kopmadan sonra izin sızıntısı olmamalı"
    );
    assert!(saglik.ends_with("ayakta"), "{saglik}");
    assert!(cocuk.try_wait().expect("worker durumu").is_none());
    let rss_sonra = rss_kib(cocuk.id());
    assert!(
        rss_sonra <= rss_once + 32 * 1024,
        "100 kopmada RSS büyümesi 32 MiB'ı aşmamalı: {rss_once} KiB → {rss_sonra} KiB"
    );

    cocuk.kill().expect("worker durdurulmalı");
    let mut stderr = String::new();
    cocuk
        .stderr
        .take()
        .expect("stderr borusu")
        .read_to_string(&mut stderr)
        .expect("stderr okunmalı");
    cocuk.wait().expect("worker sonucu");
    let kopmalar: Vec<&str> = stderr
        .lines()
        .filter(|satir| satir.starts_with("zee-web istemci yanıt sırasında koptu: "))
        .collect();
    assert_eq!(kopmalar.len(), 101, "her kopma tek satır: {stderr}");
    assert!(
        kopmalar.iter().all(|satir| satir.starts_with(
            "zee-web istemci yanıt sırasında koptu: durum=200 yöntem=GET yol=\"/buyuk\" hata=\""
        )),
        "{stderr}"
    );
    assert!(!stderr.contains("C022"), "{stderr}");
}

// K-336 (W2) / K-337 (W4): resmî çok işçi profili. Yönetici süreç tek
// loopback dinleyiciyi bağlar ve N işçiye devreder; tek komutla başlar.

#[cfg(unix)]
fn grubu_baslat(kaynak: &Path, kapi: u16, isci: u16, saglik: Option<u16>) -> Sunucu {
    let kapi_yazisi = kapi.to_string();
    let isci_yazisi = isci.to_string();
    let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
    komut.args([
        "çalıştır",
        "--web-proxy",
        "https://panel.example",
        "--web-worker-port",
        &kapi_yazisi,
        "--web-isci-sayisi",
        &isci_yazisi,
    ]);
    if let Some(saglik) = saglik {
        komut.args(["--web-saglik-port", &saglik.to_string()]);
    }
    komut.arg(kaynak.to_str().expect("UTF-8 kaynak yolu"));
    komut
        .current_dir(kaynak.parent().expect("kaynak klasörü"))
        .stderr(Stdio::null());
    Sunucu(kopma::dinleyene_dek_baslat(|| {
        use std::os::unix::process::CommandExt;
        let mut kopya = Command::new(komut.get_program());
        kopya
            .args(komut.get_args())
            .current_dir(kaynak.parent().expect("kaynak klasörü"))
            .stderr(Stdio::null())
            .process_group(0);
        kopya
    }))
}

#[cfg(unix)]
fn saglik_oku(kapi: u16) -> (u16, String) {
    let son = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(mut akis) = TcpStream::connect(("127.0.0.1", kapi)) {
            akis.set_read_timeout(Some(Duration::from_secs(5))).ok();
            akis.write_all(b"GET /saglik HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
                .expect("sağlık isteği");
            let mut yanit = String::new();
            akis.read_to_string(&mut yanit).expect("sağlık yanıtı");
            let govde = yanit.split("\r\n\r\n").nth(1).unwrap_or_default();
            return (durum(&yanit), govde.to_string());
        }
        assert!(Instant::now() < son, "sağlık portu açılmadı");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(unix)]
fn saglik_bekle(kapi: u16, kosul: impl Fn(&str) -> bool) -> String {
    let son = Instant::now() + Duration::from_secs(20);
    loop {
        let (kod, govde) = saglik_oku(kapi);
        if kod == 200 && kosul(&govde) {
            return govde;
        }
        assert!(
            Instant::now() < son,
            "grup beklenen sağlığa ulaşmadı: {kod} {govde}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(unix)]
fn surecler(govde: &str) -> Vec<u32> {
    govde
        .split("\"surecler\":[")
        .nth(1)
        .and_then(|k| k.split(']').next())
        .expect("süreç listesi")
        .split(',')
        .filter(|p| !p.is_empty())
        .map(|p| p.parse().expect("pid"))
        .collect()
}

#[cfg(unix)]
fn sinyal(pid: u32, ad: &str) -> bool {
    Command::new("kill")
        .args([ad, &pid.to_string()])
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Aynı giriş/CSRF/çıkış/oran senaryosu tek süreçte ve üç işçili grupta
/// aynı durum kodu dizisini üretmelidir (W2 conformance).
#[cfg(unix)]
fn panel_senaryosu(isci: Option<u16>) -> Vec<u16> {
    let gecici = GeciciKlasor::yeni();
    let kapi = bos_kapi();
    let yol = gecici.0.join("panel.dil");
    std::fs::write(&yol, include_str!("../../projeler/girisli-panel.dil")).expect("kaynak");
    let _sunucu = match isci {
        Some(n) => grubu_baslat(&yol, kapi, n, None),
        None => sunucuyu_baslat(&yol, kapi),
    };
    let mut durumlar = Vec::new();
    let form = istek(kapi, "203.0.113.7", "GET", "/giris", None, "");
    durumlar.push(durum(&form));
    let anonim = cerez(&form);
    let giris_csrf = csrf(&form);
    durumlar.push(durum(&istek(
        kapi,
        "203.0.113.7",
        "POST",
        "/giris-yap",
        Some(&anonim),
        "_csrf=sahte&parola=zee2026",
    )));
    let giris = istek(
        kapi,
        "203.0.113.7",
        "POST",
        "/giris-yap",
        Some(&anonim),
        &format!("_csrf={giris_csrf}&parola=zee2026"),
    );
    durumlar.push(durum(&giris));
    let oturum = cerez(&giris);
    let mut son_okuma = String::new();
    for _ in 0..8 {
        son_okuma = istek(kapi, "203.0.113.7", "GET", "/yonet", Some(&oturum), "");
        durumlar.push(durum(&son_okuma));
    }
    durumlar.push(durum(&istek(
        kapi,
        "203.0.113.7",
        "POST",
        "/cikis",
        Some(&oturum),
        &format!("_csrf={}", csrf(&son_okuma)),
    )));
    durumlar.push(durum(&istek(
        kapi,
        "203.0.113.7",
        "GET",
        "/yonet",
        Some(&oturum),
        "",
    )));
    let oran_form = istek(kapi, "198.51.100.9", "GET", "/giris", None, "");
    let oran_cerez = cerez(&oran_form);
    let oran_csrf = csrf(&oran_form);
    for _ in 0..6 {
        durumlar.push(durum(&istek(
            kapi,
            "198.51.100.9",
            "POST",
            "/giris-yap",
            Some(&oran_cerez),
            &format!("_csrf={oran_csrf}&parola=yanlis"),
        )));
    }
    durumlar
}

#[cfg(unix)]
#[test]
fn isci_grubu_oturum_csrf_ve_oran_siniri_tek_surecle_ayni() {
    let tek = panel_senaryosu(None);
    let grup = panel_senaryosu(Some(3));
    let mut beklenen = vec![200, 403, 303];
    beklenen.extend([200; 8]);
    beklenen.extend([303, 401, 303, 303, 303, 303, 303, 429]);
    assert_eq!(tek, beklenen, "tek süreç taban davranışı");
    assert_eq!(grup, tek, "üç işçili grup tek süreçle aynı olmalı");
}

#[cfg(unix)]
#[test]
fn isci_grubu_acilisi_yalniz_birincilde_kosturur_ve_yeniden_baslatmada_tekrarlamaz() {
    let gecici = GeciciKlasor::yeni();
    let kapi = bos_kapi();
    let saglik = bos_kapi();
    let yol = gecici.0.join("ana.dil");
    std::fs::write(
        &yol,
        format!(
            "\"acilis.log\" dosyasına \"süreç\" ekle\n\
             sahip birincil süreç olsun\n\
             sahip ise\n    \"acilis.log\" dosyasına \"açılış başladı\" ekle\n    \
             toplam 0 olsun\n    1 den 400000 e kadar her sayı için\n        toplamı sayıyla artır\n    \
             \"acilis.log\" dosyasına \"açılış bitti\" ekle\n\
             {kapi} kapısında sunucu başlat\n\n\
             GET \"/rol\" adresine istek geldiğinde\n    rol birincil süreç olsun\n    rol ise\n        \
             \"birincil\" yanıtını gönder\n    değilse\n        \"ikincil\" yanıtını gönder\n"
        ),
    )
    .expect("kaynak");
    let mut grup = grubu_baslat(&yol, kapi, 4, Some(saglik));
    let govde = saglik_bekle(saglik, |g| g.contains("\"canli\":4"));
    let ilk = surecler(&govde);
    assert_eq!(ilk.len(), 4);
    let gunluk_bekle = |adet: usize| {
        let son = Instant::now() + Duration::from_secs(20);
        loop {
            let gunluk = std::fs::read_to_string(gecici.0.join("acilis.log")).unwrap_or_default();
            if gunluk.lines().count() >= adet || Instant::now() >= son {
                return gunluk;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    };
    let gunluk = gunluk_bekle(6);
    let satirlar = gunluk.lines().collect::<Vec<_>>();
    assert_eq!(
        satirlar,
        vec![
            "süreç",
            "açılış başladı",
            "açılış bitti",
            "süreç",
            "süreç",
            "süreç"
        ],
        "ikinciller birincilin açılışı bitmeden başlamamalı"
    );
    for _ in 0..12 {
        let yanit = istek(kapi, "203.0.113.7", "GET", "/rol", None, "");
        assert_eq!(durum(&yanit), 200);
    }

    // Birincil öldürülür: ikincil olarak döner, açılış yeniden koşmaz.
    assert!(sinyal(ilk[0], "-KILL"));
    let govde = saglik_bekle(saglik, |g| {
        g.contains("\"canli\":4") && g.contains("\"yeniden_baslatma\":1")
    });
    let yeni = surecler(&govde);
    assert!(!yeni.contains(&ilk[0]));
    let yanit = istek(kapi, "203.0.113.7", "GET", "/rol", None, "");
    assert_eq!(durum(&yanit), 200);
    assert!(
        yanit.ends_with("ikincil"),
        "yeniden başlayan işçi birincil olamaz"
    );
    let gunluk = gunluk_bekle(7);
    assert_eq!(gunluk.matches("açılış başladı").count(), 1);
    assert_eq!(gunluk.matches("süreç").count(), 5);

    // SIGTERM: yönetici işçileri boşaltır ve başarıyla çıkar.
    assert!(sinyal(grup.0.id(), "-TERM"));
    let son = Instant::now() + Duration::from_secs(45);
    let cikis = loop {
        if let Some(durum) = grup.0.try_wait().expect("grup durumu") {
            break durum;
        }
        assert!(Instant::now() < son, "grup SIGTERM ile kapanmadı");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(cikis.success(), "{cikis}");
    for pid in yeni {
        assert!(
            !sinyal(pid, "-0"),
            "işçi {pid} yöneticiden sonra yaşamamalı"
        );
    }
}

#[cfg(unix)]
#[test]
fn isci_grubu_yonetici_olurse_isciler_dinleyiciyi_birakir() {
    let gecici = GeciciKlasor::yeni();
    let kapi = bos_kapi();
    let saglik = bos_kapi();
    let yol = gecici.0.join("ana.dil");
    std::fs::write(
        &yol,
        format!("{kapi} kapısında sunucu başlat\n\nGET \"/\" adresine istek geldiğinde\n    \"tamam\" yanıtını gönder\n"),
    )
    .expect("kaynak");
    let mut grup = grubu_baslat(&yol, kapi, 2, Some(saglik));
    let pidler = surecler(&saglik_bekle(saglik, |g| g.contains("\"canli\":2")));
    grup.0.kill().expect("yönetici SIGKILL");
    grup.0.wait().expect("yönetici sonucu");
    let son = Instant::now() + Duration::from_secs(10);
    while pidler.iter().any(|pid| sinyal(*pid, "-0")) {
        assert!(Instant::now() < son, "sahipsiz işçiler kapanmadı");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(unix)]
fn saglik_ham(kapi: u16, istek_satiri: &str) -> Option<String> {
    let mut akis = TcpStream::connect(("127.0.0.1", kapi)).ok()?;
    akis.set_read_timeout(Some(Duration::from_secs(5))).ok();
    akis.write_all(format!("{istek_satiri}\r\nHost: 127.0.0.1\r\n\r\n").as_bytes())
        .ok()?;
    let mut yanit = String::new();
    akis.read_to_string(&mut yanit).ok()?;
    Some(yanit)
}

#[cfg(unix)]
fn grup_cikisini_bekle(grup: &mut Sunucu, sure: Duration) -> (std::process::ExitStatus, Duration) {
    let baslangic = Instant::now();
    loop {
        if let Some(durum) = grup.0.try_wait().expect("grup durumu") {
            return (durum, baslangic.elapsed());
        }
        assert!(baslangic.elapsed() < sure, "grup {sure:?} içinde çıkmadı");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// K-338: yuva 10 sn dolmadan art arda 10. kez çıkınca grup çıkış 1 ile durur
/// (önceden 11. çıkışta duruyordu). Aynı koşu sağlık uç noktasının 503
/// `basliyor`/`eksik` ara durumlarını, HEAD gövdesizliğini ve 404 yolunu sınar.
#[cfg(unix)]
#[test]
fn isci_grubu_art_arda_hizli_cokuste_durur_ve_saglik_ara_durumlarini_bildirir() {
    let gecici = GeciciKlasor::yeni();
    let kapi = bos_kapi();
    let saglik = bos_kapi();
    let yol = gecici.0.join("ana.dil");
    std::fs::write(
        &yol,
        format!(
            "sahip birincil süreç olsun\n\
             sahip ise\n    toplam 0 olsun\n    1 den 1000000 e kadar her sayı için\n        \
             toplamı sayıyla artır\n\
             değilse\n    programı 3 ile bitir\n\
             {kapi} kapısında sunucu başlat\n\n\
             GET \"/\" adresine istek geldiğinde\n    \"tamam\" yanıtını gönder\n"
        ),
    )
    .expect("kaynak");
    let gunluk = gecici.0.join("yonetici.log");
    use std::os::unix::process::CommandExt;
    let mut grup = Sunucu(
        Command::new(env!("CARGO_BIN_EXE_dil"))
            .args([
                "çalıştır",
                "--web-proxy",
                "https://panel.example",
                "--web-worker-port",
                &kapi.to_string(),
                "--web-isci-sayisi",
                "2",
                "--web-saglik-port",
                &saglik.to_string(),
                yol.to_str().expect("UTF-8 kaynak yolu"),
            ])
            .current_dir(&gecici.0)
            .stdout(Stdio::null())
            .stderr(std::fs::File::create(&gunluk).expect("yönetici günlüğü"))
            .process_group(0)
            .spawn()
            .expect("yönetici"),
    );
    let _bekci = Bekci::kur(grup.0.id(), Duration::from_secs(150));

    // Birincil açılışı sürerken: 503 basliyor; HEAD aynı başlığı gövdesiz verir.
    let son = Instant::now() + Duration::from_secs(20);
    let basliyor = loop {
        if let Some(yanit) = saglik_ham(saglik, "GET /saglik HTTP/1.1") {
            break yanit;
        }
        assert!(Instant::now() < son, "sağlık portu açılmadı");
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(durum(&basliyor), 503, "{basliyor}");
    assert!(
        basliyor.ends_with(
            "{\"durum\":\"basliyor\",\"isci\":2,\"canli\":0,\"yeniden_baslatma\":0,\"surecler\":[]}\n"
        ),
        "{basliyor}"
    );
    let head = saglik_ham(saglik, "HEAD /saglik HTTP/1.1").expect("HEAD yanıtı");
    assert!(head.starts_with("HTTP/1.1 "), "{head}");
    assert!(head.ends_with("\r\n\r\n"), "HEAD gövde taşımamalı: {head}");
    assert!(head.contains("Content-Length: "), "{head}");
    for istek_satiri in [
        "GET /baska HTTP/1.1",
        "POST /saglik HTTP/1.1",
        "GET /saglik/ HTTP/1.1",
    ] {
        let yanit = saglik_ham(saglik, istek_satiri).expect("404 yanıtı");
        assert_eq!(durum(&yanit), 404, "{istek_satiri}: {yanit}");
        assert!(
            yanit.ends_with("{\"durum\":\"bulunamadi\"}\n"),
            "{istek_satiri}: {yanit}"
        );
    }

    // İkincil sürekli çöker: grup hazır olsa da işçi eksik → 503 eksik.
    let son = Instant::now() + Duration::from_secs(30);
    let eksik = loop {
        let yanit = saglik_ham(saglik, "GET /saglik HTTP/1.1").unwrap_or_default();
        if yanit.contains("\"durum\":\"eksik\"") {
            break yanit;
        }
        assert!(Instant::now() < son, "eksik durumu görülmedi: {yanit}");
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(durum(&eksik), 503, "{eksik}");
    assert!(eksik.contains("\"canli\":1"), "{eksik}");
    let birincil = surecler(eksik.split("\r\n\r\n").nth(1).unwrap_or_default());
    assert_eq!(birincil.len(), 1, "{eksik}");

    // 250 ms → 5 sn katlanan gecikmelerle dokuz yeniden başlatma ≈ 28 sn.
    let (cikis, _) = grup_cikisini_bekle(&mut grup, Duration::from_secs(90));
    assert_eq!(cikis.code(), Some(1), "{cikis}");
    let metin = std::fs::read_to_string(&gunluk).expect("yönetici günlüğü");
    assert_eq!(
        metin.matches("zee-web işçi 2 başladı").count(),
        10,
        "ikinci yuva tam 10 kez başlamalı (ilk + 9 yeniden):\n{metin}"
    );
    assert_eq!(metin.matches("zee-web işçi 2 (pid ").count(), 9, "{metin}");
    assert!(
        metin.contains("zee-web işçi 2 art arda 10 kez hızlı çıktı; grup durduruluyor"),
        "{metin}"
    );
    assert!(
        !sinyal(birincil[0], "-0"),
        "birincil işçi grup durduktan sonra yaşamamalı"
    );
}

/// K-338: SIGTERM sonrası 40 sn içinde bitmeyen işçi (burada SIGSTOP ile
/// donmuş) sonlandırılır; yönetici çıkış 1 verir. Kapanış boyunca sağlık
/// 503 `kapaniyor` döner.
#[cfg(unix)]
#[test]
fn isci_grubu_kapanista_bitmeyen_isciyi_40_sn_sonra_sonlandirir() {
    let gecici = GeciciKlasor::yeni();
    let kapi = bos_kapi();
    let saglik = bos_kapi();
    let yol = gecici.0.join("ana.dil");
    std::fs::write(
        &yol,
        format!("{kapi} kapısında sunucu başlat\n\nGET \"/\" adresine istek geldiğinde\n    \"tamam\" yanıtını gönder\n"),
    )
    .expect("kaynak");
    let mut grup = grubu_baslat(&yol, kapi, 2, Some(saglik));
    let _bekci = Bekci::kur(grup.0.id(), Duration::from_secs(120));
    let pidler = surecler(&saglik_bekle(saglik, |g| g.contains("\"canli\":2")));
    assert_eq!(pidler.len(), 2);
    assert!(sinyal(pidler[1], "-STOP"));
    assert!(sinyal(grup.0.id(), "-TERM"));
    let son = Instant::now() + Duration::from_secs(10);
    let kapaniyor = loop {
        let yanit = saglik_ham(saglik, "GET /saglik HTTP/1.1").unwrap_or_default();
        if yanit.contains("\"durum\":\"kapaniyor\"") {
            break yanit;
        }
        assert!(Instant::now() < son, "kapaniyor durumu görülmedi: {yanit}");
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(durum(&kapaniyor), 503, "{kapaniyor}");
    let son = Instant::now() + Duration::from_secs(10);
    while sinyal(pidler[0], "-0") {
        assert!(Instant::now() < son, "donmamış işçi SIGTERM ile çıkmalı");
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        sinyal(pidler[1], "-0"),
        "donmuş işçi süre dolmadan yaşamalı"
    );
    let (cikis, sure) = grup_cikisini_bekle(&mut grup, Duration::from_secs(60));
    assert_eq!(cikis.code(), Some(1), "{cikis}");
    assert!(
        sure >= Duration::from_secs(38),
        "zorla sonlandırma 40 sn dolmadan yapılmamalı: {sure:?}"
    );
    assert!(
        !sinyal(pidler[1], "-0"),
        "40 sn sonra donmuş işçi sonlandırılmış olmalı"
    );
}

/// K-340: `zee.ayar` güvenlik dalındaki `isci-sayisi`/`saglik-port` grubu
/// CLI bayrağı olmadan başlatır; işçiler yeniden okudukları config'ten
/// yönetici olmaz. Kaynak öncesi açık `--web-isci-sayisi` config'i ezer.
#[cfg(unix)]
#[test]
fn isci_grubu_zee_ayar_ile_baslar_ve_cli_ezer() {
    let gecici = GeciciKlasor::yeni();
    let kapi = bos_kapi();
    let saglik = bos_kapi();
    std::fs::write(
        gecici.0.join("proje.dil"),
        "proje \"ayar-isci\" olsun\nsürüm \"0.1.0\" olsun\nmorfoloji \"zee-tr-1\" olsun\n\
         giriş \"ana.dil\" olsun\nyetkinlikler \"ağ-sunucusu\" listesi olsun\n",
    )
    .expect("proje");
    std::fs::write(
        gecici.0.join("ana.dil"),
        "8080 kapısında sunucu başlat\n\nGET \"/\" adresine istek geldiğinde\n    rol birincil süreç olsun\n    \
         rol ise\n        \"birincil\" yanıtını gönder\n    değilse\n        \"ikincil\" yanıtını gönder\n",
    )
    .expect("kaynak");
    std::fs::write(
        gecici.0.join("zee.ayar"),
        format!(
            "{{\"sema\":\"zee-ayar-1\",\"surum\":1,\"ayarlar\":{{}},\"guvenlik\":{{\"web-modu\":\"proxy\",\
             \"proxy-origin\":\"https://panel.example\",\"worker-port\":{kapi},\"isci-sayisi\":3,\
             \"saglik-port\":{saglik}}}}}"
        ),
    )
    .expect("zee.ayar");
    let baslat = |ek: &[&str]| {
        use std::os::unix::process::CommandExt;
        let ek: Vec<String> = ek.iter().map(|s| s.to_string()).collect();
        // zee.ayar kök yolunda sembolik bağ kabul etmez (macOS /var → /private/var).
        let kok = std::fs::canonicalize(&gecici.0).expect("kanonik kök");
        Sunucu(kopma::dinleyene_dek_baslat(move || {
            let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
            komut
                .arg("çalıştır")
                .args(&ek)
                .arg(&kok)
                .current_dir(&kok)
                .stderr(Stdio::null())
                .process_group(0);
            komut
        }))
    };
    let mut grup = baslat(&[]);
    let govde = saglik_bekle(saglik, |g| g.contains("\"canli\":3"));
    assert!(govde.contains("\"isci\":3"), "{govde}");
    let ilk = surecler(&govde);
    let mut birincil = 0;
    for _ in 0..12 {
        let yanit = istek(kapi, "203.0.113.7", "GET", "/", None, "");
        assert_eq!(durum(&yanit), 200, "{yanit}");
        birincil += usize::from(yanit.ends_with("birincil"));
    }
    assert!(birincil <= 12);
    // İşçi config'i yeniden okuyup kendi grubunu açsaydı ya da yönetici
    // olsaydı aynı portu bağlayamaz, çıkar ve yeniden başlatılırdı.
    std::thread::sleep(Duration::from_millis(500));
    let (kod, govde) = saglik_oku(saglik);
    assert_eq!(kod, 200, "{govde}");
    assert!(govde.contains("\"yeniden_baslatma\":0"), "{govde}");
    assert_eq!(surecler(&govde), ilk);
    assert!(sinyal(grup.0.id(), "-TERM"));
    let (cikis, _) = grup_cikisini_bekle(&mut grup, Duration::from_secs(45));
    assert!(cikis.success(), "{cikis}");
    drop(grup);

    let mut grup = baslat(&["--web-isci-sayisi", "2"]);
    let govde = saglik_bekle(saglik, |g| g.contains("\"canli\":2"));
    assert!(govde.contains("\"isci\":2"), "CLI config'i ezmeli: {govde}");
    assert!(sinyal(grup.0.id(), "-TERM"));
    let (cikis, _) = grup_cikisini_bekle(&mut grup, Duration::from_secs(45));
    assert!(cikis.success(), "{cikis}");
}

#[test]
fn isci_grubu_gecersiz_birlesimleri_ve_platform_siniri_reddedilir() {
    let gecici = GeciciKlasor::yeni();
    let yol = gecici.0.join("ana.dil");
    std::fs::write(&yol, "1 yaz\n").expect("kaynak");
    let kaynak = yol.to_str().expect("UTF-8");
    let mut vakalar: Vec<Vec<&str>> = vec![
        vec!["çalıştır", "--web-isci-sayisi", "2", kaynak],
        vec![
            "çalıştır",
            "--deneysel-web",
            "--web-isci-sayisi",
            "2",
            kaynak,
        ],
        vec![
            "çalıştır",
            "--web-proxy",
            "https://panel.example",
            "--web-isci-sayisi",
            "2",
            kaynak,
        ],
        vec![
            "çalıştır",
            "--web-proxy",
            "https://panel.example",
            "--web-worker-port",
            "18091",
            "--web-isci-sayisi",
            "0",
            kaynak,
        ],
        vec![
            "çalıştır",
            "--web-proxy",
            "https://panel.example",
            "--web-worker-port",
            "18091",
            "--web-saglik-port",
            "18092",
            kaynak,
        ],
        vec!["denetle", "--web-isci-sayisi", "2", kaynak],
    ];
    if cfg!(windows) {
        vakalar.push(vec![
            "çalıştır",
            "--web-proxy",
            "https://panel.example",
            "--web-worker-port",
            "18091",
            "--web-isci-sayisi",
            "2",
            kaynak,
        ]);
    }
    for vaka in vakalar {
        let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
            .args(&vaka)
            .output()
            .expect("dil");
        assert_eq!(cikti.status.code(), Some(2), "{vaka:?}");
        assert!(!cikti.stderr.is_empty(), "{vaka:?}");
    }
}

fn onbellek_basligi(yanit: &str) -> &str {
    yanit
        .lines()
        .find_map(|satir| satir.strip_prefix("Cache-Control: "))
        .expect("Cache-Control başlığı")
        .trim_end()
}

/// K-335/W3: `herkese açık önbelleklenebilir 300 saniye` gerçek worker'da
/// yalnız Set-Cookie'siz 200 GET/HEAD yanıtını `public` yapar. Kayar oturum
/// yenilemesi Set-Cookie ürettiğinde aynı rota `no-store`a düşer; sıradan
/// rota ve 404 her zaman `no-store` kalır.
#[test]
fn onbelleklenebilir_rota_set_cookie_uretince_asla_public_olmaz() {
    let gecici = GeciciKlasor::yeni();
    std::fs::write(
        gecici.0.join("proje.dil"),
        "proje \"vitrin-panel\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
         yetkinlikler \"dosya-okuma\", \"dosya-yazma\", \"ağ-sunucusu\", \"web-oturumu\", \"kriptografi\" listesi olsun\n\
         oturum_ömrü_saniye 28800 olsun\noturum_kayar doğru olsun\n",
    )
    .expect("proje bildirimi");
    let kaynak = format!(
        "{}\nGET \"/vitrin\" adresine istek geldiğinde\n    herkese açık önbelleklenebilir 300 saniye\n    \"vitrin\" yanıtını gönder\n\
         \nHEAD \"/vitrin\" adresine istek geldiğinde\n    herkese açık önbelleklenebilir 300 saniye\n    \"vitrin\" yanıtını gönder\n",
        include_str!("../../projeler/girisli-panel.dil")
    );
    std::fs::write(gecici.0.join("ana.dil"), kaynak).expect("proje kaynağı");
    let kilit = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["kilitle", gecici.0.to_str().expect("UTF-8 yol")])
        .output()
        .expect("dil kilitle");
    assert!(
        kilit.status.success(),
        "{}",
        String::from_utf8_lossy(&kilit.stderr)
    );
    let kapi = bos_kapi();
    let _sunucu = sunucuyu_baslat(&gecici.0, kapi);
    let public = "public, max-age=0, s-maxage=300";

    let anonim = istek(kapi, "203.0.113.9", "GET", "/vitrin", None, "");
    assert_eq!(durum(&anonim), 200, "{anonim}");
    assert!(cerez_satiri(&anonim).is_none(), "{anonim}");
    assert_eq!(onbellek_basligi(&anonim), public);
    assert_eq!(anonim.matches("Cache-Control:").count(), 1);
    let bas = istek(kapi, "203.0.113.9", "HEAD", "/vitrin", None, "");
    assert_eq!(durum(&bas), 200, "{bas}");
    assert_eq!(onbellek_basligi(&bas), public);

    let form = istek(kapi, "203.0.113.9", "GET", "/giris", None, "");
    let anonim_oturum = cerez(&form);
    let giris = istek(
        kapi,
        "203.0.113.9",
        "POST",
        "/giris-yap",
        Some(&anonim_oturum),
        &format!("_csrf={}&parola=zee2026", csrf(&form)),
    );
    assert_eq!(durum(&giris), 303);
    assert_eq!(onbellek_basligi(&giris), "no-store");
    let oturum = cerez(&giris);
    let oturumlu = istek(kapi, "203.0.113.9", "GET", "/vitrin", Some(&oturum), "");
    assert_eq!(durum(&oturumlu), 200, "{oturumlu}");
    assert!(
        cerez_satiri(&oturumlu).is_some(),
        "kayar yenileme Set-Cookie üretmeli"
    );
    assert_eq!(
        onbellek_basligi(&oturumlu),
        "no-store",
        "Set-Cookie'li yanıt public olamaz"
    );
    assert!(!oturumlu.contains("public"), "{oturumlu}");

    let duz = istek(kapi, "203.0.113.9", "GET", "/", None, "");
    assert_eq!(durum(&duz), 200);
    assert_eq!(onbellek_basligi(&duz), "no-store");
    let yok = istek(kapi, "203.0.113.9", "GET", "/vitrin-yok", None, "");
    assert_eq!(durum(&yok), 404);
    assert_eq!(onbellek_basligi(&yok), "no-store");
}

/// K-341: dinlemeyi beklemeden ayrı portlu worker süreci açar (açılış
/// kilidinde bekleyebilir).
fn ayri_sureci_ac(kaynak: &Path, kapi: u16) -> Sunucu {
    Sunucu(
        Command::new(env!("CARGO_BIN_EXE_dil"))
            .args([
                "çalıştır",
                "--web-proxy",
                "https://panel.example",
                "--web-worker-port",
                &kapi.to_string(),
                kaynak.to_str().expect("UTF-8 kaynak yolu"),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("ayrı portlu worker"),
    )
}

fn dinleyene_dek_bekle(kapi: u16, sure: Duration) {
    let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
    let son = Instant::now() + sure;
    while TcpStream::connect_timeout(&adres, Duration::from_millis(100)).is_err() {
        assert!(
            Instant::now() < son,
            "worker {kapi} {sure:?} içinde dinlemedi"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn rol_oku(kapi: u16) -> String {
    let yanit = istek(kapi, "203.0.113.7", "GET", "/rol", None, "");
    assert_eq!(durum(&yanit), 200, "{yanit}");
    yanit
        .rsplit("\r\n\r\n")
        .next()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn ayri_portlu_surecler_tek_birincil_secer_ve_acilisi_bekler() {
    let gecici = GeciciKlasor::yeni();
    let yol = gecici.0.join("ana.dil");
    std::fs::write(
        &yol,
        "sahip birincil süreç olsun\n\
         sahip ise\n    \"acilis.log\" dosyasına \"açılış başladı\" ekle\n    \
         toplam 0 olsun\n    1 den 1000000 e kadar her sayı için\n        toplamı sayıyla artır\n    \
         \"acilis.log\" dosyasına \"açılış bitti\" ekle\n\
         değilse\n    \"acilis.log\" dosyasına \"ikincil\" ekle\n\
         8080 kapısında sunucu başlat\n\n\
         GET \"/rol\" adresine istek geldiğinde\n    rol birincil süreç olsun\n    rol ise\n        \
         \"birincil\" yanıtını gönder\n    değilse\n        \"ikincil\" yanıtını gönder\n",
    )
    .expect("kaynak");
    let gunluk = || std::fs::read_to_string(gecici.0.join("acilis.log")).unwrap_or_default();
    let (kapi1, kapi2, kapi3) = (bos_kapi(), bos_kapi(), bos_kapi());

    // Birincil açılış işini yaparken ikinci süreç açılır: açılış kilidinde
    // bekler, birincil `sunucu başlat`a ulaşmadan kendi açılışını koşmaz.
    let mut bir = ayri_sureci_ac(&yol, kapi1);
    let son = Instant::now() + Duration::from_secs(60);
    while !gunluk().contains("açılış başladı") {
        assert!(Instant::now() < son, "birincil açılışa başlamadı");
        std::thread::sleep(Duration::from_millis(10));
    }
    let iki = ayri_sureci_ac(&yol, kapi2);
    dinleyene_dek_bekle(kapi1, Duration::from_secs(90));
    dinleyene_dek_bekle(kapi2, Duration::from_secs(90));
    assert_eq!(
        gunluk().lines().collect::<Vec<_>>(),
        vec!["açılış başladı", "açılış bitti", "ikincil"],
        "ikinci süreç birincilin açılışı bitmeden karar vermemeli"
    );
    assert_eq!(rol_oku(kapi1), "birincil");
    assert_eq!(rol_oku(kapi2), "ikincil");

    // Birincil ölür; diğer süreç yaşarken aynı kapıda açılan süreç açılış
    // işini yeniden koşturmaz (K-337'nin grup içi kuralı, ayrı portlarda).
    let _ = bir.0.kill();
    let _ = bir.0.wait();
    drop(bir);
    let yeniden = ayri_sureci_ac(&yol, kapi1);
    dinleyene_dek_bekle(kapi1, Duration::from_secs(90));
    assert_eq!(
        rol_oku(kapi1),
        "ikincil",
        "yeniden açılan süreç birincil olamaz"
    );
    assert_eq!(gunluk().matches("açılış başladı").count(), 1);

    // Kökte canlı süreç kalmayınca yeni süreç yeniden birincildir.
    drop((iki, yeniden));
    // Çekirdek ölen sürecin kilidini bırakır; Windows bunu gecikmeli
    // yapabildiğinden yeni süreç, kilit gerçekten boşalınca açılır.
    let canli = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(gecici.0.join(".zee/birincil-canli.kilit"))
        .expect("canlı kilit dosyası");
    let son = Instant::now() + Duration::from_secs(30);
    while canli.try_lock().is_err() {
        assert!(
            Instant::now() < son,
            "ölen süreçlerin canlı kilidi bırakılmadı"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    drop(canli);
    let uc = ayri_sureci_ac(&yol, kapi3);
    dinleyene_dek_bekle(kapi3, Duration::from_secs(90));
    assert_eq!(rol_oku(kapi3), "birincil");
    assert_eq!(gunluk().matches("açılış başladı").count(), 2);
    drop(uc);
}
