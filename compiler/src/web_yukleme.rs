//! Native HTTP binary gövdesini bellekte biriktirmeden geçici dosyaya alır.
//!
//! K-332: geçici `.parca` dosyası isteğe aittir. Akış yarıda kalırsa
//! `AkanYukleme` düşerken, tamamlanmış yükleme ise istek bittiğinde
//! `IstekYuklemesi` düşerken dosyayı siler; rota dosyayı taşıdıysa yolda
//! dosya yoktur ve silme etkisizdir.
//!
//! K-332 eki: süreç istek ortasında ölürse Drop koşmaz. Worker açılışta ve
//! çalışırken en sık `YETIM_SUPURME_ARALIGI`'nda bir, yalnız runtime
//! adlandırmasına uyan ve `YETIM_PARCA_YASI`'ndan eski `.parca` dosyalarını
//! süpürür. Canlı yükleme bu yaşa ulaşamadığından eşzamanlı worker'ların
//! canlı dosyası kilitsiz korunur (ADR-061 eki).

use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Süreç ölümünden kalan (yetim) `.parca` bu yaştan eskiyse süpürülür.
pub const YETIM_PARCA_YASI: Duration = Duration::from_secs(15 * 60);
/// Çalışan worker'da süpürme en sık bu aralıkla tekrarlanır.
pub const YETIM_SUPURME_ARALIGI: Duration = Duration::from_secs(5 * 60);
/// Tek süpürmede incelenen en çok dizin girdisi; kalan sonraki tura kalır.
const YETIM_SUPURME_GIRDISI: usize = 10_000;

// Eşiğin canlı ömrün (yanıt yazımı dahil) en az dört katı olduğu `main.rs` içinde derleme zamanı
// denetlenir (HTTP sınırlarının sahibi orasıdır).

/// `AkanYukleme::yeni` adlandırması: 64 küçük onaltılık + `.parca`.
fn runtime_parca_adi(ad: &str) -> bool {
    ad.strip_suffix(".parca").is_some_and(|kimlik| {
        kimlik.len() == 64
            && kimlik
                .bytes()
                .all(|bayt| bayt.is_ascii_digit() || (b'a'..=b'f').contains(&bayt))
    })
}

/// `kok/.zee/yuklemeler` altında runtime adlandırmasına uyan, sembolik bağ
/// olmayan normal dosyalardan son değişikliği `yas`tan eski olanları siler ve
/// sayısını döndürür. Başka ad, alt dizin, bağ ve gelecek tarihli dosyaya
/// dokunulmaz; dizin yoksa ya da okunamazsa hiçbir şey yapılmaz.
pub fn yetim_parcalari_supur(kok: &Path, simdi: SystemTime, yas: Duration) -> usize {
    yetim_parcalari_supur_dizinde(kok, YUKLEME_DIZINI, simdi, yas)
}

/// Web yüklemelerinin geçici dizini (spec/26).
pub const YUKLEME_DIZINI: &str = ".zee/yuklemeler";
/// K-384: bağlantı bloklarının geçici dizini (RFC-0039 A2c). Ayrı dizin,
/// web süpürmesinin 15 dakikalık eşiğinin uzun bağlantı oturumundaki canlı
/// blok dosyasına dokunmasını önler. Bağlantı reaktörü yalnız Unix'tedir.
#[cfg(unix)]
pub const BAGLANTI_BLOK_DIZINI: &str = ".zee/baglanti-bloklari";

