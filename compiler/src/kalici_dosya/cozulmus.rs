//! K-389 (RFC-0043 Dilim 1, ADR-167): çözülmüş bayt içeriğin dosyaya yazımı.
//!
//! Yeni dosya yazımı geçici dosya + `fsync` + atomik adlandırma ile ve var
//! olan hedefi ezmeden yapılır. Ekleme spec/08 eklemesiyle aynı atomik
//! oku-birleştir-değiştir modelidir: boyut kilit altında önce denetlenir,
//! sınırı aşan parça hiç yazılmaz.

use super::{atomik_degistir, atomik_icerik_yaz, mevcut_icerik, DosyaKilidi};
use std::io;
use std::path::Path;

/// Var olan hedefi (dosya, dizin ya da sembolik bağ) ezmeden yeni dosya yazar.
pub fn yeni_dosyaya_yaz(yol: &Path, icerik: &[u8]) -> io::Result<()> {
    let _kilit = DosyaKilidi::al(yol)?;
    if yol.symlink_metadata().is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "hedef dosya zaten var",
        ));
    }
    atomik_icerik_yaz(yol, icerik, atomik_degistir)
}

/// Kilit altında sona ekler ve hedefin yayımlanan bütün yeni içeriğini
/// döndürür. spec/08 eklemesiyle aynı model: mevcut içerik + yeni baytlar tek
/// atomik replace ile görünür (okuyucu eski ya da yeni bütünü görür). Hedef
/// yoksa oluşturulur; normal dosya değilse ya da toplam `azami`yı aşacaksa
/// hiçbir bayt yazılmadan reddedilir.
pub fn sinirli_sona_ekle(yol: &Path, icerik: &[u8], azami: u64) -> io::Result<Vec<u8>> {
    let _kilit = DosyaKilidi::al(yol)?;
    let onceki_boy = match yol.symlink_metadata() {
        Ok(bilgi) if bilgi.is_file() => bilgi.len(),
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "hedef normal dosya değil",
            ))
        }
        Err(hata) if hata.kind() == io::ErrorKind::NotFound => 0,
        Err(hata) => return Err(hata),
    };
    onceki_boy
        .checked_add(icerik.len() as u64)
        .filter(|toplam| *toplam <= azami)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::FileTooLarge,
                format!(
                    "hedefin toplam boyutu {} MiB sınırını aşıyor",
                    azami / 1024 / 1024
                ),
            )
        })?;
    let mut yeni = mevcut_icerik(yol)?.unwrap_or_default();
    yeni.extend_from_slice(icerik);
    atomik_icerik_yaz(yol, &yeni, atomik_degistir)?;
    Ok(yeni)
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn yeni_dosya_ezilmez_ve_ekleme_siniri_yazmadan_reddeder() {
        let klasor = std::env::temp_dir().join(format!(
            "zee-k389-cozulmus-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&klasor).unwrap();
        let yol = klasor.join("ek.bin");
        yeni_dosyaya_yaz(&yol, &[0, 159, 255]).unwrap();
        let hata = yeni_dosyaya_yaz(&yol, b"yeni").unwrap_err();
        assert_eq!(hata.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&yol).unwrap(), [0, 159, 255]);

        assert_eq!(
            sinirli_sona_ekle(&yol, &[1, 2], 5).unwrap(),
            [0, 159, 255, 1, 2]
        );
        let hata = sinirli_sona_ekle(&yol, &[3], 5).unwrap_err();
        assert_eq!(hata.kind(), io::ErrorKind::FileTooLarge);
        assert_eq!(std::fs::read(&yol).unwrap(), [0, 159, 255, 1, 2]);

        let akis = klasor.join("akis.parca");
        assert!(sinirli_sona_ekle(&akis, &[9; 6], 5).is_err());
        assert!(!akis.exists(), "sınır aşımında boş dosya bile kalmaz");
        assert_eq!(
            sinirli_sona_ekle(&klasor, b"x", 5).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        std::fs::remove_dir_all(&klasor).unwrap();
    }
}
