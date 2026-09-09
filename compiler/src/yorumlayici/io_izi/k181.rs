//! K-181 kriptografi/iletişim/görsel olaylarının IO izi alan kodlaması ve şeması.

use super::codec::{bool_coz, metin_sonuc_coz, tamsayi_sonuc_coz};
use super::IzOlay;
use crate::guvenlik::sha256_hex;

pub(super) fn parola(parola: &str) -> Vec<String> {
    vec![sha256_hex(parola.as_bytes())]
}

pub(super) fn totp(gizli: &str, kod: &str) -> Vec<String> {
    vec![sha256_hex(gizli.as_bytes()), sha256_hex(kod.as_bytes())]
}

pub(super) fn eposta(alici: &str, konu: &str, govde: &str) -> Vec<String> {
    vec![alici.into(), konu.into(), sha256_hex(govde.as_bytes())]
}

pub(super) fn gorsel(kaynak: &str, hedef: &str, azami: i64) -> Vec<String> {
    vec![kaynak.into(), hedef.into(), azami.to_string()]
}

fn parmak_izi_mi(alan: &str) -> bool {
    alan.len() == 64 && alan.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Olay şeması: argüman sayısı, parmak izi biçimi ve sonuç kodlaması.
pub(super) fn sema(olay: &IzOlay) -> Result<(), String> {
    let hata = |neden: &str| Err(format!("`{}` {neden} şeması geçersiz", olay.islem));
    let (arguman, parmak_izli) = match olay.islem.as_str() {
        "parola_ozeti_uret" => (1, true),
        "totp_anahtari_uret" => (0, false),
        "totp_dogrula" => (2, true),
        "eposta_gonder" | "gorsel_kucult" => (3, false),
        _ => return hata("bilinmeyen olay"),
    };
    if olay.argumanlar.len() != arguman {
        return hata("argüman sayısı");
    }
    if parmak_izli && !olay.argumanlar.iter().all(|alan| parmak_izi_mi(alan)) {
        return hata("parmak izi");
    }
    let gecerli = match olay.islem.as_str() {
        "totp_dogrula" => olay.sonuc.len() == 1 && bool_coz(&olay.sonuc[0]).is_ok(),
        "parola_ozeti_uret" | "totp_anahtari_uret" => metin_sonuc_coz(&olay.sonuc).is_ok(),
        _ => tamsayi_sonuc_coz(&olay.sonuc).is_ok(),
    };
    if gecerli {
        Ok(())
    } else {
        hata("sonuç")
    }
}
