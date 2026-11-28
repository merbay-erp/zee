//! K-339/ADR-105 eki: native CLI proje gizli değerini systemd
//! `$CREDENTIALS_DIRECTORY` dosyasından okur. Gerçek dizin, gerçek `dil`
//! ikilisi; değer yalnız izinli program kendisi yazdığında stdout'a çıkar,
//! hiçbir ret iletisine ya da stderr'e girmez.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const HTTP_AD: &str = "ZEE_K339_HTTP_SECRET";
const ANAHTAR_AD: &str = "ZEE_K339_ANAHTAR";
const SENTINEL: &str = "K339_E2E_SENTINEL";

struct Proje {
    kok: PathBuf,
    cred: PathBuf,
}

impl Proje {
    fn yeni(etiket: &str, proje: &str, ana: &str) -> Self {
        let taban = std::env::temp_dir().canonicalize().unwrap();
        let kok = taban.join(format!("zee-k339-e2e-{etiket}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&kok);
        std::fs::create_dir_all(&kok).unwrap();
        std::fs::write(kok.join("ana.dil"), ana).unwrap();
        std::fs::write(kok.join("proje.dil"), proje).unwrap();
        let cred = kok.join("credentials");
        std::fs::create_dir(&cred).unwrap();
        kip(&cred, 0o500);
        Proje { kok, cred }
    }

    fn http(etiket: &str) -> Self {
        Self::yeni(
            etiket,
            &format!(
                "proje \"k339\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ\" listesi olsun\nağ_hedefleri \"https://service.example\" listesi olsun\nhttp_gizli_değişkenleri \"{HTTP_AD}\" listesi olsun\n"
            ),
            &format!(
                "sonuc \"{HTTP_AD}\" gizli ortam değerini okumayı dene olsun\nsonuc başarılıysa\n    sonucun değeri yaz\ndeğilse\n    sonucun hatası yaz\n"
            ),
        )
    }

    fn anahtar(etiket: &str) -> Self {
        Self::yeni(
            etiket,
            &format!(
                "proje \"k339\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"kriptografi\" listesi olsun\ngizli_anahtar_değişkeni \"{ANAHTAR_AD}\" olsun\n"
            ),
            "parola \"gidis-donus\" olsun\nsifreli parolanın gizli şifrelenmişi olsun\nsonuç sifreli gizli çözmeyi dene olsun\nsonuç başarılıysa\n    sonucun değeri yaz\n",
        )
    }

    fn credential(&self, ad: &str, icerik: &str, dosya_kipi: u32) {
        kip(&self.cred, 0o700);
        let yol = self.cred.join(ad);
        let _ = std::fs::remove_file(&yol);
        std::fs::write(&yol, icerik).unwrap();
        kip(&yol, dosya_kipi);
        kip(&self.cred, 0o500);
    }

    fn calistir(&self, dizin: Option<&Path>, ortam: &[(&str, &str)]) -> (bool, String, String) {
        let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
        komut
            .args(["çalıştır", "."])
            .current_dir(&self.kok)
            .env_remove(HTTP_AD)
            .env_remove(ANAHTAR_AD)
            .env_remove("CREDENTIALS_DIRECTORY");
        if let Some(dizin) = dizin {
            komut.env("CREDENTIALS_DIRECTORY", dizin);
        }
        for (ad, deger) in ortam {
            komut.env(ad, deger);
        }
        let cikti = komut.output().unwrap();
        (
            cikti.status.success(),
            String::from_utf8(cikti.stdout).unwrap().trim().to_string(),
            String::from_utf8(cikti.stderr).unwrap(),
        )
    }

    fn sizinti_yok(&self, cikti: &(bool, String, String)) {
        for akis in [&cikti.1, &cikti.2] {
            assert!(!akis.contains(SENTINEL), "değer sızdı: {akis}");
            assert!(!akis.contains(&"ab".repeat(32)), "anahtar sızdı: {akis}");
            assert!(
                !akis.contains(self.kok.to_str().unwrap()),
                "yol sızdı: {akis}"
            );
        }
    }
}

impl Drop for Proje {
    fn drop(&mut self) {
        kip(&self.cred, 0o700);
        let _ = std::fs::remove_dir_all(&self.kok);
    }
}

fn kip(yol: &Path, kip: u32) {
    std::fs::set_permissions(yol, std::fs::Permissions::from_mode(kip)).unwrap();
}

#[test]
fn systemd_credential_http_gizli_degerini_ortamsiz_saglar() {
    let p = Proje::http("http-olumlu");
    p.credential(HTTP_AD, SENTINEL, 0o400);
    assert_eq!(p.calistir(Some(&p.cred), &[]).1, SENTINEL);
    // Dizin yoksa eski ortam davranışı aynen sürer.
    assert_eq!(
        p.calistir(None, &[(HTTP_AD, "ORTAM_DEGERI")]).1,
        "ORTAM_DEGERI"
    );
    // Dizin var ama bu ad yoksa ortama döner; ikisi de yoksa eski ileti.
    let bos = Proje::http("http-bos");
    assert_eq!(
        bos.calistir(Some(&bos.cred), &[(HTTP_AD, "ORTAM")]).1,
        "ORTAM"
    );
    assert_eq!(
        bos.calistir(Some(&bos.cred), &[]).1,
        "HTTP işlemi başarısız; sır içeren ayrıntılar gizlendi"
    );
}

#[test]
fn systemd_credential_http_retleri_maskeli_kalir_ve_degeri_sizdirmaz() {
    let p = Proje::http("http-ret");
    p.credential(HTTP_AD, SENTINEL, 0o400);
    let maske = "HTTP işlemi başarısız; sır içeren ayrıntılar gizlendi";
    let cakisma = p.calistir(Some(&p.cred), &[(HTTP_AD, "ORTAM")]);
    assert_eq!(cakisma.1, maske);
    p.sizinti_yok(&cakisma);
    p.credential(HTTP_AD, SENTINEL, 0o644);
    let genis = p.calistir(Some(&p.cred), &[]);
    assert_eq!(genis.1, maske);
    p.sizinti_yok(&genis);
}

#[test]
fn systemd_credential_gizli_anahtari_okur_ve_retlerde_sabit_neden_verir() {
    let p = Proje::anahtar("anahtar");
    let anahtar = "ab".repeat(32);
    p.credential(ANAHTAR_AD, &anahtar, 0o400);
    let olumlu = p.calistir(Some(&p.cred), &[]);
    assert!(olumlu.0, "{olumlu:?}");
    assert_eq!(olumlu.1, "gidis-donus");
    p.sizinti_yok(&olumlu);

    let reddet = |dizin: &Path, ortam: &[(&str, &str)], beklenen: &str| {
        let cikti = p.calistir(Some(dizin), ortam);
        assert!(!cikti.0, "{cikti:?}");
        assert!(cikti.2.contains("C028"), "{cikti:?}");
        assert!(
            cikti
                .2
                .contains(&format!("{ANAHTAR_AD} gizli değeri okunamadı: {beklenen}")),
            "{cikti:?}"
        );
        p.sizinti_yok(&cikti);
    };
    reddet(
        &p.cred,
        &[(ANAHTAR_AD, "ortam")],
        "gizli değer hem ortamda hem credential dizininde var",
    );
    reddet(
        &p.kok.join("olmayan"),
        &[],
        "CREDENTIALS_DIRECTORY açılamadı",
    );
    reddet(
        Path::new("credentials"),
        &[],
        "CREDENTIALS_DIRECTORY mutlak yol olmalı",
    );

    p.credential(ANAHTAR_AD, &format!("{anahtar}\n"), 0o400);
    reddet(&p.cred, &[], "credential dosyası tek satır olmalı");
    p.credential(ANAHTAR_AD, &anahtar, 0o644);
    reddet(
        &p.cred,
        &[],
        "credential dosyası grupça yazılabilir ya da herkesçe okunabilir olamaz",
    );
    p.credential(ANAHTAR_AD, &"a".repeat(4097), 0o400);
    reddet(&p.cred, &[], "credential dosyası boyut sınırını aşıyor");

    kip(&p.cred, 0o700);
    std::fs::remove_file(p.cred.join(ANAHTAR_AD)).unwrap();
    std::fs::write(p.kok.join("hedef"), &anahtar).unwrap();
    kip(&p.kok.join("hedef"), 0o400);
    std::os::unix::fs::symlink(p.kok.join("hedef"), p.cred.join(ANAHTAR_AD)).unwrap();
    kip(&p.cred, 0o500);
    reddet(&p.cred, &[], "credential dosyası sembolik bağ olamaz");

    // Dizin yoksa eski ileti birebir.
    let eski = p.calistir(None, &[]);
    assert!(
        eski.2.contains(&format!(
            "{ANAHTAR_AD} ortam değişkeni gizli anahtarı taşımıyor"
        )),
        "{eski:?}"
    );
}

/// RFC-0047 E (ADR-182 dilim B): gerçek `dil` ikilisi uygulama parolasını
/// credential dosyasındaki etkin HMAC anahtarıyla üretir; özet kitaplığın
/// sabit zamanlı doğrulamasından geçer, anahtar hiçbir akışa sızmaz.
#[test]
fn systemd_credential_uygulama_parolasi_etkin_anahtarla_uretilir() {
    const UP_AD: &str = "ZEE_K364_UP";
    let p = Proje::yeni(
        "uygulama-parolasi",
        &format!(
            "proje \"k364\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"kriptografi\" listesi olsun\nuygulama_parolası_anahtarları \"k2026 {UP_AD}\", \"k2025 ZEE_K364_ESKI\" listesi olsun\n"
        ),
        "parola yeni uygulama parolası olsun\nparolanın \"görünen\" değeri yaz\nparolanın \"önek\" değeri yaz\nparolanın \"özet\" değeri yaz\n",
    );
    // Anahtar yokken üretim yok: ret iletisi credential adını anar, değer yok.
    let (basarili, _, hata) = p.calistir(None, &[]);
    assert!(!basarili);
    assert!(hata.contains("C028"), "{hata}");
    p.credential(UP_AD, &"ab".repeat(32), 0o400);
    let cikti = p.calistir(Some(&p.cred), &[]);
    assert!(cikti.0, "{}", cikti.2);
    p.sizinti_yok(&cikti);
    let satirlar: Vec<&str> = cikti.1.lines().collect();
    assert_eq!(satirlar.len(), 3, "{satirlar:?}");
    use dil::guvenlik::uygulama_parolasi::{ozet_dogrula, parolayi_coz};
    let (onek, gizli) = parolayi_coz(satirlar[0]).expect("biçim");
    assert_eq!(onek, satirlar[1]);
    assert!(satirlar[2].starts_with("zuah1:k2026:"), "{}", satirlar[2]);
    let anahtar = [0xabu8; 32];
    assert!(ozet_dogrula(
        &[("k2026".into(), anahtar)],
        &onek,
        &gizli,
        Some(satirlar[2])
    ));
    assert!(!ozet_dogrula(
        &[("k2026".into(), [0u8; 32])],
        &onek,
        &gizli,
        Some(satirlar[2])
    ));
    // İki çalıştırma farklı parola (CSPRNG).
    let ikinci = p.calistir(Some(&p.cred), &[]);
    assert_ne!(ikinci.1, cikti.1);
}
