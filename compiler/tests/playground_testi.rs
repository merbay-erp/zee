//! Playground çekirdeği (wasm_api) — doğal derlemede aynı determinizm.

use dil::wasm_api::playgroundda_calistir;
use std::sync::{Mutex, MutexGuard};

static ABI_TEST_KILIDI: Mutex<()> = Mutex::new(());

fn abi_test_kilidi() -> MutexGuard<'static, ()> {
    match ABI_TEST_KILIDI.lock() {
        Ok(kilit) => kilit,
        Err(zehirli) => zehirli.into_inner(),
    }
}

fn abi_girdisi(baytlar: &[u8]) -> (*mut u8, usize) {
    if baytlar.is_empty() {
        return (std::ptr::null_mut(), 0);
    }
    let ptr = dil::wasm_api::dil_bellek_ayir(baytlar.len());
    assert!(!ptr.is_null(), "ABI girdi tamponu ayrılmalı");
    // SAFETY: ABI tam `baytlar.len()` büyüklüğünde canlı tampon döndürdü;
    // kaynak ve hedef çakışmıyor.
    unsafe { std::ptr::copy_nonoverlapping(baytlar.as_ptr(), ptr, baytlar.len()) };
    (ptr, baytlar.len())
}

fn abi_sonucunu_oku_ve_birak(ptr: *mut u8) -> String {
    assert!(!ptr.is_null(), "ABI sonuç tamponu ayırabilmeli");
    let toplam = dil::wasm_api::dil_sonuc_tamponu_uzunlugu(ptr);
    assert!(toplam >= 4, "sonuç kaydı başlığı kapsamalı");
    // SAFETY: sorgu pointer'ın canlı sonuç tamponu olduğunu ve exact toplam
    // boyunu doğruladı; tampon bu okuma boyunca bırakılmıyor.
    let baytlar = unsafe { std::slice::from_raw_parts(ptr, toplam) };
    let govde_uzunlugu = u32::from_le_bytes(baytlar[..4].try_into().expect("dört bayt")) as usize;
    assert_eq!(govde_uzunlugu, toplam - 4);
    let sonuc = std::str::from_utf8(&baytlar[4..])
        .expect("ABI her zaman UTF-8 sonuç üretmeli")
        .to_string();
    assert_eq!(
        dil::wasm_api::dil_bellek_birak(ptr, toplam),
        dil::wasm_api::DIL_ABI_BASARILI
    );
    sonuc
}

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
fn scheduler_playgroundda_da_ayni_izi_verir() {
    let kaynak = "\
işlem yavaşı yap
    \"yavaş başladı\" yaz
    2 saniye bekle
    \"yavaş bitti\" yaz
    2 döndür
işlem hızlandır
    \"hızlı başladı\" yaz
    1 saniye bekle
    \"hızlı bitti\" yaz
    1 döndür
eşzamanlı olarak
    yavaş yavaşı yap
    hızlı hızlandır
hepsini bekle
";
    let cikti = playgroundda_calistir(kaynak, "", 1);
    assert_eq!(
        cikti,
        "yavaş başladı\nhızlı başladı\nhızlı bitti\nyavaş bitti"
    );
}

#[test]
fn cabi_katmani_gidis_donus() {
    let _kilit = abi_test_kilidi();
    assert_eq!(
        dil::wasm_api::dil_abi_surumu(),
        dil::wasm_api::DIL_ABI_SURUMU
    );
    let kaynak = "\"merhaba\" yaz\n".as_bytes();
    let (kaynak_ptr, kaynak_uzunluk) = abi_girdisi(kaynak);
    let sonuc = dil::wasm_api::dil_calistir(kaynak_ptr, kaynak_uzunluk, std::ptr::null(), 0, 5);
    assert_eq!(abi_sonucunu_oku_ve_birak(sonuc), "merhaba");
    assert_eq!(
        dil::wasm_api::dil_bellek_birak(kaynak_ptr, kaynak_uzunluk),
        dil::wasm_api::DIL_ABI_BASARILI
    );
}

#[test]
fn cabi_kayit_disi_pointer_uzunluk_ve_tasmayi_reddeder() {
    let _kilit = abi_test_kilidi();
    assert!(dil::wasm_api::dil_bellek_ayir(0).is_null());
    assert!(dil::wasm_api::dil_bellek_ayir(usize::MAX).is_null());

    for (ptr, uzunluk) in [
        (std::ptr::null(), 1),
        (std::ptr::dangling::<u8>(), usize::MAX),
        (usize::MAX as *const u8, 1),
    ] {
        let sonuc = dil::wasm_api::dil_calistir(ptr, uzunluk, std::ptr::null(), 0, 1);
        assert!(abi_sonucunu_oku_ve_birak(sonuc).starts_with("WASM ABI HATASI: kaynak:"));
    }

    let (ptr, uzunluk) = abi_girdisi(b"\"guvenli\" yaz\n");
    let yanlis = dil::wasm_api::dil_calistir(ptr, uzunluk + 1, std::ptr::null(), 0, 1);
    assert!(abi_sonucunu_oku_ve_birak(yanlis).contains("birebir eşleşmiyor"));
    let ic_pointer =
        dil::wasm_api::dil_calistir(ptr.wrapping_add(1), uzunluk - 1, std::ptr::null(), 0, 1);
    assert!(abi_sonucunu_oku_ve_birak(ic_pointer).contains("canlı bir tampon değil"));
    assert_eq!(
        dil::wasm_api::dil_bellek_birak(ptr, uzunluk),
        dil::wasm_api::DIL_ABI_BASARILI
    );
}

