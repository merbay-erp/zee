//! K-332: oturumsuz istemcinin reddedilen binary yüklemelerinin gerçek native
//! `dil` worker'ında `.zee/yuklemeler` altında `.parca` bırakmadığını sınar.
//! Proje Cargo hedef klasöründe geçici bir kopyadır (worker kökü kaynağın
//! klasörüdür, depo ağacına yazılmaz). ToplayanIo HTTP gövdesini diske
//! akıtmadığı için toplu replay bu kipi atlar.
#![allow(dead_code)]

use super::native_web_kopma_fixture::{bos_kapi, dinlemeyi_bekle, worker_baslat, PROXY};
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

const GOVDE_BAYTI: usize = 64 * 1024;

struct Temizle(PathBuf);

impl Drop for Temizle {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn ortak_basliklar() -> String {
    "Host: panel.example\r\nForwarded: for=198.51.100.7;proto=https;host=panel.example\r\nConnection: close\r\n".into()
}

/// Ham isteği yollar; `kisa` ise gövdenin ardından yazma yarısını kapatır.
/// Okuma hatası (RST) yanıtın parçası olarak işaretlenir.
fn istek(kapi: u16, basliklar: &str, govde: &[u8], kisa: bool) -> String {
    let adres = SocketAddr::from(([127, 0, 0, 1], kapi));
    let mut akis = TcpStream::connect_timeout(&adres, Duration::from_secs(2)).expect("bağlantı");
    akis.set_read_timeout(Some(Duration::from_secs(10)))
        .expect("okuma timeout");
    akis.write_all(basliklar.as_bytes()).expect("başlıklar");
    let _ = akis.write_all(govde);
    if kisa {
        let _ = akis.shutdown(Shutdown::Write);
    }
    let mut yanit = Vec::new();
    let _ = akis.read_to_end(&mut yanit);
    String::from_utf8_lossy(&yanit).into_owned()
}

fn durum(yanit: &str) -> &str {
    yanit
        .strip_prefix("HTTP/1.1 ")
        .and_then(|kalan| kalan.split_whitespace().next())
        .unwrap_or("yanıtsız")
}

fn yukleme(kapi: u16, yol: &str, ek: &str, govde: &[u8], uzunluk: usize, kisa: bool) -> String {
    let basliklar = format!(
        "POST {yol} HTTP/1.1\r\n{}Origin: {PROXY}\r\nContent-Type: application/octet-stream\r\nContent-Length: {uzunluk}\r\n{ek}\r\n",
        ortak_basliklar()
    );
    let yanit = istek(kapi, &basliklar, govde, kisa);
    durum(&yanit).to_string()
}

fn parca_sayisi(proje: &Path) -> usize {
    std::fs::read_dir(proje.join(".zee/yuklemeler"))
        .map(|dizin| {
            dizin
                .filter_map(Result::ok)
                .filter(|giris| {
                    giris
                        .path()
                        .extension()
                        .is_some_and(|uzanti| uzanti == "parca")
                })
                .count()
        })
        .unwrap_or(0)
}

/// K-332 eki fixture işareti: worker açılmadan önce yetim/canlı/yabancı
/// dosyalar dikilir, açılış süpürmesinden sonra kalanlar sayılır.
const YETIM_ISARETI: &str = "# yetim-parca-supurmesi";

/// Süreç ölümü taklidi: worker'dan önce `.zee/yuklemeler` altına 16 dakikalık
/// runtime adlı yetim, 1 dakikalık canlı (başka worker'ın akışı) ve 16
/// dakikalık ürün adlı dosya bırakılır. Çıktı her dosyanın kalıp kalmadığıdır.
fn yetim_calistir(proje: &Path, ana: &Path) -> (i64, Vec<String>) {
    let dizin = proje.join(".zee/yuklemeler");
    std::fs::create_dir_all(&dizin).expect("yükleme dizini");
    let simdi = std::time::SystemTime::now();
    let dosyalar = [
        ("yetim", format!("{}.parca", "a".repeat(64)), 16 * 60),
        ("canli", format!("{}.parca", "b".repeat(64)), 60),
        ("urun", "eski-medya.parca".to_string(), 16 * 60),
    ];
    for (_, ad, yas) in &dosyalar {
        let dosya = std::fs::File::create(dizin.join(ad)).expect("dikilen dosya");
        dosya
            .set_modified(simdi - Duration::from_secs(*yas))
            .expect("mtime");
    }
    let kapi = bos_kapi();
    let mut worker = worker_baslat(ana, kapi, Stdio::null);
    dinlemeyi_bekle(kapi);
    // Açılış süpürmesi `sunucu kur` içinde bind'dan sonra koşar; ilk yanıt onu izler.
    let hazir = istek(
        kapi,
        &format!("GET /form HTTP/1.1\r\n{}\r\n", ortak_basliklar()),
        b"",
        false,
    );
    let calisiyor = worker.try_wait().expect("worker durumu").is_none();
    let _ = worker.kill();
    let _ = worker.wait();
    let mut cikti = vec![format!("{} hazır", durum(&hazir))];
    for (etiket, ad, _) in &dosyalar {
        let kaldi = dizin.join(ad).exists();
        cikti.push(format!(
            "{etiket} {}",
            if kaldi { "kaldı" } else { "silindi" }
        ));
    }
    (if calisiyor { 0 } else { 1 }, cikti)
}

/// Çıktı: her senaryonun `durum senaryo` satırı, ardından kalan `.parca`
/// sayısı ve taşınan dosyanın bayt sayısı. Exit, worker hâlâ çalışıyorsa 0.
pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    let ek = format!(
        "zee-k332-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    );
    let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(ek);
    let _temizle = Temizle(proje.clone());
    std::fs::create_dir_all(proje.join("kalici")).expect("proje klasörü");
    let ana = proje.join("ana.dil");
    std::fs::copy(kaynak, &ana).expect("kaynak kopyası");
    let yetim_vakasi = std::fs::read_to_string(kaynak)
        .expect("kaynak okunmalı")
        .lines()
        .any(|satir| satir == YETIM_ISARETI);
    if yetim_vakasi {
        return yetim_calistir(&proje, &ana);
    }
    let kapi = bos_kapi();
    let mut worker = worker_baslat(&ana, kapi, Stdio::null);
    dinlemeyi_bekle(kapi);

    let form = istek(
        kapi,
        &format!("GET /form HTTP/1.1\r\n{}\r\n", ortak_basliklar()),
        b"",
        false,
    );
    let cerez = form
        .lines()
        .find_map(|satir| satir.strip_prefix("Set-Cookie: "))
        .and_then(|deger| deger.split(';').next())
        .expect("oturum çerezi")
        .to_string();
    let csrf = form
        .split("\r\n\r\n")
        .nth(1)
        .expect("CSRF gövdesi")
        .to_string();
    let gecerli = format!("X-Zee-CSRF: {csrf}\r\nCookie: {cerez}\r\n");
    let govde: Vec<u8> = (0..GOVDE_BAYTI).map(|i| (i % 251) as u8).collect();

    let mut cikti = Vec::new();
    let mut senaryo = |ad: &str, durum: String| cikti.push(format!("{durum} {ad}"));
    senaryo(
        "csrf-yok",
        yukleme(kapi, "/yukle", "", b"", GOVDE_BAYTI, true),
    );
    senaryo(
        "sahte-oturum",
        yukleme(
            kapi,
            "/yukle",
            "X-Zee-CSRF: sahte\r\nCookie: __Host-zee-oturum=sahte\r\n",
            &govde,
            GOVDE_BAYTI,
            false,
        ),
    );
    senaryo(
        "rota-yok",
        yukleme(kapi, "/yok", &gecerli, &govde, GOVDE_BAYTI, false),
    );
    senaryo(
        "kisa-govde",
        yukleme(kapi, "/yukle", &gecerli, &govde[..1000], GOVDE_BAYTI, true),
    );
    senaryo(
        "tuketmeyen-rota",
        yukleme(kapi, "/yukle", &gecerli, &govde, GOVDE_BAYTI, false),
    );
    senaryo(
        "tasiyan-rota",
        yukleme(kapi, "/tasi", &gecerli, &govde, GOVDE_BAYTI, false),
    );

    let calisiyor = worker.try_wait().expect("worker durumu").is_none();
    let _ = worker.kill();
    let _ = worker.wait();
    cikti.push(format!("parça {}", parca_sayisi(&proje)));
    let tasinan = std::fs::read(proje.join("kalici/yukleme.bin")).unwrap_or_default();
    assert!(
        tasinan.is_empty() || tasinan == govde,
        "taşınan dosya gövdeyle bayt bayt aynı olmalı"
    );
    cikti.push(format!("taşınan {}", tasinan.len()));
    (if calisiyor { 0 } else { 1 }, cikti)
}
