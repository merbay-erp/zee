//! K-381 (RFC-0039 A2b): sunucu TLS kimliği ve implicit TLS dinleyicileri.
//!
//! Sertifika zinciri ve özel anahtar kaynakta ya da proje dosyasında değil,
//! K-339 modeliyle adlandırılmış credential/ortam değişkenindedir; bildirim
//! yalnız adları taşır. Bozuk bildirim P021'dir.
use super::*;
use crate::ag_satiri::TlsBildirimi;

const ORNEK: &str = "Örnek: tls_sertifika_değişkeni \"POSTA_TLS_ZINCIRI\" olsun · tls_anahtar_değişkeni \"POSTA_TLS_ANAHTARI\" olsun · tls_dinleyicileri \"smtps_v4\" listesi olsun";

/// `gizli_anahtar_değişkeni` ile aynı ad kuralı (K-339 credential kimliğiyle uyumlu).
fn gecerli_ad(ad: &str) -> bool {
    let mut karakterler = ad.chars();
    ad.len() <= 255
        && karakterler
            .next()
            .is_some_and(|c| c.is_ascii_uppercase() || c == '_')
        && karakterler.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

fn ad_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    alan: &str,
) -> Result<Option<String>, Tani> {
    let Some((ifade, satir)) = alanlar.remove(alan) else {
        return Ok(None);
    };
    match ifade.turu() {
        Ifade::MetinSabiti(ad) if gecerli_ad(ad) => Ok(Some(ad.clone())),
        _ => Err(proje_hatasi(
            "P021",
            &format!("\"{alan}\" credential/ortam değişkeni adı olmalı (A-Z, 0-9 ve _; rakamla başlamaz)."),
            satir,
            ORNEK,
        )),
    }
}

pub(super) fn alanlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    dinleyici_adlari: &[&str],
) -> Result<Option<TlsBildirimi>, Tani> {
    let tls_satiri = alanlar.get("tls_dinleyicileri").map(|(_, s)| *s);
    let (implicit, liste_satiri) = metin_listesini_al(alanlar, "tls_dinleyicileri", "P021")?;
    let anahtar_satiri = alanlar
        .get("tls_sertifika_değişkeni")
        .or(alanlar.get("tls_anahtar_değişkeni"))
        .map(|(_, s)| *s);
    let sertifika = ad_al(alanlar, "tls_sertifika_değişkeni")?;
    let anahtar = ad_al(alanlar, "tls_anahtar_değişkeni")?;
    let (sertifika_degiskeni, anahtar_degiskeni) = match (sertifika, anahtar) {
        (None, None) if tls_satiri.is_none() => return Ok(None),
        (Some(s), Some(a)) if s != a => (s, a),
        _ => {
            return Err(proje_hatasi(
                "P021",
                "TLS için `tls_sertifika_değişkeni` ve `tls_anahtar_değişkeni` birlikte ve farklı adlarla bildirilmeli.",
                anahtar_satiri.or(tls_satiri).unwrap_or(1),
                ORNEK,
            ))
        }
    };
    for (sira, ad) in implicit.iter().enumerate() {
        if !dinleyici_adlari.contains(&ad.as_str()) || implicit[..sira].contains(ad) {
            return Err(proje_hatasi(
                "P021",
                &format!(
                    "\"{ad}\" `dinleyiciler` bildiriminde olmayan ya da yinelenen TLS dinleyicisi."
                ),
                liste_satiri,
                ORNEK,
            ));
        }
    }
    Ok(Some(TlsBildirimi {
        dinleyiciler: implicit,
        sertifika_degiskeni,
        anahtar_degiskeni,
    }))
}
