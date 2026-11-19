//! K-342: Windows `ReplaceFileW` penceresi yüklü makinede okuyucunun kısa
//! yeniden deneme bütçesini (K-182, ~50 ms) aşsa da gerçek native `dil`
//! okuyucusu hata yerine yeni bütünü görür. Harness pencereyi deterministik
//! kurar: önce hedef yok ve yazarın `.anlik.txt.zee-gecici-*` dosyası durur,
//! sonra hedef paylaşımsız tutanakla açık ve `….zee-yedek` yedeği durur;
//! toplam ~300 ms. ToplayanIo dosya sistemi penceresini taklit edemediği için
//! toplu replay bu kipi atlar.
#![allow(dead_code)]

use std::path::Path;

#[cfg(windows)]
pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    use std::io::{BufRead, BufReader, Write};
    use std::os::windows::fs::OpenOptionsExt;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    struct Temizle(std::path::PathBuf);
    impl Drop for Temizle {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "zee-okuma-penceresi-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    ));
    let _temizle = Temizle(proje.clone());
    std::fs::create_dir_all(&proje).expect("proje klasörü");
    let ana = proje.join("ana.dil");
    std::fs::copy(kaynak, &ana).expect("kaynak kopyası");
    let gecici = proje.join(".anlik.txt.zee-gecici-1-1");
    let yedek = proje.join(".anlik.txt.zee-gecici-1-1.zee-yedek");
    std::fs::write(&gecici, b"yeni").expect("yazarın geçici dosyası");

    let mut cocuk = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["çalıştır", ana.to_str().expect("UTF-8 yol")])
        .current_dir(&proje)
        .stdout(Stdio::piped())
        .spawn()
        .expect("dil süreci");
    let mut cikti = BufReader::new(cocuk.stdout.take().expect("stdout"));
    let mut ilk = String::new();
    cikti.read_line(&mut ilk).expect("ilk satır");
    // Pencere 1: ReplaceFileW eski hedefi yedeğe taşıdı, yeni henüz yerinde değil.
    std::thread::sleep(Duration::from_millis(150));
    // Pencere 2: yeni hedef yerinde ama değiştirme bitene dek paylaşımsız açık.
    std::fs::write(&yedek, b"eski").expect("yedek");
    let mut tutanak = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .share_mode(0)
        .open(proje.join("anlik.txt"))
        .expect("paylaşımsız hedef");
    tutanak.write_all(b"yeni").expect("yeni içerik");
    std::fs::remove_file(&gecici).expect("geçici taşındı");
    std::thread::sleep(Duration::from_millis(150));
    drop(tutanak);
    std::fs::remove_file(&yedek).expect("yedek silindi");

    let mut satirlar = vec![ilk.trim_end().to_string()];
    satirlar.extend(cikti.lines().map(|satir| satir.expect("stdout satırı")));
    let durum = cocuk.wait().expect("dil çıkışı");
    (i64::from(durum.code().unwrap_or(-1)), satirlar)
}

#[cfg(not(windows))]
pub fn calistir(_kaynak: &Path) -> (i64, Vec<String>) {
    panic!("native-dosya-okuma-penceresi kipi Windows ReplaceFileW penceresi ister")
}
