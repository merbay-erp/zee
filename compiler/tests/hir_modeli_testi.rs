//! B-019 ilk dilim — checker türleri ve semantic bağları ayrı HIR'da taşır.

use dil::agac::{Cumle, Ifade};
use dil::cozumleyici::Tur;
use dil::hir::{HirBagi, HirIfadeTuru, HirKaynakAraligi};
use dil::kaynagi_fazli_derle;

#[test]
fn hir_ifade_turunu_ve_symbol_id_bagini_ast_disinda_tasir() {
    let program = kaynagi_fazli_derle("sayı 1 olsun\nsayıyı yaz\n").expect("HIR üretilmeli");
    let Cumle::Yaz { deger, .. } = &program.cumleler[1] else {
        panic!("yaz cümlesi bekleniyordu")
    };
    let Ifade::Degisken {
        sembol_kimligi: Some(ast_kimligi),
        ..
    } = deger
    else {
        panic!("çözülmüş değişken bekleniyordu")
    };

    let bilgi = program
        .hir()
        .ifade_bilgisi(deger)
        .expect("her denetlenmiş ifade HIR bilgisi taşımalı");
    assert_eq!(bilgi.tur(), HirIfadeTuru::Deger(Tur::TamSayi));
    assert_eq!(bilgi.kimlik().sirasi(), 1);
    assert_eq!(bilgi.bag(), HirBagi::Sembol(*ast_kimligi));
    assert_eq!(bilgi.kaynak_araligi().kesin_konumu(), Some((2, 1, 6)));
    assert_eq!(program.hir().sembol_adi(*ast_kimligi), Some("sayı"));
}

#[test]
fn her_hir_ifadesi_zorunlu_kaynak_araligi_tasir() {
    let program = kaynagi_fazli_derle("sonuç 1 ile 2 nin toplamı olsun\nsonucu yaz\n")
        .expect("HIR üretilmeli");
    let Cumle::Olsun { deger, .. } = &program.cumleler[0] else {
        panic!("olsun cümlesi bekleniyordu")
    };
    let bilgi = program
        .hir()
        .ifade_bilgisi(deger)
        .expect("bileşik ifade HIR bilgisi taşımalı");
    assert!(matches!(bilgi.kaynak_araligi(), HirKaynakAraligi::Satir { .. }));
    assert_eq!(bilgi.kaynak_araligi().satiri(), 1);
    assert_eq!(bilgi.kaynak_araligi().kesin_konumu(), None);
}

#[test]
fn hir_islem_ve_yapi_baglarini_kaynak_adindan_bagimsiz_dizinler() {
    let kaynak = r#"
yapı Kutu
    değer TamSayı

işlem bir ver
    1 döndür

kutu yeni Kutu olsun
sonuç bir ver olsun
"#;
    let program = kaynagi_fazli_derle(kaynak).expect("HIR üretilmeli");

    let Cumle::Olsun {
        deger:
            yapi @ Ifade::YeniYapi {
                yapi_kimligi: Some(yapi_kimligi),
                ..
            },
        ..
    } = &program.cumleler[0]
    else {
        panic!("yapı örneği bekleniyordu")
    };
    let Cumle::Olsun {
        deger:
            cagri @ Ifade::IslemCagrisi {
                islem_kimligi: Some(islem_kimligi),
                ..
            },
        ..
    } = &program.cumleler[1]
    else {
        panic!("işlem çağrısı bekleniyordu")
    };

    assert_eq!(
        program.hir().ifade_bilgisi(yapi).map(|b| b.bag()),
        Some(HirBagi::Yapi(*yapi_kimligi))
    );
    assert_eq!(
        program.hir().ifade_bilgisi(cagri).map(|b| b.bag()),
        Some(HirBagi::Islem(*islem_kimligi))
    );
    assert_eq!(
        program.hir().yapi(*yapi_kimligi).map(|y| y.ad.as_str()),
        Some("Kutu")
    );
    assert_eq!(
        program.hir().islem(*islem_kimligi).map(|i| i.ad.as_str()),
        Some("bir ver")
    );
}

#[test]
fn deger_dondurmeyen_cagri_cumlesi_hirda_acik_tur_tasir() {
    let kaynak = r#"
işlem selamla
    "merhaba" yaz

selamla
"#;
    let program = kaynagi_fazli_derle(kaynak).expect("HIR üretilmeli");
    let Cumle::CagriCumlesi { cagri, .. } = &program.cumleler[0] else {
        panic!("çağrı cümlesi bekleniyordu")
    };
    let bilgi = program
        .hir()
        .ifade_bilgisi(cagri)
        .expect("dönüşsüz çağrı da semantic HIR düğümüdür");
    assert_eq!(bilgi.tur(), HirIfadeTuru::DegerDondurmez);
    assert!(matches!(bilgi.bag(), HirBagi::Islem(_)));
}
