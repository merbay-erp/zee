use crate::agac::{Cumle, Ifade};

#[test]
fn bagli_runtime_degisken_icin_kaynak_adini_yeniden_cozmez() {
    let mut program =
        crate::kaynagi_fazli_derle("sayı 1 olsun\nsayıyı yaz\n").expect("HIR üretilmeli");
    let Cumle::Yaz { deger, .. } = &mut program.hir_mut().program_mut().cumleler[1] else {
        panic!("değişken ifadesi bekleniyordu")
    };
    let Ifade::Degisken { ham, cozulmus, .. } = deger.turu_mut() else {
        panic!("değişken ifadesi bekleniyordu")
    };
    *ham = "yanlış".into();
    *cozulmus = Some("yanlış".into());

    assert_eq!(
        crate::yorumlayici::calistir_baglanmis(&program).expect("HIR bağı yeterli olmalı"),
        vec!["1"]
    );
}

#[test]
fn bagli_runtime_islem_ve_yapi_kaynak_adini_yeniden_cozmez() {
    let kaynak = r#"
yapı Kutu
    değer TamSayı
işlem bir ver
    1 döndür
kutu yeni Kutu olsun
sonuç bir ver olsun
sonucu yaz
"#;
    let mut program = crate::kaynagi_fazli_derle(kaynak).expect("HIR üretilmeli");
    let cumleler = &mut program.hir_mut().program_mut().cumleler;
    let Cumle::Olsun { deger, .. } = &mut cumleler[0] else {
        panic!("yapı ifadesi bekleniyordu")
    };
    let Ifade::YeniYapi { yapi_adi, .. } = deger.turu_mut() else {
        panic!("yapı ifadesi bekleniyordu")
    };
    *yapi_adi = "Olmayan".into();
    let Cumle::Olsun { deger, .. } = &mut cumleler[1] else {
        panic!("çağrı ifadesi bekleniyordu")
    };
    let Ifade::IslemCagrisi { islem_adi, .. } = deger.turu_mut() else {
        panic!("çağrı ifadesi bekleniyordu")
    };
    *islem_adi = "olmayan işlem".into();

    assert_eq!(
        crate::yorumlayici::calistir_baglanmis(&program).expect("HIR bağları yeterli olmalı"),
        vec!["1"]
    );
}

#[test]
fn invariant_dogrulayici_cozulmus_ad_ve_symbol_id_bagini_korur() {
    let mut program =
        crate::kaynagi_fazli_derle("sayı 1 olsun\nsayıyı yaz\n").expect("HIR üretilmeli");
    let Cumle::Yaz { deger, .. } = &mut program.hir_mut().program_mut().cumleler[1] else {
        panic!("değişken ifadesi bekleniyordu")
    };
    let Ifade::Degisken { sembol_kimligi, .. } = deger.turu_mut() else {
        panic!("değişken ifadesi bekleniyordu")
    };
    *sembol_kimligi = None;

    let hata = program
        .invariantleri_dogrula()
        .expect_err("eksik SymbolId yakalanmalı");
    assert!(hata.mesaj().contains("SymbolId"), "{hata}");
}

#[test]
fn invariant_dogrulayici_yapi_ve_islem_kimliklerini_korur() {
    let kaynak = r#"
yapı Kutu
    değer TamSayı
işlem bir ver
    1 döndür
kutu yeni Kutu olsun
sonuç bir ver olsun
"#;

    let mut yapi_programi = crate::kaynagi_fazli_derle(kaynak).expect("HIR üretilmeli");
    let Cumle::Olsun { deger, .. } = &mut yapi_programi.hir_mut().program_mut().cumleler[0] else {
        panic!("yapı ifadesi bekleniyordu")
    };
    let Ifade::YeniYapi { yapi_kimligi, .. } = deger.turu_mut() else {
        panic!("yapı ifadesi bekleniyordu")
    };
    *yapi_kimligi = None;
    let hata = yapi_programi
        .invariantleri_dogrula()
        .expect_err("eksik YapiId yakalanmalı");
    assert!(hata.mesaj().contains("YapiId"), "{hata}");

    let mut islem_programi = crate::kaynagi_fazli_derle(kaynak).expect("HIR üretilmeli");
    let Cumle::Olsun { deger, .. } = &mut islem_programi.hir_mut().program_mut().cumleler[1] else {
        panic!("işlem çağrısı bekleniyordu")
    };
    let Ifade::IslemCagrisi { islem_kimligi, .. } = deger.turu_mut() else {
        panic!("işlem çağrısı bekleniyordu")
    };
    *islem_kimligi = None;
    let hata = islem_programi
        .invariantleri_dogrula()
        .expect_err("eksik IslemId yakalanmalı");
    assert!(hata.mesaj().contains("IslemId"), "{hata}");
}
