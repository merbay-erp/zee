//! Typed HIR'ın semantic araçlar için gezinme dizinleri.

use crate::kimlik::{IslemId, SymbolId, YapiId};

use super::{HirBagi, HirKaynakAraligi, HirProgram};

/// LSP ve diğer semantic araçların kaynak metin tahmini yapmadan tükettiği
/// yerel sembol kullanımı. Tanım/yeniden atama yazımları da bu akışa dahildir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HirSembolKullanimi {
    kimlik: SymbolId,
    kaynak_araligi: HirKaynakAraligi,
}

impl HirSembolKullanimi {
    pub(crate) const fn yeni(kimlik: SymbolId, kaynak_araligi: HirKaynakAraligi) -> Self {
        Self {
            kimlik,
            kaynak_araligi,
        }
    }

    pub const fn kimlik(self) -> SymbolId {
        self.kimlik
    }

    pub const fn kaynak_araligi(self) -> HirKaynakAraligi {
        self.kaynak_araligi
    }
}

impl HirProgram {
    pub fn sembol_tanimi(&self, kimlik: SymbolId) -> Option<HirKaynakAraligi> {
        self.sembol_tanimlari.get(&kimlik).copied()
    }

    /// Kaynak sırasındaki bütün okuma/yazma kullanımları. İfade HIR'ındaki
    /// okumalar ile cümle LHS yazımları tek semantic SymbolId akışında birleşir.
    pub fn sembol_kullanimlari(&self) -> Vec<HirSembolKullanimi> {
        let mut kullanimlar = self.sembol_yazimlari.clone();
        kullanimlar.extend(self.ifadeler.values().filter_map(|bilgi| {
            let HirBagi::Sembol(kimlik) = bilgi.bag() else {
                return None;
            };
            Some(HirSembolKullanimi::yeni(kimlik, bilgi.kaynak_araligi()))
        }));
        kullanimlar.sort_by_key(|kullanim| {
            let aralik = kullanim.kaynak_araligi();
            let kesin = aralik.kesin_konumu();
            (
                aralik.satiri(),
                kesin.map(|(_, sutun, _)| sutun).unwrap_or(0),
                kesin.map(|(_, _, uzunluk)| uzunluk).unwrap_or(0),
                kullanim.kimlik().kapsam(),
                kullanim.kimlik().sirasi(),
            )
        });
        kullanimlar.dedup();
        kullanimlar
    }

    pub fn islem_adi(&self, kimlik: IslemId) -> Option<&str> {
        self.islem_adlari.get(&kimlik).map(String::as_str)
    }

    pub fn islem_kimlikleri(&self) -> Vec<IslemId> {
        let mut kimlikler = self.islem_adlari.keys().copied().collect::<Vec<_>>();
        kimlikler.sort();
        kimlikler
    }

    pub fn yapi_kimlikleri(&self) -> Vec<YapiId> {
        let mut kimlikler = self.yapi_konumlari.keys().copied().collect::<Vec<_>>();
        kimlikler.sort();
        kimlikler
    }

    pub fn islem_kullanimlari(&self) -> Vec<(IslemId, HirKaynakAraligi)> {
        self.bag_kullanimlari(|bag| match bag {
            HirBagi::Islem(kimlik) => Some(kimlik),
            _ => None,
        })
    }

    pub fn yapi_kullanimlari(&self) -> Vec<(YapiId, HirKaynakAraligi)> {
        self.bag_kullanimlari(|bag| match bag {
            HirBagi::Yapi(kimlik) => Some(kimlik),
            _ => None,
        })
    }

    fn bag_kullanimlari<K: Copy + Ord>(
        &self,
        kimlik: impl Fn(HirBagi) -> Option<K>,
    ) -> Vec<(K, HirKaynakAraligi)> {
        let mut kullanimlar = self
            .ifadeler
            .values()
            .filter_map(|bilgi| Some((kimlik(bilgi.bag())?, bilgi.kaynak_araligi())))
            .collect::<Vec<_>>();
        kullanimlar.sort_by_key(|(kimlik, aralik)| {
            let kesin = aralik.kesin_konumu();
            (
                aralik.satiri(),
                kesin.map(|(_, sutun, _)| sutun).unwrap_or(0),
                *kimlik,
            )
        });
        kullanimlar.dedup();
        kullanimlar
    }
}
