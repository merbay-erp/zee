//! K-335: `herkese açık önbelleklenebilir N saniye` rota politikası (ADR-160).

use super::*;

mod makine;

fn onbellek_bicimi_tanisi(satir: usize) -> Tani {
    Tani::yeni(
        "S043",
        format!(
            "Önbelleklenebilir rota politikası `herkese açık önbelleklenebilir <{}..{}> saniye` biçimindedir.",
            crate::agac::ROTA_ONBELLEK_ALT_SANIYE,
            crate::agac::ROTA_ONBELLEK_UST_SANIYE
        ),
        satir,
        1,
        1,
    )
    .onerili("Örnek: `herkese açık önbelleklenebilir 300 saniye`.".into())
}

/// K-335: `herkese açık önbelleklenebilir N saniye`. `None`, satırın bu
/// kalıba ait olmadığını söyler; kalıp başlayıp bozuksa S043 döner.
pub(super) fn onbellekli_politika(tokenlar: &[Token], satir: usize) -> Option<Result<Cumle, Tani>> {
    if let Some(makine) = makine::makine_politikasi(tokenlar, satir) {
        return Some(makine);
    }
    if tokenlar.len() < 3
        || !kelime_mi(&tokenlar[0], "herkese")
        || !kelime_mi(&tokenlar[1], "açık")
        || !kelime_mi(&tokenlar[2], "önbelleklenebilir")
    {
        return None;
    }
    let saniye = match tokenlar.get(3).map(|token| &token.tur) {
        Some(TokenTur::TamSayi(deger))
            if tokenlar.len() == 5 && kelime_mi(&tokenlar[4], "saniye") =>
        {
            u32::try_from(*deger).ok().filter(|deger| {
                (crate::agac::ROTA_ONBELLEK_ALT_SANIYE..=crate::agac::ROTA_ONBELLEK_UST_SANIYE)
                    .contains(deger)
            })
        }
        _ => None,
    };
    Some(match saniye {
        Some(saniye) => Ok(Cumle::RotaPolitikasi {
            erisim: RotaErisimi::HerkeseAcik,
            kip: crate::agac::RotaKipi::Onbellekli(saniye),
            satir,
        }),
        None => Err(onbellek_bicimi_tanisi(satir)),
    })
}
