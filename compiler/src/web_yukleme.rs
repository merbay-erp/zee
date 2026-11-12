//! Native HTTP binary gövdesini bellekte biriktirmeden geçici dosyaya alır.
//!
//! K-332: geçici `.parca` dosyası isteğe aittir. Akış yarıda kalırsa
//! `AkanYukleme` düşerken, tamamlanmış yükleme ise istek bittiğinde
//! `IstekYuklemesi` düşerken dosyayı siler; rota dosyayı taşıdıysa yolda
//! dosya yoktur ve silme etkisizdir.

use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

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
        let dizin = kok.join(".zee/yuklemeler");
        std::fs::create_dir_all(&dizin)
            .map_err(|hata| format!("yükleme geçici dizini oluşturulamadı: {}", hata))?;
        for _ in 0..8 {
            let kimlik = dil::guvenlik::guvenli_belirtec_uret()?;
            let goreli_yol = format!(".zee/yuklemeler/{}.parca", kimlik);
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