/// `kok/<alt>` altında aynı süpürme kuralı (`yetim_parcalari_supur`).
pub fn yetim_parcalari_supur_dizinde(
    kok: &Path,
    alt: &str,
    simdi: SystemTime,
    yas: Duration,
) -> usize {
    let Ok(girdiler) = std::fs::read_dir(kok.join(alt)) else {
        return 0;
    };
    let mut silinen = 0;
    for girdi in girdiler.take(YETIM_SUPURME_GIRDISI).filter_map(Result::ok) {
        if !girdi.file_name().to_str().is_some_and(runtime_parca_adi) {
            continue;
        }
        // `DirEntry::metadata` sembolik bağı izlemez.
        let Ok(bilgi) = girdi.metadata() else {
            continue;
        };
        if !bilgi.file_type().is_file() {
            continue;
        }
        let yasli = bilgi
            .modified()
            .ok()
            .and_then(|degisim| simdi.duration_since(degisim).ok())
            .is_some_and(|gecen| gecen >= yas);
        if !yasli {
            continue;
        }
        match std::fs::remove_file(girdi.path()) {
            Ok(()) => silinen += 1,
            // Başka worker aynı turda sildi.
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => {}
            Err(hata) => eprintln!(
                "yetim yükleme geçici dosyası silinemedi ({}): {}",
                girdi.path().display(),
                hata
            ),
        }
    }
    silinen
}

pub struct YuklemeBilgisi {
    pub goreli_yol: String,
    pub sha256: String,
    pub bayt: usize,
}

/// Bir isteğin geçici yükleme dosyasının sahibi: düşünce dosyayı siler.
/// Rota dosyayı aynı istekte taşıdıysa yol boştur ve hiçbir şey yapılmaz.
#[derive(Debug)]
pub struct IstekYuklemesi {
    yol: PathBuf,
}

impl IstekYuklemesi {
    #[cfg(test)]
    pub fn yol(&self) -> &Path {
        &self.yol
    }
}

impl Drop for IstekYuklemesi {
    fn drop(&mut self) {
        match std::fs::remove_file(&self.yol) {
            Ok(()) => {}
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => {}
            Err(hata) => eprintln!(
                "reddedilen/tüketilmeyen yükleme geçici dosyası silinemedi ({}): {}",
                self.yol.display(),
                hata
            ),
        }
    }
}

pub struct AkanYukleme {
    // Alan sırası bilinçlidir: dosya tanıtıcısı kapanmadan silme denenmez.
    dosya: File,
    koruma: IstekYuklemesi,
    goreli_yol: String,
    ozet: Sha256,
    bayt: usize,
}

impl AkanYukleme {
    pub fn yeni(kok: &Path) -> Result<Self, String> {
        Self::dizinde(kok, YUKLEME_DIZINI)
    }

