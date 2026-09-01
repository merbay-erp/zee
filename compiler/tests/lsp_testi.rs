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
    assert!(yanit.contains("\"label\":\"doğrulanıyorsa\""));
    assert!(yanit.contains("\"label\":\"yetkisi\""));
}

#[test]
fn exit_dongusu_durdurur() {
    let mut sunucu = Sunucu::yeni();
    assert!(sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#).devam);
    assert!(!sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"exit"}"#).devam);
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
    assert!(yanit.contains("\"line\":0"), "işlem başlığına gitmeli: {}", yanit);
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
    assert!(cikti.govdeler[0].contains("\"result\":null"), "{}", cikti.govdeler[0]);
}

#[test]
fn yeniden_adlandirma_ekleri_giydirir() {
    // K-072: sayaç → puan; sayacı → puanı, sayaçla → puanla, sayaçtan → puandan.
    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/r.dil","text":"sayaç 3 olsun\nsayacı 1 azalt\nsayaçla topla\nsayaçtan düş\n\"sayacı\" yaz\n"}}}"#);
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
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/r2.dil","text":"puan 3 olsun\npuanı yaz\npuana 1 ekleyemezsin\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":22,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/r2.dil"},"position":{"line":0,"character":1},"newName":"kitap"}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("\"newText\":\"kitabı\""), "{}", yanit);
    assert!(yanit.contains("\"newText\":\"kitaba\""), "{}", yanit);

    let mut sunucu = Sunucu::yeni();
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/r3.dil","text":"puan 3 olsun\npuanı yaz\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":23,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/r3.dil"},"position":{"line":0,"character":1},"newName":"renk"}}"#);
    assert!(cikti.govdeler[0].contains("\"newText\":\"rengi\""), "{}", cikti.govdeler[0]);
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
    sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/r5.dil","text":"fiyat 3 olsun\nfiyatıyla artır\nfiyatından düş\n"}}}"#);
    let cikti = sunucu.mesaj_isle(r#"{"jsonrpc":"2.0","id":25,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/r5.dil"},"position":{"line":0,"character":2},"newName":"elma"}}"#);
    let yanit = &cikti.govdeler[0];
    assert!(yanit.contains("\"newText\":\"elmasıyla\""), "{}", yanit);
    assert!(yanit.contains("\"newText\":\"elmasından\""), "{}", yanit);
    assert_eq!(yanit.matches("newText").count(), 3, "{}", yanit);
}
