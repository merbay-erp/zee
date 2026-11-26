//! RFC-0043 bellekteki metin ↔ bayt kalıpları (saf `Ozellik`; yetkinlik ve
//! IO izi yok). K-389 Dilim 1 `sha256 özeti`; K-402 Dilim 2 UTF-8 bayt
//! sayısı, base64 ve kod noktası.

use super::super::*;

pub(super) fn kalip(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    let kelimeler: Vec<Option<&str>> = tokenlar
        .iter()
        .skip(1)
        .map(|t| match &t.tur {
            TokenTur::Kelime(k) => Some(k.as_str()),
            _ => None,
        })
        .collect();
    let kuyruk: Option<Vec<&str>> = kelimeler.into_iter().collect();
    let ozellik = match kuyruk.as_deref() {
        Some(["sha256", "özeti"]) => Ozellik::Sha256Ozeti,
        Some(["utf8", "bayt", "sayısı"]) => Ozellik::Utf8BaytSayisi,
        Some(["base64", "karşılığı"]) => Ozellik::Base64Karsiligi,
        Some(["base64", "çözümünü", "almayı", "dene"]) => Ozellik::Base64Cozumu,
        Some(["kod", "noktası"]) => Ozellik::KodNoktasi,
        Some(["kod", "noktası", "harfini", "almayı", "dene"]) => Ozellik::KodNoktasiHarfi,
        _ => return Ok(None),
    };
    Ok(Some(Ifade::Ozellik {
        nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
        ozellik,
    }))
}
