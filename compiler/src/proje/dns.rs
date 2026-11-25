//! K-390 (RFC-0044 B1, ADR-169): `dns` yetkinliğinin çözücü bildirimi.
//! `dns_çözücü` yalnız loopback doğrulayıcı çözücüdür; `dns_dnssec`
//! varsayılansızdır. Üçü (yetkinlik + iki alan) birlikte bildirilir.
use super::*;
use crate::dns_teli::{DnsAyari, DnssecKipi};

pub(super) fn alanlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    yetkinlik_yazimlari: &[String],
    yetkinlik_satiri: usize,
) -> Result<Option<DnsAyari>, Tani> {
    let cozucu = opsiyonel_metni_al(alanlar, "dns_çözücü", "P015")?;
    let kip = opsiyonel_metni_al(alanlar, "dns_dnssec", "P015")?;
    let yetkinlik = yetkinlik_yazimlari.iter().any(|y| y == "dns");
    let (cozucu, kip) = match (yetkinlik, cozucu, kip) {
        (false, None, None) => return Ok(None),
        (true, Some(cozucu), Some(kip)) => (cozucu, kip),
        (false, Some((_, satir)), _) | (false, None, Some((_, satir))) => {
            return Err(proje_hatasi(
                "P015",
                "`dns_çözücü`/`dns_dnssec` yalnız `dns` yetkinliğiyle bildirilebilir.",
                satir,
                "Yetkinliklere \"dns\" ekle ya da DNS alanlarını kaldır.",
            ))
        }
        (true, cozucu, kip) => {
            let satir = cozucu
                .map(|(_, satir)| satir)
                .or(kip.map(|(_, satir)| satir))
                .unwrap_or(yetkinlik_satiri);
            return Err(proje_hatasi(
                "P015",
                "`dns` yetkinliği `dns_çözücü` ve `dns_dnssec` bildirimlerinin ikisini de ister.",
                satir,
                "Örnek: dns_çözücü \"127.0.0.1:53\" olsun · dns_dnssec \"zorunlu\" olsun",
            ));
        }
    };
    let Some(dnssec) = DnssecKipi::ayristir(&kip.0) else {
        return Err(proje_hatasi(
            "P015",
            &format!("\"{}\" geçerli bir `dns_dnssec` kipi değil.", kip.0),
            kip.1,
            "`zorunlu` (açılışta AD=1 sınanır, DANE için gerekli) ya da `bilgi` yaz.",
        ));
    };
    DnsAyari::yeni(&cozucu.0, dnssec)
        .map(Some)
        .map_err(|neden| {
            proje_hatasi(
                "P015",
                &format!("Geçersiz `dns_çözücü` \"{}\": {}.", cozucu.0, neden),
                cozucu.1,
                "Aynı makinedeki doğrulayıcı çözücüyü yaz: \"127.0.0.1:53\" ya da \"[::1]:53\".",
            )
        })
}
