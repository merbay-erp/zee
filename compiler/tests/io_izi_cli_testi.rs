//! `dil iz kaydet/oynat` uçtan uca CLI kanıtları.

use std::path::{Path, PathBuf};
use std::process::Command;

fn gecici_klasor(ad: &str) -> PathBuf {
    let benzersiz = format!(
        "zee-io-izi-{}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test"),
        ad
    );
    let yol = std::env::temp_dir().join(benzersiz);
    let _ = std::fs::remove_dir_all(&yol);
    std::fs::create_dir_all(&yol).expect("geçici klasör oluşturulmalı");
    yol
}

fn dil() -> &'static str {
    env!("CARGO_BIN_EXE_dil")
}

fn calistir(argumanlar: &[&Path], metinler: &[&str]) -> std::process::Output {
    let mut komut = Command::new(dil());
    for metin in metinler {
        komut.arg(metin);
    }
    for arguman in argumanlar {
        komut.arg(arguman);
    }
    komut.output().expect("dil süreci çalışmalı")
}

#[test]
fn cli_kaydi_dis_dunyasiz_ayni_ciktiyla_oynatir() {
    let klasor = gecici_klasor("roundtrip");
    let kaynak = klasor.join("program.dil");
    let iz = klasor.join("kosu.zee-io-izi");
    std::fs::write(
        &kaynak,
        "gelenler komut satırından gelenler olsun\nher gelen için\n    geleni yaz\nzar 1 ile 100 arasında rastgele sayı olsun\nzar yaz\n",
    )
    .expect("kaynak yazılmalı");

    let kayit = Command::new(dil())
        .args(["iz", "kaydet"])
        .arg(&iz)
        .arg(&kaynak)
        .arg("Zeynep")
        .output()
        .expect("kayıt çalışmalı");
    assert!(
        kayit.status.success(),
        "{}",
        String::from_utf8_lossy(&kayit.stderr)
    );
    assert!(String::from_utf8_lossy(&kayit.stderr).contains("IO izi kaydedildi"));
    assert!(std::fs::read_to_string(&iz)
        .expect("iz okunmalı")
        .starts_with("zee-io-izi\t1\n"));

    let tekrar = Command::new(dil())
        .args(["iz", "oynat"])
        .arg(&iz)
        .arg(&kaynak)
        .output()
        .expect("replay çalışmalı");
    assert!(
        tekrar.status.success(),
        "{}",
        String::from_utf8_lossy(&tekrar.stderr)
    );
    assert_eq!(tekrar.stdout, kayit.stdout);

    std::fs::remove_dir_all(klasor).expect("geçici klasör temizlenmeli");
}

#[test]
fn cli_kaynak_degisimini_ve_kaynak_uzerine_izi_reddeder() {
    let klasor = gecici_klasor("uyusmazlik");
    let kaynak = klasor.join("program.dil");
    let iz = klasor.join("kosu.zee-io-izi");
    std::fs::write(
        &kaynak,
        "zar 1 ile 6 arasında rastgele sayı olsun\nzar yaz\n",
    )
    .expect("kaynak yazılmalı");

    let kayit = calistir(&[&iz, &kaynak], &["iz", "kaydet"]);
    assert!(
        kayit.status.success(),
        "{}",
        String::from_utf8_lossy(&kayit.stderr)
    );

    std::fs::write(
        &kaynak,
        "zar 1 ile 10 arasında rastgele sayı olsun\nzar yaz\n",
    )
    .expect("kaynak değişmeli");
    let tekrar = calistir(&[&iz, &kaynak], &["iz", "oynat"]);
    assert!(!tekrar.status.success());
    assert!(String::from_utf8_lossy(&tekrar.stderr).contains("replay uyuşmazlığı"));

    let ezme = calistir(&[&kaynak, &kaynak], &["iz", "kaydet"]);
    assert!(!ezme.status.success());
    assert!(String::from_utf8_lossy(&ezme.stderr).contains("üzerine yazılamaz"));

    std::fs::remove_dir_all(klasor).expect("geçici klasör temizlenmeli");
}
