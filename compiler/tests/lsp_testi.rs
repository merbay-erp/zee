//! dillsp çekirdeği testleri: JSON ayrıştırıcı + mesaj döngüsü.

use dil::lsp::{json_coz, Json, Sunucu, AZAMI_LSP_JSON_DERINLIGI, AZAMI_LSP_JSON_DUGUMU};

#[test]
fn json_ayristirici_temel() {
    let json = json_coz(r#"{"a": 1, "b": [true, null, "me\"tin"], "c": {"iç": -2.5}}"#)
        .expect("çözülmeli");
    assert_eq!(
        json.alan("b"),
        Some(&Json::Dizi(vec![
            Json::Mantik(true),
            Json::Bos,
            Json::Metin("me\"tin".into())
        ]))
    );
    let Json::Sayi(sayi) = json
        .alan("c")
        .and_then(|c| c.alan("iç"))
        .expect("sayı alanı")
    else {
        panic!("sayı bekleniyordu");
    };
    assert_eq!(sayi.ham(), "-2.5");
}

#[test]
fn json_sayisi_rfc_8259_durum_makinesinden_gecer() {
    let gecerli = [
        "0",
        "-0",
        "7",
        "-42",
        "0.25",
        "10.0",
        "1e9",
        "1E+9",
        "1e-9",
        "123456789012345678901234567890",
        "1e400",
    ];
    for metin in gecerli {
        let Json::Sayi(sayi) = json_coz(metin).expect("geçerli RFC 8259 sayısı") else {
            panic!("sayı bekleniyordu: {metin}");
        };
        assert_eq!(sayi.ham(), metin, "lexeme kayıpsız korunmalı");
    }

    for metin in [
        "+1",
        "01",
        "-01",
        ".1",
        "1.",
        "1e",
        "1e+",
        "--1",
        "1+2",
        "NaN",
        "Infinity",
        "-Infinity",
    ] {
        assert!(json_coz(metin).is_none(), "reddedilmeli: {metin}");
    }
}

#[test]
fn json_sayi_korpusu_serde_json_ile_ayni_karari_verir() {
    for metin in [
        "0", "-0", "12", "-12", "0.5", "12.50", "1e4", "1E+4", "1e-4", "+1", "01", "-01", ".1",
        "1.", "1e", "1e+", "--1", "1+2", "NaN", "Infinity",
    ] {
        assert_eq!(
            json_coz(metin).is_some(),
            serde_json::from_str::<serde_json::Value>(metin).is_ok(),
            "differential sayı kararı: {metin}"
        );
    }
}

#[test]
fn json_yinelenen_ve_kacisla_esdeger_nesne_anahtarini_reddeder() {
    for metin in [
        r#"{"id":1,"id":2}"#,
        r#"{"method":1,"\u006dethod":2}"#,
        r#"{"dış":{"ad":1,"ad":2}}"#,
    ] {
        assert!(json_coz(metin).is_none(), "reddedilmeli: {metin}");
    }
}

#[test]
fn json_unicode_kacislari() {
    // Türkçe karakter + vekil çift (emoji).
    let json = json_coz(r#""ğ 😀""#).expect("çözülmeli");
    assert_eq!(json, Json::Metin("ğ 😀".into()));
}

#[test]
fn json_gecersiz_vekil_ciftlerini_reddeder() {
    assert!(json_coz(r#""\uD800\u0041""#).is_none());
    assert!(json_coz(r#""\uD800""#).is_none());
    assert!(json_coz(r#""\uDC00""#).is_none());
    assert_eq!(
        json_coz(r#""\uD83D\uDE00""#),
        Some(Json::Metin("😀".into()))
    );
}

#[test]
fn json_metin_icinde_kacissiz_kontrol_karakterini_reddeder() {
    let metin = format!("\"ön{}arka\"", '\u{0001}');
    assert!(json_coz(&metin).is_none());
}

#[test]
fn json_derinlik_butcesini_asmadan_durur() {
    let adet = AZAMI_LSP_JSON_DERINLIGI + 1;
    let metin = format!("{}null{}", "[".repeat(adet), "]".repeat(adet));
    assert!(json_coz(&metin).is_none());
}

#[test]
fn json_dugum_butcesini_asmadan_durur() {
    let mut metin = String::from("[");
    for sira in 0..AZAMI_LSP_JSON_DUGUMU {
        if sira > 0 {
            metin.push(',');
        }
        metin.push_str("null");
    }
    metin.push(']');
    assert!(json_coz(&metin).is_none());
}

#[test]
fn initialize_yaniti() {
    let mut sunucu = Sunucu::yeni();
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
    assert!(cikti.devam);
    assert_eq!(cikti.govdeler.len(), 1);
    assert!(cikti.govdeler[0].contains("\"textDocumentSync\":1"));
    assert!(cikti.govdeler[0].contains("dillsp"));
}

#[test]
fn json_rpc_parse_error_ile_invalid_request_ayrilir() {
    let mut sunucu = Sunucu::yeni();
    for bozuk in ["{", r#"{"jsonrpc":"2.0",,"method":"initialize"}"#] {
        let cikti = sunucu.mesaj_isle(bozuk);
        assert_eq!(cikti.govdeler.len(), 1);
        assert!(cikti.govdeler[0].contains("\"code\":-32700"));
        assert!(cikti.govdeler[0].contains("\"id\":null"));
    }

    for gecersiz in [
        r#"{}"#,
        r#"[]"#,
        r#"{"jsonrpc":"1.0","id":1,"method":"initialize"}"#,
        r#"{"jsonrpc":"2.0","id":1,"method":7}"#,
        r#"{"jsonrpc":"2.0","id":true,"method":"initialize"}"#,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":null}"#,
    ] {
        let cikti = sunucu.mesaj_isle(gecersiz);
        assert_eq!(cikti.govdeler.len(), 1, "{gecersiz}");
        assert!(cikti.govdeler[0].contains("\"code\":-32600"), "{gecersiz}");
        assert!(cikti.govdeler[0].contains("\"id\":null"), "{gecersiz}");
    }
}

#[test]
fn json_rpc_kimligi_kayipsiz_doner_ve_bilinmeyen_yontem_ayrilir() {
    let mut sunucu = Sunucu::yeni();
    for kimlik in ["1.25", "1e400", "123456789012345678901234567890"] {
        let istek = format!(r#"{{"jsonrpc":"2.0","id":{kimlik},"method":"zee/bilinmeyen"}}"#);
        let cikti = sunucu.mesaj_isle(&istek);
        assert_eq!(cikti.govdeler.len(), 1);
        assert!(cikti.govdeler[0].contains(&format!("\"id\":{kimlik}")));
        assert!(cikti.govdeler[0].contains("\"code\":-32601"));
        assert!(json_coz(&cikti.govdeler[0]).is_some());
    }

    let bildirim = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"zee/bilinmeyen"}"#);
    assert!(bildirim.govdeler.is_empty());
}

#[test]
fn json_rpc_bildirimine_yanit_vermez_ve_gecersiz_parametreyi_ayirir() {
    let mut sunucu = Sunucu::yeni();
    let bildirim = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"initialize","params":{}}"#);
    assert!(bildirim.govdeler.is_empty());

    let cikti =
        sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":9,"method":"textDocument/hover","params":{}}"#);
    assert_eq!(cikti.govdeler.len(), 1);
    assert!(cikti.govdeler[0].contains("\"id\":9"));
    assert!(cikti.govdeler[0].contains("\"code\":-32602"));
}

#[test]
fn utf8_olmayan_govde_parse_error_uretir_ve_sunucuyu_durdurmaz() {
    let mut sunucu = Sunucu::yeni();
    let cikti = sunucu.mesaj_baytlari_isle(&[0xff, 0xfe]);
    assert!(cikti.devam);
    assert_eq!(cikti.govdeler.len(), 1);
    assert!(cikti.govdeler[0].contains("\"code\":-32700"));
}

#[test]
fn butun_uretim_yollari_gecerli_ve_butceli_json_verir() {
    let mut sunucu = Sunucu::yeni();
    let istekler = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/butce.dil","text":"sayaç 1 olsun\nsayacı yaz\n"}}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tmp/butce.dil"},"position":{"line":1,"character":2}}}"#,
        r#"{"jsonrpc":"2.0","id":4,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///tmp/butce.dil"},"position":{"line":1,"character":2}}}"#,
        r#"{"jsonrpc":"2.0","id":5,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/butce.dil"},"position":{"line":0,"character":2},"newName":"puan"}}"#,
        r#"{"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":"file:///tmp/butce.dil"}}}"#,
        r#"{"jsonrpc":"2.0","id":6,"method":"shutdown"}"#,
    ];

    for istek in istekler {
        for govde in sunucu.mesaj_isle(istek).govdeler {
            assert!(json_coz(&govde).is_some(), "geçersiz LSP JSON'u: {govde}");
            assert!(
                govde.len() <= dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.lsp_yanit_bayti()
            );
        }
    }
}

#[test]
fn didopen_tanilari_yayinlar() {
    let mut sunucu = Sunucu::yeni();
    let mesaj = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/deneme.dil","languageId":"dil","version":1,"text":"bilinmeyeni yaz\n"}}}"#;
    let cikti = sunucu.mesaj_isle(mesaj);
    assert_eq!(cikti.govdeler.len(), 1);
    let yayin = &cikti.govdeler[0];
    assert!(yayin.contains("publishDiagnostics"), "{}", yayin);
    assert!(yayin.contains("\"code\":\"A001\""), "{}", yayin);
    assert!(yayin.contains("\"line\":0"), "{}", yayin);
}

#[test]
fn didchange_temiz_metinle_tanilari_sifirlar() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/d.dil","text":"bozuk"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///tmp/d.dil"},"contentChanges":[{"text":"\"selam\" yaz\n"}]}}"#);
    assert!(
        cikti.govdeler[0].contains("\"diagnostics\":[]"),
        "{}",
        cikti.govdeler[0]
    );
}

