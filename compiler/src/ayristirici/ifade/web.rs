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
