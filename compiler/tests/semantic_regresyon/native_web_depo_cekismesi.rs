//! K-353: `--web-proxy` profilinin `.zee/web-durumu-v1.json` ortak deposu
//! her istekte durumu yazma kilidi dışında okuyup ayrıştırıyor, sonra kilit
//! altında bayt karşılaştırıp (CAS) yayımlıyordu; 128 ardışık çakışmada istek
//! 503 olurdu. Çok işçi grubunda yavaş süreç (ör. yeniden başlatmada yeni
//! açılan işçi, büyümüş depo) hızlı yazarlara her turda yeniliyordu: K-343
//! systemd ölçümünde 1117 istekte bir 503 ve 1,4 sn'lik istek.
//!
//! Harness yarışı belirlenimli kurar: gerçek native `dil` worker'ı ayakta
//! iken depoyu ~1 MiB geçerli oran kaydıyla doldurur ve bir iş parçacığında
//! zee'nin aynı klasör yazma kilidiyle (`kalici_dosya::atomik_yaz`) her
//! birkaç ms'de yeniden yayımlar. Worker'ın kilit dışı okuma+ayrıştırma
//! penceresi yazarın aralığından uzun olduğundan eski döngü her turu kaybeder;
//! düzeltme okuma-değiştirme-yazmayı kilit içinde yaptığından istek geçer.
//! Çıktı başarılı ve 503 istek sayısıdır.
#![allow(dead_code)]

use std::path::Path;

const ISTEK: usize = 3;
const ORAN_KAYDI: usize = 10_000;

pub fn calistir(kaynak_yolu: &Path) -> (i64, Vec<String>) {
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
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
        "zee-k353-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    ));
    std::fs::create_dir_all(&proje).expect("proje klasörü");
    let ana = proje.join("ana.dil");
    std::fs::copy(kaynak_yolu, &ana).expect("kaynak kopyası");
    let kapi = super::native_web_kopma_fixture::bos_kapi();
    let cocuk = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args([
            "çalıştır",
            "--web-proxy",
            super::native_web_kopma_fixture::PROXY,
            "--web-worker-port",
            &kapi.to_string(),
            ana.to_str().expect("UTF-8 kaynak yolu"),
        ])
        .current_dir(&proje)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("worker süreci");
    let mut temizle = Temizle(proje.clone(), Some(cocuk));

    let istek = |kimlik: &str| -> Option<u16> {
        let mut akis = std::net::TcpStream::connect(("127.0.0.1", kapi)).ok()?;
        akis.set_read_timeout(Some(Duration::from_secs(120))).ok()?;
        write!(
            akis,
            "GET / HTTP/1.1\r\nHost: panel.example\r\nForwarded: for={kimlik};proto=https;host=panel.example\r\nConnection: close\r\n\r\n"
        )
        .ok()?;
        let mut yanit = Vec::new();
        akis.read_to_end(&mut yanit).ok()?;
        std::str::from_utf8(yanit.get(9..12)?).ok()?.parse().ok()
    };
    let son = Instant::now() + Duration::from_secs(60);
    while istek("203.0.113.1") != Some(200) {
        if let Some(durum) = temizle.1.as_mut().and_then(|c| c.try_wait().ok().flatten()) {
            panic!("worker hazır olmadan çıktı: {durum}");
        }
        assert!(Instant::now() < son, "worker 60 sn içinde hazır olmadı");
        std::thread::sleep(Duration::from_millis(50));
    }

    let depo = proje.join(".zee/web-durumu-v1.json");
    assert!(depo.is_file(), "ilk istek kalıcı depoyu kurmalı");
    let pencere_sonu = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("saat")
        .as_millis()
        + 3_600_000;
    let oranlar = (0..ORAN_KAYDI)
        .map(|i| format!("\"{i:064x}\":{{\"pencere_sonu_ms\":{pencere_sonu},\"sayi\":1}}"))
        .collect::<Vec<_>>()
        .join(",");
    fn govde(oranlar: &str, sira: u64) -> String {
        format!(
            "{{\"bicim_surumu\":1,\"degisiklik_sirasi\":{sira},\"sonraki_olusturma_sirasi\":0,\"oturumlar\":{{}},\"oranlar\":{{{oranlar}}}}}"
        )
    }
    dil::kalici_dosya::atomik_yaz(&depo, govde(&oranlar, 1).as_bytes()).expect("büyük depo");

    let dur = Arc::new(AtomicBool::new(false));
    let yazar = {
        let dur = Arc::clone(&dur);
        let depo = depo.clone();
        std::thread::spawn(move || {
            let mut sira = 2;
            while !dur.load(Ordering::Acquire) {
                dil::kalici_dosya::atomik_yaz(&depo, govde(&oranlar, sira).as_bytes())
                    .expect("hızlı yazar");
                sira += 1;
                std::thread::sleep(Duration::from_millis(3));
            }
        })
    };
    let durumlar = (0..ISTEK)
        .map(|i| istek(&format!("198.51.100.{}", i + 1)))
        .collect::<Vec<_>>();
    dur.store(true, Ordering::Release);
    yazar.join().expect("hızlı yazar iş parçacığı");
    let say = |kod: u16| durumlar.iter().filter(|d| **d == Some(kod)).count();
    let digerleri = ISTEK - say(200) - say(503);
    (
        0,
        vec![
            format!("başarılı {}", say(200)),
            format!("503 {}", say(503)),
            format!("diğer {digerleri}"),
        ],
    )
}
