//! K-083 — progressive disclosure: başlangıçta çıkarım, public API'de açık tür.

use dil::{kaynagi_calistir, kaynagi_derle};

#[test]
fn acik_ondalik_imza_cagri_sirasindan_bagimsizdir() {
    let govde = "\
işlem yarısını bul
    sayıyı Ondalık olarak al
    sonuç sayının 2 ye bölümü olsun
    sonucu döndür
";
    let dar_once = format!(
        "{}\na 5 için yarısını bul olsun\nb 5,0 için yarısını bul olsun\na yaz\nb yaz\n",
        govde
    );
    let genis_once = format!(
        "{}\nb 5,0 için yarısını bul olsun\na 5 için yarısını bul olsun\na yaz\nb yaz\n",
        govde
    );
    assert_eq!(
        kaynagi_calistir(&dar_once).expect("dar-geniş çalışmalı"),
        vec!["2,5", "2,5"]
    );
    assert_eq!(
        kaynagi_calistir(&genis_once).expect("geniş-dar çalışmalı"),
        vec!["2,5", "2,5"]
    );
}

#[test]
fn acik_ondalik_listesi_runtime_degerlerini_de_genisletir() {
    let kaynak = "\
işlem ilk yarıyı bul
    sayıları Ondalık listesi olarak al
    ilk sayıların ilki olsun
    sonuç ilkin 2 ye bölümü olsun
    sonucu döndür

sayılar 5, 9 listesi olsun
sonuç sayılar için ilk yarıyı bul olsun
sonucu yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("liste genişlemeli"),
        vec!["2,5"]
    );
}

#[test]
fn acik_imzali_islem_cagrilmasa_da_govdesi_denetlenir() {
    let kaynak = "\
işlem bozuk hesabı yap
    sayıyı TamSayı olarak al
    sayıyı \"metin\" artır

\"ana program\" yaz
";
    assert_eq!(
        kaynagi_derle(kaynak).expect_err("gizli gövde hatası").kod,
        "T006"
    );
}

#[test]
fn acik_ve_cikarimli_parametre_karistirilamaz() {
    let kaynak = "\
işlem ikisini seç
    birinciyi TamSayı olarak al
    ikinciyi al
    birinciyi döndür
";
    assert_eq!(
        kaynagi_derle(kaynak).expect_err("T037 bekleniyor").kod,
        "T037"
    );
}

#[test]
fn acik_donus_cikarimli_parametreyle_karistirilamaz() {
    let kaynak = "\
işlem değeri geçir
    değeri al
    TamSayı döndürür
    değeri döndür
";
    assert_eq!(
        kaynagi_derle(kaynak).expect_err("T037 bekleniyor").kod,
        "T037"
    );
}

#[test]
fn bilinmeyen_acik_tur_reddedilir() {
    let kaynak = "\
işlem değeri geçir
    değeri Bilinmez olarak al
    değeri döndür
";
    assert_eq!(
        kaynagi_derle(kaynak).expect_err("T038 bekleniyor").kod,
        "T038"
    );
}

#[test]
fn acik_imzaya_uymayan_cagri_t017dir() {
    let kaynak = "\
işlem iki katını bul
    sayıyı TamSayı olarak al
    sonuç sayı ile 2 nin çarpımı olsun
    sonucu döndür

değer \"beş\" için iki katını bul olsun
";
    assert_eq!(
        kaynagi_derle(kaynak).expect_err("T017 bekleniyor").kod,
        "T017"
    );
}

#[test]
fn yapi_turu_acik_parametre_olabilir() {
    let kaynak = "\
yapı Öğrenci
    ad Metin
    not TamSayı

işlem adını ver
    öğrenciyi Öğrenci olarak al
    öğrencinin adı döndür

eliz yeni Öğrenci olsun
elizin adı \"Eliz\" olsun
ad eliz için adını ver olsun
adı yaz
";
    assert_eq!(
        kaynagi_calistir(kaynak).expect("yapı imzası çalışmalı"),
        vec!["Eliz"]
    );
}

#[test]
fn acik_donus_turu_govdeyle_uyusur() {
    let kaynak = "\
işlem iki katını bul
    sayıyı TamSayı olarak al
    TamSayı döndürür
    sonuç sayı ile 2 nin çarpımı olsun
    sonucu döndür

x 6 için iki katını bul olsun
x yaz
";
    assert_eq!(kaynagi_calistir(kaynak).expect("tam sözleşme"), vec!["12"]);
}

#[test]
fn bilinmeyen_donus_turu_t040tir() {
    let kaynak = "\
işlem değeri ver
    Bilinmez döndürür
    1 döndür
";
    assert_eq!(
        kaynagi_derle(kaynak).expect_err("T040 bekleniyor").kod,
        "T040"
    );
}

#[test]
fn bildirilen_ve_gercek_donus_turu_uyusmalidir() {
    let kaynak = "\
işlem değeri ver
    Metin döndürür
    1 döndür
";
    assert_eq!(
        kaynagi_derle(kaynak).expect_err("T041 bekleniyor").kod,
        "T041"
    );
}

#[test]
fn acik_deger_donusu_butun_yollari_kapatir() {
    let kaynak = "\
işlem işareti ver
    sayıyı TamSayı olarak al
    Metin döndürür
    sayı 0 dan büyükse
        \"artı\" döndür
";
    assert_eq!(
        kaynagi_derle(kaynak).expect_err("T042 bekleniyor").kod,
        "T042"
    );
}

#[test]
fn deger_dondurmeyen_sozlesme_deger_donduremez() {
    let kaynak = "\
işlem selam ver
    değer döndürmez
    \"merhaba\" döndür
";
    assert_eq!(
        kaynagi_derle(kaynak).expect_err("T041 bekleniyor").kod,
        "T041"
    );
}