#[test]
fn lsp_girinti_sinirinda_kurtarip_bagimsiz_tanilari_sirayla_yayinlar() {
    let mut sunucu = Sunucu::yeni();
    let mesaj = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/kurtarma.dil","text":"yapı Kutu\n    bozuk alan satırı\n    değer TamSayı\nbozuk dış\n"}}}"#;
    let cikti = sunucu.mesaj_isle(mesaj);
    let yayin = &cikti.govdeler[0];

    assert_eq!(yayin.matches("\"code\"").count(), 2, "{yayin}");
    let yapi = yayin.find("\"code\":\"S025\"").expect("yapı alanı tanısı");
    let dis = yayin.find("\"code\":\"S004\"").expect("dış cümle tanısı");
    assert!(yapi < dis, "tanılar kaynak sırasında olmalı: {yayin}");
    assert!(yayin.contains("\"line\":1"), "{yayin}");
    assert!(yayin.contains("\"line\":3"), "{yayin}");
    assert!(!yayin.contains("\"code\":\"C000\""), "{yayin}");
}

#[test]
fn completion_kalip_kelimeleri() {
    let mut sunucu = Sunucu::yeni();
    let cikti = sunucu
        .mesaj_isle(r#"{"jsonrpc":"2.0","id":7,"method":"textDocument/completion","params":{}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("\"id\":7"));
    assert!(yanit.contains("\"label\":\"olsun\""));
    assert!(yanit.contains("\"label\":\"değilse\""));
    assert!(yanit.contains("\"label\":\"doğrulanıyorsa\""));
    assert!(yanit.contains("\"label\":\"yetkisi\""));
}

#[test]
fn exit_dongusu_durdurur() {
    let mut sunucu = Sunucu::yeni();
    assert!(
        sunucu
            .mesaj_isle(r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#)
            .devam
    );
    assert!(
        !sunucu
            .mesaj_isle(r#"{"jsonrpc":"2.0","method":"exit"}"#)
            .devam
    );
}

#[test]
fn hover_kalip_kelimesini_aciklar() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/h.dil","text":"\"selam\" yaz\n"}}}"#);
    // 0. satır, 8. karakter → "yaz" kelimesi.
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":9,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tmp/h.dil"},"position":{"line":0,"character":8}}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("**yaz**"), "{}", yanit);
    assert!(yanit.contains("markdown"), "{}", yanit);
}

