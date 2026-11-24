//! K-394 (RFC-0045, ADR-170): paylaşılan veri kökü bildirimi.
//!
//! `veri_kökü "UYGULAMA_DEPO" olsun` kökün mutlak yolunu taşıyan ortam
//! değişkeninin (ya da aynı adlı systemd credential'ın) adını bildirir; yol
//! kaynakta yazılmaz. Alan `veri-okuma` ya da `veri-yazma` yetkinliğinden en
//! az biriyle birlikte bulunur; bu yetkinlikler de alansız açılamaz (P023).
use super::*;

pub(super) fn alani_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    yetkinlik_yazimlari: &[String],
    yetkinlik_satiri: usize,
) -> Result<Option<String>, Tani> {
    let veri_yetkinligi = yetkinlik_yazimlari
        .iter()
        .any(|y| y == "veri-okuma" || y == "veri-yazma");
    let Some((ad, satir)) = istege_bagli_sabit_metni_al(alanlar, "veri_kökü")? else {
        if veri_yetkinligi {
            return Err(proje_hatasi(
                "P023",
                "`veri-okuma`/`veri-yazma` yetkinliği var ama `veri_kökü` bildirilmemiş.",
                yetkinlik_satiri,
                "Kökün mutlak yolunu taşıyan ortam değişkeni adını bildir; örnek: veri_kökü \"UYGULAMA_DEPO\" olsun",
            ));
        }
        return Ok(None);
    };
    let mut karakterler = ad.chars();
    if !karakterler
        .next()
        .is_some_and(|c| c.is_ascii_uppercase() || c == '_')
        || !karakterler.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        || ad.len() > 128
    {
        return Err(proje_hatasi(
            "P023",
            "`veri_kökü` ortam değişkeni adı A-Z, 0-9 ve _ biçiminde olmalı.",
            satir,
            "Yolu değil yalnız ortam değişkeninin adını yaz; örnek: UYGULAMA_DEPO. Mutlak yol işletim ortamında verilir.",
        ));
    }
    if !veri_yetkinligi {
        return Err(proje_hatasi(
            "P023",
            "`veri_kökü` var ama `veri-okuma` ya da `veri-yazma` yetkinliği açık değil.",
            satir,
            "Yalnız okuyacak hizmete \"veri-okuma\", yazacak hizmete ayrıca \"veri-yazma\" ekle.",
        ));
    }
    Ok(Some(ad))
}