    /// `kok/<alt>` altında `0600`, benzersiz `<64 hex>.parca`.
    pub fn dizinde(kok: &Path, alt: &str) -> Result<Self, String> {
        let dizin = kok.join(alt);
        std::fs::create_dir_all(&dizin)
            .map_err(|hata| format!("yükleme geçici dizini oluşturulamadı: {}", hata))?;
        for _ in 0..8 {
            let kimlik = dil::guvenlik::guvenli_belirtec_uret()?;
            let goreli_yol = format!("{alt}/{kimlik}.parca");
            let yol = kok.join(&goreli_yol);
            let mut secenekler = OpenOptions::new();
            secenekler.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                secenekler.mode(0o600);
            }
            match secenekler.open(&yol) {
                Ok(dosya) => {
                    return Ok(Self {
                        dosya,
                        koruma: IstekYuklemesi { yol },
                        goreli_yol,
                        ozet: Sha256::new(),
                        bayt: 0,
                    });
                }
                Err(hata) if hata.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(hata) => return Err(format!("yükleme geçici dosyası açılamadı: {}", hata)),
            }
        }
        Err("yükleme için benzersiz geçici dosya üretilemedi".into())
    }

    pub fn yaz(&mut self, parca: &[u8]) -> Result<(), String> {
        self.dosya
            .write_all(parca)
            .map_err(|hata| format!("binary yükleme yazılamadı: {}", hata))?;
        self.ozet.update(parca);
        self.bayt = self.bayt.saturating_add(parca.len());
        Ok(())
    }

    /// Dosyayı kalıcılaştırır; dönen koruma isteğin sonuna dek tutulmalıdır.
    /// Hata yolunda koruma düşer ve kısmi dosya silinir.
    pub fn tamamla(self) -> Result<(YuklemeBilgisi, IstekYuklemesi), String> {
        let Self {
            mut dosya,
            koruma,
            goreli_yol,
            ozet,
            bayt,
        } = self;
        dosya
            .flush()
            .and_then(|_| dosya.sync_all())
            .map_err(|hata| format!("binary yükleme kalıcılaştırılamadı: {}", hata))?;
        drop(dosya);
        let ozet = ozet.finalize();
        let sha256 = ozet.iter().map(|bayt| format!("{bayt:02x}")).collect();
        Ok((
            YuklemeBilgisi {
                goreli_yol,
                sha256,
                bayt,
            },
            koruma,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_olmayan_parcalari_akista_ozetler() {
        let kok = std::env::temp_dir().join(format!("zee-f030-yukleme-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&kok);
        std::fs::create_dir_all(&kok).unwrap();
        let mut yukleme = AkanYukleme::yeni(&kok).unwrap();
        yukleme.yaz(&[0, 255]).unwrap();
        yukleme.yaz(b"abc").unwrap();
        let (bilgi, koruma) = yukleme.tamamla().unwrap();
        assert_eq!(bilgi.bayt, 5);
        assert_eq!(
            bilgi.sha256,
            "24397706eb32f8691116fe4728d18eda7eacc40925e0ae26a5780cd8b8b13f80"
        );
        assert_eq!(
            std::fs::read(kok.join(&bilgi.goreli_yol)).unwrap(),
            [0, 255, 97, 98, 99]
        );
        drop(koruma);
        assert!(!kok.join(&bilgi.goreli_yol).exists());
        std::fs::remove_dir_all(kok).unwrap();
    }

    #[test]
    fn yetim_supurme_yalniz_yasli_runtime_parcasini_siler() {
        let kok = std::env::temp_dir().join(format!("zee-k332-yetim-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&kok);
        let dizin = kok.join(".zee/yuklemeler");
        // Dizin yoksa süpürme etkisizdir.
        assert_eq!(
            yetim_parcalari_supur(&kok, SystemTime::now(), YETIM_PARCA_YASI),
            0
        );
        std::fs::create_dir_all(dizin.join(format!("{}.parca", "c".repeat(64)))).unwrap();
        let simdi = SystemTime::now();
        let eski = simdi - YETIM_PARCA_YASI - Duration::from_secs(1);
        let yaz = |ad: &str, zaman: SystemTime| {
            let dosya = File::create(dizin.join(ad)).unwrap();
            dosya.set_modified(zaman).unwrap();
        };
        let yetim = format!("{}.parca", "a".repeat(64));
        let canli = format!("{}.parca", "b".repeat(64));
        let gelecek = format!("{}.parca", "d".repeat(64));
        yaz(&yetim, eski);
        yaz(&canli, simdi - YETIM_PARCA_YASI + Duration::from_secs(60));
        yaz(&gelecek, simdi + Duration::from_secs(3600));
        for yabanci in [
            "urun-eski.parca",
            &format!("{}.parca", "A".repeat(64)),
            &format!("{}.bin", "a".repeat(64)),
            &format!("{}.parca", "e".repeat(63)),
        ] {
            yaz(yabanci, eski);
        }
        #[cfg(unix)]
        {
            let hedef = kok.join("hedef.bin");
            File::create(&hedef).unwrap().set_modified(eski).unwrap();
            std::os::unix::fs::symlink(&hedef, dizin.join(format!("{}.parca", "f".repeat(64))))
                .unwrap();
        }

        assert_eq!(yetim_parcalari_supur(&kok, simdi, YETIM_PARCA_YASI), 1);
        assert!(!dizin.join(&yetim).exists(), "yaşlı yetim silinmeli");
        assert!(
            dizin.join(&canli).exists(),
            "eşik altındaki canlı dosya kalmalı"
        );
        assert!(
            dizin.join(&gelecek).exists(),
            "gelecek tarihli dosyaya dokunulmaz"
        );
        assert!(
            dizin.join("urun-eski.parca").exists(),
            "ürün adlandırması korunur"
        );
        assert!(dizin.join(format!("{}.bin", "a".repeat(64))).exists());
        assert!(dizin.join(format!("{}.parca", "c".repeat(64))).is_dir());
        #[cfg(unix)]
        {
            assert!(kok.join("hedef.bin").exists(), "bağ hedefi silinmez");
            assert!(dizin
                .join(format!("{}.parca", "f".repeat(64)))
                .symlink_metadata()
                .is_ok());
        }
        // İkinci tur (eşzamanlı başka worker) silecek bir şey bulmaz.
        assert_eq!(yetim_parcalari_supur(&kok, simdi, YETIM_PARCA_YASI), 0);
        std::fs::remove_dir_all(kok).unwrap();
    }

    #[test]
    fn canli_akan_yukleme_supurmeden_etkilenmez() {
        let kok = std::env::temp_dir().join(format!("zee-k332-canli-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&kok);
        std::fs::create_dir_all(&kok).unwrap();
        let mut yukleme = AkanYukleme::yeni(&kok).unwrap();
        yukleme.yaz(b"yarim").unwrap();
        assert_eq!(
            yetim_parcalari_supur(&kok, SystemTime::now(), YETIM_PARCA_YASI),
            0
        );
        yukleme.yaz(b"-devam").unwrap();
        let (bilgi, koruma) = yukleme.tamamla().unwrap();
        assert_eq!(
            std::fs::read(kok.join(&bilgi.goreli_yol)).unwrap(),
            b"yarim-devam"
        );
        // Aynı dosya eşiği aşmış bir saatle görülürse yetim sayılır.
        assert_eq!(
            yetim_parcalari_supur(
                &kok,
                SystemTime::now() + YETIM_PARCA_YASI + Duration::from_secs(1),
                YETIM_PARCA_YASI
            ),
            1
        );
        drop(koruma);
        std::fs::remove_dir_all(kok).unwrap();
    }

    #[test]
    fn yarida_kalan_akis_ve_tuketilmeyen_yukleme_dosya_birakmaz() {
        let kok = std::env::temp_dir().join(format!("zee-k332-yukleme-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&kok);
        std::fs::create_dir_all(&kok).unwrap();
        let dizin = kok.join(".zee/yuklemeler");
        let parca_sayisi = || std::fs::read_dir(&dizin).unwrap().count();

        // Akış ortasında bırakılan (red/okuma hatası) yükleme.
        let mut yukleme = AkanYukleme::yeni(&kok).unwrap();
        yukleme.yaz(&[7; 4096]).unwrap();
        assert_eq!(parca_sayisi(), 1);
        drop(yukleme);
        assert_eq!(parca_sayisi(), 0);

        // Tamamlanıp rotaca tüketilmeden biten istek.
        let mut yukleme = AkanYukleme::yeni(&kok).unwrap();
        yukleme.yaz(b"abc").unwrap();
        let (bilgi, koruma) = yukleme.tamamla().unwrap();
        assert_eq!(koruma.yol(), kok.join(&bilgi.goreli_yol));
        assert_eq!(parca_sayisi(), 1);
        drop(koruma);
        assert_eq!(parca_sayisi(), 0);

        // Rota dosyayı taşıdıysa koruma hedefe dokunmaz.
        let mut yukleme = AkanYukleme::yeni(&kok).unwrap();
        yukleme.yaz(b"kalici").unwrap();
        let (bilgi, koruma) = yukleme.tamamla().unwrap();
        let hedef = kok.join("hedef.bin");
        std::fs::rename(kok.join(&bilgi.goreli_yol), &hedef).unwrap();
        drop(koruma);
        assert_eq!(std::fs::read(&hedef).unwrap(), b"kalici");
        assert_eq!(parca_sayisi(), 0);
        std::fs::remove_dir_all(kok).unwrap();
    }
}
