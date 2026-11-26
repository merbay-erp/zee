//! K-397 Dilim 2 (RFC-0044 B3, ADR-174): DKIM imza ilkelinin Zee yüzeyi.
//!
//! Anahtarlar test anında üretilir (depoda özel anahtar yok): Ed25519
//! `ring` ile, RSA `openssl genpkey` ile. İmza hem K-390 B2 doğrulayıcısıyla
//! (Zee kalıbı) hem bağımsız olarak `openssl` ile doğrulanır.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use dil::dkim_imza::{DkimAnahtarDeposu, DkimImzaAnahtari};
use dil::dkim_kripto::DkimAlgoritmasi;
use dil::yetkinlik::{Yetkinlik, YetkinlikPolitikasi};
use dil::yorumlayici::{calistir_io, ToplayanIo};

#[path = "ortak/dkim_ozellikleri.rs"]
mod dkim_ozellikleri;

/// Zee'nin kanonikleştirdiği imza girdisi (relaxed başlıklar + boş `b=`).
fn girdi(a: &str) -> String {
    format!(
        "from:Zee Test <gonderen@posta.ornek.test>\nto:alici@baska.test\nsubject:Merhaba\n\
dkim-signature:v=1; a={a}; c=relaxed/relaxed; d=ornek.test; s=z2026a; h=from:to:subject:from; \
bh=47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=; b="
    )
}

fn zee_metni(m: &str) -> String {
    format!(
        "\"{}\"",
        m.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    )
}

