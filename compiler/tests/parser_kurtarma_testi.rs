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

#[test]
fn girintisiz_blok_basliklari_s007_ve_oneri_tasir() {
    for kaynak in [
        "işlem karesini hesapla\n",
        "yapı Kutu\n",
        "şekle göre\n",
        "3 kez tekrarla\n",
    ] {
        let (_, tanilar) = kurtarmali(kaynak);
        assert_eq!(tanilar.len(), 1, "{kaynak:?}: {tanilar:?}");
        assert_eq!(tanilar[0].kod, "S007", "{kaynak:?}");
        assert!(
            tanilar[0]
                .oneri
                .as_deref()
                .is_some_and(|o| o.contains("4 boşluk")),
            "{kaynak:?}: S007 girinti önerisi taşımalı: {:?}",
            tanilar[0].oneri
        );
    }
}

#[test]
fn govde_disina_dusen_parametre_satiri_oturum_kalibiyla_karistirilmaz() {
    for kaynak in ["sayıyı al\n", "sayıyı TamSayı olarak al\n"] {
        let (_, tanilar) = kurtarmali(kaynak);
        assert_eq!(tanilar.len(), 1, "{kaynak:?}: {tanilar:?}");
        assert_eq!((tanilar[0].kod.as_str(), tanilar[0].satir), ("S004", 1));
        assert!(
            tanilar[0]
                .oneri
                .as_deref()
                .is_some_and(|o| o.contains("işlem başlığının altına")),
            "{:?}",
            tanilar[0].oneri
        );
    }
    let (_, tanilar) = kurtarmali("\"ali\" kullanıcısını \"yönetici\" rolüyle oturuma al\n");
    assert!(
        tanilar.is_empty(),
        "gerçek oturum cümlesi korunur: {tanilar:?}"
    );
    let (_, tanilar) = kurtarmali("bir iki üç al\n");
    assert_eq!(
        tanilar[0].kod, "S043",
        "oturum kalıbına yakın bozuk satır S043 kalır"
    );
}

#[test]
fn ayni_tanimsiz_ad_coklu_tani_hattinda_bir_kez_raporlanir() {
    let mut yukleyici = |_: &str| -> Result<String, String> { Err("birim yok".into()) };
    let tanilar = dil::kaynagi_tanilari("isim yaz\nisim yaz\nisim ile \"!\" yaz\n", &mut yukleyici);
    assert_eq!(tanilar.len(), 1, "{tanilar:?}");
    assert_eq!((tanilar[0].kod.as_str(), tanilar[0].satir), ("A001", 1));

    let tanilar = dil::kaynagi_tanilari("a yaz\nb yaz\na yaz\n", &mut yukleyici);
    assert_eq!(
        tanilar
            .iter()
            .map(|t| (t.kod.as_str(), t.satir))
            .collect::<Vec<_>>(),
        [("A001", 1), ("A001", 2)],
        "farklı tanımsız adlar ayrı raporlanır"
    );

    let tanilar = dil::kaynagi_tanilari(
        "ayşe yeni Öğrenci olsun\nayşenin adı yaz\nayşenin yaşı yaz\n",
        &mut yukleyici,
    );
    assert_eq!(
        tanilar.iter().map(|t| t.kod.as_str()).collect::<Vec<_>>(),
        ["A007"],
        "yapı tanımsızken ona bağlı ad tanımının kullanımları kök nedeni tekrar etmez: {tanilar:?}"
    );
}

#[test]
fn ekli_bicimler_ortuk_cogul_ve_bozuk_islem_cagrilari_kok_nedene_baglanir() {
    let mut yukleyici = |_: &str| -> Result<String, String> { Err("birim yok".into()) };
    let tanilar = dil::kaynagi_tanilari(
        "cümle \"merhaba\"\ncümlenin harfleri yaz\ncümle yaz\nher kelime için\n    kelime yaz\n",
        &mut yukleyici,
    );
    assert_eq!(
        tanilar
            .iter()
            .map(|t| (t.kod.as_str(), t.satir))
            .collect::<Vec<_>>(),
        [("S004", 1), ("A003", 4)],
        "düşen tanım satırının baş sözü ekli biçimleriyle bastırılır; farklı kök ayrı raporlanır: {tanilar:?}"
    );

    let tanilar = dil::kaynagi_tanilari(
        "işlem selamla\n\"Ayşe\" için selamla\n\"Ali\" için selamla\n",
        &mut yukleyici,
    );
    assert_eq!(
        tanilar
            .iter()
            .map(|t| (t.kod.as_str(), t.satir))
            .collect::<Vec<_>>(),
        [("S007", 1)],
        "başlığı bozuk işlemin çağrıları iç hata tanısı üretmez: {tanilar:?}"
    );
}

#[test]
fn basarisiz_tanimin_kullanimlari_kok_nedeni_tekrar_etmez() {
    let mut yukleyici = |_: &str| -> Result<String, String> { Err("birim yok".into()) };
    let tanilar = dil::kaynagi_tanilari(
        "birinci yanıtın sayısı\ntoplam birinci ile 2 nin toplamı olsun\ntoplam yaz\nbirinci yaz\n",
        &mut yukleyici,
    );
    assert_eq!(
        tanilar
            .iter()
            .map(|t| (t.kod.as_str(), t.satir))
            .collect::<Vec<_>>(),
        [("S004", 1)],
        "düşen satırın baş sözü ve başarısız tanım kök nedene bağlanır: {tanilar:?}"
    );
    let tanilar = dil::kaynagi_tanilari("sayı 5 olsun\nbaşka yaz\nsayı yaz\n", &mut yukleyici);
    assert_eq!(
        tanilar
            .iter()
            .map(|t| (t.kod.as_str(), t.satir))
            .collect::<Vec<_>>(),
        [("A001", 2)],
        "geçerli tanım korunur, gerçek tanımsız ad raporlanır"
    );
}
