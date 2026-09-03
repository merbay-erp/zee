//! Zee'yi Rust içinden kullanan uygulamalar için desteklenen facade.
//!
//! V1 sözleşmesi yalnız [`v1`] altındaki öğelerdir. Crate kökündeki eski
//! yollar bootstrap CLI ve geriye uyumluluk için görünür kalsa da desteklenen
//! SemVer yüzeyi değildir.

/// Zee V1 gömme sözleşmesi.
pub mod v1 {
    pub use crate::agac::Program;
    pub use crate::bicimleyici::bicimle;
    pub use crate::faz::BaglanmisProgram;
    pub use crate::tani::Tani;
    pub use crate::{
        birim_ozeti, gomulu_birim, gomulu_birim_adlari, kaynagi_calistir,
        kaynagi_calistir_girdiyle, kaynagi_dene, kaynagi_denetle, kaynagi_derle,
        kaynagi_derle_birimlerle, kaynagi_derle_kokenlerle, kaynagi_fazli_derle,
        kaynagi_fazli_derle_birimlerle, kaynagi_fazli_derle_kokenlerle, kaynagi_tanilari,
        kaynagi_tanilari_kokenlerle, programi_dene, programi_dene_baglanmis, BirimIstegi,
        BirimYukleyici, KokenliBirimYukleyici, TestSonucu, YuklenenBirim,
    };
}
