//! K-391 (ADR-168): web isteği boyut ayarları. Hepsi isteğe bağlıdır;
//! bildirimsiz projede spec/11 §5 varsayılanları bayt bayt aynıdır.
//!
//! - `gövde_aşımı_rotası "/yol" olsun`: native adaptör gövde boyutu yüzünden
//!   reddettiği güvenli olmayan isteği ham 413 yerine bu GET rotasıyla
//!   yanıtlar (durum yine 413).
use super::*;

/// K-391 alan adları.
pub(super) const ALANLAR: [&str; 1] = ["gövde_aşımı_rotası"];

/// Rota yolu bayt tavanı.
const AZAMI_YOL: usize = 256;

const ORNEK: &str = "Örnek: gövde_aşımı_rotası \"/ileti/cok-buyuk\" olsun";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GovdeSiniriAyari {
    /// Gövde aşımında çalıştırılan GET rotasının yolu.
    pub asim_rotasi: Option<String>,
}

fn hata(mesaj: &str, satir: usize) -> Tani {
    proje_hatasi("P022", mesaj, satir, ORNEK)
}

/// Rota yolu `/` ile başlar; sorgu, parça, yüzde kaçışı, boşluk ve denetim
/// karakteri taşımaz (yorumlayıcıya istek satırı olarak doğrudan girer).
pub(super) fn yol_gecerli(yol: &str) -> bool {
    yol.starts_with('/')
        && yol.len() <= AZAMI_YOL
        && !yol
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || matches!(c, '?' | '#' | '%'))
}

pub(super) fn alanlari_al(
    alanlar: &mut HashMap<String, (Ifade, usize)>,
) -> Result<GovdeSiniriAyari, Tani> {
    let asim_rotasi = match alanlar.remove("gövde_aşımı_rotası") {
        None => None,
        Some((ifade, satir)) => match ifade.turu() {
            Ifade::MetinSabiti(yol) if yol_gecerli(yol) => Some(yol.clone()),
            _ => {
                return Err(hata(
                    "\"gövde_aşımı_rotası\" `/` ile başlayan, en çok 256 baytlık, sorgu/parça/boşluk içermeyen bir yol Metni olmalı.",
                    satir,
                ))
            }
        },
    };
    Ok(GovdeSiniriAyari { asim_rotasi })
}
