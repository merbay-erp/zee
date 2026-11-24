//! Dosya yollarının merkezî yetkinlik ve proje-kökü ön denetimi.

use crate::yetkinlik::{DosyaSiniri, Yetkinlik, YetkinlikPolitikasi};

pub(super) fn yolu_denetle(
    politika: &YetkinlikPolitikasi,
    cocuk_modu: bool,
    yol: &str,
    yetkinlik: Yetkinlik,
) -> Result<(), String> {
    // K-394 (RFC-0045): `veri:` yolu proje dosya sınırına değil bildirilen
    // paylaşılan veri köküne gider; ayrı `veri-okuma`/`veri-yazma` ister ve
    // tek dosya kipinde de (HerYer) yerel dosya adı sayılmaz.
    if let Some(bilesenler) = crate::yetkinlik::veri_yolu_bilesenleri(yol) {
        let yazma = yetkinlik == Yetkinlik::DosyaYazma;
        let gereken = Yetkinlik::dosya_icin(yol, yazma);
        if politika.bildirimsiz() || cocuk_modu {
            return Err(format!(
                "\"{yol}\" veri kökü yolu yalnız proje kipinde, proje.dil `veri_kökü` bildirimiyle kullanılabilir"
            ));
        }
        politika
            .gerektir(gereken)
            .map_err(|hata| super::super::yetkinlik_hatasi::mesaj(hata, cocuk_modu, gereken))?;
        return bilesenler.map(|_| ());
    }
    politika
        .gerektir(yetkinlik)
        .map_err(|hata| super::super::yetkinlik_hatasi::mesaj(hata, cocuk_modu, yetkinlik))?;
    if politika.dosya_siniri() == DosyaSiniri::HerYer {
        return Ok(());
    }
    let mutlak = yol.starts_with('/') || yol.starts_with('\\') || yol.chars().nth(1) == Some(':');
    let guvensiz = yol.is_empty()
        || yol.chars().any(char::is_control)
        || yol.split(['/', '\\']).any(|parca| parca == "..");
    if mutlak || guvensiz {
        if cocuk_modu {
            return Err(format!(
                "güvenli modda yalnız çalışma klasöründeki dosyalara erişilir; \"{}\" dışarıyı gösteriyor",
                yol
            ));
        }
        return Err(format!(
            "proje dosya sınırında yalnız kök içinde kalan göreli yol kullanılabilir; \"{}\" reddedildi",
            yol
        ));
    }
    Ok(())
}
