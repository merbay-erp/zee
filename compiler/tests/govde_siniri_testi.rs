//! K-391 (ADR-168, ZM-F036/F037): web isteği boyut sınırları gerçek `dil`
//! web sürecinde. Multipart metin alanları sıradan form alanıdır (Türkçe
//! metin urlencoded gibi şişmez), dosyasız multipart form kabul edilir;
//! `gövde_aşımı_rotası` gövde boyutu 413'ünü salt okuma GET rotasıyla
//! yanıtlar. Bildirimsiz projede 64 KiB urlencoded sınırı bayt bayt aynıdır.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[path = "semantic_regresyon/native_web_kopma.rs"]
mod kopma;

static SIRA: AtomicU64 = AtomicU64::new(0);
const SINIR: &str = "----zeeK391Sinir";

const ANA: &str = r#"8080 kapısında sunucu başlat

GET "/form" adresine istek geldiğinde
    csrf csrf belirteci olsun
    csrf yanıtını gönder

POST "/gonder" adresine istek geldiğinde
    herkese açık
    "govde" alanı gerekli
    metin isteğin "govde" değeri olsun
    metin yanıtını gönder

GET "/cok-buyuk" adresine istek geldiğinde
    yol isteğin "govde_asimi_yolu" değeri olsun
    sinir isteğin "govde_asimi_siniri" değeri olsun
    bayt isteğin "govde_asimi_bayti" değeri olsun
    sayfa "<p>İleti çok büyük: " ile yol ile " " ile sinir ile " " ile bayt ile "</p>" olsun
    sayfa yanıtını gönder
"#;

struct Proje(PathBuf);

impl Proje {
    fn yeni(ek: &str) -> Self {
        let yol = std::env::temp_dir().join(format!(
            "zee-k391-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&yol).expect("proje klasörü");
        std::fs::write(
            yol.join("proje.dil"),
            format!(
                "proje \"boyut\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
                 yetkinlikler \"ağ-sunucusu\", \"web-oturumu\" listesi olsun\n\
                 ağ_hedefleri boş liste olsun\n{ek}"
            ),
        )
        .expect("proje.dil");
        std::fs::write(yol.join("ana.dil"), ANA).expect("ana.dil");
        Self(yol)
    }
}

impl Drop for Proje {
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

fn baslat(proje: &Proje) -> (Sunucu, u16) {
    let kapi = kopma::bos_kapi();
    (
        Sunucu(kopma::worker_baslat(&proje.0, kapi, Stdio::null)),
        kapi,
    )
}

fn baglan(kapi: u16) -> TcpStream {
    let akis = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], kapi)),
        Duration::from_secs(5),
    )
    .expect("bağlantı");
    akis.set_read_timeout(Some(Duration::from_secs(30)))
        .expect("okuma süresi");
    akis
}

struct Yanit {
    durum: u16,
    basliklar: String,
    govde: String,
}

fn yanit_oku(mut akis: TcpStream) -> Yanit {
    let mut ham = Vec::new();
    let _ = akis.read_to_end(&mut ham);
    let metin = String::from_utf8_lossy(&ham).into_owned();
    let (basliklar, govde) = metin.split_once("\r\n\r\n").unwrap_or((&metin, ""));
    Yanit {
        durum: basliklar
            .split_whitespace()
            .nth(1)
            .and_then(|d| d.parse().ok())
            .unwrap_or(0),
        basliklar: basliklar.to_string(),
        govde: govde.to_string(),
    }
}

/// (oturum çerezi, CSRF belirteci)
fn form_al(kapi: u16) -> (String, String) {
    let mut akis = baglan(kapi);
    akis.write_all(
        b"GET /form HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.9;proto=https;host=panel.example\r\nConnection: close\r\n\r\n",
    )
    .expect("istek");
    let yanit = yanit_oku(akis);
    let cerez = yanit
        .basliklar
        .lines()
        .find_map(|s| s.strip_prefix("Set-Cookie: __Host-zee-oturum="))
        .and_then(|k| k.split(';').next())
        .expect("oturum çerezi")
        .to_string();
    (cerez, yanit.govde)
}

