//! Katmanlardan bağımsız Gregoryen takvim dönüşümleri.

/// 1970-01-01'den itibaren gün sayısını UTC Gregoryen tarihe çevirir.
pub fn gunlerden_tarih_utc(gunler: i64) -> (i64, u32, u32) {
    gunlerden_tarih(gunler)
}

/// Gregoryen tarih ↔ gün sayısı (Howard Hinnant'ın algoritmaları; 1970-01-01 = 0).
pub(crate) fn gunlerden_tarih(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let devir = if z >= 0 { z } else { z - 146096 } / 146097;
    let devir_gunu = (z - devir * 146097) as u64;
    let yil_gunu =
        (devir_gunu - devir_gunu / 1460 + devir_gunu / 36524 - devir_gunu / 146096) / 365;
    let yil = yil_gunu as i64 + devir * 400;
    let yilin_gunu = devir_gunu - (365 * yil_gunu + yil_gunu / 4 - yil_gunu / 100);
    let ay_kaba = (5 * yilin_gunu + 2) / 153;
    let gun = (yilin_gunu - (153 * ay_kaba + 2) / 5 + 1) as u32;
    let ay = if ay_kaba < 10 {
        ay_kaba + 3
    } else {
        ay_kaba - 9
    } as u32;
    (if ay <= 2 { yil + 1 } else { yil }, ay, gun)
}

pub(crate) fn tarihten_gunler(yil: i64, ay: u32, gun: u32) -> i64 {
    let yil = if ay <= 2 { yil - 1 } else { yil };
    let devir = if yil >= 0 { yil } else { yil - 399 } / 400;
    let devir_yili = (yil - devir * 400) as u64;
    let yilin_gunu = (153 * (if ay > 2 { ay - 3 } else { ay + 9 }) as u64 + 2) / 5 + gun as u64 - 1;
    let devir_gunu = devir_yili * 365 + devir_yili / 4 - devir_yili / 100 + yilin_gunu;
    devir * 146097 + devir_gunu as i64 - 719468
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn epoch_ve_artık_gun_cift_yonlu_aynidir() {
        for (gunler, tarih) in [
            (0, (1970, 1, 1)),
            (-1, (1969, 12, 31)),
            (19_782, (2024, 2, 29)),
        ] {
            assert_eq!(gunlerden_tarih_utc(gunler), tarih);
            assert_eq!(tarihten_gunler(tarih.0, tarih.1, tarih.2), gunler);
        }
    }
}
