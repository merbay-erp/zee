//! K-382: gerçek native `dil çalıştır <proje>` sürecinde göreli dosya
//! yolları giriş dosyasının klasöründen çözülür (spec/07); yokluk "dosya yolu
//! güvenle çözülemedi" diye değil, eksik hedef klasörü ya da bulunamayan
//! dosya adıyla bildirilir. Proje kökü giriş klasöründen (`kaynak/`) farklıdır
//! ve proje kökünde aynı adlı bir `depo/ekler` klasörü durur: çözüm kökü proje
//! köküne kaysaydı taşıma başarılı olurdu.

use std::path::Path;
use std::process::Command;

const BILDIRIM: &str = "proje \"yolkoku\" olsun\nsürüm \"0.1.0\" olsun\nmorfoloji \"zee-tr-1\" olsun\ngiriş \"kaynak/ana.dil\" olsun\nyetkinlikler \"dosya-okuma\", \"dosya-yazma\" listesi olsun\nyerel_bağımlılıklar boş liste olsun\n";

struct Temizle(std::path::PathBuf);

impl Drop for Temizle {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `(çıkış, stdout satırları)` döndürür; yazılan dosyanın giriş klasöründe
/// durduğunu ve proje kökündeki klasöre dokunulmadığını ayrıca doğrular.
pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    let ek = format!(
        "zee-k382-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    );
    let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(&ek);
    let _temizle = Temizle(proje.clone());
    std::fs::create_dir_all(proje.join("kaynak")).expect("giriş klasörü");
    std::fs::create_dir_all(proje.join("depo/ekler")).expect("proje kökü klasörü");
    std::fs::write(proje.join("proje.dil"), BILDIRIM).expect("bildirim");
    std::fs::copy(kaynak, proje.join("kaynak/ana.dil")).expect("kaynak kopyası");
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", proje.to_str().expect("UTF-8 yol")])
        .current_dir(std::env::temp_dir())
        .output()
        .expect("dil süreci");
    let satirlar: Vec<String> = String::from_utf8_lossy(&cikti.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(
        std::fs::read_to_string(proje.join("kaynak/parca.txt")).expect("giriş klasörüne yazılmalı"),
        "ek\n"
    );
    assert!(
        !proje.join("parca.txt").exists(),
        "proje köküne yazılmamalı"
    );
    assert_eq!(
        std::fs::read_dir(proje.join("depo/ekler"))
            .expect("kök klasörü")
            .count(),
        0,
        "proje kökündeki klasöre dokunulmamalı"
    );
    (i64::from(cikti.status.code().unwrap_or(-1)), satirlar)
}
