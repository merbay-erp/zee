//! K-395 (RFC-0043 Dilim 2, ADR-171): gerçek dosyada akışlı base64 çözme ve
//! bayt arama. Kaynak 64 KiB'lık parçalarla okunur; çözüm ve arama saf
//! `dil::kodlama` durum makineleriyle yapılır, hedef `kalici_dosya` akışlı
//! eklemesiyle atomik yayımlanır. Hiçbir aşamada `Metin` ya da bütün içerik
//! bellekte tutulmaz; hata mesajı içerik taşımaz.

use dil::kalici_dosya::{akisla_sona_ekle, AkisliEkleme, AkisliEklemeHatasi};
use dil::kodlama::{
    AkisHatasi, Base64AkisCozucu, BaytArayici, AZAMI_HEDEF_BAYTI, AZAMI_TARAMA_BAYTI,
};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const PARCA_BAYTI: usize = 64 * 1024;

fn okuma_hatasi(yol: &str, hata: std::io::Error) -> String {
    format!("\"{}\" dosyası okunamadı: {}", yol, hata)
}

fn kaynak_ac(gercek: &Path, yol: &str, konum: u64) -> Result<File, String> {
    let mut dosya =
        File::open(gercek).map_err(|hata| format!("\"{}\" dosyası açılamadı: {}", yol, hata))?;
    if !dosya
        .metadata()
        .map_err(|hata| okuma_hatasi(yol, hata))?
        .is_file()
    {
        return Err(format!("\"{}\" normal dosya değil", yol));
    }
    dosya
        .seek(SeekFrom::Start(konum))
        .map_err(|hata| okuma_hatasi(yol, hata))?;
    Ok(dosya)
}

/// Kaynağın `[konum, konum + adet)` aralığını (dosya sonunda olan kadar)
/// çözüp hedefe ekler. Bozuk base64 kaynak dosyadaki mutlak konumla C032;
/// hedef ancak bütün aralık geçerliyse ve 32 MiB tavanı aşılmazsa değişir.
pub(crate) fn base64_cozerek_ekle(
    kaynak_yolu: &Path,
    kaynak: &str,
    (konum, adet): (u64, u64),
    hedef_yolu: &Path,
    hedef: &str,
) -> Result<AkisliEkleme, AkisHatasi> {
    let ayni = match (
        std::fs::canonicalize(kaynak_yolu),
        std::fs::canonicalize(hedef_yolu),
    ) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    if ayni {
        return Err(AkisHatasi::Dosya(format!(
            "\"{}\" hem kaynak hem hedef olamaz",
            hedef
        )));
    }
    let mut sinirli = kaynak_ac(kaynak_yolu, kaynak, konum)
        .map_err(AkisHatasi::Dosya)?
        .take(adet);
    let yazim_hatasi = |hata: std::io::Error| {
        AkisHatasi::Dosya(format!("\"{}\" dosyasına yazılamadı: {}", hedef, hata))
    };
    akisla_sona_ekle(hedef_yolu, AZAMI_HEDEF_BAYTI, |yaz| {
        let mut cozucu = Base64AkisCozucu::default();
        let mut tampon = vec![0u8; PARCA_BAYTI];
        let mut cikti = Vec::with_capacity(PARCA_BAYTI);
        let mut taban = konum;
        loop {
            let okunan = sinirli
                .read(&mut tampon)
                .map_err(|hata| AkisHatasi::Dosya(okuma_hatasi(kaynak, hata)))?;
            if okunan == 0 {
                break;
            }
            cikti.clear();
            cozucu
                .besle(&tampon[..okunan], taban, &mut cikti)
                .map_err(AkisHatasi::Kodlama)?;
            taban += okunan as u64;
            yaz(&cikti).map_err(yazim_hatasi)?;
        }
        cozucu.bitir(taban).map_err(AkisHatasi::Kodlama)
    })
    .map_err(|hata| match hata {
        AkisliEklemeHatasi::Hedef(hata) => yazim_hatasi(hata),
        AkisliEklemeHatasi::Uretici(hata) => hata,
    })
}

/// `konum`dan itibaren en çok 64 MiB tarar; ilk eşleşmenin mutlak konumu ya
/// da -1. Pencere dolduğunda dosya hâlâ sürüyorsa belirsiz -1 yerine hata.
pub(crate) fn bayt_ara(gercek: &Path, yol: &str, konum: u64, igne: &[u8]) -> Result<i64, String> {
    let mut dosya = kaynak_ac(gercek, yol, konum)?;
    let mut arayici = BaytArayici::yeni(igne);
    let mut tampon = vec![0u8; PARCA_BAYTI];
    let mut taranan = 0u64;
    loop {
        let istenen = (AZAMI_TARAMA_BAYTI - taranan).min(PARCA_BAYTI as u64) as usize;
        if istenen == 0 {
            let mut fazla = [0u8; 1];
            return match dosya.read(&mut fazla).map_err(|h| okuma_hatasi(yol, h))? {
                0 => Ok(-1),
                _ => Err(format!(
                    "\"{}\" dosyasında {} MiB tarama sınırında eşleşme yok; konumu ilerletip yeniden arayın",
                    yol,
                    AZAMI_TARAMA_BAYTI / 1024 / 1024
                )),
            };
        }
        let okunan = dosya
            .read(&mut tampon[..istenen])
            .map_err(|h| okuma_hatasi(yol, h))?;
        if okunan == 0 {
            return Ok(-1);
        }
        if let Some(yer) = arayici.besle(&tampon[..okunan]) {
            return Ok((konum + yer) as i64);
        }
        taranan += okunan as u64;
    }
}
