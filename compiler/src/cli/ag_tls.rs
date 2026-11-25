//! K-381 (RFC-0039 A2b): adlı dinleyici bağlantılarının sunucu TLS'i.
//!
//! Private CLI adapter. Sertifika zinciri ve özel anahtar proje bildirimindeki
//! değişken adlarıyla K-339 sözleşmesinden (önce systemd credential dizini,
//! yoksa ortam) okunur; anahtar baytı hiçbir zaman Zee değeri, IO izi,
//! günlük ya da tanı metni olmaz. Politika sabittir: `rustls` + `ring`,
//! TLS 1.2 ve 1.3, yalnız ileri gizlilikli AEAD takımları, yeniden müzakere
//! ve sıkıştırma yok, istemci sertifikası yok. Yükleme kaynak yürütülmeden
//! önce yapılır; hata çıkış 2'dir.
//!
//! K-397 (RFC-0044 B4a, ADR-173): giden bağlantıda istemci TLS'i. Aynı sabit
//! sağlayıcı ve sürüm politikası; kökler `webpki-roots`. PKIX sonucu el
//! sıkışmayı durdurmaz, `KayitliDogrulayici` ile kayda yazılır (DANE-EE ve
//! fırsatçı TLS bunu ister); el sıkışma imzası her durumda `WebPkiServerVerifier`
//! ile doğrulanır, yani karşı taraf raporlanan SPKI'nin anahtarına sahiptir.

use dil::guvenlik::sir_dizini::{gizli_deger_oku, SatirProfili};
use dil::yorumlayici::TlsIstemciKaydi;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::WebPkiServerVerifier;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::{CertificateError, DigitallySignedStruct, SignatureScheme};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex, OnceLock};

/// PEM zinciri ve anahtarı için okuma üst sınırı.
const AZAMI_PEM_BAYTI: usize = 64 * 1024;

static YAPILANDIRMA: OnceLock<Arc<rustls::ServerConfig>> = OnceLock::new();

/// Sabit politika: TLS 1.3 üç takım, TLS 1.2 ECDHE + AES-GCM/CHACHA20.
pub(crate) fn saglayici() -> rustls::crypto::CryptoProvider {
    let mut saglayici = rustls::crypto::ring::default_provider();
    saglayici.cipher_suites.retain(|takim| {
        matches!(
            takim.suite(),
            rustls::CipherSuite::TLS13_AES_128_GCM_SHA256
                | rustls::CipherSuite::TLS13_AES_256_GCM_SHA384
                | rustls::CipherSuite::TLS13_CHACHA20_POLY1305_SHA256
                | rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
                | rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
                | rustls::CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
                | rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
                | rustls::CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
                | rustls::CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
        )
    });
    saglayici
}

fn pem_oku(degisken: &str, ne: &str) -> Result<String, String> {
    gizli_deger_oku(degisken, AZAMI_PEM_BAYTI, SatirProfili::CokSatir)
        .map_err(|hata| format!("TLS {ne} (`{degisken}`) okunamadı: {}", hata.neden()))
}

/// PEM metinlerinden sabit politikalı sunucu yapılandırması.
pub(crate) fn yapilandirma_kur(
    zincir_pem: &str,
    anahtar_pem: &str,
) -> Result<rustls::ServerConfig, String> {
    let zincir: Vec<CertificateDer<'static>> =
        CertificateDer::pem_slice_iter(zincir_pem.as_bytes())
            .collect::<Result<_, _>>()
            .map_err(|_| "TLS sertifika zinciri PEM olarak çözülemedi".to_string())?;
    if zincir.is_empty() {
        return Err("TLS sertifika zinciri boş".into());
    }
    // Hata iletisi anahtar içeriğini taşımaz.
    let anahtar = PrivateKeyDer::from_pem_slice(anahtar_pem.as_bytes())
        .map_err(|_| "TLS özel anahtarı PEM olarak çözülemedi".to_string())?;
    let mut yapilandirma = rustls::ServerConfig::builder_with_provider(Arc::new(saglayici()))
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
        .map_err(|_| "TLS sürüm politikası kurulamadı".to_string())?
        .with_no_client_auth()
        .with_single_cert(zincir, anahtar)
        .map_err(|_| {
            "TLS sertifikası ile özel anahtar uyuşmuyor ya da desteklenmiyor".to_string()
        })?;
    yapilandirma.ignore_client_order = true;
    yapilandirma.max_early_data_size = 0;
    Ok(yapilandirma)
}

