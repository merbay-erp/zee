//! K-346: gerçek native `dil` süreçleri aynı dosyaya eşzamanlı ekler. İki
//! yazar süreci her biri 1000 satırı düz `ekle` ile yazarken iki geri alan
//! süreç regression kaynağını (eylem içinde ekleyip başarısız Sonuç döndüren)
//! döngüde yeniden başlatır. Düzeltme öncesi geri alma, kilit dışında okunan
//! yedekle yazarların başarılı satırlarını siliyordu (1000'de ~50). Eylem
//! araya giren yazarı görünce C021 ile çıkar; harness onu yeniden başlatır.
//! Çıktı her yazarın kalıcı satır sayısı ve biçimi bozuk satır sayısıdır.
#![allow(dead_code)]

use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const YAZAR_SATIRI: usize = 1000;

pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    struct Temizle(std::path::PathBuf);
    impl Drop for Temizle {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "zee-eszamanli-ekleme-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    ));
    let _temizle = Temizle(proje.clone());
    std::fs::create_dir_all(&proje).expect("proje klasörü");
    std::fs::copy(kaynak, proje.join("geri-alan.dil")).expect("kaynak kopyası");
    for yazar in ["b", "c"] {
        std::fs::write(
            proje.join(format!("{yazar}.dil")),
            format!(
                "1 den {YAZAR_SATIRI} e kadar her sayı için\n    satır \"{yazar}-\" ile sayı olsun\n    \"ortak.txt\" dosyasına satır ekle\n"
            ),
        )
        .expect("yazar kaynağı");
    }

    let calistir_dil = |dosya: &str| {
        Command::new(env!("CARGO_BIN_EXE_dil"))
            .args(["çalıştır", dosya])
            .current_dir(&proje)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("dil süreci")
    };

    let bitti = Arc::new(AtomicBool::new(false));
    let geri_alanlar: Vec<_> = (0..2)
        .map(|_| {
            let bitti = Arc::clone(&bitti);
            let proje = proje.clone();
            std::thread::spawn(move || {
                while !bitti.load(Ordering::Acquire) {
                    let mut cocuk = Command::new(env!("CARGO_BIN_EXE_dil"))
                        .args(["çalıştır", "geri-alan.dil"])
                        .current_dir(&proje)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .expect("geri alan süreç");
                    while cocuk.try_wait().expect("geri alan durumu").is_none() {
                        if bitti.load(Ordering::Acquire) {
                            let _ = cocuk.kill();
                            let _ = cocuk.wait();
                            return;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(2));
                    }
                }
            })
        })
        .collect();

    let mut yazarlar = [calistir_dil("b.dil"), calistir_dil("c.dil")];
    let mut cikis = 0;
    for yazar in &mut yazarlar {
        let durum = yazar.wait().expect("yazar çıkışı");
        if !durum.success() {
            cikis = i64::from(durum.code().unwrap_or(-1));
        }
    }
    bitti.store(true, Ordering::Release);
    for kol in geri_alanlar {
        kol.join().expect("geri alan iş parçacığı");
    }

    // K-348: geri alan süreç Windows'ta ReplaceFileW ortasında öldürülürse
    // hedef geçici olarak yoktur ve eski bütün `.zee-yedek`tedir; ürün sözü
    // resmî zee okuyucusunundur (düz std okuması yarım durumu görür).
    let icerik = String::from_utf8(
        dil::kaynak_sinirlari::veri_dosyasi_baytlarini_oku(&proje.join("ortak.txt"))
            .expect("ortak dosya"),
    )
    .expect("UTF-8 ortak dosya");
    let bicimli = |satir: &str| {
        satir.split_once('-').is_some_and(|(onek, sayi)| {
            matches!(onek, "a" | "b" | "c") && sayi.parse::<usize>().is_ok()
        })
    };
    let say = |onek: &str| {
        icerik
            .lines()
            .filter(|satir| satir.starts_with(&format!("{onek}-")) && bicimli(satir))
            .collect::<std::collections::HashSet<_>>()
            .len()
    };
    let bozuk = icerik.lines().filter(|satir| !bicimli(satir)).count()
        + usize::from(!icerik.is_empty() && !icerik.ends_with('\n'));
    (
        cikis,
        vec![
            format!("b {}", say("b")),
            format!("c {}", say("c")),
            format!("bozuk {bozuk}"),
        ],
    )
}