/// Her çağrıya ayrı dizin: paralel testler aynı adı (ör. `rsa2048`)
/// kullanınca biri ötekinin dizinini silmesin.
fn gecici_dizin(ad: &str) -> PathBuf {
    static SAYAC: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let sira = SAYAC.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dizin = std::env::temp_dir().join(format!("zee-dkim-{}-{sira}-{ad}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dizin);
    std::fs::create_dir_all(&dizin).unwrap();
    dizin
}

fn base64(bayt: &[u8]) -> String {
    dil::kodlama::base64_kodla(bayt)
}

/// PEM sınırları parçalı yazılır: tek satırdaki BEGIN…END kalıbı sır
/// taramasında (gitleaks `private-key`) gömülü anahtar sanılır. Anahtar
/// her koşuda `ring` ile yeniden üretilir; depoda anahtar baytı yoktur.
const PEM_ETIKETI: &str = concat!("PRIVATE", " KEY-----");

fn ed25519_pem() -> String {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8 = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    format!(
        "-----BEGIN {PEM_ETIKETI}\n{}\n-----END {PEM_ETIKETI}\n",
        base64(pkcs8.as_ref())
    )
}

/// `openssl genpkey` ile RSA PKCS#8 PEM (yalnız bu süreçte, geçici dizinde).
fn rsa_pem(bit: usize) -> String {
    let dizin = gecici_dizin(&format!("rsa{bit}"));
    let yol = dizin.join("anahtar.pem");
    let durum = Command::new("openssl")
        .args(["genpkey", "-algorithm", "RSA", "-pkeyopt"])
        .arg(format!("rsa_keygen_bits:{bit}"))
        .arg("-out")
        .arg(&yol)
        .output()
        .expect("openssl bulunmalı");
    assert!(durum.status.success(), "{durum:?}");
    let pem = std::fs::read_to_string(&yol).unwrap();
    let _ = std::fs::remove_dir_all(&dizin);
    pem
}

fn depo(anahtarlar: &[(&str, &str, DkimAlgoritmasi, &str)]) -> Arc<DkimAnahtarDeposu> {
    let mut depo = DkimAnahtarDeposu::default();
    for (credential, secici, algoritma, pem) in anahtarlar {
        let anahtar = DkimImzaAnahtari::pemden("ornek.test", secici, *algoritma, pem).unwrap();
        depo.ekle(credential, anahtar);
    }
    Arc::new(depo)
}

/// İmzalar; başarıda `b=`, retde `C034 <tür>` döner.
fn imzala(depo: Option<Arc<DkimAnahtarDeposu>>, veri: &str, ad: &str, a: &str) -> String {
    let kaynak = format!(
        "veri {} olsun\nsonuç veri verisi için {} özel anahtarıyla {} olarak imzalamayı dene olsun\n\
sonuç başarılıysa\n    sonucun değerini yaz\n\
değilse\n    hata sonucun hatası olsun\n    hatanın kodunu yaz\n    veri2 hatanın verisi olsun\n    veri2 \"tür\" değeri yaz\n",
        zee_metni(veri),
        zee_metni(ad),
        zee_metni(a)
    );
    let program = dil::kaynagi_derle(&kaynak).expect("derlenmeli");
    let mut io = ToplayanIo::yeni(Vec::new());
    io.dkim_anahtarlari = depo;
    calistir_io(&program, &mut io).expect("çalışmalı");
    io.cikti.join(" ")
}

fn dogrula(b: &str, veri: &str, p: &str, a: &str) -> bool {
    dil::dkim_kripto::base64_imzayi_dogrula(a, p, veri, b)
        .unwrap()
        .tuttu
}

fn crlf(veri: &str) -> Vec<u8> {
    veri.replace('\n', "\r\n").into_bytes()
}

/// Bağımsız doğrulama: openssl RSA (dgst) ve RFC 8463 Ed25519 (pkeyutl -rawin
/// SHA-256 özeti üzerinde).
/// `Ed25519` için açık anahtarın ham 32 baytı, RSA için PKCS#8 PEM.
enum Anahtar<'a> {
    Ed25519(&'a [u8]),
    RsaPem(&'a str),
}

fn openssl_dogrular(anahtar: Anahtar, veri: &str, b: &str) -> bool {
    let ed25519 = matches!(anahtar, Anahtar::Ed25519(_));
    let dizin = gecici_dizin(if ed25519 { "osl-ed" } else { "osl-rsa" });
    match anahtar {
        // RFC 8410 SPKI'yi kendimiz kurarız: OpenSSL 3.0 (Ubuntu 24.04), ring'in
        // ürettiği açık anahtarlı PKCS#8 v2 Ed25519 anahtarını `pkey` ile okuyamıyor.
        Anahtar::Ed25519(ham) => {
            let mut spki = vec![
                0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
            ];
            spki.extend_from_slice(ham);
            std::fs::write(
                dizin.join("acik.pem"),
                format!(
                    "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
                    base64(&spki)
                ),
            )
            .unwrap();
        }
        Anahtar::RsaPem(pem) => {
            std::fs::write(dizin.join("ozel.pem"), pem).unwrap();
            let acik = Command::new("openssl")
                .args(["pkey", "-pubout", "-in"])
                .arg(dizin.join("ozel.pem"))
                .arg("-out")
                .arg(dizin.join("acik.pem"))
                .status()
                .unwrap();
            assert!(acik.success());
            std::fs::remove_file(dizin.join("ozel.pem")).unwrap();
        }
    }
    std::fs::write(
        dizin.join("imza.bin"),
        dil::kodlama::base64_mime_coz(b, 4096).unwrap(),
    )
    .unwrap();
    let baytlar = crlf(veri);
    let mut komut = Command::new("openssl");
    if ed25519 {
        use sha2::{Digest, Sha256};
        std::fs::write(dizin.join("veri.bin"), Sha256::digest(&baytlar)).unwrap();
        komut.args(["pkeyutl", "-verify", "-pubin", "-rawin", "-inkey"]);
        komut
            .arg(dizin.join("acik.pem"))
            .arg("-in")
            .arg(dizin.join("veri.bin"))
            .arg("-sigfile")
            .arg(dizin.join("imza.bin"));
    } else {
        std::fs::write(dizin.join("veri.bin"), &baytlar).unwrap();
        komut.args(["dgst", "-sha256", "-verify"]);
        komut
            .arg(dizin.join("acik.pem"))
            .arg("-signature")
            .arg(dizin.join("imza.bin"))
            .arg(dizin.join("veri.bin"));
    }
    let sonuc = komut.output().unwrap();
    let _ = std::fs::remove_dir_all(&dizin);
    sonuc.status.success()
}

#[test]
fn ed25519_imzasi_k390_dogrulayicisi_ve_openssl_ile_tutar() {
    let pem = ed25519_pem();
    let a = "ed25519-sha256";
    let depo = depo(&[(
        "zeemail_dkim_anahtari",
        "z2026a",
        DkimAlgoritmasi::Ed25519Sha256,
        &pem,
    )]);
    let p = base64(
        &DkimImzaAnahtari::pemden("ornek.test", "z2026a", DkimAlgoritmasi::Ed25519Sha256, &pem)
            .unwrap()
            .acik_anahtar(),
    );
    let veri = girdi(a);
    let b = imzala(Some(depo.clone()), &veri, "zeemail_dkim_anahtari", a);
    assert_eq!(b.len(), 88, "{b}");
    assert!(dogrula(&b, &veri, &p, a));
    let ham = dil::kodlama::base64_mime_coz(&p, 64).unwrap();
    assert!(openssl_dogrular(Anahtar::Ed25519(&ham), &veri, &b));
    assert!(!openssl_dogrular(
        Anahtar::Ed25519(&ham),
        &veri.replace("Merhaba", "Selam"),
        &b
    ));
    // Ed25519 deterministiktir: aynı girdi aynı `b=`.
    assert_eq!(imzala(Some(depo), &veri, "zeemail_dkim_anahtari", a), b);
}

#[test]
fn rsa_2048_ve_4096_imzasi_dogrulanir_kisa_ve_uzun_anahtar_yuklenmez() {
    let a = "rsa-sha256";
    for bit in [2048, 4096] {
        let pem = rsa_pem(bit);
        let anahtar =
            DkimImzaAnahtari::pemden("ornek.test", "z2026a", DkimAlgoritmasi::RsaSha256, &pem)
                .unwrap();
        assert_eq!(anahtar.bit, bit);
        let p = base64(&anahtar.acik_anahtar());
        let depo = depo(&[("dkim_rsa", "z2026a", DkimAlgoritmasi::RsaSha256, &pem)]);
        let veri = girdi(a);
        let b = imzala(Some(depo), &veri, "dkim_rsa", a);
        assert!(dogrula(&b, &veri, &p, a), "{bit}");
        assert!(openssl_dogrular(Anahtar::RsaPem(&pem), &veri, &b), "{bit}");
        assert!(!dogrula(&b, &veri.replace("Merhaba", "Merhabа"), &p, a));
    }
    for (bit, parca) in [(1024, "2048 bitten kısa"), (4608, "4096 bitten uzun")] {
        let pem = rsa_pem(bit);
        let hata =
            DkimImzaAnahtari::pemden("ornek.test", "z2026a", DkimAlgoritmasi::RsaSha256, &pem)
                .unwrap_err();
        assert!(hata.contains(parca), "{bit}: {hata}");
    }
    // RSA anahtarı Ed25519 diye (ve tersi) yüklenmez.
    let pem = rsa_pem(2048);
    assert!(
        DkimImzaAnahtari::pemden("ornek.test", "s", DkimAlgoritmasi::Ed25519Sha256, &pem).is_err()
    );
    assert!(DkimImzaAnahtari::pemden(
        "ornek.test",
        "s",
        DkimAlgoritmasi::RsaSha256,
        &ed25519_pem()
    )
    .is_err());
}

#[test]
fn imza_kahini_onlemi_c034_ile_reddeder() {
    let pem = ed25519_pem();
    let a = "ed25519-sha256";
    let depo = depo(&[(
        "zeemail_dkim_anahtari",
        "z2026a",
        DkimAlgoritmasi::Ed25519Sha256,
        &pem,
    )]);
    let veri = girdi(a);
    let ret = |veri: &str, ad: &str, a: &str| imzala(Some(depo.clone()), veri, ad, a);
    for (eski, yeni, tur) in [
        (
            "gonderen@posta.ornek.test",
            "gonderen@kotu.test",
            "geçersiz",
        ),
        ("d=ornek.test", "d=kotu.test", "geçersiz"),
        ("h=from:to:subject:from", "h=to:subject", "geçersiz"),
        ("; b=", "; b=AAAA", "geçersiz"),
        ("s=z2026a", "s=z2025b", "anahtar_uyuşmuyor"),
        (
            "from:Zee Test <gonderen@posta.ornek.test>\n",
            "",
            "geçersiz",
        ),
        ("a=ed25519-sha256", "a=rsa-sha1", "algoritma_reddedildi"),
        ("\nsubject", "\r\nsubject", "veri_bozuk"),
    ] {
        assert_eq!(
            ret(&veri.replace(eski, yeni), "zeemail_dkim_anahtari", a),
            format!("C034 {tur}"),
            "{eski} → {yeni}"
        );
    }
    // `d=` alt alanı From'la hizalı olsa da bildirilmiş alan değil.
    let alt = veri.replace("d=ornek.test", "d=posta.ornek.test");
    assert_eq!(
        ret(&alt, "zeemail_dkim_anahtari", a),
        "C034 anahtar_uyuşmuyor"
    );
    assert_eq!(ret(&veri, "baska_anahtar", a), "C034 anahtar_bilinmiyor");
    assert_eq!(
        ret(&veri, "zeemail_dkim_anahtari", "rsa-sha256"),
        "C034 geçersiz"
    );
    assert_eq!(
        ret("keyfi bayt", "zeemail_dkim_anahtari", a),
        "C034 geçersiz"
    );
    assert_eq!(
        imzala(None, &veri, "zeemail_dkim_anahtari", a),
        "C034 anahtar_bilinmiyor"
    );
}

#[test]
fn imzalama_kriptografi_yetkinligi_ister() {
    let program = dil::kaynagi_derle(
        "x \"\" olsun\ns x verisi için x özel anahtarıyla x olarak imzalamayı dene olsun\n",
    )
    .unwrap();
    let yalniz = YetkinlikPolitikasi::proje([Yetkinlik::Kriptografi].into(), Default::default());
    dil::cozumleyici::yetkinlikleri_denetle(&program, &yalniz).expect("izinli");
    let bos = YetkinlikPolitikasi::proje(Default::default(), Default::default());
    assert!(dil::cozumleyici::yetkinlikleri_denetle(&program, &bos).is_err());
}

#[test]
fn io_izi_anahtar_ve_girdi_tasimaz_anahtarsiz_oynatilir() {
    let pem = ed25519_pem();
    let a = "ed25519-sha256";
    let veri = girdi(a);
    let kaynak = format!(
        "sonuç {} verisi için \"zeemail_dkim_anahtari\" özel anahtarıyla \"{a}\" olarak imzalamayı dene olsun\n\
sonuç başarılıysa\n    sonucun değerini yaz\n",
        zee_metni(&veri)
    );
    let program = dil::kaynagi_derle(&kaynak).unwrap();
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban.dkim_anahtarlari = Some(depo(&[(
        "zeemail_dkim_anahtari",
        "z2026a",
        DkimAlgoritmasi::Ed25519Sha256,
        &pem,
    )]));
    let mut kaydeden = dil::yorumlayici::IzKaydedenIo::yeni(taban);
    calistir_io(&program, &mut kaydeden).expect("kayıt");
    let iz = kaydeden.iz_metni().expect("iz");
    assert!(iz.contains("dkim_imzala"));
    assert!(!iz.contains("gonderen@posta"), "izde imza girdisi olmamalı");
    let govde: String = pem.lines().filter(|s| !s.starts_with("-----")).collect();
    for parca in govde
        .as_bytes()
        .chunks(16)
        .map(|c| std::str::from_utf8(c).unwrap())
    {
        if parca.len() == 16 {
            assert!(!iz.contains(parca), "izde PEM dilimi: {parca}");
        }
    }
    let mut oynatici = dil::yorumlayici::IzYenidenOynatici::yeni(&iz).expect("iz okunur");
    calistir_io(&program, &mut oynatici).expect("replay");
    assert_eq!(oynatici.cikti.len(), 1);
    assert_eq!(oynatici.cikti[0].len(), 88);
    oynatici.bitir().expect("iz tükendi");
}

fn bildirim(ek: &str) -> Result<dil::proje::ProjeBildirimi, dil::tani::Tani> {
    dil::proje::bildirimi_oku(&format!(
        "proje \"d\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n{ek}"
    ))
}

#[test]
fn proje_dkim_anahtarlari_kriptografi_ile_ve_tekil_bildirilir() {
    let kripto = "yetkinlikler \"kriptografi\" listesi olsun\n";
    let tamam = bildirim(&format!(
        "{kripto}dkim_anahtarları \"ornek.test z2026a ed25519-sha256 zeemail_dkim_anahtari\", \"ornek.test r2026a rsa-sha256 DKIM_R2026A\" listesi olsun\n"
    ))
    .expect("geçerli");
    assert_eq!(tamam.dkim_anahtarlari.len(), 2);
    assert_eq!(tamam.dkim_anahtarlari[1].credential, "DKIM_R2026A");
    assert!(bildirim(kripto).unwrap().dkim_anahtarlari.is_empty());
    let oge = |o: &str| format!("{kripto}dkim_anahtarları \"{o}\" listesi olsun\n");
    for (ek, parca) in [
        ("dkim_anahtarları \"ornek.test s ed25519-sha256 K\" listesi olsun\n".to_string(), "kriptografi"),
        (oge("ornek.test s rsa-sha1 K"), "rsa-sha1"),
        (oge("ornek.test s rsa-sha512 K"), "desteklenmiyor"),
        (oge("ornek.test s ed25519-sha256"), "biçiminde"),
        (oge("Ornek.Test s ed25519-sha256 K"), "alan adı"),
        (oge("ornek s ed25519-sha256 K"), "alan adı"),
        (oge("ornek.test S ed25519-sha256 K"), "seçici"),
        (oge("ornek.test s ed25519-sha256 ../K"), "credential"),
        (
            format!("{kripto}dkim_anahtarları \"ornek.test s ed25519-sha256 K\", \"ornek.test s rsa-sha256 L\" listesi olsun\n"),
            "iki kez",
        ),
        (
            format!("{kripto}dkim_anahtarları \"ornek.test s ed25519-sha256 K\", \"ornek.test t rsa-sha256 K\" listesi olsun\n"),
            "iki kez",
        ),
        (
            format!("{kripto}gizli_anahtar_değişkeni \"ZEE_GIZLI\" olsun\ndkim_anahtarları \"ornek.test s ed25519-sha256 ZEE_GIZLI\" listesi olsun\n"),
            "başka bir gizli",
        ),
    ] {
        let tani = bildirim(&ek).expect_err(&ek);
        assert_eq!(tani.kod, "P015", "{ek}");
        assert!(tani.mesaj.contains(parca), "{ek}: {}", tani.mesaj);
    }
}

/// Gerçek süreç: anahtar ortamdan (ya da credential dizininden) okunur,
/// imza atılır; kötü anahtar çıkış 2 ve tanı yalnız seçiciyi anar.
fn surec(pem: Option<&str>, credential_dizini: Option<&Path>, ad: &str) -> (i32, String, String) {
    let dizin = gecici_dizin(&format!("surec-{ad}"));
    std::fs::write(
        dizin.join("proje.dil"),
        "proje \"dkim\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
         yetkinlikler \"kriptografi\" listesi olsun\n\
         dkim_anahtarları \"ornek.test z2026a ed25519-sha256 zee_dkim_testi\" listesi olsun\n",
    )
    .unwrap();
    std::fs::write(
        dizin.join("ana.dil"),
        format!(
            "sonuç {} verisi için \"zee_dkim_testi\" özel anahtarıyla \"ed25519-sha256\" olarak imzalamayı dene olsun\n\
sonuç başarılıysa\n    sonucun değerini yaz\n",
            zee_metni(&girdi("ed25519-sha256"))
        ),
    )
    .unwrap();
    let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
    komut.arg("çalıştır").arg(&dizin);
    komut
        .env_remove("zee_dkim_testi")
        .env_remove("CREDENTIALS_DIRECTORY");
    if let Some(pem) = pem {
        komut.env("zee_dkim_testi", pem);
    }
    if let Some(credential) = credential_dizini {
        komut.env("CREDENTIALS_DIRECTORY", credential);
    }
    let cikti = komut.output().unwrap();
    let _ = std::fs::remove_dir_all(&dizin);
    (
        cikti.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&cikti.stdout).into_owned(),
        String::from_utf8_lossy(&cikti.stderr).into_owned(),
    )
}

#[test]
fn gercek_surec_anahtari_acilista_yukler_bozugu_reddeder() {
    let pem = ed25519_pem();
    let (kod, cikti, hata) = surec(Some(&pem), None, "tamam");
    assert_eq!(kod, 0, "{hata}");
    assert_eq!(cikti.trim().len(), 88, "{cikti}");

    let govde: String = pem.lines().filter(|s| !s.starts_with("-----")).collect();
    let rsa = rsa_pem(2048);
    for (ad, deger) in [
        ("yok", None),
        ("bozuk", Some(pem.replace("PRIVATE KEY", "RSA PRIVATE KEY"))),
        ("tur", Some(rsa)),
    ] {
        let (kod, cikti, hata) = surec(deger.as_deref(), None, ad);
        assert_eq!(kod, 2, "{ad}: {cikti}{hata}");
        assert!(cikti.is_empty(), "{ad}");
        assert!(hata.contains("ornek.test / z2026a"), "{ad}: {hata}");
        assert!(!hata.contains(&govde[..16]), "{ad}: tanıda anahtar");
    }
}

#[cfg(unix)]
#[test]
fn credential_dizini_tercih_edilir_ortamla_cakisma_reddedilir() {
    use std::os::unix::fs::PermissionsExt;
    let pem = ed25519_pem();
    let dizin = gecici_dizin("credential");
    std::fs::set_permissions(&dizin, std::fs::Permissions::from_mode(0o700)).unwrap();
    let dosya = dizin.join("zee_dkim_testi");
    std::fs::write(&dosya, &pem).unwrap();
    std::fs::set_permissions(&dosya, std::fs::Permissions::from_mode(0o600)).unwrap();
    let mutlak = std::fs::canonicalize(&dizin).unwrap();
    let (kod, cikti, hata) = surec(None, Some(&mutlak), "credential-tamam");
    assert_eq!(kod, 0, "{hata}");
    assert_eq!(cikti.trim().len(), 88);
    let (kod, _, hata) = surec(Some(&pem), Some(&mutlak), "credential-cakisma");
    assert_eq!(kod, 2);
    assert!(hata.contains("hem ortamda hem credential"), "{hata}");
    let _ = std::fs::remove_dir_all(&dizin);
}

// ---- K-397 B3 (ZM-F046): dosyadan akışlı kanonik DKIM gövde özeti ----

fn dosya_ozeti(io: &mut ToplayanIo, yol: &str, bas: i64, kip: &str) -> String {
    let kaynak = format!(
        "sonuç {} dosyasının {bas} baytından {} kipinde dkim gövde özetini almayı dene olsun\n\
sonuç başarılıysa\n    sonucun değerini yaz\n\
değilse\n    hata sonucun hatası olsun\n    hatanın kodunu yaz\n    veri2 hatanın verisi olsun\n    veri2 \"tür\" değeri yaz\n",
        zee_metni(yol),
        zee_metni(kip)
    );
    let program = dil::kaynagi_derle(&kaynak).expect("derlenmeli");
    io.cikti.clear();
    calistir_io(&program, io).expect("çalışmalı");
    io.cikti.join(" ")
}

fn sha_b64(bayt: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    base64(&Sha256::digest(bayt))
}

/// Ham ileti: başlıklar + boş satır + gövde; dönen değer gövde başı.
fn ileti(io: &mut ToplayanIo, yol: &str, govde: &[u8]) -> i64 {
    let baslik = b"From: a@ornek.test\r\nSubject: Deneme\r\n\r\n";
    let mut ham = baslik.to_vec();
    ham.extend_from_slice(govde);
    io.ikili_dosyalar.insert(yol.into(), ham);
    baslik.len() as i64
}

#[test]
fn dosya_govde_ozeti_rfc6376_kiplerini_uygular() {
    let mut io = ToplayanIo::yeni(Vec::new());
    // RFC 6376 §3.4.5 örneği; sondaki boş satırlar ve satır içi WSP.
    let bas = ileti(&mut io, "ornek.eml", b" C \r\nD \t E\r\n\r\n\r\n");
    assert_eq!(
        dosya_ozeti(&mut io, "ornek.eml", bas, "relaxed"),
        sha_b64(b" C\r\nD E\r\n")
    );
    assert_eq!(
        dosya_ozeti(&mut io, "ornek.eml", bas, "simple"),
        sha_b64(b" C \r\nD \t E\r\n")
    );
    // Boş gövde: simple "\r\n", relaxed boş (RFC 6376 §3.4.3/§3.4.4).
    let bos = ileti(&mut io, "bos.eml", b"");
    assert_eq!(
        dosya_ozeti(&mut io, "bos.eml", bos, "simple"),
        "frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY="
    );
    assert_eq!(
        dosya_ozeti(&mut io, "bos.eml", bos, "relaxed"),
        "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU="
    );
    // Sonda satır sonu yok, yalnız boşluklu satır, sıkıştırma.
    let bas = ileti(&mut io, "kenar.eml", b"a  \t b \r\n \t \r\nson satir  ");
    assert_eq!(
        dosya_ozeti(&mut io, "kenar.eml", bas, "relaxed"),
        sha_b64(b"a b\r\n\r\nson satir\r\n")
    );
    assert_eq!(
        dosya_ozeti(&mut io, "kenar.eml", bas, "simple"),
        sha_b64(b"a  \t b \r\n \t \r\nson satir  \r\n")
    );
}

#[test]
fn dosya_govde_ozeti_lf_crlf_ve_metin_ilkesiyle_esdegerdir() {
    let mut io = ToplayanIo::yeni(Vec::new());
    let crlf: String = (0..12_000)
        .map(|i| format!("Satır {i}:  ek\tgövdesi QUJDREVGR0g= \r\n"))
        .collect::<String>()
        + "\r\n\r\n";
    assert!(crlf.len() > 256 * 1024);
    let lf = crlf.replace("\r\n", "\n");
    io.ikili_dosyalar
        .insert("crlf.eml".into(), crlf.into_bytes());
    io.ikili_dosyalar.insert("lf.eml".into(), lf.into_bytes());
    for kip in ["simple", "relaxed"] {
        let beklenen = dosya_ozeti(&mut io, "crlf.eml", 0, kip);
        assert_eq!(beklenen.len(), 44, "{kip}: {beklenen}");
        assert_eq!(dosya_ozeti(&mut io, "lf.eml", 0, kip), beklenen, "{kip}");
    }
    // Simple + LF kanonik gövde: K-390 Metin ilkesiyle aynı değer.
    for govde in [
        "\n",
        "Hi.\n\nWe lost the game. Are you hungry yet?\n\nJoe.\n",
        "tek satır\n",
    ] {
        io.ikili_dosyalar
            .insert("k.eml".into(), govde.as_bytes().to_vec());
        assert_eq!(
            dosya_ozeti(&mut io, "k.eml", 0, "simple"),
            metin_ozeti(govde),
            "{govde:?}"
        );
    }
    assert_eq!(
        dosya_ozeti(&mut io, "k.eml", 0, "simple"),
        dil::dkim_kripto::govde_ozeti("tek satır\n").unwrap()
    );
}

fn metin_ozeti(govde: &str) -> String {
    let program = dil::kaynagi_derle(&format!(
        "{} gövdesinin dkim gövde özeti yaz\n",
        zee_metni(govde)
    ))
    .unwrap();
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_io(&program, &mut io).expect("çalışmalı");
    io.cikti.join(" ")
}

#[test]
fn dosya_govde_ozeti_bas_ofseti_kip_cr_ve_32_mib_sinirini_denetler() {
    let mut io = ToplayanIo::yeni(Vec::new());
    let bas = ileti(&mut io, "m.eml", b"Govde\r\n");
    assert_eq!(
        dosya_ozeti(&mut io, "m.eml", bas, "simple"),
        sha_b64(b"Govde\r\n")
    );
    // Ofset başlıkları dışarıda bırakır; 0'dan alınan farklıdır.
    assert_ne!(
        dosya_ozeti(&mut io, "m.eml", 0, "simple"),
        sha_b64(b"Govde\r\n")
    );
    let boy = io.ikili_dosyalar["m.eml"].len() as i64;
    assert_eq!(
        dosya_ozeti(&mut io, "m.eml", boy, "simple"),
        "frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY="
    );
    assert_eq!(
        dosya_ozeti(&mut io, "m.eml", boy + 1, "simple"),
        "C034 geçersiz"
    );
    assert_eq!(dosya_ozeti(&mut io, "m.eml", -1, "simple"), "C034 geçersiz");
    for kip in ["Relaxed", "nofws", ""] {
        assert_eq!(
            dosya_ozeti(&mut io, "m.eml", 0, kip),
            "C034 geçersiz",
            "{kip}"
        );
    }
    for bozuk in [&b"a\rb\r\n"[..], b"son\r"] {
        io.ikili_dosyalar.insert("cr.eml".into(), bozuk.to_vec());
        assert_eq!(
            dosya_ozeti(&mut io, "cr.eml", 0, "relaxed"),
            "C034 veri_bozuk"
        );
    }
    let sinir = dil::dkim_kripto::DOSYA_GOVDE_OZETI_SINIRI;
    let mut tam = b"XX".to_vec();
    tam.extend(std::iter::repeat_n(b'a', sinir));
    io.ikili_dosyalar.insert("tam.eml".into(), tam);
    assert_eq!(dosya_ozeti(&mut io, "tam.eml", 2, "simple").len(), 44);
    assert_eq!(dosya_ozeti(&mut io, "tam.eml", 1, "simple"), "C034 sınır");
    assert_eq!(dosya_ozeti(&mut io, "yok.eml", 0, "simple"), "C034 dosya");
}

#[test]
fn dosya_govde_ozeti_kriptografi_ve_dosya_okuma_ister() {
    let program = dil::kaynagi_derle(
        "x \"veri:giden/govde.eml\" dosyasının 0 baytından \"relaxed\" kipinde dkim gövde özetini almayı dene olsun\n",
    )
    .unwrap();
    let politika = |y: &[Yetkinlik]| {
        YetkinlikPolitikasi::proje(y.iter().copied().collect(), Default::default())
    };
    for yeterli in [
        &[Yetkinlik::Kriptografi, Yetkinlik::DosyaOkuma][..],
        &[Yetkinlik::Kriptografi, Yetkinlik::VeriOkuma][..],
    ] {
        dil::cozumleyici::yetkinlikleri_denetle(&program, &politika(yeterli)).expect("izinli");
    }
    for eksik in [&[Yetkinlik::Kriptografi][..], &[Yetkinlik::DosyaOkuma][..]] {
        assert!(dil::cozumleyici::yetkinlikleri_denetle(&program, &politika(eksik)).is_err());
    }
}

#[test]
fn dosya_govde_ozeti_izde_icerik_tasimaz_ve_oynatilir() {
    let program = dil::kaynagi_derle(
        "sonuç \"govde.eml\" dosyasının 0 baytından \"simple\" kipinde dkim gövde özetini almayı dene olsun\nsonuç başarılıysa\n    sonucun değerini yaz\n",
    )
    .unwrap();
    let mut taban = ToplayanIo::yeni(Vec::new());
    taban
        .ikili_dosyalar
        .insert("govde.eml".into(), b"gizli govde metni\r\n".to_vec());
    let mut kaydeden = dil::yorumlayici::IzKaydedenIo::yeni(taban);
    calistir_io(&program, &mut kaydeden).expect("kayıt");
    let iz = kaydeden.iz_metni().expect("iz");
    assert!(iz.contains("dosya_dkim_govde_ozeti"));
    assert!(!iz.contains("gizli govde"));
    let mut oynatici = dil::yorumlayici::IzYenidenOynatici::yeni(&iz).expect("iz okunur");
    calistir_io(&program, &mut oynatici).expect("replay");
    assert_eq!(oynatici.cikti, vec![sha_b64(b"gizli govde metni\r\n")]);
    oynatici.bitir().expect("iz tükendi");
}

/// Gerçek süreç: 400 KB'lık ham CRLF iletinin gövdesi ofsetten akışla
/// özetlenir; 32 MiB + 1 bayt `sınır`, LF'siz CR `veri_bozuk`.
#[test]
fn gercek_surec_buyuk_ham_iletiyi_akisla_ozetler() {
    let dizin = gecici_dizin("govde-surec");
    std::fs::write(
        dizin.join("proje.dil"),
        "proje \"govde\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
         yetkinlikler \"kriptografi\", \"dosya-okuma\" listesi olsun\n",
    )
    .unwrap();
    let baslik = "From: a@ornek.test\r\nSubject: Ek\r\n\r\n";
    let govde: String = (0..14_000)
        .map(|i| format!("Satır {i}:  ek gövdesi QUJDREVGR0g=  \r\n"))
        .collect();
    assert!(govde.len() > 400 * 1000);
    std::fs::write(dizin.join("buyuk.eml"), format!("{baslik}{govde}")).unwrap();
    let asiri = std::fs::File::create(dizin.join("asiri.eml")).unwrap();
    asiri
        .set_len(dil::dkim_kripto::DOSYA_GOVDE_OZETI_SINIRI as u64 + 1)
        .unwrap();
    std::fs::write(dizin.join("cr.eml"), "a\rb\r\n").unwrap();
    let blok = |dosya: &str, bas: usize, s: &str, sin: &str| {
        format!(
            "{s} \"{dosya}\" dosyasının {bas} baytından \"relaxed\" kipinde dkim gövde özetini almayı dene olsun\n\
{s} başarılıysa\n    {sin} değerini yaz\n\
değilse\n    hata {sin} hatası olsun\n    veri2 hatanın verisi olsun\n    veri2 \"tür\" değeri yaz\n"
        )
    };
    std::fs::write(
        dizin.join("ana.dil"),
        [
            blok("buyuk.eml", baslik.len(), "ilk", "ilkin"),
            blok("asiri.eml", 0, "ikinci", "ikincinin"),
            blok("cr.eml", 0, "üçüncü", "üçüncünün"),
        ]
        .concat(),
    )
    .unwrap();
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("çalıştır")
        .arg(&dizin)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dizin);
    let stdout = String::from_utf8_lossy(&cikti.stdout);
    assert!(
        cikti.status.success(),
        "{stdout}{}",
        String::from_utf8_lossy(&cikti.stderr)
    );
    let relaxed: String = govde
        .lines()
        .map(|satir| {
            format!(
                "{}\r\n",
                satir.split_whitespace().collect::<Vec<_>>().join(" ")
            )
        })
        .collect();
    let beklenen = sha_b64(relaxed.as_bytes());
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        vec![beklenen.as_str(), "sınır", "veri_bozuk"]
    );
}