#[test]
fn cabi_gecersiz_utf8i_sessizce_bos_metne_cevirmez() {
    let _kilit = abi_test_kilidi();
    let (kaynak_ptr, kaynak_uzunluk) = abi_girdisi(&[0xff, 0xfe]);
    let sonuc = dil::wasm_api::dil_calistir(kaynak_ptr, kaynak_uzunluk, std::ptr::null(), 0, 1);
    assert_eq!(
        abi_sonucunu_oku_ve_birak(sonuc),
        "WASM ABI HATASI: kaynak geçerli UTF-8 değil"
    );
    assert_eq!(
        dil::wasm_api::dil_bellek_birak(kaynak_ptr, kaynak_uzunluk),
        dil::wasm_api::DIL_ABI_BASARILI
    );

    let (girdi_ptr, girdi_uzunluk) = abi_girdisi(&[b'a', 0x80]);
    let sonuc = dil::wasm_api::dil_calistir(std::ptr::null(), 0, girdi_ptr, girdi_uzunluk, 1);
    assert_eq!(
        abi_sonucunu_oku_ve_birak(sonuc),
        "WASM ABI HATASI: girdi geçerli UTF-8 değil"
    );
    assert_eq!(
        dil::wasm_api::dil_bellek_birak(girdi_ptr, girdi_uzunluk),
        dil::wasm_api::DIL_ABI_BASARILI
    );
}

#[test]
fn cabi_yanlis_ve_cift_birakmada_sahipligi_korur() {
    let _kilit = abi_test_kilidi();
    let (ptr, uzunluk) = abi_girdisi(b"abc");
    assert_eq!(dil::wasm_api::dil_sonuc_tamponu_uzunlugu(ptr), 0);
    assert_eq!(dil::wasm_api::dil_bellek_birak(ptr, uzunluk + 1), 0);
    assert_eq!(
        dil::wasm_api::dil_bellek_birak(ptr, uzunluk),
        dil::wasm_api::DIL_ABI_BASARILI
    );
    assert_eq!(dil::wasm_api::dil_bellek_birak(ptr, uzunluk), 0);
    assert_eq!(
        dil::wasm_api::dil_bellek_birak(std::ptr::null_mut(), 0),
        dil::wasm_api::DIL_ABI_BASARILI
    );
    assert_eq!(dil::wasm_api::dil_bellek_birak(std::ptr::null_mut(), 1), 0);

    assert!(dil::wasm_api::dil_bellek_ayir(16 * 1024 * 1024 + 5).is_null());
    let tamponlar: Vec<_> = (0..8)
        .map(|_| {
            let ptr = dil::wasm_api::dil_bellek_ayir(1);
            assert!(!ptr.is_null());
            ptr
        })
        .collect();
    assert!(dil::wasm_api::dil_bellek_ayir(1).is_null());
    for ptr in tamponlar {
        assert_eq!(
            dil::wasm_api::dil_bellek_birak(ptr, 1),
            dil::wasm_api::DIL_ABI_BASARILI
        );
    }
}

#[test]
fn cabi_bozuk_cagridan_sonra_tekrar_kullanilabilir() {
    let _kilit = abi_test_kilidi();
    for sira in 0..64 {
        let sonuc = if sira % 2 == 0 {
            dil::wasm_api::dil_calistir(
                std::ptr::dangling::<u8>(),
                sira + 1,
                std::ptr::null(),
                0,
                7,
            )
        } else {
            let (ptr, uzunluk) = abi_girdisi(b"\"ayakta\" yaz\n");
            let sonuc = dil::wasm_api::dil_calistir(ptr, uzunluk, std::ptr::null(), 0, 7);
            assert_eq!(
                dil::wasm_api::dil_bellek_birak(ptr, uzunluk),
                dil::wasm_api::DIL_ABI_BASARILI
            );
            sonuc
        };
        let metin = abi_sonucunu_oku_ve_birak(sonuc);
        if sira % 2 == 0 {
            assert!(metin.starts_with("WASM ABI HATASI:"));
        } else {
            assert_eq!(metin, "ayakta");
        }
    }
}
