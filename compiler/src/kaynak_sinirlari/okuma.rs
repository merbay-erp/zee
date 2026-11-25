//! Kaynak ve çalışma veri dosyaları için ortak bounded reader.

mod bagsiz_okuma;
use super::VARSAYILAN_KAYNAK_SINIRLARI;
use crate::tani::Tani;
pub(crate) use bagsiz_okuma::kalici_ic_baytlari_oku;
use std::io::{self, Read};
use std::path::Path;

pub fn kaynak_dosyasi_oku(yol: &Path) -> io::Result<String> {
    sinirli_metin_oku(yol, VARSAYILAN_KAYNAK_SINIRLARI.kaynak_bayti())
}

pub fn veri_dosyasi_oku(yol: &Path) -> io::Result<String> {
    sinirli_metin_oku(yol, VARSAYILAN_KAYNAK_SINIRLARI.dosya_okuma_bayti())
}

pub fn veri_dosyasi_baytlarini_oku(yol: &Path) -> io::Result<Vec<u8>> {
    sinirli_bayt_oku(yol, VARSAYILAN_KAYNAK_SINIRLARI.dosya_okuma_bayti())
}

pub(crate) fn kaynak_boyutunu_denetle(kaynak: &str) -> Result<(), Tani> {
    let azami = VARSAYILAN_KAYNAK_SINIRLARI.kaynak_bayti();
    if kaynak.len() <= azami {
        return Ok(());
    }
    Err(Tani::yeni(
        "S045",
        format!(
            "Kaynak {} bayt; güvenli profil {} MiB sınırını aşıyor.",
            kaynak.len(),
            azami / 1024 / 1024
        ),
        1,
        1,
        1,
    )
    .onerili("Kaynağı sorumluluğu açık daha küçük birimlere böl.".into()))
}

pub(crate) fn token_sayisini_denetle(sayi: usize, satir: usize) -> Result<(), Tani> {
    let azami = VARSAYILAN_KAYNAK_SINIRLARI.token_sayisi();
    if sayi <= azami {
        return Ok(());
    }
    Err(Tani::yeni(
        "S045",
        format!(
            "Kaynak {} token; güvenli profil {} token sınırını aşıyor.",
            sayi, azami
        ),
        satir,
        1,
        1,
    )
    .onerili("Üretilmiş ya da çok büyük kaynağı daha küçük birimlere böl.".into()))
}

fn sinirli_metin_oku(yol: &Path, azami: usize) -> io::Result<String> {
    let baytlar = sinirli_bayt_oku(yol, azami)?;
    String::from_utf8(baytlar).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("`{}` geçerli UTF-8 metin değil", yol.display()),
        )
    })
}

/// Windows'ta `ReplaceFileW` penceresinde okuyucu yeniden dener; spec/08
/// "eski ya da yeni bütün" sözü (K-182, K-342 `degistirme_penceresi`).
fn dosyayi_ac(yol: &Path) -> io::Result<std::fs::File> {
    #[cfg(windows)]
    {
        let uzun = std::time::Duration::from_millis(
            VARSAYILAN_KAYNAK_SINIRLARI
                .kalici_dosya()
                .kilit_bekleme_ms(),
        );
        match super::degistirme_penceresi::pencereyi_bekleyerek(yol, uzun, || {
            std::fs::File::open(yol)
        }) {
            // K-348: ölen yazarın yarım değiştirmesi; eski bütün yedektedir.
            Err(hata) if hata.kind() == io::ErrorKind::NotFound => {
                super::degistirme_penceresi::yarim_degistirme::yedekten_ac(yol, uzun).ok_or(hata)
            }
            sonuc => sonuc,
        }
    }
    #[cfg(not(windows))]
    std::fs::File::open(yol)
}

/// K-395: çağıranın kendi tavanıyla sınırlı bayt okuma (çözerek ekleme hedefi).
pub(crate) fn sinirli_bayt_oku(yol: &Path, azami: usize) -> io::Result<Vec<u8>> {
    let dosya = dosyayi_ac(yol)?;
    let bildirilen = dosya.metadata()?.len();
    if bildirilen > azami as u64 {
        return Err(sinir_hatasi(yol, azami));
    }
    let mut baytlar = Vec::with_capacity(usize::try_from(bildirilen).unwrap_or(azami).min(azami));
    dosya
        .take(azami.saturating_add(1) as u64)
        .read_to_end(&mut baytlar)?;
    if baytlar.len() > azami {
        return Err(sinir_hatasi(yol, azami));
    }
    Ok(baytlar)
}

fn sinir_hatasi(yol: &Path, azami: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "`{}` {} MiB okuma sınırını aşıyor",
            yol.display(),
            azami / 1024 / 1024
        ),
    )
}
