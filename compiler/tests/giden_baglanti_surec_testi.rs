//! K-397 (RFC-0044 B4a, ADR-173): gerçek `dil` süreci + gerçek TCP ile giden
//! bağlantı ve istemci STARTTLS. Test sunucusu bu süreçte rustls ile koşar;
//! sertifikalar test anında `openssl` ile üretilir (depoda özel anahtar yok).
//! Geçerli zincir, yanlış SNI, kendinden imzalı ve süresi dolmuş sertifika;
//! SPKI özeti openssl'in bağımsız hesabıyla karşılaştırılır. `ZEE_TEST_TLS_KOKLERI`
//! yalnız hata ayıklama derlemesinde okunur.
#![cfg(unix)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

static SIRA: AtomicUsize = AtomicUsize::new(0);

fn gecici(on_ek: &str) -> PathBuf {
    let kok = PathBuf::from("/tmp").join(format!(
        "{on_ek}-{}-{}",
        std::process::id(),
        SIRA.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&kok).unwrap();
    kok
}

fn openssl(kok: &Path, argumanlar: &[&str]) -> Vec<u8> {
    let cikti = Command::new("openssl")
        .args(argumanlar)
        .current_dir(kok)
        .output()
        .expect("openssl bulunamadı (TLS testleri test sertifikasını openssl ile üretir)");
    assert!(
        cikti.status.success(),
        "openssl {argumanlar:?}: {}",
        String::from_utf8_lossy(&cikti.stderr)
    );
    cikti.stdout
}

struct Kimlikler {
    kok: PathBuf,
}

impl Drop for Kimlikler {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.kok);
    }
}

impl Kimlikler {
    fn yol(&self, ad: &str) -> PathBuf {
        self.kok.join(ad)
    }

    fn oku(&self, ad: &str) -> Vec<u8> {
        std::fs::read(self.yol(ad)).unwrap()
    }

    /// `ad`.pem yaprağının SPKI SHA-256'sı (openssl ile bağımsız).
    fn spki_sha256(&self, ad: &str) -> String {
        let pem = self.yol(&format!("{ad}.pem"));
        let anahtar = openssl(
            &self.kok,
            &["x509", "-in", pem.to_str().unwrap(), "-pubkey", "-noout"],
        );
        std::fs::write(self.yol(&format!("{ad}.pub")), anahtar).unwrap();
        openssl(
            &self.kok,
            &[
                "pkey",
                "-pubin",
                "-in",
                &format!("{ad}.pub"),
                "-outform",
                "DER",
                "-out",
                &format!("{ad}.spki"),
            ],
        );
        let der = self.oku(&format!("{ad}.spki"));
        let dgst = openssl(&self.kok, &["dgst", "-sha256", "-r", &format!("{ad}.spki")]);
        assert!(!der.is_empty());
        String::from_utf8(dgst)
            .unwrap()
            .split(' ')
            .next()
            .unwrap()
            .to_string()
    }
}

const EC: [&str; 5] = [
    "-newkey",
    "ec",
    "-pkeyopt",
    "ec_paramgen_curve:prime256v1",
    "-nodes",
];

/// Test CA ve üç yaprak: `gecerli` (mx.zee.test, CA imzalı), `dolmus`
/// (aynı ad, süresi 2020'de bitti), `kendi` (kendinden imzalı).
fn kimlikler() -> Kimlikler {
    let kok = gecici("zk397k");
    let mut ca = vec!["req", "-x509"];
    ca.extend(EC);
    ca.extend([
        "-keyout",
        "ca.key",
        "-out",
        "ca.pem",
        "-days",
        "2",
        "-subj",
        "/CN=Zee K397 Test CA",
        "-addext",
        "basicConstraints=critical,CA:TRUE",
        "-addext",
        "keyUsage=critical,keyCertSign,cRLSign",
        "-addext",
        "subjectKeyIdentifier=hash",
    ]);
    openssl(&kok, &ca);
    std::fs::write(
        kok.join("ek.cnf"),
        "basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature\nextendedKeyUsage=serverAuth\nsubjectAltName=DNS:mx.zee.test\nsubjectKeyIdentifier=hash\nauthorityKeyIdentifier=keyid,issuer\n",
    )
    .unwrap();
    for (ad, zaman) in [
        ("gecerli", ["-days", "2", "", ""]),
        (
            "dolmus",
            [
                "-not_before",
                "20190101000000Z",
                "-not_after",
                "20200101000000Z",
            ],
        ),
    ] {
        let mut istek = vec!["req"];
        istek.extend(EC);
        let anahtar = format!("{ad}.key");
        let csr = format!("{ad}.csr");
        istek.extend([
            "-keyout",
            &anahtar,
            "-out",
            &csr,
            "-subj",
            "/CN=mx.zee.test",
        ]);
        openssl(&kok, &istek);
        let pem = format!("{ad}.pem");
        let mut imza = vec![
            "x509",
            "-req",
            "-in",
            &csr,
            "-CA",
            "ca.pem",
            "-CAkey",
            "ca.key",
            "-CAcreateserial",
            "-out",
            &pem,
            "-extfile",
            "ek.cnf",
        ];
        imza.extend(zaman.iter().filter(|z| !z.is_empty()));
        openssl(&kok, &imza);
    }
    let mut kendi = vec!["req", "-x509"];
    kendi.extend(EC);
    kendi.extend([
        "-keyout",
        "kendi.key",
        "-out",
        "kendi.pem",
        "-days",
        "2",
        "-subj",
        "/CN=mx.zee.test",
        "-addext",
        "subjectAltName=DNS:mx.zee.test",
        // openssl varsayılanı CA:TRUE'dur; webpki onu `geçersiz_sertifika`
        // (CA uç sertifika olarak) sayar. Tipik kendinden imzalı MX yaprağı:
        "-addext",
        "basicConstraints=critical,CA:FALSE",
    ]);
    openssl(&kok, &kendi);
    Kimlikler { kok }
}