/// K-417 RC fuzz bulgusu (26.11.2026, `b3b9d752`, GitHub `fuzz-rc`): son etiket
/// `b\t=` gibi adla `=` arasında FWS taşıyınca RFC 6376 §3.2'ye göre
/// geçerlidir; ürün denetimi kabul eder, ortak kâhin de panik yapmamalı.
#[test]
fn kahin_son_b_etiketinde_fws_kabul_eder() {
    let kok = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/dkim_imza_girdisi");
    // K-417: run 45838195765 crash girdisinin (430 bayt) elle küçültülmüş
    // biçimi; eski `ends_with("b=")` kâhini bunda panik yapıyordu.
    let bulgu = std::fs::read(kok.join("son-b-fws.txt")).unwrap();
    let bulgu_metni = std::str::from_utf8(&bulgu[1..]).unwrap();
    assert!(dil::dkim_imza::imza_girdisini_denetle(bulgu_metni).is_ok());
    assert!(!bulgu_metni.trim_end_matches([' ', '\t']).ends_with("b="));
    dkim_ozellikleri::denetle(&bulgu);
    let tohum = std::fs::read(kok.join("gecerli-girdi.txt")).unwrap();
    let metin = std::str::from_utf8(&tohum[1..]).unwrap();
    let govde = metin.strip_suffix("b=").expect("tohum `b=` ile biter");
    for son in ["b\t=", "b \t= ", "b\n\t="] {
        let girdi = format!("{govde}{son}");
        assert!(
            dil::dkim_imza::imza_girdisini_denetle(&girdi).is_ok(),
            "{son:?} kabul edilmeli"
        );
        let mut baytlar = vec![tohum[0]];
        baytlar.extend_from_slice(girdi.as_bytes());
        dkim_ozellikleri::denetle(&baytlar);
    }
    for son in ["bx=", "b=x"] {
        let girdi = format!("{govde}{son}");
        assert!(
            dil::dkim_imza::imza_girdisini_denetle(&girdi).is_err(),
            "{son:?}"
        );
    }
}

