//! K-348: yarım kalmış Windows değiştirmesinin izi (spec/08).
//!
//! `ReplaceFileW` hedefi iki adımda değiştirir: eski hedef
//! `.<ad>.zee-gecici-<pid>-<n>.zee-yedek` adına, ardından eksiksiz yazılmış
//! geçici dosya hedefe taşınır. Yazar iki adım arasında ölürse (çökme,
//! TerminateProcess, güç kaybı) hedef yolda dosya kalmaz; eski bütün içerik
//! yedekte, yayımlanmamış yeni içerik geçici dosyada durur. Windows CI'da
//! öldürülen yazarların %4'ü bu durumu bıraktı. Yazar başarı bildirmediği için
//! geçerli bütün eski içeriktir: yazma kilidini alan yazar yedeği geri koyar,
//! kilitsiz okuyucu yedeği paylaşımlı kilit altında okur.

// Üretimde yalnız Windows kullanır; testler her platformda koşar.
#![cfg_attr(not(windows), allow(dead_code))]

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

/// Atomik yazarın geçici dosya eki (`.<ad>` + ek + `<pid>-<n>`).
pub(crate) const GECICI_EKI: &str = ".zee-gecici-";
/// Windows `ReplaceFileW` yedeğinin geçici ada eklenen soneki.
pub(crate) const YEDEK_EKI: &str = ".zee-yedek";

/// Hedef için `.<ad>` + [`GECICI_EKI`] + sıra adını üretir.
pub(crate) fn gecici_ad(hedef: &Path, sira: &str) -> io::Result<OsString> {
    let ad = hedef.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "hedef bir dosya adı taşımalı")
    })?;
    let mut gecici = OsString::from(".");
    gecici.push(ad);
    gecici.push(GECICI_EKI);
    gecici.push(sira);
    Ok(gecici)
}

/// Hedefe ait bütün yarım değiştirme yedekleri, en yeni değişiklik zamanı
/// önce. Windows adları harf duyarsız karşılaştırılır.
pub(crate) fn yedekler(hedef: &Path) -> io::Result<Vec<PathBuf>> {
    let Some(ad) = hedef.file_name() else {
        return Ok(Vec::new());
    };
    let onek = format!(".{}{GECICI_EKI}", ad.to_string_lossy());
    let kucuk = |metin: &str| {
        if cfg!(windows) {
            metin.to_lowercase()
        } else {
            metin.to_string()
        }
    };
    let onek = kucuk(&onek);
    let klasor = hedef
        .parent()
        .filter(|klasor| !klasor.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let girdiler = match std::fs::read_dir(klasor) {
        Ok(girdiler) => girdiler,
        Err(hata) if hata.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(hata) => return Err(hata),
    };
    let mut bulunan = Vec::new();
    for girdi in girdiler {
        let girdi = girdi?;
        let ad = kucuk(&girdi.file_name().to_string_lossy());
        if ad.starts_with(&onek) && ad.ends_with(YEDEK_EKI) {
            let zaman = girdi.metadata().and_then(|m| m.modified()).ok();
            bulunan.push((zaman, girdi.path()));
        }
    }
    bulunan.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(bulunan.into_iter().map(|(_, yol)| yol).collect())
}

/// Yedeğin eşi olan, yayımlanmamış geçici dosya yolu.
pub(crate) fn yedegin_gecicisi(yedek: &Path) -> Option<PathBuf> {
    let ad = yedek.file_name()?.to_str()?;
    let gecici = ad.strip_suffix(YEDEK_EKI)?;
    Some(yedek.with_file_name(gecici))
}

/// Kilitsiz okuyucu: hedef yoksa ve yarım değiştirme yedeği duruyorsa,
/// zee yazarlarını paylaşımlı kilitle durdurup önce hedefi (bir yazar
/// arada kurtarmış olabilir), yoksa en yeni yedeği açar.
pub(crate) fn yedekten_ac(hedef: &Path, uzun: std::time::Duration) -> Option<std::fs::File> {
    if yedekler(hedef).ok()?.is_empty() {
        return None;
    }
    // Kilit tutuluyorsa ya da beklenemediyse yine okunur; yedeği kurtaran
    // yazar onu taşırsa açma düşer ve özgün hata döner.
    let _bariyer = if super::degistirme_penceresi::yazma_kilidi_tutuluyor() {
        None
    } else {
        super::degistirme_penceresi::degistirmenin_bitmesini_bekle(hedef, uzun)
    };
    if let Ok(dosya) = std::fs::File::open(hedef) {
        return Some(dosya);
    }
    let yedek = yedekler(hedef).ok()?.into_iter().next()?;
    std::fs::File::open(yedek).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hedefe_ait_yedekler_en_yeni_once_ve_gecici_esiyle_bulunur() {
        let kok = std::env::temp_dir().join(format!(
            "zee-k348-iz-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&kok).unwrap();
        let hedef = kok.join("ortak.txt");
        let eski = kok.join(".ortak.txt.zee-gecici-1-1.zee-yedek");
        let yeni = kok.join(".ortak.txt.zee-gecici-2-1.zee-yedek");
        std::fs::write(&eski, "eski").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&yeni, "yeni").unwrap();
        // Başka hedefin yedeği ve yalnız geçici dosya sayılmaz.
        std::fs::write(kok.join(".baska.txt.zee-gecici-1-1.zee-yedek"), "").unwrap();
        std::fs::write(kok.join(".ortak.txt.zee-gecici-3-1"), "").unwrap();
        assert_eq!(yedekler(&hedef).unwrap(), vec![yeni.clone(), eski]);
        assert_eq!(
            yedegin_gecicisi(&yeni),
            Some(kok.join(".ortak.txt.zee-gecici-2-1"))
        );
        assert_eq!(
            gecici_ad(&hedef, "7-8").unwrap(),
            OsString::from(".ortak.txt.zee-gecici-7-8")
        );
        let mut okunan = String::new();
        std::io::Read::read_to_string(
            &mut yedekten_ac(&hedef, std::time::Duration::from_millis(50)).expect("yedek"),
            &mut okunan,
        )
        .unwrap();
        assert_eq!(okunan, "yeni");
        std::fs::write(&hedef, "kurtarıldı").unwrap();
        okunan.clear();
        std::io::Read::read_to_string(
            &mut yedekten_ac(&hedef, std::time::Duration::from_millis(50)).expect("hedef"),
            &mut okunan,
        )
        .unwrap();
        assert_eq!(okunan, "kurtarıldı", "hedef varsa yedek okunmaz");
        let _ = std::fs::remove_dir_all(&kok);
    }
}
