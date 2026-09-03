use dil::artefakt_dogrulama::{anahtar_uret, yayini_dogrula};
use dil::yayin::paketle_zamanla;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static SAYAC: AtomicU64 = AtomicU64::new(0);

struct GeciciKlasor(PathBuf);

impl GeciciKlasor {
    fn yeni() -> Self {
        let sira = SAYAC.fetch_add(1, Ordering::Relaxed);
        let yol = std::env::temp_dir().join(format!("zee-tedarik-{}-{}", std::process::id(), sira));
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

fn proje_yaz(kok: &Path, yerel_bagimlilik: bool) {
    std::fs::create_dir(kok).expect("proje klasörü");
    let bagimlilik = if yerel_bagimlilik {
        "yerel_bağımlılıklar \"../ortak\" listesi olsun\n"
    } else {
        "yerel_bağımlılıklar boş liste olsun\n"
    };
    std::fs::write(
        kok.join("proje.dil"),
        format!(
            "proje \"miras\" olsun\nsürüm \"1.2.3\" olsun\nmorfoloji \"zee-tr-1\" olsun\ngiriş \"kaynak/ana.dil\" olsun\n{}",
            bagimlilik
        ),
    )
    .expect("bildirim");
    std::fs::create_dir(kok.join("kaynak")).expect("kaynak klasörü");
    std::fs::write(
        kok.join("kaynak/ana.dil"),
        "işlem sevgiyi söyle\n    Metin döndürür\n    \"daima\" döndür\n",
    )
    .expect("kaynak");
}

fn dosyalar(cikti: &dil::paket_modeli::PaketCiktilari) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    (
        std::fs::read(&cikti.yayin).expect("yayın"),
        std::fs::read(&cikti.paket).expect("paket"),
        std::fs::read(&cikti.sbom).expect("sbom"),
        std::fs::read(&cikti.provenance).expect("provenance"),
    )
}

fn hex_yaz(baytlar: &[u8]) -> String {
    const BASAMAK: &[u8; 16] = b"0123456789abcdef";
    let mut sonuc = String::with_capacity(baytlar.len() * 2);
    for bayt in baytlar {
        sonuc.push(BASAMAK[(bayt >> 4) as usize] as char);
        sonuc.push(BASAMAK[(bayt & 0x0f) as usize] as char);
    }
    sonuc
}

#[test]
fn kaynak_paketi_sbom_provenance_ve_imza_byte_byte_tekrar_uretilir() {
    let gecici = GeciciKlasor::yeni();
    let proje = gecici.yol().join("proje");
    let anahtar = gecici.yol().join("yayinci.anahtar");
    proje_yaz(&proje, false);
    let kimlik = anahtar_uret(&anahtar).expect("anahtar");

    let ilk = paketle_zamanla(&proje, &anahtar, &gecici.yol().join("ilk"), 1_700_000_000)
        .expect("ilk paket");
    let ikinci = paketle_zamanla(
        &proje,
        &anahtar,
        &gecici.yol().join("ikinci"),
        1_700_000_000,
    )
    .expect("ikinci paket");
    let ilk_dosyalar = dosyalar(&ilk);
    let ikinci_dosyalar = dosyalar(&ikinci);
    assert_eq!(ilk_dosyalar, ikinci_dosyalar);
    assert_eq!(ilk.yayinci_anahtar_kimligi, kimlik);

    let imzali = yayini_dogrula(
        &ilk_dosyalar.0,
        &ilk_dosyalar.1,
        &ilk_dosyalar.2,
        &ilk_dosyalar.3,
    )
    .expect("zincir doğrulanmalı");
    assert_eq!(imzali.paket, "miras");
    assert_eq!(imzali.surum, "1.2.3");
    assert_eq!(imzali.morfoloji, "zee-tr-1");
    assert!(String::from_utf8(ilk_dosyalar.2)
        .expect("sbom utf8")
        .contains("https://spdx.org/rdf/3.0.1/spdx-context.jsonld"));
    assert!(String::from_utf8(ilk_dosyalar.3)
        .expect("provenance utf8")
        .contains("https://slsa.dev/provenance/v1"));
}

#[test]
fn kanonik_zep_fixture_i_butun_tier1_platformlarda_ayni_bayttir() {
    let gecici = GeciciKlasor::yeni();
    let proje = gecici.yol().join("proje");
    let anahtar = gecici.yol().join("yayinci.anahtar");
    proje_yaz(&proje, false);
    std::fs::write(
        proje.join("kaynak/çağrı.dil"),
        "işlem anıyı taşı\n    Metin döndürür\n    \"Eliz\" döndür\n",
    )
    .expect("Unicode fixture kaynağı");
    anahtar_uret(&anahtar).expect("anahtar");

    let cikti =
        paketle_zamanla(&proje, &anahtar, &gecici.yol().join("cikti"), 0).expect("fixture paketi");
    let gercek = std::fs::read(&cikti.paket).expect("fixture .zep");
    let beklenen = include_str!("fixtures/zep-kanonik-v1.hex").trim();
    assert_eq!(
        hex_yaz(&gercek),
        beklenen,
        ".zep v1 kanonik fixture değişti; biçim değişikliği RFC/spec ister"
    );
}

#[test]
fn imza_arsiv_sbom_ve_provenance_oynamalari_fail_closed_reddedilir() {
    let gecici = GeciciKlasor::yeni();
    let proje = gecici.yol().join("proje");
    let anahtar = gecici.yol().join("yayinci.anahtar");
    proje_yaz(&proje, false);
    anahtar_uret(&anahtar).expect("anahtar");
    let cikti = paketle_zamanla(&proje, &anahtar, &gecici.yol().join("cikti"), 0).expect("paket");
    let (yayin, arsiv, sbom, provenance) = dosyalar(&cikti);

    let mut bozuk_yayin: serde_json::Value = serde_json::from_slice(&yayin).expect("json");
    bozuk_yayin["imzali"]["surum"] = serde_json::Value::String("9.9.9".into());
    let bozuk_yayin = serde_json::to_vec(&bozuk_yayin).expect("json");
    assert!(yayini_dogrula(&bozuk_yayin, &arsiv, &sbom, &provenance)
        .unwrap_err()
        .contains("imzası"));

    let mut bozuk_arsiv = arsiv.clone();
    *bozuk_arsiv.last_mut().expect("byte") ^= 1;
    assert!(yayini_dogrula(&yayin, &bozuk_arsiv, &sbom, &provenance)
        .unwrap_err()
        .contains("SHA-256"));

    let mut bozuk_sbom = sbom.clone();
    bozuk_sbom.push(b' ');
    assert!(yayini_dogrula(&yayin, &arsiv, &bozuk_sbom, &provenance)
        .unwrap_err()
        .contains("SHA-256"));

    let mut bozuk_provenance = provenance.clone();
    bozuk_provenance.push(b' ');
    assert!(yayini_dogrula(&yayin, &arsiv, &sbom, &bozuk_provenance)
        .unwrap_err()
        .contains("SHA-256"));
}

#[test]
fn anahtar_ezilmez_yerel_bagimlilik_ve_sembolik_bag_yayinlanmaz() {
    let gecici = GeciciKlasor::yeni();
    let anahtar = gecici.yol().join("yayinci.anahtar");
    anahtar_uret(&anahtar).expect("ilk anahtar");
    let ilk = std::fs::read(&anahtar).expect("ilk içerik");
    assert!(anahtar_uret(&anahtar)
        .unwrap_err()
        .contains("üzerine yazılmadı"));
    assert_eq!(std::fs::read(&anahtar).expect("korunan içerik"), ilk);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&anahtar)
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    let bagimli = gecici.yol().join("bagimli");
    proje_yaz(&bagimli, true);
    assert!(
        paketle_zamanla(&bagimli, &anahtar, &gecici.yol().join("x"), 0)
            .unwrap_err()
            .contains("Yerel yol bağımlılığı")
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let proje = gecici.yol().join("bagli");
        proje_yaz(&proje, false);
        symlink("kaynak/ana.dil", proje.join("kaçış.dil")).expect("sembolik bağ");
        assert!(
            paketle_zamanla(&proje, &anahtar, &gecici.yol().join("y"), 0)
                .unwrap_err()
                .contains("sembolik bağ")
        );
    }
}

