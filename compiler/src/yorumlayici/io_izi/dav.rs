//! RFC-0047 (ADR-182 dilim C): DAV kimlik kararının IO izi olayları.
//!
//! İz yalnız kullanıcı adını, arama önekini (gizli değil), çözücü adını,
//! kayıttaki özeti ve kova kararını taşır. `Authorization` değeri, gizli kısım
//! ve HMAC anahtarı hiçbir alana girmez: gizli kısım adaptörde kalır,
//! `dav_ozet_dogrula` yalnız karşılaştırma sonucunu kaydeder. 401/429/OPTIONS
//! yanıtları `durum_yaniti_gonder` olarak kaydedilir (başlıklar programdan ve
//! karardan deterministik türer).

use super::codec::{bool_coz, bool_yaz};
use super::*;
use crate::yorumlayici::{DavKimligi, DavKova};

/// Olay şeması.
pub(super) fn sema(olay: &IzOlay) -> Result<(), String> {
    let hata = |neden: &str| Err(format!("`{}` {neden} şeması geçersiz", olay.islem));
    match olay.islem.as_str() {
        "dav_kimligi" if olay.argumanlar.is_empty() => match olay.sonuc.as_slice() {
            [] => Ok(()),
            [kullanici, _, _] if !kullanici.is_empty() => Ok(()),
            _ => hata("sonuç"),
        },
        "dav_ozet_dogrula" if olay.argumanlar.len() == 1 => match olay.sonuc.as_slice() {
            [deger] if bool_coz(deger).is_ok() => Ok(()),
            _ => hata("sonuç"),
        },
        "dav_kovasi" if olay.argumanlar.len() == 2 => {
            if DavKova::ayristir(&olay.argumanlar[0]).is_none() {
                return hata("kova");
            }
            match olay.sonuc.as_slice() {
                [] => Ok(()),
                [saniye] if saniye.parse::<i64>().is_ok() => Ok(()),
                _ => hata("sonuç"),
            }
        }
        _ => hata("argüman sayısı"),
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
