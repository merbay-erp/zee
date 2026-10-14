use super::*;

pub(super) fn yapili_kalip(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    let kelime = |i: usize| match tokenlar.get(i).map(|t| &t.tur) {
        Some(TokenTur::Kelime(k)) => Some(k.as_str()),
        _ => None,
    };
    let n = tokenlar.len();
    // Mevcut GET kalıbı aynı AST ve argümanla HTTP handler'ında kalır.
    if n == 4
        && kelime(1) == Some("adresinden")
        && kelime(2) == Some("gelen")
        && kelime(3) == Some("yanıt")
    {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: HTTP_GETIR.into(),
            argumanlar: vec![tekil_ifade(tokenlar[0].clone())?],
        }));
    }
    let (kimlik, adet) = if n == 9
        && kelime(5) == Some("ile")
        && kelime(6) == Some("HTTP")
        && kelime(7) == Some("isteğini")
        && kelime(8) == Some("dene")
    {
        (crate::intrinsic::HTTP_ISTEK, 5)
    } else if n == 6
        && kelime(1) == Some("HTTP")
        && kelime(2) == Some("JSON")
        && kelime(3) == Some("sözlüğünü")
        && kelime(4) == Some("okumayı")
        && kelime(5) == Some("dene")
    {
        (crate::intrinsic::HTTP_JSON, 1)
    } else if n == 6
        && kelime(1) == Some("gizli")
        && kelime(2) == Some("ortam")
        && kelime(3) == Some("değerini")
        && kelime(4) == Some("okumayı")
        && kelime(5) == Some("dene")
    {
        (crate::intrinsic::HTTP_GIZLI_ORTAM, 1)
    } else {
        return Ok(None);
    };
    Ok(Some(Ifade::Intrinsic {
        kimlik: kimlik.into(),
        argumanlar: tokenlar[..adet]
            .iter()
            .cloned()
            .map(tekil_ifade)
            .collect::<Result<_, _>>()?,
    }))
}
