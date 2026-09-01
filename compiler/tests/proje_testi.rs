//! Proje modeli (K-076): proje.dil bildirimi ve klasör-temelli CLI akışı.

use dil::proje::{bildirimi_oku, ProjeBildirimi};
use std::path::{Path, PathBuf};
use std::process::Command;

struct GeciciKlasor(PathBuf);

impl GeciciKlasor {
    fn yeni() -> Self {
        let benzersiz = format!(
            "zee-proje-testi-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("saat")
                .as_nanos()
        );
        let yol = std::env::temp_dir().join(benzersiz);
        std::fs::create_dir(&yol).expect("geçici klasör");
        Self(yol)
    }

    fn yol(&self) -> &Path {
        &self.0
    }
}

impl Drop for GeciciKlasor {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn bildirim_gecerli_zee_kaynagidir() {
    let kaynak =
        "proje \"stok-paneli\" olsun\nsürüm \"1.2.3\" olsun\ngiriş \"kaynak/ana.dil\" olsun\n";
    assert_eq!(
        bildirimi_oku(kaynak).expect("bildirim geçmeli"),
        ProjeBildirimi {
            ad: "stok-paneli".into(),
            surum: "1.2.3".into(),
            giris: "kaynak/ana.dil".into(),
        }
    );
}

#[test]
fn bildirim_yalniz_tanimli_alanlari_kabul_eder() {
    let kaynak = "proje \"x\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nlisans \"özel\" olsun\n";
    assert_eq!(
        bildirimi_oku(kaynak).expect_err("alan reddedilmeli").kod,
        "P001"
    );
}

#[test]
fn eksik_alan_ogretici_tanidir() {
    let kaynak = "proje \"x\" olsun\ngiriş \"ana.dil\" olsun\n";
    let hata = bildirimi_oku(kaynak).expect_err("sürüm eksik");
    assert_eq!(hata.kod, "P002");
    assert!(hata.mesaj.contains("sürüm"));
}

#[test]
fn surum_uc_sayili_ve_giris_guvenli_olmali() {
    let kotu_surum = "proje \"x\" olsun\nsürüm \"v1\" olsun\ngiriş \"ana.dil\" olsun\n";
    assert_eq!(
        bildirimi_oku(kotu_surum)
            .expect_err("sürüm reddedilmeli")
            .kod,
        "P003"
    );

    let kotu_giris = "proje \"x\" olsun\nsürüm \"1.0.0\" olsun\ngiriş \"../ana.dil\" olsun\n";
    assert_eq!(
        bildirimi_oku(kotu_giris)
            .expect_err("kaçış reddedilmeli")
            .kod,
        "P004"
    );
}

#[test]
fn cli_proje_klasorunu_calistirir_denetler_ve_dener() {
    let gecici = GeciciKlasor::yeni();
    std::fs::write(
        gecici.yol().join("proje.dil"),
        "proje \"hesap\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n",
    )
    .expect("bildirim");
    std::fs::write(
        gecici.yol().join("yardimci.dil"),
        "işlem iki katını bul\n    sayıyı al\n    sonucu sayı ile 2 nin çarpımı olsun\n    sonucu döndür\n",
    )
    .expect("birim");
    std::fs::write(
        gecici.yol().join("ana.dil"),
        "yardimci birimini kullan\n\n\"veri.txt\" dosyasına \"projenin verisi\" yaz\nsonuç 21 için iki katını bul olsun\nsonucu yaz\n\ntest \"iki katı\"\n    x 3 için iki katını bul olsun\n    x 6 ya eşit olmalı\n",
    )
    .expect("giriş");

    let ikili = env!("CARGO_BIN_EXE_dil");
    let calistir = Command::new(ikili)
        .arg("çalıştır")
        .arg(gecici.yol())
        .output()
        .expect("çalıştır");
    assert!(
        calistir.status.success(),
        "{}",
        String::from_utf8_lossy(&calistir.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&calistir.stdout), "42\n");
    assert_eq!(
        std::fs::read_to_string(gecici.yol().join("veri.txt")).expect("veri proje kökünde"),
        "projenin verisi\n"
    );

    let denetle = Command::new(ikili)
        .arg("denetle")
        .arg(gecici.yol())
        .output()
        .expect("denetle");
    assert!(
        denetle.status.success(),
        "{}",
        String::from_utf8_lossy(&denetle.stderr)
    );

    let dene = Command::new(ikili)
        .arg("dene")
        .arg(gecici.yol())
        .output()
        .expect("dene");
    assert!(
        dene.status.success(),
        "{}",
        String::from_utf8_lossy(&dene.stderr)
    );
    assert!(String::from_utf8_lossy(&dene.stdout).contains("1 test: 1 geçti, 0 kaldı"));
}

#[test]
fn yeni_komutu_proje_bildirimi_uretir() {
    let gecici = GeciciKlasor::yeni();
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .current_dir(gecici.yol())
        .args(["yeni", "ilk-projem"])
        .output()
        .expect("yeni");
    assert!(
        cikti.status.success(),
        "{}",
        String::from_utf8_lossy(&cikti.stderr)
    );

    let proje = gecici.yol().join("ilk-projem");
    let bildirim = std::fs::read_to_string(proje.join("proje.dil")).expect("proje.dil");
    assert_eq!(
        bildirimi_oku(&bildirim).expect("üretilen bildirim").ad,
        "ilk-projem"
    );

    let dene = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("dene")
        .arg(&proje)
        .output()
        .expect("üretilen projeyi dene");
    assert!(
        dene.status.success(),
        "{}",
        String::from_utf8_lossy(&dene.stderr)
    );
}

#[test]
fn guvenli_bayragi_program_argumanlarina_sizmaz() {
    let gecici = GeciciKlasor::yeni();
    std::fs::write(
        gecici.yol().join("proje.dil"),
        "proje \"arguman\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n",
    )
    .expect("bildirim");
    std::fs::write(
        gecici.yol().join("ana.dil"),
        "gelenler komut satırından gelenler olsun\nher gelen için\n    geleni yaz\n",
    )
    .expect("giriş");

    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("çalıştır")
        .arg(gecici.yol())
        .arg("--güvenli")
        .arg("merhaba")
        .output()
        .expect("güvenli çalıştır");
    assert!(
        cikti.status.success(),
        "{}",
        String::from_utf8_lossy(&cikti.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&cikti.stdout), "merhaba\n");
}

#[test]
fn bicimle_komutu_projenin_butun_kaynaklarini_bicimler() {
    let gecici = GeciciKlasor::yeni();
    std::fs::write(
        gecici.yol().join("proje.dil"),
        "proje    \"biçim\"    olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n",
    )
    .expect("bildirim");
    std::fs::write(gecici.yol().join("ana.dil"), "\"merhaba\"    yaz\n").expect("giriş");
    std::fs::create_dir(gecici.yol().join("kaynak")).expect("alt klasör");
    std::fs::write(
        gecici.yol().join("kaynak/yardimci.dil"),
        "işlem  bir ver\n    1   döndür\n",
    )
    .expect("birim");

    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("biçimle")
        .arg(gecici.yol())
        .output()
        .expect("biçimle");
    assert!(
        cikti.status.success(),
        "{}",
        String::from_utf8_lossy(&cikti.stderr)
    );
    assert!(
        String::from_utf8_lossy(&cikti.stdout).contains("3 kaynak denetlendi; 3 dosya biçimlendi")
    );
    assert_eq!(
        std::fs::read_to_string(gecici.yol().join("ana.dil")).expect("giriş"),
        "\"merhaba\" yaz\n"
    );
    assert_eq!(
        std::fs::read_to_string(gecici.yol().join("kaynak/yardimci.dil")).expect("birim"),
        "işlem bir ver\n    1 döndür\n"
    );
}