fn sunucu_yapilandirmasi(k: &Kimlikler, ad: &str) -> Arc<rustls::ServerConfig> {
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    let mut zincir: Vec<CertificateDer<'static>> =
        CertificateDer::pem_slice_iter(&k.oku(&format!("{ad}.pem")))
            .collect::<Result<_, _>>()
            .unwrap();
    if ad != "kendi" {
        zincir.extend(CertificateDer::pem_slice_iter(&k.oku("ca.pem")).map(Result::unwrap));
    }
    let anahtar = PrivateKeyDer::from_pem_slice(&k.oku(&format!("{ad}.key"))).unwrap();
    Arc::new(
        rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(zincir, anahtar)
        .unwrap(),
    )
}

fn satir(okuyucu: &mut impl BufRead) -> String {
    let mut metin = String::new();
    okuyucu.read_line(&mut metin).unwrap();
    metin.trim_end().to_string()
}

/// Tek oturumluk SMTP benzeri sunucu: 220 → EHLO/250 → STARTTLS/220 →
/// TLS → NOOP/250 tls-ok. Dönen dizi sunucunun aldığı satırlardır.
fn smtp_sunucusu(yapilandirma: Arc<rustls::ServerConfig>) -> (SocketAddr, JoinHandle<Vec<String>>) {
    let dinleyici = TcpListener::bind("127.0.0.1:0").unwrap();
    let adres = dinleyici.local_addr().unwrap();
    let is = std::thread::spawn(move || {
        let (akis, _) = dinleyici.accept().unwrap();
        akis.set_read_timeout(Some(Duration::from_secs(20)))
            .unwrap();
        let mut alinan = Vec::new();
        let mut yazici = akis.try_clone().unwrap();
        let mut okuyucu = BufReader::new(akis.try_clone().unwrap());
        yazici.write_all(b"220 mx.zee.test ESMTP\r\n").unwrap();
        alinan.push(satir(&mut okuyucu));
        yazici.write_all(b"250 mx.zee.test\r\n").unwrap();
        alinan.push(satir(&mut okuyucu));
        yazici.write_all(b"220 2.0.0 hazir\r\n").unwrap();
        let oturum = rustls::ServerConnection::new(yapilandirma).unwrap();
        let mut tls = rustls::StreamOwned::new(oturum, akis);
        let mut tls_satir = Vec::new();
        let mut bayt = [0u8; 1];
        while tls.read(&mut bayt).unwrap_or(0) == 1 {
            if bayt[0] == b'\n' {
                break;
            }
            tls_satir.push(bayt[0]);
        }
        alinan.push(String::from_utf8_lossy(&tls_satir).trim_end().to_string());
        let _ = tls.write_all(b"250 tls-ok\r\n");
        let _ = tls.flush();
        std::thread::sleep(Duration::from_millis(200));
        alinan
    });
    (adres, is)
}

struct Proje {
    kok: PathBuf,
}

impl Drop for Proje {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.kok);
    }
}

fn proje(hedefler: &str, kaynak: &str) -> Proje {
    let kok = gecici("zk397p");
    std::fs::write(
        kok.join("proje.dil"),
        format!(
            "proje \"giden\" olsun\nsürüm \"0.1.0\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ\", \"yerel-ağ\", \"dosya-okuma\" listesi olsun\nağ_hedefleri {hedefler} listesi olsun\n"
        ),
    )
    .unwrap();
    std::fs::write(kok.join("ana.dil"), kaynak).unwrap();
    Proje { kok }
}

