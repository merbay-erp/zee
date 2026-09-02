//! B-010 — depolama konumu ile semantic identity ayrımı.

use dil::agac::{Cumle, Ifade};
use dil::kaynagi_derle;
use dil::kimlik::{IslemId, YapiId};
use std::collections::HashMap;

fn yapi_kimlikleri(kaynak: &str) -> HashMap<String, YapiId> {
    let program = kaynagi_derle(kaynak).expect("yapılar bağlanmalı");
    program
        .cumleler
        .iter()
        .filter_map(|cumle| {
            let Cumle::Olsun { deger, .. } = cumle else {
                return None;
            };
            let Ifade::YeniYapi {
                yapi_adi,
                yapi_kimligi: Some(kimlik),
            } = deger.turu()
            else {
                return None;
            };
            Some((yapi_adi.clone(), *kimlik))
        })
        .collect()
}

fn islem_kimlikleri(kaynak: &str) -> HashMap<String, IslemId> {
    let program = kaynagi_derle(kaynak).expect("işlemler bağlanmalı");
    program
        .cumleler
        .iter()
        .filter_map(|cumle| {
            let Cumle::Olsun { deger, .. } = cumle else {
                return None;
            };
            let Ifade::IslemCagrisi {
                islem_adi,
                islem_kimligi: Some(kimlik),
                ..
            } = deger.turu()
            else {
                return None;
            };
            Some((islem_adi.clone(), *kimlik))
        })
        .collect()
}

#[test]
fn yapi_kimligi_depolama_sirasindan_bagimsizdir() {
    let ilk = r#"
yapı Zebra
    ad Metin
yapı Arı
    ad Metin

zebra yeni Zebra olsun
arı yeni Arı olsun
"#;
    let ters = r#"
yapı Arı
    ad Metin
yapı Zebra
    ad Metin

zebra yeni Zebra olsun
arı yeni Arı olsun
"#;

    let ilk_kimlikler = yapi_kimlikleri(ilk);
    let ters_kimlikler = yapi_kimlikleri(ters);
    assert_eq!(ilk_kimlikler, ters_kimlikler);
    assert_eq!(ilk_kimlikler["Arı"].sirasi(), 0);
    assert_eq!(ilk_kimlikler["Zebra"].sirasi(), 1);
}

#[test]
fn islem_kimligi_hashmap_ve_cagri_sirasindan_bagimsizdir() {
    let govde = r#"
işlem bir ver
    1 döndür

işlem iki ver
    2 döndür
"#;
    let ilk = format!("{govde}\nbir bir ver olsun\niki iki ver olsun\n");
    let ters = format!("{govde}\niki iki ver olsun\nbir bir ver olsun\n");

    let ilk_kimlikler = islem_kimlikleri(&ilk);
    let ters_kimlikler = islem_kimlikleri(&ters);
    assert_eq!(ilk_kimlikler, ters_kimlikler);
    assert_eq!(ilk_kimlikler["bir ver"].sirasi(), 0);
    assert_eq!(ilk_kimlikler["iki ver"].sirasi(), 1);
}

#[test]
fn cozulmus_degisken_symbol_id_tasir() {
    let program = kaynagi_derle("sayı 1 olsun\nsayıyı yaz\n").expect("sembol bağlanmalı");
    let Cumle::Yaz { deger, .. } = &program.cumleler[1]
    else {
        panic!("çözülmüş değişken bekleniyordu")
    };
    let Ifade::Degisken {
        cozulmus: Some(ad),
        sembol_kimligi: Some(kimlik),
        ..
    } = deger.turu()
    else {
        panic!("çözülmüş değişken bekleniyordu")
    };

    assert_eq!(ad, "sayı");
    assert_eq!(kimlik.kapsam(), 0);
    assert_eq!(kimlik.sirasi(), 0);
}
