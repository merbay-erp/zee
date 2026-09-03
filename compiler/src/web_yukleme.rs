//! Native HTTP binary gövdesini bellekte biriktirmeden geçici dosyaya alır.

use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

pub struct YuklemeBilgisi {
    pub goreli_yol: String,
    pub sha256: String,
    pub bayt: usize,
}

pub struct AkanYukleme {
    dosya: File,
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

    pub fn tamamla(mut self) -> Result<YuklemeBilgisi, String> {
        self.dosya
            .flush()
            .and_then(|_| self.dosya.sync_all())
            .map_err(|hata| format!("binary yükleme kalıcılaştırılamadı: {}", hata))?;
        let ozet = std::mem::take(&mut self.ozet).finalize();
        let sha256 = ozet.iter().map(|bayt| format!("{bayt:02x}")).collect();
        Ok(YuklemeBilgisi {
            goreli_yol: self.goreli_yol.clone(),
            sha256,
            bayt: self.bayt,
        })
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
        let bilgi = yukleme.tamamla().unwrap();
        assert_eq!(bilgi.bayt, 5);
        assert_eq!(
            bilgi.sha256,
            "24397706eb32f8691116fe4728d18eda7eacc40925e0ae26a5780cd8b8b13f80"
        );
        assert_eq!(
            std::fs::read(kok.join(&bilgi.goreli_yol)).unwrap(),
            [0, 255, 97, 98, 99]
        );
        std::fs::remove_dir_all(kok).unwrap();
    }
}
