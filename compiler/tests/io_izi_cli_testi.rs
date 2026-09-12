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

#[test]
#[cfg(unix)]
fn cli_iz_izni_acik_umask_ve_eski_dosyada_ozeldir() {
    use std::os::unix::fs::PermissionsExt;
    let klasor = gecici_klasor("ozel-izin");
    let kaynak = klasor.join("program.dil");
    std::fs::write(
        &kaynak,
        include_str!("../../regression/runtime/ozel-io-izi.dil"),
    )
    .unwrap();
    let iz = klasor.join("kosu.zee-io-izi");
    for mevcut in [false, true] {
        if mevcut {
            std::fs::set_permissions(&iz, std::fs::Permissions::from_mode(0o666)).unwrap();
        }
        let sonuc = Command::new("sh")
            .args([
                "-c",
                "umask 000; exec \"$@\"",
                "zee-test",
                dil(),
                "iz",
                "kaydet",
            ])
            .arg(&iz)
            .arg(&kaynak)
            .output()
            .unwrap();
        assert!(
            sonuc.status.success(),
            "{}",
            String::from_utf8_lossy(&sonuc.stderr)
        );
        assert_eq!(
            std::fs::metadata(&iz).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    std::fs::remove_dir_all(klasor).unwrap();
}

#[test]
#[cfg(unix)]
fn cli_iz_symlink_hedefini_ezmez() {
    let klasor = gecici_klasor("symlink");
    let kaynak = klasor.join("program.dil");
    let kurban = klasor.join("dokunma.txt");
    let iz = klasor.join("kosu.zee-io-izi");
    std::fs::write(
        &kaynak,
        include_str!("../../regression/runtime/ozel-io-izi.dil"),
    )
    .unwrap();
    std::fs::write(&kurban, "korunacak").unwrap();
    std::os::unix::fs::symlink(&kurban, &iz).unwrap();
    let sonuc = calistir(&[&iz, &kaynak], &["iz", "kaydet"]);
    assert!(!sonuc.status.success());
    assert_eq!(std::fs::read_to_string(&kurban).unwrap(), "korunacak");
    assert!(std::fs::symlink_metadata(&iz)
        .unwrap()
        .file_type()
        .is_symlink());
    std::fs::remove_dir_all(klasor).unwrap();
}

#[test]
#[cfg(target_os = "macos")]
fn cli_iz_miras_acl_yetkilerini_tasimaz() {
    let klasor = gecici_klasor("acl");
    let kaynak = klasor.join("program.dil");
    let iz = klasor.join("kosu.zee-io-izi");
    std::fs::write(
        &kaynak,
        include_str!("../../regression/runtime/ozel-io-izi.dil"),
    )
    .unwrap();
    let sonuc = Command::new("chmod")
        .args(["+a", "everyone allow read,file_inherit"])
        .arg(&klasor)
        .status()
        .unwrap();
    assert!(sonuc.success());
    let sonuc = calistir(&[&iz, &kaynak], &["iz", "kaydet"]);
    assert!(
        sonuc.status.success(),
        "{}",
        String::from_utf8_lossy(&sonuc.stderr)
    );
    let liste = Command::new("ls").arg("-le").arg(&iz).output().unwrap();
    assert!(!String::from_utf8_lossy(&liste.stdout).contains("everyone"));
    std::fs::remove_dir_all(klasor).unwrap();
}

#[test]
#[cfg(windows)]
fn cli_iz_windows_dacl_yalniz_sahibine_aciktir() {
    let klasor = gecici_klasor("dacl");
    let kaynak = klasor.join("program.dil");
    let iz = klasor.join("kosu.zee-io-izi");
    std::fs::write(
        &kaynak,
        include_str!("../../regression/runtime/ozel-io-izi.dil"),
    )
    .unwrap();
    for mevcut in [false, true] {
        if mevcut {
            std::fs::remove_file(&iz).unwrap();
            std::fs::write(&iz, "eski").unwrap();
        }
        let sonuc = calistir(&[&iz, &kaynak], &["iz", "kaydet"]);
        assert!(
            sonuc.status.success(),
            "{}",
            String::from_utf8_lossy(&sonuc.stderr)
        );
        let sonuc = Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command",
            "$a=Get-Acl -LiteralPath $env:ZEE_TEST_IZ; if (!$a.AreAccessRulesProtected) {exit 1}; $r=@($a.Access); if ($r.Count -ne 1) {exit 2}; if ($r[0].IdentityReference.Translate([System.Security.Principal.SecurityIdentifier]).Value -ne 'S-1-3-4') {exit 3}"])
            .env("ZEE_TEST_IZ", &iz).output().unwrap();
        assert!(
            sonuc.status.success(),
            "{}",
            String::from_utf8_lossy(&sonuc.stderr)
        );
    }
    std::fs::remove_dir_all(klasor).unwrap();
}
