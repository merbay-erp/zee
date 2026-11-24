//! K-389 (RFC-0043 Dilim 1, ADR-167): çözülmüş bayt içeriğin dosyaya yazımı.
//!
//! Yeni dosya yazımı geçici dosya + `fsync` + atomik adlandırma ile ve var
//! olan hedefi ezmeden yapılır. Ekleme spec/08 eklemesiyle aynı atomik
//! oku-birleştir-değiştir modelidir: boyut kilit altında önce denetlenir,
//! sınırı aşan parça hiç yazılmaz.

use super::{
    atomik_degistir, atomik_icerik_yaz, ebeveyn, gecici_dosya_ac, klasoru_eszamanla, metadata,
    mevcut_icerik, DosyaKilidi,
};
use std::io::{self, Read, Write};
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

/// K-395: akışlı eklemenin sonucu — hedefin yayımlanan bütün içeriğinin
/// SHA-256'sı (küçük harf onaltılık), toplam boyu ve bu çağrıda eklenen bayt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AkisliEkleme {
    pub sha256: String,
    pub toplam: u64,
    pub eklenen: u64,
}

/// K-395: hedef (kilit, geçici dosya, tavan, yayımlama) ya da üretici hatası.
#[derive(Debug)]
pub enum AkisliEklemeHatasi<E> {
    Hedef(io::Error),
    Uretici(E),
}

fn tavan_asimi(azami: u64) -> io::Error {
    io::Error::new(
        io::ErrorKind::FileTooLarge,
        format!(
            "hedefin toplam boyutu {} MiB sınırını aşıyor",
            azami / 1024 / 1024
        ),
    )
}

/// K-395 (ADR-171): üreticinin sınırlı parçalarla akıttığı baytları sona
/// ekler; hiçbir aşamada bütün içerik bellekte tutulmaz. Kilit altında var
/// olan içerik geçici dosyaya akışla kopyalanır, üretilen baytlar arkasına
/// yazılır, `fsync` + atomik adlandırma ile tek adımda yayımlanır. Üretici
/// hatası, tavan aşımı ya da IO hatasında hedef hiç değişmez (geçici dosya
/// silinir). Hedef normal dosya değilse ya da sembolik bağsa reddedilir.
pub fn akisla_sona_ekle<E>(
    yol: &Path,
    azami: u64,
    uret: impl FnOnce(&mut dyn FnMut(&[u8]) -> io::Result<()>) -> Result<(), E>,
) -> Result<AkisliEkleme, AkisliEklemeHatasi<E>> {
    use sha2::{Digest, Sha256};
    let hedef = AkisliEklemeHatasi::Hedef;
    let _kilit = DosyaKilidi::al(yol).map_err(hedef)?;
    let eski_metadata = metadata::MetadataKaynagi::yakala(yol).map_err(hedef)?;
    let (mut dosya, mut gecici) = gecici_dosya_ac(yol).map_err(hedef)?;
    let gecici_yol = gecici.yol.clone().ok_or_else(|| {
        hedef(io::Error::new(
            io::ErrorKind::NotFound,
            "zee geçici dosya yolu kayboldu",
        ))
    })?;
    let mut ozet = Sha256::new();
    let mut toplam = 0u64;
    if let Some(mut eski) = baglarsiz_ac(yol).map_err(hedef)? {
        let mut tampon = vec![0u8; 64 * 1024];
        loop {
            let okunan = eski.read(&mut tampon).map_err(hedef)?;
            if okunan == 0 {
                break;
            }
            toplam += okunan as u64;
            if toplam > azami {
                return Err(hedef(tavan_asimi(azami)));
            }
            ozet.update(&tampon[..okunan]);
            dosya.write_all(&tampon[..okunan]).map_err(hedef)?;
        }
    }
    let onceki = toplam;
    let mut lavabo = |parca: &[u8]| -> io::Result<()> {
        toplam = toplam
            .checked_add(parca.len() as u64)
            .filter(|yeni| *yeni <= azami)
            .ok_or_else(|| tavan_asimi(azami))?;
        ozet.update(parca);
        dosya.write_all(parca)
    };
    uret(&mut lavabo).map_err(AkisliEklemeHatasi::Uretici)?;
    eski_metadata.geciciye_uygula(&dosya).map_err(hedef)?;
    dosya.sync_all().map_err(hedef)?;
    drop(dosya);
    atomik_degistir(&gecici_yol, yol).map_err(hedef)?;
    gecici.yol = None;
    klasoru_eszamanla(ebeveyn(yol)).map_err(hedef)?;
    Ok(AkisliEkleme {
        sha256: ozet
            .finalize()
            .iter()
            .map(|bayt| format!("{bayt:02x}"))
            .collect(),
        toplam,
        eklenen: toplam - onceki,
    })
}

/// Var olan hedefi sembolik bağ izlemeden okumak için açar; yoksa `None`.
fn baglarsiz_ac(yol: &Path) -> io::Result<Option<std::fs::File>> {
    let mut secenek = std::fs::OpenOptions::new();
    secenek.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        secenek.custom_flags(libc::O_NOFOLLOW);
    }
    let dosya = match secenek.open(yol) {
        Ok(dosya) => dosya,
        Err(hata) if hata.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(hata) => return Err(hata),
    };
    if !dosya.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "hedef normal dosya değil",
        ));
    }
    Ok(Some(dosya))
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

    #[test]
    fn akisli_ekleme_tavanda_ve_uretici_hatasinda_hedefe_dokunmaz() {
        let klasor = std::env::temp_dir().join(format!(
            "zee-k395-akisli-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&klasor).unwrap();
        let yol = klasor.join("ek.bin");
        let sonuc = akisla_sona_ekle::<()>(&yol, 6, |yaz| {
            yaz(&[1, 2]).unwrap();
            yaz(&[3]).unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!((sonuc.toplam, sonuc.eklenen), (3, 3));
        assert_eq!(sonuc.sha256, crate::guvenlik::sha256_hex(&[1, 2, 3]));
        let sonuc = akisla_sona_ekle::<()>(&yol, 6, |yaz| {
            yaz(&[4, 5, 6]).unwrap();
            Ok(())
        })
        .unwrap();
        assert_eq!((sonuc.toplam, sonuc.eklenen), (6, 3));
        // Tavan: üretici sınırı aşınca lavabo reddeder, hedef aynı kalır.
        let hata = akisla_sona_ekle(&yol, 6, |yaz| yaz(&[7])).unwrap_err();
        assert!(
            matches!(hata, AkisliEklemeHatasi::Uretici(ref h) if h.kind() == io::ErrorKind::FileTooLarge)
        );
        // Üretici hatası: yarım yazılan bayt yayımlanmaz.
        let hata = akisla_sona_ekle(&klasor.join("yeni.bin"), 64, |yaz| {
            yaz(b"yarim").unwrap();
            Err("bozuk")
        })
        .unwrap_err();
        assert!(matches!(hata, AkisliEklemeHatasi::Uretici("bozuk")));
        assert!(!klasor.join("yeni.bin").exists());
        assert_eq!(std::fs::read(&yol).unwrap(), [1, 2, 3, 4, 5, 6]);
        let kalan: Vec<_> = std::fs::read_dir(&klasor)
            .unwrap()
            .map(|g| g.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|ad| !ad.starts_with(".zee-yazma-kilidi"))
            .collect();
        assert_eq!(kalan, ["ek.bin"], "geçici dosya kalmaz");
        std::fs::remove_dir_all(&klasor).unwrap();
    }
}
