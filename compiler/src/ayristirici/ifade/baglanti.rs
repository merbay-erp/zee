//! K-381 (RFC-0039 A2a): o anki bağlantı görevinin satır kalıpları.

use super::*;
use crate::intrinsic::{BAGLANTI_SATIR_OKU, BAGLANTI_SATIR_YAZ};

pub(super) fn baglanti_kalibi(
    tokenlar: &[Token],
    islemler: &[String],
) -> Result<Option<Ifade>, Tani> {
    let n = tokenlar.len();
    let kelime = |i: usize| -> Option<&str> {
        match &tokenlar[i].tur {
            TokenTur::Kelime(k) => Some(k.as_str()),
            _ => None,
        }
    };
    if n == 0 {
        return Ok(None);
    }
    // bağlantıdan satır okumayı dene
    if n == 4
        && kelime(0) == Some("bağlantıdan")
        && kelime(1) == Some("satır")
        && kelime(2) == Some("okumayı")
        && kelime(3) == Some("dene")
    {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: BAGLANTI_SATIR_OKU.into(),
            argumanlar: Vec::new(),
        }));
    }
    // bağlantıya <değer> satırını yazmayı dene
    if n >= 5
        && kelime(0) == Some("bağlantıya")
        && kelime(n - 3) == Some("satırını")
        && kelime(n - 2) == Some("yazmayı")
        && kelime(n - 1) == Some("dene")
    {
        let satir = tokenlar[0].satir;
        return Ok(Some(Ifade::Intrinsic {
            kimlik: BAGLANTI_SATIR_YAZ.into(),
            argumanlar: vec![ile_ifadesi(&tokenlar[1..n - 3], satir, islemler)?],
        }));
    }
    Ok(None)
}
