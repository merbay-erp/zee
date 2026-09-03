//! Yetkinlik reddinin adaptörlere özgü hata değerine dönüşümü.

use super::VeritabaniHatasi;
use crate::yetkinlik::Yetkinlik;

pub(super) fn mesaj(hata: String, cocuk_modu: bool, yetkinlik: Yetkinlik) -> String {
    if !cocuk_modu {
        return hata;
    }
    match yetkinlik {
        Yetkinlik::Ag => "güvenli modda ağ erişimi kapalı".into(),
        Yetkinlik::AgSunucusu => "güvenli modda sunucu açılamaz".into(),
        _ => format!("güvenli modda {}", hata),
    }
}

pub(super) fn veritabani(mesaj: String) -> VeritabaniHatasi {
    VeritabaniHatasi {
        mesaj,
        veri: Vec::new(),
    }
}
