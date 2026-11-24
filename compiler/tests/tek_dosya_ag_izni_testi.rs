//! K-387/GB: bildirimsiz tek dosya kipinde dış ağ (`ağ`) varsayılan kapalıdır;
//! yalnız açık `--izin ağ` bayrağıyla açılır. `dene` ve IO izi oynatma gerçek
//! ağa çıkmadığı için eski tek dosya profilini korur; proje kipinde bayrak reddedilir.

use dil::yetkinlik::{Yetkinlik, YetkinlikPolitikasi};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const AG: &str = "cevap \"https://example.invalid/durum\" adresinden gelen yanıt olsun\ncevabın durum kodu yaz\n";
/// Ağ çağrısı yalnız çağrılmayan işlemde: derleme ağ ister, koşu ağa çıkmaz.
const OLU_AG: &str = "işlem durum getir\n    Metin döndürür\n    cevap \"https://example.invalid/x\" adresinden gelen yanıt olsun\n    cevabın gövdesi döndür\n\n\"ana program\" yaz\n";

fn klasor(ad: &str) -> PathBuf {
    let yol = std::env::temp_dir().join(format!("zee-k387-{ad}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&yol);
    std::fs::create_dir_all(&yol).unwrap();
    yol
}

fn dil(klasor: &Path, argumanlar: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(argumanlar)
        .current_dir(klasor)
        .output()
        .expect("dil çalışmalı")
}

fn stderr(cikti: &Output) -> String {
    String::from_utf8_lossy(&cikti.stderr).into_owned()
}

#[test]
fn tek_dosya_profili_ag_acmaz_bayrakli_profil_eski_siniri_korur() {
    let kapali = YetkinlikPolitikasi::gelistirici();
    assert!(!kapali.izin_verir(Yetkinlik::Ag));
    assert!(kapali.bildirimsiz());
    assert!(kapali.ag_istegini_denetle("https://example.com/").is_err());
    // Yerel adaptörler geriye uyumlu açık kalır.
    for yetkinlik in [
        Yetkinlik::DosyaOkuma,
        Yetkinlik::DosyaYazma,
        Yetkinlik::AgSunucusu,
    ] {
        assert!(kapali.izin_verir(yetkinlik), "{yetkinlik:?}");
    }
    let acik = YetkinlikPolitikasi::gelistirici_agli();
    assert!(acik.ag_istegini_denetle("https://example.com/").is_ok());
    assert!(acik
        .ag_istegini_denetle("smtps://mail.example.com:465")
        .is_ok());
    assert!(acik.ag_istegini_denetle("http://example.com/").is_err());
    assert!(!acik.izin_verir(Yetkinlik::YerelAg));
    assert!(!acik.izin_verir(Yetkinlik::Veritabani));
}

#[test]
fn calistir_ve_denetle_bayraksiz_t054_bayrakla_gecer() {
    let k = klasor("calistir");
    std::fs::write(k.join("ag.dil"), AG).unwrap();
    for argumanlar in [&["çalıştır", "ag.dil"][..], &["denetle", "ag.dil"][..]] {
        let cikti = dil(&k, argumanlar);
        let hata = format!(
            "{}{}",
            stderr(&cikti),
            String::from_utf8_lossy(&cikti.stdout)
        );
        assert_eq!(cikti.status.code(), Some(1), "{argumanlar:?}: {hata}");
        assert!(hata.contains("T054"), "{hata}");
        assert!(
            hata.contains("--izin ağ"),
            "öneri bayrağı göstermeli: {hata}"
        );
    }
    let cikti = dil(&k, &["denetle", "--izin", "ağ", "ag.dil"]);
    assert_eq!(cikti.status.code(), Some(0), "{}", stderr(&cikti));
    std::fs::write(k.join("olu.dil"), OLU_AG).unwrap();
    let cikti = dil(&k, &["çalıştır", "--izin", "ağ", "olu.dil"]);
    assert_eq!(cikti.status.code(), Some(0), "{}", stderr(&cikti));
    assert_eq!(String::from_utf8_lossy(&cikti.stdout), "ana program\n");
    // Dosyadan sonraki bayrak programın argümanıdır; yetki genişletemez.
    let cikti = dil(&k, &["çalıştır", "olu.dil", "--izin", "ağ"]);
    assert_eq!(cikti.status.code(), Some(1), "{}", stderr(&cikti));
    assert!(stderr(&cikti).contains("T054"));
    let _ = std::fs::remove_dir_all(&k);
}

#[test]
fn bayrak_yanlis_deger_proje_cocuk_ve_dene_ile_reddedilir() {
    let k = klasor("ret");
    std::fs::write(k.join("olu.dil"), OLU_AG).unwrap();
    std::fs::create_dir_all(k.join("p/kaynak")).unwrap();
    std::fs::write(
        k.join("p/proje.dil"),
        "proje \"p\" olsun\nsürüm \"0.1.0\" olsun\nmorfoloji \"zee-tr-1\" olsun\ngiriş \"kaynak/ana.dil\" olsun\n",
    )
    .unwrap();
    std::fs::write(k.join("p/kaynak/ana.dil"), "\"x\" yaz\n").unwrap();
    for (argumanlar, parca) in [
        (
            &["çalıştır", "--izin", "dosya-yazma", "olu.dil"][..],
            "yalnız `ağ`",
        ),
        (&["çalıştır", "--izin"][..], "`ağ` değeri"),
        (
            &["dene", "--izin", "ağ", "olu.dil"][..],
            "yalnız `dil çalıştır`",
        ),
        (
            &["--güvenli", "çalıştır", "--izin", "ağ", "olu.dil"][..],
            "--güvenli",
        ),
        (
            &["çalıştır", "--izin", "ağ", "p"][..],
            "bildirimsiz tek dosya",
        ),
    ] {
        let cikti = dil(&k, argumanlar);
        assert_eq!(
            cikti.status.code(),
            Some(2),
            "{argumanlar:?}: {}",
            stderr(&cikti)
        );
        assert!(
            stderr(&cikti).contains(parca),
            "{argumanlar:?}: {}",
            stderr(&cikti)
        );
    }
    let _ = std::fs::remove_dir_all(&k);
}

#[test]
fn dene_ve_iz_oynatma_eski_tek_dosya_profilini_korur_kayit_bayrak_ister() {
    let k = klasor("iz");
    let testli =
        format!("{OLU_AG}\ntest \"sabit\"\n    toplam 2 olsun\n    toplam 2 ye eşit olmalı\n");
    std::fs::write(k.join("olu.dil"), testli).unwrap();
    let cikti = dil(&k, &["dene", "olu.dil"]);
    assert_eq!(cikti.status.code(), Some(0), "{}", stderr(&cikti));
    assert!(String::from_utf8_lossy(&cikti.stdout).contains("1 geçti"));

    let cikti = dil(&k, &["iz", "kaydet", "iz.json", "olu.dil"]);
    assert_eq!(cikti.status.code(), Some(1), "{}", stderr(&cikti));
    assert!(stderr(&cikti).contains("T054"));
    let cikti = dil(&k, &["iz", "kaydet", "--izin", "ağ", "iz.json", "olu.dil"]);
    assert_eq!(cikti.status.code(), Some(0), "{}", stderr(&cikti));
    let cikti = dil(&k, &["iz", "oynat", "iz.json", "olu.dil"]);
    assert_eq!(cikti.status.code(), Some(0), "{}", stderr(&cikti));
    assert_eq!(String::from_utf8_lossy(&cikti.stdout), "ana program\n");
    let _ = std::fs::remove_dir_all(&k);
}
