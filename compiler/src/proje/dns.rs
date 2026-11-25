//! K-390 (RFC-0044 B1, ADR-169): `dns` yetkinliğinin çözücü bildirimi.
//! `dns_çözücü` yalnız loopback doğrulayıcı çözücüdür; `dns_dnssec`
//! varsayılansızdır. Üçü (yetkinlik + iki alan) birlikte bildirilir.
//! B1b: `kamu_son_ekleri` (proje köküne göreli PSL dosyası) ve
//! `kamu_son_ekleri_özeti` (exact SHA-256) yalnız `dns` ile, ikisi birlikte.
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

/// Proje köküne göreli PSL dosyası; host açılışta özeti doğrular (ADR-175).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KamuSonEkiBildirimi {
    pub yol: String,
    pub ozet: String,
}

pub(super) fn kamu_son_eklerini_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    dns_var: bool,
) -> Result<Option<KamuSonEkiBildirimi>, Tani> {
    let yol = opsiyonel_metni_al(alanlar, "kamu_son_ekleri", "P015")?;
    let ozet = opsiyonel_metni_al(alanlar, "kamu_son_ekleri_özeti", "P015")?;
    let ornek = "Örnek: kamu_son_ekleri \"veri/public_suffix_list.dat\" olsun · kamu_son_ekleri_özeti \"<64 küçük harf onaltılık>\" olsun";
    let (yol, ozet) = match (yol, ozet) {
        (None, None) => return Ok(None),
        (Some(yol), Some(ozet)) if dns_var => (yol, ozet),
        (Some((_, satir)), _) | (None, Some((_, satir))) => {
            let mesaj = if dns_var {
                "`kamu_son_ekleri` ve `kamu_son_ekleri_özeti` birlikte bildirilir."
            } else {
                "`kamu_son_ekleri` yalnız `dns` yetkinliği ve çözücü bildirimiyle kullanılabilir."
            };
            return Err(proje_hatasi("P015", mesaj, satir, ornek));
        }
    };
    let gecerli_yol = !yol.0.is_empty()
        && !yol.0.starts_with('/')
        && !yol.0.contains('\\')
        && std::path::Path::new(&yol.0)
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)));
    if !gecerli_yol {
        return Err(proje_hatasi(
            "P015",
            &format!(
                "`kamu_son_ekleri` \"{}\" proje köküne göreli bir dosya yolu değil.",
                yol.0
            ),
            yol.1,
            "`..`, mutlak yol ve ters bölü kullanma: \"veri/public_suffix_list.dat\".",
        ));
    }
    if ozet.0.len() != 64
        || !ozet
            .0
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(proje_hatasi(
            "P015",
            "`kamu_son_ekleri_özeti` 64 küçük harf onaltılık SHA-256 olmalı.",
            ozet.1,
            "Özeti `sha256sum veri/public_suffix_list.dat` ile al.",
        ));
    }
    Ok(Some(KamuSonEkiBildirimi {
        yol: yol.0,
        ozet: ozet.0,
    }))
}
