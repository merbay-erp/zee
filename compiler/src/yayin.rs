//! Proje çözümünü davranışsız paket modeli ve artefakt üretimine bağlar.

use crate::paket::ProjeGrafigi;
pub use crate::paket_modeli::PaketCiktilari;
use crate::proje::bildirimi_oku;
use std::path::Path;

pub fn paketle(kok: &Path, anahtar_yolu: &Path, cikti: &Path) -> Result<PaketCiktilari, String> {
    let saniye = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(deger) => deger.parse::<i64>().map_err(|_| {
            "SOURCE_DATE_EPOCH negatif olmayan bir Unix saniyesi olmalı.".to_string()
        })?,
        Err(std::env::VarError::NotPresent) => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Sistem saati Unix başlangıcından önce.".to_string())?
            .as_secs()
            .try_into()
            .map_err(|_| "Sistem saati desteklenen Unix saniyesini aşıyor.".to_string())?,
        Err(hata) => return Err(format!("SOURCE_DATE_EPOCH okunamadı: {}.", hata)),
    };
    paketle_zamanla(kok, anahtar_yolu, cikti, saniye)
}

pub fn paketle_zamanla(
    kok: &Path,
    anahtar_yolu: &Path,
    cikti: &Path,
    uretim_saniyesi: i64,
) -> Result<PaketCiktilari, String> {
    if uretim_saniyesi < 0 {
        return Err("Üretim zamanı negatif olamaz.".into());
    }
    let kok =
        std::fs::canonicalize(kok).map_err(|hata| format!("Paket kökü çözülemedi: {}.", hata))?;
    if !kok.is_dir() {
        return Err(format!("\"{}\" bir proje klasörü değil.", kok.display()));
    }
    let bildirim_kaynagi = std::fs::read_to_string(kok.join("proje.dil"))
        .map_err(|hata| format!("proje.dil okunamadı: {}.", hata))?;
    let bildirim =
        bildirimi_oku(&bildirim_kaynagi).map_err(|tani| format!("{}: {}", tani.kod, tani.mesaj))?;
    if !bildirim.yerel_bagimliliklar.is_empty() || !bildirim.uzak_bagimliliklar.is_empty() {
        return Err("Yerel yol bağımlılığı veya uzak registry bağımlılığı henüz yayınlanamaz; kaynak paketi v1 bağımsız olmalı.".into());
    }
    let grafik = ProjeGrafigi::cozumle(&kok)
        .map_err(|hata| format!("{}: {}", hata.tani.kod, hata.tani.mesaj))?;
    grafik
        .kilidi_denetle()
        .map_err(|hata| format!("{}: {}", hata.tani.kod, hata.tani.mesaj))?;
    let giris = grafik.ana_giris()?;
    let mut yukleyici = |istek: crate::BirimIstegi<'_>| grafik.yukle(istek);
    crate::kaynagi_derle_kokenlerle(&giris.kaynak, Some(&giris.koken), &mut yukleyici)
        .map_err(|tani| format!("Paket kaynağı {}: {}", tani.kod, tani.mesaj))?;
    crate::artefakt_dogrulama::paketle_dogrulanmis(
        &kok,
        &bildirim,
        anahtar_yolu,
        cikti,
        uretim_saniyesi,
    )
}
