//! K-378 (RFC-0041 katman C, ADR-163): proje bildirimli SMTP ek türü kaydı
//! gerçek `dil` sürecinde. Bildirimsiz proje dört türde kalır; bildirim
//! kümeyi tam belirler; imza/UTF-8 uyuşmazlığı ve yerel ret ağa çıkmadan,
//! maskeli SMTP hatasından ayrı görünür. SMTP fixture hermetiktir.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static SIRA: AtomicU64 = AtomicU64::new(0);

const ANA: &str = r#"alici "recipient@example.invalid" olsun
konu "Ekli belge" olsun
duz "Belge ektedir." olsun
html "" olsun
yanit "" olsun
ekler EKLER olsun
baglanti "" olsun
sonuc alici konu duz html yanit ekler baglanti ile ekli ileti göndermeyi dene olsun
sonuc başarılıysa
    sonucun değeri yaz
değilse
    sonucun hatası yaz
"#;

/// Geçerli küçük bir OOXML/zip başlangıcı (yerel dosya başlığı imzası).
const DOCX: &[u8] = b"PK\x03\x04\x14\x00\x06\x00word/document.xml";

struct Proje(PathBuf);

impl Proje {
    fn yeni(kapi: u16, turler: Option<&str>, ekler: &str) -> Self {
        let yol = std::env::temp_dir().join(format!(
            "zee-k378-smtp-ek-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&yol).expect("proje");
        std::fs::write(yol.join("ana.dil"), ANA.replace("EKLER", ekler)).expect("ana.dil");
        std::fs::write(
            yol.join("proje.dil"),
            format!(
                "proje \"ek-turu\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
                 yetkinlikler \"ağ\", \"yerel-ağ\", \"dosya-okuma\" listesi olsun\n\
                 ağ_hedefleri \"smtp://127.0.0.1:{kapi}\" listesi olsun\n\
                 e_posta_bağlantı_değişkeni \"ZEE_K378_SMTP\" olsun\n{}",
                turler
                    .map(|t| format!("e_posta_ek_türleri {t} listesi olsun\n"))
                    .unwrap_or_default()
            ),
        )
        .expect("proje.dil");
        Self(yol)
    }

    fn calistir(&self, smtp: Option<&str>) -> Output {
        let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
        komut
            .current_dir(&self.0)
            .args(["çalıştır", "."])
            .env_remove("ZEE_K378_SMTP");
        if let Some(url) = smtp {
            komut.env("ZEE_K378_SMTP", url);
        }
        komut.output().expect("dil")
    }
}

impl Drop for Proje {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn url(kapi: u16) -> String {
    format!("smtp://sentinel:SENTINEL_PASSWORD@127.0.0.1:{kapi}?from=sender@example.invalid")
}

fn cikti(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr)
}

fn sahte_smtp(l: TcpListener) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let (s, _) = l.accept().expect("SMTP bağlantısı");
        s.set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("süre");
        let mut s = BufReader::new(s);
        let mut satir = String::new();
        s.get_mut().write_all(b"220 fixture\r\n").expect("220");
        for (beklenen, cevap) in [
            ("EHLO ", "250 fixture\r\n"),
            ("AUTH PLAIN ", "235 ok\r\n"),
            ("MAIL FROM:", "250 ok\r\n"),
            ("RCPT TO:", "250 ok\r\n"),
            ("DATA", "354 go\r\n"),
        ] {
            satir.clear();
            s.read_line(&mut satir).expect("komut");
            assert!(satir.starts_with(beklenen), "{satir}");
            s.get_mut().write_all(cevap.as_bytes()).expect("cevap");
        }
        let mut tel = String::new();
        loop {
            satir.clear();
            assert!(s.read_line(&mut satir).expect("veri") > 0);
            if satir == ".\r\n" {
                break;
            }
            tel.push_str(&satir);
        }
        s.get_mut().write_all(b"250 accepted\r\n").expect("250");
        satir.clear();
        let _ = s.read_line(&mut satir);
        let _ = s.get_mut().write_all(b"221 bye\r\n");
        tel
    })
}

fn baglanti_olmadi(l: &TcpListener) {
    assert!(
        matches!(l.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock),
        "yerel ret ağa çıkmamalı"
    );
}

