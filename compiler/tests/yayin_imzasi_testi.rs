//! K-317 / ADR-156: resmî sürüm imzası derlemesiz doğrulanır. Gerçek `dil anahtar üret`,
//! `surum_artefakti imzala` ve `scripts/imza-dogrula.sh` (OpenSSL Ed25519) zinciri; olumsuz
//! durumlar: değişmiş ikili, değişmiş SHA256SUMS, sabit olmayan yayıncı anahtarı.
//! Kabuk zinciri POSIX `bash` ister (Windows'ta `bash` WSL'e çözülür); yayın doğrulaması
//! Linux yayın işinde koşar, tutarlılık denetimi her platformda.
#![cfg_attr(not(unix), allow(dead_code, unused_imports))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn depo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü")
        .to_path_buf()
}

fn anahtar_alanlari(anahtar: &Path) -> (String, String) {
    let metin = std::fs::read_to_string(anahtar).expect("anahtar okunmalı");
    let alan = |on: &str| {
        metin
            .lines()
            .find_map(|satir| satir.strip_prefix(on))
            .expect("anahtar alanı")
            .to_string()
    };
    (alan("kimlik "), alan("açık "))
}

fn imzala(klasor: &Path, anahtar: &Path) {
    let cikti = Command::new(env!("CARGO_BIN_EXE_surum_artefakti"))
        .args(["imzala", "--anahtar"])
        .arg(anahtar)
        .arg(klasor.join("SHA256SUMS"))
        .output()
        .expect("surum_artefakti çalışmalı");
    assert!(
        cikti.status.success(),
        "{}",
        String::from_utf8_lossy(&cikti.stderr)
    );
    std::fs::write(klasor.join("SHA256SUMS.zee-imza"), cikti.stdout).expect("imza yazılmalı");
}

fn dogrula(klasor: &Path, sabit: &Path) -> (bool, String) {
    let cikti = Command::new("bash")
        .arg(depo().join("scripts/imza-dogrula.sh"))
        .arg(klasor)
        .arg("--anahtar-dosyasi")
        .arg(sabit)
        .output()
        .expect("bash çalışmalı");
    (
        cikti.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&cikti.stdout),
            String::from_utf8_lossy(&cikti.stderr)
        ),
    )
}

fn anahtar_uret(yol: &Path) {
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(["anahtar", "üret"])
        .arg(yol)
        .output()
        .expect("dil çalışmalı");
    assert!(
        cikti.status.success(),
        "{}",
        String::from_utf8_lossy(&cikti.stderr)
    );
}

#[cfg(unix)]
#[test]
fn resmi_imza_dogrulanir_degisiklik_ve_yabanci_anahtar_reddedilir() {
    let kok_yolu = std::env::temp_dir().join(format!("zee-yayin-imzasi-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&kok_yolu);
    std::fs::create_dir_all(&kok_yolu).unwrap();
    struct Temizle(PathBuf);
    impl Drop for Temizle {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _temizle = Temizle(kok_yolu.clone());
    let kok = kok_yolu.as_path();
    let klasor = kok.join("surum");
    std::fs::create_dir(&klasor).unwrap();
    std::fs::write(klasor.join("dil"), b"derleyici").unwrap();
    std::fs::write(klasor.join("dillsp"), b"dil sunucusu").unwrap();
    let sha = |veri: &[u8]| {
        use sha2::{Digest, Sha256};
        Sha256::digest(veri)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    let toplam = format!(
        "{}  dil\n{}  dillsp\n",
        sha(b"derleyici"),
        sha(b"dil sunucusu")
    );
    std::fs::write(klasor.join("SHA256SUMS"), &toplam).unwrap();

    let anahtar = kok.join("yayinci.zee-anahtar");
    anahtar_uret(&anahtar);
    let (kimlik, acik) = anahtar_alanlari(&anahtar);
    let sabit = kok.join("yayinci-anahtari.txt");
    std::fs::write(&sabit, format!("# test\nkimlik {kimlik}\nacik {acik}\n")).unwrap();
    imzala(&klasor, &anahtar);
    let (gecti, cikti) = dogrula(&klasor, &sabit);
    assert!(gecti, "geçerli imza reddedildi: {cikti}");
    assert!(cikti.contains(&kimlik));

    std::fs::write(klasor.join("dil"), b"degismis").unwrap();
    let (gecti, cikti) = dogrula(&klasor, &sabit);
    assert!(!gecti && cikti.contains("özeti uyuşmuyor"), "{cikti}");
    std::fs::write(klasor.join("dil"), b"derleyici").unwrap();

    std::fs::write(klasor.join("SHA256SUMS"), format!("{toplam}00  ek\n")).unwrap();
    let (gecti, cikti) = dogrula(&klasor, &sabit);
    assert!(!gecti && cikti.contains("imzası geçersiz"), "{cikti}");
    std::fs::write(klasor.join("SHA256SUMS"), &toplam).unwrap();

    let yabanci = kok.join("yabanci.zee-anahtar");
    anahtar_uret(&yabanci);
    imzala(&klasor, &yabanci);
    let (gecti, cikti) = dogrula(&klasor, &sabit);
    assert!(
        !gecti && cikti.contains("resmî yayıncı anahtarıyla atılmamış"),
        "{cikti}"
    );
}

#[test]
fn sabit_yayinci_anahtari_ve_yayin_akisi_tutarli() {
    let sabit = std::fs::read_to_string(depo().join("docs/yayinci-anahtari-v1.txt")).unwrap();
    let kimlik = sabit
        .lines()
        .find_map(|s| s.strip_prefix("kimlik "))
        .unwrap();
    let acik = sabit.lines().find_map(|s| s.strip_prefix("acik ")).unwrap();
    assert_eq!(acik.len(), 64);
    use sha2::{Digest, Sha256};
    let bayt = (0..32)
        .map(|i| u8::from_str_radix(&acik[i * 2..i * 2 + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let ozet = Sha256::digest(&bayt)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert_eq!(
        kimlik,
        format!("sha256:{ozet}"),
        "kimlik açık anahtarın SHA-256'sı olmalı"
    );
    let akis = std::fs::read_to_string(depo().join(".github/workflows/surum-adayi.yml")).unwrap();
    assert!(
        akis.contains("bash scripts/imza-dogrula.sh"),
        "yayın imzayı doğrulamalı"
    );
    assert!(
        akis.contains("imzasız; resmî yayın yapılmaz"),
        "imzasız yayın reddedilmeli"
    );
    assert!(akis.contains("startsWith(github.ref, 'refs/tags/v')"));
}
