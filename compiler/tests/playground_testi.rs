//! Playground çekirdeği (wasm_api) — doğal derlemede aynı determinizm.

use dil::wasm_api::playgroundda_calistir;

#[test]
fn merhaba_calisir() {
    let cikti = playgroundda_calistir("\"Dünyaya merhaba\" yaz\n", "", 42);
    assert_eq!(cikti, "Dünyaya merhaba");
}

#[test]
fn girdiler_satir_satir() {
    let kaynak = "\"Adın ne?\" diye sor\n\"Merhaba \" ile yanıt yaz\n";
    let cikti = playgroundda_calistir(kaynak, "Zeynep", 1);
    assert_eq!(cikti, "Adın ne?\nMerhaba Zeynep");
}

#[test]
fn hata_raporu_turkce() {
    let cikti = playgroundda_calistir("bilinmeyeni yaz\n", "", 1);
    assert!(cikti.contains("HATA A001"), "{}", cikti);
    assert!(cikti.contains("dil hata A001"));
}

#[test]
fn testler_de_kosulur() {
    let kaynak = "test \"bir birdir\"\n    1 1 e eşit olmalı\n\n\"selam\" yaz\n";
    let cikti = playgroundda_calistir(kaynak, "", 1);
    assert!(cikti.contains("selam"));
    assert!(cikti.contains("testler: 1 / 1 geçti"), "{}", cikti);
}

#[test]
fn tohum_deterministik() {
    let kaynak = "x 1 ile 100 arasında rastgele sayı olsun\nx yaz\n";
    let a = playgroundda_calistir(kaynak, "", 7);
    let b = playgroundda_calistir(kaynak, "", 7);
    let c = playgroundda_calistir(kaynak, "", 8);
    assert_eq!(a, b, "aynı tohum aynı sonuç");
    assert_ne!(a, c, "farklı tohum farklı sonuç (olasılıkla)");
}

#[test]
fn son_tarih_sanal_beklemeleri_biriktirir() {
    let kaynak = "\
5 saniye içinde
    3 saniye bekle
    3 saniye bekle
    \"bu çıktı yasak\" yaz
yetişmezse
    \"süresinde iptal\" yaz
";
    let cikti = playgroundda_calistir(kaynak, "", 1);
    assert_eq!(cikti, "süresinde iptal");
}

#[test]
fn cabi_katmani_gidis_donus() {
    // C-ABI sözleşmesi: uzunluk-önekli tampon.
    let kaynak = "\"merhaba\" yaz\n".as_bytes();
    unsafe {
        let ptr =
            dil::wasm_api::dil_calistir(kaynak.as_ptr(), kaynak.len(), std::ptr::null(), 0, 5);
        let len =
            u32::from_le_bytes(std::slice::from_raw_parts(ptr, 4).try_into().unwrap()) as usize;
        let govde = std::str::from_utf8(std::slice::from_raw_parts(ptr.add(4), len)).unwrap();
        assert_eq!(govde, "merhaba");
        dil::wasm_api::dil_bellek_birak(ptr, 4 + len);
    }
}
