//! Binary medya dogfood'undan çıkan dar dosya yaşam döngüsü kalıpları.

use super::*;

pub(super) fn yasam_dongusu_kalibi(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    let n = tokenlar.len();
    let kelime = |i: usize| -> Option<&str> {
        match &tokenlar[i].tur {
            TokenTur::Kelime(k) => Some(k.as_str()),
            _ => None,
        }
    };

    // KAYNAK dosyasını HEDEF yoluna atomik taşımayı dene.
    if n == 7
        && kelime(1) == Some("dosyasını")
        && kelime(3) == Some("yoluna")
        && kelime(4) == Some("atomik")
        && kelime(5) == Some("taşımayı")
        && kelime(6) == Some("dene")
    {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: DOSYA_ATOMIK_TASI.into(),
            argumanlar: vec![
                tekil_ifade(tokenlar[0].clone())?,
                tekil_ifade(tokenlar[2].clone())?,
            ],
        }));
    }

    // YOL dosyasını silmeyi dene.
    if n == 4
        && kelime(1) == Some("dosyasını")
        && kelime(2) == Some("silmeyi")
        && kelime(3) == Some("dene")
    {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: DOSYA_SIL.into(),
            argumanlar: vec![tekil_ifade(tokenlar[0].clone())?],
        }));
    }

    // DIZIN yolundaki dosyaları listelemeyi dene.
    if n == 5
        && kelime(1) == Some("yolundaki")
        && kelime(2) == Some("dosyaları")
        && kelime(3) == Some("listelemeyi")
        && kelime(4) == Some("dene")
    {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: DOSYALARI_LISTELE.into(),
            argumanlar: vec![tekil_ifade(tokenlar[0].clone())?],
        }));
    }

    // YOL dosyasının sha256 özetini almayı dene.
    if n == 6
        && kelime(1) == Some("dosyasının")
        && kelime(2) == Some("sha256")
        && kelime(3) == Some("özetini")
        && kelime(4) == Some("almayı")
        && kelime(5) == Some("dene")
    {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: DOSYA_SHA256.into(),
            argumanlar: vec![tekil_ifade(tokenlar[0].clone())?],
        }));
    }

    // YOL dosyasının ilk N baytını onaltılık okumayı dene. (K-331)
    if n == 8
        && kelime(1) == Some("dosyasının")
        && kelime(2) == Some("ilk")
        && kelime(4) == Some("baytını")
        && kelime(5) == Some("onaltılık")
        && kelime(6) == Some("okumayı")
        && kelime(7) == Some("dene")
    {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: DOSYA_ILK_BAYTLAR.into(),
            argumanlar: vec![
                tekil_ifade(tokenlar[0].clone())?,
                tekil_ifade(tokenlar[3].clone())?,
            ],
        }));
    }

    if let Some(ifade) = kodlanmis_icerik_kalibi(tokenlar)? {
        return Ok(Some(ifade));
    }

    Ok(None)
}

/// K-389 (RFC-0043 Dilim 1): aralıklı okuma ve çözerek yazma kalıpları.
///
/// - `YOL dosyasının KONUM baytından N baytını metin olarak okumayı dene`
/// - `YOL dosyasının KONUM baytından N baytını base64 okumayı dene`
/// - `İÇERİK base64|quoted_printable çözerek YOL dosyasına yazmayı|eklemeyi dene`
fn kodlanmis_icerik_kalibi(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    use crate::intrinsic::{
        DOSYA_ARALIK_BASE64, DOSYA_ARALIK_METIN, DOSYA_BASE64_EKLE, DOSYA_BASE64_YAZ,
        DOSYA_QP_EKLE, DOSYA_QP_YAZ,
    };
    let kelime = |i: usize| -> Option<&str> {
        match &tokenlar.get(i)?.tur {
            TokenTur::Kelime(k) => Some(k.as_str()),
            _ => None,
        }
    };
    let n = tokenlar.len();
    // X in sha256 özeti — bellekteki metnin özeti (saf Ozellik).
    if n == 3 && kelime(1) == Some("sha256") && kelime(2) == Some("özeti") {
        return Ok(Some(Ifade::Ozellik {
            nesne: Box::new(tekil_ifade(tokenlar[0].clone())?),
            ozellik: Ozellik::Sha256Ozeti,
        }));
    }
    let aralik = n >= 9
        && kelime(1) == Some("dosyasının")
        && kelime(3) == Some("baytından")
        && kelime(5) == Some("baytını")
        && kelime(n - 2) == Some("okumayı")
        && kelime(n - 1) == Some("dene");
    let kimlik = match (n, kelime(6), kelime(7)) {
        (10, Some("metin"), Some("olarak")) if aralik => Some(DOSYA_ARALIK_METIN),
        (9, Some("base64"), _) if aralik => Some(DOSYA_ARALIK_BASE64),
        _ => None,
    };
    if let Some(kimlik) = kimlik {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: kimlik.into(),
            argumanlar: vec![
                tekil_ifade(tokenlar[0].clone())?,
                tekil_ifade(tokenlar[2].clone())?,
                tekil_ifade(tokenlar[4].clone())?,
            ],
        }));
    }
    if n != 7 || kelime(2) != Some("çözerek") || kelime(4) != Some("dosyasına") {
        return Ok(None);
    }
    let kimlik = match (kelime(1), kelime(5), kelime(6)) {
        (Some("base64"), Some("yazmayı"), Some("dene")) => DOSYA_BASE64_YAZ,
        (Some("base64"), Some("eklemeyi"), Some("dene")) => DOSYA_BASE64_EKLE,
        (Some("quoted_printable"), Some("yazmayı"), Some("dene")) => DOSYA_QP_YAZ,
        (Some("quoted_printable"), Some("eklemeyi"), Some("dene")) => DOSYA_QP_EKLE,
        _ => return Ok(None),
    };
    Ok(Some(Ifade::Intrinsic {
        kimlik: kimlik.into(),
        argumanlar: vec![
            tekil_ifade(tokenlar[0].clone())?,
            tekil_ifade(tokenlar[3].clone())?,
        ],
    }))
}

/// K-378: `YOL dosyasını TÜR türünde AD adıyla ek olarak göndermeyi dene`.
pub(super) fn dosya_yaniti_kalibi(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    let kelime = |i: usize| -> Option<&str> {
        match &tokenlar[i].tur {
            TokenTur::Kelime(k) => Some(k.as_str()),
            _ => None,
        }
    };
    if tokenlar.len() == 10
        && kelime(1) == Some("dosyasını")
        && kelime(3) == Some("türünde")
        && kelime(5) == Some("adıyla")
        && kelime(6) == Some("ek")
        && kelime(7) == Some("olarak")
        && kelime(8) == Some("göndermeyi")
        && kelime(9) == Some("dene")
    {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: WEB_DOSYA_YANITI.into(),
            argumanlar: vec![
                tekil_ifade(tokenlar[0].clone())?,
                tekil_ifade(tokenlar[2].clone())?,
                tekil_ifade(tokenlar[4].clone())?,
            ],
        }));
    }
    Ok(None)
}
