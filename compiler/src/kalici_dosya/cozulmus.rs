//! K-389 (RFC-0043 Dilim 1, ADR-167): çözülmüş bayt içeriğin dosyaya yazımı.
//!
//! Yeni dosya yazımı geçici dosya + `fsync` + atomik adlandırma ile ve var
//! olan hedefi ezmeden yapılır. Ekleme spec/08 eklemesiyle aynı atomik
//! oku-birleştir-değiştir modelidir: boyut kilit altında önce denetlenir,
//! sınırı aşan parça hiç yazılmaz.

use super::{atomik_degistir, atomik_icerik_yaz, nokta, sar, Adim, DosyaKilidi};
use std::fs::File;
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
    let mut yeni = denetlenmis_onek(yol, icerik.len() as u64, azami)?;
    yeni.extend_from_slice(icerik);
    atomik_icerik_yaz(yol, &yeni, atomik_degistir)?;
    Ok(yeni)
}

/// K-408 (ADR-172 eki, ZM-F049): `kaynak`ın (tam `kaynak_boyu` bayt) RFC 2045
/// satırlı base64 kodlamasını hedefin sonuna ekler ve yazılan bayt sayısını
/// (T = e + 2·⌈e/76⌉) döndürür. Kaynak 57·256 baytlık parçalarla akışla
/// kodlanır, kodlanmış çıktı bellekte bütün olarak tutulmaz. Atomiklik
/// [`sinirli_sona_ekle`] ile aynıdır: toplam `azami`yı aşacaksa ya da kaynak
/// okuma sırasında boyut değiştirirse hiçbir bayt görünür olmaz.
pub fn base64_satirlari_sona_ekle(
    kaynak: &mut dyn Read,
    kaynak_boyu: u64,
    yol: &Path,
    azami: u64,
) -> io::Result<u64> {
    let eklenecek = crate::kodlama::base64_satirli_uzunluk(kaynak_boyu);
    let _kilit = DosyaKilidi::al(yol)?;
    let onek = denetlenmis_onek(yol, eklenecek, azami)?;
    atomik_akisla_yaz(
        yol,
        |dosya| {
            dosya.write_all(&onek)?;
            satirlari_akit(kaynak, kaynak_boyu, dosya)
        },
        atomik_degistir,
    )?;
    Ok(eklenecek)
}

/// Kaynağın tam `boy` baytını satır başına hizalı parçalarla kodlayıp yazar.
fn satirlari_akit(kaynak: &mut dyn Read, boy: u64, hedef: &mut dyn Write) -> io::Result<()> {
    const PARCA: usize = crate::kodlama::BASE64_SATIR_HAM_BAYTI as usize * 256;
    let degisti = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "kaynak dosya okuma sırasında değişti",
        )
    };
    let mut ham = vec![0u8; PARCA];
    let mut cikti = Vec::with_capacity(PARCA / 57 * 78);
    let mut kalan = boy;
    while kalan > 0 {
        let istenen = PARCA.min(kalan as usize);
        let mut dolu = 0;
        while dolu < istenen {
            match kaynak.read(&mut ham[dolu..istenen]) {
                Ok(0) => return Err(degisti()),
                Ok(n) => dolu += n,
                Err(hata) if hata.kind() == io::ErrorKind::Interrupted => {}
                Err(hata) => return Err(hata),
            }
        }
        cikti.clear();
        crate::kodlama::base64_satirlari_kodla(&ham[..dolu], &mut cikti);
        hedef.write_all(&cikti)?;
        kalan -= dolu as u64;
    }
    // Kaynak bildirilen boydan uzunsa (eş yazar) kodlama dosyayı temsil etmez.
    match kaynak.read(&mut ham[..1]) {
        Ok(0) => Ok(()),
        Ok(_) => Err(degisti()),
        Err(hata) => Err(hata),
    }
}

