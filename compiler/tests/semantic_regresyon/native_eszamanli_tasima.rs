//! K-347: gerçek native `dil` süreçleri. Bir yazar süreci `kaynak.txt`e 1000
//! satırı düz `ekle` ile yazarken iki taşıyıcı süreç regression kaynağını
//! (ayrı hedef adlarıyla) döngüde yeniden başlatır: eylem içinde kaynağa
//! ekler, kaynağı hedefe taşır ve başarısız döner. Ekleme ile taşıma
//! arasındaki saf hesap döngüsü yarış penceresini genişletir (dosya okuması
//! yazma kilidini beklediğinden pencereyi açmaz).
//!
//! Düzeltme öncesi taşıma kaynağın içeriğini ve yedeğini kilit dışında
//! okuduğundan geri alma hedefi silip kaynağa yazarın satırı olmayan bayat
//! içeriği koyuyordu: 1000 satırdan 4–160'ı kayboldu (macOS debug, 4/4 koşu).
//! Düzeltmeden sonra araya giren yazar C021 ile görülür, hiçbir yol değişmez;
//! harness kalan hedefi kenara alıp taşıyıcıyı yeniden başlatır. Çıktı:
//! bütün dosyalarda yazarın kalıcı satır sayısı, birden çok dosyada görünen
//! satır ve biçimi bozuk satır sayısı.
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
        "zee-eszamanli-tasima-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    ));
    let _temizle = Temizle(proje.clone());
    std::fs::create_dir_all(&proje).expect("proje klasörü");
    std::fs::copy(kaynak, proje.join("tasiyici.dil")).expect("kaynak kopyası");
    std::fs::write(
        proje.join("yazar.dil"),
        format!(
            "1 den {YAZAR_SATIRI} e kadar her sayı için\n    satır \"b-\" ile sayı olsun\n    \"kaynak.txt\" dosyasına satır ekle\n"
        ),
    )
    .expect("yazar kaynağı");

    // İki taşıyıcı aynı kaynağı ayrı hedeflere taşır; biri diğeri için de
    // araya giren yazardır ve yarış penceresi sıklaşır.
    let kaynak_metni = std::fs::read_to_string(proje.join("tasiyici.dil")).expect("taşıyıcı");
    for sira in 1..=2 {
        std::fs::write(
            proje.join(format!("tasiyici-{sira}.dil")),
            kaynak_metni.replace("\"hedef.txt\"", &format!("\"hedef-{sira}.txt\"")),
        )
        .expect("taşıyıcı kopyası");
    }
    let bitti = Arc::new(AtomicBool::new(false));
    let tasiyicilar: Vec<_> = (1..=2)
        .map(|sira| {
            let bitti = Arc::clone(&bitti);
            let proje = proje.clone();
            std::thread::spawn(move || {
                let mut kenar = 0usize;
                while !bitti.load(Ordering::Acquire) {
                    // Taşıyıcı öldürülmez: geri alma ortasında kesilen süreç
                    // satırı iki dosyada bırakabilirdi. Döngü kısadır, kendisi biter.
                    let _ = Command::new(env!("CARGO_BIN_EXE_dil"))
                        .args(["çalıştır", &format!("tasiyici-{sira}.dil")])
                        .current_dir(&proje)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .expect("taşıyıcı süreç");
                    // C021: geri alma araya gireni ezmedi; taşınan içerik
                    // hedefte kaldı. Bu hedefe yalnız bu taşıyıcı dokunur;
                    // süreci yokken kenara alınır.
                    let hedef = proje.join(format!("hedef-{sira}.txt"));
                    if hedef.exists() {
                        kenar += 1;
                        std::fs::rename(&hedef, proje.join(format!("kenar-{sira}-{kenar}.txt")))
                            .expect("hedef kenara");
                    }
                }
            })
        })
        .collect();

    let durum = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", "yazar.dil"])
        .current_dir(&proje)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("yazar süreci");
    let cikis = if durum.success() {
        0
    } else {
        i64::from(durum.code().unwrap_or(-1))
    };
    bitti.store(true, Ordering::Release);
    for tasiyici in tasiyicilar {
        tasiyici.join().expect("taşıyıcı iş parçacığı");
    }

    let mut dosyalar: Vec<_> = std::fs::read_dir(&proje)
        .expect("proje listesi")
        .filter_map(|girdi| {
            let ad = girdi.ok()?.file_name().into_string().ok()?;
            (ad == "kaynak.txt" || ad.starts_with("kenar-") || ad.starts_with("hedef-"))
                .then(|| proje.join(ad))
        })
        .collect();
    dosyalar.sort();
    let bicimli = |satir: &str| {
        satir == "ilk"
            || satir.split_once('-').is_some_and(|(onek, sayi)| {
                matches!(onek, "a" | "b") && sayi.parse::<usize>().is_ok()
            })
    };
    let mut gorulen = std::collections::HashMap::<String, usize>::new();
    let mut bozuk = 0;
    for dosya in &dosyalar {
        // K-348: resmî zee okuyucusu yarım Windows değiştirmesini görmez.
        let icerik = String::from_utf8(
            dil::kaynak_sinirlari::veri_dosyasi_baytlarini_oku(dosya).expect("dosya"),
        )
        .expect("UTF-8 dosya");
        bozuk += icerik.lines().filter(|satir| !bicimli(satir)).count()
            + usize::from(!icerik.is_empty() && !icerik.ends_with('\n'));
        let satirlar: std::collections::HashSet<_> = icerik
            .lines()
            .filter(|satir| satir.starts_with("b-") && bicimli(satir))
            .collect();
        for satir in satirlar {
            *gorulen.entry(satir.to_string()).or_default() += 1;
        }
    }
    let kopya = gorulen.values().filter(|sayi| **sayi > 1).count();
    (
        cikis,
        vec![
            format!("b {}", gorulen.len()),
            format!("kopya {kopya}"),
            format!("bozuk {bozuk}"),
        ],
    )
}
