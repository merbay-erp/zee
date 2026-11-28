//! K-335 önbelleklenebilir rota denetimi (T056/T057, ADR-160).

use crate::agac::{Cumle, HttpYontemi, RotaKipi};
use crate::tani::Tani;

mod iz;
pub(super) use iz::OnbellekIzi;

/// Önbelleklenebilir rota yalnız GET/HEAD olabilir (T056); rota gövdesi ve
/// çağırdığı işlemler oturum/CSRF/çerez yüzeyine dokunamaz (T057).
pub(super) fn onbellekli_rotayi_denetle(
    yontem: HttpYontemi,
    govde: &[Cumle],
    dogrudan: &OnbellekIzi,
    toplam: &OnbellekIzi,
) -> Result<(), Tani> {
    let Some(Cumle::RotaPolitikasi {
        kip: RotaKipi::Onbellekli(_),
        satir,
        ..
    }) = govde.first()
    else {
        return Ok(());
    };
    if !yontem.guvenli() {
        return Err(Tani::yeni(
            "T056",
            format!(
                "{} rotası önbelleklenebilir olamaz; paylaşılan önbellek yalnız GET ve HEAD yanıtına izinlidir.",
                yontem.yazimi()
            ),
            *satir,
            1,
            1,
        )
        .onerili("Durum değiştiren rotada `herkese açık` yaz; önbellek iznini salt-okuma GET rotasına taşı.".into()));
    }
    if let Some(dokunus) = OnbellekIzi::oturum_dokunusu(dogrudan, toplam) {
        return Err(Tani::yeni(
            "T057",
            "Önbelleklenebilir rota oturum, CSRF belirteci ya da çerez kullanıyor; yanıtı başka kullanıcıya sızabilir."
                .into(),
            dokunus,
            1,
            1,
        )
        .onerili("Oturuma bağlı parçayı ayrı bir rotaya taşı ya da `önbelleklenebilir` iznini kaldırıp `herkese açık` yaz.".into()));
    }
    Ok(())
}
