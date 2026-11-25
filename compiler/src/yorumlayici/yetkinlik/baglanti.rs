//! K-397 (ADR-173): bağlantı işlemlerinin politika kapısı.
use super::*;

/// K-397: giden bağlanma `ağ` ister (hedef ve IP politikası host'ta, tek
/// çözümlemeyle); açık bağlantının işlemleri dinleyici ya da giden
/// bağlantıdan gelir, `ağ-dinleyici` ya da `ağ` yeter.
pub(super) fn baglanti_reddi<T: GirdiCikti>(
    io: &PolitikaliIo<T>,
    islem: &super::super::BaglantiIslemi,
) -> Option<super::super::BaglantiSonucu> {
    use super::super::{BaglantiIslemi, BaglantiSonucu};
    if matches!(islem, BaglantiIslemi::GidenBaglan(..)) {
        return io
            .gerektir(Yetkinlik::Ag)
            .err()
            .map(|hata| BaglantiSonucu::hata("hedef_yasak", hata));
    }
    if io.politika.izin_verir(Yetkinlik::Ag) {
        return None;
    }
    io.gerektir(Yetkinlik::AgDinleyici)
        .err()
        .map(|hata| BaglantiSonucu::hata("kapandı", hata))
}
