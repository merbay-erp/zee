//! Metin kaçışları (RFC-0002 §6.1) ve negatif sayı sabitleri (v0.2).

use dil::kaynagi_calistir;

#[test]
fn tirnak_kacisi() {
    let kaynak = "mesaj \"O bana \\\"zee\\\" dedi\" olsun\nmesaj yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("kaçış çalışmalı");
    assert_eq!(cikti, vec!["O bana \"zee\" dedi"]);
}

#[test]
fn ters_bolu_ve_yeni_satir() {
    let kaynak = "x \"üst\\nalt ve \\\\ işareti\" olsun\nx yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("kaçışlar çalışmalı");
    assert_eq!(cikti, vec!["üst\nalt ve \\ işareti"]);
}

#[test]
fn bilinmeyen_kacis_reddedilir() {
    let hata = kaynagi_calistir("x \"a\\t\" olsun\n").expect_err("S040 bekleniyor");
    assert_eq!(hata.kod, "S040");
}

#[test]
fn negatif_tam_sayi() {
    let kaynak = "sıcaklık -5 olsun\nsıcaklık 0 dan küçükse\n    \"donuyor\" yaz\nsıcaklığı yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("negatif sayı çalışmalı");
    assert_eq!(cikti, vec!["donuyor", "-5"]);
}

#[test]
fn negatif_ondalik_isaret_dogru() {
    let kaynak = "x -3,14 olsun\nx yaz\ntoplam x ile 3,14 ün toplamı olsun\ntoplam yaz\n";
    let cikti = kaynagi_calistir(kaynak).expect("negatif ondalık çalışmalı");
    assert_eq!(cikti, vec!["-3,14", "0,0"]);
}

#[test]
fn eksi_isareti_tek_basina_hala_hata() {
    let hata = kaynagi_calistir("x 5 - 3 olsun\n").expect_err("çıplak - S001");
    assert_eq!(hata.kod, "S001");
}

#[test]
fn bicimleyici_kacisli_metni_korur() {
    let girdi = "mesaj   \"a \\\"b\\\" c\"   yaz\n";
    let bicimli = dil::bicimleyici::bicimle(girdi).expect("biçimlenmeli");
    assert_eq!(bicimli, "mesaj \"a \\\"b\\\" c\" yaz\n");
    assert_eq!(dil::bicimleyici::bicimle(&bicimli).unwrap(), bicimli);
}