/// Kilit altında: hedef normal dosya (ya da yok) olmalı, mevcut boy + `ek`
/// `azami`yı aşmamalı; mevcut içerik bağ izlenmeden okunur.
fn denetlenmis_onek(yol: &Path, ek: u64, azami: u64) -> io::Result<Vec<u8>> {
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
        .checked_add(ek)
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
    // K-395: var olan hedef genel 16 MiB okuma sınırıyla değil, kilit altında
    // denetlenmiş ekleme tavanıyla okunur (32 MiB hedefe ekleme). K-394: okuma
    // yaprak bağını izlemez; eş yazar `symlink_metadata` sonrası yaprağı bağa
    // çevirip kök dışı içeriği hedefe kopyalatamaz (spec/08).
    if onceki_boy == 0 {
        return Ok(Vec::new());
    }
    nokta(Adim::MevcutOkuma)
        .and_then(|()| {
            crate::kaynak_sinirlari::bagsiz_okuma::kalici_ic_baytlari_oku_azami(yol, azami as usize)
        })
        .map_err(sar(Adim::MevcutOkuma))
}

/// K-408: geçici dosyaya içeriği `yaz` üretir (akışlı yazım); eşzamanlama,
/// metadata ve tek atomik replace [`atomik_icerik_yaz`] ile aynıdır.
pub(super) fn atomik_akisla_yaz<Y, F>(hedef: &Path, yaz: Y, degistir: F) -> io::Result<()>
where
    Y: FnOnce(&mut File) -> io::Result<()>,
    F: FnOnce(&Path, &Path) -> io::Result<()>,
{
    // K-351: her adımın hatası adını taşır; tür (`kind`) korunur.
    let eski_metadata = nokta(Adim::Metadata)
        .and_then(|()| super::metadata::MetadataKaynagi::yakala(hedef))
        .map_err(sar(Adim::Metadata))?;
    let (dosya, mut gecici) = nokta(Adim::GeciciYazim)
        .and_then(|()| super::gecici_dosya_ac(hedef))
        .map_err(sar(Adim::GeciciYazim))?;
    let gecici_yol = gecici
        .yol
        .clone()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "zee geçici dosya yolu kayboldu"))
        .map_err(sar(Adim::GeciciYazim))?;
    super::gecici_icerigi_hazirla(dosya, yaz, &eski_metadata)?;
    degistir(&gecici_yol, hedef)?;
    gecici.yol = None;
    nokta(Adim::KlasorEszamanlama)
        .and_then(|()| super::klasoru_eszamanla(super::ebeveyn(hedef)))
        .map_err(sar(Adim::KlasorEszamanlama))
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
    fn base64_satirlari_onegi_korur_boy_degisirse_hic_yazmaz() {
        let klasor = std::env::temp_dir().join(format!(
            "zee-k408-satirli-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&klasor).unwrap();
        let yol = klasor.join("govde.eml");
        std::fs::write(&yol, b"H\r\n").unwrap();
        let ham: Vec<u8> = (0..57 * 300 + 1).map(|i| (i * 7) as u8).collect();
        let yazilan =
            base64_satirlari_sona_ekle(&mut ham.as_slice(), ham.len() as u64, &yol, 1 << 20)
                .unwrap();
        let mut beklenen = b"H\r\n".to_vec();
        crate::kodlama::base64_satirlari_kodla(&ham, &mut beklenen);
        assert_eq!(
            yazilan,
            crate::kodlama::base64_satirli_uzunluk(ham.len() as u64)
        );
        assert_eq!(std::fs::read(&yol).unwrap(), beklenen);
        // Kaynak bildirilenden kısa ya da uzun: yayımlanmaz, hedef aynı kalır.
        for boy in [ham.len() as u64 + 1, ham.len() as u64 - 1] {
            let hata =
                base64_satirlari_sona_ekle(&mut ham.as_slice(), boy, &yol, 1 << 20).unwrap_err();
            assert_eq!(hata.kind(), io::ErrorKind::InvalidData);
            assert_eq!(std::fs::read(&yol).unwrap(), beklenen);
        }
        let hata = base64_satirlari_sona_ekle(&mut &b"abc"[..], 3, &yol, beklenen.len() as u64 + 5)
            .unwrap_err();
        assert_eq!(hata.kind(), io::ErrorKind::FileTooLarge);
        assert_eq!(std::fs::read(&yol).unwrap(), beklenen);
        let artik: Vec<_> = std::fs::read_dir(&klasor)
            .unwrap()
            .filter_map(|g| g.ok())
            .filter(|g| g.file_name() != "govde.eml" && g.file_name() != ".zee-yazma-kilidi")
            .collect();
        assert!(artik.is_empty(), "geçici dosya kalmaz: {artik:?}");
        std::fs::remove_dir_all(&klasor).unwrap();
    }
}