/// `dkim_imza_girdisi` fuzz korpusu stable'da aynı özellik denetiminden
/// geçer; her tohum deterministik xorshift ile 2000 kez bozulur (bayt
/// değiştirme, kesme, CR/LF ekleme).
#[test]
fn dkim_fuzz_korpusu_ozellikleri_korur() {
    let kok = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/dkim_imza_girdisi");
    let mut tohumlar: Vec<Vec<u8>> = std::fs::read_dir(&kok)
        .unwrap()
        .map(|g| std::fs::read(g.unwrap().path()).unwrap())
        .collect();
    tohumlar.sort();
    assert!(tohumlar.len() >= 6, "korpus tohumu eksik");
    let gecerli = tohumlar
        .iter()
        .filter(|t| {
            std::str::from_utf8(&t[1..])
                .is_ok_and(|m| dil::dkim_imza::imza_girdisini_denetle(m).is_ok())
        })
        .count();
    assert!(gecerli >= 2, "kâhin dalını kapsayan geçerli tohum yok");
    let mut durum = 0x9e37_79b9_7f4a_7c15_u64;
    let mut sonraki = || {
        durum ^= durum << 13;
        durum ^= durum >> 7;
        durum ^= durum << 17;
        durum
    };
    for tohum in &tohumlar {
        dkim_ozellikleri::denetle(tohum);
        for _ in 0..2000 {
            let mut bozuk = tohum.clone();
            let r = sonraki();
            let yer = (r as usize) % bozuk.len().max(1);
            match r % 4 {
                0 if !bozuk.is_empty() => bozuk[yer] ^= (r >> 8) as u8 | 1,
                1 => bozuk.truncate(yer),
                2 => bozuk.insert(yer.min(bozuk.len()), b'\r'),
                _ => bozuk.insert(yer.min(bozuk.len()), b'\n'),
            }
            dkim_ozellikleri::denetle(&bozuk);
        }
    }
}