#[test]
fn hover_kullanici_tanimini_gosterir() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/h2.dil","text":"sayaç 3 olsun\nsayacı yaz\n"}}}"#);
    // 1. satır, 2. karakter → "sayacı" — tanımı 0. satırdaki "sayaç".
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":10,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tmp/h2.dil"},"position":{"line":1,"character":2}}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("sayaç 3 olsun"), "{}", yanit);
}

#[test]
fn tanima_git_islem_basligina_gider() {
    let mut sunucu = Sunucu::yeni();
    let ac = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/t.dil","text":"işlem karesini hesapla\n    sayıyı al\n    sonucu sayı ile sayının çarpımı olsun\n    sonucu döndür\n\nkare 4 için karesini hesapla olsun\n"}}}"#;
    sunucu.mesaj_isle(ac);
    // 5. satır "kare 4 için karesini hesapla olsun" — 21. karakter "hesapla" içinde.
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":11,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///tmp/t.dil"},"position":{"line":5,"character":21}}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(
        yanit.contains("\"line\":0"),
        "işlem başlığına gitmeli: {}",
        yanit
    );
}

#[test]
fn tanima_git_ekli_degiskeni_cozer() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/t2.dil","text":"sayaç 3 olsun\nsayacı 1 azalt\n"}}}"#);
    // 1. satır "sayacı" (ek almış kullanım) → 0. satırdaki "sayaç ... olsun".
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":12,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///tmp/t2.dil"},"position":{"line":1,"character":3}}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("\"line\":0"), "{}", yanit);
    assert!(yanit.contains("\"character\":0"), "{}", yanit);
}