/// Bildirimli projede açılışta bir kez: zinciri ve anahtarı okur, doğrular.
pub(crate) fn hazirla(bildirim: &dil::ag_satiri::TlsBildirimi) -> Result<(), String> {
    let zincir = pem_oku(&bildirim.sertifika_degiskeni, "sertifika zinciri")?;
    let anahtar = pem_oku(&bildirim.anahtar_degiskeni, "özel anahtarı")?;
    let yapilandirma = yapilandirma_kur(&zincir, &anahtar)?;
    drop(anahtar);
    YAPILANDIRMA
        .set(Arc::new(yapilandirma))
        .map_err(|_| "TLS yapılandırması iki kez kuruldu".to_string())
}

pub(crate) fn yapilandirma() -> Option<Arc<rustls::ServerConfig>> {
    YAPILANDIRMA.get().cloned()
}

/// K-397: istemci TLS kökleri (süreç başına bir kez).
static ISTEMCI_KOKLERI: OnceLock<Arc<rustls::RootCertStore>> = OnceLock::new();

fn istemci_kokleri() -> Arc<rustls::RootCertStore> {
    Arc::clone(ISTEMCI_KOKLERI.get_or_init(|| {
        let mut kokler = rustls::RootCertStore::empty();
        kokler.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        test_koklerini_ekle(&mut kokler);
        Arc::new(kokler)
    }))
}

/// Yalnız hata ayıklama derlemesinde (testler): `ZEE_TEST_TLS_KOKLERI` PEM
/// dosyasındaki ek kökler. Sürüm derlemesi bu değişkeni hiç okumaz.
#[cfg(debug_assertions)]
fn test_koklerini_ekle(kokler: &mut rustls::RootCertStore) {
    let Some(yol) = std::env::var_os("ZEE_TEST_TLS_KOKLERI") else {
        return;
    };
    if let Ok(pem) = std::fs::read(yol) {
        for kok in CertificateDer::pem_slice_iter(&pem).flatten() {
            let _ = kokler.add(kok);
        }
    }
}

#[cfg(not(debug_assertions))]
fn test_koklerini_ekle(_: &mut rustls::RootCertStore) {}

type DogrulamaSonucu = Arc<Mutex<Option<Result<(), rustls::Error>>>>;

/// PKIX sonucunu kaydedip el sıkışmayı sürdüren doğrulayıcı. İmza
/// doğrulamaları iç doğrulayıcıya bırakılır (atlanmaz).
#[derive(Debug)]
struct KayitliDogrulayici {
    ic: Arc<WebPkiServerVerifier>,
    sonuc: DogrulamaSonucu,
}

impl ServerCertVerifier for KayitliDogrulayici {
    fn verify_server_cert(
        &self,
        uc: &CertificateDer<'_>,
        araciler: &[CertificateDer<'_>],
        ad: &ServerName<'_>,
        ocsp: &[u8],
        simdi: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        if araciler.len() >= dil::ag_satiri::AZAMI_TLS_ZINCIRI {
            return Err(rustls::Error::InvalidCertificate(
                CertificateError::BadEncoding,
            ));
        }
        let sonuc = self
            .ic
            .verify_server_cert(uc, araciler, ad, ocsp, simdi)
            .map(|_| ());
        if let Ok(mut yuva) = self.sonuc.lock() {
            *yuva = Some(sonuc);
        }
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        mesaj: &[u8],
        sertifika: &CertificateDer<'_>,
        imza: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.ic.verify_tls12_signature(mesaj, sertifika, imza)
    }

    fn verify_tls13_signature(
        &self,
        mesaj: &[u8],
        sertifika: &CertificateDer<'_>,
        imza: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.ic.verify_tls13_signature(mesaj, sertifika, imza)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.ic.supported_verify_schemes()
    }
}

/// PKIX hatasının Zee'ye giden kararlı adı.
fn dogrulama_hatasi_adi(hata: &rustls::Error) -> &'static str {
    match hata {
        rustls::Error::InvalidCertificate(neden) => match neden {
            CertificateError::NotValidForName | CertificateError::NotValidForNameContext { .. } => {
                "ad_uyuşmuyor"
            }
            CertificateError::Expired | CertificateError::ExpiredContext { .. } => "süresi_dolmuş",
            CertificateError::NotValidYet | CertificateError::NotValidYetContext { .. } => {
                "henüz_geçerli_değil"
            }
            CertificateError::UnknownIssuer => "bilinmeyen_yayıncı",
            CertificateError::Revoked => "iptal",
            _ => "geçersiz_sertifika",
        },
        _ => "geçersiz_sertifika",
    }
}

