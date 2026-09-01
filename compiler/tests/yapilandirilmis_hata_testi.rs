//! K-091: Sonuç'un hata tarafı kod, mesaj, neden ve veri taşıyan Hata'dır.

use dil::{kaynagi_calistir, kaynagi_calistir_girdiyle};

const ZINCIRLI_HATA: &str = r#"
işlem alt işlemi yap
    sayıyı al
    sayı 0 a eşitse
        "ALT_HATA" kodlu "Alt işlem tamamlanamadı" hatasını döndür
    sayıyı döndür

işlem üst işlemi yap
    sayıyı al
    alt sayı için alt işlemi yap olsun
    alt başarısızsa
        neden altın hatası olsun
        bilgi boş sözlük olsun
        bilginin "girdi" değeri "0" olsun
        "UST_HATA" kodlu "Üst işlem tamamlanamadı" hatasını neden nedeniyle bilgi verisiyle döndür
    sayıyı döndür

işlem hata kodunu ver
    hatayı Hata olarak al
    Metin döndürür
    hatanın kodu döndür
"#;

#[test]
fn kod_mesaj_neden_veri_ve_esleme_birlikte_calisir() {
    let kaynak = format!(
        r#"{}
sonuç 0 için üst işlemi yap olsun
sonuç başarısızsa
    hata sonucun hatası olsun
    hatanın kodu yaz
    aktarılan hata için hata kodunu ver olsun
    aktarılan yaz
    hatanın mesajı yaz
    bilgi hatanın verisi olsun
    bilginin "girdi" değeri yaz
    neden hatanın nedeni olsun
    neden varsa
        alt nedenin değeri olsun
        altın kodu yaz
    kod hatanın kodu olsun
    koda göre
        "UST_HATA" ise
            "eşleşti" yaz
        değilse
            "eşleşmedi" yaz
"#,
        ZINCIRLI_HATA
    );

    let cikti = kaynagi_calistir(&kaynak).expect("yapılandırılmış hata çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "UST_HATA",
            "UST_HATA",
            "Üst işlem tamamlanamadı",
            "0",
            "ALT_HATA",
            "eşleşti",
        ]
    );
}

#[test]
fn hata_jsonu_neden_zincirini_ve_veriyi_korur() {
    let kaynak = format!(
        "{}\nsonuç 0 için üst işlemi yap olsun\nsonuç başarısızsa\n    hata sonucun hatası olsun\n    hatanın json metni yaz\n",
        ZINCIRLI_HATA
    );
    let cikti = kaynagi_calistir(&kaynak).expect("çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "{\"kod\":\"UST_HATA\",\"mesaj\":\"Üst işlem tamamlanamadı\",\"neden\":{\"kod\":\"ALT_HATA\",\"mesaj\":\"Alt işlem tamamlanamadı\",\"neden\":null,\"veri\":{}},\"veri\":{\"girdi\":\"0\"}}"
        ]
    );
}

#[test]
fn eski_metin_hatasi_genel_koduyla_geriye_uyumludur() {
    let kaynak = r#"
işlem güvenle böl
    sayıyı al
    sayı 0 a eşitse
        "olmadı" hatasını döndür
    sayıyı döndür

sonuç 0 için güvenle böl olsun
sonuç başarısızsa
    hata sonucun hatası olsun
    hatanın kodu yaz
    hata yaz
"#;
    let cikti = kaynagi_calistir(kaynak).expect("eski kaynak çalışmalı");
    assert_eq!(cikti, vec!["GENEL", "olmadı"]);
}

#[test]
fn dene_yerlesikleri_kararli_kodlar_uretir() {
    let sayi = r#"
"Sayı?" diye sor
sonuç yanıtın sayısını almayı dene olsun
sonuç başarısızsa
    hata sonucun hatası olsun
    hatanın kodu yaz
"#;
    let cikti = kaynagi_calistir_girdiyle(sayi, vec!["elma".into()]).expect("çalışmalı");
    assert_eq!(cikti, vec!["Sayı?", "SAYI_BICIMI"]);

    let ondalik = r#"
sonuç "elma" ondalığını almayı dene olsun
sonuç başarısızsa
    hata sonucun hatası olsun
    hatanın kodu yaz
"#;
    let cikti = kaynagi_calistir(ondalik).expect("çalışmalı");
    assert_eq!(cikti, vec!["ONDALIK_BICIMI"]);

    let dosya = r#"
sonuç "kesinlikle-yok.txt" dosyasını okumayı dene olsun
sonuç başarısızsa
    hata sonucun hatası olsun
    hatanın kodu yaz
"#;
    let cikti = kaynagi_calistir(dosya).expect("çalışmalı");
    assert_eq!(cikti, vec!["DOSYA_OKUMA"]);
}

#[test]
fn hata_yeniden_yayilirken_yapi_korunur() {
    let kaynak = format!(
        r#"{}
işlem yeniden yay
    sayıyı al
    ilk sayı için üst işlemi yap olsun
    ilk başarısızsa
        hata ilkin hatası olsun
        hata hatasını döndür
    sayıyı döndür

sonuç 0 için yeniden yay olsun
sonuç başarısızsa
    hata sonucun hatası olsun
    hatanın kodu yaz
"#,
        ZINCIRLI_HATA
    );
    let cikti = kaynagi_calistir(&kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["UST_HATA"]);
}

#[test]
fn neden_ve_veri_turleri_derlemede_korunur() {
    let yanlis_neden = r#"
işlem dene
    sayıyı al
    sayı 0 a eşitse
        "KOD" kodlu "mesaj" hatasını 5 nedeniyle döndür
    sayıyı döndür

sonuç 0 için dene olsun
"#;
    let hata = kaynagi_calistir(yanlis_neden).expect_err("neden Hata olmalı");
    assert_eq!(hata.kod, "T052");

    let yanlis_veri = r#"
işlem dene
    sayıyı al
    sayı 0 a eşitse
        "KOD" kodlu "mesaj" hatasını 5 verisiyle döndür
    sayıyı döndür

sonuç 0 için dene olsun
"#;
    let hata = kaynagi_calistir(yanlis_veri).expect_err("veri sözlük olmalı");
    assert_eq!(hata.kod, "T052");
}

#[test]
fn hata_kodu_kanonik_olmali() {
    let kaynak = r#"
işlem dene
    sayıyı al
    sayı 0 a eşitse
        "küçük-kod" kodlu "mesaj" hatasını döndür
    sayıyı döndür

sonuç 0 için dene olsun
"#;
    let hata = kaynagi_calistir(kaynak).expect_err("kod sözdizimde doğrulanmalı");
    assert_eq!(hata.kod, "S044");
}
