//! K-384: ad çözümleme tanılarına bağlam ipuçları (RFC-0039 A2c, ZeeMail
//! gözlemleri). Kod ve kabul/ret davranışı değişmez; yalnız öneri metni.
use super::*;

/// K-384: A001'e bağlam ipucu. Örtük `bağlantı` Sözlüğü yalnız bağlantı
/// bloğunun kendi gövdesindedir (sözcüksel kapsam; işleme dinamik olarak
/// geçmez); yerleşik fiille biten işlem adı cümle olarak çağrılamaz.
pub(super) fn a001_ipucu<'a>(
    tani: Tani,
    ham: &str,
    islem_adlari: impl Iterator<Item = &'a String>,
) -> Tani {
    if tani.kod != "A001" {
        return tani;
    }
    let mut adaylar = crate::morfoloji::kok_adaylari(ham);
    adaylar.push(ham.to_string());
    if adaylar.iter().any(|aday| aday == "bağlantı") {
        return tani.onerili(
            "Örtük `bağlantı` Sözlüğü yalnız `\"<ad>\" dinleyicisinde bağlantı geldiğinde` bloğunun kendi gövdesinde tanımlıdır; çağrılan işlem onu görmez (bağlantı okuma/yazma kalıpları işlemde de çalışır). Gereken alanı işleme parametre olarak geçir: `bağlantının \"uzak_adres\" değeri ile adresi kaydet`."
                .into(),
        );
    }
    let mut eslesen: Vec<&String> = islem_adlari
        .filter(|ad| ad.split(' ').next() == Some(ham) && ad.contains(' '))
        .collect();
    eslesen.sort();
    match eslesen
        .first()
        .and_then(|ad| crate::tani::fiil_cakismasi_ipucu(ad))
    {
        Some(ipucu) => tani.onerili(ipucu),
        None => tani,
    }
}