fn gonder(kapi: u16, cerez: &str, tur: &str, govde: &[u8]) -> Yanit {
    let mut akis = baglan(kapi);
    let mut ham = format!(
        "POST /gonder HTTP/1.1\r\nHost: panel.example\r\nForwarded: for=203.0.113.9;proto=https;host=panel.example\r\nOrigin: https://panel.example\r\nConnection: close\r\nContent-Type: {tur}\r\nContent-Length: {}\r\nCookie: __Host-zee-oturum={cerez}\r\n\r\n",
        govde.len()
    )
    .into_bytes();
    ham.extend_from_slice(govde);
    let _ = akis.write_all(&ham);
    yanit_oku(akis)
}

fn mp_tur() -> String {
    format!("multipart/form-data; boundary={SINIR}")
}

fn alan(ad: &str, deger: &[u8]) -> Vec<u8> {
    let mut p =
        format!("--{SINIR}\r\nContent-Disposition: form-data; name=\"{ad}\"\r\n\r\n").into_bytes();
    p.extend_from_slice(deger);
    p.extend_from_slice(b"\r\n");
    p
}

fn mp_form(alanlar: &[(&str, &[u8])]) -> Vec<u8> {
    let mut g: Vec<u8> = alanlar.iter().flat_map(|(a, d)| alan(a, d)).collect();
    g.extend(format!("--{SINIR}--\r\n").into_bytes());
    g
}

