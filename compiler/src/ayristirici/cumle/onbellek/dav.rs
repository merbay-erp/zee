//! RFC-0047 F (ADR-182 dilim C): `dav istemcisi gerekli` rota politikası.

use super::*;

/// Erişim kararı Basic + uygulama parolasıyla rota öncesi verilir; çerez
/// oturumu yoktur. `None`, satırın bu kalıp olmadığını söyler.
pub(super) fn dav_politikasi(tokenlar: &[Token], satir: usize) -> Option<Result<Cumle, Tani>> {
    let kalip = tokenlar.len() == 3
        && kelime_mi(&tokenlar[0], "dav")
        && kelime_mi(&tokenlar[1], "istemcisi")
        && kelime_mi(&tokenlar[2], "gerekli");
    kalip.then_some(Ok(Cumle::RotaPolitikasi {
        erisim: RotaErisimi::HerkeseAcik,
        kip: crate::agac::RotaKipi::Dav,
        satir,
    }))
}
