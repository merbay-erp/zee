use dil::agac::{Cumle, Ifade};
use dil::faz::KaynakMetni;
use dil::invariant::{ayristirilmis_ast_dogrula, InvariantFazi};

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
        deger: Ifade::Degisken {
            ham: "sayıyı".into(),
            cozulmus: Some("sayı".into()),
            sembol_kimligi: None,
            satir: 1,
            sutun: 1,
            uzunluk: 6,
        },
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
        deger: Ifade::Birlestir(vec![Ifade::MetinSabiti("tek".into())]),
        satir: 1,
    }];
    let hata = ayristirilmis_ast_dogrula(&tek_parcali).expect_err("tek parçalı zincir");
    assert!(hata.mesaj().contains("en az iki parça"), "{hata}");

    let cagri_olmayan = vec![Cumle::CagriCumlesi {
        cagri: Ifade::MetinSabiti("çağrı değil".into()),
        satir: 1,
    }];
    let hata = ayristirilmis_ast_dogrula(&cagri_olmayan).expect_err("imkânsız çağrı cümlesi");
    assert!(hata.mesaj().contains("işlem çağrısı taşımıyor"), "{hata}");
}
