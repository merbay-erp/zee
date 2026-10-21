//! K-329 (W5): ayrı bağlama noktaları arasındaki dosya taşımasının gerçek
//! native `dil` sürecinde EXDEV'e rağmen tamamlandığını sınar. Proje Cargo
//! hedef klasöründe (disk), `medya` ise `/dev/shm` (tmpfs) altındaki bir
//! klasöre sembolik bağdır; `rename(2)` bu iki yol arasında EXDEV verir. Bu,
//! systemd `ReadWritePaths=` ile ayrı bind mount edilen `.zee` ve `medya`
//! yollarının root gerektirmeyen eşdeğeridir. ToplayanIo aygıt sınırını
//! taklit edemediği için toplu replay bu kipi atlar.
#![allow(dead_code)]

use std::path::Path;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process::Command;

#[cfg(target_os = "linux")]
const ICERIK: &[u8] = b"K-329 medya";

#[cfg(target_os = "linux")]
struct Temizle(Vec<PathBuf>);

#[cfg(target_os = "linux")]
impl Drop for Temizle {
    fn drop(&mut self) {
        for yol in &self.0 {
            let _ = std::fs::remove_dir_all(yol);
        }
    }
}

/// `(çıkış, stdout satırları)` döndürür; taşıma sonrası dosya durumu
/// sözleşmesini (kaynak yok, hedef tam içerik ve 0640, artık geçici yok) ayrıca
/// doğrular.
#[cfg(target_os = "linux")]
pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let ek = format!(
        "zee-exdev-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    );
    let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(&ek);
    let ayri = Path::new("/dev/shm").join(&ek);
    let _temizle = Temizle(vec![proje.clone(), ayri.clone()]);
    std::fs::create_dir_all(proje.join(".zee/yuklemeler")).expect("yükleme klasörü");
    std::fs::create_dir_all(&ayri).expect("/dev/shm klasörü");
    assert_ne!(
        std::fs::metadata(&proje).expect("proje").dev(),
        std::fs::metadata(&ayri).expect("tmpfs").dev(),
        "Cargo hedef klasörü ile /dev/shm aynı aygıtta; EXDEV kanıtı üretilemez"
    );
    std::os::unix::fs::symlink(&ayri, proje.join("medya")).expect("medya bağı");
    let parca = proje.join(".zee/yuklemeler/parca");
    std::fs::write(&parca, ICERIK).expect("parça");
    std::fs::set_permissions(&parca, std::fs::Permissions::from_mode(0o640)).expect("mod");
    assert_eq!(
        std::fs::rename(&parca, ayri.join("deneme"))
            .expect_err("aygıt sınırı")
            .raw_os_error(),
        Some(18),
        "rename EXDEV vermeli"
    );
    let ana = proje.join("ana.dil");
    std::fs::copy(kaynak, &ana).expect("kaynak kopyası");
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", ana.to_str().expect("UTF-8 yol")])
        .current_dir(&proje)
        .output()
        .expect("dil süreci");
    let satirlar: Vec<String> = String::from_utf8_lossy(&cikti.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    let cikis = i64::from(cikti.status.code().unwrap_or(-1));
    if satirlar == ["taşındı"] {
        assert!(!parca.exists(), "kaynak silinmeli");
        let hedef = ayri.join("resim.bin");
        assert_eq!(std::fs::read(&hedef).expect("hedef"), ICERIK);
        assert_eq!(
            std::fs::metadata(&hedef)
                .expect("hedef")
                .permissions()
                .mode()
                & 0o777,
            0o640,
            "izinler korunmalı"
        );
        let adlar: Vec<_> = std::fs::read_dir(&ayri)
            .expect("tmpfs")
            .map(|giris| giris.expect("giriş").file_name())
            .filter(|ad| ad != ".zee-yazma-kilidi")
            .collect();
        assert_eq!(adlar, ["resim.bin"], "yarım/geçici dosya kalmamalı");
    } else {
        assert_eq!(std::fs::read(&parca).expect("kaynak korunur"), ICERIK);
    }
    (cikis, satirlar)
}

#[cfg(not(target_os = "linux"))]
pub fn calistir(_kaynak: &Path) -> (i64, Vec<String>) {
    panic!("native-dosya-exdev kipi gerçek Linux /dev/shm aygıt sınırı ister")
}