#[test]
fn bildirimsiz_proje_dort_turde_kalir_yerel_ret_olculebilir() {
    let l = TcpListener::bind("127.0.0.1:0").expect("dinleyici");
    l.set_nonblocking(true).expect("nonblocking");
    let kapi = l.local_addr().expect("adres").port();
    let p = Proje::yeni(kapi, None, "\"rapor.docx\" listesi");
    std::fs::write(p.0.join("rapor.docx"), DOCX).expect("docx");
    for smtp in [Some(url(kapi)), None] {
        let out = p.calistir(smtp.as_deref());
        let metin = cikti(&out);
        assert!(out.status.success(), "{metin}");
        assert!(
            metin.contains("ek reddedildi")
                && metin.contains("izinli değil")
                && metin.contains("pdf, png, jpeg, webp")
                && metin.contains("ağa çıkılmadı"),
            "{metin}"
        );
        assert!(!metin.contains("SENTINEL"));
        baglanti_olmadi(&l);
    }
}

#[test]
fn bildirimli_proje_docx_ve_csv_gonderir_tel_bicimi_kayittan() {
    let l = TcpListener::bind("127.0.0.1:0").expect("dinleyici");
    let kapi = l.local_addr().expect("adres").port();
    let sunucu = sahte_smtp(l);
    let p = Proje::yeni(
        kapi,
        Some("\"pdf\", \"docx\", \"csv\""),
        "\"rapor.docx\", \"kişiler.csv\" listesi",
    );
    std::fs::write(p.0.join("rapor.docx"), DOCX).expect("docx");
    std::fs::write(p.0.join("kişiler.csv"), "ad;e-posta\nZeynep;z@ornek.test\n").expect("csv");
    let out = p.calistir(Some(&url(kapi)));
    assert!(out.status.success(), "{}", cikti(&out));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "1",
        "{}",
        cikti(&out)
    );
    let tel = sunucu.join().expect("SMTP");
    for beklenen in [
        "Content-Type: application/vnd.openxmlformats-officedocument.wordprocessingml.document\r\n",
        "Content-Type: text/csv; charset=utf-8\r\n",
        "filename*=UTF-8''rapor.docx",
        "filename*=UTF-8''ki%C5%9Filer.csv",
        "Content-Disposition: attachment;",
    ] {
        assert!(tel.contains(beklenen), "{beklenen}\n{tel}");
    }
}

#[test]
fn imza_utf8_ve_kume_disi_tur_ag_oncesi_reddedilir() {
    let l = TcpListener::bind("127.0.0.1:0").expect("dinleyici");
    l.set_nonblocking(true).expect("nonblocking");
    let kapi = l.local_addr().expect("adres").port();
    for (dosya, icerik, beklenen) in [
        ("sahte.docx", &b"MZ\x90\x00 yurutulebilir"[..], "uyuşmuyor"),
        ("bozuk.csv", &b"ad\xff\xfe"[..], "UTF-8"),
        ("nul.csv", &b"a\x00b"[..], "UTF-8"),
        // Bildirim kümeyi tam belirler: listede olmayan png reddedilir.
        ("foto.png", &b"\x89PNG\r\n"[..], "izinli değil"),
        // Kayıtta olmayan uzantı bildirimle de açılmaz.
        ("makro.docm", &b"PK\x03\x04"[..], "izinli değil"),
    ] {
        let p = Proje::yeni(
            kapi,
            Some("\"docx\", \"csv\""),
            &format!("\"{dosya}\" listesi"),
        );
        std::fs::write(p.0.join(dosya), icerik).expect("ek");
        let out = p.calistir(Some(&url(kapi)));
        let metin = cikti(&out);
        assert!(
            metin.contains("ek reddedildi") && metin.contains(beklenen),
            "{dosya}: {metin}"
        );
        baglanti_olmadi(&l);
    }
}

#[test]
fn makro_yurutulebilir_ve_bos_bildirim_p019() {
    for turler in [
        "\"docm\"",
        "\"exe\"",
        "\"html\"",
        "\"svg\"",
        "\"jpg\"",
        "\"pdf\", \"pdf\"",
        "\"ikili\"",
    ] {
        let p = Proje::yeni(1, Some(turler), "boş liste");
        let out = p.calistir(None);
        let metin = cikti(&out);
        assert!(!out.status.success(), "{turler}: {metin}");
        assert!(metin.contains("P019"), "{turler}: {metin}");
    }
}
