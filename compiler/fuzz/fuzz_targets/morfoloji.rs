#![no_main]

use dil::cozumleyici::{ad_cozumle, Tur};
use dil::morfoloji::{cozumleri_bul, ek_zinciri_uydur, kok_adaylari, SoyutEk};
use libfuzzer_sys::fuzz_target;
use std::collections::HashMap;

const ILK_HARFLER: &[char] = &[
    'a', 'b', 'c', 'ç', 'd', 'e', 'f', 'g', 'ğ', 'h', 'ı', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'ö',
    'p', 'r', 's', 'ş', 't', 'u', 'ü', 'v', 'y', 'z', 'â', 'î', 'û', 'A', 'Ç', 'Ğ', 'I', 'İ', 'Ö',
    'Ş', 'Ü', 'Â', 'Î', 'Û', '_',
];
const DEVAM_HARFLERI: &[char] = &[
    'a', 'e', 'ı', 'i', 'o', 'ö', 'u', 'ü', 'b', 'c', 'ç', 'd', 'f', 'g', 'ğ', 'h', 'j', 'k', 'l',
    'm', 'n', 'p', 'r', 's', 'ş', 't', 'v', 'y', 'z', 'â', 'î', 'û', '_', '0', '1', '2', '7', '9',
];
const TEK_EKLER: &[SoyutEk] = &[
    SoyutEk::Belirtme,
    SoyutEk::Tamlayan,
    SoyutEk::Yonelme,
    SoyutEk::Ayrilma,
    SoyutEk::Bulunma,
    SoyutEk::Arac,
    SoyutEk::CogulYonelme,
];
const DIS_EKLER: &[SoyutEk] = &[
    SoyutEk::Belirtme,
    SoyutEk::Tamlayan,
    SoyutEk::Yonelme,
    SoyutEk::Ayrilma,
    SoyutEk::Bulunma,
    SoyutEk::Arac,
];

fn kok_uret(veri: &[u8]) -> String {
    let uzunluk = 2 + usize::from(veri[1]) % 63;
    let mut kok = String::with_capacity(uzunluk);
    kok.push(ILK_HARFLER[usize::from(veri[2]) % ILK_HARFLER.len()]);
    for sira in 1..uzunluk {
        let bayt = veri[2 + sira % (veri.len() - 2)];
        kok.push(DEVAM_HARFLERI[usize::from(bayt) % DEVAM_HARFLERI.len()]);
    }
    kok
}

fn belirsizlik_fail_closed(yuzey: &str) {
    let adaylar = kok_adaylari(yuzey);
    let ortam = adaylar
        .iter()
        .cloned()
        .map(|kok| (kok, Tur::TamSayi))
        .collect::<HashMap<_, _>>();
    let sonuc = ad_cozumle(yuzey, &ortam, 1, 1, yuzey.chars().count());

    match adaylar.as_slice() {
        [] => assert_eq!(sonuc.expect_err("adaysız yüzey çözülemez").kod, "A001"),
        [tek] => assert_eq!(sonuc.expect("tek aday çözülmeli"), *tek),
        _ => assert_eq!(
            sonuc.expect_err("çoklu aday sessizce seçilemez").kod,
            "A002"
        ),
    }
}

fuzz_target!(|veri: &[u8]| {
    if veri.len() < 3 {
        return;
    }

    let kok = kok_uret(veri);
    let ekler = if veri[0] & 1 == 0 {
        vec![TEK_EKLER[usize::from(veri[0]) % TEK_EKLER.len()]]
    } else {
        vec![
            SoyutEk::Iyelik,
            DIS_EKLER[usize::from(veri[0]) % DIS_EKLER.len()],
        ]
    };
    let yuzey = ek_zinciri_uydur(&kok, &ekler).expect("üretilen zincir geçerli olmalı");
    let cozumler = cozumleri_bul(&yuzey);
    assert!(
        cozumler
            .iter()
            .any(|cozum| cozum.kok == kok && cozum.ekler == ekler),
        "üret→çöz değişmezi bozuldu: {kok:?} + {ekler:?} = {yuzey:?}: {cozumler:?}"
    );
    belirsizlik_fail_closed(&yuzey);
});
