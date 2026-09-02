//! Parser AST ifadelerinin kesin kaynak aralığı kapısı.

use crate::agac::{AstKaynakAraligi, Ifade};
use crate::sozcukleyici::Token;
use crate::tani::Tani;

/// Tüketilen token bölgesini tek satırlık kesin AST kaynak aralığına çevirir.
/// Parser bütün ifade düğümlerini bu kapıdan geçirir; boş/taşmış/çok satırlı
/// bölge kullanıcı sözdizimi değil derleyici iç sözleşme ihlalidir.
pub(super) fn konumlu_ifade(ifade: Ifade, tokenlar: &[Token]) -> Result<Ifade, Tani> {
    if ifade.dogrudan_kaynakli_mi() {
        return Ok(ifade);
    }
    let Some(ilki) = tokenlar.first() else {
        return Err(Tani::yeni(
            "C000",
            "AST ifadesi boş token bölgesinden kaynaklandırılamadı.".into(),
            1,
            1,
            1,
        ));
    };
    let Some(sonuncu) = tokenlar.last() else {
        return Err(Tani::yeni(
            "C000",
            "AST ifadesinin son tokenı bulunamadı.".into(),
            ilki.satir,
            ilki.sutun,
            ilki.uzunluk,
        ));
    };
    if ilki.satir != sonuncu.satir {
        return Err(Tani::yeni(
            "C000",
            "Tek satırlık AST ifadesi birden çok kaynak satırına yayıldı.".into(),
            ilki.satir,
            ilki.sutun,
            ilki.uzunluk,
        ));
    }
    let Some(son) = sonuncu.sutun.checked_add(sonuncu.uzunluk) else {
        return Err(Tani::yeni(
            "C000",
            "AST kaynak aralığının sonu sayı sınırını aştı.".into(),
            ilki.satir,
            ilki.sutun,
            ilki.uzunluk,
        ));
    };
    let Some(uzunluk) = son.checked_sub(ilki.sutun) else {
        return Err(Tani::yeni(
            "C000",
            "AST kaynak tokenları ters sırada.".into(),
            ilki.satir,
            ilki.sutun,
            ilki.uzunluk,
        ));
    };
    let Some(aralik) = AstKaynakAraligi::yeni(ilki.satir, ilki.sutun, uzunluk) else {
        return Err(Tani::yeni(
            "C000",
            "AST için sıfır kaynak aralığı kurulamaz.".into(),
            ilki.satir,
            ilki.sutun,
            ilki.uzunluk,
        ));
    };
    Ok(Ifade::kaynakli(ifade, aralik))
}
