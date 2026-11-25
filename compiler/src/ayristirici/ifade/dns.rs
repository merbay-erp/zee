//! K-390 (RFC-0044 B1): DNS sorgu kalıbı. Biçim ürün H reproducer'ından
//! (ZM-R006) gelir: `AD için MX kayıtlarını sorgulamayı dene`. Protokol
//! adları (MX, TXT, TLSA) çevrilmez ve operatörün aradığı yazımla büyük
//! harftir; `adres` A ve AAAA'yı birlikte sorar.
use super::*;
use crate::intrinsic::{
    DNS_ADRES, DNS_MX, DNS_TLSA, DNS_TXT, POSTA_DKIM_DOGRULA, POSTA_DKIM_GOVDE_OZETI,
    POSTA_DKIM_IMZALA, POSTA_KURUMSAL_ALAN,
};

/// `AD için MX|TXT|TLSA|adres kayıtlarını sorgulamayı dene`.
pub(super) fn dns_kalibi(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    let kelime = |i: usize| -> Option<&str> {
        match &tokenlar.get(i)?.tur {
            TokenTur::Kelime(k) => Some(k.as_str()),
            _ => None,
        }
    };
    if let Some(ifade) = posta_kalibi(tokenlar, &kelime)? {
        return Ok(Some(ifade));
    }
    if tokenlar.len() != 6
        || kelime(1) != Some("için")
        || kelime(3) != Some("kayıtlarını")
        || kelime(4) != Some("sorgulamayı")
        || kelime(5) != Some("dene")
    {
        return Ok(None);
    }
    let kimlik = match kelime(2) {
        Some("MX") => DNS_MX,
        Some("TXT") => DNS_TXT,
        Some("TLSA") => DNS_TLSA,
        Some("adres") => DNS_ADRES,
        _ => return Ok(None),
    };
    Ok(Some(Ifade::Intrinsic {
        kimlik: kimlik.into(),
        argumanlar: vec![tekil_ifade(tokenlar[0].clone())?],
    }))
}

/// K-390 (RFC-0044 B1b/B2b) posta kimlik doğrulama ilkelleri; her işlenen
/// tekil token'dır (ad ya da sabit):
/// - `AD alanının kurumsal alanı` (PSL, `dns`);
/// - `B dkim imzasını VERİ verisi için P anahtarıyla A olarak doğrulamayı dene`
///   (`kriptografi`; kanonikleştirme Zee'de);
/// - `X gövdesinin dkim gövde özeti` (`bh=` biçiminde base64 SHA-256);
/// - K-397: `VERİ verisi için AD özel anahtarıyla A olarak imzalamayı dene`
///   (`kriptografi`; anahtar `dkim_anahtarları` bildiriminden).
fn posta_kalibi<'a>(
    tokenlar: &[Token],
    kelime: &impl Fn(usize) -> Option<&'a str>,
) -> Result<Option<Ifade>, Tani> {
    let kalip = |kelimeler: &[(usize, &str)]| kelimeler.iter().all(|(i, k)| kelime(*i) == Some(k));
    let (kimlik, konumlar): (&str, &[usize]) = match tokenlar.len() {
        4 if kalip(&[(1, "alanının"), (2, "kurumsal"), (3, "alanı")]) => {
            (POSTA_KURUMSAL_ALAN, &[0])
        }
        5 if kalip(&[(1, "gövdesinin"), (2, "dkim"), (3, "gövde"), (4, "özeti")]) => {
            (POSTA_DKIM_GOVDE_OZETI, &[0])
        }
        10 if kalip(&[
            (1, "verisi"),
            (2, "için"),
            (4, "özel"),
            (5, "anahtarıyla"),
            (7, "olarak"),
            (8, "imzalamayı"),
            (9, "dene"),
        ]) =>
        {
            (POSTA_DKIM_IMZALA, &[0, 3, 6])
        }
        12 if kalip(&[
            (1, "dkim"),
            (2, "imzasını"),
            (4, "verisi"),
            (5, "için"),
            (7, "anahtarıyla"),
            (9, "olarak"),
            (10, "doğrulamayı"),
            (11, "dene"),
        ]) =>
        {
            (POSTA_DKIM_DOGRULA, &[0, 3, 6, 8])
        }
        _ => return Ok(None),
    };
    let argumanlar = konumlar
        .iter()
        .map(|i| tekil_ifade(tokenlar[*i].clone()))
        .collect::<Result<_, _>>()?;
    Ok(Some(Ifade::Intrinsic {
        kimlik: kimlik.into(),
        argumanlar,
    }))
}
