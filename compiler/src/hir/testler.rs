use crate::agac::{Cumle, Ifade};

#[test]
fn bagli_runtime_degisken_icin_kaynak_adini_yeniden_cozmez() {
    let mut program =
        crate::kaynagi_fazli_derle("sayı 1 olsun\nsayıyı yaz\n").expect("HIR üretilmeli");
    let Cumle::Yaz {
        deger: Ifade::Degisken { ham, cozulmus, .. },
        ..
    } = &mut program.hir_mut().program_mut().cumleler[1]
    else {
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
    let Cumle::Olsun {
        deger: Ifade::YeniYapi { yapi_adi, .. },
        ..
    } = &mut cumleler[0]
    else {
        panic!("yapı ifadesi bekleniyordu")
    };
    *yapi_adi = "Olmayan".into();
    let Cumle::Olsun {
        deger: Ifade::IslemCagrisi { islem_adi, .. },
        ..
    } = &mut cumleler[1]
    else {
        panic!("çağrı ifadesi bekleniyordu")
    };
    *islem_adi = "olmayan işlem".into();

    assert_eq!(
        crate::yorumlayici::calistir_baglanmis(&program).expect("HIR bağları yeterli olmalı"),
        vec!["1"]
    );
}
