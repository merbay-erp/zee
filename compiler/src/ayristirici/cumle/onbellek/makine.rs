//! K-405: `herkese açık makine uç noktası` rota politikası (ADR-179).

use super::*;

/// `None`, satırın `makine uç noktası` ile bitmediğini söyler. Kalıp yalnız
/// `herkese açık` ile birlikte tek politika cümlesidir; oturumlu/yetkili ya
/// da tek başına yazım S050'dir.
pub(super) fn makine_politikasi(tokenlar: &[Token], satir: usize) -> Option<Result<Cumle, Tani>> {
    let n = tokenlar.len();
    if n < 3
        || !kelime_mi(&tokenlar[n - 3], "makine")
        || !kelime_mi(&tokenlar[n - 2], "uç")
        || !kelime_mi(&tokenlar[n - 1], "noktası")
    {
        return None;
    }
    if n == 5 && kelime_mi(&tokenlar[0], "herkese") && kelime_mi(&tokenlar[1], "açık") {
        return Some(Ok(Cumle::RotaPolitikasi {
            erisim: RotaErisimi::HerkeseAcik,
            kip: crate::agac::RotaKipi::Makine,
            satir,
        }));
    }
    Some(Err(Tani::yeni(
        "S050",
        "Makine uç noktası yalnız `herkese açık makine uç noktası` politika cümlesiyle yazılır."
            .into(),
        satir,
        1,
        1,
    )
    .onerili(
        "Rotanın ilk satırına `herkese açık makine uç noktası` yaz; oturumlu ya da yetkili rota makine ucu olamaz."
            .into(),
    )))
}
