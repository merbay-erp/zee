use super::*;

/// Özellik sözdizimi bir yapı alanına çözüldüğünde alt ifade kutusunu taşır.
/// Klonlama, eski alt düğüm adresinin HIR kaydını yetim bırakırdı.
pub(super) fn ozelligi_alana_donustur(
    ifade: &mut Ifade,
    alan: String,
    satir: usize,
) -> Result<(), Tani> {
    let eski = std::mem::replace(ifade.turu_mut(), Ifade::BosListe);
    let Ifade::Ozellik { nesne, .. } = eski else {
        return Err(ic_tutarlilik_hatasi(
            "Özellik→alan dönüşümünde AST varyantı kayboldu",
            satir,
        ));
    };
    *ifade.turu_mut() = Ifade::AlanErisim { nesne, alan };
    Ok(())
}

/// `her sayı için` yüzeyinin çözümlenen örtük `sayılar` kaynağını, döngü
/// adının gerçek lexer aralığına bağlayarak kurar.
pub(super) fn ortuk_cogul_kaynagi(
    kaynak_adi: String,
    sembol_kimligi: Option<crate::kimlik::SymbolId>,
    satir: usize,
    sutun: usize,
    uzunluk: usize,
) -> Result<Ifade, Tani> {
    let aralik = crate::agac::AstKaynakAraligi::yeni(satir, sutun, uzunluk)
        .ok_or_else(|| hir_kaynak_hatasi(satir))?;
    Ok(Ifade::kaynakli(
        Ifade::Degisken {
            ham: kaynak_adi.clone(),
            sembol_kimligi,
            cozulmus: Some(kaynak_adi),
            satir,
            sutun,
            uzunluk,
        },
        aralik,
    ))
}
