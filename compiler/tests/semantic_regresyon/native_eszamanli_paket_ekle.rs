//! K-347: gerçek native `dil ekle` süreçleri aynı projeye eşzamanlı yerel
//! paket ekler. Düzeltme öncesi her komut `proje.dil`i okuyup grafiği
//! çözdükten sonra karşılaştırmasız yazıyordu; araya giren komutun eklediği
//! bağımlılık ezilir, komut yine başarı bildirirdi. Düzeltmeden sonra yazım
//! okunan byte'larla karşılaştırılır; kaybeden komut hiçbir dosyaya dokunmadan
//! "başka bir süreç" iletisiyle çıkar ve harness onu yeniden dener. Çıktı:
//! başarı bildiren komutların kalıcı bağımlılık sayısı, kaybolan bağımlılık
//! sayısı ve son kilidin `dil çalıştır` doğrulamasından geçip geçmediği.
#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

const PAKET: usize = 6;
const TUR: usize = 4;
const DENEME: usize = 40;

pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    struct Temizle(std::path::PathBuf);
    impl Drop for Temizle {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    let kok = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "zee-eszamanli-paket-ekle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    ));
    let _temizle = Temizle(kok.clone());
    let paket_kaynagi = std::fs::read_to_string(kaynak).expect("paket kaynağı");
    let mut kayip = 0;
    let mut kalici = 0;
    let mut kilit_gecerli = true;
    let mut cikis = 0;
    for tur in 0..TUR {
        let uygulama = kok.join(format!("uygulama-{tur}"));
        std::fs::create_dir_all(&uygulama).expect("uygulama");
        std::fs::write(
            uygulama.join("proje.dil"),
            "proje \"uygulama\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n",
        )
        .expect("uygulama bildirimi");
        std::fs::write(uygulama.join("ana.dil"), "\"tamam\" yaz\n").expect("ana");
        let adlar: Vec<String> = (0..PAKET).map(|i| format!("paket{tur}x{i}")).collect();
        for ad in &adlar {
            let paket = kok.join(ad);
            std::fs::create_dir_all(&paket).expect("paket");
            std::fs::write(
                paket.join("proje.dil"),
                format!("proje \"{ad}\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"paket.dil\" olsun\n"),
            )
            .expect("paket bildirimi");
            std::fs::write(paket.join("paket.dil"), &paket_kaynagi).expect("paket girişi");
        }

        let iplikler: Vec<_> = adlar
            .iter()
            .map(|ad| {
                let (kok, ad, uygulama) = (kok.clone(), ad.clone(), uygulama.clone());
                std::thread::spawn(move || {
                    let mut son_hata = String::new();
                    for deneme in 0..DENEME {
                        let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
                            .current_dir(&kok)
                            .args(["ekle", ad.as_str(), uygulama.to_str().expect("utf8")])
                            .output()
                            .expect("dil ekle");
                        if cikti.status.success() {
                            return Some(ad);
                        }
                        son_hata = String::from_utf8_lossy(&cikti.stderr).into_owned();
                        // Yalnız karşılaştırma kaybı yeniden denenir.
                        if !son_hata.contains("başka bir süreç") {
                            // Tanı: libtest başarısız testin yakalanan stderr'ini gösterir.
                            eprintln!(
                                "K-347 harness: `dil ekle {ad}` {}. denemede {:?} ile düştü:\n{son_hata}",
                                deneme + 1,
                                cikti.status.code()
                            );
                            return None;
                        }
                    }
                    eprintln!("K-347 harness: `dil ekle {ad}` {DENEME} denemede kazanamadı; son:\n{son_hata}");
                    None
                })
            })
            .collect();
        let basarili: Vec<String> = iplikler
            .into_iter()
            .filter_map(|iplik| iplik.join().expect("ekle iş parçacığı"))
            .collect();
        if basarili.len() != PAKET {
            cikis = 2;
        }
        let bildirim = std::fs::read_to_string(uygulama.join("proje.dil")).expect("son bildirim");
        for ad in &basarili {
            if bildirim.contains(&format!("\"../{ad}\"")) {
                kalici += 1;
            } else {
                kayip += 1;
            }
        }
        let calistir = Command::new(env!("CARGO_BIN_EXE_dil"))
            .args(["çalıştır", uygulama.to_str().expect("utf8")])
            .output()
            .expect("dil çalıştır");
        if !calistir.status.success() {
            eprintln!(
                "K-347 harness: son kilit doğrulanamadı:\n{}",
                String::from_utf8_lossy(&calistir.stderr)
            );
            kilit_gecerli = false;
        }
    }
    (
        cikis,
        vec![
            format!("bağımlılık {kalici}"),
            format!("kayıp {kayip}"),
            format!("kilit {}", if kilit_gecerli { "geçerli" } else { "bozuk" }),
        ],
    )
}
