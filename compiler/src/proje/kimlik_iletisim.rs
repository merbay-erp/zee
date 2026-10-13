//! ürün E SMTP ve gizli anahtar bildirimleri yalnız ortam değişkeni adlarını taşır.
use super::*;
pub(super) fn alanlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    yetkinlik_yazimlari: &[String],
) -> Result<(Option<String>, Option<String>), Tani> {
    let eposta_baglanti_degiskeni = match istege_bagli_sabit_metni_al(
        alanlar,
        "e_posta_bağlantı_değişkeni",
    )? {
        None => None,
        Some((degisken, satir)) => {
            if !yetkinlik_yazimlari.iter().any(|y| y == "ağ") {
                return Err(proje_hatasi(
                        "P015",
                        "E-posta ayarı var ama `ağ` yetkinliği açık değil.",
                        satir,
                        "Yetkinliklere \"ağ\" ekle ve SMTP sunucusunu `ağ_hedefleri` içinde smtps://sunucu:465 olarak bildir.",
                    ));
            }
            let mut karakterler = degisken.chars();
            if !karakterler
                .next()
                .is_some_and(|c| c.is_ascii_uppercase() || c == '_')
                || !karakterler.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            {
                return Err(proje_hatasi(
                        "P015",
                        "E-posta bağlantı ortam değişkeni A-Z, 0-9 ve _ biçiminde olmalı.",
                        satir,
                        "Sır değerini değil yalnız ortam değişkeninin adını yaz; örnek: UYGULAMA_SMTP_URL.",
                    ));
            }
            Some(degisken)
        }
    };
    let gizli_anahtar_degiskeni = match istege_bagli_sabit_metni_al(
        alanlar,
        "gizli_anahtar_değişkeni",
    )? {
        None => None,
        Some((degisken, satir)) => {
            if !yetkinlik_yazimlari.iter().any(|y| y == "kriptografi") {
                return Err(proje_hatasi(
                    "P015",
                    "Gizli anahtar ayarı var ama `kriptografi` yetkinliği açık değil.",
                    satir,
                    "Yetkinliklere \"kriptografi\" ekle; anahtarı `dil gizli-anahtar` ile üretip ortam değişkenine koy.",
                ));
            }
            let mut karakterler = degisken.chars();
            if !karakterler
                .next()
                .is_some_and(|c| c.is_ascii_uppercase() || c == '_')
                || !karakterler.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            {
                return Err(proje_hatasi(
                    "P015",
                    "Gizli anahtar ortam değişkeni A-Z, 0-9 ve _ biçiminde olmalı.",
                    satir,
                    "Anahtarın kendisini değil yalnız ortam değişkeninin adını yaz; örnek: UYGULAMA_GIZLI_ANAHTAR.",
                ));
            }
            Some(degisken)
        }
    };
    Ok((eposta_baglanti_degiskeni, gizli_anahtar_degiskeni))
}
