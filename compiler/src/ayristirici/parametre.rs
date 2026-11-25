//! Parametre bağlama adı (K-398, ADR-175).

use super::ifade::yalin_ad;
use crate::morfoloji::{baglama_koku, kok_adaylari};

/// `yalin_ad` yalnız belirtme ekini atar, ünsüz yumuşaması ve ünlü düşmesini
/// geri çevirmez (`kalıbı`→`kalıb`). Parametre satırı bunun üstüne sözlük
/// kökünü (`kalıp`, `metin`) seçer; kök yalnız ekli yazımın morfoloji aday
/// kümesindeyse kabul edilir, değilse gövde korunur. Profil çıktısı değişmez;
/// eski yumuşamış yazım çözümleyicide aynı sembole çözülür.
pub(super) fn parametre_adi(ekli: &str) -> String {
    let govde = yalin_ad(ekli);
    match baglama_koku(&govde) {
        Some(kok) if kok_adaylari(ekli).contains(&kok) => kok,
        _ => govde,
    }
}
