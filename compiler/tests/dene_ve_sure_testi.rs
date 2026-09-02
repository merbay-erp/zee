//! `almayı dene` genelleşmesi (RFC-0008 §4.3) ve Süre türü (RFC-0011).

use dil::{kaynagi_calistir, kaynagi_calistir_girdiyle};

#[test]
fn sayiyi_almayi_dene_cokme_yerine_sonuc() {
    // Sayı tahmini oyununun kırılgan noktası: çocuk "elma" yazarsa program
    // çökmez, konuşur.
    let kaynak = "\
\"Tahminin?\" diye sor
deneme yanıtın sayısını almayı dene olsun
deneme başarılıysa
    \"Sayın: \" ile denemenin değeri yaz
değilse
    \"Bu bir sayı değil: \" ile denemenin hatası yaz
";
    let cikti = kaynagi_calistir_girdiyle(kaynak, vec!["elma".into()]).expect("çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "Tahminin?",
            "Bu bir sayı değil: \"elma\" sayıya çevrilemedi"
        ]
    );

    let cikti = kaynagi_calistir_girdiyle(kaynak, vec!["42".into()]).expect("çalışmalı");
    assert_eq!(cikti, vec!["Tahminin?", "Sayın: 42"]);
}

#[test]
fn ondaligi_almayi_dene() {
    let kaynak = "\
\"Kilo?\" diye sor
deneme yanıtın ondalığını almayı dene olsun
deneme başarılıysa
    fiyat denemenin değeri ile 2 nin çarpımı olsun
    fiyat yaz
değilse
    denemenin hatası yaz
";
    let cikti = kaynagi_calistir_girdiyle(kaynak, vec!["2,5".into()]).expect("çalışmalı");
    assert_eq!(cikti, vec!["Kilo?", "5,0"]);

    let cikti = kaynagi_calistir_girdiyle(kaynak, vec!["çok".into()]).expect("çalışmalı");
    assert_eq!(cikti, vec!["Kilo?", "\"çok\" ondalığa çevrilemedi"]);
}

#[test]
fn sure_sabitleri_ve_basim() {
    let kaynak = "\
a yarım saniye olsun
a yaz
b 2 dakika olsun
b yaz
c 1,5 saniye olsun
c yaz
d 1 saat olsun
d yaz
e 0,0005 saniye olsun
e yaz
f 0,12345678901234567890 saniye olsun
f yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(
        cikti,
        vec![
            "0,5 saniye",
            "2 dakika",
            "1,5 saniye",
            "1 saat",
            "0,001 saniye",
            "0,123 saniye",
        ]
    );

    let hata =
        kaynagi_calistir("x 999999999999999999999999999999999999999999999999,0 saat olsun\n")
            .expect_err("S006 bekleniyor");
    assert_eq!(hata.kod, "S006");
}

#[test]
fn sure_toplama_ve_karsilastirma() {
    let kaynak = "\
a yarım saniye olsun
b 2 dakika olsun
toplam a ile b nin toplamı olsun
toplam yaz
a b den küçükse
    \"kısa\" yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["120,5 saniye", "kısa"]);
}

#[test]
fn sure_carpilamaz() {
    let kaynak = "a 5 saniye olsun\nb a ile a nın çarpımı olsun\n";
    let hata = kaynagi_calistir(kaynak).expect_err("süre çarpımı T008");
    assert_eq!(hata.kod, "T008");
}

#[test]
fn sure_sayiyla_toplanamaz() {
    let kaynak = "a 5 saniye olsun\nb a ile 3 ün toplamı olsun\n";
    let hata = kaynagi_calistir(kaynak).expect_err("süre+sayı T008");
    assert_eq!(hata.kod, "T008");
    assert!(hata.mesaj.contains("Süre"), "{}", hata.mesaj);
}
