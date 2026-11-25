//! K-394 (RFC-0045, ADR-170): paylaşılan veri kökü yol sözdizimi ve yol
//! sınıfına göre gereken dosya yetkinliği.

use super::Yetkinlik;

/// Bildirimsiz tek dosya profillerinde örtük açılmayan yetkinlikler; veri
/// kökü yalnız proje bildirimiyle açılır.
pub(super) const BILDIRIMSIZ_KAPALI: [Yetkinlik; 5] = [
    Yetkinlik::YerelAg,
    Yetkinlik::Veritabani,
    Yetkinlik::Dns,
    Yetkinlik::VeriOkuma,
    Yetkinlik::VeriYazma,
];

impl Yetkinlik {
    /// K-394: dosya yolunun gerektirdiği yetkinlik. `veri:` önekli yol
    /// paylaşılan veri köküne gider ve `veri-okuma`/`veri-yazma` ister;
    /// diğer her yol proje dosya sınırındadır (`dosya-okuma`/`dosya-yazma`).
    pub fn dosya_icin(yol: &str, yazma: bool) -> Self {
        match (veri_yolu_mu(yol), yazma) {
            (true, false) => Self::VeriOkuma,
            (true, true) => Self::VeriYazma,
            (false, false) => Self::DosyaOkuma,
            (false, true) => Self::DosyaYazma,
        }
    }
}

/// K-394 (RFC-0045, spec/07): paylaşılan veri kökü yollarının öneki.
pub const VERI_ONEKI: &str = "veri:";

/// Yol `veri:` önekiyle mi başlıyor.
pub fn veri_yolu_mu(yol: &str) -> bool {
    yol.starts_with(VERI_ONEKI)
}

/// K-394: `veri:` sonrasındaki göreli yolun sözdizimi. Boş olmayan, `/` ile
/// ayrılmış bileşenler; boş bileşen, `.`, `..`, ters bölü, sürücü/ADS iki
/// noktası, NUL ve denetim karakteri yasak; bileşen en çok 255, toplam en
/// çok 1024 bayt. Önek yoksa `None`.
pub fn veri_yolu_bilesenleri(yol: &str) -> Option<Result<Vec<&str>, String>> {
    let goreli = yol.strip_prefix(VERI_ONEKI)?;
    let red = |neden: &str| Err(format!("\"{yol}\" veri kökü yolu reddedildi: {neden}"));
    if goreli.is_empty() {
        return Some(red("`veri:` sonrasında göreli bir yol olmalı"));
    }
    if goreli.len() > 1024 {
        return Some(red("yol 1024 baytı aşıyor"));
    }
    if goreli.starts_with('/') {
        return Some(red("mutlak yol kullanılamaz"));
    }
    let mut bilesenler = Vec::new();
    for parca in goreli.split('/') {
        let neden = if parca.is_empty() {
            Some("boş yol bileşeni (`//` ya da sondaki `/`) kullanılamaz")
        } else if parca == "." || parca == ".." {
            Some("`.` ya da `..` bileşeni kullanılamaz")
        } else if parca.len() > 255 {
            Some("yol bileşeni 255 baytı aşıyor")
        } else if parca
            .chars()
            .any(|c| c.is_control() || c == '\\' || c == ':')
        {
            Some("ters bölü, iki nokta ve denetim karakteri kullanılamaz")
        } else {
            None
        };
        if let Some(neden) = neden {
            return Some(red(neden));
        }
        bilesenler.push(parca);
    }
    Some(Ok(bilesenler))
}
