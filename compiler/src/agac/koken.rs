//! İsteğe bağlı kaynak kökeni: grammar/runtime değişmeden araçların kaynak sahipliğini korur.
use super::Cumle;
use std::{marker::PhantomData, rc::Rc};
// Compile-time route provenance is opt-in too. This scope never allocates when disabled.
thread_local! { static DERLEME: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
pub struct KaynakKokenNobetcisi {
    onceki: bool,
    etkin: bool,
    _yerel: PhantomData<Rc<()>>,
}
pub fn kaynak_kokenlerini_izle(etkin: bool) -> KaynakKokenNobetcisi {
    let onceki = if etkin {
        DERLEME.with(|s| s.replace(true))
    } else {
        false
    };
    KaynakKokenNobetcisi {
        onceki,
        etkin,
        _yerel: PhantomData,
    }
}
impl Drop for KaynakKokenNobetcisi {
    fn drop(&mut self) {
        if self.etkin {
            DERLEME.with(|s| s.set(self.onceki));
        }
    }
}
pub(crate) fn rota_kokenlerini_isaretle(cumleler: &mut [Cumle], koken: Option<&str>) {
    if !DERLEME.with(std::cell::Cell::get) {
        return;
    }
    for c in cumleler {
        if let Cumle::IstekGeldiginde { koken: hedef, .. } = c {
            if hedef.is_none() {
                *hedef = koken.map(str::to_owned);
            }
        }
    }
}
