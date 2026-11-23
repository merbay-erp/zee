//! K-381 (RFC-0039 A2a): adlı dinleyici bağlantılarının kaynak sınırları.
//!
//! Hepsi isteğe bağlıdır; bildirimsiz varsayılanlar sıkıdır. Gevşetme yalnız
//! üst tavanın altında ve `dinleyiciler` bildirimiyle birlikte anlamlıdır.
use super::*;

/// Süreç geneli eşzamanlı bağlantı üst tavanı (`poll(2)` reaktörü için).
pub const BAGLANTI_SINIRI_UST: i64 = 4096;
pub const BAGLANTI_BOSTA_UST_SANIYE: i64 = 3600;
pub const BAGLANTI_OMRU_UST_SANIYE: i64 = 86_400;

pub use crate::ag_satiri::BaglantiAyari;

/// Bağlantı ayarı alan adları (dinleyici bildirimi şartı için).
pub(super) const ALANLAR: [&str; 9] = [
    "bağlantı_sınırı",
    "uzak_adres_bağlantı_sınırı",
    "bağlantı_satır_sınırı",
    "bağlantı_boşta_süresi",
    "bağlantı_ömrü",
    "bağlantı_dolu_yanıtı",
    "tls_dinleyicileri",
    "tls_sertifika_değişkeni",
    "tls_anahtar_değişkeni",
];

const ORNEK: &str = "Örnek: bağlantı_sınırı 256 olsun · bağlantı_boşta_süresi 300 olsun · bağlantı_dolu_yanıtı \"421 4.3.2 Hizmet meşgul\" olsun";

fn sayi_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    ad: &str,
    alt: i64,
    ust: i64,
    varsayilan: i64,
) -> Result<(i64, bool), Tani> {
    let Some((ifade, satir)) = alanlar.remove(ad) else {
        return Ok((varsayilan, false));
    };
    match ifade.turu() {
        Ifade::SayiSabiti(deger) if (alt..=ust).contains(deger) => Ok((*deger, true)),
        Ifade::SayiSabiti(deger) => Err(proje_hatasi(
            "P020",
            &format!("\"{ad}\" {deger}; {alt} ile {ust} arasında olmalı."),
            satir,
            ORNEK,
        )),
        _ => Err(proje_hatasi(
            "P020",
            &format!("\"{ad}\" alanı sabit Sayı olmalı."),
            satir,
            ORNEK,
        )),
    }
}

fn dolu_yanitini_al(alanlar: &mut HashMap<String, (Ifade, usize)>) -> Result<Option<String>, Tani> {
    Ok(match alanlar.remove("bağlantı_dolu_yanıtı") {
        None => None,
        Some((ifade, satir)) => match ifade.turu() {
            Ifade::MetinSabiti(metin) if crate::ag_satiri::dolu_yaniti_gecerli(metin) => {
                Some(metin.clone())
            }
            _ => {
                return Err(proje_hatasi(
                    "P020",
                    "\"bağlantı_dolu_yanıtı\" en çok 512 baytlık, CR/LF içermeyen yazdırılabilir ASCII Metin olmalı.",
                    satir,
                    ORNEK,
                ))
            }
        },
    })
}

pub(super) fn alanlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
    dinleyici_adlari: &[&str],
) -> Result<BaglantiAyari, Tani> {
    let dinleyici_var = !dinleyici_adlari.is_empty();
    let ilk_satir = ALANLAR
        .iter()
        .filter_map(|ad| alanlar.get(*ad).map(|(_, satir)| *satir))
        .min();
    if let (Some(satir), false) = (ilk_satir, dinleyici_var) {
        return Err(proje_hatasi(
            "P020",
            "Bağlantı ayarı var ama `dinleyiciler` bildirimi yok.",
            satir,
            "Önce devralınacak dinleyicileri bildir: dinleyiciler \"smtp_v4 0.0.0.0:25\" listesi olsun",
        ));
    }
    let varsayilan = BaglantiAyari::default();
    let (sinir, _) = sayi_al(
        alanlar,
        "bağlantı_sınırı",
        1,
        BAGLANTI_SINIRI_UST,
        varsayilan.sinir as i64,
    )?;
    let uzak_satiri = alanlar.get("uzak_adres_bağlantı_sınırı").map(|(_, s)| *s);
    let (uzak, uzak_yazildi) = sayi_al(
        alanlar,
        "uzak_adres_bağlantı_sınırı",
        1,
        BAGLANTI_SINIRI_UST,
        varsayilan.uzak_adres_siniri as i64,
    )?;
    if uzak > sinir && uzak_yazildi {
        return Err(proje_hatasi(
                "P020",
                &format!(
                    "\"uzak_adres_bağlantı_sınırı\" {uzak}; \"bağlantı_sınırı\" ({sinir}) değerini aşamaz."
                ),
                uzak_satiri.unwrap_or(1),
                ORNEK,
            ));
    }
    let uzak = uzak.min(sinir);
    let (satir_siniri, _) = sayi_al(
        alanlar,
        "bağlantı_satır_sınırı",
        crate::ag_satiri::SATIR_SINIRI_ALT as i64,
        crate::ag_satiri::SATIR_SINIRI_UST as i64,
        varsayilan.satir_siniri as i64,
    )?;
    let (bosta, _) = sayi_al(
        alanlar,
        "bağlantı_boşta_süresi",
        1,
        BAGLANTI_BOSTA_UST_SANIYE,
        varsayilan.bosta_suresi_ms / 1000,
    )?;
    let (omur, _) = sayi_al(
        alanlar,
        "bağlantı_ömrü",
        1,
        BAGLANTI_OMRU_UST_SANIYE,
        varsayilan.omur_ms / 1000,
    )?;
    let dolu_yaniti = dolu_yanitini_al(alanlar)?;
    let tls = super::baglanti_tls::alanlari_al(alanlar, dinleyici_adlari)?;
    Ok(BaglantiAyari {
        sinir: sinir as usize,
        uzak_adres_siniri: uzak as usize,
        satir_siniri: satir_siniri as usize,
        bosta_suresi_ms: bosta * 1000,
        omur_ms: omur * 1000,
        dolu_yaniti,
        tls,
    })
}
