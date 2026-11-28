//! RFC-0047 (ADR-182 dilim C): DAV kimlik kararının IO izi olayları.
//!
//! İz yalnız kullanıcı adını, arama önekini (gizli değil), çözücü adını,
//! kayıttaki özeti ve kova kararını taşır. `Authorization` değeri, gizli kısım
//! ve HMAC anahtarı hiçbir alana girmez: gizli kısım adaptörde kalır,
//! `dav_ozet_dogrula` yalnız karşılaştırma sonucunu kaydeder. 401/429/OPTIONS
//! yanıtları `durum_yaniti_gonder` olarak kaydedilir (başlıklar programdan ve
//! karardan deterministik türer).

use super::codec::{birim_sonuc_coz, birim_sonuc_yaz, bool_coz, bool_yaz};
use super::*;
use crate::yorumlayici::{DavKimligi, DavKova};

/// DAV kimlik kararı ve DAV isteği olayları.
pub(super) const OLAYLAR: [&str; 5] = [
    "dav_kimligi",
    "dav_ozet_dogrula",
    "dav_kovasi",
    "etag_yaz",
    "dav_istek_basliklari",
];

fn kova_gecerli(olay: &IzOlay) -> bool {
    DavKova::ayristir(&olay.argumanlar[0]).is_some()
        && olay.sonuc.len() <= 1
        && olay.sonuc.iter().all(|s| s.parse::<i64>().is_ok())
}

fn sonuc_gecerli(olay: &IzOlay) -> bool {
    match (olay.islem.as_str(), olay.argumanlar.len()) {
        ("dav_kimligi", 0) => matches!(olay.sonuc.len(), 0 | 3),
        ("dav_ozet_dogrula", 1) => olay.sonuc.len() == 1 && bool_coz(&olay.sonuc[0]).is_ok(),
        ("dav_kovasi", 2) => kova_gecerli(olay),
        ("etag_yaz", 1) => birim_sonuc_coz(&olay.sonuc).is_ok(),
        ("dav_istek_basliklari", 0) => olay.sonuc.len().is_multiple_of(2),
        _ => false,
    }
}

/// Olay şeması.
pub(super) fn sema(olay: &IzOlay) -> Result<(), String> {
    if sonuc_gecerli(olay) {
        Ok(())
    } else {
        Err(format!("`{}` DAV olay şeması geçersiz", olay.islem))
    }
}

fn kimlik_yaz(kimlik: &Option<DavKimligi>) -> Vec<String> {
    match kimlik {
        None => Vec::new(),
        Some(k) => vec![
            k.kullanici.clone(),
            k.onek.clone().unwrap_or_default(),
            k.cozucu.clone().unwrap_or_default(),
        ],
    }
}

fn kova_yaz(sonuc: &Result<(), i64>) -> Vec<String> {
    match sonuc {
        Ok(()) => Vec::new(),
        Err(saniye) => vec![saniye.to_string()],
    }
}

fn bos_degilse(metin: &str) -> Option<String> {
    (!metin.is_empty()).then(|| metin.to_string())
}

pub(super) fn kaydet_kimlik<T: GirdiCikti>(io: &mut IzKaydedenIo<T>) -> Option<DavKimligi> {
    let kimlik = io.ic.dav_kimligi();
    io.kaydet("dav_kimligi", Vec::new(), kimlik_yaz(&kimlik));
    kimlik
}

pub(super) fn kaydet_ozet<T: GirdiCikti>(io: &mut IzKaydedenIo<T>, ozet: Option<&str>) -> bool {
    let sonuc = io.ic.dav_ozet_dogrula(ozet);
    io.kaydet(
        "dav_ozet_dogrula",
        vec![ozet.unwrap_or_default().into()],
        vec![bool_yaz(sonuc)],
    );
    sonuc
}

pub(super) fn kaydet_kova<T: GirdiCikti>(
    io: &mut IzKaydedenIo<T>,
    kova: DavKova,
    anahtar: &str,
) -> Result<(), i64> {
    let sonuc = io.ic.dav_kovasi(kova, anahtar);
    io.kaydet(
        "dav_kovasi",
        vec![kova.yazimi().into(), anahtar.into()],
        kova_yaz(&sonuc),
    );
    sonuc
}

pub(super) fn oynat_kimlik(io: &mut IzYenidenOynatici) -> Option<DavKimligi> {
    match io.siradaki("dav_kimligi", Vec::new())?.as_slice() {
        [] => None,
        [kullanici, onek, cozucu] => Some(DavKimligi {
            kullanici: kullanici.clone(),
            onek: bos_degilse(onek),
            cozucu: bos_degilse(cozucu),
        }),
        _ => None,
    }
}

pub(super) fn oynat_ozet(io: &mut IzYenidenOynatici, ozet: Option<&str>) -> bool {
    let sonuc = io.siradaki("dav_ozet_dogrula", vec![ozet.unwrap_or_default().into()]);
    io.bool_sonuc("dav_ozet_dogrula", sonuc)
}

pub(super) fn oynat_kova(
    io: &mut IzYenidenOynatici,
    kova: DavKova,
    anahtar: &str,
) -> Result<(), i64> {
    match io
        .siradaki("dav_kovasi", vec![kova.yazimi().into(), anahtar.into()])
        .as_deref()
    {
        Some([saniye]) => Err(saniye.parse().unwrap_or(1)),
        _ => Ok(()),
    }
}

pub(super) fn kaydet_etag<T: GirdiCikti>(
    io: &mut IzKaydedenIo<T>,
    etag: &str,
) -> Result<(), String> {
    let sonuc = io.ic.etag_yaz(etag);
    io.kaydet("etag_yaz", vec![etag.into()], birim_sonuc_yaz(&sonuc));
    sonuc
}

pub(super) fn kaydet_basliklar<T: GirdiCikti>(io: &mut IzKaydedenIo<T>) -> Vec<(String, String)> {
    let basliklar = io.ic.dav_istek_basliklari();
    let alanlar = basliklar
        .iter()
        .flat_map(|(ad, deger)| [ad.clone(), deger.clone()])
        .collect();
    io.kaydet("dav_istek_basliklari", Vec::new(), alanlar);
    basliklar
}

pub(super) fn oynat_etag(io: &mut IzYenidenOynatici, etag: &str) -> Result<(), String> {
    match io.siradaki("etag_yaz", vec![etag.into()]) {
        Some(alanlar) => birim_sonuc_coz(&alanlar).unwrap_or_else(Err),
        None => Err("IO izi uyuşmuyor".into()),
    }
}

pub(super) fn oynat_basliklar(io: &mut IzYenidenOynatici) -> Vec<(String, String)> {
    io.siradaki("dav_istek_basliklari", Vec::new())
        .unwrap_or_default()
        .chunks(2)
        .filter_map(|cift| match cift {
            [ad, deger] => Some((ad.clone(), deger.clone())),
            _ => None,
        })
        .collect()
}
