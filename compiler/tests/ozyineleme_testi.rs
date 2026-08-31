//! Özyineleme (v0.2, RFC-0006 §5.1): temel-durum-önce kuralı + derinlik korkuluğu.

use dil::kaynagi_calistir;

const FAKTORIYEL: &str = "\
işlem faktöriyelini hesapla
    sayıyı al

    sayı 1 den küçükse
        1 döndür
    bir_eksiği sayı ile 1 in farkı olsun
    alt bir_eksiği için faktöriyelini hesapla olsun
    sonucu sayı ile altın çarpımı olsun
    sonucu döndür
";

#[test]
fn faktoriyel_calisir() {
    let kaynak = format!("{}\nx 5 için faktöriyelini hesapla olsun\nx yaz\n", FAKTORIYEL);
    let cikti = kaynagi_calistir(&kaynak).expect("faktöriyel çalışmalı");
    assert_eq!(cikti, vec!["120"]);
}

#[test]
fn fibonacci_cift_ozyineleme() {
    let kaynak = "\
işlem fibonaççiyi hesapla
    sayıyı al

    sayı 2 den küçükse
        sayıyı döndür
    bir_önce sayı ile 1 in farkı olsun
    iki_önce sayı ile 2 nin farkı olsun
    a bir_önce için fibonaççiyi hesapla olsun
    b iki_önce için fibonaççiyi hesapla olsun
    toplam a ile b nin toplamı olsun
    toplamı döndür

x 10 için fibonaççiyi hesapla olsun
x yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("fibonacci çalışmalı");
    assert_eq!(cikti, vec!["55"]);
}

#[test]
fn karsilikli_ozyineleme() {
    // çift mi / tek mi birbirini çağırır.
    let kaynak = "\
işlem çiftliğine bak
    sayıyı al
    sayı 0 a eşitse
        doğru döndür
    bir_eksiği sayı ile 1 in farkı olsun
    sonuç bir_eksiği için tekliğine bak olsun
    sonucu döndür

işlem tekliğine bak
    sayıyı al
    sayı 0 a eşitse
        yanlış döndür
    bir_eksiği sayı ile 1 in farkı olsun
    sonuç bir_eksiği için çiftliğine bak olsun
    sonucu döndür

x 7 için çiftliğine bak olsun
x doğru olana kadar tekrarla
    \"7 tek\" yaz
    programı bitir
";
    let cikti = kaynagi_calistir(kaynak).expect("karşılıklı özyineleme çalışmalı");
    assert_eq!(cikti, vec!["7 tek"]);
}

#[test]
fn derinlik_korkulugu() {
    // Temel durum yazılı ama argüman değişmediği için asla yakalanmıyor:
    // Rust yığını taşmadan Türkçe C019 tanısı gelmeli.
    let kaynak = "\
işlem düş
    sayıyı al
    sayı 0 a eşitse
        1 döndür
    sonuç sayı için düş olsun
    sonucu döndür

x 5 için düş olsun
";
    let hata = kaynagi_calistir(kaynak).expect_err("sonsuz iniş C019 vermeli");
    assert_eq!(hata.kod, "C019");
    assert!(hata.oneri.as_deref().unwrap_or("").contains("temel durum"));
}

#[test]
fn ozyinelemeli_tur_uyusmazligi() {
    let kaynak = "\
işlem karıştır
    sayıyı al
    sayı 0 a eşitse
        1 döndür
    x sayı için karıştır olsun
    \"metin\" döndür

y 3 için karıştır olsun
";
    let hata = kaynagi_calistir(kaynak).expect_err("dönüş birleşimi T018");
    assert_eq!(hata.kod, "T018");
}
