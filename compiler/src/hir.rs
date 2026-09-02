//! Tür denetiminden sonra üretilen semantik ara gösterim (HIR).
//!
//! Kaynak AST, tanı ve geriye uyum için korunur. Çalıştırma ve sonraki
//! derleyici geçişleri ise ad metinlerini yeniden çözmek yerine buradaki açık
//! tür ve semantic bağ kayıtlarını tüketir.

use std::collections::HashMap;

use crate::agac::{Ifade, Islem, Program, Yapi};
use crate::kimlik::{IslemId, SymbolId, YapiId};
use crate::semantic_model::Tur;

mod gezinme;
mod kaynak;
#[cfg(test)]
mod testler;

pub use gezinme::HirSembolKullanimi;
pub use kaynak::HirKaynakAraligi;

/// Bir ifadenin kaynak yazımından bağımsız semantic bağı.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirBagi {
    Yok,
    Sembol(SymbolId),
    Islem(IslemId),
    Yapi(YapiId),
}

/// Bir HIR ifadesinin değer üretme sözleşmesi. Çağrı cümlesi gibi ifadeler
/// semantic düğümdür ama değer konumunda değildir; bu durum tür eksikliği
/// yerine açık `DegerDondurmez` olarak taşınır.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HirIfadeTuru {
    Deger(Tur),
    DegerDondurmez,
}

/// Bir HIR programı içindeki semantic ifade düğümünün kararlı kimliği.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HirDugumId(usize);

impl HirDugumId {
    pub(crate) const fn yeni(sira: usize) -> Self {
        Self(sira)
    }

    pub const fn sirasi(self) -> usize {
        self.0
    }
}

/// Checker'ın tek bir ifade için kanıtladığı HIR bilgisi.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HirIfadeBilgisi {
    kimlik: HirDugumId,
    tur: HirIfadeTuru,
    bag: HirBagi,
    kaynak_araligi: HirKaynakAraligi,
}

impl HirIfadeBilgisi {
    pub(crate) const fn yeni_turle(
        kimlik: HirDugumId,
        tur: HirIfadeTuru,
        bag: HirBagi,
        kaynak_araligi: HirKaynakAraligi,
    ) -> Self {
        Self {
            kimlik,
            tur,
            bag,
            kaynak_araligi,
        }
    }

    pub const fn kimlik(self) -> HirDugumId {
        self.kimlik
    }

    pub const fn tur(self) -> HirIfadeTuru {
        self.tur
    }

    pub const fn bag(self) -> HirBagi {
        self.bag
    }

    pub const fn kaynak_araligi(self) -> HirKaynakAraligi {
        self.kaynak_araligi
    }
}

/// Checker ile HIR lowering arasındaki crate-içi aktarım paketi.
pub(crate) struct HirOlusturmaBilgisi {
    pub(crate) ifadeler: HashMap<usize, HirIfadeBilgisi>,
    pub(crate) sembol_adlari: HashMap<SymbolId, String>,
    pub(crate) sembol_tanimlari: HashMap<SymbolId, HirKaynakAraligi>,
    pub(crate) sembol_yazimlari: Vec<HirSembolKullanimi>,
    pub(crate) islem_adlari: HashMap<IslemId, String>,
    pub(crate) yapi_konumlari: HashMap<YapiId, usize>,
}

/// Türleri ve çözülmüş bağları kaynak AST'den ayrı taşıyan program HIR'ı.
///
/// AST kutuda tutulur; böylece checker sırasında kaydedilen düğüm adresleri
/// HIR'ın ömrü boyunca kararlı kalır. Bu adresler public identity değildir;
/// dışarıya yalnız güvenli `ifade_bilgisi` sorgusu açılır.
pub struct HirProgram {
    program: Box<Program>,
    ifadeler: HashMap<usize, HirIfadeBilgisi>,
    sembol_adlari: HashMap<SymbolId, String>,
    sembol_tanimlari: HashMap<SymbolId, HirKaynakAraligi>,
    sembol_yazimlari: Vec<HirSembolKullanimi>,
    islem_adlari: HashMap<IslemId, String>,
    yapi_konumlari: HashMap<YapiId, usize>,
}

impl HirProgram {
    pub(crate) fn yeni(program: Program, bilgi: HirOlusturmaBilgisi) -> Self {
        Self {
            program: Box::new(program),
            ifadeler: bilgi.ifadeler,
            sembol_adlari: bilgi.sembol_adlari,
            sembol_tanimlari: bilgi.sembol_tanimlari,
            sembol_yazimlari: bilgi.sembol_yazimlari,
            islem_adlari: bilgi.islem_adlari,
            yapi_konumlari: bilgi.yapi_konumlari,
        }
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn ifade_bilgisi(&self, ifade: &Ifade) -> Option<HirIfadeBilgisi> {
        self.ifadeler.get(&ifade_adresi(ifade)).copied()
    }

    pub fn sembol_adi(&self, kimlik: SymbolId) -> Option<&str> {
        self.sembol_adlari.get(&kimlik).map(String::as_str)
    }

    pub fn islem(&self, kimlik: IslemId) -> Option<&Islem> {
        let ad = self.islem_adlari.get(&kimlik)?;
        self.program.islemler.get(ad)
    }

    pub fn yapi(&self, kimlik: YapiId) -> Option<&Yapi> {
        let konum = *self.yapi_konumlari.get(&kimlik)?;
        self.program.yapilar.get(konum)
    }

    pub(crate) fn ifade_sayisi(&self) -> usize {
        self.ifadeler.len()
    }

    pub(crate) fn into_program(self) -> Program {
        *self.program
    }

    #[cfg(test)]
    pub(crate) fn program_mut(&mut self) -> &mut Program {
        &mut self.program
    }
}

pub(crate) fn ifade_adresi(ifade: &Ifade) -> usize {
    ifade as *const Ifade as usize
}
