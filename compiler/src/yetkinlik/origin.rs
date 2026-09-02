//! Ağ hedefi ve web proxy için tek kanonik origin ayrıştırıcısı.

use std::net::{IpAddr, Ipv6Addr};

use super::{AgHedefi, AgSemasi};

pub(super) fn ayristir(yazim: &str) -> Result<(AgHedefi, &str), String> {
    if yazim.is_empty() || yazim.chars().any(char::is_control) || yazim.contains(['\\', '#']) {
        return Err("ağ adresi boş, denetim karakterli, ters bölülü veya parçalı olamaz".into());
    }
    let (sema, kalan) = yazim
        .split_once("://")
        .ok_or_else(|| "ağ adresi https:// veya açıkça izinli http:// ile başlamalı".to_string())?;
    let sema = match sema {
        "https" => AgSemasi::Https,
        "http" => AgSemasi::Http,
        _ => return Err("yalnız https:// ve açık izinli http:// şemaları desteklenir".into()),
    };
    let kuyruk_yeri = kalan.find(['/', '?']).unwrap_or(kalan.len());
    let (yetki, kuyruk) = kalan.split_at(kuyruk_yeri);
    if yetki.is_empty() || yetki.contains('@') {
        return Err("ağ adresi boş host veya kullanıcı bilgisi taşıyamaz".into());
    }
    let (konak, kapi) = konak_ve_kapi(yetki, sema.varsayilan_kapi())?;
    Ok((AgHedefi { sema, konak, kapi }, kuyruk))
}

fn konak_ve_kapi(yetki: &str, varsayilan: u16) -> Result<(String, u16), String> {
    let (konak, kapi) = if let Some(kalan) = yetki.strip_prefix('[') {
        let (konak, son) = kalan
            .split_once(']')
            .ok_or_else(|| "IPv6 host kapanış köşeli ayracını taşımıyor".to_string())?;
        let kapi = if son.is_empty() {
            varsayilan
        } else {
            kapiyi_ayristir(
                son.strip_prefix(':')
                    .ok_or_else(|| "IPv6 host sonrasında yalnız port yazılabilir".to_string())?,
            )?
        };
        konak
            .parse::<Ipv6Addr>()
            .map_err(|_| "köşeli ayraç içinde geçerli IPv6 adresi olmalı".to_string())?;
        (konak, kapi)
    } else {
        let (konak, kapi) = match yetki.rsplit_once(':') {
            Some((konak, kapi)) if !konak.contains(':') => (konak, kapiyi_ayristir(kapi)?),
            Some(_) => return Err("IPv6 adresi köşeli ayraç içinde yazılmalı".into()),
            None => (yetki, varsayilan),
        };
        (konak, kapi)
    };
    if kapi == 0 || !gecerli_konak(konak) {
        return Err("ağ hostu ASCII DNS adı veya kanonik IP, portu 1–65535 olmalı".into());
    }
    let konak = konak
        .parse::<IpAddr>()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| konak.to_ascii_lowercase());
    Ok((konak, kapi))
}

fn kapiyi_ayristir(yazim: &str) -> Result<u16, String> {
    if yazim.is_empty() || !yazim.bytes().all(|bayt| bayt.is_ascii_digit()) {
        return Err("ağ portu 1–65535 aralığında olmalı".into());
    }
    yazim
        .parse::<u16>()
        .map_err(|_| "ağ portu 1–65535 aralığında olmalı".to_string())
}

fn gecerli_konak(konak: &str) -> bool {
    if konak.parse::<IpAddr>().is_ok() {
        return true;
    }
    !konak.is_empty()
        && konak.len() <= 253
        && !konak.ends_with('.')
        && konak.split('.').all(|etiket| {
            !etiket.is_empty()
                && etiket.len() <= 63
                && !etiket.starts_with('-')
                && !etiket.ends_with('-')
                && etiket
                    .bytes()
                    .all(|bayt| bayt.is_ascii_alphanumeric() || bayt == b'-')
        })
}