// ---- K-413: açılışta yayımlanmış DKIM anahtarı (DNS p=) denetimi ----

/// Loopback'te tek yanıtlı sahte UDP çözücü: sorguya `yanitla` ile karşılık.
fn sahte_cozucu(yanitla: fn(&[u8], &str) -> Vec<u8>, txt: String) -> u16 {
    let soket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let kapi = soket.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let mut tampon = [0u8; 2048];
        while let Ok((n, kaynak)) = soket.recv_from(&mut tampon) {
            let _ = soket.send_to(&yanitla(&tampon[..n], &txt), kaynak);
        }
    });
    kapi
}

/// Sorgunun kimlik ve sorusunu kopyalayan yanıt; `txt` boşsa NOERROR/NODATA.
fn txt_yaniti(sorgu: &[u8], txt: &str) -> Vec<u8> {
    let mut i = 12;
    while sorgu[i] != 0 {
        i += 1 + usize::from(sorgu[i]);
    }
    let soru_sonu = i + 5;
    let cevap = u16::from(!txt.is_empty());
    let mut p = sorgu[..2].to_vec();
    p.extend_from_slice(&0x8180u16.to_be_bytes());
    p.extend_from_slice(&1u16.to_be_bytes());
    p.extend_from_slice(&cevap.to_be_bytes());
    p.extend_from_slice(&[0, 0, 0, 0]);
    p.extend_from_slice(&sorgu[12..soru_sonu]);
    if cevap == 1 {
        let mut veri = Vec::new();
        for parca in txt.as_bytes().chunks(255) {
            veri.push(parca.len() as u8);
            veri.extend_from_slice(parca);
        }
        p.extend_from_slice(&[0xC0, 0x0C, 0, 16, 0, 1, 0, 0, 0, 60]);
        p.extend_from_slice(&(veri.len() as u16).to_be_bytes());
        p.extend_from_slice(&veri);
    }
    p
}

