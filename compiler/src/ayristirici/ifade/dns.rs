//! K-390 (RFC-0044 B1): DNS sorgu kalıbı. Biçim ürün H reproducer'ından
//! (ZM-R006) gelir: `AD için MX kayıtlarını sorgulamayı dene`. Protokol
//! adları (MX, TXT, TLSA) çevrilmez ve operatörün aradığı yazımla büyük
//! harftir; `adres` A ve AAAA'yı birlikte sorar.
use super::*;
use crate::intrinsic::{DNS_ADRES, DNS_MX, DNS_TLSA, DNS_TXT};

/// `AD için MX|TXT|TLSA|adres kayıtlarını sorgulamayı dene`.
pub(super) fn dns_kalibi(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    let kelime = |i: usize| -> Option<&str> {
        match &tokenlar.get(i)?.tur {
            TokenTur::Kelime(k) => Some(k.as_str()),
            _ => None,
        }
    };
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
