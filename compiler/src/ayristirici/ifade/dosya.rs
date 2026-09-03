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

    Ok(None)
}
