//! K-330: web oturumu ömrü ve isteğe bağlı kayar yenileme bildirimi.
use super::*;

/// Anahtar verilmediğinde kimlikli oturum ömrü; `kaynak_sinirlari` web varsayılanıyla aynıdır.
pub const OTURUM_OMRU_VARSAYILAN_SANIYE: i64 = 30 * 60;
/// Kimlikli oturum ömrünün alt sınırı (5 dakika).
pub const OTURUM_OMRU_ALT_SANIYE: i64 = 5 * 60;
/// Kimlikli oturum ömrünün üst sınırı (30 gün); kayar yenilemenin mutlak tavanıdır.
pub const OTURUM_OMRU_UST_SANIYE: i64 = 30 * 24 * 60 * 60;

/// Proje sahibinin kimlikli web oturumu için seçtiği ömür ve yenileme biçimi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebOturumuBildirimi {
    pub omur_saniye: i64,
    /// Açıkken her kimlikli istek geçerliliği `şimdi + ömür`e taşır.
    pub kayar: bool,
}

pub(super) fn alanlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    yetkinlik_yazimlari: &[String],
) -> Result<Option<WebOturumuBildirimi>, Tani> {
    let omur = alanlar.remove("oturum_ömrü_saniye");
    let kayar = alanlar.remove("oturum_kayar");
    if omur.is_none() && kayar.is_none() {
        return Ok(None);
    }
    let satir = omur
        .as_ref()
        .or(kayar.as_ref())
        .map(|(_, satir)| *satir)
        .unwrap_or(1);
    if !yetkinlik_yazimlari.iter().any(|y| y == "web-oturumu") {
        return Err(proje_hatasi(
            "P015",
            "Oturum ömrü ayarı var ama `web-oturumu` yetkinliği açık değil.",
            satir,
            "Yetkinliklere \"web-oturumu\" ve \"ağ-sunucusu\" ekle ya da oturum ayarlarını kaldır.",
        ));
    }
    let omur_saniye = match omur {
        None => OTURUM_OMRU_VARSAYILAN_SANIYE,
        Some((ifade, satir)) => match ifade.turu() {
            Ifade::SayiSabiti(deger)
                if (OTURUM_OMRU_ALT_SANIYE..=OTURUM_OMRU_UST_SANIYE).contains(deger) =>
            {
                *deger
            }
            Ifade::SayiSabiti(deger) => {
                let mesaj = format!(
                    "Oturum ömrü {deger} saniye; {OTURUM_OMRU_ALT_SANIYE} ile \
                     {OTURUM_OMRU_UST_SANIYE} saniye (5 dakika – 30 gün) arasında olmalı."
                );
                return Err(proje_hatasi(
                    "P015",
                    &mesaj,
                    satir,
                    "Örnek: oturum_ömrü_saniye 28800 olsun (8 saat)",
                ));
            }
            _ => {
                return Err(proje_hatasi(
                    "P001",
                    "\"oturum_ömrü_saniye\" alanı sabit Sayı olmalı.",
                    satir,
                    "Örnek: oturum_ömrü_saniye 28800 olsun",
                ))
            }
        },
    };
    let kayar = match kayar {
        None => false,
        Some((ifade, satir)) => match ifade.turu() {
            Ifade::MantiksalSabiti(deger) => *deger,
            _ => {
                return Err(proje_hatasi(
                    "P001",
                    "\"oturum_kayar\" alanı doğru ya da yanlış olmalı.",
                    satir,
                    "Örnek: oturum_kayar doğru olsun",
                ))
            }
        },
    };
    Ok(Some(WebOturumuBildirimi { omur_saniye, kayar }))
}