fn servfail(sorgu: &[u8], _: &str) -> Vec<u8> {
    let mut p = txt_yaniti(sorgu, "");
    p[2..4].copy_from_slice(&0x8182u16.to_be_bytes());
    p
}

/// `dns` + `kriptografi` projesi; anahtar ortamdan, çözücü sahte. Program
/// imza dener: başarıda `imzalandı`, retde C034 `tür` yazar.
fn dns_surec(
    pem: &str,
    algoritma: &str,
    yanitla: fn(&[u8], &str) -> Vec<u8>,
    txt: String,
) -> (i32, String, String) {
    let kapi = sahte_cozucu(yanitla, txt);
    let dizin = gecici_dizin("dns-denetimi");
    std::fs::write(
        dizin.join("proje.dil"),
        format!(
            "proje \"dkimdns\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\n\
             yetkinlikler \"dns\", \"kriptografi\" listesi olsun\n\
             dns_çözücü \"127.0.0.1:{kapi}\" olsun\ndns_dnssec \"bilgi\" olsun\n\
             dkim_anahtarları \"ornek.test z2026a {algoritma} zee_dkim_dns_testi\" listesi olsun\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dizin.join("ana.dil"),
        format!(
            "sonuç {} verisi için \"zee_dkim_dns_testi\" özel anahtarıyla \"{algoritma}\" olarak imzalamayı dene olsun\n\
sonuç başarılıysa\n    \"imzalandı\" yaz\n\
değilse\n    hata sonucun hatası olsun\n    veri2 hatanın verisi olsun\n    veri2 \"tür\" değeri yaz\n",
            zee_metni(&girdi(algoritma))
        ),
    )
    .unwrap();
    let cikti = Command::new(env!("CARGO_BIN_EXE_dil"))
        .arg("çalıştır")
        .arg(&dizin)
        .env_remove("CREDENTIALS_DIRECTORY")
        .env("zee_dkim_dns_testi", pem)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dizin);
    (
        cikti.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&cikti.stdout).into_owned(),
        String::from_utf8_lossy(&cikti.stderr).into_owned(),
    )
}

