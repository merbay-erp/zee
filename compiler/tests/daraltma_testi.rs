//! Akış-duyarlı daraltma (RFC-0008 §4.2, v0.2): varsa/başarılıysa dallarında
//! değeri/hatası statik güvenli; korumasız erişim T036 derleme hatası.

use dil::kaynagi_calistir;

const BULUCU: &str = "\
işlem çift bul
    sayıyı al
    sayı çiftse
        sayıyı döndür
    yok döndür
";

#[test]
fn varsa_dalinda_deger_guvenli() {
    let kaynak = format!(
        "{}\nbulunan 4 için çift bul olsun\nbulunan varsa\n    bulunanın değeri yaz\n",
        BULUCU
    );
    assert_eq!(kaynagi_calistir(&kaynak).expect("çalışmalı"), vec!["4"]);
}

#[test]
fn yoksa_nin_degilse_dali_da_guvenli() {
    let kaynak = format!(
        "{}\nbulunan 4 için çift bul olsun\nbulunan yoksa\n    \"boş\" yaz\ndeğilse\n    bulunanın değeri yaz\n",
        BULUCU
    );
    assert_eq!(kaynagi_calistir(&kaynak).expect("çalışmalı"), vec!["4"]);
}

#[test]
fn korumasiz_erisim_t036() {
    let kaynak = format!(
        "{}\nbulunan 4 için çift bul olsun\nbulunanın değeri yaz\n",
        BULUCU
    );
    let hata = kaynagi_calistir(&kaynak).expect_err("T036");
    assert_eq!(hata.kod, "T036");
}

#[test]
fn dal_disina_cikinca_koruma_biter() {
    let kaynak = format!(
        "{}\nbulunan 4 için çift bul olsun\nbulunan varsa\n    \"var\" yaz\nbulunanın değeri yaz\n",
        BULUCU
    );
    let hata = kaynagi_calistir(&kaynak).expect_err("dal dışı T036");
    assert_eq!(hata.kod, "T036");
}

#[test]
fn sonuc_hatasi_ancak_basarisizsa_dalinda() {
    let kaynak = "sonuç \"yok.txt\" dosyasını okumayı dene olsun\nsonucun hatası yaz\n";
    let hata = kaynagi_calistir(kaynak).expect_err("korumasız hatası T036");
    assert_eq!(hata.kod, "T036");
    assert!(hata.oneri.as_deref().unwrap_or("").contains("başarısızsa"));
}

#[test]
fn basarisizsa_dalinda_hata_guvenli() {
    let kaynak = "\
sonuç \"yok.txt\" dosyasını okumayı dene olsun
sonuç başarısızsa
    sonucun hatası yaz
";
    let cikti = kaynagi_calistir(kaynak).expect("çalışmalı");
    assert_eq!(cikti, vec!["\"yok.txt\" dosyası bulunamadı"]);
}

#[test]
fn cagiranin_daraltmasi_cagrilan_isleme_sizmaz() {
    // K-380: daraltma çağıranın sembolüne aittir; çağrılan işlemin aynı adlı
    // yereli korumasız kalır (önceden çalışma zamanında C008'e düşüyordu).
    let kaynak = format!(
        "{}\nişlem boşu aç\n    sayıyı al\n    bulunan sayı için çift bul olsun\n    bulunanın değerini döndür\n\nbulunan 4 için çift bul olsun\nbulunan varsa\n    x 3 için boşu aç olsun\n    x yaz\n",
        BULUCU
    );
    let hata = kaynagi_calistir(&kaynak).expect_err("T036");
    assert_eq!(hata.kod, "T036");
    assert!(hata.mesaj.contains("\"bulunan\""), "{}", hata.mesaj);
}

#[test]
fn cagri_sonrasi_cagiranin_daraltmasi_korunur() {
    // K-380 olumlu eşi: çağrılan işlemin ayrı akışı geri konur; işlem kendi
    // yerelini kendi `varsa` dalında güvenle açar.
    let kaynak = format!(
        "{}\nişlem güvenle aç\n    sayıyı al\n    bulunan sayı için çift bul olsun\n    bulunan varsa\n        bulunanın değerini döndür\n    0 döndür\n\nbulunan 4 için çift bul olsun\nbulunan varsa\n    x 3 için güvenle aç olsun\n    x yaz\n    bulunanın değeri yaz\n",
        BULUCU
    );
    assert_eq!(
        kaynagi_calistir(&kaynak).expect("çalışmalı"),
        vec!["0", "4"]
    );
}
