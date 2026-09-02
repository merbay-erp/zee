//! B-023/B-049: kaynak, proje ve runtime aynı fail-closed yetkinlik sözleşmesini taşır.

use dil::yetkinlik::{AgHedefi, Yetkinlik, YetkinlikPolitikasi};
use std::collections::BTreeSet;

fn proje_politikasi(hedef: &str) -> YetkinlikPolitikasi {
    YetkinlikPolitikasi::proje(
        BTreeSet::from([Yetkinlik::Ag]),
        BTreeSet::from([AgHedefi::bildirimden(hedef).unwrap()]),
    )
}

#[test]
fn derleme_oncesi_kullanilan_yetkinligi_kesin_kaynakta_reddeder() {
    let kaynak =
        "başlık \"izin\" olsun\ncevap \"https://api.example/veri\" adresinden gelen yanıt olsun\n";
    let program = dil::kaynagi_derle(kaynak).expect("dil geçerli");
    let hata = dil::cozumleyici::yetkinlikleri_denetle(&program, &YetkinlikPolitikasi::kapali())
        .expect_err("ağ bildirimsiz açılmamalı");
    assert_eq!(hata.kod, "T054");
    assert_eq!(hata.satir, 2);
    assert!(hata.sutun > 1);
    assert!(hata.uzunluk > 1);
    assert!(hata.mesaj.contains("`ağ`"), "{}", hata.mesaj);
}

#[test]
fn sabit_outbound_hedefi_proje_allowlistine_baglanir() {
    let program =
        dil::kaynagi_derle("cevap \"https://diger.example/veri\" adresinden gelen yanıt olsun\n")
            .unwrap();
    let hata =
        dil::cozumleyici::yetkinlikleri_denetle(&program, &proje_politikasi("https://api.example"))
            .expect_err("başka origin reddedilmeli");
    assert_eq!(hata.kod, "T054");
    assert!(hata.mesaj.contains("ağ_hedefleri"), "{}", hata.mesaj);

    let izinli =
        dil::kaynagi_derle("cevap \"https://api.example/veri\" adresinden gelen yanıt olsun\n")
            .unwrap();
    dil::cozumleyici::yetkinlikleri_denetle(&izinli, &proje_politikasi("https://api.example"))
        .expect("aynı origin geçmeli");
}

#[test]
fn kullanilmayan_islem_de_yetkinligi_gizleyemez() {
    let kaynak = "işlem gizli oku\n    Metin döndürür\n    satırlar \"özel.txt\" dosyasının satırları olsun\n    satırların ilki döndür\n\n\"ana program\" yaz\n";
    let program = dil::kaynagi_derle(kaynak).expect("derlenmeli");
    let hata = dil::cozumleyici::yetkinlikleri_denetle(&program, &YetkinlikPolitikasi::kapali())
        .expect_err("ölü kod yetki saklamamalı");
    assert_eq!(hata.kod, "T054");
    assert!(hata.mesaj.contains("dosya-okuma"));
}