/// PKCS#8 PEM'den `openssl pkey -pubout` SPKI DER'inin base64'ü (DNS'te
/// yaygın `p=` biçimi).
fn spki_p(pem: &str) -> String {
    let dizin = gecici_dizin("spki-p");
    std::fs::write(dizin.join("k.pem"), pem).unwrap();
    let cikti = Command::new("openssl")
        .args(["pkey", "-pubout", "-outform", "DER", "-in"])
        .arg(dizin.join("k.pem"))
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dizin);
    assert!(cikti.status.success());
    base64(&cikti.stdout)
}

fn ed_p(pem: &str) -> String {
    base64(
        &DkimImzaAnahtari::pemden("ornek.test", "z2026a", DkimAlgoritmasi::Ed25519Sha256, pem)
            .unwrap()
            .acik_anahtar(),
    )
}

#[test]
fn acilis_dns_denetimi_anahtari_devre_disi_birakir_sureci_durdurmaz() {
    let pem = ed25519_pem();
    let p = ed_p(&pem);
    let govde: String = pem.lines().filter(|s| !s.starts_with("-----")).collect();
    let ed = "ed25519-sha256";
    let (kod, cikti, hata) = dns_surec(&pem, ed, txt_yaniti, format!("v=DKIM1; k=ed25519; p={p}"));
    assert_eq!((kod, cikti.as_str()), (0, "imzalandı\n"), "{hata}");
    assert!(!hata.contains("UYARI") && !hata.contains("HATA"), "{hata}");

    let baska = ed_p(&ed25519_pem());
    for (txt, tur, parca) in [
        (
            format!("v=DKIM1; k=ed25519; p={baska}"),
            "anahtar_dns_uyuşmuyor",
            "uymuyor",
        ),
        (
            "v=DKIM1; k=ed25519; p=".to_string(),
            "anahtar_iptal",
            "iptal",
        ),
    ] {
        let (kod, cikti, hata) = dns_surec(&pem, ed, txt_yaniti, txt);
        assert_eq!((kod, cikti.trim()), (0, tur), "{hata}");
        assert!(
            hata.contains("HATA:") && hata.contains("ornek.test / z2026a"),
            "{hata}"
        );
        assert!(hata.contains(parca), "{hata}");
        assert!(
            !hata.contains(&govde[..16]) && !hata.contains(&p[..16]),
            "tanıda anahtar"
        );
    }
    for (yanitla, parca) in [
        (txt_yaniti as fn(&[u8], &str) -> Vec<u8>, "TXT kaydı yok"),
        (servfail, "denetlenemedi"),
    ] {
        let (kod, cikti, hata) = dns_surec(&pem, ed, yanitla, String::new());
        assert_eq!((kod, cikti.as_str()), (0, "imzalandı\n"), "{hata}");
        assert!(hata.contains("UYARI") && hata.contains(parca), "{hata}");
    }
}

