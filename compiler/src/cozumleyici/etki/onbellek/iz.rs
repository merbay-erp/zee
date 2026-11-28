//! K-335/K-405/RFC-0047 E: rota gövdesinin oturum/CSRF/çerez, ham gövde ve
//! uygulama parolası dokunuş izi (geçişli işlemler dahil).

use crate::agac::{Cumle, Ifade};
use crate::yetkinlik::Yetkinlik;

/// Oturum/CSRF/çerez yüzeyine ilk dokunuşun satırları. `dokunus` çağrılan
/// işlemlerden geçişli yayılır; `degisken` (rota ortamındaki `oturum` /
/// `çerezler` okuması) yalnız rota gövdesinde anlamlıdır, çağrılan işlemin
/// aynı adlı yereli sayılmaz. K-405 aynı izi makine uç noktası için kullanır;
/// `ham_govde` `isteğin ham gövdesi` okumasının ilk satırıdır.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::cozumleyici::etki) struct OnbellekIzi {
    dokunus: Option<usize>,
    degisken: Option<usize>,
    ham_govde: Option<usize>,
    /// RFC-0047 E: `yeni uygulama parolası` üretiminin ilk satırı (T063).
    pub(in crate::cozumleyici::etki) uygulama_parolasi: Option<usize>,
}

fn en_erken(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

impl OnbellekIzi {
    pub(in crate::cozumleyici::etki) fn birlestir(&mut self, baska: OnbellekIzi) {
        self.dokunus = en_erken(self.dokunus, baska.dokunus);
        self.degisken = en_erken(self.degisken, baska.degisken);
        self.ham_govde = en_erken(self.ham_govde, baska.ham_govde);
        self.uygulama_parolasi = en_erken(self.uygulama_parolasi, baska.uygulama_parolasi);
    }

    /// Rota gövdesinin doğrudan değişken okuması ya da geçişli dokunuşu.
    pub(in crate::cozumleyici::etki) fn oturum_dokunusu(
        dogrudan: &Self,
        toplam: &Self,
    ) -> Option<usize> {
        en_erken(toplam.dokunus, dogrudan.degisken)
    }

    /// Geçişli oturum/CSRF/çerez yazma dokunuşu (RFC-0047 DAV, T066).
    pub(in crate::cozumleyici::etki) fn dokunus(&self) -> Option<usize> {
        self.dokunus
    }

    pub(in crate::cozumleyici::etki) fn ham_govde(&self) -> Option<usize> {
        self.ham_govde
    }

    /// Çerez yazma/silme ve oturum açma/kapatma cümleleri.
    pub(in crate::cozumleyici::etki) fn cumle(&mut self, cumle: &Cumle) {
        if let Cumle::CerezYaz { satir, .. }
        | Cumle::CerezSil { satir, .. }
        | Cumle::OturumAc { satir, .. }
        | Cumle::OturumKapat { satir } = cumle
        {
            self.dokunus = en_erken(self.dokunus, Some(*satir));
        }
    }

    /// `csrf belirteci`, web oturumu listeleme/iptal gibi web-oturumu intrinsic'leri.
    pub(in crate::cozumleyici::etki) fn intrinsic(&mut self, yetkinlik: Yetkinlik, ifade: &Ifade) {
        let satir = ifade.kaynak_araligi().map_or(1, |aralik| aralik.satir());
        if yetkinlik == Yetkinlik::WebOturumu {
            self.dokunus = en_erken(self.dokunus, Some(satir));
        }
        let hedef = match ifade.turu() {
            Ifade::Intrinsic { kimlik, .. } if kimlik == crate::intrinsic::WEB_HAM_GOVDE => {
                &mut self.ham_govde
            }
            Ifade::Intrinsic { kimlik, .. } if kimlik == crate::intrinsic::UYGULAMA_PAROLASI => {
                &mut self.uygulama_parolasi
            }
            _ => return,
        };
        *hedef = en_erken(*hedef, Some(satir));
    }

    /// Ad çözümlemesinden önce çalıştığı için çekimli biçimde kök adayına bakar.
    pub(in crate::cozumleyici::etki) fn degisken(&mut self, ham: &str, satir: usize) {
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
