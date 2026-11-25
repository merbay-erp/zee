//! K-401 (ZM-F044): gerçek native `dil çalıştır <proje>` sürecinde eylem
//! dışındaki `atomik taşımayı dene` içeriği belleğe okumaz; 16 MiB veri okuma
//! sınırını aşan 19 MiB'lık dosya taşınır. Düzeltme öncesi taşıma izli yoldan
//! içeriği okuyup sınıra çarpıyor, çıktı `taşıma reddedildi` oluyordu.

use std::path::Path;
use std::process::Command;

const BILDIRIM: &str = "proje \"buyuktasima\" olsun\nsürüm \"0.1.0\" olsun\nmorfoloji \"zee-tr-1\" olsun\ngiriş \"kaynak/ana.dil\" olsun\nyetkinlikler \"dosya-okuma\", \"dosya-yazma\" listesi olsun\nyerel_bağımlılıklar boş liste olsun\n";

/// 19 MiB: genel 16 MiB veri okuma sınırının üstünde.
const BOYUT: usize = 19 * 1024 * 1024;

struct Temizle(std::path::PathBuf);

impl Drop for Temizle {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn icerik() -> Vec<u8> {
    (0..BOYUT).map(|i| (i % 251) as u8).collect()
}

/// `(çıkış, stdout satırları)` döndürür; `taşındı` gözlenirse kaynağın
/// silindiğini ve hedefin bayt bayt aynı içerikle durduğunu ayrıca doğrular.
pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    let ek = format!(
        "zee-k395b-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    );
    let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(&ek);
    let _temizle = Temizle(proje.clone());
    std::fs::create_dir_all(proje.join("kaynak/gelen")).expect("gelen klasörü");
    std::fs::create_dir_all(proje.join("kaynak/ekler")).expect("ekler klasörü");
    std::fs::write(proje.join("proje.dil"), BILDIRIM).expect("bildirim");
    std::fs::copy(kaynak, proje.join("kaynak/ana.dil")).expect("kaynak kopyası");
    let beklenen = icerik();
    std::fs::write(proje.join("kaynak/gelen/ek.bin"), &beklenen).expect("19 MiB kaynak");
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", proje.to_str().expect("UTF-8 yol")])
        .current_dir(std::env::temp_dir())
        .output()
        .expect("dil süreci");
    let satirlar: Vec<String> = String::from_utf8_lossy(&cikti.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    if satirlar == ["taşındı"] {
        assert!(
            !proje.join("kaynak/gelen/ek.bin").exists(),
            "kaynak taşımadan sonra kalmamalı"
        );
        let hedef = std::fs::read(proje.join("kaynak/ekler/ek.bin")).expect("hedef");
        assert!(hedef == beklenen, "hedef içerik bayt bayt korunmalı");
    } else {
        assert!(
            !proje.join("kaynak/ekler/ek.bin").exists(),
            "reddedilen taşıma hedef bırakmamalı"
        );
    }
    (i64::from(cikti.status.code().unwrap_or(-1)), satirlar)
}