fn calistir(p: &Proje, kokler: Option<&Path>) -> (Vec<String>, String) {
    let mut komut = Command::new(env!("CARGO_BIN_EXE_dil"));
    komut
        .arg("çalıştır")
        .arg(&p.kok)
        .current_dir(&p.kok)
        .env_remove("CREDENTIALS_DIRECTORY")
        .env_remove("ZEE_TEST_TLS_KOKLERI");
    if let Some(kokler) = kokler {
        komut.env("ZEE_TEST_TLS_KOKLERI", kokler);
    }
    let cikti = komut.output().unwrap();
    (
        String::from_utf8_lossy(&cikti.stdout)
            .lines()
            .map(str::to_string)
            .collect(),
        String::from_utf8_lossy(&cikti.stderr).into_owned(),
    )
}

fn tls_kaynagi(adres: SocketAddr, sni: &str) -> String {
    format!(
        r#"sonuç "{adres}" adresine bağlanmayı dene olsun
sonuç başarılıysa
    selam bağlantıdan satır okumayı dene olsun
    selam başarılıysa
        selamın değeri yaz
    bağlantıya "EHLO gonderen.zee.test" satırını yaz
    ehlo bağlantıdan satır okumayı 5 saniye içinde dene olsun
    bağlantıya "STARTTLS" satırını yaz
    hazır bağlantıdan satır okumayı dene olsun
    tls bağlantıda TLS istemcisi başlatmayı dene "{sni}" ile olsun
    tls başarılıysa
        rapor tlsin değeri olsun
        raporun "doğrulandı" değeri yaz
        raporun "doğrulama_hatası" değeri yaz
        raporun "sürüm" değeri yaz
        raporun "spki_sha256" değeri yaz
        raporun "zincir_spki_sha256" değeri yaz
        bağlantıya "NOOP" satırını yaz
        noop bağlantıdan satır okumayı dene olsun
        noop başarılıysa
            noopun değeri yaz
    değilse
        hata tlsin hatası olsun
        veri hatanın verisi olsun
        "tls hatası: " ile verinin "tür" değeri yaz
    bağlantıyı kapat
değilse
    hata sonucun hatası olsun
    veri hatanın verisi olsun
    "bağlanma hatası: " ile verinin "tür" değeri yaz
"#
    )
}

fn tls_kosusu(sertifika: &str, sni: &str) -> (Vec<String>, Vec<String>, Kimlikler) {
    let k = kimlikler();
    let (adres, is) = smtp_sunucusu(sunucu_yapilandirmasi(&k, sertifika));
    let p = proje(&format!("\"tcp://{adres}\""), &tls_kaynagi(adres, sni));
    let (cikti, hata) = calistir(&p, Some(&k.yol("ca.pem")));
    let alinan = is.join().unwrap_or_default();
    assert!(!cikti.is_empty(), "çıktı yok; stderr: {hata}");
    (cikti, alinan, k)
}

#[test]
fn gecerli_zincir_dogrulanir_spki_openssl_ile_esit_ve_tls_ustunden_konusur() {
    let (cikti, alinan, k) = tls_kosusu("gecerli", "mx.zee.test");
    let spki = k.spki_sha256("gecerli");
    assert_eq!(cikti[0], "220 mx.zee.test ESMTP");
    assert_eq!(&cikti[1..3], ["evet", ""], "{cikti:?}");
    assert!(cikti[3].starts_with("TLSv1_"), "{cikti:?}");
    assert_eq!(cikti[4], spki);
    let zincir: Vec<&str> = cikti[5].split(' ').collect();
    assert_eq!(zincir.len(), 2, "yaprak + CA");
    assert_eq!(zincir[0], spki);
    assert_eq!(cikti[6], "250 tls-ok");
    assert_eq!(alinan, vec!["EHLO gonderen.zee.test", "STARTTLS", "NOOP"]);
}

#[test]
fn yanlis_sni_kendinden_imzali_ve_suresi_dolmus_acik_hata_verir() {
    for (sertifika, sni, neden) in [
        ("gecerli", "baska.zee.test", "ad_uyuşmuyor"),
        ("kendi", "mx.zee.test", "bilinmeyen_yayıncı"),
        ("dolmus", "mx.zee.test", "süresi_dolmuş"),
    ] {
        let (cikti, alinan, k) = tls_kosusu(sertifika, sni);
        assert_eq!(
            &cikti[1..3],
            ["hayır", neden],
            "{sertifika}/{sni}: {cikti:?}"
        );
        // El sıkışma tamamlandı (DANE/fırsatçı TLS), SPKI yine raporlandı.
        assert_eq!(cikti[4], k.spki_sha256(sertifika), "{sertifika}");
        assert_eq!(cikti.last().unwrap(), "250 tls-ok");
        assert_eq!(alinan.last().unwrap(), "NOOP");
    }
}