#[test]
fn derlenmeyen_ve_girisi_eksik_proje_yayin_ciktisi_birakmaz() {
    let gecici = GeciciKlasor::yeni();
    let anahtar = gecici.yol().join("yayinci.anahtar");
    anahtar_uret(&anahtar).expect("anahtar");

    let bozuk = gecici.yol().join("bozuk");
    proje_yaz(&bozuk, false);
    std::fs::write(bozuk.join("kaynak/ana.dil"), "bu bir zee cümlesi değil\n")
        .expect("bozuk kaynak");
    let bozuk_cikti = gecici.yol().join("bozuk-cikti");
    let hata = paketle_zamanla(&bozuk, &anahtar, &bozuk_cikti, 0).unwrap_err();
    assert!(hata.contains("Paket kaynağı S004"), "{}", hata);
    assert!(!bozuk_cikti.exists());

    let eksik = gecici.yol().join("eksik");
    proje_yaz(&eksik, false);
    std::fs::remove_file(eksik.join("kaynak/ana.dil")).expect("giriş silinir");
    let eksik_cikti = gecici.yol().join("eksik-cikti");
    let hata = paketle_zamanla(&eksik, &anahtar, &eksik_cikti, 0).unwrap_err();
    assert!(hata.contains("P009"), "{}", hata);
    assert!(!eksik_cikti.exists());
}

#[test]
fn cli_anahtar_ve_paketle_akisini_turkce_sunar() {
    let gecici = GeciciKlasor::yeni();
    let proje = gecici.yol().join("proje");
    let anahtar = gecici.yol().join("yayinci.anahtar");
    let cikti = gecici.yol().join("yayin");
    proje_yaz(&proje, false);
    let ikili = env!("CARGO_BIN_EXE_dil");

    let uret = Command::new(ikili)
        .args(["anahtar", "üret", anahtar.to_str().expect("utf8")])
        .output()
        .expect("anahtar komutu");
    assert!(
        uret.status.success(),
        "{}",
        String::from_utf8_lossy(&uret.stderr)
    );
    assert!(String::from_utf8_lossy(&uret.stdout).contains("Açık anahtar kimliği"));

    let paketle = Command::new(ikili)
        .env("SOURCE_DATE_EPOCH", "1700000000")
        .args([
            "paketle",
            proje.to_str().expect("utf8"),
            "--anahtar",
            anahtar.to_str().expect("utf8"),
            "--çıktı",
            cikti.to_str().expect("utf8"),
        ])
        .output()
        .expect("paketle komutu");
    assert!(
        paketle.status.success(),
        "{}",
        String::from_utf8_lossy(&paketle.stderr)
    );
    let stdout = String::from_utf8_lossy(&paketle.stdout);
    assert!(stdout.contains("Paketlendi:"));
    assert!(stdout.contains("SBOM:"));
    assert!(stdout.contains("Provenance:"));
    assert!(cikti.join("miras-1.2.3.zep").is_file());
    assert!(cikti.join("miras-1.2.3.zee-yayin.json").is_file());
}