#[test]
fn acilis_dns_denetimi_rsa_spki_p_ile_eslesir() {
    let pem = rsa_pem(2048);
    let p = spki_p(&pem);
    let rsa = "rsa-sha256";
    for txt in [format!("v=DKIM1; k=rsa; p={p}"), format!("v=DKIM1; p={p}")] {
        let (kod, cikti, hata) = dns_surec(&pem, rsa, txt_yaniti, txt);
        assert_eq!((kod, cikti.as_str()), (0, "imzalandı\n"), "{hata}");
        assert!(!hata.contains("HATA"), "{hata}");
    }
    let baska = spki_p(&rsa_pem(2048));
    let (kod, cikti, hata) = dns_surec(&pem, rsa, txt_yaniti, format!("v=DKIM1; k=rsa; p={baska}"));
    assert_eq!((kod, cikti.trim()), (0, "anahtar_dns_uyuşmuyor"), "{hata}");
}

/// İki kiracı: birinin DNS kaydı uymuyor ya da iptal, öteki imzalamaya
/// devam eder; kaydı olmayan anahtar etkin kalır.
#[test]
fn iki_anahtarli_depoda_yalniz_uyusmayan_devre_disi_kalir() {
    let (pem_a, pem_b) = (ed25519_pem(), ed25519_pem());
    let yukle = |secici: &str, pem: &str| {
        DkimImzaAnahtari::pemden("ornek.test", secici, DkimAlgoritmasi::Ed25519Sha256, pem).unwrap()
    };
    let p_a = base64(&yukle("a", &pem_a).acik_anahtar());
    let p_yabanci = base64(&yukle("x", &ed25519_pem()).acik_anahtar());
    let a = "ed25519-sha256";
    let veri = |secici: &str| girdi(a).replace("s=z2026a", &format!("s={secici}"));
    for (b_kaydi, b_sonucu) in [
        (
            Some(format!("v=DKIM1; k=ed25519; p={p_yabanci}")),
            Err("anahtar_dns_uyuşmuyor"),
        ),
        (
            Some("v=DKIM1; k=ed25519; p=".to_string()),
            Err("anahtar_iptal"),
        ),
        (None, Ok(())),
    ] {
        let mut depo = DkimAnahtarDeposu::default();
        depo.ekle("kiraci_a", yukle("a", &pem_a));
        depo.ekle("kiraci_b", yukle("b", &pem_b));
        let tanilar = depo.yayimlananlarla_denetle(|ad| match ad {
            "a._domainkey.ornek.test" => {
                Ok(vec![format!("v=DKIM1; k=ed25519; p={p_a}").into_bytes()])
            }
            "b._domainkey.ornek.test" => Ok(b_kaydi
                .clone()
                .map(String::into_bytes)
                .into_iter()
                .collect()),
            _ => Err("beklenmeyen ad".into()),
        });
        assert_eq!(tanilar.len(), 1, "{tanilar:?}");
        assert!(tanilar[0].contains("ornek.test / b"), "{tanilar:?}");
        let depo = Some(Arc::new(depo));
        assert_eq!(imzala(depo.clone(), &veri("a"), "kiraci_a", a).len(), 88);
        let b = imzala(depo, &veri("b"), "kiraci_b", a);
        match b_sonucu {
            Ok(()) => assert_eq!(b.len(), 88, "{b}"),
            Err(tur) => assert_eq!(b, format!("C034 {tur}")),
        }
    }
}
