//! K-342: Windows `ReplaceFileW` okuyucu penceresi (spec/08).
//!
//! `ReplaceFileW` hedefi iki yeniden adlandırmayla değiştirir ve değiştirme
//! bitene kadar yeni hedefi paylaşımsız tutanakla açık tutar. Aradaki anda
//! okuyucu `ERROR_FILE_NOT_FOUND`/`PATH_NOT_FOUND` ya da
//! `ERROR_SHARING_VIOLATION`/`ACCESS_DENIED` alır. Boşta bu pencere ~1 ms'dir;
//! yüklü makinede yazar iş parçacığı değiştirmenin ortasında kesilirse
//! onlarca-yüzlerce ms'ye uzar. K-182'nin sayıya bağlı ~50 ms bütçesi CI'da
//! tükendi. Hedefin yanında `kalici_dosya` geçici/yedek dosyası varken bir zee
//! değiştirmesi sürüyordur; okuyucu o sürece yazma kilidi bütçesi kadar bekler.
//! Böyle bir iz yokken gerçekten olmayan ya da başka süreçte kilitli dosya eski
//! kısa bütçeyle hata verir.

// Üretimde yalnız Windows okuyucusu kullanır; testler her platformda koşar.
#![cfg_attr(not(windows), allow(dead_code))]

use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

/// `kalici_dosya` geçici dosyalarının adı `.<hedef>.zee-gecici-…`, Windows
/// yedeği `.<hedef>.zee-gecici-….zee-yedek` biçimindedir.
pub(crate) const ZEE_GECICI_EKI: &str = ".zee-gecici-";

/// İz bulunmayan pencere için kısa bütçe (K-182 ile aynı ölçek).
pub(crate) const KISA_BUTCE: Duration = Duration::from_millis(50);

/// Hedefle aynı klasörde ona ait bir zee geçici ya da yedek dosyası var mı?
pub(crate) fn degistirme_izi_var(hedef: &Path) -> bool {
    let Some(ad) = hedef.file_name() else {
        return false;
    };
    let mut onek = OsString::from(".");
    onek.push(ad);
    onek.push(ZEE_GECICI_EKI);
    // Windows adları büyük/küçük harf duyarsızdır.
    let onek = onek.to_string_lossy().to_lowercase();
    let klasor = hedef
        .parent()
        .filter(|klasor| !klasor.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let Ok(girdiler) = std::fs::read_dir(klasor) else {
        return false;
    };
    girdiler.filter_map(Result::ok).any(|girdi| {
        girdi
            .file_name()
            .to_string_lossy()
            .to_lowercase()
            .starts_with(&onek)
    })
}

/// `ac` geçici pencere kodlarıyla düştükçe yeniden dener. İlk `KISA_BUTCE`
/// koşulsuzdur; sonrasında yalnız değiştirme izi sürdükçe `uzun` bütçeye kadar
/// devam eder. Diğer hatalar hemen döner.
pub(crate) fn pencereyi_bekleyerek<T>(
    hedef: &Path,
    uzun: Duration,
    mut ac: impl FnMut() -> io::Result<T>,
) -> io::Result<T> {
    let baslangic = Instant::now();
    let mut bekleme = Duration::from_millis(1);
    loop {
        let hata = match ac() {
            Ok(deger) => return Ok(deger),
            Err(hata) => hata,
        };
        // 2/3/5/32: `ACCESS_DENIED` ve `SHARING_VIOLATION` `Uncategorized`
        // sınıfına düştüğü için ham kod eşlenir.
        if !matches!(hata.raw_os_error(), Some(2 | 3 | 5 | 32)) {
            return Err(hata);
        }
        let gecen = baslangic.elapsed();
        let butce = if gecen < KISA_BUTCE || !degistirme_izi_var(hedef) {
            KISA_BUTCE
        } else {
            uzun
        };
        if gecen >= butce {
            return Err(hata);
        }
        std::thread::sleep(bekleme.min(butce - gecen));
        bekleme = (bekleme * 2).min(Duration::from_millis(16));
    }
}
