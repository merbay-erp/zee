use super::*;

pub(super) fn gizli_adlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    yetkinlikler: &[String],
) -> Result<Vec<String>, Tani> {
    let (adlar, satir) = metin_listesini_al(alanlar, "http_gizli_değişkenleri", "P015")?;
    let mut gorulen = BTreeSet::new();
    if adlar.len() > 16 || (!adlar.is_empty() && !yetkinlikler.iter().any(|y| y == "ağ")) {
        return Err(proje_hatasi(
            "P015",
            "HTTP gizli ortam listesi ağ yetkinliği ve en fazla 16 ad ister.",
            satir,
            "Yalnız gerekli ortam adlarını açıkça listele; sır değerlerini kaynağa yazma.",
        ));
    }
    for ad in &adlar {
        let mut baytlar = ad.bytes();
        if ad.len() > 64
            || !baytlar
                .next()
                .is_some_and(|b| b.is_ascii_uppercase() || b == b'_')
            || !baytlar.all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
            || !gorulen.insert(ad.clone())
        {
            return Err(proje_hatasi(
                "P015",
                "HTTP gizli ortam adları tekil A-Z, 0-9 ve _ biçiminde olmalı.",
                satir,
                "Örnek: http_gizli_değişkenleri \"TURNSTILE_SECRET\" listesi olsun",
            ));
        }
    }
    Ok(adlar)
}
