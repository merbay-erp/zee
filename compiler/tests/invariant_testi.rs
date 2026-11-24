use dil::agac::{AstKaynakAraligi, Cumle, Ifade};
use dil::faz::KaynakMetni;
use dil::invariant::{ayristirilmis_ast_dogrula, InvariantFazi};

fn kaynakli(ifade: Ifade, sutun: usize, uzunluk: usize) -> Ifade {
    Ifade::kaynakli(
        ifade,
        AstKaynakAraligi::yeni(1, sutun, uzunluk).expect("geçerli test aralığı"),
    )
}

#[test]
fn gecerli_parser_astsi_ve_bagli_hir_invariantlerden_gecer() {
    let kaynak = "\
yapı Ürün
    adet TamSayı

kalem yeni Ürün olsun
kalemin adedi 7 olsun
kalemin adedi yaz
";
    let ast = KaynakMetni::yeni(kaynak)
        .sozcukle()
        .expect("lexer")
        .ayristir(Vec::new())
        .expect("parser");
    ast.invariantleri_dogrula().expect("parser değişmezleri");

    let program = dil::kaynagi_fazli_derle(kaynak).expect("bağlanmış HIR");
    program
        .invariantleri_dogrula()
        .expect("bağlanmış değişmezler");
}

#[test]
fn parser_astsi_checker_semantik_bagi_tasiyamaz() {
    let cumleler = vec![Cumle::Yaz {
        deger: kaynakli(
            Ifade::Degisken {
                ham: "sayıyı".into(),
                cozulmus: Some("sayı".into()),
                sembol_kimligi: None,
                satir: 1,
                sutun: 1,
                uzunluk: 6,
            },
            1,
            6,
        ),
        satir: 1,
    }];

    let hata = ayristirilmis_ast_dogrula(&cumleler).expect_err("semantic alan reddedilmeli");
    assert_eq!(hata.faz(), InvariantFazi::AyristirilmisAst);
    assert!(hata.mesaj().contains("çözülmüş ad/SymbolId"), "{hata}");
    assert!(hata.yol().contains("ifade"), "{hata}");
}

#[test]
fn imkansiz_ifade_ve_cagri_cumlesi_durumlari_reddedilir() {
    let tek_parcali = vec![Cumle::Yaz {
        deger: kaynakli(
            Ifade::Birlestir(vec![kaynakli(Ifade::MetinSabiti("tek".into()), 1, 5)]),
            1,
            5,
        ),
        satir: 1,
    }];
    let hata = ayristirilmis_ast_dogrula(&tek_parcali).expect_err("tek parçalı zincir");
    assert!(hata.mesaj().contains("en az iki parça"), "{hata}");

    let cagri_olmayan = vec![Cumle::CagriCumlesi {
        cagri: kaynakli(Ifade::MetinSabiti("çağrı değil".into()), 1, 13),
        satir: 1,
    }];
    let hata = ayristirilmis_ast_dogrula(&cagri_olmayan).expect_err("imkânsız çağrı cümlesi");
    assert!(hata.mesaj().contains("işlem çağrısı taşımıyor"), "{hata}");
}

#[test]
fn parser_astsi_kesin_kaynak_zarfi_olmadan_ifade_kabul_etmez() {
    let cumleler = vec![Cumle::Yaz {
        deger: Ifade::MetinSabiti("zarfsız".into()),
        satir: 1,
    }];
    let hata = ayristirilmis_ast_dogrula(&cumleler).expect_err("kaynak zarfı zorunlu");
    assert!(hata.mesaj().contains("kesin kaynak aralığı yok"), "{hata}");
}

#[test]
fn cagrilmayan_cikarimli_islem_govdesi_parser_astsi_olarak_kalir() {
    // K-392: çıkarımlı parametre türü yalnız çağrı kısıtından gelir (spec/04).
    // Hiç çağrılmayan işlemin gövdesi bağlanmaz; HIR değişmezi C000 vermez,
    // çağrılan komşu işlemin gövdesi ise bağlı HIR kurallarıyla denetlenir.
    let kaynak = "\
işlem birini ver
    sayıyı al
    sayıyı döndür

işlem geri say
    sayıyı al
    sayı 0 a eşitse
        0 döndür
    önceki sayı ile 1 in farkı olsun
    önceki için geri say döndür

işlem dış hesapla
    sayıyı al
    sayı için iç hesapla döndür

işlem iç hesapla
    değeri al
    değeri döndür

işlem karesini hesapla
    sayıyı al
    sonucu sayı ile sayının çarpımı olsun
    sonucu döndür

kare 4 için karesini hesapla olsun
kare yaz
";
    let program = dil::kaynagi_fazli_derle(kaynak).expect("bağlanmış HIR");
    program
        .invariantleri_dogrula()
        .expect("çağrılmayan çıkarımlı gövde C000 üretmemeli");
    let hir = program.hir();
    let islemler = &hir.program().islemler;
    let Cumle::Dondur { deger, .. } = &islemler["birini ver"].govde[0] else {
        panic!("birini ver gövdesi döndür ile başlamalı");
    };
    assert!(
        hir.ifade_bilgisi(deger).is_none(),
        "çağrılmayan gövde HIR kaydı taşımamalı"
    );
    let Cumle::Dondur { deger, .. } = &islemler["karesini hesapla"].govde[1] else {
        panic!("karesini hesapla dönüşü ikinci cümle olmalı");
    };
    assert!(
        hir.ifade_bilgisi(deger).is_some(),
        "çağrılan gövde HIR kaydı taşımalı"
    );
}

#[test]
fn cagrilmayan_cikarimli_govde_hatasi_ilk_cagriyla_raporlanir() {
    // K-392: çağrısız gövde denetlenmez; testten gelen ilk çağrı ad hatasını
    // kullanıcı tanısı olarak verir (C000 değil).
    let govde = "işlem birini ver\n    sayıyı al\n    bilinmeyen döndür\n\n";
    dil::kaynagi_fazli_derle(&format!("{govde}1 yaz\n"))
        .expect("çağrısız gövde denetlenmez")
        .invariantleri_dogrula()
        .expect("çağrısız gövde C000 üretmez");
    let hata = match dil::kaynagi_fazli_derle(&format!(
        "{govde}test \"t\"\n    x 1 için birini ver olsun\n    x 1 e eşit olmalı\n"
    )) {
        Ok(_) => panic!("ilk çağrı gövdeyi denetlemeli"),
        Err(hata) => hata,
    };
    assert_eq!(hata.kod, "A001");
    assert_eq!(hata.satir, 3);
}
