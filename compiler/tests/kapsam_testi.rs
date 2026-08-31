//! Blok kapsamı (RFC-0004 kararı, v0.2).

use dil::kaynagi_calistir;

#[test]
fn blokta_dogan_ad_disari_sizmaz() {
    let kaynak = "5 kez tekrarla\n    iç 1 olsun\n\niç yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("iç ad dışarıda A001 olmalı");
    assert_eq!(hata.kod, "A001");
}

#[test]
fn dongu_degiskeni_govdeyle_olur() {
    let kaynak = "1 den 3 e kadar her sayı için\n    sayıyı yaz\n\nsayı yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("döngü değişkeni ölmeli");
    assert_eq!(hata.kod, "A001");
}

#[test]
fn distaki_ada_atama_kalicidir() {
    let kaynak = "\
toplam 0 olsun
1 den 4 e kadar her sayı için
    toplamı sayıyla artır
toplam yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("dış atama kalıcı");
    assert_eq!(cikti, vec!["10"]);
}

#[test]
fn ise_dallari_ayri_kapsam() {
    let kaynak = "\
x 1 olsun
x 1 e eşitse
    gecici \"a\" olsun
    gecici yaz
gecici yaz
";
    let hata = kaynagi_calistir(kaynak).expect_err("dal içi ad dışarı sızmaz");
    assert_eq!(hata.kod, "A001");
}

#[test]
fn golge_yoktur_tur_korunur() {
    // İçeride aynı ada farklı türle "olsun" gölge açmaz; dıştaki ada atamadır → T002.
    let kaynak = "x 1 olsun\n3 kez tekrarla\n    x \"metin\" olsun\n";
    let hata = kaynagi_calistir(kaynak).expect_err("tür değişimi T002");
    assert_eq!(hata.kod, "T002");
}