#[test]
fn hover_bilinmeyen_konumda_null() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/h3.dil","text":"\"selam\" yaz\n"}}}"#);
    // Tırnak içine hover → kelime yok → null.
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":13,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tmp/h3.dil"},"position":{"line":0,"character":3}}}"#);
    assert!(
        cikti.govdeler[0].contains("\"result\":null"),
        "{}",
        cikti.govdeler[0]
    );
}

#[test]
fn yeniden_adlandirma_ekleri_giydirir() {
    // K-072: sayaç → puan; sayacı → puanı, sayaçla → puanla, sayaçtan → puandan.
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/r.dil","text":"sayaç 3 olsun\nsayacı 1 azalt\nsayaçla yaz\nsayaçtan yaz\n\"sayacı\" yaz\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":21,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/r.dil"},"position":{"line":0,"character":2},"newName":"puan"}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("\"newText\":\"puan\""), "{}", yanit);
    assert!(yanit.contains("\"newText\":\"puanı\""), "{}", yanit);
    assert!(yanit.contains("\"newText\":\"puanla\""), "{}", yanit);
    assert!(yanit.contains("\"newText\":\"puandan\""), "{}", yanit);
    // Metin sabitindeki "sayacı" DOKUNULMAZ: yalnız 4 düzenleme olmalı.
    assert_eq!(yanit.matches("newText").count(), 4, "{}", yanit);
}

#[test]
fn yeniden_adlandirma_yumusama_uretir() {
    // puan → kitap: puanı → kitabı (p→b); renk hedefi: rengi (nk→ng).
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/r2.dil","text":"puan 3 olsun\npuanı yaz\npuana yaz\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":22,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/r2.dil"},"position":{"line":0,"character":1},"newName":"kitap"}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("\"newText\":\"kitabı\""), "{}", yanit);
    assert!(yanit.contains("\"newText\":\"kitaba\""), "{}", yanit);

    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/r3.dil","text":"puan 3 olsun\npuanı yaz\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":23,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/r3.dil"},"position":{"line":0,"character":1},"newName":"renk"}}"#);
    assert!(
        cikti.govdeler[0].contains("\"newText\":\"rengi\""),
        "{}",
        cikti.govdeler[0]
    );
}

