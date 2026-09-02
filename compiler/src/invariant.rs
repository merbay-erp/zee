//! AST/HIR faz değişmezlerinin yürütülebilir doğrulayıcısı.
//!
//! Parser AST'si semantic bağ taşıyamaz; başarılı checker HIR'ı ise her ifade
//! için tür, kaynak aralığı ve varyantıyla uyumlu semantic bağ taşımak zorundadır.

mod cumle;
mod ifade;

use std::collections::HashSet;
use std::fmt;

use crate::agac::{Cumle, Ifade};
use crate::faz::{AyristirilmisAst, BaglanmisProgram};
use crate::hir::{HirBagi, HirDugumId, HirIfadeTuru, HirProgram};
#[cfg(debug_assertions)]
use crate::tani::Tani;

/// Değişmezin denetlendiği fiziksel derleyici fazı.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantFazi {
    AyristirilmisAst,
    BaglanmisHir,
}

impl InvariantFazi {
    pub const fn adi(self) -> &'static str {
        match self {
            Self::AyristirilmisAst => "ayrıştırılmış AST",
            Self::BaglanmisHir => "bağlanmış HIR",
        }
    }
}

/// Kullanıcı hatası olmayan, compiler iç yapı sözleşmesi ihlali.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvariantHatasi {
    faz: InvariantFazi,
    yol: String,
    mesaj: String,
    satir: usize,
}

impl InvariantHatasi {
    fn yeni(
        faz: InvariantFazi,
        yol: impl Into<String>,
        mesaj: impl Into<String>,
        satir: usize,
    ) -> Self {
        Self {
            faz,
            yol: yol.into(),
            mesaj: mesaj.into(),
            satir: satir.max(1),
        }
    }

    pub const fn faz(&self) -> InvariantFazi {
        self.faz
    }

    pub fn yol(&self) -> &str {
        &self.yol
    }

    pub fn mesaj(&self) -> &str {
        &self.mesaj
    }

    pub const fn satir(&self) -> usize {
        self.satir
    }

    #[cfg(debug_assertions)]
    pub(crate) fn tani(&self) -> Tani {
        Tani::yeni(
            "C000",
            format!(
                "{} değişmezi bozuldu ({}): {}",
                self.faz.adi(),
                self.yol,
                self.mesaj
            ),
            self.satir,
            1,
            1,
        )
    }
}

impl fmt::Display for InvariantHatasi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} değişmezi bozuldu ({}): {}",
            self.faz.adi(),
            self.yol,
            self.mesaj
        )
    }
}

impl std::error::Error for InvariantHatasi {}

/// Parser'ın ürettiği cümle ağacının semantic bağ taşımadığını ve temel yapı
/// durumlarının mümkün olduğunu doğrular.
pub fn ayristirilmis_ast_dogrula(cumleler: &[Cumle]) -> Result<(), InvariantHatasi> {
    let mut denetleyici = Denetleyici::yeni(InvariantFazi::AyristirilmisAst, None);
    denetleyici.cumleleri_dogrula(cumleler, "ast.cümleler", false)?;
    denetleyici.bitir()
}

/// Başarılı checker çıktısındaki AST alanlarıyla HIR tür/kimlik tablolarının
/// birbirini eksiksiz ve çelişkisiz doğruladığını denetler.
pub fn baglanmis_hir_dogrula(hir: &HirProgram) -> Result<(), InvariantHatasi> {
    let mut denetleyici = Denetleyici::yeni(InvariantFazi::BaglanmisHir, Some(hir));
    denetleyici.programi_dogrula(hir.program())?;
    denetleyici.bitir()
}

impl AyristirilmisAst {
    pub fn invariantleri_dogrula(&self) -> Result<(), InvariantHatasi> {
        ayristirilmis_ast_dogrula(self.cumleler())
    }
}

impl BaglanmisProgram {
    pub fn invariantleri_dogrula(&self) -> Result<(), InvariantHatasi> {
        baglanmis_hir_dogrula(self.hir())
    }
}

struct Denetleyici<'a> {
    faz: InvariantFazi,
    hir: Option<&'a HirProgram>,
    ifade_adresleri: HashSet<usize>,
    hir_kimlikleri: HashSet<HirDugumId>,
}

impl<'a> Denetleyici<'a> {
    fn yeni(faz: InvariantFazi, hir: Option<&'a HirProgram>) -> Self {
        Self {
            faz,
            hir,
            ifade_adresleri: HashSet::new(),
            hir_kimlikleri: HashSet::new(),
        }
    }

    fn hata(
        &self,
        yol: impl Into<String>,
        mesaj: impl Into<String>,
        satir: usize,
    ) -> InvariantHatasi {
        InvariantHatasi::yeni(self.faz, yol, mesaj, satir)
    }

    fn hir(&self, yol: &str, satir: usize) -> Result<&'a HirProgram, InvariantHatasi> {
        self.hir
            .ok_or_else(|| self.hata(yol, "bağlanmış fazda HIR tablosu yok", satir))
    }

    fn hir_ifadesini_dogrula(
        &mut self,
        ifade: &Ifade,
        yol: &str,
        satir: usize,
        beklenen_bag: HirBagi,
        deger_dondurmez_olabilir: bool,
    ) -> Result<(), InvariantHatasi> {
        if self.faz == InvariantFazi::AyristirilmisAst {
            return Ok(());
        }

        let adres = crate::hir::ifade_adresi(ifade);
        if !self.ifade_adresleri.insert(adres) {
            return Err(self.hata(yol, "aynı AST ifade düğümü iki kez ziyaret edildi", satir));
        }
        let hir = self.hir(yol, satir)?;
        let bilgi = hir
            .ifade_bilgisi(ifade)
            .ok_or_else(|| self.hata(yol, "ifadenin HIR tür/bağ kaydı yok", satir))?;
        let ast_araligi = ifade
            .kaynak_araligi()
            .ok_or_else(|| self.hata(yol, "AST ifadesinin kesin kaynak aralığı yok", satir))?;
        if bilgi.kaynak_araligi().kesin_konumu() != Some(ast_araligi.uclu()) {
            return Err(self.hata(
                yol,
                "HIR kaynak aralığı AST'nin kesin kaynak aralığıyla eşleşmiyor",
                satir,
            ));
        }
        if !self.hir_kimlikleri.insert(bilgi.kimlik()) {
            return Err(self.hata(
                yol,
                format!("HIR düğüm kimliği yinelendi: {}", bilgi.kimlik().sirasi()),
                satir,
            ));
        }
        if bilgi.bag() != beklenen_bag {
            return Err(self.hata(
                yol,
                format!("HIR bağı {:?}, beklenen {:?}", bilgi.bag(), beklenen_bag),
                satir,
            ));
        }
        if bilgi.tur() == HirIfadeTuru::DegerDondurmez && !deger_dondurmez_olabilir {
            return Err(self.hata(yol, "değer konumundaki ifade DeğerDöndürmez taşıyor", satir));
        }
        Ok(())
    }

    fn bitir(self) -> Result<(), InvariantHatasi> {
        if let Some(hir) = self.hir {
            if self.ifade_adresleri.len() != hir.ifade_sayisi() {
                return Err(InvariantHatasi::yeni(
                    self.faz,
                    "hir.ifadeler",
                    format!(
                        "AST'de {} ifade ziyaret edildi, HIR tablosunda {} kayıt var",
                        self.ifade_adresleri.len(),
                        hir.ifade_sayisi()
                    ),
                    1,
                ));
            }
        }
        Ok(())
    }
}