fn form_kodla(metin: &str) -> String {
    metin
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                char::from(b).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

/// 25 000 harflik Türkçe-yoğun ileti: UTF-8 ~33 KB, urlencoded ~100 KB.
fn turkce_ileti() -> String {
    "Işığın gölgesi, çiçeğin üşüyen yüzünde. "
        .chars()
        .cycle()
        .take(25_000)
        .collect()
}

#[test]
fn multipart_metin_alani_turkce_yogun_ileti_kabul_urlencoded_varsayilan_ayni() {
    let proje = Proje::yeni("çok_parçalı_yükleme doğru olsun\n");
    let (_sunucu, kapi) = baslat(&proje);
    let (cerez, csrf) = form_al(kapi);
    let ileti = turkce_ileti();
    assert!(ileti.len() < 64 * 1024, "{}", ileti.len());

    // Olumlu: dosyasız multipart form, metin alanı sıradan form alanıdır.
    let g = mp_form(&[("_csrf", csrf.as_bytes()), ("govde", ileti.as_bytes())]);
    let yanit = gonder(kapi, &cerez, &mp_tur(), &g);
    assert!(
        yanit.basliklar.starts_with("HTTP/1.1 200 OK\r\n"),
        "{}",
        yanit.basliklar
    );
    assert_eq!(yanit.govde, ileti);

    // Dosyasız formda CSRF yine olağan rota denetimindedir.
    let g = mp_form(&[("_csrf", b"sahte"), ("govde", b"x")]);
    assert_eq!(gonder(kapi, &cerez, &mp_tur(), &g).durum, 403);

    // Olumsuz: aynı ileti urlencoded olarak 64 KiB varsayılanını aşar (413,
    // aşım rotası bildirimsiz: ham ileti); sınır ve iletisi değişmedi.
    let form = format!("_csrf={}&govde={}", form_kodla(&csrf), form_kodla(&ileti));
    assert!(form.len() > 64 * 1024);
    let yanit = gonder(
        kapi,
        &cerez,
        "application/x-www-form-urlencoded",
        form.as_bytes(),
    );
    assert_eq!(yanit.durum, 413);
    assert!(yanit.govde.contains("64 KiB"), "{}", yanit.govde);

    // Tam 64 KiB urlencoded gövde hâlâ kabul edilir.
    let on = format!("_csrf={}&govde=", form_kodla(&csrf));
    let tam = format!("{on}{}", "a".repeat(64 * 1024 - on.len()));
    let yanit = gonder(
        kapi,
        &cerez,
        "application/x-www-form-urlencoded",
        tam.as_bytes(),
    );
    assert_eq!(yanit.durum, 200, "{}", yanit.basliklar);

    // Multipart metin alanları toplamı form bütçesini (64 KiB) aşarsa 413.
    let yarim = vec![b'x'; 33 * 1024];
    let g = mp_form(&[
        ("_csrf", csrf.as_bytes()),
        ("govde", &yarim),
        ("ek", &yarim),
    ]);
    assert_eq!(gonder(kapi, &cerez, &mp_tur(), &g).durum, 413);
}

#[test]
fn govde_asimi_rotasi_413u_salt_okuma_sayfasiyla_yanitlar() {
    let proje =
        Proje::yeni("çok_parçalı_yükleme doğru olsun\ngövde_aşımı_rotası \"/cok-buyuk\" olsun\n");
    let (_sunucu, kapi) = baslat(&proje);
    let (cerez, csrf) = form_al(kapi);
    let ileti = turkce_ileti();

    // Bildirilen Content-Length zarfı aşar: gövde okunmadan aşım sayfası.
    let form = format!("_csrf={}&govde={}", form_kodla(&csrf), form_kodla(&ileti));
    let yanit = gonder(
        kapi,
        &cerez,
        "application/x-www-form-urlencoded",
        form.as_bytes(),
    );
    assert!(
        yanit
            .basliklar
            .starts_with("HTTP/1.1 413 Payload Too Large\r\n"),
        "{}",
        yanit.basliklar
    );
    assert!(
        yanit.basliklar.contains("Content-Type: text/html"),
        "{}",
        yanit.basliklar
    );
    assert!(
        yanit.basliklar.contains("Cache-Control: no-store"),
        "{}",
        yanit.basliklar
    );
    assert_eq!(
        yanit.govde,
        format!("<p>İleti çok büyük: /gonder 65536 {}</p>", form.len())
    );

    // Multipart metin bütçesi akış sırasında aşılır: yine aşım sayfası.
    let buyuk = vec![b'y'; 70 * 1024];
    let g = mp_form(&[("_csrf", csrf.as_bytes()), ("govde", &buyuk)]);
    let yanit = gonder(kapi, &cerez, &mp_tur(), &g);
    assert_eq!(yanit.durum, 413, "{}", yanit.basliklar);
    assert_eq!(
        yanit.govde,
        format!("<p>İleti çok büyük: /gonder 65536 {}</p>", g.len())
    );

    // Sınır içindeki istek ve aynı işçinin sonraki isteği etkilenmez.
    let g = mp_form(&[("_csrf", csrf.as_bytes()), ("govde", ileti.as_bytes())]);
    let yanit = gonder(kapi, &cerez, &mp_tur(), &g);
    assert_eq!(yanit.durum, 200);
    assert_eq!(yanit.govde, ileti);
}

#[test]
fn govde_asimi_rotasi_200_vermezse_duz_413_doner() {
    let proje = Proje::yeni("gövde_aşımı_rotası \"/yok\" olsun\n");
    let (_sunucu, kapi) = baslat(&proje);
    let (cerez, _) = form_al(kapi);
    let govde = vec![b'a'; 70 * 1024];
    let yanit = gonder(kapi, &cerez, "application/x-www-form-urlencoded", &govde);
    assert_eq!(yanit.durum, 413, "{}", yanit.basliklar);
    assert!(yanit.govde.contains("64 KiB"), "{}", yanit.govde);
    assert!(
        !yanit.govde.contains("aranan sayfa yok"),
        "404 metni sızmamalı"
    );
}
