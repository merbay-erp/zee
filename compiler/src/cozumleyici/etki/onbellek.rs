//! K-335 önbelleklenebilir rota denetimi (T056/T057, ADR-160).

use crate::agac::{Cumle, HttpYontemi, Ifade, RotaKipi};
use crate::tani::Tani;
use crate::yetkinlik::Yetkinlik;

/// Oturum/CSRF/çerez yüzeyine ilk dokunuşun satırları. `dokunus` çağrılan
/// işlemlerden geçişli yayılır; `degisken` (rota ortamındaki `oturum` /
/// `çerezler` okuması) yalnız rota gövdesinde anlamlıdır, çağrılan işlemin
/// aynı adlı yereli sayılmaz. K-405 aynı izi makine uç noktası için kullanır;
/// `ham_govde` `isteğin ham gövdesi` okumasının ilk satırıdır.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct OnbellekIzi {
    dokunus: Option<usize>,
    degisken: Option<usize>,
    ham_govde: Option<usize>,
}

fn en_erken(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

impl OnbellekIzi {
    pub(super) fn birlestir(&mut self, baska: OnbellekIzi) {
        self.dokunus = en_erken(self.dokunus, baska.dokunus);
        self.degisken = en_erken(self.degisken, baska.degisken);
        self.ham_govde = en_erken(self.ham_govde, baska.ham_govde);
    }

    /// Rota gövdesinin doğrudan değişken okuması ya da geçişli dokunuşu.
    pub(super) fn oturum_dokunusu(dogrudan: &Self, toplam: &Self) -> Option<usize> {
        en_erken(toplam.dokunus, dogrudan.degisken)
    }

    pub(super) fn ham_govde(&self) -> Option<usize> {
        self.ham_govde
    }

    /// Çerez yazma/silme ve oturum açma/kapatma cümleleri.
    pub(super) fn cumle(&mut self, cumle: &Cumle) {
        if let Cumle::CerezYaz { satir, .. }
        | Cumle::CerezSil { satir, .. }
        | Cumle::OturumAc { satir, .. }
        | Cumle::OturumKapat { satir } = cumle
        {
            self.dokunus = en_erken(self.dokunus, Some(*satir));
        }
    }

    /// `csrf belirteci`, web oturumu listeleme/iptal gibi web-oturumu intrinsic'leri.
    pub(super) fn intrinsic(&mut self, yetkinlik: Yetkinlik, ifade: &Ifade) {
        let satir = ifade.kaynak_araligi().map_or(1, |aralik| aralik.satir());
        if yetkinlik == Yetkinlik::WebOturumu {
            self.dokunus = en_erken(self.dokunus, Some(satir));
        }
        if matches!(ifade.turu(), Ifade::Intrinsic { kimlik, .. } if kimlik == crate::intrinsic::WEB_HAM_GOVDE)
        {
            self.ham_govde = en_erken(self.ham_govde, Some(satir));
        }
    }

    /// Ad çözümlemesinden önce çalıştığı için çekimli biçimde kök adayına bakar.
    pub(super) fn degisken(&mut self, ham: &str, satir: usize) {
        const ADLAR: [&str; 2] = ["oturum", "çerezler"];
        if ADLAR.contains(&ham)
            || crate::morfoloji::kok_adaylari(ham)
                .iter()
                .any(|kok| ADLAR.contains(&kok.as_str()))
        {
            self.degisken = en_erken(self.degisken, Some(satir));
        }
    }
}

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
