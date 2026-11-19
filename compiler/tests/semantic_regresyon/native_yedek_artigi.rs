//! K-351: Windows'ta başarılı `ReplaceFileW` sonrası yedek silinemezse
//! (dış süreç onu silme paylaşımı olmadan açık tutuyor) yedek hedef varken
//! klasörde kalıyordu. Hedef sonra zee dışından silinince sonraki okuma ve
//! ekleme o eski içeriği "yarım değiştirme kurtarması" sanıp diriltiyordu.
//! Harness artığı CI'daki adla kurar, gerçek native `dil`i iki kez koşar:
//! ilk koşu hedef varken yazar (artık silinmeli), harness hedefi dışarıdan
//! siler, ikinci koşu dosyayı bulamamalı. Düzeltme öncesi `iz 1`, ikinci
//! koşu `eski` okur ve dosya `eski|b-1` olur. Yedek yalnız Windows yazarında
//! oluştuğundan kip Windows dışında ignored.
#![allow(dead_code)]

use std::path::Path;

pub fn calistir(kaynak: &Path) -> (i64, Vec<String>) {
    struct Temizle(std::path::PathBuf);
    impl Drop for Temizle {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    let proje = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "zee-yedek-artigi-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("saat")
            .as_nanos()
    ));
    let _temizle = Temizle(proje.clone());
    std::fs::create_dir_all(&proje).expect("proje klasörü");
    std::fs::copy(kaynak, proje.join("ana.dil")).expect("kaynak kopyası");
    std::fs::write(proje.join(".zee-yazma-kilidi"), b"").expect("kilit dosyası");
    std::fs::write(
        proje.join(".ortak.txt.zee-gecici-4242-1.zee-yedek"),
        "eski\n",
    )
    .expect("başarılı yazımın artık yedeği");
    std::fs::write(proje.join("ortak.txt"), "a-1\n").expect("güncel hedef");

    let kos = |satirlar: &mut Vec<String>| {
        let cikti = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
            .args(["çalıştır", "ana.dil"])
            .current_dir(&proje)
            .output()
            .expect("dil süreci");
        satirlar.extend(
            String::from_utf8_lossy(&cikti.stdout)
                .lines()
                .filter(|satir| !satir.is_empty())
                .map(str::to_string),
        );
        i64::from(cikti.status.code().unwrap_or(-1))
    };
    let mut satirlar = Vec::new();
    let ilk = kos(&mut satirlar);
    let artik = std::fs::read_dir(&proje)
        .expect("proje listesi")
        .filter(|girdi| {
            girdi
                .as_ref()
                .map(|g| g.file_name().to_string_lossy().contains("zee-gecici"))
                .unwrap_or(false)
        })
        .count();
    satirlar.push(format!("iz {artik}"));
    // Zee dışı silme (kullanıcı, yedekleme aracı).
    std::fs::remove_file(proje.join("ortak.txt")).expect("dış silme");
    let ikinci = kos(&mut satirlar);
    let son = std::fs::read_to_string(proje.join("ortak.txt")).unwrap_or_default();
    satirlar.push(format!("son {}", son.lines().collect::<Vec<_>>().join("|")));
    (ilk.max(ikinci), satirlar)
}
