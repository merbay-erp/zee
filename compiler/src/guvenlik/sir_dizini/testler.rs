use super::*;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};

const SENTINEL: &str = "K339_SENTINEL_DEGER";

struct Dizin(PathBuf);
impl Dizin {
    fn yeni(etiket: &str) -> Self {
        let taban = std::env::temp_dir().canonicalize().unwrap();
        let yol = taban.join(format!("zee-k339-{etiket}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&yol);
        std::fs::create_dir(&yol).unwrap();
        izin(&yol, 0o700);
        Dizin(yol)
    }
    fn yaz(&self, ad: &str, icerik: &[u8], kip: u32) -> PathBuf {
        let yol = self.0.join(ad);
        std::fs::write(&yol, icerik).unwrap();
        izin(&yol, kip);
        yol
    }
    fn os(&self) -> Option<OsString> {
        Some(self.0.clone().into_os_string())
    }
}
impl Drop for Dizin {
    fn drop(&mut self) {
        izin(&self.0, 0o700);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn izin(yol: &Path, kip: u32) {
    std::fs::set_permissions(yol, std::fs::Permissions::from_mode(kip)).unwrap();
}

fn oku(d: Option<OsString>, ad: &str, ortam: Option<&str>) -> Result<String, SirHatasi> {
    kaynaklardan_oku(
        ad,
        ortam.is_some(),
        ortam.map(str::to_string),
        d,
        64,
        SatirProfili::TekSatir,
    )
}

fn neden(sonuc: Result<String, SirHatasi>) -> &'static str {
    match sonuc {
        Err(SirHatasi::Gecersiz(neden)) => {
            assert!(!neden.contains(SENTINEL));
            assert!(!neden.contains("zee-k339"));
            neden
        }
        diger => panic!("Gecersiz bekleniyordu: {diger:?}"),
    }
}

#[test]
fn dizin_yoksa_davranis_yalniz_ortamdir() {
    assert_eq!(oku(None, "A", Some("x")), Ok("x".into()));
    assert_eq!(oku(None, "A", None), Err(SirHatasi::Yok));
    // Güvensiz ad bile eski davranışı korur; yol kurulmaz.
    assert_eq!(oku(None, "../A", Some("x")), Ok("x".into()));
}

#[test]
fn credential_dosyasi_okunur_ve_kirpilmaz() {
    let d = Dizin::yeni("olumlu");
    d.yaz(
        "UYGULAMA_SMTP_URL",
        format!(" {SENTINEL} ").as_bytes(),
        0o400,
    );
    assert_eq!(
        oku(d.os(), "UYGULAMA_SMTP_URL", None),
        Ok(format!(" {SENTINEL} "))
    );
    izin(&d.0, 0o500);
    assert_eq!(
        oku(d.os(), "UYGULAMA_SMTP_URL", None),
        Ok(format!(" {SENTINEL} "))
    );
    // 0600 dosya ve noktalı/tireli ad da kabul.
    izin(&d.0, 0o700);
    d.yaz("db.url-1", b"v", 0o600);
    assert_eq!(oku(d.os(), "db.url-1", None), Ok("v".into()));
}

#[test]
fn dizinde_yoksa_ortama_doner_ikisinde_de_yoksa_yok() {
    let d = Dizin::yeni("yok");
    assert_eq!(oku(d.os(), "A", Some("ortam")), Ok("ortam".into()));
    assert_eq!(oku(d.os(), "A", None), Err(SirHatasi::Yok));
}

#[test]
fn ayni_ad_iki_kaynakta_ise_ret() {
    let d = Dizin::yeni("cakisma");
    d.yaz("A", SENTINEL.as_bytes(), 0o400);
    assert!(neden(oku(d.os(), "A", Some("ortam"))).contains("hem ortamda"));
    // Boş ortam değeri de çakışmadır.
    assert!(neden(oku(d.os(), "A", Some(""))).contains("hem ortamda"));
}

#[test]
fn dizin_eksik_goreli_noktali_ya_da_sembolik_bag_ise_acik_hata() {
    let d = Dizin::yeni("dizin");
    let yok = d.0.join("olmayan").into_os_string();
    assert!(neden(oku(Some(yok), "A", Some("ortam"))).contains("açılamadı"));
    assert!(neden(oku(Some("goreli/dizin".into()), "A", None)).contains("mutlak"));
    assert!(neden(oku(Some("".into()), "A", None)).contains("mutlak"));
    let noktali = format!(
        "{}/../{}",
        d.0.display(),
        d.0.file_name().unwrap().to_str().unwrap()
    );
    assert!(neden(oku(Some(noktali.into()), "A", None)).contains("`..`"));
    let bag = d.0.with_extension("bag");
    let _ = std::fs::remove_file(&bag);
    symlink(&d.0, &bag).unwrap();
    d.yaz("A", b"v", 0o400);
    let sonuc = oku(Some(bag.clone().into_os_string()), "A", None);
    std::fs::remove_file(&bag).unwrap();
    assert!(neden(sonuc).contains("sembolik bağ"));
    let dosya = d.yaz("B", b"v", 0o400).into_os_string();
    assert!(neden(oku(Some(dosya), "A", None)).contains("açılamadı"));
}

#[test]
fn dizin_izni_genis_ise_ret() {
    let d = Dizin::yeni("dizin-izni");
    d.yaz("A", b"v", 0o400);
    for kip in [0o720, 0o702, 0o777] {
        izin(&d.0, kip);
        assert!(
            neden(oku(d.os(), "A", None)).contains("yazılabilir"),
            "{kip:o}"
        );
    }
}

#[test]
fn dosya_sembolik_bag_hard_link_fifo_ve_dizin_reddedilir() {
    let d = Dizin::yeni("dosya-turu");
    let hedef = d.yaz("hedef", SENTINEL.as_bytes(), 0o400);
    symlink(&hedef, d.0.join("BAG")).unwrap();
    assert!(neden(oku(d.os(), "BAG", None)).contains("sembolik bağ"));
    std::fs::hard_link(&hedef, d.0.join("IKI")).unwrap();
    assert!(neden(oku(d.os(), "IKI", None)).contains("hard-link"));
    let fifo = std::ffi::CString::new(d.0.join("FIFO").as_os_str().as_encoded_bytes()).unwrap();
    // SAFETY: geçerli NUL sonlu yol.
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    assert!(neden(oku(d.os(), "FIFO", None)).contains("düzenli dosya"));
    std::fs::create_dir(d.0.join("ALT")).unwrap();
    assert!(neden(oku(d.os(), "ALT", None)).contains("düzenli dosya"));
}

#[test]
fn dosya_izni_genis_ise_ret() {
    let d = Dizin::yeni("dosya-izni");
    for kip in [0o604, 0o644, 0o620, 0o666] {
        d.yaz("A", SENTINEL.as_bytes(), kip);
        assert!(
            neden(oku(d.os(), "A", None)).contains("credential dosyası"),
            "{kip:o}"
        );
        std::fs::remove_file(d.0.join("A")).unwrap();
    }
    // Grup okuma (ACL maskesi) kabul edilir.
    d.yaz("A", b"v", 0o440);
    assert_eq!(oku(d.os(), "A", None), Ok("v".into()));
}

#[test]
fn boyut_siniri_bos_ve_tam_sinir() {
    let d = Dizin::yeni("boyut");
    d.yaz("TAM", &[b'a'; 64], 0o400);
    assert_eq!(oku(d.os(), "TAM", None), Ok("a".repeat(64)));
    d.yaz("FAZLA", &[b'a'; 65], 0o400);
    assert!(neden(oku(d.os(), "FAZLA", None)).contains("boyut sınırını"));
    d.yaz("BOS", b"", 0o400);
    assert!(neden(oku(d.os(), "BOS", None)).contains("boş"));
}

#[test]
fn metin_profili_utf8_nul_bom_ve_satir_sonu() {
    let d = Dizin::yeni("metin");
    d.yaz("SATIR", format!("{SENTINEL}\n").as_bytes(), 0o400);
    assert!(neden(oku(d.os(), "SATIR", None)).contains("tek satır"));
    d.yaz("CR", b"a\rb", 0o400);
    assert!(neden(oku(d.os(), "CR", None)).contains("tek satır"));
    d.yaz("UTF", &[0xff, 0xfe], 0o400);
    assert!(neden(oku(d.os(), "UTF", None)).contains("UTF-8"));
    d.yaz("BOM", b"\xEF\xBB\xBFa", 0o400);
    assert!(neden(oku(d.os(), "BOM", None)).contains("BOM"));
    d.yaz("NUL", b"a\0b", 0o400);
    assert!(neden(oku(d.os(), "NUL", None)).contains("NUL"));
    d.yaz("TURKCE", "şifre-ğüı".as_bytes(), 0o400);
    assert_eq!(oku(d.os(), "TURKCE", None), Ok("şifre-ğüı".into()));
    let pem = "-----BEGIN CERTIFICATE-----\nAAAA\n-----END CERTIFICATE-----\n";
    d.yaz("PEM", pem.as_bytes(), 0o400);
    assert_eq!(
        kaynaklardan_oku("PEM", false, None, d.os(), 256, SatirProfili::CokSatir),
        Ok(pem.into())
    );
    assert!(neden(oku(d.os(), "PEM", None)).contains("tek satır"));
}

#[test]
fn guvensiz_ad_dizinde_aranmaz() {
    let d = Dizin::yeni("ad");
    d.yaz("gizli", SENTINEL.as_bytes(), 0o400);
    for ad in [
        "../gizli",
        "a/b",
        ".gizli",
        "-x",
        "",
        "ş",
        "a b",
        &"a".repeat(256),
    ] {
        assert!(!ad_guvenli(ad), "{ad}");
        assert_eq!(oku(d.os(), ad, None), Err(SirHatasi::Yok), "{ad}");
        assert_eq!(oku(d.os(), ad, Some("o")), Ok("o".into()), "{ad}");
    }
    for ad in [
        "A",
        "_A",
        "UYGULAMA_DB_URL_TLS_CA_PEM",
        "a.b-c",
        &"a".repeat(255),
    ] {
        assert!(ad_guvenli(ad), "{ad}");
    }
}