fn onaltilik(bayt: &[u8]) -> String {
    bayt.iter().map(|b| format!("{b:02x}")).collect()
}

/// Bağlantının bayt kanalı: düz TCP ya da `rustls` sunucu/istemci oturumu.
/// `Read`/`Write` düz metin verir; TLS kayıtları engellemeyen sokete taşınır.
pub(crate) struct Kanal {
    pub(crate) akis: std::net::TcpStream,
    tls: Option<Box<rustls::Connection>>,
    /// K-397: istemci oturumunun PKIX sonucu ve istenen sunucu adı.
    istemci: Option<(DogrulamaSonucu, String)>,
}

fn tls_hatasi(_: rustls::Error) -> std::io::Error {
    // Ayrıntı (sertifika, uzak taraf iletisi) günlüğe/Zee'ye taşınmaz.
    std::io::Error::new(std::io::ErrorKind::InvalidData, "TLS protokol hatası")
}

impl Kanal {
    pub(crate) fn duz(akis: std::net::TcpStream) -> Self {
        Self {
            akis,
            tls: None,
            istemci: None,
        }
    }

    /// K-397: giden bağlantıda istemci oturumu (SNI = `sunucu_adi`).
    pub(crate) fn tls_istemci_ac(&mut self, sunucu_adi: &str) -> Result<(), String> {
        let ad = ServerName::try_from(sunucu_adi.to_string())
            .map_err(|_| "TLS sunucu adı geçersiz".to_string())?;
        let saglayici = Arc::new(saglayici());
        let ic =
            WebPkiServerVerifier::builder_with_provider(istemci_kokleri(), Arc::clone(&saglayici))
                .build()
                .map_err(|_| "TLS doğrulayıcısı kurulamadı".to_string())?;
        let sonuc: DogrulamaSonucu = Arc::new(Mutex::new(None));
        let mut yapilandirma = rustls::ClientConfig::builder_with_provider(saglayici)
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
            .map_err(|_| "TLS sürüm politikası kurulamadı".to_string())?
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(KayitliDogrulayici {
                ic,
                sonuc: Arc::clone(&sonuc),
            }))
            .with_no_client_auth();
        yapilandirma.enable_early_data = false;
        yapilandirma.resumption = rustls::client::Resumption::disabled();
        let mut oturum = rustls::ClientConnection::new(Arc::new(yapilandirma), ad)
            .map_err(|_| "TLS oturumu kurulamadı".to_string())?;
        oturum.set_buffer_limit(Some(dil::ag_satiri::GIDEN_TAMPON_BAYTI));
        self.tls = Some(Box::new(rustls::Connection::Client(oturum)));
        self.istemci = Some((sonuc, sunucu_adi.to_ascii_lowercase()));
        Ok(())
    }

    /// K-397: tamamlanan istemci el sıkışmasının kaydı.
    pub(crate) fn istemci_kaydi(&self) -> TlsIstemciKaydi {
        let mut kayit = TlsIstemciKaydi::default();
        let (Some(tls), Some((sonuc, ad))) = (self.tls.as_ref(), self.istemci.as_ref()) else {
            return kayit;
        };
        let pkix = sonuc.lock().ok().and_then(|mut yuva| yuva.take());
        match pkix {
            Some(Ok(())) => kayit.dogrulandi = true,
            Some(Err(hata)) => kayit.dogrulama_hatasi = dogrulama_hatasi_adi(&hata).into(),
            None => kayit.dogrulama_hatasi = "geçersiz_sertifika".into(),
        }
        kayit.sunucu_adi = ad.clone();
        kayit.surum = tls
            .protocol_version()
            .map_or_else(|| "?".into(), |s| format!("{s:?}"));
        kayit.takim = tls
            .negotiated_cipher_suite()
            .map_or_else(|| "?".into(), |s| format!("{:?}", s.suite()));
        for sertifika in tls.peer_certificates().unwrap_or_default() {
            kayit
                .zincir_sha256
                .push(onaltilik(&Sha256::digest(sertifika.as_ref())));
            let spki = dil::ag_satiri::x509::spki_dilimi(sertifika.as_ref())
                .map(|spki| onaltilik(&Sha256::digest(spki)))
                .unwrap_or_default();
            kayit.zincir_spki_sha256.push(spki);
        }
        kayit.sertifika_sha256 = kayit.zincir_sha256.first().cloned().unwrap_or_default();
        kayit.spki_sha256 = kayit
            .zincir_spki_sha256
            .first()
            .cloned()
            .unwrap_or_default();
        if kayit.spki_sha256.is_empty() && kayit.dogrulandi {
            kayit.dogrulandi = false;
            kayit.dogrulama_hatasi = "geçersiz_sertifika".into();
        }
        kayit
    }

    /// Implicit TLS ya da STARTTLS için sunucu oturumu başlatır.
    pub(crate) fn tls_ac(&mut self) -> Result<(), String> {
        let yapilandirma =
            yapilandirma().ok_or_else(|| dil::ag_satiri::TLS_KIMLIGI_YOK.to_string())?;
        let mut oturum = rustls::ServerConnection::new(yapilandirma)
            .map_err(|_| "TLS oturumu kurulamadı".to_string())?;
        oturum.set_buffer_limit(Some(dil::ag_satiri::GIDEN_TAMPON_BAYTI));
        self.tls = Some(Box::new(rustls::Connection::Server(oturum)));
        Ok(())
    }

    pub(crate) fn tls_mi(&self) -> bool {
        self.tls.is_some()
    }

    /// Sokete taşınmayı bekleyen TLS kaydı var mı (POLLOUT gerekir).
    pub(crate) fn yazma_bekliyor(&self) -> bool {
        self.tls.as_ref().is_some_and(|t| t.wants_write())
    }

    /// Bekleyen TLS kayıtlarını engellemeden sokete yazar.
    pub(crate) fn tls_bosalt(&mut self) -> std::io::Result<()> {
        let Some(tls) = self.tls.as_mut() else {
            return Ok(());
        };
        while tls.wants_write() {
            match tls.write_tls(&mut self.akis) {
                Ok(0) => return Err(std::io::ErrorKind::WriteZero.into()),
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Gelen TLS kayıtlarını işler; `Ok(true)` el sıkışma bitti.
    pub(crate) fn el_sikis(&mut self) -> std::io::Result<bool> {
        loop {
            self.tls_bosalt()?;
            let Some(tls) = self.tls.as_mut() else {
                return Ok(true);
            };
            if !tls.is_handshaking() {
                return Ok(true);
            }
            match tls.read_tls(&mut self.akis) {
                Ok(0) => return Err(std::io::ErrorKind::UnexpectedEof.into()),
                Ok(_) => {
                    if let Err(hata) = tls.process_new_packets() {
                        let _ = self.tls_bosalt();
                        return Err(tls_hatasi(hata));
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
    }

    /// Anlaşılan sürüm ve şifre takımı (Zee'ye `Sonuç<Metin>`).
    pub(crate) fn tls_bilgisi(&self) -> String {
        let Some(tls) = self.tls.as_ref() else {
            return String::new();
        };
        let surum = tls
            .protocol_version()
            .map_or_else(|| "?".into(), |s| format!("{s:?}"));
        let takim = tls
            .negotiated_cipher_suite()
            .map_or_else(|| "?".into(), |s| format!("{:?}", s.suite()));
        format!("{surum} {takim}")
    }

    /// Kapanışta TLS close_notify kuyruğa alınır.
    pub(crate) fn kapanis_bildir(&mut self) {
        if let Some(tls) = self.tls.as_mut() {
            tls.send_close_notify();
        }
    }
}

impl std::io::Read for Kanal {
    fn read(&mut self, tampon: &mut [u8]) -> std::io::Result<usize> {
        if self.tls.is_none() {
            return self.akis.read(tampon);
        }
        loop {
            let Some(tls) = self.tls.as_mut() else {
                return Ok(0);
            };
            match tls.reader().read(tampon) {
                Ok(n) => return Ok(n),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e),
            }
            match tls.read_tls(&mut self.akis) {
                Ok(0) => return Ok(0),
                Ok(_) => {
                    if let Err(hata) = tls.process_new_packets() {
                        let _ = self.tls_bosalt();
                        return Err(tls_hatasi(hata));
                    }
                    self.tls_bosalt()?;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl std::io::Write for Kanal {
    fn write(&mut self, veri: &[u8]) -> std::io::Result<usize> {
        let Some(tls) = self.tls.as_mut() else {
            return self.akis.write(veri);
        };
        let n = tls.writer().write(veri)?;
        self.tls_bosalt()?;
        if n == 0 && !veri.is_empty() {
            return Err(std::io::ErrorKind::WouldBlock.into());
        }
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.tls_bosalt()
    }
}
