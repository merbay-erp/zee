//! K-370: `--web-proxy` ortak deposu 4096 oturum sınırına yalnız kimlikli
//! kayıtlarla dolduğunda oturumsuz istemcinin `csrf belirteci` isteği anonim
//! oturum açamaz. Bu ret rota içinde C022 tanısına, istek-yerel hata sınırında
//! da 500 "sunucu hatası" yanıtına dönüşüyordu; aynı kaynak durumu istek
//! commit'inde 503 `web güvenlik durumu güvenle kaydedilemedi` idi (spec/12).
//!
//! Harness gerçek native `dil` worker'ını açar, ilk istekle kalıcı depoyu
//! kurdurur, depoyu 4096 geçerli kimlikli oturumla zee'nin klasör yazma
//! kilidiyle (`kalici_dosya::atomik_yaz`) yeniden yayımlar ve `/form` ile `/`
//! isteklerinin durum satırı + gövdesini yazar.
#![allow(dead_code)]

use std::path::Path;

const OTURUM: usize = 4096;

pub fn calistir(kaynak_yolu: &Path) -> (i64, Vec<String>) {
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    struct Temizle(std::path::PathBuf, Option<std::process::Child>);
    impl Drop for Temizle {
        fn drop(&mut self) {
            if let Some(mut cocuk) = self.1.take() {
                let _ = cocuk.kill();
                let _ = cocuk.wait();
            }
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "zee-k370-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    ));
    std::fs::create_dir_all(&proje).expect("proje klasörü");
    std::fs::copy(kaynak_yolu, proje.join("ana.dil")).expect("kaynak kopyası");
    std::fs::write(
        proje.join("proje.dil"),
        "proje \"k370\" olsun\nsürüm \"0.1.0\" olsun\nmorfoloji \"zee-tr-1\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ-sunucusu\", \"web-oturumu\" listesi olsun\nyerel_bağımlılıklar boş liste olsun\n",
    )
    .expect("proje bildirimi");
    let kapi = super::native_web_kopma_fixture::bos_kapi();
    let cocuk = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args([
            "çalıştır",
            "--web-proxy",
            super::native_web_kopma_fixture::PROXY,
            "--web-worker-port",
            &kapi.to_string(),
            proje.to_str().expect("UTF-8 proje yolu"),
        ])
        .current_dir(&proje)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("worker süreci");
    let mut temizle = Temizle(proje.clone(), Some(cocuk));

    let istek = |yol: &str, kimlik: &str| -> Option<(u16, String)> {
        let mut akis = std::net::TcpStream::connect(("127.0.0.1", kapi)).ok()?;
        akis.set_read_timeout(Some(Duration::from_secs(60))).ok()?;
        write!(
            akis,
            "GET {yol} HTTP/1.1\r\nHost: panel.example\r\nForwarded: for={kimlik};proto=https;host=panel.example\r\nConnection: close\r\n\r\n"
        )
        .ok()?;
        let mut yanit = Vec::new();
        akis.read_to_end(&mut yanit).ok()?;
        let metin = String::from_utf8(yanit).ok()?;
        let durum = metin.get(9..12)?.parse().ok()?;
        let govde = metin.split_once("\r\n\r\n")?.1.trim().to_string();
        Some((durum, govde))
    };
    let son = Instant::now() + Duration::from_secs(60);
    while istek("/", "203.0.113.1").map(|(d, _)| d) != Some(200) {
        if let Some(durum) = temizle.1.as_mut().and_then(|c| c.try_wait().ok().flatten()) {
            panic!("worker hazır olmadan çıktı: {durum}");
        }
        assert!(Instant::now() < son, "worker 60 sn içinde hazır olmadı");
        std::thread::sleep(Duration::from_millis(50));
    }

    let depo = proje.join(".zee/web-durumu-v1.json");
    assert!(depo.is_file(), "ilk istek kalıcı depoyu kurmalı");
    let an = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("saat")
        .as_millis();
    let oturumlar = (0..OTURUM)
        .map(|i| {
            format!(
                "\"{i:064x}\":{{\"csrf\":\"{i:064x}\",\"kullanici\":\"k{i}\",\"rol\":\"uye\",\"son_gecerlilik_ms\":{},\"son_erisim_ms\":{an},\"olusturma_ms\":{an},\"olusturma_sirasi\":{i}}}",
                an + 3_600_000
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let govde = format!(
        "{{\"bicim_surumu\":1,\"degisiklik_sirasi\":1000000,\"sonraki_olusturma_sirasi\":{OTURUM},\"oturumlar\":{{{oturumlar}}},\"oranlar\":{{}}}}"
    );
    dil::kalici_dosya::atomik_yaz(&depo, govde.as_bytes()).expect("dolu depo");

    let yaz = |ad: &str, sonuc: Option<(u16, String)>| match sonuc {
        Some((durum, govde)) => format!("{ad} {durum} {govde}"),
        None => format!("{ad} yanıt yok"),
    };
    let form = yaz("form", istek("/form", "198.51.100.1"));
    let sonraki = yaz("sonraki", istek("/", "198.51.100.2"));
    (0, vec![form, sonraki])
}
