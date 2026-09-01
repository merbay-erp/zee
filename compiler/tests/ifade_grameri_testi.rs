//! RFC-0021/spec-20 ifade katmanı conformance testleri.
//!
//! Bu dosya tek tek özellikleri değil, katmanların birbirine göre bağlanma
//! sözünü korur. Yeni ifade yüzeyi bu matris güncellenmeden parser'a eklenmez.

use dil::kaynagi_calistir;

#[test]
fn cagri_aritmetikten_daha_siki_baglanir() {
    let kaynak = r#"
işlem biri ekle
    sayıyı al
    sonuç sayı ile 1 in toplamı olsun
    sonucu döndür

değer 2 için biri ekle ile 4 ün çarpımı olsun
değeri yaz
"#;

    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["12"]);
}

#[test]
fn erisim_postfix_aritmetikten_daha_siki_baglanir() {
    let kaynak = r#"
metin "6" olsun
değer metnin sayısı ile 2 nin çarpımı olsun
değeri yaz
"#;

    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["12"]);
}

#[test]
fn karsilastirmalar_boolean_zincirden_once_kurulur() {
    let kaynak = r#"
x 5 olsun
y 10 olsun
x 3 den büyükse ve y 20 den küçükse
    "evet" yaz
"#;

    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["evet"]);
}

#[test]
fn ve_veya_icin_ortuk_oncelik_uydurulmaz() {
    let kaynak = r#"
x 1 olsun
x 1 e eşitse ve x 2 ye eşitse veya x 3 e eşitse
    "ulaşılmaz" yaz
"#;

    let hata = kaynagi_calistir(kaynak).expect_err("karışık bağlaç belirsizdir");
    assert_eq!(hata.kod, "S030");
}

#[test]
fn en_uzun_islem_adi_cagri_katmaninda_kazanir() {
    let kaynak = r#"
işlem değer ver
    sonuç 1 olsun
    sonucu döndür

işlem iki değer ver
    sonuç 2 olsun
    sonucu döndür

seçilen iki değer ver olsun
seçileni yaz
"#;

    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["2"]);
}

#[test]
fn postfix_gorunur_islem_adi_tarafindan_golgelenmez() {
    let kaynak = r#"
işlem sayısı
    sonuç 99 olsun
    sonucu döndür

metin "6" olsun
değer metnin sayısı olsun
değeri yaz
"#;

    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["6"]);
}

#[test]
fn tam_islem_adi_sifir_argumanli_cagriyi_korur() {
    let kaynak = r#"
işlem metnin sayısı
    sonuç 99 olsun
    sonucu döndür

seçilen metnin sayısı olsun
seçileni yaz
"#;

    assert_eq!(kaynagi_calistir(kaynak).expect("çalışmalı"), vec!["99"]);
}

#[test]
fn tanimsiz_katman_bilesimi_sessizce_yorumlanmaz() {
    let kaynak = r#"
işlem sayıları ver
    sonuç 1, 2 listesi olsun
    sonucu döndür

adet sayıları ver adedi olsun
"#;

    let hata = kaynagi_calistir(kaynak).expect_err("çağrı sonrası erişim henüz tanımlı değil");
    assert_eq!(hata.kod, "S015");
}
