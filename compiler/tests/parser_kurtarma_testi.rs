use dil::agac::Cumle;
use dil::faz::KaynakMetni;

fn kurtarmali(kaynak: &str) -> (dil::faz::AyristirilmisAst, Vec<dil::tani::Tani>) {
    KaynakMetni::yeni(kaynak)
        .sozcukle()
        .expect("lexer")
        .ayristir_kurtarmali(Vec::new())
}

#[test]
fn ic_blok_hatasi_gecerli_kardes_cumleyi_bloktan_disari_sizdirmaz() {
    let kaynak = "2 kez tekrarla\n    tanınmayan iç\n    \"iç\" yaz\n\n\"dış\" yaz\n";
    let (ast, tanilar) = kurtarmali(kaynak);

    assert_eq!(tanilar.len(), 1, "{tanilar:?}");
    assert_eq!((tanilar[0].kod.as_str(), tanilar[0].satir), ("S004", 2));
    assert_eq!(ast.cumleler().len(), 2);
    let Cumle::KezTekrarla { govde, .. } = &ast.cumleler()[0] else {
        panic!("ilk cümle döngü olmalı")
    };
    assert_eq!(govde.len(), 1, "geçerli iç cümle döngüde kalmalı");
    assert!(matches!(govde[0], Cumle::Yaz { .. }));
    assert!(matches!(ast.cumleler()[1], Cumle::Yaz { .. }));
}

#[test]
fn bozuk_yapi_alani_sonraki_alani_ve_ust_cumleyi_yutmaz() {
    let kaynak = "yapı Kutu\n    bozuk alan satırı\n    değer TamSayı\n\n\"devam\" yaz\n";
    let (ast, tanilar) = kurtarmali(kaynak);

    assert_eq!(tanilar.len(), 1, "{tanilar:?}");
    assert_eq!((tanilar[0].kod.as_str(), tanilar[0].satir), ("S025", 2));
    assert_eq!(ast.cumleler().len(), 2);
    let Cumle::YapiTanimi(yapi) = &ast.cumleler()[0] else {
        panic!("yapı tanımı korunmalı")
    };
    assert_eq!(yapi.alanlar, vec![("değer".into(), "TamSayı".into())]);
    assert!(matches!(ast.cumleler()[1], Cumle::Yaz { .. }));
}

#[test]
fn ic_hata_derinligi_sizdirip_sonraki_ust_tanimi_bozmaz() {
    let kaynak = "2 kez tekrarla\n    tanınmayan iç\n\nişlem bir ver\n    1 döndür\n";
    let (ast, tanilar) = kurtarmali(kaynak);

    assert_eq!(tanilar.len(), 1, "yanlış S021 üretilmemeli: {tanilar:?}");
    assert_eq!(tanilar[0].kod, "S004");
    assert!(matches!(ast.cumleler()[1], Cumle::IslemTanimi(_)));
}

#[test]
fn tani_seli_kaynak_sirasinda_yirmi_kayitla_sinirlanir() {
    let kaynak = (1..=25)
        .map(|sira| format!("tanınmayan satır {sira}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut yukleyici = |_: &str| Err("yok".to_string());
    let tanilar = dil::kaynagi_tanilari(&kaynak, &mut yukleyici);

    assert_eq!(tanilar.len(), 20, "{tanilar:?}");
    assert!(tanilar.iter().all(|tani| tani.kod == "S004"));
    assert_eq!(
        tanilar.iter().map(|tani| tani.satir).collect::<Vec<_>>(),
        (1..=20).collect::<Vec<_>>()
    );
}

#[test]
fn bozuk_eszamanli_gorev_sonraki_gecerli_gorevi_korur() {
    let kaynak = "işlem bir ver\n    1 döndür\n\neşzamanlı olarak\n    bozuk\n    görev bir ver\n";
    let (ast, tanilar) = kurtarmali(kaynak);

    assert_eq!(tanilar.len(), 1, "{tanilar:?}");
    assert_eq!((tanilar[0].kod.as_str(), tanilar[0].satir), ("S038", 5));
    let Cumle::Eszamanli { gorevler, .. } = &ast.cumleler()[1] else {
        panic!("eşzamanlı blok korunmalı")
    };
    assert_eq!(gorevler.len(), 1);
    assert_eq!(gorevler[0].0, "görev");
}

#[test]
fn bozuk_gore_kolu_kendi_govdesini_atlayip_sonraki_kolu_korur() {
    let kaynak = "şekil \"kare\" olsun\nşekle göre\n    bozuk kol\n        \"bu gövde yutulmalı\" yaz\n    \"kare\" ise\n        \"geçerli\" yaz\n\n\"dış\" yaz\n";
    let (ast, tanilar) = kurtarmali(kaynak);

    assert_eq!(tanilar.len(), 1, "{tanilar:?}");
    assert_eq!((tanilar[0].kod.as_str(), tanilar[0].satir), ("S024", 3));
    let Cumle::Gore { kollar, .. } = &ast.cumleler()[1] else {
        panic!("göre cümlesi korunmalı")
    };
    assert_eq!(kollar.len(), 1);
    assert!(matches!(ast.cumleler()[2], Cumle::Yaz { .. }));
}
