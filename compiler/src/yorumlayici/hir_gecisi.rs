//! Typed HIR ile eski raw-AST runtime'ı arasındaki yürütme sınırı.

use super::*;
use crate::hir::{HirBagi, HirProgram};

/// Raw v0 AST ile typed-HIR bağlı standart hattı aynı davranış çekirdeğinde
/// tutar. `Hir` kolu semantic seçimlerde kaynak adını yedek olarak kullanmaz.
#[derive(Clone, Copy)]
pub(super) enum CalistirmaProgrami<'a> {
    Ham(&'a Program),
    Hir(&'a HirProgram),
}

impl<'a> CalistirmaProgrami<'a> {
    pub(super) fn program(self) -> &'a Program {
        match self {
            Self::Ham(program) => program,
            Self::Hir(hir) => hir.program(),
        }
    }

    pub(super) fn sembol_adi(self, ifade: &Ifade, ham_ad: Option<&str>) -> Option<String> {
        match self {
            Self::Ham(_) => ham_ad.map(str::to_owned),
            Self::Hir(hir) => match hir.ifade_bilgisi(ifade)?.bag() {
                HirBagi::Sembol(kimlik) => hir.sembol_adi(kimlik).map(str::to_owned),
                _ => None,
            },
        }
    }

    pub(super) fn islem(self, ifade: &Ifade, ham_ad: &str) -> Option<&'a crate::agac::Islem> {
        match self {
            Self::Ham(program) => program.islemler.get(ham_ad),
            Self::Hir(hir) => match hir.ifade_bilgisi(ifade)?.bag() {
                HirBagi::Islem(kimlik) => hir.islem(kimlik),
                _ => None,
            },
        }
    }

    pub(super) fn yapi(self, ifade: &Ifade, ham_ad: &str) -> Option<&'a crate::agac::Yapi> {
        match self {
            Self::Ham(program) => program.yapilar.iter().find(|yapi| yapi.ad == ham_ad),
            Self::Hir(hir) => match hir.ifade_bilgisi(ifade)?.bag() {
                HirBagi::Yapi(kimlik) => hir.yapi(kimlik),
                _ => None,
            },
        }
    }
}

pub fn calistir_baglanmis(program: &crate::faz::BaglanmisProgram) -> Result<Vec<String>, Tani> {
    let mut io = ToplayanIo::yeni(Vec::new());
    calistir_baglanmis_io(program, &mut io)?;
    Ok(io.cikti)
}

pub fn calistir_baglanmis_io(
    program: &crate::faz::BaglanmisProgram,
    io: &mut dyn GirdiCikti,
) -> Result<(), Tani> {
    calistir_baglanmis_io_kodla(program, io).map(|_| ())
}

/// Typed HIR üzerinden çalıştırır ve `programı N ile bitir` sonucunu korur.
///
/// Gömücüler normal bitişte `0`, görünür bitirişte `N` alır; runtime hataları
/// yine kodlu [`Tani`] olarak döner. Böylece çıkış kodunu gözleyen yüzeyin faz
/// bilgisini silip ham AST yorumlayıcısına dönmesi gerekmez.
pub fn calistir_baglanmis_io_kodla(
    program: &crate::faz::BaglanmisProgram,
    io: &mut dyn GirdiCikti,
) -> Result<i64, Tani> {
    calistir_program_kodla(CalistirmaProgrami::Hir(program.hir()), io)
}

pub fn test_calistir_baglanmis(
    program: &crate::faz::BaglanmisProgram,
    test: &crate::agac::Test,
    io: &mut dyn GirdiCikti,
) -> Result<(), Tani> {
    let _butce = CalistirmaButcesiNobetcisi::yeni();
    let mut ortam: HashMap<String, Deger> = HashMap::new();
    match blok_calistir(
        &test.govde,
        &mut ortam,
        CalistirmaProgrami::Hir(program.hir()),
        io,
        0,
    ) {
        Err(tani) if tani.kod == "Ç000" => Ok(()),
        sonuc => sonuc.map(|_| ()),
    }
}
