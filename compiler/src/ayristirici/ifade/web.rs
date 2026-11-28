use super::*;

/// Zee ERP K-188 oturum yönetimi ifadelerini merkezi intrinsic kimliklerine
/// indirir. Alan büyümesi ana ifade parser'ının faz bütçesini şişirmez.
pub(super) fn oturum_yonetimi_kalibi(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    let kelime = |i: usize| -> Option<&str> {
        match &tokenlar[i].tur {
            TokenTur::Kelime(k) => Some(k.as_str()),
            _ => None,
        }
    };
    match tokenlar.len() {
        6 if kelime(1) == Some("ile")
            && kelime(2) == Some("web")
            && kelime(3) == Some("oturumlarını")
            && kelime(4) == Some("listelemeyi")
            && kelime(5) == Some("dene") =>
        {
            Ok(Some(Ifade::Intrinsic {
                kimlik: WEB_OTURUMLARINI_LISTELE.into(),
                argumanlar: vec![tekil_ifade(tokenlar[0].clone())?],
            }))
        }
        7 if kelime(1) == Some("ile")
            && kelime(2) == Some("web")
            && kelime(3) == Some("oturumunu")
            && kelime(4) == Some("iptal")
            && kelime(5) == Some("etmeyi")
            && kelime(6) == Some("dene") =>
        {
            Ok(Some(Ifade::Intrinsic {
                kimlik: WEB_OTURUMUNU_IPTAL_ET.into(),
                argumanlar: vec![tekil_ifade(tokenlar[0].clone())?],
            }))
        }
        _ => Ok(None),
    }
}

/// Contextual, zero-argument HTTP metadata expressions (K-209).
/// K-405: `isteğin ham gövdesi` aynı yüzeyin üç kelimelik tek biçimidir.
/// RFC-0047 (ADR-182 dilim D): `isteğin "<ad>" başlığı` de bu yüzeydedir.
pub(super) fn sifir_argumanli(
    tokenlar: &[Token],
    ilk: Option<&str>,
    son: &str,
) -> Result<Option<Ifade>, Tani> {
    if let Some(sonuc) = crate::ayristirici::dav_basligi::istek_basligi(tokenlar, tokenlar[0].satir)
    {
        return sonuc.map(Some);
    }
    Ok(sifir_argumanli_ic(tokenlar, ilk, son))
}

fn sifir_argumanli_ic(tokenlar: &[Token], ilk: Option<&str>, son: &str) -> Option<Ifade> {
    let n = tokenlar.len();
    if n == 3
        && ilk == Some("isteğin")
        && matches!(&tokenlar[1].tur, TokenTur::Kelime(k) if k == "ham")
        && son == "gövdesi"
    {
        return Some(Ifade::Intrinsic {
            kimlik: crate::intrinsic::WEB_HAM_GOVDE.into(),
            argumanlar: Vec::new(),
        });
    }
    if n != 2 {
        return None;
    }
    let kimlik = match (ilk, son) {
        (Some("csrf"), "belirteci") => crate::intrinsic::CSRF_BELIRTECI,
        (Some("istemci"), "adresi") => crate::intrinsic::ISTEMCI_ADRESI,
        (Some("birincil"), "süreç") => crate::intrinsic::WEB_BIRINCIL_SUREC,
        _ => return None,
    };
    Some(Ifade::Intrinsic {
        kimlik: kimlik.into(),
        argumanlar: Vec::new(),
    })
}
