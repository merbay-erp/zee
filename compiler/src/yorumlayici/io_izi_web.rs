//! Web güvenliği olaylarının IO izi alan kodlaması.

use crate::agac::RotaErisimi;
use crate::web_guvenligi::WebReddi;

fn bool_yaz(deger: bool) -> String {
    if deger { "1" } else { "0" }.into()
}

fn bool_gecerli(metin: &str) -> bool {
    matches!(metin, "0" | "1")
}

fn secenek_yaz(deger: Option<&str>) -> Vec<String> {
    match deger {
        Some(deger) => vec!["var".into(), deger.into()],
        None => vec!["yok".into()],
    }
}

fn secenek_gecerli(alanlar: &[String]) -> bool {
    matches!(alanlar, [etiket] if etiket == "yok")
        || matches!(alanlar, [etiket, _] if etiket == "var")
}

pub(super) fn rota_argumanlari(
    erisim: &RotaErisimi,
    csrf: Option<&str>,
    csrf_gerekli: bool,
) -> Vec<String> {
    let mut alanlar = match erisim {
        RotaErisimi::HerkeseAcik => vec!["herkese_acik".into()],
        RotaErisimi::Oturumlu => vec!["oturumlu".into()],
        RotaErisimi::Rol(rol) => vec!["rol".into(), rol.clone()],
    };
    alanlar.extend(secenek_yaz(csrf));
    alanlar.push(bool_yaz(csrf_gerekli));
    alanlar
}

pub(super) fn rota_arguman_semasi(alanlar: &[String]) -> bool {
    let secenek_basi = match alanlar.first().map(String::as_str) {
        Some("herkese_acik" | "oturumlu") => 1,
        Some("rol") if alanlar.get(1).is_some() => 2,
        _ => return false,
    };
    let Some(gerekli) = alanlar.last() else {
        return false;
    };
    bool_gecerli(gerekli)
        && secenek_basi < alanlar.len()
        && secenek_gecerli(&alanlar[secenek_basi..alanlar.len() - 1])
}

pub(super) fn sonuc_yaz(sonuc: &Result<(), WebReddi>) -> Vec<String> {
    match sonuc {
        Ok(()) => vec!["ok".into()],
        Err(hata) => vec!["ret".into(), hata.durum.to_string(), hata.mesaj.into()],
    }
}

fn ret_mesaji(mesaj: &str) -> Option<&'static str> {
    match mesaj {
        "IO adaptörü web güvenlik profilini desteklemiyor" => {
            Some("IO adaptörü web güvenlik profilini desteklemiyor")
        }
        "CSRF doğrulaması için geçerli form oturumu yok" => {
            Some("CSRF doğrulaması için geçerli form oturumu yok")
        }
        "CSRF belirteci eksik" => Some("CSRF belirteci eksik"),
        "CSRF belirteci geçersiz" => Some("CSRF belirteci geçersiz"),
        "bu rota kimliği doğrulanmış oturum istiyor" => {
            Some("bu rota kimliği doğrulanmış oturum istiyor")
        }
        "oturum bu rota için gerekli role sahip değil" => {
            Some("oturum bu rota için gerekli role sahip değil")
        }
        _ => None,
    }
}

pub(super) fn sonuc_coz(alanlar: &[String]) -> Result<Result<(), WebReddi>, String> {
    match alanlar {
        [etiket] if etiket == "ok" => Ok(Ok(())),
        [etiket, durum, mesaj] if etiket == "ret" => {
            let durum = durum
                .parse::<u16>()
                .ok()
                .filter(|deger| deger.to_string() == *durum)
                .ok_or_else(|| "ret durumu kanonik u16 değil".to_string())?;
            let mesaj =
                ret_mesaji(mesaj).ok_or_else(|| "bilinmeyen web reddi mesajı".to_string())?;
            Ok(Err(WebReddi { durum, mesaj }))
        }
        _ => Err("geçersiz web sonucu".into()),
    }
}
