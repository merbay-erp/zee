//! Checker tanılarının AST/HIR kesin kaynak aralığına bağlanması.

use crate::hir::HirKaynakAraligi;
use crate::tani::Tani;

pub(super) fn taniyi_ifadeye_bagla(mut tani: Tani, kaynak_araligi: HirKaynakAraligi) -> Tani {
    // Alt ifade kendi kesin konumunu daha önce yazdıysa onu koru. Checker'ın
    // eski satır-zarfı tanıları ise bu düğümün gerçek AST aralığına yükselir.
    if tani.sutun == 1 && tani.uzunluk == 1 {
        if let Some((satir, sutun, uzunluk)) = kaynak_araligi.kesin_konumu() {
            tani.satir = satir;
            tani.sutun = sutun;
            tani.uzunluk = uzunluk;
        }
    }
    tani
}
