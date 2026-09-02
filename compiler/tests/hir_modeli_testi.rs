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
    } = deger.turu()
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
fn hir_sembol_tanimini_ve_butun_yazimlarini_symbolid_ile_tasir() {
    let program =
        kaynagi_fazli_derle("puan 1 olsun\npuan 2 olsun\npuanı yaz\n").expect("HIR üretilmeli");
    let Cumle::Yaz { deger, .. } = &program.cumleler[2] else {
        panic!("yaz cümlesi bekleniyordu")
    };
    let bilgi = program.hir().ifade_bilgisi(deger).expect("HIR sembol bağı");
    let HirBagi::Sembol(kimlik) = bilgi.bag() else {
        panic!("SymbolId bekleniyordu")
    };

    assert_eq!(
        program
            .hir()
            .sembol_tanimi(kimlik)
            .and_then(HirKaynakAraligi::kesin_konumu),
        Some((1, 1, 4))
    );
    let kullanimlar = program
        .hir()
        .sembol_kullanimlari()
        .into_iter()
        .filter(|kullanim| kullanim.kimlik() == kimlik)
        .collect::<Vec<_>>();
    assert_eq!(kullanimlar.len(), 3, "iki yazım + bir okuma korunmalı");
    assert_eq!(
        kullanimlar
            .iter()
            .map(|kullanim| kullanim.kaynak_araligi().satiri())
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn her_hir_ifadesi_astden_gelen_kesin_kaynak_araligi_tasir() {
    let program = kaynagi_fazli_derle("sonuç 1 ile 2 nin toplamı olsun\nsonucu yaz\n")
        .expect("HIR üretilmeli");
    let Cumle::Olsun { deger, .. } = &program.cumleler[0] else {
        panic!("olsun cümlesi bekleniyordu")
    };
    let bilgi = program
        .hir()
        .ifade_bilgisi(deger)
        .expect("bileşik ifade HIR bilgisi taşımalı");
    assert_eq!(bilgi.kaynak_araligi().kesin_konumu(), Some((1, 7, 19)));
}

#[test]
fn checker_tanisi_satir_zarfi_yerine_ast_ifadesini_isaretler() {
    let hata = dil::kaynagi_derle("sonuç \"x\" ile 1 nin toplamı olsun\n")
        .expect_err("Metin ile sayı toplanmamalı");
    assert_eq!(hata.kod, "T008");
    assert_eq!(hata.satir, 1);
    assert_eq!(hata.sutun, 7);
    assert!(hata.uzunluk > 1, "bileşik ifade bütünü işaretlenmeli: {hata:?}");
}

#[test]
fn ast_bilesik_ve_yaprak_ifadelerin_ayri_kesin_araliklarini_korur() {
    let program = dil::kaynagi_derle("sonuç 1 ile 2 nin toplamı olsun\n").expect("derlenmeli");
    let Cumle::Olsun { deger, .. } = &program.cumleler[0] else {
        panic!("değer tanımı bekleniyordu")
    };
    assert_eq!(deger.kaynak_araligi().map(|a| a.uclu()), Some((1, 7, 19)));
    let Ifade::Aritmetik { sol, sag, .. } = deger.turu() else {
        panic!("aritmetik ifade bekleniyordu")
    };
    assert_eq!(sol.kaynak_araligi().map(|a| a.uclu()), Some((1, 7, 1)));
    assert_eq!(sag.kaynak_araligi().map(|a| a.uclu()), Some((1, 13, 1)));
}

#[test]
fn ortuk_cogul_ifadesi_uydurma_bir_bir_yerine_dongu_adina_baglanir() {
    let kaynak = "sayılar 1, 2 listesi olsun\nher sayı için\n    sayıyı yaz\n";
    let program = kaynagi_fazli_derle(kaynak).expect("örtük çoğul bağlanmalı");
    let Cumle::HerBiri {
        kaynak: Some(kaynak),
        ..
    } = &program.cumleler[1]
    else {
        panic!("koleksiyon döngüsü bekleniyordu")
    };
    assert_eq!(kaynak.kaynak_araligi().map(|a| a.uclu()), Some((2, 5, 4)));
    assert_eq!(
        program
            .hir()
            .ifade_bilgisi(kaynak)
            .and_then(|bilgi| bilgi.kaynak_araligi().kesin_konumu()),
        Some((2, 5, 4))
    );
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

    let Cumle::Olsun { deger: yapi, .. } = &program.cumleler[0]
    else {
        panic!("yapı örneği bekleniyordu")
    };
    let Ifade::YeniYapi {
        yapi_kimligi: Some(yapi_kimligi),
        ..
    } = yapi.turu()
    else {
        panic!("yapı örneği bekleniyordu")
    };
    let Cumle::Olsun { deger: cagri, .. } = &program.cumleler[1]
    else {
        panic!("işlem çağrısı bekleniyordu")
    };
    let Ifade::IslemCagrisi {
        islem_kimligi: Some(islem_kimligi),
        ..
    } = cagri.turu()
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
