//! K-210 kriptografi/iletişim/görsel olaylarının IO izi alan kodlaması ve şeması.

use super::codec::{bool_coz, metin_sonuc_coz, tamsayi_sonuc_coz};
use super::IzOlay;
use crate::guvenlik::sha256_hex;

pub(super) fn parola(parola: &str) -> Vec<String> {
    vec![sha256_hex(parola.as_bytes())]
}

pub(super) fn totp(gizli: &str, kod: &str) -> Vec<String> {
    vec![sha256_hex(gizli.as_bytes()), sha256_hex(kod.as_bytes())]
}

/// Bağlantı (parola taşır) ve gövde yalnız parmak iziyle kaydedilir; bağlantı
/// verilmemişse (ortam değişkeni yolu) alan boş kalır.
pub(super) fn eposta(baglanti: Option<&str>, alici: &str, konu: &str, govde: &str) -> Vec<String> {
    let baglanti = baglanti.map_or_else(String::new, |b| sha256_hex(b.as_bytes()));
    vec![
        baglanti,
        alici.into(),
        konu.into(),
        sha256_hex(govde.as_bytes()),
    ]
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
        "parola_ozeti_uret" | "gizli_sifrele" | "gizli_coz" => (1, true),
        "totp_anahtari_uret" => (0, false),
        "totp_dogrula" => (2, true),
        "eposta_gonder" => (4, false),
        "gorsel_kucult" => (3, false),
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
        "parola_ozeti_uret" | "totp_anahtari_uret" | "gizli_sifrele" | "gizli_coz" => {
            metin_sonuc_coz(&olay.sonuc).is_ok()
        }
        _ => tamsayi_sonuc_coz(&olay.sonuc).is_ok(),
    };
    if gecerli {
        Ok(())
    } else {
        hata("sonuç")
    }
}

use super::*;

pub(super) fn oynat_parola_ozeti_uret(
    io: &mut IzYenidenOynatici,
    parola: &str,
) -> Result<String, String> {
    let sonuc = io.siradaki("parola_ozeti_uret", self::parola(parola));
    io.metin_sonuc("parola_ozeti_uret", sonuc)
}

pub(super) fn kaydet_parola_ozeti_uret<T: GirdiCikti>(
    io: &mut IzKaydedenIo<T>,
    parola: &str,
) -> Result<String, String> {
    let sonuc = io.ic.parola_ozeti_uret(parola);
    let alanlar = self::parola(parola);
    io.kaydet("parola_ozeti_uret", alanlar, metin_sonuc_yaz(&sonuc));
    sonuc
}

pub(super) fn oynat_gizli_sifrele(io: &mut IzYenidenOynatici, duz: &str) -> Result<String, String> {
    let sonuc = io.siradaki("gizli_sifrele", self::parola(duz));
    io.metin_sonuc("gizli_sifrele", sonuc)
}

pub(super) fn kaydet_gizli_sifrele<T: GirdiCikti>(
    io: &mut IzKaydedenIo<T>,
    duz: &str,
) -> Result<String, String> {
    let sonuc = io.ic.gizli_sifrele(duz);
    io.kaydet("gizli_sifrele", self::parola(duz), metin_sonuc_yaz(&sonuc));
    sonuc
}

pub(super) fn oynat_gizli_coz(io: &mut IzYenidenOynatici, sifreli: &str) -> Result<String, String> {
    let sonuc = io.siradaki("gizli_coz", self::parola(sifreli));
    io.metin_sonuc("gizli_coz", sonuc)
}

pub(super) fn kaydet_gizli_coz<T: GirdiCikti>(
    io: &mut IzKaydedenIo<T>,
    sifreli: &str,
) -> Result<String, String> {
    let sonuc = io.ic.gizli_coz(sifreli);
    io.kaydet("gizli_coz", self::parola(sifreli), metin_sonuc_yaz(&sonuc));
    sonuc
}

pub(super) fn oynat_totp_anahtari_uret(io: &mut IzYenidenOynatici) -> Result<String, String> {
    let sonuc = io.siradaki("totp_anahtari_uret", Vec::new());
    io.metin_sonuc("totp_anahtari_uret", sonuc)
}

pub(super) fn kaydet_totp_anahtari_uret<T: GirdiCikti>(
    io: &mut IzKaydedenIo<T>,
) -> Result<String, String> {
    let sonuc = io.ic.totp_anahtari_uret();
    io.kaydet("totp_anahtari_uret", Vec::new(), metin_sonuc_yaz(&sonuc));
    sonuc
}

pub(super) fn oynat_totp_dogrula(io: &mut IzYenidenOynatici, gizli: &str, kod: &str) -> bool {
    let sonuc = io.siradaki("totp_dogrula", self::totp(gizli, kod));
    io.bool_sonuc("totp_dogrula", sonuc)
}

pub(super) fn kaydet_totp_dogrula<T: GirdiCikti>(
    io: &mut IzKaydedenIo<T>,
    gizli: &str,
    kod: &str,
) -> bool {
    let sonuc = io.ic.totp_dogrula(gizli, kod);
    let alanlar = self::totp(gizli, kod);
    io.kaydet("totp_dogrula", alanlar, vec![bool_yaz(sonuc)]);
    sonuc
}

pub(super) fn oynat_eposta_gonder(
    io: &mut IzYenidenOynatici,
    b: Option<&str>,
    alici: &str,
    konu: &str,
    govde: &str,
) -> Result<i64, String> {
    let sonuc = io.siradaki("eposta_gonder", self::eposta(b, alici, konu, govde));
    io.tamsayi_sonuc("eposta_gonder", sonuc)
}

pub(super) fn kaydet_eposta_gonder<T: GirdiCikti>(
    io: &mut IzKaydedenIo<T>,
    b: Option<&str>,
    alici: &str,
    konu: &str,
    govde: &str,
) -> Result<i64, String> {
    let sonuc = io.ic.eposta_gonder(b, alici, konu, govde);
    let alanlar = self::eposta(b, alici, konu, govde);
    io.kaydet("eposta_gonder", alanlar, tamsayi_sonuc_yaz(&sonuc));
    sonuc
}

pub(super) fn oynat_gorsel_kucult(
    io: &mut IzYenidenOynatici,
    kaynak: &str,
    hedef: &str,
    azami: i64,
) -> Result<i64, String> {
    let sonuc = io.siradaki("gorsel_kucult", self::gorsel(kaynak, hedef, azami));
    io.tamsayi_sonuc("gorsel_kucult", sonuc)
}

pub(super) fn kaydet_gorsel_kucult<T: GirdiCikti>(
    io: &mut IzKaydedenIo<T>,
    kaynak: &str,
    hedef: &str,
    azami: i64,
) -> Result<i64, String> {
    let sonuc = io.ic.gorsel_kucult(kaynak, hedef, azami);
    let alanlar = self::gorsel(kaynak, hedef, azami);
    io.kaydet("gorsel_kucult", alanlar, tamsayi_sonuc_yaz(&sonuc));
    sonuc
}
