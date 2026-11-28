//! RFC-0047 (ADR-182 dilim C, K-364): DAV istemci rotası proje bildirimi.
//!
//! - `dav_kimlik_çözücüsü "kimlik_coz" olsun`: Basic kullanıcı adı ve
//!   uygulama parolası önekiyle çağrılan salt okuma `işlem`in adı (soru 1).
//!   Programdaki varlığı ve salt okuması derlemede T067'dir.
//! - `dav_kökeni "https://dav.ornek.test" olsun`: üretim profilinde
//!   (`--web-proxy`) DAV rotalarının sunulduğu ayrı HTTPS origin (soru 8).
//!
//! İkisi de yalnız `kriptografi` ve dolu `uygulama_parolası_anahtarları` ile
//! geçerlidir; aksi P015.
use super::*;

const ORNEK: &str = "Örnek: dav_kimlik_çözücüsü \"kimlik_coz\" olsun ve dav_kökeni \"https://dav.ornek.test\" olsun";

/// DAV istemci rotası bildirimi.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DavBildirimi {
    pub kimlik_cozucusu: Option<String>,
    /// Kanonik `https://ana[:kapı]` yazımı.
    pub koken: Option<String>,
}

pub(super) fn alanlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    yetkinlik_yazimlari: &[String],
    anahtar_var: bool,
) -> Result<DavBildirimi, Tani> {
    let cozucu = istege_bagli_sabit_metni_al(alanlar, "dav_kimlik_çözücüsü")?;
    let koken = istege_bagli_sabit_metni_al(alanlar, "dav_kökeni")?;
    let Some(satir) = cozucu.iter().chain(koken.iter()).map(|(_, s)| *s).min() else {
        return Ok(DavBildirimi::default());
    };
    let hata = |mesaj: &str| proje_hatasi("P015", mesaj, satir, ORNEK);
    if !yetkinlik_yazimlari.iter().any(|y| y == "kriptografi") || !anahtar_var {
        return Err(hata(
            "DAV bildirimi `kriptografi` yetkinliği ve dolu `uygulama_parolası_anahtarları` ister.",
        ));
    }
    if let Some((ad, _)) = &cozucu {
        if ad.is_empty() || ad.chars().any(|c| c.is_control() || c == '"') {
            return Err(hata("`dav_kimlik_çözücüsü` bir işlem adı olmalı."));
        }
    }
    let koken = match koken {
        Some((yazim, _)) => {
            let hedef = crate::yetkinlik::AgHedefi::https_origininden(&yazim).map_err(|neden| {
                hata(&format!(
                    "`dav_kökeni` \"{yazim}\" yolsuz, kimliksiz bir HTTPS origin değil: {neden}."
                ))
            })?;
            Some(hedef.yazimi())
        }
        None => None,
    };
    Ok(DavBildirimi {
        kimlik_cozucusu: cozucu.map(|(ad, _)| ad),
        koken,
    })
}
