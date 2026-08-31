//! dillsp çekirdeği testleri: JSON ayrıştırıcı + mesaj döngüsü.

use dil::lsp::{json_coz, Json, Sunucu};

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
    assert_eq!(json.alan("c").and_then(|c| c.alan("iç")), Some(&Json::Sayi(-2.5)));
}

#[test]
fn json_unicode_kacislari() {
    // Türkçe karakter + vekil çift (emoji).
    let json = json_coz(r#""ğ 😀""#).expect("çözülmeli");
    assert_eq!(json, Json::Metin("ğ 😀".into()));
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
    assert!(cikti.govdeler[0].contains("\"diagnostics\":[]"), "{}", cikti.govdeler[0]);
}

#[test]
fn completion_kalip_kelimeleri() {
    let mut sunucu = Sunucu::yeni();
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":7,"method":"textDocument/completion","params":{}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("\"id\":7"));
    assert!(yanit.contains("\"label\":\"olsun\""));
    assert!(yanit.contains("\"label\":\"değilse\""));
}

#[test]
fn exit_dongusu_durdurur() {
    let mut sunucu = Sunucu::yeni();
    assert!(sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#).devam);
    assert!(!sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"exit"}"#).devam);
}