#[test]
fn kok_verilmezse_test_ca_tanimsiz_kalir() {
    let k = kimlikler();
    let (adres, is) = smtp_sunucusu(sunucu_yapilandirmasi(&k, "gecerli"));
    let p = proje(
        &format!("\"tcp://{adres}\""),
        &tls_kaynagi(adres, "mx.zee.test"),
    );
    let (cikti, hata) = calistir(&p, None);
    let _ = is.join();
    assert_eq!(
        &cikti[1..3],
        ["hayır", "bilinmeyen_yayıncı"],
        "{cikti:?} {hata}"
    );
}

#[test]
fn joker_yerel_adrese_gercek_soket_acmaz_ve_izinsiz_kapi_reddedilir() {
    let dinleyici = TcpListener::bind("127.0.0.1:0").unwrap();
    dinleyici.set_nonblocking(true).unwrap();
    let adres = dinleyici.local_addr().unwrap();
    let kaynak = format!(
        "{}{}",
        tls_kaynagi(adres, "mx.zee.test"),
        tls_kaynagi("127.0.0.1:25".parse().unwrap(), "mx.zee.test")
    );
    let p = proje("\"tcp://*:25\"", &kaynak);
    let (cikti, hata) = calistir(&p, None);
    assert_eq!(
        cikti,
        vec![
            "bağlanma hatası: hedef_yasak",
            "bağlanma hatası: hedef_yasak"
        ],
        "{hata}"
    );
    assert_eq!(
        dinleyici.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock,
        "hiç bağlantı açılmamalı"
    );
}

#[test]
fn duz_giden_baglantida_parca_aktarim_ve_sureli_okuma_calisir() {
    let dinleyici = TcpListener::bind("127.0.0.1:0").unwrap();
    let adres = dinleyici.local_addr().unwrap();
    let is = std::thread::spawn(move || {
        let (mut akis, _) = dinleyici.accept().unwrap();
        akis.set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        akis.write_all(b"220 hazir\r\n").unwrap();
        let mut alinan = Vec::new();
        let mut parca = [0u8; 256];
        while !alinan.ends_with(b"SON\r\n") {
            match akis.read(&mut parca) {
                Ok(0) | Err(_) => break,
                Ok(n) => alinan.extend_from_slice(&parca[..n]),
            }
        }
        // Yanıt vermeden bekle: istemcinin 1 sn'lik okuması zaman aşımına düşer.
        std::thread::sleep(Duration::from_millis(2500));
        let _ = akis.write_all(b"250 gec\r\n");
        std::thread::sleep(Duration::from_millis(300));
        alinan
    });
    let kaynak = format!(
        r#"sonuç "{adres}" adresine bağlanmayı dene olsun
sonuç başarılıysa
    selam bağlantıdan satır okumayı dene olsun
    selam başarılıysa
        selamın değeri yaz
    parça bağlantıya "DATA" parçasını yazmayı dene olsun
    parça başarılıysa
        bağlantıya "" satırını yaz
    aktarım bağlantıya "ileti.txt" dosyasını aktarmayı dene olsun
    aktarım başarılıysa
        "aktarılan " ile aktarımın değeri yaz
    bağlantıya "SON" satırını yaz
    bekleme bağlantıdan satır okumayı 1 saniye içinde dene olsun
    bekleme başarısızsa
        hata beklemenin hatası olsun
        veri hatanın verisi olsun
        verinin "tür" değeri yaz
    ikinci bağlantıdan satır okumayı dene olsun
    ikinci başarılıysa
        ikincinin değeri yaz
    bağlantıyı kapat
"#
    );
    let p = proje(&format!("\"tcp://{adres}\""), &kaynak);
    std::fs::write(p.kok.join("ileti.txt"), "Konu: deneme\r\n\r\ngövde\r\n").unwrap();
    let (cikti, hata) = calistir(&p, None);
    let alinan = is.join().unwrap();
    assert_eq!(
        cikti,
        vec!["220 hazir", "aktarılan 24", "zaman_aşımı", "250 gec"],
        "{hata}"
    );
    assert_eq!(
        String::from_utf8(alinan).unwrap(),
        "DATA\r\nKonu: deneme\r\n\r\ngövde\r\nSON\r\n"
    );
}