#[test]
fn yeniden_adlandirma_unluyle_bitene_tampon() {
    // sayı → elma? elmayı; tamlayan: elmanın.
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/r4.dil","text":"sayı 3 olsun\nsayıyı yaz\nsayının metni yaz\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":24,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/r4.dil"},"position":{"line":0,"character":1},"newName":"elma"}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("\"newText\":\"elmayı\""), "{}", yanit);
    assert!(yanit.contains("\"newText\":\"elmanın\""), "{}", yanit);
}

#[test]
fn yeniden_adlandirma_iki_katmanli_iyelik_zincirini_korur() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/r5.dil","text":"fiyat 3 olsun\nfiyatıyla yaz\nfiyatından yaz\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":25,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/r5.dil"},"position":{"line":0,"character":2},"newName":"elma"}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("\"newText\":\"elmasıyla\""), "{}", yanit);
    assert!(yanit.contains("\"newText\":\"elmasından\""), "{}", yanit);
    assert_eq!(yanit.matches("newText").count(), 3, "{}", yanit);
}

#[test]
fn tanima_git_ayni_yazimli_ayri_kapsami_symbolid_ile_ayirir() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/kapsam.dil","text":"1 kez tekrarla\n    dal 1 olsun\n    dalı yaz\n1 kez tekrarla\n    dal 2 olsun\n    dalı yaz\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":31,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///tmp/kapsam.dil"},"position":{"line":5,"character":5}}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(
        yanit.contains("\"line\":4"),
        "ikinci SymbolId tanımına gitmeli: {yanit}"
    );
    assert!(
        !yanit.contains("\"line\":1"),
        "ilk kapsam seçilmemeli: {yanit}"
    );
}

#[test]
fn yeniden_adlandirma_yalniz_secilen_symbolid_kapsamini_degistirir() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/kapsam-rename.dil","text":"1 kez tekrarla\n    dal 1 olsun\n    dalı yaz\n1 kez tekrarla\n    dal 2 olsun\n    dalı yaz\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":32,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/kapsam-rename.dil"},"position":{"line":1,"character":5},"newName":"kol"}}"#);
    let yanit = &cikti.govdeler[0];
    assert_eq!(yanit.matches("newText").count(), 2, "{yanit}");
    assert!(yanit.contains("\"newText\":\"kol\""), "{yanit}");
    assert!(yanit.contains("\"newText\":\"kolu\""), "{yanit}");
    assert!(
        !yanit.contains("\"line\":4"),
        "ikinci tanım korunmalı: {yanit}"
    );
    assert!(
        !yanit.contains("\"line\":5"),
        "ikinci kullanım korunmalı: {yanit}"
    );
}

#[test]
fn yeniden_adlandirma_ayni_symbolid_yeniden_atamalarini_kapsar() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/atama.dil","text":"puan 1 olsun\npuan 2 olsun\npuanı yaz\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":33,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/atama.dil"},"position":{"line":2,"character":2},"newName":"kitap"}}"#);
    let yanit = &cikti.govdeler[0];
    assert_eq!(yanit.matches("newText").count(), 3, "{yanit}");
    assert_eq!(yanit.matches("\"newText\":\"kitap\"").count(), 2, "{yanit}");
    assert!(yanit.contains("\"newText\":\"kitabı\""), "{yanit}");
}

#[test]
fn semantic_olarak_belirsiz_belgede_rename_tahmin_yapmaz() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/belirsiz.dil","text":"sayaç 1 olsun\nsayac 2 olsun\nsayacı yaz\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":34,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/belirsiz.dil"},"position":{"line":0,"character":2},"newName":"puan"}}"#);
    assert!(
        cikti.govdeler[0].contains("\"result\":null"),
        "belirsiz A002 belgesinde metin tahmini yasak: {}",
        cikti.govdeler[0]
    );
}

