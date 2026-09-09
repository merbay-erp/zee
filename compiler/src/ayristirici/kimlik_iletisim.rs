//! K-181/ADR-075 kaynak kalıpları: parola özeti, TOTP, e-posta ve görsel
//! küçültme. Yalnız lowering yapar; tür, yetkinlik ve etki merkezi intrinsic
//! kaydındadır.

use super::ifade::tekil_ifade;
use super::*;

fn kelime(tokenlar: &[Token], i: usize) -> Option<&str> {
    match &tokenlar[i].tur {
        TokenTur::Kelime(k) => Some(k.as_str()),
        _ => None,
    }
}

/// Değer kalıpları (`yapili_kalip` içinden çağrılır).
pub(super) fn yapili_kalip(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    let n = tokenlar.len();
    let Some(son) = kelime(tokenlar, n - 1) else {
        return Ok(None);
    };
    let k = |i: usize| kelime(tokenlar, i);

    // W ın argon2 özeti — Argon2id PHC parola özeti (K-181).
    if n == 3 && son == "özeti" && k(1) == Some("argon2") {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: PAROLA_OZETI.into(),
            argumanlar: vec![tekil_ifade(tokenlar[0].clone())?],
        }));
    }

    // yeni totp anahtarı — 160 bit Base32 gizli anahtar (K-181).
    if n == 3 && k(0) == Some("yeni") && k(1) == Some("totp") && son == "anahtarı" {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: TOTP_ANAHTARI.into(),
            argumanlar: Vec::new(),
        }));
    }

    // <alıcı> <konu> <gövde> ile ileti göndermeyi dene → TamSayı Sonucu (K-181).
    if n == 7
        && k(3) == Some("ile")
        && k(4) == Some("ileti")
        && k(5) == Some("göndermeyi")
        && son == "dene"
    {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: EPOSTA_GONDER.into(),
            argumanlar: vec![
                tekil_ifade(tokenlar[0].clone())?,
                tekil_ifade(tokenlar[1].clone())?,
                tekil_ifade(tokenlar[2].clone())?,
            ],
        }));
    }

    // <kaynak> görselini <hedef> dosyasına <N> piksele küçültmeyi dene (K-181).
    if n == 8
        && k(1) == Some("görselini")
        && k(3) == Some("dosyasına")
        && k(5) == Some("piksele")
        && k(6) == Some("küçültmeyi")
        && son == "dene"
    {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: GORSEL_KUCULT.into(),
            argumanlar: vec![
                tekil_ifade(tokenlar[0].clone())?,
                tekil_ifade(tokenlar[2].clone())?,
                tekil_ifade(tokenlar[4].clone())?,
            ],
        }));
    }

    Ok(None)
}

/// Koşul kalıpları (`kosul_atomu` içinden çağrılır).
pub(super) fn kosul_kalibi(tokenlar: &[Token]) -> Result<Option<Ifade>, Tani> {
    let n = tokenlar.len();
    let kelimeler: Vec<Option<&str>> = (0..n).map(|i| kelime(tokenlar, i)).collect();
    let Some(yuklem) = kelimeler.last().copied().flatten() else {
        return Ok(None);
    };
    // <kod> <gizli> anahtarıyla doğrulanıyorsa — RFC 6238 TOTP (K-181).
    if n == 4 && kelimeler[2] == Some("anahtarıyla") && yuklem == "doğrulanıyorsa" {
        return Ok(Some(Ifade::Intrinsic {
            kimlik: TOTP_DOGRULA.into(),
            argumanlar: vec![
                tekil_ifade(tokenlar[0].clone())?,
                tekil_ifade(tokenlar[1].clone())?,
            ],
        }));
    }

    Ok(None)
}
