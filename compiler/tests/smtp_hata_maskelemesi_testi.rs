//! K-236/GB-060: actual SMTP AUTH echo ve runtime/CLI hata maskesi.
use dil::eposta::{EpostaAyari, Ileti};
use dil::yorumlayici::{calistir_io, ToplayanIo};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

const MASK: &str = "E-posta gönderilemedi; ayar geçersiz veya gönderim sonucu belirsiz; ayrıntılar gizlendi; otomatik tekrar yok";
const REGRESSION: &str = include_str!("../../regression/security/smtp-auth-echo-sir.dil");

fn echo_server(l: TcpListener) -> std::thread::JoinHandle<String> {
    l.set_nonblocking(true).unwrap();
    std::thread::spawn(move || {
        let son = Instant::now() + Duration::from_secs(5);
        let s = loop {
            match l.accept() {
                Ok((s, _)) => break s,
                Err(hata) if hata.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < son,
                        "SMTP fixture bağlantı beklerken bitti"
                    );
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(hata) => panic!("SMTP fixture accept: {hata}"),
            }
        };
        // macOS accepted socket listener'ın nonblocking kipini devralabilir.
        s.set_nonblocking(false).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        s.set_write_timeout(Some(Duration::from_secs(3))).unwrap();
        let mut io = BufReader::new(s);
        io.get_mut().write_all(b"220 fixture\r\n").unwrap();
        let mut line = String::new();
        io.read_line(&mut line).unwrap();
        assert!(line.starts_with("EHLO "));
        io.get_mut().write_all(b"250 fixture\r\n").unwrap();
        line.clear();
        io.read_line(&mut line).unwrap();
        let auth = line.trim().strip_prefix("AUTH PLAIN ").unwrap().to_string();
        io.get_mut()
            .write_all(format!("535 denied {auth}\r\n").as_bytes())
            .unwrap();
        line.clear();
        assert_eq!(
            io.read_line(&mut line).unwrap(),
            0,
            "AUTH reddi sonrası retry/mail/data yok"
        );
        auth
    })
}

struct GeciciProje(std::path::PathBuf);

impl GeciciProje {
    fn yeni() -> Self {
        let kimlik = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let kok =
            std::env::temp_dir().join(format!("zee-k236-auth-{}-{kimlik}", std::process::id()));
        std::fs::create_dir(&kok).unwrap();
        Self(kok)
    }
}

impl Drop for GeciciProje {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn url(kapi: u16) -> String {
    format!("smtp://sentinel:SENTINEL_PASSWORD@127.0.0.1:{kapi}?from=sender@example.invalid")
}

#[test]
fn native_legacy_auth535_echo_sirri_ve_base64_credentiali_gizler() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let kapi = l.local_addr().unwrap().port();
    let server = echo_server(l);
    let ayar = EpostaAyari::ayristir(&url(kapi)).unwrap();
    let hata = dil::eposta::gonder(
        &ayar,
        &Ileti {
            alici: "ali@ornek.invalid",
            konu: "Konu",
            govde: "Gövde",
            kimlik: "fixture123",
            tarih: "Wed, 14 Oct 2026 11:00:00 +0000",
        },
        Duration::from_secs(2),
        &|_| Ok(()),
    )
    .unwrap_err();
    let auth = server.join().unwrap();
    assert_eq!(hata, MASK);
    for gizli in [
        "SENTINEL_PASSWORD",
        "sentinel",
        auth.as_str(),
        "535",
        "127.0.0.1",
    ] {
        assert!(!hata.contains(gizli));
    }
}

#[test]
fn runtime_explicit_url_ayristirma_hatasi_c029_ile_sirsizdir() {
    let baglanti="smtp://sentinel:SENTINEL_PASSWORD@localhost:2525?from=sender@example.invalid&SENTINEL_QUERY=1";
    assert!(EpostaAyari::ayristir(baglanti)
        .unwrap_err()
        .contains("SENTINEL_QUERY"));
    let src=format!("sonuc \"ali@ornek.invalid\" \"Konu\" \"Gövde\" ile \"{baglanti}\" üzerinden ileti göndermeyi dene olsun\nsonuc başarısızsa\n    hata sonucun hatası olsun\n    hatanın kodu yaz\n    hatanın mesajı yaz\n");
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&dil::kaynagi_derle(&src).unwrap(), &mut io).unwrap();
    assert_eq!(io.cikti, ["C029", MASK]);
    assert!(io.eposta_kutusu.is_empty());
}

#[test]
fn cli_legacy_auth_echo_regression_sonuc_metni_sirsizdir() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let kapi = l.local_addr().unwrap().port();
    let server = echo_server(l);
    let gecici = GeciciProje::yeni();
    let kok = &gecici.0;
    std::fs::write(kok.join("ana.dil"), REGRESSION).unwrap();
    std::fs::write(kok.join("proje.dil"),format!("proje \"smtp-auth-fixture\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ\", \"yerel-ağ\" listesi olsun\nağ_hedefleri \"smtp://127.0.0.1:{kapi}\" listesi olsun\ne_posta_bağlantı_değişkeni \"ZEE_AUTH_TEST_SMTP\" olsun\n")).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
        .current_dir(kok)
        .args(["çalıştır", "."])
        .env("ZEE_AUTH_TEST_SMTP", url(kapi))
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(text.trim(), MASK);
    assert!(out.stderr.is_empty());
    let auth = server.join().unwrap();
    assert!(!text.contains(&auth));
    assert!(!text.contains("SENTINEL"));
}
