//! K-236/K-237: manifest fixture'ı gerçek SMTP adaptörü üzerinden loopback'te yürütür.
use dil::faz::BaglanmisProgram;
use dil::tani::Tani;
use dil::yorumlayici::{calistir_baglanmis_io_kodla, GirdiCikti, ToplayanIo};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const GIZLI_HATA: &str = "E-posta gönderilemedi; ayar geçersiz veya gönderim sonucu belirsiz; ayrıntılar gizlendi; otomatik tekrar yok";

#[derive(Clone, Copy)]
enum SmtpVaka {
    YanitSiniri,
    AuthEcho,
}

pub(super) fn calistir(
    program: &BaglanmisProgram,
    kaynak: &str,
) -> (Result<i64, Tani>, Vec<String>) {
    let vakalar: Vec<_> = kaynak
        .lines()
        .filter_map(|satir| satir.strip_prefix("# smtp-vaka: "))
        .collect();
    let vaka = match vakalar.as_slice() {
        ["yanit-siniri"] => SmtpVaka::YanitSiniri,
        ["auth-echo"] => SmtpVaka::AuthEcho,
        _ => panic!("SMTP fixture tek exact yanit-siniri/auth-echo başlığı taşımalı"),
    };
    let dinleyici = TcpListener::bind("127.0.0.1:0").unwrap();
    let adres = dinleyici.local_addr().unwrap();
    dinleyici.set_nonblocking(true).unwrap();
    let (bitir, bitis) = mpsc::channel();
    let sunucu = std::thread::spawn(move || {
        let son = Instant::now() + Duration::from_secs(5);
        let mut akis = loop {
            match dinleyici.accept() {
                Ok((akis, _)) => break akis,
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
        akis.set_nonblocking(false).unwrap();
        akis.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        akis.set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        match vaka {
            SmtpVaka::YanitSiniri => {
                let mut veri = vec![b'x'; 16 * 1024 + 1];
                veri[..4].copy_from_slice(b"220 ");
                akis.write_all(&veri).unwrap();
                // LF/EOF gelmez. Her 50 ms yeni bayt read timeout'ını canlı
                // tutar; yalnız gerçek çağrı döndüğünde fixture kapanır.
                // Eski sınırsız helper fixture son tarihinde test'i kırar.
                let son = Instant::now() + Duration::from_secs(5);
                loop {
                    match bitis.recv_timeout(Duration::from_millis(50)) {
                        Ok(()) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            assert!(
                                Instant::now() < son,
                                "SMTP sınır fixture'ı LF/EOF olmadan dönmedi"
                            );
                            akis.write_all(b"x").unwrap();
                        }
                        Err(hata) => panic!("SMTP fixture kontrol kanalı: {hata}"),
                    }
                }
            }
            SmtpVaka::AuthEcho => {
                akis.write_all(b"220 fixture\r\n").unwrap();
                let mut okuyucu = BufReader::new(akis);
                let mut satir = String::new();
                okuyucu.read_line(&mut satir).unwrap();
                assert_eq!(satir, "EHLO zee.invalid\r\n");
                okuyucu.get_mut().write_all(b"250 fixture\r\n").unwrap();
                satir.clear();
                okuyucu.read_line(&mut satir).unwrap();
                let payload = satir
                    .strip_prefix("AUTH PLAIN ")
                    .and_then(|s| s.strip_suffix("\r\n"))
                    .expect("tek AUTH PLAIN");
                assert!(!payload.is_empty());
                let yanit = format!("535 denied {payload}\r\n");
                okuyucu.get_mut().write_all(yanit.as_bytes()).unwrap();
                let mut sonra = [0u8; 1];
                assert_eq!(
                    okuyucu.read(&mut sonra).unwrap(),
                    0,
                    "AUTH/mail/data tekrar yok"
                );
            }
        }
    });
    let mut io = SmtpIo {
        ic: ToplayanIo::yeni(Vec::new()),
        ayar: dil::eposta::EpostaAyari::ayristir(&format!(
            "smtp://sentinel:SENTINEL_PASSWORD@{adres}?from=sender@example.invalid"
        ))
        .unwrap(),
        vaka,
        gonderim: 0,
    };
    let sonuc = calistir_baglanmis_io_kodla(program, &mut io);
    bitir.send(()).ok();
    sunucu.join().expect("SMTP fixture iş parçacığı");
    assert_eq!(
        io.gonderim, 1,
        "fixture yalnız tek gerçek gönderim çağrısını yürütür"
    );
    (sonuc, io.ic.cikti)
}

struct SmtpIo {
    ic: ToplayanIo,
    ayar: dil::eposta::EpostaAyari,
    vaka: SmtpVaka,
    gonderim: usize,
}

impl GirdiCikti for SmtpIo {
    fn eposta_gonder(
        &mut self,
        baglanti: Option<&str>,
        alici: &str,
        konu: &str,
        govde: &str,
    ) -> Result<i64, String> {
        assert!(baglanti.is_none(), "fixture dış SMTP URL'sini çalıştıramaz");
        self.gonderim += 1;
        assert_eq!(self.gonderim, 1, "SMTP otomatik tekrar yok");
        let sonuc = dil::eposta::gonder(
            &self.ayar,
            &dil::eposta::Ileti {
                alici,
                konu,
                govde,
                kimlik: "fixture",
                tarih: "Tue, 14 Oct 2026 12:00:00 +0000",
            },
            Duration::from_secs(2),
            &|ip| {
                if ip.is_loopback() {
                    Ok(())
                } else {
                    Err("fixture loopback dışını reddeder".into())
                }
            },
        );
        match self.vaka {
            SmtpVaka::YanitSiniri => assert_eq!(
                sonuc.as_ref().unwrap_err(),
                GIZLI_HATA,
                "native sınır hatası K236 güvenli dış metnini korumalı"
            ),
            SmtpVaka::AuthEcho => assert_eq!(
                sonuc.as_ref().unwrap_err(),
                GIZLI_HATA,
                "native AUTH535 echo tanısı sır taşımamalı"
            ),
        }
        // Gerçek adaptör sonucu; runtime'ın ayrı maskelemesi .dil çıktısında sınanır.
        sonuc.map(|()| 1)
    }
    fn yazdir(&mut self, s: String) {
        self.ic.yazdir(s);
    }
    fn sor(&mut self, s: &str) -> Option<String> {
        self.ic.sor(s)
    }
    fn rastgele(&mut self, a: i64, b: i64) -> i64 {
        self.ic.rastgele(a, b)
    }
    fn dosya_oku(&mut self, s: &str) -> Result<String, String> {
        self.ic.dosya_oku(s)
    }
    fn dosya_yaz(&mut self, p: &str, s: &str, e: bool) -> Result<(), String> {
        self.ic.dosya_yaz(p, s, e)
    }
    fn simdi(&mut self) -> (i64, u32, u32, u32, u32) {
        self.ic.simdi()
    }
    fn argumanlar(&mut self) -> Vec<String> {
        self.ic.argumanlar()
    }
    fn http_getir(&mut self, s: &str, t: Option<i64>) -> Result<(i64, String), String> {
        self.ic.http_getir(s, t)
    }
    fn sunucu_kur(&mut self, p: i64) -> Result<(), String> {
        self.ic.sunucu_kur(p)
    }
    fn istek_al(&mut self) -> Option<String> {
        self.ic.istek_al()
    }
    fn yanit_gonder(&mut self, s: &str) {
        self.ic.yanit_gonder(s);
    }
    fn yonlendir_gonder(&mut self, s: &str) -> Result<(), String> {
        self.ic.yonlendir_gonder(s)
    }
    fn cerez_yaz(&mut self, a: &str, d: &str) -> Result<(), String> {
        self.ic.cerez_yaz(a, d)
    }
    fn cerez_sil(&mut self, s: &str) -> Result<(), String> {
        self.ic.cerez_sil(s)
    }
    fn sensor_acik_mi(&mut self, s: &str) -> bool {
        self.ic.sensor_acik_mi(s)
    }
    fn isik_ayarla(&mut self, s: &str, b: bool) {
        self.ic.isik_ayarla(s, b);
    }
    fn bekle_ms(&mut self, ms: i64) {
        self.ic.bekle_ms(ms);
    }
    fn an_ms(&mut self) -> i64 {
        self.ic.an_ms()
    }
}