#[test]
fn yeniden_adlandirma_islemid_ile_tam_islem_adini_degistirir() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/islem-rename.dil","text":"işlem karesini hesapla\n    sayıyı al\n    sayıyı döndür\n\nsonuç 4 için karesini hesapla olsun\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":35,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/islem-rename.dil"},"position":{"line":4,"character":24},"newName":"iki katını bul"}}"#);
    let yanit = &cikti.govdeler[0];
    assert_eq!(yanit.matches("newText").count(), 2, "{yanit}");
    assert_eq!(
        yanit.matches("\"newText\":\"iki katını bul\"").count(),
        2,
        "{yanit}"
    );
    assert!(
        yanit.contains("\"line\":0"),
        "işlem tanımı düzenlenmeli: {yanit}"
    );
    assert!(yanit.contains("\"line\":4"), "çağrı düzenlenmeli: {yanit}");
}

#[test]
fn islem_adi_argumanla_ayni_yazilsa_da_kesin_cagri_kuyrugu_secilir() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/kesin-cagri.dil","text":"işlem ver\n    değeri al\n    değeri döndür\n\nver 1 olsun\nsonuç ver için ver olsun\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":351,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/kesin-cagri.dil"},"position":{"line":5,"character":16},"newName":"sun"}}"#);
    let yanit = &cikti.govdeler[0];
    assert_eq!(
        yanit.matches("newText").count(),
        2,
        "tanım ve çağrı kuyruğu: {yanit}"
    );
    assert_eq!(yanit.matches("\"newText\":\"sun\"").count(), 2, "{yanit}");
    assert!(
        yanit.contains("\"line\":0"),
        "işlem tanımı düzenlenmeli: {yanit}"
    );
    assert!(
        yanit.contains("\"line\":5"),
        "çağrı kuyruğu düzenlenmeli: {yanit}"
    );
    assert!(
        !yanit.contains("\"line\":4"),
        "aynı adlı değişken korunmalı: {yanit}"
    );
}

#[test]
fn yapi_definition_ve_rename_yapiid_ile_baglanir() {
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/yapi-rename.dil","text":"yapı Kutu\n    değer TamSayı\n\nkutu yeni Kutu olsun\n"}}}"#);

    let tanim = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":36,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///tmp/yapi-rename.dil"},"position":{"line":3,"character":11}}}"#);
    assert!(
        tanim.govdeler[0].contains("\"line\":0"),
        "{}",
        tanim.govdeler[0]
    );

    let rename = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":37,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/yapi-rename.dil"},"position":{"line":3,"character":11},"newName":"Sandık"}}"#);
    let yanit = &rename.govdeler[0];
    assert_eq!(yanit.matches("newText").count(), 2, "{yanit}");
    assert_eq!(
        yanit.matches("\"newText\":\"Sandık\"").count(),
        2,
        "{yanit}"
    );
}

#[test]
fn baska_birimdeki_tanim_ayni_satira_dusse_de_eksik_rename_uretilmez() {
    let mut sunucu = Sunucu::yeni();
    // Gömülü matematik birimindeki tanım 5. satırdadır. Açık belgedeki
    // çağrıyı bilerek aynı satıra koyup kaynak sahipliği çakışmasını sınarız.
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/birim-rename.dil","text":"matematik birimini kullan\n\n\n\nx 3 için mutlak değerini hesapla olsun\n"}}}"#);

    let tanim = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":38,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///tmp/birim-rename.dil"},"position":{"line":4,"character":12}}}"#);
    assert!(
        tanim.govdeler[0].contains("\"result\":null"),
        "{}",
        tanim.govdeler[0]
    );

    let rename = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":39,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/birim-rename.dil"},"position":{"line":4,"character":12},"newName":"büyüklüğünü bul"}}"#);
    assert!(
        rename.govdeler[0].contains("\"result\":null"),
        "{}",
        rename.govdeler[0]
    );
}
