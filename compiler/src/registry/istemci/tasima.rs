//! Registry statik yerleşiminin güvenli HTTPS taşıyıcısı.

use super::super::RegistryHatasi;
use crate::yetkinlik::{AgHedefi, AgSemasi, Yetkinlik, YetkinlikPolitikasi};
use std::collections::BTreeSet;

/// Statik registry yerleşiminden sınırlı bir nesne getirir. `None` yalnız 404
/// anlamına gelir; diğer taşıma hataları görünür P016 olmalıdır.
pub trait RegistryTasiyici {
    fn getir(
        &mut self,
        goreli_yol: &str,
        azami_bayt: usize,
    ) -> Result<Option<Vec<u8>>, RegistryHatasi>;
}

/// Yalnız HTTPS origin kabul eden, redirect/proxy kullanmayan ve DNS sonrası
/// public-IP korkuluklarını ortak ağ istemcisinden alan gerçek taşıyıcı.
pub struct HttpsRegistryTasiyici {
    origin: String,
    politika: YetkinlikPolitikasi,
}

impl HttpsRegistryTasiyici {
    pub fn yeni(origin: &str) -> Result<Self, RegistryHatasi> {
        let hedef = AgHedefi::bildirimden(origin).map_err(|neden| {
            RegistryHatasi::tasima(format!("Registry origin'i geçersiz: {}.", neden))
        })?;
        if hedef.semasi() != AgSemasi::Https {
            return Err(RegistryHatasi::tasima(
                "Registry taşıması yalnız https:// origin kabul eder.",
            ));
        }
        let origin = hedef.yazimi();
        let politika =
            YetkinlikPolitikasi::proje(BTreeSet::from([Yetkinlik::Ag]), BTreeSet::from([hedef]));
        Ok(Self { origin, politika })
    }
}

impl RegistryTasiyici for HttpsRegistryTasiyici {
    fn getir(
        &mut self,
        goreli_yol: &str,
        azami_bayt: usize,
    ) -> Result<Option<Vec<u8>>, RegistryHatasi> {
        goreli_yolu_dogrula(goreli_yol)?;
        let url = format!("{}/{}", self.origin, goreli_yol);
        let (durum, govde) =
            crate::ag_istemcisi::getir_bayt(&url, None, &self.politika, azami_bayt).map_err(
                |neden| RegistryHatasi::tasima(format!("{} alınamadı: {}.", goreli_yol, neden)),
            )?;
        match durum {
            200 => Ok(Some(govde)),
            404 => Ok(None),
            _ => Err(RegistryHatasi::tasima(format!(
                "Registry {} için HTTP {} döndürdü.",
                goreli_yol, durum
            ))),
        }
    }
}

pub(super) fn goreli_yolu_dogrula(yol: &str) -> Result<(), RegistryHatasi> {
    if yol.is_empty()
        || !yol.is_ascii()
        || yol.starts_with('/')
        || yol.contains('\\')
        || yol.split('/').any(|parca| {
            parca.is_empty()
                || matches!(parca, "." | "..")
                || !parca
                    .bytes()
                    .all(|bayt| bayt.is_ascii_alphanumeric() || matches!(bayt, b'.' | b'-' | b'_'))
        })
    {
        return Err(RegistryHatasi::tasima(
            "Registry statik yolu güvenli göreli ASCII bileşenlerden oluşmalı.",
        ));
    }
    Ok(())
}
