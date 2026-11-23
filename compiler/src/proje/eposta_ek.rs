//! K-378 (RFC-0041 katman C): SMTP ek türü kümesi bildirimi.
//!
//! `e_posta_ek_türleri "pdf", "docx" listesi olsun` kümeyi tam olarak
//! belirler (varsayılan dört tür örtük eklenmez); yalnız `icerik_turu`
//! kaydının SMTP adayları yazılabilir. Alan yoksa `zee-smtp-mime-1`'in dört
//! türü geçerlidir.
use super::*;

pub(super) fn alani_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    yetkinlik_yazimlari: &[String],
) -> Result<Option<Vec<String>>, Tani> {
    if !alanlar.contains_key("e_posta_ek_türleri") {
        return Ok(None);
    }
    let (adlar, satir) = metin_listesini_al(alanlar, "e_posta_ek_türleri", "P019")?;
    if !yetkinlik_yazimlari.iter().any(|y| y == "ağ") {
        return Err(proje_hatasi(
            "P015",
            "E-posta ek türü ayarı var ama `ağ` yetkinliği açık değil.",
            satir,
            "Yetkinliklere \"ağ\" ekle ve SMTP sunucusunu `ağ_hedefleri` içinde bildir.",
        ));
    }
    let hata = |neden: String| {
        proje_hatasi(
            "P019",
            &format!("Geçersiz `e_posta_ek_türleri`: {neden}."),
            satir,
            "Geçerli türler: pdf, png, jpeg, webp, txt, csv, ics, docx, xlsx, pptx, zip. Liste kümeyi tam belirler; varsayılanları da yaz.",
        )
    };
    if adlar.is_empty() {
        return Err(hata("ek türü listesi boş olamaz".into()));
    }
    for (sira, ad) in adlar.iter().enumerate() {
        if !crate::icerik_turu::adla(ad).is_some_and(|tur| tur.smtp_aday) {
            return Err(hata(format!(
                "\"{ad}\" SMTP ek türü kaydında yok (makro, yürütülebilir ve betik türleri açılamaz)"
            )));
        }
        if adlar[..sira].contains(ad) {
            return Err(hata(format!("\"{ad}\" ek türü birden çok kez yazıldı")));
        }
    }
    Ok(Some(adlar))
}
