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

/// Bir sembol tablosundaki tanımı gösterir. Üst 32 bit kapsamı, alt 32 bit
/// o kapsamdaki tanım sırasını taşır; bu düzen depolama indeksi olarak
/// kullanılamaz.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolId(u64);

impl SymbolId {
    pub(crate) fn yeni(kapsam: usize, sira: usize) -> Self {
        assert!(kapsam <= u32::MAX as usize, "sembol kapsamı u32 sınırını aştı");
        assert!(sira <= u32::MAX as usize, "sembol sırası u32 sınırını aştı");
        Self(((kapsam as u64) << 32) | sira as u64)
    }

    pub const fn kapsam(self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub const fn sirasi(self) -> u32 {
        self.0 as u32
    }
}
