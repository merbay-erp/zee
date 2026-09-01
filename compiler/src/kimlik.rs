//! Derleyici içi semantik kimlikler.
//!
//! Bu türler depolama konumunu, kaynak adını ve semantic identity'yi birbirine
//! karıştırmayı engeller. Kimlikler bir derleme birimi içinde kararlı ve tür
//! güvenlidir; kalıcı paket/ABI kimliği değildir.

macro_rules! sirali_kimlik {
    ($ad:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $ad(usize);

        impl $ad {
            pub(crate) const fn yeni(sira: usize) -> Self {
                Self(sira)
            }

            pub const fn sirasi(self) -> usize {
                self.0
            }
        }
    };
}

sirali_kimlik!(YapiId);
sirali_kimlik!(IslemId);

/// Bir sembol tablosundaki tanımı gösterir. Kapsam ve o kapsamdaki tanım
/// sırası ayrı bileşenlerdir; bu düzen depolama indeksi olarak kullanılamaz
/// ve kaynak büyüklüğüne yapay 32-bit panic sınırı koymaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolId {
    kapsam: usize,
    sira: usize,
}

impl SymbolId {
    pub(crate) const fn yeni(kapsam: usize, sira: usize) -> Self {
        Self { kapsam, sira }
    }

    pub const fn kapsam(self) -> usize {
        self.kapsam
    }

    pub const fn sirasi(self) -> usize {
        self.sira
    }
}
