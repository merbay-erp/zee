//! Derleyicinin fiziksel veri fazları.
//!
//! ```compile_fail
//! use dil::faz::KaynakMetni;
//! use dil::yorumlayici::{calistir_baglanmis_io, ToplayanIo};
//!
//! let tokenlar = KaynakMetni::yeni("\"merhaba\" yaz\n").sozcukle().unwrap();
//! let ast = tokenlar.ayristir(Vec::new()).unwrap();
//! let mut io = ToplayanIo::yeni(Vec::new());
//! // Parsed AST doğrudan yürütülemez; önce hoist + checker bağı gerekir.
//! calistir_baglanmis_io(&ast, &mut io).unwrap();
//! ```

use std::ops::Deref;

use crate::agac::{Cumle, Program};
use crate::hir::HirProgram;
use crate::sozcukleyici::Token;
use crate::tani::Tani;

/// Henüz sözcüklenmemiş UTF-8 zee kaynağı.
#[derive(Debug, Clone, Copy)]
pub struct KaynakMetni<'a> {
    metin: &'a str,
}

impl<'a> KaynakMetni<'a> {
    pub const fn yeni(metin: &'a str) -> Self {
        Self { metin }
    }

    pub fn sozcukle(self) -> Result<TokenAkisi, Tani> {
        crate::sozcukleyici::sozcukle(self.metin).map(TokenAkisi::yeni)
    }
}

/// Lexer'dan çıkmış, parser'a girmeye hazır token akışı.
#[derive(Debug, Clone)]
pub struct TokenAkisi {
    tokenlar: Vec<Token>,
}

impl TokenAkisi {
    pub(crate) fn yeni(tokenlar: Vec<Token>) -> Self {
        Self { tokenlar }
    }

    pub fn tokenlar(&self) -> &[Token] {
        &self.tokenlar
    }

    pub fn ayristir(self, islem_adlari: Vec<String>) -> Result<AyristirilmisAst, Tani> {
        let cumleler = crate::ayristirici::ayristir_tohumla(self.tokenlar, islem_adlari)?;
        let ast = AyristirilmisAst::yeni(cumleler);
        #[cfg(debug_assertions)]
        ast.invariantleri_dogrula().map_err(|hata| hata.tani())?;
        Ok(ast)
    }

    pub fn ayristir_kurtarmali(
        self,
        islem_adlari: Vec<String>,
    ) -> (AyristirilmisAst, Vec<Tani>) {
        let (cumleler, tanilar) =
            crate::ayristirici::ayristir_kurtarmali(self.tokenlar, islem_adlari);
        let ast = AyristirilmisAst::yeni(cumleler);
        #[cfg(debug_assertions)]
        let tanilar = {
            let mut tanilar = tanilar;
            if let Err(hata) = ast.invariantleri_dogrula() {
                tanilar.push(hata.tani());
            }
            tanilar
        };
        (ast, tanilar)
    }
}

/// Parser'ın ürettiği; henüz hoist, ad çözümü ve tür denetimi görmemiş AST.
#[derive(Debug, Clone)]
pub struct AyristirilmisAst {
    cumleler: Vec<Cumle>,
}

impl AyristirilmisAst {
    fn yeni(cumleler: Vec<Cumle>) -> Self {
        Self { cumleler }
    }

    pub fn cumleler(&self) -> &[Cumle] {
        &self.cumleler
    }

    pub fn into_cumleler(self) -> Vec<Cumle> {
        self.cumleler
    }
}

/// Hoist edilmiş fakat checker bağı henüz kurulmamış program.
pub(crate) struct BaglanmamisProgram {
    program: Program,
}

impl BaglanmamisProgram {
    pub(crate) fn yeni(program: Program) -> Self {
        Self { program }
    }

    pub(crate) fn denetle(mut self) -> Result<BaglanmisProgram, Tani> {
        let bilgi = crate::cozumleyici::denetle_ve_hir_bilgisi(&mut self.program)?;
        let program = BaglanmisProgram {
            hir: HirProgram::yeni(self.program, bilgi),
        };
        #[cfg(debug_assertions)]
        program
            .invariantleri_dogrula()
            .map_err(|hata| hata.tani())?;
        Ok(program)
    }
}

/// Adları/semantic ID'leri bağlanmış, tür/etki/akış denetimi tamamlanmış program.
///
/// Zorunlu typed HIR sahibidir. Kaynak AST tanı ve v0 uyumluluğu için
/// salt-okunur korunur; standart runtime semantic kararlarını HIR'dan alır.
pub struct BaglanmisProgram {
    hir: HirProgram,
}

impl BaglanmisProgram {
    pub fn program(&self) -> &Program {
        self.hir.program()
    }

    pub fn hir(&self) -> &HirProgram {
        &self.hir
    }

    #[cfg(test)]
    pub(crate) fn hir_mut(&mut self) -> &mut HirProgram {
        &mut self.hir
    }

    /// Eski `Program` tüketicileri için yalnız başarılı checker geçişinden
    /// sonra faz bilgisini bilinçli olarak siler.
    pub fn into_program(self) -> Program {
        self.hir.into_program()
    }
}

impl Deref for BaglanmisProgram {
    type Target = Program;

    fn deref(&self) -> &Self::Target {
        self.hir.program()
    }
}
