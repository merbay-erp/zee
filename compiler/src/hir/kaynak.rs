use std::num::NonZeroUsize;

use crate::agac::Ifade;

/// Bir semantic düğümün kaynak metindeki zorunlu kökeni.
///
/// Ayrıştırıcının token konumunu koruduğu düğümlerde `Kesin`, bugünkü AST'nin
/// yalnız cümle satırını koruduğu düğümlerde `Satir` kullanılır. İkinci biçim
/// sahte bir sütun/uzunluk üretmek yerine belirsizliğini tipte açıkça taşır.
/// Her iki varyantta da satır sıfır olamaz; dolayısıyla HIR'da "konumsuz"
/// semantic düğüm kurulamaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirKaynakAraligi {
    Satir {
        satir: NonZeroUsize,
    },
    Kesin {
        satir: NonZeroUsize,
        sutun: NonZeroUsize,
        uzunluk: NonZeroUsize,
    },
}

impl HirKaynakAraligi {
    pub(crate) fn satir(satir: usize) -> Option<Self> {
        Some(Self::Satir {
            satir: NonZeroUsize::new(satir)?,
        })
    }

    pub(crate) fn kesin(satir: usize, sutun: usize, uzunluk: usize) -> Option<Self> {
        Some(Self::Kesin {
            satir: NonZeroUsize::new(satir)?,
            sutun: NonZeroUsize::new(sutun)?,
            uzunluk: NonZeroUsize::new(uzunluk)?,
        })
    }

    pub(crate) fn ifadeden(ifade: &Ifade, satir: usize) -> Option<Self> {
        match ifade {
            Ifade::Degisken {
                satir,
                sutun,
                uzunluk,
                ..
            } => Self::kesin(*satir, *sutun, *uzunluk),
            _ => Self::satir(satir),
        }
    }

    pub const fn satiri(self) -> usize {
        match self {
            Self::Satir { satir } | Self::Kesin { satir, .. } => satir.get(),
        }
    }

    pub const fn kesin_konumu(self) -> Option<(usize, usize, usize)> {
        match self {
            Self::Satir { .. } => None,
            Self::Kesin {
                satir,
                sutun,
                uzunluk,
            } => Some((satir.get(), sutun.get(), uzunluk.get())),
        }
    }
}
